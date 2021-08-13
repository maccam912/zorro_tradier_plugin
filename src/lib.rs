#![feature(once_cell)]

use libc::{c_double, c_float};
use tradier::market_data::get_time_and_sales::Data;
use util::epoch_timestamp_to_t6_date;

type Date = c_double;
type Var = c_double;

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
#[repr(align(1))]
#[repr(C)]
pub struct T6 {
    pub time: Date,
    pub fHigh: c_float,
    pub fLow: c_float,
    pub fOpen: c_float,
    pub fClose: c_float,
    pub fVal: c_float,
    pub fVol: c_float,
}

impl From<&Data> for T6 {
    fn from(item: &Data) -> Self {
        T6 {
            time: epoch_timestamp_to_t6_date(item.timestamp * 1000),
            fHigh: item.high as c_float,
            fLow: item.low as c_float,
            fOpen: item.open as c_float,
            fClose: item.close as c_float,
            fVal: 0.0,
            fVol: item.volume as c_float,
        }
    }
}

pub mod dll;
pub mod util;
