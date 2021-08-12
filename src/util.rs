use std::{intrinsics::copy_nonoverlapping, lazy::SyncLazy};

use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, Utc};
use libc::c_char;
use serde::{Deserialize, Serialize};

static START_DATE: SyncLazy<DateTime<FixedOffset>> =
    SyncLazy::new(|| DateTime::parse_from_rfc2822("Sat, 30 Dec 1899 00:00:00 GMT").unwrap());
static SECONDS_IN_A_DAY: f64 = 24.0 * 60.0 * 60.0;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    pub client_id: String,
    pub redirect_uri: String,
    pub authorization_code: Option<String>,
    pub refresh_token: Option<String>,
    pub refresh_token_expiration: Option<DateTime<Utc>>,
    pub access_token: Option<String>,
    pub access_token_expiration: Option<DateTime<Utc>>,
}

pub fn copy_into(src: &str, dst: *const c_char) -> eyre::Result<()> {
    unsafe {
        copy_nonoverlapping(src.as_ptr() as *const c_char, dst as *mut c_char, src.len());
        let newloc = dst.add(src.len() + 1) as *mut c_char;
        *newloc = 0;
    }
    Ok(())
}

pub(crate) fn timestamp_to_datetime(timestamp_millis: i64) -> DateTime<Utc> {
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp_millis / 1000, 0);
    DateTime::from_utc(naive_datetime, Utc)
}

pub(crate) fn epoch_timestamp_to_t6_date(millis: i64) -> f64 {
    let naive_datetime = NaiveDateTime::from_timestamp(millis / 1000, 0);
    let dt: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
    let localstart: DateTime<Utc> = START_DATE.clone().with_timezone(&Utc);
    let diff = dt - localstart;
    (diff.num_seconds() as f64) / SECONDS_IN_A_DAY
}

// pub(crate) fn t6_date_to_datetime(days: f64) -> DateTime<Utc> {
//     let seconds = days * 24.0 * 60.0 * 60.0;
//     let duration = Duration::seconds(seconds.round() as i64);
//     let start_date: DateTime<Utc> = START_DATE.clone().with_timezone(&Utc) + duration;
//     start_date
// }

pub(crate) fn t6_date_to_epoch_timestamp(days: f64) -> i64 {
    let seconds = days * 24.0 * 60.0 * 60.0;
    let duration = Duration::seconds(seconds.round() as i64);
    let start_date: DateTime<Utc> = START_DATE.clone().with_timezone(&Utc);
    let ts = start_date + duration;
    ts.timestamp_millis()
}
