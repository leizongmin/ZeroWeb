//! animation-range start/end 写面文法 + canonical 串化钉测
//!（css-scroll-animations-1 §view-timelines-ranges；css-animations/parsing
//! animation-range-* 八页 valid/invalid 断言形逐条对映——canonical 逻辑
//! 单源 [`fold_decl_specified`]，写面裁决单源 [`parse_animation_range_side`]）。

use crate::values::parse_animation_range::{
    parse_animation_range_side, AnimationRangeSide,
};
use crate::values::serialize_math::fold_decl_specified;

fn fold_start(v: &str) -> Option<String> {
    fold_decl_specified("animation-range-start", v)
}

fn fold_end(v: &str) -> Option<String> {
    fold_decl_specified("animation-range-end", v)
}

#[test]
fn animation_range_start_valid_canonical() {
    // start-valid 页逐条（(输入, 期望串化) 对）。
    let cases: &[(&str, &str)] = &[
        ("normal", "normal"),
        ("cover 0px", "cover 0px"),
        ("cover 0%", "cover"),
        ("cover 100%", "cover 100%"),
        ("cover 120%", "cover 120%"),
        ("cover 42%", "cover 42%"),
        ("0", "0px"),
        ("120%", "120%"),
        ("120px", "120px"),
        ("cover -42%", "cover -42%"),
        ("contain 42%", "contain 42%"),
        ("exit 42%", "exit 42%"),
        ("exit 1%, cover 2%, contain 0%", "exit 1%, cover 2%, contain"),
        ("exit 1%, cover 2%, contain 100%", "exit 1%, cover 2%, contain 100%"),
        ("exit-crossing 42%", "exit-crossing 42%"),
        ("entry 42px", "entry 42px"),
        ("entry-crossing 42px", "entry-crossing 42px"),
        ("entry 1em", "entry 1em"),
        ("exit calc(1em + 10px)", "exit calc(1em + 10px)"),
        ("cover", "cover"),
        ("contain", "contain"),
        ("entry", "entry"),
        ("exit", "exit"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            fold_start(input).as_deref(),
            Some(*expected),
            "start canonical {input:?}"
        );
    }
}

#[test]
fn animation_range_start_calc_canonical() {
    // calc percent-first 排序 + 同族折叠保壳（valid 页 + shorthand 页指定形）。
    assert_eq!(
        fold_start("contain calc(10px + 10%)").as_deref(),
        Some("contain calc(10% + 10px)")
    );
    assert_eq!(
        fold_start("cover calc(sign(100em - 1px) * 1%)").as_deref(),
        Some("cover calc(1% * sign(100em - 1px))")
    );
    assert_eq!(fold_start("contain calc(10% - 10%)").as_deref(), Some("contain calc(0%)"));
}

#[test]
fn animation_range_end_valid_canonical() {
    // end-valid 页逐条：End 侧 100% 省略（'contain 100%' → 'contain'）。
    let cases: &[(&str, &str)] = &[
        ("cover 42%", "cover 42%"),
        ("0", "0px"),
        ("120%", "120%"),
        ("120px", "120px"),
        ("cover -42%", "cover -42%"),
        ("contain 42%", "contain 42%"),
        ("exit 42%", "exit 42%"),
        ("exit 1%, cover 2%, contain 0%", "exit 1%, cover 2%, contain 0%"),
        ("exit 1%, cover 2%, contain 100%", "exit 1%, cover 2%, contain"),
        ("exit-crossing 42%", "exit-crossing 42%"),
        ("entry 42px", "entry 42px"),
        ("entry-crossing 42px", "entry-crossing 42px"),
        ("contain calc(10px + 10%)", "contain calc(10% + 10px)"),
        ("entry 1em", "entry 1em"),
        ("cover", "cover"),
        ("contain", "contain"),
        ("entry", "entry"),
        ("exit", "exit"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            fold_end(input).as_deref(),
            Some(*expected),
            "end canonical {input:?}"
        );
    }
}

#[test]
fn animation_range_invalid_reject() {
    // start/end-invalid 页全形 + normal 独占 + 空段。
    let invalid = [
        "50% exit",
        "contain contain",
        "none",
        "cover 50% enter 50%",
        "normal 10px",
        "normal foo",
        "50% contain",
        "50% cover",
        "50% entry",
        "50% enter",
        "normal, cover",
        "",
        "cover 42% 42%",
        "cover 10s",
    ];
    for v in invalid {
        assert!(
            parse_animation_range_side(v, AnimationRangeSide::Start).is_none(),
            "start invalid {v:?}"
        );
        assert!(
            parse_animation_range_side(v, AnimationRangeSide::End).is_none(),
            "end invalid {v:?}"
        );
    }
}

#[test]
fn animation_range_fold_none_on_invalid() {
    // fold 对非文法形 None 保留原文零漂移（写面拒收在 validate 门）。
    assert_eq!(fold_start("50% exit"), None);
    assert_eq!(fold_end("bogus"), None);
}
