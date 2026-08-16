//! 从 DOM 树和计算样式构建 taffy 布局树。
//!
//! 提供将 DOM 元素节点与 taffy 节点关联的功能，
//! 跳过文本节点、注释节点和 display:none 的元素。

use std::collections::{HashMap, HashSet};
use taffy::prelude::*;
use zero_css_parser::values::{DisplayValue, FlexDirectionValue, FloatValue, LengthValue, PositionValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::{ComputedStyle, WritingModeValue};

use crate::converter::{GridAreaMap, computed_style_to_taffy, parse_grid_template_areas};
use crate::inline_block_split::{
    InlineBlockSegment, block_container_has_mixed_content, compute_block_container_split, compute_inline_block_split,
    inline_has_block_child, is_whitespace_only_inline_segment,
};

/// R1311b：判断 `<br>` 元素是否处于「纯 inline 上下文」——br 且无 block-level in-flow
/// 同胞。此类 br 由父容器 IFC 作 InlineItem::Br 处理（inline/mod.rs:1122），不需要独立
/// taffy 节点。display 判定与 R1285 一致。
fn br_is_inline_only(doc: &Document, styles: &HashMap<NodeId, ComputedStyle>, id: NodeId) -> bool {
    let is_br = doc
        .get(id)
        .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("br")));
    if !is_br {
        return false;
    }
    let Some(pid) = doc.parent_node(id) else {
        return true;
    };
    !doc.child_nodes(pid).iter().any(|&s| {
        s != id
            && styles.get(&s).is_some_and(|st| {
                matches!(
                    st.display,
                    DisplayValue::Block
                        | DisplayValue::Flex
                        | DisplayValue::Grid
                        | DisplayValue::Table
                        | DisplayValue::ListItem
                        | DisplayValue::FlowRoot
                )
            })
    })
}

/// R1311b：判断 br 的父块在其容器中是否有「后续 **in-flow** 元素兄弟」——即 br 父块
/// 不是其容器的最后一个 in-flow 子元素。这是 br-as-taffy-node 致父块测 0 高、后续
/// 兄弟错位重叠 bug 的**精确触发条件**（末子 br 父块无后续兄弟可错位）。仅在此时跳过
/// br 的 taffy 节点（让父块成 leaf 由 IFC 测高修正兄弟定位），可避免对末子 br 父块
/// 做 leaf 转换引发容器高度连锁重排（welcome p.tagline 是末子 → 豁免 → welcome 字节
/// 一致）。OOF（abspos/fixed）后续兄弟不算（它们不参与常规流，不被父块 0 高错位）。
fn br_parent_has_following_inflow_sibling(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    br_id: NodeId,
) -> bool {
    let pid = match doc.parent_node(br_id) {
        Some(p) => p,
        None => return false,
    };
    let gpid = match doc.parent_node(pid) {
        Some(gp) => gp,
        None => return false,
    };
    let mut after = false;
    for &s in &doc.child_nodes(gpid) {
        if s == pid {
            after = true;
            continue;
        }
        if !after {
            continue;
        }
        if let Some(node) = doc.get(s)
            && matches!(&node.kind, NodeKind::Element(_))
            && let Some(st) = styles.get(&s)
        {
            if matches!(st.display, DisplayValue::None | DisplayValue::Contents) {
                continue;
            }
            // in-flow = 非 abspos/fixed（OOF 兄弟不被父块 0 高错位，不算触发条件）
            if !matches!(st.position, PositionValue::Absolute | PositionValue::Fixed) {
                return true;
            }
        }
    }
    false
}

/// R109 §9.2.1.1 生产端接线（匿名块生成 + fragment border）默认**启用**——经全量
/// reftest（+2 零回归：inline-box-001 / block-in-inline-align-001）+ 全量 make test
/// 验证。设 `R109_WIRE=0` 可关闭（回退到旧 inline→block 行为，仅用于对比/调试）。
fn r109_wired() -> bool {
    std::env::var("R109_WIRE").ok().as_deref() != Some("0")
}

/// R2160 Phase A slice 2：判定某子节点是否为「childless plain inline」——display:inline +
/// 非 ooflow（abspos/fixed）+ 非 line-break 元素（br/wbr）+ 无 Element 子（仅文本后代）+
/// 子树无 ooflow 后代。此类 inline 在多 inline block 容器中可跳过 taffy 节点
///（orphan → painter R639 part2 per-fragment 绘 bg/border），消除 a/i/b 块级栈列。
/// 子树无 ooflow 守卫 ≡ R2156（保 abspos CB）。
pub(crate) fn phasea_multi_inline_eligible(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    child_id: NodeId,
) -> bool {
    let Some(node) = doc.get(child_id) else {
        return false;
    };
    let is_line_break_el = matches!(
        &node.kind,
        NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("br") || e.local_name().eq_ignore_ascii_case("wbr")
    );
    if is_line_break_el {
        // br/wbr 是换行/换行机会元素，须保留 taffy 节点维持 line-break 语义（跳过会丢强制换行，
        // 致 line-break-*/white-space 簇回归——R2161 gate-tighten 实测定位）。
        return false;
    }
    if !matches!(&node.kind, NodeKind::Element(_)) {
        return false;
    }
    let Some(s) = styles.get(&child_id) else {
        return false;
    };
    if !matches!(s.display, DisplayValue::Inline) {
        return false;
    }
    if matches!(s.position, PositionValue::Absolute | PositionValue::Fixed) {
        return false;
    }
    // childless：无 Element 子（仅文本后代）。含 Element 子的 inline（如 <a><img></a>）由
    // R2156（嵌套 atomic）或须保留 taffy（嵌套 block/abspos）处理，不入此路径。
    let has_elem_child = doc
        .child_nodes(child_id)
        .iter()
        .any(|&gc| doc.get(gc).is_some_and(|gn| matches!(&gn.kind, NodeKind::Element(_))));
    if has_elem_child {
        return false;
    }
    // 子树无 ooflow 后代守卫（childless 已无 Element 子，理论上无后代可 ooflow；保留以防边界）。
    !crate::inline::InlineFormattingContext::inline_subtree_has_ooflow_descendant(doc, styles, child_id)
}

/// R2161 Phase A slice 2 gate-tighten：判定容器 `dom_id` 是否处于 multicol 列流上下文
///（自身或任一祖先是 multicol 容器，即 column-count / column-width 非 Auto）。multicol 列流
/// 依赖 taffy 对 inline 内容的精确测量以决定列宽 / 列高 / 断列；multi-inline skip-taffy probe
/// 改变该测量会破坏列几何（multicol-width-large / multicol-gap-large / multicol-clip 簇：R2161
/// 实测 css-multicol −5 全为真几何 damage，0.00%→1.7-2.4%，非阈值噪声）。故容器在 multicol 上下文
/// 时抑制 probe。含 `dom_id` 自身：multicol-width-large-* 的 multicol 容器直接含 ≥2 inline 子，
/// probe 即在该容器触发（须查自身，非仅祖先）。
pub(crate) fn container_in_multicol_context(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    dom_id: NodeId,
) -> bool {
    use zero_style_system::property::types::{ColumnCountComputedValue, ColumnWidthComputedValue};
    let mut cur = Some(dom_id);
    while let Some(id) = cur {
        if let Some(s) = styles.get(&id) {
            if !matches!(s.column_count, ColumnCountComputedValue::Auto)
                || !matches!(s.column_width, ColumnWidthComputedValue::Auto)
            {
                return true;
            }
        }
        cur = doc.parent_node(id);
    }
    false
}

/// R2161 Phase A slice 2 gate-tighten：容器 `dom_id` 的 text-wrap 为「均衡 / 美观 / 稳定」模式
///（Balance/Pretty/Stable，非默认 Wrap / Nowrap）时抑制 probe。这些 CSS Text 4 模式的行分配
/// 算法非贪心换行，probe 改 inline 测量会偏移其结果——即便 ZW 当前未实现 balancing（text-wrap 仅
/// 在 paint resolve_text_wrap 消费 Nowrap，layout 不 balance），test 仍对比 chromium 的 balancing
/// ref（text-wrap-balance-003：OFF 4.97% 已是 ZW 不 balance 的输出，probe 推过 5% 阈值→5.39%
/// = 阈值噪声但严格违「零 delta」）。legacy 页（HTML 3.2/4）不用 CSS Text 4 text-wrap，故此 guard
/// 不影响 19-testpage / 20-mixed-legacy 产品增益。
pub(crate) fn container_has_balancing_text_wrap(styles: &HashMap<NodeId, ComputedStyle>, dom_id: NodeId) -> bool {
    use zero_style_system::property::types::TextWrapComputedValue;
    matches!(
        styles.get(&dom_id).map(|s| &s.text_wrap),
        Some(TextWrapComputedValue::Balance | TextWrapComputedValue::Pretty | TextWrapComputedValue::Stable)
    )
}

/// R109 §9.2.1.1 生产端接线产物（仅 env `R109_WIRE=1` 时非空）。
///
/// - `fragment_registry`：匿名块片段 taffy 节点 → 该片段包含的 DOM 子节点，
///   供 extract_layout 写入 LayoutBox.fragment_node_ids。
/// - `split_parents`：被拆分的 inline 元素 DOM NodeId 集合，
///   供 extract_layout 标记 LayoutBox.is_r109_split（抑制其自身 paint IFC）。
/// - `first_inline_fragments` / `last_inline_fragments`：每个 split inline 的
///   匿名块片段序列中，首/末 **Inline** 片段的 anon taffy 节点。供 extract_layout
///   标记 LayoutBox.r109_first/last_fragment——fragment border 边选择（首片段开放
///   右分裂边 border_right=0，末片段开放左边 border_left=0）。
#[derive(Default)]
pub(crate) struct R109Wiring {
    pub fragment_registry: HashMap<taffy::NodeId, Vec<NodeId>>,
    pub split_parents: HashSet<NodeId>,
    pub first_inline_fragments: HashSet<taffy::NodeId>,
    pub last_inline_fragments: HashSet<taffy::NodeId>,
}

/// 构建上下文 — 跟踪 DOM 节点与 taffy 节点的映射。
struct BuildContext {
    /// taffy 布局树。
    taffy: TaffyTree<NodeId>,
    /// DOM NodeId → taffy NodeId 映射。
    node_map: HashMap<NodeId, taffy::NodeId>,
    /// taffy NodeId → DOM NodeId 反向映射。
    taffy_to_dom: HashMap<taffy::NodeId, NodeId>,
    /// `<img>` 元素的解码固有尺寸（DOM NodeId → (width, height)）。
    /// 由调用方（engine pipeline，持有 image_sizes + simple_hash）从解码后的
    /// ImageCache 预解析得到；当 `<img>` 无 width/height 属性时作为固有尺寸回退。
    img_intrinsic_sizes: HashMap<NodeId, (f32, f32)>,
    /// `<img>` 元素的 ratio-only 信号（DOM NodeId → width/height 比，CSS §10.3.2）。
    /// 仅 %-dim / viewBox-only SVG 出现：这些图像无确定固有尺寸，仅有 viewBox 宽高比。
    /// 当 `<img>` 无 width/height 属性且无确定固有尺寸时，仅设 aspect_ratio（不设 size）。
    img_intrinsic_ratios: HashMap<NodeId, f32>,
    /// `<img>` 元素的 no-ratio 信号（DOM NodeId → (真实固有宽, 真实固有高)，各 Option）。
    /// 仅 no-ratio SVG 出现（CSS §10.3.2）：width/height 非双绝对且无 viewBox，既无确定
    /// 固有尺寸也无固有宽高比。值为真实固有维（仅 abs 属性存在的维，缺失维 None）；
    /// 当 `<img>` 无 width/height 属性时按 default object size sizing（不设 aspect_ratio）。
    img_intrinsic_no_ratio: HashMap<NodeId, (Option<f32>, Option<f32>)>,
    /// R109 接线产物（仅 R109_WIRE=1 时填充）。
    r109: R109Wiring,
}

impl BuildContext {
    /// 创建空的构建上下文。
    fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            node_map: HashMap::new(),
            taffy_to_dom: HashMap::new(),
            img_intrinsic_sizes: HashMap::new(),
            img_intrinsic_ratios: HashMap::new(),
            img_intrinsic_no_ratio: HashMap::new(),
            r109: R109Wiring::default(),
        }
    }
}

/// 从 DOM 树和计算样式构建 taffy 树。
///
/// # 参数
///
/// - `doc` — DOM 文档
/// - `styles` — 元素 NodeId → ComputedStyle 映射
/// - `_viewport_width` — 视口宽度（预留，暂未使用）
/// - `_viewport_height` — 视口高度（预留，暂未使用）
///
/// # 返回值
///
/// 返回 (taffy 树, 根节点 ID, taffy→DOM 映射)
pub fn build_layout_tree(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    viewport_width: f32,
    viewport_height: f32,
    img_intrinsic_sizes: HashMap<NodeId, (f32, f32)>,
    img_intrinsic_ratios: HashMap<NodeId, f32>,
) -> (TaffyTree<NodeId>, taffy::NodeId, HashMap<taffy::NodeId, NodeId>) {
    let (taffy, root_id, taffy_to_dom, _r109) = build_layout_tree_with_r109(
        doc,
        styles,
        viewport_width,
        viewport_height,
        img_intrinsic_sizes,
        img_intrinsic_ratios,
        HashMap::new(),
    );
    (taffy, root_id, taffy_to_dom)
}

/// 与 `build_layout_tree` 相同，但额外返回 R109 接线产物（fragment 注册表 +
/// split 父集合），供 extract_layout 写入 LayoutBox.fragment_node_ids / is_r109_split。
pub(crate) fn build_layout_tree_with_r109(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    viewport_width: f32,
    viewport_height: f32,
    img_intrinsic_sizes: HashMap<NodeId, (f32, f32)>,
    img_intrinsic_ratios: HashMap<NodeId, f32>,
    img_intrinsic_no_ratio: HashMap<NodeId, (Option<f32>, Option<f32>)>,
) -> (
    TaffyTree<NodeId>,
    taffy::NodeId,
    HashMap<taffy::NodeId, NodeId>,
    R109Wiring,
) {
    let mut ctx = BuildContext::new();
    ctx.img_intrinsic_sizes = img_intrinsic_sizes;
    ctx.img_intrinsic_ratios = img_intrinsic_ratios;
    ctx.img_intrinsic_no_ratio = img_intrinsic_no_ratio;

    // 找到第一个元素节点作为根（通常是 document > html）
    let root = doc.root();
    let first_element = find_first_element(doc, root);

    let root_taffy_id = build_subtree(
        &mut ctx,
        doc,
        styles,
        first_element,
        None,
        false,
        WritingModeValue::HorizontalTb,
        viewport_width,
        viewport_height,
    );

    (ctx.taffy, root_taffy_id, ctx.taffy_to_dom, ctx.r109)
}

/// R1024：dom_id 的父元素是否为 flex/grid 容器（即 dom_id 是 flex/grid item）。
fn is_flex_grid_item(doc: &Document, styles: &HashMap<NodeId, ComputedStyle>, dom_id: NodeId) -> bool {
    doc.parent_node(dom_id)
        .and_then(|pid| styles.get(&pid))
        .is_some_and(|s| {
            matches!(
                s.display,
                DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
            )
        })
}

/// 查找指定节点子树中的第一个元素节点。
fn find_first_element(doc: &Document, node: NodeId) -> NodeId {
    let node_data = match doc.get(node) {
        Some(n) => n,
        None => return node,
    };

    if matches!(&node_data.kind, NodeKind::Element(_)) {
        return node;
    }

    // 深度优先搜索子节点
    for &child in &node_data.children {
        let found = find_first_element(doc, child);
        let child_data = doc.get(found);
        if child_data.is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))) {
            return found;
        }
    }

    node
}

/// 递归构建 DOM 子树对应的 taffy 子树。
///
/// 返回创建的 taffy 节点 ID。如果元素为 display:none 则不创建节点。
/// `parent_grid_areas` 为父级 grid 容器的区域映射（如果有），
/// 用于解析子元素的 grid-area 命名引用。
/// `in_shadow` 为 true 时表示当前节点处于 shadow 树内部，需要将 <slot>
/// 元素替换为已分配的 light DOM 节点。
/// 为替换元素（img、video、canvas、iframe）注入固有尺寸。
///
/// CSS 规范中，替换元素有 intrinsic size（自然宽高）。
/// 当 CSS width/height 为 auto 时，使用 HTML 属性值作为固有尺寸。
/// 对于 `<img>` 元素，从 `width`/`height` HTML 属性读取尺寸，
/// 并通过 taffy 的 `aspect_ratio` 和尺寸约束传递给布局引擎。
#[allow(clippy::too_many_arguments)]
fn apply_replaced_element_sizing(
    taffy_style: &mut taffy::Style,
    computed: &ComputedStyle,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    dom_id: NodeId,
    img_intrinsic_sizes: &HashMap<NodeId, (f32, f32)>,
    img_intrinsic_ratios: &HashMap<NodeId, f32>,
    img_intrinsic_no_ratio: &HashMap<NodeId, (Option<f32>, Option<f32>)>,
) {
    // R1363：判定本替换元素是否为 flex 容器的直接子（flex item），及主轴方向。
    // 用于 cross-size 推导门控（见下方 width 显式/height auto 分支）。仅水平书写模式
    //（vertical 模式主/交叉轴互换，aspect-ratio 推导不同，跳过会致 vert-lr 回归）。
    use zero_css_parser::values::{DisplayValue, FlexDirectionValue};
    use zero_style_system::property::types::WritingModeValue;
    let (is_flex_row_item, is_flex_col_item) = match doc.parent_node(dom_id).and_then(|p| styles.get(&p)) {
        Some(ps)
            if matches!(ps.display, DisplayValue::Flex | DisplayValue::InlineFlex)
                && matches!(ps.writing_mode, WritingModeValue::HorizontalTb) =>
        {
            let row = matches!(
                ps.flex_direction,
                FlexDirectionValue::Row | FlexDirectionValue::RowReverse
            );
            (row, !row)
        }
        _ => (false, false),
    };

    // 仅处理有 DOM 关联的元素
    let node_data = match doc.get(dom_id) {
        Some(n) => n,
        None => return,
    };

    let elem = match &node_data.kind {
        NodeKind::Element(e) => e,
        _ => return,
    };

    let tag = elem.local_name();

    // 处理 <img> 和 <canvas>：都是替换元素，HTML width/height 属性给出固有尺寸
    // （canvas 的 bitmap 大小）。R784：canvas 此前未处理→被当普通 block 拉伸填满父宽
    // （aspect-ratio-intrinsic-size 簇 canvas 渲染 784px）。video/iframe 等暂无 driving
    // reftest，不处理。
    // 注：<svg> 替换元素 sizing（CSS §10.3.2 默认 300px）经实测对 driving reftest 0-effect
    // （inline-replaced-width 簇依赖 inline SVG 形状渲染，goal line 118 out of scope），暂不处理。
    // R1683：+ <embed>/<object>/<applet>（同为替换元素，HTML width/height 属性定 viewport 固有
    // 尺寸）。此前三者走早返回 → embed 渲成 784×0、object/applet 按 fallback 内容宽。仅当元素
    // 显式带 width/height 属性时应用（无属性回落原行为，避免默认 300×150 改动 ripple）。
    if tag != "img" && tag != "canvas" && tag != "embed" && tag != "object" && tag != "applet" {
        return;
    }

    // R2429：`contain: size`（CSS Containment 1）——元素按「无内容」sized，替换元素固有尺寸
    // 须忽略（intrinsic size → 0）。converter（mod.rs:123 `contain.has_size()`）已把 auto 尺寸
    // 解析为 0（含 contain-intrinsic-size 覆盖），此处若再用固有尺寸覆盖会把 0 拉回 intrinsic，
    // 破坏 size containment（driving：css-contain/contain-size-013 `<img contain:size padding:50>`
    // 固有 60×60 应按 padding-only=100×100，非 160×160）。故 size containment 时早返回，让
    // converter 的 contain:size 处理（含 CIS）生效。
    if computed.contain.has_size() {
        return;
    }
    // R2440：`aspect-ratio: auto <ratio>` —— `auto` 优先 replaced 元素的固有比（CSS Sizing 4
    // §aspect-ratio），显式 <ratio> 仅在无固有比时 fallback。converter 已把显式 ratio 写入
    // taffy_style.aspect_ratio；此处 auto + 有 decoded 固有尺寸时覆盖为固有比（如 img 固有 1:1
    // + `auto 10/1` 应按 1:1 而非 10/1）。无固有尺寸（img_intrinsic_sizes 缺失）则保留显式 ratio。
    if std::env::var("ZW_ASPECT_AUTO").as_deref() != Ok("0")
        && computed.aspect_ratio_auto
        && let Some(&(iw, ih)) = img_intrinsic_sizes.get(&dom_id)
        && ih > 0.0
    {
        taffy_style.aspect_ratio = Some(iw / ih);
    }
    // R1683：embed/object/applet 仅消费 HTML width/height 属性（无 decoded intrinsic / SVG data URI
    // 回退，那是 <img> 专属）。无属性时直接返回保持原行为。
    let is_attr_only_replaced = tag == "embed" || tag == "object" || tag == "applet";

    // 读取 HTML width/height 属性
    let attr_w = elem.get_attribute("width").and_then(|v| v.parse::<f32>().ok());
    let attr_h = elem.get_attribute("height").and_then(|v| v.parse::<f32>().ok());

    // R1683：embed/object/applet 仅消费 HTML width/height 属性（无 decoded intrinsic / SVG data
    // URI，那是 <img> 专属）。无属性时直接返回保持原行为（避免默认 300×150 改动 ripple）。
    // img/canvas 走 SVG data URI 回退填补缺失侧。
    let (attr_w, attr_h) = if is_attr_only_replaced {
        match (attr_w, attr_h) {
            (Some(w), Some(h)) if w > 0.0 && h > 0.0 => (attr_w, attr_h),
            _ => return,
        }
    } else {
        match (attr_w, attr_h) {
            (Some(w), Some(h)) => (Some(w), Some(h)),
            _ => {
                let (svg_w, svg_h) = extract_svg_data_uri_size(elem);
                (attr_w.or(svg_w), attr_h.or(svg_h))
            }
        }
    };

    match (attr_w, attr_h) {
        (Some(w), Some(h)) if w > 0.0 && h > 0.0 => {
            // 两个属性都有：设置固有尺寸（当 CSS 为 auto 时）
            let w = w.max(1.0);
            let h = h.max(1.0);

            // 设置 aspect_ratio（如果 CSS 没有显式设置）。R325：仅当至少一侧 CSS 尺寸为
            // auto 时才设（两侧都显式时 taffy 会强制比例覆盖显式 height，见 _ 分支注释）。
            let css_w_auto = matches!(computed.width, LengthValue::Auto);
            let css_h_auto = matches!(computed.height, LengthValue::Auto);
            if computed.aspect_ratio.is_none() && (css_w_auto || css_h_auto) {
                taffy_style.aspect_ratio = Some(w / h);
            }

            // CSS §10 替换元素尺寸：auto 侧从显式侧按固有宽高比推导（而非直接用 HTML
            // 绝对值）。仅当两侧 CSS 都 auto 时用 HTML 固有尺寸；一侧显式（可为 %）时，
            // auto 侧由 taffy 按 aspect_ratio 从显式侧解析后推导。R784：旧实现 auto 侧
            // 无条件设为 HTML 属性值，致 <canvas width=10 height=10 style="height:100%">
            // 的 width 仍为 HTML 值 10（应按 1:1 比例从 height 100px 推导为 100px）。
            if css_w_auto && css_h_auto {
                taffy_style.size.width = taffy::style::Dimension::length(w);
                taffy_style.size.height = taffy::style::Dimension::length(h);
            }
            // 一侧 auto、一侧显式：不设 auto 侧尺寸，taffy 按 aspect_ratio 推导
        }
        (Some(w), None) if w > 0.0 => {
            // 仅有 width：设置宽度，高度由 aspect_ratio 推导
            if computed.aspect_ratio.is_none() {
                // R2172：SVG data URI unitless width attr（`extract_svg_data_uri_size` 解析
                // '200' 命中此分支，区别于 '50px' 解析失败落 img_intrinsic_sizes 分支）须补设
                // aspect_ratio（从 decoded intrinsic 比），否则替换元素 cross 维 = 0（img-row-010
                // 200×0；img-row-011 等）。仅 CSS aspect-ratio 未设 + 至少一侧 auto + decoded
                // intrinsic 可用时。kill-switch ZW_SVG_ATTR_AR=0。
                if (matches!(computed.width, LengthValue::Auto) || matches!(computed.height, LengthValue::Auto))
                    && std::env::var("ZW_SVG_ATTR_AR").as_deref() != Ok("0")
                    && let Some(&(iw, ih)) = img_intrinsic_sizes.get(&dom_id)
                    && ih > 0.0
                {
                    taffy_style.aspect_ratio = Some(iw / ih);
                }
                // 无 aspect_ratio 也无 height，使用固定宽度
                if matches!(computed.width, LengthValue::Auto) {
                    taffy_style.size.width = taffy::style::Dimension::length(w.max(1.0));
                }
            } else if matches!(computed.width, LengthValue::Auto) {
                taffy_style.size.width = taffy::style::Dimension::length(w.max(1.0));
            }
        }
        (None, Some(h)) if h > 0.0 => {
            // 仅有 height：设置高度，宽度由 aspect_ratio 推导
            if computed.aspect_ratio.is_none() {
                // R2172：对称——SVG unitless height attr 须补设 aspect_ratio（见上分支注释）。
                if (matches!(computed.width, LengthValue::Auto) || matches!(computed.height, LengthValue::Auto))
                    && std::env::var("ZW_SVG_ATTR_AR").as_deref() != Ok("0")
                    && let Some(&(iw, ih)) = img_intrinsic_sizes.get(&dom_id)
                    && ih > 0.0
                {
                    taffy_style.aspect_ratio = Some(iw / ih);
                }
                if matches!(computed.height, LengthValue::Auto) {
                    taffy_style.size.height = taffy::style::Dimension::length(h.max(1.0));
                }
            } else if matches!(computed.height, LengthValue::Auto) {
                taffy_style.size.height = taffy::style::Dimension::length(h.max(1.0));
            }
        }
        _ => {
            // 无 HTML 属性：按解码信号分派。no-ratio / both-abs-sizes / ratio-only 三者互斥
            //（一张图只命中其一；no-ratio 图虽也留在 image_sizes 供背景图读 pixmap 尺寸，
            // 但此处先命中 no_ratio 即跳过 sizes 的 aspect_ratio 逻辑）。
            //
            // no-ratio SVG（CSS §10.3.2）：既无确定固有尺寸也无固有宽高比（width/height
            // 非双绝对且无 viewBox）。usvg 对缺失维的默认值非真实固有尺寸，故**不设
            // aspect_ratio**——auto 侧用真实固有维（若有），否则 default object size
            //（宽 300 / 高 150）。显式 CSS 侧由 converter 处理，min/max 由 taffy 钳制。
            // 驱动案：visudet replaced-elements-{height-20,width-40,max-height-20,max-width-40}。
            if let Some(&(w_opt, h_opt)) = img_intrinsic_no_ratio.get(&dom_id) {
                let width_auto = matches!(computed.width, LengthValue::Auto);
                let height_auto = matches!(computed.height, LengthValue::Auto);
                // 不设 aspect_ratio（no-ratio）
                if width_auto && height_auto {
                    taffy_style.size.width = taffy::style::Dimension::length(w_opt.unwrap_or(300.0).max(0.5));
                    taffy_style.size.height = taffy::style::Dimension::length(h_opt.unwrap_or(150.0).max(0.5));
                } else if !width_auto && height_auto {
                    // width 显式，height auto → 用真实固有高或 default 150
                    taffy_style.size.height = taffy::style::Dimension::length(h_opt.unwrap_or(150.0).max(0.5));
                } else if width_auto && !height_auto {
                    // height 显式，width auto → 用真实固有宽或 default 300
                    taffy_style.size.width = taffy::style::Dimension::length(w_opt.unwrap_or(300.0).max(0.5));
                }
                // 两侧都显式：由 converter 处理，不干预
            } else if let Some(&(w, h)) = img_intrinsic_sizes.get(&dom_id) {
                // both-abs SVG / PNG / JPEG：真固有尺寸（pixmap w/h 有效）。CSS 规范：
                // 替换元素无显式尺寸时使用固有尺寸（intrinsic size）。
                let w = w.max(1.0);
                let h = h.max(1.0);
                let width_auto = matches!(computed.width, LengthValue::Auto);
                let height_auto = matches!(computed.height, LengthValue::Auto);
                // R325：CSS §10 替换元素——仅当【恰好一侧为 auto】时才用固有宽高比推导该 auto 侧。
                // 两侧都显式时【不得】设 aspect_ratio，否则 taffy 会强制比例，把显式 height
                // 拉到 width 比例（如 <img style="width:200px;height:50px"> 渲染成 200×200
                // 而非 200×50）。object-fit 控制内容如何填充 box，box 尺寸由两侧显式值决定。
                // R2428：两侧都 auto 时下方会把 size.width/height 都设为固有值（等同两侧显式）。
                // 此时若【父非 flex/grid 容器】也不得设 aspect_ratio——taffy 会把 ratio 作用到
                // border-box 宽（含 padding）覆盖显式 height（如 <img padding-right:40> 固有 60×60
                // 渲染成 100×100 非 100×60，css-box/margin-trim replaced 簇）。flex/grid 容器的
                // both-auto 仍需 aspect_ratio 推 main-from-cross（R1366 flex-aspect-ratio-img-row-006）。
                let parent_is_flex_grid = doc.parent_node(dom_id).and_then(|p| styles.get(&p)).is_some_and(|ps| {
                    matches!(
                        ps.display,
                        DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
                    )
                });
                let needs_ar = (width_auto ^ height_auto) || (width_auto && height_auto && parent_is_flex_grid);
                if computed.aspect_ratio.is_none() && needs_ar {
                    taffy_style.aspect_ratio = Some(w / h);
                }
                // CSS §10.3/§10.6 替换元素：一侧显式、另一侧 auto 时，auto 侧按
                // 固有宽高比从显式侧推导（而非用固有绝对值）。旧实现把 auto 侧直接设为
                // 固有绝对值（如 width:80px 的正方形 SVG 渲染成 80×441 而非 80×80），
                // 致真实页面 logo（仅设 width 或 height）严重变形（wintertc logo 巨高）。
                //
                // R976：CSS `aspect-ratio` 优先于固有宽高比（css-sizing-4 §4）。auto 侧须按
                // **有效比例**（CSS aspect-ratio 若设，否则固有 w/h）推导。旧实现恒用固有
                // w/h，致 `<img style="block-size:55vw;aspect-ratio:2/1">`（固有 8×16）的 width
                // 被算成 440×(8/16)=220 而非 440×2=880（nested-grid-item-block-size-001 64% diff）。
                // R2441：aspect-ratio `auto <ratio>` 组合（CSS Sizing 4 §aspect-ratio）——`auto`
                // 优先 replaced 固有比。本分支已有 decoded 固有 (w,h)（外层 img_intrinsic_sizes.get），
                // 故 auto 时 eff_ratio=w/h（固有），显式 <ratio> 仅在无固有比时 fallback（本分支不达）。
                // 否则沿用显式 aspect_ratio（R976 优先于固有比）或回落固有。
                let eff_ratio = if computed.aspect_ratio_auto {
                    w / h
                } else {
                    computed.aspect_ratio.unwrap_or(w / h)
                }; // width/height
                if width_auto && height_auto {
                    taffy_style.size.width = taffy::style::Dimension::length(w);
                    taffy_style.size.height = taffy::style::Dimension::length(h);
                } else if !width_auto
                    && height_auto
                    && let LengthValue::Px(cw) = &computed.width
                {
                    // width 显式，height auto：height = cw / eff_ratio
                    // R1363：flex row item 的 main(width) 可能被 min-size:auto 钳制（如
                    // flex-minimum-width-flex-items-013：width:999 → min 钳到 100）。此处用未钳制
                    // 的 cw(999) 预推 height=500 会设为 definite，致 taffy 不再按钳制后 main 重推，
                    // 且不 stretch 到容器 cross。跳过（留 height auto + aspect_ratio），让 taffy 按
                    // 最终（钳制后）main 推 cross（100/2=50）。仅 flex row + 有 aspect_ratio 时跳过。
                    let skip_for_flex_row = is_flex_row_item && taffy_style.aspect_ratio.is_some();
                    if !skip_for_flex_row {
                        taffy_style.size.height = taffy::style::Dimension::length(((*cw as f32) / eff_ratio).max(0.5));
                    }
                } else if width_auto
                    && !height_auto
                    && let LengthValue::Px(ch) = &computed.height
                {
                    // height 显式，width auto：width = ch * eff_ratio
                    // R1363 对称：flex column item 的 main(height) 可能被 min-size:auto 钳制，
                    // 跳过预推 width（留 auto + aspect_ratio），让 taffy 按钳制后 main 推 cross。
                    let skip_for_flex_col = is_flex_col_item && taffy_style.aspect_ratio.is_some();
                    if !skip_for_flex_col {
                        taffy_style.size.width = taffy::style::Dimension::length(((*ch as f32) * eff_ratio).max(0.5));
                    }
                }
                // 两侧都显式：由 converter 从 CSS 处理，不干预
            }
            // ratio-only / computed-intrinsic SVG（viewBox 宽高比，无确定固有尺寸）。
            // ★ chromium 实测（2026-07-15，visudet replaced-elements 4 变体 + css-flexbox
            // aspect-ratio-intrinsic-007）：viewBox 宽高比 **仅 FLEX transferred-size 用**；
            // **INLINE `<img>`**（CSS2 §10.3.2）不应用 viewBox 比，按 default object size 300×150
            // sizing（height-25-ratio-2.svg 配 width:40 → 40×150，非 40×20）。故按上下文分派：
            if let Some(&ratio) = img_intrinsic_ratios.get(&dom_id)
                && ratio > 0.0
            {
                let width_auto = matches!(computed.width, LengthValue::Auto);
                let height_auto = matches!(computed.height, LengthValue::Auto);
                // R2062：replaced 元素的 max-content/min-content 关键字（CSS §10.6.2/§10.3.2
                // 固有尺寸）。converter 把 MinContent|MaxContent→length(0)，但 ratio-only SVG
                //（仅有 viewBox 比，无确定固有维）+ 另一侧 definite 时，content-keyword 侧应按
                // 固有比从 definite 侧推导（chromium position-absolute-replaced-no-intrinsic-size：
                // img width:100 + height:max-content + viewBox 1:1 → 100×100，非 100×0）。
                let width_content_kw = matches!(computed.width, LengthValue::MaxContent | LengthValue::MinContent);
                let height_content_kw = matches!(computed.height, LengthValue::MaxContent | LengthValue::MinContent);
                if is_flex_row_item || is_flex_col_item {
                    // FLEX：保留 ratio 供 transferred-size（aspect-ratio-intrinsic-007：flex column
                    // → 784×392，cross stretch + main=width/ratio）。★ 不设确定 size——definite size
                    // 阻止 taffy transferred-size ratio-derivation（R980/R991/R992 三证）。
                    if computed.aspect_ratio.is_none() && (width_auto || height_auto) {
                        taffy_style.aspect_ratio = Some(ratio);
                    }
                    let eff_ratio = computed.aspect_ratio.unwrap_or(ratio);
                    if !width_auto
                        && height_auto
                        && let LengthValue::Px(cw) = &computed.width
                    {
                        taffy_style.size.height = taffy::style::Dimension::length(((*cw as f32) / eff_ratio).max(0.5));
                    } else if width_auto
                        && !height_auto
                        && let LengthValue::Px(ch) = &computed.height
                    {
                        taffy_style.size.width = taffy::style::Dimension::length(((*ch as f32) * eff_ratio).max(0.5));
                    }
                    // both-auto flex：不设确定 size，仅 aspect_ratio（transferred-size 由 taffy 推）。
                } else {
                    // INLINE（非 flex）：R2054 实测 chromium visudet width-40 ref——img6
                    //（RatioOnly ratio=2）配 width:40 → 40×20 = width/ratio，**应用 viewBox 比**
                    //（纠正 decode_svg_bytes / 旧注释「INLINE 不应用比」误判，该误判仅对
                    // ComputedIntrinsic 显式宽度成立但被 R2054 Fix B 推翻）。设 aspect_ratio 让
                    // taffy 从显式侧推 auto 侧；auto+auto 用 default object size 300×150（ratio=2
                    // 时自洽，其他 ratio 仍 300×150——auto+auto ratio-only 的 chromium 精确 size
                    // 如 container-width×ratio 属 §10.3.2 "should" 未定义，此处保守 default）。
                    if computed.aspect_ratio.is_none() && (width_auto || height_auto) {
                        taffy_style.aspect_ratio = Some(ratio);
                    }
                    if width_auto && height_auto {
                        // R2054 C2：ratio-only auto+auto——chromium visudet all-auto 实测 img6
                        //（ratio-2.svg 无 w/h）在 div width:200 内渲染 200×100 = **父 Px 宽 ×
                        // ratio**（§10.3.2 "should" undefined 的 chromium 非标准行为；default object
                        // size 300×150 会溢出父盒，chromium 收束到父宽）。仅父有 Px 宽时用之
                        //（限 blast radius——auto 父或无父回落 default 300，避免普遍撑满父宽）。
                        let container_w =
                            doc.parent_node(dom_id)
                                .and_then(|p| styles.get(&p))
                                .and_then(|s| match &s.width {
                                    zero_css_parser::values::LengthValue::Px(w) => Some(*w as f32),
                                    _ => None,
                                });
                        let w = container_w.unwrap_or(300.0);
                        taffy_style.size.width = taffy::style::Dimension::length(w);
                        taffy_style.size.height = taffy::style::Dimension::length((w / ratio).max(0.5));
                    } else if height_content_kw && let LengthValue::Px(cw) = &computed.width {
                        // R2062：height:max-content/min-content + 定宽 + 比 → height = width/ratio
                        //（converter 把 max-content→length(0)，此处显式覆写定高，使 abspos
                        // margin:auto 居中等下游用 definite 高度）。对称分支见下。
                        let r = computed.aspect_ratio.unwrap_or(ratio);
                        taffy_style.size.height = taffy::style::Dimension::length(((*cw as f32) / r).max(0.5));
                    } else if width_content_kw && let LengthValue::Px(ch) = &computed.height {
                        // R2062 对称：width:max-content/min-content + 定高 + 比 → width = height*ratio
                        let r = computed.aspect_ratio.unwrap_or(ratio);
                        taffy_style.size.width = taffy::style::Dimension::length(((*ch as f32) * r).max(0.5));
                    }
                    // 显式 width + auto height / 显式 height + auto width：不设 auto 侧 default，
                    // taffy 按 aspect_ratio 从显式侧推导（与 image_sizes BothAbs 路径一致）。
                }
            }
        }
    }

    // CSS Flexbox §4.5 / csswg #5663：替换元素 flex item 的 min-size:auto。
    // taffy 0.7 把 leaf flex item 的 auto-min 当作其 definite 主尺寸（width:999→min 999），
    // 致替换 flex item 无法 flex-shrink（flex-minimum-width-flex-items-013 82% diff：
    // img 999×500 溢出 flex width:0 容器，应被 min-width:auto=100 floor 后收缩到 100）。
    // spec：auto-min = min(content suggestion, transferred suggestion)，transferred =
    // 明确 cross size × 固有比。此处仅当父是 flex 容器且有明确 cross size 时计算
    // transferred 并设 min_size.main（row + column 对称，仅水平书写模式）。
    apply_flex_transferred_min_size(taffy_style, computed, doc, styles, dom_id);
}

/// 替换元素 flex item 的 min-size:auto transferred-size-suggestion（CSS Flexbox §4.5 / csswg #5663）。
///
/// taffy 0.7 把 leaf（替换元素）flex item 的自动最小尺寸当作其 definite 主尺寸本身
/// （如 `width:999px` → min-width 999），导致替换 flex item 无法被 flex-shrink 收缩。
/// 规范要求自动最小 = min(content size suggestion, transferred size suggestion)，
/// 其中 transferred = flex item 的「明确 cross size」× 固有宽高比。当 cross 明确时
/// content suggestion 也是 cross 推导（= transferred），故 auto_min = transferred。
/// 例如 `<img 固有 300×150>` 在 `display:flex;height:50px` 容器内：cross(height)=50，
/// transferred main(width) = 50 × 300/150 = 100px。
///
/// 仅当父是 flex/inline-flex 容器、有明确 cross size（Px）、子有 aspect_ratio、且
/// 子在水平书写模式下、cross-margin 非 auto、align-self 为 Auto/Stretch（cross 被拉伸）
/// 时计算并设置 `min_size.main = transferred`（再被 specified main 钳制）。
/// 其余情况保持 taffy 默认，避免回归。
fn apply_flex_transferred_min_size(
    taffy_style: &mut taffy::Style,
    computed: &ComputedStyle,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    dom_id: NodeId,
) {
    use zero_css_parser::values::{DisplayValue, FlexDirectionValue, LengthValue};

    // 仅水平书写模式（保守；垂直书写模式轴映射需额外处理）
    if !matches!(computed.writing_mode, WritingModeValue::HorizontalTb) {
        return;
    }
    // 父须是 flex 容器
    let Some(parent_id) = doc.parent_node(dom_id) else {
        return;
    };
    let Some(parent_style) = styles.get(&parent_id) else {
        return;
    };
    if !matches!(parent_style.display, DisplayValue::Flex | DisplayValue::InlineFlex) {
        return;
    }
    // 子须有 aspect_ratio（由上方 sizing 设好，或 CSS aspect-ratio）
    let ratio = match taffy_style.aspect_ratio {
        Some(r) if r > 0.0 => r,
        _ => return,
    };
    // 主/交叉轴：column → main=height/cross=width；row(含 reverse) → main=width/cross=height。
    let is_column = matches!(
        parent_style.flex_direction,
        FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
    );
    // §4.5 transferred-size-suggestion 需 item 有明确 cross size。两种来源（容器优先，最小化回归）：
    //  (a) 容器明确 cross（Px）+ item align-stretch → item cross = 容器 cross（原有逻辑）。
    //  (b) item 自身明确 cross（img height:50px）——0.8.3 taffy 不再原生兜底 auto-cross 容器
    //      case（width:0 / auto-height 容器，#819 leaf available_space 变化），须 ZW 覆盖。
    //      cross 须取 min/max 钳制后的 used size（flex-minimum-width-flex-items-012：
    //      height:2000 + max-height:50 → used cross=50，transferred=100，非 2000×ratio）。
    let container_cross = if is_column {
        match &parent_style.width {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        }
    } else {
        match &parent_style.height {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        }
    };
    let (cross, from_item_cross) = match container_cross {
        Some(c) if c > 0.0 => (c, false),
        _ => {
            let item_cross_specified = if is_column {
                match &computed.width {
                    LengthValue::Px(v) => Some(*v as f32),
                    _ => None,
                }
            } else {
                match &computed.height {
                    LengthValue::Px(v) => Some(*v as f32),
                    _ => None,
                }
            };
            let item_max_cross = if is_column {
                match &computed.max_width {
                    LengthValue::Px(v) => Some(*v as f32),
                    _ => None,
                }
            } else {
                match &computed.max_height {
                    LengthValue::Px(v) => Some(*v as f32),
                    _ => None,
                }
            };
            // used cross = min(specified, max)（§4.5 transferred 基于 used cross，非 specified）
            let item_cross = match (item_cross_specified, item_max_cross) {
                (Some(spec), Some(max)) => Some(spec.min(max)),
                (spec, None) => spec,
                _ => None,
            };
            match item_cross {
                Some(c) if c > 0.0 => (c, true),
                _ => return,
            }
        }
    };
    // 仅 case (a)（容器 cross 来源）须确认 item 被 align-stretch（cross = 容器 cross）。
    // case (b) item 自身 cross 已明确，align-self 不影响其 cross size，跳过 stretch 校验。
    if !from_item_cross {
        // item 有 auto cross-margin（margin:auto 居中而非拉伸）或显式非 stretch align-self
        // 时 cross size ≠ 容器 cross → 跳过（auto-margins-002 / flex-aspect-ratio-img-column-012）。
        // row cross=height→margin-top/bottom；column cross=width→margin-left/right。
        let (cross_margin_a, cross_margin_b) = if is_column {
            (&computed.margin_left, &computed.margin_right)
        } else {
            (&computed.margin_top, &computed.margin_bottom)
        };
        if matches!(cross_margin_a, LengthValue::Auto) || matches!(cross_margin_b, LengthValue::Auto) {
            return;
        }
        use zero_css_parser::values::AlignmentValue;
        match computed.align_self {
            // Auto（继承容器，默认 stretch）/ Stretch → item 被拉伸，cross size = 容器 cross
            AlignmentValue::Auto | AlignmentValue::Stretch => {}
            // 显式 center/flex-start/flex-end/baseline/start/end/space-* → 不拉伸，跳过
            _ => return,
        }
    }
    // transferred main size：row → cross_h × ratio(w/h)；column → cross_w / ratio。
    // transferred 须基于 item 的 **content-box** cross size（扣除 item cross 方向 padding），
    // 非 border-box（flex-aspect-ratio-intrinsic-padding-001：img padding:20，cross 240 border-box
    // → content 200 → transferred height = 200/ratio=100，非 240/ratio=120）。无 padding 时
    // item_cross_padding=0，不影响（driving/007/022 均无 padding）。仅扣 padding 不扣 border：
    // ZW 默认 border-width=medium=3px（即使 border-style:none），扣 border 会污染无 border 项。
    let px = |lv: &LengthValue| match lv {
        LengthValue::Px(v) => *v as f32,
        _ => 0.0,
    };
    let item_cross_padding = if is_column {
        px(&computed.padding_left) + px(&computed.padding_right)
    } else {
        px(&computed.padding_top) + px(&computed.padding_bottom)
    };
    let content_cross = (cross - item_cross_padding).max(0.0);
    if content_cross <= 0.0 {
        return;
    }
    let transferred = if is_column {
        content_cross / ratio
    } else {
        content_cross * ratio
    };
    // §4.5 + csswg #5663：当 cross size 明确时，content size suggestion 也是从 cross 推导
    // （= transferred），而非 raw 固有主尺寸。故 auto_min = transferred（非 min(intrinsic, transferred)）。
    // flex-minimum-height-flex-items-007：img 固有 60×60，column cross(width)=100，transferred=100；
    // 旧 min(intrinsic=60, transferred=100)=60 错（应 100），致回归。现直接用 transferred。
    let mut auto_min = transferred;
    // §4.5：auto-min 由 specified size suggestion（子元素明确主尺寸）钳制（auto-min ≤ specified）。
    // 例：img 显式 width:50（< transferred 160）时，auto-min 须 ≤50，否则会错误 floor 到 160，
    // 把本应 50px 的 img 撑大（flex-item-transferred-sizes-padding-* 回归）。
    let specified_main = if is_column {
        match &computed.height {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        }
    } else {
        match &computed.width {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        }
    };
    if let Some(spec) = specified_main {
        auto_min = auto_min.min(spec);
    }
    if auto_min > 0.0 && auto_min.is_finite() {
        if is_column {
            taffy_style.min_size.height = taffy::style::Dimension::length(auto_min);
        } else {
            taffy_style.min_size.width = taffy::style::Dimension::length(auto_min);
        }
    }
}
/// 仅解析简单的内联 SVG（非 base64 编码），提取 `<svg ... width="..." height="...">` 中的数值。
fn extract_svg_data_uri_size(elem: &zero_dom::ElementData) -> (Option<f32>, Option<f32>) {
    let src = match elem.get_attribute("src") {
        Some(s) if s.starts_with("data:image/svg+xml") => s,
        _ => return (None, None),
    };

    // 解码 data URI：去掉 "data:image/svg+xml," 前缀
    let comma_pos = match src.find(',') {
        Some(p) => p,
        None => return (None, None),
    };
    let svg_content = &src[comma_pos + 1..];

    // URL 解码（%xx → 字符）
    let decoded = percent_decode(svg_content);

    // 从 <svg> 开始标签中提取 width/height
    // 查找 <svg ... > 标签
    let svg_start = match decoded.find("<svg") {
        Some(p) => p,
        None => return (None, None),
    };
    let tag_end = match decoded[svg_start..].find('>') {
        Some(p) => svg_start + p,
        None => return (None, None),
    };
    let tag_content = &decoded[svg_start..tag_end];

    let w = extract_attr_float(tag_content, "width");
    let h = extract_attr_float(tag_content, "height");

    (w, h)
}

/// 简易 percent-decode（处理 %xx 转义）。
fn percent_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// 从 XML 标签属性字符串中提取指定属性名对应的浮点数值。
/// 支持单引号和双引号。
fn extract_attr_float(tag: &str, attr: &str) -> Option<f32> {
    // 查找 attr= 模式（前面是空白或标签名）
    let prefix = format!("{}=", attr);
    let pos = tag.find(&prefix)?;
    let value_start = pos + prefix.len();
    let rest = &tag[value_start..];

    // 跳过引号
    let (quote, content_start) = if rest.starts_with('"') {
        ('"', 1)
    } else if rest.starts_with('\'') {
        ('\'', 1)
    } else {
        return None;
    };

    let value_str = &rest[content_start..];
    let end = value_str.find(quote)?;
    value_str[..end].parse::<f32>().ok().filter(|&v| v > 0.0)
}

/// R1684：`<details>` 是否处于闭合态（无 `open` 属性）。
///
/// HTML 渲染规范：`<details>` 无 `open` boolean 属性时，仅 `<summary>` 子可见，其余子
/// 按 UA `details:not([open]) > *:not(summary) { display: none }` 隐藏。ZW 无 UA CSS 父
/// 条件选择器，故在 layout-tree 构建期用此谓词过滤直接子。
fn is_closed_details(doc: &Document, id: NodeId) -> bool {
    let Some(node) = doc.get(id) else {
        return false;
    };
    if !matches!(&node.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("details")) {
        return false;
    }
    // `open` 是 HTML boolean attribute（出现即开启，无论值）。缺失 = 闭合态。
    doc.get_attribute(id, "open").is_none()
}

/// R1684：节点是否为 `<summary>` 元素（details 的 disclosure summary）。
fn is_summary_element(doc: &Document, id: NodeId) -> bool {
    doc.get(id)
        .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("summary")))
}

/// R2439：元素 `content` 属性是否为 `url()`（element-replacement，CSS Content §content-property）。
/// content:url() 的普通元素应**自身**变 replaced（element-becomes-replaced）：抑制其全部真实
/// 子节点（含匿名文本），元素盒自身按 image 固有尺寸 sizing + paint 渲染图片（绕 R109 IFC
/// 不测 inline replaced 的阻塞，见 R2438 child-injection 证伪）。kill-switch `ZW_CONTENT_REPLACE=0`。
/// R57（M3）：replaced 元素（canvas/video/audio/iframe/embed/object/applet）受支持时
/// 抑制 fallback 子内容参与布局（HTML §4.8.10）。display 判定用 InlineBlock（UA 表
/// 中这些元素的内联块映射；非 replaced 的 InlineBlock 不受影响）。
fn is_replaced_with_fallback(computed: &ComputedStyle, doc: &Document, dom_id: NodeId) -> bool {
    // R57（M3）：display 匹配放宽 InlineBlock → Block | InlineBlock——canvas-grid
    // reftest 的 .grid-cell-content { display: block } 使 canvas 为 block，旧条件
    // 下 fallback 子（<p class="fallback">）仍建盒（撑高 span → grid 行高错——
    // composite.grid 对角线布局 12.86% 之一）。HTML §4.8.10：fallback 仅在元素
    // 不支持时显示——canvas 渲染时任何 display 都应排除 fallback 子。
    matches!(computed.display, DisplayValue::Block | DisplayValue::InlineBlock)
        && doc.get(dom_id).is_some_and(|n| match &n.kind {
            NodeKind::Element(e) => matches!(
                e.local_name(),
                "canvas" | "video" | "audio" | "iframe" | "embed" | "object" | "applet"
            ),
            _ => false,
        })
}

fn is_content_url_element(computed: &ComputedStyle) -> bool {
    matches!(
        computed.content,
        zero_style_system::property::types::ContentComputedValue::Url(_)
    )
}

/// margin-trim 首末子判定：元素是否为参与块格式化上下文的 in-flow block-level 子。
/// 排除 display:none、inline/contents 级（垂直 margin 不参与块轴）、abspos/fixed（脱流）。
/// display 谓词与 R1285 `br_is_inline_only` 的 block-level 判定一致。
fn is_block_level_in_flow(display: &DisplayValue, position: &PositionValue) -> bool {
    matches!(
        display,
        DisplayValue::Block
            | DisplayValue::FlowRoot
            | DisplayValue::ListItem
            | DisplayValue::Flex
            | DisplayValue::Grid
            | DisplayValue::Table
    ) && !matches!(position, PositionValue::Absolute | PositionValue::Fixed)
}

#[allow(clippy::too_many_arguments)]
fn build_subtree(
    ctx: &mut BuildContext,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    dom_id: NodeId,
    parent_grid_areas: Option<&GridAreaMap>,
    in_shadow: bool,
    parent_writing_mode: WritingModeValue,
    viewport_w: f32,
    viewport_h: f32,
) -> taffy::NodeId {
    // LAY-08: 先检查 display:none 再克隆，避免对隐藏元素做不必要的堆分配
    if styles.get(&dom_id).is_some_and(|s| s.display == DisplayValue::None) {
        let hidden_style = taffy::Style {
            display: taffy::style::Display::None,
            ..taffy::Style::default()
        };
        return ctx
            .taffy
            .new_leaf(hidden_style)
            .unwrap_or_else(|_| ctx.taffy.new_leaf(taffy::Style::default()).unwrap());
    }

    // 获取计算样式（或使用默认值）
    let computed = styles.get(&dom_id).cloned().unwrap_or_default();

    // 解析此元素的 grid-template-areas（如果有）
    let grid_areas = computed
        .grid_template_areas
        .as_ref()
        .map(|s| parse_grid_template_areas(s));

    // 转换为 taffy 样式（传入父级区域映射）
    let mut taffy_style = computed_style_to_taffy(&computed, parent_grid_areas, viewport_w, viewport_h);

    // margin-trim（css-box-4 §margin-trim）：父块容器声明 margin-trim 的 block / block-start /
    // block-end 时，归零首子 block-start（margin-top）与/或末子 block-end（margin-bottom）。
    // 在 taffy 布局前修改 taffy_style.margin，使整个流正确重算（trim 到 0 即移除参与折叠的
    // margin，对 collapsing / non-collapsing 案均正确）。bounded scope：仅水平书写模式
    // （horizontal-tb）；inline 轴 trim 对块级子无效（block-container-inline-001 实证：`margin-trim:
    // inline` 不裁剪块级子的 inline 边距）；flex/grid/multicol 容器有独立语义（defer）；自折叠 /
    // 嵌套深案（block-container-block-*-self-collapsing-*）defer。kill-switch `ZW_MARGIN_TRIM=0`
    // （default-on）。driving: css/css-box/margin-trim/block-container-block-001 等。
    if std::env::var("ZW_MARGIN_TRIM").as_deref() != Ok("0")
        && matches!(computed.writing_mode, WritingModeValue::HorizontalTb)
        && let Some(parent_id) = doc.parent_node(dom_id)
        && let Some(ps) = styles.get(&parent_id)
        && matches!(
            ps.display,
            DisplayValue::Block | DisplayValue::FlowRoot | DisplayValue::ListItem
        )
        && (ps.margin_trim.block_start || ps.margin_trim.block_end)
        && is_block_level_in_flow(&computed.display, &computed.position)
    {
        // 父容器 in-flow block-level 子（按文档序），用于定位当前子是否为首/末子。
        let in_flow_block: Vec<NodeId> = doc
            .child_nodes(parent_id)
            .iter()
            .copied()
            .filter(|&s| {
                styles
                    .get(&s)
                    .is_some_and(|st| is_block_level_in_flow(&st.display, &st.position))
            })
            .collect();
        if let Some(idx) = in_flow_block.iter().position(|&s| s == dom_id) {
            let zero = taffy::style::LengthPercentageAuto::length(0.0_f32);
            if idx == 0 && ps.margin_trim.block_start {
                taffy_style.margin.top = zero;
            }
            if idx + 1 == in_flow_block.len() && ps.margin_trim.block_end {
                taffy_style.margin.bottom = zero;
            }
        }
    }

    // margin-trim（css-box-4 §margin-trim）— flex 容器主轴（horizontal-tb，单行）：
    // row → 主轴 = inline，`margin_trim.inline_start`/`inline_end` 裁首子 margin-left / 末子
    // margin-right；column → 主轴 = block，`block_start`/`block_end` 裁首子 margin-top / 末子
    // margin-bottom。单项时首末为同一子 → 两侧均裁（flex-grow/shrink 得空间填容器）。
    // **bounded defer**：row-reverse/column-reverse（物理首末反向）、RTL row、cross 轴 trim、
    // 多行（wrap）逐行首末——均需 taffy 行信息或方向映射，暂不处理（driving tests 皆 LTR 单行）。
    // kill-switch 同 block 分支 `ZW_MARGIN_TRIM=0`（default-on）。driving: css/css-box/margin-trim/
    // flex-row-grow / flex-row-shrink / flex-column-grow / flex-column-shrink。
    if std::env::var("ZW_MARGIN_TRIM").as_deref() != Ok("0")
        && matches!(computed.writing_mode, WritingModeValue::HorizontalTb)
        && let Some(parent_id) = doc.parent_node(dom_id)
        && let Some(ps) = styles.get(&parent_id)
        && matches!(ps.display, DisplayValue::Flex)
        && is_block_level_in_flow(&computed.display, &computed.position)
        && matches!(ps.flex_direction, FlexDirectionValue::Row | FlexDirectionValue::Column)
    {
        let in_flow_flex: Vec<NodeId> = doc
            .child_nodes(parent_id)
            .iter()
            .copied()
            .filter(|&s| {
                styles
                    .get(&s)
                    .is_some_and(|st| is_block_level_in_flow(&st.display, &st.position))
            })
            .collect();
        if let Some(idx) = in_flow_flex.iter().position(|&s| s == dom_id) {
            let zero = taffy::style::LengthPercentageAuto::length(0.0_f32);
            let is_first = idx == 0;
            let is_last = idx + 1 == in_flow_flex.len();
            match ps.flex_direction {
                FlexDirectionValue::Row => {
                    if is_first && ps.margin_trim.inline_start {
                        taffy_style.margin.left = zero;
                    }
                    if is_last && ps.margin_trim.inline_end {
                        taffy_style.margin.right = zero;
                    }
                }
                FlexDirectionValue::Column => {
                    if is_first && ps.margin_trim.block_start {
                        taffy_style.margin.top = zero;
                    }
                    if is_last && ps.margin_trim.block_end {
                        taffy_style.margin.bottom = zero;
                    }
                }
                _ => {}
            }
        }
    }

    // R1284：`<br>` 经 convert_display 映射为 taffy Block leaf（无内容 → height 0）。
    // 当 br 处于 block 兄弟之间（如 `<div/><br><div/>`），它作为 direct block 子渲染
    // 0px 高（chromium ~line-height），致后续块累积垂直错位（table-cell-width-0 等）。
    // CSS §10.8.1：空 line box 仍含 strut（容器 line-height）。给 br 的 taffy min-height
    // 设 line-height strut，使其占一行高。
    // **仅当 br 有 block-level in-flow 同胞时应用**——否则 br 在 inline 内容中（如
    // `<p>a<br>b</p>`）由父 IFC 的 InlineItem::Br 处理，加 min-height 会双计（taffy 子 +
    // IFC 行）致回归（css-text -7 实证）。kill-switch `ZW_BR_LINEHEIGHT=0`（default-on）。
    if std::env::var("ZW_BR_LINEHEIGHT").as_deref() != Ok("0")
        && doc
            .get(dom_id)
            .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("br")))
        && doc.parent_node(dom_id).is_some_and(|pid| {
            doc.child_nodes(pid).iter().any(|&s| {
                use zero_css_parser::values::DisplayValue;
                s != dom_id
                    && styles.get(&s).is_some_and(|st| {
                        matches!(
                            st.display,
                            DisplayValue::Block
                                | DisplayValue::Flex
                                | DisplayValue::Grid
                                | DisplayValue::Table
                                | DisplayValue::ListItem
                                | DisplayValue::FlowRoot
                        )
                    })
            })
        })
    {
        let (_fs, lh) = crate::inline::resolve_font_metrics(Some(&computed));
        if lh > 0.0 {
            taffy_style.min_size.height = taffy::style::Dimension::length(lh);
        }
    }

    // 替换元素固有尺寸：检测 <img> 元素并注入 HTML 属性中的 width/height，
    // 无属性时回退到解码后的固有尺寸（img_intrinsic_sizes）
    apply_replaced_element_sizing(
        &mut taffy_style,
        &computed,
        doc,
        styles,
        dom_id,
        &ctx.img_intrinsic_sizes,
        &ctx.img_intrinsic_ratios,
        &ctx.img_intrinsic_no_ratio,
    );

    // R1365：flex item 的 flex-basis 为百分比且容器 main 尺寸不明确时，item 的 main-size
    // 属性（height/width）不应被当 definite（CSS-Flexbox §9 + §7.1：百分比 flex-basis 对不明确
    // 容器回退到 content，显式 main-size 属性被忽略）。converter 已从 main-size 属性设了 definite
    // size.main，致 taffy 优先用它（如 flex-basis-010：height:500 被用，应回退 content 100）。
    // 修复：百分比 flex-basis + 容器 main 不明确（auto）→ 把 item 的 size.main 改 auto。
    // 仅水平书写模式（vertical 主/交叉轴互换）。
    {
        use zero_css_parser::values::{DisplayValue, FlexDirectionValue};
        use zero_style_system::property::types::{FlexBasisValue, WritingModeValue};
        if matches!(computed.writing_mode, WritingModeValue::HorizontalTb)
            && let Some(parent_id) = doc.parent_node(dom_id)
            && let Some(ps) = styles.get(&parent_id)
            && matches!(ps.display, DisplayValue::Flex | DisplayValue::InlineFlex)
        {
            let is_column = matches!(
                ps.flex_direction,
                FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
            );
            let basis_is_percentage = matches!(computed.flex_basis, FlexBasisValue::Length(LengthValue::Percentage(_)));
            // 容器 main 尺寸不明确：column→height auto，row→width auto。
            let parent_main_indefinite = if is_column {
                matches!(ps.height, LengthValue::Auto)
            } else {
                matches!(ps.width, LengthValue::Auto)
            };
            if basis_is_percentage && parent_main_indefinite {
                if is_column {
                    taffy_style.size.height = taffy::style::Dimension::auto();
                } else {
                    taffy_style.size.width = taffy::style::Dimension::auto();
                }
            }
        }
    }

    // 多列容器：设置 overflow: Hidden 阻止 taffy 内部的父子 margin 折叠。
    // CSS Multi-column Layout Module §2 规定多列容器建立 BFC。
    // taffy 的 is_scroll_container() 仅对 Hidden/Scroll 返回 true，Clip 不会阻止折叠。
    // 此处仅影响 taffy 的 margin 折叠行为，不影响视觉裁剪
    // （paint 层使用 LayoutBox.overflow_x/y 做裁剪，不依赖 taffy overflow）。
    {
        use zero_style_system::property::types::{ColumnCountComputedValue, ColumnWidthComputedValue};
        let is_multicol = !matches!(computed.column_count, ColumnCountComputedValue::Auto)
            || !matches!(computed.column_width, ColumnWidthComputedValue::Auto);
        if is_multicol {
            taffy_style.overflow.x = taffy::style::Overflow::Hidden;
            taffy_style.overflow.y = taffy::style::Overflow::Hidden;
        }
    }

    // 垂直书写模式轴交换
    // CSS Writing Modes §7.1：在垂直书写模式中，水平/垂直维度互换。
    // 当父元素（即当前元素的 containing block）具有 vertical writing mode 时，
    // 交换盒模型属性使 taffy 以「水平=行内」模型计算布局，
    // 然后在提取结果时交换坐标还原视觉位置。
    //
    // 注意：此轴交换仅影响布局盒的几何位置。文本字符的垂直排列（逐字竖排）
    // 和字符旋转由 paint 层的 GlyphPrimitive.rotation 字段控制。
    let is_vertical = parent_writing_mode.is_vertical_block_flow();
    if is_vertical {
        crate::converter::apply_vertical_writing_mode(&mut taffy_style);
    }

    // 记录此元素的 writing mode，用于子元素轴交换判断
    let own_writing_mode = computed.writing_mode.clone();

    // 收集需要创建 taffy 节点的子元素
    // 当元素有 ShadowRoot 时，遍历 shadow 树而非 light DOM 子节点；
    // shadow 树中的 <slot> 元素替换为已分配的 light DOM 节点（或回退内容）。
    let mut child_taffy_ids: Vec<taffy::NodeId> = Vec::new();

    // R2251 content-visibility:hidden（CSS Containment Module Level 2；kill-switch
    // `ZW_CONTENT_VISIBILITY`，default-on；`=0` 关闭回旧「不解析」等价行为）。
    // 元素自身盒（背景/边框）仍经 paint_node 绘制，但其整个子树被跳过：不收集任何
    // taffy 子节点 → 子元素无 layout box（不绘制）、亦不贡献尺寸（→ size containment：
    // auto 尺寸取 padding/border，content=0）。元素直属文本的尺寸抑制与绘制跳过分别在
    // measure_text_content（inline_finalization.rs）与 painter paint_text 门控处理。
    // driving: css/css-contain/content-visibility/content-visibility-001/003/005.. 等。
    let content_visibility_hidden =
        std::env::var("ZW_CONTENT_VISIBILITY").as_deref() != Ok("0") && computed.content_visibility_hidden_effective();

    if content_visibility_hidden {
        // 跳过子树收集：child_taffy_ids 保持空 → 元素作 leaf 创建（见下方 new_leaf_with_context）。
    } else if let Some(shadow_id) = doc.shadow_root(dom_id) {
        // 有 ShadowRoot → 遍历 shadow 树，slot 解析在任意深度生效
        collect_shadow_children(
            ctx,
            doc,
            styles,
            shadow_id,
            grid_areas.as_ref(),
            &mut child_taffy_ids,
            &own_writing_mode,
            viewport_w,
            viewport_h,
        );
        // 注意：未分配到任何 slot 的 light DOM 子节点不会出现在布局树中
    } else if in_shadow {
        // 在 shadow 树内部，需要检查子元素是否为 <slot> 以进行替换
        collect_shadow_slot_children(
            ctx,
            doc,
            styles,
            dom_id,
            grid_areas.as_ref(),
            &mut child_taffy_ids,
            &own_writing_mode,
            viewport_w,
            viewport_h,
        );
    } else {
        // 无 ShadowRoot，不在 shadow 树中 → 正常遍历 light DOM 子节点
        let node_data = doc.get(dom_id);
        let children_dom: Vec<NodeId> = node_data.map(|n| n.children.clone()).unwrap_or_default();
        // R1684：`<details>` 无 `open` 属性（闭合态 disclosure）时，仅 `<summary>` 子渲染，
        // 其余子隐藏（HTML 渲染规范 `details:not([open]) > *:not(summary) { display: none }`）。
        // ZW 无 UA CSS 父条件选择器，故在 layout-tree 构建期按 details 状态过滤直接子。
        // 仅保留 summary 元素子（闭合态非 summary 内容不建 layout box → 不渲染）。
        //
        // R2439：`content:url()` 普通元素（element-becomes-replaced）→ 抑制**全部**真实子节点
        //（含匿名文本）；元素盒自身渲染图片（pipeline pre-layout sizing + paint_img_element
        // 扩展，绕 R109 IFC，见 R2438）。kill-switch `ZW_CONTENT_REPLACE=0`。
        // R57（M3）：canvas/video/audio/iframe/embed/object/applet（replaced + fallback 内容）
        // 受支持时不布局 fallback 子——HTML §4.8.10 fallback 仅在元素不支持时显示；子盒会
        // 引入 margin collapse（fallback `<p>` 的 16px 上边距塌穿 canvas → canvas 盒下移 16px）
        // 与多余盒（painter 曾叠绘 "FAIL (fallback content)" 文本）。canvas-grid reftest
        // 2d.gradient.colorInterpolationMethod 的格子 38px 偏移即此（oracle A/B）。
        let children_dom: Vec<NodeId> = if (std::env::var("ZW_CONTENT_REPLACE").as_deref() != Ok("0")
            && is_content_url_element(&computed))
            || is_replaced_with_fallback(&computed, doc, dom_id)
        {
            Vec::new()
        } else if is_closed_details(doc, dom_id) {
            children_dom
                .into_iter()
                .filter(|&c| is_summary_element(doc, c))
                .collect()
        } else {
            children_dom
        };

        // 检测是否为 flex/grid 容器 — 在这些容器中，文本节点成为匿名 flex/grid 项
        let is_flex_or_grid = matches!(
            computed.display,
            DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
        );

        if is_flex_or_grid {
            // Flex/Grid 容器：文本节点成为匿名 flex/grid 项参与布局。
            // CSS Flexbox §4：每个连续的文本运行生成一个匿名 flex item。
            // 收集所有子节点（元素 + 文本），为文本节点创建匿名 taffy 节点。
            let mut children_with_order: Vec<(NodeId, i32)> = Vec::new();

            for &child_dom in &children_dom {
                let child_data = doc.get(child_dom);
                if let Some(data) = child_data {
                    match &data.kind {
                        NodeKind::Element(_) => {
                            let order = styles.get(&child_dom).map_or(0, |s| {
                                // CSS Flexbox §8.1：`order` 只重排 in-flow flex item。
                                // abspos（position:absolute/fixed）不是 flex item，其
                                // 绘制顺序遵循 DOM 顺序（CSS Appendix E step 6），不受
                                // `order` 影响（flexbox-paint-ordering-003）。用 0 作排序键
                                // → stable sort 保持 abspos 的 DOM 相对顺序。
                                if matches!(
                                    s.position,
                                    zero_style_system::property::types::PositionValue::Absolute
                                        | zero_style_system::property::types::PositionValue::Fixed
                                ) {
                                    0
                                } else {
                                    s.order
                                }
                            });
                            children_with_order.push((child_dom, order));
                        }
                        NodeKind::Text(text_data) if !text_data.content.trim().is_empty() => {
                            // 匿名 flex item 的 order 默认为 0
                            children_with_order.push((child_dom, 0));
                        }
                        _ => {}
                    }
                }
            }

            // 按 order 稳定排序（相同 order 保持 DOM 顺序）
            children_with_order.sort_by_key(|(_, order)| *order);

            for &(child_dom, _) in &children_with_order {
                let child_data = doc.get(child_dom);
                let is_text = child_data.is_some_and(|n| matches!(&n.kind, NodeKind::Text(_)));

                if is_text {
                    // 文本节点：创建匿名 taffy leaf 节点
                    // 使用文本 NodeId 作为 context，使测量回调能识别文本内容
                    let anon_style = taffy::Style {
                        display: taffy::style::Display::Block,
                        ..taffy::Style::default()
                    };
                    let anon_taffy = ctx
                        .taffy
                        .new_leaf_with_context(anon_style, child_dom)
                        .unwrap_or_else(|_| ctx.taffy.new_leaf(taffy::Style::default()).unwrap());
                    ctx.node_map.insert(child_dom, anon_taffy);
                    ctx.taffy_to_dom.insert(anon_taffy, child_dom);
                    child_taffy_ids.push(anon_taffy);
                } else {
                    // 元素节点：正常递归构建
                    let child_taffy = build_subtree(
                        ctx,
                        doc,
                        styles,
                        child_dom,
                        grid_areas.as_ref(),
                        false,
                        own_writing_mode.clone(),
                        viewport_w,
                        viewport_h,
                    );
                    child_taffy_ids.push(child_taffy);
                }
            }
        } else {
            // 非 flex/grid 容器
            // R109 §9.2.1.1（env R109_WIRE=1）：
            // ① inline 元素含 in-flow block-level 子元素 → inline 被拆分为匿名块盒序列
            // ② block 容器含混合 inline+block 子元素 → inline 子元素被匿名块盒包裹
            let is_inline_r109 = r109_wired() && inline_has_block_child(doc, styles, dom_id);
            let is_block_mixed = r109_wired() && block_container_has_mixed_content(doc, styles, dom_id);
            let r109_segments = if is_inline_r109 {
                ctx.r109.split_parents.insert(dom_id);
                compute_inline_block_split(doc, styles, dom_id)
            } else if is_block_mixed {
                compute_block_container_split(doc, styles, dom_id)
            } else {
                None
            };

            if let Some(segments) = r109_segments {
                // 收集本 split inline 的 Inline 片段 anon taffy ID（按片段顺序），
                // 循环后标记首/末，供 fragment border 边选择。
                let mut inline_anon_ids: Vec<taffy::NodeId> = Vec::new();
                for seg in segments {
                    match seg {
                        InlineBlockSegment::Inline { item_node_ids } => {
                            // block 容器：跳过纯空白 inline 片段保 collapse-through 语义
                            if is_block_mixed && is_whitespace_only_inline_segment(doc, &item_node_ids) {
                                continue;
                            }
                            // 取片段首个文本节点作为 measure context（单文本片段精确；
                            // 多节点片段仅按首节点近似尺寸，已知限制）。
                            let ctx_node = item_node_ids
                                .iter()
                                .copied()
                                .find(|&nid| doc.get(nid).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))))
                                .unwrap_or(dom_id);
                            // R57（M3）：片段内非纯 inline display 的元素（img/canvas/
                            // inline-block 等原子行内级）建独立 taffy 子树作为匿名块子盒
                            // ——否则原子 inline 无 LayoutBox，painter 无法绘制（canvas-grid
                            // reftest `<span><div>srgb</div><canvas></canvas></span>` 的 canvas
                            // 曾无盒 → 格子全空白，2d.gradient.colorInterpolationMethod
                            // oracle A/B 10.7%）。纯 inline 元素与文本仍走 IFC（fragment
                            // 收集），此处只补需要盒子的原子项。
                            let atomic_children: Vec<taffy::NodeId> = item_node_ids
                                .iter()
                                .copied()
                                .filter(|&nid| {
                                    doc.get(nid).is_some_and(|n| {
                                        matches!(&n.kind, NodeKind::Element(_))
                                            && styles
                                                .get(&nid)
                                                .is_some_and(|s| !matches!(s.display, DisplayValue::Inline))
                                    })
                                })
                                .map(|nid| {
                                    build_subtree(
                                        ctx,
                                        doc,
                                        styles,
                                        nid,
                                        grid_areas.as_ref(),
                                        false,
                                        own_writing_mode.clone(),
                                        viewport_w,
                                        viewport_h,
                                    )
                                })
                                .collect();
                            let anon_style = if is_block_mixed {
                                // block 容器匿名块：plain Block（不继承容器盒模型，容器
                                // 自身的 bg/border/padding 仍由容器盒绘制）。
                                taffy::Style {
                                    display: taffy::style::Display::Block,
                                    ..taffy::Style::default()
                                }
                            } else {
                                // inline 元素匿名块：继承 split inline 的盒模型（border/
                                // padding/background），使其 border/background 经 shrink
                                // 落在文本宽（§9.2.1.1：被拆分 inline 的 border/background
                                // 在 inline 级=各匿名块绘制）。
                                let mut anon_style =
                                    computed_style_to_taffy(&computed, parent_grid_areas, viewport_w, viewport_h);
                                anon_style.display = taffy::style::Display::Block;
                                // 清零 inset：匿名块片段不应继承 split inline 的 position 偏移
                                anon_style.inset = taffy::geometry::Rect {
                                    left: taffy::style::LengthPercentageAuto::AUTO,
                                    right: taffy::style::LengthPercentageAuto::AUTO,
                                    top: taffy::style::LengthPercentageAuto::AUTO,
                                    bottom: taffy::style::LengthPercentageAuto::AUTO,
                                };
                                anon_style
                            };
                            let anon_taffy = if atomic_children.is_empty() {
                                ctx.taffy
                                    .new_leaf_with_context(anon_style, ctx_node)
                                    .unwrap_or_else(|_| ctx.taffy.new_leaf(taffy::Style::default()).unwrap())
                            } else {
                                let node = ctx
                                    .taffy
                                    .new_with_children(anon_style, &atomic_children)
                                    .unwrap_or_else(|_| ctx.taffy.new_leaf(taffy::Style::default()).unwrap());
                                let _ = ctx.taffy.set_node_context(node, Some(ctx_node));
                                node
                            };
                            ctx.taffy_to_dom.insert(anon_taffy, dom_id);
                            ctx.r109.fragment_registry.insert(anon_taffy, item_node_ids);
                            inline_anon_ids.push(anon_taffy);
                            child_taffy_ids.push(anon_taffy);
                        }
                        InlineBlockSegment::Block { node_id } => {
                            let child_taffy = build_subtree(
                                ctx,
                                doc,
                                styles,
                                node_id,
                                grid_areas.as_ref(),
                                false,
                                own_writing_mode.clone(),
                                viewport_w,
                                viewport_h,
                            );
                            child_taffy_ids.push(child_taffy);
                        }
                    }
                }
                // 标记首/末 Inline 片段（fragment border 边选择）。
                if let Some(&first) = inline_anon_ids.first() {
                    ctx.r109.first_inline_fragments.insert(first);
                }
                if inline_anon_ids.len() > 1 {
                    if let Some(&last) = inline_anon_ids.last() {
                        ctx.r109.last_inline_fragments.insert(last);
                    }
                }
            } else {
                // 非 flex/grid 容器
                // R1024：block 容器的子若**全部**是 inline 级（文本 + display:Inline 元素如 br/span/a，
                // 无 block/inline-block/img 等需独立 taffy 子树的子），整容器作 **leaf**（context=dom_id），
                // 让 measure 回调经 has_inline_content 把全部 inline 文本作为一个 IFC 单位测量——
                // 否则容器成 new_with_children（仅 Element 子）非 leaf，measure 不触发，intrinsic 宽不含文本
                //（flex/grid item 含文本+br 时塌缩 w=0；rootpos 4 案 body 驱动）。inline 元素的样式由
                // paint IFC 读 DOM 处理（与纯文本块一致），不需要独立 LayoutBox。
                let has_text_child = children_dom
                    .iter()
                    .any(|&c| doc.get(c).is_some_and(|n| matches!(&n.kind, NodeKind::Text(_))));
                let has_element_child = children_dom
                    .iter()
                    .any(|&c| doc.get(c).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))));
                let all_inline = children_dom.iter().all(|&c| {
                    let Some(n) = doc.get(c) else {
                        return true;
                    };
                    match &n.kind {
                        NodeKind::Text(_) => true,
                        NodeKind::Element(_) => {
                            // R1024：inline 元素还须「无 Element 子」——含 Element 后代（如 span 内
                            // 嵌 abspos/block）的 inline 须保留 taffy 子树，否则其后代被丢出 taffy 树
                            //（abspos-in-inline 簇 regression：span 内 abspos 失去 CB）。
                            // R1492：**自身** position:absolute/fixed 的 inline 元素亦须保留 taffy 节点
                            //（OO-flow 定位，不能流入父 IFC）——否则 leaf-path 把它丢出 taffy 树
                            //（hit_test_absolute_positioned_link 回归：div 内 abspos <a> 被 leaf 吞）。
                            let cs = styles.get(&c);
                            let is_inline = cs.is_some_and(|s| matches!(s.display, DisplayValue::Inline));
                            let not_ooflow = cs
                                .is_some_and(|s| !matches!(s.position, PositionValue::Absolute | PositionValue::Fixed));
                            let no_elem_child = !doc
                                .child_nodes(c)
                                .iter()
                                .any(|&gc| doc.get(gc).is_some_and(|gn| matches!(&gn.kind, NodeKind::Element(_))));
                            is_inline && not_ooflow && no_elem_child
                        }
                        _ => true,
                    }
                });
                // R1492/R1494：plain-block leaf-path 扩展（Phase-4 inline-ownership）已证伪并 revert。
                // 实测 ZW_PLAIN_INLINE_LEAF=1 borders oracle 411→401（-10），inline 子回流父 IFC 比
                // R1480 shrink（inline→独立 box 收缩）更偏离 chromium——Phase 4 非 R1492 正解。
                // R1492（plain block + inline 子 → measure 低估 → 兄弟重叠）须 measure/post-process 侧
                // 修（保 inline 子为独立 box，修正容器高 + 移后续兄弟），见 master.md R1494 forward。
                if has_text_child
                    && has_element_child
                    && all_inline
                    && (is_flex_grid_item(doc, styles, dom_id)
                        || matches!(computed.display, DisplayValue::InlineBlock)
                        || !matches!(computed.float, FloatValue::None))
                {
                    // R1024/R1025：content-sized block（flex/grid item / inline-block / float）的全 inline 子
                    // 作 leaf——让 measure 经 has_inline_content 把全部 inline 文本作一个 IFC 单位测量，
                    // 修 flex/grid item 含文本+br 塌缩 w=0 + inline-block/float 含文本+br 误填满父宽
                    //（w=800，应 shrink-to-fit）。fill-width block（multicol 容器、普通 div/table-cell）
                    // 不入此路径（multicol -6 回归、table auto-layout 独立、welcome 非必需）。
                    // inline Element 须无 Element 子（abspos-in-inline 簇的 span 内 abspos 须保留 CB）。
                } else {
                    // 仅处理元素子节点（原有行为）
                    //
                    // R2160 Phase A slice 2（env `ZW_PHASEA_MULTI_INLINE`，**R2198 default-on**；`=0`
                    // kill-switch）：多 inline Element 子 block 容器中，**childless plain inline**（display:
                    // inline + 无 Element 子 + 非 ooflow + 子树无 ooflow 后代）的 taffy 节点跳过——让其
                    // 文本流入父 IFC（消除 a/i/b 块级栈列）。orphan 信号（inline_heights 无条目 =
                    // owner_h=0）驱动 painter R639 part2 对 orphan 触发 per-fragment bg/border 绘制
                    //（part1+part2 经 orphan 信号耦合，单行非 orphan 不触发=无双绘，避 R1492）。
                    // gate 仅容器有 ≥2 个合格 inline Element 子时生效（精确触发 multi-inline 栈列
                    // bug；单 inline 子仍走 LayoutBox=R1492-safe，缩 blast radius）。限 horizontal-tb。
                    // ★ R2161 gate-tighten（br/wbr tag 排除 + multicol-context + text-wrap balance 守卫）
                    //   使 self-source 由 net −20 拉回 net +2。**R2163 曾 REVERT default-on → default-off**
                    //   （orphan 丢 LayoutBox 破 hit-test + struct item-tag:0）；**R2197 slice 3 external-set**
                    //   （orphan LayoutBox 回填 + paint_skip）修 hit-test/struct，**R2198 struct-check
                    //   paint_skip-aware**（修窄屏 multi-line `<a>` 假阳性）后 default-on 复开。
                    let phasea_multi_inline_on = std::env::var("ZW_PHASEA_MULTI_INLINE").as_deref() != Ok("0")
                        && matches!(own_writing_mode, WritingModeValue::HorizontalTb)
                        && !container_in_multicol_context(doc, styles, dom_id)
                        && !container_has_balancing_text_wrap(styles, dom_id);
                    let eligible_inline_count = if phasea_multi_inline_on {
                        children_dom
                            .iter()
                            .filter(|&&c| phasea_multi_inline_eligible(doc, styles, c))
                            .count()
                    } else {
                        0
                    };
                    // R2197 atomicity：若容器含**非 eligible 的 inline Element 子**（如含嵌套元素
                    // 子节点的 inline，典型 syntax-highlight `<span class=hljs-function><span
                    // class=hljs-title>…</span></span>`），部分 orphan（仅 eligible 子跳 taffy）
                    // 会扭曲剩余 taffy inline 的几何——移除 eligible 兄弟后，非 eligible inline 被
                    // taffy 当唯一 inline 子撑到全宽（morning-work 代码块 hljs-function 620×42 实测），
                    // 致 sibling-overlap 假阳性。要求 inline 子**全部** eligible 或 br/wbr（零宽换行
                    // 元素混排安全），否则整容器不 orphan（all-or-nothing 保几何一致）。
                    let has_non_eligible_inline = phasea_multi_inline_on
                        && children_dom.iter().any(|&c| {
                            let Some(node) = doc.get(c) else {
                                return false;
                            };
                            let NodeKind::Element(e) = &node.kind else {
                                return false;
                            };
                            if e.local_name().eq_ignore_ascii_case("br") || e.local_name().eq_ignore_ascii_case("wbr") {
                                return false;
                            }
                            styles.get(&c).is_some_and(|s| {
                                matches!(s.display, DisplayValue::Inline)
                                    && !phasea_multi_inline_eligible(doc, styles, c)
                            })
                        });
                    let multi_inline_block_skip =
                        phasea_multi_inline_on && eligible_inline_count >= 2 && !has_non_eligible_inline;
                    let mut children_with_order: Vec<(NodeId, i32)> = Vec::new();
                    for &child_dom in &children_dom {
                        let child_data = doc.get(child_dom);
                        if child_data.is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))) {
                            // R1311b：纯 inline 上下文的 `<br>`（无 block 同胞）且其父块有后续
                            // in-flow 兄弟（即重叠 bug 的精确触发条件）时，跳过 br 的 taffy 节点
                            // 让父 IFC 作 InlineItem::Br 处理。否则 br 作 0 高 Block leaf 使父块成
                            // new_with_children，taffy 按子 br=0 定父高、忽略 IFC measure 回调，
                            // 后续兄弟 margin-collapse-through 落父顶重叠（anonymous-boxes-001b /
                            // position-absolute-percentage-inherit-001）。末子 br 父块无后续兄弟，
                            // 跳过无益反引发容器高度连锁重排（welcome p.tagline），故要求「有后续
                            // in-flow 兄弟」精确 gate。br-between-blocks（R1285 strut）仍建节点。
                            // kill-switch ZW_BR_INLINE_NO_NODE=0 关闭（重建 br 节点=旧行为）。
                            if std::env::var("ZW_BR_INLINE_NO_NODE").as_deref() != Ok("0")
                                && br_is_inline_only(doc, styles, child_dom)
                                && br_parent_has_following_inflow_sibling(doc, styles, child_dom)
                            {
                                continue;
                            }
                            // R2156 Phase A inline-box-model coherence（env
                            // `ZW_INLINE_BOX_MODEL_COHERENCE`，default-on；`=0` kill）：当子元素
                            // display:inline 且含嵌套 atomic inline 后代（R1576 判定），不把其 taffy
                            // 节点作为块级子附加——让父 IFC 经 R1576 递归整体收集（文本 + 后代
                            // atomic inline），由 IFC 单次布局定位。解 37-form-controls
                            // `<p><label>text <input></label></p>`：label 被建为块级 taffy 子致兄弟
                            // label 盒重叠 + 父 IFC 又吸收其文本致串联（R109 inline-ownership 分裂）。
                            // ★ 三态 A/B 实测 net-positive（非净负）：全 10 目录 self-source reftest
                            // 零 delta（CSS2 5612=5612 / css-text 1742=1742 / writing-modes 631=631
                            // 等）+ css-position chromium-oracle 66=66 零漂移 + 37-form-controls oracle
                            // 4.33%→3.85% 结构 FAIL→PASS + welcome 字节一致 + legacy 套件 1→0 struct
                            // FAIL。故 default-on（kill-switch 保留）。
                            // ★ ooflow 守卫（关键）：若 inline 子树含 position:absolute/fixed 后代，
                            // 必须保留 taffy 子树供其 CB——否则 abspos 后代丢 CB（nested-inline-
                            // abspos-child 簇：`<div class=inline-content>` 同时 inline-block + absolute，
                            // 跳过外层 span 会把整棵子树丢出 taffy）。gate ON 无守卫 css-position -2；
                            // 加守卫后 83=83 零回归。限 horizontal-tb（vertical = R109-blocked）。
                            if std::env::var("ZW_INLINE_BOX_MODEL_COHERENCE").as_deref() != Ok("0")
                                && matches!(own_writing_mode, WritingModeValue::HorizontalTb)
                                && styles.get(&child_dom).is_some_and(|s| {
                                    matches!(s.display, DisplayValue::Inline)
                                        && !matches!(s.position, PositionValue::Absolute | PositionValue::Fixed)
                                })
                                && crate::inline::InlineFormattingContext::inline_elem_has_nested_inline_block(
                                    doc, styles, child_dom,
                                )
                                && !crate::inline::InlineFormattingContext::inline_subtree_has_ooflow_descendant(
                                    doc, styles, child_dom,
                                )
                                // R57（M3）：含 in-flow block-level 子元素的 inline 不跳过——
                                // 该 inline 须经 R109 §9.2.1.1 拆分（block 子独立 taffy 子树）。
                                // canvas-grid reftest 的 `<span><div>srgb</div><canvas></canvas></span>`
                                // 曾整棵被丢出 taffy 树（canvas 盒缺失 → 格子全空白，
                                // 2d.gradient.colorInterpolationMethod oracle A/B 10.7%）。
                                && !doc.child_nodes(child_dom).iter().any(|&gc| {
                                    doc.get(gc).is_some_and(|gn| {
                                        matches!(&gn.kind, NodeKind::Element(_))
                                            && styles.get(&gc).is_some_and(|gs| is_block_level_in_flow(&gs.display, &gs.position))
                                    })
                                })
                            {
                                continue;
                            }
                            // R2160 part1：multi-inline block 容器中 childless plain inline 跳过
                            // taffy 节点（orphan → owner_h=0 → painter R639 part2 per-fragment 绘 bg/border）。
                            if multi_inline_block_skip && phasea_multi_inline_eligible(doc, styles, child_dom) {
                                continue;
                            }
                            let order = styles.get(&child_dom).map_or(0, |s| s.order);
                            children_with_order.push((child_dom, order));
                        }
                    }

                    // 按 order 稳定排序（相同 order 保持 DOM 顺序）
                    children_with_order.sort_by_key(|(_, order)| *order);

                    for &(child_dom, _) in &children_with_order {
                        let child_taffy = build_subtree(
                            ctx,
                            doc,
                            styles,
                            child_dom,
                            grid_areas.as_ref(),
                            false,
                            own_writing_mode.clone(),
                            viewport_w,
                            viewport_h,
                        );
                        child_taffy_ids.push(child_taffy);
                    }
                }
            }
        }
    }

    // 创建 taffy 节点
    let taffy_id = if child_taffy_ids.is_empty() {
        ctx.taffy.new_leaf_with_context(taffy_style, dom_id).unwrap()
    } else {
        let id = ctx.taffy.new_with_children(taffy_style, &child_taffy_ids).unwrap();
        ctx.taffy.set_node_context(id, Some(dom_id)).unwrap();
        id
    };

    // 记录映射
    ctx.node_map.insert(dom_id, taffy_id);
    ctx.taffy_to_dom.insert(taffy_id, dom_id);

    taffy_id
}

/// 递归遍历 shadow 树，收集 taffy 子节点。
///
/// 遇到 `<slot>` 元素时：
/// - 有已分配节点 → 用分配的 light DOM 节点替换
/// - 无已分配节点 → 使用 slot 的回退子元素
///
/// 非 slot 元素正常递归调用 `build_subtree`（该元素自身可能有嵌套 shadow root）。
#[allow(clippy::too_many_arguments)]
fn collect_shadow_children(
    ctx: &mut BuildContext,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    shadow_root_id: NodeId,
    parent_grid_areas: Option<&GridAreaMap>,
    output: &mut Vec<taffy::NodeId>,
    writing_mode: &WritingModeValue,
    viewport_w: f32,
    viewport_h: f32,
) {
    let children = doc.get(shadow_root_id).map(|n| n.children.clone()).unwrap_or_default();
    for &child_id in &children {
        let child_data = match doc.get(child_id) {
            Some(d) => d,
            None => continue,
        };

        // 只处理元素节点
        let elem_data = match &child_data.kind {
            NodeKind::Element(elem) => elem,
            _ => continue,
        };

        // 检查是否为 <slot> 元素
        if elem_data.local_name() == "slot" {
            let assigned = doc.get_assigned_nodes(child_id);
            if !assigned.is_empty() {
                // 有分配的 light DOM 节点 → 替换 slot
                for &assigned_id in &assigned {
                    if doc
                        .get(assigned_id)
                        .is_some_and(|n| matches!(&n.kind, NodeKind::Element(_)))
                    {
                        let taffy_id = build_subtree(
                            ctx,
                            doc,
                            styles,
                            assigned_id,
                            parent_grid_areas,
                            false,
                            writing_mode.clone(),
                            viewport_w,
                            viewport_h,
                        );
                        output.push(taffy_id);
                    }
                }
            } else {
                // 无分配 → 使用 slot 的回退内容（slot 自身的子元素）
                let slot_children = doc.get(child_id).map(|n| n.children.clone()).unwrap_or_default();
                for &fallback_id in &slot_children {
                    if doc
                        .get(fallback_id)
                        .is_some_and(|n| matches!(&n.kind, NodeKind::Element(_)))
                    {
                        let taffy_id = build_subtree(
                            ctx,
                            doc,
                            styles,
                            fallback_id,
                            parent_grid_areas,
                            true,
                            writing_mode.clone(),
                            viewport_w,
                            viewport_h,
                        );
                        output.push(taffy_id);
                    }
                }
            }
        } else {
            // 非 slot 元素，递归进入 shadow 树子节点（in_shadow=true）
            let taffy_id = build_subtree(
                ctx,
                doc,
                styles,
                child_id,
                parent_grid_areas,
                true,
                writing_mode.clone(),
                viewport_w,
                viewport_h,
            );
            output.push(taffy_id);
        }
    }
}

/// 在 shadow 树内部遍历元素的子节点，处理 <slot> 替换。
///
/// 与 `collect_shadow_children` 类似，但起点是普通元素（非 ShadowRoot）。
/// 用于 shadow 树内部嵌套元素遍历其子节点时检查是否有 <slot> 需要替换。
#[allow(clippy::too_many_arguments)]
fn collect_shadow_slot_children(
    ctx: &mut BuildContext,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    parent_id: NodeId,
    parent_grid_areas: Option<&GridAreaMap>,
    output: &mut Vec<taffy::NodeId>,
    writing_mode: &WritingModeValue,
    viewport_w: f32,
    viewport_h: f32,
) {
    let children = doc.get(parent_id).map(|n| n.children.clone()).unwrap_or_default();
    for &child_id in &children {
        let child_data = match doc.get(child_id) {
            Some(d) => d,
            None => continue,
        };

        // 只处理元素节点
        let elem_data = match &child_data.kind {
            NodeKind::Element(elem) => elem,
            _ => continue,
        };

        // 检查是否为 <slot> 元素
        if elem_data.local_name() == "slot" {
            let assigned = doc.get_assigned_nodes(child_id);
            if !assigned.is_empty() {
                for &assigned_id in &assigned {
                    if doc
                        .get(assigned_id)
                        .is_some_and(|n| matches!(&n.kind, NodeKind::Element(_)))
                    {
                        let taffy_id = build_subtree(
                            ctx,
                            doc,
                            styles,
                            assigned_id,
                            parent_grid_areas,
                            false,
                            writing_mode.clone(),
                            viewport_w,
                            viewport_h,
                        );
                        output.push(taffy_id);
                    }
                }
            } else {
                let slot_children = doc.get(child_id).map(|n| n.children.clone()).unwrap_or_default();
                for &fallback_id in &slot_children {
                    if doc
                        .get(fallback_id)
                        .is_some_and(|n| matches!(&n.kind, NodeKind::Element(_)))
                    {
                        let taffy_id = build_subtree(
                            ctx,
                            doc,
                            styles,
                            fallback_id,
                            parent_grid_areas,
                            true,
                            writing_mode.clone(),
                            viewport_w,
                            viewport_h,
                        );
                        output.push(taffy_id);
                    }
                }
            }
        } else {
            // 非 slot 元素，继续在 shadow 树中递归
            let taffy_id = build_subtree(
                ctx,
                doc,
                styles,
                child_id,
                parent_grid_areas,
                true,
                writing_mode.clone(),
                viewport_w,
                viewport_h,
            );
            output.push(taffy_id);
        }
    }
}

#[cfg(test)]
mod tests;
