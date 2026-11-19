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

#[test]
fn animation_range_segments_storage() {
    // apply 存储槽逐段 canonical（逗号剥离 + 段内侧别省略；gCS computed
    // 通道逐段再变换）。
    use crate::values::animation_range_segments;
    assert_eq!(
        animation_range_segments("exit 1%, cover 2%, contain 0%", AnimationRangeSide::Start),
        Some(vec!["exit 1%".to_string(), "cover 2%".to_string(), "contain".to_string()])
    );
    assert_eq!(
        animation_range_segments("contain 100%", AnimationRangeSide::End),
        Some(vec!["contain".to_string()])
    );
    assert_eq!(
        animation_range_segments("cover 42%", AnimationRangeSide::End),
        Some(vec!["cover 42%".to_string()])
    );
    assert_eq!(animation_range_segments("50% exit", AnimationRangeSide::Start), None);
}

#[test]
fn animation_range_shorthand_parse_and_serialize() {
    // shorthand 页 valid 全对映：(输入, 期望读回)。None 第三段=roundtrip。
    use crate::values::{parse_animation_range_shorthand, serialize_animation_range_shorthand};
    let cases: &[(&str, Option<&str>)] = &[
        ("normal", Some("normal")),
        ("normal normal", Some("normal")),
        ("cover", Some("cover")),
        ("contain", Some("contain")),
        ("entry", Some("entry")),
        ("entry-crossing", Some("entry-crossing")),
        ("exit", Some("exit")),
        ("exit-crossing", Some("exit-crossing")),
        ("entry, exit", Some("entry, exit")),
        ("0% 100%", Some("0%")),
        ("entry 0% entry 100%", Some("entry")),
        ("entry-crossing 0% entry-crossing 100%", Some("entry-crossing")),
        ("exit 0% exit 100%", Some("exit")),
        ("exit-crossing 0% exit-crossing 100%", Some("exit-crossing")),
        ("cover 0% cover 100%", Some("cover")),
        ("contain 0% contain 100%", Some("contain")),
        (
            "entry calc(10% - 10%) entry calc(50% + 50%)",
            Some("entry calc(0%) entry calc(100%)"),
        ),
        ("cover 50%", Some("cover 50%")),
        ("contain 50%", Some("contain 50%")),
        ("entry 50%", Some("entry 50%")),
        ("entry-crossing 50%", Some("entry-crossing 50%")),
        ("exit 50%", Some("exit 50%")),
        ("exit-crossing 50%", Some("exit-crossing 50%")),
        ("entry 50px exit 100px", Some("entry 50px exit 100px")),
        ("exit calc(10% + 50px)", Some("exit calc(10% + 50px)")),
        ("entry 50% exit 50%", Some("entry 50% exit 50%")),
        (
            "cover 50% entry 50%, contain 50% exit 50%",
            Some("cover 50% entry 50%, contain 50% exit 50%"),
        ),
        ("50% exit 50%", Some("50% exit 50%")),
        ("normal 100px", Some("normal 100px")),
        ("100px", Some("100px")),
        ("100px normal", Some("100px")),
        ("10% normal", Some("10%")),
        ("entry normal", Some("entry normal")),
        // end-zero-offset 页 6 案。
        ("normal cover 0%", Some("normal cover 0%")),
        ("entry 0% exit 0%", Some("entry exit 0%")),
        ("cover 50% exit 0%", Some("cover 50% exit 0%")),
        ("normal cover 100%", Some("normal cover")),
        ("entry 0% exit 100%", Some("entry exit")),
        ("normal exit 50%", Some("normal exit 50%")),
    ];
    for (input, expected) in cases {
        let parsed = parse_animation_range_shorthand(input);
        assert!(parsed.is_some(), "shorthand parse {input:?}");
        let (s, e) = parsed.unwrap();
        let got = serialize_animation_range_shorthand(&s, &e);
        assert_eq!(&got, expected.unwrap(), "shorthand serialize {input:?}");
    }
}

#[test]
fn animation_range_shorthand_invalid_reject() {
    // shorthand 页 invalid 全形（时间偏移/未知名/3 半/slash/色值）。
    use crate::values::parse_animation_range_shorthand;
    let invalid = [
        "0s entry 50%",
        "1s",
        "-1s",
        "1s 2s",
        "1s, 2s",
        "1s 2s, 3s",
        "1s, 2s 3s",
        "1s, 2s, 3s",
        "1s 2s 3s",
        "0s, 1s 2s 3s",
        "1s / 2s",
        "1s, 2px",
        "#ff0000",
        "red",
        "thing",
        "thing 0%",
        "thing 42%",
        "thing 100%",
        "thing 100px",
        "100% thing",
        "normal foo normal 100%",
        "normal normal 100%",
    ];
    for v in invalid {
        assert!(
            parse_animation_range_shorthand(v).is_none(),
            "shorthand invalid {v:?}"
        );
    }
}
