//! R5165（css-text-3 §5.5 line-break-details + UAX #14 GL/WJ/ZWJ）：原子 inline 与
//! 相邻 GL/WJ/ZWJ 类字符（U+00A0 web-compat 例外除外）间无软换行机会。
//!
//! GL 身份旁带链路三段各有断言：collect 期原始 DOM 边界字符判定
//!（`InlineBlockBox::prev_char_gl`/`next_char_gl`）→ 断行层前邻 GL 抑制原子 fit 折行
//!（arm 1）→ 后邻 GL 抑制首词折行（arm 2）。断行断言用行数关系值（estimate 启发式
//! 即可，无字体依赖）；`ZW_ATOMIC_GL_NO_BREAK=0` 的 OFF 反证由语料 A/B 承担
//!（OnceLock 缓存不可进程内翻转）。

use super::super::*;
use zero_css_parser::values::{DisplayValue, LengthValue, VerticalAlignValue as VA};
use zero_style_system::ComputedStyle;

/// DOM：`<div>{children}</div>`，children 每项 = (内容, display, 元素内文本)：
/// 内容 `t:<text>` 为 div 直挂文本节点（display/内文本忽略）；其余为元素名，
/// 注入 ComputedStyle（display + 显式 Px 20×10——inline-block 需过 `w>0 && h>0`
/// 收集门，否则降级零宽 TextRun 出不了 InlineBlockBox）。返回 collect 出的
/// InlineItem 流。
fn collect_div(children: &[(&str, Option<DisplayValue>, &str)]) -> Vec<InlineItem> {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    doc.append_child(root, div).unwrap();
    let mut styles = std::collections::HashMap::new();
    for (content, display, inner_text) in children {
        if let Some(text) = content.strip_prefix("t:") {
            let node = doc.create_text_node(text);
            doc.append_child(div, node).unwrap();
            continue;
        }
        let elem = doc.create_element(content);
        doc.append_child(div, elem).unwrap();
        if !inner_text.is_empty() {
            let node = doc.create_text_node(inner_text);
            doc.append_child(elem, node).unwrap();
        }
        let mut style = ComputedStyle::default();
        style.display = display.clone().unwrap_or(DisplayValue::Inline);
        style.width = LengthValue::Px(20.0);
        style.height = LengthValue::Px(10.0);
        styles.insert(elem, style);
    }
    let ctx = InlineFormattingContext::new(50.0);
    ctx.collect_inline_items(&doc, div, &styles)
}

/// 从 collect 流取唯一原子盒的 (prev_char_gl, next_char_gl)。
fn atomic_flags(items: &[InlineItem]) -> (bool, bool) {
    let mut flags = None;
    for item in items {
        if let InlineItem::InlineBlock(b) = item {
            assert!(flags.is_none(), "测试假设单原子盒");
            flags = Some((b.prev_char_gl, b.next_char_gl));
        }
    }
    flags.unwrap_or_else(|| panic!("无原子盒: {items:?}"))
}

fn ib_box(width: f32, prev_gl: bool, next_gl: bool) -> InlineItem {
    InlineItem::InlineBlock(InlineBlockBox {
        width,
        height: 10.0,
        node_id: NodeId::default(),
        vertical_align: VA::Baseline,
        baseline: 10.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        prev_char_gl: prev_gl,
        next_char_gl: next_gl,
    })
}

fn text_run(text: &str) -> InlineItem {
    InlineItem::Text(TextRun::simple(
        text.to_string(),
        NodeId::default(),
        16.0,
        20.0,
        VA::Baseline,
    ))
}

/// collect 旁带：NNBSP（GL）贴原子 → prev_gl=true；FIGURE SPACE / WJ 贴原子 →
/// next_gl=true（css-text-3 §5.5 原子-GL 禁断，U+00A0 例外见下一测试）。
#[test]
fn r5165_collect_flags_gl_boundary_chars() {
    // <div>A&#x202F;<span style="display:inline-block"/></div>
    let items = collect_div(&[("t:A\u{202F}", None, ""), ("span", Some(DisplayValue::InlineBlock), "")]);
    assert_eq!(atomic_flags(&items), (true, false), "U+202F 末字符贴原子 → prev_gl");
    // <div><span inline-block/>&#x2007;B</div>
    let items = collect_div(&[("span", Some(DisplayValue::InlineBlock), ""), ("t:\u{2007}B", None, "")]);
    assert_eq!(atomic_flags(&items), (false, true), "U+2007 首字符贴原子 → next_gl");
    // WJ（U+2060）同禁断类（line-breaking-atomic-005 族）。
    let items = collect_div(&[("span", Some(DisplayValue::InlineBlock), ""), ("t:\u{2060}B", None, "")]);
    assert_eq!(atomic_flags(&items), (false, true), "WJ 贴原子 → next_gl");
}

/// collect 旁带例外与遍历边界：U+00A0 显式例外不旁带（001/002 断行必须保留）；
/// 普通空格非禁断；GL 跨 display:inline 包裹层仍贴端（字符邻接对 inline 边界
/// 透明）；br 打断邻接。
#[test]
fn r5165_collect_flags_exceptions_and_walk_bounds() {
    // U+00A0 属 GL 但 css-text-3 §5.5 显式例外——不旁带。
    let items = collect_div(&[("t:A\u{A0}", None, ""), ("span", Some(DisplayValue::InlineBlock), "")]);
    assert_eq!(atomic_flags(&items), (false, false), "U+00A0 例外不旁带");
    // 普通空格收尾非禁断。
    let items = collect_div(&[("t:A ", None, ""), ("span", Some(DisplayValue::InlineBlock), "")]);
    assert_eq!(atomic_flags(&items), (false, false), "普通空格贴原子非禁断");
    // GL 在 display:inline 包裹层内，旁带跨层判定。
    let items = collect_div(&[
        ("span", Some(DisplayValue::Inline), "A\u{202F}"),
        ("span", Some(DisplayValue::InlineBlock), ""),
    ]);
    assert_eq!(atomic_flags(&items), (true, false), "GL 跨 inline 包裹层仍贴原子");
    // br 打断字符邻接（后随原子在新行行首，无 fit 门）。
    let items = collect_div(&[
        ("t:A\u{202F}", None, ""),
        ("br", None, ""),
        ("span", Some(DisplayValue::InlineBlock), ""),
    ]);
    assert_eq!(atomic_flags(&items), (false, false), "br 打断字符邻接");
}

/// arm 1：前邻 GL 的原子盒不因 fit 门折行——随前词横向溢出（单行）。
#[test]
fn r5165_arm1_prev_gl_atomic_does_not_wrap() {
    // 生产形态：折叠后 run "A "（U+202F 折叠吸收）+ 原子盒 prev_gl=true，
    // 50px 容器放不下 60px 原子。
    let mut ctx = InlineFormattingContext::new(50.0);
    ctx.break_items_into_lines(vec![text_run("A "), ib_box(60.0, true, false)]);
    assert_eq!(ctx.lines.len(), 1, "前邻 GL：原子盒溢出不折行");
}

/// arm 1 对照：无 GL 旁带的原子盒照旧 fit 折行（既有行为锁定）。
#[test]
fn r5165_arm1_control_no_flag_still_wraps() {
    let mut ctx = InlineFormattingContext::new(50.0);
    ctx.break_items_into_lines(vec![text_run("A "), ib_box(60.0, false, false)]);
    assert_eq!(ctx.lines.len(), 2, "无旁带：fit 折行行为不变");
}

/// arm 2：后邻 GL 的首词不脱离原子折行（溢出单行）。
#[test]
fn r5165_arm2_next_gl_first_word_does_not_wrap() {
    // 生产形态：原子后 run 折叠为 " BBBBBB"（GL 折叠吸收为前导空白），
    // 40px 原子 + 48px 词超 50px 容器。
    let mut ctx = InlineFormattingContext::new(50.0);
    ctx.break_items_into_lines(vec![ib_box(40.0, false, true), text_run(" BBBBBB")]);
    assert_eq!(ctx.lines.len(), 1, "后邻 GL：首词随原子溢出不折行");
}

/// arm 2 对照组：无旁带折行照旧；同 run 非首词与原子不直邻，旁带不适用。
#[test]
fn r5165_arm2_controls() {
    // 对照 1：无旁带——首词照旧折行。
    let mut ctx = InlineFormattingContext::new(50.0);
    ctx.break_items_into_lines(vec![ib_box(40.0, false, false), text_run(" BBBBBB")]);
    assert_eq!(ctx.lines.len(), 2, "无旁带：首词照旧折行");
    // 对照 2：旗标仅 run 首词（word_idx==0）消费——首词 "aa" 放得下即耗尽
    // 旁带，第二词 "bbbbbb" 与原子不直邻，照旧折行。
    let mut ctx = InlineFormattingContext::new(50.0);
    ctx.break_items_into_lines(vec![ib_box(40.0, false, true), text_run(" aa bbbbbb")]);
    assert_eq!(ctx.lines.len(), 2, "非首词与原子不直邻，旁带不适用");
}

/// 谓词成员表：GL/WJ/ZWJ 全集命中、U+00A0 例外排除、普通字符排除。
#[test]
fn r5165_predicate_membership() {
    for ch in [
        '\u{202F}', '\u{2007}', '\u{2011}', '\u{180E}', '\u{2060}', '\u{FEFF}', '\u{200D}', '\u{0F08}',
    ] {
        assert!(
            crate::inline::is_atomic_gl_suppressing(ch),
            "U+{:04X} 应属禁断类",
            ch as u32
        );
    }
    for ch in ['\u{A0}', ' ', '\u{3000}', 'A', '\u{200B}'] {
        assert!(
            !crate::inline::is_atomic_gl_suppressing(ch),
            "U+{:04X} 不应属禁断类",
            ch as u32
        );
    }
}
