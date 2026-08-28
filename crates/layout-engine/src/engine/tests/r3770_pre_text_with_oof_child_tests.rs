//! R3770 回归：white-space:pre 容器高度在 remeasure/R109 片段路径的多行保真。
//!
//! 三层根因（均表现为「多行 pre 文本被测成 1 行，容器高度塌缩」）：
//! 1. `remeasure_inline_only_containers`（step 6.5）的 IFC 漏传 preserve/break_at_newline
//!    ——[OOF 子 + pre 直接文本] 容器（abspos 不贡献 taffy 高，taffy content_height=0 →
//!    needs_dom_text_remeasure 走此 IFC）`\n` 被折叠，4 行 128px 被测成 32px
//!    （line-clamp-with-abspos-002/004/006/008 族）。
//! 2. 跨块 line-clamp `walk_children` 进入子盒时 `remaining == 0` = 整个子盒（含其
//!    abspos/fixed 后代与嵌套 CB 盒）都在 clamp point 之后 → 整体隐藏。旧实现仍递归
//!    下传，嵌套 CB 盒自身与 abspos 照留（line-clamp-with-abspos-011/012/022）。
//! 3. R109 匿名块片段非 stored（非 pure-Ahem）路径高度 = taffy ctx_node 单行测量，
//!    多行片段欠计且无人回填（R109_BACKFILL ① 只对 stored inline_layout 生效）
//!    （line-clamp-with-abspos-011/013 的 4 行 anon 片段被测成 32px）。
//!
//! 规范：CSS2 §9.5（abspos 脱流不占位）、css-overflow-4 §line-clamp（clamp point 后
//! 的 containing block 不绘制）、CSS2 §9.2.1.1（匿名块盒高度 = 其行盒内容）。
use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn compute_body_height(html: &str) -> f32 {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn body_height(b: &LayoutBox, doc: &zero_dom::Document) -> Option<f32> {
        for c in &b.children {
            let is_body = c
                .node_id
                .and_then(|id| doc.get(id))
                .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "body"));
            if is_body {
                return Some(c.height);
            }
            if let Some(h) = body_height(c, doc) {
                return Some(h);
            }
        }
        None
    }

    body_height(&result.root, &doc).expect("body box")
}

const ABSPOS_SKYBLUE: &str =
    "<div style=\"position: absolute; top: 0; left: 0; width: 20px; height: 20px; background-color: skyblue;\"></div>";

/// 根因 1：[abspos 子 + pre 直接文本] 容器高度 = 4 行 128px（塌缩 32px 回归）。
/// driving: line-clamp-with-abspos-002/004/006/008。
#[test]
fn r3770_pre_text_with_abspos_child_keeps_line_count() {
    let html = format!(
        "<html><body style=\"margin:0\">\
<div style=\"font: 16px/32px serif; padding: 0 4px; white-space: pre; background-color: yellow;\">{ABSPOS_SKYBLUE}Line 1\nLine 2\nLine 3\nLine 4</div>\
</body></html>"
    );
    assert_eq!(compute_body_height(&html), 128.0, "abspos 脱流，4 行 pre 文本高度 128");
}

/// 根因 1 对照（inline 元素子变体）：span 子 + pre 文本同样保行数。
#[test]
fn r3770_pre_text_with_inline_child_keeps_line_count() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font: 16px/32px serif; padding: 0 4px; white-space: pre; background-color: yellow;\">\
<span></span>Line 1\nLine 2\nLine 3\nLine 4</div></body></html>";
    assert_eq!(compute_body_height(html), 128.0);
}

/// 根因 2：clamp point 后的嵌套 CB 盒整体隐藏（含其 abspos），容器收缩到可见 extent。
/// driving: line-clamp-with-abspos-012（CB 完全在 clamp point 后 → abspos 不绘制）。
/// 注：断言 clamp 容器自身高度（祖先 body 不收缩是既有全局行为，非本修复范围）。
#[test]
fn r3770_cross_block_clamp_hides_subtree_after_clamp_point() {
    let html = format!(
        "<html><body style=\"margin:0\">\
<div style=\"line-clamp: 4; font: 16px/32px serif; padding: 0 4px; background-color: yellow;\">\
<div>Line 1</div><div>Line 2</div><div>Line 3</div><div>Line 4</div>\
<div style=\"position: relative;\">{ABSPOS_SKYBLUE}<div>Line 5</div><div>Line 6</div></div></div>\
</body></html>"
    );
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_clamp_container(b: &LayoutBox) -> Option<&LayoutBox> {
        for c in &b.children {
            if c.line_clamp_hidden || c.children.iter().any(|g| g.line_clamp_hidden) {
                return Some(c);
            }
            if let Some(f) = find_clamp_container(c) {
                return Some(f);
            }
        }
        None
    }
    let container = find_clamp_container(&result.root).expect("clamp container with hidden subtree");
    assert_eq!(container.height, 128.0, "clamp 点后 .rel 整体隐藏，容器收缩到 4 行 128");
}

/// 根因 3：R109 匿名块片段（非 Ahem 非 stored）多行片段高度回填。
/// 结构：clamp 容器 [4 行 pre 直接文本] + [block 子] → R109 mixed split，
/// anon 片段高度应为 4 行 128px 而非 ctx_node 单行 32px。
/// driving: line-clamp-with-abspos-011/013。
#[test]
fn r3770_r109_anon_fragment_multiline_height_backfill() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font: 16px/32px serif; white-space: pre; background-color: yellow;\">Line 1\nLine 2\nLine 3\nLine 4\
<div style=\"height: 20px; background-color: skyblue;\"></div></div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_anon(b: &LayoutBox) -> Option<f32> {
        if b.fragment_node_ids.is_some() {
            return Some(b.content_height);
        }
        b.children.iter().find_map(find_anon)
    }
    let anon_h = find_anon(&result.root).expect("R109 anon fragment box");
    assert_eq!(anon_h, 128.0, "多行 anon 片段高度 = 4 行 128px（非 ctx 单行 32px）");
}

/// R3770b：clamp 点落在嵌套 CB 中部时祖先盒高度压缩传播 + ellipsis host。
/// driving: line-clamp-with-abspos-014（.rel 含 clamp point，L4 可见 + …，L5 隐藏）。
#[test]
fn r3770b_nested_cb_midway_clamp_compacts_heights() {
    let html = format!(
        "<html><body style=\"margin:0\">\
<div style=\"line-clamp: 4; font: 16px/32px serif; padding: 0 4px; background-color: yellow;\">\
<div>Line 1</div><div>Line 2</div><div>Line 3</div>\
<div style=\"position: relative;\"><div>Line 4</div><div>Line 5</div>{ABSPOS_SKYBLUE}</div></div>\
</body></html>"
    );
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    use zero_style_system::property::types::LineClampComputedValue as LCC;
    fn find_clamp_container<'a>(
        b: &'a LayoutBox,
        styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
    ) -> Option<&'a LayoutBox> {
        for c in &b.children {
            if c.node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.line_clamp, LCC::Count(4)))
            {
                return Some(c);
            }
            if let Some(f) = find_clamp_container(c, styles) {
                return Some(f);
            }
        }
        None
    }
    let container = find_clamp_container(&result.root, &styles).expect("clamp container");
    assert_eq!(container.height, 128.0, "clamp 点在 .rel 内部：容器收缩到 4 行 128");
    // 含 clamp 点的嵌套 CB 盒自身也压缩到可见 extent（L4 一行）。
    fn find_nested_cb(b: &LayoutBox) -> Option<&LayoutBox> {
        for c in &b.children {
            if c.is_relative && c.children.iter().any(|g| g.line_clamp_hidden) {
                return Some(c);
            }
            if let Some(f) = find_nested_cb(c) {
                return Some(f);
            }
        }
        None
    }
    let rel = find_nested_cb(container).expect("nested CB box");
    assert_eq!(rel.height, 32.0, "嵌套 CB 盒压缩到 L4 一行 32（L5 隐藏不计）");
    // CB 含 clamp point 的 abspos 保留几何（不被隐藏清零）。
    let abspos = rel.children.iter().find(|c| c.is_absolute).expect("abspos box");
    assert!(
        !abspos.line_clamp_hidden && abspos.width > 0.0,
        "CB 含 clamp point 的 abspos 不隐藏（spec: shown iff CB precedes or contains clamp point）"
    );
}

/// R3770b：ellipsis host——clamp point 落在最后完整消耗预算的子末行末，省略号附该子
/// 末行末（cap = 消耗行数 + clamped；paint 截到自身行数 no-op + 补 …）。
/// driving: line-clamp-with-abspos-010/014（'Line 4…'）。
#[test]
fn r3770b_ellipsis_host_marked_on_last_consuming_child() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: 4; font: 16px/32px serif; padding: 0 4px; background-color: yellow;\">\
<div>Line 1</div><div>Line 2</div><div>Line 3</div><div>Line 4</div><div>Line 5</div></div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_host(b: &LayoutBox) -> Option<&LayoutBox> {
        for c in &b.children {
            if c.line_clamp_clamped && c.line_clamp_cap == Some(1) {
                return Some(c);
            }
            if let Some(f) = find_host(c) {
                return Some(f);
            }
        }
        None
    }
    let host = find_host(&result.root).expect("ellipsis host box");
    assert!(
        host.line_clamp_clamped,
        "L4（第 4 个完整消耗预算的子）应为 ellipsis host"
    );
}
