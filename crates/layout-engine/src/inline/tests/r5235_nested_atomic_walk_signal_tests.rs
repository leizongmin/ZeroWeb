//! R5235：R1576 递归臂触发集信号（paint Path B 空 styles 镜像分叉）单测。
//!
//! 根因（before-after-positioned-001 盒3/4 叠盖形态，D1）：paint Path B（空 styles）
//! 重跑容器 IFC 时 `inline_elem_has_nested_inline_block` 的 BFS 逐节点
//! `styles.get` 失败即 continue、恒 false → 含 img 后代的 span 落 FLAT_CHILD_WALK，
//! img（leaf、无文本、无 frame）在 walk 分支被 `continue` 丢弃 → items 无
//! InlineBlock 项 → img 后文本少推进一个图宽。layout 期（真实 styles）判定为真、
//! 递归收集、advance 正确——两段分歧。
//!
//! 修复：layout IFC 触发 R1576 臂即记录元素 id（`nested_atomic_walk_nodes`），经
//! `LayoutBox.inline_nested_atomic_nodes` 存储；paint Path B 注入后逐位镜像同一分叉。
//!
//! 覆盖：① 真实 styles 布局后触发集含 span（含嵌套递归层合并回传）；
//! ② 空 styles + 信号集注入 → items 恢复 InlineBlock 项与 img advance；
//! ③ 空 styles + 空信号集（旧行为）→ img 被丢弃（判别臂，证明测试可分辨）。

use super::super::*;
use zero_css_parser::values::{DisplayValue, PositionValue};
use zero_dom::parse_html;
use zero_style_system::ComputedStyle;

/// 构造一个 ComputedStyle，display + position 可指定，其余 default。
fn style(display: DisplayValue, position: PositionValue) -> ComputedStyle {
    let mut s = ComputedStyle::default();
    s.display = display;
    s.position = position;
    s
}

/// 从 doc 取 body 下首个元素（div 容器锚点）。
fn container_id(doc: &Document) -> NodeId {
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    doc.first_child(body).unwrap()
}

/// 容器下首个元素子（span）。
fn first_elem_child(doc: &Document, id: NodeId) -> NodeId {
    doc.child_nodes(id)
        .iter()
        .copied()
        .find(|&c| doc.get(c).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))))
        .unwrap()
}

/// span 下首个元素子（img）；无则 None。
fn first_elem_child_opt(doc: &Document, id: NodeId) -> Option<NodeId> {
    doc.child_nodes(id)
        .iter()
        .copied()
        .find(|&c| doc.get(c).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))))
}

/// 展开行内全部文本 run（跨行扁平化，行装配后的 TextFragment 带 x/width）。
fn text_runs(ctx: &InlineFormattingContext) -> Vec<&TextFragment> {
    ctx.lines
        .iter()
        .flat_map(|l| l.runs.iter())
        .filter(|r| !r.text.is_empty())
        .collect()
}

/// R5235 ①：真实 styles 布局后，触发集须含 span（layout 趟 R1576 臂触发即记录）。
#[test]
fn r5235_signal_records_r1576_trigger() {
    let doc = parse_html(r#"<div><span>PRE<img>POST</span>Inner</div>"#);
    let div = container_id(&doc);
    let span = first_elem_child(&doc, div);
    let img = first_elem_child_opt(&doc, span).unwrap();

    let mut styles = HashMap::new();
    styles.insert(div, style(DisplayValue::Block, PositionValue::Static));
    styles.insert(span, style(DisplayValue::Inline, PositionValue::Static));
    // img UA 默认 display:inline-block（ua_default_display），此处显式同值。
    styles.insert(img, style(DisplayValue::InlineBlock, PositionValue::Static));

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, div, &styles);
    assert!(
        ctx.nested_atomic_walk_nodes.borrow().contains(&span),
        "R1576 arm triggered for span must be recorded for paint Path B mirror; got {:?}",
        ctx.nested_atomic_walk_nodes.borrow()
    );
}

/// R5235 ①b：嵌套递归层（span>span>img）触发的判定须合并回传到容器 ctx 信号集。
#[test]
fn r5235_signal_records_nested_trigger() {
    let doc = parse_html(r#"<div><span><em>PRE<img>POST</em></span>Inner</div>"#);
    let div = container_id(&doc);
    let outer = first_elem_child(&doc, div);
    let inner = first_elem_child(&doc, outer);
    let img = first_elem_child_opt(&doc, inner).unwrap();

    let mut styles = HashMap::new();
    styles.insert(div, style(DisplayValue::Block, PositionValue::Static));
    styles.insert(outer, style(DisplayValue::Inline, PositionValue::Static));
    styles.insert(inner, style(DisplayValue::Inline, PositionValue::Static));
    styles.insert(img, style(DisplayValue::InlineBlock, PositionValue::Static));

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, div, &styles);
    assert!(
        ctx.nested_atomic_walk_nodes.borrow().contains(&outer),
        "outer span must be recorded; got {:?}",
        ctx.nested_atomic_walk_nodes.borrow()
    );
    assert!(
        ctx.nested_atomic_walk_nodes.borrow().contains(&inner),
        "nested inner span trigger must merge back into container ctx set; got {:?}",
        ctx.nested_atomic_walk_nodes.borrow()
    );
}

/// R5235 ②：空 styles + 信号集注入 → items 恢复 InlineBlock 项，img 后文本 run
/// x 含 32px advance（paint Path B 镜像分叉的核心断言）。
#[test]
fn r5235_empty_styles_with_signal_preserves_img_advance() {
    let doc = parse_html(r#"<div><span>PRE<img>POST</span>Inner</div>"#);
    let div = container_id(&doc);
    let span = first_elem_child(&doc, div);
    let img = first_elem_child_opt(&doc, span).unwrap();

    let mut ctx = InlineFormattingContext::new(800.0)
        // paint Path B 通道：`collect_atomic_inline_sizes` 从盒树收集的原子尺寸
        //（img UA display=inline-block，32×32 盒在树中）。
        .with_inline_block_sizes(HashMap::from([(img, (32.0_f32, 32.0_f32))]))
        // layout 期存储的 R1576 触发集。
        .with_nested_atomic_walk_nodes(NodeIdSet::from_iter([span]));
    let empty: HashMap<NodeId, ComputedStyle> = HashMap::new();
    ctx.layout(&doc, div, &empty);

    let runs = text_runs(&ctx);
    let pre = runs.iter().find(|r| r.text == "PRE").expect("PRE run");
    let post = runs.iter().find(|r| r.text == "POST").expect("POST run");
    let advance = post.x - pre.x - pre.width;
    assert!(
        (advance - 32.0).abs() <= 0.5,
        "POST must advance by img width 32 after PRE; got advance={advance:.2} \
         (pre.x={:.2} pre.width={:.2} post.x={:.2})",
        pre.x,
        pre.width,
        post.x
    );
    // img 项参与行盒高度（原子 32 高 > 文本 strut）。
    assert!(
        ctx.lines.first().is_some_and(|l| (l.height - 32.0).abs() <= 0.5),
        "line height must reflect 32px atomic; got {:?}",
        ctx.lines.iter().map(|l| l.height).collect::<Vec<_>>()
    );
}

/// R5235 ③：空 styles + 空信号集（旧行为）→ img 被丢弃、无 advance（判别臂，
/// 证明 ② 的断言可分辨信号有无）。
#[test]
fn r5235_empty_styles_without_signal_drops_img() {
    let doc = parse_html(r#"<div><span>PRE<img>POST</span>Inner</div>"#);
    let div = container_id(&doc);

    let mut ctx = InlineFormattingContext::new(800.0);
    let empty: HashMap<NodeId, ComputedStyle> = HashMap::new();
    ctx.layout(&doc, div, &empty);

    // 旧行为：img 项丢弃 → 行盒仅文本 strut（< 32），POST 无 img advance。
    assert!(
        ctx.lines.first().is_some_and(|l| l.height < 32.0),
        "without signal the img item is dropped; line height must be text-strut only; got {:?}",
        ctx.lines.iter().map(|l| l.height).collect::<Vec<_>>()
    );
}
