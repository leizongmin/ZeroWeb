//! R5300：R2156 跳过（虚拟化）inline 包装的合成盒回填。
//!
//! 背景：R2156 inline-box-model coherence（default-on）把「含嵌套原子 inline 后代」
//! 的 inline 包装整体跳过 taffy 树，其原子后代经 slice19 提升为容器 taffy 直子。
//! 包装因此完全无盒（无 taffy 节点、无 LayoutBox、无 IFC run）——gBCR/命中面对
//! 该形态返回 0×0（p109 复核 z2-z8：`<a><svg></a>` 无样式等价标记 a=0 而 svg=32；
//! 活体 github logo 经站点 CSS 为 `inline-flex`，不走本路径）。CSSOM View 的
//! gBCR = 元素与全部子孙 border box 并集，包装几何必须有出口。
//!
//! 本模块在布局管线终末（全部盒几何/行位同步 pass 之后、LayoutResult 构造前）
//! 按 DOM 祖先链把提升的原子盒重新嵌回虚拟 inline 祖先链（div→a→svg），几何 =
//! 分组子盒并集 ± 自身 padding/border（CSS2 §10.6/§8.4 inline 盒 border box 包住
//! 内容）。终末时点保证同步 pass（R4234 原子行位/R4941 提升持位）仍按基线
//! 「原子盒为容器直子」形态工作；包装盒不参与流（无 bg/border 时零绘制面）。
//! 仅处理**子树无非空白文本**的包装——含文本的包装其几何还须并入容器 IFC
//! 片段域，本切片不展开（p109 问题池挂账）。
//!
//! kill-switch：`ZW_INLINE_WRAPPER_BOX=0` 回退无盒旧行为。
//! https://www.w3.org/TR/cssom-view-1/#dom-element-getboundingclientrect
//! https://www.w3.org/TR/CSS22/visuren.html#inline-boxes

use std::collections::HashMap;

use zero_css_parser::values::{DisplayValue, LengthValue, PositionValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::{ComputedStyle, WritingModeValue};

use crate::types::LayoutBox;

fn synthesis_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    // OPTIMIZATION：kill-switch 进程级单读（R5108 教训：per-node env::var 致
    // bench-gate 微基准超预算）。
    *ON.get_or_init(|| std::env::var("ZW_INLINE_WRAPPER_BOX").as_deref() != Ok("0"))
}

/// 与 tree.rs R2156 skip 判据同源（同条件才可能有被跳过的包装）：
/// inline + 非 abspos/fixed + 含嵌套原子后代 + 子树无 ooflow + 无 in-flow 块级子（R57）。
fn is_r2156_virtual_inline(id: NodeId, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    let Some(s) = styles.get(&id) else {
        return false;
    };
    if !matches!(s.display, DisplayValue::Inline) {
        return false;
    }
    if matches!(s.position, PositionValue::Absolute | PositionValue::Fixed) {
        return false;
    }
    let has_block_child = doc.child_nodes(id).iter().any(|&gc| {
        doc.get(gc).is_some_and(|gn| {
            matches!(&gn.kind, NodeKind::Element(_))
                && styles
                    .get(&gc)
                    .is_some_and(|gs| crate::tree::is_block_level_in_flow(&gs.display, &gs.position))
        })
    });
    crate::inline::InlineFormattingContext::inline_elem_has_nested_inline_block(doc, styles, id)
        && !crate::inline::InlineFormattingContext::inline_subtree_has_ooflow_descendant(doc, styles, id)
        && !has_block_child
}

/// 包装子树是否无非空白文本（含文本的包装本切片不合成——几何须并入 IFC 片段域）。
fn subtree_has_no_text(doc: &Document, id: NodeId) -> bool {
    doc.child_nodes(id).iter().all(|&c| {
        let Some(node) = doc.get(c) else {
            return true;
        };
        match &node.kind {
            NodeKind::Text(t) => t.content.trim().is_empty(),
            NodeKind::Element(_) => subtree_has_no_text(doc, c),
            _ => true,
        }
    })
}

/// 子盒到属主容器之间的连续虚拟 inline 链（DOM 自下而上收集后反转；
/// 首个非虚拟祖先即止——按构造，被跳过包装的原子后代与其容器之间只有虚拟 inline）。
fn virtual_chain(
    start: Option<NodeId>,
    owner: NodeId,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Vec<NodeId> {
    let Some(mut cur) = start else {
        return Vec::new();
    };
    let mut chain = Vec::new();
    while let Some(parent) = doc.parent_node(cur) {
        if parent == owner {
            break;
        }
        if !is_r2156_virtual_inline(parent, doc, styles) {
            break;
        }
        if !subtree_has_no_text(doc, parent) {
            break;
        }
        chain.push(parent);
        cur = parent;
    }
    chain.reverse();
    chain
}

/// 包装自身各维 padding/border（合计 frame + 独立分量供 LayoutBox 字段拆写）。
/// padding 仅取 Px（百分比对 inline 包装按 0 近似——本臂覆盖的原子包装形态未见
/// 百分比 padding，挂账备查）；border none/hidden 记 0（converter 同口径）。
struct Frame {
    pad: [f32; 4], // left, right, top, bottom
    border: [f32; 4],
}

impl Frame {
    fn of(style: Option<&ComputedStyle>) -> Self {
        let zero = Self {
            pad: [0.0; 4],
            border: [0.0; 4],
        };
        let Some(s) = style else {
            return zero;
        };
        let px = |l: &LengthValue| match l {
            LengthValue::Px(n) if n.is_finite() => *n as f32,
            _ => 0.0,
        };
        use zero_style_system::property::types::BorderStyleValue;
        let bw = |w: &LengthValue, st: &BorderStyleValue| match st {
            BorderStyleValue::None | BorderStyleValue::Hidden => 0.0,
            _ => px(w),
        };
        Self {
            pad: [
                px(&s.padding_left),
                px(&s.padding_right),
                px(&s.padding_top),
                px(&s.padding_bottom),
            ],
            border: [
                bw(&s.border_left_width, &s.border_left_style),
                bw(&s.border_right_width, &s.border_right_style),
                bw(&s.border_top_width, &s.border_top_style),
                bw(&s.border_bottom_width, &s.border_bottom_style),
            ],
        }
    }

    fn left(&self) -> f32 {
        self.pad[0] + self.border[0]
    }

    fn right(&self) -> f32 {
        self.pad[1] + self.border[1]
    }

    fn top(&self) -> f32 {
        self.pad[2] + self.border[2]
    }

    fn bottom(&self) -> f32 {
        self.pad[3] + self.border[3]
    }
}

/// 单层包装盒：几何 = 分组子盒并集 ± 自身 frame；子坐标换基到包装内容盒。
fn wrap_one(id: NodeId, style: Option<&ComputedStyle>, mut children: Vec<LayoutBox>) -> LayoutBox {
    let f = Frame::of(style);
    let min_x = children.iter().map(|c| c.x - f.left()).fold(f32::INFINITY, f32::min);
    let min_y = children.iter().map(|c| c.y - f.top()).fold(f32::INFINITY, f32::min);
    let max_x = children
        .iter()
        .map(|c| c.x + c.width + f.right())
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = children
        .iter()
        .map(|c| c.y + c.height + f.bottom())
        .fold(f32::NEG_INFINITY, f32::max);
    let (x, y) = (min_x, min_y);
    let width = (max_x - min_x).max(0.0);
    let height = (max_y - min_y).max(0.0);
    for c in &mut children {
        c.x -= x + f.left();
        c.y -= y + f.top();
    }
    LayoutBox {
        node_id: Some(id),
        x,
        y,
        width,
        height,
        content_x: f.left(),
        content_y: f.top(),
        content_width: (width - f.left() - f.right()).max(0.0),
        content_height: (height - f.top() - f.bottom()).max(0.0),
        padding_left: f.pad[0],
        padding_right: f.pad[1],
        padding_top: f.pad[2],
        padding_bottom: f.pad[3],
        border_left: f.border[0],
        border_right: f.border[1],
        border_top: f.border[2],
        border_bottom: f.border[3],
        children,
        ..LayoutBox::default()
    }
}

/// 链由外到内逐层包裹（[a, span] → a-box(span-box(child))）。
fn build_nested(chain: &[NodeId], styles: &HashMap<NodeId, ComputedStyle>, children: Vec<LayoutBox>) -> LayoutBox {
    let (id, rest) = (chain[0], &chain[1..]);
    let boxed = wrap_one(id, styles.get(&id), children);
    if rest.is_empty() {
        boxed
    } else {
        build_nested(rest, styles, vec![boxed])
    }
}

/// 终末回填入口：自底向上把每层容器的提升原子盒嵌回虚拟 inline 祖先链。
/// 只能在全部盒几何/行位同步 pass 之后、LayoutResult 构造前调用。
pub(crate) fn synthesize_inline_wrapper_boxes_final(
    root: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    if !synthesis_on() {
        return;
    }
    for child in &mut root.children {
        synthesize_inline_wrapper_boxes_final(child, doc, styles);
    }
    let htb = matches!(root.writing_mode, WritingModeValue::HorizontalTb);
    synthesize_inline_wrapper_boxes(&mut root.children, root.node_id, htb, doc, styles);
}

/// 单层容器回填：把被提升为容器 taffy 直子的原子盒按 DOM 祖先链嵌回虚拟
/// inline 包装。仅水平书写模式（R2156 skip 同域）；属主无 DOM 身份（匿名盒）跳过。
fn synthesize_inline_wrapper_boxes(
    children_boxes: &mut Vec<LayoutBox>,
    owner_dom: Option<NodeId>,
    htb: bool,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    if !synthesis_on() || !htb {
        return;
    }
    let Some(owner) = owner_dom else {
        return;
    };
    let chains: Vec<Vec<NodeId>> = children_boxes
        .iter()
        .map(|c| virtual_chain(c.node_id, owner, doc, styles))
        .collect();
    if chains.iter().all(|c| c.is_empty()) {
        return;
    }
    // 组序 = 组首出现序；同链子盒（同包装的多个提升原子）并入同组。
    let mut order: Vec<Vec<NodeId>> = Vec::new();
    let mut members: Vec<Vec<usize>> = Vec::new();
    for (idx, chain) in chains.iter().enumerate() {
        if chain.is_empty() {
            continue;
        }
        if let Some(pos) = order.iter().position(|k| k == chain) {
            members[pos].push(idx);
        } else {
            order.push(chain.clone());
            members.push(vec![idx]);
        }
    }
    let mut original: Vec<Option<LayoutBox>> = children_boxes.drain(..).map(Some).collect();
    let mut wrapped: Vec<LayoutBox> = Vec::with_capacity(original.len());
    for idx in 0..original.len() {
        let chain = &chains[idx];
        if chain.is_empty() {
            wrapped.push(original[idx].take().expect("未消费子盒"));
            continue;
        }
        let pos = order.iter().position(|k| k == chain).expect("链已登记");
        if members[pos][0] != idx {
            continue; // 已随组首消费
        }
        let grouped: Vec<LayoutBox> = members[pos]
            .iter()
            .map(|&i| original[i].take().expect("未消费子盒"))
            .collect();
        wrapped.push(build_nested(chain, styles, grouped));
    }
    *children_boxes = wrapped;
}
