use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp() -> String {
    let s = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let (days, h, m, sec) = (s / 86400, s / 3600 % 24, s / 60 % 60, s % 60);

    // days since 1970-01-01 -> year/month/day (Hinnant's algorithm)
    let z = days + 719468;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = z / 146097 * 400 + yoe + (month <= 2) as u64;

    format!("{year:04}{month:02}{day:02}-{h:02}{m:02}{sec:02}")
}
