//! 布局树命中测试 — 用于链接点击等交互。

use std::collections::{HashMap, HashSet};

use slotmap::{Key, KeyData};
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::LayoutBox;

/// 主线程只读命中测试快照（由 tab worker 在推送快照时构建）。
#[derive(Debug, Clone)]
pub struct HitTestCache {
    layout_root: LayoutBox,
    doc_root: NodeId,
    nodes: HashMap<NodeId, HitTestNodeMeta>,
    parents: HashMap<NodeId, NodeId>,
}

#[derive(Debug, Clone)]
struct HitTestNodeMeta {
    tag_name: String,
    id: Option<String>,
    class_name: Option<String>,
    href: Option<String>,
    /// 图片 `src`（仅 `img` 元素，绝对化后存储）。
    src: Option<String>,
}

impl HitTestCache {
    /// 从管线缓存的 DOM 与布局树构建命中测试快照。
    pub fn from_document(doc: &Document, layout_root: &LayoutBox) -> Self {
        let mut nodes = HashMap::new();
        let mut parents = HashMap::new();
        collect_hit_test_nodes(layout_root, doc, &mut nodes, &mut parents);
        Self {
            layout_root: layout_root.clone(),
            doc_root: doc.root(),
            nodes,
            parents,
        }
    }

    /// 命中测试链接，返回 `href`（若存在）。
    pub fn hit_test_link(&self, x: f32, y: f32) -> Option<String> {
        let mut best = (0, self.doc_root);
        deepest_node_at(&self.layout_root, 0.0, 0.0, x, y, 0, &mut best);
        find_link_href_cached(best.1, &self.nodes, &self.parents)
    }

    /// 命中测试图片，返回 `src`（若点中 img 或其子元素）。
    pub fn hit_test_image(&self, x: f32, y: f32) -> Option<String> {
        let mut best = (0, self.doc_root);
        deepest_node_at(&self.layout_root, 0.0, 0.0, x, y, 0, &mut best);
        find_image_src_cached(best.1, &self.nodes, &self.parents)
    }

    /// 命中测试元素，返回最深元素及其布局盒。
    pub fn hit_test_element(&self, x: f32, y: f32) -> Option<ElementHit> {
        let mut best = (0, self.doc_root);
        deepest_node_at(&self.layout_root, 0.0, 0.0, x, y, 0, &mut best);
        element_hit_from_cache(&self.layout_root, best.1, &self.nodes, &self.parents)
    }

    /// 命中测试：返回 `(x,y)` 处所有元素，按绘制序（最前/最深在前 → 最后/最浅在后）。
    ///
    /// 收集所有包含该点的盒（[`collect_nodes_at`]），按深度降序（深度≈绘制层级，最深元素绘制
    /// 在最前），每盒经 [`nearest_element_cached`] 取其元素并去重（同元素多盒仅保留最深=最前那次）。
    /// [`HitTestCache::hit_test_element`]（=`elementFromPoint`）即本序列的首元素。z-index/绝对定位
    /// 的精确绘制序未建模（树深近似，见 elementFromPoint 已知限制）。
    pub fn elements_at_point(&self, x: f32, y: f32) -> Vec<ElementHit> {
        let mut hits: Vec<(usize, NodeId)> = Vec::new();
        collect_nodes_at(&self.layout_root, 0.0, 0.0, x, y, 0, &mut hits);
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
                            href: meta.href.clone(),
                            src: meta.src.clone(),
                        },
                    )
                })
                .collect(),
            parents: self.parents.iter().map(|(c, p)| (*c, *p)).collect(),
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
                            href: meta.href,
                            src: meta.src,
                        },
                    )
                })
                .collect(),
            parents: snap.parents.into_iter().collect(),
        }
    }

    /// P1a gBCR：把布局树每节点 rect（相对父内容区）写入共享 rect snapshot。
    /// 直接遍历内部 `layout_root`（避免 [`Self::snapshot`] 的整树 clone）。render 后调；
    /// 无 `node_id` 的匿名/伪盒跳过。js_worker 的 RectBridge handler 经 identity→NodeId 查此 snapshot。
    pub fn fill_layout_rect_snapshot(&self, snapshot: &crate::rect_bridge::LayoutRectSnapshot) {
        if let Ok(mut map) = snapshot.lock() {
            map.clear();
            fill_rect_from_layout_box(&self.layout_root, &mut map);
        }
    }
}

/// `LayoutBox` 递归填充 rect snapshot（`fill_layout_rect_snapshot` 的内部实现，直接走 LayoutBox 避 clone）。
fn fill_rect_from_layout_box(box_node: &LayoutBox, map: &mut HashMap<u64, crate::rect_bridge::Rect4>) {
    if let Some(id) = box_node.node_id {
        map.insert(
            node_id_to_u64(id),
            (box_node.x, box_node.y, box_node.width, box_node.height),
        );
    }
    for child in &box_node.children {
        fill_rect_from_layout_box(child, map);
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
}

fn layout_snapshot_from_box(layout: &LayoutBox) -> HitTestLayoutSnapshot {
    HitTestLayoutSnapshot {
        node_id: layout.node_id,
        x: layout.x,
        y: layout.y,
        width: layout.width,
        height: layout.height,
        children: layout.children.iter().map(layout_snapshot_from_box).collect(),
    }
}

fn layout_box_from_snapshot(snapshot: &HitTestLayoutSnapshot) -> LayoutBox {
    LayoutBox {
        node_id: snapshot.node_id,
        x: snapshot.x,
        y: snapshot.y,
        width: snapshot.width,
        height: snapshot.height,
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
        tag_name: meta.tag_name.clone(),
        id: meta.id.clone(),
        class_name: meta.class_name.clone(),
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
    point_x: f32,
    point_y: f32,
    depth: usize,
    best: &mut (usize, NodeId),
) {
    let box_x = abs_x + layout.x;
    let box_y = abs_y + layout.y;

    if point_x < box_x || point_y < box_y || point_x >= box_x + layout.width || point_y >= box_y + layout.height {
        return;
    }

    if let Some(node_id) = layout.node_id
        && depth >= best.0
    {
        *best = (depth, node_id);
    }

    for child in &layout.children {
        deepest_node_at(child, box_x, box_y, point_x, point_y, depth + 1, best);
    }
}

/// 收集所有包含 `(point_x, point_y)` 的盒节点（含深度），供 [`HitTestCache::elements_at_point`]。
/// 镜像 [`deepest_node_at`] 的包含判定与坐标累积（同 `LayoutBox` 坐标相对父内容区须累积），
/// 但收集全部命中盒而非仅最深。点不在盒内则不递归（与 `deepest_node_at` 一致）。
fn collect_nodes_at(
    layout: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
    point_x: f32,
    point_y: f32,
    depth: usize,
    out: &mut Vec<(usize, NodeId)>,
) {
    let box_x = abs_x + layout.x;
    let box_y = abs_y + layout.y;

    if point_x < box_x || point_y < box_y || point_x >= box_x + layout.width || point_y >= box_y + layout.height {
        return;
    }

    if let Some(node_id) = layout.node_id {
        out.push((depth, node_id));
    }

    for child in &layout.children {
        collect_nodes_at(child, box_x, box_y, point_x, point_y, depth + 1, out);
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
    /// 元素标签名（小写）。
    pub tag_name: String,
    /// `id` 属性。
    pub id: Option<String>,
    /// `class` 属性。
    pub class_name: Option<String>,
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
    for child in &layout.children {
        if let Some(found) = layout_box_for_node(child, target, box_x, box_y) {
            return Some(found);
        }
    }
    None
}

fn element_hit_from_node(doc: &Document, layout: &LayoutBox, node: NodeId) -> Option<ElementHit> {
    let element = nearest_element_node(doc, node);
    let data = doc.get(element)?;
    let NodeKind::Element(elem) = &data.kind else {
        return None;
    };
    let (x, y, width, height) = layout_box_for_node(layout, element, 0.0, 0.0)?;
    Some(ElementHit {
        tag_name: elem.local_name().to_ascii_lowercase(),
        id: doc.get_attribute(element, "id"),
        class_name: doc.get_attribute(element, "class"),
        x,
        y,
        width,
        height,
    })
}

/// 在文档布局中命中测试链接，返回 `href`（若存在）。
pub fn hit_test_link(doc: &Document, layout: &LayoutBox, x: f32, y: f32) -> Option<String> {
    let mut best = (0, doc.root());
    deepest_node_at(layout, 0.0, 0.0, x, y, 0, &mut best);
    find_link_href(doc, best.1)
}

/// 在文档布局中命中测试图片，返回 `src`（文档原始值，未绝对化）。
pub fn hit_test_image(doc: &Document, layout: &LayoutBox, x: f32, y: f32) -> Option<String> {
    let mut best = (0, doc.root());
    deepest_node_at(layout, 0.0, 0.0, x, y, 0, &mut best);
    find_image_src(doc, best.1)
}

/// 在文档布局中命中测试元素，返回最深元素及其布局盒。
pub fn hit_test_element(doc: &Document, layout: &LayoutBox, x: f32, y: f32) -> Option<ElementHit> {
    let mut best = (0, doc.root());
    deepest_node_at(layout, 0.0, 0.0, x, y, 0, &mut best);
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
        let href = hit_test_link(&doc, &layout.root, 50.0, 20.0);
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
        assert!(hit_test_link(&doc, &layout.root, 900.0, 20.0).is_none());
    }

    /// 测试点击非链接元素返回 None。
    #[test]
    fn hit_test_non_link_element() {
        let html = r#"<html><body>
            <div style="display: block; width: 200px; height: 40px;">Not a link</div>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, 50.0, 20.0).is_none());
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
        let href = hit_test_link(&doc, &layout.root, 30.0, 30.0);
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
        let href = hit_test_link(&doc, &layout.root, 20.0, 20.0);
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

        let href1 = hit_test_link(&doc, &layout.root, 50.0, 10.0);
        assert_eq!(href1.as_deref(), Some("/first"));

        let href2 = hit_test_link(&doc, &layout.root, 50.0, 40.0);
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
        assert!(hit_test_link(&doc, &layout.root, 50.0, 20.0).is_none());
    }

    /// 测试 href="#" 的链接不应被返回。
    #[test]
    fn hit_test_hash_href_ignored() {
        let html = r##"<html><body>
            <a href="#" style="display: block; width: 200px; height: 40px;">Hash</a>
        </body></html>"##;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, 50.0, 20.0).is_none());
    }

    /// 测试 href 只含空格的链接不应被返回。
    #[test]
    fn hit_test_whitespace_only_href_ignored() {
        let html = r#"<html><body>
            <a href="  " style="display: block; width: 200px; height: 40px;">Whitespace</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, 50.0, 20.0).is_none());
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
        assert!(hit_test_link(&doc, &layout.root, 8.0, 8.0).is_some());

        // 元素内部（接近右下角但不超出）
        let near_edge = hit_test_link(&doc, &layout.root, 107.0, 57.0);
        assert!(near_edge.is_some());

        // 元素外部（body margin 区域，不应命中链接）
        assert!(hit_test_link(&doc, &layout.root, 0.0, 0.0).is_none());
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
        let href = hit_test_link(&doc, &layout.root, 50.0, 20.0);
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
        let href = hit_test_link(&doc, &layout.root, 120.0, 60.0);
        assert_eq!(href.as_deref(), Some("/abs"));
    }

    /// 测试点击空白区域（无任何元素）返回 None。
    #[test]
    fn hit_test_empty_body() {
        let html = "<html><body></body></html>";
        let (doc, layout) = render(html, "");
        assert!(hit_test_link(&doc, &layout.root, 100.0, 100.0).is_none());
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
        let mut best = (0usize, doc.root());
        deepest_node_at(&layout.root, 0.0, 0.0, 10.0, 10.0, 0, &mut best);
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
        assert!(hit_test_link(&doc, &layout.root, -10.0, -10.0).is_none());
    }

    /// 元素命中测试返回标签与属性。
    #[test]
    fn hit_test_element_returns_div_attributes() {
        let html =
            r#"<html><body><div id="main" class="box" style="width:100px;height:40px">Hello</div></body></html>"#;
        let css = "div { display: block; }";
        let (doc, layout) = render(html, css);
        let hit = hit_test_element(&doc, &layout.root, 10.0, 10.0).expect("element");
        assert_eq!(hit.tag_name, "div");
        assert_eq!(hit.id.as_deref(), Some("main"));
        assert_eq!(hit.class_name.as_deref(), Some("box"));
    }

    #[test]
    fn hit_test_link_with_query_and_fragment() {
        let html = r#"<html><body>
            <a href="/page?foo=bar#section" style="display: block; width: 200px; height: 40px;">Link</a>
        </body></html>"#;
        let (doc, layout) = render(html, "");
        let href = hit_test_link(&doc, &layout.root, 50.0, 20.0);
        assert_eq!(href.as_deref(), Some("/page?foo=bar#section"));
    }
}
