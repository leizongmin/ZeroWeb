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

pub(super) fn bidi_fragment_source() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_BIDI_FRAGMENT_SOURCE"));
    selected(*VALUE, || default_on("ZW_BIDI_FRAGMENT_SOURCE"))
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
