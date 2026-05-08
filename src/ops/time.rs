//! ISO 8601 / RFC 3339 timestamp formatting in pure Rust. Used for FS
//! mtimes everywhere ops/ talks to disk.

use std::time::SystemTime;

/// RFC 3339 / ISO 8601 in UTC. Howard Hinnant's civil-from-days; ample
/// range for filesystem mtimes; no chrono dep.
pub(super) fn iso8601_utc(unix_secs: i64) -> String {
    let days = unix_secs.div_euclid(86_400);
    let secs_in_day = unix_secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    let hh = (secs_in_day / 3600) as u32;
    let mm = ((secs_in_day / 60) % 60) as u32;
    let ss = (secs_in_day % 60) as u32;
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

/// SystemTime → ISO 8601 string, or empty when the FS doesn't expose mtime.
pub(super) fn iso8601_mtime(t: Option<SystemTime>) -> String {
    let Some(t) = t else { return String::new() };
    let Ok(dur) = t.duration_since(SystemTime::UNIX_EPOCH) else {
        return String::new();
    };
    iso8601_utc(dur.as_secs() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso8601_known_values() {
        assert_eq!(iso8601_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601_utc(1_704_067_200), "2024-01-01T00:00:00Z");
        assert_eq!(iso8601_utc(1_777_766_400), "2026-05-03T00:00:00Z");
        assert_eq!(iso8601_utc(1_777_823_730), "2026-05-03T15:55:30Z");
    }
}
