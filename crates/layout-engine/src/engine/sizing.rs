//! 布局固有尺寸/纵横比修正辅助方法（从 `engine.rs` 抽出，run-rules §5 文件大小控制）。
//!
//! 含 `apply_intrinsic_content_sizing` / `apply_flex_aspect_ratio_item_size` /
//! `apply_aspect_ratio_container_cross_size` / `apply_indefinite_percent_height_to_auto`
//! / `gather_replaced_html_attr_intrinsic` / `resolve_percentage_padding` 等——
//! 皆为 `impl LayoutEngine` 的关联函数（无 `&self`，按参接收 taffy tree/styles），
//! 经 `Self::method(...)` 由 `engine.rs` 的 compute 方法调用。`pub(super)` 等价原
//! 「engine 模块私有」语义。

use super::*;

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
            if !is_max_min && !is_auto_float {
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
            if is_block && !matches!(s.width, LengthValue::MaxContent) && !mincontent_block && !is_auto_float {
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
            let should_apply = if is_auto_float {
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
                style.size.width = taffy::style::Dimension::length(intrinsic);
                let _ = taffy_tree.set_style(taffy_id, style);
                let _ = taffy_tree.mark_dirty(taffy_id);
                changed = true;
            }
        }
        changed
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
                let main_has_definite_min = if is_column {
                    matches!(item_style.min_height, LengthValue::Px(_))
                } else {
                    matches!(item_style.min_width, LengthValue::Px(_))
                };
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
                    // 仅 item cross CSS-auto（未显式指定，将被 stretch）+ 容器 cross Px 时覆盖。
                    let item_cross_is_auto = if is_column {
                        matches!(item_style.width, LengthValue::Auto)
                    } else {
                        matches!(item_style.height, LengthValue::Auto)
                    };
                    let parent_cross_definite = if is_column {
                        matches!(ps.width, LengthValue::Px(_)).then(|| match ps.width {
                            LengthValue::Px(v) => v as f32,
                            _ => 0.0,
                        })
                    } else {
                        matches!(ps.height, LengthValue::Px(_)).then(|| match ps.height {
                            LengthValue::Px(v) => v as f32,
                            _ => 0.0,
                        })
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
                        LengthValue::Px(v) => {
                            // 明确高度：按 box-sizing 折算内容高度供子元素百分比解析。
                            let pb = b.padding_top + b.padding_bottom + b.border_top + b.border_bottom;
                            my_definite = Some(if matches!(s.box_sizing, BoxSizingValue::BorderBox) {
                                (*v as f32 - pb).max(0.0)
                            } else {
                                *v as f32
                            });
                        }
                        _ => {
                            // Auto / Em / Rem 等内容决定型 → 子元素 CB 不明确。
                            my_definite = None;
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

            for child in &b.children {
                changed |= walk(
                    child,
                    my_definite,
                    child_parent_flex_grid,
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
                let attr_w = elem.get_attribute("width").and_then(|v| v.parse::<f32>().ok());
                let attr_h = elem.get_attribute("height").and_then(|v| v.parse::<f32>().ok());
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
