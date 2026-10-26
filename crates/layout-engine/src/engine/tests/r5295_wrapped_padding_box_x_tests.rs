//! R5295（CSS2 §8.4 inline box model）：换行 padded 行内盒的 border-box 左缘写回。
//!
//! 两个耦合根因：
//! 1. `collect_items::resolve_inline_padding` 对 Percentage 归 0——span
//!    `padding-left:50%` 的水平推进整体丢失（c547-indent-001 ref div 臂）；
//!    修复为按 IFC 容器宽解析（resolve_inline_padding_pct，resolver 单元测试
//!    在 inline/collect_items.rs::r5295_inline_pct_padding_tests）。
//! 2. `inline_finalization` 盒 x 写回取跨行最小 run.x 再统一扣 first_pl——
//!    换行 padded 行内盒 min_x 命中续行（≈0）再扣首行 padding → 盒 x 负偏
//!    （196px 容器 50% padding 跨 4 行 → x=−98；px padding 同构负偏 −40 为
//!    既有缺口，本测试以 px 形态固化）。修复为聚合处逐片段换算（run 首片段
//!    扣自身 pl、续行/空片段不扣），写回不再统一扣。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::Parser as CssParser;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn compute_span_box(html: &str, css: &str) -> (f32, f32) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let sheets = vec![CssParser::parse_stylesheet(css)];
    let styles = sys.compute_styles(&doc, &sheets);
    let span_id = doc.get_elements_by_tag_name("span")[0];
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let span = find(&result.root, span_id).expect("span 盒");
    (span.x, span.width)
}

/// px padding + 换行：span 盒 border-box 左缘 = 容器内容左缘（0），不得负偏。
/// 修前路径：min_x 取续行 fragment 缘（≈0）再统一扣首行 padding → x=−40。
#[test]
fn r5295_wrapped_padding_inline_box_x_not_negative() {
    let html = r#"<html><body><div><span>X The first X in this sentence should be indented to the center of this block.</span></div></body></html>"#;
    let css = "body{margin:0} div{width:196px} span{padding-left:40px}";
    let (x, _w) = compute_span_box(html, css);
    assert!(
        x.abs() < 0.01,
        "换行 padded 行内盒 x = 容器内容左缘 0（旧式负偏 −40），got {x}"
    );
}

/// 不换行 padded 行内盒：单行场景写回口径不变（x=0，装饰形态盒宽含 padding）。
/// （带背景形态走装饰行内 metrics 写回路径——未装饰行内盒宽不含 padding 为
/// 既有 slice13 挂账近似，不在本修范围。）
#[test]
fn r5295_unwrapped_padding_inline_box_unchanged() {
    let html = r#"<html><body><div><span>X</span></div></body></html>"#;
    let css = "body{margin:0} div{width:400px} span{padding-left:40px;background:lime}";
    let (x, w) = compute_span_box(html, css);
    assert!(x.abs() < 0.01, "单行场景 x=0 不变，got {x}");
    // 宽度断言字体无关：盒宽 = 40 pl + 文本推进（本环境 "X"≈8.8px → 48.8；
    // wpt-runner 管线字体度量略异）。下界排除 padding 丢失（≈文本宽），上界
    // 排除盒缘口径混入内容缘 min_x 的 padding 双计（回归形态 88.8）。
    assert!(w > 40.0 && w < 70.0, "盒宽 = 40 pl + 文本（无双计），got {w}");
}
