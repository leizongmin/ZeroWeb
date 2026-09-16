//! Ruby（注音）分段 helper — 收集 `<ruby>` 的 per-segment base/annotation 配对。
//!
//! R1694 从 painter/text.rs 抽离（text.rs 减负，单文件超 2000 行 guideline）。
//! 独立纯函数 + 专属单测；paint_text 两处 paint path 通过 `use super::super::text_ruby::*` 调用。

use zero_dom::{Document, NodeId, NodeKind};

/// R1689：收集 `<ruby>` 的 per-segment annotation —— 按 DOM 序遍历 ruby 直接子，每个 `<rt>`
/// 配对其**前**累积的 base 文本段，返回 `[(base_segment, annotation, block_axis_spacing)]`。
///
/// 匹配 CSS Ruby 语义：`<ruby>漢<rt>かん</rt>字<rt>じ</rt></ruby>` → [("漢","かann"..) ...]，
/// 每个 annotation 居中于对应 base segment（非整 base 扁平化）。whole-word ruby
/// `<ruby>漢字<rt>かんじ</rt></ruby>` → 单 segment [("漢字","かんじ")]。owner 非 ruby 或
/// 无 rt 时返回 None（paint 走普通文本路径）。base/annotation 去空白字符。
///
/// R4409：第三元 = 配对 rt 的**块轴间距**（padding-bottom + border-bottom + margin-bottom，
/// 水平书写；css-ruby-1 #interlinear-block——rt 盒轴 spacing 把注音行推离 base，驱动
/// interlinear-block-margin-box mismatch 哨兵：旧实现忽略之使 test 与无 spacing ref 逐像素
/// 同像）。零间距（常规 ruby）= 0，overlay 落位逐字节不变。styles 缺失（Path B 无表，此
/// 函数仅 layout 趟消费）或字段非 px 时按 resolve_length 解析、缺失臂取 0。
pub(super) fn ruby_annotation_segments(
    doc: &Document,
    owner_id: NodeId,
    styles: Option<&std::collections::HashMap<NodeId, zero_style_system::ComputedStyle>>,
) -> Option<Vec<(String, String, f32)>> {
    let owner = doc.get(owner_id)?;
    if !matches!(&owner.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("ruby")) {
        return None;
    }
    let mut segs: Vec<(String, String, f32)> = Vec::new();
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
                    let spacing = rt_block_axis_spacing(doc, child_id, styles);
                    segs.push((base, annot, spacing));
                } else if name.eq_ignore_ascii_case("rp") {
                    // rp 已 display:none（R1676），无绘制语义，跳过。
                } else if name.eq_ignore_ascii_case("rtc") {
                    // R4405：rtc（span-all 注音容器，css-ruby-1 §ruby-box-model）内的 rt 与
                    // 直接子 rt 同语义——旧实现整树跳过使 complex ruby（<rbc> bases +
                    // <rtc> annotations）的注音信息丢失：Path B flatten 排除 rtc/rt 文本后
                    // 注音无处绘制，页面与「无注音裸文本」逐像素相同（ruby-reflow-001-
                    // opaqueruby mismatch ref 误匹配翻红实证）。下降一层按 rt 直接子同款配对。
                    for &rt_id in doc.child_nodes(child_id).iter() {
                        let Some(rt) = doc.get(rt_id) else {
                            continue;
                        };
                        if let NodeKind::Element(rt_elem) = &rt.kind
                            && rt_elem.local_name().eq_ignore_ascii_case("rt")
                        {
                            let annot: String = doc
                                .text_content(rt_id)
                                .unwrap_or_default()
                                .chars()
                                .filter(|c| !c.is_whitespace())
                                .collect();
                            let base: String = std::mem::take(&mut base_buf)
                                .chars()
                                .filter(|c| !c.is_whitespace())
                                .collect();
                            let spacing = rt_block_axis_spacing(doc, rt_id, styles);
                            segs.push((base, annot, spacing));
                        }
                    }
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

/// R4409：配对 rt 的块轴间距（水平书写：rt 盒底侧朝向 base，padding/border/margin-bottom
/// 把注音行推离 base）。css-ruby-1 #interlinear-block；驱动 interlinear-block-margin-box
/// （mismatch：test 与无 spacing ref 须异像）。缺 styles / 非 rt 元素 / 字段缺失均取 0。
fn rt_block_axis_spacing(
    doc: &Document,
    rt_id: NodeId,
    styles: Option<&std::collections::HashMap<NodeId, zero_style_system::ComputedStyle>>,
) -> f32 {
    let Some(map) = styles else { return 0.0 };
    let Some(style) = map.get(&rt_id) else { return 0.0 };
    let is_rt = doc
        .get(rt_id)
        .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("rt")));
    if !is_rt {
        return 0.0;
    }
    let px = |lv: &zero_css_parser::values::LengthValue| -> f32 {
        match lv {
            zero_css_parser::values::LengthValue::Px(v) => *v as f32,
            other => {
                let fs = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
                zero_style_system::computed::resolve_length(other, fs, None, None) as f32
            }
        }
    };
    let auto0 = |lv: &zero_css_parser::values::LengthValue| -> f32 {
        if matches!(lv, zero_css_parser::values::LengthValue::Auto) {
            0.0
        } else {
            px(lv)
        }
    };
    px(&style.padding_bottom) + px(&style.border_bottom_width) + auto0(&style.margin_bottom)
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
        let segs = ruby_annotation_segments(&doc, ruby, None).expect("ruby has segments");
        assert_eq!(segs.len(), 2, "per-kanji ruby → 2 segments");
        assert_eq!(segs[0], ("漢".to_string(), "かん".to_string(), 0.0));
        assert_eq!(segs[1], ("字".to_string(), "じ".to_string(), 0.0));
    }

    /// whole-word ruby → 单 segment，整 base 配整 annotation。
    #[test]
    fn whole_word_ruby_single_segment() {
        let doc = first_ruby_owner("<body><ruby>漢字<rt>かんじ</rt></ruby></body>");
        let ruby = doc.get_elements_by_tag_name("ruby")[0];
        let segs = ruby_annotation_segments(&doc, ruby, None).expect("ruby has segment");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], ("漢字".to_string(), "かんじ".to_string(), 0.0));
    }

    /// 非 ruby owner → None（paint 走普通文本路径）。
    #[test]
    fn non_ruby_owner_returns_none() {
        let doc = first_ruby_owner("<body><p>text</p></body>");
        let p = doc.get_elements_by_tag_name("p")[0];
        assert!(ruby_annotation_segments(&doc, p, None).is_none());
    }

    /// rp（display:none）不参与分段（括号 fallback 不算 annotation）。
    #[test]
    fn rp_excluded_from_segments() {
        let doc = first_ruby_owner("<body><ruby>漢<rt>kan</rt><rp>(</rp><rt>字</rt><rp>)</rp></ruby></body>");
        let ruby = doc.get_elements_by_tag_name("ruby")[0];
        let segs = ruby_annotation_segments(&doc, ruby, None).expect("segments");
        // 两个 rt → 2 segments；rp 文本不计入。
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].1, "kan");
        assert_eq!(segs[1].1, "字");
    }

    /// R4405：complex ruby（<rbc> bases + <rtc> annotations）——rtc 内 rt 与直接子 rt
    /// 同款配对。旧实现整树跳过 rtc 使注音信息丢失（ruby-reflow-001-opaqueruby：Path B
    /// flatten 排除 rtc 文本后注音无处绘制，页面与 mismatch ref 逐像素相同翻红）。
    /// 注意：base 侧沿用 R1689 累积模型（rbc 整体文本 = 一个 base 段），per-rb unit
    /// 级配对（css-ruby unit pairing）为后续精化，不在本 slice。
    #[test]
    fn rtc_nested_rt_pairs_with_preceding_base() {
        let doc = first_ruby_owner(
            "<body><ruby><rbc><rb>新</rb><rb>幹</rb><rb>線</rb></rbc><rtc><rt>しん</rt><rt>かん</rt><rt>せん</rt></rtc></ruby></body>",
        );
        let ruby = doc.get_elements_by_tag_name("ruby")[0];
        let segs = ruby_annotation_segments(&doc, ruby, None).expect("rtc rt pairs");
        assert_eq!(segs.len(), 3, "rtc 内 3 个 rt → 3 segments");
        assert_eq!(segs[0], ("新幹線".to_string(), "しん".to_string(), 0.0));
        assert_eq!(segs[1], (String::new(), "かん".to_string(), 0.0));
        assert_eq!(segs[2], (String::new(), "せん".to_string(), 0.0));
    }
}
