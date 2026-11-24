//! columns 简写槽分派与指定值 canonical 串化（css-multicol-2 §4.5）。
//!
//! 期望表锚定 css-multicol/parsing columns-valid 语料（test_valid_value 通道）
//! 与插值语料 columns-interpolation 声明形；apply 臂消费同一
//! [`crate::values::columns_shorthand_slots`]（style-system cascade apply /
//! CSS.supports apply oracle / 插值 canonical 串回灌共用）。

use crate::values::{columns_shorthand_slots, serialize_columns_canonical};

fn slots(value: &str) -> (Option<String>, Option<String>, Option<String>) {
    columns_shorthand_slots(value).expect("expected valid columns grammar form")
}

#[test]
fn columns_slots_order_insensitive_two_component() {
    // width 先 count 后：`2 10px` → width=10px count=2（声明序无关）
    assert_eq!(slots("2 10px"), (Some("10px".into()), Some("2".into()), None));
    assert_eq!(slots("10px 2"), (Some("10px".into()), Some("2".into()), None));
    // unitless zero 归 width 槽（column-width 语法允许 <length [0,∞]>）
    assert_eq!(slots("0 1"), (Some("0".into()), Some("1".into()), None));
    assert_eq!(slots("1 0"), (Some("0".into()), Some("1".into()), None));
    // em 槽保留原文
    assert_eq!(slots("auto 10em"), (Some("10em".into()), Some("auto".into()), None));
    assert_eq!(slots("10em auto"), (Some("10em".into()), Some("auto".into()), None));
}

#[test]
fn columns_slots_auto_fill_and_bare_forms() {
    assert_eq!(slots("auto 3"), (Some("auto".into()), Some("3".into()), None));
    assert_eq!(slots("3 auto"), (Some("auto".into()), Some("3".into()), None));
    assert_eq!(slots("auto auto"), (Some("auto".into()), Some("auto".into()), None));
    // 裸 `auto`：单 auto 补 width 槽，count 槽缺省（reset 语义同归 auto）
    assert_eq!(slots("auto"), (Some("auto".into()), None, None));
    assert_eq!(slots("7"), (None, Some("7".into()), None));
    assert_eq!(slots("7em"), (Some("7em".into()), None, None));
}

#[test]
fn columns_slots_height_slash_forms() {
    assert_eq!(
        slots("10px 2 / 100px"),
        (Some("10px".into()), Some("2".into()), Some("100px".into()))
    );
    // count / height 形（`10 / 100px`：count=10、height=100px）
    assert_eq!(slots("10 / 100px"), (None, Some("10".into()), Some("100px".into())));
    // width / height 形
    assert_eq!(
        slots("100px / 200px"),
        (Some("100px".into()), None, Some("200px".into()))
    );
    // auto / height：双 auto + height
    assert_eq!(slots("auto / 100px"), (Some("auto".into()), None, Some("100px".into())));
    // height 显式 auto → None（串化省略、apply 归 Auto）
    assert_eq!(slots("10px 2 / auto"), (Some("10px".into()), Some("2".into()), None));
    assert_eq!(slots("auto / auto"), (Some("auto".into()), None, None));
    // calc height（math 函数内部空白不破分割，R3749）
    assert_eq!(
        slots("10px / calc(2em + 4px)"),
        (Some("10px".into()), None, Some("calc(2em + 4px)".into()))
    );
    // calc width（R3749 同通道）
    assert_eq!(
        slots("2 calc(6em + 5px)"),
        (Some("calc(6em + 5px)".into()), Some("2".into()), None)
    );
}

#[test]
fn columns_slots_invalid_forms_rejected() {
    // 前置组空 / 超 2、斜杠后组空 / 超 1
    assert!(columns_shorthand_slots("").is_none());
    assert!(columns_shorthand_slots("/ 100px").is_none());
    assert!(columns_shorthand_slots("10px 2 3").is_none());
    assert!(columns_shorthand_slots("10px /").is_none());
    assert!(columns_shorthand_slots("10px / 100px 2").is_none());
    // 同槽重复 / 未知 token / 负数
    assert!(columns_shorthand_slots("2 3").is_none());
    assert!(columns_shorthand_slots("10px 20px").is_none());
    assert!(columns_shorthand_slots("3 3").is_none());
    assert!(columns_shorthand_slots("thin").is_none());
    assert!(columns_shorthand_slots("-10px 2").is_none());
    assert!(columns_shorthand_slots("0 0").is_none());
    // css-wide 关键字非本简写文法（串化通道 None 保留原文）
    assert!(columns_shorthand_slots("inherit").is_none());
    assert!(columns_shorthand_slots("initial").is_none());
}

#[test]
fn columns_canonical_serialization_matches_parsing_corpus() {
    // columns-valid.html test_valid_value 期望表逐条
    let s = |v: &str| {
        let (w, c, h) = slots(v);
        serialize_columns_canonical(w.as_deref(), c.as_deref(), h.as_deref())
    };
    assert_eq!(s("auto 3"), "3");
    assert_eq!(s("auto 10em"), "10em");
    assert_eq!(s("3 auto"), "3");
    assert_eq!(s("10em auto"), "10em");
    assert_eq!(s("2 10px"), "10px 2");
    assert_eq!(s("10px 2"), "10px 2");
    assert_eq!(s("auto"), "auto");
    assert_eq!(s("auto auto"), "auto");
    assert_eq!(s("7"), "7");
    assert_eq!(s("7em"), "7em");
    assert_eq!(s("0 1"), "0px 1");
    assert_eq!(s("1 0"), "0px 1");
    assert_eq!(s("0px 1"), "0px 1");
    assert_eq!(s("10px 2 / 100px"), "10px 2 / 100px");
    assert_eq!(s("2 10px / 100px"), "10px 2 / 100px");
    assert_eq!(s("10px / 100px"), "10px / 100px");
    assert_eq!(s("auto / 100px"), "auto / 100px");
    assert_eq!(s("10px 2 / auto"), "10px 2");
    assert_eq!(s("2 10px / auto"), "10px 2");
    assert_eq!(s("10px / auto"), "10px");
    assert_eq!(s("2 / auto"), "2");
    assert_eq!(s("auto / auto"), "auto");
    // 插值语料 canonical 回灌形（animation serialize_columns_components 输出族）
    assert_eq!(s("150px / 150px"), "150px / 150px");
    assert_eq!(s("15 / 150px"), "15 / 150px");
    assert_eq!(s("0px / 0px"), "0px / 0px");
}

#[test]
fn columns_canonical_case_insensitive_auto() {
    let (w, c, h) = slots("AUTO 3");
    assert_eq!(
        serialize_columns_canonical(w.as_deref(), c.as_deref(), h.as_deref()),
        "3"
    );
}
