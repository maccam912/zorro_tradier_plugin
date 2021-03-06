#![allow(non_snake_case)]

use std::{
    cmp::max,
    env,
    ffi::{CStr, CString},
    intrinsics::copy_nonoverlapping,
    lazy::SyncLazy,
    sync::Mutex,
    thread::sleep,
    time::Duration,
};

use libc::{abs, c_char, c_double, c_int, c_void};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config,
};

use crate::{
    util::{self, t6_date_to_epoch_timestamp, timestamp_to_datetime},
    Date, Var, T6,
};
use tradier::market_data::get_time_and_sales::{get_time_and_sales, Data};
use tradier::{Class, OrderType, Side, TradierConfig};

#[no_mangle]
#[allow(unused_variables)]
extern "system" fn DllMain(dll_module: c_void, call_reason: c_void, reserved: c_void) -> bool {
    true
}

type FpType = extern "C" fn(*const c_char) -> c_int;

#[derive(Debug, Clone)]
pub(crate) struct State {
    pub(crate) handle: Option<extern "C" fn(*const c_char) -> c_int>,
    pub(crate) subscriptions: Vec<String>,
    pub(crate) config: Option<TradierConfig>,
}
unsafe impl Send for State {}
unsafe impl Sync for State {}

pub(crate) static STATE: SyncLazy<Mutex<State>> = SyncLazy::new(|| {
    Mutex::new(State {
        handle: None,
        subscriptions: vec![],
        config: None,
    })
});

pub(crate) fn log(msg: &str) {
    if let Ok(myref) = STATE.try_lock() {
        if let Some(func) = myref.handle {
            let cstr = CString::new(format!("{}\n", msg)).unwrap();
            func(cstr.as_ptr());
        }
    }
}

#[no_mangle]
pub extern "C" fn BrokerCommand(command: c_int, data: c_int) -> Var {
    if command == 43 {
        300.0
    } else {
        0.0 // Return 0 for all unimplemented commands
    }
}

#[no_mangle]
pub extern "C" fn BrokerOpen(Name: *const c_char, fpError: FpType, fpProgress: FpType) -> c_int {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("log/output.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(Logger::builder().build("app::backend::db", LevelFilter::Debug))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();

    let _ = util::copy_into("TR", Name);
    if let Ok(mut state) = STATE.lock() {
        state.handle = Some(fpError);
    }

    2 // Return 2 if successful open
}

#[no_mangle]
pub extern "C" fn BrokerLogin(
    User: *const c_char,
    Pwd: *const c_char,
    Type: *const c_char,
    Accounts: *const c_char,
) -> c_int {
    let type_string: String = unsafe { CStr::from_ptr(Type).to_str().unwrap().into() };

    let endpoint: String = if type_string == "Real".to_string() {
        "https://api.tradier.com".into()
    } else {
        "https://sandbox.tradier.com".into()
    };

    let token = unsafe { CStr::from_ptr(Pwd) };
    let token_str: String = token.to_str().unwrap().into();
    let config = TradierConfig {
        token: token_str,
        endpoint,
    };
    STATE.lock().unwrap().config = Some(config.clone());

    let profile = tradier::account::get_user_profile::get_user_profile(&config);
    if profile.is_ok() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn BrokerAsset(
    Asset: *const c_char,
    pPrice: *mut c_double,
    pSpread: *mut c_double,
    pVolume: *mut c_double,
    pPip: *mut c_double,
    pPipCost: *mut c_double,
    pLotAmount: *mut c_double,
    pMarginCost: *mut c_double,
    pRollLong: *mut c_double,
    pRollShort: *mut c_double,
) -> c_int {
    unsafe {
        let asset_str = CStr::from_ptr(Asset).to_str();
        if asset_str.is_err() {
            log("Could not convert asset to a string");
            return 0;
        }

        if !STATE
            .lock()
            .unwrap()
            .subscriptions
            .contains(&asset_str.unwrap().to_owned())
        {
            // we haven't subscribed yet. Subscribe and return.
            STATE
                .lock()
                .unwrap()
                .subscriptions
                .push(asset_str.unwrap().to_owned());
            *pPrice = 0 as f64;
        }

        1
    }
}

#[no_mangle]
pub extern "C" fn BrokerHistory2(
    Asset: *const c_char,
    tStart: Date,
    tEnd: Date,
    nTickMinutes: c_int,
    nTicks: c_int,
    ticks: *mut T6,
) -> c_int {
    assert_eq!(nTickMinutes, 1);
    let start = timestamp_to_datetime(t6_date_to_epoch_timestamp(tStart as f64));
    let end = timestamp_to_datetime(t6_date_to_epoch_timestamp(tEnd));
    let asset_str = unsafe { CStr::from_ptr(Asset).to_str().unwrap() };
    let month_back = chrono::offset::Utc::now().checked_sub_signed(chrono::Duration::days(27));
    let correct_start = max(start, month_back.unwrap());
    let config = STATE.lock().unwrap().config.clone().unwrap();

    let history = get_time_and_sales(
        &config,
        asset_str.into(),
        Some("1min".into()),
        Some(correct_start),
        Some(end),
        None,
    );

    match history {
        Ok(historyseries) => {
            let candles: Vec<Data> = historyseries.series.data;
            let mut t6_candles: Vec<T6> = candles
                .iter()
                .map(|quote| {
                    let c: T6 = quote.into();
                    c
                })
                .collect();
            t6_candles.reverse();
            t6_candles.truncate(nTicks as usize);
            let t6_candles_ptr: *const T6 = t6_candles.as_ptr();
            unsafe { copy_nonoverlapping(t6_candles_ptr, ticks, t6_candles.len()) };
            t6_candles.len() as c_int
        }
        Err(err) => {
            // log(&(err.to_string()));
            0
        }
    }
    // let t6_candles_mut_ptr: *mut T6 = t6_candles.clone().as_mut_ptr();
    // t6_candles.len() as i32
}

#[no_mangle]
pub extern "C" fn BrokerBuy2(
    Asset: *const c_char,
    Amount: c_int,
    StopDist: c_double,
    Limit: c_double,
    pPrice: *const c_double,
    pFill: *const c_int,
) -> c_int {
    let config = STATE.lock().unwrap().config.clone().unwrap();

    let acct = &tradier::account::get_user_profile::get_user_profile(&config)
        .unwrap()
        .profile
        .account[0]
        .account_number;

    let side = if Amount < 0 { Side::sell } else { Side::buy };

    let quantity: u64 = Amount.abs() as u64;
    unsafe {
        let symbol: String = CStr::from_ptr(Asset)
            .to_str()
            .expect("Couldn't get Asset as string")
            .into();

        log::warn!("{:?}", symbol);
        log::warn!("{:?}", side);
        log::warn!("{:?}", quantity);

        let resp_option = tradier::trading::orders::post_order(
            &config,
            acct.into(),
            Class::equity,
            symbol,
            side,
            quantity,
            OrderType::market,
            tradier::Duration::gtc,
            None,
            None,
            None,
        );
        log::warn!("Response: {:?}", resp_option);
        let resp = resp_option.unwrap();
        resp.order.id as c_int
    }
}
