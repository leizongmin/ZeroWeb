//! R5121（css-writing-modes-3 §2.4 bidi box model slice·phase 1）：**段落级 UBA**。
//!
//! 既有路径 `BidiFragmentCursor::with_direction` 按 **run**（文本节点/inline 元素
//! 文本）各自 `BidiInfo::new`——U+202A..=U+202E 控制符的嵌入/覆盖作用域在 run 边界
//! 截断，跨 run 重排（RLO 在一个文本节点开启、内容延续到兄弟节点）不可达
//!（R5120 bidi-001 取证：`.test` span 的 RLO 作用域应覆盖兄弟 `FFFEEEDDD`，
//! chromium 真值 = 跨 run 段整体 L2 重排 `DDDEEEFFF` 插 `CCC`/`GGG` 之间）。
//!
//! 本模块把 UBA 提升到 **IFC 段落级**：全部 Text run 的逻辑文本一次拼接解析，
//! 按字符取 resolved level；**不拆分 items**（拆分会新增词间折行机会——
//! line-breaking-bidi-001 1.07%→76% 实证）。R5122 起（phase 2）标记 run 走
//! `BidiFragmentCursor::with_levels_logical`——**断行消费逻辑序文本**（规范顺序：
//! 先按逻辑序断行，再逐行 bidi 重排；R5121 v3 的断行前视觉重排使贪心折行点
//! 偏离，bidi-004 2.07%→6.19% 实证），视觉重排延迟到行级
//! `reorder_lines_l2_for_para_bidi`：片段内逐字符 parity L2 +
//! 极大奇 parity 片段序列反序（x 镜像）。
//!
//! 边界（FIXME 谱系，翻默认前置逐项收口）：
//! - 段落基方向取**首个** Text run 的 CSS direction（未直读容器 style；direction
//!   为继承属性，无 inline 覆盖页等价）；
//! - 行级 L2 为 **parity 近似**（`uniform_resolved_rtl` 布尔）——基 LTR（level
//!   0/1 两级）精确；RTL 基内嵌套 ≥2 层偶 level 的组序不重排；混合 parity
//!   片段按偶处理（不参与序列反序，仅片段内反转）；
//! - 元素级 `unicode-bidi: embed/isolate` 未映射为显式 level（UAX #9 X5-X8）；
//!   含 `bidi_override`/`plaintext` run 的 IFC **整体回退**既有 per-run 校准路径；
//! - 仅当至少一个 run 段落级 level 非 0 才激活——纯 LTR 信号面零扰动；
//! - 反转片段宽度不变（字符集恒等，advance 求和可交换）；shaped kerning 级
//!   度量漂移为接受近似（reftest 主面 Ahem 等宽零差异）。
//!
//! kill-switch `ZW_PARA_BIDI`（opt-in `=1`，**默认关**）——dormant 同 R5120
//! `ZW_INLINE_SINGLE_FRAG_BOX_BORDER`：翻默认前置 = 旗标开全量 A/B 零新红。

use super::inline_types::{InlineItem, TextFragmentSource};

/// UAX #9 信号面：bidi 控制符（LRE/RLE/LRO/RLO/PDF + 隔离符 FSI/LRI/RLI/PDI）
/// 或 RTL 脚本区段。与 `bidi_reorder_with_direction` 的 `has_rtl_script` 同口径。
fn has_bidi_signal(text: &str) -> bool {
    text.chars().any(|ch| {
        let cp = ch as u32;
        (0x0590..=0x05FF).contains(&cp)
            || (0x0600..=0x06FF).contains(&cp)
            || (0x0700..=0x074F).contains(&cp)
            || (0x08A0..=0x08FF).contains(&cp)
            || (0xFB50..=0xFDFF).contains(&cp)
            || (0xFE70..=0xFEFF).contains(&cp)
            || (0x202A..=0x202E).contains(&cp)
            || (0x2066..=0x2069).contains(&cp)
    })
}

/// 段落级 UBA 计划（与输入 items 逐位平行）。
#[derive(Debug, Clone)]
pub(crate) struct ParaBidiPlan {
    /// `Some(levels)` = 该 Text run 走 `with_levels`（逐字符段落级 level）；
    /// `None` = 原样走既有 per-run 路径。非 Text item 恒 `None`。
    pub item_levels: Vec<Option<Vec<u8>>>,
    /// 任一字符 resolved level 为奇数（存在 RTL 呈现，需要行级组序重排）。
    pub has_rtl: bool,
}

/// 对 IFC 全部 Text run 做段落级 UBA 解析。
///
/// 门（任一不满足 → `None` = 回退既有 per-run 路径）：
/// ① 无 `bidi_override`/`is_plaintext_bidi` run（特例路径各自校准，phase 1 不混排）；
/// ② 至少一个 run 文本含 bidi 信号（控制符/RTL 脚本）；
/// ③ 至少一个 run 段落级 level 非全 0（有真实改写）。
pub(crate) fn resolve(items: &[InlineItem]) -> Option<ParaBidiPlan> {
    let mut texts: Vec<&str> = Vec::new();
    let mut base_is_rtl: Option<bool> = None;
    for item in items {
        if let InlineItem::Text(run) = item {
            if run.bidi_override.is_some() || run.is_plaintext_bidi {
                return None;
            }
            texts.push(&run.text);
            if base_is_rtl.is_none() {
                base_is_rtl = Some(run.is_rtl);
            }
        }
    }
    if !texts.iter().any(|t| has_bidi_signal(t)) {
        return None;
    }

    // 段落级 UBA：一次解析拼接文本（FIXME(phase 2)：元素级 embed/isolate 显式 level；
    // 容器 style 直读基方向）。
    // https://unicode.org/reports/tr9/#P1
    let paragraph: String = texts.concat();
    let base_level = if base_is_rtl.unwrap_or(false) {
        Some(unicode_bidi::Level::rtl())
    } else {
        None
    };
    let bidi_info = unicode_bidi::BidiInfo::new(&paragraph, base_level);

    // 逐 item 取逐字符 level（levels 按 byte 索引，char 取其起始 byte 的 level——
    // level 段在字符边界对齐，段内一致；X9 控制符随周边文段不携带自身 level）。
    // item_levels 与**全部** items 逐位平行（非 Text 位恒 None）——
    // break_lines 按枚举位索引。
    let mut item_levels: Vec<Option<Vec<u8>>> = Vec::with_capacity(items.len());
    let mut has_rtl = false;
    let mut any_transform = false;
    let mut byte_offset = 0_usize;
    let mut texts_iter = texts.iter();
    for item in items {
        if !matches!(item, InlineItem::Text(_)) {
            item_levels.push(None);
            continue;
        }
        let text = texts_iter.next().expect("texts 与 Text items 一一对应");
        let mut levels = Vec::with_capacity(text.chars().count());
        for ch in text.chars() {
            let level = bidi_info.levels[byte_offset].number();
            has_rtl |= level % 2 == 1;
            levels.push(level);
            byte_offset += ch.len_utf8();
        }
        // 改写面：任一字符 level 非 0（偶 level ≥2 与全 0 视觉恒等，但标记无害——
        // with_levels 对无奇 level 序列 identity 短路）。全 0 → 既有 per-run 路径
        //（已正确面，零扰动）。
        let transform = levels.iter().any(|&lv| lv != 0);
        if transform {
            any_transform = true;
            item_levels.push(Some(levels));
        } else {
            item_levels.push(None);
        }
    }

    if !any_transform {
        return None;
    }
    Some(ParaBidiPlan { item_levels, has_rtl })
}

/// 取 fragment 的 uniform resolved parity（`None` = identity/mixed → 偶）。
pub(crate) fn fragment_is_rtl_parity(source: Option<&TextFragmentSource>) -> bool {
    source.and_then(|s| s.uniform_resolved_rtl()).unwrap_or(false)
}

/// R5122（phase 2）：片段内 parity L2——反转极大奇 parity 字符 run，返回
/// 视觉位 → 逻辑字符索引的置换（`None` = 恒等：无奇字符或反转无变化）。
/// 基 LTR（level 0/1）下与 UAX #9 L2 精确一致；奇偶嵌套（≥2 层）为 parity
/// 近似（FIXME：完整 level 栈按层反转）。
/// https://unicode.org/reports/tr9/#L2
pub(crate) fn parity_l2_permutation(parities: &[bool]) -> Option<Vec<usize>> {
    if !parities.iter().any(|&p| p) {
        return None;
    }
    let mut order: Vec<usize> = (0..parities.len()).collect();
    let mut i = 0;
    while i < parities.len() {
        if !parities[i] {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < parities.len() && parities[j] {
            j += 1;
        }
        order[i..j].reverse();
        i = j;
    }
    (order.iter().enumerate().any(|(a, b)| a != *b)).then_some(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline::TextRun;
    use zero_dom::NodeId;
    use zero_style_system::VerticalAlignValue;

    fn text_item(text: &str) -> InlineItem {
        InlineItem::Text(TextRun {
            text: text.to_string(),
            ..TextRun::simple(
                String::new(),
                NodeId::default(),
                16.0,
                19.0,
                VerticalAlignValue::Baseline,
            )
        })
    }

    /// bidi-001 形态：RLO 在 span 文本内开启、作用域跨到兄弟文本节点——兄弟整段
    /// 均匀奇 level 必须拿到段落级 level（per-run 路径按 LTR 基恒等，永不反转）。
    #[test]
    fn rlo_scope_crosses_run_boundary_marks_sibling_rtl() {
        let items = vec![
            text_item(" AAABBBCCC\u{202E}IIIHHHGGG"),
            text_item("FFFEEEDDD\u{202C}"),
            text_item("JJJKKKLLL"),
        ];
        let plan = resolve(&items).expect("RLO 跨 run 信号必须激活段落级 UBA");
        assert!(plan.has_rtl);
        // item1：前缀 level 0，RLO 后 IIIHHHGGG level 1（UAX #9 X9：控制符不携带
        // 自身 level，随周边文段——RLO 归 level 0，不可见不影响呈现）。
        let lv1 = plan.item_levels[0].as_ref().expect("item1 必须改写");
        let expected1: Vec<u8> = " AAABBBCCC\u{202e}"
            .chars()
            .map(|_| 0)
            .chain("IIIHHHGGG".chars().map(|_| 1))
            .collect();
        assert_eq!(lv1, &expected1);
        // item2 整段奇 level（FFFEEEDDD + PDF 都在 RLO 覆盖下）。
        let lv2 = plan.item_levels[1].as_ref().expect("item2 整段奇 level 必须改写");
        assert!(lv2.iter().all(|&lv| lv == 1));
        // item3（PDF 后）回到 level 0，原样保留。
        assert!(plan.item_levels[2].is_none(), "item3 偶 level 不改写");
    }

    /// 纯 LTR 页零信号 → None（不激活，零扰动面）。
    #[test]
    fn plain_ltr_page_not_activated() {
        let items = vec![text_item("hello world"), text_item("  second")];
        assert!(resolve(&items).is_none());
    }

    /// 元素级 bidi-override run 在场 → 整体回退既有校准路径（phase 1 不混排）。
    #[test]
    fn element_override_bails_out() {
        let mut overridden = TextRun {
            text: "rtl".to_string(),
            ..TextRun::simple(
                String::new(),
                NodeId::default(),
                16.0,
                19.0,
                VerticalAlignValue::Baseline,
            )
        };
        overridden.bidi_override = Some(true);
        let items = vec![InlineItem::Text(overridden), text_item("\u{05D0}\u{05D1}")];
        assert!(resolve(&items).is_none());
    }

    /// R5122 phase 2：片段内 parity L2——奇字符 run 反转，偶字符不动。
    #[test]
    fn parity_l2_reverses_odd_runs_only() {
        // [偶×3, 奇×3, 偶×1] → 奇段反转，其余原位。
        let perm = parity_l2_permutation(&[false, false, false, true, true, true, false]).expect("含奇字符必有置换");
        assert_eq!(perm, vec![0, 1, 2, 5, 4, 3, 6]);
        // 无奇字符 → 恒等（None）。
        assert_eq!(parity_l2_permutation(&[false, false]), None);
        // 单一奇 run 恰跨全串 → 整体反转。
        let perm = parity_l2_permutation(&[true, true, true]).expect("全奇串有置换");
        assert_eq!(perm, vec![2, 1, 0]);
    }
}
