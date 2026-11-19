//! mask 简写展开（css-masking-1 §3.10 `<mask-layer>#`，刀 MK 全文法重构）。
//!
//! 逐层文法：`<mask-reference> || <position> [/ <bg-size>]? || <repeat-style> ||
//! <geometry-box> || [no-clip || <geometry-box>]? || <compositing-operator> ||
//! <masking-mode>`；括号感知顶层逗号分层（复用 background 层拆）。
//! 拒收集（语料 mask-invalid.html 13 形定谳，`||` 槽唯一性收紧）：
//! 双 reference（none+\<image\> 或双 image）、双 mode、双 composite、双 `/` size、
//! 双 repeat 单轴（repeat-x/repeat-y 不可配对）、盒 ≥2 加 no-clip、三盒、
//! `margin-box`（§3.7 geometry-box 无此值）、三四裸 length 位语。
//!
//! §3.10 reset 语义（投影名单与旧单值版一致）：mask 声明把全部 mask 长手与
//! mask-border-\* 五长手重置为初值——展开即全投影。mask-mode 与 mask-border-\*
//! 五长手有完整 apply 通道；六长手（position/size/repeat/clip/origin/composite）
//! 写值 apply 通道 FIXME(mask-longhands-unimplemented) 未实现——投影 reset 真、
//! 写值不生效（computed 面位移归其分诊池，不入刀 MK 账）。

use super::background::{split_bg_layer_list, split_bg_position_and_size};
use super::{MatchingDecl, matches_css_wide_keyword, wide_keyword_to_longhands};
use crate::computed::contains_var_function;

/// css-masking-1 §3.7 `<geometry-box>`：盒关键字集（无 margin-box）。
const MASK_GEOMETRY_BOXES: [&str; 6] = [
    "content-box",
    "padding-box",
    "border-box",
    "fill-box",
    "stroke-box",
    "view-box",
];

/// 单层槽位（原始文本域存储——CSSOM 读面 canonical 重组同源）。
struct MaskLayerSlots {
    /// `<mask-reference>`：none 旗标（与 `image` 互斥；双置 = 双 reference 拒）。
    reference_none: bool,
    /// 图像函数整段文本（`url(...)`/渐变/image-set 等，括号感知提取）。
    image: String,
    /// `<position>` 原文（token 空格回拼；空 = 初值 `0% 0%`）。
    position: String,
    /// `<bg-size>` 原文（`/` 后段；None = 初值 `auto`）。
    size: Option<String>,
    /// `<repeat-style>` 1-2 token（单轴 repeat-x/y 仅单 token 合法）。
    repeats: Vec<String>,
    /// `<geometry-box>` 1-2 个（首 origin 次 clip；≥3 拒）。
    boxes: Vec<String>,
    /// no-clip 旗标（与盒 ≥2 同现拒——语料定谳；单盒/零盒合法）。
    no_clip: bool,
    /// `<compositing-operator>`（add|subtract|intersect|exclude）。
    composite: String,
    /// `<masking-mode>`（alpha|luminance|match-source）。
    mode: String,
}

/// 逐层解析（层内无 var()/attr()——替换函数形由调用方放行或上层拒绝）。
fn parse_mask_layer(layer: &str) -> Option<MaskLayerSlots> {
    // 图像函数括号感知提取（background 刀 R2878 同型——渐变内逗号/空白不越级）。
    let image_funcs = [
        "url(",
        "linear-gradient(",
        "repeating-linear-gradient(",
        "radial-gradient(",
        "repeating-radial-gradient(",
        "conic-gradient(",
        "repeating-conic-gradient(",
        "image-set(",
        "filter(",
    ];
    let lower = layer.to_ascii_lowercase();
    let mut image = String::new();
    let working_owned: String = match image_funcs.iter().filter_map(|f| lower.find(f)).min() {
        Some(start) => {
            let bytes = layer.as_bytes();
            let mut depth = 0i32;
            let mut found_open = false;
            let mut end = layer.len();
            for (i, &b) in bytes.iter().enumerate().skip(start) {
                match b {
                    b'(' => {
                        depth += 1;
                        found_open = true;
                    }
                    b')' if depth > 0 => depth -= 1,
                    _ => {}
                }
                if found_open && depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            image = layer[start..end].to_string();
            let mut s = String::new();
            let head = layer[..start].trim();
            let tail = layer[end..].trim();
            if !head.is_empty() {
                s.push_str(head);
            }
            if !tail.is_empty() {
                if !s.is_empty() {
                    s.push(' ');
                }
                s.push_str(tail);
            }
            s
        }
        None => layer.to_string(),
    };
    let working = working_owned.trim();
    if contains_var_function(working) {
        return None;
    }
    let (pos_part, size_part) = split_bg_position_and_size(working);
    let mut slots = MaskLayerSlots {
        reference_none: false,
        image,
        position: String::new(),
        size: None,
        repeats: Vec::new(),
        boxes: Vec::new(),
        no_clip: false,
        composite: String::new(),
        mode: String::new(),
    };
    // `<bg-size>` 段增量消费：`/` 后仅取 1-2 个 bg-size 组件（2 token 优先、
    // 1 token 回退，严门 parse_background_size 同源），余 token 溢出归主流
    // 分类——`||` 文法 size 位后可续 repeat/盒/composite/mode（语料 valid 形
    // `1px 2px / contain space round view-box no-clip intersect` 驱动；整段
    // 当 size 会误拒全层）。`/` 后无任一合法 bg-size 前缀 = 语法拒。
    let mut spill: Vec<&str> = Vec::new();
    if let Some(sp) = size_part {
        let toks: Vec<&str> = sp.split_whitespace().collect();
        let taken = (1..=toks.len().min(2))
            .rev()
            .find(|&take| zero_css_parser::values::parse_background_size(&toks[..take].join(" ")).is_some())?;
        slots.size = Some(toks[..taken].join(" "));
        spill = toks[taken..].to_vec();
    }
    let mut tokens = super::split_top_level_whitespace(pos_part)?;
    tokens.extend(spill);
    for token in tokens {
        match token {
            "none" => {
                // `<mask-reference>` none 形；与图像函数同现 = 双 reference 拒。
                if slots.reference_none || !slots.image.is_empty() {
                    return None;
                }
                slots.reference_none = true;
            }
            t if MASK_GEOMETRY_BOXES.contains(&t) => {
                if slots.boxes.len() >= 2 {
                    return None;
                }
                slots.boxes.push(t.to_string());
            }
            "margin-box" => return None, // §3.7：geometry-box 无 margin-box
            "no-clip" => {
                if slots.no_clip {
                    return None;
                }
                slots.no_clip = true;
            }
            "repeat-x" | "repeat-y" | "repeat" | "no-repeat" | "space" | "round" => {
                if slots.repeats.len() >= 2 {
                    return None;
                }
                slots.repeats.push(token.to_string());
            }
            "add" | "subtract" | "intersect" | "exclude" => {
                if !slots.composite.is_empty() {
                    return None;
                }
                slots.composite = token.to_string();
            }
            "alpha" | "luminance" | "match-source" => {
                if !slots.mode.is_empty() {
                    return None;
                }
                slots.mode = token.to_string();
            }
            t => {
                // `<position>` 组件（keyword/length/percent/bare-0）原文累积，
                // 终验走 parse_position_strict（三四裸 length 形拒收）。
                use zero_css_parser::values::LengthValue;
                let is_pos_token = matches!(t, "top" | "center" | "bottom" | "left" | "right")
                    || matches!(
                        zero_css_parser::values::parse_length(t),
                        Some(LengthValue::Px(_) | LengthValue::Percentage(_))
                    )
                    || (t.parse::<f32>().map(|n| n == 0.0).unwrap_or(false));
                if !is_pos_token {
                    return None;
                }
                if !slots.position.is_empty() {
                    slots.position.push(' ');
                }
                slots.position.push_str(t);
            }
        }
    }
    // 槽唯一性/配对收紧（语料定谳）。
    if slots.repeats.len() == 2 && slots.repeats.iter().any(|r| r == "repeat-x" || r == "repeat-y") {
        return None; // 单轴 repeat 不可配对（`repeat-y repeat-x` 拒）
    }
    if slots.no_clip && slots.boxes.len() >= 2 {
        return None; // no-clip + 双盒（`stroke-box no-clip view-box` 族拒）
    }
    if !slots.position.is_empty() {
        zero_css_parser::values::parse_position_strict(&slots.position)?;
    }
    Some(slots)
}

/// mask 简写全投影名单（css-masking-1 §3.10 reset 语义：八 `mask-*` 长手 +
/// `mask-border-*` 五长手）。与 `expand_mask` 投影同表；css-wide 关键字逐名透传。
const MASK_LONGHANDS: [&str; 13] = [
    "mask-image",
    "mask-mode",
    "mask-position",
    "mask-size",
    "mask-repeat",
    "mask-origin",
    "mask-clip",
    "mask-composite",
    "mask-border-source",
    "mask-border-slice",
    "mask-border-width",
    "mask-border-outset",
    "mask-border-repeat",
];

/// mask 简写展开（css-masking-1 §3.10）。展开失败 → 空表（声明丢弃，简写不可
/// 解析惯例一致）；多层逐 longhand 逗号连接（mask-origin/clip 文法即 `#` 列表）。
pub(super) fn expand_mask(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    // CSS-wide 关键字门（R2464 gap class 同型）：无此门 `mask: inherit` 等落
    // token 分类失败 → 声明丢弃 → raw-store 回显回归（P2F）。§3.10 reset 名单
    // = 全投影 13 长手，逐名透传同表。
    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(value, &MASK_LONGHANDS, important, specificity);
    }
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    let mut layers = Vec::new();
    for seg in split_bg_layer_list(value) {
        let Some(l) = parse_mask_layer(seg) else {
            return Vec::new();
        };
        layers.push(l);
    }
    let join = |get: fn(&MaskLayerSlots) -> String, default: &str| -> String {
        layers
            .iter()
            .map(|l| {
                let v = get(l);
                if v.is_empty() { default.to_string() } else { v }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    // clip 派生：no-clip 旗标 → `no-clip`；双盒取次盒；单盒/零盒同 origin 值
    // （§3.10 prose：单 `<geometry-box>` 同时定 origin 与 clip）。
    let clip_of = |l: &MaskLayerSlots| -> String {
        if l.no_clip {
            "no-clip".to_string()
        } else if let Some(second) = l.boxes.get(1) {
            second.clone()
        } else if let Some(first) = l.boxes.first() {
            first.clone()
        } else {
            "border-box".to_string()
        }
    };
    let origin_of =
        |l: &MaskLayerSlots| -> String { l.boxes.first().cloned().unwrap_or_else(|| "border-box".to_string()) };
    let result = vec![
        mk(
            "mask-image",
            &join(
                |l| {
                    if l.reference_none {
                        "none".to_string()
                    } else {
                        l.image.clone()
                    }
                },
                "none",
            ),
        ),
        mk("mask-mode", &join(|l| l.mode.clone(), "match-source")),
        mk("mask-position", &join(|l| l.position.clone(), "0% 0%")),
        mk("mask-size", &join(|l| l.size.clone().unwrap_or_default(), "auto")),
        mk("mask-repeat", &join(|l| l.repeats.join(" "), "repeat")),
        mk("mask-origin", &join(origin_of, "border-box")),
        mk("mask-clip", &join(clip_of, "border-box")),
        mk("mask-composite", &join(|l| l.composite.clone(), "add")),
        // mask-border-* 五长手初值（§3.4–3.9 reset 名单；slice/width 初值与
        // border-image 简写不同——100%/1 vs 0/auto，同 MASK_BORDER_SHORTHAND）。
        mk("mask-border-source", "none"),
        mk("mask-border-slice", "0"),
        mk("mask-border-width", "auto"),
        mk("mask-border-outset", "0"),
        mk("mask-border-repeat", "stretch"),
    ];
    result
}

#[cfg(test)]
mod knife_mask_expand_tests {
    //! 刀 MK 写面展开钉测（语料锚定：css-masking/parsing/mask-valid.sub.html
    //! + mask-invalid.html 13 拒收形）。投影序固定：
    //!   [image, mode, position, size, repeat, origin, clip, composite, border×5]。
    use super::{MASK_LONGHANDS, expand_mask};

    /// 投影名按固定序取值（expand_mask 输出契约）。
    fn vals(value: &str) -> Vec<(String, String)> {
        expand_mask(value, false, (0, 0, 0))
            .into_iter()
            .map(|(p, v, _, _)| (p, v))
            .collect()
    }
    fn get(value: &str, prop: &str) -> String {
        vals(value)
            .iter()
            .find(|(p, _)| p == prop)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "<DROPPED>".to_string())
    }
    const GRAD: &str = "linear-gradient(to left bottom, red, blue)";

    #[test]
    fn knife_mask_expand_defaults_and_reset() {
        // 单图像形：8 mask 长手初值 + mask-border 五长手 reset（§3.10 名单）。
        let d = vals(GRAD);
        assert_eq!(d.len(), 13, "投影 13 长手（8 mask + 5 mask-border）");
        assert_eq!(get(GRAD, "mask-image"), GRAD);
        assert_eq!(get(GRAD, "mask-mode"), "match-source");
        assert_eq!(get(GRAD, "mask-position"), "0% 0%");
        assert_eq!(get(GRAD, "mask-size"), "auto");
        assert_eq!(get(GRAD, "mask-repeat"), "repeat");
        assert_eq!(get(GRAD, "mask-origin"), "border-box");
        assert_eq!(get(GRAD, "mask-clip"), "border-box");
        assert_eq!(get(GRAD, "mask-composite"), "add");
        assert_eq!(get(GRAD, "mask-border-source"), "none");
        assert_eq!(get(GRAD, "mask-border-slice"), "0");
        assert_eq!(get(GRAD, "mask-border-width"), "auto");
        assert_eq!(get(GRAD, "mask-border-outset"), "0");
        assert_eq!(get(GRAD, "mask-border-repeat"), "stretch");
    }

    #[test]
    fn knife_mask_expand_box_forms() {
        // 单盒 = origin+clip 同值（§3.10 prose）；双盒 = 首 origin 次 clip
        //（`border-box content-box` 语料形）；no-clip 旗标 → clip 字面。
        assert_eq!(get(&format!("{GRAD} stroke-box"), "mask-origin"), "stroke-box");
        assert_eq!(get(&format!("{GRAD} stroke-box"), "mask-clip"), "stroke-box");
        assert_eq!(get(&format!("{GRAD} padding-box"), "mask-clip"), "padding-box");
        assert_eq!(get("border-box content-box", "mask-origin"), "border-box");
        assert_eq!(get("border-box content-box", "mask-clip"), "content-box");
        assert_eq!(get("border-box content-box", "mask-image"), "none");
        assert_eq!(get("border-box no-clip", "mask-origin"), "border-box");
        assert_eq!(get("border-box no-clip", "mask-clip"), "no-clip");
    }

    #[test]
    fn knife_mask_expand_slots_and_full_layer() {
        // 槽位形（语料 valid）：none+mode / composite / mode / position。
        assert_eq!(get("none alpha", "mask-image"), "none");
        assert_eq!(get("none alpha", "mask-mode"), "alpha");
        assert_eq!(get(&format!("{GRAD} exclude"), "mask-composite"), "exclude");
        assert_eq!(get(&format!("{GRAD} luminance"), "mask-mode"), "luminance");
        assert_eq!(get(&format!("{GRAD} 1px 2px"), "mask-position"), "1px 2px");
        assert_eq!(get(&format!("{GRAD} 1px 2px"), "mask-size"), "auto");
        // 全槽单层（语料 valid）：image pos / size repeat box no-clip composite
        // mode 乱序 `||` 集合式解析——size 段增量消费（`/` 后仅 contain，余
        // token 溢出主流）。
        let full = format!("{GRAD} 1px 2px / contain space round stroke-box no-clip intersect luminance");
        assert_eq!(get(&full, "mask-image"), GRAD);
        assert_eq!(get(&full, "mask-position"), "1px 2px");
        assert_eq!(get(&full, "mask-size"), "contain");
        assert_eq!(get(&full, "mask-repeat"), "space round");
        assert_eq!(get(&full, "mask-origin"), "stroke-box");
        assert_eq!(get(&full, "mask-clip"), "no-clip");
        assert_eq!(get(&full, "mask-composite"), "intersect");
        assert_eq!(get(&full, "mask-mode"), "luminance");
        // view-box 换盒形（语料 valid 首层）。
        let vb = "1px 2px / contain space round view-box no-clip intersect";
        assert_eq!(get(vb, "mask-origin"), "view-box");
        assert_eq!(get(vb, "mask-clip"), "no-clip");
        assert_eq!(get(vb, "mask-composite"), "intersect");
    }

    #[test]
    fn knife_mask_expand_multilayer_join() {
        // 多层逐 longhand `, ` join（缺省层补初值——语料 computed 形
        // `0% 0%, 0% 0%` / `add, add` / `match-source, match-source` / `auto, auto`）。
        assert_eq!(get("border-box, padding-box", "mask-origin"), "border-box, padding-box");
        assert_eq!(get("border-box, padding-box", "mask-clip"), "border-box, padding-box");
        assert_eq!(get("border-box, padding-box", "mask-position"), "0% 0%, 0% 0%");
        assert_eq!(get("border-box, padding-box", "mask-repeat"), "repeat, repeat");
        assert_eq!(get("border-box, padding-box", "mask-image"), "none, none");
        // 语料 valid 双层形：首层 view-box+no-clip，次层 gradient+stroke-box+luminance。
        let two = "1px 2px / contain space round view-box no-clip intersect, linear-gradient(to left bottom, red, blue) stroke-box luminance";
        assert_eq!(get(two, "mask-image"), format!("none, {GRAD}"));
        assert_eq!(get(two, "mask-origin"), "view-box, stroke-box");
        assert_eq!(get(two, "mask-clip"), "no-clip, stroke-box");
        assert_eq!(get(two, "mask-position"), "1px 2px, 0% 0%");
        assert_eq!(get(two, "mask-size"), "contain, auto");
        assert_eq!(get(two, "mask-repeat"), "space round, repeat");
        assert_eq!(get(two, "mask-composite"), "intersect, add");
        assert_eq!(get(two, "mask-mode"), "match-source, luminance");
    }

    #[test]
    fn knife_mask_css_wide_gate() {
        // css-wide 关键字门钉（fork C 补笔防丢针——Write 相撞致门丢失时本钉
        // 首先翻红）：13 长手逐名透传同值，无 token 分类旁路。
        for kw in ["inherit", "initial", "unset", "revert", "revert-layer"] {
            let d = vals(kw);
            assert_eq!(d.len(), 13, "{kw} 应 13 长手透传");
            assert!(d.iter().all(|(_, v)| v == kw), "{kw} 逐名透传");
            assert!(d.iter().all(|(p, _)| MASK_LONGHANDS.contains(&p.as_str())));
        }
    }

    #[test]
    fn knife_mask_reject_invalid_corpus_forms() {
        // mask-invalid.html 13 拒收形逐条钉死（展开失败 → 声明丢弃）。
        let rejects = [
            // 双 reference：none+image / image 函数+url。
            "none linear-gradient(to 0% 0%, rgb(0, 0, 0), rgb(0, 0, 255))",
            "linear-gradient(to 0% 0%, rgb(0, 0, 0), rgb(0, 0, 255)) url(\"https://example.com/\")",
            // 双 mode / 双 composite。
            "none alpha luminance",
            "add intersect",
            // 双 `/` size 段（首段 contain 溢出后 `/` token 文法外拒）。
            "1px 2px / contain / cover",
            // 三四裸 length 位语（position 终验 parse_position_strict 拒）。
            "1px 2px 3px",
            "1px 2px 3px 4px",
            // 三盒 / no-clip+双盒（三形）。
            "border-box view-box padding-box",
            "content-box stroke-box no-clip",
            "stroke-box no-clip view-box",
            "no-clip border-box content-box",
            // margin-box（§3.7 geometry-box 无此值）。
            "margin-box",
            // 单轴 repeat 配对拒。
            "repeat-y repeat-x",
        ];
        for v in rejects {
            assert!(expand_mask(v, false, (0, 0, 0)).is_empty(), "应拒收：{v}");
        }
    }
}
