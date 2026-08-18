//! ZeroWeb 产品构建日期版本。

/// 当前构建的产品版本，格式为 `YY.M.D`。
pub const VERSION: &str = env!("ZERO_BUILD_VERSION");

#[cfg(test)]
#[path = "../../../build-support/product_version.rs"]
mod product_version_test_support;

#[cfg(test)]
mod tests {
    #[test]
    fn formats_utc_date_as_short_product_version() {
        let version = super::product_version_test_support::from_unix_seconds(1_754_697_600).unwrap();
        assert_eq!(version.text, "25.8.9");
        assert_eq!(version.windows_value, 0x0019_0008_0009_0000);
    }

    #[test]
    fn embedded_version_is_a_valid_short_date() {
        let parts: Vec<_> = super::VERSION.split('.').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|part| part.parse::<u8>().is_ok()));
        assert!((1..=12).contains(&parts[1].parse::<u8>().unwrap()));
        assert!((1..=31).contains(&parts[2].parse::<u8>().unwrap()));
    }
}
