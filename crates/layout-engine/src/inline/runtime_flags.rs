//! Process-lifetime IFC feature switches with a live rollback path.

use std::sync::LazyLock;

static SNAPSHOT_ENABLED: LazyLock<bool> = LazyLock::new(|| default_on("ZW_IFC_ENV_SNAPSHOT"));
static RESIDUAL_SNAPSHOT_ENABLED: LazyLock<bool> = LazyLock::new(|| default_on("ZW_LAYOUT_RESIDUAL_ENV_SNAPSHOT"));

pub(super) fn plaintext_line_direction() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_PLAINTEXT_LINE_DIRECTION"));
    selected(*VALUE, || default_on("ZW_PLAINTEXT_LINE_DIRECTION"))
}

/// R5087（CSS2 §10.8.1 leading）：strut half-leading 允许为负（line-height < 字体
/// content 高时字形越出行盒顶 = bleed）。default-on，`ZW_STRUT_NEG_LEADING=0` 回退钳制。
pub(super) fn strut_neg_leading() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_STRUT_NEG_LEADING"));
    selected(*VALUE, || default_on("ZW_STRUT_NEG_LEADING"))
}

pub(super) fn debug_ifc() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| std::env::var("ZW_DEBUG_IFC").is_ok());
    selected(*VALUE, || std::env::var("ZW_DEBUG_IFC").is_ok())
}

pub(super) fn br_ifc_line() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_BR_IFC_LINE"));
    selected(*VALUE, || default_on("ZW_BR_IFC_LINE"))
}

/// R5322（CSS Text 3 §5.2 / CSS2 §10.8.1）：pre/pre-line 的 `\n` 强制断行空行
/// 写回 strut 行盒 height（与 R1286 `<br>` 空行同构），空行 strut 取容器 used
/// line-height（R5298 捕获）而非硬编码 20px。default-on；
/// `ZW_PRE_NEWLINE_STRUT=0` 回退旧行为（不写 height + 20px）。
pub(super) fn pre_newline_strut() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_PRE_NEWLINE_STRUT"));
    selected(*VALUE, || default_on("ZW_PRE_NEWLINE_STRUT"))
}

pub(super) fn skip_oof() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_IFC_SKIP_OOF"));
    selected(*VALUE, || default_on("ZW_IFC_SKIP_OOF"))
}

pub(super) fn line_clamp() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_LINE_CLAMP"));
    selected(*VALUE, || default_on("ZW_LINE_CLAMP"))
}

/// R3779b：float 子在 IFC 内不发占位 Br（无 R1286 幽灵空行）。default-on；
/// `ZW_FLOAT_NO_GHOST_LINE=0` 回退旧行为（float → Br + 行首空行 strut）。
pub(super) fn float_no_ghost_line() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_FLOAT_NO_GHOST_LINE"));
    selected(*VALUE, || default_on("ZW_FLOAT_NO_GHOST_LINE"))
}

pub(super) fn inline_box_recurse() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_INLINE_BOX_RECURSE"));
    selected(*VALUE, || default_on("ZW_INLINE_BOX_RECURSE"))
}

/// R5235：R1576 递归臂触发集信号（paint Path B 空 styles 镜像分叉）。default-on；
/// `ZW_NESTED_ATOMIC_SIGNAL=0` 回退旧行为（记录与消费同关）。
pub(super) fn nested_atomic_signal() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_NESTED_ATOMIC_SIGNAL"));
    selected(*VALUE, || default_on("ZW_NESTED_ATOMIC_SIGNAL"))
}

pub(super) fn prewrap_hang() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_PREWRAP_HANG"));
    selected(*VALUE, || default_on("ZW_PREWRAP_HANG"))
}

/// R4345：CJK 逐字符断词**连续模式** default-on——per-CJK-char 词不再追加尾部空格
/// （旧 opt-in 模式下每 CJK 字符隐含 0.25em 词间 advance（16px 字号 = 4px），虚增行宽
/// 致 line-break 族 18 案在 10.2em 容器 8 字即断，chromium 10 字收敛——css-text-3
/// 「CJK 字符间无词间距」语义）。旧 blocker（park 时「全局启用 welcome +6.7pp」）
/// 已失效：ZRG-2026-08-15 advance 同源（generic→hmtx / 显式 face→shaping）后
/// welcome 20.30%→20.00% 反而改善。`ZW_CJK_CONTIGUOUS=0` 回退旧尾空格模式。
pub(super) fn cjk_contiguous() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_CJK_CONTIGUOUS"));
    selected(*VALUE, || default_on("ZW_CJK_CONTIGUOUS"))
}

/// R5161（css-text-3 §5.2 / UAX#14）：跨 run 词中拼接边界无软换行机会——
/// `PASS<span>FAIL</span>` 的 S|F 边界（无空白相邻的字母对）不构成断点，FAIL 随前词
/// 横向溢出而非折行（旧实现逐 run 独立断行，4ch 容器内 FAIL 误折第 2 行红底外露，
/// contain-paint-047 族）。default-on，`ZW_CROSS_RUN_GLUE=0` 回退逐 run 独立断行。
pub(super) fn cross_run_glue() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_CROSS_RUN_GLUE"));
    selected(*VALUE, || default_on("ZW_CROSS_RUN_GLUE"))
}

/// R5165（css-text-3 §5.5 line-break-details + UAX #14 GL/WJ/ZWJ）：原子 inline 与
/// 相邻 GL/WJ/ZWJ 类字符（U+00A0 web-compat 例外除外）间**无软换行机会**——前邻 GL
/// 的原子盒不因 fit 门折入新行、后邻 GL 的首词不脱离原子折行（spec 语义为横向溢出，
/// chromium 同；line-breaking-atomic-003/004/018/019 族：NNBSP/FIGURE SPACE 邻原子
/// 被误折第 2 行，透明 span 露红底 2.0833%）。GL 身份经 `InlineBlockBox::prev_char_gl`
/// /`next_char_gl` 旁带（§3 折叠吸收后断行层不可见，collect 期在原始 DOM 文本上判定）。
/// default-on，`ZW_ATOMIC_GL_NO_BREAK=0` 回退无条件原子 fit 折行。
pub(super) fn atomic_gl_no_break() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_ATOMIC_GL_NO_BREAK"));
    selected(*VALUE, || default_on("ZW_ATOMIC_GL_NO_BREAK"))
}

pub(super) fn bidi_fragment_source() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_BIDI_FRAGMENT_SOURCE"));
    selected(*VALUE, || default_on("ZW_BIDI_FRAGMENT_SOURCE"))
}

/// R5246：U+2028 LS / U+2029 PS 进 white-space 强制断行字符类（pre/pre-line 下与
/// `\n` 同为强制断行点，本体不渲染；css-writing-modes-3 §2.2 / css-text-3 §5.2）。
/// default-on，`ZW_FORCED_BREAK_LS_PS=0` 回退既有折叠为空格行为。
pub(super) fn forced_break_ls_ps_enabled() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_FORCED_BREAK_LS_PS"));
    selected(*VALUE, || default_on("ZW_FORCED_BREAK_LS_PS"))
}

/// R5249：element-first 首槽 spacer——walk 站位上首内容为元素子时，外层自身
/// ml+pl（R4332 折入含 border advance）以空文本零宽 run（归因外层自身）落首槽，
/// fold 仍落自名 run、盒 x 主臂检测式不变（position-relative-033-ref：外 span
/// 首内容 color 分歧内层 span，fold 整体丢失内容 −2px 实证）。CSS2 §10.2/§8.4：
/// inline 水平 margin/padding 参与推进，不因首内容是元素子而失效。
/// default-on，`ZW_INLINE_ELEMENT_FIRST_SPACER=0` 回退首槽被元素 run 消耗行为。
pub(super) fn element_first_spacer() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_INLINE_ELEMENT_FIRST_SPACER"));
    selected(*VALUE, || default_on("ZW_INLINE_ELEMENT_FIRST_SPACER"))
}

pub(super) fn bidi_mirroring() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_BIDI_MIRRORING"));
    selected(*VALUE, || default_on("ZW_BIDI_MIRRORING"))
}

pub(super) fn bidi_override() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_BIDI_OVERRIDE"));
    selected(*VALUE, || default_on("ZW_BIDI_OVERRIDE"))
}

pub(super) fn font_size_adjust_normal_line() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_FONT_SIZE_ADJUST_NORMAL_LINE"));
    residual_selected(*VALUE, || default_on("ZW_FONT_SIZE_ADJUST_NORMAL_LINE"))
}

pub(super) fn font_face_size_adjust_normal_line() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_FONT_FACE_SIZE_ADJUST_NORMAL_LINE"));
    residual_selected(*VALUE, || default_on("ZW_FONT_FACE_SIZE_ADJUST_NORMAL_LINE"))
}

pub(super) fn content_visibility() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_CONTENT_VISIBILITY"));
    residual_selected(*VALUE, || default_on("ZW_CONTENT_VISIBILITY"))
}

pub(super) fn shaped_advance_trace() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| opt_in("ZW_SHAPED_ADVANCE_TRACE"));
    residual_selected(*VALUE, || opt_in("ZW_SHAPED_ADVANCE_TRACE"))
}

pub(super) fn shaped_fallback() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| opt_in("ZW_SHAPED_FALLBACK"));
    residual_selected(*VALUE, || opt_in("ZW_SHAPED_FALLBACK"))
}

/// R4374/R4376：CJK 回退字体垂直度量参与行盒——run 各字符实际使用字体（per-char
/// 回退链）的 ascent/descent max 撑开行盒（chromium CSS2 §10.8.1 各 inline box 按
/// 自身字体度量分布；NotoSansCJK hhea 1.448em > 主字体 strut 1.164em）。
/// **R4376 默认 on**：oracle +12/+13 且 0 目录下滑、product-smoke welcome
/// 19.95%→15.40%、corpus −6（−9/+3，counter-styles 墙 20 案已由 R4375 marker 基线
/// 同步收复）；`ZW_FALLBACK_LINE_METRICS=0` 回退旧行为。
pub(super) fn fallback_line_metrics() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_FALLBACK_LINE_METRICS"));
    residual_selected(*VALUE, || default_on("ZW_FALLBACK_LINE_METRICS"))
}

/// R5147：per-font 行度量整数化（Blink `SimpleFontData` 对 ascent/descent 逐项取整，
/// `line-height:normal` 与回退链行盒贡献恒整数——chromium 实测 quotes-025 CJK 行
/// 46.336→46、p3 y 179.336→179 逐像素一致）。**R5147 默认 on**：corpus 1,159→1,156
/// 翻绿 3（quotes-025/026/027 全 0.00%）+ strict 口径 +29 案、零新增红零 churn；
/// `ZW_INT_FONT_METRICS=0` 回退原始分数度量。
///
/// R5147 同轮扩展（同旗标）：常数比率 `line-height:normal` 路径（`fs × 1.164`，
/// provider 未命中臂）floor 整数化纳入——16px serif 18.624→18 = chromium 实测 p 高
/// （fill-001 div top 分数相位消除，0.00% 翻绿）。corpus 1,156→1,146（+14/−4）、
/// strict 口径 9,222→9,760、product-smoke welcome 17.43→17.07%（vs 关闭 18.17%；
/// 15.35→18.17 台阶 = welcome.html +42 行 × 陈旧 oracle，ZRG-2026-10-16-01 定谳
/// 非渲染回归，oracle 重录待用户放行）。round 变体 corpus/welcome
/// 双劣已 refute（1,148 / 17.63%）。clamp 行数估记 `container_used_line_height_px`
/// **不**整数化——分数 18.624 与 ref 自洽（auto-039/040/043/044 实证，整数化 +4 红）。
pub(super) fn int_font_metrics() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_INT_FONT_METRICS"));
    selected(*VALUE, || default_on("ZW_INT_FONT_METRICS"))
}

fn default_on(name: &str) -> bool {
    std::env::var(name).as_deref() != Ok("0")
}

fn opt_in(name: &str) -> bool {
    std::env::var(name).as_deref() == Ok("1")
}

#[inline]
fn selected(cached: bool, live: impl FnOnce() -> bool) -> bool {
    select_value(*SNAPSHOT_ENABLED, cached, live)
}

#[inline]
fn residual_selected(cached: bool, live: impl FnOnce() -> bool) -> bool {
    select_value(*RESIDUAL_SNAPSHOT_ENABLED, cached, live)
}

#[inline]
fn select_value(snapshot_enabled: bool, cached: bool, live: impl FnOnce() -> bool) -> bool {
    if snapshot_enabled { cached } else { live() }
}

/// R5068（CSS 2.1 §8.3）：inline 水平 margin advance 无符号——负 margin 同效推进。
/// default-on；`ZW_INLINE_NEG_MARGIN=0` 回退旧 `> 0.0` 门（负 margin 静默丢弃）。
pub(super) fn inline_neg_margin_on() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_INLINE_NEG_MARGIN"));
    selected(*VALUE, || default_on("ZW_INLINE_NEG_MARGIN"))
}

/// R5121（css-writing-modes-3 §2.4 bidi box model slice·phase 1）：段落级 UBA——
/// 控制符嵌入/覆盖作用域跨 run 边界 + level 边界 run 裂解 + 行级 L2 组序重排。
/// **opt-in 默认关**（`ZW_PARA_BIDI=1` 启用）——dormant 同 R5120
/// `ZW_INLINE_SINGLE_FRAG_BOX_BORDER`；翻默认前置 = 旗标开全量 A/B 零新红 +
/// per-fragment 完整 level 栈（parity 近似收口）。
pub(super) fn para_bidi() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| opt_in("ZW_PARA_BIDI"));
    selected(*VALUE, || opt_in("ZW_PARA_BIDI"))
}

/// R5166（css-position-3 §static-position + css-text-3 line-breaking-019）：行内流 OOF
/// 元素发 `OofAnchor` 条目，断行层记录行内流静态位供 abspos 后处理锚定。default-on；
/// `ZW_OOF_INLINE_ANCHOR=0` 回退 collect 期静默跳过（旧行为：taffy block-flow cursor
/// 静态位）。
pub(super) fn oof_inline_anchor() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_OOF_INLINE_ANCHOR"));
    selected(*VALUE, || default_on("ZW_OOF_INLINE_ANCHOR"))
}

/// R5216（CSS2 §9.5 rule 8「shortened line box」）：原子盒 fit 门断行产生的**新空行**
/// 同样按 R4234 下绕 float——断行只保证原子盒离开前行 y 带，新行 y 带仍可能被 float
/// 缩到放不下（floats-wrap-top-below-inline-002l/r：行 2 带 50..100 被 float 左缘/
/// 右缘夹击 avail=0，span2 滞留前行右侧 (150,50)/(300,50) 与 float 重叠；chromium
/// 下绕到 float 底 y=150 行首 (0,150)）。default-on；
/// `ZW_ATOMIC_POSTBREAK_FLOAT_FIT=0` 回退仅入口空行下绕的旧行为。
pub(super) fn atomic_postbreak_float_fit() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_ATOMIC_POSTBREAK_FLOAT_FIT"));
    selected(*VALUE, || default_on("ZW_ATOMIC_POSTBREAK_FLOAT_FIT"))
}

/// R5240（CSS2 §9.5 rule 8「shortened line box」）：行首**词**放不下 float 缩短后的
/// 剩余带宽（0 < avail < 词宽，R3785 只盖 avail≤0）时同 R4234 行首原子盒下绕——
/// 行位下移到最早结束的重叠 float 带底重算，直到放得下或不再与 float 重叠
///（floats-132：160px 词在 float 缩短的 140px 首行带强放溢出，满宽第二行本可容纳，
/// 上行溢出+下行红露）。default-on；`ZW_FIRST_WORD_FLOAT_FIT=0` 回退行首词原地强放。
pub(super) fn first_word_float_fit() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_FIRST_WORD_FLOAT_FIT"));
    selected(*VALUE, || default_on("ZW_FIRST_WORD_FLOAT_FIT"))
}

/// R5316（CSS2 §10.8.1 va:text-* × 负 leading）：text-top/text-bottom 对齐参照 =
/// **父 content area**（字体度量、基线相对）——负 leading（line-height < A+D）时参照
/// 位越出行盒，half_leading 扩展下限被钳零、行盒不生长
///（vertical-align-negative-leading-001：chromium 行高 30/20/20，ZW 恒 10）。
/// 基线锚定需求与既有 half_leading 校准值取 max（正 leading 域零扰动）。
/// default-on；`ZW_VA_TEXT_BASELINE_PIN=0` 回退纯 half_leading 扩展（旧行为）。
pub(super) fn va_text_baseline_pin() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_VA_TEXT_BASELINE_PIN"));
    selected(*VALUE, || default_on("ZW_VA_TEXT_BASELINE_PIN"))
}

/// R5317（CSS2 §10.3.2 replaced 元素行内参与 + §10.8.1 行盒围合）：img 原子级行内盒
/// 尺寸 = **border box**——行内 advance/行盒围合按 margin box（= ml + border box + mr）
/// 参与（chromium 同）。旧实现推 content box（CSS width/height），padding+border 不参与
/// advance/围合：c44-ln-box-001 img1 后文本 x−40（advance 丢 padding+border 40px）、
/// 行盒度量多 build 分裂（layout 树 border box 60 vs 存面/测量 CSS content 30 →
/// 行高 170 vs 110）。default-on；`ZW_IMG_ATOM_BORDER_BOX=0` 回退 content box 口径。
pub(super) fn img_atom_border_box() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_IMG_ATOM_BORDER_BOX"));
    selected(*VALUE, || default_on("ZW_IMG_ATOM_BORDER_BOX"))
}

/// R5323（css-text-3 §4.1 phase-1 collapse 跨 inline 边界）：纯空白 run 的空格与
/// 上一内容词尾随空格同属一个空白序列时只保留前者 advance——R5297 词臂前导空格
/// 门（`!last_ended_collapsible_ws`）的对偶补全，本臂当时按「纯空白 run 路径不变」
/// 收窄未覆盖。`<span>X </span> <span>X </span>` 旧实现词间双计 2 空格
///（white-space-007 每行 4 X 共 3 行 vs chromium 折叠单空格 5 X 共 2 行）。
/// default-on；`ZW_WS_SEQ_PURE_RUN_COLLAPSE=0` 回退双 advance（旧行为）。
pub(super) fn ws_seq_pure_run_collapse() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_WS_SEQ_PURE_RUN_COLLAPSE"));
    selected(*VALUE, || default_on("ZW_WS_SEQ_PURE_RUN_COLLAPSE"))
}

#[cfg(test)]
mod tests {
    use super::select_value;

    #[test]
    fn snapshot_skips_live_lookup_and_fallback_uses_it() {
        let calls = std::cell::Cell::new(0);
        assert!(select_value(true, true, || {
            calls.set(calls.get() + 1);
            false
        }));
        assert_eq!(calls.get(), 0);

        assert!(!select_value(false, true, || {
            calls.set(calls.get() + 1);
            false
        }));
        assert_eq!(calls.get(), 1);
    }
}
