//! CSS 简写属性展开。
//!
//! 将 CSS 简写属性（如 `margin: 10px`）展开为对应的长属性列表
//! （如 `margin-top: 10px; margin-right: 10px; ...`）。
//!
//! 展开在级联之前，确保简写属性与长属性的特异性竞争正确处理。

/// 匹配声明类型：(属性名, 属性值, 是否 important, 特异性)
type MatchingDecl = (String, String, bool, (u32, u32, u32));
mod background;
use background::expand_background;
mod transition;
use transition::{expand_animation, expand_transition};

/// 是否为简写层支持的 CSS-wide 关键字（inherit/initial/unset，大小写不敏感）。
///
/// 简写层仅对这三个关键字做整体展开；`revert` 由级联层另行处理，故此处不含。
/// CSS 关键字大小写不敏感（CSS Syntax §：keyword），且简写收到的 value 是级联直传的
/// 原始声明值（parser 仅 lowercase 属性名，不 lowercase 值），故必须用 eq_ignore_ascii_case。
///
/// 覆盖**全部 5 个 CSS-wide 关键字**（inherit/initial/unset/revert/revert-layer，CSS Cascading
/// 4 §? / 5 §6.1），与 `cascade::is_css_wide_keyword` 对齐。R2422 修正：此前仅列前 3 个，
/// 漏 revert/revert-layer——致依赖本 helper 的简写展开器（border 全写/轴/单边、background、
/// text-emphasis）对 `border: revert`/`background: revert-layer` 等跳过 keyword 分支 → 落值解析
/// 失败 → 整条声明静默丢弃（driving: css-cascade inline-style-background.html `background:revert`）。
fn matches_css_wide_keyword(value: &str) -> bool {
    let v = value.trim();
    v.eq_ignore_ascii_case("inherit")
        || v.eq_ignore_ascii_case("initial")
        || v.eq_ignore_ascii_case("unset")
        || v.eq_ignore_ascii_case("revert")
        || v.eq_ignore_ascii_case("revert-layer")
}

/// CSS-wide 关键字透传到所有给定 longhand（R2464 shorthand+wide-keyword gap class）。
///
/// token-classifying 简写（flex-flow/flex/transition/column-rule/animation/border-image）
/// 的 expander 会把 wide keyword 当未知 token 分类失败 → 各 longhand 取默认值。本助手
/// 在 expander 顶部短路：wide keyword → 等价于对所有 longhand 各发一条 `<longhand>: <keyword>`
/// 声明（镜像 R2354 border 助手，语义 = cascade 各 longhand 独立解析关键字）。
fn wide_keyword_to_longhands(
    value: &str,
    longhands: &[&str],
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    longhands
        .iter()
        .map(|p| (p.to_string(), value.to_string(), important, specificity))
        .collect()
}

/// R2873：常见简写属性展开后的长属性名集合（用于 `var()` pending-substitution）。
///
/// 仅覆盖常见简写。未列出的简写在值含 `var()` 时走既有「展开失败 → 丢弃」行为（无回归，
/// 仅是该项不享受 pending-substitution）。
fn pending_shorthand_longhands(property: &str) -> Option<&'static [&'static str]> {
    Some(match property {
        "margin" => &["margin-top", "margin-right", "margin-bottom", "margin-left"],
        "padding" => &["padding-top", "padding-right", "padding-bottom", "padding-left"],
        "border-width" => &[
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
        ],
        "border-style" => &[
            "border-top-style",
            "border-right-style",
            "border-bottom-style",
            "border-left-style",
        ],
        "border-color" => &[
            "border-top-color",
            "border-right-color",
            "border-bottom-color",
            "border-left-color",
        ],
        "border" => &[
            "border-top-width",
            "border-top-style",
            "border-top-color",
            "border-right-width",
            "border-right-style",
            "border-right-color",
            "border-bottom-width",
            "border-bottom-style",
            "border-bottom-color",
            "border-left-width",
            "border-left-style",
            "border-left-color",
        ],
        "border-top" => &["border-top-width", "border-top-style", "border-top-color"],
        "border-right" => &["border-right-width", "border-right-style", "border-right-color"],
        "border-bottom" => &["border-bottom-width", "border-bottom-style", "border-bottom-color"],
        "border-left" => &["border-left-width", "border-left-style", "border-left-color"],
        "border-radius" => &[
            "border-top-left-radius",
            "border-top-right-radius",
            "border-bottom-right-radius",
            "border-bottom-left-radius",
        ],
        "inset" => &["top", "right", "bottom", "left"],
        "gap" => &["gap", "row-gap", "column-gap"],
        "overflow" => &["overflow-x", "overflow-y"],
        "overscroll-behavior" => &["overscroll-behavior-x", "overscroll-behavior-y"],
        "flex" => &["flex-grow", "flex-shrink", "flex-basis"],
        "flex-flow" => &["flex-direction", "flex-wrap"],
        "background" => &[
            "background-image",
            "background-position",
            "background-size",
            "background-repeat",
            "background-attachment",
            "background-origin",
            "background-clip",
            "background-color",
        ],
        "font" => &["font-style", "font-weight", "font-size", "line-height", "font-family"],
        "text-decoration" => &[
            "text-decoration-line",
            "text-decoration-style",
            "text-decoration-color",
            "text-decoration-thickness",
        ],
        "text-emphasis" => &["text-emphasis-style", "text-emphasis-color"],
        "list-style" => &["list-style-image", "list-style-position", "list-style-type"],
        "outline" => &["outline-width", "outline-style", "outline-color"],
        "columns" => &["column-count", "column-width"],
        "column-rule" => &["column-rule-width", "column-rule-style", "column-rule-color"],
        "border-image" => &[
            "border-image-source",
            "border-image-slice",
            "border-image-width",
            "border-image-outset",
            "border-image-repeat",
        ],
        "animation" => &[
            "animation-name",
            "animation-duration",
            "animation-timing-function",
            "animation-delay",
            "animation-iteration-count",
            "animation-direction",
            "animation-fill-mode",
            "animation-play-state",
        ],
        "transition" => &[
            "transition-property",
            "transition-duration",
            "transition-timing-function",
            "transition-delay",
        ],
        "place-items" => &["align-items", "justify-items"],
        "place-content" => &["align-content", "justify-content"],
        "place-self" => &["align-self", "justify-self"],
        "grid-column" => &["grid-column-start", "grid-column-end"],
        "grid-row" => &["grid-row-start", "grid-row-end"],
        "margin-block" => &["margin-block-start", "margin-block-end"],
        "margin-inline" => &["margin-inline-start", "margin-inline-end"],
        "padding-block" => &["padding-block-start", "padding-block-end"],
        "padding-inline" => &["padding-inline-start", "padding-inline-end"],
        "inset-block" => &["inset-block-start", "inset-block-end"],
        "inset-inline" => &["inset-inline-start", "inset-inline-end"],
        _ => return None,
    })
}

/// R2873：pending-substitution 标记前缀（SOH 分隔，CSS 值不可能出现 SOH）。
///
/// 标记格式：`\x01zwsp\x01{shorthand}\x01{raw_value}`。`raw_value` 仍含未解析的 `var()`；
/// 经 `resolve_env_and_var` 处理后 `var()` 被代入，前缀结构保持不变。
pub(crate) const ZWSP_SENTINEL_PREFIX: &str = "\x01zwsp\x01";

/// R2873：var() pending-substitution 第二阶段——在 `var()` 解析后，把标记为 pending 的
/// 长属性用解析后的简写值重新展开并应用回该长属性。
///
/// 规范语义（CSS Variables §2 + CSS Cascade）：含 `var()` 的简写在解析期无法展开，故把
/// 简写的每个长属性设为携带原简写文本的「pending substitution value」，各自独立参与级联；
/// `var()` 解析后，用代入后的简写值重新展开，并应用到「仍是 pending 标记」的长属性
/// （被显式长属性 cascade 覆盖的，值已不是标记，自然跳过——等价于显式长属性胜出）。
///
/// kill-switch：`ZW_SHORTHAND_VAR=0` 关闭（回退到旧行为）。
pub fn expand_pending_shorthands(
    mut resolved: std::collections::HashMap<String, String>,
) -> std::collections::HashMap<String, String> {
    if std::env::var("ZW_SHORTHAND_VAR").as_deref() == Ok("0") {
        return resolved;
    }
    let mut updates: Vec<(String, Option<String>)> = Vec::new();
    for (prop, val) in resolved.iter() {
        let Some(rest) = val.strip_prefix(ZWSP_SENTINEL_PREFIX) else {
            continue;
        };
        let Some(delim) = rest.find('\x01') else {
            continue;
        };
        let shorthand = &rest[..delim];
        // raw 已经 resolve_env_and_var 处理（var() 已代入）；仍含 var() = 解析失败 → 展开必失败。
        let raw = &rest[delim + 1..];
        let expanded = expand_one(shorthand, raw, false, (0, 0, 0));
        let new_val = expanded
            .iter()
            .find(|(p, _, _, _)| p == prop)
            .map(|(_, v, _, _)| v.clone());
        updates.push((prop.clone(), new_val));
    }
    for (prop, new_val) in updates {
        match new_val {
            Some(v) => {
                resolved.insert(prop, v);
            }
            // 简写重新展开未产生该长属性（代入后值非法）→ 该长属性按未声明处理
            //（取 initial/inherit，与规范 invalid-at-computed-time 一致）。
            None => {
                resolved.remove(&prop);
            }
        }
    }
    resolved
}

/// 展开简写属性声明。
///
/// 遍历所有匹配声明，将简写属性展开为长属性列表。
/// 非简写属性原样保留。
///
/// # 参数
///
/// - `declarations` — 原始匹配声明列表
///
/// # 返回值
///
/// 展开后的声明列表（所有属性均为长属性）
pub fn expand_shorthands(declarations: &[MatchingDecl]) -> Vec<MatchingDecl> {
    let mut result = Vec::with_capacity(declarations.len() * 2);

    for (property, value, important, specificity) in declarations {
        let expanded = expand_one(property, value, *important, *specificity);
        result.extend(expanded);
    }

    result
}

/// 展开单个声明。
fn expand_one(property: &str, value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    // R2920：vendor 前缀 shorthand 别名 canonical 化——须在简写展开前（即 expand_one 顶部）
    // 规范化。否则 `-webkit-flex`/`-webkit-transition`/`-webkit-animation` 落到下方 match 无对应
    // 分支 → 当非简写透传 → 经 cascade canonical_property_name 规范化为标准名时已是简写 →
    // apply 阶段无简写匹配（apply 只消费 longhand）→ 静默 no-op。longhand -webkit- 别名
    //（user-select/transform 簇等）在 cascade canonical_property_name 处理（R2919/R2920），不
    // 经此路径。仅显式列安全 1:1 简写别名（值语法与标准完全一致）；`-webkit-border-radius`/
    // `-webkit-background` 历史值语法差异（per-corner/elliptical/no-suffix）故排除。
    let property = match property {
        "-webkit-flex" => "flex",
        "-webkit-transition" => "transition",
        "-webkit-animation" => "animation",
        other => other,
    };
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // 简写值首尾空白守卫：consume_declaration 的 deferred-ws（R2127）已保证声明值
    // 无首尾空白 **token**，故值字符串首尾若出现空白，必来自转义序列（如
    // `background:\0020red` → 单个 ident `" red"`）。简写展开用 split_whitespace
    // 重新切分值串，会把这种转义空白当分隔符剥掉（`" red"`→`"red"`），误把非法
    // 颜色当合法应用。此处直接丢弃整个简写声明（与 chromium 一致：含转义首尾空白
    // 的简写值视为非法）。长属性（color 等）不经此路径，由 apply/parse_color 不 trim
    // 自行拒绝（R2127）。driving：escapes-014/015/016（与 R2132 tokenizer `\` 路由联合）。
    if value.starts_with(char::is_whitespace) || value.ends_with(char::is_whitespace) {
        return vec![];
    }

    // R2873：CSS 变量在简写值中的 pending-substitution。简写在 cascade 前展开，但含
    // `var()` 的值此时无法解析（var 在 cascade 后才解析），强行展开会丢弃整条简写
    // （driving：css-variables vars-font-shorthand-001 / vars-background-shorthand-001 /
    // wide-keyword-fallback-001 的 `border-style: var(--unknown, inherit)`）。改为把简写
    // 的每个长属性标记为 pending（携带原简写名+原值），var() 解析后由
    // `expand_pending_shorthands` 重新展开。kill-switch：`ZW_SHORTHAND_VAR=0`。
    if std::env::var("ZW_SHORTHAND_VAR").as_deref() != Ok("0")
        && value.contains("var(")
        && let Some(longhands) = pending_shorthand_longhands(property)
    {
        let sentinel = format!("{ZWSP_SENTINEL_PREFIX}{property}\x01{value}");
        return longhands.iter().map(|lh| mk(lh, &sentinel)).collect();
    }

    match property {
        // ── 4 边简写 ──
        "margin" => {
            // https://drafts.csswg.org/css-box-4/#margin-physical
            let Some((t, r, b, l)) = (if matches_css_wide_keyword(value) {
                parse_rect_values(value)
            } else {
                parse_rect_values_with(value, is_margin_rect_value)
            }) else {
                return vec![];
            };
            vec![
                mk("margin-top", t),
                mk("margin-right", r),
                mk("margin-bottom", b),
                mk("margin-left", l),
            ]
        }
        "padding" => {
            // https://drafts.csswg.org/css-box-4/#padding-physical
            let Some((t, r, b, l)) = (if matches_css_wide_keyword(value) {
                parse_rect_values(value)
            } else {
                parse_rect_values_with(value, is_padding_rect_value)
            }) else {
                return vec![];
            };
            vec![
                mk("padding-top", t),
                mk("padding-right", r),
                mk("padding-bottom", b),
                mk("padding-left", l),
            ]
        }

        // ── border 边简写 ──
        // https://drafts.csswg.org/css-backgrounds-3/#border-width
        // https://drafts.csswg.org/css-backgrounds-3/#border-style
        // https://drafts.csswg.org/css-backgrounds-3/#border-color
        "border-width" => {
            let Some((t, r, b, l)) = (if matches_css_wide_keyword(value) {
                parse_rect_values(value)
            } else {
                parse_rect_values_with(value, is_border_width_rect_value)
            }) else {
                return vec![];
            };
            vec![
                mk("border-top-width", t),
                mk("border-right-width", r),
                mk("border-bottom-width", b),
                mk("border-left-width", l),
            ]
        }
        "border-style" => {
            let Some((t, r, b, l)) = (if matches_css_wide_keyword(value) {
                parse_rect_values(value)
            } else {
                parse_rect_values_with(value, is_border_style_keyword)
            }) else {
                return vec![];
            };
            vec![
                mk("border-top-style", t),
                mk("border-right-style", r),
                mk("border-bottom-style", b),
                mk("border-left-style", l),
            ]
        }
        "border-color" => {
            let Some((t, r, b, l)) = (if matches_css_wide_keyword(value) {
                parse_rect_values(value)
            } else {
                parse_rect_values_with(value, looks_like_color)
            }) else {
                return vec![];
            };
            vec![
                mk("border-top-color", t),
                mk("border-right-color", r),
                mk("border-bottom-color", b),
                mk("border-left-color", l),
            ]
        }

        // ── 单边 border 简写 ──
        "border-top" => expand_border_side(
            value,
            "border-top-width",
            "border-top-style",
            "border-top-color",
            important,
            specificity,
        ),
        "border-right" => expand_border_side(
            value,
            "border-right-width",
            "border-right-style",
            "border-right-color",
            important,
            specificity,
        ),
        "border-bottom" => expand_border_side(
            value,
            "border-bottom-width",
            "border-bottom-style",
            "border-bottom-color",
            important,
            specificity,
        ),
        "border-left" => expand_border_side(
            value,
            "border-left-width",
            "border-left-style",
            "border-left-color",
            important,
            specificity,
        ),

        // ── border 逻辑属性简写（CSS Logical Properties §3）──
        // 展开为 logical longhand（border-{axis}-{side}-{width,style,color}），由
        // apply_advanced 按元素 computed writing-mode 映射到物理边。简写层不感知元素
        // 上下文，故只拆分组件、不做 logical→物理映射。
        "border-inline-start" => expand_border_side(
            value,
            "border-inline-start-width",
            "border-inline-start-style",
            "border-inline-start-color",
            important,
            specificity,
        ),
        "border-inline-end" => expand_border_side(
            value,
            "border-inline-end-width",
            "border-inline-end-style",
            "border-inline-end-color",
            important,
            specificity,
        ),
        "border-block-start" => expand_border_side(
            value,
            "border-block-start-width",
            "border-block-start-style",
            "border-block-start-color",
            important,
            specificity,
        ),
        "border-block-end" => expand_border_side(
            value,
            "border-block-end-width",
            "border-block-end-style",
            "border-block-end-color",
            important,
            specificity,
        ),

        // ── border 逻辑属性轴简写（CSS Logical Properties §3.1）──
        // https://drafts.csswg.org/css-logical-1/#border-shorthands
        // border-inline-width/style/color = 该轴 start+end 两边的同名组件，复用
        // expand_axis_logical（1 值两侧同、2 值 start/end），与 margin-inline 同模式。
        "border-inline-width" => expand_axis_logical_with(
            value,
            "border-inline-start-width",
            "border-inline-end-width",
            is_border_width_rect_value,
            important,
            specificity,
        ),
        "border-inline-style" => expand_axis_logical_with(
            value,
            "border-inline-start-style",
            "border-inline-end-style",
            is_border_style_keyword,
            important,
            specificity,
        ),
        "border-inline-color" => expand_axis_logical_with(
            value,
            "border-inline-start-color",
            "border-inline-end-color",
            looks_like_color,
            important,
            specificity,
        ),
        "border-block-width" => expand_axis_logical_with(
            value,
            "border-block-start-width",
            "border-block-end-width",
            is_border_width_rect_value,
            important,
            specificity,
        ),
        "border-block-style" => expand_axis_logical_with(
            value,
            "border-block-start-style",
            "border-block-end-style",
            is_border_style_keyword,
            important,
            specificity,
        ),
        "border-block-color" => expand_axis_logical_with(
            value,
            "border-block-start-color",
            "border-block-end-color",
            looks_like_color,
            important,
            specificity,
        ),
        // border-inline / border-block 全写：<'border-top-width'> || <'border-top-style'>
        // || <color>，应用于该轴 start+end 两边（6 个 logical longhand），见
        // expand_border_axis_logical。
        "border-inline" => expand_border_axis_logical(
            value,
            "border-inline-start",
            "border-inline-end",
            important,
            specificity,
        ),
        "border-block" => {
            expand_border_axis_logical(value, "border-block-start", "border-block-end", important, specificity)
        }

        // ── border 全写 ──
        "border" => expand_border_all(value, important, specificity),

        // ── overflow ──
        // 单值：同时应用于 overflow-x 和 overflow-y
        // 双值：第一个为 overflow-x，第二个为 overflow-y
        "overflow" => {
            let parts: Vec<&str> = value.split_whitespace().collect();
            if parts.is_empty()
                || parts.len() > 2
                || parts
                    .iter()
                    .any(|part| zero_css_parser::values::parse_overflow(part).is_none())
            {
                return vec![];
            }
            match parts.len() {
                1 => vec![mk("overflow-x", parts[0]), mk("overflow-y", parts[0])],
                2 => vec![mk("overflow-x", parts[0]), mk("overflow-y", parts[1])],
                _ => unreachable!(),
            }
        }

        // ── overscroll-behavior ──
        // 单值：同时应用于 overscroll-behavior-x 和 overscroll-behavior-y
        // 双值：第一个为 x，第二个为 y
        "overscroll-behavior" => {
            let parts: Vec<&str> = value.split_whitespace().collect();
            if parts.is_empty()
                || parts.len() > 2
                || parts
                    .iter()
                    .any(|part| zero_css_parser::values::parse_overscroll_behavior(part).is_none())
            {
                return vec![];
            }
            match parts.len() {
                1 => vec![
                    mk("overscroll-behavior-x", parts[0]),
                    mk("overscroll-behavior-y", parts[0]),
                ],
                2 => vec![
                    mk("overscroll-behavior-x", parts[0]),
                    mk("overscroll-behavior-y", parts[1]),
                ],
                _ => unreachable!(),
            }
        }

        // ── border-radius ──
        "border-radius" => expand_border_radius(value, important, specificity),

        // ── flex ──
        "flex" => expand_flex(value, important, specificity),

        // ── flex-flow: <flex-direction> || <flex-wrap> ──
        // CSS Flexbox §5.1: 当 flex-flow 省略一个组件时，缺失组件应设为初始值
        // (flex-direction: row, flex-wrap: nowrap)。始终同时输出两个子属性，
        // 确保简写正确覆盖之前的长写属性值。
        "flex-flow" => {
            if matches_css_wide_keyword(value) {
                return wide_keyword_to_longhands(value, &["flex-direction", "flex-wrap"], important, specificity);
            }
            let mut direction = None;
            let mut wrap = None;
            let parts: Vec<&str> = value.split_whitespace().collect();
            if parts.is_empty() {
                return vec![];
            }
            for token in parts {
                if matches!(token, "row" | "row-reverse" | "column" | "column-reverse") {
                    if direction.is_some() {
                        return vec![];
                    }
                    direction = Some(token);
                } else if matches!(token, "nowrap" | "wrap" | "wrap-reverse") {
                    if wrap.is_some() {
                        return vec![];
                    }
                    wrap = Some(token);
                } else {
                    return vec![];
                }
            }
            vec![
                mk("flex-direction", direction.unwrap_or("row")),
                mk("flex-wrap", wrap.unwrap_or("nowrap")),
            ]
        }

        // ── inset ──
        "inset" => {
            let Some((t, r, b, l)) = parse_rect_values(value) else {
                return vec![];
            };
            vec![mk("top", t), mk("right", r), mk("bottom", b), mk("left", l)]
        }

        // ── transition 简写 ──
        // transition: <property> <duration> <timing-function> <delay>
        // 简化实现：单组值
        "transition" => expand_transition(value, important, specificity),

        // ── 逻辑属性简写 ──
        // https://drafts.csswg.org/css-logical-1/#propdef-margin-block
        "margin-block" => expand_axis_logical_with(
            value,
            "margin-block-start",
            "margin-block-end",
            is_margin_rect_value,
            important,
            specificity,
        ),
        "margin-inline" => expand_axis_logical_with(
            value,
            "margin-inline-start",
            "margin-inline-end",
            is_margin_rect_value,
            important,
            specificity,
        ),
        // https://drafts.csswg.org/css-logical-1/#propdef-padding-block
        "padding-block" => expand_axis_logical_with(
            value,
            "padding-block-start",
            "padding-block-end",
            is_padding_rect_value,
            important,
            specificity,
        ),
        "padding-inline" => expand_axis_logical_with(
            value,
            "padding-inline-start",
            "padding-inline-end",
            is_padding_rect_value,
            important,
            specificity,
        ),
        "inset-block" => expand_axis_logical(value, "inset-block-start", "inset-block-end", important, specificity),
        "inset-inline" => expand_axis_logical(value, "inset-inline-start", "inset-inline-end", important, specificity),

        // ── animation 简写 ──
        // animation: name duration timing-function delay iteration-count direction fill-mode play-state
        "animation" => expand_animation(value, important, specificity),

        // ── Grid placement 简写 ──
        "grid-column" => expand_grid_axis(value, "grid-column-start", "grid-column-end", important, specificity),
        "grid-row" => expand_grid_axis(value, "grid-row-start", "grid-row-end", important, specificity),
        "grid-area" => expand_grid_area(value, important, specificity),

        // ── Grid 对齐简写 ──
        "place-items" => expand_place(value, "align-items", "justify-items", important, specificity),
        "place-content" => expand_place(value, "align-content", "justify-content", important, specificity),
        "place-self" => expand_place(value, "align-self", "justify-self", important, specificity),

        // ── Grid template 简写 ──
        "grid-template" => expand_grid_template(value, important, specificity),

        // ── background 简写 ──
        "background" => expand_background(value, important, specificity),

        // ── font 简写 ──
        "font" => expand_font(value, important, specificity),

        // ── font-variant 简写（CSS Fonts 4 §6.10）──
        "font-variant" => expand_font_variant(value, important, specificity),

        // ── text-decoration 简写 ──
        "text-decoration" => expand_text_decoration(value, important, specificity),

        // ── text-emphasis 简写（CSS Text Decoration 3 §3.1）──
        // text-emphasis: <text-emphasis-style> || <text-emphasis-color>
        // R2523：color 与 style 均展开为独立 longhand。
        "text-emphasis" => expand_text_emphasis(value, important, specificity),

        // ── list-style 简写 ──
        "list-style" => expand_list_style(value, important, specificity),

        // ── outline 简写 ──
        "outline" => expand_outline(value, important, specificity),

        // ── columns 简写 ──
        // columns: <column-count> <column-width>
        // 若值为纯整数 → column-count，其余设为 auto
        // 若值为长度 → column-width，其余设为 auto
        // 双值：依次为 column-count 和 column-width
        "columns" => expand_columns(value, important, specificity),

        // ── gap 简写 ──
        // https://drafts.csswg.org/css-align-3/#gap-shorthand
        // gap: <row-gap> <column-gap>
        // 单值同时应用于 gap、row-gap 和 column-gap
        "gap" => {
            if matches_css_wide_keyword(value) {
                return wide_keyword_to_longhands(value, &["gap", "row-gap", "column-gap"], important, specificity);
            }
            let parts: Vec<&str> = value.split_whitespace().collect();
            if parts.is_empty() || parts.len() > 2 || parts.iter().any(|part| !is_gap_value_token(part)) {
                return vec![];
            }
            match parts.len() {
                1 => vec![mk("gap", parts[0]), mk("row-gap", parts[0]), mk("column-gap", parts[0])],
                2 => vec![mk("gap", parts[0]), mk("row-gap", parts[0]), mk("column-gap", parts[1])],
                _ => unreachable!(),
            }
        }

        // ── column-rule 简写 ──
        // column-rule: [width] [style] [color]
        // 与 outline 类似，各部分顺序无关
        "column-rule" => expand_column_rule(value, important, specificity),

        // ── border-image 简写 ──
        "border-image" => expand_border_image(value, important, specificity),

        // ── 非简写，原样返回 ──
        _ => vec![mk(property, value)],
    }
}

/// 展开 border-image 简写。
///
/// CSS `border-image` 简写格式为：
/// `border-image: source slice / width / outset repeat`
///
/// 简化实现：
/// - 若值为 "none" → border-image-source: none
/// - 若值含 url(...) → 提取为 source
/// - 解析 slice、width（/ 后第一组）、outset（/ 后第二组）、repeat 关键字
fn expand_border_image(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(
            value,
            &[
                "border-image-source",
                "border-image-slice",
                "border-image-width",
                "border-image-outset",
                "border-image-repeat",
            ],
            important,
            specificity,
        );
    }

    // 特殊值 "none" → 仅设置 source（R2354：关键字大小写不敏感，CSS Syntax §）
    if value.eq_ignore_ascii_case("none") {
        return vec![mk("border-image-source", "none")];
    }

    let tokens = split_outside_parens(value);

    // 提取 source（url(...) 或 none）
    let mut source: Option<String> = None;
    let mut remaining: Vec<String> = Vec::new();

    for token in &tokens {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        // R2354：none 关键字大小写不敏感（CSS Syntax §）
        if source.is_none() && (t.starts_with("url(") || t.eq_ignore_ascii_case("none")) {
            source = Some(t.to_string());
        } else {
            remaining.push(t.to_string());
        }
    }

    // 处理斜杠分隔：在 remaining 中将 "/ " 转为独立 token
    // 按 "/" 分割 remaining 为最多 3 组：slice_part / width_part / outset_part
    let mut slash_groups: Vec<Vec<String>> = vec![Vec::new()];
    for token in &remaining {
        if token == "/" {
            slash_groups.push(Vec::new());
        } else {
            slash_groups.last_mut().unwrap().push(token.clone());
        }
    }

    let mut source_val = "none".to_string();
    let mut slice_val = String::new();
    let mut width_val = String::new();
    let mut outset_val = String::new();
    let mut repeat_val = String::new();

    if let Some(s) = source {
        source_val = s;
    }

    // repeat 关键字
    let is_repeat = |s: &str| matches!(s, "stretch" | "repeat" | "round" | "space");

    // 从 slash_groups 中提取值
    // 第一组（slice）：可能包含数字和 fill 关键字
    // 第二组（width）：数字/长度
    // 第三组（outset）：数字/长度
    for (gi, group) in slash_groups.iter().enumerate() {
        // 收集非 repeat 的 token 为组值，repeat 关键字单独记录
        let mut group_tokens: Vec<String> = Vec::new();
        for token in group {
            if is_repeat(token) {
                if repeat_val.is_empty() {
                    repeat_val = token.clone();
                }
            } else {
                group_tokens.push(token.clone());
            }
        }
        let group_str = group_tokens.join(" ");

        match gi {
            0 => slice_val = group_str,
            1 => width_val = group_str,
            2 => outset_val = group_str,
            _ => {}
        }
    }

    let mut result = Vec::new();
    result.push(mk("border-image-source", &source_val));
    if !slice_val.is_empty() {
        result.push(mk("border-image-slice", &slice_val));
    }
    if !width_val.is_empty() {
        result.push(mk("border-image-width", &width_val));
    }
    if !outset_val.is_empty() {
        result.push(mk("border-image-outset", &outset_val));
    }
    if !repeat_val.is_empty() {
        result.push(mk("border-image-repeat", &repeat_val));
    }
    result
}

/// 解析 4 边简写的值部分。
///
/// 返回 (top, right, bottom, left) 值字符串。
///
/// 模式：
/// - 1 值：全部相同
/// - 2 值：上下, 左右
/// - 3 值：上, 左右, 下
/// - 4 值：上, 右, 下, 左
fn parse_rect_values(value: &str) -> Option<(&str, &str, &str, &str)> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => Some((parts[0], parts[0], parts[0], parts[0])),
        2 => Some((parts[0], parts[1], parts[0], parts[1])),
        3 => Some((parts[0], parts[1], parts[2], parts[1])),
        4 => Some((parts[0], parts[1], parts[2], parts[3])),
        _ => None,
    }
}

fn parse_rect_values_with(value: &str, is_valid: fn(&str) -> bool) -> Option<(&str, &str, &str, &str)> {
    let rect = parse_rect_values(value)?;
    if [rect.0, rect.1, rect.2, rect.3].iter().all(|part| is_valid(part)) {
        Some(rect)
    } else {
        None
    }
}

fn is_border_width_rect_value(value: &str) -> bool {
    if value.eq_ignore_ascii_case("thin") || value.eq_ignore_ascii_case("medium") || value.eq_ignore_ascii_case("thick")
    {
        return true;
    }
    match zero_css_parser::values::parse_length(value) {
        Some(zero_css_parser::values::LengthValue::Px(v))
        | Some(zero_css_parser::values::LengthValue::Em(v))
        | Some(zero_css_parser::values::LengthValue::Ex(v))
        | Some(zero_css_parser::values::LengthValue::Rex(v))
        | Some(zero_css_parser::values::LengthValue::Cap(v))
        | Some(zero_css_parser::values::LengthValue::Rcap(v))
        | Some(zero_css_parser::values::LengthValue::Rem(v))
        | Some(zero_css_parser::values::LengthValue::Vh(v))
        | Some(zero_css_parser::values::LengthValue::Vw(v))
        | Some(zero_css_parser::values::LengthValue::Vmin(v))
        | Some(zero_css_parser::values::LengthValue::Vmax(v))
        | Some(zero_css_parser::values::LengthValue::Ch(v))
        | Some(zero_css_parser::values::LengthValue::Rch(v))
        | Some(zero_css_parser::values::LengthValue::Ic(v))
        | Some(zero_css_parser::values::LengthValue::Ric(v)) => v >= 0.0,
        _ => false,
    }
}

fn is_border_radius_rect_value(value: &str) -> bool {
    match zero_css_parser::values::parse_length(value) {
        Some(zero_css_parser::values::LengthValue::Px(v))
        | Some(zero_css_parser::values::LengthValue::Em(v))
        | Some(zero_css_parser::values::LengthValue::Ex(v))
        | Some(zero_css_parser::values::LengthValue::Rex(v))
        | Some(zero_css_parser::values::LengthValue::Cap(v))
        | Some(zero_css_parser::values::LengthValue::Rcap(v))
        | Some(zero_css_parser::values::LengthValue::Rem(v))
        | Some(zero_css_parser::values::LengthValue::Vh(v))
        | Some(zero_css_parser::values::LengthValue::Vw(v))
        | Some(zero_css_parser::values::LengthValue::Vmin(v))
        | Some(zero_css_parser::values::LengthValue::Vmax(v))
        | Some(zero_css_parser::values::LengthValue::Ch(v))
        | Some(zero_css_parser::values::LengthValue::Rch(v))
        | Some(zero_css_parser::values::LengthValue::Ic(v))
        | Some(zero_css_parser::values::LengthValue::Ric(v))
        | Some(zero_css_parser::values::LengthValue::Percentage(v)) => v >= 0.0,
        Some(zero_css_parser::values::LengthValue::Calc(_)) => true,
        _ => zero_css_parser::values::parse_math_function(value).is_some(),
    }
}

fn is_padding_rect_value(value: &str) -> bool {
    match zero_css_parser::values::parse_length(value) {
        Some(zero_css_parser::values::LengthValue::Px(v))
        | Some(zero_css_parser::values::LengthValue::Em(v))
        | Some(zero_css_parser::values::LengthValue::Ex(v))
        | Some(zero_css_parser::values::LengthValue::Rex(v))
        | Some(zero_css_parser::values::LengthValue::Cap(v))
        | Some(zero_css_parser::values::LengthValue::Rcap(v))
        | Some(zero_css_parser::values::LengthValue::Rem(v))
        | Some(zero_css_parser::values::LengthValue::Vh(v))
        | Some(zero_css_parser::values::LengthValue::Vw(v))
        | Some(zero_css_parser::values::LengthValue::Vmin(v))
        | Some(zero_css_parser::values::LengthValue::Vmax(v))
        | Some(zero_css_parser::values::LengthValue::Ch(v))
        | Some(zero_css_parser::values::LengthValue::Rch(v))
        | Some(zero_css_parser::values::LengthValue::Ic(v))
        | Some(zero_css_parser::values::LengthValue::Ric(v))
        | Some(zero_css_parser::values::LengthValue::Percentage(v)) => v >= 0.0,
        Some(zero_css_parser::values::LengthValue::Calc(_)) => true,
        _ => zero_css_parser::values::parse_math_function(value).is_some(),
    }
}

fn is_margin_rect_value(value: &str) -> bool {
    if value.eq_ignore_ascii_case("thin") || value.eq_ignore_ascii_case("medium") || value.eq_ignore_ascii_case("thick")
    {
        return false;
    }
    matches!(
        zero_css_parser::values::parse_length(value),
        Some(
            zero_css_parser::values::LengthValue::Px(_)
                | zero_css_parser::values::LengthValue::Em(_)
                | zero_css_parser::values::LengthValue::Ex(_)
                | zero_css_parser::values::LengthValue::Rex(_)
                | zero_css_parser::values::LengthValue::Cap(_)
                | zero_css_parser::values::LengthValue::Rcap(_)
                | zero_css_parser::values::LengthValue::Rem(_)
                | zero_css_parser::values::LengthValue::Vh(_)
                | zero_css_parser::values::LengthValue::Vw(_)
                | zero_css_parser::values::LengthValue::Vmin(_)
                | zero_css_parser::values::LengthValue::Vmax(_)
                | zero_css_parser::values::LengthValue::Ch(_)
                | zero_css_parser::values::LengthValue::Rch(_)
                | zero_css_parser::values::LengthValue::Ic(_)
                | zero_css_parser::values::LengthValue::Ric(_)
                | zero_css_parser::values::LengthValue::Percentage(_)
                | zero_css_parser::values::LengthValue::Auto
                | zero_css_parser::values::LengthValue::Calc(_)
        )
    ) || zero_css_parser::values::parse_math_function(value).is_some()
}

fn is_gap_value_token(value: &str) -> bool {
    if value.eq_ignore_ascii_case("normal") {
        return true;
    }
    if value.eq_ignore_ascii_case("thin") || value.eq_ignore_ascii_case("medium") || value.eq_ignore_ascii_case("thick")
    {
        return false;
    }
    match zero_css_parser::values::parse_length(value) {
        Some(zero_css_parser::values::LengthValue::Px(v))
        | Some(zero_css_parser::values::LengthValue::Em(v))
        | Some(zero_css_parser::values::LengthValue::Ex(v))
        | Some(zero_css_parser::values::LengthValue::Rex(v))
        | Some(zero_css_parser::values::LengthValue::Cap(v))
        | Some(zero_css_parser::values::LengthValue::Rcap(v))
        | Some(zero_css_parser::values::LengthValue::Rem(v))
        | Some(zero_css_parser::values::LengthValue::Vh(v))
        | Some(zero_css_parser::values::LengthValue::Vw(v))
        | Some(zero_css_parser::values::LengthValue::Vmin(v))
        | Some(zero_css_parser::values::LengthValue::Vmax(v))
        | Some(zero_css_parser::values::LengthValue::Ch(v))
        | Some(zero_css_parser::values::LengthValue::Rch(v))
        | Some(zero_css_parser::values::LengthValue::Ic(v))
        | Some(zero_css_parser::values::LengthValue::Ric(v))
        | Some(zero_css_parser::values::LengthValue::Percentage(v)) => v >= 0.0,
        Some(zero_css_parser::values::LengthValue::Calc(_)) => true,
        _ => zero_css_parser::values::parse_math_function(value).is_some(),
    }
}

/// 展开 border 全写（如 `border: 1px solid red`）。
///
/// 将 `border` 展开为 12 个长属性（4 边 × width/style/color）。
fn expand_border_all(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // CSS-wide keywords: 展开为所有子属性（R2354：大小写不敏感，CSS Syntax §）
    if matches_css_wide_keyword(value) {
        let mut result = Vec::with_capacity(12);
        for side in &["top", "right", "bottom", "left"] {
            result.push(mk(&format!("border-{side}-width"), value));
            result.push(mk(&format!("border-{side}-style"), value));
            result.push(mk(&format!("border-{side}-color"), value));
        }
        return result;
    }

    let parsed = parse_border_shorthand(value);
    let Some(parsed) = parsed else {
        return vec![];
    };

    let mut result = Vec::with_capacity(12);
    for side in &["top", "right", "bottom", "left"] {
        result.push(mk(&format!("border-{side}-width"), &parsed.width));
        result.push(mk(&format!("border-{side}-style"), &parsed.style));
        result.push(mk(&format!("border-{side}-color"), &parsed.color));
    }
    result
}

/// 展开 border 逻辑轴全写（`border-inline` / `border-block`，CSS Logical Properties §3.1）。
///
/// 语法 `<'border-top-width'> || <'border-top-style'> || <color>` 应用于同一逻辑轴的
/// start 与 end 两边（6 个 logical longhand：start/end × width/style/color）。与
/// `expand_border_all`（4 物理边 × 3 = 12）对称，逻辑轴仅 2 边 × 3 = 6。简写层不感知元素
/// 上下文，logical→物理映射由 apply_advanced 按元素 computed writing-mode 完成。
fn expand_border_axis_logical(
    value: &str,
    start_prefix: &str,
    end_prefix: &str,
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // CSS-wide keywords: 展开为所有 6 个子属性（R2354：大小写不敏感，CSS Syntax §）
    if matches_css_wide_keyword(value) {
        return vec![
            mk(&format!("{start_prefix}-width"), value),
            mk(&format!("{start_prefix}-style"), value),
            mk(&format!("{start_prefix}-color"), value),
            mk(&format!("{end_prefix}-width"), value),
            mk(&format!("{end_prefix}-style"), value),
            mk(&format!("{end_prefix}-color"), value),
        ];
    }

    let Some(parsed) = parse_border_shorthand(value) else {
        return vec![];
    };
    vec![
        mk(&format!("{start_prefix}-width"), &parsed.width),
        mk(&format!("{start_prefix}-style"), &parsed.style),
        mk(&format!("{start_prefix}-color"), &parsed.color),
        mk(&format!("{end_prefix}-width"), &parsed.width),
        mk(&format!("{end_prefix}-style"), &parsed.style),
        mk(&format!("{end_prefix}-color"), &parsed.color),
    ]
}

/// 展开 border 单边简写（如 `border-top: 1px solid red`）。
fn expand_border_side(
    value: &str,
    width_prop: &str,
    style_prop: &str,
    color_prop: &str,
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // CSS-wide keywords: 展开为所有子属性（R2354：大小写不敏感）
    if matches_css_wide_keyword(value) {
        return vec![mk(width_prop, value), mk(style_prop, value), mk(color_prop, value)];
    }

    let parsed = parse_border_shorthand(value);
    let Some(parsed) = parsed else {
        return vec![];
    };
    vec![
        mk(width_prop, &parsed.width),
        mk(style_prop, &parsed.style),
        mk(color_prop, &parsed.color),
    ]
}

/// border 简写解析结果。
struct BorderShorthand {
    width: String,
    style: String,
    color: String,
}

/// 解析 border 简写值（如 `1px solid red`）。
///
/// 识别 width（长度值）、style（关键字）和 color 部分，
/// 未指定的部分使用初始值。
fn parse_border_shorthand(value: &str) -> Option<BorderShorthand> {
    let toks = zero_css_parser::values::split_paren_aware_tokens(value);
    let parts: Vec<&str> = toks.iter().map(|s| s.as_str()).collect();
    let mut width = "medium".to_string();
    let mut style = "none".to_string();
    let mut color = "currentcolor".to_string();
    // CSS 规范：border 简写的 width/style/color 各至多一个；重复组件值（如
    // `border: red solid 16px red` 的两个 color）使整个声明非法，必须忽略。
    let mut seen_width = false;
    let mut seen_style = false;
    let mut seen_color = false;

    for part in parts {
        if is_border_style_keyword(part) {
            if seen_style {
                return None;
            }
            seen_style = true;
            style = part.to_string();
        } else if looks_like_length(part) {
            if seen_width {
                return None;
            }
            seen_width = true;
            width = part.to_string();
        } else if looks_like_color(part) {
            if seen_color {
                return None;
            }
            seen_color = true;
            color = part.to_string();
        } else {
            return None;
        }
    }

    Some(BorderShorthand { width, style, color })
}

/// 检查字符串是否为 border-style 关键字（R2354：大小写不敏感，CSS Syntax §）。
fn is_border_style_keyword(s: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset", "outset",
    ];
    KEYWORDS.iter().any(|k| s.eq_ignore_ascii_case(k))
}

/// 检查字符串是否看起来像长度值。
///
/// 单位后缀集须与 live `parse_length`（css-parser values/types.rs）保持一致，
/// 否则 border/outline 简写中的长度 token 会落回 medium/0 默认值
/// （R1231：缺 `ex` 致 `border-bottom: 6ex solid black` 丢 6ex → medium 3px，
/// top-091/092/bottom-091/092 簇 15-16% diff）。`%` 不列入——border-width/outline-width
/// 不接受百分比，`border: 5%` 应判非法落回默认（与 parse_length 接受 % 但 border-width
/// 拒绝无关）。`Q` 单位大小写敏感（CSS 规范 `Q`）。
fn looks_like_length(s: &str) -> bool {
    use zero_css_parser::values::LengthValue;

    matches!(
        zero_css_parser::values::parse_length(s),
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
        )
    ) || s.eq_ignore_ascii_case("thin")
        || s.eq_ignore_ascii_case("medium")
        || s.eq_ignore_ascii_case("thick")
}

fn looks_like_color(s: &str) -> bool {
    zero_css_parser::values::parse_color(s).is_some()
}

/// 展开 border-radius 简写。
///
/// https://drafts.csswg.org/css-backgrounds-3/#border-radius
/// 支持 1-4 值模式，与 4 边简写相同。
fn expand_border_radius(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    // FIXME: slash-separated elliptical radii need two-axis storage before shorthand validation can cover them.
    let Some((tl, tr, br, bl)) = (if matches_css_wide_keyword(value) || value.contains('/') {
        parse_rect_values(value)
    } else {
        parse_rect_values_with(value, is_border_radius_rect_value)
    }) else {
        return vec![];
    };
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    vec![
        mk("border-top-left-radius", tl),
        mk("border-top-right-radius", tr),
        mk("border-bottom-right-radius", br),
        mk("border-bottom-left-radius", bl),
    ]
}

/// 展开 flex 简写（CSS Flexbox §7.1）。
///
/// 语法：`none | auto | initial | [ <'flex-grow'> <'flex-shrink'>? || <'flex-basis'> ]`
///
/// 解析是 **type-based**（非位置式）：裸 `<number>` 计入 grow/shrink，任何
/// `<width>`/百分比/关键字（auto/content 等）计入 basis。因此：
/// - `none` → grow: 0, shrink: 0, basis: auto
/// - `auto` → grow: 1, shrink: 1, basis: auto
/// - `initial` → grow: 0, shrink: 1, basis: auto
/// - 单值 `<number>`（如 `flex:1`）→ grow, shrink: 1, basis: 0
/// - 单值 `<width>`（如 `flex:50%`）→ grow: 0, shrink: 1, basis: <width>
/// - 双值（如 `flex:1 2`）→ grow, shrink, basis: 0
/// - 双值（如 `flex:1 100px` / `flex:0 auto`）→ grow, shrink: 1, basis: <width>
/// - 三值 → grow, shrink, basis
fn expand_flex(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // 判定一个 token 是否为裸 `<number>`（可作 flex-grow/shrink）。
    // `f64::parse` 拒绝带单位的长度/百分比（"100px"/"50%"），但接受 "inf"/"nan"，
    // 故用 `is_finite()` 排除这两个非有效 CSS 数字的边界。
    let is_number = |s: &str| s.parse::<f64>().map(|n| n.is_finite()).unwrap_or(false);

    if value.eq_ignore_ascii_case("none") {
        return vec![mk("flex-grow", "0"), mk("flex-shrink", "0"), mk("flex-basis", "auto")];
    }
    if value.eq_ignore_ascii_case("auto") {
        return vec![mk("flex-grow", "1"), mk("flex-shrink", "1"), mk("flex-basis", "auto")];
    }
    if value.eq_ignore_ascii_case("initial") {
        return vec![mk("flex-grow", "0"), mk("flex-shrink", "1"), mk("flex-basis", "auto")];
    }
    // R2464：其余 CSS-wide 关键字（inherit/unset/revert/revert-layer）透传到 grow/shrink/basis
    //（initial 已由上式特化为 spec 初值 0/1/auto）。
    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(
            value,
            &["flex-grow", "flex-shrink", "flex-basis"],
            important,
            specificity,
        );
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        // 单值：<number> → grow；否则（<width>/关键字）→ basis
        1 => {
            if is_number(parts[0]) {
                // R2754：spec §7.1.1 省略 basis 时 flex-basis=0%（百分比，非长度 0）；
                // Chromium getComputedStyle `flex: 1` → flex-basis "0%"（oracle 核实）。
                vec![
                    mk("flex-grow", parts[0]),
                    mk("flex-shrink", "1"),
                    mk("flex-basis", "0%"),
                ]
            } else {
                vec![mk("flex-grow", "0"), mk("flex-shrink", "1"), mk("flex-basis", parts[0])]
            }
        }
        // 双值：首值=grow；次值 <number> → shrink，否则 → basis（shrink 默认 1）
        2 => {
            let (grow, second) = (parts[0], parts[1]);
            if is_number(second) {
                // R2754：同单值，省略 basis → 0%。
                vec![mk("flex-grow", grow), mk("flex-shrink", second), mk("flex-basis", "0%")]
            } else {
                vec![mk("flex-grow", grow), mk("flex-shrink", "1"), mk("flex-basis", second)]
            }
        }
        // 三值：grow, shrink, basis
        3 => vec![
            mk("flex-grow", parts[0]),
            mk("flex-shrink", parts[1]),
            mk("flex-basis", parts[2]),
        ],
        _ => vec![],
    }
}

/// 按空格分割字符串，但保留括号内的内容不分割。
fn split_outside_parens(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;

    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ' ' | '\t' if depth == 0 => {
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

/// 展开轴方向逻辑属性简写。
///
/// `margin-block: 10px` → `margin-block-start: 10px; margin-block-end: 10px`
/// `margin-block: 10px 20px` → `margin-block-start: 10px; margin-block-end: 20px`
fn expand_axis_logical(
    value: &str,
    start_prop: &str,
    end_prop: &str,
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    match parts.len() {
        1 => vec![mk(start_prop, parts[0]), mk(end_prop, parts[0])],
        2 => vec![mk(start_prop, parts[0]), mk(end_prop, parts[1])],
        _ => vec![],
    }
}

fn expand_axis_logical_with(
    value: &str,
    start_prop: &str,
    end_prop: &str,
    is_valid: fn(&str) -> bool,
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    let valid = matches_css_wide_keyword(value) || parts.iter().all(|part| is_valid(part));
    match parts.len() {
        1 if valid => vec![mk(start_prop, parts[0]), mk(end_prop, parts[0])],
        2 if valid => vec![mk(start_prop, parts[0]), mk(end_prop, parts[1])],
        _ => vec![],
    }
}

/// 展开 grid-column / grid-row 简写。
///
/// https://drafts.csswg.org/css-grid-2/#placement-shorthands
///
/// `grid-column: 1` → `grid-column-start: 1; grid-column-end: auto`
/// `grid-column: 1 / 3` → `grid-column-start: 1; grid-column-end: 3`
/// `grid-column: span 2` → `grid-column-start: span 2; grid-column-end: auto`
fn expand_grid_axis(
    value: &str,
    start_prop: &str,
    end_prop: &str,
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    if matches_css_wide_keyword(value) {
        return vec![mk(start_prop, value), mk(end_prop, value)];
    }
    if crate::property::parse::parse_grid_line_shorthand(value).is_none() {
        return vec![];
    }
    if let Some(slash_pos) = value.find('/') {
        let start = value[..slash_pos].trim();
        let end = value[slash_pos + 1..].trim();
        vec![mk(start_prop, start), mk(end_prop, end)]
    } else {
        vec![mk(start_prop, value.trim()), mk(end_prop, "auto")]
    }
}

/// 展开 grid-area 简写。
///
/// https://drafts.csswg.org/css-grid-2/#placement-shorthands
///
/// `grid-area: 1` → row-start: 1
/// `grid-area: 1 / 2` → row-start: 1, col-start: 2
/// `grid-area: 1 / 2 / 3` → row-start: 1, col-start: 2, row-end: 3
/// `grid-area: 1 / 2 / 3 / 4` → row-start: 1, col-start: 2, row-end: 3, col-end: 4
fn expand_grid_area(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(
            value,
            &["grid-row-start", "grid-row-end", "grid-column-start", "grid-column-end"],
            important,
            specificity,
        );
    }
    if crate::property::parse::parse_grid_area_shorthand(value).is_none() {
        return vec![];
    }
    // 用 `/` 分割，但 span 内部可能有空格
    let parts: Vec<&str> = value.split('/').map(|s| s.trim()).collect();
    match parts.len() {
        1 => vec![
            mk("grid-row-start", parts[0]),
            mk("grid-row-end", "auto"),
            mk("grid-column-start", "auto"),
            mk("grid-column-end", "auto"),
        ],
        2 => vec![
            mk("grid-row-start", parts[0]),
            mk("grid-row-end", "auto"),
            mk("grid-column-start", parts[1]),
            mk("grid-column-end", "auto"),
        ],
        3 => vec![
            mk("grid-row-start", parts[0]),
            mk("grid-row-end", parts[2]),
            mk("grid-column-start", parts[1]),
            mk("grid-column-end", "auto"),
        ],
        4 => vec![
            mk("grid-row-start", parts[0]),
            mk("grid-row-end", parts[2]),
            mk("grid-column-start", parts[1]),
            mk("grid-column-end", parts[3]),
        ],
        _ => vec![],
    }
}

/// 展开 place-items / place-content / place-self 简写。
///
/// https://drafts.csswg.org/css-align-3/#place-items-property
/// https://drafts.csswg.org/css-align-3/#place-content-property
/// https://drafts.csswg.org/css-align-3/#place-self-property
///
/// `place-items: center` → align-items: center; justify-items: center
/// `place-items: start end` → align-items: start; justify-items: end
///
/// 单值时两个子属性获得相同值，双值时分别对应 align 和 justify。
fn expand_place(
    value: &str,
    align_prop: &str,
    justify_prop: &str,
    important: bool,
    specificity: (u32, u32, u32),
) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    if matches_css_wide_keyword(value) {
        return vec![mk(align_prop, value), mk(justify_prop, value)];
    }
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 if is_place_longhand_value(align_prop, parts[0]) && is_place_longhand_value(justify_prop, parts[0]) => {
            vec![mk(align_prop, parts[0]), mk(justify_prop, parts[0])]
        }
        2 if is_place_longhand_value(align_prop, parts[0]) && is_place_longhand_value(justify_prop, parts[1]) => {
            vec![mk(align_prop, parts[0]), mk(justify_prop, parts[1])]
        }
        _ => vec![],
    }
}

fn is_place_longhand_value(property: &str, value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    match property {
        "align-items" | "align-self" | "justify-content" => zero_css_parser::values::parse_alignment(value).is_some(),
        "align-content" => matches!(
            lower.as_str(),
            "auto"
                | "normal"
                | "start"
                | "end"
                | "flex-start"
                | "flex-end"
                | "center"
                | "stretch"
                | "baseline"
                | "space-between"
                | "space-around"
                | "space-evenly"
        ),
        "justify-items" | "justify-self" => matches!(
            lower.as_str(),
            "auto" | "normal" | "start" | "end" | "center" | "stretch" | "baseline" | "left" | "right"
        ),
        _ => false,
    }
}

/// 展开 grid-template 简写。
///
/// 简单形式：`grid-template: <rows> / <columns>`
/// - 按 `/` 分割，第一部分为 grid-template-rows，第二部分为 grid-template-columns
/// - 若 rows 部分含引号字符串，提取为 grid-template-areas
///
/// 示例：
/// - `grid-template: 100px 200px / 1fr 1fr 1fr`
///   → grid-template-rows: 100px 200px; grid-template-columns: 1fr 1fr 1fr
/// - `grid-template: "header header" 50px "main main" 1fr / 1fr 1fr`
///   → grid-template-areas + grid-template-rows + grid-template-columns
fn expand_grid_template(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };
    let value = value.trim();

    // 按 `/` 分割为 rows 部分和 columns 部分
    if let Some(slash_pos) = value.find('/') {
        let rows_part = value[..slash_pos].trim();
        let cols_part = value[slash_pos + 1..].trim();

        // 从 rows 部分提取引号字符串作为 grid-template-areas
        let (areas_str, rows_only) = extract_quoted_areas(rows_part);

        let mut result = Vec::with_capacity(3);
        if !areas_str.is_empty() {
            result.push(mk("grid-template-areas", &areas_str));
        }
        if !rows_only.is_empty() {
            result.push(mk("grid-template-rows", rows_only.trim()));
        }
        result.push(mk("grid-template-columns", cols_part));
        result
    } else {
        // 无 `/`：整个值作为 rows
        let (areas_str, rows_only) = extract_quoted_areas(value);
        let mut result = Vec::with_capacity(2);
        if !areas_str.is_empty() {
            result.push(mk("grid-template-areas", &areas_str));
        }
        if !rows_only.is_empty() {
            result.push(mk("grid-template-rows", rows_only.trim()));
        }
        result
    }
}

/// 从 grid-template 的 rows 部分提取引号区域字符串和纯行尺寸。
///
/// `"header header" 50px "main main" 1fr` →
///   areas: `"header header" "main main"`, rows_only: `50px 1fr`
fn extract_quoted_areas(rows_part: &str) -> (String, String) {
    let mut areas = String::new();
    let mut rows_tokens: Vec<String> = Vec::new();
    let mut in_quotes = false;
    let mut current_area = String::new();
    let mut current_row_token = String::new();

    let flush_row_token = |token: &mut String, tokens: &mut Vec<String>| {
        let trimmed = token.trim().to_string();
        if !trimmed.is_empty() {
            tokens.push(trimmed);
        }
        token.clear();
    };

    for ch in rows_part.chars() {
        if ch == '"' {
            if in_quotes {
                // 闭引号
                current_area.push(ch);
                if !areas.is_empty() {
                    areas.push(' ');
                }
                areas.push_str(&current_area);
                current_area.clear();
                in_quotes = false;
            } else {
                // 开引号 — 先 flush 任何正在累积的 row token
                flush_row_token(&mut current_row_token, &mut rows_tokens);
                current_area.clear();
                current_area.push(ch);
                in_quotes = true;
            }
        } else if in_quotes {
            current_area.push(ch);
        } else {
            // 不在引号内，属于行尺寸
            current_row_token.push(ch);
        }
    }

    // flush 最后一个 row token
    flush_row_token(&mut current_row_token, &mut rows_tokens);

    (areas, rows_tokens.join(" "))
}

/// 展开 list-style 简写。
///
/// CSS `list-style` 简写格式为：
/// `list-style: [type] [position] [image]`
///
/// 识别每个部分：type 是关键字，position 是 inside/outside，其余视为 image。
/// 未指定的部分保持初始值。
fn expand_list_style(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(
            value,
            &["list-style-position", "list-style-type", "list-style-image"],
            important,
            specificity,
        );
    }

    // CSS Lists 3 §5.1：`list-style: [<'list-style-position'> || <'list-style-image'> || <'list-style-type'>]`。
    // R2487 修三处缺口：① 旧实现 is_type 硬编码枚举，漏 R2445-R2450 预定义计数器样式（lower-greek/
    // georgian/hebrew/...）与全部自定义 @counter-style 名（`Custom-Style` 等）→ 这些 token 落「其他值
    // 暂不处理」→ type 退回默认 disc；② list-style-image（url()/image()）从不展开；③ 简写未复位 image。
    // 新实现：position→position 槽；url()/image()→image 槽；其余 token（含 `none` + 任意计数器样式名/
    // 自定义 ident，**保留大小写**——计数器样式名大小写敏感）→ type 槽。`none` 同时把 type 与 image
    // 默认置 none（CSS Lists 3：「The none keyword sets list-style-image to none and list-style-type
    // to none; other values are assigned to the property they fit」）。
    let tokens = split_outside_parens(value);
    let mut list_type: Option<String> = None;
    let mut position = "outside".to_string();
    let mut image: Option<String> = None;
    let mut none_seen = false;
    let mut seen_position = false;
    let mut seen_type = false;
    let mut seen_image = false;

    for token in &tokens {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        if t.eq_ignore_ascii_case("inside") || t.eq_ignore_ascii_case("outside") {
            if seen_position {
                return vec![];
            }
            seen_position = true;
            position = t.to_ascii_lowercase();
        } else if t.starts_with("url(") || t.starts_with("image(") || t.starts_with("image-set(") {
            if seen_image {
                return vec![];
            }
            seen_image = true;
            image = Some(t.to_string());
        } else if t.eq_ignore_ascii_case("none") {
            none_seen = true;
        } else {
            if seen_type {
                return vec![];
            }
            seen_type = true;
            // 任意计数器样式（内置关键字或自定义 @counter-style 名）——保留原样大小写，
            // 由 list-style-type longhand parser 负责关键字匹配/自定义名解析。
            list_type = Some(t.to_string());
        }
    }

    // type 默认：显式 token > none_seen→none > 初始 disc。
    let final_type = list_type
        .clone()
        .or_else(|| if none_seen { Some("none".to_string()) } else { None })
        .unwrap_or_else(|| "disc".to_string());
    // image 默认 none（初始值即 none；显式 url 覆盖；`none` 关键字亦 none——简写须复位 image）。
    let final_image = image.unwrap_or_else(|| "none".to_string());

    vec![
        mk("list-style-type", &final_type),
        mk("list-style-position", &position),
        mk("list-style-image", &final_image),
    ]
}

/// 展开 outline 简写。
///
/// CSS `outline` 简写格式为：
/// `outline: [width] [style] [color]`
///
/// 各部分顺序无关，未指定的部分使用初始值。
fn expand_outline(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // "none" → 全部重置（R2354：关键字大小写不敏感）
    if value.eq_ignore_ascii_case("none") {
        return vec![
            mk("outline-width", "0px"),
            mk("outline-style", "none"),
            mk("outline-color", "currentcolor"),
        ];
    }
    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(
            value,
            &["outline-width", "outline-style", "outline-color"],
            important,
            specificity,
        );
    }

    let toks = zero_css_parser::values::split_paren_aware_tokens(value);
    let parts: Vec<&str> = toks.iter().map(|s| s.as_str()).collect();
    let mut width = "0px";
    let mut style = "none";
    let mut color = "currentcolor";
    let mut seen_width = false;
    let mut seen_style = false;
    let mut seen_color = false;

    for part in parts {
        if is_border_style_keyword(part) {
            if seen_style {
                return vec![];
            }
            seen_style = true;
            style = part;
        } else if looks_like_length(part) {
            if seen_width {
                return vec![];
            }
            seen_width = true;
            width = part;
        } else if looks_like_color(part) {
            if seen_color {
                return vec![];
            }
            seen_color = true;
            color = part;
        } else {
            return vec![];
        }
    }

    vec![
        mk("outline-width", width),
        mk("outline-style", style),
        mk("outline-color", color),
    ]
}

/// 展开 columns 简写。
///
/// CSS `columns` 简写格式为：
/// `columns: [column-count] [column-width]`
///
/// - 单个整数（如 `3`）→ column-count: 3, column-width: auto
/// - 单个长度值（如 `200px`）→ column-count: auto, column-width: 200px
/// - 双值（如 `3 200px`）→ column-count: 3, column-width: 200px
fn expand_columns(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(value, &["column-width", "column-count"], important, specificity);
    }

    /// 检查值是否为有效的 column-count 值（正整数或 auto）。
    /// CSS Multicol §3.2：column-count 须为正整数；0 非法（zero-column-width-layout：
    /// `columns: 0` 的 0 不可归 column-count，须归 column-width）。
    fn is_valid_column_count(s: &str) -> bool {
        s.eq_ignore_ascii_case("auto") || s.parse::<u32>().is_ok_and(|n| n >= 1)
    }

    /// 检查值是否为有效的 column-width 值（长度或 auto）
    fn is_valid_column_width(s: &str) -> bool {
        s.eq_ignore_ascii_case("auto") || looks_like_length(s)
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            // 单值：判断是正整数（column-count）还是长度（column-width）。
            // 0 非正整数 → 归 column-width（CSS Multicol §3.2：column-count 须 ≥1）。
            let is_positive_int = parts[0].parse::<u32>().is_ok_and(|n| n >= 1);
            if is_positive_int || parts[0] == "auto" {
                vec![mk("column-count", parts[0]), mk("column-width", "auto")]
            } else if is_valid_column_width(parts[0]) {
                vec![mk("column-count", "auto"), mk("column-width", parts[0])]
            } else {
                // 无效值 — 整个声明无效
                vec![]
            }
        }
        2 => {
            // 双值：CSS Multicol §3.4 `columns: <column-width> || <column-count>`，顺序无关。
            // 消歧规则：正整数 → column-count；长度 → column-width；auto 填余下槽位。
            // R1425 修复：旧逻辑 `parts[0]=="auto"` 把 auto 当 count 指示，致 `auto 6` 误解析为
            // column-count:auto + column-width:6（应 column-count:6 + column-width:auto），使
            // `columns: auto N` 案（如 multicol-columns-007）列数变 auto → 退回 column-width 驱动。
            let p0_int = parts[0].parse::<u32>().is_ok_and(|n| n >= 1);
            let p1_int = parts[1].parse::<u32>().is_ok_and(|n| n >= 1);
            let (count_val, width_val) = if p0_int {
                (parts[0], parts[1])
            } else if p1_int {
                (parts[1], parts[0])
            } else if looks_like_length(parts[0]) {
                // parts[0] 是长度 → width，parts[1]（auto 或长度）→ count
                (parts[1], parts[0])
            } else {
                // parts[0] 非 int/length（即 auto）→ count，parts[1] → width
                (parts[0], parts[1])
            };
            // 验证两个值都有效：count 必须是整数/auto，width 必须是长度/auto
            if is_valid_column_count(count_val) && is_valid_column_width(width_val) {
                vec![mk("column-count", count_val), mk("column-width", width_val)]
            } else {
                // 无效值（如 "8 normal"）— 整个声明无效，不覆盖先前值
                vec![]
            }
        }
        _ => vec![],
    }
}

/// 展开 column-rule 简写。
///
/// CSS `column-rule` 简写格式为：
/// `column-rule: [width] [style] [color]`
///
/// 各部分顺序无关，未指定的部分使用初始值。
fn expand_column_rule(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(
            value,
            &["column-rule-width", "column-rule-style", "column-rule-color"],
            important,
            specificity,
        );
    }

    let toks = zero_css_parser::values::split_paren_aware_tokens(value);
    let parts: Vec<&str> = toks.iter().map(|s| s.as_str()).collect();
    let mut width = "medium".to_string();
    let mut style = "none".to_string();
    let mut color = "currentcolor".to_string();
    let mut seen_width = false;
    let mut seen_style = false;
    let mut seen_color = false;

    for part in parts {
        if is_border_style_keyword(part) {
            if seen_style {
                return vec![];
            }
            seen_style = true;
            style = part.to_string();
        } else if looks_like_length(part) {
            if seen_width {
                return vec![];
            }
            seen_width = true;
            width = part.to_string();
        } else if looks_like_color(part) {
            if seen_color {
                return vec![];
            }
            seen_color = true;
            color = part.to_string();
        } else {
            return vec![];
        }
    }

    vec![
        mk("column-rule-width", &width),
        mk("column-rule-style", &style),
        mk("column-rule-color", &color),
    ]
}

/// @supports 求值用：`font` 简写值是否合法可解析（CSS Conditional §7：声明 supported
/// 的充要条件是 UA 能解析该值）。复用 `expand_font` 的严格校验（无 font-size 或负
/// line-height 等非法值 → 空 Vec = 不支持）。driving: WPT css-supports-024 `(font: 16px serif)`。
pub(crate) fn font_shorthand_supported(value: &str) -> bool {
    !expand_font(value, false, (0, 0, 0)).is_empty()
}

/// 展开 font 简写。
///
/// 简化实现：`font: [style] [weight] <size>[/<line-height>] <family>`
/// 识别 font-weight 关键字、font-size 和 line-height（通过 `/` 分隔）以及 font-family。
fn expand_font(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(
            value,
            &[
                "font-style",
                "font-weight",
                "font-size",
                "line-height",
                "font-family",
                "font-variant-ligatures",
                "font-variant-caps",
                "font-variant-numeric",
                "font-variant-east-asian",
                "font-variant-position",
                "font-variant-alternates",
                "font-stretch",
                "font-kerning",
            ],
            important,
            specificity,
        );
    }

    let mut weight = "normal".to_string();
    let mut style = "normal".to_string();
    let mut size = "medium".to_string();
    let mut line_height = "normal".to_string();
    let mut family = String::new();

    let is_weight = |s: &str| {
        matches!(
            s.to_ascii_lowercase().as_str(),
            "normal"
                | "bold"
                | "bolder"
                | "lighter"
                | "100"
                | "200"
                | "300"
                | "400"
                | "500"
                | "600"
                | "700"
                | "800"
                | "900"
        )
    };

    let is_style = |s: &str| matches!(s.to_ascii_lowercase().as_str(), "normal" | "italic" | "oblique");

    // 找到 size 部分：包含数字或者带 / 的部分
    let parts: Vec<&str> = value.split_whitespace().collect();
    let mut size_found = false;
    let mut family_parts: Vec<&str> = Vec::new();

    // R2486：`font-size / line-height` 的 `/` 允许两侧空白（CSS Fonts §4 font shorthand）。
    // spaced `/` 经 split_whitespace 成独立 token；用 expect_line_height 旗在遇 `/` 后把下一
    // token 归 line-height（而非 family）。attached `16px/1.5` 仍走 contains('/') 分支。
    let mut expect_line_height = false;
    for part in &parts {
        if expect_line_height {
            line_height = part.to_string();
            expect_line_height = false;
            continue;
        }
        if size_found {
            if *part == "/" {
                expect_line_height = true;
            } else {
                family_parts.push(part);
            }
        } else if part.contains('/') {
            // attached size/line-height 格式
            let sub: Vec<&str> = part.splitn(2, '/').collect();
            size = sub[0].to_string();
            if sub.len() > 1 {
                line_height = sub[1].to_string();
            }
            size_found = true;
        } else if !size_found && (looks_like_length(part) || part.parse::<f64>().is_ok()) {
            size = part.to_string();
            size_found = true;
        } else if !size_found && is_weight(part) {
            weight = part.to_string();
        } else if !size_found && is_style(part) {
            style = part.to_string();
        }
    }

    if !family_parts.is_empty() {
        family = family_parts.join(" ");
    }

    // CSS 规范：font 简写必须至少包含 font-size 和 font-family
    // 如果没有找到 size 部分，声明无效（除非是系统字体关键字）
    if !size_found {
        // 检查是否为系统字体关键字
        let system_fonts = ["caption", "icon", "menu", "message-box", "small-caption", "status-bar"];
        if system_fonts.contains(&value.to_ascii_lowercase().as_str()) {
            // 系统字体关键字：设置所有子属性为 normal
            return vec![
                mk("font-style", "normal"),
                mk("font-weight", "normal"),
                mk("font-size", "medium"),
                mk("line-height", "normal"),
                mk("font-family", value),
                mk("font-variant-ligatures", "normal"),
                mk("font-variant-caps", "normal"),
                mk("font-variant-numeric", "normal"),
                mk("font-variant-east-asian", "normal"),
                mk("font-variant-position", "normal"),
                mk("font-variant-alternates", "normal"),
                mk("font-stretch", "normal"),
                mk("font-kerning", "auto"),
            ];
        }
        // 无效的 font 简写声明
        return vec![];
    }

    // CSS 规范：font 简写中 line-height 为负值时，整个声明无效
    // CSS Fonts §3.7: "Values have the same meanings as for their non-shorthand equivalents.
    // Negative <line-height> values are illegal."
    if line_height.starts_with('-') {
        return vec![];
    }

    vec![
        mk("font-style", &style),
        mk("font-weight", &weight),
        mk("font-size", &size),
        mk("line-height", &line_height),
        mk("font-family", &family),
        // CSS Fonts 4 §4: font shorthand resets font-variant, font-stretch, and font-kerning.
        mk("font-variant-ligatures", "normal"),
        mk("font-variant-caps", "normal"),
        mk("font-variant-numeric", "normal"),
        mk("font-variant-east-asian", "normal"),
        mk("font-variant-position", "normal"),
        mk("font-variant-alternates", "normal"),
        mk("font-stretch", "normal"),
        mk("font-kerning", "auto"),
    ]
}

/// 展开 text-decoration 简写。
///
/// `text-decoration: [line] [style] [color]`
/// line: underline, overline, line-through, none
/// style: solid, double, dotted, dashed, wavy
/// color: 颜色值
fn expand_text_decoration(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if matches_css_wide_keyword(value) {
        return wide_keyword_to_longhands(
            value,
            &[
                "text-decoration-line",
                "text-decoration-style",
                "text-decoration-color",
                "text-decoration-thickness",
            ],
            important,
            specificity,
        );
    }

    if value.eq_ignore_ascii_case("none") {
        return vec![
            mk("text-decoration-line", "none"),
            mk("text-decoration-style", "solid"),
            mk("text-decoration-color", "currentcolor"),
            mk("text-decoration-thickness", "auto"),
        ];
    }

    let toks = zero_css_parser::values::split_paren_aware_tokens(value);
    let parts: Vec<&str> = toks.iter().map(|s| s.as_str()).collect();
    // https://drafts.csswg.org/css-text-decor-4/#text-decoration-property
    // 多值组合支持（CSS Text Decoration §3）：`text-decoration: underline overline red`
    // 需把多个 line 关键字累加为 `text-decoration-line: underline overline`（旧实现 `line = part`
    // 覆盖致仅保留最后一个）。driving: css-text-decor text-decoration-line-010/011/012/013。
    let mut line_toks: Vec<&str> = Vec::new();
    let mut dec_style = "solid".to_string();
    let mut color = "currentcolor".to_string();
    // CSS Text Decoration 4：`text-decoration` 简写含第 4 个 longhand `text-decoration-thickness`
    // （§2.3）。简写应把未显式给定的 longhand 重置为 initial（与 style:solid / color:currentcolor
    // 同谱），故默认 auto（initial）。R2592。driving: text-decoration-shorthands-001(auto)/002(100px)。
    let mut thickness = "auto".to_string();
    let mut seen_none_line = false;
    let mut seen_style = false;
    let mut seen_color = false;
    let mut seen_thickness = false;

    let is_line = |s: &str| matches!(s, "underline" | "overline" | "line-through" | "blink" | "none");

    let is_dec_style = |s: &str| matches!(s, "solid" | "double" | "dotted" | "dashed" | "wavy");

    // text-decoration-thickness 值域 = `auto | from-font | <length-percentage>`，与 line/style/color
    // 值域无交集，故未匹配前三类的 token 落到此分类（旧实现静默丢弃致 thickness 永远是 initial auto）。
    // thin/medium/thick 是 border-width 关键字非 thickness 合法值，排除避免误分类。
    let is_thickness = |s: &str| -> bool {
        if s.eq_ignore_ascii_case("auto") || s.eq_ignore_ascii_case("from-font") {
            return true;
        }
        if let Some(num) = s.strip_suffix('%') {
            return num.trim().parse::<f64>().map(|n| n.is_finite()).unwrap_or(false);
        }
        !(s.eq_ignore_ascii_case("thin") || s.eq_ignore_ascii_case("medium") || s.eq_ignore_ascii_case("thick"))
            && looks_like_length(s)
    };

    for part in &parts {
        if is_line(part) {
            if part.eq_ignore_ascii_case("none") {
                if seen_none_line || !line_toks.is_empty() {
                    return vec![];
                }
                seen_none_line = true;
            } else {
                if seen_none_line || line_toks.iter().any(|line| line.eq_ignore_ascii_case(part)) {
                    return vec![];
                }
            }
            line_toks.push(part); // 累加多值（underline overline → "underline overline"）
        } else if is_dec_style(part) {
            if seen_style {
                return vec![];
            }
            seen_style = true;
            dec_style = part.to_string();
        } else if looks_like_color(part) {
            if seen_color {
                return vec![];
            }
            seen_color = true;
            color = part.to_string();
        } else if is_thickness(part) {
            if seen_thickness {
                return vec![];
            }
            seen_thickness = true;
            thickness = part.to_string();
        } else {
            return vec![];
        }
    }
    let line = if line_toks.is_empty() {
        "none".to_string()
    } else {
        line_toks.join(" ")
    };

    vec![
        mk("text-decoration-line", &line),
        mk("text-decoration-style", &dec_style),
        mk("text-decoration-color", &color),
        mk("text-decoration-thickness", &thickness),
    ]
}

/// 展开 text-emphasis 简写（CSS Text Decoration 3 §3.1）。
///
/// `text-emphasis: <text-emphasis-style> || <text-emphasis-color>`
/// color token 展开为 text-emphasis-color longhand，剩余 token 拼回作为 style 值
/// （支持 `circle`、`filled circle`、`"*"` 等形式）。R2523。
fn expand_text_emphasis(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    // CSS-wide keywords：透传到 style longhand（R2354：大小写不敏感）
    if matches_css_wide_keyword(value) {
        return vec![mk("text-emphasis-style", value)];
    }

    let toks = zero_css_parser::values::split_paren_aware_tokens(value);
    let mut style_parts: Vec<String> = Vec::new();
    let mut color_part: Option<String> = None;
    for tok in &toks {
        if looks_like_color(tok) {
            // R2523：text-emphasis-color 现已存储，展开为独立 longhand（CSS Text Decor 3 §3）。
            color_part = Some(tok.clone());
            continue;
        }
        style_parts.push(tok.clone());
    }
    let mut out: Vec<MatchingDecl> = Vec::new();
    if let Some(c) = color_part {
        out.push(mk("text-emphasis-color", &c));
    }
    if !style_parts.is_empty() {
        out.push(mk("text-emphasis-style", &style_parts.join(" ")));
    }
    out
}

// https://drafts.csswg.org/css-fonts-4/#font-variant-prop
fn expand_font_variant(value: &str, important: bool, specificity: (u32, u32, u32)) -> Vec<MatchingDecl> {
    let value = value.trim();
    let mk = |prop: &str, val: &str| -> MatchingDecl { (prop.to_string(), val.to_string(), important, specificity) };

    if matches_css_wide_keyword(value) {
        return vec![
            mk("font-variant-ligatures", value),
            mk("font-variant-caps", value),
            mk("font-variant-numeric", value),
            mk("font-variant-east-asian", value),
            mk("font-variant-position", value),
            mk("font-variant-alternates", value),
        ];
    }

    let lower = value.to_ascii_lowercase();
    if lower == "normal" {
        return vec![
            mk("font-variant-ligatures", "normal"),
            mk("font-variant-caps", "normal"),
            mk("font-variant-numeric", "normal"),
            mk("font-variant-east-asian", "normal"),
            mk("font-variant-position", "normal"),
            mk("font-variant-alternates", "normal"),
        ];
    }
    if lower == "none" {
        return vec![
            mk("font-variant-ligatures", "none"),
            mk("font-variant-caps", "normal"),
            mk("font-variant-numeric", "normal"),
            mk("font-variant-east-asian", "normal"),
            mk("font-variant-position", "normal"),
            mk("font-variant-alternates", "normal"),
        ];
    }

    let mut ligatures: Option<String> = None;
    let mut caps: Option<String> = None;
    let mut numeric: Option<String> = None;
    let mut east_asian: Option<String> = None;
    let mut position: Option<String> = None;
    let mut alternates: Option<String> = None;

    for token in zero_css_parser::values::split_paren_aware_tokens(&lower) {
        match token.as_str() {
            // font-variant-ligatures keywords
            "no-common-ligatures"
            | "common-ligatures"
            | "no-discretionary-ligatures"
            | "discretionary-ligatures"
            | "no-historical-ligatures"
            | "historical-ligatures"
            | "no-contextual"
            | "contextual" => {
                ligatures = Some(token.to_string());
            }
            // font-variant-caps keywords
            "small-caps" | "all-small-caps" | "petite-caps" | "all-petite-caps" | "unicase" | "titling-caps" => {
                caps = Some(token.to_string());
            }
            // font-variant-numeric keywords
            "lining-nums" | "oldstyle-nums" | "proportional-nums" | "tabular-nums" | "ordinal" | "slashed-zero"
            | "diagonal-fractions" | "stacked-fractions" => {
                numeric = Some(token.to_string());
            }
            // font-variant-east-asian keywords
            "jis78" | "jis83" | "jis90" | "jis04" | "simplified" | "traditional" | "full-width"
            | "proportional-width" | "ruby" => {
                east_asian = Some(token.to_string());
            }
            // font-variant-position keywords
            "sub" | "super" => {
                position = Some(token.to_string());
            }
            "historical-forms" => {
                alternates = Some(token.to_string());
            }
            _ if zero_css_parser::values::parse_font_variant_alternates(&token).is_some() => {
                let value = alternates.get_or_insert_with(String::new);
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(&token);
            }
            _ => {}
        }
    }

    vec![
        mk("font-variant-ligatures", ligatures.as_deref().unwrap_or("normal")),
        mk("font-variant-caps", caps.as_deref().unwrap_or("normal")),
        mk("font-variant-numeric", numeric.as_deref().unwrap_or("normal")),
        mk("font-variant-east-asian", east_asian.as_deref().unwrap_or("normal")),
        mk("font-variant-position", position.as_deref().unwrap_or("normal")),
        mk("font-variant-alternates", alternates.as_deref().unwrap_or("normal")),
    ]
}

#[cfg(test)]
mod tests;
