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
