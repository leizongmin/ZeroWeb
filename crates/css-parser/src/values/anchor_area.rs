//! `position-area` 布局期区域解析（css-anchor-position-1 §position-area）。
//!
//! canonical 串（engine `dom_bindings::position_area::canonicalize_decl` 产出的
//! CSSOM 形）→ 3×3 网格区域格索引。布局（layout-engine anchor_pos pass）与
//! engine 侧校验共用本单点——layout-engine 不依赖 zero-engine（依赖方向单向），
//! 矩阵下沉本 crate（双方共同依赖）。
//!
//! htb 写作模式第一刀（writing-mode 变体挂账）；隐式轴（单词折叠形）按
//! spec：物理/逻辑单轴 long 关键字隐式轴恒 `span-all`（`left` ≡ `span-all
//! left`），center/span-all/逻辑 short 族两轴同用（`start` ≡ 两轴 start）。
//! 双词形：轴标签词 order-free（`axis_of` 显式归类，author 原文序不保证）；
//! 双 shared 词对（short/center/span-all/self 短族混形）**首词=block、次词=
//! inline**（css-anchor-position-1 两值语法 + WPT position-area-ambiguous-self
//! 页 `'self-end self-start'` htb/rtl = 物理 bottom right 证据）。
//! `none` → `None`（无区域定位）。
//!
//! self 轴语义（`self-*`/`span-self-*`）按**元素自身** writing mode/direction
//! 解析（css-anchor-position-1 §position-area）：`position_area_cells_dir` 的
//! `self_inline_rtl` 臂镜像 inline 轴格（元素 rtl 时 inline start/end 物理互换）；
//! htb 元素 block 轴无方向性不镜像；非 htb 元素与 CB 侧 writing-mode 面（非 self
//! 逻辑词按 CB 写作模式解析）挂账。
//!
//! https://drafts.csswg.org/css-anchor-position-1/#position-area-property

/// 关键字轴归属：`Some(true)` = X 轴（inline 物理/逻辑族），`Some(false)` = Y 轴
/// （block 物理/逻辑族），`None` = 表外/两轴同用族（center/span-all/short/self 短族）。
fn axis_of(kw: &str) -> Option<bool> {
    match kw {
        "left"
        | "x-start"
        | "span-left"
        | "span-x-start"
        | "right"
        | "x-end"
        | "span-right"
        | "span-x-end"
        | "inline-start"
        | "inline-end"
        | "span-inline-start"
        | "span-inline-end"
        | "self-inline-start"
        | "self-inline-end"
        | "span-self-inline-start"
        | "span-self-inline-end"
        | "self-x-start"
        | "self-x-end"
        | "span-self-x-start"
        | "span-self-x-end" => Some(true),
        "top"
        | "y-start"
        | "span-top"
        | "span-y-start"
        | "bottom"
        | "y-end"
        | "span-bottom"
        | "span-y-end"
        | "block-start"
        | "block-end"
        | "span-block-start"
        | "span-block-end"
        | "self-block-start"
        | "self-block-end"
        | "span-self-block-start"
        | "span-self-block-end"
        | "self-y-start"
        | "self-y-end"
        | "span-self-y-start"
        | "span-self-y-end" => Some(false),
        _ => None,
    }
}

/// 单轴关键字 → 格范围 `(start, end)`（0..3 格索引，end 独占）。`self-`/
/// `span-self-` 前缀剥离后同格复用（对齐蕴含差异属布局期 align/justify 面，挂账）。
fn cells(kw: &str) -> Option<(u8, u8)> {
    const START: (u8, u8) = (0, 1);
    const SPAN_START: (u8, u8) = (0, 2);
    const CENTER: (u8, u8) = (1, 2);
    const SPAN_END: (u8, u8) = (1, 3);
    const END: (u8, u8) = (2, 3);
    const ALL: (u8, u8) = (0, 3);
    let s = kw
        .strip_prefix("span-self-")
        .map(|rest| format!("span-{}", rest))
        .or_else(|| kw.strip_prefix("self-").map(str::to_string))
        .unwrap_or_else(|| kw.to_string());
    Some(match s.as_str() {
        "left" | "x-start" | "inline-start" | "top" | "y-start" | "block-start" | "start" => START,
        "span-left" | "span-x-start" | "span-inline-start" | "span-top" | "span-y-start" | "span-block-start"
        | "span-start" => SPAN_START,
        "center" => CENTER,
        "span-right" | "span-x-end" | "span-inline-end" | "span-bottom" | "span-y-end" | "span-block-end"
        | "span-end" => SPAN_END,
        "right" | "x-end" | "inline-end" | "bottom" | "y-end" | "block-end" | "end" => END,
        "span-all" => ALL,
        _ => return None,
    })
}

/// X 轴格解析：self 族词在元素 rtl 时物理镜像（inline start/end 互换；
/// 格范围按 (3-e, 3-s) 对称翻转）。
fn cells_x(kw: &str, self_inline_rtl: bool) -> Option<(u8, u8)> {
    let c = cells(kw)?;
    Some(
        if self_inline_rtl && (kw.starts_with("self-") || kw.starts_with("span-self-")) {
            (3 - c.1, 3 - c.0)
        } else {
            c
        },
    )
}

/// canonical 串 → `((row_start,row_end),(col_start,col_end))`。元素 htb/ltr 或
/// 不涉 self 轴时的便捷形（详见 [`position_area_cells_dir`]）。
pub fn position_area_cells(canonical: &str) -> Option<((u8, u8), (u8, u8))> {
    position_area_cells_dir(canonical, false)
}

/// 同 [`position_area_cells`]，`self_inline_rtl` = anchored 元素自身 direction
/// 为 rtl 时置 true：self 族 inline 轴格物理镜像（START↔END）。
pub fn position_area_cells_dir(canonical: &str, self_inline_rtl: bool) -> Option<((u8, u8), (u8, u8))> {
    let words: Vec<&str> = canonical.split_whitespace().collect();
    match words.len() {
        1 => {
            let w = words[0];
            match w {
                "center" => Some(((1, 2), (1, 2))),
                "span-all" => Some(((0, 3), (0, 3))),
                "none" => None,
                // 逻辑 short 族：两轴同格（折叠语义）；self 族半边走 rtl 镜像。
                "start" | "span-start" | "end" | "span-end" | "self-start" | "span-self-start" | "self-end"
                | "span-self-end" => {
                    let c = cells(w)?;
                    Some((c, cells_x(w, self_inline_rtl)?))
                }
                _ => {
                    // 物理/逻辑单轴 long：对应轴取格、隐式轴 span-all。
                    match axis_of(w)? {
                        true => Some(((0, 3), cells_x(w, self_inline_rtl)?)), // X 轴 kw → 行 span-all
                        false => Some((cells(w)?, (0, 3))),                   // Y 轴 kw → 列 span-all
                    }
                }
            }
        }
        2 => {
            // 轴归类落格：author 原文序不保证 canonical X 前 Y 后（stylesheet
            // 声明不经 shim canonicalize）——轴标签词按 axis_of 显式归类；
            // 双 shared 词对（全 None）**首词=block、次词=inline**（两值语法
            // 约定 + ambiguous-self 页证据）；同轴显式双词非法返 None。
            let (w0, w1) = (words[0], words[1]);
            let a0 = axis_of(w0);
            let a1 = axis_of(w1);
            if a0.is_some() && a0 == a1 {
                return None;
            }
            let (x_kw, y_kw) = match (a0, a1) {
                (Some(true), _) => (w0, w1),
                (Some(false), _) => (w1, w0),
                (_, Some(false)) => (w0, w1),
                (_, Some(true)) => (w1, w0),
                _ => (w1, w0),
            };
            Some((cells(y_kw)?, cells_x(x_kw, self_inline_rtl)?))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::position_area_cells as cells;
    use super::position_area_cells_dir as cells_dir;

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
        // 双 shared 词对：首词=block、次词=inline（ambiguous-self 页证据）。
        // 'self-end self-start' htb/rtl = 物理 bottom right ⇒ block=self-end。
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
        // 非 self 轴标签词按 CB 写作模式解析（挂账），元素 rtl 不镜像。
        assert_eq!(cells_dir("left", true), Some(((0, 3), (0, 1))));
        assert_eq!(cells_dir("self-end center", true), Some(((2, 3), (1, 2))));
    }
}
