//! ZeroWeb 产品版本信息（构建日期版本 + 构建 commit）。

/// 当前构建的产品版本，格式为 `YY.M.D`。
pub const VERSION: &str = env!("ZERO_BUILD_VERSION");

/// 构建时所在 git commit 短 hash；非 git 环境构建为 `unknown`。
pub const BUILD_COMMIT: &str = env!("ZERO_BUILD_COMMIT");

/// 单行版本描述：`ZeroBrowser <版本> (commit <hash>)`。
pub fn version_string() -> String {
    format!("ZeroBrowser {VERSION} (commit {BUILD_COMMIT})")
}

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

    #[test]
    fn version_string_contains_version_and_commit() {
        let text = super::version_string();
        assert_eq!(
            text,
            format!("ZeroBrowser {} (commit {})", super::VERSION, super::BUILD_COMMIT)
        );
    }

    #[test]
    fn embedded_commit_is_short_hash_or_unknown() {
        assert!(
            super::BUILD_COMMIT == "unknown"
                || (super::BUILD_COMMIT.len() >= 7
                    && super::BUILD_COMMIT
                        .chars()
                        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()))
        );
    }
}
