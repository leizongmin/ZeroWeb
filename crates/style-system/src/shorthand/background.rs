//! `background` 简写展开族（从 `mod.rs` 抽出，run-rules §5 文件大小控制）。
//!
//! 含 `expand_background` + 其私有辅助（bg_append / split_bg_position_and_size /
//! classify_bg_token / BgSlots）。仅 `expand_background` 对外（`pub(super)`，供
//! `mod.rs::expand_one` 调度）；其余为族内私有。

use super::{MatchingDecl, matches_css_wide_keyword};
use crate::computed::contains_var_function;

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
    expand_background_inner(value, important, specificity)
}

/// R4350：顶层逗号拆层（括号/引号感知）。至少返回 1 段。
fn split_bg_layer_list(value: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (i, ch) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if quote.is_some() => escaped = true,
            '\'' | '"' if quote == Some(ch) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(ch),
            '(' if quote.is_none() => depth += 1,
            ')' if quote.is_none() && depth > 0 => depth -= 1,
            ',' if quote.is_none() && depth == 0 => {
                let item = value[start..i].trim();
                if !item.is_empty() {
                    segments.push(item);
                }
                start = i + ch.len_utf8();
            }
            _ => {}
        }
    }
    let tail = value[start..].trim();
    if !tail.is_empty() {
        segments.push(tail);
    }
    if segments.is_empty() { vec![value] } else { segments }
}

/// R4350：单层 bg-layer 解析——image 函数括号感知提取 + BgSlots 分类。
/// 返回 `(image, slots)`；None = 层内含 var()（跨层 var 语义单层解析不可判定，
/// 整条放弃交由 var 失败兜底）或 token 分类失败（非法简写，与单层路径 vec![] 同口径）。
fn parse_background_layer(layer: &str) -> Option<(String, BgSlots)> {
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
    let lower = layer.to_ascii_lowercase();
    let earliest: Option<usize> = image_funcs.iter().filter_map(|f| lower.find(f)).min();
    let mut bg_image = String::new();
    let working_owned: String = match earliest {
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
            bg_image = layer[start..end].to_string();
            let head = layer[..start].trim();
            let tail = layer[end..].trim();
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
        None => layer.to_string(),
    };
    let working = working_owned.trim();
    if contains_var_function(working) {
        return None;
    }
    let mut slots = BgSlots {
        color: String::new(),
        repeat: String::new(),
        attachment: String::new(),
        position: String::new(),
        boxes: Vec::new(),
        size: String::new(),
    };
    // 颜色函数（rgb()/hsl()）整段作为 color（与单层臂同款 lenient——层内 color 位置）。
    if working.contains("rgb(") || working.contains("rgba(") || working.contains("hsl(") || working.contains("hsla(") {
        slots.color = working.to_string();
        return Some((bg_image, slots));
    }
    let (pos_part, size_part) = split_bg_position_and_size(working);
    let pos_tokens = crate::shorthand::split_top_level_whitespace(pos_part)?;
    for token in pos_tokens {
        if !classify_bg_token(token, &mut slots, false) {
            return None;
        }
    }
    if let Some(size) = size_part {
        let size_tokens = crate::shorthand::split_top_level_whitespace(size)?;
        for token in size_tokens {
            if !classify_bg_token(token, &mut slots, true) {
                return None;
            }
        }
    }
    Some((bg_image, slots))
}

/// R4350：多层简写展开——逐层解析（复用单层分类器），逐 longhand 逗号连接。
/// color 仅取最后一层（spec：color 只允许在最后一层声明，非末层的 color lenient 忽略）；
/// attachment/clip/origin 单值存储限制 → 取首层（slice 2 记账）。
fn expand_background_layers(segments: &[&str], important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    let mut layers = Vec::new();
    for seg in segments {
        match parse_background_layer(seg) {
            Some(layer) => layers.push(layer),
            None => return vec![],
        }
    }
    let with_default = |slot: &String, default: &str| -> String {
        if slot.is_empty() {
            default.to_string()
        } else {
            slot.clone()
        }
    };
    let images: Vec<String> = layers
        .iter()
        .map(|(img, _)| {
            if img.is_empty() {
                "none".to_string()
            } else {
                img.clone()
            }
        })
        .collect();
    let repeats: Vec<String> = layers.iter().map(|(_, s)| with_default(&s.repeat, "repeat")).collect();
    let positions: Vec<String> = layers.iter().map(|(_, s)| with_default(&s.position, "0% 0%")).collect();
    let sizes: Vec<String> = layers.iter().map(|(_, s)| with_default(&s.size, "auto")).collect();
    let first = layers.first().map(|(_, s)| s.clone()).unwrap_or(BgSlots {
        color: String::new(),
        repeat: String::new(),
        attachment: String::new(),
        position: String::new(),
        boxes: Vec::new(),
        size: String::new(),
    });
    let color = layers
        .last()
        .map(|(_, s)| {
            if s.color.is_empty() {
                "transparent".to_string()
            } else {
                s.color.clone()
            }
        })
        .unwrap_or_else(|| "transparent".to_string());
    let result = vec![
        mk("background-color", &color),
        mk("background-image", &images.join(", ")),
        mk("background-repeat", &repeats.join(", ")),
        mk("background-position", &positions.join(", ")),
        // attachment/clip/origin：单值存储（slice 2）——首层。
        mk(
            "background-attachment",
            with_default(&first.attachment, "scroll").as_str(),
        ),
        mk(
            "background-clip",
            first
                .boxes
                .get(1)
                .map(String::as_str)
                .unwrap_or_else(|| first.boxes.first().map(String::as_str).unwrap_or("border-box")),
        ),
        mk(
            "background-origin",
            first.boxes.first().map(String::as_str).unwrap_or("padding-box"),
        ),
        mk("background-size", &sizes.join(", ")),
    ];
    if background_longhands_are_valid(&result) {
        result
    } else {
        vec![]
    }
}

/// 单层/分发入口（既有实现，R2481/R2878/R3753 谱系 + R4350 分层分发）。
fn expand_background_inner(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
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

    // R4350：background 简写语法 = `<bg-layer>#`（css-backgrounds-3 §2.1 逗号分层，
    // color 仅最后一层合法）。顶层逗号拆层（括号/引号感知——渐变/rgb() 内逗号不分层）；
    // 多层逐层解析后逐 longhand 逗号连接（image/repeat/position/size 走既有 Vec 存储，
    // attachment/clip/origin 为单值存储 → 取首层，slice 2 记账）。
    // 单层路径与既有实现逐字节一致（下方原 body）。
    let segments = split_bg_layer_list(value);
    if segments.len() > 1 {
        return expand_background_layers(&segments, important, specificity);
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
    // CSS function names are ASCII case-insensitive.
    let lower_value = value.to_ascii_lowercase();
    let earliest: Option<usize> = image_funcs.iter().filter_map(|f| lower_value.find(f)).min();
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
    if contains_var_function(working)
        || working.contains("rgb(")
        || working.contains("rgba(")
        || working.contains("hsl(")
        || working.contains("hsla(")
    {
        let bg_color = working.to_string();
        let result = vec![
            mk("background-color", &bg_color),
            mk("background-image", if bg_image.is_empty() { "none" } else { &bg_image }),
            mk("background-repeat", "repeat"),
            mk("background-position", "0% 0%"),
            mk("background-attachment", "scroll"),
            mk("background-clip", "border-box"),
            mk("background-origin", "padding-box"),
            mk("background-size", "auto"),
        ];
        return if background_longhands_are_valid(&result) {
            result
        } else {
            vec![]
        };
    }

    // R2481：分离 position 部分与 size 部分（depth-0 `/`，url()/渐变内的 `/` 被排除）。
    let (pos_part, size_part) = split_bg_position_and_size(working);

    // R3753：math 函数内部空白不是组件边界（`background: red min(0%, 100%) no-repeat`）。
    // 逐 token 分类 pos-side（图函数已提取，pos_part 仅含 color/position/repeat/attachment/box）。
    let Some(pos_tokens) = crate::shorthand::split_top_level_whitespace(pos_part) else {
        return vec![];
    };
    for token in pos_tokens {
        if !classify_bg_token(token, &mut slots, false) {
            return vec![];
        }
    }

    // R2481：size 部分（`/` 之后）—— size-side（length/percent/auto/contain/cover→size；
    // repeat/attachment/box/color 仍正常分类，因它们可在 `/` 后出现，如 `... / 100% auto no-repeat`）。
    if let Some(size) = size_part {
        let Some(size_tokens) = crate::shorthand::split_top_level_whitespace(size) else {
            return vec![];
        };
        for token in size_tokens {
            if !classify_bg_token(token, &mut slots, true) {
                return vec![];
            }
        }
    }

    let result = vec![
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
    ];
    if !background_longhands_are_valid(&result) {
        return vec![];
    }
    result
}

fn background_longhands_are_valid(decls: &[MatchingDecl]) -> bool {
    decls.iter().all(|(property, value, _, _)| match property.as_str() {
        "background-color" => zero_css_parser::values::parse_color(value).is_some(),
        // R4350：image 接受单值或逗号多层（parse_background_image_layers）。
        "background-image" => {
            zero_css_parser::values::parse_background_image(value).is_some()
                || zero_css_parser::values::parse_background_image_layers(value).is_some()
        }
        // R4350：Vec 存储的 longhand（repeat/position/size）接受逗号多层——
        // 逐层校验（split_bg_layer_list 括号/引号感知，单值返回单段）。
        "background-repeat" => split_bg_layer_list(value)
            .iter()
            .all(|p| zero_css_parser::values::parse_background_repeat(p).is_some()),
        "background-position" => split_bg_layer_list(value)
            .iter()
            .all(|p| zero_css_parser::values::parse_background_position(p).is_some()),
        "background-attachment" => zero_css_parser::values::parse_background_attachment(value).is_some(),
        "background-clip" => zero_css_parser::values::parse_background_clip(value).is_some(),
        "background-origin" => zero_css_parser::values::parse_background_origin(value).is_some(),
        "background-size" => split_bg_layer_list(value)
            .iter()
            .all(|p| zero_css_parser::values::parse_background_size(p).is_some()),
        _ => false,
    })
}

/// R2481：background 简写分类累积槽。token 按 CSS Backgrounds §3.2/§3.10-§3.12 分类到
/// 各子属性；`/size` 语境（size_side=true）下长度/百分比归 background-size 而非 position。
/// R4350：derive Clone（多层展开按层收集槽后逐 longhand 连接）。
#[derive(Clone)]
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
    // R3753：math size（`min(50%, 25%)`）是一个组件，函数内部空白不计。
    crate::shorthand::split_top_level_whitespace(&slots.size).is_some_and(|parts| parts.len() < 2)
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
