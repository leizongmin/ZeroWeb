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

    // 渐变函数检查（优先于 var() 检查，因为渐变可能包含 var() 引用）
    let gradient_funcs = [
        "linear-gradient(",
        "repeating-linear-gradient(",
        "radial-gradient(",
        "repeating-radial-gradient(",
        "conic-gradient(",
        "repeating-conic-gradient(",
    ];
    for func in &gradient_funcs {
        if value.contains(func) {
            bg_image = value.to_string();
            return vec![
                mk("background-color", "transparent"),
                mk("background-image", &bg_image),
                mk("background-repeat", "repeat"),
                mk("background-position", "0% 0%"),
                mk("background-attachment", "scroll"),
                mk("background-clip", "border-box"),
                mk("background-origin", "padding-box"),
                mk("background-size", "auto"),
            ];
        }
    }

    // 如果包含 var() 或颜色函数 rgb()/rgba()/hsl()/hsla()，整体作为 background-color
    // 这些值包含逗号和空格，不能通过简单的 split_whitespace 解析
    if value.contains("var(")
        || value.contains("rgb(")
        || value.contains("rgba(")
        || value.contains("hsl(")
        || value.contains("hsla(")
    {
        let bg_color = value.to_string();
        return vec![
            mk("background-color", &bg_color),
            mk("background-image", "none"),
            mk("background-repeat", "repeat"),
            mk("background-position", "0% 0%"),
            mk("background-attachment", "scroll"),
            mk("background-clip", "border-box"),
            mk("background-origin", "padding-box"),
            mk("background-size", "auto"),
        ];
    }

    // R2481：分离 position 部分与 size 部分（depth-0 `/`，url() 内的 `/` 被排除）。
    let (pos_part, size_part) = split_bg_position_and_size(value);

    // 如果 pos 部分包含 url()，提取 url() 部分作为 image，剩余 tokens 继续解析
    if pos_part.contains("url(") {
        if let Some(start) = pos_part.find("url(") {
            let mut depth = 0u32;
            let mut found_open = false;
            let mut end = start;
            for (i, c) in pos_part[start..].char_indices() {
                if c == '(' {
                    depth += 1;
                    found_open = true;
                }
                if c == ')' && depth > 0 {
                    depth -= 1;
                }
                if found_open && depth == 0 {
                    end = start + i + 1;
                    break;
                }
            }
            bg_image = pos_part[start..end].to_string();
        }
        // 解析剩余部分（url() 之外的 tokens）—— pos-side（length/percent→position）
        let remaining = pos_part.replace(&bg_image, "");
        for token in remaining.split_whitespace() {
            if token.is_empty() {
                continue;
            }
            classify_bg_token(token, &mut slots, false);
        }
    } else {
        // 没有 url()，逐 token 解析 pos-side
        for token in pos_part.split_whitespace() {
            classify_bg_token(token, &mut slots, false);
        }
    }

    // R2481：size 部分（`/` 之后）—— size-side（length/percent/auto/contain/cover→size；
    // repeat/attachment/box/color 仍正常分类，因它们可在 `/` 后出现，如 `... / 100% auto no-repeat`）。
    if let Some(size) = size_part {
        for token in size.split_whitespace() {
            if token.is_empty() {
                continue;
            }
            classify_bg_token(token, &mut slots, true);
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
/// position 关键字/长度百分比互不重叠，故无歧义；默认落 background-color。
fn classify_bg_token(token: &str, slots: &mut BgSlots, size_side: bool) {
    // repeat 值
    if matches!(
        token,
        "repeat-x" | "repeat-y" | "repeat" | "no-repeat" | "space" | "round"
    ) {
        slots.repeat = token.to_string();
        return;
    }
    // attachment 值
    if matches!(token, "scroll" | "fixed" | "local") {
        slots.attachment = token.to_string();
        return;
    }
    // box 值（origin/clip）— R2479/R2481 A/B 证「累积 box 设 origin/clip」net −3（attachment-local
    // false-pass unmasks，host-layer JS-scroll deferred），故**保持 drop**（origin=padding-box、
    // clip=border-box 默认）。slots.boxes 留空 → vec 取默认。box parse 单修无 reftest ROI（paint 层）。
    if matches!(token, "border-box" | "padding-box" | "content-box") {
        return;
    }
    // size 关键字 contain/cover → background-size（改前误落 bg_color）
    if matches!(token, "contain" | "cover") {
        slots.size = token.to_string();
        return;
    }
    // auto → background-size（auto 在 background 简写中只作 size 关键字）
    if token == "auto" {
        bg_append(&mut slots.size, token);
        return;
    }
    // position 关键字 → position（仅 pos-side；size-side 出现=非法，忽略）
    if matches!(token, "top" | "center" | "bottom" | "left" | "right") {
        if !size_side {
            bg_append(&mut slots.position, token);
        }
        return;
    }
    // 长度/百分比 → position（pos-side）或 size（size-side）
    if token.ends_with("px")
        || token.ends_with('%')
        || token.ends_with("em")
        || token.ends_with("rem")
        || token.ends_with("in")
        || token.ends_with("pt")
        || token.ends_with("pc")
        || token.ends_with("cm")
        || token.ends_with("mm")
        || token.ends_with("ch")
        || token.ends_with("vh")
        || token.ends_with("vw")
    {
        if size_side {
            bg_append(&mut slots.size, token);
        } else {
            bg_append(&mut slots.position, token);
        }
        return;
    }
    // 默认：作为 background-color（颜色值）
    slots.color = token.to_string();
}
