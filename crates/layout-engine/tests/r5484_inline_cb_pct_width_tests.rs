//! R5484：display:inline 中间层百分比宽包含块修正（CSS2 §10.2 包含块定义）。
//!
//! floats-149 形：`<div style="display:inline">` 内 `<span style="float:left;
//! width:50%">`——CSS2 §10.2：position:static 的 display:inline 盒**不建立包含块**，
//! 子百分比宽应相对最近**块容器**祖先 content 宽（400）解算 = 200（chromium oracle
//! 同）。taffy 物化层把 inline 中间层表示为 fill 剩余宽的块状节点（blk 400 − inline
//! margin 40 = 360），converter 把 percent 交给 taffy（converter/mod.rs percent 直通）
//! 即相对物化层 360 解算 → 180（chromium 偏离）。修复＝tree.rs build_subtree 后处理
//! 窄臂把「DOM 父 display:inline 且 position:static」的节点百分比宽预解算为 definite
//! （祖先链首个块容器 definite Px 宽，border-box 扣 frame）。
//!
//! taffy 树 dump 定谳（ZW_TAFFY_TREE_DUMP=1，会话临时探针）：inline 中间层物化节点
//! dom=9 loc=(40,0) size=360x0——物化 fill 宽即分母侵蚀源头。
//! kill-switch：`ZW_INLINE_CB_PCT_WIDTH=0`（R3858 独立进程锚见
//! r5484_inline_cb_pct_width_kill_switch.rs）。

use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::StyleSystem;

fn sheets(html: &str) -> Vec<zero_css_parser::Stylesheet> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        let Some(tail_start) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + tail_start + 1..];
        let Some(end) = after.find("</style>") else { break };
        out.push(zero_css_parser::Parser::parse_stylesheet(&after[..end]));
        rest = &after[end + 8..];
    }
    out
}

fn find_box(root: &LayoutBox, id: zero_dom::NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn span1_width(html: &str) -> f32 {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let inline_div = doc.get_element_by_id("inl").expect("div.inline 存在");
    let span_ids: Vec<zero_dom::NodeId> = doc
        .child_nodes(inline_div)
        .into_iter()
        .filter(|&c| {
            doc.get(c)
                .is_some_and(|n| matches!(n.kind, zero_dom::NodeKind::Element(_)))
        })
        .collect();
    find_box(&result.root, span_ids[0]).expect("span1 盒存在").width
}

const TPL: &str = r#"<html><head><style>
body { margin: 0; font: 16px/18px Ahem, sans-serif; }
.block { display: block; width: 400px; }
.inline { display: inline; MARGIN }
span { float: left; width: 50%; }
</style></head><body>
<div class="block" id="blk"><div class="inline" id="inl">
  <span>Ahem Ahem</span><span></span>
</div></div>
</body></html>"#;

/// floats-149 形斩口锚：blk 400 + inline ml 40 → span width:50% 应= 200（块容器
/// content 宽为分母；pre-fix 物化层分母 360 → 180 红证据谱系）。
#[test]
fn r5484_pct_width_uses_block_container_width() {
    let html = TPL.replace("MARGIN", "margin-left: 40px;");
    let w = span1_width(&html);
    assert!(
        (w - 200.0).abs() <= 0.5,
        "inline 中间层子百分比宽应相对块容器 400 解算 = 200（chromium oracle），实际 {w}"
    );
}

/// inline 无 margin 变体：物化层分母无侵蚀（360→400）时旧径已绿，本锚防窄臂对
/// 已对齐路径引入回归。
#[test]
fn r5484_pct_width_no_inline_margin_stays_200() {
    let html = TPL.replace("MARGIN", "margin-left: 0px;");
    let w = span1_width(&html);
    assert!((w - 200.0).abs() <= 0.5, "无 inline margin 时分母 400 不变，实际 {w}");
}

/// 块容器自身 margin 变体：blk ml 60 后其 content 宽仍 400（viewport 800 内），
/// 窄臂取块容器 content 而非物化层（400−60−40=300 → 150 旧径偏离域）。
#[test]
fn r5484_pct_width_ignores_block_margin_offset() {
    let html = TPL.replace("MARGIN", "margin-left: 40px;").replace(
        ".block { display: block; width: 400px; }",
        ".block { display: block; width: 400px; margin-left: 60px; }",
    );
    let w = span1_width(&html);
    assert!(
        (w - 200.0).abs() <= 0.5,
        "块容器 margin 不改 content 宽 400，span 应仍 = 200，实际 {w}"
    );
}

/// abspos 排除锚（§10.2 定位节点包含块＝最近定位祖先 padding box）：abspos span
/// width:50% 相对 ICB 800 解算 = 400，窄臂不得用块容器分母覆写为 200。
#[test]
fn r5484_pct_width_abspos_node_excluded_from_arm() {
    let html = r#"<html><head><style>
body { margin: 0; font: 16px/18px Ahem, sans-serif; }
.block { display: block; width: 400px; }
.inline { display: inline; margin-left: 40px; }
span.abs { position: absolute; width: 50%; }
span { float: left; width: 50%; }
</style></head><body>
<div class="block" id="blk"><div class="inline" id="inl">
  <span class="abs">X</span><span>Ahem</span>
</div></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let inline_div = doc.get_element_by_id("inl").expect("div.inline 存在");
    let span_ids: Vec<zero_dom::NodeId> = doc
        .child_nodes(inline_div)
        .into_iter()
        .filter(|&c| {
            doc.get(c)
                .is_some_and(|n| matches!(n.kind, zero_dom::NodeKind::Element(_)))
        })
        .collect();
    let abs_span = find_box(&result.root, span_ids[0]).expect("abspos span 盒存在");
    assert!(
        (abs_span.width - 400.0).abs() <= 0.5,
        "abspos 节点百分比宽归 taffy 定位 CB（ICB 800×50% = 400），窄臂排除，实际 {}",
        abs_span.width
    );
    let fl_span = find_box(&result.root, span_ids[1]).expect("float span 盒存在");
    assert!(
        (fl_span.width - 200.0).abs() <= 0.5,
        "同树 float span 仍走窄臂 = 200（块容器分母），实际 {}",
        fl_span.width
    );
}
