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

/// R3770c：remeasure 增高 block 子盒后，后续 in-flow 兄弟随之下移（与收缩位移对称）。
/// driving: line-clamp-with-abspos-010 的 ref 页结构——[pre 文本 anon] + [relative 子
/// （remeasure 0→64 增高）] + [trailing anon 'Line 4…']：trailing anon 须从旧 taffy 位
/// （与 .rel 重叠）移到 .rel 之后。
#[test]
fn r3770c_growth_of_block_child_shifts_following_siblings() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font: 16px/32px serif; white-space: pre; background-color: yellow;\">Line 1\
<div style=\"position: relative;\">Line 2\nLine 3</div>Line 4</div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_mixed_container(b: &LayoutBox) -> Option<&LayoutBox> {
        for c in &b.children {
            if c.children.iter().any(|g| g.fragment_node_ids.is_some()) && c.children.iter().any(|g| g.is_relative) {
                return Some(c);
            }
            if let Some(f) = find_mixed_container(c) {
                return Some(f);
            }
        }
        None
    }
    let container = find_mixed_container(&result.root).expect("mixed container（frag 子 + relative 子）");
    let rel = container
        .children
        .iter()
        .find(|c| c.is_relative)
        .expect("relative child");
    assert_eq!(rel.height, 64.0, "relative 子 remeasure 后 2 行 64");
    let rel_bottom = rel.y + rel.height;
    let trailing = container
        .children
        .iter()
        .filter(|c| c.fragment_node_ids.is_some())
        .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
        .expect("trailing anon fragment");
    assert!(
        trailing.y >= rel_bottom - 0.5,
        "trailing anon（y={:.1}）应移到增高后的 relative 子（bottom={:.1}）之后",
        trailing.y,
        rel_bottom
    );
}

/// R3770c 守卫：inline-level 子（ruby 等）的 remeasure 增长**不**触发兄弟位移——
/// 同行 inline 流语义下 taffy 竖排是既有约定（driving: line-clamp-026 回归）。
#[test]
fn r3770c_inline_child_growth_does_not_shift_siblings() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font: 16px/32px serif; background-color: yellow;\">\
<ruby>a<rt>x</ruby><ruby>b<rt>y</ruby><ruby>c<rt>z</ruby></div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn ruby_ys(b: &LayoutBox, out: &mut Vec<f32>) {
        for c in &b.children {
            if c.node_id.is_some() && c.height == 32.0 && c.width >= 800.0 {
                out.push(c.y);
            }
            ruby_ys(c, out);
        }
    }
    let mut ys = Vec::new();
    ruby_ys(&result.root, &mut ys);
    assert!(ys.len() >= 2, "前置：应有多 ruby 盒，实得 {}", ys.len());
    let all_same = ys.iter().all(|&y| (y - ys[0]).abs() < 0.5);
    assert!(all_same, "同行 ruby 盒 y 应一致（不被增长位移推开），实得 {:?}", ys);
}

/// R3770d：clamp 边界上的零高 in-flow 子盒豁免整体隐藏——空 div 不占行预算，恰在
/// clamp point 处而非之后，其 abspos shown（css-overflow-4 with-abspos-023：
/// 「other-wise empty, zero-height div, which does fit before the clamp point」）。
#[test]
fn r3770d_zero_height_child_at_clamp_boundary_survives() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: 4; font: 16px/32px serif; background-color: yellow;\">\
Line 1<br>Line 2<br>Line 3<br>Line 4<br>\
<div style=\"position: relative;\"><div style=\"position: absolute; right: 0; width: 100px; height: 100px; background-color: green;\">V</div></div>\
Line 5</div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_boundary_rel(b: &LayoutBox) -> Option<&LayoutBox> {
        for c in &b.children {
            if c.is_relative && !c.is_absolute {
                let has_abs = c.children.iter().any(|g| g.is_absolute);
                if has_abs {
                    return Some(c);
                }
            }
            if let Some(f) = find_boundary_rel(c) {
                return Some(f);
            }
        }
        None
    }
    let rel = find_boundary_rel(&result.root).expect("boundary .rel box");
    assert!(
        !rel.line_clamp_hidden && rel.width > 0.0,
        "零高 .rel 在 clamp 边界不隐藏（fits before the clamp point）"
    );
    let abspos = rel.children.iter().find(|c| c.is_absolute).expect("abspos box");
    assert!(
        !abspos.line_clamp_hidden && abspos.width > 0.0 && abspos.height > 0.0,
        "边界 .rel 内 abspos 保留几何（spec: shown）"
    );
    // clamp 点后的有内容子仍隐藏（Line 5 在 clamp point 之后）。
    let container = &result.root;
    fn any_hidden_after(b: &LayoutBox) -> bool {
        b.children.iter().any(|c| c.line_clamp_hidden) || b.children.iter().any(any_hidden_after)
    }
    assert!(
        any_hidden_after(container),
        "clamp point 后的有内容子（Line 5 anon）仍隐藏"
    );
}

/// R3771：独立 BFC 子盒（overflow:hidden/auto/scroll）豁免跨块 clamp 行计数——其行不计
/// 预算、不截断、整体照绘（css-overflow-4：clamp 行计数跳过 independent formatting
/// context 子树）。
/// driving: webkit-line-clamp-029（overflow:hidden .child 5 行全显，ref 为不 clamp 全内容）、
/// webkit-line-clamp-008（两个 overflow:hidden div 行不计数，clamp 点落容器自身 IFC 第 2 行）。
#[test]
fn r3771_cross_block_clamp_skips_bfc_child_lines() {
    // 预算 3：block 子 A（2 行）+ BFC 子（overflow:hidden，2 行）+ block 子 C（2 行）。
    // 旧实现（无 BFC 豁免）：A 消 2、BFC 子被 leaf 计 2 行 → 第 3 行截断 C。
    // 新行为：BFC 子不计数 → C 完整消耗第 3-4 行超预算 → C 截到 1 行 + BFC 子全 2 行可见。
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: 3; font: 16px/32px serif; white-space: pre; background-color: yellow;\">\
<div>Line 1\nLine 2</div>\
<div class=\"bfc\" style=\"overflow: hidden;\">Line A\nLine B</div>\
<div class=\"tail\">Line 3\nLine 4</div></div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_child<'a>(b: &'a LayoutBox, doc: &'a zero_dom::Document, class: &str) -> Option<&'a LayoutBox> {
        for c in &b.children {
            let hits = c
                .node_id
                .and_then(|id| doc.get(id))
                .and_then(|n| match &n.kind {
                    zero_dom::NodeKind::Element(e) => e.attributes.iter().find_map(|a| {
                        if a.name.local.as_ref() == "class" {
                            Some(a.value.to_string())
                        } else {
                            None
                        }
                    }),
                    _ => None,
                })
                .is_some_and(|cls| cls.split_whitespace().any(|w| w == class));
            if hits {
                return Some(c);
            }
            if let Some(f) = find_child(c, doc, class) {
                return Some(f);
            }
        }
        None
    }
    // BFC 子完整保留（未被 cap/隐藏）。
    let bfc = find_child(&result.root, &doc, "bfc").expect("BFC child box");
    assert!(!bfc.line_clamp_hidden, "BFC 子盒不被跨块隐藏");
    assert_eq!(bfc.line_clamp_cap, None, "BFC 子盒不受 clamp cap");
    assert_eq!(bfc.height, 64.0, "BFC 子盒 2 行 64px 完整保留");
    // clamp 点落在 BFC 子之后的 block 子 C（预算余 1 行）。
    let c = find_child(&result.root, &doc, "tail").expect("tail block child box");
    assert_eq!(c.line_clamp_cap, Some(1), "后续 block 子被截到余量 1 行");
}

/// R3771 对照：flow-root 子盒**不是**独立 BFC 豁免——其行仍参与 clamp 计数
///（css-overflow-4 auto-034：clamp point 落于两个 IFC 之间，flow-root 子行计数）。
#[test]
fn r3771_flow_root_child_still_counts_lines() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: 3; font: 16px/32px serif; white-space: pre; background-color: yellow;\">\
<div style=\"display: flow-root;\">Line 1\nLine 2</div>\
<div>Line 3\nLine 4</div></div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_clamp_container<'a>(
        b: &'a LayoutBox,
        styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
    ) -> Option<&'a LayoutBox> {
        for c in &b.children {
            if c.node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| s.line_clamp != zero_style_system::property::types::LineClampComputedValue::None)
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
    // 预算 3：flow-root 子消 2 行（**正常计数**，未被 BFC 豁免跳过），后续兄弟截到余量 1 行。
    assert!(
        container.children.iter().any(|c| c.line_clamp_cap == Some(1)),
        "flow-root 子行参与计数：后续兄弟截到余量 1 行"
    );
    assert!(
        container.children.iter().all(|c| !c.line_clamp_hidden),
        "flow-root 子与后续兄弟均不被整体隐藏（flow-root 非独立 BFC）"
    );
}

/// R3772：跨块 clamp leaf 收缩 used line-height 取**被 cap 子盒自身**样式——行盒属于子盒
/// IFC，行高由子盒 font/line-height 决定，与 clamp 容器（可能无字体声明、line-height 继承
/// normal）无关。
/// driving: line-clamp-with-abspos-017（.child `font: 16px/32px monospace`、.clamp 无字体
/// → 旧实现收缩高 3×18.62=55.9px，应 3×32=96px）。
#[test]
fn r3772_cross_block_cap_shrink_uses_child_line_height() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: 3; background-color: yellow;\">\
<div style=\"font: 16px/32px monospace; white-space: pre;\">Line 1\nLine 2\nLine 3\nLine 4</div></div>\
</body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_clamp_container<'a>(
        b: &'a LayoutBox,
        styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
    ) -> Option<&'a LayoutBox> {
        for c in &b.children {
            if c.node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| s.line_clamp != zero_style_system::property::types::LineClampComputedValue::None)
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
    let child = container.children.first().expect("text child");
    assert_eq!(child.line_clamp_cap, Some(3), "4 行文本被截到余量 3 行");
    assert_eq!(
        child.height, 96.0,
        "收缩高 = 3 × 子盒自身 32px 行高 = 96（非容器 normal 行高 55.9）"
    );
}

/// R3773：R109 split inline（block-in-inline）载体盒对跨块 clamp 可见——载体递归计入
/// 预算，其内 inline 片段行计入预算、block 子正常 cap。
/// driving: line-clamp-030（span 包 [Line 1 片段 + div(2 行) + div(2 行) + Line 6 片段]，
/// 预算 4 → clamp 点落第 2 个 div 首行，旧实现整簇不 clamp 渲全部 6 行）。
#[test]
fn r3773_block_in_inline_carrier_clamps() {
    let html = r##"<html><body style="margin:0">
<div style="line-clamp: 4; font: 16px/32px serif; background-color: yellow; padding: 0 4px;">
  <span>
    Line 1
    <div>
      Line 2 <br>
      Line 3
    </div>
    <div>
      Line 4 <br>
      Line 5
    </div>
    Line 6
  </span>
</div>
</body></html>"##;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // clamp 点落第 2 个 div 首行：该 div cap=1 行 + clamped；第 1 个 div 为 ellipsis
    // host（cap=2 末行补 …）；L6 片段隐藏；容器收缩到 4 行。
    let mut caps: Vec<Option<usize>> = Vec::new();
    fn collect_caps(b: &LayoutBox, out: &mut Vec<Option<usize>>) {
        out.push(b.line_clamp_cap);
        for c in &b.children {
            collect_caps(c, out);
        }
    }
    collect_caps(&result.root, &mut caps);
    assert!(
        caps.contains(&Some(2)) && caps.contains(&Some(1)),
        "div1 host cap=2 + div2 cap=1（L4 + …），旧实现全树无 cap：{caps:?}"
    );
    fn find_hidden(b: &LayoutBox) -> bool {
        b.line_clamp_hidden || b.children.iter().any(find_hidden)
    }
    assert!(find_hidden(&result.root), "L6 片段（clamp 点后）被隐藏");
    // 容器收缩到可见 extent：L1(32) + div1(64) + div2 一行(32) = 128（不含隐藏的 L5/L6）。
    fn find_clamp_container<'a>(
        b: &'a LayoutBox,
        styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
    ) -> Option<&'a LayoutBox> {
        for c in &b.children {
            if c.node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| s.line_clamp != zero_style_system::property::types::LineClampComputedValue::None)
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
    assert!(
        (container.height - 128.0).abs() < 1.0,
        "容器收缩到 4 行 128px（旧实现 6 行全显 192px）"
    );
}

/// R3773 踩坑对照：ruby 元素（display:Inline、盒高含注音）不作行计数/收缩——
/// 027/028 实证误计数致回退；ruby 行数已由容器自身 IFC（R1022 rb 文本收集）承载。
#[test]
fn r3773_ruby_inline_children_not_counted() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: 3; font-size: 16px/16px serif; white-space: pre-wrap; background-color: yellow;\">\
Line 1\nLine 2\n<ruby style=\"font-size: 48px;\">Line 3<rt>r</rt></ruby>\nLine 4</div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_ruby<'a>(b: &'a LayoutBox, doc: &'a zero_dom::Document) -> Option<&'a LayoutBox> {
        for c in &b.children {
            let is_ruby = c
                .node_id
                .and_then(|id| doc.get(id))
                .and_then(|n| match &n.kind {
                    zero_dom::NodeKind::Element(e) => Some(e.local_name() == "ruby"),
                    _ => None,
                })
                .unwrap_or(false);
            if is_ruby {
                return Some(c);
            }
            if let Some(f) = find_ruby(c, doc) {
                return Some(f);
            }
        }
        None
    }
    let ruby = find_ruby(&result.root, &doc).expect("ruby box");
    assert_eq!(ruby.line_clamp_cap, None, "ruby 盒不被行计数收缩");
    assert!(!ruby.line_clamp_clamped, "ruby 盒不被置 clamped");
}
