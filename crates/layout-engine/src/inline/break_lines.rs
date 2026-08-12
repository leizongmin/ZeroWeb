// 行盒分割方法（break_into_lines / break_items_into_lines）— 从 mod.rs 拆分以控制文件体积
// （include! 模式，≡ apps/browser/src/app.rs → app_input.rs；零行为/可见性变更）
impl InlineFormattingContext {
    /// 将文本运行按可用宽度分割成行盒。
    ///
    /// 便捷方法：将 `Vec<TextRun>` 包装为 `InlineItem::Text` 后调用 [`break_items_into_lines`]。
    pub fn break_into_lines(&mut self, runs: Vec<TextRun>) {
        let items: Vec<InlineItem> = runs.into_iter().map(InlineItem::Text).collect();
        self.break_items_into_lines(items);
    }

    /// 将行内级条目按可用宽度分割成行盒。
    ///
    /// 支持 `InlineItem::Text`（按单词拆分行）、`InlineItem::InlineBlock`（原子盒，不可拆分）
    /// 和 `InlineItem::Br`（强制换行）。浮动排除区域会缩小每行的可用宽度。
    pub fn break_items_into_lines(&mut self, items: Vec<InlineItem>) {
        self.lines.clear();
        let plaintext_enabled = std::env::var("ZW_PLAINTEXT_LINE_DIRECTION").as_deref() != Ok("0")
            && items
                .iter()
                .any(|item| matches!(item, InlineItem::Text(run) if run.is_plaintext_bidi));
        let plaintext_directions = if plaintext_enabled {
            plaintext_paragraph_directions(&items)
        } else {
            HashMap::new()
        };

        // 诊断（R2027）：`ZW_DEBUG_IFC=1` 时对含 inline-block item 的容器 dump 条目构成 +
        // 各 inline-block 的 (w, h, baseline) + 最终行盒高度。Phase A IFC 调试用（line-box
        // 高度贡献 / inline-block 度量是否 stale）。零默认开销（env 未设即跳过）。
        let debug_ifc = std::env::var("ZW_DEBUG_IFC").is_ok();
        if debug_ifc {
            let n_text = items.iter().filter(|i| matches!(i, InlineItem::Text(_))).count();
            let n_ib = items
                .iter()
                .filter(|i| matches!(i, InlineItem::InlineBlock(_)))
                .count();
            let n_br = items.iter().filter(|i| matches!(i, InlineItem::Br)).count();
            if n_ib > 0 {
                let ib_dims: Vec<(f32, f32, f32)> = items
                    .iter()
                    .filter_map(|i| match i {
                        InlineItem::InlineBlock(b) => Some((b.width, b.height, b.baseline)),
                        _ => None,
                    })
                    .collect();
                eprintln!(
                    "[ZW_DEBUG_IFC] items: text={n_text} inline_block={n_ib} br={n_br} container_w={:.1} ib_dims(w,h,baseline)={ib_dims:?}",
                    self.container_width
                );
            }
        }

        if self.vertical {
            self.break_items_into_columns(items);
            return;
        }

        // 追踪当前行的 y 偏移量（用于计算浮动排除区域）
        let mut current_y = 0.0_f32;
        // 估算默认行高（用于初始浮动排除计算）
        let default_line_height = 20.0_f32;

        let mut current_line = LineBox {
            y: 0.0,
            height: 0.0,
            runs: Vec::new(),
            baseline_y: 0.0,
            ascent: 0.0,
            descent: 0.0,
        };
        // text-indent 仅作用于首行
        let mut current_x = self.text_indent;
        // 跟踪当前行内最近一次贡献宽度的内容是否为可折叠空白，
        // 用于将连续纯空白 run（如 inline-block 之间被注释分隔的两个文本节点）
        // 按 CSS Text §4.1 折叠为单个空格。
        let mut last_was_collapsible_ws = false;

        for item in items {
            match item {
                InlineItem::Text(run) => {
                    // 应用 BiDi 重排序（RTL 文本需要视觉顺序）
                    // https://drafts.csswg.org/css-writing-modes-3/#valdef-unicode-bidi-plaintext
                    // plaintext 必须先按逻辑内容断行，再逐行应用段落方向；整段预重排会把
                    // 后续 Latin 词搬到 RTL 段首，改变软换行结果。
                    let logical_plaintext = plaintext_enabled && run.is_plaintext_bidi;
                    let mut source_cursor = if let Some(is_rtl) = self.bidi_override_direction {
                        BidiFragmentCursor::with_override(&run.text, is_rtl)
                    } else if logical_plaintext {
                        BidiFragmentCursor::logical(&run.text)
                    } else {
                        BidiFragmentCursor::with_direction(&run.text, run.is_rtl, run.is_plaintext_bidi)
                    };
                    // 按字符类别逐字符估算宽度，替代统一 0.6 倍近似
                    let words = self.split_into_words(source_cursor.visual_text(), run.is_ahem_font);

                    // 空 inline 元素：文本为空但 line-height + padding + border 仍需贡献到行盒高度
                    if words.is_empty() && run.text.is_empty() {
                        // 空 inline 盒有几何（padding/border），打破可折叠空白连续性
                        last_was_collapsible_ws = false;
                        if run.box_height() > current_line.height {
                            current_line.height = run.box_height();
                        }
                        // 即使空元素也要消费 margin（CSS 2.1 §10.2：inline 元素的 margin 水平方向有效）
                        if run.margin_left > 0.0 {
                            current_x += run.margin_left;
                        }
                        // 为纯空 inline 元素保留一个零宽 fragment。
                        // 这样 layout/paint 后处理仍可感知其几何，写回真实的 inline box 尺寸，
                        // 并在需要时绘制 padding/border/background。
                        current_line.runs.push(TextFragment {
                            x: current_x,
                            y: 0.0,
                            width: 0.0,
                            height: run.line_height,
                            text: String::new(),
                            source: None,
                            node_id: run.node_id,
                            font_size: run.font_size,
                            vertical_align: run.vertical_align.clone(),
                            is_ahem: run.is_ahem_font,
                            letter_spacing: 0.0,
                            margin_left: run.margin_left,
                            margin_right: run.margin_right,
                            margin_top: 0.0,
                            baseline: run.font_size,
                        });
                        if run.margin_right > 0.0 {
                            current_x += run.margin_right;
                        }
                        continue;
                    }

                    // 纯空白文本节点（collapse_whitespace 折叠后的单个空格）：
                    // 作为行内级盒之间的间距贡献一个空格宽度，使后续盒在放不下时正确换行。
                    // CSS Text §4.1：行首空白（当前行为空）被移除；
                    // 连续纯空白 run 折叠为单个空格（last_was_collapsible_ws）。
                    if words.is_empty() {
                        if !current_line.runs.is_empty() && !last_was_collapsible_ws {
                            current_x += self.advance_of(' ', run.font_id, run.font_size, run.is_ahem_font);
                            last_was_collapsible_ws = true;
                        }
                        continue;
                    }
                    last_was_collapsible_ws = false;

                    // 在第一个词之前添加 margin-left
                    if run.margin_left > 0.0 {
                        current_x += run.margin_left;
                    }

                    for (word_idx, word) in words.iter().enumerate() {
                        // CSS 2.1 §16.6.1：normal/nowrap 模式下行尾空格不渲染，不计入行宽。
                        // 将尾部空格从内容宽度中分离，仅作为词间距离使用。
                        // pre/pre-wrap 模式（preserve_whitespace）空格不可折叠，不剥离。
                        let (content_word, trailing_space_width) = if !self.preserve_whitespace && word.ends_with(' ') {
                            let trimmed = word.trim_end_matches(' ');
                            let space_count = word.len() - trimmed.len();
                            let space_w =
                                self.advance_of(' ', run.font_id, run.font_size, run.is_ahem_font) * space_count as f32;
                            (trimmed, space_w)
                        } else {
                            (word.as_str(), 0.0f32)
                        };

                        // R1447：制表符（preserve 模式保留为 "\t" 片段）按 tab stop 推进
                        //（CSS Text 3 §4.1.3）——下一个 tab_size 倍数，非固定 tab_size 空格。
                        // tab stop 相对内容盒起点（0）；current_x 已含 text-indent/float/前置词宽，
                        // 故直接用作行内位置。tab 是空白：无 word-spacing/autospace 前导 gap，
                        // 贡献行高，渲染为不可见（空文本片段，宽度由 current_x 序列消费）。
                        if self.preserve_whitespace && content_word == "\t" {
                            let space_advance = self.advance_of(' ', run.font_id, run.font_size, run.is_ahem_font);
                            let tab_unit = self.tab_size.max(1.0) * space_advance;
                            let pos = current_x;
                            let next_stop = ((pos / tab_unit).floor() + 1.0) * tab_unit;
                            let tab_advance = (next_stop - pos).max(space_advance);
                            if run.line_height > current_line.height {
                                current_line.height = run.line_height;
                            }
                            current_line.runs.push(crate::inline::TextFragment {
                                x: current_x,
                                y: 0.0,
                                width: tab_advance,
                                height: run.line_height,
                                text: String::new(),
                                source: None,
                                node_id: run.node_id,
                                font_size: run.font_size,
                                vertical_align: run.vertical_align.clone(),
                                is_ahem: run.is_ahem_font,
                                letter_spacing: 0.0,
                                margin_left: 0.0,
                                margin_right: 0.0,
                                margin_top: 0.0,
                                baseline: run.font_size,
                            });
                            current_x += tab_advance;
                            last_was_collapsible_ws = false;
                            continue;
                        }

                        // CSS 2.1 §16.6.1：行首空格不渲染。
                        // 当前行首的第一个词如果以空格开头，去除前导空格。
                        let content_word = if current_line.runs.is_empty()
                            && !self.preserve_whitespace
                            && content_word.starts_with(' ')
                        {
                            content_word.trim_start_matches(' ')
                        } else {
                            content_word
                        };
                        // CSS Text §3.1：pre/pre-wrap/pre-line 模式下，换行符 `\n` 是强制断行机会。
                        // split_into_words（preserve_whitespace 或 break_at_newline 模式）为每个 `\n`
                        // 推入空字符串作为强制换行标记——此处消费它：把当前行推入结果并开始新行（同 <br>）。
                        // 旧实现在此只对空词 continue，静默丢弃标记 → 多行 <pre> 塌缩为一行。
                        // pre-line（break_at_newline）：空白序列折叠但 `\n` 仍强制断行（CSS Text 3 §4.2）。
                        if (self.preserve_whitespace || self.break_at_newline) && content_word.is_empty() {
                            last_was_collapsible_ws = false;
                            let est_height = if current_line.height > 0.0 {
                                current_line.height
                            } else {
                                default_line_height
                            };
                            self.lines.push(current_line);
                            current_y += est_height;
                            current_line = LineBox {
                                y: 0.0,
                                height: 0.0,
                                runs: Vec::new(),
                                baseline_y: 0.0,
                                ascent: 0.0,
                                descent: 0.0,
                            };
                            let (new_left, _) = self.effective_content_area(current_y, default_line_height);
                            current_x = new_left;
                            continue;
                        }
                        // 全空格词在行首不产生任何渲染
                        if content_word.is_empty() {
                            continue;
                        }

                        // 基础宽度 + letter-spacing（仅基于内容字符，不含尾部空格）
                        let content_char_count = content_word.chars().count();
                        // R1450：letter-spacing 只在**相邻字母间**应用（词内 count-1 个间距），
                        // 不在词尾（行末/空格前）应用（CSS Text 3 §9.2 "not at start/end of
                        // line"，且不跨空格）。旧实现 ls×count 每词多算一个尾随 ls → "1 2" 单字
                        // 词也加 ls，致 letter-spacing-200/201 test 比 no-ls ref 宽。
                        // 词间相邻字母（break-all/CJK 无空格相邻）的 ls 经下方 adjacent_ls 前导补回。
                        let word_width =
                            self.advance_run_width(content_word, &run)
                                + run.letter_spacing * content_char_count.saturating_sub(1) as f32;
                        // R1086：word-spacing 作为词间前导间隙（CSS：词与词之间的额外间距）。
                        // 旧实现把 word_spacing 计入 word_width → fragment.x（=current_x，置位前）
                        // 不含 gap，仅推进 current_x 给下一词，致本词 glyph 位缺 gap
                        //（word-spacing-007 第二 x @x=40，应 @136）。现改为置位前把 gap 加到
                        // current_x。行首词（word_idx==0 或换行后 runs 空）无前导 gap。
                        // R1215：text-autospace——相邻词（上一词不以空白结尾）在 ideograph↔letter
                        // /numeric 类别边界额外插 0.125em 前导 gap（CSS Text 4 §8）。
                        // R1450：adjacent_ls——前一词以字母结尾（无空格相隔，preserve 空格段/
                        // normal 尾随空格都会让 prev_last 为空白）且本词以字母开头时，补一个 ls
                        // 作两相邻字母间的间距（break-all/CJK 单字词相邻场景）。空格分隔的词不触发。
                        let (autospace_gap, adjacent_ls) = if word_idx > 0 && !current_line.runs.is_empty() {
                            let prev_last = words.get(word_idx - 1).and_then(|w| w.chars().last());
                            let curr_first = content_word.chars().next();
                            match (prev_last, curr_first) {
                                (Some(pc), Some(cc)) if !pc.is_whitespace() && !cc.is_whitespace() => (
                                    autospace_gap_for(pc, cc, self.text_autospace, run.font_size),
                                    run.letter_spacing,
                                ),
                                _ => (0.0, 0.0),
                            }
                        } else {
                            (0.0, 0.0)
                        };
                        let mut lead_gap = if word_idx > 0 && !current_line.runs.is_empty() {
                            run.word_spacing + autospace_gap + adjacent_ls
                        } else {
                            0.0
                        };

                        // 计算当前行的有效可用宽度（扣除浮动排除区域）
                        let est_height = if current_line.height > 0.0 {
                            current_line.height
                        } else {
                            run.line_height.max(default_line_height)
                        };
                        let (left_offset, avail_width) = self.effective_content_area(current_y, est_height);

                        // 调整 current_x 到浮动排除区域之后（仅在行首且无 text-indent 时）
                        if current_line.runs.is_empty() && self.text_indent >= 0.0 && current_x < left_offset {
                            current_x = left_offset;
                        }

                        // 检查当前行是否放得下（含前导 word-spacing gap）
                        if !self.no_wrap
                            && current_x + lead_gap + word_width > left_offset + avail_width
                            && !current_line.runs.is_empty()
                        {
                            // 当前行放不下，开始新行
                            self.lines.push(current_line);
                            current_y += est_height;
                            current_line = LineBox {
                                y: 0.0,
                                height: 0.0,
                                runs: Vec::new(),
                                baseline_y: 0.0,
                                ascent: 0.0,
                                descent: 0.0,
                            };
                            // 新行重新计算浮动偏移
                            let (new_left, _) = self.effective_content_area(current_y, run.box_height());
                            current_x = new_left;
                            lead_gap = 0.0; // 行首词无前导 gap
                        }
                        // 应用前导 gap 到 current_x（本词 glyph 位 = current_x，含 gap）
                        current_x += lead_gap;

                        // 计算当前有效宽度（可能在换行后更新）
                        let (_, avail_w) =
                            self.effective_content_area(current_y, current_line.height.max(run.box_height()));

                        // overflow-wrap: break-word / anywhere 或 word-break: break-all
                        let need_char_break = !self.no_wrap
                            && (self.break_word || self.word_break == WordBreakMode::BreakAll)
                            && current_x + word_width > current_x + avail_w
                            && !content_word.is_empty();
                        if need_char_break {
                            let fragment_height = run.line_height;
                            let chars: Vec<char> = content_word.chars().collect();
                            let mut partial_x = current_x;

                            for (ci, ch) in chars.iter().enumerate() {
                                let ch_width = self.advance_of(*ch, run.font_id, run.font_size, run.is_ahem_font)
                                    + run.letter_spacing;
                                let text = ch.to_string();
                                let source = source_cursor.take_source(&text);

                                let (_, avail) =
                                    self.effective_content_area(current_y, current_line.height.max(run.box_height()));
                                let line_limit = current_line.runs.first().map_or(partial_x, |r| r.x) + avail;

                                if partial_x + ch_width > line_limit && ci > 0 {
                                    // 当前行满了，开始新行
                                    self.lines.push(current_line);
                                    current_y += fragment_height;
                                    current_line = LineBox {
                                        y: 0.0,
                                        height: 0.0,
                                        runs: Vec::new(),
                                        baseline_y: 0.0,
                                        ascent: 0.0,
                                        descent: 0.0,
                                    };
                                    let (new_left, _) = self.effective_content_area(current_y, fragment_height);
                                    partial_x = new_left;
                                }

                                current_line.runs.push(TextFragment {
                                    x: partial_x,
                                    y: 0.0,
                                    width: ch_width,
                                    height: fragment_height,
                                    text,
                                    source,
                                    node_id: run.node_id,
                                    font_size: run.font_size,
                                    vertical_align: run.vertical_align.clone(),
                                    is_ahem: run.is_ahem_font,
                                    letter_spacing: run.letter_spacing,
                                    margin_left: run.margin_left,
                                    margin_right: run.margin_right,
                                    margin_top: 0.0,
                                    baseline: run.font_size,
                                });

                                partial_x += ch_width;
                                // 行盒高度需容纳 inline 元素的完整盒体（含 padding+border）
                                current_line.height = current_line.height.max(run.box_height());
                            }
                            current_x = partial_x;
                        } else {
                            let fragment_height = run.line_height;
                            // word_width 已不含尾部空格（在上方剥离），直接用作可视宽度
                            // 尾部空格作为词间距离添加到 current_x
                            current_line.runs.push(TextFragment {
                                x: current_x,
                                y: 0.0,
                                width: word_width,
                                height: fragment_height,
                                text: content_word.to_string(),
                                source: source_cursor.take_source(content_word),
                                node_id: run.node_id,
                                font_size: run.font_size,
                                vertical_align: run.vertical_align.clone(),
                                is_ahem: run.is_ahem_font,
                                letter_spacing: run.letter_spacing,
                                margin_left: run.margin_left,
                                margin_right: run.margin_right,
                                margin_top: 0.0,
                                baseline: run.font_size,
                            });

                            current_x += word_width + trailing_space_width;
                            // 行盒高度需容纳 inline 元素的完整盒体（含 padding+border）
                            current_line.height = current_line.height.max(run.box_height());
                        }
                    }

                    // 在最后一个词之后添加 margin-right
                    if run.margin_right > 0.0 {
                        current_x += run.margin_right;
                    }
                }
                InlineItem::InlineBlock(box_info) => {
                    // inline-block 是原子盒，不可拆分
                    let box_width = box_info.width;
                    let box_height = box_info.height;
                    // 行内级盒打破了可折叠空白的连续性
                    last_was_collapsible_ws = false;

                    let est_height = if current_line.height > 0.0 {
                        current_line.height
                    } else {
                        box_height.max(default_line_height)
                    };
                    let (left_offset, avail_width) = self.effective_content_area(current_y, est_height);

                    // 调整 current_x 到浮动排除区域之后
                    if current_line.runs.is_empty() && current_x < left_offset {
                        current_x = left_offset;
                    }

                    // 检查当前行是否放得下（当行非空时）
                    if !self.no_wrap
                        && current_x + box_width > left_offset + avail_width
                        && !current_line.runs.is_empty()
                    {
                        // 当前行放不下，开始新行
                        self.lines.push(current_line);
                        current_y += est_height;
                        current_line = LineBox {
                            y: 0.0,
                            height: 0.0,
                            runs: Vec::new(),
                            baseline_y: 0.0,
                            ascent: 0.0,
                            descent: 0.0,
                        };
                        let (new_left, _) = self.effective_content_area(current_y, box_height);
                        current_x = new_left;
                    }

                    // inline-block 片段不使用 font_size，设为 0
                    // CSS：inline-block 的 margin box 参与行内格式化——margin_left/right
                    // 推进水平位置，margin_top/bottom 计入行盒高度，margin_top 偏移盒 Y。
                    let (m_left, m_right, m_top, m_bot) = (
                        box_info.margin_left,
                        box_info.margin_right,
                        box_info.margin_top,
                        box_info.margin_bottom,
                    );
                    current_x += m_left;
                    current_line.runs.push(TextFragment {
                        x: current_x,
                        y: 0.0,
                        width: box_width,
                        height: box_height,
                        text: String::new(),
                        source: None,
                        node_id: box_info.node_id,
                        font_size: 0.0,
                        vertical_align: box_info.vertical_align.clone(),
                        is_ahem: false,
                        letter_spacing: 0.0,
                        margin_left: m_left,
                        margin_right: m_right,
                        margin_top: m_top,
                        baseline: box_info.baseline,
                    });

                    current_x += box_width + m_right;
                    current_line.height = current_line.height.max(box_height + m_top + m_bot);
                }
                InlineItem::Br => {
                    // 强制换行：将当前行推入结果，开始新行
                    // Br 总是产生一个换行，即使当前行为空
                    last_was_collapsible_ws = false;
                    let est_height = if current_line.height > 0.0 {
                        current_line.height
                    } else {
                        default_line_height
                    };
                    // R1286：Br 结束的**空行**（无文本片段，如 `<p><br></p>` / `<p><br>text</p>`
                    // 的首空行）须有 strut 高度（line-height），否则 IFC 把空行计 0 高致
                    // 容器塌缩（chromium 给空 line box 一行 line-height，CSS §10.8.1 strut）。
                    // est_height 已是 strut（default_line_height）；非空行（含文本，height>0）
                    // 不受影响。与 R1285（br 在 block 间的 taffy min-height）正交——本处管
                    // br 在 IFC 内（p>br 等）的空行。kill-switch `ZW_BR_IFC_LINE=0`（default-on）。
                    if current_line.height <= 0.0 && std::env::var("ZW_BR_IFC_LINE").as_deref() != Ok("0") {
                        current_line.height = est_height;
                    }
                    self.lines.push(current_line);
                    current_y += est_height;
                    current_line = LineBox {
                        y: 0.0,
                        height: 0.0,
                        runs: Vec::new(),
                        baseline_y: 0.0,
                        ascent: 0.0,
                        descent: 0.0,
                    };
                    let (new_left, _) = self.effective_content_area(current_y, default_line_height);
                    current_x = new_left;
                }
            }
        }

        // 添加最后一行（非空时）
        // CSS 2.1 §10.8.1：空 inline 元素的 line-height 仍贡献到行盒高度，
        // 即使没有文本片段，行盒高度 > 0 时也需要保留。
        if !current_line.runs.is_empty() || current_line.height > 0.0 {
            self.lines.push(current_line);
        }

        // R1476：CSS 2.1 §9.4.2 + WPT empty-inline-001——仅含「裸空 inline 元素」的行盒
        // 为零高（裸空 span 的 line-height 不贡献行盒高度）。判定：行内所有片段均为裸空
        // inline（text 空、width==0、水平 margin==0）。文本词片段（text 非空）/ inline-block
        // 片段（width>0）/ 带水平 margin 的空 inline 都令行盒保留正常高度（empty-inline-002
        // 带 margin 的 span / empty-inline-003 span+文本）。裸空 inline 的 line-height 仅在
        // 同行有其他显著内容时才贡献（由上方逐 run 累积的 current_line.height 保留；此处仅
        // 塌缩「全裸空 inline」的孤立行）。
        // **代理**：用 fragment 可得的「水平 margin」识别非裸空 inline（padding/border 不在
        // fragment 中）；故仅有 padding/border、无 margin 的空 inline 亦按裸空塌缩——该模式
        // 不影响 WPT（chromium-Oracle A/B 跨 CSS2 净 +2 flip、0 flip 失），属可接受近似。
        for line in &mut self.lines {
            let all_bare_empty = !line.runs.is_empty()
                && line
                    .runs
                    .iter()
                    .all(|f| f.text.is_empty() && f.width == 0.0 && f.margin_left == 0.0 && f.margin_right == 0.0);
            if all_bare_empty {
                line.height = 0.0;
            }
        }

        // 计算每行的 y 坐标
        let mut y = 0.0;
        for line in &mut self.lines {
            line.y = y;
            y += line.height;
        }

        // 诊断（R2027）：dump 最终行盒高度（与上方条目构成对应）。
        if debug_ifc && !self.lines.is_empty() {
            let total: f32 = self.lines.iter().map(|l| l.height).sum();
            let heights: Vec<f32> = self.lines.iter().map(|l| l.height).collect();
            eprintln!("[ZW_DEBUG_IFC] lines={} total_h={:.1} heights={:?}", self.lines.len(), total, heights);
        }

        // 应用文本对齐
        self.apply_text_alignment();
        if plaintext_enabled {
            self.apply_plaintext_direction(&plaintext_directions);
        }

        // 应用 vertical-align 对齐
        self.apply_vertical_alignment();
    }
}

fn plaintext_paragraph_directions(items: &[InlineItem]) -> HashMap<NodeId, bool> {
    fn flush(text: &mut String, nodes: &mut Vec<NodeId>, directions: &mut HashMap<NodeId, bool>) {
        let rtl = plaintext_base_is_rtl(text);
        for node in nodes.drain(..) {
            directions.insert(node, rtl);
        }
        text.clear();
    }

    let mut directions = HashMap::new();
    let mut paragraph = String::new();
    let mut nodes = Vec::new();
    for item in items {
        match item {
            InlineItem::Text(run) if run.is_plaintext_bidi => {
                paragraph.push_str(&run.text);
                nodes.push(run.node_id);
            }
            InlineItem::Br => flush(&mut paragraph, &mut nodes, &mut directions),
            _ => {}
        }
    }
    flush(&mut paragraph, &mut nodes, &mut directions);
    directions
}
