//! Ruby（注音）分段 helper — 收集 `<ruby>` 的 per-segment base/annotation 配对。
//!
//! R1694 从 painter/text.rs 抽离（text.rs 减负，单文件超 2000 行 guideline）。
//! 独立纯函数 + 专属单测；paint_text 两处 paint path 通过 `use super::super::text_ruby::*` 调用。

use zero_dom::{Document, NodeId, NodeKind};

/// R1689：收集 `<ruby>` 的 per-segment annotation —— 按 DOM 序遍历 ruby 直接子，每个 `<rt>`
/// 配对其**前**累积的 base 文本段，返回 `[(base_segment, annotation)]`。
///
/// 匹配 CSS Ruby 语义：`<ruby>漢<rt>かん</rt>字<rt>じ</rt></ruby>` → [("漢","かん"),("字","じ")]，
/// 每个 annotation 居中于对应 base segment（非整 base 扁平化）。whole-word ruby
/// `<ruby>漢字<rt>かんじ</rt></ruby>` → 单 segment [("漢字","かんじ")]。owner 非 ruby 或
/// 无 rt 时返回 None（paint 走普通文本路径）。base/annotation 去空白字符。
pub(super) fn ruby_annotation_segments(doc: &Document, owner_id: NodeId) -> Option<Vec<(String, String)>> {
    let owner = doc.get(owner_id)?;
    if !matches!(&owner.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("ruby")) {
        return None;
    }
    let mut segs: Vec<(String, String)> = Vec::new();
    let mut base_buf = String::new();
    for child_id in doc.child_nodes(owner_id) {
        let Some(node) = doc.get(child_id) else {
            continue;
        };
        match &node.kind {
            NodeKind::Element(elem) => {
                let name = elem.local_name();
                if name.eq_ignore_ascii_case("rt") {
                    // rt 配对其前累积的 base 段（CSS Ruby：rt 标注其前 base）。
                    let annot: String = doc
                        .text_content(child_id)
                        .unwrap_or_default()
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect();
                    let base: String = std::mem::take(&mut base_buf)
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect();
                    segs.push((base, annot));
                } else if name.eq_ignore_ascii_case("rp") || name.eq_ignore_ascii_case("rtc") {
                    // rp 已 display:none（R1676）；rtc 多 annotation 超出 simple ruby scope，跳过。
                } else {
                    // 嵌套元素（含嵌套 ruby）的文本累积进当前 base 段。
                    if let Some(t) = doc.text_content(child_id) {
                        base_buf.push_str(&t);
                    }
                }
            }
            NodeKind::Text(t) => base_buf.push_str(&t.content),
            _ => {}
        }
    }
    // 尾部 base（无后续 rt）无 annotation，丢弃（chromium 亦不标注）。
    if segs.is_empty() { None } else { Some(segs) }
}

#[cfg(test)]
mod r1689_ruby_segment_tests {
    use super::ruby_annotation_segments;

    fn first_ruby_owner(html: &str) -> zero_dom::Document {
        // 返回 doc（caller 用 get_elements_by_tag_name 取 ruby）；为简化直接重建。
        zero_dom::parse_html(html)
    }

    /// R1689：per-kanji ruby → 每个 rt 配对其前 base 段。
    #[test]
    fn per_kanji_ruby_segments_pair_rt_with_preceding_base() {
        let doc = first_ruby_owner("<body><ruby>漢<rt>かん</rt>字<rt>じ</rt></ruby></body>");
        let ruby = doc.get_elements_by_tag_name("ruby")[0];
        let segs = ruby_annotation_segments(&doc, ruby).expect("ruby has segments");
        assert_eq!(segs.len(), 2, "per-kanji ruby → 2 segments");
        assert_eq!(segs[0], ("漢".to_string(), "かん".to_string()));
        assert_eq!(segs[1], ("字".to_string(), "じ".to_string()));
    }

    /// whole-word ruby → 单 segment，整 base 配整 annotation。
    #[test]
    fn whole_word_ruby_single_segment() {
        let doc = first_ruby_owner("<body><ruby>漢字<rt>かんじ</rt></ruby></body>");
        let ruby = doc.get_elements_by_tag_name("ruby")[0];
        let segs = ruby_annotation_segments(&doc, ruby).expect("ruby has segment");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], ("漢字".to_string(), "かんじ".to_string()));
    }

    /// 非 ruby owner → None（paint 走普通文本路径）。
    #[test]
    fn non_ruby_owner_returns_none() {
        let doc = first_ruby_owner("<body><p>text</p></body>");
        let p = doc.get_elements_by_tag_name("p")[0];
        assert!(ruby_annotation_segments(&doc, p).is_none());
    }

    /// rp（display:none）不参与分段（括号 fallback 不算 annotation）。
    #[test]
    fn rp_excluded_from_segments() {
        let doc = first_ruby_owner("<body><ruby>漢<rt>kan</rt><rp>(</rp><rt>字</rt><rp>)</rp></ruby></body>");
        let ruby = doc.get_elements_by_tag_name("ruby")[0];
        let segs = ruby_annotation_segments(&doc, ruby).expect("segments");
        // 两个 rt → 2 segments；rp 文本不计入。
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].1, "kan");
        assert_eq!(segs[1].1, "字");
    }
}
