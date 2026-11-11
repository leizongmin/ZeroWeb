// random()（CSS Values 5 §random，tentative）文法门钉——random-invalid
// （39 形）/random-serialize（71 接受形零漂移锚）两页语料驱动。identification
// 节形态（--ident/element 域 × UA 域 scoping/auto/fixed 组合与错序拒）、节数门
//（identification 有无分支）、数字节族一致（% 归 length 族）、范围门
//（fixed [0,1)）、族随属性槽（random_family_allowed）。
//
// <https://drafts.csswg.org/css-values-5/#random>

use super::super::*;

// ---- 合法形（random-serialize 页 71 接受形 + computed 页 in-range 面）----

#[test]
fn test_random_parse_valid_forms() {
    // 纯数字形：min,max[,step]。
    assert!(parse_random_args("0px, 100px").is_some());
    assert!(parse_random_args("0px, 100px, 3px").is_some());
    assert!(parse_random_args("1, 11").is_some());
    assert!(parse_random_args("0, 10, 5").is_some());
    assert!(parse_random_args("-100px, -10px").is_some());
    assert!(parse_random_args("-100px, -10px, -5px").is_some());
    // min>max 文法合法（serialize 页 `random(--foo, 100px, 0px)` 行——求值期交换）。
    assert!(parse_random_args("--foo, 100px, 0px").is_some());
    // identification 形：--ident + 可选 scoping。
    assert!(parse_random_args("--foo, 0px, 100px").is_some());
    assert!(parse_random_args("--foo, 0px, 100px, 3px").is_some());
    assert!(parse_random_args("--foo element-scoped, 0px, 100px").is_some());
    assert!(parse_random_args("--foo, 10px, 20%").is_some()); // % 归 length 族
    // auto shorthand（serialize 页 `random(auto, ...)` 行）。
    assert!(parse_random_args("auto, 0px, 100px").is_some());
    assert!(parse_random_args("auto, 0px, 100px, 3px").is_some());
    // scoping 独占形（无 ident）。
    assert!(parse_random_args("element-scoped, 0px, 100px").is_some());
    assert!(parse_random_args("property-scoped, 0px, 100px").is_some());
    assert!(parse_random_args("--foo property-scoped, 0px, 100px").is_some());
    assert!(parse_random_args("property-index-scoped, 1, 300000").is_some());
    assert!(parse_random_args("--foo property-index-scoped, 0px, 100px").is_some());
    // element 域 × UA 域组合（任意序；serialize 页行）。
    assert!(parse_random_args("element-scoped property-scoped, 0px, 100px").is_some());
    assert!(parse_random_args("property-scoped element-scoped, 0px, 100px").is_some());
    assert!(parse_random_args("--foo element-scoped property-scoped, 0px, 100px").is_some());
    assert!(parse_random_args("--foo property-scoped element-scoped, 0px, 100px").is_some());
    assert!(parse_random_args("element-scoped property-index-scoped, 0px, 100px").is_some());
    assert!(parse_random_args("property-index-scoped element-scoped , 0px, 100px").is_some());
    assert!(parse_random_args("--foo element-scoped property-index-scoped, 0px, 100px").is_some());
    assert!(parse_random_args("--foo property-index-scoped element-scoped , 0px, 100px").is_some());
    // 显式 ua-*（serialize 页 `element-scoped ua-height-1` 行；property-index
    // 域不带显式 ua-*）。
    assert!(parse_random_args("element-scoped ua-height-1, 10px, 20%").is_some());
    assert!(parse_random_args("ua-width, 10px, 20%").is_some());
    assert!(parse_random_args("--foo ua-width, 10px, 20%").is_some());
    assert!(parse_random_args("ua-width-1, 10px, 20%").is_some());
    assert!(parse_random_args("--foo element-scoped, 0px, 100px, 3px").is_some());
    // 转义 ident（serialize 页 `random(--\{ element-scoped, ...)` /
    // `random(element-scoped ua-\{, ...)` 行）。
    assert!(parse_random_args("--\\{ element-scoped, 0px, 100px, 3px").is_some());
    assert!(parse_random_args("element-scoped ua-\\{, 0px, 100px, 3px").is_some());
    // fixed 形：[0,1) 无单位 number。
    assert!(parse_random_args("fixed 0.5, 0px, 100px").is_some());
    assert!(parse_random_args("fixed 0.5, 1px, 100px, 3px").is_some());
    // 大范围（computed 页 in-range 面形）。
    assert!(parse_random_args("0px, 300000px").is_some());
    assert!(parse_random_args("--identifier element-scoped, 0px, 300000px").is_some());
}

// ---- invalid 形（random-invalid 页 39 形逐组，全 width 槽）----

#[test]
fn test_random_parse_invalid_syntax_group() {
    // 结构残缺组（空参/空节/单参/尾逗号）。
    assert!(parse_random_args("").is_none()); // random()
    assert!(parse_random_args(" ").is_none()); // random( )
    assert!(parse_random_args(",").is_none()); // random(,)
    assert!(parse_random_args("1px, ").is_none()); // random(1px, )
    assert!(parse_random_args(",1px").is_none()); // random(,1px)
    assert!(parse_random_args("1px").is_none()); // 单节 = max-only 无 identification
    assert!(parse_random_args("1px,2px,").is_none());
    assert!(parse_random_args("1px,2px,1px,").is_none());
    // step 节残缺/多 token/第四节非数字。
    assert!(parse_random_args("1px,2px,1px foo").is_none());
    assert!(parse_random_args("1px,2px,1px, element-shared").is_none());
}

#[test]
fn test_random_parse_invalid_identification_group() {
    // ident/scoping/fixed 形态与错序组（invalid 页逐形）。
    assert!(parse_random_args("foo, 1px, 2px").is_none()); // 裸 ident 非 --
    assert!(parse_random_args("\"foo\", 1px, 2px").is_none()); // 字符串节
    assert!(parse_random_args("\"--foo\", 1px, 2px").is_none()); // 引号包裹 ident
    assert!(parse_random_args("element-shared foo, 1px, 2px").is_none()); // 未知关键字
    assert!(parse_random_args("element-shared 1px, 2px").is_none());
    assert!(parse_random_args("--foo --bar, 1px, 2px").is_none()); // ident 重复
    assert!(parse_random_args("--foo element-scoped element-scoped, 1px, 2px").is_none());
    assert!(parse_random_args("--foo element-scoped property-scoped element-scoped, 1px, 2px").is_none());
    assert!(parse_random_args("fixed 0.5 element-shared, 1px, 2px").is_none());
    assert!(parse_random_args("fixed 0.5 auto, 1px, 2px").is_none());
    assert!(parse_random_args("fixed 0.5 --foo, 1px, 2px").is_none());
    assert!(parse_random_args("fixed 0.5%, 1px, 2px").is_none()); // 非 <number>
    assert!(parse_random_args("fixed 0.5px, 1px, 2px").is_none());
    assert!(parse_random_args("fixed -1, 1px, 2px").is_none()); // 出 [0,1) 范围
    // scoping 错序/未知/重复组。
    assert!(parse_random_args("element-scoped --foo --bar, 1px, 2px").is_none());
    assert!(parse_random_args("element-scoped uawidth, 1px, 2px").is_none()); // 非 ua- 前缀
    assert!(parse_random_args("element-scoped 1px, 2px").is_none());
    assert!(parse_random_args("foo element-scoped, 1px, 2px").is_none());
    assert!(parse_random_args("foo property-scoped, 1px, 2px").is_none());
    assert!(parse_random_args("foo property-index-scoped, 1px, 2px").is_none());
    // 双 UA 域/UA 域重复组（property-scoped ≙ ua-<prop> 域内互斥）。
    assert!(parse_random_args("property-scoped --foo ua-width, 1px, 2px").is_none());
    assert!(parse_random_args("property-scoped ua-width-1, 1px, 2px").is_none());
    assert!(parse_random_args("property-scoped property-index-scoped, 1px, 2px").is_none());
    assert!(parse_random_args("property-index-scoped --foo property-scoped, 1px, 2px").is_none());
    assert!(parse_random_args("property-index-scoped ua-width-1, 1px, 2px").is_none());
    assert!(parse_random_args("property-index-scoped property-scoped, 1px, 2px").is_none());
    assert!(parse_random_args("property-index-scoped 1px, 2px").is_none());
    // ident 滞后（scoping 已出再出 ident）。
    assert!(parse_random_args("property-index-scoped --foo property-scoped, 1px, 2px").is_none());
}

#[test]
fn test_random_parse_invalid_family_group() {
    // 族一致门：跨族混/未知单位。angle 族文法 valid（槽位随属性拒）。
    assert!(parse_random_args("1px, 2s").is_none());
    assert!(parse_random_args("--foo, 1px, 2").is_none());
    assert!(parse_random_args("1foo, 2foo").is_none());
    assert!(parse_random_args("10deg, 20deg").is_some()); // 文法合法；width 槽拒见下
}

// ---- 族随属性槽（random_family_allowed）----

#[test]
fn test_random_family_allowed_by_property() {
    // length 槽：length 族过，angle/number/time 拒（invalid 页 width+10deg 行）。
    assert!(random_family_allowed("width", RandomFamily::Length));
    assert!(!random_family_allowed("width", RandomFamily::Angle));
    assert!(!random_family_allowed("width", RandomFamily::Number));
    assert!(!random_family_allowed("width", RandomFamily::Time));
    assert!(random_family_allowed("margin-left", RandomFamily::Length));
    // number 槽。
    assert!(random_family_allowed("scale", RandomFamily::Number));
    assert!(random_family_allowed("z-index", RandomFamily::Number));
    assert!(!random_family_allowed("scale", RandomFamily::Length));
    // time 槽。
    assert!(random_family_allowed("animation-duration", RandomFamily::Time));
    assert!(!random_family_allowed("animation-duration", RandomFamily::Length));
    // angle 槽。
    assert!(random_family_allowed("rotate", RandomFamily::Angle));
    // 未列槽放行（complex 槽无语料钉面）。
    assert!(random_family_allowed("transform", RandomFamily::Angle));
    assert!(random_family_allowed("max-lines", RandomFamily::Number));
}
