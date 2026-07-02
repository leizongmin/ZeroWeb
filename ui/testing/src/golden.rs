//! Golden 对比 — 把实际快照与黄金串逐行对比，首个差异返回 [`SnapshotDiff`]（spec FR-016 testing）。
//!
//! 用于 CI 回归：snapshot_scene / snapshot_semantics / snapshot_layout_bounds 的输出与黄金串对比，
//! 不一致时给出首个差异行（line/actual/expected），避免「整串不等」的模糊失败。

use std::fmt;

/// 快照差异（首个不一致行）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotDiff {
    /// 行号（1-based）。
    pub line: usize,
    pub expected: String,
    pub actual: String,
}

impl fmt::Display for SnapshotDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "snapshot diff at line {}: expected {:?}, got {:?}",
            self.line, self.expected, self.actual
        )
    }
}

impl std::error::Error for SnapshotDiff {}

/// 逐行对比 `actual` 与 `expected`；首个差异行返回 [`SnapshotDiff`]，全等返回 `Ok(())`。
///
/// 行数不同也按差异处理（缺失行记为 `<missing>`）。
pub fn compare_snapshots(actual: &str, expected: &str) -> Result<(), SnapshotDiff> {
    let actual_lines: Vec<&str> = actual.lines().collect();
    let expected_lines: Vec<&str> = expected.lines().collect();
    let n = actual_lines.len().max(expected_lines.len());
    for i in 0..n {
        let a = actual_lines.get(i).copied().unwrap_or("<missing>");
        let e = expected_lines.get(i).copied().unwrap_or("<missing>");
        if a != e {
            return Err(SnapshotDiff {
                line: i + 1,
                expected: e.to_string(),
                actual: a.to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_snapshots_match() {
        let s = "fill 0,0,10,10 #ff\nstroke 0,0,5,5\n";
        assert!(compare_snapshots(s, s).is_ok());
    }

    #[test]
    fn first_diff_returned_with_line_number() {
        let actual = "a\nb\nc\n";
        let expected = "a\nX\nc\n";
        let diff = compare_snapshots(actual, expected).unwrap_err();
        assert_eq!(diff.line, 2);
        assert_eq!(diff.expected, "X");
        assert_eq!(diff.actual, "b");
    }

    #[test]
    fn length_mismatch_is_diff() {
        // actual 比 expected 多一行 → 末行差异（expected 视为 <missing>）。
        let actual = "a\nb\nextra\n";
        let expected = "a\nb\n";
        let diff = compare_snapshots(actual, expected).unwrap_err();
        assert_eq!(diff.line, 3);
        assert_eq!(diff.actual, "extra");
        assert_eq!(diff.expected, "<missing>");
    }

    #[test]
    fn empty_strings_match() {
        assert!(compare_snapshots("", "").is_ok());
    }

    #[test]
    fn diff_displays_nicely() {
        let diff = SnapshotDiff {
            line: 5,
            expected: "foo".into(),
            actual: "bar".into(),
        };
        let s = format!("{diff}");
        assert!(s.contains("line 5"));
        assert!(s.contains("\"foo\""));
        assert!(s.contains("\"bar\""));
    }
}
