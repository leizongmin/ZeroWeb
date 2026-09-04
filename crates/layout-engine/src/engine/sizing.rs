//! 布局固有尺寸/纵横比修正辅助方法（从 `engine.rs` 抽出，run-rules §5 文件大小控制）。
//!
//! 含 `apply_intrinsic_content_sizing` / `apply_flex_aspect_ratio_item_size` /
//! `apply_aspect_ratio_container_cross_size` / `apply_indefinite_percent_height_to_auto`
//! / `gather_replaced_html_attr_intrinsic` / `resolve_percentage_padding` 等——
//! 皆为 `impl LayoutEngine` 的关联函数（无 `&self`，按参接收 taffy tree/styles），
//! 经 `Self::method(...)` 由 `engine.rs` 的 compute 方法调用。`pub(super)` 等价原
//! 「engine 模块私有」语义。

use super::*;

fn resolve_sizing_definite_real_length(value: &LengthValue, style: &ComputedStyle) -> Option<f32> {
    match value {
        LengthValue::Auto
        | LengthValue::Percentage(_)
        | LengthValue::MinContent
        | LengthValue::MaxContent
        | LengthValue::FitContent(_) => None,
        LengthValue::Px(v) if *v == f64::INFINITY => None,
        other => {
            let font_size_px = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
            let px = zero_style_system::computed::resolve_length(other, font_size_px, None, None);
            px.is_finite().then_some(px.max(0.0) as f32)
        }
    }
}

impl LayoutEngine {
    /// 两趟固有宽度布局的第一趟修正：对 `width:max-content`/`min-content` 的
    /// flex/grid 容器提升宽度到测得的 intrinsic。
    ///
    /// 这些容器在第一趟布局中塌缩为 ~0（converter 把 MaxContent/MinContent 映射为
    /// `length(0)`，与旧「resolve 为 Px(0)」行为中性）。`intrinsic_sizing` 模块基于
    /// **显式宽度**测量其 max-content 宽度（不依赖塌缩后的布局宽度），若可测
    /// （>0）且大于当前宽度，则把对应 taffy 节点的 size.width 设为 intrinsic 并
    /// `mark_dirty`。调用方随后重跑 `compute_layout_with_measure` 并重新提取，
    /// 该容器及其子元素即按 intrinsic 宽度重新布局（grid track / flex item 重新分配）。
    ///
    /// 安全性：仅「可测且确实更宽」时才改动（0→intrinsic 纯改善，非破坏）；
    /// intrinsic 不可测（如纯文本 item，Round C IFC 文本测量未就绪）的容器保持塌缩。
    /// 仅水平书写模式、width 为 MaxContent/MinContent 的 flex/grid 容器。
    ///
    /// 返回是否有节点被修改。
    pub(super) fn apply_intrinsic_content_sizing(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
        doc: &Document,
    ) -> bool {
        let mut changed = false;
        let mut stack: Vec<&LayoutBox> = vec![root];
        while let Some(b) = stack.pop() {
            stack.extend(b.children.iter());
            let Some(id) = b.node_id else { continue };
            let Some(s) = styles.get(&id) else { continue };
            // 仅水平书写模式的 flex/grid 容器，或 R1018 block-level（width:max-content/fit-content）
            let is_flex_grid = matches!(
                s.display,
                DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
            );
            let is_block = matches!(s.display, DisplayValue::Block);
            if !(is_flex_grid || is_block) || !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                continue;
            }
            // R1015/R1019：扩展 gate——除 MaxContent/MinContent 外，width:Auto + float（shrink-to-fit
            // 上下文）的 flex 容器或 block 容器也触发固有宽度计算。flex container float:left shrink
            // 到 item intrinsic；block container float:left 含 flex/grid 子时 shrink 到子 intrinsic
            //（aspect-ratio-intrinsic-014：float:left block + flex 子 + aspect-ratio item）。
            // float:block 由本 gate 处理（用 block_max_content_width 测 flex 子），float shrink
            // postprocess 路径（adjust_float_positions_with_context）见此宽度后为 no-op，无双重 shrink。
            let is_max_min = matches!(s.width, LengthValue::MaxContent | LengthValue::MinContent);
            let is_auto_float = matches!(s.width, LengthValue::Auto)
                && !matches!(s.float, FloatValue::None)
                && matches!(
                    s.display,
                    DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Block
                );
            // R3925（css-sizing-3 §fit-content(length-percentage)）：width:fit-content(arg) 的
            // arg 是上限不是定宽——fit-content = min(max-content, max(min-content, arg))。
            // converter 把它映射为定长 arg，内容超出 arg 时溢出、内容窄于 arg 时不收缩。
            // 任意 arg 形态都进入本 pass（百分比 arg 的解析值即第一趟布局宽 b.width），
            // target = min(解析 arg, intrinsic)（min-content 测量未实现，max-content 近似同 R1304）。
            let is_fitcontent = matches!(s.width, LengthValue::FitContent(_));
            let fitcontent_clamp = is_fitcontent.then_some(()).filter(|_| b.width > 1.0);
            if !is_max_min && !is_auto_float && !is_fitcontent {
                continue;
            }
            // R1018：block-level 仅在 width:MaxContent 或 auto-float 时触发（bare fit-content 经
            // parser 映射 MaxContent）。
            // R1304：block + MinContent 经 block_max_content_width 测（max-content 近似——固定宽/
            // 原子内容（img/固定宽子）min==max 正确；文本内容 overestimate 最宽词但远优于 0 塌缩；
            // true min-content 最宽词测量独立子问题，见 intrinsic_sizing.rs:29）。table-intrinsic-size
            // 簇（固定宽 .content 子）min==max 精确命中。kill-switch ZW_MINCONTENT_BLOCK=0 回退旧行为。
            let mincontent_block = std::env::var("ZW_MINCONTENT_BLOCK").as_deref() != Ok("0")
                && matches!(s.width, LengthValue::MinContent);
            let fitcontent_block = is_block && fitcontent_clamp.is_some();
            if is_block
                && !matches!(s.width, LengthValue::MaxContent)
                && !mincontent_block
                && !is_auto_float
                && !fitcontent_block
            {
                continue;
            }
            // R1018：block-level 用 block_max_content_width（对 flex/grid 子分发到专用 intrinsic）。
            // multicol 容器 intrinsic = columns × column-content，block_max_content_width 不解（只测
            // 单子宽）——可测时给出部分正确值（change-intrinsic-width -14pp），不可测时走下方 Auto-fallback。
            // multicol intrinsic sizing 精度（columns × content）独立 gap。
            let intrinsic: Option<f32> = if is_block {
                Some(crate::intrinsic_sizing::block_max_content_width(b, doc, styles))
            } else if matches!(s.display, DisplayValue::Grid | DisplayValue::InlineGrid) {
                crate::intrinsic_sizing::grid_intrinsic_width(b, doc, styles)
            } else if matches!(
                s.flex_direction,
                FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
            ) {
                crate::intrinsic_sizing::flex_column_intrinsic_width(b, doc, styles)
            } else {
                crate::intrinsic_sizing::flex_row_intrinsic_width(b, doc, styles)
            };
            let Some(intrinsic) = intrinsic else { continue };
            // intrinsic 不可测 → 跳过。否则按上下文判定 apply 条件：
            // - MaxContent/MinContent（grow）：current 比 intrinsic 窄 → grow 到 intrinsic。
            // - Auto+float（R1015 shrink-to-fit）：current 比 intrinsic 宽 → shrink 到 intrinsic。
            // R1018：block + MaxContent（含 bare fit-content）当 intrinsic 不可测（≤1，如 multicol
            // 容器或 aspect-ratio block 子 box_content 无法度量）时，回退 Auto（fill）而非留 0 塌缩
            // ——converter 已把 MaxContent width 映射 0，gate 测不出则元素归零（intrinsic-size-005
            // multicol + aspect-ratio 子回归）。fill（父宽）比 collapse 更接近 fit-content 语义。
            if intrinsic <= 1.0 {
                if is_block
                    && (matches!(s.width, LengthValue::MaxContent) || mincontent_block)
                    && let Some(&taffy_id) = dom_to_taffy.get(&id)
                    && let Ok(mut style) = taffy_tree.style(taffy_id).cloned()
                {
                    style.size.width = taffy::style::Dimension::auto();
                    let _ = taffy_tree.set_style(taffy_id, style);
                    let _ = taffy_tree.mark_dirty(taffy_id);
                    changed = true;
                }
                continue;
            }
            let should_apply = if is_fitcontent {
                // R3925：fit-content(arg) 双向钳制——target = min(解析后 arg, intrinsic)
                //（spec min(W_max, max(W_min, arg)) 的 max-content 近似），当前宽偏离
                // target >1px 即重设（converter 定宽 arg 在内容窄于 arg 时不会收缩）。
                let target = b.width.min(intrinsic);
                (b.width - target).abs() > 1.0
            } else if is_auto_float {
                b.width > intrinsic + 1.0
            } else {
                b.width < intrinsic + 1.0
            };
            if !should_apply {
                continue;
            }
            let Some(&taffy_id) = dom_to_taffy.get(&id) else {
                continue;
            };
            if let Ok(mut style) = taffy_tree.style(taffy_id).cloned() {
                let width = if is_fitcontent {
                    b.width.min(intrinsic)
                } else {
                    intrinsic
                };
                style.size.width = taffy::style::Dimension::length(width);
                let _ = taffy_tree.set_style(taffy_id, style);
                let _ = taffy_tree.mark_dirty(taffy_id);
                changed = true;
            }
        }
        changed
    }

    /// R3929（CSS2 §10.3.7/§10.6.4）：abspos 元素 shrink-to-fit 尺寸。
    ///
    /// 宽：width:auto + 水平 inset 非双定（双定 = stretch，taffy 已解）→ 宽 = 内容
    /// max-content（≤CB−已定 inset）。taffy 对全/半 auto inset 的 abspos 不做内容测量
    /// （layout dump 实证 0 宽，absolute-non-replaced-max-height-002：`&nbsp;` + Ahem
    /// 100px 应 100 宽，taffy 给 0）。
    /// 高：height:auto + 垂直 inset 非双定 → 高 = 行高（单行近似；taffy max_size 已按
    /// max-height 钳，002 案 100→50；009 案 top:25 定 + bottom:auto 同样收缩）。多行
    /// 折行测量独立 gap（FIXME）。
    pub(super) fn apply_abspos_shrink_to_fit_width(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
        doc: &Document,
    ) -> bool {
        use zero_css_parser::values::LengthValue;
        let mut changed = false;
        let mut stack: Vec<(&LayoutBox, f32, f32)> = vec![(root, f32::INFINITY, f32::INFINITY)];
        while let Some((b, cb_width, cb_height)) = stack.pop() {
            // CB 更新在 push 时——positioned 盒（含 contain CB）的子代 CB = 本盒宽/高。
            let child_is_cb = b.is_abspos_cb
                || b.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
                    matches!(
                        s.position,
                        zero_css_parser::values::PositionValue::Relative
                            | zero_css_parser::values::PositionValue::Absolute
                            | zero_css_parser::values::PositionValue::Fixed
                    )
                });
            let child_cb = if child_is_cb { b.width } else { cb_width };
            // CB 高按 **padding-box** 语义传递（% 固有高相对 CB padding-box；
            // b.height 是 border-box，剥自身 border）——消费方 R4018 臂直用。
            let child_cb_h = if child_is_cb {
                (b.height - b.border_top - b.border_bottom).max(0.0)
            } else {
                cb_height
            };
            for child in &b.children {
                stack.push((child, child_cb, child_cb_h));
            }
            let Some(id) = b.node_id else { continue };
            let Some(s) = styles.get(&id) else { continue };
            // R4018（CSS2 §10.6.6 + SVG2 sizing）：abspos svg 的 % attr 固有高——
            // `height="50%"` 是**存在的百分比声明**（非缺失），used 高 = % × abspos CB 高
            //（absolute-replaced-height-027/034：50% × 192 = 96，旧落 default 150）。
            // taffy 的 svg gate 对 % attr 无信号面（负 dh 已被比信号占用），taffy 布局后
            // 修正：仅在 CSS height auto 时覆写，% × cb_height 定值写入。
            if b.is_replaced
                && b.is_absolute
                && matches!(s.height, LengthValue::Auto)
                && cb_height.is_finite()
                && cb_height > 0.5
                && let Some(pct) = crate::svg_default_size::svg_attr_percentage_height(id, doc)
                && let Some(&taffy_id) = dom_to_taffy.get(&id)
                && let Ok(mut style) = taffy_tree.style(taffy_id).cloned()
            {
                let h = (pct / 100.0 * cb_height).max(0.0);
                if (b.height - h).abs() > 0.5 {
                    style.size.height = taffy::style::Dimension::length(h);
                    let _ = taffy_tree.set_style(taffy_id, style);
                    let _ = taffy_tree.mark_dirty(taffy_id);
                    changed = true;
                }
                continue;
            }
            // R4015/R4015b：replaced 排除例外——taffy 任一维塌 0 的 abspos replaced
            //（无 attr 固有尺寸，如 height-only svg / 无尺寸 svg 的 height 面）缺固有维
            // 解析，仍需 shrink-to-fit 补测（§10.3.8 + css-sizing-3 default object size）。
            // 双维均已解析的 replaced 维持排除（R1683/R3935 警告）。
            let replaced_collapsed = b.is_replaced && (b.width <= 0.5 || b.height <= 0.5);
            if !b.is_absolute || (b.is_replaced && !replaced_collapsed) {
                continue;
            }
            // R4034b（CSS Containment 1 §3）：contain:size（含 strict）→ 元素按「无内容」
            // sized——converter 已把 auto 尺寸解析为 0（或 CIS 替代），shrink-to-fit 补测
            // 不得把内容贡献拉回（contain-animation-001：contain:strict abspos div 的
            // nbsp 内容被补测拉宽 → 红底露出 4%）。R4018 svg % attr 臂不受影响（其已
            // continue 在前——attr 是存在的声明，非内容测量）。
            if s.contain.has_size() {
                continue;
            }
            // 内含 float 后代时跳过：float 子的 max-width/约束宽度参与 shrink-to-fit
            // preferred width（width-019/020），max-content 测量不含此语义。
            if Self::subtree_has_float(b) {
                continue;
            }
            let res_px = |v: &LengthValue| -> Option<f32> {
                match v {
                    LengthValue::Px(p) => Some(*p as f32),
                    _ => None,
                }
            };
            let left_def = res_px(&s.left);
            let right_def = res_px(&s.right);
            let top_def = res_px(&s.top);
            let bottom_def = res_px(&s.bottom);
            let mut width_fix = matches!(s.width, LengthValue::Auto) && !(left_def.is_some() && right_def.is_some());
            let mut height_fix = matches!(s.height, LengthValue::Auto) && !(top_def.is_some() && bottom_def.is_some());
            // taffy 已解出非 0 宽/高（如 width-019 的 float 内容、margin-applies-to 族）时不
            // 覆写——本 pass 只救 taffy 恒 0 的场景（全/半 auto inset 无内容测量）。
            if !width_fix && !height_fix {
                continue;
            }
            if width_fix && b.width > 0.5 {
                width_fix = false;
            }
            if height_fix && b.height > 0.5 {
                height_fix = false;
            }
            if !width_fix && !height_fix {
                continue;
            }
            let Some(&taffy_id) = dom_to_taffy.get(&id) else {
                continue;
            };
            if let Ok(mut style) = taffy_tree.style(taffy_id).cloned() {
                let width_collapsed = b.width <= 0.5;
                if width_fix && (width_collapsed || !b.is_replaced) {
                    let used = left_def.unwrap_or(0.0) + right_def.unwrap_or(0.0);
                    let available = (cb_width - used).max(0.0);
                    // R4015：塌 0 的 replaced 叶（height-only svg 等）——子树递归测 0（svg
                    // 内容不生成 in-flow 子贡献），用 svg default object size contribution
                    //（css-sizing-3：无固有宽/比 → 300；% 宽 → 300；ratio-only → 0）。
                    let measured = if replaced_collapsed {
                        crate::intrinsic_sizing::abspos_replaced_max_content(b, doc, styles)
                    } else {
                        crate::intrinsic_sizing::block_max_content_width(b, doc, styles)
                    }
                    .max(0.0);
                    if measured > 0.5 {
                        let target = if available.is_finite() {
                            measured.min(available)
                        } else {
                            measured
                        };
                        let frame = b.padding_left + b.padding_right + b.border_left + b.border_right;
                        let target_bw = (target + frame).max(0.0);
                        if b.width < target_bw - 0.5 {
                            style.size.width = taffy::style::Dimension::length(target_bw);
                        }
                    }
                }
                if height_fix {
                    // R4015b/R4016：replaced-collapse 的 height 侧——svg 有 attr/CSS 固有高
                    // 时优先用之（009/023/030：height="50" attr = 固有高 50，default 150 错），
                    // 无固有高回落 default object size 高 150（004：无任何尺寸来源 → 300×150）；
                    // 文本叶 line-height 近似不适用（svg 无行盒语义）。
                    let h = if replaced_collapsed {
                        crate::svg_default_size::svg_attr_intrinsic_height(b.node_id, doc, s)
                            .unwrap_or(crate::svg_default_size::SVG_DEFAULT_H)
                    } else {
                        let (fs, lh) = crate::inline::resolve_font_metrics(Some(s));
                        lh.max(fs).max(1.0)
                    };
                    style.size.height = taffy::style::Dimension::length(h);
                }
                let _ = taffy_tree.set_style(taffy_id, style);
                let _ = taffy_tree.mark_dirty(taffy_id);
                changed = true;
            }
        }
        changed
    }

    /// R3929 辅助：子树内是否有 float 盒（abspos 容器含 float 子时 shrink-to-fit 的
    /// preferred width 须含 float 约束语义，max-content 近似失准，见 width-019/020）。
    fn subtree_has_float(b: &LayoutBox) -> bool {
        if b.float != FloatValue::None {
            return true;
        }
        b.children.iter().any(Self::subtree_has_float)
    }

    /// R717（CSS §10.3.2 + Flexbox §4.5）：`aspect-ratio` flex item（ratio-only SVG `<img>`
    /// 或 CSS `aspect-ratio` 的 leaf 块）在 flex 容器内时，第一趟 taffy 对该 leaf 项无法
    /// 从 `aspect_ratio` + Auto-cross（容器 cross 尺寸在 computed style 中为 Auto，但实际
    /// 解析为视口/包含块尺寸）推导出 main 尺寸——item collapses 到 0。
    ///
    /// `apply_flex_transferred_min_size`（build_layout_tree 期）尝试设 transferred min，
    /// 但它读 `parent_style.width` 仅接受 `LengthValue::Px`，对 Auto 容器（007 驱动案：
    /// `<div style="display:flex;flex-direction:column">` 宽度 Auto→解析 800）提前返回。
    ///
    /// 本 pass 在**第一趟布局后**运行——此时 LayoutBox 已含解析出的 cross 尺寸（经
    /// align-stretch / 包含块解析）。对 leaf flex item（无 in-flow 子元素，故无内容决定 main）
    /// 且 main 轴 CSS 为 auto、taffy style 有 `aspect_ratio` 的项，按 cross × ratio（row）
    /// 或 cross / ratio（column）推导 main 尺寸，改写 taffy `size.main = Length(...)` 并
    /// mark_dirty，由调用方重跑 taffy。仅水平书写模式；仅当 cross>0 且 main 与推导值显著
    /// 不同时触发。leaf 限制避免误覆盖有文本/子内容决定 main 的 flex item。
    /// R1364：判断长度值是否为「零-ish」（Auto 或 Px(0)）。用于 flex item cross 轴
    /// padding/border 是否为零的守卫——cross=parent_cross 仅在无 padding/border 时精确。
    pub(super) fn is_zeroish_len(v: &LengthValue) -> bool {
        match v {
            LengthValue::Auto => true,
            LengthValue::Px(x) => *x == 0.0,
            _ => false,
        }
    }

    /// R1366 v2：flex item aspect-ratio main 按容器 stretched cross 推导（row-006）。
    pub(super) fn apply_flex_aspect_ratio_item_size(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> bool {
        use zero_css_parser::values::{DisplayValue, FlexDirectionValue, LengthValue};

        fn walk(
            b: &LayoutBox,
            parent_style: Option<&ComputedStyle>,
            taffy_tree: &mut TaffyTree<NodeId>,
            dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
            styles: &HashMap<NodeId, ComputedStyle>,
        ) -> bool {
            if !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                return false;
            }
            let mut changed = false;
            let my_style = b.node_id.and_then(|id| styles.get(&id));

            // leaf flex item（无 in-flow 子盒）+ 父是 flex 容器 + taffy style 有 aspect_ratio。
            if b.children.is_empty()
                && let Some(id) = b.node_id
                && let Some(ps) = parent_style
                && matches!(ps.display, DisplayValue::Flex | DisplayValue::InlineFlex)
                && let Some(item_style) = my_style
                && let Some(&tid) = dom_to_taffy.get(&id)
                && let Ok(mut st) = taffy_tree.style(tid).cloned()
                && let Some(ratio) = st.aspect_ratio
                && ratio > 0.0
            {
                let is_column = matches!(
                    ps.flex_direction,
                    FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
                );
                // main 轴 CSS 须为 auto（否则 converter 已从显式 CSS 处理，不应覆盖）。
                let main_is_auto = if is_column {
                    matches!(item_style.height, LengthValue::Auto)
                } else {
                    matches!(item_style.width, LengthValue::Auto)
                };
                // R1013：非替换 leaf（div + CSS aspect-ratio）+ main 轴 definite min-size 时跳过——
                // 此约束驱动尺寸（transferred-size 由 min-size × ratio 推导 cross），cross→main
                // 反向推导会覆盖并破坏（flex-item-transferred-sizes-padding 回归 +73pp 证）。
                // 替换元素（img/SVG）保留 fixup：其 transferred-size 由固有 ratio + cross 推导正确
                //（flex-aspect-ratio-img-column-006 / row-004 需 fixup 才 <1%，min-size 不改变语义）。
                // R993 driving case（aspect-ratio-intrinsic-size-007 SVG img）+ R994 +2（CSS aspect-ratio
                // leaf 无 min-size）均不受影响。
                // R3862：`min-*: 0`（显式零）不算 definite min 约束——R1013 skip 的本意是
                // 「真实非零 min 约束驱动 transferred-size」（其 driving 案 min-height:100px）；
                // 显式 0 不约束任何东西，却把 item 推进 skip 分支、错过 R1364 stretch-cross
                // 传递（flex-aspect-ratio-009：容器 height:100 + item aspect 1/1 +
                // min-width:0 → stretched cross 100 应传 main width=100，ZW 塌 0 宽）。
                let main_has_definite_min = if is_column {
                    resolve_sizing_definite_real_length(&item_style.min_height, item_style).is_some_and(|v| v > 0.0)
                } else {
                    resolve_sizing_definite_real_length(&item_style.min_width, item_style).is_some_and(|v| v > 0.0)
                };
                // R3859：R1013 skip 分支的**最小侵入补强**——非替换 leaf + main auto +
                // main min definite 时不做 cross→main 反向推导（R1013），但「definite min
                // **cross** × ratio → min main」（CSS Flexbox §4 transferred size suggestion，
                // css-sizing-4 §4）仍须喂给 taffy：taffy 自身不解 min-cross→min-main 传递，
                // min main = 0 时 aspect 主轴塌缩（flex-aspect-ratio-035：容器 width:0 +
                // min-width:100 → item min main(height) 应 = 100×1，ZW 塌 0 高）。
                // padding 守卫沿用 R1364（cross 轴有 padding 时 transfer 基准偏移，跳过保 R1013
                // baseline）。仅提升 min（max()），不覆盖 size——taffy 以 min 钳自然求解。
                // R3862 修订：本分支的触发键是「definite min **cross** > 0」（min 传递的
                // 语义源），非 main min definite——main min 零与否只决定 R1364 stretch
                // 传递是否适用（R3862 已把零 main min 让路给 R1364）。035（min-height:0 +
                // min-width:100）靠本分支而非 main-min 门。
                if main_is_auto && !b.is_replaced {
                    let definite_min_cross = if is_column {
                        resolve_sizing_definite_real_length(&item_style.min_width, item_style)
                    } else {
                        resolve_sizing_definite_real_length(&item_style.min_height, item_style)
                    };
                    let cross_has_no_box = if is_column {
                        LayoutEngine::is_zeroish_len(&item_style.padding_left)
                            && LayoutEngine::is_zeroish_len(&item_style.padding_right)
                    } else {
                        LayoutEngine::is_zeroish_len(&item_style.padding_top)
                            && LayoutEngine::is_zeroish_len(&item_style.padding_bottom)
                    };
                    if let (Some(min_cross), true) = (definite_min_cross, cross_has_no_box) {
                        let transferred_min_main = min_cross * ratio;
                        if let Ok(mut st) = taffy_tree.style(tid).cloned() {
                            let cur = if is_column {
                                st.min_size.height
                            } else {
                                st.min_size.width
                            };
                            let cur_px = if cur.is_auto() { 0.0 } else { cur.value() };
                            if transferred_min_main > cur_px + 0.5 {
                                if is_column {
                                    st.min_size.height = taffy::style::Dimension::length(transferred_min_main);
                                } else {
                                    st.min_size.width = taffy::style::Dimension::length(transferred_min_main);
                                }
                                let _ = taffy_tree.set_style(tid, st);
                                let _ = taffy_tree.mark_dirty(tid);
                                changed = true;
                            }
                        }
                    }
                }
                if main_is_auto && (!main_has_definite_min || b.is_replaced) {
                    // column: main=height, cross=width；row: main=width, cross=height。
                    let (main_resolved, cross_resolved) = if is_column {
                        (b.height, b.width)
                    } else {
                        (b.width, b.height)
                    };
                    // R1364：若 item cross 为 CSS-auto 且 flex 容器 cross 为 definite，用容器
                    // cross（将被 align-items:stretch 拉伸到的值）推 main，而非 b 的固有/预算 cross。
                    // 驱动 flex-aspect-ratio-img-row-006：img 固有 200x200 + width/height auto +
                    // 容器 height:100 → main(width) 应 = 100×ratio(1)=100，非固有 200×1=200。
                    // 仅 item cross CSS-auto（未显式指定，将被 stretch）+ 容器 definite cross 时覆盖。
                    // R3862：cross 轴 **auto margin** 优先吸收空间 → item 不被 stretch，
                    // cross 停留在 content 尺寸——用容器 cross 推 main 即错（flex-010：
                    // `margin: auto 0` 垂直居中场景 cross 保持 0，误传 main=容器高）。
                    let cross_margin_auto = if is_column {
                        matches!(item_style.margin_left, LengthValue::Auto)
                            || matches!(item_style.margin_right, LengthValue::Auto)
                    } else {
                        matches!(item_style.margin_top, LengthValue::Auto)
                            || matches!(item_style.margin_bottom, LengthValue::Auto)
                    };
                    let item_cross_is_auto = (if is_column {
                        matches!(item_style.width, LengthValue::Auto)
                    } else {
                        matches!(item_style.height, LengthValue::Auto)
                    }) && !cross_margin_auto;
                    let parent_cross_definite = if is_column {
                        resolve_sizing_definite_real_length(&ps.width, ps)
                    } else {
                        resolve_sizing_definite_real_length(&ps.height, ps)
                    };
                    let cross_resolved = if item_cross_is_auto {
                        parent_cross_definite.unwrap_or(cross_resolved)
                    } else {
                        cross_resolved
                    };
                    let expected_main = if is_column {
                        cross_resolved / ratio
                    } else {
                        cross_resolved * ratio
                    };
                    // 仅当 cross 已解析（>0）且 main 与推导值显著不同（collapsed 或不一致）时改写。
                    if cross_resolved > 0.0 && (main_resolved - expected_main).abs() > 0.5 {
                        if is_column {
                            st.size.height = taffy::style::Dimension::length(expected_main.max(0.5));
                        } else {
                            st.size.width = taffy::style::Dimension::length(expected_main.max(0.5));
                        }
                        // R1364：同步把 cross 设为容器 cross（stretch 目标值）。**仅当 item cross 轴
                        // 无 padding** 时（cross=parent_cross 精确；border-style:none 不渲染故 border-width
                        // 默认值不影响）——否则 cross 须减 padding（naive parent_cross 致 padding-001 回归）。
                        // row-006（img 无 padding）满足；padding-001（有 padding）跳过守 baseline。
                        let cross_has_no_box = if is_column {
                            LayoutEngine::is_zeroish_len(&item_style.padding_left)
                                && LayoutEngine::is_zeroish_len(&item_style.padding_right)
                        } else {
                            LayoutEngine::is_zeroish_len(&item_style.padding_top)
                                && LayoutEngine::is_zeroish_len(&item_style.padding_bottom)
                        };
                        if item_cross_is_auto
                            && cross_has_no_box
                            && let Some(pc) = parent_cross_definite
                        {
                            if is_column {
                                st.size.width = taffy::style::Dimension::length(pc.max(0.5));
                            } else {
                                st.size.height = taffy::style::Dimension::length(pc.max(0.5));
                            }
                        }
                        let _ = taffy_tree.set_style(tid, st);
                        let _ = taffy_tree.mark_dirty(tid);
                        changed = true;
                    }
                }
            }

            for c in &b.children {
                changed |= walk(c, my_style, taffy_tree, dom_to_taffy, styles);
            }
            changed
        }
        walk(root, None, taffy_tree, dom_to_taffy, styles)
    }

    /// R3913：row flex 容器（自身无 ratio + CSS height Auto）的 cross 从**flexed main ×
    /// item ratio** 传递（css-flexbox §9.2.3.B + css-sizing-4 §4，csswg #line-sizing 决议：
    /// aspect-ratio 传递按 **flexed** 主轴尺寸——011 item width:50 + flex:1 在 100px 容器
    /// flex 到 100 → transferred cross 100 → 容器高 100；ZW 塌 50 = 按指定宽传递）。
    ///
    /// 触发面（收窄）：单 item + row + item ratio>0 + 容器 height Auto 且自身无 ratio +
    /// item main **指定 Px**（flex-basis auto 用 width 作 base，grow 后 flexed ≠ specified
    /// 才有意义；main auto 的塌缩案归 R1366v2）+ 容器 main definite。首趟 taffy 后读
    /// LayoutBox 的 flexed main，设 item taffy size.cross 与容器 taffy size.cross =
    /// flexed_main / ratio（nowrap 假设——wrap 多线容器 max 语义另案），mark_dirty 重跑。
    /// kill-switch `ZW_AR_FLEX_CROSS_TRANSFER=0`（default-on）。
    pub(super) fn apply_flex_cross_from_flexed_main(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> bool {
        use zero_css_parser::values::{DisplayValue, FlexDirectionValue, FlexWrapValue, LengthValue};
        if std::env::var("ZW_AR_FLEX_CROSS_TRANSFER").as_deref() == Ok("0") {
            return false;
        }

        fn walk(
            b: &LayoutBox,
            taffy_tree: &mut TaffyTree<NodeId>,
            dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
            styles: &HashMap<NodeId, ComputedStyle>,
        ) -> bool {
            if !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                return false;
            }
            let mut changed = false;
            if b.children.len() == 1
                && let Some(id) = b.node_id
                && let Some(cs) = styles.get(&id)
                && matches!(cs.display, DisplayValue::Flex | DisplayValue::InlineFlex)
                && matches!(
                    cs.flex_direction,
                    FlexDirectionValue::Row | FlexDirectionValue::RowReverse
                )
                && matches!(cs.flex_wrap, FlexWrapValue::Nowrap)
                && matches!(cs.height, LengthValue::Auto)
                && cs.aspect_ratio.is_none()
                && resolve_sizing_definite_real_length(&cs.width, cs).is_some()
                && let Some(item) = b.children.first()
                && let Some(item_id) = item.node_id
                && let Some(item_style) = styles.get(&item_id)
                && !item.is_absolute
                && let Some(ratio) = item_style.aspect_ratio.filter(|&r| r > 0.0)
                && matches!(item_style.width, LengthValue::Px(_))
                && matches!(item_style.height, LengthValue::Auto)
                && let Some(&item_tid) = dom_to_taffy.get(&item_id)
                && let Some(&container_tid) = dom_to_taffy.get(&id)
                && let Ok(mut item_st) = taffy_tree.style(item_tid).cloned()
                && let Ok(mut container_st) = taffy_tree.style(container_tid).cloned()
            {
                // flexed main（row: width）来自首趟布局。
                let flexed_main = item.width;
                let transferred_cross = flexed_main / ratio;
                let container_cross = b.height;
                if transferred_cross > 0.5 && (container_cross - transferred_cross).abs() > 0.5 {
                    item_st.size.height = taffy::style::Dimension::length(transferred_cross);
                    container_st.size.height = taffy::style::Dimension::length(transferred_cross);
                    let _ = taffy_tree.set_style(item_tid, item_st);
                    let _ = taffy_tree.mark_dirty(item_tid);
                    let _ = taffy_tree.set_style(container_tid, container_st);
                    let _ = taffy_tree.mark_dirty(container_tid);
                    changed = true;
                }
            }
            for c in &b.children {
                changed |= walk(c, taffy_tree, dom_to_taffy, styles);
            }
            changed
        }
        walk(root, taffy_tree, dom_to_taffy, styles)
    }

    /// R3860：grid item「definite 单 row × stretch × aspect-ratio → cross 钳到 row、
    /// main 传递」（css-grid §6.6 + css-sizing-4 §3.2 transferred size）。
    ///
    /// taffy 0.12 grid 布局对带 aspect_ratio 的 item 先按 ratio（cross=列宽）解 main，
    /// 再被 align-self:stretch 拉伸时**不回传 ratio**——driving grid-aspect-ratio-028：
    /// `grid-template: 100px / 200px` + item `aspect-ratio:1/1; align-self:stretch` →
    /// ZW item 200×200 溢出 100px row（chromium 100×100：stretch 钳 height=row，ratio
    /// 传递 width=height×ratio）。
    ///
    /// 作用域（守卫收紧防误伤）：水平书写 + 父 grid + item 有 ratio + item CSS
    /// width/height 均 auto + **单条 definite 长度 row** + align-self（或容器 align_items
    /// 回退）为 stretch 语义（Stretch / Normal / Auto）+ taffy 已算高度 ≠ row（溢出
    /// 签名）→ 钳 st.size.height = row、st.size.width = row × ratio（min/max 尺寸
    /// 留给 taffy 钳）。align-start/content 对齐 item 的内容尺寸合法性不受影响。
    /// kill-switch `ZW_AR_GRID_STRETCH=0`（default-on）。
    pub(super) fn apply_grid_aspect_ratio_item_size(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> bool {
        use zero_css_parser::values::{DisplayValue, LengthValue};
        if std::env::var("ZW_AR_GRID_STRETCH").as_deref() == Ok("0") {
            return false;
        }
        fn walk(
            b: &LayoutBox,
            parent_style: Option<&ComputedStyle>,
            parent_taffy_id: Option<taffy::NodeId>,
            taffy_tree: &mut TaffyTree<NodeId>,
            dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
            styles: &HashMap<NodeId, ComputedStyle>,
        ) -> bool {
            if !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                return false;
            }
            let mut changed = false;
            let my_style = b.node_id.and_then(|id| styles.get(&id));
            let my_taffy_id = b.node_id.and_then(|id| dom_to_taffy.get(&id).copied());
            if let Some(id) = b.node_id
                && let Some(ps) = parent_style
                && matches!(ps.display, DisplayValue::Grid | DisplayValue::InlineGrid)
                && let Some(item_style) = my_style
                && matches!(item_style.width, LengthValue::Auto | LengthValue::Percentage(_))
                && matches!(item_style.height, LengthValue::Auto)
                && let Some(&tid) = dom_to_taffy.get(&id)
                && let Some(parent_tid) = parent_taffy_id
                && let Ok(mut st) = taffy_tree.style(tid).cloned()
                && let Some(ratio) = st.aspect_ratio
                && ratio > 0.0
                && let Ok(parent_tstyle) = taffy_tree.style(parent_tid).cloned()
            {
                // 单条 definite 长度 row（grid-template 单 track；repeat/多 track 不触）。
                let row_definite = match parent_tstyle.grid_template_rows.as_slice() {
                    [taffy::style::GridTemplateComponent::Single(f)] => f
                        .max_sizing_function()
                        .definite_value(None, |_, _| 0.0)
                        .or_else(|| f.min_sizing_function().definite_value(None, |_, _| 0.0)),
                    _ => None,
                };
                // 列 definite（单 track）同 row 提取——inline-axis stretch 传递需要。
                let col_definite = match parent_tstyle.grid_template_columns.as_slice() {
                    [taffy::style::GridTemplateComponent::Single(f)] => f
                        .max_sizing_function()
                        .definite_value(None, |_, _| 0.0)
                        .or_else(|| f.min_sizing_function().definite_value(None, |_, _| 0.0)),
                    _ => None,
                };
                use taffy::style::AlignItemsKeyword;
                // 仅**显式** stretch 关键字触发（normal/auto 在 inline 轴经 ratio 传递，
                // 不作为 track 拉伸——css-sizing-4「block-axis stretch preferred over
                // inline-axis normal」028/029；inline-axis stretch 对称 030）。
                let align_stretch = matches!(st.align_self.map(|a| a.keyword), Some(AlignItemsKeyword::Stretch));
                let justify_stretch = matches!(st.justify_self.map(|j| j.keyword), Some(AlignItemsKeyword::Stretch));
                let item_overflowing_row = row_definite.is_some_and(|r| (b.height - r).abs() > 0.5);
                let item_overflowing_col = col_definite.is_some_and(|c| (b.width - c).abs() > 0.5);
                // R3861：block-stretch + inline **definite 长度**（px 或 %→col track）→
                // ratio 忽略（css-sizing-4：一轴 stretch + 另轴 definite 长度 = 双轴约束），
                // 双钳 track（034/035：aspect 2/1 或 1/2 + width:100% → 100×100 而非 ratio 高）。
                let inline_definite = matches!(item_style.width, LengthValue::Percentage(_));
                if align_stretch
                    && !justify_stretch
                    && inline_definite
                    && let (Some(row_h), Some(col_w)) = (row_definite, col_definite)
                    && (item_overflowing_row || item_overflowing_col)
                {
                    st.size.height = taffy::style::Dimension::length(row_h.max(0.5));
                    st.size.width = taffy::style::Dimension::length(col_w.max(0.5));
                    let _ = taffy_tree.set_style(tid, st);
                    let _ = taffy_tree.mark_dirty(tid);
                    changed = true;
                } else if align_stretch
                    && !justify_stretch
                    && let Some(row_h) = row_definite
                    && item_overflowing_row
                {
                    // block-axis stretch 胜 inline normal：height=row、width=ratio 传递
                    //（可溢出列轨，029 列 50 item 100）。
                    st.size.height = taffy::style::Dimension::length(row_h.max(0.5));
                    st.size.width = taffy::style::Dimension::length((row_h * ratio).max(0.5));
                    let _ = taffy_tree.set_style(tid, st);
                    let _ = taffy_tree.mark_dirty(tid);
                    changed = true;
                } else if justify_stretch
                    && !align_stretch
                    && let Some(col_w) = col_definite
                    && item_overflowing_col
                {
                    // inline-axis stretch 胜 block normal：width=column、height 反向传递。
                    st.size.width = taffy::style::Dimension::length(col_w.max(0.5));
                    st.size.height = taffy::style::Dimension::length((col_w / ratio).max(0.5));
                    let _ = taffy_tree.set_style(tid, st);
                    let _ = taffy_tree.mark_dirty(tid);
                    changed = true;
                } else if align_stretch
                    && justify_stretch
                    && let (Some(row_h), Some(col_w)) = (row_definite, col_definite)
                    && (item_overflowing_row || item_overflowing_col)
                {
                    // 双轴显式 stretch：两轴都被 track 约束，ratio 让位（css-grid §6.6）。
                    st.size.height = taffy::style::Dimension::length(row_h.max(0.5));
                    st.size.width = taffy::style::Dimension::length(col_w.max(0.5));
                    let _ = taffy_tree.set_style(tid, st);
                    let _ = taffy_tree.mark_dirty(tid);
                    changed = true;
                }
            }

            for c in &b.children {
                changed |= walk(c, my_style, my_taffy_id, taffy_tree, dom_to_taffy, styles);
            }
            changed
        }
        walk(root, None, None, taffy_tree, dom_to_taffy, styles)
    }

    /// R2171：flex/grid **容器**自身 cross 尺寸从 aspect-ratio + Auto-main 推导（taffy 0.12.1 gap）。
    /// 驱动 flex-aspect-ratio-cross-size-002：outer{display:flex; aspect-ratio:4}（width:auto→200，
    /// height:auto）taffy 给 height=0，应 width/ratio=200/4=50。实测 taffy 仅在 main **显式** Px 时
    /// 应用 ar（-001 .flex width:400px → h=200）；main 为 Auto（解析到 definite）时不事后应用 ar
    /// → 容器 cross 塌缩到 0（standards + quirks 同）。chromium 两案都应用 ar（h=50）。
    ///
    /// 本 pass 在第一趟后运行：对 flex/grid 容器（非替换）+ ar + main/cross 均 CSS-Auto +
    /// **cross 当前为 0**（taffy 失败模式）+ main 已解析 definite（>0）→ 推导 cross 并 set taffy
    /// size + mark_dirty，由调用方重跑 taffy。`cross==0` 守卫把作用域精确锁在 taffy 失败案——
    /// taffy 已给非零 cross（内容/main-explicit 等正确案）不受影响。仅水平书写模式；row/column
    /// 对称。kill-switch `ZW_AR_CONTAINER_CROSS=0`（default-on）。
    pub(super) fn apply_aspect_ratio_container_cross_size(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> bool {
        use zero_css_parser::values::{DisplayValue, FlexDirectionValue, LengthValue};
        if std::env::var("ZW_AR_CONTAINER_CROSS").as_deref() == Ok("0") {
            return false;
        }
        if !matches!(root.writing_mode, WritingModeValue::HorizontalTb) {
            return false;
        }

        fn walk(
            b: &LayoutBox,
            taffy_tree: &mut TaffyTree<NodeId>,
            dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
            styles: &HashMap<NodeId, ComputedStyle>,
        ) -> bool {
            if !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                return false;
            }
            let mut changed = false;
            let Some(style) = b.node_id.and_then(|id| styles.get(&id)) else {
                for c in &b.children {
                    changed |= walk(c, taffy_tree, dom_to_taffy, styles);
                }
                return changed;
            };
            let is_flex_grid = matches!(
                style.display,
                DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
            );
            // 仅 ar + main/cross 均 Auto 的容器（taffy 失败案）；非替换（替换走固有尺寸路径）。
            // R3994 注：plain block 的传递改由 postprocess `transfer_aspect_ratio_height`
            // 在**最终宽度**上做（first-pass 宽度未含 float 避让/BFC 收缩，floats-aspect-ratio-001
            // 会传 200 而非避让后 40）。
            if is_flex_grid
                && !b.is_replaced
                && let Some(ratio) = style.aspect_ratio.filter(|&r| r > 0.0)
                && matches!(style.width, LengthValue::Auto)
                && matches!(style.height, LengthValue::Auto)
                && let Some(id) = b.node_id
                && let Some(&tid) = dom_to_taffy.get(&id)
                && let Ok(mut st) = taffy_tree.style(tid).cloned()
            {
                let is_column = matches!(
                    style.flex_direction,
                    FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
                );
                // row: main=width(definite), cross=height(collapsed)→ height=width/ratio。
                // column: main=height(definite), cross=width(collapsed)→ width=height×ratio。
                let (main, cross) = if is_column {
                    (b.height, b.width)
                } else {
                    (b.width, b.height)
                };
                if main > 0.5 && cross < 0.5 {
                    let derived_cross = if is_column { main * ratio } else { main / ratio };
                    if is_column {
                        st.size.width = taffy::style::Dimension::length(derived_cross.max(0.5));
                    } else {
                        st.size.height = taffy::style::Dimension::length(derived_cross.max(0.5));
                    }
                    let _ = taffy_tree.set_style(tid, st);
                    let _ = taffy_tree.mark_dirty(tid);
                    changed = true;
                }
            }
            for c in &b.children {
                changed |= walk(c, taffy_tree, dom_to_taffy, styles);
            }
            changed
        }
        walk(root, taffy_tree, dom_to_taffy, styles)
    }

    /// R695（CSS §10.5）：百分比 `height` 仅当包含块高度**明确指定**时才解析，
    /// 否则 compute-to-auto。taffy 0.7 对「百分比 height + 不明确 CB」回退到 CB
    /// **宽度**解析（非规范），致 `grandparent{height:0} > parent{auto} >
    /// child{height:100%}` 链中 child/img 被拉到满宽（如 784）。
    ///
    /// 本 pass 自上而下按**样式**判定 CB 高度明确性（与 [`clamp_percentage_max_height`]
    /// 的 `my_definite_content_height` 同语义），对水平书写模式 normal-flow 块级元素
    /// 的 `height:Percentage`（CB 不明确）改写 taffy `size.height = Auto`。替换元素
    /// 同时补设固有绝对尺寸（无 HTML width/height 属性时 taffy style 不含绝对固有
    /// 尺寸，仅 aspect_ratio）。返回是否有改动；调用方据此重跑 taffy——第二趟里
    /// taffy 正确计算非替换块的内容高度 / 替换元素的固有尺寸，无需手工重算。
    ///
    /// 范围限定：跳过 abspos（由 `adjust_absolute_pct_to_viewport` 处理）；跳过
    /// flex/grid item（其 %height 有独立 stretch 语义，taffy-gated，见 R691）。常见
    /// `html,body{height:100%}` 不受影响——根 CB 为视口（明确），整条链明确。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply_indefinite_percent_height_to_auto(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
        img_intrinsic_sizes: &HashMap<NodeId, (f32, f32)>,
        viewport_height: f32,
        quirks_mode: bool,
        html_attr_intrinsic_ids: &HashSet<NodeId>,
    ) -> bool {
        use zero_css_parser::values::{BoxSizingValue, DisplayValue, LengthValue, PositionValue};

        #[allow(clippy::too_many_arguments)]
        fn walk(
            b: &LayoutBox,
            cb_definite: Option<f32>,
            parent_is_flex_grid: bool,
            // R3795：包含块宽度是否为 intrinsic 关键字（min-content/max-content/fit-content）。
            // 此上下文中 % height compute-to-auto 的叶盒不得保留 taffy aspect_ratio——
            // shrink-to-fit CB 下 cross 是第一趟可用宽伪影（058 outer 784），ar 反推 main
            // 会撑出 784×784（应 content 0）。definite CB（030 width:100px）保留 ar 供
            // transferred（child 100×100 ✓）。
            cb_width_intrinsic: bool,
            // R2101：当前 box 是否处于 table-cell 子树内（含自身为 table-cell）。
            // CSS Quirks §percentage-height：百分比高度 quirk（不明确 CB 按 ICB 解析）**不适用**
            // 于 table-cell 的后代——后代 height:% 须 compute-to-auto（standards 行为）。
            inside_table_cell: bool,
            // R2170：当前 box 是否处于 flex/grid 容器子树内（含自身为 flex/grid 容器）。
            // 驱动 flex-aspect-ratio-cross-size-002（quirks 模式无 DOCTYPE）：嵌套 flex 容器内
            // 后代 `height:100%` 不应按 ICB quirks 解析（否则以视口高度 inflate 容器）——flex/grid
            // 容器高度由 flex/grid 算法决定，非 quirks-definite。与 R2101 table-cell 同型 gate。
            // chromium quirks 实测：flex 子树内百分比高度 compute-to-auto（不 inflate）。
            inside_flex_grid: bool,
            // R2107：最近 definite-height 祖先的内容高度（穿透 auto-height 祖先，
            // 不像 cb_definite 在 auto 上 reset）。CSS Quirks §percentage-height：百分比高度
            // 解析应针对「first ancestor with a defined height」非恒 ICB——有 definite 祖先时
            // 解析对其高度，否则回退 ICB。None 仅理论（root 起手 Some(viewport)）。
            quirks_nearest_definite: Option<f32>,
            quirks_mode: bool,
            viewport_height: f32,
            taffy_tree: &mut TaffyTree<NodeId>,
            dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
            styles: &HashMap<NodeId, ComputedStyle>,
            img_intrinsic_sizes: &HashMap<NodeId, (f32, f32)>,
            html_attr_intrinsic_ids: &HashSet<NodeId>,
        ) -> bool {
            // 垂直书写模式块轴为 X，高度语义不同——保守跳过整棵子树。
            if !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                return false;
            }
            let mut changed = false;
            let style = b.node_id.and_then(|id| styles.get(&id));

            // R2170：自身是否为 flex/grid 容器（供 gate + 子代 inside_flex_grid 传播）。
            let self_is_flex_grid = style.is_some_and(|s| {
                matches!(
                    s.display,
                    DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
                )
            });

            // 本元素提供给子元素的「明确内容高度」（None = 不明确）。
            // 默认沿用父级传入的明确性（无样式节点如匿名盒透传）。
            let mut my_definite: Option<f32> = cb_definite;

            if let Some(s) = style {
                let is_abs = matches!(s.position, PositionValue::Absolute | PositionValue::Fixed);
                // R2091：grid/flex item 中 **HTML-attr-only intrinsic 替换元素**（canvas/embed/
                // object/applet，即 `gather_replaced_html_attr_intrinsic` 收集的；排除 img——img
                // 有 decoded intrinsic，其 definite-track grid 百分比案须保持 taffy 原生解析）
                // + Percentage height + indefinite CB → taffy 在 indefinite 容器中 double-resolve
                //（容器从祖父辈尺寸、item 再按 aspect_ratio 重解，致 `<canvas width=10 height=10
                // style="height:200%">` 作 grid item 时 h=400 vs intrinsic 10）。本 gate 把此类
                // item 交本 pass 处理（走下方 else 分支 → auto+intrinsic）。definite CB（容器定高）
                // 不入此 gate → 百分比正常解析。kill-switch `ZW_GRID_REPLACED_PCT_INDEFINITE=0`。
                let is_html_attr_intrinsic_replaced = b.node_id.is_some_and(|id| html_attr_intrinsic_ids.contains(&id));
                let grid_replaced_pct_indefinite = parent_is_flex_grid
                    && is_html_attr_intrinsic_replaced
                    && cb_definite.is_none()
                    && matches!(s.height, LengthValue::Percentage(_))
                    && std::env::var("ZW_GRID_REPLACED_PCT_INDEFINITE").as_deref() != Ok("0");
                if !is_abs && (!parent_is_flex_grid || grid_replaced_pct_indefinite) {
                    match &s.height {
                        LengthValue::Percentage(p) => match cb_definite {
                            Some(cbh) => {
                                // 明确 CB → 解析为百分比（明确），供子元素继续链。
                                my_definite = Some(*p as f32 / 100.0 * cbh);
                            }
                            None => {
                                // R2170 kill-switch：ZW_QUIRKS_PCT_FLEX_GATE=0 禁用 flex/grid gate
                                //（回退旧行为——flex 子树内仍按 ICB quirks 解析，驱动测试将 fail）。
                                // R2170：flex/grid 子树内禁用 ICB quirks（kill-switch
                                // ZW_QUIRKS_PCT_FLEX_GATE=0 回退旧行为）。
                                let quirk_blocked_by_flex_grid =
                                    inside_flex_grid && std::env::var("ZW_QUIRKS_PCT_FLEX_GATE").as_deref() != Ok("0");
                                if quirks_mode && !b.is_replaced && !inside_table_cell && !quirk_blocked_by_flex_grid {
                                    // R2016 quirks mode（CSS quirks §percentage-height）：不明确 CB
                                    //（auto 父）的百分比 height 解析——legacy「百分比高度生效」行为。
                                    // R2107：解析针对**最近 definite-height 祖先**（穿透 auto 祖先），
                                    // 而非恒 ICB（chromium quirks：float-percentage-resolution-quirks-mode
                                    // 实测解析对 first ancestor with defined height）。无 definite 祖先时
                                    // 回退 ICB（viewport_height）。非替换块专用（替换元素保留固有尺寸回退）。
                                    // box-sizing 折算内容高供子链解析。
                                    let pb = b.padding_top + b.padding_bottom + b.border_top + b.border_bottom;
                                    let base = quirks_nearest_definite.unwrap_or(viewport_height);
                                    let resolved = (*p as f32 / 100.0 * base).max(0.0);
                                    if let Some(id) = b.node_id
                                        && let Some(&tid) = dom_to_taffy.get(&id)
                                        && let Ok(mut st) = taffy_tree.style(tid).cloned()
                                    {
                                        st.size.height = taffy::style::Dimension::length(resolved);
                                        let _ = taffy_tree.set_style(tid, st);
                                        let _ = taffy_tree.mark_dirty(tid);
                                        changed = true;
                                    }
                                    my_definite = Some(if matches!(s.box_sizing, BoxSizingValue::BorderBox) {
                                        (resolved - pb).max(0.0)
                                    } else {
                                        resolved
                                    });
                                } else {
                                    // standards 或替换元素：compute-to-auto：改写 taffy height 为 Auto。
                                    if let Some(id) = b.node_id
                                        && let Some(&tid) = dom_to_taffy.get(&id)
                                        && let Ok(mut st) = taffy_tree.style(tid).cloned()
                                    {
                                        st.size.height = taffy::style::Dimension::auto();
                                        // R3795（css-sizing-4 §4.1 + bugzilla 1918576）：%
                                        // height compute-to-auto 的叶盒，shrink-to-fit CB
                                        //（width:min/max/fit-content——block-aspect-ratio-058
                                        // `.outer{width:min-content; max-height:100px}` 内
                                        // `.target{height:100%; aspect-ratio:1/1}`）下第一趟
                                        // cross 是可用宽伪影（784），taffy 经 aspect_ratio 反推
                                        // main → 784×784 红满屏（chromium 不 transfer，应
                                        // content 0）。清 aspect_ratio 阻断。definite CB
                                        //（030 width:100px → child 拉伸 cross 100 definite）保留
                                        // ar 供 transferred（100×100 ✓）。仅叶盒 + 非替换
                                        //（替换元素 ar 供固有尺寸回退）。
                                        if cb_width_intrinsic && b.children.is_empty() && !b.is_replaced {
                                            st.aspect_ratio = None;
                                        }
                                        // 替换元素补设固有绝对尺寸：taffy 需要绝对值才能
                                        // 在两侧 auto 时定尺寸（aspect_ratio 只够推导比例）。
                                        if b.is_replaced
                                            && let Some(&(iw, ih)) = img_intrinsic_sizes.get(&id)
                                        {
                                            let iw = iw.max(1.0);
                                            let ih = ih.max(1.0);
                                            if matches!(s.width, LengthValue::Auto) {
                                                st.size.width = taffy::style::Dimension::length(iw);
                                            }
                                            st.size.height = taffy::style::Dimension::length(ih);
                                            if st.aspect_ratio.is_none() {
                                                st.aspect_ratio = Some(iw / ih);
                                            }
                                        }
                                        let _ = taffy_tree.set_style(tid, st);
                                        let _ = taffy_tree.mark_dirty(tid);
                                        changed = true;
                                    }
                                    // 现为 auto（内容决定）→ 子元素 CB 不明确。
                                    my_definite = None;
                                }
                            }
                        },
                        other => {
                            // 明确高度：按 box-sizing 折算内容高度供子元素百分比解析。
                            my_definite = resolve_sizing_definite_real_length(other, s).map(|v| {
                                let pb = b.padding_top + b.padding_bottom + b.border_top + b.border_bottom;
                                if !matches!(other, LengthValue::Px(_))
                                    && (b.height - v).abs() > 0.5
                                    && let Some(id) = b.node_id
                                    && let Some(&tid) = dom_to_taffy.get(&id)
                                    && let Ok(mut st) = taffy_tree.style(tid).cloned()
                                {
                                    st.size.height = taffy::style::Dimension::length(v);
                                    let _ = taffy_tree.set_style(tid, st);
                                    let _ = taffy_tree.mark_dirty(tid);
                                    changed = true;
                                }
                                if matches!(s.box_sizing, BoxSizingValue::BorderBox) {
                                    (v - pb).max(0.0)
                                } else {
                                    v
                                }
                            });
                        }
                    }
                }
            }

            // 子元素是否为 flex/grid item（其 %height 走独立语义，本 pass 跳过）。
            // R2170：复用 self_is_flex_grid（本盒为 flex/grid → 子代为 flex/grid item）。
            let child_parent_flex_grid = self_is_flex_grid;

            // R2101：当前 box 若为 table-cell，则其子树标记为「table-cell 内」，
            // 阻断后代 height:% 的 quirks ICB 解析。
            let self_is_table_cell = style.is_some_and(|s| matches!(s.display, DisplayValue::TableCell));
            let child_inside_table_cell = inside_table_cell || self_is_table_cell;

            // R2170：子代「是否在 flex/grid 子树内」= 本盒在 flex/grid 子树内 OR 本盒自身为 flex/grid。
            let child_inside_flex_grid = inside_flex_grid || self_is_flex_grid;

            // R2107：子代的「最近 definite 祖先」——本盒 definite 时更新为本盒内容高，
            // 否则透传继承（auto 盒不阻断）。供 quirks 百分比高度按 first definite ancestor 解析。
            let child_quirks_nearest = my_definite.or(quirks_nearest_definite);

            // R3795：子代 CB = 本盒；本盒 width 为 intrinsic 关键字时子代处于 shrink-to-fit
            // CB（% height + ar 反推伪影来源）。
            let child_cb_width_intrinsic = style.is_some_and(|s| {
                matches!(
                    s.width,
                    LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_)
                )
            });

            for child in &b.children {
                changed |= walk(
                    child,
                    my_definite,
                    child_parent_flex_grid,
                    child_cb_width_intrinsic,
                    child_inside_table_cell,
                    child_inside_flex_grid,
                    child_quirks_nearest,
                    quirks_mode,
                    viewport_height,
                    taffy_tree,
                    dom_to_taffy,
                    styles,
                    img_intrinsic_sizes,
                    html_attr_intrinsic_ids,
                );
            }
            changed
        }

        walk(
            root,
            Some(viewport_height),
            false,
            false,
            false,
            false,
            Some(viewport_height),
            quirks_mode,
            viewport_height,
            taffy_tree,
            dom_to_taffy,
            styles,
            img_intrinsic_sizes,
            html_attr_intrinsic_ids,
        )
    }

    /// R2091：从 DOM 收集 `canvas`/`embed`/`object`/`applet` 的 HTML `width`/`height` 属性
    /// 固有尺寸，注入 `intrinsic_for_r695` 供 R2016 else 分支为这些替换元素补设 intrinsic。
    ///
    /// caller 传入的 `img_intrinsic_sizes` 只含 decoded `<img>` 像素尺寸；canvas 等替换元素
    /// 无解码像素，其固有尺寸来自 HTML 属性（在 `tree.rs::apply_replaced_element_sizing` 消费
    /// 但不回填该 map）。R2016 的 else 分支（替换元素 + Percentage height + indefinite CB）
    /// 读 `img_intrinsic_sizes` 取不到 canvas → 高度置 auto → taffy 在 grid/flex 中按
    /// justify-self:stretch + aspect_ratio 重解致尺寸错误（grid-item-percentage-quirk-001）。
    /// 本函数补齐该缺口。`<img>` 不在此收集（caller 已填 decoded 尺寸；HTML-attr-only img 由
    /// tree.rs SVG data URI 回退处理，且其在 img_intrinsic_sizes 缺失时 R2016 行为不变）。
    pub(super) fn gather_replaced_html_attr_intrinsic(
        doc: &zero_dom::Document,
        root: &LayoutBox,
    ) -> HashMap<zero_dom::NodeId, (f32, f32)> {
        use zero_dom::NodeKind;
        let mut map = HashMap::new();
        let mut stack: Vec<&LayoutBox> = vec![root];
        while let Some(b) = stack.pop() {
            if b.is_replaced
                && let Some(nid) = b.node_id
                && let Some(node) = doc.get(nid)
                && let NodeKind::Element(elem) = &node.kind
                && matches!(elem.local_name(), "canvas" | "embed" | "object" | "applet")
            {
                // https://html.spec.whatwg.org/multipage/embedded-content.html#dimension-attributes
                let attr_w = elem
                    .get_attribute("width")
                    .and_then(|v| v.parse::<f32>().ok().filter(|n| n.is_finite()));
                let attr_h = elem
                    .get_attribute("height")
                    .and_then(|v| v.parse::<f32>().ok().filter(|n| n.is_finite()));
                if let (Some(w), Some(h)) = (attr_w, attr_h)
                    && w > 0.0
                    && h > 0.0
                {
                    map.insert(nid, (w.max(1.0), h.max(1.0)));
                }
            }
            stack.extend(&b.children);
        }
        map
    }

    /// CSS §8.3/§8.4：百分比 padding 相对**包含块的内容宽度**解析（与元素自身宽度无关）。
    ///
    /// taffy 0.7 的 `LengthPercentage::Percent` padding 在多数布局路径上解析为 0
    /// （实测 `#box{width:150px;padding:20%}` 在 800px 视口内 pt=0，应 160）。
    /// 本 pass 在第一趟布局（父级 content_width 已确定）后，把百分比 padding 预解析为
    /// 绝对 px，改写 taffy style 为 `Length(px)` 并 mark_dirty，由 compute() 重跑。
    ///
    /// 非循环：百分比 padding 仅依赖父级内容宽（第一趟已知），不依赖元素自身宽度，
    /// 故一次重跑即可收敛（与 R695 %height 同模式）。
    pub(super) fn resolve_percentage_padding(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
        viewport_width: f32,
    ) -> bool {
        use zero_css_parser::values::LengthValue;

        fn walk(
            b: &LayoutBox,
            parent_content_width: f32,
            taffy_tree: &mut TaffyTree<NodeId>,
            dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
            styles: &HashMap<NodeId, ComputedStyle>,
        ) -> bool {
            // 垂直书写模式下块轴为 X，padding 百分比语义不同——保守跳过。
            if !matches!(b.writing_mode, WritingModeValue::HorizontalTb) {
                return false;
            }
            let mut changed = false;
            let style = b.node_id.and_then(|id| styles.get(&id));

            // 本元素提供给子元素的「内容宽度」（百分比 padding 的解析基准）。
            // taffy 第一趟已算出 content_width（b.content_width）；匿名盒透传父级宽度。
            let my_content_width = if b.content_width > 0.0 {
                b.content_width
            } else {
                parent_content_width
            };

            if let Some(s) = style {
                let has_pct = matches!(s.padding_top, LengthValue::Percentage(_))
                    || matches!(s.padding_right, LengthValue::Percentage(_))
                    || matches!(s.padding_bottom, LengthValue::Percentage(_))
                    || matches!(s.padding_left, LengthValue::Percentage(_));
                if has_pct
                    && let Some(id) = b.node_id
                    && let Some(&tid) = dom_to_taffy.get(&id)
                    && let Ok(mut st) = taffy_tree.style(tid).cloned()
                {
                    let resolve = |v: &LengthValue| match v {
                        LengthValue::Percentage(p) => {
                            taffy::style::LengthPercentage::length((*p as f32 / 100.0 * parent_content_width).max(0.0))
                        }
                        // 其它值保持原 taffy 值（converter 已转换）；此处只覆盖百分比。
                        _ => taffy::style::LengthPercentage::length(0.0),
                    };
                    // 仅改写为百分比的边，其余保留 taffy 已转换值。
                    if let LengthValue::Percentage(_) = s.padding_top {
                        st.padding.top = resolve(&s.padding_top);
                    }
                    if let LengthValue::Percentage(_) = s.padding_right {
                        st.padding.right = resolve(&s.padding_right);
                    }
                    if let LengthValue::Percentage(_) = s.padding_bottom {
                        st.padding.bottom = resolve(&s.padding_bottom);
                    }
                    if let LengthValue::Percentage(_) = s.padding_left {
                        st.padding.left = resolve(&s.padding_left);
                    }
                    let _ = taffy_tree.set_style(tid, st);
                    let _ = taffy_tree.mark_dirty(tid);
                    changed = true;
                }
            }

            for child in &b.children {
                changed |= walk(child, my_content_width, taffy_tree, dom_to_taffy, styles);
            }
            changed
        }

        walk(root, viewport_width, taffy_tree, dom_to_taffy, styles)
    }

    /// CSS 堆叠上下文（stacking context）触发器（CSS 2.1 §9.9 + CSS3）：
    /// (1) positioned 元素 + 显式整数 z-index；(2) opacity < 1（CSS3，R505 scope）；
    /// (3) `isolation: isolate`（CSS Compositing §3，R2302 补——隔离后代与祖先背景的混合）；
    /// (4) R2309 补：非 none 的 `filter`/`backdrop-filter`/`clip-path`/`will-change`、
    ///     非 normal 的 `mix-blend-mode`、含 paint/layout 的 `contain`（CSS Filter Effects /
    ///     CSS Masking / CSS Will Change / CSS Compositing §3.5 / CSS Containment §4）；
    /// (5) R2310 补：非 none 的 `transform`（CSS Transforms §6）——这些属性值都会建立 SC，
    ///     使后代与祖先背景隔离（paint 层已消费此标记做 paint-order/scope）。
    /// 抽出为独立纯函数便于单测（creates_stacking_context 在 extract_layout 内联组装）。
    pub(crate) fn style_creates_stacking_context(is_positioned: bool, s: &ComputedStyle) -> bool {
        (is_positioned && matches!(s.z_index, ZIndexValue::Integer(_)))
            || s.opacity < 1.0
            || matches!(s.isolation, IsolationValue::Isolate)
            || !s.will_change.is_empty()
            || !s.filter.is_empty()
            || !s.backdrop_filter.is_empty()
            || !matches!(s.mix_blend_mode, MixBlendModeComputedValue::Normal)
            || !matches!(s.clip_path, ClipPathComputedValue::None)
            || s.contain.has_paint()
            || s.contain.has_layout()
            || !matches!(s.transform, zero_css_parser::values::TransformValue::None)
    }
}
