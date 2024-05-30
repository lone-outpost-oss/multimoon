//! Common utilities.

// use crate::prelude::*;

pub fn timestamp_from_zipfile(file: zip::read::ZipFile, fallback: i64) -> i64 {
    // TODO: remove `chrome` crate, use `time` crate for local datetime instead
    use chrono::TimeZone;
    if let Some(z) = file.last_modified() {
        let (y, mo, d, h, mn, s) = (z.year().into(), z.month().into(), 
            z.day().into(), z.hour().into(), z.minute().into(), z.second().into());
        chrono::offset::Local
            .with_ymd_and_hms(y, mo, d, h, mn, s)
            .earliest()
            .map(|t| t.timestamp())
            .unwrap_or(fallback)
    } else {
        fallback
    }
}
