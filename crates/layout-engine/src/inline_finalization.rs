//! 行内格式化（IFC）终化逻辑。
//!
//! 从 `engine.rs` 抽出（R342，2000 行规则 + Phase A IFC 统一 Phase 5 准备）。
//! 包含：compute_final_inline_layouts（存储权威行盒）、font-size/ascent 度量存储、
//! 文本测量（measure_text_content）、float 排斥下的重新测量。Phase A 的目标代码集中于此模块。

use std::collections::HashMap;
use taffy::prelude::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue, VerticalAlignValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::ComputedStyle;

use crate::inline::{FloatExclusion, InlineFormattingContext, TextAlign};
use crate::types::LayoutBox;
use zero_style_system::WritingModeValue;

/// R1099 Slice α-1 decoration-gate：vertical 容器子树是否含 text-decoration 或
/// text-emphasis（非 None）。
///
/// 用于 `container_width` WM-aware fix 的 gate——回避 Layer 4 装饰坐标耦合：
/// α-3（vertical 装饰坐标 re-enable）未实施前，vertical 装饰仍按水平绘制，
/// 若此时 container_width fix 让 vertical 文本列化（chars 同 x 列、y 递增），
/// 装饰仍水平绘制 → mismatch，css-text-decor -22 回归（R1099 A/B 实测）。
/// 有装饰的 vertical 容器保持 `content_width`（旧行为），等 α-3 后解除 gate。
///
/// 递归扫描 `root_id` 子树所有元素的 `text_decoration_line` / `text_emphasis_style`。
pub fn subtree_has_text_decoration(doc: &Document, styles: &HashMap<NodeId, ComputedStyle>, root_id: NodeId) -> bool {
    use zero_style_system::property::types::{TextDecorationLineValue, TextEmphasisStyleValue};
    fn scan(doc: &Document, styles: &HashMap<NodeId, ComputedStyle>, id: NodeId) -> bool {
        if let Some(s) = styles.get(&id)
            && (!matches!(s.text_decoration_line, TextDecorationLineValue::None)
                || !matches!(s.text_emphasis_style, TextEmphasisStyleValue::None))
        {
            return true;
        }
        for child_id in doc.child_nodes(id) {
            if scan(doc, styles, child_id) {
                return true;
            }
        }
        false
    }
    scan(doc, styles, root_id)
}

/// 解析 `text-indent` 为像素值（CSS §10.3.1）。
///
/// 支持 Px / Em（× font_size）/ Percentage（× container_width）。其他单位回退 0。
/// font_size 由 ComputedStyle.font_size（通常已 compute 到 Px）解析；Em 嵌套以父 font-size 为准。
/// 与 paint 路径（`painter/text.rs`）的 text_indent 解析保持一致（IFC 双路径同源）。
pub fn resolve_text_indent(text_indent: &LengthValue, font_size: &LengthValue, container_width: f32) -> f32 {
    let font_size_px = match font_size {
        LengthValue::Px(v) => *v as f32,
        _ => 16.0, // computed font_size 应为 Px；防御性回退
    };
    match text_indent {
        LengthValue::Px(v) => *v as f32,
        LengthValue::Em(v) => *v as f32 * font_size_px,
        LengthValue::Percentage(v) => *v as f32 / 100.0 * container_width,
        _ => 0.0,
    }
}

/// 从 ComputedStyle 读取 text-align 并转换为 IFC 的 TextAlign 枚举。
///
/// `start`/`end` 是**方向感知**值（CSS Text 3 §6.1）：`start` = inline-start 边
/// （LTR→left, RTL→right），`end` 反之。旧实现无条件 start→left/end→right，
/// 致 `direction:rtl` + `text-align:start` 错误左对齐（应右对齐）。
pub fn resolve_text_align(style: Option<&ComputedStyle>) -> TextAlign {
    use zero_style_system::property::{DirectionValue, TextAlignValue};
    let s = match style {
        Some(s) => s,
        None => return TextAlign::Left, // 默认 Start 在 LTR 下 = Left
    };
    let is_rtl = matches!(s.direction, DirectionValue::Rtl);
    match s.text_align {
        TextAlignValue::Left => TextAlign::Left,
        TextAlignValue::Right => TextAlign::Right,
        TextAlignValue::Center => TextAlign::Center,
        TextAlignValue::Justify => TextAlign::Justify,
        TextAlignValue::Start => {
            if is_rtl {
                TextAlign::Right
            } else {
                TextAlign::Left
            }
        }
        TextAlignValue::End => {
            if is_rtl {
                TextAlign::Left
            } else {
                TextAlign::Right
            }
        }
    }
}

/// 从 ComputedStyle 读取 text-align-last 并转换为 IFC 的 `Option<TextAlign>`。
/// `Auto` → `None`（末行跟随 text-align，如 justify 末行回退 Left）；其余映射同 resolve_text_align。
///
/// **修复**：`compute_final_inline_layouts` 构建 stored IFC 此前只传 `text_align` 漏传
/// `text_align_last`，致**存储路径**（pure-Ahem 容器，如 justifyall 簇）末行 text-align-last
/// 不应用——末行恒按 text-align 默认处理。paint 非存储路径已传（text.rs:949），此处补齐使
/// layout/paint 双路径一致。
pub fn resolve_text_align_last(style: Option<&ComputedStyle>) -> Option<TextAlign> {
    use zero_style_system::property::{DirectionValue, TextAlignLastValue};
    let s = style?;
    let is_rtl = matches!(s.direction, DirectionValue::Rtl);
    match s.text_align_last {
        TextAlignLastValue::Auto => None,
        TextAlignLastValue::Left => Some(TextAlign::Left),
        TextAlignLastValue::Right => Some(TextAlign::Right),
        TextAlignLastValue::Center => Some(TextAlign::Center),
        TextAlignLastValue::Justify => Some(TextAlign::Justify),
        // start/end 方向感知，与 resolve_text_align 一致（CSS Text 3 §6.2）。
        TextAlignLastValue::Start => Some(if is_rtl { TextAlign::Right } else { TextAlign::Left }),
        TextAlignLastValue::End => Some(if is_rtl { TextAlign::Left } else { TextAlign::Right }),
    }
}

/// 构建「文本节点 → 父元素」override map（IFC Path A/B 共享，R2189）。
///
/// 把 `LayoutBox.text_node_*` HashMap（键 = 文本节点 NodeId）重映射为「父元素 NodeId → 值」，
/// 仅保留文本节点片段——`text_node_*` 混入了内联元素片段（如 `<img>`，font_size=0、height=96），
/// 它们与文本片段共享同一父元素，直接 collect 时 last-write-wins，结果随 HashMap 迭代顺序
///（每进程随机）变化 → 渲染非确定性；过滤为纯文本节点后结果确定。
///
/// Path A（`compute_final_inline_layouts` stored IFC）与 Path B（paint 重跑 IFC）此逻辑字节一致，
/// 提取为共享 helper 消除 7 处重复（同 R2187/R2188 text-align / text-indent DRY 谱系）。
/// 注：Path B 的 `is_ahem`（multicol flatten 元素自键 else 分支）与 `text_transforms`（None 过滤）
/// 逻辑不同，未走此 helper，仍各自内联。
pub fn build_text_parent_override_map<T: Copy>(doc: &Document, source: &HashMap<NodeId, T>) -> HashMap<NodeId, T> {
    source
        .iter()
        .filter_map(|(&tn, &v)| {
            if !matches!(doc.get(tn).map(|n| &n.kind), Some(NodeKind::Text(_))) {
                return None;
            }
            doc.parent_node(tn).map(|pid| (pid, v))
        })
        .collect()
}

/// R645：从 ComputedStyle 读取 white-space，返回 IFC 测量所需的 `no_wrap`。
///
/// `measure_text_content` / `remeasure_*` 测量函数此前构造 IFC 时未传 white-space（no_wrap 恒
/// false），致 pre/nowrap 容器在**测量高度**时被错误换行（box content_height 偏大）。该 bug
/// 长期被「单 token 无法换行」掩盖；R645 SEA 词典分词文字的 per-char fallback breaking 使每个
/// 字符成为独立断行点，暴露了 pre 容器被测成多行（line-breaking-024/026/027 mismatch test/ref
/// 同错→0.00%）。
///
/// **仅传 `no_wrap`，不传 `preserve_whitespace`**：preserve 会改变 pre-wrap 空白折叠行为，实测致
/// letter-spacing-201（pre-wrap + 多空格）测量行断变化而回归。no_wrap 是修 pre/nowrap 容器「测量
/// 时不换行」的最小充分条件；preserve 仅影响空白折叠，对 box 高度测量非必要，故测量路径保持
/// preserve=false（与历史行为一致），空白保真由 paint 路径（text.rs:744）负责。
pub(crate) fn resolve_no_wrap_for_ifc_measure(style: Option<&ComputedStyle>) -> bool {
    use zero_style_system::property::types::WhiteSpaceValue;
    matches!(
        style.map(|s| &s.white_space).unwrap_or(&WhiteSpaceValue::Normal),
        WhiteSpaceValue::Pre | WhiteSpaceValue::Nowrap
    )
}

/// R1935：measure 期是否保留空白序列（pre/pre-wrap）。镜像 `resolve_no_wrap_for_ifc_measure`。
///
/// R645 仅传 no_wrap 不传 preserve（letter-spacing-201 pre-wrap 多空格回归），但 binary-search
///（R1935）实证 `<pre>` 容器（white-space-pre-001）因 measure 不传 preserve 被测成 1 行
///（content \n 被折叠，应 5 行）→ box 高度错。本函数让 measure 期也保留 pre/pre-wrap 空白，
/// 修 `<pre>`/pre-wrap 容器测量行数。kill-switch `ZW_MEASURE_PRESERVE=0` 关闭（恢复 R645 行为）。
pub(crate) fn resolve_preserve_for_ifc_measure(style: Option<&ComputedStyle>) -> bool {
    use zero_style_system::property::types::WhiteSpaceValue;
    matches!(
        style.map(|s| &s.white_space).unwrap_or(&WhiteSpaceValue::Normal),
        WhiteSpaceValue::Pre | WhiteSpaceValue::PreWrap
    )
}

/// R1935：measure 期是否在 `\n` 处强制断行（pre-line，与 layout 派生一致）。
pub(crate) fn resolve_break_at_newline_for_ifc_measure(style: Option<&ComputedStyle>) -> bool {
    use zero_style_system::property::types::WhiteSpaceValue;
    matches!(
        style.map(|s| &s.white_space).unwrap_or(&WhiteSpaceValue::Normal),
        WhiteSpaceValue::PreLine
    )
}

/// 将 IFC 片段结果存储到 LayoutBox.inline_layout，供 paint 系统复用。
///
/// ⚠️ **死代码，保持不启用（R1526 调研定论）**：
/// - 存储职责已由 `compute_final_inline_layouts`（本文件 line 521）inline 实现
///   （line 933 `root.inline_layout = Some(lines)` + line 872 `store_font_sizes_from_ifc`），
///   wiring 本函数要么 redundant（存储已发生），要么无对应调用场景。
/// - broad-authoritative-storage 机制（paint 经 `use_stored` 复用 layout 行盒、不再重跑 IFC）
///   经 R1487 env-gate A/B 决定性证伪（normal-flow NET -7 revert）：layout 行断用
///   `estimate_char_width` 与 chromium 分歧**大于** paint Path B 的 fontdue 重跑，故强制
///   paint 复用 layout 结果**更差**而非更好。narrow ascent/baseline override 变体亦
///   net-negative（R1194 / R1206 NET -22 / R1208）。
///
/// 旧 TODO「基线计算修复后启用」hereby 作废——勿再以 wiring 本函数为 Phase A lever。
#[allow(dead_code)]
pub(crate) fn store_inline_layout_results(
    inline_ctx: &crate::inline::InlineFormattingContext,
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    if !inline_ctx.lines.is_empty() {
        let container_width = box_node.content_width;
        let stored: Vec<crate::types::InlineLayoutLine> = inline_ctx
            .lines
            .iter()
            .map(|line| crate::types::InlineLayoutLine {
                y: line.y,
                height: line.height,
                baseline_y: line.baseline_y,
                ascent: line.ascent,
                descent: line.descent,
                fragments: line
                    .runs
                    .iter()
                    .map(|frag| {
                        let is_ahem = box_node.node_id.is_some_and(|id| {
                            styles
                                .get(&id)
                                .is_some_and(|s| s.font_family.contains(&"Ahem".to_string()))
                        });
                        crate::types::InlineLayoutFragment {
                            x: frag.x,
                            y: frag.y,
                            width: frag.width,
                            height: frag.height,
                            font_size: frag.font_size,
                            is_ahem,
                            is_ahem_font: frag.is_ahem,
                            text: frag.text.clone(),
                            node_id: Some(frag.node_id),
                            // R816 Phase 1：片段基线 = 行基线（baseline 对齐片段）。
                            baseline_y: line.baseline_y,
                        }
                    })
                    .collect(),
            })
            .collect();
        box_node.inline_layout = Some(stored);
        box_node.inline_layout_width = container_width;
    }
}

/// R900：env `MULTICOL_COLUMN_FRAG` 门控——为 inline-only `column-fill:auto` + 明确高度
/// multicol 容器按列宽重排 IFC、`fragment_lines_into_columns_overflow` 分布行盒到列（溢出时
/// 创建溢出列，R1429）、重定位后存入 `inline_layout`，使 paint `use_stored` 按列渲染。
///
/// 返回 `true` 表示已存储列分布行盒（调用方早返回）；`false` 表示非目标结构（调用方走默认路径）。
///
/// 目标结构（R897/R900 实证真缺口）：单层 multicol + `column-fill:auto` + 明确高度 + 无 block 子
/// （直接 inline 文本）。当前 ZW 把此类容器渲染为**单个全宽列**（multicol.rs 仅处理 block 子，
/// inline 内容从不分布到列）——`multicol-fill-auto-001` self-source 9.41% 真失败。
fn store_inline_multicol_columns(
    root: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> bool {
    use crate::inline::{
        ColumnFillMode, ColumnFragmentationContext, InlineFormattingContext, fragment_lines_into_columns_overflow,
    };

    let node_id = match root.node_id {
        Some(id) => id,
        None => return false,
    };
    let style = match styles.get(&node_id) {
        Some(s) => s,
        None => return false,
    };
    // column-fill:auto 或 balance + 列数≥2。
    // R1423：balance 也进入本函数以填充 text_node_is_ahem 等（paint 侧重跑 IFC 需要），
    // 但不为 balance 存列分布——paint 对 balance 用自己的 multicol_info 重跑（use_stored=false），
    // 存了也被忽略，且 R902/R1422 证 balance 存列布局 net-negative。仅 auto 存列分布。
    let info = match crate::multicol::compute_column_info(style, root.content_width) {
        Some(i) if i.count >= 2 => i,
        _ => return false,
    };
    // inline-only：无 in-flow block-level 子（block 子走 multicol.rs assign_children 路径）
    let has_block_child = root
        .children
        .iter()
        .any(|c| c.is_block_level && !c.is_absolute && !c.is_fixed);
    if has_block_child {
        return false;
    }
    // 列宽重排 IFC（目标案为简单文本；复杂 override 留后续扩展）
    let mut col_ctx = InlineFormattingContext::new(info.column_width);
    col_ctx.layout(doc, node_id, styles);
    if col_ctx.lines.is_empty() {
        return false;
    }
    // R1423：填充 text_node_is_ahem / font_sizes / letter_spacing / line_heights 等度量，
    // 供 paint 侧重跑 IFC（balance 模式 multicol_info 路径，use_stored=false）获得正确字体
    // 度量。此前 multicol 早返回跳过 store_font_sizes_from_ifc → text_node_is_ahem 空 →
    // paint IFC is_ahem=false（Ahem 'x' 估 11px 应 20px）→ 列宽下少换行 → 列欠填
    // （multicol-columns-001 仅渲染 42% 文本，22 行应 44）。auto 模式 use_stored=true
    // 走存布局路径，本填充对其无影响（无消耗）。
    store_font_sizes_from_ifc(&col_ctx, root, doc, styles);
    // balance 模式：度量已填，不存列分布，让 paint 用正确度量重跑（avoid R902/R1422 回归）。
    if !info.sequential_fill {
        return false;
    }
    // auto 模式：列高预算 + 顺序填列 + 存列分布。
    // 列高预算：明确 height 优先，否则 max-height，最后回退 content_height。
    // R905：max-height 容器（height:auto）的 content_height 来自全宽 IFC（偏小，列更窄→更多行），
    // 须用 max-height 作 budget；分布后再修正容器高度。columnfill-auto-max-height-001：
    // max-height:100px 但 content_height=50（全宽 2 行），列宽（100px）下应为 4 行=100px。
    let (available_height, from_max_height) = match (&style.height, &style.max_height) {
        (LengthValue::Px(h), _) => (*h as f32, false),
        (_, LengthValue::Px(m)) => (*m as f32, true),
        _ => (root.content_height, false),
    };
    if available_height <= 0.0 {
        return false;
    }
    // 分布行盒到列（整行不裁断，列高 respected）。R1429：用 overflow 变体——内容溢出
    // column-count 时创建溢出列（CSS Multicol §8.2：column-fill:auto + 定高 + 溢出 →
    // 溢出列在容器内容边外水平延伸，column-rule 在每个间隙绘制）。
    let ctx = ColumnFragmentationContext {
        col_count: info.count,
        col_width: info.column_width,
        col_gap: info.gap,
        available_height: Some(available_height),
        col_filled_heights: vec![0.0; info.count],
        fill_mode: ColumnFillMode::Auto,
    };
    let (assignments, total_col_count) = fragment_lines_into_columns_overflow(&col_ctx.lines, &ctx);
    if assignments.is_empty() {
        return false;
    }
    // 重定位存储：line.y = y_in_column（各列均从容器内容顶 0 起）；
    // 每个 fragment.x += col_idx × (col_width + gap)（横向偏移到对应列）。
    let is_ahem_container = style.font_family.iter().any(|f| f.eq_ignore_ascii_case("Ahem"));
    let stored: Vec<crate::types::InlineLayoutLine> = assignments
        .iter()
        .map(|a| {
            let line = &col_ctx.lines[a.line_idx];
            let col_x_offset = a.column as f32 * (info.column_width + info.gap);
            crate::types::InlineLayoutLine {
                y: a.y_in_column,
                height: line.height,
                baseline_y: line.baseline_y,
                ascent: line.ascent,
                descent: line.descent,
                fragments: line
                    .runs
                    .iter()
                    .map(|frag| crate::types::InlineLayoutFragment {
                        x: frag.x + col_x_offset,
                        y: frag.y,
                        width: frag.width,
                        height: frag.height,
                        font_size: frag.font_size,
                        is_ahem: is_ahem_container,
                        is_ahem_font: frag.is_ahem,
                        text: frag.text.clone(),
                        node_id: Some(frag.node_id),
                        baseline_y: line.baseline_y,
                    })
                    .collect(),
            }
        })
        .collect();
    if stored.is_empty() {
        return false;
    }
    // R905：max-height 容器（height:auto）的 content_height 来自全宽 IFC（偏小），分布后须用
    // 最高列累计高度修正容器高度（否则下方行盒被容器高度裁剪不可见）。
    // R1429：col_heights 须按 total_col_count（含溢出列）开长，否则 a.column ≥ info.count 越界。
    if from_max_height {
        let mut col_heights = vec![0.0f32; total_col_count];
        for a in &assignments {
            let line_h = col_ctx.lines[a.line_idx].height;
            col_heights[a.column] = col_heights[a.column].max(a.y_in_column + line_h);
        }
        let tallest = col_heights.into_iter().fold(0.0f32, f32::max);
        if tallest > root.content_height {
            let delta = tallest - root.content_height;
            root.content_height = tallest;
            root.height += delta;
        }
    }
    // R1429：内容溢出 column-count → 存实际列数（含溢出列），供 paint_column_rules 在每个
    // 间隙（含溢出间隙）绘制 column-rule。仅溢出时置 Some（无溢出 None → paint 用 style count，零回归）。
    if total_col_count > info.count {
        root.multicol_overflow_column_count = Some(total_col_count as u32);
    }
    root.inline_layout = Some(stored);
    // inline_layout_width = 容器内容宽（使 paint width_matches → use_stored=true，按列渲染）
    root.inline_layout_width = root.content_width;
    true
}

/// 从 IFC 片段中提取各文本节点的 font_size、is_ahem 标志、letter-spacing 和 line-height 并存储到 LayoutBox。
///
/// paint 系统在运行空 styles IFC 时无法获取正确的 font_size、字体信息、letter-spacing 和 line-height，
/// 导致基线偏移、字符宽度、间距和行盒高度计算错误。通过此函数存储 layout IFC 的相关值，
/// paint 可以在渲染时使用正确的值。
///
/// `doc` + `styles` 用于 R1012 text-transform 覆盖：按片段的文本节点 NodeId 查父元素
/// computed text-transform 存入 `text_node_text_transform`，paint Path B 据此在空 styles
/// 下应用 transform（行断用转换后宽度）。
pub(crate) fn store_font_sizes_from_ifc(
    inline_ctx: &crate::inline::InlineFormattingContext,
    box_node: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    for line in &inline_ctx.lines {
        for frag in &line.runs {
            box_node.text_node_font_sizes.insert(frag.node_id, frag.font_size);
            // R1464：per-fragment font-family（key = frag.node_id，element 或 text node）。
            // owner 元素 = frag.node_id（若 element）或其父（若 text node）。Path B 空 styles
            // 无 per-fragment font-family → 非-Ahem webfont/跨字体 inline 回落容器字体。
            let font_owner = if doc
                .get(frag.node_id)
                .is_some_and(|n| matches!(n.kind, NodeKind::Element(_)))
            {
                Some(frag.node_id)
            } else {
                doc.parent_node(frag.node_id)
            };
            let family = font_owner
                .and_then(|oid| styles.get(&oid))
                .map(|s| s.font_family.clone())
                .unwrap_or_default();
            box_node.text_node_font_families.insert(frag.node_id, family);
            box_node.text_node_is_ahem.insert(frag.node_id, frag.is_ahem);
            box_node
                .text_node_letter_spacing
                .insert(frag.node_id, frag.letter_spacing);
            // line-height 不影响行断（仅影响垂直定位），传递到 paint IFC 是安全的。
            // 使用片段的 height 作为行盒高度贡献（已含 line-height + padding + border）。
            box_node.text_node_line_heights.insert(frag.node_id, frag.height);
            // R1012：存 text-transform（按文本节点 NodeId）。paint Path B 重跑 IFC 时
            // styles 为空，据此映射构造 text_transform_overrides 让 collect_inline_items
            // 在空 styles 下应用 transform。仅对真正的文本节点存（其父元素 style 携带
            // 继承的 text-transform）；inline 元素片段跳过（无对应 DOM 文本节点父链）。
            if doc
                .get(frag.node_id)
                .is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
            {
                if let Some(pid) = doc.parent_node(frag.node_id) {
                    let transform = styles
                        .get(&pid)
                        .map(|s| s.text_transform)
                        .unwrap_or(zero_style_system::TextTransformValue::None);
                    box_node.text_node_text_transform.insert(frag.node_id, transform);
                }
            }
            // 内联元素片段（node_id 是元素 NodeId 而非文本节点 NodeId）：
            // 存储其 (font_size, line_height) 供 paint IFC 使用。
            // 内联元素在 paint IFC 中无法获取自己的样式，导致使用默认值。
            // line_height 近似使用 height（对文本片段来说等于 run.line_height）。
            box_node
                .inline_element_metrics
                .insert(frag.node_id, (frag.font_size, frag.height));
            // 内联元素的水平 margin 不影响行断（仅影响水平偏移），传递到 paint IFC 是安全的。
            box_node
                .inline_element_margins
                .insert(frag.node_id, (frag.margin_left, frag.margin_right));
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct InlineVisualMetrics {
    padding_top: f32,
    padding_right: f32,
    padding_bottom: f32,
    padding_left: f32,
    border_top: f32,
    border_right: f32,
    border_bottom: f32,
    border_left: f32,
}

pub(crate) fn resolve_px_length(value: &LengthValue) -> f32 {
    match value {
        LengthValue::Px(v) => *v as f32,
        _ => 0.0,
    }
}

pub(crate) fn extract_inline_visual_metrics(style: &ComputedStyle) -> InlineVisualMetrics {
    InlineVisualMetrics {
        padding_top: resolve_px_length(&style.padding_top),
        padding_right: resolve_px_length(&style.padding_right),
        padding_bottom: resolve_px_length(&style.padding_bottom),
        padding_left: resolve_px_length(&style.padding_left),
        border_top: resolve_px_length(&style.border_top_width),
        border_right: resolve_px_length(&style.border_right_width),
        border_bottom: resolve_px_length(&style.border_bottom_width),
        border_left: resolve_px_length(&style.border_left_width),
    }
}

/// 将 IFC 计算出的直接 inline 子元素几何写回 LayoutBox。
///
/// 仅处理「单个 fragment 即可完整表示」的简单 inline 元素：
/// - `display:inline`
/// - 非 absolute/fixed
/// - 在当前 IFC 中恰好对应一个 fragment
///
/// 这样可以让 paint 阶段使用更接近真实 inline box 的几何去绘制背景/边框，
/// 避免 taffy 将 inline 元素当作 block 后得到的零尺寸或错误尺寸。
pub(crate) fn sync_inline_child_boxes_from_ifc(
    box_node: &mut LayoutBox,
    inline_ctx: &InlineFormattingContext,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    let fragments = inline_ctx.all_fragments_with_line_y();

    for child in &mut box_node.children {
        if child.is_block_level || child.is_absolute || child.is_fixed {
            continue;
        }

        let Some(child_id) = child.node_id else {
            continue;
        };
        let Some(style) = styles.get(&child_id) else {
            continue;
        };
        if !matches!(style.display, DisplayValue::Inline) {
            continue;
        }

        let mut matching = fragments.iter().filter(|fragment| fragment.node_id == child_id);
        let Some(fragment) = matching.next() else {
            continue;
        };
        if matching.next().is_some() {
            continue;
        }
        // 跳过含文本内容的 fragment：
        // 文本 fragment 的位置来自 layout IFC（使用真实样式），
        // 而 paint 阶段运行独立的 paint IFC（使用空样式），
        // 两者行断行为不同，直接使用 layout IFC 坐标会导致背景与文字错位。
        // 仅对空 inline 元素（零宽度 TextRun）应用几何修正。
        if !fragment.text.is_empty() {
            continue;
        }

        let metrics = extract_inline_visual_metrics(style);
        child.x = fragment.x;
        child.y = fragment.y - metrics.padding_top - metrics.border_top;
        child.width =
            fragment.width + metrics.padding_left + metrics.padding_right + metrics.border_left + metrics.border_right;
        child.height =
            fragment.height + metrics.padding_top + metrics.padding_bottom + metrics.border_top + metrics.border_bottom;
        child.content_x = metrics.border_left + metrics.padding_left;
        child.content_y = metrics.border_top + metrics.padding_top;
        child.content_width = fragment.width;
        child.content_height = fragment.height;
        child.padding_top = metrics.padding_top;
        child.padding_right = metrics.padding_right;
        child.padding_bottom = metrics.padding_bottom;
        child.padding_left = metrics.padding_left;
        child.border_top = metrics.border_top;
        child.border_right = metrics.border_right;
        child.border_bottom = metrics.border_bottom;
        child.border_left = metrics.border_left;
    }
}

/// 为含有直接文本子节点的容器计算最终行内布局并存储 IFC 片段结果。
/// paint 系统消费这些结果渲染文字，不再重跑 IFC。
///
/// 使用与 paint-IFC 相同的空样式 + override maps 上下文，
/// 确保存储结果与 paint 路径完全一致，零回归。
pub(crate) fn compute_final_inline_layouts(
    root: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    ancestor_floats: &[crate::inline::FloatExclusion],
    img_intrinsic_sizes: &HashMap<NodeId, (f32, f32)>,
    font_metric_provider: Option<&crate::inline::FontMetricProviderHandle>,
) {
    // R362：先收集本容器直接 float 子（既用于自身 IFC 排除，也向后代传播）。
    // 坐标系：c.y 是 float 子相对 root box 顶部的位置，与 IFC 行 y（0=root box 顶）一致。
    // 携带 node_id 以便递归时排除子节点自身（float 不应在自身 IFC 中排除自己）。
    let own_floats: Vec<(zero_dom::NodeId, crate::inline::FloatExclusion)> = root
        .children
        .iter()
        .filter(|c| !matches!(c.float, zero_css_parser::values::FloatValue::None))
        .filter_map(|c| {
            let rel_y = c.y;
            if rel_y < 0.0 || c.width <= 0.0 || c.height <= 0.0 {
                return None;
            }
            let id = c.node_id?;
            Some((
                id,
                crate::inline::FloatExclusion {
                    y: rel_y + c.margin_top,
                    height: c.height + c.margin_bottom,
                    width: c.width + c.margin_left + c.margin_right,
                    is_left: matches!(c.float, zero_css_parser::values::FloatValue::Left),
                },
            ))
        })
        .collect();

    // 递归子节点。CSS float 侵入：祖先 BFC 内的 float 会侵入未建 BFC 的后代 block 的 line box，
    // 故把（祖先 float + 本容器直接 float）换算到每个子节点 box 坐标系后向其传播。
    // **排除子节点自身**：float 不应在自身 IFC 中排除自己（float-005 回归实证）。
    // FloatExclusion 无 x 字段（IFC 仅按 left/right + width 缩减行盒可用宽），故只需平移 y。
    let transform = |f: &crate::inline::FloatExclusion, child: &LayoutBox| crate::inline::FloatExclusion {
        y: f.y - child.y,
        height: f.height,
        width: f.width,
        is_left: f.is_left,
    };
    for child in &mut root.children {
        let child_ancestor: Vec<crate::inline::FloatExclusion> = own_floats
            .iter()
            .filter(|(id, _)| Some(*id) != child.node_id) // 排除子节点自身
            .map(|(_, f)| transform(f, child))
            .chain(ancestor_floats.iter().map(|f| transform(f, child)))
            .filter(|f| f.y + f.height > 0.0) // 裁掉完全在子节点上方的 float
            .collect();
        compute_final_inline_layouts(
            child,
            doc,
            styles,
            &child_ancestor,
            img_intrinsic_sizes,
            font_metric_provider,
        );
    }

    // 仅处理有 node_id 且含有直接文本子节点的容器
    let Some(node_id) = root.node_id else { return };
    let Some(_) = doc.get(node_id) else { return };

    // 跳过 flex/grid/table 容器（它们不需要独立的 inline layout）
    let Some(style) = styles.get(&node_id) else { return };
    use zero_css_parser::values::DisplayValue;
    if matches!(
        style.display,
        DisplayValue::Flex
            | DisplayValue::InlineFlex
            | DisplayValue::Grid
            | DisplayValue::InlineGrid
            | DisplayValue::Table
            | DisplayValue::InlineTable
    ) {
        return;
    }

    // 跳过多列容器（多列在 paint 阶段按列分配 IFC 内容，不适合预存储）
    // R900：inline-only column-fill:auto + 明确高度 multicol 容器列分布存储（命中则 paint
    // use_stored 按列渲染，无 paint 改动）。**默认开启**（实测 css-multicol oracle +1 零回归，
    // multicol-fill-auto-001 8.56%→0.00%；触发条件极窄：inline-only + auto + 明确高度 + 无 block
    // 子，welcome/legacy 不命中）。env `MULTICOL_COLUMN_FRAG=0` 可关闭作回退开关。
    if root.is_multicol {
        let enabled = std::env::var("MULTICOL_COLUMN_FRAG").as_deref() != Ok("0");
        if enabled && store_inline_multicol_columns(root, doc, styles) {
            return;
        }
        return;
    }

    // 跳过非块级元素（display: inline）：
    // 这些元素的文本内容已经参与父级 IFC 排列，不需要单独存储。
    // 如果为它们也存储 inline_layout，paint 系统会双重渲染文本——
    // 一次从父级 IFC（含 float exclusion），一次从自身 IFC（无 float exclusion），
    // 导致文本与 float 重叠。
    // inline-block/inline-flex/inline-grid 虽然也是 inline-level，
    // 但它们有独立的布局上下文，is_block_level 不会是 false。
    //
    // R978 例外：table-internal 行/行组容器（TableRowGroup/HeaderGroup/FooterGroup/Row）
    // 若含**直接文本子节点**（裸文本，无 row/cell 包裹），CSS Tables §3.1 要求生成匿名 row+cell。
    // ZW 未实现匿名 cell 生成（text node 非 LayoutBox child），裸文本会完全 orphan
    // （table_grid 跳过 text node + 此处 is_block_level=false 跳过 IFC）→ 渲染为 16px 默认。
    // 让此类容器跑 IFC，使裸文本至少按容器 font/size 渲染（table-row-group-color-inheritance-001：
    // 200px green Ahem X）。仅「直接 text child」触发，正常 table（rows/cells 子元素）不受影响。
    let is_table_internal_with_text = matches!(
        style.display,
        DisplayValue::TableRowGroup
            | DisplayValue::TableHeaderGroup
            | DisplayValue::TableFooterGroup
            | DisplayValue::TableRow
    ) && doc
        .child_nodes(node_id)
        .iter()
        .any(|c| doc.get(*c).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))));
    if !root.is_block_level && !is_table_internal_with_text {
        return;
    }

    // 检查是否有直接文本子节点
    let mut has_text_children = root.children.iter().any(|c| c.is_anonymous_text_item)
        || doc
            .child_nodes(node_id)
            .iter()
            .any(|child_id| doc.get(*child_id).is_some_and(|n| matches!(&n.kind, NodeKind::Text(_))));
    // PHASEA stored-line-boxes 路径（默认启用；env PHASEA_STORE_EXT=0 关闭）：也覆盖含 **inline-level** 元素子节点且**无 block-level
    // 元素子节点**的容器（纯 inline 内容，如 div>span 间接文本）。compute_final 传真实 styles 给
    // IFC（line 1851），存储行盒度量正确，paint use_stored 渲染解 Phase A font_size（font-051 实证）。
    // **排除混合 inline+block 内容**（如 block-in-inline R109 inline-box-001、span+h4 multicol-
    // block-no-clip-001）：此类容器的存储路径与现 paint 重跑在匿名块/碎片化上分歧致回归。
    if !has_text_children && std::env::var("PHASEA_STORE_EXT").as_deref() != Ok("0") {
        use zero_css_parser::values::DisplayValue;
        let is_inline_display = |d: &DisplayValue| {
            matches!(
                d,
                DisplayValue::Inline
                    | DisplayValue::InlineBlock
                    | DisplayValue::InlineFlex
                    | DisplayValue::InlineGrid
                    | DisplayValue::InlineTable
            )
        };
        let child_ids: Vec<NodeId> = doc.child_nodes(node_id);
        let child_displays: Vec<Option<&DisplayValue>> =
            child_ids.iter().map(|c| styles.get(c).map(|s| &s.display)).collect();
        let has_inline_elem = child_displays.iter().any(|d| d.is_some_and(is_inline_display));
        let has_block_elem = child_displays
            .iter()
            .any(|d| d.is_some_and(|dd| !is_inline_display(dd)));
        // 进一步要求 inline-level 子元素为**叶文本容器**（无元素子节点）：排除 block-in-inline
        //（inline 子元素含 block 后代，如 inline-box-002 的 div2>div3，R109 碎片化存储路径无法处理）。
        let inline_children_have_elem = child_ids.iter().any(|c| {
            styles.get(c).is_some_and(|s| is_inline_display(&s.display))
                && doc
                    .child_nodes(*c)
                    .iter()
                    .any(|gc| doc.get(*gc).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))))
        });
        has_text_children = has_inline_elem && !has_block_elem && !inline_children_have_elem;
    }
    if !has_text_children {
        return;
    }

    // 创建 IFC 并使用与 paint_text 相同的 CSS 属性配置
    use crate::inline::InlineFormattingContext;
    use crate::inline::WordBreakMode;
    use crate::types::InlineLayoutFragment;
    use crate::types::InlineLayoutLine;
    use zero_css_parser::values::LengthValue;
    use zero_style_system::property::types::{OverflowWrapValue, WhiteSpaceValue, WordBreakValue};

    // R1099 Slice α-1（vertical-mode IFC 四层协调）：container_width WM-aware。
    // vertical-rl/lr 下 IFC 的 `max_depth = self.container_width`（break_items_into_columns）
    // 表示字符向下推进的可用深度，须取竖直 inline 尺寸（content_height），非水平 block 尺寸
    //（content_width，vertical auto 容器常为 0 → max_depth=0 → 每字符一列横向排列，R1052 根因）。
    // horizontal-tb 取 content_width，字节一致零回归（WM gate 隔离）。
    let is_vertical_wm = matches!(
        root.writing_mode,
        zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
    );
    // decoration-gate（TBD-2）：vertical 容器子树有 text-decoration/emphasis 时
    // 保持 content_width（旧行为），回避 Layer 4 装饰坐标耦合（α-3 未实施）。
    let vertical_decoration_free = root
        .node_id
        .is_some_and(|id| !subtree_has_text_decoration(doc, styles, id));
    let container_width = if is_vertical_wm && vertical_decoration_free {
        root.content_height
    } else {
        root.content_width
    };

    // 解析 CSS 属性（与 paint_text 相同的配置）
    let break_word = matches!(
        style.overflow_wrap,
        OverflowWrapValue::BreakWord | OverflowWrapValue::Anywhere
    );
    let (no_wrap, preserve_whitespace, break_at_newline) = match &style.white_space {
        WhiteSpaceValue::Pre => (true, true, false),
        WhiteSpaceValue::PreWrap => (false, true, false),
        // pre-line：空白序列折叠（preserve_whitespace=false）但 `\n` 强制断行
        //（break_at_newline=true，CSS Text 3 §4.2）。kill-switch ZW_PRELINE_NEWLINE_BREAK=0 恢复旧行为。
        WhiteSpaceValue::PreLine => (
            false,
            false,
            std::env::var("ZW_PRELINE_NEWLINE_BREAK").as_deref() != Ok("0"),
        ),
        WhiteSpaceValue::Nowrap => (true, false, false),
        _ => (false, false, false),
    };
    let break_word = break_word
        || !no_wrap
            && matches!(
                style.overflow_wrap,
                OverflowWrapValue::BreakWord | OverflowWrapValue::Anywhere
            );
    let word_break_mode = match &style.word_break {
        WordBreakValue::BreakAll => WordBreakMode::BreakAll,
        WordBreakValue::KeepAll => WordBreakMode::KeepAll,
        _ => WordBreakMode::Normal,
    };
    // CSS Text 3 §5.3：line-break: anywhere 在每个排版字符处创建换行机会（覆盖
    // GL/JW/ZJW 禁则）。ZW 复用 BreakAll（任意字符可断）作为近似——break-all 在
    // overflow 时逐字符断行，对 width:1ch/窄容器场景产出与 anywhere 一致的逐字换行
    // （line-break-anywhere 簇驱动）。strict/loose/normal/auto 涉及 CJK 标点禁则，
    // 当前不实现（按 normal 默认行为）。
    let word_break_mode = if matches!(
        style.line_break,
        zero_style_system::property::types::LineBreakValue::Anywhere
    ) {
        WordBreakMode::BreakAll
    } else {
        word_break_mode
    };
    // 复用 resolve_text_align：start/end 方向感知（RTL→反向），避免此处独立 match 与
    // resolve_text_align / paint 路径三处分叉（R958 统一）。
    let text_align = resolve_text_align(Some(style));
    let text_align_last = resolve_text_align_last(Some(style));
    let text_indent_px = resolve_text_indent(&style.text_indent, &style.font_size, container_width);
    let tab_size_px = match &style.tab_size {
        zero_style_system::TabSizeValue::Number(n) => *n as f32 * 8.0,
        zero_style_system::TabSizeValue::Length(LengthValue::Px(v)) => *v as f32,
        _ => 8.0,
    };
    let is_vertical = matches!(
        root.writing_mode,
        zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
    );
    let is_vertical_rtl = matches!(root.writing_mode, zero_style_system::WritingModeValue::VerticalRl);

    // 构造 override maps（文本节点 → 父元素重键，过滤混入的内联元素片段防 last-write-wins
    // 非确定性）。R2189：此 4 map 与 paint Path B 三 map 逻辑字节一致，走共享 helper
    // build_text_parent_override_map（详见该 fn 文档）。
    let parent_font_sizes: HashMap<NodeId, f32> = build_text_parent_override_map(doc, &root.text_node_font_sizes);
    let parent_is_ahem: HashMap<NodeId, bool> = build_text_parent_override_map(doc, &root.text_node_is_ahem);
    let parent_letter_spacing: HashMap<NodeId, f32> =
        build_text_parent_override_map(doc, &root.text_node_letter_spacing);
    let parent_line_heights: HashMap<NodeId, f32> = build_text_parent_override_map(doc, &root.text_node_line_heights);

    // 收集浮动排除区域 = 本容器直接 float 子 + 祖先传播下来的 float（均已在 root box 坐标系）
    let exclusions: Vec<crate::inline::FloatExclusion> = own_floats
        .iter()
        .map(|(_, f)| f.clone())
        .chain(ancestor_floats.iter().cloned())
        .collect();

    let mut inline_ctx = InlineFormattingContext::new(container_width)
        .with_text_align(text_align)
        .with_text_align_last(text_align_last)
        .with_break_word(break_word)
        .with_no_wrap(no_wrap)
        .with_preserve_whitespace(preserve_whitespace)
        .with_break_at_newline(break_at_newline)
        .with_word_break(word_break_mode)
        .with_text_autospace(style.text_autospace)
        .with_text_indent(text_indent_px)
        .with_tab_size(tab_size_px)
        .with_vertical(is_vertical)
        .with_vertical_rtl(is_vertical_rtl)
        .with_block_extent(
            if is_vertical
                && root.node_id.is_some_and(|id| {
                    styles
                        .get(&id)
                        .is_some_and(|s| matches!(s.display, DisplayValue::TableCaption))
                })
            {
                root.content_width
            } else {
                container_width
            },
        )
        .with_font_size_overrides(parent_font_sizes)
        .with_is_ahem_overrides(parent_is_ahem)
        .with_letter_spacing_overrides(parent_letter_spacing)
        .with_line_height_overrides(parent_line_heights)
        .with_inline_element_metrics(root.inline_element_metrics.clone())
        .with_margin_overrides(root.inline_element_margins.clone())
        .with_img_intrinsic_sizes(img_intrinsic_sizes.clone());

    // U1b-wiring 切片 A（dormant）：注入 font-metric provider 使 line-height:normal 走
    // per-font 真实度量（`resolve_font_metrics_with_provider` 消费者已在 inline/mod.rs
    // :749/:1087 存在，读 IFC.font_metric_provider 字段）。默认 `None`（生产未注入）→
    // 消费者回退常数 1.164/Ahem 1.0 = 逐字节等价旧路径（零回归）。`Some` 时经既有
    // override-map 链路（frag.height → store_font_sizes → text_node_line_heights → paint）
    // 触达 paint，绕 R890 空 styles 阻塞。Handle 内部 `Rc` clone 廉价共享。
    if let Some(provider) = font_metric_provider {
        inline_ctx = inline_ctx.with_font_metric_provider(provider.0.clone());
    }

    if !exclusions.is_empty() {
        inline_ctx = inline_ctx.with_float_exclusions(exclusions);
    }

    // R109 §9.2.1.1：匿名块片段只收集其片段的 inline 内容（fragment_node_ids），
    // 而非 inline 元素的全部 DOM 子节点。
    if let Some(frag) = root.fragment_node_ids.clone() {
        inline_ctx.set_fragment_node_ids(frag);
    }

    // R84/R355：用真实样式跑 IFC 并存储行盒。仅当容器为**纯 Ahem 字体**时存储：
    // - 纯 Ahem（font-family 恰好为 ["Ahem"]）：避免多字体列表（如 "Courier New, Ahem"）
    //   在真实样式下的 font 解析/fallback 差异导致回归。
    // R355 放宽 R84 的「单行」限制为「多行」：解 large-font 簇（inline-formatting-context-008/009
    // 的 100px 文本 paint 阶段被 16px 默认值覆盖）。多行存储经 chromium-Oracle 实测确证 net-positive
    // （ifc-008 -4.01% / ifc-009 -1.95% Z_vs_chromium，见 evidence/r355-multiline-oracle-*.txt）。
    // **例外**：浮动容器保持 R84 单行限制——multicol-fill-auto-001 的 ref 用 float div 模拟列
    // （其 test 用真 multicol 已在上方 line 242 排除存储），浮动容器多行存储打破 test/ref 对称致
    // self-source 发散；该 case chromium-Oracle 9.15% 不变 = 非真回归，guard 仅维持 self-source 一致。
    inline_ctx.layout(doc, node_id, styles);
    // R632：存 font_size/line_height/is_ahem/letter_spacing overrides 供 paint Path B 重跑 IFC 用。
    // compute_final 此前不存（仅 remeasure 路径 line 801/935 存），致走 Path B 的容器（非 pure-Ahem，
    // 含 wrap/auto-wrap 多行块）paint IFC override 全空 → line_height fallback 19.2 (16×1.2) 而非
    // CSS line-height，行间距度量错误（R630 修了多行 y 分行，本修复补 line_height 度量）。
    // line_height 不影响行断（mod.rs 注释），但 font_size override 命中会影响 paint IFC
    // char-width 行断——R627 曾 net -15（pre-wrap），R630 后重试（with_line_y 可能吸收）。
    store_font_sizes_from_ifc(&inline_ctx, root, doc, styles);
    let is_pure_ahem = style.font_family.len() == 1 && style.font_family[0].eq_ignore_ascii_case("Ahem");
    let is_floated = !matches!(style.float, FloatValue::None);
    // R1280：含 float 子的容器（[inline 内容 + float] 模式，如 floats-006 的 div1）须存 IFC，
    // 让 paint 走 Path A（真实 styles → 折叠 inline 子用其真实字体度量 + is_ahem_font=true →
    // render v_offset is_ahem 分支正确）。Path B（override maps）对折叠 inline 元素的 is_ahem
    // 不传播 + baseline_offset 非 is_ahem-aware，致混合字体（default 容器 + Ahem span）glyph
    // 位错（floats-006 残余 4.04%）。kill-switch `ZW_FLOAT_INLINE_PAINT=0` 关闭（default-on）。
    let has_float_exclusions = !own_floats.is_empty();
    let allow_non_ahem_store = has_float_exclusions && std::env::var("ZW_FLOAT_INLINE_PAINT").as_deref() != Ok("0");
    if (!is_pure_ahem && !allow_non_ahem_store) || (is_floated && inline_ctx.lines.len() > 1) {
        return;
    }

    // 转换 IFC 结果为 InlineLayoutLine/InlineLayoutFragment
    let lines: Vec<InlineLayoutLine> = inline_ctx
        .lines
        .iter()
        .map(|line| InlineLayoutLine {
            y: line.y,
            height: line.height,
            baseline_y: line.baseline_y,
            ascent: line.ascent,
            descent: line.descent,
            fragments: line
                .runs
                .iter()
                .map(|frag| {
                    // R822 Phase 3：per-fragment valign-aware glyph 基线。paint Path A is_ahem
                    // glyph 位图顶 = baseline_y_abs - font_size，故 baseline_y 决定字形基线。
                    // text-bottom ↑ half_leading（对齐父 content-area 底，glyph 上移到 strut 之上）
                    // / text-top ↓ half_leading / sub ↓0.3·fs / super ↑0.3·fs；
                    // baseline/top/bottom/middle = line.baseline_y。配合 R822 line-box 扩展
                    // （apply_vertical_alignment 已把 line-box 撑高、strut 下移）使 div 高度正确。
                    let strut_fs = ((line.ascent - line.height / 2.0) / 0.3).max(0.0);
                    let half_leading = (line.ascent - 0.8 * strut_fs).max(0.0);
                    let frag_baseline_y = match frag.vertical_align {
                        VerticalAlignValue::TextBottom => line.baseline_y - half_leading,
                        VerticalAlignValue::TextTop => line.baseline_y + half_leading,
                        VerticalAlignValue::Sub => line.baseline_y + 0.3 * frag.font_size,
                        VerticalAlignValue::Super => line.baseline_y - 0.3 * frag.font_size,
                        _ => line.baseline_y,
                    };
                    InlineLayoutFragment {
                        x: frag.x,
                        y: frag.y,
                        width: frag.width,
                        height: frag.height,
                        font_size: frag.font_size,
                        is_ahem: frag.is_ahem,
                        is_ahem_font: frag.is_ahem,
                        text: frag.text.clone(),
                        node_id: Some(frag.node_id),
                        baseline_y: frag_baseline_y,
                    }
                })
                .collect(),
        })
        .collect();

    if !lines.is_empty() {
        root.inline_layout = Some(lines);
        root.inline_layout_width = container_width;
    }
}

pub(crate) fn measure_text_content(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    dom_id: NodeId,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    img_intrinsic_sizes: &HashMap<NodeId, (f32, f32)>,
    font_metric_provider: Option<&crate::inline::FontMetricProviderHandle>,
) -> Size<f32> {
    // 检查是否为文本节点（匿名 flex/grid item）
    // 在 flex/grid 容器中，文本节点被包装为匿名 taffy 节点参与布局。
    if let Some(node) = doc.get(dom_id)
        && let NodeKind::Text(text_data) = &node.kind
    {
        let text = text_data.content.trim().to_string();
        if text.is_empty() {
            return Size::ZERO;
        }
        // 获取父元素的 ComputedStyle 用于字体指标
        let parent_style = doc.parent_node(dom_id).and_then(|pid| styles.get(&pid));
        let (font_size, line_height) =
            crate::inline::resolve_font_metrics_with_provider(parent_style, font_metric_provider);
        let is_ahem = parent_style
            .map(|s| s.font_family.iter().any(|f| f.eq_ignore_ascii_case("Ahem")))
            .unwrap_or(false);

        // 包含 letter-spacing：CSS 规范中 letter-spacing 适用于每个字符
        let letter_spacing: f32 = parent_style
            .map(|s| match &s.letter_spacing {
                zero_style_system::property::types::LengthValue::Px(v) => *v as f32,
                _ => 0.0,
            })
            .unwrap_or(0.0);
        // R1750：respect available_space MinContent —— bare-text 匿名 flex/grid item 的
        // min-width:auto（CSS Flexbox §4.5）须取 min-content（最宽不可拆词）非 max-content
        //（全文本累加）。旧实现恒返全文本宽 → taffy 把 min-size:auto 算成 max-content → flex
        // item 无法收缩到最宽词以下（flex-minimum-width-flex-items 谱系 + grid item min-size）。
        // MaxContent/Definite 行为不变（单行全文本累加）。
        let measured_width: f32 = if matches!(available_space.width, AvailableSpace::MinContent) {
            text.split(char::is_whitespace)
                .filter(|word| !word.is_empty())
                .map(|word| {
                    word.chars()
                        .map(|ch| crate::inline::estimate_char_width(ch, font_size, is_ahem) + letter_spacing)
                        .sum::<f32>()
                })
                .fold(0.0f32, f32::max)
        } else {
            text.chars()
                .map(|ch| crate::inline::estimate_char_width(ch, font_size, is_ahem) + letter_spacing)
                .sum()
        };

        return Size {
            width: known_dimensions.width.unwrap_or(measured_width),
            height: known_dimensions.height.unwrap_or(line_height),
        };
    }

    if !has_inline_content(doc, styles, dom_id) {
        // 无行内内容的叶节点（如空的 flex/grid 子元素）：
        // 尺寸来自 known_dimensions（taffy 已知的尺寸），
        // 回退到 CSS computed style 的显式 width/height。
        // 注意：taffy flexbox 在 measure callback 中会将主轴 known_dimensions 设为 None
        // （因为主轴尺寸由 flex 布局控制），所以需要从 computed style 获取。
        // 但当 available_space 指示「内容测量」（MinContent/MaxContent，用于 CSS Flexbox
        // §4.5 min-size:auto 的 content size suggestion，或 grid 轨道 intrinsic sizing）时，
        // taffy 传 None 主轴是为了测量「忽略显式尺寸的纯内容尺寸」——空叶节点的内容为 0。
        // 此处不能再回退到显式 width/height（否则空 flex item 的 min-size:auto = 显式宽，
        // flex-shrink 永不收缩，flex-shrink-001/002/003/006/007/008 FAIL）。显式尺寸已由
        // converter 写入 taffy style.size，由 taffy 自行应用，measure 仅汇报内容。
        let style = styles.get(&dom_id);
        let measuring_content_w = matches!(available_space.width, AvailableSpace::MinContent)
            || matches!(available_space.width, AvailableSpace::MaxContent);
        let measuring_content_h = matches!(available_space.height, AvailableSpace::MinContent)
            || matches!(available_space.height, AvailableSpace::MaxContent);
        let explicit_w = known_dimensions.width.or_else(|| {
            if measuring_content_w {
                return None;
            }
            style.and_then(|s| match &s.width {
                LengthValue::Px(v) => Some(*v as f32),
                _ => None,
            })
        });
        let explicit_h = known_dimensions.height.or_else(|| {
            if measuring_content_h {
                return None;
            }
            style.and_then(|s| match &s.height {
                LengthValue::Px(v) => Some(*v as f32),
                _ => None,
            })
        });
        return Size {
            width: explicit_w.unwrap_or(0.0),
            height: explicit_h.unwrap_or(0.0),
        };
    }

    let width = if matches!(available_space.width, AvailableSpace::MinContent) {
        // MinContent 测量：用 0 宽强制每个不可拆单元（单词/原子行内盒）独占一行，
        // 则最宽行 = 最宽不可拆单元 = min-content 宽度。
        // 旧实现 MinContent 也落入 INFINITY → 全部单词一行 → measured_width = max-content
        // （错误偏大）。R428 min-size:auto 默认后，grid/flex item 的 min-width 被这个偏大值
        // floor → 卡片过宽（welcome.html +7.65pp 回归，R541 实证 min-width:0 可恢复）。
        0.0
    } else {
        known_dimensions
            .width
            .or(available_space.width.into_option())
            .unwrap_or(f32::INFINITY)
            .max(0.0)
    };
    let is_vertical = doc
        .parent_node(dom_id)
        .and_then(|pid| styles.get(&pid))
        .is_some_and(|s| {
            matches!(
                s.writing_mode,
                WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
            )
        });
    let is_vertical_rtl = doc
        .parent_node(dom_id)
        .and_then(|pid| styles.get(&pid))
        .is_some_and(|s| matches!(s.writing_mode, WritingModeValue::VerticalRl));
    // 收集 inline-block 子元素的尺寸，供 IFC 正确计算行盒和换行。
    // resolve_inline_block_dimension 对 Percentage 值返回 0，
    // 需要用容器宽度解析百分比后提供给 IFC。
    let ib_sizes: HashMap<NodeId, (f32, f32)> = doc
        .child_nodes(dom_id)
        .iter()
        .filter_map(|&child_id| {
            let child_node = doc.get(child_id)?;
            if !matches!(&child_node.kind, NodeKind::Element(_)) {
                return None;
            }
            let style = styles.get(&child_id)?;
            if !matches!(style.display, DisplayValue::InlineBlock) {
                return None;
            }
            let w = crate::inline::resolve_inline_block_dimension(&style.width, style, true);
            let h = crate::inline::resolve_inline_block_dimension(&style.height, style, false);
            // Percentage 宽度用 container_width 解析
            let resolved_w = if w > 0.0 {
                w
            } else if let LengthValue::Percentage(pct) = &style.width {
                (*pct as f32 / 100.0) * width
            } else {
                0.0
            };
            let resolved_h = if h > 0.0 {
                h
            } else if let LengthValue::Percentage(pct) = &style.height {
                (*pct as f32 / 100.0) * width
            } else {
                0.0
            };
            if resolved_w > 0.0 || resolved_h > 0.0 {
                Some((child_id, (resolved_w, resolved_h)))
            } else {
                None
            }
        })
        .collect();
    // R645：white-space 影响 taffy 测量的内容高度——pre/nowrap 容器不应在测量时换行
    //（否则 box content_height 偏大，暴露于 SEA 词典分词文字 per-char fallback breaking）。
    // R1855：overflow-wrap:break-word/anywhere 须在测量期也 char-break——否则 break-word 容器被
    // 测成 1 行（box 高度偏小），与 paint/stored IFC（char-break 多行）不一致，致 #ref 该断词
    // 多行却只占 1 行高度（word-wrap-002/overflow-wrap-002 等）。no_wrap=true 时 char-break 已被
    // break_lines.rs 的 `!self.no_wrap` gate 关闭，故此处 break_word 仅对非 nowrap 容器生效（spec 正确）。
    let measure_style = styles.get(&dom_id);
    let no_wrap = resolve_no_wrap_for_ifc_measure(measure_style);
    // R1935：measure 期也传 preserve/break_at_newline（pre/pre-wrap/pre-line 容器），kill-switch 可关。
    let measure_preserve_on = std::env::var("ZW_MEASURE_PRESERVE").as_deref() != Ok("0");
    let preserve = measure_preserve_on && resolve_preserve_for_ifc_measure(measure_style);
    let break_at_newline = measure_preserve_on && resolve_break_at_newline_for_ifc_measure(measure_style);
    let break_word = measure_style.is_some_and(|s| {
        use zero_style_system::property::types::OverflowWrapValue;
        matches!(
            s.overflow_wrap,
            OverflowWrapValue::BreakWord | OverflowWrapValue::Anywhere
        )
    });
    let mut inline_ctx = InlineFormattingContext::new(width)
        .with_vertical(is_vertical)
        .with_vertical_rtl(is_vertical_rtl)
        .with_no_wrap(no_wrap)
        .with_preserve_whitespace(preserve)
        .with_break_at_newline(break_at_newline)
        .with_break_word(break_word)
        .with_inline_block_sizes(ib_sizes)
        .with_img_intrinsic_sizes(img_intrinsic_sizes.clone());
    if let Some(provider) = font_metric_provider {
        inline_ctx = inline_ctx.with_font_metric_provider(provider.0.clone());
    }
    inline_ctx.layout(doc, dom_id, styles);

    let measured_width = inline_ctx
        .all_fragments()
        .iter()
        .map(|fragment| fragment.x + fragment.width)
        .fold(0.0_f32, f32::max);

    let full_total = inline_ctx.total_height();
    // R1433：layout-time balance 高度——balance multicol 容器在 taffy 测量期就返回均衡列高
    //（ceil(L/N)×行高），而非全宽 IFC 全高。避 R1432/R1432b post-remeasure sibling-shift 级联
    //（layout-time taffy 从源头定高，无 post-hoc 修正）。严格 gate：① balance（balance_column_geometry）；
    // ② text-only（DOM 无元素子）；③ deterministic（列宽 IFC 行数 == 全宽行数，内容不在列宽换行）；
    // ④ overflow:visible（clip 容器平衡语义不同，R1432b clip-001 pass→fail 实证）。
    let balanced_height = {
        let style = styles.get(&dom_id);
        let mut h = full_total;
        if let Some(s) = style
            && let Some((cw, cols)) = crate::multicol::balance_column_geometry(s, width)
            && cw > 0.0
            && cols >= 2
            && matches!(s.overflow_y, zero_style_system::property::types::OverflowValue::Visible)
        {
            let text_only = doc.child_nodes(dom_id).iter().all(|c| {
                // 允许文本节点 + 行内元素（br=行内换行、span 等=文本流一部分）；
                // 排除 block-level 元素子（独立列项，balance 行公式不适用）。
                if let Some(node) = doc.get(*c)
                    && let NodeKind::Element(e) = &node.kind
                {
                    if e.local_name().eq_ignore_ascii_case("br") {
                        return true;
                    }
                    let cs = styles.get(c);
                    return cs.is_some_and(|s| {
                        matches!(
                            s.display,
                            zero_style_system::property::types::DisplayValue::Inline
                                | zero_style_system::property::types::DisplayValue::InlineBlock
                        )
                    });
                }
                true
            });
            if text_only {
                let mut col_ctx = InlineFormattingContext::new(cw)
                    .with_no_wrap(no_wrap)
                    .with_preserve_whitespace(preserve)
                    .with_break_at_newline(break_at_newline);
                if let Some(provider) = font_metric_provider {
                    col_ctx = col_ctx.with_font_metric_provider(provider.0.clone());
                }
                col_ctx.layout(doc, dom_id, styles);
                let cn = col_ctx.lines.len();
                let ct = col_ctx.total_height();
                // deterministic：列宽不换行（行数 == 全宽）才用均衡高，否则 font-sensitive off-by-one。
                if cn > 0 && cn == inline_ctx.lines.len() {
                    h = cn.div_ceil(cols) as f32 * (ct / cn as f32);
                }
            }
        }
        h
    };

    Size {
        width: known_dimensions.width.unwrap_or(measured_width),
        height: known_dimensions.height.unwrap_or(balanced_height),
    }
}

pub(crate) fn has_direct_text(doc: &Document, dom_id: NodeId) -> bool {
    doc.child_nodes(dom_id).iter().any(|child_id| {
        matches!(
            doc.get(*child_id).map(|node| &node.kind),
            Some(NodeKind::Text(text)) if !text.content.trim().is_empty()
        )
    })
}

/// 检查容器是否包含行内级内容（文本节点或行内级元素）。
///
/// CSS 2.1 规范要求空 inline 元素仍通过 line-height + padding + border
/// 贡献到行盒高度。仅检查文本节点会遗漏仅包含空 inline 元素的容器，
/// 导致 IFC 不被调用，行盒高度计算不正确。
pub(crate) fn has_inline_content(doc: &Document, styles: &HashMap<NodeId, ComputedStyle>, dom_id: NodeId) -> bool {
    // 快速路径：有直接文本子节点
    if has_direct_text(doc, dom_id) {
        return true;
    }

    // 检查是否有 inline-level 元素子节点
    use zero_style_system::property::types::DisplayValue;
    doc.child_nodes(dom_id).iter().any(|child_id| {
        if let Some(node) = doc.get(*child_id)
            && let NodeKind::Element(_elem_data) = &node.kind
            && let Some(style) = styles.get(child_id)
        {
            return matches!(style.display, DisplayValue::Inline | DisplayValue::InlineBlock);
        }
        false
    })
}

/// 为包含 float 元素的容器重新测量行内文本，使文本环绕 float 排列。
///
/// 工作原理：
/// 1. 遍历 LayoutBox 树，找到同时包含 float 子元素和直接文本内容的容器
/// 2. 收集容器内的 float 元素的几何信息，构建 FloatExclusion 列表
/// 3. 使用 float exclusions 重新运行 InlineFormattingContext 排列文本
/// 4. 用重新排列后的行盒更新容器的内部布局信息
pub(crate) fn remeasure_text_with_float_exclusions(
    box_node: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    img_intrinsic_sizes: &HashMap<NodeId, (f32, f32)>,
    font_metric_provider: Option<&crate::inline::FontMetricProviderHandle>,
) {
    // 收集此容器的 float 排除区域
    let has_floats = box_node.children.iter().any(|c| !matches!(c.float, FloatValue::None));

    if has_floats {
        // 构建 float 排除区域列表
        let exclusions: Vec<FloatExclusion> = box_node
            .children
            .iter()
            .filter(|c| !matches!(c.float, FloatValue::None))
            .filter_map(|c| {
                // c.y 现在是相对于父级内容区域的坐标（与 taffy 一致）
                let rel_y = c.y;
                if rel_y < 0.0 || c.width <= 0.0 || c.height <= 0.0 {
                    return None;
                }
                Some(FloatExclusion {
                    y: rel_y + c.margin_top,
                    height: c.height + c.margin_bottom,
                    width: c.width + c.margin_left + c.margin_right,
                    is_left: matches!(c.float, FloatValue::Left),
                })
            })
            .collect();

        // 如果有排除区域且容器有行内级内容
        if !exclusions.is_empty()
            && let Some(dom_id) = box_node.node_id
            && has_inline_content(doc, styles, dom_id)
        {
            // 收集 inline-block 子元素的 LayoutBox 尺寸
            let ib_sizes: HashMap<NodeId, (f32, f32)> = box_node
                .children
                .iter()
                .filter(|c| {
                    c.node_id.is_some_and(|id| {
                        styles
                            .get(&id)
                            .is_some_and(|s| matches!(s.display, DisplayValue::InlineBlock))
                    })
                })
                .filter_map(|c| {
                    let node_id = c.node_id?;
                    // R1147：empty inline-block（content_height≈0）用 border-box height
                    //（含 border），避免 IFC 降级零宽（见 postprocess.rs 同改）。
                    let ib_h = if c.content_height.abs() < 1.0 {
                        c.height
                    } else {
                        c.content_height
                    };
                    Some((node_id, (c.content_width, ib_h)))
                })
                .collect();

            // 重新运行 inline layout with float exclusions
            let container_width = box_node.content_width;
            let is_vertical = matches!(
                box_node.writing_mode,
                WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
            );
            let is_vertical_rtl = matches!(box_node.writing_mode, WritingModeValue::VerticalRl);
            let text_align = resolve_text_align(styles.get(&dom_id));
            let text_align_last = resolve_text_align_last(styles.get(&dom_id));
            let no_wrap = resolve_no_wrap_for_ifc_measure(styles.get(&dom_id));
            let mut inline_ctx = InlineFormattingContext::new(container_width)
                .with_float_exclusions(exclusions)
                .with_vertical(is_vertical)
                .with_vertical_rtl(is_vertical_rtl)
                .with_text_align(text_align)
                .with_text_align_last(text_align_last)
                .with_no_wrap(no_wrap)
                .with_inline_block_sizes(ib_sizes)
                .with_img_intrinsic_sizes(img_intrinsic_sizes.clone());
            if let Some(provider) = font_metric_provider {
                inline_ctx = inline_ctx.with_font_metric_provider(provider.0.clone());
            }
            inline_ctx.layout(doc, dom_id, styles);

            // 存储 IFC 片段中各文本节点的 font_size，供 paint 系统计算基线偏移
            store_font_sizes_from_ifc(&inline_ctx, box_node, doc, styles);
            sync_inline_child_boxes_from_ifc(box_node, &inline_ctx, styles);

            // 容器高度需要包含 float 元素占用的空间
            let text_height = inline_ctx.total_height();
            let float_bottom = box_node
                .children
                .iter()
                .filter(|c| !matches!(c.float, FloatValue::None))
                .map(|c| c.y + c.height + c.margin_bottom)
                .fold(0.0_f32, f32::max);

            // 使用文本和 float 中较大的高度
            let content_height = text_height.max(float_bottom);
            // 更新容器的内容高度：文本环绕 float 后可能需要更大的高度。
            // ★ R1616：仅 height:auto 容器才按 float/文本底扩展——definite height
            //（如 height:100px）容器 float 应溢出而非撑高（CSS §10.5/§10.6：显式高度
            // 不被 float 子覆盖）。floats-placement-006：container height:100 +
            // float-left clear:both @y=100 被错误扩到 150（float_bottom=150）。
            // env ZW_REMEASURE_FLOAT_DEFHEIGHT=0 关闭（kill-switch，default-on）：
            // 关闭时退回旧行为（无视 is_auto_height，一律扩展）。
            let fix_active = std::env::var("ZW_REMEASURE_FLOAT_DEFHEIGHT").as_deref() != Ok("0");
            let is_auto_height = box_node
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.height, LengthValue::Auto));
            let should_expand = if fix_active { is_auto_height } else { true };
            if should_expand && content_height > box_node.content_height {
                let diff = content_height - box_node.content_height;
                box_node.content_height = content_height;
                box_node.height += diff;
            }
        }
    }

    // 递归处理子容器
    for child in &mut box_node.children {
        remeasure_text_with_float_exclusions(child, doc, styles, img_intrinsic_sizes, font_metric_provider);
    }
}

/// 为包含行内级子元素但无 float 的容器重新测量内容高度。
///
/// 当一个 block 容器只包含 inline 或 inline-block 子元素时（无文本节点），
/// taffy 将这些元素当作 block 排列，无法正确计算行盒高度。
/// 此函数检测这类容器，运行 IFC 获取正确的内容高度。
///
/// 典型场景：`<div><span style="line-height:5"></span></div>`
/// 空 span 的 line-height 应贡献到行盒高度，但 taffy 无法处理此情况。
pub(crate) fn remeasure_inline_only_containers(
    box_node: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    img_intrinsic_sizes: &HashMap<NodeId, (f32, f32)>,
) {
    // flex/grid 容器不走 IFC 重算——它们的子元素是 flex/grid item，
    // 尺寸由 taffy 决定，不应被 IFC 片段覆盖。
    // table 容器仅在有 table-internal 子元素时跳过（如 tbody/tr/td）；
    // 无 table-internal 子元素的 table 容器行为等价于 block，需要 IFC 重算。
    if box_node.is_layout_container {
        let is_table_without_internals = box_node.node_id.is_some_and(|id| {
            styles
                .get(&id)
                .is_some_and(|s| matches!(s.display, DisplayValue::Table | DisplayValue::InlineTable))
        }) && !box_node.children.iter().any(|c| {
            c.node_id.is_some_and(|cid| {
                styles.get(&cid).is_some_and(|s| {
                    matches!(
                        s.display,
                        DisplayValue::TableRowGroup
                            | DisplayValue::TableHeaderGroup
                            | DisplayValue::TableFooterGroup
                            | DisplayValue::TableRow
                            | DisplayValue::TableCell
                            | DisplayValue::TableColumn
                            | DisplayValue::TableColumnGroup
                            | DisplayValue::TableCaption
                    )
                })
            })
        });
        if !is_table_without_internals {
            // 仍然递归处理子容器
            for child in &mut box_node.children {
                remeasure_inline_only_containers(child, doc, styles, img_intrinsic_sizes);
            }
            return;
        }
    }

    // 检查此容器是否有 inline-level 子元素（is_block_level == false）
    // 且不包含 float 子元素（float 容器由 remeasure_text_with_float_exclusions 处理）
    let has_floats = box_node.children.iter().any(|c| !matches!(c.float, FloatValue::None));
    let has_inline_children = box_node
        .children
        .iter()
        .any(|c| !c.is_block_level && !c.is_absolute && !c.is_fixed);
    // R105：仅含直接 DOM 文本（无 inline 元素子，文本不生成独立 LayoutBox 子）且 taffy 未测量
    // （content_height≈0）的块也需要 remeasure——否则其 font_size 不会被 store_font_sizes_from_ifc
    // 存储，paint IFC 默认 16，导致大字号（100px）reftest（如 inline-formatting-context-008）渲染成 16px。
    // content_height≈0 守卫避免覆盖 taffy 已正确测量的块（font-feature/multicol-fill-auto/abspos 回归源）。
    let has_dom_text = box_node.node_id.is_some_and(|id| {
        doc.child_nodes(id)
            .iter()
            .any(|c| doc.get(*c).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))))
    });
    let needs_dom_text_remeasure =
        has_dom_text && box_node.content_height < 1.0 && box_node.children.iter().all(|c| c.is_absolute || c.is_fixed);

    if !has_floats
        && (has_inline_children || needs_dom_text_remeasure)
        && let Some(dom_id) = box_node.node_id
        && let Some(style) = styles.get(&dom_id)
        && matches!(style.height, LengthValue::Auto)
    {
        let container_width = box_node.content_width;
        let is_vertical = matches!(
            box_node.writing_mode,
            WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
        );
        let is_vertical_rtl = matches!(box_node.writing_mode, WritingModeValue::VerticalRl);
        let text_align = resolve_text_align(styles.get(&dom_id));
        let text_align_last = resolve_text_align_last(styles.get(&dom_id));
        let no_wrap = resolve_no_wrap_for_ifc_measure(styles.get(&dom_id));
        // 收集 inline-block 子元素的 LayoutBox 尺寸，供 IFC 解析百分比宽度。
        let ib_sizes: HashMap<NodeId, (f32, f32)> = box_node
            .children
            .iter()
            .filter(|c| {
                c.node_id.is_some_and(|id| {
                    styles
                        .get(&id)
                        .is_some_and(|s| matches!(s.display, DisplayValue::InlineBlock))
                })
            })
            .filter_map(|c| {
                let node_id = c.node_id?;
                // R1147：empty inline-block 用 border-box height（见 postprocess.rs）。
                let ib_h = if c.content_height.abs() < 1.0 {
                    c.height
                } else {
                    c.content_height
                };
                Some((node_id, (c.content_width, ib_h)))
            })
            .collect();
        let ib_sizes_for_mc = ib_sizes.clone();
        let mut inline_ctx = InlineFormattingContext::new(container_width)
            .with_vertical(is_vertical)
            .with_vertical_rtl(is_vertical_rtl)
            .with_text_align(text_align)
            .with_text_align_last(text_align_last)
            .with_no_wrap(no_wrap)
            .with_inline_block_sizes(ib_sizes)
            .with_img_intrinsic_sizes(img_intrinsic_sizes.clone());
        inline_ctx.layout(doc, dom_id, styles);

        // 存储 IFC 片段中各文本节点的 font_size，供 paint 系统计算基线偏移
        store_font_sizes_from_ifc(&inline_ctx, box_node, doc, styles);
        sync_inline_child_boxes_from_ifc(box_node, &inline_ctx, styles);

        let full_height = inline_ctx.total_height();
        // balance 模式多列容器：按列宽单独测量，计算均衡分布后的高度
        // （tallest column = ceil(num_lines / col_count) 行），使容器高度匹配
        // 分配后的列内容，而非全宽 IFC 的较短高度。
        let content_height = if let Some((cw, cols)) = crate::multicol::balance_column_geometry(style, container_width)
        {
            let mut col_ctx = InlineFormattingContext::new(cw)
                .with_vertical(is_vertical)
                .with_vertical_rtl(is_vertical_rtl)
                .with_text_align(text_align)
                .with_text_align_last(text_align_last)
                .with_no_wrap(no_wrap)
                .with_inline_block_sizes(ib_sizes_for_mc)
                .with_img_intrinsic_sizes(img_intrinsic_sizes.clone());
            col_ctx.layout(doc, dom_id, styles);
            let total = col_ctx.total_height();
            let n = col_ctx.lines.len();
            if n > 0 && cols > 0 {
                n.div_ceil(cols) as f32 * (total / n as f32)
            } else {
                total
            }
        } else {
            full_height
        };
        if content_height > box_node.content_height {
            // 如果 IFC 计算的高度大于 taffy 的高度，更新容器高度
            let diff = content_height - box_node.content_height;
            box_node.content_height = content_height;
            box_node.height += diff;
        } else if content_height < box_node.content_height {
            // 纯 inline-level 容器且非特殊布局容器：允许减小高度。
            // taffy 将 inline 元素映射为 Block，会错误地包含 inline 元素的垂直 margin，
            // 而 CSS 2.1 规定 inline 元素的 margin-top/margin-bottom 不影响行盒高度。
            let has_block_children = box_node
                .children
                .iter()
                .any(|c| c.is_block_level && !c.is_absolute && !c.is_fixed);
            let is_layout_container = matches!(
                style.display,
                DisplayValue::Flex
                    | DisplayValue::InlineFlex
                    | DisplayValue::Grid
                    | DisplayValue::InlineGrid
                    | DisplayValue::Table
                    | DisplayValue::InlineTable
            );
            if !has_block_children && !is_layout_container {
                let diff = content_height - box_node.content_height;
                box_node.content_height = content_height;
                box_node.height += diff;
            }
        }
    }

    // 递归处理子容器，并在 inline-only 容器收缩后把后续普通流兄弟一并上移。
    let mut idx = 0usize;
    while idx < box_node.children.len() {
        let old_height = box_node.children[idx].height;
        let old_content_height = box_node.children[idx].content_height;
        remeasure_inline_only_containers(&mut box_node.children[idx], doc, styles, img_intrinsic_sizes);
        let height_delta = box_node.children[idx].height - old_height;
        let content_height_delta = box_node.children[idx].content_height - old_content_height;
        let shrink_delta = height_delta.min(content_height_delta);
        if shrink_delta < -0.01
            && matches!(box_node.children[idx].float, FloatValue::None)
            && !box_node.children[idx].is_absolute
            && !box_node.children[idx].is_fixed
            // 垂直书写模式下块流方向为水平（x 轴），「高度」是 inline 轴跨度。
            // inline 轴收缩不会在块轴留下空隙，故不应移动后续块兄弟（它们按 x 排列）。
            // 旧代码无条件 `sibling.y += shrink_delta` 会把垂直模式的兄弟推到负 y（屏幕外），
            // 例如 writing-mode:vertical-rl 根页面整页渲染为空白（box-offsets-rel-pos-vrl-004）。
            && matches!(box_node.writing_mode, WritingModeValue::HorizontalTb)
        {
            for sibling in box_node.children.iter_mut().skip(idx + 1) {
                if sibling.is_absolute || sibling.is_fixed || !matches!(sibling.float, FloatValue::None) {
                    continue;
                }
                sibling.y += shrink_delta;
            }
        }
        idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_text_indent;
    use super::{ComputedStyle, TextAlign, resolve_text_align, resolve_text_align_last};
    use zero_css_parser::values::LengthValue;
    use zero_style_system::property::{DirectionValue, TextAlignLastValue, TextAlignValue};

    #[test]
    fn test_resolve_text_align_start_end_direction_aware() {
        // R958：start/end 是方向感知值（CSS Text 3 §6.1）。LTR 下 start=left/end=right；
        // RTL 下 start=right/end=left。旧实现无条件 start→left 致 direction:rtl 错误左对齐。
        let mut style = ComputedStyle::default();
        // LTR（默认）
        style.direction = DirectionValue::Ltr;
        style.text_align = TextAlignValue::Start;
        assert_eq!(resolve_text_align(Some(&style)), TextAlign::Left);
        style.text_align = TextAlignValue::End;
        assert_eq!(resolve_text_align(Some(&style)), TextAlign::Right);
        // 显式 Left/Right 不受 direction 影响
        style.text_align = TextAlignValue::Left;
        assert_eq!(resolve_text_align(Some(&style)), TextAlign::Left);
        // RTL：start/end 翻转
        style.direction = DirectionValue::Rtl;
        style.text_align = TextAlignValue::Start;
        assert_eq!(resolve_text_align(Some(&style)), TextAlign::Right);
        style.text_align = TextAlignValue::End;
        assert_eq!(resolve_text_align(Some(&style)), TextAlign::Left);
        // None → 默认 Start 在 LTR 下 = Left
        assert_eq!(resolve_text_align(None), TextAlign::Left);
    }

    #[test]
    fn test_resolve_text_align_last_mapping() {
        // text-align-last → Option<TextAlign> 映射（compute_final 存储路径 IFC 传递用）
        let mut style = ComputedStyle::default();
        // Auto（默认）→ None：末行跟随 text-align（justify 末行回退 Left）
        style.text_align_last = TextAlignLastValue::Auto;
        assert_eq!(resolve_text_align_last(Some(&style)), None);
        // Justify → Some(Justify)：末行也两端对齐（justify-all 语义）
        style.text_align_last = TextAlignLastValue::Justify;
        assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Justify));
        // Right → Some(Right)
        style.text_align_last = TextAlignLastValue::Right;
        assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Right));
        // Center → Some(Center)
        style.text_align_last = TextAlignLastValue::Center;
        assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Center));
        // Left → Some(Left)
        style.text_align_last = TextAlignLastValue::Left;
        assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Left));
        // 无 style 引用（None）→ 默认 Auto → None
        assert_eq!(resolve_text_align_last(None), None);
        // R958：start/end 方向感知（默认 LTR）
        style.direction = DirectionValue::Ltr;
        style.text_align_last = TextAlignLastValue::Start;
        assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Left));
        style.text_align_last = TextAlignLastValue::End;
        assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Right));
        // RTL 下翻转
        style.direction = DirectionValue::Rtl;
        style.text_align_last = TextAlignLastValue::Start;
        assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Right));
        style.text_align_last = TextAlignLastValue::End;
        assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Left));
    }

    #[test]
    fn test_resolve_text_indent_px_em_percentage() {
        // Px 直传
        assert_eq!(
            resolve_text_indent(&LengthValue::Px(40.0), &LengthValue::Px(16.0), 800.0),
            40.0
        );
        // Em × font_size：5em @ 16px → 80
        assert_eq!(
            resolve_text_indent(&LengthValue::Em(5.0), &LengthValue::Px(16.0), 800.0),
            80.0
        );
        // Percentage × container_width：50% @ 800 → 400
        assert_eq!(
            resolve_text_indent(&LengthValue::Percentage(50.0), &LengthValue::Px(16.0), 800.0),
            400.0
        );
        // 其他单位（Auto/Rem/…）回退 0
        assert_eq!(
            resolve_text_indent(&LengthValue::Auto, &LengthValue::Px(16.0), 800.0),
            0.0
        );
    }
}
