//! 从 DOM 树和计算样式构建 taffy 布局树。
//!
//! 提供将 DOM 元素节点与 taffy 节点关联的功能，
//! 跳过文本节点、注释节点和 display:none 的元素。

use std::collections::{HashMap, HashSet};
use taffy::prelude::*;
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::{ComputedStyle, WritingModeValue};

use crate::converter::{GridAreaMap, computed_style_to_taffy, parse_grid_template_areas};
use crate::inline_block_split::{
    InlineBlockSegment, block_container_has_mixed_content, compute_block_container_split, compute_inline_block_split,
    inline_has_block_child, is_whitespace_only_inline_segment,
};

/// R109 §9.2.1.1 生产端接线（匿名块生成 + fragment border）默认**启用**——经全量
/// reftest（+2 零回归：inline-box-001 / block-in-inline-align-001）+ 全量 make test
/// 验证。设 `R109_WIRE=0` 可关闭（回退到旧 inline→block 行为，仅用于对比/调试）。
fn r109_wired() -> bool {
    std::env::var("R109_WIRE").ok().as_deref() != Some("0")
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
) -> (TaffyTree<NodeId>, taffy::NodeId, HashMap<taffy::NodeId, NodeId>) {
    let (taffy, root_id, taffy_to_dom, _r109) =
        build_layout_tree_with_r109(doc, styles, viewport_width, viewport_height, img_intrinsic_sizes);
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
) -> (
    TaffyTree<NodeId>,
    taffy::NodeId,
    HashMap<taffy::NodeId, NodeId>,
    R109Wiring,
) {
    let mut ctx = BuildContext::new();
    ctx.img_intrinsic_sizes = img_intrinsic_sizes;

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
fn apply_replaced_element_sizing(
    taffy_style: &mut taffy::Style,
    computed: &ComputedStyle,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    dom_id: NodeId,
    img_intrinsic_sizes: &HashMap<NodeId, (f32, f32)>,
) {
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
    if tag != "img" && tag != "canvas" {
        return;
    }

    // 读取 HTML width/height 属性
    let attr_w = elem.get_attribute("width").and_then(|v| v.parse::<f32>().ok());
    let attr_h = elem.get_attribute("height").and_then(|v| v.parse::<f32>().ok());

    // 回退到 SVG data URI 内的固有尺寸
    let (attr_w, attr_h) = match (attr_w, attr_h) {
        (Some(w), Some(h)) => (Some(w), Some(h)),
        _ => {
            let (svg_w, svg_h) = extract_svg_data_uri_size(elem);
            (attr_w.or(svg_w), attr_h.or(svg_h))
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
                taffy_style.size.width = taffy::style::Dimension::Length(w);
                taffy_style.size.height = taffy::style::Dimension::Length(h);
            }
            // 一侧 auto、一侧显式：不设 auto 侧尺寸，taffy 按 aspect_ratio 推导
        }
        (Some(w), None) if w > 0.0 => {
            // 仅有 width：设置宽度，高度由 aspect_ratio 推导
            if computed.aspect_ratio.is_none() {
                // 无 aspect_ratio 也无 height，使用固定宽度
                if matches!(computed.width, LengthValue::Auto) {
                    taffy_style.size.width = taffy::style::Dimension::Length(w.max(1.0));
                }
            } else if matches!(computed.width, LengthValue::Auto) {
                taffy_style.size.width = taffy::style::Dimension::Length(w.max(1.0));
            }
        }
        (None, Some(h)) if h > 0.0 => {
            // 仅有 height：设置高度，宽度由 aspect_ratio 推导
            if computed.aspect_ratio.is_none() {
                if matches!(computed.height, LengthValue::Auto) {
                    taffy_style.size.height = taffy::style::Dimension::Length(h.max(1.0));
                }
            } else if matches!(computed.height, LengthValue::Auto) {
                taffy_style.size.height = taffy::style::Dimension::Length(h.max(1.0));
            }
        }
        _ => {
            // 无 HTML 属性：回退到解码后的固有尺寸（img_intrinsic_sizes）。
            // CSS 规范：替换元素无显式尺寸时使用固有尺寸（intrinsic size）。
            if let Some(&(w, h)) = img_intrinsic_sizes.get(&dom_id) {
                let w = w.max(1.0);
                let h = h.max(1.0);
                let width_auto = matches!(computed.width, LengthValue::Auto);
                let height_auto = matches!(computed.height, LengthValue::Auto);
                // R325：CSS §10 替换元素——仅当至少一侧为 auto 时才用固有宽高比推导另一侧。
                // 两侧都显式时【不得】设 aspect_ratio，否则 taffy 会强制比例，把显式 height
                // 拉到 width 比例（如 <img style="width:200px;height:50px"> 渲染成 200×200
                // 而非 200×50）。object-fit 控制内容如何填充 box，box 尺寸由两侧显式值决定。
                if computed.aspect_ratio.is_none() && (width_auto || height_auto) {
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
                let eff_ratio = computed.aspect_ratio.unwrap_or(w / h); // width/height
                if width_auto && height_auto {
                    taffy_style.size.width = taffy::style::Dimension::Length(w);
                    taffy_style.size.height = taffy::style::Dimension::Length(h);
                } else if !width_auto
                    && height_auto
                    && let LengthValue::Px(cw) = &computed.width
                {
                    // width 显式，height auto：height = cw / eff_ratio
                    taffy_style.size.height = taffy::style::Dimension::Length(((*cw as f32) / eff_ratio).max(0.5));
                } else if width_auto
                    && !height_auto
                    && let LengthValue::Px(ch) = &computed.height
                {
                    // height 显式，width auto：width = ch * eff_ratio
                    taffy_style.size.width = taffy::style::Dimension::Length(((*ch as f32) * eff_ratio).max(0.5));
                }
                // 两侧都显式：由 converter 从 CSS 处理，不干预
            }
        }
    }

    // CSS Flexbox §4.5 / csswg #5663：替换元素 flex item 的 min-size:auto。
    // taffy 0.7 把 leaf flex item 的 auto-min 当作其 definite 主尺寸（width:999→min 999），
    // 致替换 flex item 无法 flex-shrink（flex-minimum-width-flex-items-013 82% diff：
    // img 999×500 溢出 flex width:0 容器，应被 min-width:auto=100 floor 后收缩到 100）。
    // spec：auto-min = min(content suggestion, transferred suggestion)，transferred =
    // 明确 cross size × 固有比。此处仅当父是 flex 容器且有明确 cross size 时计算
    // transferred 并设 min_size.main（仅在水平书写模式，保守避免 writing-mode 轴混淆）。
    apply_flex_transferred_min_size(
        taffy_style,
        computed,
        doc,
        styles,
        dom_id,
        img_intrinsic_sizes.get(&dom_id).copied(),
    );
}

/// 替换元素 flex item 的 min-size:auto transferred-size-suggestion（CSS Flexbox §4.5 / csswg #5663）。
///
/// taffy 0.7 把 leaf（替换元素）flex item 的自动最小尺寸当作其 definite 主尺寸本身
/// （如 `width:999px` → min-width 999），导致替换 flex item 无法被 flex-shrink 收缩。
/// 规范要求自动最小 = min(content size suggestion, transferred size suggestion)，
/// 其中 transferred = flex item 的「明确 cross size」× 固有宽高比。例如
/// `<img 固有 300×150>` 在 `display:flex;height:50px` 容器内：cross(height)=50，
/// transferred main(width) = 50 × 300/150 = 100px。
///
/// 仅当父是 flex/inline-flex 容器、有明确 cross size（Px）、子有 aspect_ratio、且
/// 子在水平书写模式下时计算并设置 `min_size.main = min(intrinsic_main, transferred)`。
/// 其余情况（非 flex / cross 不明确 / 无 ratio / 垂直书写模式）保持 taffy 默认，避免回归。
fn apply_flex_transferred_min_size(
    taffy_style: &mut taffy::Style,
    computed: &ComputedStyle,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    dom_id: NodeId,
    intrinsic: Option<(f32, f32)>,
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
    // ★ 仅 row 实现（column transferred 在 flex-aspect-ratio-img-column-012 /
    // flex-minimum-height-flex-items-007 致回归，column 轴的 cross-stretch 语义需额外验证，
    // 待多 session）。column 直接返回，保持 taffy 默认。
    let is_column = matches!(
        parent_style.flex_direction,
        FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
    );
    if is_column {
        return;
    }
    // 父容器的明确 cross size（Px）。仅 Px 算「明确」；百分比/auto 不算（无法解 transferred）。
    let cross = if is_column {
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
    let cross = match cross {
        Some(c) if c > 0.0 => c,
        _ => return,
    };
    // §4.5：transferred-size-suggestion 仅当 item 的 cross size「明确」（即被 align-stretch
    // 拉到容器 cross size）时成立。item 有 auto cross-margin（margin:auto 居中而非拉伸）
    // 或显式非 stretch 的 align-self（center/flex-start/...）时 cross size 不等于容器
    // cross → 跳过（auto-margins-002 回归：img margin:auto + max-width，transferred 会
    // 与 max 冲突）。row cross=height→margin-top/bottom；column cross=width→margin-left/right。
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
    // transferred main size：row → cross_h × ratio(w/h)；column → cross_w / ratio
    let transferred = if is_column { cross / ratio } else { cross * ratio };
    // content suggestion proxy = 固有主尺寸（若有）
    let intrinsic_main = intrinsic.map(|(w, h)| if is_column { h } else { w });
    let mut auto_min = match intrinsic_main {
        Some(im) => im.min(transferred),
        None => transferred,
    };
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
            taffy_style.min_size.height = taffy::style::Dimension::Length(auto_min);
        } else {
            taffy_style.min_size.width = taffy::style::Dimension::Length(auto_min);
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

    // 替换元素固有尺寸：检测 <img> 元素并注入 HTML 属性中的 width/height，
    // 无属性时回退到解码后的固有尺寸（img_intrinsic_sizes）
    apply_replaced_element_sizing(
        &mut taffy_style,
        &computed,
        doc,
        styles,
        dom_id,
        &ctx.img_intrinsic_sizes,
    );

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
    let is_vertical = matches!(
        parent_writing_mode,
        WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
    );
    if is_vertical {
        crate::converter::apply_vertical_writing_mode(&mut taffy_style);
    }

    // 记录此元素的 writing mode，用于子元素轴交换判断
    let own_writing_mode = computed.writing_mode.clone();

    // 收集需要创建 taffy 节点的子元素
    // 当元素有 ShadowRoot 时，遍历 shadow 树而非 light DOM 子节点；
    // shadow 树中的 <slot> 元素替换为已分配的 light DOM 节点（或回退内容）。
    let mut child_taffy_ids: Vec<taffy::NodeId> = Vec::new();

    if let Some(shadow_id) = doc.shadow_root(dom_id) {
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
                            let anon_taffy = if is_block_mixed {
                                // block 容器匿名块：plain Block（不继承容器盒模型，容器
                                // 自身的 bg/border/padding 仍由容器盒绘制）。
                                let anon_style = taffy::Style {
                                    display: taffy::style::Display::Block,
                                    ..taffy::Style::default()
                                };
                                ctx.taffy
                                    .new_leaf_with_context(anon_style, ctx_node)
                                    .unwrap_or_else(|_| ctx.taffy.new_leaf(taffy::Style::default()).unwrap())
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
                                ctx.taffy
                                    .new_leaf_with_context(anon_style, ctx_node)
                                    .unwrap_or_else(|_| ctx.taffy.new_leaf(taffy::Style::default()).unwrap())
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
                // 非 flex/grid 容器：仅处理元素子节点（原有行为）
                let mut children_with_order: Vec<(NodeId, i32)> = Vec::new();
                for &child_dom in &children_dom {
                    let child_data = doc.get(child_dom);
                    if child_data.is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))) {
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
