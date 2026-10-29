// text-wrap: balance 断点规划器（R5325）— 纯函数规划面（同 include! 模式并入本模块）
//
// css-text-4 §text-wrap-balance：「the used line-breaking opportunities are chosen
// to minimize the variation in line length, without changing the number of lines」。
// 算法 UA 自定义（spec 不锁定）；本实现 = 贪心行数 N 内做 min-Σ(rendered²) DP 词界
// 重排，仅在 **scope 门内**的纯文本段生效：
//   - IFC 级：无浮动排除带（行带宽恒 = container_width）、无 text-indent、容器
//     white-space normal、word-break normal、非 break-word、items 仅 Text/Br；
//   - run 级：无水平 margin/padding advance、无 bidi（rtl/override/plaintext）、
//     ws_override 为 None 或普通折叠（非 preserve/nonewrap/break_at_newline）；
//   - 词界级：相邻词对必须空白分隔（词尾空格 / 纯空白 run / run 前导空格）——
//     跨 run 粘连（R5161 GL 面）、ZWSP、CJK per-char 词界一律回退贪心。
// 规划结果 = (item_idx, word_idx) 断点集合，由 break_items_into_lines 词臂在贪心
// fit 门之前消费（行非空时落成推行）；贪心与 DP 共享同一 span 模拟（advance 公式
// 与词臂逐项同式），贪心已是参差最优时零改动（N=1 / cost 非严格更优）。
// https://drafts.csswg.org/css-text-4/#text-wrap-balance

/// balance 断点计划：(item_idx, run 内 word_idx) —— 在该词**之前**推行（行非空时）。
pub(super) type BalanceBreakPlan = std::collections::HashSet<(usize, usize)>;

/// 段内词条槽（词级扁平化，宽度与词臂同源）。
struct BalanceWordSlot {
    item_idx: usize,
    word_idx: usize,
    /// 内容词 advance（advance_run_width + ls×count，词臂同式）。
    w: f32,
    /// 词尾可折叠空格 advance（随词推进、行尾不渲染，词臂剥离式同源）。
    space_after: f32,
    /// run 首词且 run 文本以可折叠空白开头时的前导空格 advance（仅非行首施加）。
    lead_space: f32,
}

/// 段内槽：词 / 纯空白 run（words 为空且 text 非空 → R5323 同源单空格 advance）。
enum BalanceSegSlot {
    Word(BalanceWordSlot),
    Space(f32),
}

/// 一次行内推进模拟结果：cursor = 含尾随空白的推进量（fit 判据口径），
/// rendered = 去掉行尾未渲染尾随空白后的可视宽度（DP 代价口径），fits = 是否放得下。
struct BalanceSpan {
    #[allow(dead_code)]
    cursor: f32,
    rendered: f32,
    fits: bool,
}

impl InlineFormattingContext {
    /// R5325：balance 断点规划入口——返回 Some(plan) 时由断行循环消费。
    ///
    /// scope 门整体不满足 / 无任何段产出严格更优 DP 解时返回 None（零扰动）。
    pub(super) fn balance_line_break_plan(&self, items: &[InlineItem]) -> Option<BalanceBreakPlan> {
        if !self.balance || !runtime_flags::text_wrap_balance() {
            return None;
        }
        // IFC 级 scope：恒定行带 + 标准 white-space/断词环境（其余环境维持贪心，
        // 与「balance 不改变行数、不影响 min/max-content」的 spec 面一致）。
        if !self.float_exclusions.is_empty()
            || self.text_indent != 0.0
            || self.no_wrap
            || self.preserve_whitespace
            || self.break_at_newline
            || self.word_break != WordBreakMode::Normal
            || self.break_word
        {
            return None;
        }
        if items.iter().any(|i| !matches!(i, InlineItem::Text(_) | InlineItem::Br)) {
            return None;
        }

        let mut plan = BalanceBreakPlan::new();
        let mut changed = false;
        let mut seg_start = 0usize;
        for (idx, item) in items.iter().enumerate() {
            // 强制断行分组：每个 Br 分隔的段独立均衡（balance-004「Groups of lines
            // separated by a forced line break are processed separately」）。
            if matches!(item, InlineItem::Br) {
                changed |= self.balance_plan_segment(&items[seg_start..idx], seg_start, &mut plan);
                seg_start = idx + 1;
            }
        }
        changed |= self.balance_plan_segment(&items[seg_start..], seg_start, &mut plan);
        if changed { Some(plan) } else { None }
    }

    /// 规划单个 Br 分隔段：贪心行数 N ≥ 2 且 DP min-Σrendered² 严格更优时落计划。
    /// `item_base` = 段首在全 items 向量中的下标（plan 键须与断行循环的 item_idx 同轴）。
    fn balance_plan_segment(&self, items: &[InlineItem], item_base: usize, plan: &mut BalanceBreakPlan) -> bool {
        let slots = match self.balance_flatten_segment(items, item_base) {
            Some(slots) => slots,
            None => return false,
        };
        if slots.len() < 2 {
            return false;
        }
        let avail = self.container_width;
        let greedy = balance_greedy_rendered(&slots, avail);
        if greedy.len() < 2 {
            return false;
        }
        let greedy_cost: f64 = greedy.iter().map(|w| f64::from(*w) * f64::from(*w)).sum();
        let best = match balance_dp_min_raggedness(&slots, avail, greedy.len()) {
            Some(best) => best,
            None => return false,
        };
        // 非严格更优不改断点（贪心已是参差最优时保持字节级旧行为——green 面零扰动）。
        if best.cost + 0.01 >= greedy_cost {
            return false;
        }
        for &slot_pos in &best.line_starts[1..] {
            let slot = match &slots[slot_pos] {
                BalanceSegSlot::Word(w) => w,
                BalanceSegSlot::Space(_) => continue,
            };
            plan.insert((slot.item_idx, slot.word_idx));
        }
        true
    }

    /// 段内词级扁平化（宽度与 break_items_into_lines 词臂逐项同式）。
    /// `item_base` 加进槽键，使 plan 键与断行循环的全向量 item_idx 同轴。
    /// 段内出现规划面之外的形态（run 盒几何 / bidi / 粘连词界）返回 None 回退贪心。
    fn balance_flatten_segment(&self, items: &[InlineItem], item_base: usize) -> Option<Vec<BalanceSegSlot>> {
        let mut slots: Vec<BalanceSegSlot> = Vec::new();
        for (item_idx, item) in items.iter().enumerate() {
            let run = match item {
                InlineItem::Text(run) => run,
                _ => return None,
            };
            // run 级 scope：无水平盒几何 advance、无 bidi、普通折叠 white-space
            //（ws_override 三态 None/遮蔽均回退容器级 normal，见下方守卫）。
            if run.margin_left != 0.0
                || run.margin_right != 0.0
                || run.padding_left != 0.0
                || run.padding_right != 0.0
                || run.is_rtl
                || run.bidi_override.is_some()
                || run.is_plaintext_bidi
            {
                return None;
            }
            let (run_preserve, run_break_at_newline, run_no_wrap) = match run.ws_override {
                Some(ws) => (ws.preserve, ws.break_at_newline, ws.no_wrap),
                None => (self.preserve_whitespace, self.break_at_newline, self.no_wrap),
            };
            if run_preserve || run_break_at_newline || run_no_wrap {
                return None;
            }
            let space_advance = self.advance_of(' ', run.font_id, run.font_size, run.is_ahem_font)
                + run.letter_spacing
                + run.word_spacing;
            // run 首前导可折叠空格 advance（词臂 :331 同式；行首/上一序列收尾时吞并）。
            let lead_space = if run.text.chars().next().is_some_and(is_collapsible_ws) {
                space_advance
            } else {
                0.0
            };
            let words = self.split_into_words_with_ws(&run.text, run.is_ahem_font, run_preserve, run_break_at_newline);
            if words.is_empty() {
                if run.text.is_empty() {
                    // 空 inline：零 advance（几何不参与断行），仅复位折叠连续性语义。
                    continue;
                }
                // 纯空白 run：折叠为单空格 advance（词臂 words.is_empty() 臂同式）。
                slots.push(BalanceSegSlot::Space(space_advance));
                continue;
            }
            for (word_idx, word) in words.iter().enumerate() {
                // 词尾可折叠空格剥离（词臂同式：advance_of×count + ls×count + ws×count）。
                let (content, space_count) = if word.ends_with(' ') {
                    let trimmed = word.trim_end_matches(' ');
                    (trimmed, word.len() - trimmed.len())
                } else {
                    (word.as_str(), 0usize)
                };
                if content.is_empty() {
                    // 全空格词：词臂在 fit 门之前 continue，零贡献 → 无断点槽。
                    continue;
                }
                slots.push(BalanceSegSlot::Word(BalanceWordSlot {
                    item_idx: item_base + item_idx,
                    word_idx,
                    w: self.advance_run_width(content, run) + run.letter_spacing * content.chars().count() as f32,
                    space_after: if space_count > 0 {
                        self.advance_of(' ', run.font_id, run.font_size, run.is_ahem_font) * space_count as f32
                            + run.letter_spacing * space_count as f32
                            + run.word_spacing * space_count as f32
                    } else {
                        0.0
                    },
                    lead_space: if word_idx == 0 { lead_space } else { 0.0 },
                }));
            }
        }
        // 词界合法性扫描：相邻词对必须空白分隔（词尾空格 / 纯空白 run / run 前导
        // 空格）。粘连对（跨 run 无空白拼接、ZWSP 丢字符合并、CJK per-char）不是
        // 本规划面的合法软换行点 → 整段回退贪心（R5161 GL 面不越界）。
        let mut prev: Option<&BalanceWordSlot> = None;
        for slot in &slots {
            match slot {
                BalanceSegSlot::Space(_) => prev = None,
                BalanceSegSlot::Word(w) => {
                    if let Some(p) = prev
                        && p.space_after <= 0.0
                        && w.lead_space <= 0.0
                    {
                        return None;
                    }
                    prev = Some(w);
                }
            }
        }
        Some(slots)
    }
}

/// 单行推进模拟（scope 门内与词臂逐项同式）：
/// - fit 判据：非行首词 `cursor + lead + word_w > avail` 即放不下（词臂 fit 门同式；
///   行首词不判 fit，溢出强放）；
/// - lead 门：行首 / 上一词尾可折叠空白（R5297 吞并）/ 上一槽纯空白（行首序列折叠）
///   时前导空格 advance 不施加；
/// - rendered：cursor 扣除行尾未渲染的尾随空白 advance（词尾空格 / 纯空白 run 行尾
///   不产 fragment）。
fn balance_line_span(slots: &[BalanceSegSlot], avail: f32) -> BalanceSpan {
    let mut x = 0.0_f32;
    let mut trailing = 0.0_f32;
    let mut first = true;
    let mut ended_ws = false;
    let mut was_ws = false;
    for slot in slots {
        match slot {
            BalanceSegSlot::Word(w) => {
                let lead = if first || ended_ws || was_ws { 0.0 } else { w.lead_space };
                if !first && x + lead + w.w > avail {
                    return BalanceSpan {
                        cursor: x,
                        rendered: x - trailing,
                        fits: false,
                    };
                }
                x += lead + w.w + w.space_after;
                trailing = w.space_after;
                ended_ws = w.space_after > 0.0;
                was_ws = false;
                first = false;
            }
            BalanceSegSlot::Space(v) => {
                if !first && !was_ws && !(runtime_flags::ws_seq_pure_run_collapse() && ended_ws) {
                    x += v;
                    trailing = *v;
                }
                was_ws = true;
                ended_ws = true;
            }
        }
    }
    BalanceSpan {
        cursor: x,
        rendered: x - trailing,
        fits: true,
    }
}

/// 贪心断行（与词臂 fit 门同判据）：返回各行 rendered 宽度。
fn balance_greedy_rendered(slots: &[BalanceSegSlot], avail: f32) -> Vec<f32> {
    let n = slots.len();
    let mut widths = Vec::new();
    let mut i = 0usize;
    while i < n {
        let mut e = i + 1;
        while e < n {
            if !balance_line_span(&slots[i..e + 1], avail).fits {
                break;
            }
            e += 1;
        }
        widths.push(balance_line_span(&slots[i..e], avail).rendered);
        i = e;
    }
    widths
}

struct BalanceDpResult {
    /// 各行首槽位（首元素恒 0；长度 = 行数）。
    line_starts: Vec<usize>,
    cost: f64,
}

/// 固定 N 行的 min-Σ(rendered²) DP（贪心 split 恒可行 → 必有解）。
fn balance_dp_min_raggedness(slots: &[BalanceSegSlot], avail: f32, lines: usize) -> Option<BalanceDpResult> {
    const INF: f64 = f64::INFINITY;
    let n = slots.len();
    let mut dp = vec![vec![INF; n + 1]; lines + 1];
    let mut parent = vec![vec![usize::MAX; n + 1]; lines + 1];
    dp[0][0] = 0.0;
    for k in 0..lines {
        for j in 0..n {
            if dp[k][j].is_infinite() {
                continue;
            }
            for e in (j + 1)..=n {
                let span = balance_line_span(&slots[j..e], avail);
                // 行内溢出对行尾扩展单调（更长的行只会更宽）→ 直接终止内层。
                if !span.fits {
                    break;
                }
                let cost = dp[k][j] + f64::from(span.rendered) * f64::from(span.rendered);
                if cost < dp[k + 1][e] {
                    dp[k + 1][e] = cost;
                    parent[k + 1][e] = j;
                }
            }
        }
    }
    if dp[lines][n].is_infinite() {
        return None;
    }
    let mut line_starts = vec![n; lines];
    let mut pos = n;
    for k in (1..=lines).rev() {
        pos = parent[k][pos];
        if pos == usize::MAX {
            return None;
        }
        line_starts[k - 1] = pos;
    }
    line_starts[0] = 0;
    Some(BalanceDpResult {
        line_starts,
        cost: dp[lines][n],
    })
}
