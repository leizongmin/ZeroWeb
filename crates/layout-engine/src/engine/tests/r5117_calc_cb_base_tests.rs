//! R5117：calc(P% ± Npx) 百分比基准 = 包含块 content 尺寸测试（css-values-4 §8.5 +
//! CSS2 §10.5）。旧实现以盒子自身布局宽为基（taffy 已解析的百分比部分）→ 百分比
//! 二次应用（300 容器内 calc(50%-10px)：150 → 65，应 140）。driving:
//! box-sizing-content-box-002（float 对 calc(50%-10px) 纵叠，12.59%）。

use crate::engine::LayoutEngine;

fn compute_root(html: &str) -> std::sync::Arc<crate::types::LayoutBox> {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles).root
}

/// 普通块：300 容器内 calc(50%-10px) → 140（旧基 65）。
#[test]
fn test_r5117_normal_block_calc_percent_base_is_containing_block() {
    let root = compute_root(
        r#"<html><body style="margin:0"><div style="width:300px;height:100px"><div style="width:calc(50% - 10px);height:40px"></div></div></body></html>"#,
    );
    // 树形不定（anon 层）：取最深盒（内层 div）。
    fn deepest(b: &crate::types::LayoutBox) -> &crate::types::LayoutBox {
        let mut cur = b;
        while let Some(last) = cur.children.last() {
            cur = last;
        }
        cur
    }
    let child = deepest(&root);
    assert!((child.width - 140.0).abs() < 0.5, "w={}", child.width);
}

/// float 对：calc(50%-10px) + padding 5 + content-box → content 140、border-box
/// 150，两块并排（box-sizing-content-box-002 主体）。
#[test]
fn test_r5117_float_content_box_calc_side_by_side() {
    let root = compute_root(
        r#"<html><body style="margin:0"><div style="width:300px;height:100px;position:relative">
        <div style="box-sizing:content-box;width:calc(50% - 10px);height:90px;float:left;padding:5px;background:green"></div>
        <div style="box-sizing:content-box;width:calc(50% - 10px);height:90px;float:left;padding:5px;background:blue"></div>
        </div></body></html>"#,
    );
    // 找两个 float 盒（同层兄弟、宽度 >100）。
    fn find_pair(
        b: &crate::types::LayoutBox,
    ) -> Option<(&crate::types::LayoutBox, &crate::types::LayoutBox)> {
        for w in b.children.windows(2) {
            if w[0].width > 100.0 && w[1].width > 100.0 {
                return Some((&w[0], &w[1]));
            }
        }
        for c in &b.children {
            if let Some(p) = find_pair(c) {
                return Some(p);
            }
        }
        None
    }
    let (a, b) = find_pair(&root).expect("float 对");
    // content-box：calc 值 140 = content；border-box = 150。
    assert!((a.width - 150.0).abs() < 0.5, "a w={}", a.width);
    assert!((a.content_width - 140.0).abs() < 0.5, "a cw={}", a.content_width);
    // 并排：b 的 x = a 右缘。
    assert!(
        (b.x - (a.x + a.width)).abs() < 0.5,
        "b x={} a right={}",
        b.x,
        a.x + a.width
    );
    assert!((b.y - a.y).abs() < 0.5, "b y={} a y={}", b.y, a.y);
}
