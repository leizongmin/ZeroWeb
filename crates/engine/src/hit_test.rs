//! 布局树命中测试 — 用于链接点击等交互。
//!
//! 已知限制（PR #81 审查记档）：block-in-inline 配置（pe:none 块盒被 inline 包裹，
//! text hoist 路径）下穿透命中回落到包裹 inline 祖先而非完全穿透——与匿名盒/hoist
//! 盒 nearest-element 上溯归属机制同根，非命中谓词层面。

use std::collections::{HashMap, HashSet};

use slotmap::{Key, KeyData};
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::LayoutBox;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::{PointerEventsValue, VisibilityValue};

/// slice15（R4384）：inline 命中面 kill-switch（默认开，"0" 回退布局树行盒几何）。
/// 关断后命中遍历忽略 `inline_reported_rect`，恢复 slice13 返修语义（行盒几何命中面）。
/// 进程级稳定开关，公开边界读一次（LazyLock 快照），不进逐节点热路径。
static INLINE_HIT_SURFACE_ON: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("ZW_INLINE_HIT_SURFACE").as_deref() != Ok("0"));

/// 命中包含判定消费的 `(y, h)`（父内容区坐标系，与 `LayoutBox.y` 同帧）。
///
/// slice15（R4384）：inline 非替换盒（layout 侧已记录 `inline_reported_rect` 时）命中面 =
/// **上报 border box**（content area 主字体 A+D ± padding/border，不含 leading）——与 gBCR
/// 面同源（同一字段，slice13 端到端）。活体实证 Chrome inline 命中面 = content area：
/// 行距 gap 带命中包含块、content 带内命中锚本体（slice13 rework-chrome-gap-*.json +
/// slice15 逐点差异表，diag/evidence/slice15/）。此前读树行盒几何（含上 half-leading、
/// vertical-align 不感知），行距宽的页面命中区大于视觉区。
/// 规范：css-ui-4 §6.2 pointer-events——命中域 = 元素生成的盒（普通 hit-testing 规范
/// 明确 open issue 未成文）；Chrome 对 inline 的实现 = 逐 fragment border box
///（content area ± padding/border，CSS2 §10.6.2 content area 与 §10.8.1 leading 属行盒）。
/// 其余盒（reported 缺席：块级/替换/跨行 wrap 并集）回退树几何。
/// https://drafts.csswg.org/css-ui-4/#hit-testing
/// https://www.w3.org/TR/CSS22/visudet.html#inline-non-replaced
fn hit_extent(layout: &LayoutBox) -> (f32, f32) {
    if *INLINE_HIT_SURFACE_ON && let Some((reported_y, reported_h)) = layout.inline_reported_rect {
        (reported_y, reported_h)
    } else {
        (layout.y, layout.height)
    }
}

/// 元素 computed visibility 是否不可见（Hidden/Collapse，与绘制侧 painter 谓词一致）：
/// 不可见盒不绘制、命中穿透（布局保留——gBCR/布局不受影响）。
/// https://drafts.csswg.org/css-visibility/#visibility
fn is_hidden_style(styles: &HashMap<NodeId, ComputedStyle>, node: NodeId) -> bool {
    styles
        .get(&node)
        .is_some_and(|style| matches!(style.visibility, VisibilityValue::Hidden | VisibilityValue::Collapse))
}

/// 元素 computed pointer-events 是否为 none：盒自身不是命中目标（鼠标/elementFromPoint
/// 穿透），后代显式 `pointer-events: auto` 恢复可命中——与 visibility 同构（候选资格
/// 剥夺 + 继续下探）。真站实证：bilibili 轮播遮罩 `pointer-events:none` 盖住搜索框，
/// 命中未穿透致点击路由命中遮罩、焦点不迁移、键入不落值。
/// 宽化边界：只判 `none`，SVG 值（visiblePainted/fill/stroke…）一律按可命中——盒级
/// 命中模型下正确（Chrome 对 HTML 元素亦只区分 auto/none），SVG 几何级语义（如
/// `fill:none` + `visiblePainted` 不可命中）在无 SVG 几何命中面前提下不可表达。
/// https://drafts.csswg.org/css-ui-4/#pointer-events
fn is_pe_none_style(styles: &HashMap<NodeId, ComputedStyle>, node: NodeId) -> bool {
    styles
        .get(&node)
        .is_some_and(|style| style.pointer_events == PointerEventsValue::None)
}

/// 命中遍历共享上下文：查询点 + 候选资格谓词。
///
/// CSS Visibility：hidden/collapse 盒不参与命中，但后代显式 `visibility: visible` 仍可命中
/// ——递归不剪枝，仅剥夺盒自身的候选资格。 https://drafts.csswg.org/css-visibility/#visibility
/// CSS pointer-events（css-ui-4）：`none` 盒同构——自身非命中目标，后代显式 `auto` 恢复。
/// https://drafts.csswg.org/css-ui-4/#pointer-events
struct HitWalk<'a> {
    point_x: f32,
    point_y: f32,
    /// 盒 node_id → 是否不可见。live 树按 computed style 判定；缓存树按构建期集合判定。
    is_hidden: &'a dyn Fn(NodeId) -> bool,
    /// 盒 node_id → 是否 `pointer-events: none`（同构语义，见 struct 文档）。
    is_pe_none: &'a dyn Fn(NodeId) -> bool,
}

/// 主线程只读命中测试快照（由 tab worker 在推送快照时构建）。
#[derive(Debug, Clone)]
pub struct HitTestCache {
    layout_root: LayoutBox,
    doc_root: NodeId,
    nodes: HashMap<NodeId, HitTestNodeMeta>,
    parents: HashMap<NodeId, NodeId>,
    /// 构建期 computed visibility hidden/collapse 的元素（快照导出为 `hidden_nodes`，
    /// 跨进程恢复后语义一致）。
    hidden: HashSet<NodeId>,
    /// 构建期 computed `pointer-events: none` 的元素（快照导出为 `pe_none_nodes`，
    /// 跨进程恢复后语义一致）。https://drafts.csswg.org/css-ui-4/#pointer-events
    pe_none: HashSet<NodeId>,
}

#[derive(Debug, Clone)]
struct HitTestNodeMeta {
    tag_name: String,
    id: Option<String>,
    class_name: Option<String>,
    selector: String,
    href: Option<String>,
    /// 图片 `src`（仅 `img` 元素，绝对化后存储）。
    src: Option<String>,
}

impl HitTestCache {
    /// 从管线缓存的 DOM 与布局树构建命中测试快照。
    ///
    /// `styles` 用于构建期标记 computed visibility hidden/collapse 的元素（命中穿透，
    /// 见 [`HitWalk`]）；布局树原样保留（hidden 元素仍参与布局，gBCR 返回真实 rect）。
    pub fn from_document(doc: &Document, layout_root: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> Self {
        let mut nodes = HashMap::new();
        let mut parents = HashMap::new();
        collect_hit_test_nodes(layout_root, doc, &mut nodes, &mut parents);
        Self {
            layout_root: layout_root.clone(),
            doc_root: doc.root(),
            nodes,
            parents,
            hidden: styles
                .iter()
                .filter(|(id, style)| {
                    matches!(style.visibility, VisibilityValue::Hidden | VisibilityValue::Collapse)
                        && doc
                            .get(**id)
                            .is_some_and(|data| matches!(data.kind, NodeKind::Element(_)))
                })
                .map(|(id, _)| *id)
                .collect(),
            pe_none: styles
                .iter()
                .filter(|(id, style)| {
                    style.pointer_events == PointerEventsValue::None
                        && doc
                            .get(**id)
                            .is_some_and(|data| matches!(data.kind, NodeKind::Element(_)))
                })
                .map(|(id, _)| *id)
                .collect(),
        }
    }

    /// 命中测试链接，返回 `href`（若存在）。
    pub fn hit_test_link(&self, x: f32, y: f32) -> Option<String> {
        let walk = HitWalk {
            point_x: x,
            point_y: y,
            is_hidden: &|n| self.hidden.contains(&n),
            is_pe_none: &|n| self.pe_none.contains(&n),
        };
        let mut best = (0, self.doc_root);
        deepest_node_at(&self.layout_root, 0.0, 0.0, 0, &mut best, &walk);
        find_link_href_cached(best.1, &self.nodes, &self.parents)
    }

    /// 命中测试图片，返回 `src`（若点中 img 或其子元素）。
    pub fn hit_test_image(&self, x: f32, y: f32) -> Option<String> {
        let walk = HitWalk {
            point_x: x,
            point_y: y,
            is_hidden: &|n| self.hidden.contains(&n),
            is_pe_none: &|n| self.pe_none.contains(&n),
        };
        let mut best = (0, self.doc_root);
        deepest_node_at(&self.layout_root, 0.0, 0.0, 0, &mut best, &walk);
        find_image_src_cached(best.1, &self.nodes, &self.parents)
    }

    /// 命中测试元素，返回最深元素及其布局盒。
    pub fn hit_test_element(&self, x: f32, y: f32) -> Option<ElementHit> {
        let walk = HitWalk {
            point_x: x,
            point_y: y,
            is_hidden: &|n| self.hidden.contains(&n),
            is_pe_none: &|n| self.pe_none.contains(&n),
        };
        let mut best = (0, self.doc_root);
        deepest_node_at(&self.layout_root, 0.0, 0.0, 0, &mut best, &walk);
        element_hit_from_cache(&self.layout_root, best.1, &self.nodes, &self.parents)
    }

    /// 命中测试：返回 `(x,y)` 处所有元素，按绘制序（最前/最深在前 → 最后/最浅在后）。
    ///
    /// 收集所有包含该点的盒（[`collect_nodes_at`]），按深度降序（深度≈绘制层级，最深元素绘制
    /// 在最前），每盒经 [`nearest_element_cached`] 取其元素并去重（同元素多盒仅保留最深=最前那次）。
    /// [`HitTestCache::hit_test_element`]（=`elementFromPoint`）即本序列的首元素。z-index/绝对定位
    /// 的精确绘制序未建模（树深近似，见 elementFromPoint 已知限制）。
    pub fn elements_at_point(&self, x: f32, y: f32) -> Vec<ElementHit> {
        let walk = HitWalk {
            point_x: x,
            point_y: y,
            is_hidden: &|n| self.hidden.contains(&n),
            is_pe_none: &|n| self.pe_none.contains(&n),
        };
        let mut hits: Vec<(usize, NodeId)> = Vec::new();
        collect_nodes_at(&self.layout_root, 0.0, 0.0, 0, &mut hits, &walk);
        // 深度降序：最前/最深在前（sort_by_key + Reverse 稳定，同深保文档序）。
        hits.sort_by_key(|b| std::cmp::Reverse(b.0));
        let mut seen: HashSet<NodeId> = HashSet::new();
        let mut out = Vec::new();
        for (_, node) in hits {
            let element = nearest_element_cached(node, &self.nodes, &self.parents);
            if seen.insert(element)
                && let Some(hit) = element_hit_from_cache(&self.layout_root, element, &self.nodes, &self.parents)
            {
                out.push(hit);
            }
        }
        out
    }

    /// 导出可跨进程传输的快照（不含完整 DOM）。
    pub fn snapshot(&self) -> HitTestCacheSnapshot {
        HitTestCacheSnapshot {
            doc_root: self.doc_root,
            layout_root: layout_snapshot_from_box(&self.layout_root),
            nodes: self
                .nodes
                .iter()
                .map(|(id, meta)| {
                    (
                        *id,
                        HitTestNodeSnapshot {
                            tag_name: meta.tag_name.clone(),
                            id: meta.id.clone(),
                            class_name: meta.class_name.clone(),
                            selector: meta.selector.clone(),
                            href: meta.href.clone(),
                            src: meta.src.clone(),
                        },
                    )
                })
                .collect(),
            parents: self.parents.iter().map(|(c, p)| (*c, *p)).collect(),
            hidden_nodes: self.hidden.iter().map(|id| node_id_to_u64(*id)).collect(),
            pe_none_nodes: self.pe_none.iter().map(|id| node_id_to_u64(*id)).collect(),
        }
    }

    /// 从跨进程快照恢复命中测试缓存。
    pub fn from_snapshot(snap: HitTestCacheSnapshot) -> Self {
        Self {
            layout_root: layout_box_from_snapshot(&snap.layout_root),
            doc_root: snap.doc_root,
            nodes: snap
                .nodes
                .into_iter()
                .map(|(id, meta)| {
                    (
                        id,
                        HitTestNodeMeta {
                            tag_name: meta.tag_name,
                            id: meta.id,
                            class_name: meta.class_name,
                            selector: meta.selector,
                            href: meta.href,
                            src: meta.src,
                        },
                    )
                })
                .collect(),
            parents: snap.parents.into_iter().collect(),
            hidden: snap.hidden_nodes.into_iter().map(node_id_from_u64).collect(),
            pe_none: snap.pe_none_nodes.into_iter().map(node_id_from_u64).collect(),
        }
    }

    /// P1a gBCR：把布局树每节点 rect（相对父内容区）写入共享 rect snapshot。
    /// 直接遍历内部 `layout_root`（避免 [`Self::snapshot`] 的整树 clone）。render 后调；
    /// 无 `node_id` 的匿名/伪盒跳过。js_worker 的 RectBridge handler 经 identity→NodeId 查此 snapshot。
    pub fn fill_layout_rect_snapshot(&self, snapshot: &crate::rect_bridge::LayoutRectSnapshot) {
        if let Ok(mut map) = snapshot.lock() {
            map.clear();
            fill_rect_from_layout_box(&self.layout_root, 0.0, 0.0, &mut map);
        }
    }
}

/// `LayoutBox` 递归填充 rect snapshot（`fill_layout_rect_snapshot` 的内部实现，直接走 LayoutBox 避 clone）。
fn fill_rect_from_layout_box(
    box_node: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
    map: &mut HashMap<u64, crate::rect_bridge::Rect4>,
) {
    // slice13（CSS2 §10.6.2）：inline 盒 gBCR 上报 y/h = content area（主字体 A+D +
    // padding/border）。布局树 y/h 保持行盒几何，sync 记录 `inline_reported_rect`，
    // 此处（gBCR rect 桥直填路径）消费上报值；slice15 起（R4384）命中面
    //（deepest_node_at/collect_nodes_at 经 hit_extent）同消费上报值——命中面与 gBCR
    // 面同源。子盒偏移仍按布局 y 累计（子盒 y 存于布局坐标系，上报覆写只作用于本盒 rect）。
    // https://www.w3.org/TR/CSS22/visudet.html#inline-non-replaced
    let (reported_y, reported_h) = box_node.inline_reported_rect.unwrap_or((box_node.y, box_node.height));
    let box_x = abs_x + box_node.x;
    let box_y = abs_y + reported_y;
    if let Some(id) = box_node.node_id {
        map.insert(node_id_to_u64(id), (box_x, box_y, box_node.width, reported_h));
    }
    let (child_x, child_y) = child_origin(box_node, box_x, abs_y + box_node.y);
    for child in &box_node.children {
        fill_rect_from_layout_box(child, child_x, child_y, map);
    }
}

/// IPC / 快照可传输的命中测试布局节点（仅几何 + node id）。
#[derive(Debug, Clone)]
pub struct HitTestLayoutSnapshot {
    /// 关联 DOM 节点。
    pub node_id: Option<NodeId>,
    /// 相对父内容区 x。
    pub x: f32,
    /// 相对父内容区 y。
    pub y: f32,
    /// 盒宽。
    pub width: f32,
    /// 盒高。
    pub height: f32,
    /// slice13：inline 盒 gBCR 上报 (y, h)（content area，与 `LayoutBox` 同坐标约定——
    /// 相对父内容区）。slice15（R4384）起 gBCR 面（rect 桥）与命中面（hit_extent）同消费
    /// 此字段——命中面 = 上报 border box，与 Chrome 一致（gap 带命中包含块）。
    /// `y`/`height` 字段保持布局树行盒几何（子盒坐标累积锚）。
    /// https://www.w3.org/TR/CSS22/visudet.html#inline-non-replaced
    pub reported: Option<(f32, f32)>,
    /// 子盒。
    pub children: Vec<HitTestLayoutSnapshot>,
}

/// IPC / 快照可传输的命中测试节点元数据。
#[derive(Debug, Clone)]
pub struct HitTestNodeSnapshot {
    /// 标签名（小写）。
    pub tag_name: String,
    /// `id` 属性。
    pub id: Option<String>,
    /// `class` 属性。
    pub class_name: Option<String>,
    /// 在文档中唯一定位该元素的选择器。
    pub selector: String,
    /// 链接 `href`（仅 `a` 元素）。
    pub href: Option<String>,
    /// 图片 `src`（仅 `img` 元素）。
    pub src: Option<String>,
}

/// IPC / 快照可传输的完整命中测试缓存。
#[derive(Debug, Clone)]
pub struct HitTestCacheSnapshot {
    /// 文档根节点。
    pub doc_root: NodeId,
    /// 布局树根。
    pub layout_root: HitTestLayoutSnapshot,
    /// 元素元数据。
    pub nodes: Vec<(NodeId, HitTestNodeSnapshot)>,
    /// 父节点索引。
    pub parents: Vec<(NodeId, NodeId)>,
    /// computed visibility hidden/collapse 的元素（[`node_id_to_u64`] 编码；命中穿透）。
    pub hidden_nodes: Vec<u64>,
    /// computed `pointer-events: none` 的元素（[`node_id_to_u64`] 编码；命中穿透）。
    /// https://drafts.csswg.org/css-ui-4/#pointer-events
    pub pe_none_nodes: Vec<u64>,
}

fn layout_snapshot_from_box(layout: &LayoutBox) -> HitTestLayoutSnapshot {
    layout_snapshot_from_box_with_offset(layout, 0.0, 0.0)
}

fn layout_snapshot_from_box_with_offset(
    layout: &LayoutBox,
    parent_content_x: f32,
    parent_content_y: f32,
) -> HitTestLayoutSnapshot {
    // slice13（CSS2 §10.6.2）：inline 非替换盒 getBoundingClientRect y/h 上报语义 =
    // content area（主字体 A+D + padding/border，与 line-height 无关）。快照 y/height
    // **保持布局树行盒几何**（子盒坐标累积锚）；slice15（R4384）起上报值随 `reported`
    // 字段携带，rect 桥（gBCR）与命中遍历（hit_extent）同消费——单/多进程命中面与
    // gBCR 面同源、同 Chrome。跨行 wrap 并集语义挂账。
    // https://www.w3.org/TR/CSS22/visudet.html#inline-non-replaced
    HitTestLayoutSnapshot {
        node_id: layout.node_id,
        x: layout.x + parent_content_x,
        y: layout.y + parent_content_y,
        width: layout.width,
        height: layout.height,
        reported: layout.inline_reported_rect,
        children: layout
            .children
            .iter()
            .map(|child| layout_snapshot_from_box_with_offset(child, child_offset_x(layout), child_offset_y(layout)))
            .collect(),
    }
}

fn layout_box_from_snapshot(snapshot: &HitTestLayoutSnapshot) -> LayoutBox {
    LayoutBox {
        node_id: snapshot.node_id,
        x: snapshot.x,
        y: snapshot.y,
        width: snapshot.width,
        height: snapshot.height,
        // 上报值物化回 LayoutBox（坐标约定不变，相对父内容区）——主进程
        // fill_layout_rect_snapshot（gBCR 直填方法路径）据此填 rect 表，与渲染进程
        // live 树同值；slice15 起命中遍历（hit_extent）同消费——快照往返后命中面
        // 与 gBCR 面仍同源。
        inline_reported_rect: snapshot.reported,
        children: snapshot.children.iter().map(layout_box_from_snapshot).collect(),
        ..LayoutBox::default()
    }
}

/// 将 `NodeId` 编码为 IPC 友好的整数。
pub fn node_id_to_u64(id: NodeId) -> u64 {
    id.data().as_ffi()
}

/// 从 IPC 整数解码 `NodeId`。
pub fn node_id_from_u64(value: u64) -> NodeId {
    NodeId::from(KeyData::from_ffi(value))
}

fn collect_hit_test_nodes(
    layout: &LayoutBox,
    doc: &Document,
    nodes: &mut HashMap<NodeId, HitTestNodeMeta>,
    parents: &mut HashMap<NodeId, NodeId>,
) {
    if let Some(node_id) = layout.node_id
        && let Some(data) = doc.get(node_id)
    {
        if let NodeKind::Element(elem) = &data.kind {
            let tag = elem.local_name().to_ascii_lowercase();
            let href = if tag == "a" {
                doc.get_attribute(node_id, "href")
            } else {
                None
            };
            let src = if tag == "img" {
                doc.get_attribute(node_id, "src")
            } else {
                None
            };
            nodes.insert(
                node_id,
                HitTestNodeMeta {
                    tag_name: tag,
                    id: doc.get_attribute(node_id, "id"),
                    class_name: doc.get_attribute(node_id, "class"),
                    // FIXME(R3254-L8)：每节点唯一选择器随每帧 PaintSnapshot 全量传输——
                    // 深 DOM 结构路径显著膨胀 IPC 负载且构建为 O(N²)（每节点一次全文档
                    // query_selector_all）。已评估按需生成/截断方案收益风险比低，deferred。
                    selector: crate::unique_selector_for_node(doc, node_id)
                        .unwrap_or_else(|| elem.local_name().to_ascii_lowercase()),
                    href,
                    src,
                },
            );
        }
        if let Some(parent) = doc.parent_node(node_id) {
            parents.insert(node_id, parent);
        }
    }
    for child in &layout.children {
        collect_hit_test_nodes(child, doc, nodes, parents);
    }
}

fn find_link_href_cached(
    mut node: NodeId,
    nodes: &HashMap<NodeId, HitTestNodeMeta>,
    parents: &HashMap<NodeId, NodeId>,
) -> Option<String> {
    loop {
        if let Some(meta) = nodes.get(&node)
            && meta.tag_name == "a"
            && let Some(href) = &meta.href
        {
            let href = href.trim();
            if !href.is_empty() && href != "#" {
                return Some(href.to_string());
            }
        }
        node = parents.get(&node).copied()?;
    }
}

/// 从命中节点向上查找最近的 `img` 元素的 `src`（绝对化后）。
fn find_image_src_cached(
    mut node: NodeId,
    nodes: &HashMap<NodeId, HitTestNodeMeta>,
    parents: &HashMap<NodeId, NodeId>,
) -> Option<String> {
    loop {
        if let Some(meta) = nodes.get(&node)
            && meta.tag_name == "img"
            && let Some(src) = &meta.src
        {
            let src = src.trim();
            if !src.is_empty() {
                return Some(src.to_string());
            }
        }
        node = parents.get(&node).copied()?;
    }
}

fn nearest_element_cached(
    mut node: NodeId,
    nodes: &HashMap<NodeId, HitTestNodeMeta>,
    parents: &HashMap<NodeId, NodeId>,
) -> NodeId {
    loop {
        if nodes.contains_key(&node) {
            return node;
        }
        node = match parents.get(&node) {
            Some(p) => *p,
            None => return node,
        };
    }
}

fn element_hit_from_cache(
    layout: &LayoutBox,
    node: NodeId,
    nodes: &HashMap<NodeId, HitTestNodeMeta>,
    parents: &HashMap<NodeId, NodeId>,
) -> Option<ElementHit> {
    let element = nearest_element_cached(node, nodes, parents);
    let meta = nodes.get(&element)?;
    let (x, y, width, height) = layout_box_for_node(layout, element, 0.0, 0.0)?;
    Some(ElementHit {
        node_handle: node_id_to_u64(element),
        tag_name: meta.tag_name.clone(),
        id: meta.id.clone(),
        class_name: meta.class_name.clone(),
        selector: meta.selector.clone(),
        x,
        y,
        width,
        height,
    })
}

/// 在布局树中查找点击位置对应的最深 DOM 节点。
fn deepest_node_at(
    layout: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
    depth: usize,
    best: &mut (usize, NodeId),
    walk: &HitWalk,
) {
    let box_x = abs_x + layout.x;
    let box_y = abs_y + layout.y;
    // slice15（R4384）：包含判定消费命中面 `hit_extent`（inline = 上报 border box，与
    // gBCR 面同源）；子盒坐标累积仍走布局树 y（`child_origin`，与 fill_rect_from_layout_box
    // 同款——子盒 y 存于布局坐标系，命中面覆写只作用于本盒）。
    let (hit_y, hit_h) = hit_extent(layout);
    let hit_top = abs_y + hit_y;
    let contains = walk.point_x >= box_x
        && walk.point_y >= hit_top
        && walk.point_x < box_x + layout.width
        && walk.point_y < hit_top + hit_h;

    // S12（cdp-protocol hit-target）：**不按祖先包含剪枝**——祖先盒不包含点仍继续下探，
    // 只把「盒包含点」的节点记入候选。祖先盒可能小于溢出的子内容（实测：body 高 6px、
    // 按钮 24.6px 溢出——按钮在自身中心 elementFromPoint 返 html 兜底）；真浏览器按绘制
    // 盒命中，溢出内容（overflow:visible）可命中。overflow:hidden 的裁剪语义未建模
    //（FIXME：被裁剪子盒在此近似下仍可命中，边缘语义偏差可接受）。
    // CSS Visibility：hidden/collapse 盒不绘制，剥夺候选资格但继续下探（后代显式
    // visible 仍可命中）。https://drafts.csswg.org/css-visibility/#visibility
    if contains
        && let Some(node_id) = layout.node_id
        && !(walk.is_hidden)(node_id)
        && !(walk.is_pe_none)(node_id)
        && depth >= best.0
    {
        *best = (depth, node_id);
    }

    let (child_x, child_y) = child_origin(layout, box_x, box_y);
    for child in &layout.children {
        deepest_node_at(child, child_x, child_y, depth + 1, best, walk);
    }
}

/// 收集所有包含 `(walk.point_x, walk.point_y)` 的盒节点（含深度），供 [`HitTestCache::elements_at_point`]。
/// 镜像 [`deepest_node_at`] 的包含判定与坐标累积（同 `LayoutBox` 坐标相对父内容区须累积），
/// 但收集全部命中盒而非仅最深。点不在盒内则不递归（与 `deepest_node_at` 一致）。
fn collect_nodes_at(
    layout: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
    depth: usize,
    out: &mut Vec<(usize, NodeId)>,
    walk: &HitWalk,
) {
    let box_x = abs_x + layout.x;
    let box_y = abs_y + layout.y;
    // slice15（R4384）：同 deepest_node_at——包含判定消费命中面（inline = 上报 border box），
    // 子盒累积走布局树 y。
    let (hit_y, hit_h) = hit_extent(layout);
    let hit_top = abs_y + hit_y;
    let contains = walk.point_x >= box_x
        && walk.point_y >= hit_top
        && walk.point_x < box_x + layout.width
        && walk.point_y < hit_top + hit_h;

    // S12：同 deepest_node_at——不按祖先包含剪枝（溢出子内容可命中），仅记录包含点
    // 的盒（elementsAtPoint 序列语义不变）；hidden/collapse 盒剥夺候选资格。
    if contains
        && let Some(node_id) = layout.node_id
        && !(walk.is_hidden)(node_id)
        && !(walk.is_pe_none)(node_id)
    {
        out.push((depth, node_id));
    }

    let (child_x, child_y) = child_origin(layout, box_x, box_y);
    for child in &layout.children {
        collect_nodes_at(child, child_x, child_y, depth + 1, out, walk);
    }
}

/// 从节点向上查找最近的 `<a href="...">`。
fn find_link_href(doc: &Document, mut node: NodeId) -> Option<String> {
    loop {
        let is_anchor = doc.get(node).is_some_and(
            |data| matches!(&data.kind, NodeKind::Element(elem) if elem.local_name().eq_ignore_ascii_case("a")),
        );
        if is_anchor && let Some(href) = doc.get_attribute(node, "href") {
            let href = href.trim();
            if !href.is_empty() && href != "#" {
                return Some(href.to_string());
            }
        }
        node = doc.parent_node(node)?;
    }
}

/// 从节点向上查找最近的 `<img src="...">`。
fn find_image_src(doc: &Document, mut node: NodeId) -> Option<String> {
    loop {
        let is_img = doc.get(node).is_some_and(
            |data| matches!(&data.kind, NodeKind::Element(elem) if elem.local_name().eq_ignore_ascii_case("img")),
        );
        if is_img && let Some(src) = doc.get_attribute(node, "src") {
            let src = src.trim();
            if !src.is_empty() {
                return Some(src.to_string());
            }
        }
        node = doc.parent_node(node)?;
    }
}

/// 元素命中测试结果（文档坐标系）。
#[derive(Debug, Clone, PartialEq)]
pub struct ElementHit {
    /// 当前 Document 内的 opaque DOM 节点句柄。
    pub node_handle: u64,
    /// 元素标签名（小写）。
    pub tag_name: String,
    /// `id` 属性。
    pub id: Option<String>,
    /// `class` 属性。
    pub class_name: Option<String>,
    /// 在文档中唯一定位该元素的选择器。
    pub selector: String,
    /// 布局盒左上角 X（CSS 逻辑像素）。
    pub x: f32,
    /// 布局盒左上角 Y。
    pub y: f32,
    /// 布局盒宽度。
    pub width: f32,
    /// 布局盒高度。
    pub height: f32,
}

/// 从命中测试结果构造用于 JS 事件派发的稳定选择器。
pub fn selector_from_element_hit(hit: &ElementHit) -> String {
    if !hit.selector.is_empty() {
        return hit.selector.clone();
    }
    if let Some(id) = &hit.id {
        let id = id.trim();
        if !id.is_empty() {
            return format!("#{}", id);
        }
    }
    if let Some(class) = &hit.class_name {
        let first = class.split_whitespace().find(|c| !c.is_empty());
        if let Some(c) = first {
            return format!("{}.{}", hit.tag_name, c);
        }
    }
    hit.tag_name.clone()
}

fn nearest_element_node(doc: &Document, mut node: NodeId) -> NodeId {
    loop {
        if doc
            .get(node)
            .is_some_and(|data| matches!(data.kind, NodeKind::Element(_)))
        {
            return node;
        }
        node = match doc.parent_node(node) {
            Some(p) => p,
            None => return node,
        };
    }
}

fn layout_box_for_node(layout: &LayoutBox, target: NodeId, abs_x: f32, abs_y: f32) -> Option<(f32, f32, f32, f32)> {
    let box_x = abs_x + layout.x;
    let box_y = abs_y + layout.y;
    if layout.node_id == Some(target) {
        return Some((box_x, box_y, layout.width, layout.height));
    }
    let (child_x, child_y) = child_origin(layout, box_x, box_y);
    for child in &layout.children {
        if let Some(found) = layout_box_for_node(child, target, child_x, child_y) {
            return Some(found);
        }
    }
    None
}

fn child_origin(layout: &LayoutBox, box_x: f32, box_y: f32) -> (f32, f32) {
    (box_x + child_offset_x(layout), box_y + child_offset_y(layout))
}

fn child_offset_x(layout: &LayoutBox) -> f32 {
    let scroll_x = if matches!(layout.overflow_x, zero_layout_engine::OverflowClip::Scroll) {
        layout.scroll_x
    } else {
        0.0
    };
    layout.border_left + layout.padding_left - scroll_x
}

fn child_offset_y(layout: &LayoutBox) -> f32 {
    let scroll_y = if matches!(layout.overflow_y, zero_layout_engine::OverflowClip::Scroll) {
        layout.scroll_y
    } else {
        0.0
    };
    layout.border_top + layout.padding_top - scroll_y
}

fn element_hit_from_node(doc: &Document, layout: &LayoutBox, node: NodeId) -> Option<ElementHit> {
    let element = nearest_element_node(doc, node);
    let data = doc.get(element)?;
    let NodeKind::Element(elem) = &data.kind else {
        return None;
    };
    let (x, y, width, height) = layout_box_for_node(layout, element, 0.0, 0.0)?;
    Some(ElementHit {
        node_handle: node_id_to_u64(element),
        tag_name: elem.local_name().to_ascii_lowercase(),
        id: doc.get_attribute(element, "id"),
        class_name: doc.get_attribute(element, "class"),
        selector: crate::unique_selector_for_node(doc, element)
            .unwrap_or_else(|| elem.local_name().to_ascii_lowercase()),
        x,
        y,
        width,
        height,
    })
}

/// 在文档布局中命中测试链接，返回 `href`（若存在）。
/// `styles` 供 visibility 命中穿透判定（hidden 盒不参与命中，见 [`HitWalk`]）。
pub fn hit_test_link(
    doc: &Document,
    layout: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    x: f32,
    y: f32,
) -> Option<String> {
    let walk = HitWalk {
        point_x: x,
        point_y: y,
        is_hidden: &|n| is_hidden_style(styles, n),
        is_pe_none: &|n| is_pe_none_style(styles, n),
    };
    let mut best = (0, doc.root());
    deepest_node_at(layout, 0.0, 0.0, 0, &mut best, &walk);
    find_link_href(doc, best.1)
}

/// 在文档布局中命中测试图片，返回 `src`（文档原始值，未绝对化）。
pub fn hit_test_image(
    doc: &Document,
    layout: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    x: f32,
    y: f32,
) -> Option<String> {
    let walk = HitWalk {
        point_x: x,
        point_y: y,
        is_hidden: &|n| is_hidden_style(styles, n),
        is_pe_none: &|n| is_pe_none_style(styles, n),
    };
    let mut best = (0, doc.root());
    deepest_node_at(layout, 0.0, 0.0, 0, &mut best, &walk);
    find_image_src(doc, best.1)
}

/// 在文档布局中命中测试元素，返回最深元素及其布局盒。
pub fn hit_test_element(
    doc: &Document,
    layout: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    x: f32,
    y: f32,
) -> Option<ElementHit> {
    let walk = HitWalk {
        point_x: x,
        point_y: y,
        is_hidden: &|n| is_hidden_style(styles, n),
        is_pe_none: &|n| is_pe_none_style(styles, n),
    };
    let mut best = (0, doc.root());
    deepest_node_at(layout, 0.0, 0.0, 0, &mut best, &walk);
    element_hit_from_node(doc, layout, best.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::Parser;
    use zero_layout_engine::LayoutEngine;
    use zero_style_system::StyleSystem;

    /// 辅助函数：解析 HTML 并运行完整样式+布局管线。
    fn render(html: &str, css: &str) -> (Document, zero_layout_engine::LayoutResult) {
        let doc = zero_dom::parse_html(html);
        let stylesheets = vec![Parser::parse_stylesheet(css)];
        let mut style_system = StyleSystem::new();
        style_system.set_viewport(800.0, 600.0);
        let styles = style_system.compute_styles(&doc, &stylesheets);
        let mut layout_engine = LayoutEngine::new(800.0, 600.0);
        let layout = layout_engine.compute(&doc, &styles);
        (doc, layout)
    }

    #[test]
    fn nested_content_offsets_are_included_in_hit_and_rect_geometry() {
        let doc = zero_dom::parse_html(r#"<body><fieldset><input id="target"></fieldset></body>"#);
        let body = doc.get_elements_by_tag_name("body")[0];
        let fieldset = doc.get_elements_by_tag_name("fieldset")[0];
        let input = doc.get_element_by_id("target").expect("target input");
        let mut root = LayoutBox {
            node_id: Some(body),
            width: 800.0,
            height: 600.0,
            padding_left: 8.0,
            padding_top: 8.0,
            ..LayoutBox::default()
        };
        let mut fieldset_box = LayoutBox {
            node_id: Some(fieldset),
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 200.0,
            border_left: 2.0,
            border_top: 2.0,
            padding_left: 16.0,
            padding_top: 16.0,
            ..LayoutBox::default()
        };
        fieldset_box.children.push(LayoutBox {
            node_id: Some(input),
            x: 5.0,
            y: 6.0,
            width: 100.0,
            height: 40.0,
            ..LayoutBox::default()
        });
        root.children.push(fieldset_box);
        let cache = HitTestCache::from_document(&doc, &root, &HashMap::new());

        let hit = cache.hit_test_element(42.0, 53.0).expect("input hit");
        assert_eq!(hit.id.as_deref(), Some("target"));
        assert_eq!((hit.x, hit.y, hit.width, hit.height), (41.0, 52.0, 100.0, 40.0));

        let snapshot = crate::rect_bridge::new_layout_rect_snapshot();
        cache.fill_layout_rect_snapshot(&snapshot);
        let rects = snapshot.lock().expect("rect snapshot");
        assert_eq!(rects[&node_id_to_u64(input)], (41.0, 52.0, 100.0, 40.0));
    }

    /// slice13（CSS2 §10.6.2）：`HitTestCache::fill_layout_rect_snapshot`（gBCR 直填
    /// 路径，webview/tab_worker 消费）同样消费 `inline_reported_rect` 上报值——布局树
    /// y/h 保持行盒几何，仅 rect 上报变为 content area。子盒 rect 仍按布局帧累计
    ///（子盒 y 存于布局坐标系，覆写只作用于本盒）。
    /// https://www.w3.org/TR/CSS22/visudet.html#inline-non-replaced
    #[test]
    fn inline_reported_rect_drives_gcr_direct_fill() {
        let doc = zero_dom::parse_html(r#"<body><div><span id="tgt">更多</span></div></body>"#);
        let body = doc.get_elements_by_tag_name("body")[0];
        let span = doc.get_element_by_id("tgt").expect("target span");
        let mut root = LayoutBox {
            node_id: Some(body),
            width: 800.0,
            height: 600.0,
            ..LayoutBox::default()
        };
        root.children.push(LayoutBox {
            node_id: doc.get_elements_by_tag_name("div").first().copied(),
            x: 0.0,
            y: 10.0,
            width: 800.0,
            height: 23.0,
            children: vec![LayoutBox {
                // 布局树行盒几何（y=0 行盒顶、h=23 行高）+ 上报记录（content area）。
                inline_reported_rect: Some((5.0, 15.132)),
                node_id: Some(span),
                x: 0.0,
                y: 0.0,
                width: 26.0,
                height: 23.0,
                ..LayoutBox::default()
            }],
            ..LayoutBox::default()
        });
        let cache = HitTestCache::from_document(&doc, &root, &HashMap::new());
        let snapshot = crate::rect_bridge::new_layout_rect_snapshot();
        cache.fill_layout_rect_snapshot(&snapshot);
        let rects = snapshot.lock().expect("rect snapshot");
        assert_eq!(
            rects[&node_id_to_u64(span)],
            (0.0, 15.0, 26.0, 15.132),
            "gBCR 直填路径上报 content area（div y=10 + 上报 y=5）"
        );
    }

    /// slice15（R4384）：命中面 = inline 上报 border box（`inline_reported_rect`，与
    /// gBCR 面同源）——快照路径（from_snapshot，多进程主进程点击/elementFromPoint 消费）
    /// 与文档路径（from_document，渲染进程内部命中）对同一几何输入命中结果一致；
    /// inline 半 leading 空隙带命中**父容器**（div），content 带内命中锚本体。
    /// baidu 导航锚实测形态：行盒 abs y=19/h=23、上报 abs y=24/h=15.132——空隙带
    /// [19,24)（~5px）由 slice13 返修的「命中锚（行盒语义）」翻转为「命中父容器」。
    /// 翻转依据：Chrome 活体实证 inline 命中面 = content area（±padding/border），
    /// gap 带命中包含块（slice13 rework-chrome-gap-pure.json + slice15 逐点差异表
    /// diag/evidence/slice15/s15-before-*.json）；slice13 钉测试
    /// `slice13rw_hit_band_uses_tree_geometry_snapshot_parity_with_document` 按新语义
    /// 处置（gap 带断言翻转，快照字段断言保留），无静默放松。
    /// 快照 `y`/`height` 字段仍为树行盒几何（子盒坐标累积锚），上报值随 `reported`
    /// 字段单独走：跨快照往返后主进程 gBCR（rect 桥方法路径）仍为 content area。
    /// 负控制：`ZW_INLINE_HIT_SURFACE=0`（kill-switch 回退树几何）下 gap 带断言显红。
    /// 装置为手工构造（fixture 直填 reported）——验证消费管道本身，生产值生成由
    /// inline_finalization 单测 + 下方端到端测试覆盖。
    /// https://www.w3.org/TR/CSS22/visudet.html#inline-non-replaced
    /// https://drafts.csswg.org/css-ui-4/#hit-testing
    #[test]
    fn slice15_hit_band_uses_reported_border_box_snapshot_parity_with_document() {
        let doc = zero_dom::parse_html(r#"<html><body><div><a id="nav" href="/more">更多</a></div></body></html>"#);
        let body = doc.get_elements_by_tag_name("body")[0];
        let div = doc.get_elements_by_tag_name("div").first().copied();
        let anchor = doc.get_element_by_id("nav").expect("nav anchor");
        let mut root = LayoutBox {
            node_id: Some(body),
            width: 800.0,
            height: 600.0,
            ..LayoutBox::default()
        };
        root.children.push(LayoutBox {
            node_id: div,
            x: 0.0,
            y: 19.0,
            width: 800.0,
            height: 23.0,
            children: vec![LayoutBox {
                inline_reported_rect: Some((5.0, 15.132)),
                node_id: Some(anchor),
                x: 0.0,
                y: 0.0,
                width: 26.0,
                height: 23.0,
                ..LayoutBox::default()
            }],
            ..LayoutBox::default()
        });
        let live = HitTestCache::from_document(&doc, &root, &HashMap::new());

        // 快照字段：y/h = 行盒树几何（子盒累积锚，不变），上报值随 reported 单独携带。
        let snap = live.snapshot();
        let a_snap = &snap.layout_root.children[0].children[0];
        assert_eq!((a_snap.y, a_snap.height), (0.0, 23.0), "快照 y/h = 行盒树几何");
        assert_eq!(a_snap.reported, Some((5.0, 15.132)), "上报值随 reported 字段携带");

        let restored = HitTestCache::from_snapshot(snap);

        // 空隙带（半 leading，abs y∈[19,24) ∪ [39.13,42)）：不命中锚（命中面 = 上报
        // border box [24,39.13)），from_document 与 from_snapshot 同结果。
        for (x, y, label) in [
            (5.0, 21.0, "上空隙带（行盒顶~content 顶）"),
            (5.0, 40.5, "下空隙带（content 底~行盒底）"),
        ] {
            assert_eq!(
                live.hit_test_link(x, y),
                None,
                "{label}: from_document 不命中锚（gap 带命中父容器）"
            );
            assert_eq!(
                restored.hit_test_link(x, y),
                None,
                "{label}: from_snapshot 不命中锚（与 from_document 一致）"
            );
            let hit = live.hit_test_element(x, y).expect("element hit");
            assert_eq!(
                hit.tag_name, "div",
                "{label}: from_document elementFromPoint 命中父容器"
            );
            assert_eq!(hit.id, None, "{label}: 父容器无 id（非锚）");
            let restored_hit = restored.hit_test_element(x, y).expect("element hit");
            assert_eq!(restored_hit.tag_name, "div", "{label}: from_snapshot 命中父容器");
        }
        // content 带内命中锚（两路径一致）。
        assert_eq!(live.hit_test_link(5.0, 30.0).as_deref(), Some("/more"));
        assert_eq!(restored.hit_test_link(5.0, 30.0).as_deref(), Some("/more"));
        assert_eq!(
            live.hit_test_element(5.0, 30.0).expect("element hit").id.as_deref(),
            Some("nav"),
            "content 带内 elementFromPoint 命中锚"
        );
        assert_eq!(
            restored.hit_test_element(5.0, 30.0).expect("element hit").id.as_deref(),
            Some("nav"),
            "content 带内 from_snapshot elementFromPoint 命中锚"
        );
        // 行盒外不命中锚（负控制）。
        assert_eq!(live.hit_test_link(5.0, 50.0), None);
        assert_eq!(restored.hit_test_link(5.0, 50.0), None);

        // gBCR 面经快照往返保真：主进程方法路径（fill_layout_rect_snapshot）在
        // from_snapshot 物化树上消费 reported——锚 rect = (0, 19+5, 26, 15.132)。
        let rects = crate::rect_bridge::new_layout_rect_snapshot();
        restored.fill_layout_rect_snapshot(&rects);
        let map = rects.lock().expect("rect snapshot");
        assert_eq!(
            map.get(&node_id_to_u64(anchor)),
            Some(&(0.0, 24.0, 26.0, 15.132)),
            "gBCR 面经快照往返仍上报 content area（多进程主进程路径）"
        );
    }

    /// slice15（R4384）：命中面含 padding/border——上报 border box 超出旧树 content
    /// 带的 padding/border 区仍命中锚（Chrome 同：border box 命中域），而行盒 leading
    /// 区（树几何含、上报不含）不再命中锚。装置手填 reported=[2.0,31.0) vs 树
    /// [0,23.0)：y=0.5 ∈ 树 ∉ 上报 → 不命中（slice13 旧行盒语义命中，翻转点）；
    /// y=29.0 ∈ 上报 ∉ 树 → 命中（slice13 树语义不命中，扩展点）。
    /// 负控制：`ZW_INLINE_HIT_SURFACE=0` 下两断言均显红（回退树几何）。
    /// https://drafts.csswg.org/css-ui-4/#hit-testing
    #[test]
    fn slice15_hit_surface_extends_to_reported_padding_border() {
        let doc = zero_dom::parse_html(r#"<html><body><div><a id="nav" href="/more">更多</a></div></body></html>"#);
        let body = doc.get_elements_by_tag_name("body")[0];
        let anchor = doc.get_element_by_id("nav").expect("nav anchor");
        let mut root = LayoutBox {
            node_id: Some(body),
            width: 800.0,
            height: 600.0,
            ..LayoutBox::default()
        };
        root.children.push(LayoutBox {
            node_id: doc.get_elements_by_tag_name("div").first().copied(),
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 40.0,
            children: vec![LayoutBox {
                inline_reported_rect: Some((2.0, 29.0)),
                node_id: Some(anchor),
                x: 0.0,
                y: 0.0,
                width: 46.0,
                height: 23.0,
                ..LayoutBox::default()
            }],
            ..LayoutBox::default()
        });
        let live = HitTestCache::from_document(&doc, &root, &HashMap::new());
        let restored = HitTestCache::from_snapshot(live.snapshot());

        // leading 区（树含、上报不含）不命中锚——slice13 行盒语义翻转点。
        assert_eq!(
            live.hit_test_link(5.0, 0.5),
            None,
            "行盒 leading 区不命中锚（上报面外）"
        );
        assert_eq!(restored.hit_test_link(5.0, 0.5), None);
        // padding/border 区（上报含、树 content 不含）命中锚——border box 命中域。
        assert_eq!(
            live.hit_test_link(5.0, 29.0).as_deref(),
            Some("/more"),
            "上报 padding/border 区命中锚"
        );
        assert_eq!(restored.hit_test_link(5.0, 29.0).as_deref(), Some("/more"));
        // 上报面外不命中。
        assert_eq!(live.hit_test_link(5.0, 31.0), None);
        assert_eq!(restored.hit_test_link(5.0, 31.0), None);
    }

    /// 从布局树定位锚盒几何（相对父内容区 + 绝对，与命中遍历同坐标累积）。
    /// slice15 端到端与突变重排用例共用，提升为测试模块级 helper。
    fn find_abs(
        layout: &LayoutBox,
        id: NodeId,
        ax: f32,
        ay: f32,
    ) -> Option<(f32, f32, f32, f32, f32, Option<(f32, f32)>)> {
        let bx = ax + layout.x;
        let by = ay + layout.y;
        if layout.node_id == Some(id) {
            return Some((
                layout.y,
                bx,
                by,
                layout.width,
                layout.height,
                layout.inline_reported_rect,
            ));
        }
        let (cx, cy) = child_origin(layout, bx, by);
        for child in &layout.children {
            if let Some(found) = find_abs(child, id, cx, cy) {
                return Some(found);
            }
        }
        None
    }

    /// slice15 端到端（真实管线，R4384）：同一渲染产物上命中面 = gBCR 面（同源，
    /// 均消费 `inline_reported_rect`）——13px/23px inline 锚 gBCR h≈15.13，空隙带
    ///（半 leading）不再命中锚、命中父容器（div，无 href → link None）。
    /// slice13 返修钉「空隙带命中锚」按新语义处置（断言翻转 Some→None），依据：
    /// Chrome 活体 gap 带命中包含块（slice13 rework-chrome-baidu-gapband.json 与
    /// slice15 逐点差异表）。负控制（双向）：`ZW_INLINE_HIT_SURFACE=0`（命中面
    /// kill-switch 回退树几何）下 gap 带断言显红（命中回 `/more`，slice13 语义）；
    /// `ZW_INLINE_CONTENT_AREA=0`（layout 侧上报缺席）下本测红于前置断言
    /// `reported.expect`（生产侧回退行盒 h=23，差 7.87px，容差 0.5 → 裕度 15.7×）。
    /// https://www.w3.org/TR/CSS22/visudet.html#inline-non-replaced
    /// https://drafts.csswg.org/css-ui-4/#hit-testing
    #[test]
    fn slice15_reported_hit_surface_matches_gcr_end_to_end() {
        let html = r#"<html><body>
            <div style="line-height: 23px;"><a id="nav" href="/more" style="font-size: 13px;">更多</a></div>
        </body></html>"#;
        let (doc, layout, _styles) = render_with_styles(html, "");
        let anchor = doc.get_element_by_id("nav").expect("nav anchor");
        let (rel_y, abs_x, abs_y, abs_w, abs_h, reported) =
            find_abs(&layout.root, anchor, 0.0, 0.0).expect("anchor box in layout tree");
        // 上报前提：单行 inline 已记录 content area（dormant 常数臂，kill-switch 默认开），
        // 且上报顶在树行盒顶之下（空隙带存在）。
        let (rep_y, rep_h) = reported.expect("单行 inline 应记录上报矩形");
        let gap = rep_y - rel_y;
        assert!(gap > 2.5, "用例前提：半 leading 空隙带应存在（gap={}）", gap);
        assert!(
            (rep_h - 13.0 * (0.928 + 0.236)).abs() < 0.5,
            "用例前提：上报 h 应为 content area 15.132，实际 {}",
            rep_h
        );
        assert!((abs_h - 23.0).abs() < 0.5, "用例前提：树行盒高 23，实际 {}", abs_h);

        let live = HitTestCache::from_document(&doc, &layout.root, &HashMap::new());
        let restored = HitTestCache::from_snapshot(live.snapshot());
        let gap_y = abs_y + gap * 0.5;
        let content_y = abs_y + gap + 2.0;
        for (cache, label) in [(&live, "from_document"), (&restored, "from_snapshot")] {
            assert_eq!(
                cache.hit_test_link(abs_x + 5.0, gap_y),
                None,
                "{label}: 空隙带（行盒顶+{:.1}px）命中父容器，不命中锚",
                gap * 0.5
            );
            assert_eq!(
                cache.hit_test_link(abs_x + 5.0, content_y).as_deref(),
                Some("/more"),
                "{label}: content 带内应命中锚"
            );
            assert_eq!(
                cache.hit_test_link(abs_x + 5.0, abs_y + abs_h + 2.0),
                None,
                "{label}: 行盒外不应命中锚"
            );
        }
        // gap 带 elementFromPoint = 父容器（Chrome 同：包含块）。
        let gap_hit = live.hit_test_element(abs_x + 5.0, gap_y).expect("gap band hit");
        assert_eq!(gap_hit.tag_name, "div", "gap 带 elementFromPoint 命中包含块 div");

        // gBCR 面：rect 表 = content area（abs = 树行盒顶 + 空隙带高）。
        let rects = crate::rect_bridge::new_layout_rect_snapshot();
        live.fill_layout_rect_snapshot(&rects);
        let map = rects.lock().expect("rect snapshot");
        let rect = map.get(&node_id_to_u64(anchor)).copied().expect("anchor rect");
        assert_eq!(rect.2, abs_w, "gBCR 宽 = 布局宽");
        assert!(
            (rect.1 - (abs_y + gap)).abs() < 0.5 && (rect.3 - rep_h).abs() < 0.5,
            "gBCR y/h = content area（y={}, h={}；树行盒顶 {}）",
            rect.1,
            rect.3,
            abs_y
        );
    }

    /// slice15 × slice14（R4384）：textContent 突变 → 重排后命中面仍 = 上报 border box。
    /// 突变触发的重排若让命中面回落行盒树几何，gap 带会重新命中锚（静默回归；活体
    /// baidu 已证不再命中：diag/evidence/slice15/s15-after-baidu-gap.json），此处补
    /// 常驻钉。管线与 [`render_with_styles`] 相同（renderer 实际突变重排路径）。
    /// https://www.w3.org/TR/CSS22/visudet.html#inline-non-replaced
    /// https://drafts.csswg.org/css-ui-4/#hit-testing
    #[test]
    fn slice15_hit_surface_stays_reported_after_textcontent_mutation() {
        let html = r#"<html><body>
            <div style="line-height: 23px;"><a id="nav" href="/more" style="font-size: 13px;">更多</a></div>
        </body></html>"#;
        let mut doc = zero_dom::parse_html(html);
        let anchor = doc.get_element_by_id("nav").expect("nav anchor");
        doc.set_text_content(anchor, "热榜");

        let stylesheets = vec![Parser::parse_stylesheet("")];
        let mut style_system = StyleSystem::new();
        style_system.set_viewport(800.0, 600.0);
        let styles = style_system.compute_styles(&doc, &stylesheets);
        let mut layout_engine = LayoutEngine::new(800.0, 600.0);
        let layout = layout_engine.compute(&doc, &styles);

        let (rel_y, abs_x, abs_y, _abs_w, abs_h, reported) =
            find_abs(&layout.root, anchor, 0.0, 0.0).expect("anchor box in relayout tree");
        // 突变重排后上报仍要生产：单行 inline 记录 content area（gap 带存在才可判别）。
        let (rep_y, rep_h) = reported.expect("突变重排后单行 inline 仍应记录上报矩形");
        let gap = rep_y - rel_y;
        assert!(gap > 2.5, "用例前提：突变后半 leading 空隙带应存在（gap={}）", gap);
        assert!(
            (rep_h - 13.0 * (0.928 + 0.236)).abs() < 0.5,
            "用例前提：上报 h 应为 content area 15.132，实际 {}",
            rep_h
        );
        assert!((abs_h - 23.0).abs() < 0.5, "用例前提：树行盒高 23，实际 {}", abs_h);

        let cache = HitTestCache::from_document(&doc, &layout.root, &HashMap::new());
        let gap_y = abs_y + gap * 0.5;
        assert_eq!(
            cache.hit_test_link(abs_x + 5.0, gap_y),
            None,
            "突变重排后 gap 带不命中锚（链接动作 None）"
        );
        let gap_hit = cache.hit_test_element(abs_x + 5.0, gap_y).expect("gap band hit");
        assert_eq!(
            gap_hit.tag_name, "div",
            "突变重排后 gap 带 elementFromPoint 命中父容器 div"
        );
        assert_eq!(
            cache.hit_test_link(abs_x + 5.0, abs_y + gap + 2.0).as_deref(),
            Some("/more"),
            "突变重排后 content 带仍命中锚"
        );
    }

    // ── 基础命中测试 ──

    /// 测试点击链接元素返回 href。
    #[test]
    fn hit_test_finds_anchor_href() {
        let html = r#"<html><body>
            <a href="https://example.com" style="display: block; width: 200px; height: 40px; padding: 10px;">
                Example
            </a>
        </body></html>"#;
        let (doc, layout) = render(html, "a { background-color: #eeeeee; }");
        let href = hit_test_link(&doc, &layout.root, &HashMap::new(), 50.0, 20.0);
        assert_eq!(href.as_deref(), Some("https://example.com"));
    }

    /// 测试点击视口外返回 None。
    #[test]
    fn hit_test_outside_viewport() {
        let html = r#"<html><body>
            <a href="https://example.com" style="display: block; width: 200px; height: 40px;">
                Link
            </a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, &HashMap::new(), 900.0, 20.0).is_none());
    }

    /// 测试点击非链接元素返回 None。
    #[test]
    fn hit_test_non_link_element() {
        let html = r#"<html><body>
            <div style="display: block; width: 200px; height: 40px;">Not a link</div>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, &HashMap::new(), 50.0, 20.0).is_none());
    }

    // ── 嵌套链接测试 ──

    /// 测试点击嵌套在 div 内的链接能正确找到 href。
    #[test]
    fn hit_test_nested_link_in_div() {
        let html = r#"<html><body>
            <div style="display: block; width: 300px; height: 100px; padding: 20px;">
                <a href="/page" style="display: block; width: 100px; height: 30px;">Link</a>
            </div>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        let href = hit_test_link(&doc, &layout.root, &HashMap::new(), 30.0, 30.0);
        assert_eq!(href.as_deref(), Some("/page"));
    }

    /// 测试深层嵌套链接（div > p > a）能正确命中。
    #[test]
    fn hit_test_deeply_nested_link() {
        let html = r#"<html><body>
            <div style="width: 400px; height: 200px;">
                <p style="width: 300px; height: 100px;">
                    <a href="https://deep.example.com" style="display: block; width: 200px; height: 40px;">Deep Link</a>
                </p>
            </div>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        let href = hit_test_link(&doc, &layout.root, &HashMap::new(), 20.0, 20.0);
        assert!(href.is_some(), "深层嵌套链接应能被命中");
    }

    // ── 多链接测试 ──

    /// 测试页面中有多个链接时点击不同位置命中不同链接。
    #[test]
    fn hit_test_multiple_links() {
        let html = r#"<html><body>
            <a href="/first" style="display: block; width: 200px; height: 30px;">First</a>
            <a href="/second" style="display: block; width: 200px; height: 30px;">Second</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");

        let href1 = hit_test_link(&doc, &layout.root, &HashMap::new(), 50.0, 10.0);
        assert_eq!(href1.as_deref(), Some("/first"));

        let href2 = hit_test_link(&doc, &layout.root, &HashMap::new(), 50.0, 40.0);
        assert_eq!(href2.as_deref(), Some("/second"));
    }

    // ── 边界条件 ──

    /// 测试空 href 的链接不应被返回。
    #[test]
    fn hit_test_empty_href_ignored() {
        let html = r#"<html><body>
            <a href="" style="display: block; width: 200px; height: 40px;">Empty</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, &HashMap::new(), 50.0, 20.0).is_none());
    }

    /// 测试 href="#" 的链接不应被返回。
    #[test]
    fn hit_test_hash_href_ignored() {
        let html = r##"<html><body>
            <a href="#" style="display: block; width: 200px; height: 40px;">Hash</a>
        </body></html>"##;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, &HashMap::new(), 50.0, 20.0).is_none());
    }

    /// 测试 href 只含空格的链接不应被返回。
    #[test]
    fn hit_test_whitespace_only_href_ignored() {
        let html = r#"<html><body>
            <a href="  " style="display: block; width: 200px; height: 40px;">Whitespace</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, &HashMap::new(), 50.0, 20.0).is_none());
    }

    /// 测试点击元素边界（恰好包含）和边界外（恰好不包含）。
    /// 注意：body 有 UA 默认 margin:8px，因此 <a> 元素从约 (8,8) 开始。
    #[test]
    fn hit_test_exact_boundary() {
        let html = r#"<html><body>
            <a href="/edge" style="display: block; width: 100px; height: 50px;">Edge</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");

        // 元素内部（包含左上角，含 body 8px margin 偏移）
        assert!(hit_test_link(&doc, &layout.root, &HashMap::new(), 8.0, 8.0).is_some());

        // 元素内部（接近右下角但不超出）
        let near_edge = hit_test_link(&doc, &layout.root, &HashMap::new(), 107.0, 57.0);
        assert!(near_edge.is_some());

        // 元素外部（body margin 区域，不应命中链接）
        assert!(hit_test_link(&doc, &layout.root, &HashMap::new(), 0.0, 0.0).is_none());
    }

    /// 测试链接文本包含子元素（如 span）时命中测试仍正确。
    #[test]
    fn hit_test_link_with_inline_children() {
        let html = r#"<html><body>
            <a href="/with-span" style="display: block; width: 200px; height: 40px;">
                <span>Link Text</span>
            </a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        let href = hit_test_link(&doc, &layout.root, &HashMap::new(), 50.0, 20.0);
        assert_eq!(href.as_deref(), Some("/with-span"));
    }

    /// 测试绝对定位元素的命中测试。
    #[test]
    fn hit_test_absolute_positioned_link() {
        let html = r#"<html><body style="margin: 0;">
            <div style="position: relative; width: 400px; height: 300px;">
                <a href="/abs" style="position: absolute; top: 50px; left: 100px; width: 150px; height: 30px;">Abs</a>
            </div>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        let href = hit_test_link(&doc, &layout.root, &HashMap::new(), 120.0, 60.0);
        assert_eq!(href.as_deref(), Some("/abs"));
    }

    /// 测试点击空白区域（无任何元素）返回 None。
    #[test]
    fn hit_test_empty_body() {
        let html = "<html><body></body></html>";
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, &HashMap::new(), 100.0, 100.0).is_none());
    }

    // ── deepest_node_at 直接测试 ──

    /// 测试 deepest_node_at 选择更深的节点。
    #[test]
    fn test_deepest_node_prefers_deeper() {
        let html = r#"<html><body>
            <div style="width: 200px; height: 100px;">
                <div style="width: 100px; height: 50px;">
                    <span style="display: block; width: 50px; height: 20px;">Inner</span>
                </div>
            </div>
        </body></html>"#;
        let (doc, layout) = render(html, "");

        // 点击内部 span 的位置
        let styles = HashMap::new();
        let walk = HitWalk {
            point_x: 10.0,
            point_y: 10.0,
            is_hidden: &|n| is_hidden_style(&styles, n),
            is_pe_none: &|n| is_pe_none_style(&styles, n),
        };
        let mut best = (0usize, doc.root());
        deepest_node_at(&layout.root, 0.0, 0.0, 0, &mut best, &walk);
        // 应该找到一个节点（不一定是 span，取决于布局结果，但深度 > 0）
        assert!(best.0 > 0, "应命中嵌套元素，深度 > 0");
    }

    /// 测试负坐标不命中任何元素。
    #[test]
    fn test_negative_coordinates_miss() {
        let html = r#"<html><body>
            <a href="/test" style="display: block; width: 200px; height: 40px;">Link</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, &HashMap::new(), -10.0, -10.0).is_none());
    }

    /// 元素命中测试返回标签与属性。
    #[test]
    fn hit_test_element_returns_div_attributes() {
        let html =
            r#"<html><body><div id="main" class="box" style="width:100px;height:40px">Hello</div></body></html>"#;
        let css = "div { display: block; }";
        let (doc, layout) = render(html, css);
        let hit = hit_test_element(&doc, &layout.root, &HashMap::new(), 10.0, 10.0).expect("element");
        assert_eq!(hit.tag_name, "div");
        assert_eq!(hit.id.as_deref(), Some("main"));
        assert_eq!(hit.class_name.as_deref(), Some("box"));
        let node = crate::find_by_selector(&doc, "#main").expect("node");
        assert_eq!(hit.node_handle, node_id_to_u64(node));
    }

    /// Form controls must remain their own hit-test targets even when wrapped by a label.
    /// https://html.spec.whatwg.org/multipage/forms.html#the-label-element
    #[test]
    fn hit_test_element_returns_wrapped_text_input() {
        let html = r#"<html><body style="margin:0"><label>Name <input id="name" style="display:block;width:160px;height:32px"></label></body></html>"#;
        let (doc, layout) = render(html, "");
        let hit = hit_test_element(&doc, &layout.root, &HashMap::new(), 10.0, 25.0).expect("input element");
        assert_eq!(hit.tag_name, "input");
        assert_eq!(hit.id.as_deref(), Some("name"));
    }

    #[test]
    fn hit_test_link_with_query_and_fragment() {
        let html = r#"<html><body>
            <a href="/page?foo=bar#section" style="display: block; width: 200px; height: 40px;">Link</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        let href = hit_test_link(&doc, &layout.root, &HashMap::new(), 50.0, 20.0);
        assert_eq!(href.as_deref(), Some("/page?foo=bar#section"));
    }

    /// 辅助函数：render + 保留 computed style 图（visibility 用例需要）。
    fn render_with_styles(
        html: &str,
        css: &str,
    ) -> (
        Document,
        zero_layout_engine::LayoutResult,
        HashMap<NodeId, ComputedStyle>,
    ) {
        let doc = zero_dom::parse_html(html);
        let stylesheets = vec![Parser::parse_stylesheet(css)];
        let mut style_system = StyleSystem::new();
        style_system.set_viewport(800.0, 600.0);
        let styles = style_system.compute_styles(&doc, &stylesheets);
        let mut layout_engine = LayoutEngine::new(800.0, 600.0);
        let layout = layout_engine.compute(&doc, &styles);
        (doc, layout, styles)
    }

    /// CSS Visibility：hidden 覆盖盒不参与命中，点击穿透到下层可见链接（free-fn 路径，
    /// renderer 真实输入 + elementFromPoint 消费）。复现 html5test.com 落地页：
    /// 后绘制 hidden 公告 P 覆盖可见链接，深度打平时（修复前）后绘制者赢 → 命中 P。
    /// https://drafts.csswg.org/css-visibility/#visibility
    #[test]
    fn hit_passes_through_hidden_overlay_to_visible_link() {
        let html = r#"<html><body>
            <a id="link" href="/p2" style="position:absolute; left:10px; top:10px; width:100px; height:40px;">go</a>
            <p id="cover" style="position:absolute; left:0px; top:0px; width:200px; height:100px; visibility:hidden;">notice</p>
        </body></html>"#;
        let (doc, layout, styles) = render_with_styles(html, "");
        let hit = hit_test_element(&doc, &layout.root, &styles, 50.0, 30.0).expect("hit link");
        assert_eq!(hit.id.as_deref(), Some("link"), "hidden 覆盖盒应被跳过，命中链接");
        assert_eq!(
            hit_test_link(&doc, &layout.root, &styles, 50.0, 30.0).as_deref(),
            Some("/p2"),
            "hidden 覆盖下链接默认动作可达"
        );
    }

    /// 缓存路径（elementFromPoint/elementsFromPoint 消费）：from_document 构建期记录
    /// hidden 集合，缓存查询同样穿透；hidden 集合随 snapshot/from_snapshot 跨进程一致。
    #[test]
    fn cache_hit_test_skips_hidden_overlay() {
        let html = r#"<html><body>
            <a id="link" href="/p2" style="position:absolute; left:10px; top:10px; width:100px; height:40px;">go</a>
            <p id="cover" style="position:absolute; left:0px; top:0px; width:200px; height:100px; visibility:hidden;">notice</p>
        </body></html>"#;
        let (doc, layout, styles) = render_with_styles(html, "");
        let cache = HitTestCache::from_document(&doc, &layout.root, &styles);
        let hit = cache.hit_test_element(50.0, 30.0).expect("hit link");
        assert_eq!(hit.id.as_deref(), Some("link"));
        let stack = cache.elements_at_point(50.0, 30.0);
        assert!(!stack.is_empty());
        assert_eq!(stack[0].id.as_deref(), Some("link"), "elementsAtPoint 首元素应为链接");
        // 跨进程快照往返后语义一致（browser tab_js_worker 消费此形态）。
        let restored = HitTestCache::from_snapshot(cache.snapshot());
        assert_eq!(
            restored.hit_test_element(50.0, 30.0).expect("hit link").id.as_deref(),
            Some("link")
        );
    }

    /// hidden 祖先 + 显式 `visibility: visible` 后代：后代仍可命中（CSS 允许 hidden
    /// 祖先下 visible 后代可见绘制）；且布局保留——hidden 元素 gBCR rect 不丢失。
    #[test]
    fn visible_descendant_of_hidden_ancestor_still_hit() {
        let html = r#"<html><body>
            <div id="panel" style="position:absolute; left:0px; top:0px; width:200px; height:100px; visibility:hidden;">
                <a id="link" href="/p2" style="position:absolute; left:10px; top:10px; width:100px; height:40px; visibility:visible;">go</a>
            </div>
        </body></html>"#;
        let (doc, layout, styles) = render_with_styles(html, "");
        let hit = hit_test_element(&doc, &layout.root, &styles, 50.0, 30.0).expect("hit link");
        assert_eq!(hit.id.as_deref(), Some("link"), "显式 visible 后代仍可命中");
        // 布局保留：hidden 祖先的 rect 仍可查（gBCR 消费 fill_layout_rect_snapshot）。
        let cache = HitTestCache::from_document(&doc, &layout.root, &styles);
        let rects = crate::rect_bridge::new_layout_rect_snapshot();
        cache.fill_layout_rect_snapshot(&rects);
        let panel = doc.get_element_by_id("panel").expect("panel");
        assert!(
            rects.lock().unwrap().contains_key(&node_id_to_u64(panel)),
            "hidden 元素布局 rect 不应丢失（gBCR 保真）"
        );
    }

    /// `visibility: collapse` 与 hidden 同义处理（与绘制侧 painter 谓词一致）。
    #[test]
    fn visibility_collapse_skipped_like_hidden() {
        let html = r#"<html><body>
            <a id="link" href="/p2" style="position:absolute; left:10px; top:10px; width:100px; height:40px;">go</a>
            <p id="cover" style="position:absolute; left:0px; top:0px; width:200px; height:100px; visibility:collapse;">notice</p>
        </body></html>"#;
        let (doc, layout, styles) = render_with_styles(html, "");
        let hit = hit_test_element(&doc, &layout.root, &styles, 50.0, 30.0).expect("hit link");
        assert_eq!(hit.id.as_deref(), Some("link"));
    }

    /// 链接自身 hidden（无可见后代）时不可点击：hit_test_link 返 None（可见内容缺失，
    /// 无可激活目标）。
    #[test]
    fn hidden_link_itself_not_clickable() {
        let html = r#"<html><body>
            <a id="link" href="/p2" style="position:absolute; left:10px; top:10px; width:100px; height:40px; visibility:hidden;">go</a>
        </body></html>"#;
        let (doc, layout, styles) = render_with_styles(html, "");
        assert!(hit_test_link(&doc, &layout.root, &styles, 50.0, 30.0).is_none());
    }

    /// `pointer-events: none` 遮罩盖住 input：命中穿透到 input（bilibili 轮播遮罩
    /// 盖搜索框实站场景最小化——遮罩 absolute+pe:none 盖导航区，点击须路由到
    /// input 而非遮罩）。live 与缓存路径 + 跨进程快照往返语义一致。
    /// https://drafts.csswg.org/css-ui-4/#pointer-events
    #[test]
    fn pe_none_overlay_penetrates_to_input() {
        let html = r#"<html><body>
            <input id="q" style="position:absolute; left:10px; top:10px; width:160px; height:24px;">
            <div id="mask" style="position:absolute; left:0px; top:0px; width:200px; height:100px; pointer-events:none;">mask</div>
        </body></html>"#;
        let (doc, layout, styles) = render_with_styles(html, "");
        let hit = hit_test_element(&doc, &layout.root, &styles, 80.0, 20.0).expect("hit input");
        assert_eq!(hit.id.as_deref(), Some("q"), "pe:none 遮罩下应命中 input");
        let cache = HitTestCache::from_document(&doc, &layout.root, &styles);
        assert_eq!(
            cache.hit_test_element(80.0, 20.0).expect("hit input").id.as_deref(),
            Some("q"),
            "缓存路径与 live 路径一致"
        );
        let stack = cache.elements_at_point(80.0, 20.0);
        assert_eq!(
            stack.first().and_then(|h| h.id.as_deref()),
            Some("q"),
            "elementsAtPoint 序列非空且以 input 为首"
        );
        assert!(
            stack.iter().all(|h| h.id.as_deref() != Some("mask")),
            "elementsAtPoint 序列不应含 pe:none 遮罩: {:?}",
            stack.iter().map(|h| h.id.clone()).collect::<Vec<_>>()
        );
        let restored = HitTestCache::from_snapshot(cache.snapshot());
        assert_eq!(
            restored.hit_test_element(80.0, 20.0).expect("hit input").id.as_deref(),
            Some("q"),
            "跨进程快照往返后语义一致"
        );
    }

    /// `pointer-events: none` 元素自身位置命中穿透到下层元素。
    #[test]
    fn pe_none_element_itself_penetrates() {
        let html = r#"<html><body>
            <div id="target" style="position:absolute; left:0px; top:0px; width:200px; height:100px;">t</div>
            <div id="ghost" style="position:absolute; left:0px; top:0px; width:200px; height:100px; pointer-events:none;">ghost</div>
        </body></html>"#;
        let (doc, layout, styles) = render_with_styles(html, "");
        let hit = hit_test_element(&doc, &layout.root, &styles, 100.0, 50.0).expect("hit target");
        assert_eq!(hit.id.as_deref(), Some("target"), "pe:none 层应穿透到下层 target");
    }

    /// pe:none 祖先 + 显式 `pointer-events: auto` 后代：后代仍可命中（与 visibility
    /// hidden/visible 同构：候选资格剥夺仅作用于盒自身，递归不剪枝）。本测试钉住
    /// 「不剪枝」语义（错误实现剪枝子树时 FAIL）；「删除谓词」轴由穿透用例钉住
    ///（本配置下 link 更深，谓词在否都赢）。
    #[test]
    fn pe_auto_descendant_of_pe_none_ancestor_still_hit() {
        let html = r#"<html><body>
            <div id="panel" style="position:absolute; left:0px; top:0px; width:200px; height:100px; pointer-events:none;">
                <a id="link" href="/p2" style="position:absolute; left:10px; top:10px; width:100px; height:40px; pointer-events:auto;">go</a>
            </div>
        </body></html>"#;
        let (doc, layout, styles) = render_with_styles(html, "");
        let hit = hit_test_element(&doc, &layout.root, &styles, 50.0, 30.0).expect("hit link");
        assert_eq!(hit.id.as_deref(), Some("link"), "显式 auto 后代仍可命中");
    }
}
