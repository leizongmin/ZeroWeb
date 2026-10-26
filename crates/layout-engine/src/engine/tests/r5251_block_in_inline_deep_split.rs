//! R5251：容器层 block-in-inline 升格拆分锚（CSS2 §9.2.1.1）。
//!
//! 案源：wpt `css/CSS2/box-display/three-block-in-inlines-cascading-margins.html`
//! ——block 容器的 inline 子**自身**含 in-flow block 子（`<span><div></div></span>` ×3）。
//! spec：含块级盒的 inline 盒在其周围断开，块级盒提升到块容器层垂直堆叠，inline 的
//! inline 片段并入容器匿名块。修复前：容器误走纯 inline 行排——含块 span 同挤一行
//!（y 相同 h=18 行高），内嵌块全部重叠同位，容器高塌缩成单行。
//!
//! kill-switch `ZW_R109_DEEP_SPLIT=0` 回退旧口径（本锚随回退失败属预期）。

use super::*;

/// 按 tag 名在盒树中递归查找首个匹配盒（测试导航辅助）。
fn find_box_by_tag<'a>(box_: &'a LayoutBox, doc: &zero_dom::Document, tag: &str) -> Option<&'a LayoutBox> {
    let matched = box_
        .node_id
        .and_then(|id| doc.get(id))
        .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case(tag)));
    if matched {
        return Some(box_);
    }
    box_.children.iter().find_map(|c| find_box_by_tag(c, doc, tag))
}

/// 三带垂直堆叠 + margin collapse：gap = max(前块 mb, 后块 mt)。
#[test]
fn r5251_container_hoists_blocks_from_inline_children() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"width:100px\">\
<span><div style=\"width:100px;height:20px;margin-bottom:30px\"></div></span>\
<span><div style=\"width:100px;height:20px;margin-top:20px;margin-bottom:40px\"></div></span>\
<span><div style=\"width:100px;height:20px;margin-top:50px\"></div></span>\
</div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container = find_box_by_tag(&result.root, &doc, "div")
        .unwrap_or_else(|| panic!("容器盒缺失，树={}", describe(&result.root)));

    // 升格后容器子盒应为 3 个 block div（span 自身无 inline 内容 → 无盒），垂直堆叠。
    assert_eq!(
        container.children.len(),
        3,
        "容器应恰有 3 个块子盒（span 升格消失），实际={}",
        describe(container)
    );
    assert!(
        container.children.iter().all(|c| c.is_block_level),
        "子盒应全部 block-level，实际={}",
        describe(container)
    );

    // y=0/50/120：gap1 = max(30, 20) = 30，gap2 = max(40, 50) = 50（相邻兄弟 margin collapse）。
    let ys: Vec<f32> = container.children.iter().map(|c| c.y).collect();
    let heights: Vec<f32> = container.children.iter().map(|c| c.height).collect();
    assert_eq!(heights, vec![20.0, 20.0, 20.0], "带高={}", describe(container));
    assert!(
        ys[0].abs() < 0.5 && (ys[1] - 50.0).abs() < 0.5 && (ys[2] - 120.0).abs() < 0.5,
        "三带 y 应 0/50/120（collapse gap 30/50），实际 ys={ys:?} 树={}",
        describe(container)
    );
    assert!(
        (container.height - 140.0).abs() < 0.5,
        "容器高应 140=20+30+20+50+20，实际={}",
        container.height
    );
}

/// 升格边界：文本+块混排 wrapper（`<span>text<div></div></span>`）**不**升格——
/// 其匿名块需继承 split inline 盒模型与 wrapper relative 施加（r1748/r5225/r1044b
/// 锚形），维持旧路径：span wrapper 盒仍在（is_inline_r109 自身拆分路径）。
#[test]
fn r5251_mixed_text_block_wrapper_keeps_old_path() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"width:200px\">\
<span id=\"s\">before<div style=\"width:100px;height:20px\"></div>after</span>\
</div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // span wrapper 盒仍存在（旧路径未变），容器子含该 span。
    let span_box = find_box_by_tag(&result.root, &doc, "span").unwrap_or_else(|| {
        panic!(
            "混排 wrapper span 盒应存在（旧路径维持），树={}",
            describe(&result.root)
        )
    });
    assert!(
        !span_box.children.is_empty() && span_box.children.iter().any(|c| c.is_block_level),
        "span wrapper 内应保留块子盒，树={}",
        describe(span_box)
    );
}

fn describe(box_: &LayoutBox) -> String {
    let mut s = String::new();
    fn walk(b: &LayoutBox, depth: usize, s: &mut String) {
        s.push_str(&format!(
            "\n{}box nid={:?} block={} y={} h={}",
            "  ".repeat(depth),
            b.node_id,
            b.is_block_level,
            b.y,
            b.height
        ));
        for c in &b.children {
            walk(c, depth + 1, s);
        }
    }
    walk(box_, 0, &mut s);
    s
}
