//! `position-area` 布局期区域解析（css-anchor-position-1 §position-area）。
//!
//! canonical 串（engine `dom_bindings::position_area::canonicalize_decl` 产出的
//! CSSOM 形）→ 3×3 网格区域格索引。布局（layout-engine anchor_pos pass）与
//! engine 侧校验共用本单点——layout-engine 不依赖 zero-engine（依赖方向单向），
//! 矩阵下沉本 crate（双方共同依赖）。
//!
//! 隐式轴（单词折叠形）按 spec：物理/逻辑单轴 long 关键字隐式轴恒 `span-all`
//!（`left` ≡ `span-all left`），center/span-all/逻辑 short 族两轴同用（`start`
//! ≡ 两轴 start）。双词形：轴标签词 order-free（author 原文序不保证，按单词
//! 轴类别显式归类）；双 shared 词对（short/center/span-all/self 短族混形）
//! **首词=block、次词=inline**（css-anchor-position-1 两值语法 + WPT
//! position-area-ambiguous-self 页 `'self-end self-start'` htb/rtl = 物理
//! bottom right 证据）。`none` → `None`（无区域定位）。
//!
//! 写作模式/方向面（css-writing-modes-3 §6 轴映射）：非 self 逻辑词
//!（inline-*/block-*/x-*/y-*/shared）按 **CB** 坐标系解析（inline/block 轴
//! 物理归属与 start 侧随 CB writing-mode/direction 翻转），self 族
//!（`self-*`/`span-self-*`）按**元素自身**坐标系解析（css-anchor-position-1
//! §position-area）；纯物理词（left/right/top/bottom）方向无关。格索引恒
//! 物理（行=Y 列=X），翻转只作用于「关键字 → 格」映射。
//!
//! https://drafts.csswg.org/css-anchor-position-1/#position-area-property

/// 写作模式/方向坐标系（css-writing-modes-3 §6 轴映射）。`vertical` = 垂直块流
///（vertical-rl/lr；sideways-* parse 期已归一为 vertical）；`block_rl` = 块流
/// 右到左（vertical-rl）；`rtl` = inline 流右到左。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AreaWmDir {
    /// 垂直块流（vertical-rl/lr；sideways-* parse 期已归一为 vertical）。
    pub vertical: bool,
    /// 块流右到左（vertical-rl）。
    pub block_rl: bool,
    /// inline 流右到左（direction: rtl）。
    pub rtl: bool,
}

impl AreaWmDir {
    /// horizontal-tb / ltr。
    pub const HTB_LTR: AreaWmDir = AreaWmDir {
        vertical: false,
        block_rl: false,
        rtl: false,
    };
}

/// 坐标系 → 轴归属与 start 侧（css-writing-modes-3 §6.1/§6.2）：
/// htb：inline=X、inline-start=left(ltr)/right(rtl)、block=Y start=top；
/// vertical-lr：inline=Y、inline-start=top(ltr)/bottom(rtl)、block=X start=left；
/// vertical-rl：同 lr 但 block-start=right。
/// 返回 `(inline_is_x, inline_start_low, block_is_x, block_start_low)`。
fn wm_axes(coords: AreaWmDir) -> (bool, bool, bool, bool) {
    let inline_is_x = !coords.vertical;
    let inline_low = !coords.rtl;
    let block_is_x = coords.vertical;
    // htb block=Y start=top（低值端）；vertical-lr block=X start=left（低）；
    // vertical-rl block=X start=right（高）。
    let block_low = !coords.vertical || !coords.block_rl;
    (inline_is_x, inline_low, block_is_x, block_low)
}

/// 参照盒的 X/Y 轴 start 侧：`(x_start_low, y_start_low, x_is_inline)`。
fn ref_axes(coords: AreaWmDir) -> (bool, bool, bool) {
    let (inline_is_x, inline_low, _, block_low) = wm_axes(coords);
    if inline_is_x {
        (inline_low, block_low, true)
    } else {
        (block_low, inline_low, false)
    }
}

/// 参照系 block 轴物理向（X=列=true / Y=行=false）。
fn block_axis(coords: AreaWmDir) -> bool {
    let (_, _, block_is_x, _) = wm_axes(coords);
    block_is_x
}

/// 边形（格范围语义）。
#[derive(Clone, Copy)]
enum Edge {
    Start,
    SpanStart,
    Center,
    SpanEnd,
    End,
    All,
}

/// 边形 → 格范围 `(start, end)`（0..3 格索引，end 独占）；`start_low` = 该轴
/// start 侧在低值端，false 时 START/END/SPAN-* 镜像（rtl inline / rl block）。
fn edge_range(e: Edge, start_low: bool) -> (u8, u8) {
    match (e, start_low) {
        (Edge::Start, true) => (0, 1),
        (Edge::Start, false) => (2, 3),
        (Edge::SpanStart, true) => (0, 2),
        (Edge::SpanStart, false) => (1, 3),
        (Edge::Center, _) => (1, 2),
        (Edge::SpanEnd, true) => (1, 3),
        (Edge::SpanEnd, false) => (0, 2),
        (Edge::End, true) => (2, 3),
        (Edge::End, false) => (0, 1),
        (Edge::All, _) => (0, 3),
    }
}

/// 关键字轴槽位：物理轴（X/Y，坐标方向随参照系）、逻辑轴角色（Inline/Block，
/// 物理轴随参照系写作模式）、shared（两轴同用或由两值语法槽位定轴）。
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    X,
    Y,
    Inline,
    Block,
    Shared,
}

/// 单词三分：（轴槽位, 边形, self 族）。css-anchor-position-1 §position-area
/// 关键字全集；表外词 → `None`。
fn word_kind(kw: &str) -> Option<(Slot, Edge, bool)> {
    Some(match kw {
        // 纯物理（方向无关）。
        "left" => (Slot::X, Edge::Start, false),
        "span-left" => (Slot::X, Edge::SpanStart, false),
        "right" => (Slot::X, Edge::End, false),
        "span-right" => (Slot::X, Edge::SpanEnd, false),
        "top" => (Slot::Y, Edge::Start, false),
        "span-top" => (Slot::Y, Edge::SpanStart, false),
        "bottom" => (Slot::Y, Edge::End, false),
        "span-bottom" => (Slot::Y, Edge::SpanEnd, false),
        // 物理轴 + 逻辑方向（x/y-*）：轴固定，start/end 随参照系该轴方向。
        "x-start" => (Slot::X, Edge::Start, false),
        "span-x-start" => (Slot::X, Edge::SpanStart, false),
        "x-end" => (Slot::X, Edge::End, false),
        "span-x-end" => (Slot::X, Edge::SpanEnd, false),
        "y-start" => (Slot::Y, Edge::Start, false),
        "span-y-start" => (Slot::Y, Edge::SpanStart, false),
        "y-end" => (Slot::Y, Edge::End, false),
        "span-y-end" => (Slot::Y, Edge::SpanEnd, false),
        // 逻辑轴角色（inline/block-*）：物理轴随参照系写作模式。
        "inline-start" => (Slot::Inline, Edge::Start, false),
        "span-inline-start" => (Slot::Inline, Edge::SpanStart, false),
        "inline-end" => (Slot::Inline, Edge::End, false),
        "span-inline-end" => (Slot::Inline, Edge::SpanEnd, false),
        "block-start" => (Slot::Block, Edge::Start, false),
        "span-block-start" => (Slot::Block, Edge::SpanStart, false),
        "block-end" => (Slot::Block, Edge::End, false),
        "span-block-end" => (Slot::Block, Edge::SpanEnd, false),
        // self 族（按元素自身坐标系）。
        "self-inline-start" => (Slot::Inline, Edge::Start, true),
        "span-self-inline-start" => (Slot::Inline, Edge::SpanStart, true),
        "self-inline-end" => (Slot::Inline, Edge::End, true),
        "span-self-inline-end" => (Slot::Inline, Edge::SpanEnd, true),
        "self-block-start" => (Slot::Block, Edge::Start, true),
        "span-self-block-start" => (Slot::Block, Edge::SpanStart, true),
        "self-block-end" => (Slot::Block, Edge::End, true),
        "span-self-block-end" => (Slot::Block, Edge::SpanEnd, true),
        "self-x-start" => (Slot::X, Edge::Start, true),
        "span-self-x-start" => (Slot::X, Edge::SpanStart, true),
        "self-x-end" => (Slot::X, Edge::End, true),
        "span-self-x-end" => (Slot::X, Edge::SpanEnd, true),
        "self-y-start" => (Slot::Y, Edge::Start, true),
        "span-self-y-start" => (Slot::Y, Edge::SpanStart, true),
        "self-y-end" => (Slot::Y, Edge::End, true),
        "span-self-y-end" => (Slot::Y, Edge::SpanEnd, true),
        // shared 短族（两轴同用 / 槽位定轴）。
        "start" => (Slot::Shared, Edge::Start, false),
        "span-start" => (Slot::Shared, Edge::SpanStart, false),
        "center" => (Slot::Shared, Edge::Center, false),
        "span-all" => (Slot::Shared, Edge::All, false),
        "span-end" => (Slot::Shared, Edge::SpanEnd, false),
        "end" => (Slot::Shared, Edge::End, false),
        "self-start" => (Slot::Shared, Edge::Start, true),
        "span-self-start" => (Slot::Shared, Edge::SpanStart, true),
        "self-end" => (Slot::Shared, Edge::End, true),
        "span-self-end" => (Slot::Shared, Edge::SpanEnd, true),
        _ => return None,
    })
}

/// 单词 → 物理轴（X=列=true / Y=行=false）+ 格范围。`slot` = Shared 时由
/// `forced_axis`（两值语法槽位定轴）决定；其余槽位轴由参照系写作模式解析。
fn resolve_word(
    kind: (Slot, Edge, bool),
    forced_axis: Option<bool>,
    cb: AreaWmDir,
    self_box: AreaWmDir,
) -> Option<(bool, (u8, u8))> {
    let (slot, edge, is_self) = kind;
    let coords = if is_self { self_box } else { cb };
    let (x_low, y_low, x_is_inline) = ref_axes(coords);
    let (_, inline_low, _, block_low) = wm_axes(coords);
    let (axis, start_low) = match slot {
        Slot::X => (true, x_low),
        Slot::Y => (false, y_low),
        Slot::Inline => (x_is_inline, inline_low),
        Slot::Block => (!x_is_inline, block_low),
        Slot::Shared => {
            let axis = forced_axis?;
            (axis, if axis { x_low } else { y_low })
        }
    };
    Some((axis, edge_range(edge, start_low)))
}

/// canonical 串 → `((row_start,row_end),(col_start,col_end))`。元素 htb/ltr 或
/// 不涉 self 轴时的便捷形（详见 [`position_area_cells_wm`]）。
pub fn position_area_cells(canonical: &str) -> Option<((u8, u8), (u8, u8))> {
    position_area_cells_wm(canonical, AreaWmDir::HTB_LTR, AreaWmDir::HTB_LTR)
}

/// 同 [`position_area_cells`]，`self_inline_rtl` = anchored 元素自身 direction
/// 为 rtl 时置 true（旧签名兼容：CB 恒 htb/ltr，元素恒 htb）。
pub fn position_area_cells_dir(canonical: &str, self_inline_rtl: bool) -> Option<((u8, u8), (u8, u8))> {
    position_area_cells_wm(
        canonical,
        AreaWmDir::HTB_LTR,
        AreaWmDir {
            vertical: false,
            block_rl: false,
            rtl: self_inline_rtl,
        },
    )
}

/// canonical 串 → `((row_start,row_end),(col_start,col_end))`，写作模式/方向
/// 全量解析：非 self 逻辑词按 `cb` 坐标系、self 族按 `self_box` 坐标系
///（css-anchor-position-1 §position-area + css-writing-modes-3 §6）。
///
/// 格索引恒物理（行=Y 列=X）。双词形：轴显式词按解析轴归类（同轴 → 非法
/// `None`）；恰好一词 shared 时 shared 词取另一物理轴；双 shared 词对按两值
/// 语法**首词=block、次词=inline**。单词形：shared 词两轴同用（`start` ≡ 两轴
/// start），轴显式词另一轴 span-all。
pub fn position_area_cells_wm(canonical: &str, cb: AreaWmDir, self_box: AreaWmDir) -> Option<((u8, u8), (u8, u8))> {
    let words: Vec<&str> = canonical.split_whitespace().collect();
    let (x_range, y_range) = match words.len() {
        1 => {
            let kind = word_kind(words[0])?;
            match kind.0 {
                Slot::Shared => {
                    // 两轴同用：行/列各自按参照系该轴 start 侧取格。
                    let coords = if kind.2 { self_box } else { cb };
                    let (x_low, y_low, _) = ref_axes(coords);
                    (edge_range(kind.1, x_low), edge_range(kind.1, y_low))
                }
                _ => {
                    let (axis, range) = resolve_word(kind, None, cb, self_box)?;
                    if axis { (range, (0, 3)) } else { ((0, 3), range) }
                }
            }
        }
        2 => {
            let k0 = word_kind(words[0])?;
            let k1 = word_kind(words[1])?;
            let (a0, r0, a1, r1) = match (k0.0, k1.0) {
                // 双 shared：首词=block、次词=inline（各按自身参照系定物理轴）。
                (Slot::Shared, Slot::Shared) => {
                    let block = block_axis(if k0.2 { self_box } else { cb });
                    let (ab, rb) = resolve_word(k0, Some(block), cb, self_box)?;
                    let (ai, ri) = resolve_word(k1, Some(!block), cb, self_box)?;
                    (ab, rb, ai, ri)
                }
                // 恰好一词 shared：shared 词取显式词的另一物理轴。
                (Slot::Shared, _) => {
                    let (ae, re) = resolve_word(k1, None, cb, self_box)?;
                    let (as_, rs) = resolve_word(k0, Some(!ae), cb, self_box)?;
                    (as_, rs, ae, re)
                }
                (_, Slot::Shared) => {
                    let (ae, re) = resolve_word(k0, None, cb, self_box)?;
                    let (as_, rs) = resolve_word(k1, Some(!ae), cb, self_box)?;
                    (ae, re, as_, rs)
                }
                // 双显式：同解析轴非法（css-anchor-position-1 两值语法）。
                _ => {
                    let (a0, r0) = resolve_word(k0, None, cb, self_box)?;
                    let (a1, r1) = resolve_word(k1, None, cb, self_box)?;
                    if a0 == a1 {
                        return None;
                    }
                    (a0, r0, a1, r1)
                }
            };
            debug_assert_ne!(a0, a1, "shared 定轴已保证双词异轴");
            // 物理归类：行 = Y 轴词、列 = X 轴词。
            if a0 { (r0, r1) } else { (r1, r0) }
        }
        _ => return None,
    };
    Some((y_range, x_range))
}

#[cfg(test)]
mod tests {
    use super::AreaWmDir;
    use super::position_area_cells as cells;
    use super::position_area_cells_dir as cells_dir;
    use super::position_area_cells_wm as cells_wm;

    #[test]
    fn test_basic_and_folded() {
        // 折叠形：物理单轴隐式轴 span-all。
        assert_eq!(cells("left"), Some(((0, 3), (0, 1))));
        assert_eq!(cells("top"), Some(((0, 1), (0, 3))));
        assert_eq!(cells("right"), Some(((0, 3), (2, 3))));
        assert_eq!(cells("bottom"), Some(((2, 3), (0, 3))));
        // 两轴同用折叠形。
        assert_eq!(cells("center"), Some(((1, 2), (1, 2))));
        assert_eq!(cells("span-all"), Some(((0, 3), (0, 3))));
        assert_eq!(cells("start"), Some(((0, 1), (0, 1))));
        assert_eq!(cells("end"), Some(((2, 3), (2, 3))));
        assert_eq!(cells("none"), None);
    }

    #[test]
    fn test_pairs_canonical_order() {
        // canonical 序：首词 X、次词 Y（"left top" = X left × Y top）。
        assert_eq!(cells("left top"), Some(((0, 1), (0, 1))));
        assert_eq!(cells("top left"), Some(((0, 1), (0, 1))));
        assert_eq!(cells("center center"), Some(((1, 2), (1, 2))));
        // span 组合形。
        assert_eq!(cells("span-right bottom"), Some(((2, 3), (1, 3))));
        assert_eq!(cells("span-all span-all"), Some(((0, 3), (0, 3))));
        // 逻辑 long 对（block 前 inline）。
        assert_eq!(cells("block-start inline-end"), Some(((0, 1), (2, 3))));
        // 表外词拒。
        assert_eq!(cells("foo bar"), None);
    }

    #[test]
    fn test_self_prefix_and_span_forms() {
        // self-/span-self- 前缀同格复用。
        assert_eq!(cells("self-x-start"), Some(((0, 3), (0, 1))));
        assert_eq!(cells("span-self-inline-end"), Some(((0, 3), (1, 3))));
        assert_eq!(cells("span-self-block-start"), Some(((0, 2), (0, 3))));
        // span-start/span-end 折叠形。
        assert_eq!(cells("span-start"), Some(((0, 2), (0, 2))));
        assert_eq!(cells("span-end"), Some(((1, 3), (1, 3))));
    }

    #[test]
    fn test_double_shared_pair_block_first() {
        // 双 shared 词对：首词=block、次词=inline（ambiguous-self 页证据：
        // 'self-end self-start' 元素 htb/rtl = 物理 bottom right ⇒ 首词落 block
        // 轴；htb/ltr 下同对 = bottom left）。
        assert_eq!(cells("self-end self-start"), Some(((2, 3), (0, 1))));
        assert_eq!(cells("end start"), Some(((2, 3), (0, 1))));
        assert_eq!(cells("span-start end"), Some(((0, 2), (2, 3))));
    }

    #[test]
    fn test_rtl_self_inline_mirror() {
        // 元素 rtl：self 族 inline 轴镜像，block 轴与非 self 轴不动。
        assert_eq!(cells_dir("self-end self-start", true), Some(((2, 3), (2, 3))));
        assert_eq!(cells_dir("self-start", true), Some(((0, 1), (2, 3))));
        assert_eq!(cells_dir("self-x-start", true), Some(((0, 3), (2, 3))));
        // 非 self 轴标签词按 CB 写作模式解析（htb CB 恒不镜像），元素 rtl 不镜像。
        assert_eq!(cells_dir("left", true), Some(((0, 3), (0, 1))));
        assert_eq!(cells_dir("self-end center", true), Some(((2, 3), (1, 2))));
    }

    // ── 写作模式/方向面（css-writing-modes-3 轴映射）──

    /// WPT position-area-wm-dir 页锚定：CB htb/rtl。
    #[test]
    fn test_cb_htb_rtl() {
        let rtl = AreaWmDir {
            vertical: false,
            block_rl: false,
            rtl: true,
        };
        // "start start"：block(start)=top × inline(start)=right。
        assert_eq!(cells_wm("start start", rtl, AreaWmDir::HTB_LTR), Some(((0, 1), (2, 3))));
        // "self-start self-start"：元素 htb/ltr 不镜像 → top left。
        assert_eq!(
            cells_wm("self-start self-start", rtl, AreaWmDir::HTB_LTR),
            Some(((0, 1), (0, 1)))
        );
        // "x-start y-start"：CB rtl → x-start=right。
        assert_eq!(
            cells_wm("x-start y-start", rtl, AreaWmDir::HTB_LTR),
            Some(((0, 1), (2, 3)))
        );
        // "inline-start block-end"：inline-start(right) × block-end(bottom)。
        assert_eq!(
            cells_wm("inline-start block-end", rtl, AreaWmDir::HTB_LTR),
            Some(((2, 3), (2, 3)))
        );
    }

    /// WPT position-area-wm-dir 页锚定：CB vertical-lr。
    #[test]
    fn test_cb_vertical_lr() {
        let vlr = AreaWmDir {
            vertical: true,
            block_rl: false,
            rtl: false,
        };
        let htb_ltr = AreaWmDir::HTB_LTR;
        // "start start"：block(left) × inline(top) → top left。
        assert_eq!(cells_wm("start start", vlr, htb_ltr), Some(((0, 1), (0, 1))));
        // "x-start y-start"：x 轴（=block）start=left、y 轴（=inline）start=top。
        assert_eq!(cells_wm("x-start y-start", vlr, htb_ltr), Some(((0, 1), (0, 1))));
        // "inline-start block-end"：inline(top) × block(right) → top right。
        assert_eq!(cells_wm("inline-start block-end", vlr, htb_ltr), Some(((0, 1), (2, 3))));
        // self 族按元素 htb/ltr："self-inline-start self-block-end" = left × bottom。
        assert_eq!(
            cells_wm("self-inline-start self-block-end", vlr, htb_ltr),
            Some(((2, 3), (0, 1)))
        );
    }

    /// WPT position-area-wm-dir 页锚定：CB vertical-lr / rtl（inline 反向）。
    #[test]
    fn test_cb_vertical_lr_rtl() {
        let vlr_rtl = AreaWmDir {
            vertical: true,
            block_rl: false,
            rtl: true,
        };
        let htb_ltr = AreaWmDir::HTB_LTR;
        // "start start"：block(left) × inline(bottom) → bottom left。
        assert_eq!(cells_wm("start start", vlr_rtl, htb_ltr), Some(((2, 3), (0, 1))));
        // self 族元素 htb/ltr 不受 CB rtl 影响。
        assert_eq!(
            cells_wm("self-start self-start", vlr_rtl, htb_ltr),
            Some(((0, 1), (0, 1)))
        );
        // x-start：x 轴 = block（rtl 不影响块流）→ left；y-start：inline rtl → bottom。
        assert_eq!(cells_wm("x-start y-start", vlr_rtl, htb_ltr), Some(((2, 3), (0, 1))));
    }

    /// WPT position-area-wm-dir 页锚定：CB vertical-rl / ltr。
    #[test]
    fn test_cb_vertical_rl() {
        let vrl = AreaWmDir {
            vertical: true,
            block_rl: true,
            rtl: false,
        };
        let htb_ltr = AreaWmDir::HTB_LTR;
        // "start start"：block(right) × inline(top) → top right。
        assert_eq!(cells_wm("start start", vrl, htb_ltr), Some(((0, 1), (2, 3))));
        // "x-start y-start"：x 轴 block-start=right、y 轴 inline-start=top。
        assert_eq!(cells_wm("x-start y-start", vrl, htb_ltr), Some(((0, 1), (2, 3))));
        // "inline-start block-end"：inline(top) × block(left) → top left。
        assert_eq!(cells_wm("inline-start block-end", vrl, htb_ltr), Some(((0, 1), (0, 1))));
    }

    /// 双 shared 词对在 vertical CB：首词=block（=X）、次词=inline（=Y）——
    /// 格归属仍按物理轴（行=Y 列=X）。
    #[test]
    fn test_double_shared_vertical() {
        let vlr = AreaWmDir {
            vertical: true,
            block_rl: false,
            rtl: false,
        };
        // "end start"：block(end)=right × inline(start)=top → top right。
        assert_eq!(cells_wm("end start", vlr, AreaWmDir::HTB_LTR), Some(((0, 1), (2, 3))));
    }

    /// 逻辑词写面单轴折叠形在 vertical CB：隐式轴恒 span-all（物理另一轴）。
    #[test]
    fn test_vertical_single_axis_word() {
        let vlr = AreaWmDir {
            vertical: true,
            block_rl: false,
            rtl: false,
        };
        // "inline-start"：inline 轴=Y → 行 start（top）、列 span-all。
        assert_eq!(
            cells_wm("inline-start", vlr, AreaWmDir::HTB_LTR),
            Some(((0, 1), (0, 3)))
        );
        // "block-end"：block 轴=X → 列 end（right）、行 span-all。
        assert_eq!(cells_wm("block-end", vlr, AreaWmDir::HTB_LTR), Some(((0, 3), (2, 3))));
    }

    /// 显式同解析轴双词非法：vertical CB 下 top(Y) × inline-start(Y) 同轴。
    #[test]
    fn test_same_resolved_axis_invalid() {
        let vlr = AreaWmDir {
            vertical: true,
            block_rl: false,
            rtl: false,
        };
        assert_eq!(cells_wm("top inline-start", vlr, AreaWmDir::HTB_LTR), None);
        // htb 下同对不冲突（top=Y、inline-start=X）。
        assert_eq!(
            cells_wm("top inline-start", AreaWmDir::HTB_LTR, AreaWmDir::HTB_LTR),
            Some(((0, 1), (0, 1)))
        );
    }
}
