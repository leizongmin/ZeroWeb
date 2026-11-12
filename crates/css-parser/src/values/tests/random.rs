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

// ---- §9.4.1 <random-key> parse 期 canonical 化（random-serialize 页 42F
// 分桶钉面：A key 展开/替换 28、B 单位归一 4、C 实参 lite 折叠 3、E 显式
// ua-ident 序归一 4；D color 通道嵌套挂账不在此刀）----

#[test]
fn test_random_canonicalize_key_expansion_and_reorder() {
    // auto/省略 key → element-scoped ua-<prop>-<index> 展开（A 桶）。
    assert_eq!(
        canonicalize_random_keys("random(0px, 100px)", "width").as_deref(),
        Some("random(element-scoped ua-width-1, 0px, 100px)")
    );
    assert_eq!(
        canonicalize_random_keys("random(auto, 0px, 100px)", "height").as_deref(),
        Some("random(element-scoped ua-height-1, 0px, 100px)")
    );
    // property-scoped/index-scoped → ua-<prop>[-<index>] 替换（关键字摘除）。
    assert_eq!(
        canonicalize_random_keys("random(property-scoped, 0px, 100px)", "font-size").as_deref(),
        Some("random(ua-font-size, 0px, 100px)")
    );
    assert_eq!(
        canonicalize_random_keys("random(property-index-scoped, 0px, 100px)", "width").as_deref(),
        Some("random(ua-width-1, 0px, 100px)")
    );
    assert_eq!(
        canonicalize_random_keys("random(--foo property-index-scoped, 0px, 100px)", "width").as_deref(),
        Some("random(--foo ua-width-1, 0px, 100px)")
    );
    // element 域 × UA 域任意序 → 序归一 [--ident] element-scoped ua-ident。
    assert_eq!(
        canonicalize_random_keys("random(element-scoped property-scoped, 0px, 100px)", "width").as_deref(),
        Some("random(element-scoped ua-width, 0px, 100px)")
    );
    assert_eq!(
        canonicalize_random_keys("random(property-scoped element-scoped, 0px, 100px)", "height").as_deref(),
        Some("random(element-scoped ua-height, 0px, 100px)")
    );
    assert_eq!(
        canonicalize_random_keys("random(--foo property-scoped element-scoped, 0px, 100px)", "height").as_deref(),
        Some("random(--foo element-scoped ua-height, 0px, 100px)")
    );
    assert_eq!(
        canonicalize_random_keys("random(property-index-scoped element-scoped , 0px, 100px)", "width").as_deref(),
        Some("random(element-scoped ua-width-1, 0px, 100px)")
    );
    assert_eq!(
        canonicalize_random_keys(
            "random(--foo property-index-scoped element-scoped , 0px, 100px)",
            "width"
        )
        .as_deref(),
        Some("random(--foo element-scoped ua-width-1, 0px, 100px)")
    );
    assert_eq!(
        canonicalize_random_keys(
            "random(--foo element-scoped property-index-scoped, 0px, 100px)",
            "width"
        )
        .as_deref(),
        Some("random(--foo element-scoped ua-width-1, 0px, 100px)")
    );
    // step 节透传（A 桶 3px 形）。
    assert_eq!(
        canonicalize_random_keys("random(0px, 100px, 3px)", "height").as_deref(),
        Some("random(element-scoped ua-height-1, 0px, 100px, 3px)")
    );
    assert_eq!(
        canonicalize_random_keys("random(property-scoped, 0px, 100px, 3px)", "width").as_deref(),
        Some("random(ua-width, 0px, 100px, 3px)")
    );
    // 显式 ua-ident 原样保留、仅序归一（E 桶）。
    assert_eq!(
        canonicalize_random_keys("random(ua-height-1 element-scoped, 10px, 20%)", "width").as_deref(),
        Some("random(element-scoped ua-height-1, 10px, 20%)")
    );
    assert_eq!(
        canonicalize_random_keys("random(--foo ua-x element-scoped, 10px, 20%)", "width").as_deref(),
        Some("random(--foo element-scoped ua-x, 10px, 20%)")
    );
}

#[test]
fn test_random_canonicalize_section_units_and_lite_fold() {
    // B 桶：单位归一 ms→s / turn·grad→deg；canonical 单位（s/deg）原样。
    assert_eq!(
        canonicalize_random_keys("random(25deg, 1turn)", "rotate").as_deref(),
        Some("random(element-scoped ua-rotate-1, 25deg, 360deg)")
    );
    assert_eq!(
        canonicalize_random_keys("random(--foo, 25deg, 1turn, 5deg)", "rotate").as_deref(),
        Some("random(--foo, 25deg, 360deg, 5deg)")
    );
    assert_eq!(
        canonicalize_random_keys("random(25ms, 50s)", "transition-delay").as_deref(),
        Some("random(element-scoped ua-transition-delay-1, 0.025s, 50s)")
    );
    assert_eq!(
        canonicalize_random_keys("random(--foo, 25ms, 50s, 5s)", "transition-delay").as_deref(),
        Some("random(--foo, 0.025s, 50s, 5s)")
    );
    assert_eq!(
        canonicalize_random_keys("random(--foo, 100grad, 1turn)", "rotate").as_deref(),
        Some("random(--foo, 90deg, 360deg)")
    );
    // C 桶：实参节 lite 折叠（保单位语义；calc 包裹形保包裹）。
    assert_eq!(
        canonicalize_random_keys("random(10 * 100px, 200em / 2)", "width").as_deref(),
        Some("random(element-scoped ua-width-1, 1000px, 100em)")
    );
    assert_eq!(
        canonicalize_random_keys("random(--foo, 10 * 100px, 200em / 2)", "width").as_deref(),
        Some("random(--foo, 1000px, 100em)")
    );
    assert_eq!(
        canonicalize_random_keys("random(fixed calc(2 / 4), 0px, 100px)", "width").as_deref(),
        Some("random(fixed calc(0.5), 0px, 100px)")
    );
}

#[test]
fn test_random_canonicalize_roundtrip_unchanged() {
    // 已 canonical/无改写点形 → None（调用方保留原文）。
    assert!(canonicalize_random_keys("random(fixed 0.5, 0px, 100px)", "width").is_none());
    assert!(canonicalize_random_keys("random(--foo, 0px, 100px)", "width").is_none());
    assert!(canonicalize_random_keys("random(element-scoped, 0px, 100px)", "width").is_none());
    // 转义 ident 原样（serialize 页 `--\{` / `ua-\{` 行）。
    assert!(canonicalize_random_keys("random(--\\{ element-scoped, 0px, 100px, 3px)", "width").is_none());
    assert!(canonicalize_random_keys("random(element-scoped ua-\\{, 0px, 100px, 3px)", "width").is_none());
    // 混合族同族单位原样（`1em, 200rem` 行）。
    assert!(canonicalize_random_keys("random(--foo, 1em, 200rem)", "width").is_none());
    // 嵌套 calc 形内 --foo 纯 key 调用不改写（serialize 页 `calc(2 *
    // random(--foo, 0px, 100px))` 行现状过锚）。
    assert!(canonicalize_random_keys("calc(2 * random(--foo, 0px, 100px))", "width").is_none());
    // invalid 形不改写。
    assert!(canonicalize_random_keys("random(foo, 1px, 2px)", "width").is_none());
    // 无 random 值。
    assert!(canonicalize_random_keys("10px", "width").is_none());
}

#[test]
fn test_random_canonicalize_string_and_item_immunity() {
    // 字符串字面内函数名不识别（content 文本值合法含函数名）。
    assert!(canonicalize_random_keys("\"random(0px, 100px)\"", "content").is_none());
    assert!(canonicalize_random_keys("\"pre random(0px, 100px)\"", "content").is_none());
    // random-item 指定值原样（arbitrary substitution 序列化不 canonical 化）。
    assert!(canonicalize_random_keys("random-item(auto, serif, sans-serif)", "font-family").is_none());
    // random-item 计入 index 序但不改写；后随 random( 取序 2。
    assert_eq!(
        canonicalize_random_keys("random-item(--x, a) random(0px, 100px)", "width").as_deref(),
        Some("random-item(--x, a) random(element-scoped ua-width-2, 0px, 100px)")
    );
    // 同值多调用序数递增（§9.4 margin shorthand 语义）。
    assert_eq!(
        canonicalize_random_keys("random(0px, 1px) random(0px, 2px)", "margin").as_deref(),
        Some("random(element-scoped ua-margin-1, 0px, 1px) random(element-scoped ua-margin-2, 0px, 2px)")
    );
}

#[test]
fn test_random_parse_calc_sum_sections() {
    // calc-sum-lite 节写面门接受面（serialize 页 C 桶三形）。
    let spec = parse_random_args("10 * 100px, 200em / 2").unwrap();
    assert_eq!(spec.family, RandomFamily::Length);
    assert!(!spec.has_fixed);
    let spec = parse_random_args("fixed calc(2 / 4), 0px, 100px").unwrap();
    assert!(spec.has_fixed);
    assert_eq!(spec.family, RandomFamily::Length);
    // fixed 实参门：calc 形须无单位 [0,1)。
    assert!(parse_random_args("fixed calc(1px / 2), 0px, 100px").is_none());
    assert!(parse_random_args("fixed calc(2), 0px, 100px").is_none());
    assert!(parse_random_args("fixed 2 / 4, 0px, 100px").is_none());
    // lite 语法门：双操作数相邻拒（invalid 页 `1px foo` 同族超纲形）。
    assert!(parse_random_args("10 100px, 2px").is_none());
    assert!(parse_random_args("1px + 1em, 2px").is_none());
}

#[test]
fn test_random_fold_decl_entry() {
    // fold 入口接线：白名单外槽（height）同样 canonical 化；custom property
    // 跳过（§9.4 substitution 语义）。
    assert_eq!(
        fold_decl_specified("height", "random(0px, 100px)").as_deref(),
        Some("random(element-scoped ua-height-1, 0px, 100px)")
    );
    assert!(fold_decl_specified("--x", "random(0px, 100px)").is_none());
    assert!(fold_decl_specified("width", "random(foo, 1px, 2px)").is_none());
    assert!(fold_decl_specified("width", "10px").is_none());
}
