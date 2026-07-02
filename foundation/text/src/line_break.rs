//! 换行机会（spec §8.4.1 `line_break.rs`）。
//!
//! 在自动换行（`max_width`）时确定可断行位置。M1 提供最小实现：ASCII 空格/制表符后与
//! 连字符后可断；完整 UAX #14 留 M2。

/// 返回文本中可换行的「断点字节偏移」（断点位置 = 可在该偏移前换行）。
pub fn line_break_opportunities(s: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut prev = b' ';
    for (i, &b) in bytes.iter().enumerate() {
        // 空白后可断（在空白字符之后的位置）。
        if (prev == b' ' || prev == b'\t') && i > 0 {
            out.push(i);
        }
        // 连字符 '‐'/'-' 之后可断。
        if b == b'-' {
            out.push(i + 1);
        }
        prev = b;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaks_after_spaces() {
        let s = "foo bar baz";
        let ops = line_break_opportunities(s);
        // 在 "foo "（偏移 4）、"bar "（偏移 8）之后可断。
        assert!(ops.contains(&4));
        assert!(ops.contains(&8));
    }

    #[test]
    fn breaks_after_hyphen() {
        let s = "co-op";
        let ops = line_break_opportunities(s);
        // hyphen 在字节 2，断点在 3（hyphen 之后）。
        assert!(ops.contains(&3), "hyphen yields a break opportunity right after it");
    }
}
