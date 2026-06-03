//! CSS 解析器覆盖率补充测试：color.rs 命名颜色全量覆盖、其他遗漏路径。

use crate::values::ColorValue;
use crate::values::parse_color;

// ═══════════════════════════════════════════════════════════════════════
// color.rs — 命名颜色全量覆盖测试
// ═══════════════════════════════════════════════════════════════════════

fn assert_named_color(name: &str, r: u8, g: u8, b: u8) {
    let c = parse_color(name).unwrap_or_else(|| panic!("failed to parse color: {}", name));
    assert_eq!(c, ColorValue::Rgba(r, g, b, 255), "mismatch for {}", name);
}

#[test]
fn test_named_colors_basic_16() {
    assert_named_color("black", 0, 0, 0);
    assert_named_color("white", 255, 255, 255);
    assert_named_color("red", 255, 0, 0);
    assert_named_color("green", 0, 128, 0);
    assert_named_color("blue", 0, 0, 255);
    assert_named_color("yellow", 255, 255, 0);
    assert_named_color("cyan", 0, 255, 255);
    assert_named_color("aqua", 0, 255, 255);
    assert_named_color("magenta", 255, 0, 255);
    assert_named_color("fuchsia", 255, 0, 255);
    assert_named_color("silver", 192, 192, 192);
    assert_named_color("gray", 128, 128, 128);
    assert_named_color("grey", 128, 128, 128);
    assert_named_color("maroon", 128, 0, 0);
    assert_named_color("olive", 128, 128, 0);
    assert_named_color("lime", 0, 255, 0);
    assert_named_color("teal", 0, 128, 128);
    assert_named_color("navy", 0, 0, 128);
    assert_named_color("purple", 128, 0, 128);
    assert_named_color("orange", 255, 165, 0);
}

#[test]
fn test_named_colors_a_to_b() {
    assert_named_color("aliceblue", 240, 248, 255);
    assert_named_color("antiquewhite", 250, 235, 215);
    assert_named_color("aquamarine", 127, 255, 212);
    assert_named_color("azure", 240, 255, 255);
    assert_named_color("beige", 245, 245, 220);
    assert_named_color("bisque", 255, 228, 196);
    assert_named_color("blanchedalmond", 255, 235, 205);
    assert_named_color("burlywood", 222, 184, 135);
}

#[test]
fn test_named_colors_c() {
    assert_named_color("cadetblue", 95, 158, 160);
    assert_named_color("chartreuse", 127, 255, 0);
    assert_named_color("chocolate", 210, 105, 30);
    assert_named_color("coral", 255, 127, 80);
    assert_named_color("cornflowerblue", 100, 149, 237);
    assert_named_color("cornsilk", 255, 248, 220);
    assert_named_color("crimson", 220, 20, 60);
}

#[test]
fn test_named_colors_d() {
    assert_named_color("darkblue", 0, 0, 139);
    assert_named_color("darkcyan", 0, 139, 139);
    assert_named_color("darkgoldenrod", 184, 134, 11);
    assert_named_color("darkgray", 169, 169, 169);
    assert_named_color("darkgrey", 169, 169, 169);
    assert_named_color("darkgreen", 0, 100, 0);
    assert_named_color("darkkhaki", 189, 183, 107);
    assert_named_color("darkmagenta", 139, 0, 139);
    assert_named_color("darkolivegreen", 85, 107, 47);
    assert_named_color("darkorange", 255, 140, 0);
    assert_named_color("darkorchid", 153, 50, 204);
    assert_named_color("darkred", 139, 0, 0);
    assert_named_color("darksalmon", 233, 150, 122);
    assert_named_color("darkseagreen", 143, 188, 143);
    assert_named_color("darkslateblue", 72, 61, 139);
    assert_named_color("darkslategray", 47, 79, 79);
    assert_named_color("darkslategrey", 47, 79, 79);
    assert_named_color("darkturquoise", 0, 206, 209);
    assert_named_color("darkviolet", 148, 0, 211);
    assert_named_color("deeppink", 255, 20, 147);
    assert_named_color("deepskyblue", 0, 191, 255);
    assert_named_color("dimgray", 105, 105, 105);
    assert_named_color("dimgrey", 105, 105, 105);
    assert_named_color("dodgerblue", 30, 144, 255);
}

#[test]
fn test_named_colors_f_to_g() {
    assert_named_color("firebrick", 178, 34, 34);
    assert_named_color("floralwhite", 255, 250, 240);
    assert_named_color("forestgreen", 34, 139, 34);
    assert_named_color("gainsboro", 220, 220, 220);
    assert_named_color("ghostwhite", 248, 248, 255);
    assert_named_color("gold", 255, 215, 0);
    assert_named_color("goldenrod", 218, 165, 32);
    assert_named_color("greenyellow", 173, 255, 47);
}

#[test]
fn test_named_colors_h_to_i() {
    assert_named_color("honeydew", 240, 255, 240);
    assert_named_color("hotpink", 255, 105, 180);
    assert_named_color("indianred", 205, 92, 92);
    assert_named_color("indigo", 75, 0, 130);
    assert_named_color("ivory", 255, 255, 240);
}

#[test]
fn test_named_colors_k_to_l() {
    assert_named_color("khaki", 240, 230, 140);
    assert_named_color("lavender", 230, 230, 250);
    assert_named_color("lavenderblush", 255, 240, 245);
    assert_named_color("lawngreen", 124, 252, 0);
    assert_named_color("lemonchiffon", 255, 250, 205);
    assert_named_color("lightblue", 173, 216, 230);
    assert_named_color("lightcoral", 240, 128, 128);
    assert_named_color("lightcyan", 224, 255, 255);
    assert_named_color("lightgoldenrodyellow", 250, 250, 210);
    assert_named_color("lightgray", 211, 211, 211);
    assert_named_color("lightgrey", 211, 211, 211);
    assert_named_color("lightgreen", 144, 238, 144);
    assert_named_color("lightpink", 255, 182, 193);
    assert_named_color("lightsalmon", 255, 160, 122);
    assert_named_color("lightseagreen", 32, 178, 170);
    assert_named_color("lightskyblue", 135, 206, 250);
    assert_named_color("lightslategray", 119, 136, 153);
    assert_named_color("lightslategrey", 119, 136, 153);
    assert_named_color("lightsteelblue", 176, 196, 222);
    assert_named_color("lightyellow", 255, 255, 224);
    assert_named_color("limegreen", 50, 205, 50);
    assert_named_color("linen", 250, 240, 230);
}

#[test]
fn test_named_colors_m_to_p() {
    assert_named_color("mediumaquamarine", 102, 205, 170);
    assert_named_color("mediumblue", 0, 0, 205);
    assert_named_color("mediumorchid", 186, 85, 211);
    assert_named_color("mediumpurple", 147, 112, 219);
    assert_named_color("mediumseagreen", 60, 179, 113);
    assert_named_color("mediumslateblue", 123, 104, 238);
    assert_named_color("mediumspringgreen", 0, 250, 154);
    assert_named_color("mediumturquoise", 72, 209, 204);
    assert_named_color("mediumvioletred", 199, 21, 133);
    assert_named_color("midnightblue", 25, 25, 112);
    assert_named_color("mintcream", 245, 255, 250);
    assert_named_color("mistyrose", 255, 228, 225);
    assert_named_color("moccasin", 255, 228, 181);
    assert_named_color("navajowhite", 255, 222, 173);
    assert_named_color("oldlace", 253, 245, 230);
    assert_named_color("olivedrab", 107, 142, 35);
    assert_named_color("orangered", 255, 69, 0);
    assert_named_color("orchid", 218, 112, 214);
    assert_named_color("palegoldenrod", 238, 232, 170);
    assert_named_color("palegreen", 152, 251, 152);
    assert_named_color("paleturquoise", 175, 238, 238);
    assert_named_color("palevioletred", 219, 112, 147);
    assert_named_color("papayawhip", 255, 239, 213);
    assert_named_color("peachpuff", 255, 218, 185);
    assert_named_color("peru", 205, 133, 63);
    assert_named_color("pink", 255, 192, 203);
    assert_named_color("plum", 221, 160, 221);
    assert_named_color("powderblue", 176, 224, 230);
}

#[test]
fn test_named_colors_r_to_t() {
    assert_named_color("rosybrown", 188, 143, 143);
    assert_named_color("royalblue", 65, 105, 225);
    assert_named_color("saddlebrown", 139, 69, 19);
    assert_named_color("salmon", 250, 128, 114);
    assert_named_color("sandybrown", 244, 164, 96);
    assert_named_color("seagreen", 46, 139, 87);
    assert_named_color("seashell", 255, 245, 238);
    assert_named_color("sienna", 160, 82, 45);
    assert_named_color("skyblue", 135, 206, 235);
    assert_named_color("slateblue", 106, 90, 205);
    assert_named_color("slategray", 112, 128, 144);
    assert_named_color("slategrey", 112, 128, 144);
    assert_named_color("snow", 255, 250, 250);
    assert_named_color("springgreen", 0, 255, 127);
    assert_named_color("steelblue", 70, 130, 180);
}

#[test]
fn test_named_colors_t_to_w() {
    assert_named_color("tan", 210, 180, 140);
    assert_named_color("thistle", 216, 191, 216);
    assert_named_color("tomato", 255, 99, 71);
    assert_named_color("turquoise", 64, 224, 208);
    assert_named_color("violet", 238, 130, 238);
    assert_named_color("wheat", 245, 222, 179);
    assert_named_color("whitesmoke", 245, 245, 245);
}

#[test]
fn test_named_colors_y() {
    assert_named_color("yellowgreen", 154, 205, 50);
}
