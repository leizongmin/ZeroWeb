//! CSSOM `position-area`（css-anchor-position-1）声明值文法校验与两级序列化（组 F 批）。
//!
//! 与 [`super::grid_cssom`] 同架构（单实现双通道）：
//! - **CSSOM declared 面**（[`canonicalize_decl`]）：shim 写通道经
//!   `__zw_position_area_normalize_decl` 回调、native 绑定 setter 直接分派——非法值
//!   丢弃声明（spec `dom-cssstyledeclaration-setproperty`），合法值回写 canonical 串。
//! - **computed 面**（[`computed_value`]）：gCS plain 臂经 `__zw_position_area_computed`
//!   回调，把 declared 值按 spec computed-value 规则折叠（逻辑轴 long→short、span-all
//!   轴消解、center 槽位归位）。
//!
//! 序列化矩阵全部以语料为 oracle（css-anchor-position/position-area-parsing.html 与
//! position-area-computed.html，anchorbase 基线 1800F/633F 两桶），非 spec 字面推演：
//! 物理 X/Y 对 canonical 序化为 X 前 Y 后；无歧义逻辑对保 long 形（computed 折 short）；
//! 轴歧义 start/end 族 CSSOM 保输入序（computed 同），配 span-all 时按槽位消解为
//! block/inline long 形。center 槽位：X/I/SI 配 center 归 `center X` 形，V 归
//! `center V` 形，H 归 `H center` 形——B/SB/Se/SSe 配 center CSSOM 保输入序（语料
//! 实证 Blink 如此，非 spec 对称推演）。
//!
//! https://drafts.csswg.org/css-anchor-position-1/#position-area

/// 关键字轴类。Se/SSe 为轴歧义 start/end 族（SSe 含 self 前缀）；NoneKw 仅单词
/// 合法（`none none`/`none start` 等对非法——语料钉面）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Axis {
    H,
    V,
    B,
    I,
    SB,
    SI,
    Se,
    SSe,
    Center,
    SpanAll,
    NoneKw,
}

/// 关键字 → 轴类；表外 ident 返 `None`。
fn classify(kw: &str) -> Option<Axis> {
    match kw {
        "left" | "right" | "span-left" | "span-right" | "x-start" | "x-end" | "span-x-start" | "span-x-end"
        | "self-x-start" | "self-x-end" | "span-self-x-start" | "span-self-x-end" => Some(Axis::H),
        "top" | "bottom" | "span-top" | "span-bottom" | "y-start" | "y-end" | "span-y-start" | "span-y-end"
        | "self-y-start" | "self-y-end" | "span-self-y-start" | "span-self-y-end" => Some(Axis::V),
        "block-start" | "block-end" | "span-block-start" | "span-block-end" => Some(Axis::B),
        "inline-start" | "inline-end" | "span-inline-start" | "span-inline-end" => Some(Axis::I),
        "self-block-start" | "self-block-end" | "span-self-block-start" | "span-self-block-end" => Some(Axis::SB),
        "self-inline-start" | "self-inline-end" | "span-self-inline-start" | "span-self-inline-end" => Some(Axis::SI),
        "start" | "end" | "span-start" | "span-end" => Some(Axis::Se),
        "self-start" | "self-end" | "span-self-start" | "span-self-end" => Some(Axis::SSe),
        "center" => Some(Axis::Center),
        "span-all" => Some(Axis::SpanAll),
        "none" => Some(Axis::NoneKw),
        _ => None,
    }
}

/// CSSOM declared 面：canonical 串；非法值返 `None`（调用方丢弃声明）。
///
/// 空串 = 移除声明（CSSOM 空值语义，调用方 setProperty/setter 均以此判 remove），
/// 放行 `Some("")`；CSS 全局关键字 ASCII ci 归小写 echo；var() 值 parse 期恒有效
/// 原文保形（css-variables-1 §3，grid_cssom 同款）。
pub(crate) fn canonicalize_decl(value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return Some(String::new());
    }
    let lower = v.to_ascii_lowercase();
    if ["inherit", "initial", "unset", "revert", "revert-layer"]
        .iter()
        .any(|k| lower == *k)
    {
        return Some(lower);
    }
    // https://drafts.csswg.org/css-variables-1/#using-variables
    if lower.contains("var(") {
        return Some(v.to_string());
    }
    let toks: Vec<&str> = lower.split_whitespace().collect();
    match toks.len() {
        // 单关键字：none/center/span-all/任一轴关键字，echo。
        1 => classify(toks[0]).map(|_| toks[0].to_string()),
        2 => serialize_decl_pair(toks[0], toks[1]),
        // 三关键字及以上非法（语料 `top left top` 钉面）。
        _ => None,
    }
}

/// 双关键字 CSSOM canonical 序列化（输入假定 ASCII 小写、单词元）。
fn serialize_decl_pair(a: &str, b: &str) -> Option<String> {
    let (ka, kb) = (classify(a)?, classify(b)?);
    use Axis::*;
    // 全折叠/保序对（与轴无关）。
    match (ka, kb) {
        (Center, Center) => return Some("center".to_string()),
        (SpanAll, SpanAll) => return Some("span-all".to_string()),
        // center/span-all 混合与轴歧义族：canonical = 输入序保形（语料实证）。
        (Center, SpanAll) | (SpanAll, Center) => return Some(format!("{a} {b}")),
        // 轴歧义族：同词折叠单词（语料 `start start`→`start`），异词输入序保形。
        (Se, Se) | (SSe, SSe) if a == b => return Some(a.to_string()),
        (Se, Se) | (SSe, SSe) => return Some(format!("{a} {b}")),
        // 互补轴对：canonical 固定序（物理 X 前 Y 后；逻辑 block 前 inline）。
        (H, V) | (V, H) => {
            let (h, v) = if ka == H { (a, b) } else { (b, a) };
            return Some(format!("{h} {v}"));
        }
        (B, I) | (I, B) => {
            let (blk, inl) = if ka == B { (a, b) } else { (b, a) };
            return Some(format!("{blk} {inl}"));
        }
        (SB, SI) | (SI, SB) => {
            let (blk, inl) = if ka == SB { (a, b) } else { (b, a) };
            return Some(format!("{blk} {inl}"));
        }
        _ => {}
    }
    // 单轴关键字 + center：槽位归位（H/B/SB→`K center`；V/I/SI→`center K`；轴歧义
    // Se/SSe 保输入序——语料 span_all_center helper 实证，B/SB 不保序）。
    if ka == Center || kb == Center {
        let k = if ka == Center { b } else { a };
        return match (ka, kb) {
            (H, _) | (_, H) | (B, _) | (_, B) | (SB, _) | (_, SB) => Some(format!("{k} center")),
            (V, _) | (_, V) | (I, _) | (_, I) | (SI, _) | (_, SI) => Some(format!("center {k}")),
            _ => Some(format!("{a} {b}")),
        };
    }
    // 单轴关键字 + span-all：span-all 冗余丢弃（轴歧义族除外——保输入序）。
    if ka == SpanAll || kb == SpanAll {
        let k = if ka == SpanAll { b } else { a };
        return match classify(k) {
            Some(Se) | Some(SSe) => Some(format!("{a} {b}")),
            Some(H) | Some(V) | Some(B) | Some(I) | Some(SB) | Some(SI) => Some(k.to_string()),
            _ => None,
        };
    }
    // 其余跨类组合非法（语料 invalid 矩阵：H×B/I/SI/SB/Se/SSe 等同轴/异族对）。
    None
}

/// 逻辑轴歧义关键字 → block 轴消解 long 形（`start`→`block-start`）。
fn resolve_block(kw: &str) -> &'static str {
    match kw {
        "start" => "block-start",
        "end" => "block-end",
        "span-start" => "span-block-start",
        "span-end" => "span-block-end",
        "self-start" => "self-block-start",
        "self-end" => "self-block-end",
        "span-self-start" => "span-self-block-start",
        "span-self-end" => "span-self-block-end",
        _ => "none",
    }
}

/// 逻辑轴歧义关键字 → inline 轴消解 long 形（`start`→`inline-start`）。
fn resolve_inline(kw: &str) -> &'static str {
    match kw {
        "start" => "inline-start",
        "end" => "inline-end",
        "span-start" => "span-inline-start",
        "span-end" => "span-inline-end",
        "self-start" => "self-inline-start",
        "self-end" => "self-inline-end",
        "span-self-start" => "span-self-inline-start",
        "span-self-end" => "span-self-inline-end",
        _ => "none",
    }
}

/// 逻辑轴 long 形 → short 形（`block-start`/`inline-start`→`start`，
/// `span-self-block-start`/`span-self-inline-start`→`span-self-start`）。
fn logical_short(kw: &str) -> &'static str {
    match kw {
        "block-start" | "inline-start" => "start",
        "block-end" | "inline-end" => "end",
        "span-block-start" | "span-inline-start" => "span-start",
        "span-block-end" | "span-inline-end" => "span-end",
        "self-block-start" | "self-inline-start" => "self-start",
        "self-block-end" | "self-inline-end" => "self-end",
        "span-self-block-start" | "span-self-inline-start" => "span-self-start",
        "span-self-block-end" | "span-self-inline-end" => "span-self-end",
        _ => "none",
    }
}

/// computed 面：declared 值（CSSOM canonical 形）→ computed 串。
///
/// 输入非法/CSS-wide/var() 形时原样 echo（防御——正常链路 CSS-wide 由 shim plain
/// 臂先消解，var() computed 替换未建模挂账）；折叠矩阵以 position-area-computed
/// 语料为 oracle。
pub(crate) fn computed_value(decl: &str) -> String {
    let v = decl.trim();
    let lower = v.to_ascii_lowercase();
    let toks: Vec<&str> = lower.split_whitespace().collect();
    if toks.len() == 1 {
        return toks[0].to_string();
    }
    let (a, b) = match toks.as_slice() {
        [a, b] => (*a, *b),
        _ => return v.to_string(),
    };
    let Some(ka) = classify(a) else {
        return v.to_string();
    };
    let Some(kb) = classify(b) else {
        return v.to_string();
    };
    use Axis::*;
    match (ka, kb) {
        (Center, Center) => "center".to_string(),
        (SpanAll, SpanAll) => "span-all".to_string(),
        (Se, Se) | (SSe, SSe) if a == b => a.to_string(),
        (Center, SpanAll)
        | (SpanAll, Center)
        | (Se, Se)
        | (SSe, SSe)
        | (Se, Center)
        | (Center, Se)
        | (SSe, Center)
        | (Center, SSe) => format!("{a} {b}"),
        (H, V) | (V, H) => {
            let (h, v) = if ka == H { (a, b) } else { (b, a) };
            format!("{h} {v}")
        }
        // 无歧义逻辑对：long → short 折叠，short 相等折叠单词。
        (B, I) | (I, B) => {
            let (blk, inl) = if ka == B { (a, b) } else { (b, a) };
            let (bs, is) = (logical_short(blk), logical_short(inl));
            if bs == is { bs.to_string() } else { format!("{bs} {is}") }
        }
        (SB, SI) | (SI, SB) => {
            let (blk, inl) = if ka == SB { (a, b) } else { (b, a) };
            let (bs, is) = (logical_short(blk), logical_short(inl));
            if bs == is { bs.to_string() } else { format!("{bs} {is}") }
        }
        // X + span-all：显式轴关键字丢 span-all；轴歧义族按槽位消解 long 形。
        (Se, SpanAll) => resolve_block(a).to_string(),
        (SpanAll, Se) => resolve_inline(b).to_string(),
        (SSe, SpanAll) => resolve_block(a).to_string(),
        (SpanAll, SSe) => resolve_inline(b).to_string(),
        (H, SpanAll)
        | (SpanAll, H)
        | (V, SpanAll)
        | (SpanAll, V)
        | (B, SpanAll)
        | (SpanAll, B)
        | (I, SpanAll)
        | (SpanAll, I)
        | (SB, SpanAll)
        | (SpanAll, SB)
        | (SI, SpanAll)
        | (SpanAll, SI) => {
            if ka == SpanAll {
                b.to_string()
            } else {
                a.to_string()
            }
        }
        // X + center：槽位归位 + 逻辑轴 short 折叠（I/SI center 后形同样归约短形，
        // 语料 `center inline-start`→`center start`）；轴歧义族保输入序（上方已截获）。
        (H, Center) | (Center, H) => {
            let h = if ka == Center { b } else { a };
            format!("{h} center")
        }
        (V, Center) | (Center, V) => {
            let k = if ka == Center { b } else { a };
            format!("center {k}")
        }
        (I, Center) | (Center, I) | (SI, Center) | (Center, SI) => {
            let k = if ka == Center { b } else { a };
            format!("center {}", logical_short(k))
        }
        (B, Center) | (Center, B) | (SB, Center) | (Center, SB) => {
            let k = if ka == Center { b } else { a };
            format!("{} center", logical_short(k))
        }
        _ => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decl_singles_and_collapse() {
        // 单关键字 echo（含 none/center/span-all/逻辑 long 形）。
        for v in [
            "none",
            "center",
            "span-all",
            "left",
            "self-x-end",
            "block-start",
            "inline-start",
            "span-self-inline-end",
            "start",
            "span-self-start",
        ] {
            assert_eq!(canonicalize_decl(v).as_deref(), Some(v), "single {v}");
        }
        // 大写归一。
        assert_eq!(canonicalize_decl("LEFT Top").as_deref(), Some("left top"));
        // 全折叠对。
        assert_eq!(canonicalize_decl("center center").as_deref(), Some("center"));
        assert_eq!(canonicalize_decl("span-all span-all").as_deref(), Some("span-all"));
        assert_eq!(canonicalize_decl("start start").as_deref(), Some("start"));
        assert_eq!(canonicalize_decl("self-end self-end").as_deref(), Some("self-end"));
    }

    #[test]
    fn test_decl_pair_canonical_orders() {
        // 物理对 X 前 Y 后（双序输入）。
        assert_eq!(canonicalize_decl("left top").as_deref(), Some("left top"));
        assert_eq!(canonicalize_decl("top left").as_deref(), Some("left top"));
        assert_eq!(
            canonicalize_decl("bottom span-right").as_deref(),
            Some("span-right bottom")
        );
        // 逻辑对 block 前 inline（long 保形）。
        assert_eq!(
            canonicalize_decl("inline-end block-start").as_deref(),
            Some("block-start inline-end")
        );
        assert_eq!(
            canonicalize_decl("self-inline-start span-self-block-end").as_deref(),
            Some("span-self-block-end self-inline-start")
        );
        // 轴歧义族：输入序保形。
        assert_eq!(canonicalize_decl("end start").as_deref(), Some("end start"));
        assert_eq!(
            canonicalize_decl("span-self-end self-start").as_deref(),
            Some("span-self-end self-start")
        );
        // center 槽位：H→`H center`；V/I/SI→`center X`；B/SB 保输入序。
        assert_eq!(canonicalize_decl("center left").as_deref(), Some("left center"));
        assert_eq!(canonicalize_decl("top center").as_deref(), Some("center top"));
        assert_eq!(
            canonicalize_decl("inline-start center").as_deref(),
            Some("center inline-start")
        );
        assert_eq!(
            canonicalize_decl("center self-inline-end").as_deref(),
            Some("center self-inline-end")
        );
        assert_eq!(
            canonicalize_decl("center block-start").as_deref(),
            Some("block-start center")
        );
        assert_eq!(
            canonicalize_decl("self-block-start center").as_deref(),
            Some("self-block-start center")
        );
        assert_eq!(canonicalize_decl("center start").as_deref(), Some("center start"));
        // span-all：显式轴关键字冗余丢弃；轴歧义族保输入序。
        assert_eq!(canonicalize_decl("span-all top").as_deref(), Some("top"));
        assert_eq!(canonicalize_decl("span-all inline-end").as_deref(), Some("inline-end"));
        assert_eq!(canonicalize_decl("start span-all").as_deref(), Some("start span-all"));
        assert_eq!(canonicalize_decl("center span-all").as_deref(), Some("center span-all"));
        assert_eq!(canonicalize_decl("span-all center").as_deref(), Some("span-all center"));
    }

    #[test]
    fn test_decl_invalid_and_passthrough() {
        // 同轴/异族非法对。
        for v in [
            "left left",
            "left right",
            "top bottom",
            "block-start block-start",
            "inline-start inline-start",
            "self-inline-start self-inline-start",
            "left inline-start",
            "top block-end",
            "left start",
            "inline-start self-block-end",
            "start self-start",
            "none none",
            "none start",
            "top left top",
            "foobar",
            "start foobar",
        ] {
            assert_eq!(canonicalize_decl(v), None, "invalid {v}");
        }
        // 空串 = 移除声明放行；CSS 全局关键字归小写；var() 原文保形。
        assert_eq!(canonicalize_decl(""), Some(String::new()));
        assert_eq!(canonicalize_decl("  ").as_deref(), Some(""));
        assert_eq!(canonicalize_decl("INHERIT").as_deref(), Some("inherit"));
        assert_eq!(canonicalize_decl("var(--pa)").as_deref(), Some("var(--pa)"));
    }

    #[test]
    fn test_computed_fold() {
        // 单关键字 echo（long 形保留）。
        assert_eq!(computed_value("inline-start"), "inline-start");
        assert_eq!(computed_value("block-start"), "block-start");
        assert_eq!(computed_value("start"), "start");
        // 逻辑对 short 折叠 + 相等折叠。
        assert_eq!(computed_value("block-start inline-end"), "start end");
        assert_eq!(computed_value("inline-end block-start"), "start end");
        assert_eq!(computed_value("block-start inline-start"), "start");
        assert_eq!(computed_value("span-block-start span-inline-start"), "span-start");
        assert_eq!(
            computed_value("self-block-start self-inline-end"),
            "self-start self-end"
        );
        // span-all 轴消解。
        assert_eq!(computed_value("left span-all"), "left");
        assert_eq!(computed_value("span-all block-start"), "block-start");
        assert_eq!(computed_value("inline-start span-all"), "inline-start");
        assert_eq!(computed_value("start span-all"), "block-start");
        assert_eq!(computed_value("span-all start"), "inline-start");
        assert_eq!(computed_value("span-all self-end"), "self-inline-end");
        assert_eq!(computed_value("span-self-start span-all"), "span-self-block-start");
        // center 槽位 + 逻辑 short。
        assert_eq!(computed_value("center left"), "left center");
        assert_eq!(computed_value("top center"), "center top");
        assert_eq!(computed_value("block-start center"), "start center");
        assert_eq!(computed_value("center block-start"), "start center");
        assert_eq!(computed_value("inline-start center"), "center start");
        assert_eq!(computed_value("center self-inline-end"), "center self-end");
        // 轴歧义族保输入序。
        assert_eq!(computed_value("start end"), "start end");
        assert_eq!(computed_value("center start"), "center start");
        assert_eq!(computed_value("center span-end"), "center span-end");
        // 物理对与折叠对。
        assert_eq!(computed_value("top left"), "left top");
        assert_eq!(computed_value("center center"), "center");
        assert_eq!(computed_value("span-all span-all"), "span-all");
        assert_eq!(computed_value("center span-all"), "center span-all");
        // 非法/未建模输入 echo（防御）。
        assert_eq!(computed_value("var(--pa)"), "var(--pa)");
    }

    #[test]
    fn test_decl_canonical_idempotent() {
        // CSSOM canonical 形复写幂等（round-trip 钉面：parsing 语料二次赋值读回一致）。
        for v in [
            "left top",
            "block-start inline-end",
            "end start",
            "center top",
            "center block-start",
            "start span-all",
            "center span-all",
            "span-self-block-end self-inline-start",
            "start",
            "span-all",
            "none",
        ] {
            let once = canonicalize_decl(v).unwrap();
            assert_eq!(canonicalize_decl(&once).as_deref(), Some(once.as_str()), "{v}");
        }
        // computed 串回写 CSSOM 幂等（computed-test round-trip 臂）。
        for v in [
            "start end",
            "block-start",
            "center start",
            "span-block-start",
            "self-start self-end",
            "inline-start",
            "center self-end",
        ] {
            let cssom = canonicalize_decl(v).unwrap();
            assert_eq!(canonicalize_decl(&cssom).as_deref(), Some(cssom.as_str()), "{v}");
            assert_eq!(computed_value(&cssom), v, "{v}");
        }
    }
}
