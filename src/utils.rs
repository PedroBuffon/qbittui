use chrono::Utc;
use chrono_tz::{Tz, UTC};
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;

pub fn format_timestamp_with_timezone(timezone_str: &str) -> String {
    let now = Utc::now();

    // Try to parse the timezone string
    match Tz::from_str(timezone_str) {
        Ok(tz) => {
            let local_time = now.with_timezone(&tz);
            local_time.format("%Y-%m-%d %H:%M:%S %Z").to_string()
        }
        Err(_) => {
            // Fallback to UTC if timezone parsing fails
            let utc_time = now.with_timezone(&UTC);
            utc_time.format("%Y-%m-%d %H:%M:%S UTC").to_string()
        }
    }
}

pub fn is_valid_timezone(timezone_str: &str) -> bool {
    Tz::from_str(timezone_str).is_ok()
}

// Common timezone examples for user reference
pub fn get_common_timezones() -> Vec<&'static str> {
    vec![
        "UTC",
        "US/Eastern",
        "US/Central",
        "US/Mountain",
        "US/Pacific",
        "Europe/London",
        "Europe/Paris",
        "Europe/Berlin",
        "Europe/Rome",
        "Asia/Tokyo",
        "Asia/Shanghai",
        "Asia/Kolkata",
        "Australia/Sydney",
        "America/Sao_Paulo",
        "America/Mexico_City",
        "America/Toronto",
        "America/New_York",
        "America/Los_Angeles",
        "America/Chicago",
        "Africa/Cairo",
        "Pacific/Auckland",
    ]
}

pub fn log_debug(message: &str, timezone: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("qbittui_debug.log")
    {
        let timestamp = format_timestamp_with_timezone(timezone);
        let _ = writeln!(file, "[{timestamp}] {message}");
    }
}
