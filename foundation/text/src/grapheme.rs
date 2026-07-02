//! Grapheme cluster 边界（spec §8.4.1 `grapheme.rs`）。
//!
//! 用户感知的「字符」边界（光标移动/删除/选区的最小单位）。
//! M1 提供最小实现：以 Unicode char 边界为 cluster 边界（正确处理 ASCII 与 BMP 单码点）；
//! 完整的 UAX #29 扩展 cluster（CR+LF、Extend/SpacingMark/ZWJ 序列）留 M2 接 unicode-segmentation。

/// 返回字符串中所有 grapheme cluster 起始字节偏移（含末尾 `len`）。
///
/// M1 = char 边界。例：`"abc"` → `[0, 1, 2, 3]`。
pub fn grapheme_cluster_boundaries(s: &str) -> Vec<usize> {
    let mut out = Vec::with_capacity(s.len() + 1);
    for (i, _) in s.char_indices() {
        out.push(i);
    }
    out.push(s.len());
    out
}

/// `byte_idx` 是否为 grapheme cluster 边界。
pub fn is_grapheme_boundary(s: &str, byte_idx: usize) -> bool {
    s.is_char_boundary(byte_idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_boundaries() {
        assert_eq!(grapheme_cluster_boundaries("abc"), vec![0, 1, 2, 3]);
        assert!(is_grapheme_boundary("abc", 0));
        assert!(is_grapheme_boundary("abc", 1));
        assert!(is_grapheme_boundary("abc", 3));
        // 字符中间字节不是边界（"a" 是 1 字节，所以这里 1 仍是边界；用多字节示例验证非边界）。
        assert!(!is_grapheme_boundary("aé", 2)); // "é" 的中间字节
    }

    #[test]
    fn empty_string() {
        assert_eq!(grapheme_cluster_boundaries(""), vec![0]);
    }

    #[test]
    fn bmp_multibyte_boundary_on_char_edge() {
        // "é" 单码点 U+00E9 → 一个 cluster，边界在 0 与 2。
        let s = "aé";
        assert_eq!(grapheme_cluster_boundaries(s), vec![0, 1, 3]);
    }
}
