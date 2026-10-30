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
//! `reorder_lines_l2_for_para_bidi`：片段内逐字符精确 L2 + 跨片段逐层序列反序
//!（x 镜像）——R5337 起 parity 近似收口，见 `exact_l2_permutation`。
//!
//! 边界（FIXME 谱系，翻默认前置逐项收口）：
//! - 段落基方向取**首个** Text run 的 CSS direction（未直读容器 style；direction
//!   为继承属性，无 inline 覆盖页等价）；
//! - 行级 L2 自 R5337 起**精确**（`exact_l2_permutation` 逐层反转 + 行级跨片段
//!   逐层序列反序，`visual_levels` 携带完整 level 值）——R5122 parity 近似的
//!   ≥2 层嵌套组序缺口（bidi-011 LRO 偶层跨片段反序实证）已收口；精确性以
//!   片段一致 level（min==max）为前提，混合 level 片段（如 RLO/LRO 作用域跨
//!   元素页）保守维持逻辑序，精确性挂账按 level 拆片段（见 break_lines ② 守卫注）；
//! - 元素级 `unicode-bidi: embed` 未映射为显式 level（UAX #9 X5-X8）；含
//!   `bidi_override`/`plaintext`/`isolate`（R5337，含 `<bdi>`）run 的 IFC
//!   **整体回退**既有 per-run 校准路径——isolate 显式 level 映射前保守不混排；
//! - 仅当至少一个 run 段落级 level 非 0 才激活——纯 LTR 信号面零扰动；
//! - 反转片段宽度不变（字符集恒等，advance 求和可交换）；shaped kerning 级
//!   度量漂移为接受近似（reftest 主面 Ahem 等宽零差异）。
//!
//! kill-switch `ZW_PARA_BIDI`（R5337 起**默认开**，`=0` 回退 per-run 校准路径）
//! ——翻默认前置已满足：行级 L2 精确化 + 旗标开全量 corpus A/B 零新红。

use super::inline_types::InlineItem;

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
/// ① 无 `bidi_override`/`is_plaintext_bidi`/`is_isolated_bidi` run（特例路径各自
/// 校准，不混排；isolate 为 R5337 补——未映射 X5-X8 前透明拼接会把 `<bdi>` 标记
/// 文本与后继内容跨边界重排，chromium 探针 `X⦿RW⦿⦿IL⦿` B/C 行分叉实证）；
/// ② 至少一个 run 文本含 bidi 信号（控制符/RTL 脚本）；
/// ③ 至少一个 run 段落级 level 非全 0（有真实改写）。
pub(crate) fn resolve(items: &[InlineItem]) -> Option<ParaBidiPlan> {
    let mut texts: Vec<&str> = Vec::new();
    let mut base_is_rtl: Option<bool> = None;
    for item in items {
        if let InlineItem::Text(run) = item {
            if run.bidi_override.is_some() || run.is_plaintext_bidi || run.is_isolated_bidi {
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
    // R5337：基方向显式取首个 Text run 的 CSS direction（HL1）——`None` 是
    // unicode_bidi 的 P2/P3 首强字符 auto-detect 而非 LTR，R-first 内容的 LTR
    // 块会被误判 RTL 段整段反转（R5244 修 per-run 路径同款缺陷，`ZW_BIDI_LTR_PARA_BASE`
    // 锚点；翻默认后由 r5244_css_ltr_para_base 单测暴露）。plaintext 例外已在上
    // 方门①整体回退，此处两态都显式传 level。
    // https://unicode.org/reports/tr9/#HL1
    let base_level = Some(if base_is_rtl.unwrap_or(false) {
        unicode_bidi::Level::rtl()
    } else {
        unicode_bidi::Level::ltr()
    });
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

/// R5122（phase 2）/ R5337（精确化）：片段内完整 L2——按 level 值逐层反转极大
/// 连续 ≥L 段（L 自最高层降到最低奇层），返回视觉位 → 逻辑字符索引的置换
/// （`None` = 恒等：全 0 层或反转无变化）。
///
/// parity 近似（R5122 原实现）只反转奇 parity 段——≥2 层嵌套（如 bidi-011 的
/// LRO 偶层内容随 ≥1 层组序跨片段移动）不可达，chrome 探针实证精确模型
/// （`X⦿RLO RW⦿⦿LRO IL⦿` → `XILWR`：偶层内容先层内反转、再随 ≥1 层序列反序，
/// 两次反转抵消 = 内容保持逻辑序、位置移到奇层段之前）。
/// https://unicode.org/reports/tr9/#L2
pub(crate) fn exact_l2_permutation(levels: &[u8]) -> Option<Vec<usize>> {
    let max = levels.iter().copied().max()?;
    if max == 0 {
        return None;
    }
    let mut order: Vec<usize> = (0..levels.len()).collect();
    // L 自最高层降到最低奇层（= 1；含中间偶层——level ≥2 偶层段在自己的层先
    // 反转、再随 ≥1 层组序参与反序）。
    let mut l = max;
    while l >= 1 {
        let mut i = 0;
        while i < levels.len() {
            if levels[i] < l {
                i += 1;
                continue;
            }
            let mut j = i + 1;
            while j < levels.len() && levels[j] >= l {
                j += 1;
            }
            order[i..j].reverse();
            i = j;
        }
        l -= 1;
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

    /// R5337：`<bdi>`/`unicode-bidi: isolate` run 在场 → 整体回退 per-run 路径
    /// （X5-X8 显式 level 映射前的保守近似——chromium 探针实证透明拼接把隔离标记
    /// 文本与后继内容跨边界重排，css3-counter-styles-102 ref 回归）。
    #[test]
    fn isolated_run_bails_out() {
        let mut isolated = TextRun {
            text: "\u{0661}\u{0660}. ".to_string(),
            ..TextRun::simple(
                String::new(),
                NodeId::default(),
                16.0,
                19.0,
                VerticalAlignValue::Baseline,
            )
        };
        isolated.is_isolated_bidi = true;
        let items = vec![InlineItem::Text(isolated), text_item("\u{0661}\u{0660}")];
        assert!(resolve(&items).is_none());
        // 守卫腿：无 isolate run 的同形页照常激活（翻默认主面不收窄）。
        let plain = vec![text_item("\u{0661}\u{0660}. "), text_item("\u{0661}\u{0660}")];
        assert!(resolve(&plain).is_some());
    }

    /// R5122 phase 2 / R5337 精确化：片段内完整 L2——逐层反转 ≥L 连续段。
    #[test]
    fn exact_l2_reverses_by_level() {
        // [0×3, 1×3, 0×1] → 奇段反转，其余原位（单层 = parity 特例）。
        let perm = exact_l2_permutation(&[0, 0, 0, 1, 1, 1, 0]).expect("含奇层必有置换");
        assert_eq!(perm, vec![0, 1, 2, 5, 4, 3, 6]);
        // 全 0 层 → 恒等（None）。
        assert_eq!(exact_l2_permutation(&[0, 0]), None);
        // 单一奇 run 恰跨全串 → 整体反转。
        let perm = exact_l2_permutation(&[1, 1, 1]).expect("全奇串有置换");
        assert_eq!(perm, vec![2, 1, 0]);
    }

    /// R5337：≥2 层嵌套组序——chrome 探针模型（bidi-011 形态）。
    /// [T(0) E(0) | TSET(1) | ST(2)]：L=2 反转偶层段 [ST]→[TS]；L=1 反转整个
    /// ≥1 连续段 [TSET][TS] → 视觉序 = TE + ST + TEST（chrome `XILWR` 同构）。
    #[test]
    fn exact_l2_handles_nested_level_two() {
        let levels = [0, 0, 1, 1, 1, 1, 2, 2];
        let perm = exact_l2_permutation(&levels).expect("多层必有置换");
        let src: Vec<char> = "TETSETST".chars().collect();
        let visual: String = perm.iter().map(|&i| src[i]).collect();
        assert_eq!(visual, "TESTTEST");
        // 纯偶层（LRO only，无 RLO）：L=2 与 L=1 两次反转抵消 = 恒等。
        let levels = [0, 2, 2];
        assert_eq!(exact_l2_permutation(&levels), None);
        // 探针同构：X(0) RW(1) IL(2) → XILWR。
        let levels = [0, 1, 1, 2, 2];
        let perm = exact_l2_permutation(&levels).expect("多层必有置换");
        let src: Vec<char> = "XRWIL".chars().collect();
        let visual: String = perm.iter().map(|&i| src[i]).collect();
        assert_eq!(visual, "XILWR");
    }
}
