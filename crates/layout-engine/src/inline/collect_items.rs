// 行内条目收集方法（collect_inline_items）— 从 mod.rs 拆分以控制文件体积
// （include! 模式，≡ apps/browser/src/app.rs → app_input.rs；零行为/可见性变更）
impl InlineFormattingContext {
    /// R3778：ComputedStyle.white_space → run 级 white-space 三标志
    ///（preserve / break_at_newline / no_wrap）。与 inline_finalization 容器级映射同源
    ///（Pre=(T,T,T) 的 `\n` 断行由 preserve 模式的 split_into_words 承载，故
    /// break_at_newline 恒随 preserve 置位以简化 run 侧判定）。
    fn run_white_space(ws: &zero_style_system::WhiteSpaceValue) -> crate::inline::RunWhiteSpace {
        use zero_style_system::WhiteSpaceValue;
        match ws {
            WhiteSpaceValue::Pre => crate::inline::RunWhiteSpace { preserve: true, break_at_newline: true, no_wrap: true },
            WhiteSpaceValue::PreWrap => crate::inline::RunWhiteSpace { preserve: true, break_at_newline: true, no_wrap: false },
            WhiteSpaceValue::PreLine => crate::inline::RunWhiteSpace { preserve: false, break_at_newline: true, no_wrap: false },
            WhiteSpaceValue::BreakSpaces => crate::inline::RunWhiteSpace { preserve: true, break_at_newline: true, no_wrap: false },
            WhiteSpaceValue::Nowrap => crate::inline::RunWhiteSpace { preserve: false, break_at_newline: false, no_wrap: true },
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
                                && let Some((dw, dh)) =
                                    crate::svg_default_size::svg_default_used_size(elem_data, style)
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

                        // 其他 inline 元素的文本内容也收集进来
                        // R1022：<ruby> 默认 text_content 会扁平化 <rt>/<rp> 文本
                        // （● 当行内字符渲染）。改为只收集 rb 文本作 inline 流，
                        // rt 文本由 paint 期作 zero-width annotation 上移到 rb 之上。
                        let style = styles.get(&child_id);
                        // R3778：run 级有效 white-space 在**折叠前**判定（collapse 有损，
                        // `\n`→空格不可逆）——inline 元素声明的 pre 使其整段文本保留原始
                        // 换行/空白（line-clamp-014 类：span 包裹 pre 代码块）。
                        let run_ws = style
                            .map(|s| Self::run_white_space(&s.white_space))
                            .or_else(|| self.ws_overrides.get(&child_id).copied());
                        let run_preserves = run_ws.map_or(self.preserve_whitespace, |ws| ws.preserve);
                        let text = if elem_data.local_name() == "ruby" {
                            Self::collect_text_excluding(doc, child_id, &["rt", "rp"])
                        } else {
                            doc.text_content(child_id).unwrap_or_default()
                        };
                        let trimmed = if run_preserves { text } else { collapse_whitespace(&text) };
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
                        // R3837：inline 水平 padding 参与 inline 轴推进（CSS2.1 §8.4）。
                        // paint IFC（无 style）经 padding_overrides 恢复（fragment 级存储）。
                        let padding_left = style
                            .map(|s| Self::resolve_inline_padding(&s.padding_left, s))
                            .unwrap_or_else(|| self.padding_overrides.get(&child_id).map(|(pl, _)| *pl).unwrap_or(0.0));
                        let padding_right = style
                            .map(|s| Self::resolve_inline_padding(&s.padding_right, s))
                            .unwrap_or_else(|| self.padding_overrides.get(&child_id).map(|(_, pr)| *pr).unwrap_or(0.0));
                        let is_ahem_font = style
                            .map(|s| s.font_family.iter().any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem")))
                            .unwrap_or_else(|| self.is_ahem_overrides.get(&child_id).copied().unwrap_or(false));
                        // CSS 2.1: inline 元素的 padding 和 border 参与行盒高度计算
                        let (padding_top, padding_bottom, border_top, border_bottom) =
                            Self::extract_inline_box_metrics(style);
                        if !trimmed.is_empty() {
                            items.push(InlineItem::Text(TextRun {
                                ws_override: run_ws,
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
                                // R3840：元素级 unicode-bidi:bidi-override——其文本按 UAX #9
                                // X2/X3 强制方向逐字符反转（R3319 只实现容器级）。
                                bidi_override: Self::element_bidi_override(style),
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
                                ws_override: style.map(|s| Self::run_white_space(&s.white_space)),
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
}
