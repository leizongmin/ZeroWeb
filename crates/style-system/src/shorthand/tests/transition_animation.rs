use super::super::*;

#[test]
fn test_transition_shorthand_none() {
    let result = expand_one("transition", "none", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "transition-property");
    assert_eq!(result[0].1, "none");
    assert_eq!(result[1].0, "transition-duration");
    assert_eq!(result[1].1, "0s");
    assert_eq!(result[2].0, "transition-timing-function");
    assert_eq!(result[2].1, "ease");
    assert_eq!(result[3].0, "transition-delay");
    assert_eq!(result[3].1, "0s");
}

#[test]
fn test_transition_shorthand_property_duration() {
    let result = expand_one("transition", "opacity 0.3s", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "transition-property");
    assert_eq!(result[0].1, "opacity");
    assert_eq!(result[1].0, "transition-duration");
    assert_eq!(result[1].1, "0.3s");
}

#[test]
fn test_transition_shorthand_full() {
    let result = expand_one("transition", "opacity 0.3s ease-in 0.1s", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].1, "opacity");
    assert_eq!(result[1].1, "0.3s");
    assert_eq!(result[2].1, "ease-in");
    assert_eq!(result[3].1, "0.1s");
}

#[test]
fn test_transition_shorthand_with_cubic_bezier() {
    let result = expand_one(
        "transition",
        "transform 0.5s cubic-bezier(0.25, 0.1, 0.25, 1.0)",
        false,
        (0, 0, 1),
    );
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].1, "transform");
    assert_eq!(result[1].1, "0.5s");
    assert_eq!(result[2].1, "cubic-bezier(0.25, 0.1, 0.25, 1.0)");
}

#[test]
fn test_transition_shorthand_duration_only() {
    let result = expand_one("transition", "0.5s", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].1, "all"); // default property
    assert_eq!(result[1].1, "0.5s");
}

// ── R2307：多 transition 列表（CSS Transitions：<single-transition>#）──

#[test]
fn test_transition_shorthand_multiple_comma() {
    // 两条 transition：逗号分割，各 longhand 跨条目用 ", " 连接
    let result = expand_one(
        "transition",
        "width 0.3s ease 0s, height 0.5s linear 0.1s",
        false,
        (0, 0, 1),
    );
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "transition-property");
    assert_eq!(result[0].1, "width, height");
    assert_eq!(result[1].1, "0.3s, 0.5s");
    assert_eq!(result[2].1, "ease, linear");
    assert_eq!(result[3].1, "0s, 0.1s");
}

#[test]
fn test_transition_shorthand_multiple_with_cubic_bezier() {
    // paren-aware：cubic-bezier 内部逗号不分割，仍是 2 条 transition
    let result = expand_one(
        "transition",
        "transform 0.5s cubic-bezier(0.25, 0.1, 0.25, 1.0), opacity 0.2s",
        false,
        (0, 0, 1),
    );
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].1, "transform, opacity");
    assert_eq!(result[1].1, "0.5s, 0.2s");
    assert_eq!(
        result[2].1, "cubic-bezier(0.25, 0.1, 0.25, 1.0), ease",
        "cubic-bezier 内部逗号必须保持一体"
    );
}

#[test]
fn test_transition_shorthand_single_is_unchanged() {
    // 回归守护：单条 transition 输出不应含逗号（byte-identical 旧行为）
    let result = expand_one("transition", "opacity 0.3s", false, (0, 0, 1));
    assert_eq!(result[0].1, "opacity");
    assert_eq!(result[1].1, "0.3s");
    assert!(!result[0].1.contains(','));
}

// ── 逻辑属性简写测试 ──

#[test]
fn test_margin_block_shorthand_single() {
    let result = expand_one("margin-block", "10px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "margin-block-start");
    assert_eq!(result[0].1, "10px");
    assert_eq!(result[1].0, "margin-block-end");
    assert_eq!(result[1].1, "10px");
}

#[test]
fn test_margin_block_shorthand_two_values() {
    let result = expand_one("margin-block", "10px 20px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "margin-block-start");
    assert_eq!(result[0].1, "10px");
    assert_eq!(result[1].0, "margin-block-end");
    assert_eq!(result[1].1, "20px");
}

#[test]
fn test_margin_inline_shorthand() {
    let result = expand_one("margin-inline", "5px 15px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "margin-inline-start");
    assert_eq!(result[0].1, "5px");
    assert_eq!(result[1].0, "margin-inline-end");
    assert_eq!(result[1].1, "15px");
}

#[test]
fn test_padding_block_shorthand() {
    let result = expand_one("padding-block", "8px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "padding-block-start");
    assert_eq!(result[1].0, "padding-block-end");
    assert_eq!(result[0].1, "8px");
    assert_eq!(result[1].1, "8px");
}

#[test]
fn test_padding_inline_shorthand() {
    let result = expand_one("padding-inline", "3px 7px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "padding-inline-start");
    assert_eq!(result[0].1, "3px");
    assert_eq!(result[1].0, "padding-inline-end");
    assert_eq!(result[1].1, "7px");
}

#[test]
fn test_inset_block_shorthand() {
    let result = expand_one("inset-block", "100px 200px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "inset-block-start");
    assert_eq!(result[0].1, "100px");
    assert_eq!(result[1].0, "inset-block-end");
    assert_eq!(result[1].1, "200px");
}

#[test]
fn test_inset_inline_shorthand() {
    let result = expand_one("inset-inline", "50px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "inset-inline-start");
    assert_eq!(result[1].0, "inset-inline-end");
    assert_eq!(result[0].1, "50px");
    assert_eq!(result[1].1, "50px");
}

// ── border 逻辑属性轴简写（CSS Logical Properties §3.1）──

#[test]
fn test_border_inline_width_shorthand() {
    // 1 值 → start/end 同值
    let result = expand_one("border-inline-width", "2px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "border-inline-start-width");
    assert_eq!(result[0].1, "2px");
    assert_eq!(result[1].0, "border-inline-end-width");
    assert_eq!(result[1].1, "2px");
}

#[test]
fn test_border_inline_width_two_values() {
    // 2 值 → start, end
    let result = expand_one("border-inline-width", "2px 4px", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "border-inline-start-width");
    assert_eq!(result[0].1, "2px");
    assert_eq!(result[1].0, "border-inline-end-width");
    assert_eq!(result[1].1, "4px");
}

#[test]
fn test_border_block_style_and_color_shorthand() {
    let style = expand_one("border-block-style", "dashed", false, (0, 0, 1));
    assert_eq!(style.len(), 2);
    assert_eq!(style[0].0, "border-block-start-style");
    assert_eq!(style[1].0, "border-block-end-style");
    assert!(style.iter().all(|(_, v, _, _)| v == "dashed"));

    let color = expand_one("border-block-color", "red blue", false, (0, 0, 1));
    assert_eq!(color.len(), 2);
    assert_eq!(color[0].0, "border-block-start-color");
    assert_eq!(color[0].1, "red");
    assert_eq!(color[1].0, "border-block-end-color");
    assert_eq!(color[1].1, "blue");
}

#[test]
fn test_border_inline_full_shorthand() {
    // <width> || <style> || <color>，应用于 start+end 两边（6 longhand）
    let result = expand_one("border-inline", "2px solid red", false, (0, 0, 1));
    assert_eq!(result.len(), 6);
    // start 三组件
    assert_eq!(result[0].0, "border-inline-start-width");
    assert_eq!(result[0].1, "2px");
    assert_eq!(result[1].0, "border-inline-start-style");
    assert_eq!(result[1].1, "solid");
    assert_eq!(result[2].0, "border-inline-start-color");
    assert_eq!(result[2].1, "red");
    // end 三组件（与 start 同值——轴简写不支持 per-side 取值）
    assert_eq!(result[3].0, "border-inline-end-width");
    assert_eq!(result[3].1, "2px");
    assert_eq!(result[4].0, "border-inline-end-style");
    assert_eq!(result[4].1, "solid");
    assert_eq!(result[5].0, "border-inline-end-color");
    assert_eq!(result[5].1, "red");
}

#[test]
fn test_border_block_full_shorthand_partial() {
    // 仅 style → width/color 取默认（medium / currentcolor）
    let result = expand_one("border-block", "dotted", false, (0, 0, 1));
    assert_eq!(result.len(), 6);
    assert_eq!(result[1].0, "border-block-start-style");
    assert_eq!(result[1].1, "dotted");
    assert_eq!(result[0].1, "medium"); // width 默认
    assert_eq!(result[2].1, "currentcolor"); // color 默认
    assert_eq!(result[4].0, "border-block-end-style");
    assert_eq!(result[4].1, "dotted");
}

#[test]
fn test_border_inline_full_css_wide_keyword() {
    // CSS-wide keyword → 展开 6 子属性透传关键字
    let result = expand_one("border-inline", "initial", false, (0, 0, 1));
    assert_eq!(result.len(), 6);
    assert!(result.iter().all(|(_, v, _, _)| v == "initial"));
    assert_eq!(result[0].0, "border-inline-start-width");
    assert_eq!(result[5].0, "border-inline-end-color");
}

// ── animation 简写测试 ──

#[test]
fn test_animation_shorthand_none() {
    let result = expand_one("animation", "none", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].0, "animation-name");
    assert_eq!(result[0].1, "none");
}

#[test]
fn test_animation_shorthand_name_duration() {
    let result = expand_one("animation", "fadeIn 0.5s", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].1, "fadeIn");
    assert_eq!(result[1].1, "0.5s");
}

#[test]
fn test_animation_shorthand_full() {
    let result = expand_one(
        "animation",
        "slideIn 0.3s ease-in 0.1s 3 alternate forwards",
        false,
        (0, 0, 1),
    );
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].1, "slideIn"); // name
    assert_eq!(result[1].1, "0.3s"); // duration
    assert_eq!(result[2].1, "ease-in"); // timing
    assert_eq!(result[3].1, "0.1s"); // delay
    assert_eq!(result[4].1, "3"); // iteration-count
    assert_eq!(result[5].1, "alternate"); // direction
    assert_eq!(result[6].1, "forwards"); // fill-mode
}

#[test]
fn test_animation_shorthand_infinite() {
    let result = expand_one("animation", "bounce 1s linear infinite", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].1, "bounce");
    assert_eq!(result[1].1, "1s");
    assert_eq!(result[2].1, "linear");
    assert_eq!(result[4].1, "infinite");
}

#[test]
fn test_animation_shorthand_paused() {
    let result = expand_one("animation", "spin 2s paused", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].1, "spin");
    assert_eq!(result[7].1, "paused");
}

// ── R2307：多 animation 列表（CSS Animations：<single-animation>#）──

#[test]
fn test_animation_shorthand_multiple_comma() {
    // 两条 animation：逗号分割，各 longhand 跨条目用 ", " 连接
    let result = expand_one("animation", "spin 2s linear infinite, fade 1s ease 2", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].0, "animation-name");
    assert_eq!(result[0].1, "spin, fade");
    assert_eq!(result[1].1, "2s, 1s");
    assert_eq!(result[2].1, "linear, ease");
    assert_eq!(result[4].1, "infinite, 2");
}

#[test]
fn test_animation_shorthand_multiple_with_steps() {
    // paren-aware：steps() 内部逗号不分割，仍是 2 条 animation
    let result = expand_one(
        "animation",
        "bounce 0.5s steps(4, end) infinite, spin 2s",
        false,
        (0, 0, 1),
    );
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].1, "bounce, spin");
    assert_eq!(result[2].1, "steps(4, end), ease", "steps() 内部逗号必须保持一体");
}

#[test]
fn test_animation_shorthand_single_is_unchanged() {
    // 回归守护：单条 animation 输出不应含逗号（byte-identical 旧行为）
    let result = expand_one("animation", "fadeIn 0.5s", false, (0, 0, 1));
    assert_eq!(result[0].1, "fadeIn");
    assert_eq!(result[1].1, "0.5s");
    assert!(!result[0].1.contains(','));
}
