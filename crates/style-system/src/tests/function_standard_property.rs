//! k125（kc5r125）：标准属性值域 `<dashed-function>` 替换钉测。
//!
//! WPT 判据 dashed-function-standard-property 族（css-mixins-1
//! §substitute-a-dashed-function）：`<dashed-function>` 调用直接用于标准
//! 属性值——级联面按替换函数同 var() 口径放行（值域文法不可判定），4.755
//! 站替换后由 apply 消费；简写形并入 pending 哨兵域由 4.85 二趟展开落长
//! 属性；调用不可解（未注册名）→ guaranteed-invalid → IACVT 初始值。

use super::super::*;
use super::helpers::{ColorValue, LengthValue, make_test_dom};
use zero_css_parser::Parser as CssParser;

#[test]
/// 「`<dashed-function>` in longhand」：untyped 函数直接形长属性——
/// `width: --ten-px()` 替换后 apply（k125 前级联按 invalid 丢声明 → Auto）。
fn stdlong_longhand_direct() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --ten-px() { result: 10px; } #main { width: --ten-px(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let st = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(st.width, LengthValue::Px(10.0));
}

#[test]
/// 「`<dashed-function>` with typed result in longhand」：typed 函数
///（`returns <length>`）直接形长属性——会话内求值产物同域落地。
fn stdlong_typed_longhand() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --ten-px-typed() returns <length> { result: 10px; } \
               #main { height: --ten-px-typed(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let st = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(st.height, LengthValue::Px(10.0));
}

#[test]
/// 「`<dashed-function>` with in inherited longhand」：继承性长属性
///（color）替换产物落地＋子元素继承计算值。
fn stdlong_inherited_longhand() {
    let (doc, _html, _body, div, p) = make_test_dom();
    let css = "@function --green() { result: green; } #main { color: --green(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let green = ColorValue::Rgba(0, 128, 0, 255);
    assert_eq!(styles.get(&div).expect("div 应有计算样式").color, green);
    assert_eq!(styles.get(&p).expect("p 应有计算样式").color, green);
}

#[test]
/// 「`<dashed-function>` with typed result in shorthand」：简写直接形——
/// pending 哨兵域幂等保形（4.6 复燃），4.755 站代入后 4.85 二趟展开落 4
/// 长属性（k125 前族解析器拒函数 token 丢整条 → 全落初始值）。
fn stdlong_shorthand_pending_expansion() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --ten-px() { result: 10px; } #main { padding: --ten-px(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let st = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(st.padding_top, LengthValue::Px(10.0));
    assert_eq!(st.padding_right, LengthValue::Px(10.0));
    assert_eq!(st.padding_bottom, LengthValue::Px(10.0));
    assert_eq!(st.padding_left, LengthValue::Px(10.0));
}

#[test]
/// 守成：var() 间接形（`--w: --ten-px(); width: var(--w)`）——放行门与
/// 哨兵域补域不得翻转既有间接路径（k109 起已通）。
fn stdlong_custom_prop_baseline_preserved() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --ten-px() { result: 10px; } #main { --w: --ten-px(); width: var(--w); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let st = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(st.width, LengthValue::Px(10.0));
    assert_eq!(st.custom_properties.get("--w").map(String::as_str), Some("10px"));
}

#[test]
/// guaranteed-invalid（未注册名调用）：级联放行保原文 → 站保留原文（无
/// 注册函数可代入）→ apply 拒函数形 → IACVT 初始值——放行门的下游裁决
/// 兑现，非级联丢声明（css-mixins-1 §3 invalid at substitution 同观测）。
fn stdlong_unregistered_fn_iacvt_initial() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "#main { width: --no-such-fn(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let st = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(st.width, LengthValue::Auto);
}

#[test]
/// 源序 last-wins 跨简写展开：`padding: --ten-px()` 在前、`padding-top`
/// 长属性在后——后置长属性胜出该槽，其余槽保持替换产物。
fn stdlong_shorthand_then_longhand_source_order() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = "@function --ten-px() { result: 10px; } \
               #main { padding: --ten-px(); padding-top: 3px; }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    let st = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(st.padding_top, LengthValue::Px(3.0));
    assert_eq!(st.padding_right, LengthValue::Px(10.0));
}

#[test]
/// 函数产物残 attr() 调用形 → 消费声明 IACVT → 继承属性继承回落（WPT
/// function-attr「Returned url() is attr-tainted」族观测通道；css-values-5
/// §attr-security taint 拦截全域挂账随函数内 attr 求值刀，本钉锚定回落臂）。
fn stdlong_attr_word_product_iacvt_inherit() {
    let (mut doc, _html, _body, div, p) = make_test_dom();
    doc.set_attribute(p, "id", "actual");
    doc.set_attribute(p, "data-cat", "url(cat.png)");
    let css = "@function --f() { result: attr(data-cat type(*)); } \
               #main { list-style-image: url(parent); } \
               #actual { list-style-image: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    assert_eq!(
        styles.get(&p).expect("p 应有计算样式").list_style_image,
        ListStyleImageComputedValue::Url("parent".into())
    );
}

#[test]
/// 函数产物残未注册嵌套调用形（guaranteed-invalid「保留原文」系 custom
/// property 值域口径）→ 标准属性消费域 IACVT 继承回落——apply 拒＝槽默
/// 认（k125 放行门暴露的级联丢/下游拒观测断裂收口）。
fn stdlong_unregistered_nested_iacvt_inherit() {
    let (doc, _html, _body, div, p) = make_test_dom();
    let css = "@function --f() { result: --no-such-nested(); } \
               #main { list-style-image: url(parent); } \
               #main p { list-style-image: --f(); }";
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    assert_eq!(
        styles.get(&p).expect("p 应有计算样式").list_style_image,
        ListStyleImageComputedValue::Url("parent".into())
    );
}
