// tests_2 溢出测试（从 tests_2.rs 自动拆分）
use super::*;
use crate::values::*;
use crate::ast::*;
use crate::tokenizer::{Token, Tokenizer, Spanned};
use crate::parser::Parser;


// ═══════════════════════════════════════════════════════════════════════
// 额外的颜色解析测试（覆盖 color.rs 的 uncovered 路径）
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试十六进制颜色的边界值和错误处理
fn test_hex_color_edge_cases() {
    // 测试 #RGB 和 #RGBA 边界值
    assert_eq!(parse_color("#000"), Some(ColorValue::Rgba(0, 0, 0, 255)));
    assert_eq!(parse_color("#FFF"), Some(ColorValue::Rgba(255, 255, 255, 255)));
    assert_eq!(parse_color("#0000"), Some(ColorValue::Rgba(0, 0, 0, 0)));
    assert_eq!(parse_color("#FFFF"), Some(ColorValue::Rgba(255, 255, 255, 255)));

    // 测试 #RRGGBB 和 #RRGGBBAA 边界值
    assert_eq!(parse_color("#000000"), Some(ColorValue::Rgba(0, 0, 0, 255)));
    assert_eq!(parse_color("#FFFFFF"), Some(ColorValue::Rgba(255, 255, 255, 255)));
    assert_eq!(parse_color("#00000000"), Some(ColorValue::Rgba(0, 0, 0, 0)));
    assert_eq!(parse_color("#FFFFFFFF"), Some(ColorValue::Rgba(255, 255, 255, 255)));

    // 测试无效的十六进制格式
    assert_eq!(parse_color("#"), None);
    assert_eq!(parse_color("#G00"), None);
    assert_eq!(parse_color("#GG0000"), None);
    assert_eq!(parse_color("#12345"), None);
    assert_eq!(parse_color("#1234567"), None);
    assert_eq!(parse_color("#123456789"), None);

    // 测试大小写不敏感
    assert_eq!(parse_color("#abc"), Some(ColorValue::Rgba(170, 187, 204, 255)));
    assert_eq!(parse_color("#ABC"), Some(ColorValue::Rgba(170, 187, 204, 255)));
    assert_eq!(parse_color("#AbC"), Some(ColorValue::Rgba(170, 187, 204, 255)));
}

#[test]
/// 测试 rgb/rgba 函数的边界值和错误处理
fn test_rgb_function_edge_cases() {
    // 测试边界值
    assert_eq!(parse_color("rgb(0, 0, 0)"), Some(ColorValue::Rgba(0, 0, 0, 255)));
    assert_eq!(parse_color("rgb(255, 255, 255)"), Some(ColorValue::Rgba(255, 255, 255, 255)));
    assert_eq!(parse_color("rgba(0, 0, 0, 0)"), Some(ColorValue::Rgba(0, 0, 0, 0)));
    assert_eq!(parse_color("rgba(255, 255, 255, 255)"), Some(ColorValue::Rgba(255, 255, 255, 255)));

    // 测试百分比值边界值
    assert_eq!(parse_color("rgb(0%, 0%, 0%)"), Some(ColorValue::Rgba(0, 0, 0, 255)));
    assert_eq!(parse_color("rgb(100%, 100%, 100%)"), Some(ColorValue::Rgba(255, 255, 255, 255)));
    assert_eq!(parse_color("rgba(50%, 50%, 50%, 50%)"), Some(ColorValue::Rgba(128, 128, 128, 128)));

    // 测试浮点数输入
    assert_eq!(parse_color("rgb(12.3, 45.6, 78.9)"), Some(ColorValue::Rgba(12, 46, 79, 255)));
    assert_eq!(parse_color("rgba(12.5%, 45.5%, 78.5%, 0.5)"), Some(ColorValue::Rgba(32, 116, 200, 128)));

    // 测试无效输入
    assert_eq!(parse_color("rgb()"), None);
    assert_eq!(parse_color("rgb(1, 2)"), None);
    assert_eq!(parse_color("rgb(256, 0, 0)"), None);
    assert_eq!(parse_color("rgb(0, -1, 0)"), None);
    assert_eq!(parse_color("rgb(0%, 101%, 0%)"), None);
    assert_eq!(parse_color("rgba(0, 0, 0, -1)"), None);
    assert_eq!(parse_color("rgba(0, 0, 0, 2)"), None);

    // 测试空格处理
    assert_eq!(parse_color("rgb( 1 , 2 , 3 )"), Some(ColorValue::Rgba(1, 2, 3, 255)));
    assert_eq!(parse_color("rgba( 10% , 20% , 30% , 40% )"), Some(ColorValue::Rgba(26, 51, 77, 102)));
}

#[test]
/// 测试 hsl/hsla 函数的边界值和错误处理
fn test_hsl_function_edge_cases() {
    // 测试基本 HSL 值
    assert_eq!(parse_color("hsl(0, 0%, 0%)"), Some(ColorValue::Hsla(0.0, 0.0, 0.0, 1.0)));
    assert_eq!(parse_color("hsl(360, 100%, 100%)"), Some(ColorValue::Hsla(360.0, 100.0, 100.0, 1.0)));
    assert_eq!(parse_color("hsla(0, 0%, 0%, 0)"), Some(ColorValue::Hsla(0.0, 0.0, 0.0, 0.0)));
    assert_eq!(parse_color("hsla(180, 50%, 50%, 1)"), Some(ColorValue::Hsla(180.0, 50.0, 50.0, 1.0)));

    // 测试负角度和超过 360 的角度
    assert_eq!(parse_color("hsl(-10, 50%, 50%)"), Some(ColorValue::Hsla(-10.0, 50.0, 50.0, 1.0)));
    assert_eq!(parse_color("hsl(370, 50%, 50%)"), Some(ColorValue::Hsla(370.0, 50.0, 50.0, 1.0)));
    assert_eq!(parse_color("hsl(720, 50%, 50%)"), Some(ColorValue::Hsla(720.0, 50.0, 50.0, 1.0)));

    // 测试饱和度和亮度边界值
    assert_eq!(parse_color("hsl(0, -10%, 50%)"), Some(ColorValue::Hsla(0.0, -10.0, 50.0, 1.0)));
    assert_eq!(parse_color("hsl(0, 110%, 50%)"), Some(ColorValue::Hsla(0.0, 110.0, 50.0, 1.0)));
    assert_eq!(parse_color("hsl(0, 50%, -10%)"), Some(ColorValue::Hsla(0.0, 50.0, -10.0, 1.0)));
    assert_eq!(parse_color("hsl(0, 50%, 110%)"), Some(ColorValue::Hsla(0.0, 50.0, 110.0, 1.0)));

    // 测试带 deg 后缀的角度
    assert_eq!(parse_color("hsl(90deg, 50%, 50%)"), Some(ColorValue::Hsla(90.0, 50.0, 50.0, 1.0)));
    assert_eq!(parse_color("hsl(90.5deg, 50%, 50%)"), Some(ColorValue::Hsla(90.5, 50.0, 50.0, 1.0)));

    // 测试无效输入
    assert_eq!(parse_color("hsl()"), None);
    assert_eq!(parse_color("hsl(1, 2)"), None);
    assert_eq!(parse_color("hsla(1, 2, 3)"), None);
    assert_eq!(parse_color("hsla(1, 2, 3, 4, 5)"), None);
}

#[test]
/// 测试 HWB 颜色函数的边界值和错误处理
fn test_hwb_function_edge_cases() {
    // 测试基本 HWB 值
    assert_eq!(parse_color("hwb(0 0% 0%)"), Some(ColorValue::Rgba(255, 255, 255, 255)));
    assert_eq!(parse_color("hwb(0 100% 0%)"), Some(ColorValue::Rgba(255, 0, 0, 255)));
    assert_eq!(parse_color("hwb(60 0% 0%)"), Some(ColorValue::Rgba(255, 255, 0, 255)));
    assert_eq!(parse_color("hwb(0 0% 100%)"), Some(ColorValue::Rgba(0, 0, 0, 255)));

    // 测试 W+B > 100% 的情况
    assert_eq!(parse_color("hwb(0 150% 150%)"), Some(ColorValue::Rgba(255, 0, 0, 255)));
    assert_eq!(parse_color("hwb(120 80% 80%)"), Some(ColorValue::Rgba(0, 255, 0, 255)));

    // 测试带 alpha 的情况
    assert_eq!(parse_color("hwb(0 50% 50% / 0.5)"), Some(ColorValue::Rgba(128, 128, 128, 128)));
    assert_eq!(parse_color("hwb(0 50% 50% / 50%)"), Some(ColorValue::Rgba(128, 128, 128, 128)));

    // 测试角度带 deg 后缀
    assert_eq!(parse_color("hwb(90deg 50% 50%)"), Some(ColorValue::Rgba(128, 255, 128, 255)));

    // 测试无效输入
    assert_eq!(parse_color("hwb()"), None);
    assert_eq!(parse_color("hwb(1)"), None);
    assert_eq!(parse_color("hwb(1 2)"), None);
    assert_eq!(parse_color("hwb(1 2 3 4 5)"), None);
    assert_eq!(parse_color("hwb(1 2 3 / 4 5)"), None);

    // 测试百分比不是数字
    assert_eq!(parse_color("hwb(0 fifty% 50%)"), None);
    assert_eq!(parse_color("hwb(0 50% fifty%)"), None);
}

#[test]
/// 测试命名颜色的大小写不敏感和别名
fn test_named_case_insensitive() {
    // 测试大小写不敏感
    assert_eq!(parse_color("RED"), Some(ColorValue::Rgba(255, 0, 0, 255)));
    assert_eq!(parse_color("red"), Some(ColorValue::Rgba(255, 0, 0, 255)));
    assert_eq!(parse_color("rEd"), Some(ColorValue::Rgba(255, 0, 0, 255)));

    // 测试颜色别名
    assert_eq!(parse_color("aqua"), Some(ColorValue::Rgba(0, 255, 255, 255)));
    assert_eq!(parse_color("cyan"), Some(ColorValue::Rgba(0, 255, 255, 255)));
    assert_eq!(parse_color("fuchsia"), Some(ColorValue::Rgba(255, 0, 255, 255)));
    assert_eq!(parse_color("magenta"), Some(ColorValue::Rgba(255, 0, 255, 255)));
    assert_eq!(parse_color("grey"), Some(ColorValue::Rgba(128, 128, 128, 255)));
    assert_eq!(parse_color("gray"), Some(ColorValue::Rgba(128, 128, 128, 255)));

    // 测试某些颜色的别名变体
    assert_eq!(parse_color("slategrey"), Some(ColorValue::Rgba(112, 128, 144, 255)));
    assert_eq!(parse_color("slategray"), Some(ColorValue::Rgba(112, 128, 144, 255)));
}

#[test]
/// 测试 parse_color 函数的特殊关键字
fn test_special_keywords() {
    // 测试 transparent 和 currentColor
    assert_eq!(parse_color("transparent"), Some(ColorValue::Transparent));
    assert_eq!(parse_color("TRANSPARENT"), Some(ColorValue::Transparent));
    assert_eq!(parse_color("currentcolor"), Some(ColorValue::CurrentColor));
    assert_eq!(parse_color("currentColor"), Some(ColorValue::CurrentColor));
    assert_eq!(parse_color("CURRENTCOLOR"), Some(ColorValue::CurrentColor));

    // 测试空格和空输入
    assert_eq!(parse_color(""), None);
    assert_eq!(parse_color("   "), None);
    assert_eq!(parse_color("  transparent  "), Some(ColorValue::Transparent));
    assert_eq!(parse_color("  currentColor  "), Some(ColorValue::CurrentColor));
}

#[test]
/// 测试 alpha 值解析的边界情况
fn test_alpha_parsing() {
    // 测试 rgb 中的 alpha
    assert_eq!(parse_color("rgba(0, 0, 0, 0)"), Some(ColorValue::Rgba(0, 0, 0, 0)));
    assert_eq!(parse_color("rgba(0, 0, 0, 1)"), Some(ColorValue::Rgba(0, 0, 0, 255)));
    assert_eq!(parse_color("rgba(0, 0, 0, 0.5)"), Some(ColorValue::Rgba(0, 0, 0, 128)));
    assert_eq!(parse_color("rgba(0, 0, 0, 0.999)"), Some(ColorValue::Rgba(0, 0, 0, 255)));
    assert_eq!(parse_color("rgba(0, 0, 0, 1.001)"), Some(ColorValue::Rgba(0, 0, 0, 255)));

    // 测试百分比 alpha
    assert_eq!(parse_color("rgba(0, 0, 0, 0%)"), Some(ColorValue::Rgba(0, 0, 0, 0)));
    assert_eq!(parse_color("rgba(0, 0, 0, 100%)"), Some(ColorValue::Rgba(0, 0, 0, 255)));
    assert_eq!(parse_color("rgba(0, 0, 0, 50%)"), Some(ColorValue::Rgba(0, 0, 0, 128)));
}

#[test]
/// 测试 hwb_to_rgba 函数的边界值
fn test_hwb_to_rgba_boundary_values() {
    // 测试边界 HWB 值转换为 RGBA
    // 纯色（W=0%, B=0%）应该为纯色
    assert_eq!(hwb_to_rgba(0.0, 0.0, 0.0), (255, 0, 0, 255)); // 红色
    assert_eq!(hwb_to_rgba(120.0, 0.0, 0.0), (0, 255, 0, 255)); // 绿色
    assert_eq!(hwb_to_rgba(240.0, 0.0, 0.0), (0, 0, 255, 255)); // 蓝色

    // 测试纯白
    assert_eq!(hwb_to_rgba(0.0, 100.0, 0.0), (255, 255, 255, 255)); // 白色

    // 测试纯黑
    assert_eq!(hwb_to_rgba(0.0, 0.0, 100.0), (0, 0, 0, 255)); // 黑色

    // 测试 W+B > 100% 的情况
    assert_eq!(hwb_to_rgba(0.0, 150.0, 150.0), (255, 0, 0, 255)); // 红色

    // 测试大角度值
    assert_eq!(hwb_to_rgba(720.0, 0.0, 0.0), (255, 0, 0, 255)); // 与 0.0 相同

    // 测试负角度
    assert_eq!(hwb_to_rgba(-120.0, 0.0, 0.0), (0, 0, 255, 255)); // 与 240.0 相同

    // 测试极端的 W 和 B 值
    assert_eq!(hwb_to_rgba(0.0, -50.0, 50.0), (128, 0, 0, 255));
    assert_eq!(hwb_to_rgba(0.0, 150.0, 50.0), (255, 0, 0, 255));
    assert_eq!(hwb_to_rgba(0.0, 50.0, -50.0), (255, 128, 128, 255));
    assert_eq!(hwb_to_rgba(0.0, 50.0, 150.0), (0, 0, 0, 255));
}

// ═══════════════════════════════════════════════════════════════════════
// 额外的变换解析测试（覆盖 parse_transform.rs 的 uncovered 路径）
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_transform 的 none 值和空格处理
fn test_transform_none_and_whitespace() {
    // 测试 none 值
    assert_eq!(parse_transform("none"), Some(TransformValue::None));
    assert_eq!(parse_transform("NONE"), Some(TransformValue::None));
    assert_eq!(parse_transform("  none  "), Some(TransformValue::None));

    // 测试多个函数间的空格
    assert_eq!(parse_transform("translate(10px) rotate(45deg)"), Some(TransformValue::List(vec![
        TransformFunction::Translate(10.0, 0.0),
        TransformFunction::Rotate(45.0),
    ])));

    // 测试换行和制表符
    assert_eq!(parse_transform("translate(10px,\n20px)\r\nrotate(45deg)"), Some(TransformValue::List(vec![
        TransformFunction::Translate(10.0, 20.0),
        TransformFunction::Rotate(45.0),
    ])));
}

#[test]
/// 测试 translate 函数的各种参数组合
fn test_translate_functions() {
    // translate(tx, ty)
    assert_eq!(parse_transform("translate(10px, 20px)"), Some(TransformValue::List(vec![
        TransformFunction::Translate(10.0, 20.0),
    ])));
    assert_eq!(parse_transform("translate(10%, 20%)"), Some(TransformValue::List(vec![
        TransformFunction::Translate(10.0, 20.0),
    ])));
    assert_eq!(parse_transform("translate(0.5em, 2rem)"), Some(TransformValue::List(vec![
        TransformFunction::Translate(0.5, 2.0),
    ])));

    // translateX(tx)
    assert_eq!(parse_transform("translateX(10px)"), Some(TransformValue::List(vec![
        TransformFunction::TranslateX(10.0),
    ])));
    assert_eq!(parse_transform("translateX(-5%)"), Some(TransformValue::List(vec![
        TransformFunction::TranslateX(-5.0),
    ])));

    // translateY(ty)
    assert_eq!(parse_transform("translateY(20px)"), Some(TransformValue::List(vec![
        TransformFunction::TranslateY(20.0),
    ])));
    assert_eq!(parse_transform("translateY(3em)"), Some(TransformValue::List(vec![
        TransformFunction::TranslateY(3.0),
    ])));

    // 测试边界值
    assert_eq!(parse_transform("translate(0, 0)"), Some(TransformValue::List(vec![
        TransformFunction::Translate(0.0, 0.0),
    ])));
    assert_eq!(parse_transform("translate(1e6, -1e6)"), Some(TransformValue::List(vec![
        TransformFunction::Translate(1000000.0, -1000000.0),
    ])));

    // 测试无效输入
    assert_eq!(parse_transform("translate()"), None);
    assert_eq!(parse_transform("translate(1px)"), Some(TransformValue::List(vec![
        TransformFunction::Translate(1.0, 0.0),
    ]))); // 只有一个参数时，第二个默认为 0
    assert_eq!(parse_transform("translate(1px, 2px, 3px)"), None);
    assert_eq!(parse_transform("translate(invalid)"), None);
}

#[test]
/// 测试 rotate 函数的各种角度单位
fn test_rotate_functions() {
    // rotate(angle) - 度
    assert_eq!(parse_transform("rotate(45deg)"), Some(TransformValue::List(vec![
        TransformFunction::Rotate(45.0),
    ])));
    assert_eq!(parse_transform("rotate(90)"), Some(TransformValue::List(vec![
        TransformFunction::Rotate(90.0),
    ])));
    assert_eq!(parse_transform("rotate(-180deg)"), Some(TransformValue::List(vec![
        TransformFunction::Rotate(-180.0),
    ])));

    // rotateX(angle)
    assert_eq!(parse_transform("rotateX(45deg)"), Some(TransformValue::List(vec![
        TransformFunction::RotateX(45.0),
    ])));
    assert_eq!(parse_transform("rotateX(90rad)"), Some(TransformValue::List(vec![
        TransformFunction::RotateX(90.0 * 180.0 / std::f64::consts::PI),
    ])));

    // rotateY(angle)
    assert_eq!(parse_transform("rotateY(45deg)"), Some(TransformValue::List(vec![
        TransformFunction::RotateY(45.0),
    ])));
    assert_eq!(parse_transform("rotateY(0.5turn)"), Some(TransformValue::List(vec![
        TransformFunction::RotateY(180.0),
    ])));

    // rotateZ(angle)
    assert_eq!(parse_transform("rotateZ(45deg)"), Some(TransformValue::List(vec![
        TransformFunction::RotateZ(45.0),
    ])));

    // 测试角度边界值
    assert_eq!(parse_transform("rotate(0deg)"), Some(TransformValue::List(vec![
        TransformFunction::Rotate(0.0),
    ])));
    assert_eq!(parse_transform("rotate(360deg)"), Some(TransformValue::List(vec![
        TransformFunction::Rotate(360.0),
    ])));
    assert_eq!(parse_transform("rotate(720deg)"), Some(TransformValue::List(vec![
        TransformFunction::Rotate(720.0),
    ])));
    assert_eq!(parse_transform("rotate(-360deg)"), Some(TransformValue::List(vec![
        TransformFunction::Rotate(-360.0),
    ])));

    // 测试无效输入
    assert_eq!(parse_transform("rotate()"), None);
    assert_eq!(parse_transform("rotate(45degx)"), None);
    assert_eq!(parse_transform("rotate(invalid)"), None);
}

#[test]
/// 测试 scale 函数的各种参数组合
fn test_scale_functions() {
    // scale(sx, sy)
    assert_eq!(parse_transform("scale(2, 3)"), Some(TransformValue::List(vec![
        TransformFunction::Scale(2.0, Some(3.0)),
    ])));
    assert_eq!(parse_transform("scale(1.5)"), Some(TransformValue::List(vec![
        TransformFunction::Scale(1.5, None),
    ])));
    assert_eq!(parse_transform("scale(-1, 1)"), Some(TransformValue::List(vec![
        TransformFunction::Scale(-1.0, Some(1.0)),
    ])));

    // scaleX(sx)
    assert_eq!(parse_transform("scaleX(2)"), Some(TransformValue::List(vec![
        TransformFunction::ScaleX(2.0),
    ])));
    assert_eq!(parse_transform("scaleX(-0.5)"), Some(TransformValue::List(vec![
        TransformFunction::ScaleX(-0.5),
    ])));

    // scaleY(sy)
    assert_eq!(parse_transform("scaleY(3)"), Some(TransformValue::List(vec![
        TransformFunction::ScaleY(3.0),
    ])));
    assert_eq!(parse_transform("scaleY(0)"), Some(TransformValue::List(vec![
        TransformFunction::ScaleY(0.0),
    ])));

    // 测试边界值
    assert_eq!(parse_transform("scale(1, 1)"), Some(TransformValue::List(vec![
        TransformFunction::Scale(1.0, Some(1.0)),
    ])));
    assert_eq!(parse_transform("scale(0, 0)"), Some(TransformValue::List(vec![
        TransformFunction::Scale(0.0, Some(0.0)),
    ])));

    // 测试无效输入
    assert_eq!(parse_transform("scale()"), None);
    assert_eq!(parse_transform("scale(1, 2, 3)"), None);
    assert_eq!(parse_transform("scale(invalid)"), None);
}

#[test]
/// 测试 skew 函数的各种参数组合
fn test_skew_functions() {
    // skew(ax, ay)
    assert_eq!(parse_transform("skew(30deg, 45deg)"), Some(TransformValue::List(vec![
        TransformFunction::Skew(30.0, Some(45.0)),
    ])));
    assert_eq!(parse_transform("skew(10deg)"), Some(TransformValue::List(vec![
        TransformFunction::Skew(10.0, None),
    ])));

    // 测试角度单位
    assert_eq!(parse_transform("skew(1.57rad, 90deg)"), Some(TransformValue::List(vec![
        TransformFunction::Skew(1.57 * 180.0 / std::f64::consts::PI, Some(90.0)),
    ])));
    assert_eq!(parse_transform("skew(0.25turn)"), Some(TransformValue::List(vec![
        TransformFunction::Skew(90.0, None),
    ])));

    // 测试边界值
    assert_eq!(parse_transform("skew(0deg, 0deg)"), Some(TransformValue::List(vec![
        TransformFunction::Skew(0.0, None),
    ])));
    assert_eq!(parse_transform("skew(-180deg, 180deg)"), Some(TransformValue::List(vec![
        TransformFunction::Skew(-180.0, Some(180.0)),
    ])));

    // 测试无效输入
    assert_eq!(parse_transform("skew()"), None);
    assert_eq!(parse_transform("skew(1deg, 2deg, 3deg)"), None);
    assert_eq!(parse_transform("skew(invalid)"), None);
}

#[test]
/// 测试 3D 变换函数
fn test_3d_transform_functions() {
    // translate3d(tx, ty, tz)
    assert_eq!(parse_transform("translate3d(10px, 20px, 30px)"), Some(TransformValue::List(vec![
        TransformFunction::Translate3d(10.0, 20.0, 30.0),
    ])));
    assert_eq!(parse_transform("translate3d(1, 2, 3)"), Some(TransformValue::List(vec![
        TransformFunction::Translate3d(1.0, 2.0, 3.0),
    ])));

    // scale3d(sx, sy, sz)
    assert_eq!(parse_transform("scale3d(1, 2, 3)"), Some(TransformValue::List(vec![
        TransformFunction::Scale3d(1.0, 2.0, 3.0),
    ])));
    assert_eq!(parse_transform("scale3d(0.5, 1, 2)"), Some(TransformValue::List(vec![
        TransformFunction::Scale3d(0.5, 1.0, 2.0),
    ])));

    // rotate3d(x, y, z, angle)
    assert_eq!(parse_transform("rotate3d(1, 0, 0, 45deg)"), Some(TransformValue::List(vec![
        TransformFunction::Rotate3d(1.0, 0.0, 0.0, 45.0),
    ])));
    assert_eq!(parse_transform("rotate3d(0, 1, 0, 90deg)"), Some(TransformValue::List(vec![
        TransformFunction::Rotate3d(0.0, 1.0, 0.0, 90.0),
    ])));
    assert_eq!(parse_transform("rotate3d(1, 1, 1, 180deg)"), Some(TransformValue::List(vec![
        TransformFunction::Rotate3d(1.0, 1.0, 1.0, 180.0),
    ])));

    // perspective(length)
    assert_eq!(parse_transform("perspective(1000px)"), Some(TransformValue::List(vec![
        TransformFunction::Perspective(1000.0),
    ])));
    assert_eq!(parse_transform("perspective(10em)"), Some(TransformValue::List(vec![
        TransformFunction::Perspective(10.0),
    ])));
    assert_eq!(parse_transform("perspective(1000)"), Some(TransformValue::List(vec![
        TransformFunction::Perspective(1000.0),
    ])));

    // 测试 perspective 的边界值
    assert_eq!(parse_transform("perspective(0)"), None);
    assert_eq!(parse_transform("perspective(-1px)"), None);
    assert_eq!(parse_transform("perspective(1e-6)"), None); // 非常小的正数

    // 测试 3D 函数的无效输入
    assert_eq!(parse_transform("translate3d(1, 2)"), None);
    assert_eq!(parse_transform("scale3d(1, 2)"), None);
    assert_eq!(parse_transform("rotate3d(1, 2)"), None);
    assert_eq!(parse_transform("rotate3d(1, 2, 3, 4, 5)"), None);
    assert_eq!(parse_transform("perspective()"), None);
    assert_eq!(parse_transform("perspective(invalid)"), None);
}

#[test]
/// 测试 matrix 函数
fn test_matrix_function() {
    // matrix(a, b, c, d, e, f)
    assert_eq!(parse_transform("matrix(1, 0, 0, 1, 0, 0)"), Some(TransformValue::List(vec![
        TransformFunction::Matrix(1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
    ])));
    assert_eq!(parse_transform("matrix(2, 0, 0, 2, 10, 20)"), Some(TransformValue::List(vec![
        TransformFunction::Matrix(2.0, 0.0, 0.0, 2.0, 10.0, 20.0),
    ])));
    assert_eq!(parse_transform("matrix(1, 0.5, -0.5, 1, 100, 50)"), Some(TransformValue::List(vec![
        TransformFunction::Matrix(1.0, 0.5, -0.5, 1.0, 100.0, 50.0),
    ])));

    // 测试浮点数和科学计数法
    assert_eq!(parse_transform("matrix(1.5, -0.25, 0.75, 2, 1e2, -5e1)"), Some(TransformValue::List(vec![
        TransformFunction::Matrix(1.5, -0.25, 0.75, 2.0, 100.0, -50.0),
    ])));

    // 测试边界值
    assert_eq!(parse_transform("matrix(0, 0, 0, 0, 0, 0)"), Some(TransformValue::List(vec![
        TransformFunction::Matrix(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    ])));
    assert_eq!(parse_transform("matrix(1e6, -1e6, 1e6, -1e6, 1e6, -1e6)"), Some(TransformValue::List(vec![
        TransformFunction::Matrix(1000000.0, -1000000.0, 1000000.0, -1000000.0, 1000000.0, -1000000.0),
    ])));

    // 测试无效输入
    assert_eq!(parse_transform("matrix()"), None);
    assert_eq!(parse_transform("matrix(1, 2, 3, 4, 5)"), None);
    assert_eq!(parse_transform("matrix(1, 2, 3, 4, 5, 6, 7)"), None);
    assert_eq!(parse_transform("matrix(invalid)"), None);
}

#[test]
/// 测试复杂变换组合
fn test_complex_transforms() {
    // 测试多个变换函数的组合
    assert_eq!(parse_transform("translate(10px, 20px) rotate(45deg) scale(1.5)"), Some(TransformValue::List(vec![
        TransformFunction::Translate(10.0, 20.0),
        TransformFunction::Rotate(45.0),
        TransformFunction::Scale(1.5, None),
    ])));

    // 测试 2D 和 3D 混合
    assert_eq!(parse_transform("translateX(10px) rotateY(45deg) translateZ(100px)"), Some(TransformValue::List(vec![
        TransformFunction::TranslateX(10.0),
        TransformFunction::RotateY(45.0),
        TransformFunction::Translate3d(0.0, 0.0, 100.0),
    ])));

    // 测试带有空格和换行的复杂变换
    assert_eq!(parse_transform("scale(2)\n   rotate(30deg)\t  translate(5px, 10px)"), Some(TransformValue::List(vec![
        TransformFunction::Scale(2.0, None),
        TransformFunction::Rotate(30.0),
        TransformFunction::Translate(5.0, 10.0),
    ])));

    // 测试嵌套函数（虽然 CSS 不支持，但测试解析器的错误处理）
    assert_eq!(parse_transform("translate(rotate(45deg))"), None);

    // 测试语法错误
    assert_eq!(parse_transform("translate(10px"), None);
    assert_eq!(parse_transform("translate10px)"), None);
    assert_eq!(parse_transform("translate(10px)"), Some(TransformValue::List(vec![
        TransformFunction::Translate(10.0, 0.0),
    ])));
}

#[test]
/// 测试无效变换输入
fn test_invalid_transforms() {
    // 测试无效的函数名
    assert_eq!(parse_transform("invalid(10px)"), None);
    assert_eq!(parse_transform("move(10px)"), None);
    assert_eq!(parse_transform("flip(180deg)"), None);

    // 测试不匹配的括号
    assert_eq!(parse_transform("translate(10px"), None);
    assert_eq!(parse_transform("translate10px)"), None);
    assert_eq!(parse_transform("translate((10px))"), None);
    assert_eq!(parse_transform("translate(10px))"), None);

    // 测试空的函数参数
    assert_eq!(parse_transform("translate()"), None);
    assert_eq!(parse_transform("rotate()"), None);
    assert_eq!(parse_transform("scale()"), None);

    // 测试非数字参数
    assert_eq!(parse_transform("translate(abc)"), None);
    assert_eq!(parse_transform("rotate(deg)"), None);
    assert_eq!(parse_transform("scale(two)"), None);

    // 测试空输入
    assert_eq!(parse_transform(""), None);
    assert_eq!(parse_transform("   "), None);

    // 测试只有 none 的空格
    assert_eq!(parse_transform("   none   "), Some(TransformValue::None));
}
