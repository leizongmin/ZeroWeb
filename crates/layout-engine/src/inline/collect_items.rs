// 行内条目收集方法（collect_inline_items）— 从 mod.rs 拆分以控制文件体积
// （include! 模式，≡ apps/browser/src/app.rs → app_input.rs；零行为/可见性变更）
impl InlineFormattingContext {
    /// 收集容器中所有行内级内容（文本节点 + inline 元素 + `<br>` 元素），
    /// 从 ComputedStyle 中读取 font-size 和 line-height。
    fn collect_inline_items(
        &self,
        doc: &Document,
        container: NodeId,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> Vec<InlineItem> {
        let mut items = Vec::new();
        // R109 §9.2.1.1：匿名块盒片段只收集该片段的 inline 内容（fragment_node_ids），
        // 而非 container 的全部 DOM 子节点。None = 正常遍历 container 子节点。
        let children: Vec<NodeId> = match &self.fragment_node_ids {
            Some(ids) => ids.clone(),
            None => doc.child_nodes(container),
        };

        for &child_id in &children {
            if let Some(node) = doc.get(child_id) {
                match &node.kind {
                    NodeKind::Text(text_data) => {
                        // CSS Text §4.1: 白空格折叠 — 将连续空白字符折叠为单个空格，
                        // 但不在此阶段去除（行首/行尾空格由 IFC break_items_into_lines 处理）。
                        // 保留仅含空白的文本节点为单个空格（用于 inline-block 之间的间隔）。
                        //
                        // CSS Text §3.1：white-space: pre / pre-wrap / break-spaces 模式下
                        // **不折叠空白**，原始文本（含换行符 `\n`、连续空格、制表符）原样保留——
                        // `\n` 在 break_into_lines 中作为强制换行机会（见 split_into_words）。
                        // 旧实现无条件 collapse_whitespace，把 `\n` 折叠为普通空格 → 多行
                        // `<pre>` 内容塌缩为一行（如 morning-work 文章代码块垂直压缩）。
                        let text = if self.preserve_whitespace {
                            text_data.content.clone()
                        } else {
                            collapse_whitespace(&text_data.content)
                        };
                        if !text.is_empty() {
                            // 文本节点没有自己的 ComputedStyle，查找父元素
                            let parent_id = doc.parent_node(child_id);
                            let style = parent_id.and_then(|pid| styles.get(&pid));
                            let (font_size, line_height) = if style.is_some() {
                                // U1b：layout IFC（有真实 styles）首消费 font_metric_provider，
                                // 使 line-height:normal 用 per-font 真实度量。provider 缺省
                                // （生产默认 None）时逐字节等价于 resolve_font_metrics。
                                resolve_font_metrics_with_provider(style, self.font_metric_provider.as_ref())
                            } else if let Some(pid) = parent_id {
                                // paint IFC 传入空 styles：使用 layout IFC 存储的 font_size 覆盖
                                // 替代 16px 默认值，使字符宽度和行高计算更准确
                                if let Some(&fs) = self.font_size_overrides.get(&pid) {
                                    // line-height 覆盖：使用 layout IFC 存储的真实 line-height，
                                    // 而非 font_size * 1.2 近似值。line-height 仅影响行盒高度，
                                    // 不影响行断行为，因此传递覆盖是安全的。
                                    let lh = self
                                        .line_height_overrides
                                        .get(&pid)
                                        .copied()
                                        .unwrap_or(fs * NORMAL_LINE_HEIGHT_RATIO);
                                    (fs, lh)
                                } else {
                                    self.default_font_metrics
                                        .unwrap_or((DEFAULT_FONT_SIZE, DEFAULT_FONT_SIZE * NORMAL_LINE_HEIGHT_RATIO))
                                }
                            } else {
                                self.default_font_metrics
                                    .unwrap_or((DEFAULT_FONT_SIZE, DEFAULT_FONT_SIZE * NORMAL_LINE_HEIGHT_RATIO))
                            };
                            let vertical_align = style
                                .map(|s| s.vertical_align.clone())
                                .unwrap_or(VerticalAlignValue::Baseline);
                            let letter_spacing = style
                                .map(|s| Self::resolve_letter_spacing(&s.letter_spacing, font_size))
                                .unwrap_or_else(|| {
                                    // paint IFC（空 styles）：使用覆盖映射获取 letter-spacing
                                    parent_id
                                        .and_then(|pid| self.letter_spacing_overrides.get(&pid).copied())
                                        .unwrap_or(0.0)
                                });
                            let word_spacing = style
                                .map(|s| Self::resolve_word_spacing(&s.word_spacing, font_size))
                                .unwrap_or_else(|| {
                                    parent_id
                                        .and_then(|pid| self.word_spacing_overrides.get(&pid).copied())
                                        .unwrap_or(0.0)
                                });
                            // R1012：text-transform 须在行断前应用，使 layout 用转换后
                            // 文本宽度行断（与 chromium 一致）。layout IFC（有 styles）读
                            // 父元素 computed text-transform；paint Path B（空 styles）走
                            // text_transform_overrides 覆盖（re-key 到父元素）。
                            let text_transform = style.map(|s| s.text_transform).unwrap_or_else(|| {
                                parent_id
                                    .and_then(|pid| self.text_transform_overrides.get(&pid).copied())
                                    .unwrap_or(TextTransformValue::None)
                            });
                            let text = text_transform.apply(&text);
                            let is_ahem_font = style
                                .map(|s| s.font_family.iter().any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem")))
                                .unwrap_or_else(|| {
                                    // paint IFC（空 styles）：使用覆盖映射检测 Ahem 字体
                                    parent_id
                                        .and_then(|pid| self.is_ahem_overrides.get(&pid).copied())
                                        .unwrap_or(false)
                                });
                            items.push(InlineItem::Text(TextRun {
                                text,
                                node_id: child_id,
                                font_size,
                                line_height,
                                vertical_align,
                                letter_spacing,
                                word_spacing,
                                margin_left: 0.0,
                                margin_right: 0.0,
                                padding_top: 0.0,
                                padding_bottom: 0.0,
                                border_top: 0.0,
                                border_bottom: 0.0,
                                is_ahem_font,
                                font_id: self.shaping_font_id_for_style(
                                    Some(child_id),
                                    style,
                                    is_ahem_font,
                                    letter_spacing,
                                    word_spacing,
                                    false,
                                ),
                                is_rtl: style.is_some_and(|s| {
                                    matches!(s.direction, zero_style_system::DirectionValue::Rtl)
                                }),
                                is_plaintext_bidi: style
                                    .map(|s| {
                                        matches!(s.unicode_bidi, zero_style_system::UnicodeBidiValue::Plaintext)
                                    })
                                    .unwrap_or_else(|| {
                                        self.plaintext_bidi_override
                                            || parent_id.is_some_and(|id| self.plaintext_bidi_overrides.contains(&id))
                                    }),
                            }));
                        }
                    }
                    NodeKind::Element(elem_data) => {
                        // `<br>` 元素产生强制换行条目
                        if elem_data.local_name() == "br" {
                            items.push(InlineItem::Br);
                            continue;
                        }

                        // R1682：`<wbr>` 是零宽断行机会标记（HTML §12.3）——无可见渲染，仅提示
                        // 换行。跳过不产生 InlineItem → 零宽不可见（修 R1676 latent gap：旧把它当
                        // 普通 inline 元素收集 text_content 渲成可见盒）。断行机会语义（长词在 wbr
                        // 处可断）是 line-breaker 增强，本 slice 只修零宽可见性。
                        if elem_data.local_name() == "wbr" {
                            continue;
                        }

                        // CSS2 §9.4.3/§9.7：position:absolute/fixed 元素脱离常规流（含
                        // 行内流），不参与 IFC 行盒——由 abspos pass 独立定位/绘制。旧实现
                        // 把它们当 inline 盒收入 IFC，其全高撑大行盒 max_ascent，错位
                        // baseline-对齐的 inline-block（vertical-align-baseline-004a 的
                        // position:absolute ruler img 撑大行盒致 inline-block 下移 ~51px）。
                        // float 不在此跳过（由 float exclusion 路径单独 shaping 行盒）。
                        // kill-switch ZW_IFC_SKIP_OOF=0 关闭（回退旧行为：OOF 元素留入 IFC）。
                        // 仅 horizontal 模式跳过：vertical-rl 的 abspos shrink-to-fit 尺寸依赖
                        // IFC 内测量（writing_mode_tests），且 vertical 是 R1043 已知结构性缺口。
                        let style = styles.get(&child_id);
                        if !self.vertical
                            && runtime_flags::skip_oof()
                            && style
                                .is_some_and(|s| matches!(s.position, PositionValue::Absolute | PositionValue::Fixed))
                        {
                            continue;
                        }

                        // CSS 2.1 §9.2.1.1 匿名块盒生成：
                        // 当 inline 元素包含 block-level 子元素时，inline 元素
                        // 被拆分为匿名块盒。这里简化处理：如果子元素是 block-level
                        // display，强制换行（与 <br> 类似），跳过其文本内容。
                        // block-level 子元素由 taffy 正常布局为独立的块盒。
                        let is_block_level = style.is_some_and(|s| {
                            matches!(
                                s.display,
                                DisplayValue::Block
                                    | DisplayValue::Flex
                                    | DisplayValue::Grid
                                    | DisplayValue::Table
                                    | DisplayValue::ListItem
                                    | DisplayValue::FlowRoot
                            )
                        });
                        if is_block_level {
                            // R57（M3）：in-flow block 子 → BlockBreak（无 R1286 空行 strut——
                            // block 前被折叠的空白行不应获得 line-height，canvas-grid 22px 偏移
                            // 根因）；**浮动元素保留 Br**（r1733 float-avoidance 依赖旧 strut
                            // 语义定位 inline-block 与 float 的可用宽）。
                            if style.is_some_and(crate::inline_block_split::is_out_of_flow) {
                                items.push(InlineItem::Br);
                            } else {
                                items.push(InlineItem::BlockBreak);
                            }
                            continue;
                        }

                        // 检查该元素是否为原子行内级盒（inline-block / inline-flex / inline-grid / inline-table）。
                        // 这些元素参与行内格式化上下文，作为不可拆分的原子盒。
                        let stored_inline_size = self.inline_block_sizes.get(&child_id).copied();
                        let is_inline_block = stored_inline_size.is_some()
                            || style.is_some_and(|s| {
                                matches!(
                                    s.display,
                                    DisplayValue::InlineBlock
                                        | DisplayValue::InlineFlex
                                        | DisplayValue::InlineGrid
                                        | DisplayValue::InlineTable
                                )
                            });

                        if is_inline_block {
                            // 从 CSS 计算样式提取尺寸（仅支持绝对长度单位）
                            let mut w = style
                                .map(|s| resolve_inline_block_dimension(&s.width, s, /* is_width */ true))
                                .unwrap_or(0.0);
                            let mut h = style
                                .map(|s| resolve_inline_block_dimension(&s.height, s, /* is_width */ false))
                                .unwrap_or(0.0);
                            // IFC 中原子行内盒参与排版的是 used border-box。计算样式的 width/height
                            // 可能是 content-box，不能直接拿来推进下一项；优先使用已完成布局的盒尺寸。
                            if let Some((lw, lh)) = stored_inline_size {
                                w = lw;
                                h = lh;
                            }
                            // R57（M3）：replaced 元素（canvas/video/iframe/embed/object）的
                            // HTML width/height 属性固有尺寸回退（同下方 img 分支语义）——
                            // CSS 为 auto 时 `resolve_inline_block_dimension` 返 0，且主
                            // inline_finalization 的 inline_block_sizes 仅收集 CSS 非 auto
                            // 尺寸，canvas 曾落空降级为 inline 文本（fallback 内容
                            // "FAIL (fallback content)" 文本宽 ~188px 覆盖 taffy 固有 400px，
                            // 2d.reset.render.global_composite_operation oracle A/B 6.7%）。
                            if w <= 0.0 || h <= 0.0 {
                                if matches!(
                                    elem_data.local_name(),
                                    "canvas" | "video" | "iframe" | "embed" | "object" | "applet"
                                ) {
                                    if w <= 0.0 {
                                        w = elem_data
                                            .get_attribute("width")
                                            .and_then(|v| v.parse::<f32>().ok())
                                            .unwrap_or(0.0)
                                            .max(0.0);
                                    }
                                    if h <= 0.0 {
                                        h = elem_data
                                            .get_attribute("height")
                                            .and_then(|v| v.parse::<f32>().ok())
                                            .unwrap_or(0.0)
                                            .max(0.0);
                                    }
                                }
                            }
                            if w > 0.0 && h > 0.0 {
                                let vertical_align =
                                    style.map(|s| s.vertical_align.clone()).unwrap_or(VerticalAlignValue::Baseline);
                                // 计算基线：
                                // - inline-block：基线在底部边缘
                                // - inline-flex/inline-grid：基线从第一个子元素合成
                                //   优先使用 baseline_overrides（由 adjust_inline_block_positions
                                //   从 LayoutBox 子元素位置计算），回退到 height/2
                                let baseline = if let Some(&b) = self.baseline_overrides.get(&child_id) {
                                    b
                                } else {
                                    match style.map(|s| &s.display) {
                                        Some(DisplayValue::InlineFlex | DisplayValue::InlineGrid) => h * 0.5,
                                        Some(DisplayValue::InlineBlock) => {
                                            // CSS §10.8.1：inline-block 基线 = 其最后 in-flow 行盒基线；
                                            // 但「无 in-flow 行盒」或 overflow != visible 时基线 = 底 margin edge
                                            // （h + margin-bottom）。adjust_inline_block_positions 早于
                                            // compute_final_inline_layouts，无法读 IB 自身行盒；「空元素（无 DOM
                                            // 子节点）」必无行盒可静态判定，overflow 值亦可从计算样式直接读取。
                                            let no_line_boxes = doc.first_child(child_id).is_none();
                                            let clips = style.is_some_and(|s| {
                                                !matches!(s.overflow_x, OverflowValue::Visible)
                                                    || !matches!(s.overflow_y, OverflowValue::Visible)
                                            });
                                            if no_line_boxes || clips {
                                                h + style
                                                    .map(|s| Self::resolve_inline_margin(&s.margin_bottom, s))
                                                    .unwrap_or(0.0)
                                            } else {
                                                h
                                            }
                                        }
                                        _ => h, // inline-table: 基线在底部
                                    }
                                };
                                items.push(InlineItem::InlineBlock(InlineBlockBox {
                                    width: w,
                                    height: h,
                                    node_id: child_id,
                                    vertical_align,
                                    baseline,
                                    margin_top: style
                                        .map(|s| Self::resolve_inline_margin(&s.margin_top, s))
                                        .unwrap_or(0.0),
                                    margin_right: style
                                        .map(|s| Self::resolve_inline_margin(&s.margin_right, s))
                                        .or_else(|| self.margin_overrides.get(&child_id).map(|(_, right)| *right))
                                        .unwrap_or(0.0),
                                    margin_bottom: style
                                        .map(|s| Self::resolve_inline_margin(&s.margin_bottom, s))
                                        .unwrap_or(0.0),
                                    margin_left: style
                                        .map(|s| Self::resolve_inline_margin(&s.margin_left, s))
                                        .or_else(|| self.margin_overrides.get(&child_id).map(|(left, _)| *left))
                                        .unwrap_or(0.0),
                                }));
                                continue;
                            }
                            // 无有效尺寸的 inline-block 降级为零宽度 TextRun
                        }

                        // `<img>` 替换元素：作为原子行内级盒（不可拆分）参与 IFC。
                        // 尺寸来源优先级：HTML width/height 属性 → CSS computed width/height →
                        // LayoutBox 预计算尺寸（含百分比解析和固有尺寸回退）。
                        if elem_data.local_name() == "img" {
                            let mut w = elem_data
                                .get_attribute("width")
                                .and_then(|v| v.parse::<f32>().ok())
                                .unwrap_or(0.0)
                                .max(0.0);
                            let mut h = elem_data
                                .get_attribute("height")
                                .and_then(|v| v.parse::<f32>().ok())
                                .unwrap_or(0.0)
                                .max(0.0);
                            // HTML 属性不足时，回退到 CSS computed style
                            if w <= 0.0 || h <= 0.0 {
                                if let Some(s) = styles.get(&child_id) {
                                    if w <= 0.0 {
                                        let css_w = resolve_inline_block_dimension(&s.width, s, true);
                                        if css_w > 0.0 {
                                            w = css_w;
                                        }
                                    }
                                    if h <= 0.0 {
                                        let css_h = resolve_inline_block_dimension(&s.height, s, false);
                                        if css_h > 0.0 {
                                            h = css_h;
                                        }
                                    }
                                }
                            }
                            // CSS 属性仍不足时（如 width:100% 是百分比，resolve 返回 0），
                            // 尝试从 CSS 百分比值 + 容器尺寸解析。
                            if w <= 0.0 || h <= 0.0 {
                                if let Some(s) = styles.get(&child_id) {
                                    if w <= 0.0 {
                                        if let LengthValue::Percentage(pct) = &s.width {
                                            let resolved = (*pct as f32 / 100.0) * self.container_width;
                                            if resolved > 0.0 {
                                                w = resolved;
                                            }
                                        }
                                    }
                                    if h <= 0.0 {
                                        if let LengthValue::Percentage(pct) = &s.height {
                                            // 百分比高度相对于包含块高度；
                                            // measure callback 上下文中暂用 0（无法解析）。
                                            let _ = pct;
                                        }
                                    }
                                }
                            }
                            // 回退到 LayoutBox 预计算尺寸（由 taffy 从 CSS 百分比 + 固有尺寸计算）。
                            if w <= 0.0 || h <= 0.0 {
                                if let Some(&(lw, lh)) = self.inline_block_sizes.get(&child_id) {
                                    if w <= 0.0 {
                                        w = lw;
                                    }
                                    if h <= 0.0 {
                                        h = lh;
                                    }
                                }
                            }
                            // R1578：以上回退全部无法给出两侧维度时，若 img 恰有一侧已知
                            //（显式 width 或 height，如 `class="h-6"` = height:24px / width:auto），
                            // 用解码固有宽高比推导缺失侧。解「inline 元素（`<a>`/`<span>`）包裹
                            // auto-width img 致 IFC 不收集 → 父容器塌缩 h=0」（wintertc footer）。
                            // env-gated `ZW_IFC_IMG_INTRINSIC`（default-on，`=0` 关闭）；
                            // 排除 vertical（R109-blocked，沿用 R1576 gate）；两侧都未知不推导
                            //（避免与 final path `apply_replaced_element_sizing` 的 default-object-size
                            // 300×150 冲突）。eff_ratio 与 tree.rs:597 一致：CSS aspect-ratio 优先。
                            if Self::ifc_img_intrinsic_enabled()
                                && !self.vertical
                                && (w > 0.0) != (h > 0.0)
                                && let Some(&(iw, ih)) = self.img_intrinsic_sizes.get(&child_id)
                                && iw > 0.0
                                && ih > 0.0
                            {
                                let eff_ratio = styles.get(&child_id).and_then(|s| s.aspect_ratio).unwrap_or(iw / ih);
                                if w > 0.0 && h <= 0.0 {
                                    h = (w / eff_ratio).max(0.5);
                                } else if h > 0.0 && w <= 0.0 {
                                    w = (h * eff_ratio).max(0.5);
                                }
                            }
                            if w > 0.0 && h > 0.0 {
                                let img_style = styles.get(&child_id);
                                let vertical_align = img_style
                                    .map(|s| s.vertical_align.clone())
                                    .unwrap_or(VerticalAlignValue::Baseline);
                                // img 替换元素的基线在底部边缘
                                items.push(InlineItem::InlineBlock(InlineBlockBox {
                                    width: w,
                                    height: h,
                                    node_id: child_id,
                                    vertical_align,
                                    baseline: h,
                                    margin_top: img_style
                                        .map(|s| Self::resolve_inline_margin(&s.margin_top, s))
                                        .unwrap_or(0.0),
                                    margin_right: img_style
                                        .map(|s| Self::resolve_inline_margin(&s.margin_right, s))
                                        .unwrap_or(0.0),
                                    margin_bottom: img_style
                                        .map(|s| Self::resolve_inline_margin(&s.margin_bottom, s))
                                        .unwrap_or(0.0),
                                    margin_left: img_style
                                        .map(|s| Self::resolve_inline_margin(&s.margin_left, s))
                                        .unwrap_or(0.0),
                                }));
                                continue;
                            }
                            // 无有效尺寸的 img 降级为零宽度 TextRun
                        }

                        // R1576 inline-box-model：若 inline 元素含**嵌套 inline-block 后代**，
                        // 递归收集（保留 atomic inline 盒参与行盒高度计算），否则保持扁平化文本
                        //（向后兼容，纯文本 inline 行为不变）。修复 `<p><a><img class=inline-block></a></p>`
                        // 的 `<p>` 塌缩 h=0（旧扁平化 `text_content` 漏嵌套 inline-block，IFC 产 0 item）。
                        // env `ZW_INLINE_BOX_RECURSE=0` 关闭。仅当后代有 inline-block 才递归（最小行为变化）。
                        if Self::inline_box_model_recurse()
                            && !self.vertical
                            && Self::inline_elem_has_nested_inline_block(doc, styles, child_id)
                        {
                            let nested = self.collect_inline_items(doc, child_id, styles);
                            items.extend(nested);
                            continue;
                        }

                        // 其他 inline 元素的文本内容也收集进来
                        // R1022：<ruby> 默认 text_content 会扁平化 <rt>/<rp> 文本
                        // （● 当行内字符渲染）。改为只收集 rb 文本作 inline 流，
                        // rt 文本由 paint 期作 zero-width annotation 上移到 rb 之上。
                        let text = if elem_data.local_name() == "ruby" {
                            Self::collect_text_excluding(doc, child_id, &["rt", "rp"])
                        } else {
                            doc.text_content(child_id).unwrap_or_default()
                        };
                        let trimmed = collapse_whitespace(&text);
                        let style = styles.get(&child_id);
                        let (font_size, line_height) = if style.is_some() {
                            // U1b：layout IFC（有真实 styles）首消费 font_metric_provider
                            // （per-font line-height）。provider 缺省时等价于 resolve_font_metrics。
                            resolve_font_metrics_with_provider(style, self.font_metric_provider.as_ref())
                        } else if let Some(&(fs, lh)) = self.inline_element_metrics.get(&child_id) {
                            // paint IFC（空 styles）：使用 layout IFC 存储的 (font_size, line_height)
                            // 这仅影响行盒高度（垂直定位），不影响行断。
                            (fs, lh)
                        } else {
                            self.default_font_metrics
                                .unwrap_or((DEFAULT_FONT_SIZE, DEFAULT_FONT_SIZE * NORMAL_LINE_HEIGHT_RATIO))
                        };
                        let vertical_align = style
                            .map(|s| s.vertical_align.clone())
                            .unwrap_or(VerticalAlignValue::Baseline);
                        let letter_spacing = style
                            .map(|s| Self::resolve_letter_spacing(&s.letter_spacing, font_size))
                            .unwrap_or_else(|| self.letter_spacing_overrides.get(&child_id).copied().unwrap_or(0.0));
                        let word_spacing = style
                            .map(|s| Self::resolve_word_spacing(&s.word_spacing, font_size))
                            .unwrap_or_else(|| self.word_spacing_overrides.get(&child_id).copied().unwrap_or(0.0));
                        // 提取 inline 元素的水平 margin
                        // 优先从 style 获取；若无 style（paint IFC），使用 margin_overrides。
                        let margin_left = style
                            .map(|s| Self::resolve_inline_margin(&s.margin_left, s))
                            .unwrap_or_else(|| self.margin_overrides.get(&child_id).map(|(ml, _)| *ml).unwrap_or(0.0));
                        let margin_right = style
                            .map(|s| Self::resolve_inline_margin(&s.margin_right, s))
                            .unwrap_or_else(|| self.margin_overrides.get(&child_id).map(|(_, mr)| *mr).unwrap_or(0.0));
                        let is_ahem_font = style
                            .map(|s| s.font_family.iter().any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem")))
                            .unwrap_or_else(|| self.is_ahem_overrides.get(&child_id).copied().unwrap_or(false));
                        // CSS 2.1: inline 元素的 padding 和 border 参与行盒高度计算
                        let (padding_top, padding_bottom, border_top, border_bottom) =
                            Self::extract_inline_box_metrics(style);
                        if !trimmed.is_empty() {
                            items.push(InlineItem::Text(TextRun {
                                text: trimmed,
                                node_id: child_id,
                                font_size,
                                line_height,
                                vertical_align,
                                letter_spacing,
                                word_spacing,
                                margin_left,
                                margin_right,
                                padding_top,
                                padding_bottom,
                                border_top,
                                border_bottom,
                                is_ahem_font,
                                font_id: self.shaping_font_id_for_style(
                                    Some(child_id),
                                    style,
                                    is_ahem_font,
                                    letter_spacing,
                                    word_spacing,
                                    elem_data.local_name() == "ruby",
                                ),
                                is_rtl: style.is_some_and(|s| {
                                    matches!(s.direction, zero_style_system::DirectionValue::Rtl)
                                }),
                                is_plaintext_bidi: style
                                    .map(|s| {
                                        matches!(s.unicode_bidi, zero_style_system::UnicodeBidiValue::Plaintext)
                                    })
                                    .unwrap_or_else(|| {
                                        self.plaintext_bidi_override
                                            || self.plaintext_bidi_overrides.contains(&child_id)
                                    }),
                            }));
                        } else {
                            // CSS 规范：空 inline 元素仍需通过 line-height + padding + border 影响行盒高度
                            // 生成零宽度 TextRun，贡献 line-height + padding + border
                            items.push(InlineItem::Text(TextRun {
                                text: String::new(),
                                node_id: child_id,
                                font_size,
                                line_height,
                                vertical_align,
                                letter_spacing: 0.0,
                                word_spacing: 0.0,
                                margin_left,
                                margin_right,
                                padding_top,
                                padding_bottom,
                                border_top,
                                border_bottom,
                                is_ahem_font,
                                font_id: None,
                                is_rtl: style.is_some_and(|s| {
                                    matches!(s.direction, zero_style_system::DirectionValue::Rtl)
                                }),
                                is_plaintext_bidi: style
                                    .map(|s| {
                                        matches!(s.unicode_bidi, zero_style_system::UnicodeBidiValue::Plaintext)
                                    })
                                    .unwrap_or_else(|| {
                                        self.plaintext_bidi_override
                                            || self.plaintext_bidi_overrides.contains(&child_id)
                                    }),
                            }));
                        }
                    }
                    _ => {}
                }
            }
        }

        items
    }

    fn resolve_word_spacing(value: &LengthValue, font_size: f32) -> f32 {
        match value {
            LengthValue::Px(v) => *v as f32,
            LengthValue::Percentage(p) => font_size * (*p as f32 / 100.0),
            other => zero_style_system::computed::resolve_length(other, font_size as f64, None, None) as f32,
        }
    }

    fn resolve_letter_spacing(value: &LengthValue, font_size: f32) -> f32 {
        match value {
            LengthValue::Px(v) => *v as f32,
            other => zero_style_system::computed::resolve_length(other, font_size as f64, None, None) as f32,
        }
    }

    fn resolve_inline_margin(value: &LengthValue, style: &ComputedStyle) -> f32 {
        match value {
            LengthValue::Auto
            | LengthValue::Percentage(_)
            | LengthValue::MinContent
            | LengthValue::MaxContent
            | LengthValue::FitContent(_) => 0.0,
            other => {
                let font_size_px =
                    zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
                let px = zero_style_system::computed::resolve_length(other, font_size_px, None, None);
                if px.is_finite() { px as f32 } else { 0.0 }
            }
        }
    }
}
