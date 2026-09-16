// 行内条目收集方法（collect_inline_items）— 从 mod.rs 拆分以控制文件体积
// （include! 模式，≡ apps/browser/src/app.rs → app_input.rs；零行为/可见性变更）
impl InlineFormattingContext {
    fn capture_downloaded_font_metrics(
        &mut self,
        items: &[InlineItem],
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) {
        let Some(provider) = &self.font_metric_provider else { return };
        // https://drafts.csswg.org/css2/#s10.8.1
        // 只在本 IFC 已收集的 run 上查询一次；不扫描整个文档，也不改变系统字体策略。
        for item in items {
            let InlineItem::Text(run) = item else { continue };
            let style = styles.get(&run.node_id).or_else(||
                doc.parent_node(run.node_id).and_then(|id| styles.get(&id)));
            if let Some(metrics) = style.and_then(|s| provider.downloaded_line_metrics(&s.font_family, 1.0))
                && metrics.ascent.is_finite() && metrics.ascent > 0.0
            {
                self.ascent_ratio_overrides.insert(run.node_id, metrics.ascent);
            }
        }
    }

    /// R3778：ComputedStyle.white_space → run 级 white-space 三标志
    ///（preserve / break_at_newline / no_wrap）。与 inline_finalization 容器级映射同源
    ///（Pre=(T,T,T) 的 `\n` 断行由 preserve 模式的 split_into_words 承载，故
    /// break_at_newline 恒随 preserve 置位以简化 run 侧判定）。
    fn run_white_space(ws: &zero_style_system::WhiteSpaceValue) -> crate::inline::RunWhiteSpace {
        use zero_style_system::WhiteSpaceValue;
        match ws {
            WhiteSpaceValue::Pre => crate::inline::RunWhiteSpace { preserve: true, break_at_newline: true, no_wrap: true, hang_trailing: false },
            WhiteSpaceValue::PreWrap => crate::inline::RunWhiteSpace { preserve: true, break_at_newline: true, no_wrap: false, hang_trailing: true },
            WhiteSpaceValue::PreLine => crate::inline::RunWhiteSpace { preserve: false, break_at_newline: true, no_wrap: false, hang_trailing: false },
            WhiteSpaceValue::BreakSpaces => crate::inline::RunWhiteSpace { preserve: true, break_at_newline: true, no_wrap: false, hang_trailing: false },
            WhiteSpaceValue::Nowrap => crate::inline::RunWhiteSpace { preserve: false, break_at_newline: false, no_wrap: true, hang_trailing: false },
            _ => crate::inline::RunWhiteSpace::default(),
        }
    }

    fn parse_html_dimension_attr(value: Option<String>) -> f32 {
        // https://html.spec.whatwg.org/multipage/rendering.html#attributes-for-embedded-content-and-images
        value.and_then(|v| v.parse::<f32>().ok().filter(|n| n.is_finite()))
            .unwrap_or(0.0)
            .max(0.0)
    }

    /// R3997：inline `<svg>` 的 viewBox 固有宽高比（w/h；无效/缺失 → None）。
    fn svg_viewbox_ratio(elem: &zero_dom::ElementData) -> Option<f32> {
        let vb = elem.get_attribute("viewBox").or_else(|| elem.get_attribute("viewbox"))?;
        let nums: Vec<&str> = vb.split([' ', ',']).filter(|t| !t.is_empty()).collect();
        if nums.len() != 4 {
            return None;
        }
        let vw: f32 = nums[2].parse().ok()?;
        let vh: f32 = nums[3].parse().ok()?;
        (vh > 0.0 && vw.is_finite() && vh.is_finite() && vw > 0.0).then_some(vw / vh)
    }

    /// R3997：元素参与 IFC 排版的 CSS 宽高比（css-sizing-4 §3 aspect-ratio 值）。
    /// computed aspect_ratio 已剥 `auto` 前缀（converter 语义同 taffy 直传）；`auto <ratio>`
    /// 时 replaced 元素固有比优先，但 inline svg 无解码尺寸信号面，直接用显式 ratio
    ///（与 tree.rs eff_ratio 的 fallback 臂一致）。None = 无有效 ratio。
    fn css_aspect_ratio(style: &ComputedStyle) -> Option<f32> {
        style.aspect_ratio.filter(|r| r.is_finite() && *r > 0.0)
    }

    /// R4321：paint Path B（IFC 重跑，`styles` 空表）的 HTML UA 隐藏元素兜底——
    /// `ua_default_display` 静态 display:none 集在无 styles 时仍须跳过（script 源文本
    /// 泄漏防护，见 collect 主路径 Element 分支注释）。rt/rtc 与 option/optgroup 走
    /// `ua_default_display` 的 env 条件臂/抑制臂，不属于本兜底域（ruby/select 有专门
    /// 收集语义，Path B 行为保持不变）。
    fn ua_hidden_without_styles(local_name: &str) -> bool {
        !matches!(local_name, "rt" | "rtc" | "option" | "optgroup")
            && zero_style_system::ua_default_display(local_name)
                .is_some_and(|d| matches!(d, DisplayValue::None))
    }

    /// 收集容器中所有行内级内容（文本节点 + inline 元素 + `<br>` 元素），
    /// 从 ComputedStyle 中读取 font-size 和 line-height。
    fn collect_inline_items(
        &self,
        doc: &Document,
        container: NodeId,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> Vec<InlineItem> {
        let mut items = Vec::new();
        // R4405：Path B（paint 重建 IFC，空 styles）以 **ruby 元素自身**为容器时，按
        // R1022 的 ruby-as-child 同款语义整段扁平化（base 文本一条 run、rt/rp/rtc 注音
        // 由 collect_text_excluding 排除、注音绘制走 R1689 overlay）。旧实现遍历 ruby
        // 的子节点逐个收集——Path B 无 styles 无法识别 display:none（rt/rtc/rp 的 UA
        // display 仅 layout 趟可见），注音文本落成 fs=16 常规 run 绘在 base 基线上
        //（intra-base REF 页 vs TEST 页注音不对称、ruby-bidi rider 三案 Path B 接管后
        // 阈值骑线翻红实证）。layout 趟永不以 ruby 为 IFC 容器（ruby inline 参与**父级**
        // IFC；compute_final 的 ruby 排除使 ruby 子树无 stored IFC，R4392 注释），此臂
        // 仅 paint Path B 触发，layout 行为零变化。
        if doc.get(container).is_some_and(|n| {
            matches!(&n.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("ruby"))
        }) && let Some(item) = self.build_flatten_run_for_element(doc, container, styles)
        {
            items.push(item);
            return items;
        }
        // R4376：旗标每次 collect 读一次（非每 run——wide_tree 500 run 页 0.25% bench
        // 成本实证，hoist 后每容器一次）。
        let glyph_verticals_enabled = runtime_flags::fallback_line_metrics();
        // R109 §9.2.1.1：匿名块盒片段只收集该片段的 inline 内容（fragment_node_ids），
        // 而非 container 的全部 DOM 子节点。None = 正常遍历 container 子节点。
        // R3991：run-in 并入——先收集 run-in 元素的 inline 内容（其文本/子 inline
        // 按其样式渲染，作为本容器首行开头），再收集容器常规子节点。
        let mut children: Vec<NodeId> = Vec::new();
        if let Some(run_in_id) = self.run_in_prepended {
            for &gc in doc.child_nodes(run_in_id).iter() {
                // 仅 inline 级内容参与（run-in 的块级子按 spec 降级，罕见形态保守跳过
                // 块级子以保 IFC 纯度——WPT run-in 簇均为文本/inline 子形态）。
                let is_block_child = doc.get(gc).is_some_and(|n| {
                    matches!(&n.kind, NodeKind::Element(_))
                }) && styles.get(&gc).is_some_and(|s| {
                    matches!(
                        s.display,
                        DisplayValue::Block
                            | DisplayValue::FlowRoot
                            | DisplayValue::ListItem
                            | DisplayValue::Flex
                            | DisplayValue::Grid
                            | DisplayValue::Table
                    )
                });
                if !is_block_child {
                    children.push(gc);
                }
            }
        }
        children.extend(match &self.fragment_node_ids {
            Some(ids) => ids.clone(),
            None => doc.child_nodes(container),
        });
        let children = children;

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
                        // R3778：run 级有效 white-space 在**折叠前**判定——文本节点的
                        // 有效值 = 最近祖先声明（styles 键为元素，text 节点取父元素样式）。
                        // pre 族原始文本不折叠（`\n`/连续空格/制表符保留给 break_lines）；
                        // collapse 有损（`\n`→空格），事后无法恢复。
                        let parent_id = doc.parent_node(child_id);
                        let run_ws = doc
                            .parent_node(child_id)
                            .and_then(|pid| styles.get(&pid))
                            .map(|s| Self::run_white_space(&s.white_space))
                            .or_else(|| {
                                // R3778：paint Path B（空 styles）——layout 期存储的 run 级
                                // white-space 覆盖（按文本节点/其父元素 id 键）。
                                self.ws_overrides
                                    .get(&child_id)
                                    .copied()
                                    .or_else(|| parent_id.and_then(|pid| self.ws_overrides.get(&pid)).copied())
                            });
                        let run_preserves = run_ws.map_or(self.preserve_whitespace, |ws| ws.preserve);
                        let text = if run_preserves {
                            text_data.content.clone()
                        } else if run_ws.is_some_and(|ws| ws.break_at_newline) {
                            // R4398（css-text-3 §white-space-phase-1）：pre-line 的 `\n` 是
                            // 强制断行 identity——collapse 有损（\n→空格）使断行标记在
                            // collect 即丢失（pre-line-with-space-and-newline：run " \n"
                            // 折成 " "，marker 永不产出、两行并排一行 PNG 实证）。换行
                            // 保留式折叠；段内折叠由 break_lines 的 break_at_newline
                            // 分支按段完成（对已折叠文本幂等）。
                            crate::inline::collapse_whitespace_keep_newlines(&text_data.content)
                        } else {
                            collapse_whitespace(&text_data.content)
                        };
                        if !text.is_empty() {
                            // CSS Pseudo 4: generated-content text uses the computed style
                            // of its pseudo element. Normal DOM text nodes still inherit by
                            // looking at their parent element.
                            let parent_id = doc.parent_node(child_id);
                            let style = styles.get(&child_id).or_else(|| parent_id.and_then(|pid| styles.get(&pid)));
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
                            // R4374：回退链垂直度量（glyph_ascent/glyph_descent）——layout 趟按
                            // line-height 语义归一为最终行盒贡献；paint Path B（styles 空）经
                            // glyph_verticals_overrides 复用 layout 趟结果（不用再判 normal）。
                            let run_font_id = self.shaping_font_id_for_style(
                                Some(child_id),
                                style,
                                is_ahem_font,
                                letter_spacing,
                                word_spacing,
                                false,
                            );
                            // R4378：垂直度量按有序 CSS face 列表解析（多 @font-face +
                            // unicode-range 页面按 font-family 序落 face；Path B 经
                            // font_ids_overrides 复用 layout 序）。
                            let run_font_ids = self
                                .ordered_font_ids_for_style(style)
                                .or_else(|| self.font_ids_overrides.get(&child_id).cloned())
                                .unwrap_or_else(|| run_font_id.into_iter().collect());
                            let (glyph_ascent, glyph_descent) = match self.glyph_verticals_overrides.get(&child_id)
                            {
                                Some(&(ga, gd)) => (ga, gd),
                                None => Self::run_glyph_verticals(
                                    glyph_verticals_enabled,
                                    &run_font_ids,
                                    &text,
                                    font_size,
                                    is_ahem_font,
                                    line_height,
                                    style.is_some_and(|s| {
                                        matches!(s.line_height, zero_style_system::LineHeightValue::Normal)
                                    }),
                                ),
                            };
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
                                padding_left: 0.0,
                                padding_right: 0.0,
                                padding_top: 0.0,
                                padding_bottom: 0.0,
                                border_top: 0.0,
                                border_bottom: 0.0,
                                is_ahem_font,
                                font_id: run_font_id,
                                is_rtl: style.is_some_and(|s| {
                                    matches!(s.direction, zero_style_system::DirectionValue::Rtl)
                                }),
                                // R3840：paint Path B（空 styles）经 text_node_bidi_overrides
                                // 恢复元素级 bidi-override（layout 期按文本节点 id 存储）。
                                bidi_override: Self::element_bidi_override(style).or_else(|| {
                                    parent_id.and_then(|pid| self.text_node_bidi_overrides.get(&pid).copied())
                                }),
                                is_plaintext_bidi: style
                                    .map(|s| {
                                        matches!(s.unicode_bidi, zero_style_system::UnicodeBidiValue::Plaintext)
                                    })
                                    .unwrap_or_else(|| {
                                        self.plaintext_bidi_override
                                            || parent_id.is_some_and(|id| self.plaintext_bidi_overrides.contains(&id))
                                    }),
                                ws_override: run_ws,
                                ruby_rt_ascent: 0.0,
                                glyph_ascent,
                                glyph_descent,
                            }));
                        }
                    }
                    NodeKind::Element(elem_data) => {
                        // CSS2 §9.2.1 / CSS Display 3 §2.1：display:none 子树不生成任何盒——
                        // 其文本不得泄入父 IFC（head-metadata 渲染链 content-067/085/131：
                        // `head{display:block}` 匿名块内 <title>/<link> 等 display:none 元素
                        // 的文本/空盒曾作为父 IFC 文本 run 参与行盒测量，产生幻影行把
                        // body 整体推下 ~2 行）。必须先于 br/wbr 判定（display:none 的 br
                        // 同样不产生强制换行，CSS2 §14.1 命名实例 br{display:none}）。
                        // R4321：paint Path B（IFC 重跑，`styles` 空表——painter/text.rs
                        // `ctx.layout(doc, node_id, &HashMap::new())`）下上方判定恒 false，
                        // HTML UA 隐藏元素（script/style/head/title/...，静态 display:none
                        // 集）的源文本经 text_content 扁平化泄入父 IFC——WPT
                        // counter-style-at-rule/system-symbolic ref 页 div 内 `<script>` +
                        // 可见文本（".&nbsp;"）时脚本源码整行渲染实证。styles 空时按 UA 默认
                        // display 兜底；rt/rtc（ZW_RUBY_RT_NONE env 臂）与 option/optgroup
                        // （select 抑制臂）不纳入——ruby/select 有专门收集语义，Path B 行为
                        // 保持不变。
                        if styles
                            .get(&child_id)
                            .is_some_and(|s| matches!(s.display, DisplayValue::None))
                            || (styles.is_empty() && Self::ua_hidden_without_styles(elem_data.local_name()))
                        {
                            continue;
                        }

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
                        // R4205（CSS2 §17.2.1 匿名盒 + CSS Display 3 §2.4 blockification）：
                        // table-internal 盒（cell/row/row-group/header-group/footer-group/
                        // column/column-group/caption）在非 table 父（block 容器/inline 元素）
                        // 中须生成**块级**匿名 table 包装——同 LayoutBox 层 adjust_table_layout
                        // 的匿名包装语义。旧实现不在名单 → 走普通 inline 递归产出空 TextRun，
                        // preserve 模式下空 run 被 split_into_words_with_ws 兜底成幻影空格词
                        // （1 字宽推进）→ 后继文本右移（table-anonymous-objects-214 的
                        // X 字形偏移 1ch 根因）。发 BlockBreak 使其脱离 IFC、后继文本独立成行。
                        // R4339：paint Path B（styles 空）的块级判定——空表下
                        // `style.is_some_and` 恒 false，run-in 前缀模式容器的前缀流会把
                        // 块级子的 text_content 扁平拼入（run-in-basic-005：
                        // "Run-inheaderStart of block..." 连排）。布局期盒已分类：
                        // painter 注入 `block_child_nodes`（box_node.children 中
                        // is_block_level 子的 dom id 集），此处查集发 BlockBreak。
                        // UA 默认 display 兜底不可用——页样式可把 div 转 inline
                        //（run-in-contains-block-005 的 #r），空表无法区分。
                        let is_block_level = style
                            .map(|s| {
                                matches!(
                                    s.display,
                                    DisplayValue::Block
                                        | DisplayValue::Flex
                                        | DisplayValue::Grid
                                        | DisplayValue::Table
                                        | DisplayValue::ListItem
                                        | DisplayValue::FlowRoot
                                        | DisplayValue::TableCell
                                        | DisplayValue::TableRow
                                        | DisplayValue::TableRowGroup
                                        | DisplayValue::TableHeaderGroup
                                        | DisplayValue::TableFooterGroup
                                        | DisplayValue::TableColumn
                                        | DisplayValue::TableColumnGroup
                                        | DisplayValue::TableCaption
                                )
                            })
                            .unwrap_or_else(|| {
                                styles.is_empty()
                                    && self.block_child_nodes.as_ref().is_some_and(|ids| ids.contains(&child_id))
                            });
                        if is_block_level {
                            // R57（M3）：in-flow block 子 → BlockBreak（无 R1286 空行 strut——
                            // block 前被折叠的空白行不应获得 line-height，canvas-grid 22px 偏移
                            // 根因）。
                            // R3779b：**float 子同发 BlockBreak**——CSS2 §9.5 float 脱离常规流
                            // 不产生行盒，其后的行通过 float exclusion 缩宽（effective_content_area），
                            // 无需断行条目占位。旧发 Br + R1286 strut 给行首空行赋 20px 高 →
                            // line-clamp 计数含幽灵行（line-clamp-with-floats-001：cap=4 裁掉
                            // 4 行真文本只留 3 行；floats-002 ref 同塌）；floats-zero-height-wrap /
                            // floats-wrap-top-below-bfc-001l 簇同享此修复。kill-switch
                            // `ZW_FLOAT_NO_GHOST_LINE=0` 回退旧行为。
                            if style.is_some_and(crate::inline_block_split::is_out_of_flow)
                                && !runtime_flags::float_no_ghost_line()
                            {
                                items.push(InlineItem::Br);
                            } else if style.is_some_and(|s| !matches!(s.float, zero_css_parser::values::FloatValue::None)) {
                                // R3784：float 子 → FloatAnchor(id)——断行语义同 BlockBreak，
                                // 额外记录行内流锚 y（remeasure 据此把 float 从 taffy 堆叠位
                                // 搬到源序行位）。
                                items.push(InlineItem::FloatAnchor(child_id));
                            } else {
                                items.push(InlineItem::BlockBreak);
                            }
                            continue;
                        }

                        // 检查该元素是否为原子行内级盒（inline-block / inline-flex / inline-grid / inline-table）。
                        // 这些元素参与行内格式化上下文，作为不可拆分的原子盒。
                        // R3987（CSS Display 3 §2.4 / CSS2 §9.2.1.1）：replaced 类元素的
                        // display:inline 是 **atomic inline**——内部结构（svg 子元素等）不参与
                        // 父 IFC，CSS width/height 应用不依赖行内容存在。旧实现 svg 走普通
                        // inline 递归 → 子树内容为空时 IFC 收集 0 项 → width 不应用（盒塌
                        // 6×24，r3986 两态锚实证）。img 已有独立原子分支；此处把其余 replaced
                        // 类（svg 为 driving，canvas 等已有 attr 回退名单不冲突）并入。
                        let is_replaced_inline = style.is_some_and(|s| {
                            matches!(s.display, DisplayValue::Inline)
                                && matches!(
                                    elem_data.local_name(),
                                    "svg" | "canvas" | "video" | "iframe" | "embed" | "object"
                                        | "applet"
                                )
                        });
                        let stored_inline_size = self.inline_block_sizes.get(&child_id).copied();
                        let is_inline_block = is_replaced_inline
                            || stored_inline_size.is_some()
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
                                    "svg" | "canvas" | "video" | "iframe" | "embed" | "object"
                                        | "applet"
                                ) {
                                    if w <= 0.0 {
                                        w = Self::parse_html_dimension_attr(elem_data.get_attribute("width"));
                                    }
                                    if h <= 0.0 {
                                        h = Self::parse_html_dimension_attr(elem_data.get_attribute("height"));
                                    }
                                }
                            }
                            // R4000（css-sizing-3 §intrinsic-sizes + csswg #1801581）：
                            // inline `<svg>` 的 IFC 原子盒 used size——CSS/attr 均无 abs 值时
                            // 按 default object size 规则补齐（viewBox/ar-only → 0×0；无来源
                            // → 300×150；width % → 宽交容器解析、高 150）。taffy 层已设
                            // definite 的场景（tree.rs svg gate）经 stored_inline_size 优先生效，
                            // 此处兜 IFC 直接收集（taffy 节点被跳过）的路径。
                            // kill-switch `ZW_SVG_DEFAULT_SIZE=0`。
                            if std::env::var("ZW_SVG_DEFAULT_SIZE").as_deref() != Ok("0")
                                && elem_data.local_name() == "svg"
                                && (w <= 0.0 || h <= 0.0)
                                && let Some(style) = style
                                && let Some((dw, dh)) = {
                                    let shape_children: Vec<(&str, &zero_dom::ElementData)> = doc
                                        .child_nodes(child_id)
                                        .iter()
                                        .filter_map(|child| doc.get(*child))
                                        .filter_map(|n| match &n.kind {
                                            zero_dom::NodeKind::Element(e) => Some((e.local_name(), e)),
                                            _ => None,
                                        })
                                        .collect();
                                    let bbox = crate::svg_default_size::svg_content_bbox(&shape_children);
                                    crate::svg_default_size::svg_default_used_size(elem_data, style, bbox)
                                }
                            {
                                if w <= 0.0 {
                                    // width %：按容器宽解析（taffy % 语义；IFC 直收集路径
                                    // 无 CB 解析——img 分支 Percentage 同款）。
                                    w = match (&style.width, elem_data.get_attribute("width")) {
                                        (LengthValue::Percentage(p), _)
                                            if self.container_width > 0.0 =>
                                        {
                                            (*p as f32 / 100.0) * self.container_width
                                        }
                                        (_, Some(attr)) if attr.trim().ends_with('%')
                                            && self.container_width > 0.0 =>
                                        {
                                            attr.trim()
                                                .trim_end_matches('%')
                                                .parse::<f32>()
                                                .map(|p| p / 100.0 * self.container_width)
                                                .unwrap_or(0.0)
                                        }
                                        // 隐式 width:100%（ratio-only，(a) 路径）：
                                        // dw None 且负 dh = 比信号 → 宽 = 容器宽
                                        //（SVG 根缺省 100% 语义；definite 块容器
                                        // 下 fills，max-content 语境走 contribution 0）。
                                        _ if dh < 0.0 && self.container_width > 0.0 => {
                                            self.container_width
                                        }
                                        _ => dw.unwrap_or(0.0),
                                    };
                                }
                                if h <= 0.0 {
                                    // 负 dh = 比信号：h = 解析宽 / |ratio|（svg_default_size
                                    // 模块注释——% 宽时比随解析宽生效）。
                                    h = if dh < 0.0 {
                                        let ratio = -dh;
                                        if ratio > 0.0 && w > 0.0 { w / ratio } else { 0.0 }
                                    } else {
                                        dh
                                    };
                                }
                                // R4007（css-sizing-3 §5.2.1 stretch-fit min/max constraint +
                                // 比回传）：h 钳 min-height 后按比回传扩宽（w = min(h×ratio,
                                // max_width)），替换首次 w。001：50→25→min-h 100→回传 200→max-w
                                // 钳 100；002：50→50→100→100。仅比信号（dh<0）+ Px 有限值触发。
                                if dh < 0.0
                                    && h > 0.0
                                    && let LengthValue::Px(min_h) = style.min_height
                                    && min_h.is_finite()
                                    && min_h > 0.0
                                    && (min_h as f32) > h
                                {
                                    let ratio = -dh;
                                    let transferred = (min_h as f32) * ratio;
                                    let max_w = match style.max_width {
                                        LengthValue::Px(mw) if mw.is_finite() && mw > 0.0 => mw as f32,
                                        _ => f32::INFINITY,
                                    };
                                    h = min_h as f32;
                                    w = transferred.min(max_w).max(w);
                                }
                            }
                            // R3997（css-sizing-4 §4.1/§4.2 transferred size）：CSS aspect-ratio
                            // （或 `auto <ratio>` 的 ratio 部分）+ 恰一侧显式、另一侧 auto 时，
                            // auto 侧由显式侧 ×/÷ ratio 推导（img 分支 R1578 固有比推导的 CSS
                            // ratio 泛化）。driving: css-sizing replaced-element-007/008/015/016
                            //（inline `<svg>` width:100px + aspect-ratio:1/1 → 100×100，旧塌 6×24）。
                            // 关 kill-switch `ZW_IFC_AR_TRANSFER=0`。eff_ratio 语义与 tree.rs 一致：
                            // `auto <ratio>` 时 replaced 元素固有比优先（此处 img_intrinsic 缺失
                            // 时回落显式 ratio——svg inline 无解码尺寸信号面）。
                            if std::env::var("ZW_IFC_AR_TRANSFER").as_deref() != Ok("0")
                                && !self.vertical
                                && (w > 0.0) != (h > 0.0)
                                && let Some(s) = style
                                // R2440 语义（css-sizing-4 §aspect-ratio）：`auto <ratio>` 时
                                // replaced 元素**固有比**优先（显式 ratio 仅 fallback）。inline
                                // svg 的固有比直接从 viewBox attr 解析（无需解码信号面——
                                // SVG2 viewport 建立语义，viewBox w/h 比 = 固有宽高比）。
                                && let Some(ratio) = {
                                    let explicit = Self::css_aspect_ratio(s);
                                    if s.aspect_ratio_auto && elem_data.local_name() == "svg" {
                                        Self::svg_viewbox_ratio(elem_data).or(explicit)
                                    } else {
                                        explicit
                                    }
                                }
                                && ratio > 0.0
                            {
                                if w > 0.0 && h <= 0.0 {
                                    h = (w / ratio).max(0.5);
                                } else if h > 0.0 && w <= 0.0 {
                                    w = (h * ratio).max(0.5);
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
                            let mut w = Self::parse_html_dimension_attr(elem_data.get_attribute("width"));
                            let mut h = Self::parse_html_dimension_attr(elem_data.get_attribute("height"));
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
                            // R3806：两侧均未知（attrs 缺 + CSS auto，如 ::before content:url()
                            // 注入的 <img>）→ 用解码固有尺寸双向补齐，与 final path
                            // apply_replaced_element_sizing 的 both-auto 臂（tree.rs 同名逻辑）
                            // 一致。旧实现直接跳过收集 → img 降级零宽 TextRun、content:url()
                            // 伪元素图片整体不渲染（content-004 族 driving）。
                            if w <= 0.0
                                && h <= 0.0
                                && let Some(&(iw, ih)) = self.img_intrinsic_sizes.get(&child_id)
                                && iw > 0.0
                                && ih > 0.0
                            {
                                w = iw;
                                h = ih;
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
                            // R3997：fragment_node_ids 是 **split 容器作用域**的片段成员表
                            //（R109 §9.2.1.1 ②匿名块），递归进 inline 子元素后必须清除——
                            // 否则子元素把自己的成员表当容器子列表（子元素 ∈ 成员表 →
                            // collect_inline_items 以自身为容器无限递归栈溢出，
                            // css-sizing replaced-element-012 `<picture>` 实证）。
                            let nested = {
                                let mut nested_ctx = self.clone();
                                nested_ctx.fragment_node_ids = None;
                                nested_ctx.collect_inline_items(doc, child_id, styles)
                            };
                            items.extend(nested);
                            continue;
                        }

                        // R4300（CSS2.1 §8.4 + §9.4.2）：含**元素子**的 inline 元素扁平化
                        // 保持子序列——旧 `text_content` 扁平化把整棵子树折成单 run，
                        // **子元素被丢弃**：childless inline 子（spacer span）的零宽 run 不入
                        // IFC，其水平 margin/padding 不推进（word-spacing-characters-001
                        // control 条 `A <spacer>B` 短 64px、r4134 outer 并集宽丢 spacer 贡献
                        // 实证）。walk 规则：文本子 → run（node_id = **本元素 id**，延续既有
                        // 归因契约：R4297 sync / R2197 orphan / R638 inline_heights 均按元素
                        // id 查）；childless inline 子 → 零宽 run（node_id = 该子 id，样式取
                        // 自该子，其 ml/pl/mr/pr 经 break_lines 空元素分支推进）；有元素子的
                        // inline 子 → 递归同规则。本元素水平 frame（ml/pl/mr/pr）只落在
                        // 首/末 run（break_lines 逐 run 推进，多 run 会重复推进）。
                        // 纯文本子（无元素子）走下方既有单 run 路径（字节不变）。
                        // kill-switch `ZW_FLAT_CHILD_WALK=0`。ruby 的 rt/rp 特例
                        //（R1022）不走 walk（含元素子时仍走旧扁平化）。
                        // **default-on（R4312，2026-09-13）**：R4300 探针挂账的逐域
                        // gate 已收敛——quotes 配序（R4308 font 度量三级回退）、
                        // ruby 递归（R4308 特例门）、bidi 控制字符（R4300c/R4303）、
                        // 竖排 writing-mode（R4310 per-node 信号通道 + 门）、SVG 行盒
                        // 贡献（R4311 特例门）逐域清偿后，walk-on 全量 corpus 14765
                        // 反超 walk-off baseline 14763（净 +2：5 翻绿/3 翻红，余
                        // font-size-121/R109-split/intrinsic×IB 三深域挂账）。
                        // `ZW_FLAT_CHILD_WALK=0` 回退旧扁平化。
                        static FLAT_CHILD_WALK: std::sync::LazyLock<bool> = std::sync::LazyLock::new(|| {
                            std::env::var("ZW_FLAT_CHILD_WALK").as_deref() != Ok("0")
                        });
                        // R4300b：bidi 特殊元素（rtl / unicode-bidi ≠ normal）不走 walk——
                        // 双向重排按 run 序列切分视觉段，拆 run 会改变重排段组成
                        // （line-breaking-bidi-003 0.91→7.67 实证），保持整段扁平化。
                        let bidi_special = style
                            .is_some_and(|st| {
                                matches!(st.direction, zero_style_system::DirectionValue::Rtl)
                                    || !matches!(
                                        st.unicode_bidi,
                                        zero_style_system::UnicodeBidiValue::Normal
                                    )
                            });
                        // R4300c：扁平化文本含 bidi 控制字符（U+202A-U+202E）不走 walk——
                        // 控制字符的重排作用域横跨整个 run，拆 run 改变重排段组成
                        //（line-breaking-bidi-003 0.91→7.67 实证），保持整段扁平化。
                        let has_bidi_controls = style
                            .and_then(|_| doc.text_content(child_id))
                            .is_some_and(|t| {
                                t.chars().any(|c| ('\u{202A}'..='\u{202E}').contains(&c))
                            });
                        // R4310：自身声明竖排 writing-mode（vertical-rl/lr）的子不走 walk
                        // ——竖排子内容有自身盒几何/列偏移，扁平化进正交轴 IFC 会丢失
                        // 原点（ruby-overhang-spaces-vertical-004/006 实证：walk-off 落
                        // span 盒原点 x=51.1、walk-on 落容器游标 x=0.0 全体左移）。有
                        // styles（layout IFC）直判；paint Path B（空 styles）读存储信号
                        //（见 child_declares_vertical_wm）。
                        let child_declares_vertical_wm = style
                            .map(|st| {
                                matches!(
                                    st.writing_mode,
                                    zero_style_system::WritingModeValue::VerticalRl
                                        | zero_style_system::WritingModeValue::VerticalLr
                                )
                            })
                            .unwrap_or_else(|| self.vertical_walk_nodes.contains(&child_id));
                        // R4312：含块级元素子的 inline 不走 walk——块子经 R109
                        // block-in-inline 机制处理，walk 展开会改变盒树/intrinsic 测量
                        //（td>span>div{width:500} cell 被过测到 500px，
                        // r1153_table_cell_inline_child_not_over_measured 实证）。判定
                        // 通道同竖排门：layout 有 styles 直判，paint Path B 读存储信号。
                        let child_has_block_element_child = Self::has_block_level_child(
                            doc,
                            styles,
                            child_id,
                            &self.block_child_walk_nodes,
                        );
                        let has_element_children = *FLAT_CHILD_WALK
                            && elem_data.local_name() != "ruby"
                            // R4311：SVG 子树不走 walk——flatten 路径对 svg 落零宽 run
                            //（行盒高参与），walk 递归对无文本 SVG 内部产出零 item，
                            // svg 的行盒贡献丢失 → 首行高变化 → img 等 sibling 位移
                            //（effect-reference-after-001 1.23% 实证，页面无文本节点）。
                            && elem_data.local_name() != "svg"
                            && !bidi_special
                            && !has_bidi_controls
                            && !child_declares_vertical_wm
                            && !child_has_block_element_child
                            && !self.vertical
                            && doc.child_nodes(child_id).iter().any(|&gc| {
                                doc.get(gc).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_)))
                            });
                        if has_element_children {
                            let mut walked = Vec::new();
                            self.collect_flat_inline_children(doc, child_id, styles, &mut walked);
                            items.extend(walked);
                            continue;
                        }
                        // 其他 inline 元素的文本内容也收集进来
                        // R1022：<ruby> 默认 text_content 会扁平化 <rt>/<rp> 文本
                        // （● 当行内字符渲染）。改为只收集 rb 文本作 inline 流，
                        // rt 文本由 paint 期作 zero-width annotation 上移到 rb 之上。
                        // R4300：构造逻辑提取至 `build_flatten_run_for_element`（walk 路径
                        // 复用同一构造，保证两条路径 run 形状一致）。
                        if let Some(item) = self.build_flatten_run_for_element(doc, child_id, styles) {
                            items.push(item);
                        }
                    }
                    _ => {}
                }
            }
        }

        // R4330：run-in 分裂边框后处理——载荷（IFC 字段）折入前缀首/末 TextRun 的
        // 水平 margin（margin 已参与首/末 run 推进与对齐尾盒，免改 break_lines），
        // paint 侧按载荷对首/末片段绘竖条/横条于 margin 空间（无文字重叠）。
        if let (Some(run_in_id), Some(b)) = (self.run_in_prepended, &self.run_in_border) {
            let in_prefix = |it: &InlineItem| match it {
                InlineItem::Text(r) => doc.parent_node(r.node_id) == Some(run_in_id),
                InlineItem::Br => true,
                _ => false,
            };
            let mut prefix_end = 0usize;
            for (i, it) in items.iter().enumerate() {
                if in_prefix(it) {
                    prefix_end = i + 1;
                } else {
                    break;
                }
            }
            let first_text = items[..prefix_end].iter().position(|it| matches!(it, InlineItem::Text(_)));
            let last_text = items[..prefix_end].iter().rposition(|it| matches!(it, InlineItem::Text(_)));
            if let (Some(fi), Some(li)) = (first_text, last_text) {
                for (i, it) in items.iter_mut().enumerate() {
                    if i >= prefix_end {
                        break;
                    }
                    let InlineItem::Text(r) = it else { continue };
                    if i == fi {
                        r.margin_left += b.left;
                    }
                    if i == li {
                        r.margin_right += b.right;
                    }
                }
            }
        }
        items
    }

    /// R4300：inline 元素扁平化 run 构造（主 collect 路径与 `collect_flat_inline_children`
    /// R4357（css-ruby-1 §ruby-overhang；chromium probe 模型 15 数据点全拟合）：ruby 注音
    /// 行宽参与——rt 注释宽（0.5em 字号）计入行盒，可悬挂部分吸收到邻字符空白半侧：
    /// `box_w = max(base_w, rt_w − hang_l − hang_r)`，extra = box_w − base_w 对折进 run
    /// 左右 margin（base 字形居中于盒，与 ruby-align:center 一致；rt paint 层 R1689
    /// overlay 本就按 base 段居中，无需改动）。hang = ruby 紧邻字符朝向 ruby 的空白半侧：
    /// 收窄标点）】〉》」』等（字形居左、右半空白）作左邻 → 0.5em；开括标点（【〈「等
    /// （字形居右、左半空白）作右邻 → 0.5em；全角空白 U+3000 → 1em；其余 0。
    /// 水平书写模式限定（vertical 系独立域）；kill-switch `ZW_RUBY_OVERHANG_MODEL=0`。
    /// walk 共用）。text_content（ruby 按 R1022 排除 rt/rp）折叠后：
    /// 非空 → 文本 run（node_id = 元素自身，归因契约：R4297 sync / R2197 orphan /
    /// R638 inline_heights 按元素 id 查）；空 → 零宽 run（line-height + padding + border
    /// 仍贡献行盒高，CSS2.1 §10.8）。返回 None 仅当... 不发生（两分支都 push）。
    fn build_flatten_run_for_element(
        &self,
        doc: &Document,
        child_id: NodeId,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> Option<InlineItem> {
        let NodeKind::Element(elem_data) = &doc.get(child_id)?.kind else {
            return None;
        };
        // R4376：旗标每次 collect 读一次（同 collect_inline_items hoist 口径）。
        let glyph_verticals_enabled = runtime_flags::fallback_line_metrics();
        let style = styles.get(&child_id);
        let run_ws = style
            .map(|s| Self::run_white_space(&s.white_space))
            .or_else(|| self.ws_overrides.get(&child_id).copied());
        let run_preserves = run_ws.map_or(self.preserve_whitespace, |ws| ws.preserve);
        // R4402：注音排除泛化到任意元素的扁平化收集——rt/rp/rtc 是 display:none 域的
        // 注音文本（R4395：rtc 为 span-all 注音容器），不参与任何行宽。旧实现只对
        // local_name=="ruby" 走排除表，其余元素（rbc/rb 基容器、包裹 span）走
        // doc.text_content 把 rt 注音并进 run 文本（improperly-contained-annotation：
        // rbc flatten run="BA" 宽 55.6 vs ruby 26.7，6.5 remeasure 母子宽倒挂实证）。
        let text = Self::collect_text_excluding(doc, child_id, &["rt", "rp", "rtc"]);
        let trimmed = if run_preserves {
            text
        } else if run_ws.is_some_and(|ws| ws.break_at_newline) {
            // R4398：pre-line `\n` 断行 identity 保留（同 walk flush 站注）。
            crate::inline::collapse_whitespace_keep_newlines(&text)
        } else {
            collapse_whitespace(&text)
        };
        // R4382（CSS Text 3 §3.1）：扁平化 run 的 text-transform——layout 趟读元素自身
        // computed（含继承）；paint Path B（空 styles）经 text_transform_overrides
        //（inline_metric_storage 对元素归因 run 按元素 id 存储）。driving:
        // text-transform-bicameral-003/004/007（`.test span { text-transform }` 全簇
        // 不生效——span 文本走 flatten run，此前从未应用 transform）。
        let text_transform = style
            .map(|s| s.text_transform)
            .unwrap_or_else(|| self.text_transform_overrides.get(&child_id).copied().unwrap_or(TextTransformValue::None));
        let trimmed = text_transform.apply(&trimmed);
        let (font_size, line_height) = if style.is_some() {
            // U1b：layout IFC（有真实 styles）首消费 font_metric_provider
            // （per-font line-height）。provider 缺省时等价于 resolve_font_metrics。
            resolve_font_metrics_with_provider(style, self.font_metric_provider.as_ref())
        } else if let Some(&(fs, lh)) = self.inline_element_metrics.get(&child_id) {
            // paint IFC（空 styles）：使用 layout IFC 存储的 (font_size, line_height)
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
        // R4332：水平边框占行内空间（单 run 既首且末，双边框都折入 margin）。
        // bidi 控制符臂同 walk 位（见彼处注释）；零边框短路同 walk 位。
        let adv0 = Self::inline_horizontal_border_advance(style);
        let (border_adv_l, border_adv_r) = if adv0 == (0.0, 0.0) {
            adv0
        } else if Self::inline_horizontal_border_advance_has_bidi_controls(
            doc.text_content(child_id).unwrap_or_default().as_str(),
        ) {
            (0.0, 0.0)
        } else {
            adv0
        };
        let margin_left = style
            .map(|s| Self::resolve_inline_margin(&s.margin_left, s))
            .unwrap_or_else(|| self.margin_overrides.get(&child_id).map(|(ml, _)| *ml).unwrap_or(0.0))
            + border_adv_l;
        let margin_right = style
            .map(|s| Self::resolve_inline_margin(&s.margin_right, s))
            .unwrap_or_else(|| self.margin_overrides.get(&child_id).map(|(_, mr)| *mr).unwrap_or(0.0))
            + border_adv_r;
        // R4357（css-ruby-1 §ruby-overhang）：rt 注释行宽参与——extra 对折进 ruby run
        // 左右 margin（模型见 ruby_overhang_pads）。水平书写限定。
        // **仅 layout 趟（styles 在场）计算**：pads 经 inline_element_margins 存储后由
        // override-replay 趟（styles 空的 with_margin_overrides / paint Path B）原样回放，
        // 此处再加即双计（layout 20 → replay 40 实证——R4357 卡点根因）。
        let (margin_left, margin_right) = if elem_data.local_name() == "ruby"
            && style.is_some()
            && !style.is_some_and(|s| {
                matches!(
                    s.writing_mode,
                    zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
                )
            }) {
            let annot = ruby_annotation_width_text(doc, child_id);
            // R4415：base_w 契约 = **完整折叠 base 宽**（collapsed 文本含前后边缘空格
            // advance；变量名 trimmed 系历史遗留）——其 advance 落在 run region 内，
            // pads 分母必须含之，否则 region = 空格 + base + pads 超出 rt_w
            //（overhang-spaces-015 ref 页形态：region 65 vs test 60，双页右边框错位
            // 5px）。intrinsic walk 侧同口径（R4415 ruby 臂边缘空格 flush 双前置）。
            let base_w: f32 = trimmed
                .chars()
                .map(|c| {
                    crate::inline::estimate_char_width(
                        c,
                        font_size,
                        style.is_some_and(|s| {
                            s.font_family.iter().any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"))
                        }),
                    )
                })
                .sum::<f32>()
                + letter_spacing * trimmed.chars().count() as f32;
            let (pl, pr) = ruby_overhang_pads(
                doc,
                child_id,
                base_w,
                &annot,
                font_size,
                letter_spacing,
                style.is_some_and(|s| {
                    s.font_family.iter().any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"))
                }),
                // R4416：preserve 语境容量走原始空白串（tab 名义 unit 与行内 stop 推进
                // 同量级——tab-size 长度形 max(px, space)、数字形 n×space，与
                // break_lines tab_unit 公式同源）；折叠/回放趟 None = collapse 语义。
                // replay 趟（style 空）经 margin_overrides 原样回放，不重算。
                run_preserves.then(|| {
                    let space = crate::inline::estimate_char_width(' ', font_size, false);
                    let unit = if self.tab_size_is_length {
                        self.tab_size.max(space)
                    } else {
                        self.tab_size.max(1.0) * space
                    };
                    (unit, space)
                }),
            );
            (margin_left + pl, margin_right + pr)
        } else {
            (margin_left, margin_right)
        };
        let padding_left = style
            .map(|s| Self::resolve_inline_padding(&s.padding_left, s))
            .unwrap_or_else(|| self.padding_overrides.get(&child_id).map(|(pl, _)| *pl).unwrap_or(0.0));
        let padding_right = style
            .map(|s| Self::resolve_inline_padding(&s.padding_right, s))
            .unwrap_or_else(|| self.padding_overrides.get(&child_id).map(|(pr, _)| *pr).unwrap_or(0.0));
        let is_ahem_font = style
            .map(|s| s.font_family.iter().any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem")))
            .unwrap_or_else(|| self.is_ahem_overrides.get(&child_id).copied().unwrap_or(false));
        let (padding_top, padding_bottom, border_top, border_bottom) = Self::extract_inline_box_metrics(style);
        // R4359（css-ruby-1）：ruby 注音行高——rt（0.5em）的行盒高度参与所在行的
        // ascent/height（chromium 行距 35 = base 23.3 + rt 11.7 实证）。与 pads 同门
        // （ZW_RUBY_OVERHANG_MODEL=1，layout 趟计算、replay 趟经 margin_overrides 同款
        // 存储回放需另行接线——rt ascent 走 TextRun/TextFragment 直传，无需 override）。
        // R4412：**符号语义**——正值 = rt 在 base 上方（over，行 ascent 叠加）；负值 =
        // rt 在 base 下方（under，行 descent 侧扩展，消费点 mod.rs 行装配 + break_lines
        // 高度取 abs）。under 判据读 ruby 自身 computed ruby_position（继承）；Path B
        // （styles 空）读不到声明按 over 近似——under 页 paint 线内注音位为后续 slice。
        let ruby_rt_ascent = if elem_data.local_name() == "ruby"
            && std::env::var("ZW_RUBY_OVERHANG_MODEL").as_deref() != Ok("0")
            && !ruby_annotation_width_text(doc, child_id).is_empty()
        {
            // 注意：此处**不得** gate 在 style.is_some()——override-replay 趟（styles 空）
            // 须同样产出，否则 paint IFC 的行基线缺 rt ascent（行高 34.9 但基线停在 16，
            // base/rt 整体上浮 ~14px，R4361 实证）。rt ascent 是 doc+env 的纯函数（与
            // margin pads 不同），跨趟幂等，无双计风险。
            let h = font_size * 0.5 * crate::inline::text_metrics::NORMAL_LINE_HEIGHT_RATIO;
            match style.map(|st| st.ruby_position) {
                Some(zero_css_parser::values::RubyPositionValue::Under) => -h,
                _ => h,
            }
        } else {
            0.0
        };
        if !trimmed.is_empty() {
            // R4374：回退链垂直度量（同文本节点路径——layout 趟按 line-height 语义归一，
            // Path B 经 overrides 复用；门禁关闭或未注册回调时恒 0）。
            let run_font_id = self.shaping_font_id_for_style(
                Some(child_id),
                style,
                is_ahem_font,
                letter_spacing,
                word_spacing,
                // R4389：ruby run 不再豁免 shaping font_id——旧 dodge（R3213 shaping 时代）
                // 使 ruby run advance 落 estimate 0.55em，与 intrinsic walk 的真实 serif
                // hmtx 分裂（ruby-intrinsic-isize-001：render ABC=26.4 vs intrinsic 32.9，
                // chromium serif ≈ 32.9 = intrinsic 一侧）。generic id 走 hmtx 批量
                // （ZRG 修复 A 语义），旧 shaping 风险不适用。
                false,
            );
            let run_font_ids = self
                .ordered_font_ids_for_style(style)
                .or_else(|| self.font_ids_overrides.get(&child_id).cloned())
                .unwrap_or_else(|| run_font_id.into_iter().collect());
            let (glyph_ascent, glyph_descent) = match self.glyph_verticals_overrides.get(&child_id) {
                Some(&(ga, gd)) => (ga, gd),
                None => Self::run_glyph_verticals(
                    glyph_verticals_enabled,
                    &run_font_ids,
                    &trimmed,
                    font_size,
                    is_ahem_font,
                    line_height,
                    style.is_some_and(|s| matches!(s.line_height, zero_style_system::LineHeightValue::Normal)),
                ),
            };
            Some(InlineItem::Text(TextRun {
                ws_override: run_ws,
                ruby_rt_ascent,
                text: trimmed,
                node_id: child_id,
                font_size,
                line_height,
                vertical_align,
                letter_spacing,
                word_spacing,
                margin_left,
                margin_right,
                padding_left,
                padding_right,
                padding_top,
                padding_bottom,
                border_top,
                border_bottom,
                is_ahem_font,
                font_id: run_font_id,
                is_rtl: style.is_some_and(|s| {
                    matches!(s.direction, zero_style_system::DirectionValue::Rtl)
                }),
                bidi_override: Self::element_bidi_override(style),
                is_plaintext_bidi: style
                    .map(|s| {
                        matches!(s.unicode_bidi, zero_style_system::UnicodeBidiValue::Plaintext)
                    })
                    .unwrap_or_else(|| {
                        self.plaintext_bidi_override || self.plaintext_bidi_overrides.contains(&child_id)
                    }),
                glyph_ascent,
                glyph_descent,
            }))
        } else {
            // CSS 规范：空 inline 元素仍需通过 line-height + padding + border 影响行盒高度
            Some(InlineItem::Text(TextRun {
                ws_override: style.map(|s| Self::run_white_space(&s.white_space)),
                ruby_rt_ascent: 0.0,
                glyph_ascent: 0.0,
                glyph_descent: 0.0,
                text: String::new(),
                node_id: child_id,
                font_size,
                line_height,
                vertical_align,
                letter_spacing: 0.0,
                word_spacing: 0.0,
                margin_left,
                margin_right,
                padding_left,
                padding_right,
                padding_top,
                padding_bottom,
                border_top,
                border_bottom,
                is_ahem_font,
                font_id: None,
                is_rtl: style.is_some_and(|s| {
                    matches!(s.direction, zero_style_system::DirectionValue::Rtl)
                }),
                bidi_override: Self::element_bidi_override(style),
                is_plaintext_bidi: style
                    .map(|s| {
                        matches!(s.unicode_bidi, zero_style_system::UnicodeBidiValue::Plaintext)
                    })
                    .unwrap_or_else(|| {
                        self.plaintext_bidi_override || self.plaintext_bidi_overrides.contains(&child_id)
                    }),
            }))
        }
    }

    /// R4300：含元素子的 inline 元素扁平化 walk——保持**子序列**收集（详见 flatten
    /// 分支处的 gate 注释）。文本子归因到**外层元素 id**（既有归因契约不变）；零宽子
    /// 归因到该子自身 id。外层水平 frame（ml/pl/mr/pr）只落首/末 run（break_lines
    /// 逐 run 推进）；子级 frame 由其自身 run 携带。
    fn collect_flat_inline_children(
        &self,
        doc: &Document,
        elem_id: NodeId,
        styles: &HashMap<NodeId, ComputedStyle>,
        items: &mut Vec<InlineItem>,
    ) {
        let style = styles.get(&elem_id);
        let run_ws = style
            .map(|s| Self::run_white_space(&s.white_space))
            .or_else(|| self.ws_overrides.get(&elem_id).copied());
        let run_preserves = run_ws.map_or(self.preserve_whitespace, |ws| ws.preserve);
        // R4308（quotes-001 walk-on 9324px paint 残差根因）：font 度量须走与
        // `build_flatten_run_for_element` 同款三级回退——paint IFC（空 styles）下
        // `resolve_font_metrics_with_provider(None)` 落 16px 默认，而 layout IFC 走
        // 真实 styles（32px）：同一段落 layout 32px / paint 16px，paint 侧行断与
        // 行盒 y 全体错位（此前误归因为「字形发射坐标系混用」）。有 styles 首消费
        // font_metric_provider（per-font line-height），paint IFC 回落 layout 存储的
        // `inline_element_metrics`，最后才落默认值。
        let (font_size, line_height) = if style.is_some() {
            resolve_font_metrics_with_provider(style, self.font_metric_provider.as_ref())
        } else if let Some(&(fs, lh)) = self.inline_element_metrics.get(&elem_id) {
            (fs, lh)
        } else {
            self.default_font_metrics
                .unwrap_or((DEFAULT_FONT_SIZE, DEFAULT_FONT_SIZE * NORMAL_LINE_HEIGHT_RATIO))
        };
        let letter_spacing = style
            .map(|s| Self::resolve_letter_spacing(&s.letter_spacing, font_size))
            .unwrap_or_else(|| self.letter_spacing_overrides.get(&elem_id).copied().unwrap_or(0.0));
        let word_spacing = style
            .map(|s| Self::resolve_word_spacing(&s.word_spacing, font_size))
            .unwrap_or_else(|| self.word_spacing_overrides.get(&elem_id).copied().unwrap_or(0.0));
        let is_ahem_font = style
            .map(|s| s.font_family.iter().any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem")))
            .unwrap_or_else(|| self.is_ahem_overrides.get(&elem_id).copied().unwrap_or(false));
        let (padding_top, padding_bottom, border_top, border_bottom) = Self::extract_inline_box_metrics(style);
        // R4332：水平边框占行内空间——首 run 左边框、末 run 右边框折入 margin
        //（margin 参与 break_lines 推进；paint 侧 R1442 竖边锚 frag_x - pad - bl
        //  在含推进的 frag.x 下仍正确：边框区 = [x-pad-bl, x-pad]）。
        // 子树含 bidi 控制符时不折入——css-writing-modes §bidi-box-model 的逻辑侧
        // 映射（拆分片段物理左右边随重排翻转）未实现，物理折入破坏既有一致形态
        //（bidi-003/005/006 RLO/PDF 族）。零边框（绝大多数 inline）短路不扫文本
        //（text_content O(子树)，避免无谓扫描）。
        let adv0 = Self::inline_horizontal_border_advance(style);
        let (border_adv_l, border_adv_r) = if adv0 == (0.0, 0.0) {
            adv0
        } else if Self::inline_horizontal_border_advance_has_bidi_controls(
            &doc.text_content(elem_id).unwrap_or_default(),
        ) {
            (0.0, 0.0)
        } else {
            adv0
        };
        let margin_left = style
            .map(|s| Self::resolve_inline_margin(&s.margin_left, s))
            .unwrap_or_else(|| self.margin_overrides.get(&elem_id).map(|(ml, _)| *ml).unwrap_or(0.0))
            + border_adv_l;
        let margin_right = style
            .map(|s| Self::resolve_inline_margin(&s.margin_right, s))
            .unwrap_or_else(|| self.margin_overrides.get(&elem_id).map(|(_, mr)| *mr).unwrap_or(0.0))
            + border_adv_r;
        let padding_left = style
            .map(|s| Self::resolve_inline_padding(&s.padding_left, s))
            .unwrap_or_else(|| self.padding_overrides.get(&elem_id).map(|(pl, _)| *pl).unwrap_or(0.0));
        let padding_right = style
            .map(|s| Self::resolve_inline_padding(&s.padding_right, s))
            .unwrap_or_else(|| self.padding_overrides.get(&elem_id).map(|(pr, _)| *pr).unwrap_or(0.0));

        // R4376：旗标每次 collect 读一次（同 collect_inline_items hoist 口径）。
        let glyph_verticals_enabled = runtime_flags::fallback_line_metrics();
        // 依次收集子节点。
        let children = doc.child_nodes(elem_id);
        let mut text_pending = String::new();
        let mut emitted_text_run = false;
        let flush_pending = |text_pending: &mut String, items: &mut Vec<InlineItem>, first: bool, last_text: bool| {
            if text_pending.is_empty() {
                return;
            }
            let trimmed = if run_preserves {
                std::mem::take(text_pending)
            } else if run_ws.is_some_and(|ws| ws.break_at_newline) {
                // R4398（css-text-3 §white-space-phase-1）：pre-line 的 `\n` 强制断行
                // identity 保留到 break_lines（collapse 有损，`\n`→空格使 marker 永不
                // 产出——pre-line-with-space-and-newline 两行并排一行实证）；段内折叠
                // 由 split 的 break_at_newline 分支按段完成（幂等）。
                crate::inline::collapse_whitespace_keep_newlines(&std::mem::take(text_pending))
            } else {
                collapse_whitespace(text_pending)
            };
            if trimmed.is_empty() {
                return;
            }
            // R4382（CSS Text 3 §3.1）：walk 路径的 run 归因 inline 元素自身——
            // text-transform 同扁平化 run 语义（layout 读元素 computed；Path B 读
            // text_transform_overrides 元素键）。
            let text_transform = style
                .map(|s| s.text_transform)
                .unwrap_or_else(|| {
                    self.text_transform_overrides
                        .get(&elem_id)
                        .copied()
                        .unwrap_or(TextTransformValue::None)
                });
            let trimmed = text_transform.apply(&trimmed);
            // R4374：回退链垂直度量（同主路径——layout 趟按 line-height 语义归一，
            // Path B 经 overrides 复用；门禁关闭或未注册回调时恒 0）。
            let flat_font_id =
                self.shaping_font_id_for_style(Some(elem_id), style, is_ahem_font, letter_spacing, word_spacing, false);
            let flat_font_ids = self
                .ordered_font_ids_for_style(style)
                .or_else(|| self.font_ids_overrides.get(&elem_id).cloned())
                .unwrap_or_else(|| flat_font_id.into_iter().collect());
            let (glyph_ascent, glyph_descent) = match self.glyph_verticals_overrides.get(&elem_id) {
                Some(&(ga, gd)) => (ga, gd),
                None => Self::run_glyph_verticals(
                    glyph_verticals_enabled,
                    &flat_font_ids,
                    &trimmed,
                    font_size,
                    is_ahem_font,
                    line_height,
                    style.is_some_and(|s| matches!(s.line_height, zero_style_system::LineHeightValue::Normal)),
                ),
            };
            items.push(InlineItem::Text(TextRun {
                ws_override: run_ws,
                ruby_rt_ascent: 0.0,
                text: trimmed,
                node_id: elem_id,
                font_size,
                line_height,
                vertical_align: style
                    .map(|s| s.vertical_align.clone())
                    .unwrap_or(VerticalAlignValue::Baseline),
                letter_spacing,
                word_spacing,
                margin_left: if first { margin_left } else { 0.0 },
                margin_right: if last_text { margin_right } else { 0.0 },
                padding_left: if first { padding_left } else { 0.0 },
                padding_right: if last_text { padding_right } else { 0.0 },
                padding_top,
                padding_bottom,
                border_top,
                border_bottom,
                is_ahem_font,
                font_id: flat_font_id,
                is_rtl: style.is_some_and(|s| matches!(s.direction, zero_style_system::DirectionValue::Rtl)),
                bidi_override: Self::element_bidi_override(style),
                is_plaintext_bidi: style
                    .map(|s| matches!(s.unicode_bidi, zero_style_system::UnicodeBidiValue::Plaintext))
                    .unwrap_or_else(|| {
                        self.plaintext_bidi_override || self.plaintext_bidi_overrides.contains(&elem_id)
                    }),
                glyph_ascent,
                glyph_descent,
            }));
            *text_pending = String::new();
        };

        for &gc in &children {
            let Some(node) = doc.get(gc) else { continue };
            match &node.kind {
                NodeKind::Text(text_data) => {
                    text_pending.push_str(&text_data.content);
                }
                NodeKind::Element(elem_data) => {
                    // R109 split 容器成员表：元素子不在成员表时跳过（文本子随外层扁平化，
                    // 不受成员表约束）。
                    if let Some(ids) = &self.fragment_node_ids {
                        if !ids.contains(&gc) {
                            continue;
                        }
                    }
                    // R4321：UA 隐藏元素兜底与主路径同源（见主路径 Element 分支注释）——
                    // walk 递归经 text_content 折回同样会把 Path B 下无 styles 的
                    // script/style 等源文本吸收进 pending。
                    if styles.is_empty() && Self::ua_hidden_without_styles(elem_data.local_name()) {
                        continue;
                    }
                    // R4331（CSS2 §9.2.1 / HTML br 元素）：`<br>` 强制换行条目与主
                    // collect 路径同语义（本文件主循环 Element 分支 br 臂）——walk 扁平化
                    // 此前把 br 落「空文本 + 零 frame」臂 continue 吞掉：inline 元素包裹的
                    // `<br>`（`<span>a<br/>b</span>`）整条丢失强制断行，段落行结构整体
                    // 漂移（run-in-breaking-001 ref 页 span 包裹 br 实证）。按 local_name
                    // 判定（style 无关谓词，paint IFC 与 layout IFC 判定恒同）。
                    // R4332：br 后置 emitted_text_run = true——br 前若已有文本 emit，br 后
                    // 的文本 run 不得再带首 run frame（margin/padding 左值；chromium 首片段
                    // 语义），否则 br 两侧文本都收左推进（ref 页 "header" 多缩进 border 宽实证）。
                    if elem_data.local_name() == "br" {
                        flush_pending(&mut text_pending, items, !emitted_text_run, false);
                        emitted_text_run = true;
                        items.push(InlineItem::Br);
                        continue;
                    }
                    let gc_has_element_children = doc.child_nodes(gc).iter().any(|&gk| {
                        doc.get(gk).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_)))
                    });
                    // R4300c：无元素子的 inline 子一律**折回 pending**（text_content 并入
                    // 外层 pending、归因外层）= 旧扁平化的祖先吸收语义（border-color-012 的
                    // `.text` 壳归因 .inner、ruby rt/rp 随祖先 text_content 携带、bidi 特殊子
                    // 整段吸收，均以此为准）。唯一例外：「空文本 + 有 frame」的 spacer 类子
                    // 落零宽 run（其 ml/pl/mr/pr 经 break_lines 空元素分支推进——这是本
                    // walk 的存在意义，R4299；全无 frame 的空壳不落——保持空白折叠连续性，
                    // border-color-012 的空 `<span class=text>` 后导空格实证）。有元素子：
                    // 递归同规则。
                    if !gc_has_element_children {
                        // R4300d：**white-space 模式不一致的子不折回**——折叠语义随 run 的
                        // ws_override 走，折入父 pending 会按父模式坍缩子文本
                        //（white-space-mixed-001 的 `<span class=pre> </span>` 嵌入 normal
                        // 父：pre 空格被坍缩，20.47→21.38 实证）。模式一致 → 折回（旧祖先
                        // 吸收语义，border-color-012 `.text` 壳归因 .inner）；不一致 → 子自
                        // 身构造独立 run（= 主 collect 路径到达该子时的形状）。
                        let gc_text_empty = doc
                            .text_content(gc)
                            .is_none_or(|t| t.chars().all(|c| c.is_whitespace()));
                        if gc_text_empty {
                            let child_style0 = styles.get(&gc);
                            // R4300e：paint IFC（空 styles）经 overrides 恢复 frame——walk 的
                            // frame 判定若在 paint IFC 恒 0，spacer 零宽 run 在 paint 侧被丢
                            // → paint/layout IFC 分段分歧 → 行断/绘制错位（quotes-001 行 3
                            // 9324px 实证：layout 恒等而 paint 分歧）。padding/margin 走
                            // overrides（R3837 同源），垂直 metrics paint IFC 无源可依记 0
                            //（垂直 frame 只影响行盒高，不影响 inline 轴推进分段）。
                            let frame_sum = {
                                let (pt, pb, bt, bb) = Self::extract_inline_box_metrics(child_style0);
                                let m =
                                    |v: &LengthValue, st: &ComputedStyle| Self::resolve_inline_margin(v, st);
                                let pd =
                                    |v: &LengthValue, st: &ComputedStyle| Self::resolve_inline_padding(v, st);
                                let from_style = child_style0
                                    .map(|st| {
                                        m(&st.margin_left, st)
                                            + m(&st.margin_right, st)
                                            + pd(&st.padding_left, st)
                                            + pd(&st.padding_right, st)
                                            + pt
                                            + pb
                                            + bt
                                            + bb
                                    })
                                    .unwrap_or(0.0);
                                let from_overrides = self
                                    .padding_overrides
                                    .get(&gc)
                                    .map(|(pl, pr)| pl + pr)
                                    .unwrap_or(0.0)
                                    + self.margin_overrides.get(&gc).map(|(ml, mr)| ml + mr).unwrap_or(0.0);
                                from_style.max(from_overrides)
                            };
                            if frame_sum > 0.0 {
                                flush_pending(&mut text_pending, items, !emitted_text_run, false);
                                emitted_text_run = true;
                                if let Some(item) = self.build_flatten_run_for_element(doc, gc, styles) {
                                    items.push(item);
                                }
                            }
                            continue;
                        }
                        let child_ws = styles
                            .get(&gc)
                            .map(|st| Self::run_white_space(&st.white_space))
                            .or_else(|| self.ws_overrides.get(&gc).copied());
                        // R4300d 修正：ws_override=None = **继承容器 flags**（collect 主路径
                        // 同语义），非「与父不同」——旧比较把 (Some, None) 判异，把继承态
                        // 子（如 `<q>` 嵌套）无谓拆 run（quotes-001 的 27v1 整段被拆成
                        // 27/29/32 三段，页高 74→111.7 实证）。按**有效三元组**比较：
                        // None 解析为容器级 preserve/break_at_newline/no_wrap。
                        let effective = |ws: Option<crate::inline::RunWhiteSpace>| {
                            ws.map(|w| (w.preserve, w.break_at_newline, w.no_wrap))
                                .unwrap_or((self.preserve_whitespace, self.break_at_newline, self.no_wrap))
                        };
                        let ws_same = effective(run_ws) == effective(child_ws);
                        if ws_same {
                            if let Some(txt) = doc.text_content(gc) {
                                text_pending.push_str(&txt);
                            }
                            continue;
                        }
                        flush_pending(&mut text_pending, items, !emitted_text_run, false);
                        emitted_text_run = true;
                        if let Some(item) = self.build_flatten_run_for_element(doc, gc, styles) {
                            items.push(item);
                        }
                        continue;
                    }
                    flush_pending(&mut text_pending, items, !emitted_text_run, false);
                    emitted_text_run = true;
                    // R4308：递归不绕过主路径特例门——collect 主路径对 ruby / bidi
                    // 控制字符子不走 walk（R1022 ruby rt/rp 注音语义 / R4300c 控制字符
                    // 重排作用域横跨整段），walk 递归若直接深入会以扁平化吞噬这些特例
                    //（ruby-overhang-spaces-vertical-002/004/006 + ruby-intrinsic-isize-003
                    // walk-on 翻红实证：span.walk 递归进 <ruby> 把 rt 注音折进 base 流）。
                    // 门取**style 无关谓词**（local_name / 文本内容）：paint IFC（空
                    // styles）与 layout IFC 判定恒等，不引入两段分歧。命中即按旧扁平化
                    // 形状落独立 run（build_flatten_run_for_element 对 ruby 自带 rt/rp
                    // 排除）。
                    let gc_is_special_elem = doc
                        .get(gc)
                        .and_then(|n| match &n.kind {
                            // R4308：ruby（rt/rp 注音语义）；R4311：svg（行盒贡献语义，
                            // 见主 collect 门注释）——均 style 无关谓词，paint IFC 判定恒等。
                            NodeKind::Element(e) => {
                                Some(matches!(e.local_name(), "ruby" | "svg"))
                            }
                            _ => None,
                        })
                        .unwrap_or(false);
                    let gc_has_bidi_controls = doc
                        .text_content(gc)
                        .is_some_and(|t| t.chars().any(|c| ('\u{202A}'..='\u{202E}').contains(&c)));
                    // R4310：竖排 writing-mode 子同门（主 collect 路径同判定，见
                    // child_declares_vertical_wm）——递归深入会把竖排子内容扁平化进
                    // 正交轴 IFC 丢失其盒原点。
                    let gc_declares_vertical_wm = styles
                        .get(&gc)
                        .map(|st| {
                            matches!(
                                st.writing_mode,
                                zero_style_system::WritingModeValue::VerticalRl
                                    | zero_style_system::WritingModeValue::VerticalLr
                            )
                        })
                        .unwrap_or_else(|| self.vertical_walk_nodes.contains(&gc));
                    // R4312：块级元素子同门（主 collect 路径同判定）——递归深入含块子
                    // 的 inline 会改变 R109 block-in-inline 盒树/intrinsic 测量。
                    let gc_has_block_child =
                        Self::has_block_level_child(doc, styles, gc, &self.block_child_walk_nodes);
                    if gc_is_special_elem
                        || gc_has_bidi_controls
                        || gc_declares_vertical_wm
                        || gc_has_block_child
                    {
                        if let Some(item) = self.build_flatten_run_for_element(doc, gc, styles) {
                            items.push(item);
                        }
                        continue;
                    }
                    // R4313：子树**无文本**的包装层（嵌套空 span，如 block-in-inline
                    // split ref 页的 notstart/notend 空包装）不递归——递归会把零宽 run
                    // 经 frame_sum 分支归因到最内层空子，外层自身的垂直 border/padding
                    // 行盒贡献丢失（walk-off flatten 按 text_content 归因外层，ref 页
                    // 行盒高随之漂移 → block-in-inline-insert-011 对比 0.82%→3.25%）。
                    // 按 text_content 判定（style 无关，paint IFC 恒等），与 walk-off
                    // 形状逐字节一致。
                    if doc
                        .text_content(gc)
                        .is_none_or(|t| t.chars().all(|c| c.is_whitespace()))
                    {
                        if let Some(item) = self.build_flatten_run_for_element(doc, gc, styles) {
                            items.push(item);
                        }
                        continue;
                    }
                    self.collect_flat_inline_children(doc, gc, styles, items);
                    continue;
                }
                _ => {}
            }
        }
        // 末段文本：仅当其后无元素子时携带外层 mr/pr（有则由末元素子后续 frame 承接，
        // 简化处理：末段文本始终携带——外层 mr/pr 丢失于「末子为元素」形态，挂账）。
        flush_pending(&mut text_pending, items, !emitted_text_run, true);
    }

    /// R4312：`id` 的元素子中是否存在**块级**（display 非 inline 级）——walk 块子门
    /// 判定。有 styles（layout IFC）直判；paint Path B（空 styles）读存储信号
    /// `stored`（`LayoutBox.inline_block_child_nodes`，store_font_sizes_from_ifc 按
    /// run owner 填充，覆盖完备性与竖排门同论证）。
    fn has_block_level_child(
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
        id: NodeId,
        stored: &NodeIdSet,
    ) -> bool {
        styles
            .get(&id)
            .map(|_| {
                doc.child_nodes(id).iter().any(|&gc| {
                    doc.get(gc).is_some_and(|n| matches!(n.kind, NodeKind::Element(_)))
                        && styles.get(&gc).is_some_and(|st| {
                            !matches!(
                                st.display,
                                DisplayValue::Inline
                                    | DisplayValue::InlineBlock
                                    | DisplayValue::InlineFlex
                                    | DisplayValue::InlineGrid
                                    | DisplayValue::InlineTable
                            )
                        })
                })
            })
            .unwrap_or_else(|| stored.contains(&id))
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
            // R4228（css-text-4 #letter-spacing）：百分比相对 used font-size。
            LengthValue::Percentage(p) => font_size * (*p as f32 / 100.0),
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

    /// R3837：inline 水平 padding 解析（px）。百分比按 containing-block 内联轴应基于
    /// 容器宽，但此处沿 margin 同款 font-size 基近似（与 `extract_inline_box_metrics`
    /// 的垂直 padding 域一致）；em/rem 经 resolve_length 按元素字号解析。
    fn resolve_inline_padding(value: &LengthValue, style: &ComputedStyle) -> f32 {
        Self::resolve_inline_margin(value, style)
    }

    /// R3840：元素级 `unicode-bidi: bidi-override` → Some(方向 rtl?)；非 override → None。
    /// 文本节点分支（paint Path B）经 `text_node_bidi_overrides` 映射恢复。
    fn element_bidi_override(style: Option<&ComputedStyle>) -> Option<bool> {
        style.filter(|s| matches!(s.unicode_bidi, zero_style_system::UnicodeBidiValue::BidiOverride))
            .map(|s| matches!(s.direction, zero_style_system::DirectionValue::Rtl))
    }

    /// R4374：run 回退链垂直度量 `(glyph_ascent, glyph_descent)`（行盒贡献幅长 px）。
    ///
    /// 门禁 [`runtime_flags::fallback_line_metrics`]（opt-in `ZW_FALLBACK_LINE_METRICS=1`）
    /// 与宿主注册的全局回调（`FontLoader::fallback_text_line_metrics`，按 run 字体链取
    /// 实际使用字体的行度量 max）须同时满足，任一不满足返回 (0.0, 0.0) = 旧行为。
    /// Ahem run 跳过（Ahem 是测试字体，1.0em 方盒度量恒定，回退链语义无意义——
    /// strut 模型已特判）。
    ///
    /// 行盒贡献按 line-height 语义二分（CSS2 §10.8.1 leading 模型）：
    /// - `normal`：used line-height = 字体自身 ascent+descent（+gap）——行盒按字形
    ///   字体度量**撑开**（CJK 回退字体 1.448em > 主字体 strut 1.164em），贡献 =
    ///   原始 (ascent, descent)。
    /// - 显式（Length/Number）：行盒高度 = line-height **不撑开**，大度量只把基线
    ///   在盒内下移——贡献 = `L/2 ± (a−d)/2`（leading 对半分布于字体盒两侧）。
    fn run_glyph_verticals(
        fallback_enabled: bool,
        font_ids: &[u32],
        text: &str,
        font_size: f32,
        is_ahem: bool,
        line_height: f32,
        line_height_is_normal: bool,
    ) -> (f32, f32) {
        if !fallback_enabled || is_ahem || text.is_empty() {
            return (0.0, 0.0);
        }
        let Some((ga, gd)) = crate::inline::font_metrics::fallback_line_metrics_for_layout(font_ids, text, font_size)
        else {
            return (0.0, 0.0);
        };
        crate::inline::font_metrics::glyph_verticals_contribution(ga, gd, line_height, line_height_is_normal)
    }

    /// R4378：run 的**有序 CSS face 列表**——元素 `font-family` 逐族 weight-aware 解析
    /// （`font_id_for_style` 同款单族匹配判定，收集**全部**命中族）。多 `@font-face` +
    /// unicode-range 页面的垂直度量必须按 CSS 序落 face：`high-a-only, deep-b-only` 的
    /// 'b' 应落 deep-b-only（元素声明的第二 face），而非 global fallback chain 的
    /// DejaVu/CJK——错链解析会引入错误（偏大）度量撑开行盒（content-height-004 翻红
    /// 实证）。无 style（paint Path B）由调用方经 `font_ids_overrides` 取有序表。
    fn ordered_font_ids_for_style(&self, style: Option<&ComputedStyle>) -> Option<Vec<u32>> {
        use zero_css_parser::values::FontWeightValue;
        let s = style?;
        let want_bold = matches!(s.font_weight, FontWeightValue::Bold | FontWeightValue::Bolder)
            || matches!(s.font_weight, FontWeightValue::Absolute(weight) if weight >= 600);
        let want_italic = matches!(
            s.font_style,
            zero_style_system::values::types::FontStyleValue::Italic
                | zero_style_system::values::types::FontStyleValue::Oblique(_)
        );
        let mut ids: Vec<u32> = Vec::new();
        for family in &s.font_family {
            let is_quoted = family.starts_with('"') || family.starts_with('\'');
            let bare = family.trim_matches('"').trim_matches('\'');
            if is_quoted
                && ["serif", "sans-serif", "monospace", "cursive", "fantasy", "system-ui"]
                    .iter()
                    .any(|g| g.eq_ignore_ascii_case(bare))
            {
                continue;
            }
            let Some(resolver) = self.font_resolver.as_ref() else {
                continue;
            };
            if let Some((id, _)) =
                zero_render_foundation::font::resolve_font_face(resolver, bare, want_bold, want_italic, s.font_stretch)
                && !ids.contains(&id)
            {
                ids.push(id);
            }
        }
        (!ids.is_empty()).then_some(ids)
    }
}

/// R4357（css-ruby-1 §ruby-overhang；chromium probe 模型 15 数据点全拟合）：ruby 注音
/// 行宽参与——rt 注释宽（0.5em 字号）计入行盒，可悬挂部分吸收到邻字符空白半侧：
/// `box_w = max(base_w, rt_w − hang_l − hang_r)`，extra = box_w − base_w 对折进 ruby run
/// 左右 margin（base 字形居中于盒，与 ruby-align:center 一致；rt paint 层 R1689 overlay
/// 本就按 base 段居中，无需改动）。hang = ruby 紧邻字符朝向 ruby 的空白半侧：收窄标点
/// ）】等（字形居左、右半空白）作左邻 → 0.5em；开括标点（【等（字形居右、左半空白）
/// 作右邻 → 0.5em；全角空白 U+3000 → 1em；其余 0。水平书写限定；
/// kill-switch `ZW_RUBY_OVERHANG_MODEL=0`。collect（run margin）与 intrinsic
/// （dom_inline_text_max_width）双消费——两侧必须同口径，否则 max-content 与行宽分裂。
#[allow(clippy::too_many_arguments)]
pub(crate) fn ruby_overhang_pads(
    doc: &Document,
    ruby_id: NodeId,
    base_w: f32,
    annot_text: &str,
    font_size: f32,
    letter_spacing: f32,
    is_ahem: bool,
    tab: Option<(f32, f32)>,
) -> (f32, f32) {
    // R4422 flip：模型默认**开**（ZW_RUBY_OVERHANG_MODEL=0 为杀开关）——R4357 时代的
    // 默认开净回退卡点（margin/paint overlay 坐标契约、max-content 与行宽分裂）已经
    // R4358-R4422 七轮清算（独有红清零、零新翻红 bar 达成），按 roadmap flip 实测。
    if std::env::var("ZW_RUBY_OVERHANG_MODEL").as_deref() == Ok("0") || annot_text.is_empty() {
        return (0.0, 0.0);
    }
    // R4415：嵌套 ruby 卫兵——css-ruby #nested-pairing 下外层 ruby 的 base 序列含内层
    // ruby（自身已按各自 rt 配对展开），外层 annotation 的单段 extra 模型不再成立
    //（nested-ruby-pairing-001：東南+内层 ruby 配 とう/なん/Southeast，外层 pads 1/2.6
    // 把 base 推离 ref 位）。含嵌套 ruby 子树时整层跳过 pads（各自内层已建模）。
    if has_nested_ruby_descendant(doc, ruby_id) {
        return (0.0, 0.0);
    }
    let rt_w: f32 = annot_text
        .chars()
        .map(|c| crate::inline::estimate_char_width(c, font_size * 0.5, is_ahem))
        .sum::<f32>()
        + letter_spacing * annot_text.chars().count() as f32;
    // R4411：悬挂容量优先取**边界空白串**（css-ruby #ruby-overhang spaces——注音可悬挂
    // 于相邻空白字符的空白部分；ruby-overhang-spaces-003/006/009/014/015/016 邻接均为
    // U+0020/U+3000/thin spaces 串）。旧 adjacent_text_char 显式跳过空白 → 空白邻接时
    // 容量恒 0（extra 全额入 margin → 行宽膨胀翻红）。空白串容量 = 折叠后各空白字符
    // advance 之和（U+3000 = 1em，其余走 estimate_char_width 与 IFC 行宽同源）；
    // 边界无空白时回落旧标点半空白逻辑（15 数据点拟合域不变）。
    // R4416：preserve 语境（pre/pre-wrap/break-spaces）容量走**原始**空白串——tab 按
    // 名义 unit 计（行内按 stop 推进，stop 对齐页同量；折叠语义会把 tab 串塌成单空格
    // 5px，与行内 40px 分裂 → pads 55/55 过冲，overhang-spaces-006 region 210≠140）。
    let hang_l = ruby_hang_capacity(doc, ruby_id, false, font_size, tab)
        .unwrap_or_else(|| ruby_hang_extent(adjacent_text_char(doc, ruby_id, false), font_size, false));
    let hang_r = ruby_hang_capacity(doc, ruby_id, true, font_size, tab)
        .unwrap_or_else(|| ruby_hang_extent(adjacent_text_char(doc, ruby_id, true), font_size, true));
    let extra = (rt_w - hang_l - hang_r - base_w).max(0.0);
    let half = (extra / 2.0).floor();
    // R4415 临时诊断（ZW_DEBUG_IFC=1）。
    if std::env::var("ZW_DEBUG_IFC").as_deref() == Ok("1") {
        eprintln!(
            "[pads] base_w={} rt_w={} hang_l={} hang_r={} extra={} -> ({}, {})",
            base_w, rt_w, hang_l, hang_r, extra, half,
            extra - half
        );
    }
    (half, extra - half)
}

/// ruby 紧邻方向的贴端字符（prev=false 取前兄弟末字符、true 取后兄弟首字符）；
/// 跨元素兄弟递归取端字符（折叠后空白跳过）。
/// ruby 紧邻方向的贴端字符（prev=false 取前兄弟末字符、true 取后兄弟首字符）；
/// 跨元素兄弟递归取端字符（折叠后空白跳过）。
fn adjacent_text_char(doc: &Document, ruby_id: NodeId, forward: bool) -> Option<char> {
    let mut cur = if forward { doc.next_sibling(ruby_id) } else { doc.previous_sibling(ruby_id) };
    while let Some(sid) = cur {
        if let Some(node) = doc.get(sid) {
            match &node.kind {
                zero_dom::NodeKind::Text(t) => {
                    let collapsed = crate::inline::collapse_whitespace(&t.content);
                    let ch = if forward {
                        collapsed.chars().find(|c| !c.is_whitespace())
                    } else {
                        collapsed.chars().rev().find(|c| !c.is_whitespace())
                    };
                    if ch.is_some() {
                        return ch;
                    }
                }
                zero_dom::NodeKind::Element(e) => {
                    // display:none 兄弟（rt/rp 不该出现在这里，防御性跳过）与 br 停止。
                    if e.local_name().eq_ignore_ascii_case("br") {
                        return None;
                    }
                    let text = doc.text_content(sid).unwrap_or_default();
                    let collapsed = crate::inline::collapse_whitespace(&text);
                    let ch = if forward {
                        collapsed.chars().find(|c| !c.is_whitespace())
                    } else {
                        collapsed.chars().rev().find(|c| !c.is_whitespace())
                    };
                    if ch.is_some() {
                        return ch;
                    }
                }
                _ => {}
            }
        }
        cur = if forward { doc.next_sibling(sid) } else { doc.previous_sibling(sid) };
    }
    None
}

/// ruby 的 rt 注释文本（全部 rt 拼接；多段 ruby 近似为单段——家族案均单段，
/// 多段逐段 box 模型独立子问题）。空白剔除与 paint 侧 ruby_annotation_segments 同口径。
/// ruby 的 rt 注释文本（全部 rt 拼接；多段 ruby 近似为单段——家族案均单段，
/// 多段逐段 box 模型独立子问题）。空白剔除与 paint 侧 ruby_annotation_segments 同口径。
pub(crate) fn ruby_annotation_width_text(doc: &Document, ruby_id: NodeId) -> String {
    // R4422（css-ruby-1 #nested-pairing）：注音宽 = **最宽注音行**的文本（行 0 =
    // 最内层 per-base 注音拼接，行 1 = span-all 外层注音），非全量拼接——嵌套/rtc
    // 双形态（nested-pairing test 页 vs ref 页）的行结构同构后行宽必须同值，否则
    // pads 分母分裂 → 折行差（ref 页行宽多出 "とうなん" 段 → 角 折行）。普通单 rt
    // （行 1 缺席）与 rbc/rtc complex（行 0 = rtc 内 rt 拼接）口径不变。
    // 消费方（R4357 overhang pads / R4359 rt ascent / intrinsic walk ruby 臂）均
    // opt-in 门（base 排除在 collect 主路径已同步，注音宽与 base 口径一致）。
    let stripped = |id: NodeId| -> String {
        doc.text_content(id)
            .unwrap_or_default()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect()
    };
    let has_nested_ruby = doc.child_nodes(ruby_id).iter().any(|c| {
        doc.get(*c).is_some_and(|n| {
            matches!(n.kind, NodeKind::Element(ref e) if e.local_name().eq_ignore_ascii_case("ruby"))
        })
    });
    let mut row0 = String::new();
    let mut row1 = String::new();
    for child_id in doc.child_nodes(ruby_id) {
        let Some(node) = doc.get(child_id) else { continue };
        if let NodeKind::Element(elem) = &node.kind {
            let name = elem.local_name();
            if name.eq_ignore_ascii_case("rt") {
                if has_nested_ruby {
                    row1 = stripped(child_id);
                } else {
                    row0.push_str(&stripped(child_id));
                }
            } else if name.eq_ignore_ascii_case("rtc") {
                let has_rt_child = doc.child_nodes(child_id).iter().any(|c| {
                    doc.get(*c).is_some_and(|n| {
                        matches!(n.kind, NodeKind::Element(ref e) if e.local_name().eq_ignore_ascii_case("rt"))
                    })
                });
                if has_rt_child {
                    for &rt_id in doc.child_nodes(child_id).iter() {
                        if doc.get(rt_id).is_some_and(|n| {
                            matches!(n.kind, NodeKind::Element(ref e) if e.local_name().eq_ignore_ascii_case("rt"))
                        }) {
                            row0.push_str(&stripped(rt_id));
                        }
                    }
                } else {
                    row1 = stripped(child_id);
                }
            } else if name.eq_ignore_ascii_case("ruby") {
                // 嵌套 ruby：其 rt 文本 = 行 0（内层配对），与 paint 侧分段同分类。
                for &nid in doc.child_nodes(child_id).iter() {
                    if doc.get(nid).is_some_and(|n| {
                        matches!(n.kind, NodeKind::Element(ref e) if e.local_name().eq_ignore_ascii_case("rt"))
                    }) {
                        row0.push_str(&stripped(nid));
                    }
                }
            }
        }
    }
    // 取估算宽更宽的行（rt fs = 0.5em 语义与 pads 同源；行内同 fs 下字符数等价）。
    if row1.chars().count() > row0.chars().count() {
        row1
    } else {
        row0
    }
}

/// 朝向 ruby 的空白半侧额度（em 由 font_size 表达）。
/// 朝向 ruby 的空白半侧额度（em 由 font_size 表达）。
fn ruby_hang_extent(ch: Option<char>, font_size: f32, leading: bool) -> f32 {
    let Some(c) = ch else { return 0.0 };
    // 全角空白（U+3000 ideographic space）：两侧全额 1em。
    if c == '\u{3000}' {
        return font_size;
    }
    // 收窄标点（字形居左、右半空白；U+3001..U+301F CJK 符号 + 全角形式 U+FF01..）——
    // 作 ruby 前邻字符时右半空白朝向 ruby。含 WPT ruby-overhang-spaces-002 全对集
    // （）」』】〗〕〉》〙〛〞〟］｝｠）。
    const CLOSING: [char; 20] = [
        '）', '】', '〉', '》', '」', '』', '〕', '〗', '〙', '〛', '〞', '〟', '｝', '｠', '］', '、', '。', '！', '？', '：',
    ];
    // 开括标点（字形居右、左半空白）——作 ruby 后邻字符时左半空白朝向 ruby。
    const OPENING: [char; 15] = ['（', '【', '〈', '《', '「', '『', '〔', '〖', '〘', '〚', '〝', '｛', '｟', '［', '＄'];
    if !leading && (CLOSING.contains(&c) || matches!(c, '；' | '，' | '．' | '＇')) {
        return font_size * 0.5;
    }
    if leading && OPENING.contains(&c) {
        return font_size * 0.5;
    }
    0.0
}


/// R4415：ruby 子树是否嵌套内层 ruby（css-ruby #nested-pairing 判据）。
/// 递归元素子树；rt/rp/rtc 的 display:none 子树同样检查（防御性，注音内嵌 ruby
/// 语境 pads 亦不成立）。
fn has_nested_ruby_descendant(doc: &Document, ruby_id: NodeId) -> bool {
    fn walk(doc: &Document, node_id: NodeId) -> bool {
        for child_id in doc.child_nodes(node_id) {
            let Some(node) = doc.get(child_id) else { continue };
            if let zero_dom::NodeKind::Element(e) = &node.kind {
                if e.local_name().eq_ignore_ascii_case("ruby") || walk(doc, child_id) {
                    return true;
                }
            }
        }
        false
    }
    walk(doc, ruby_id)
}

/// R4411：ruby 边界空白串的悬挂容量——前/后兄弟文本贴 ruby 端的折叠空白 run
/// 各字符 advance 之和。无空白（首字符即非空白 / 无兄弟文本）→ None（调用方回落
/// 标点半空白逻辑）。css-ruby-1 §ruby-overhang：注音可悬挂于空白字符的空白部分
/// ——整段空白均为可用容量（006/014 双/三连 U+3000 实证）。
fn ruby_hang_capacity(
    doc: &Document,
    ruby_id: NodeId,
    forward: bool,
    font_size: f32,
    tab: Option<(f32, f32)>,
) -> Option<f32> {
    let mut cur = if forward { doc.next_sibling(ruby_id) } else { doc.previous_sibling(ruby_id) };
    while let Some(sid) = cur {
        if let Some(node) = doc.get(sid) {
            match &node.kind {
                zero_dom::NodeKind::Text(t) => {
                    // R4416：preserve 语境用原始文本（空白串不折叠，tab 计入容量）。
                    let source = if tab.is_some() {
                        t.content.clone()
                    } else {
                        crate::inline::collapse_whitespace(&t.content)
                    };
                    let run: String = if forward {
                        source.chars().take_while(|c| c.is_whitespace()).collect()
                    } else {
                        source.chars().rev().take_while(|c| c.is_whitespace()).collect()
                    };
                    if !run.is_empty() {
                        // (unit, space)：preserve 臂 tab 计名义 unit、其余空白计空格
                        // advance；折叠臂保持 estimate 同源（None → unit/space 恒 0，
                        // 分支不达）。
                        let (tab_unit, tab_space) = tab.unwrap_or((0.0, 0.0));
                        let cap: f32 = run
                            .chars()
                            .map(|c| match c {
                                '\u{3000}' => font_size,
                                '\t' => tab_unit,
                                _ if tab.is_some() => tab_space,
                                _ => crate::inline::estimate_char_width(c, font_size, false),
                            })
                            .sum();
                        return Some(cap);
                    }
                    return None;
                }
                // br 停止（行界）；其他元素子树无裸空白边界（跨元素空白已折叠跳过域）。
                zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("br") => {
                    return None;
                }
                zero_dom::NodeKind::Element(_) => {}
                _ => {}
            }
        }
        cur = if forward { doc.next_sibling(sid) } else { doc.previous_sibling(sid) };
    }
    None
}

#[cfg(test)]
mod r4395_rtc_annotation_tests {
    use super::*;

    /// R4395（css-ruby-1 §ruby-text-container）：rtc 是注音容器——
    /// `ruby_annotation_width_text` 并入 rtc 文本（与 rt 同口径）。
    #[test]
    fn rtc_text_joins_annotation_width_text() {
        let doc = zero_dom::parse_html(
            r#"<html><body><ruby><rb>東</rb><rt>とう</rt><rtc>Southeast</rtc></ruby></body></html>"#,
        );
        let ruby = doc.get_elements_by_tag_name("ruby")[0];
        let annot = ruby_annotation_width_text(&doc, ruby);
        // R4422 行感知模型：行 0（とう）与行 1（Southeast）取**最宽行**——双行注音
        // （nested-pairing ref 形态）行宽与 paint span-all 行同源，拼接口径废（行宽
        // 多出 とう 段 → pads 分母分裂 → 折行差）。Southeast（9 字符）> とう（2）。
        assert_eq!(annot, "Southeast", "行 max 模型：最宽注音行");
    }

    /// 纯 rt ruby 注音宽不受 rtc 扩展影响（回归守护）。
    #[test]
    fn rt_only_annotation_width_unchanged() {
        let doc = zero_dom::parse_html(r#"<html><body><ruby>漢<rt>かん</rt></ruby></body></html>"#);
        let ruby = doc.get_elements_by_tag_name("ruby")[0];
        assert_eq!(ruby_annotation_width_text(&doc, ruby), "かん");
    }

    /// 无注音 ruby → 空串（R4357 pads / R4359 ascent 消费方的 empty 判据不变）。
    #[test]
    fn annotation_width_empty_without_rt_or_rtc() {
        let doc = zero_dom::parse_html(r#"<html><body><ruby><rb>東</rb><rb>南</rb></ruby></body></html>"#);
        let ruby = doc.get_elements_by_tag_name("ruby")[0];
        assert_eq!(ruby_annotation_width_text(&doc, ruby), "");
    }

    /// R4402：注音排除泛化到**任意**元素的扁平化收集——rbc/rb/包裹 span 等非 ruby
    /// 元素的 doc.text_content 同样会吞并 rt/rp/rtc 注音文本
    ///（improperly-contained-annotation：rbc flatten run="BA" 55.6 vs ruby 26.7，
    /// 6.5 remeasure 母子宽倒挂实证）。collect 主路径（build_flatten_run_for_element）
    /// 现对所有元素走同一排除表。
    #[test]
    fn flatten_annotation_exclusion_applies_to_non_ruby_elements() {
        // rbc 基容器形态：rt 注音不进收集文本。
        let doc = zero_dom::parse_html(
            r#"<html><body><rbc>B<rt>A</rt></rbc></body></html>"#,
        );
        let rbc = doc.get_elements_by_tag_name("rbc")[0];
        let text = InlineFormattingContext::collect_text_excluding(&doc, rbc, &["rt", "rp", "rtc"]);
        assert_eq!(text, "B", "rbc 收集 = base 文本，rt 排除");
        // 包裹 span 形态：嵌套 ruby 的注音同样排除。
        let doc = zero_dom::parse_html(
            r#"<html><body><span>x<ruby>y<rt>z</rt></ruby></span></body></html>"#,
        );
        let span = doc.get_elements_by_tag_name("span")[0];
        let text = InlineFormattingContext::collect_text_excluding(&doc, span, &["rt", "rp", "rtc"]);
        assert_eq!(text, "xy", "span 收集排除嵌套 rt");
    }

    /// ruby base run 文本排除 rtc 子树（Nested-ruby-pairing-001 ref 页语义：
    /// "Southeast" 不再并入 base 流以全尺寸字形渲染）。
    #[test]
    fn ruby_base_run_excludes_rtc_subtree() {
        let doc = zero_dom::parse_html(
            r#"<html><body><ruby><rb>東</rb><rb>南</rb><rt>とう</rt><rtc lang=en>Southeast</rtc></ruby></body></html>"#,
        );
        let ruby = doc.get_elements_by_tag_name("ruby")[0];
        // InlineFormattingContext::collect_text_excluding 为 base run 收集入口
        //（collect 主路径同参调用 &["rt", "rp", "rtc"]）。
        let base = InlineFormattingContext::collect_text_excluding(&doc, ruby, &["rt", "rp", "rtc"]);
        assert_eq!(base, "東南", "base = rb 文本，rt/rtc 子树排除");
    }
}
