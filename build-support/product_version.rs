#![allow(dead_code)]

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// 构建日期版本及 Windows 资源使用的四段数值版本。
pub struct ProductVersion {
    pub text: String,
    pub windows_value: u64,
}

/// 解析显式版本、可复现时间戳或当前本地日期，生成产品版本。
pub fn resolve() -> Result<ProductVersion, String> {
    if let Ok(value) = std::env::var("ZERO_BUILD_VERSION") {
        return parse(&value);
    }

    match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(value) => {
            let seconds = value
                .parse::<u64>()
                .map_err(|_| "SOURCE_DATE_EPOCH must be a non-negative Unix timestamp".to_string())?;
            from_unix_seconds(seconds)
        }
        Err(_) => match local_date_version() {
            Ok(value) => parse(&value),
            Err(_) => {
                let seconds = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| "system time is before the Unix epoch".to_string())?
                    .as_secs();
                from_unix_seconds(seconds)
            }
        },
    }
}

/// 导出产品版本，并在显式版本或可复现时间戳变化时让 Cargo 重新构建。
pub fn emit_cargo_env() {
    let version = resolve().unwrap_or_else(|error| panic!("invalid product version: {error}"));
    println!("cargo:rustc-env=ZERO_BUILD_VERSION={}", version.text);
    println!("cargo:rerun-if-env-changed=ZERO_BUILD_VERSION");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
}

/// 从 Unix 时间戳生成 UTC 日期版本。
pub fn from_unix_seconds(seconds: u64) -> Result<ProductVersion, String> {
    let days = i64::try_from(seconds / 86_400).map_err(|_| "Unix timestamp is too large".to_string())?;
    let (year, month, day) = civil_date_from_days(days);
    if !(2000..=2099).contains(&year) {
        return Err(format!(
            "product version date must be between 2000 and 2099, got {year:04}-{month:02}-{day:02}"
        ));
    }
    Ok(from_parts((year - 2000) as u16, month as u16, day as u16))
}

fn parse(value: &str) -> Result<ProductVersion, String> {
    let parts = value
        .split('.')
        .map(|part| {
            part.parse::<u16>()
                .map_err(|_| format!("product version must use YY.M.D, got {value:?}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if parts.len() != 3
        || parts[0] > 99
        || !(1..=12).contains(&parts[1])
        || parts[2] == 0
        || parts[2] > days_in_month(parts[0], parts[1])
    {
        return Err(format!("product version must use YY.M.D, got {value:?}"));
    }
    Ok(from_parts(parts[0], parts[1], parts[2]))
}

fn local_date_version() -> Result<String, String> {
    #[cfg(windows)]
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", "(Get-Date).ToString('yy.MM.dd')"])
        .output();
    #[cfg(not(windows))]
    let output = Command::new("date").arg("+%y.%m.%d").output();

    let output = output.map_err(|error| format!("failed to read local date: {error}"))?;
    if !output.status.success() {
        return Err("local date command failed".to_string());
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_string())
        .map_err(|error| format!("local date is not UTF-8: {error}"))
}

#[allow(clippy::manual_is_multiple_of)] // MSRV 1.85 尚不提供 is_multiple_of()。
fn days_in_month(year: u16, month: u16) -> u16 {
    match month {
        2 if year % 4 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn from_parts(year: u16, month: u16, day: u16) -> ProductVersion {
    let text = format!("{year}.{month}.{day}");
    let windows_value = (u64::from(year) << 48) | (u64::from(month) << 32) | (u64::from(day) << 16);
    ProductVersion { text, windows_value }
}

// Howard Hinnant's civil-from-days algorithm, with day zero at 1970-01-01.
fn civil_date_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let days = days_since_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_normalizes_short_date_version() {
        let version = parse("26.08.09").unwrap();
        assert_eq!(version.text, "26.8.9");
    }

    #[test]
    fn rejects_invalid_calendar_date() {
        assert!(parse("26.2.29").is_err());
        assert!(parse("26.13.1").is_err());
    }
}
