//! `background` 简写展开族（从 `mod.rs` 抽出，run-rules §5 文件大小控制）。
//!
//! 含 `expand_background` + 其私有辅助（bg_append / split_bg_position_and_size /
//! classify_bg_token / BgSlots）。仅 `expand_background` 对外（`pub(super)`，供
//! `mod.rs::expand_one` 调度）；其余为族内私有。

use super::{MatchingDecl, matches_css_wide_keyword};

/// 展开 background 简写。
///
/// CSS 规范要求 `background` 简写必须展开为所有子属性。
/// 此实现解析每个 token 并分类到对应的子属性：
/// - 颜色值 → background-color
/// - url() / 渐变 → background-image
/// - repeat-x/repeat-y/repeat/no-repeat → background-repeat
/// - scroll/fixed/local → background-attachment
/// - 位置关键字/长度/百分比 → background-position
/// - border-box/padding-box/content-box → background-origin / background-clip
pub(super) fn expand_background(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // CSS-wide keywords: 展开为所有子属性（R2354：大小写不敏感）
    if matches_css_wide_keyword(value) {
        let subprops = [
            "background-color",
            "background-image",
            "background-repeat",
            "background-position",
            "background-size",
            "background-attachment",
            "background-clip",
            "background-origin",
        ];
        return subprops.iter().map(|p| mk(p, value)).collect();
    }

    let mut bg_image = String::new();
    // R2481：分类累积槽（color/repeat/attachment/position/boxes/size）。/size 语境下
    // length/percent 归 size 而非 position；box 按 origin/clip 语义累积。
    let mut slots = BgSlots {
        color: String::new(),
        repeat: String::new(),
        attachment: String::new(),
        position: String::new(),
        boxes: Vec::new(),
        size: String::new(),
    };

    // R2878：函数式背景图（url()/渐变/image-set）括号感知提取为 bg_image，剩余 tokens 经
    // BgSlots 分类（color/position/repeat/attachment/size）。替代旧「含渐变 → 整个值当 image」
    // 早返回——旧路径丢失 color/position/size（driving：css-variables vars-background-shorthand-001
    // d4 `background: green linear-gradient(red,red) var(--foo,)` → R2873 var-sub 后
    // `green linear-gradient(red,red) center / 0 0`，应拆为 color=green / image=渐变 /
    // position=center / size=0 0；R2878 渲染器现消费 size:0 0 → 0×0 不可见，故 d4 = solid green）。
    // 须在 var()/rgb() color 早返回 **之前** 提取——渐变内含 rgb()/逗号会被 color 分支误吞。
    let image_funcs = [
        "url(",
        "linear-gradient(",
        "repeating-linear-gradient(",
        "radial-gradient(",
        "repeating-radial-gradient(",
        "conic-gradient(",
        "repeating-conic-gradient(",
        "image-set(",
    ];
    let earliest: Option<usize> = image_funcs.iter().filter_map(|f| value.find(f)).min();
    // working = 移除 image 函数后的剩余值（供后续 color/position/size 分类）。
    let working_owned: String = match earliest {
        Some(start) => {
            let bytes = value.as_bytes();
            let mut depth = 0i32;
            let mut found_open = false;
            let mut end = value.len();
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
            bg_image = value[start..end].to_string();
            let head = value[..start].trim();
            let tail = value[end..].trim();
            let mut s = String::new();
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
        None => value.to_string(),
    };
    let working = working_owned.trim();

    // 剩余值若含未解析 var() 或裸颜色函数 rgb()/hsl()，整体作为 background-color
    //（这些值含逗号/空格，不能 split_whitespace；图函数已在上方提取为 bg_image）
    if working.contains("var(")
        || working.contains("rgb(")
        || working.contains("rgba(")
        || working.contains("hsl(")
        || working.contains("hsla(")
    {
        let bg_color = working.to_string();
        return vec![
            mk("background-color", &bg_color),
            mk("background-image", if bg_image.is_empty() { "none" } else { &bg_image }),
            mk("background-repeat", "repeat"),
            mk("background-position", "0% 0%"),
            mk("background-attachment", "scroll"),
            mk("background-clip", "border-box"),
            mk("background-origin", "padding-box"),
            mk("background-size", "auto"),
        ];
    }

    // R2481：分离 position 部分与 size 部分（depth-0 `/`，url()/渐变内的 `/` 被排除）。
    let (pos_part, size_part) = split_bg_position_and_size(working);

    // 逐 token 分类 pos-side（图函数已提取，pos_part 仅含 color/position/repeat/attachment/box）。
    for token in pos_part.split_whitespace() {
        if token.is_empty() {
            continue;
        }
        if !classify_bg_token(token, &mut slots, false) {
            return vec![];
        }
    }

    // R2481：size 部分（`/` 之后）—— size-side（length/percent/auto/contain/cover→size；
    // repeat/attachment/box/color 仍正常分类，因它们可在 `/` 后出现，如 `... / 100% auto no-repeat`）。
    if let Some(size) = size_part {
        for token in size.split_whitespace() {
            if token.is_empty() {
                continue;
            }
            if !classify_bg_token(token, &mut slots, true) {
                return vec![];
            }
        }
    }

    vec![
        mk(
            "background-color",
            if slots.color.is_empty() {
                "transparent"
            } else {
                &slots.color
            },
        ),
        mk("background-image", if bg_image.is_empty() { "none" } else { &bg_image }),
        mk(
            "background-repeat",
            if slots.repeat.is_empty() {
                "repeat"
            } else {
                &slots.repeat
            },
        ),
        mk(
            "background-position",
            if slots.position.is_empty() {
                "0% 0%"
            } else {
                &slots.position
            },
        ),
        mk(
            "background-attachment",
            if slots.attachment.is_empty() {
                "scroll"
            } else {
                &slots.attachment
            },
        ),
        // R2481：<box> 消费——0=默认（origin padding-box / clip border-box）、1=origin&clip 同值、
        // 2=第一个 origin、第二个 clip（CSS §3.10/§3.11）。
        mk(
            "background-clip",
            slots
                .boxes
                .get(1)
                .map(String::as_str)
                .unwrap_or_else(|| slots.boxes.first().map(String::as_str).unwrap_or("border-box")),
        ),
        mk(
            "background-origin",
            slots.boxes.first().map(String::as_str).unwrap_or("padding-box"),
        ),
        mk(
            "background-size",
            if slots.size.is_empty() { "auto" } else { &slots.size },
        ),
    ]
}

/// R2481：background 简写分类累积槽。token 按 CSS Backgrounds §3.2/§3.10-§3.12 分类到
/// 各子属性；`/size` 语境（size_side=true）下长度/百分比归 background-size 而非 position。
struct BgSlots {
    color: String,
    repeat: String,
    attachment: String,
    position: String,
    /// `<box>` 值（origin/clip），按出现顺序累积：0=默认、1=origin&clip 同值、2=origin/clip。
    boxes: Vec<String>,
    size: String,
}

/// 空格累积 token 到 slot（首值直赋，后续空格连接，用于多值 position/size）。
fn bg_append(slot: &mut String, token: &str) {
    if slot.is_empty() {
        *slot = token.to_string();
    } else {
        slot.push(' ');
        slot.push_str(token);
    }
}

/// R2481：在 background 简写值中按 depth-0 `/` 分离 position 部分与 size 部分（CSS §3.4）。
/// 返回 `(position_part, Option<size_part>)`。url(...) 内的 `/`（paren-depth≥1）不作为
/// separator——如 `url(support/60x60-green.png)` 内的路径 `/` 被排除。
fn split_bg_position_and_size(value: &str) -> (&str, Option<&str>) {
    let mut depth = 0i32;
    for (i, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            '/' if depth == 0 => {
                let size = value[i + 1..].trim();
                return (value[..i].trim(), if size.is_empty() { None } else { Some(size) });
            }
            _ => {}
        }
    }
    (value.trim(), None)
}

/// 将 background 简写中的 token 分类到 `BgSlots`。
///
/// `size_side=true` 时（`/` 之后的 token）长度/百分比归 background-size 而非 position
/// （CSS Backgrounds §3.4）。分类顺序与位置无关——repeat/attachment/box/contain/cover/auto/
/// position 关键字/长度百分比互不重叠，故无歧义；无法分类时返回 false。
fn classify_bg_token(token: &str, slots: &mut BgSlots, size_side: bool) -> bool {
    // repeat 值
    if matches!(
        token,
        "repeat-x" | "repeat-y" | "repeat" | "no-repeat" | "space" | "round"
    ) {
        slots.repeat = token.to_string();
        return true;
    }
    // attachment 值
    if matches!(token, "scroll" | "fixed" | "local") {
        if !slots.attachment.is_empty() {
            return false;
        }
        slots.attachment = token.to_string();
        return true;
    }
    // box 值（origin/clip）— R2479/R2481 A/B 证「累积 box 设 origin/clip」net −3（attachment-local
    // false-pass unmasks，host-layer JS-scroll deferred），故**保持 drop**（origin=padding-box、
    // clip=border-box 默认）。slots.boxes 留空 → vec 取默认。box parse 单修无 reftest ROI（paint 层）。
    if matches!(token, "border-box" | "padding-box" | "content-box") {
        return true;
    }
    // size 关键字 contain/cover → background-size（改前误落 bg_color）
    if matches!(token, "contain" | "cover") {
        if !size_side || !slots.size.is_empty() {
            return false;
        }
        slots.size = token.to_string();
        return true;
    }
    // auto → background-size（auto 在 background 简写中只作 size 关键字）
    if token == "auto" {
        if !size_side || !can_append_background_size(slots) {
            return false;
        }
        bg_append(&mut slots.size, token);
        return true;
    }
    // position 关键字 → position（仅 pos-side；size-side 出现=非法）
    if matches!(token, "top" | "center" | "bottom" | "left" | "right") {
        if !size_side {
            bg_append(&mut slots.position, token);
            return true;
        }
        return false;
    }
    // 长度/百分比 → position（pos-side）或 size（size-side）
    if is_background_length_percentage(token) {
        if size_side {
            if !can_append_background_size(slots) {
                return false;
            }
            bg_append(&mut slots.size, token);
        } else {
            bg_append(&mut slots.position, token);
        }
        return true;
    }
    // R2878：裸 `0`（unitless-zero）是合法 `<length>`（CSS Values §：仅 0 允许无单位），
    // 归 position（pos-side）或 size（size-side）。修旧路径把 `/ 0 0` 的 bare-0 token 误归
    // background-color（driving：vars-background-shorthand-001 d4 `... / 0 0` 经简写展开）。
    // 非 0 的无单位数字对 `<length>` 非法，不在此处理（落入下方 color default）。
    if let Ok(n) = token.parse::<f32>() {
        if n == 0.0 {
            if size_side {
                if !can_append_background_size(slots) {
                    return false;
                }
                bg_append(&mut slots.size, token);
            } else {
                bg_append(&mut slots.position, token);
            }
            return true;
        }
    }
    if zero_css_parser::values::parse_color(token).is_some() {
        if !slots.color.is_empty() {
            return false;
        }
        slots.color = token.to_string();
        return true;
    }
    false
}

fn can_append_background_size(slots: &BgSlots) -> bool {
    if slots.size == "cover" || slots.size == "contain" {
        return false;
    }
    slots.size.split_whitespace().count() < 2
}

fn is_background_length_percentage(token: &str) -> bool {
    use zero_css_parser::values::LengthValue;

    matches!(
        zero_css_parser::values::parse_length(token),
        Some(
            LengthValue::Px(_)
                | LengthValue::Em(_)
                | LengthValue::Ex(_)
                | LengthValue::Rex(_)
                | LengthValue::Cap(_)
                | LengthValue::Rcap(_)
                | LengthValue::Rem(_)
                | LengthValue::Vh(_)
                | LengthValue::Vw(_)
                | LengthValue::Vmin(_)
                | LengthValue::Vmax(_)
                | LengthValue::Ch(_)
                | LengthValue::Rch(_)
                | LengthValue::Ic(_)
                | LengthValue::Ric(_)
                | LengthValue::Percentage(_)
        )
    ) || zero_css_parser::values::parse_math_function(token).is_some()
}
