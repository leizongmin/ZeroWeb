//! HTML presentational-attributes → CSS hints 测试（body/text/link bgcolor、table
//! border/cellpadding、td/th/tr bgcolor、font color/face、hr size/noshade/width/align、
//! center、img align 等）。
//!
//! R1695 从 lib.rs 内联 mod 抽到独立文件（lib.rs 减负，2116→<2000，CLAUDE.md §5）。
//! 体原为 `mod presentational_hint_tests { use super::*; ... }`，`use super::*` 保留
//!（子模块可访问父私有项），内容字节一致。
use super::*;
use zero_dom::parse_html;

#[test]
fn body_bgcolor_maps_to_background_color() {
    let doc = parse_html("<body bgcolor=\"#FFFFCC\"><p>x</p></body>");
    let body = doc.get_elements_by_tag_name("body")[0];
    let hints = collect_presentational_hints(&doc, body);
    assert!(
        hints.iter().any(|(p, v)| p == "background-color" && v == "#FFFFCC"),
        "hints: {hints:?}"
    );
}

#[test]
fn table_border_and_cell_padding_map_to_css() {
    let doc = parse_html("<table border=\"1\" cellpadding=\"6\"><tr><td>Layer</td></tr></table>");
    let table = doc.get_elements_by_tag_name("table")[0];
    let td = doc.get_elements_by_tag_name("td")[0];
    let table_hints = collect_presentational_hints(&doc, table);
    assert!(
        table_hints.iter().any(|(p, v)| p == "border" && v.contains("1px")),
        "table hints: {table_hints:?}"
    );
    let td_hints = collect_presentational_hints(&doc, td);
    assert!(
        td_hints.iter().any(|(p, v)| p == "padding" && v == "6px"),
        "td hints: {td_hints:?}"
    );
    assert!(
        td_hints.iter().any(|(p, _)| p == "border"),
        "td should inherit table border hint"
    );
}

#[test]
fn anchor_ua_uses_body_link_color() {
    let doc = parse_html("<body LINK=\"#0000EE\"><a href=\"#\">x</a></body>");
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    let a_id = doc.get_elements_by_tag_name("a")[0];
    let style = styles.get(&a_id).expect("anchor styled");
    assert!(
        matches!(&style.color, zero_css_parser::values::ColorValue::Rgba(0, 0, 238, _)),
        "link color {:?}",
        style.color
    );
}

#[test]
fn tr_bgcolor_applies_to_cells_not_row() {
    let doc = parse_html("<table><tr bgcolor=\"#CCCCCC\"><th>Layer</th><td>x</td></tr></table>");
    let tr = doc.get_elements_by_tag_name("tr")[0];
    let th = doc.get_elements_by_tag_name("th")[0];
    let tr_hints = collect_presentational_hints(&doc, tr);
    assert!(
        !tr_hints.iter().any(|(p, _)| p == "background-color"),
        "tr should not get row-wide bgcolor: {tr_hints:?}"
    );
    let th_hints = collect_presentational_hints(&doc, th);
    assert!(
        th_hints.iter().any(|(p, v)| p == "background-color" && v == "#CCCCCC"),
        "th hints: {th_hints:?}"
    );
}

#[test]
fn bold_tag_gets_font_weight_from_ua() {
    let doc = parse_html("<p><b>bold</b></p>");
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    let b_id = doc.get_elements_by_tag_name("b")[0];
    let style = styles.get(&b_id).expect("b styled");
    assert!(matches!(
        style.font_weight,
        zero_css_parser::values::FontWeightValue::Bold
    ));
}

/// R1690：块级元素的 UA 默认 margin（chromium UA 样式表）。ZW 此前无 → blockquote/dd/figure
/// 无缩进。钉死 blockquote/figure margin 1em 40px、dl 1em 0、dd margin-left 40px。
#[test]
fn blockquote_dd_figure_dl_get_ua_margins() {
    use zero_css_parser::values::LengthValue;
    let doc = parse_html("<body><blockquote>q</blockquote><dl><dt>t</dt><dd>d</dd></dl><figure>f</figure></body>");
    let mut system = StyleSystem::new();
    system.set_viewport(800.0, 600.0);
    let styles = system.compute_styles(&doc, &[]);
    let em = 16.0; // 默认 font-size 16px
    let check = |tag: &str, mt: f64, mr: f64, mb: f64, ml: f64| {
        let id = doc.get_elements_by_tag_name(tag)[0];
        let s = styles.get(&id).unwrap_or_else(|| panic!("{tag} styled"));
        assert!(
            matches!(s.margin_top, LengthValue::Px(v) if (v - mt).abs() < 0.5),
            "<{tag}> margin-top should be {mt}, got {:?}",
            s.margin_top
        );
        assert!(
            matches!(s.margin_right, LengthValue::Px(v) if (v - mr).abs() < 0.5),
            "<{tag}> margin-right should be {mr}, got {:?}",
            s.margin_right
        );
        assert!(
            matches!(s.margin_bottom, LengthValue::Px(v) if (v - mb).abs() < 0.5),
            "<{tag}> margin-bottom should be {mb}, got {:?}",
            s.margin_bottom
        );
        assert!(
            matches!(s.margin_left, LengthValue::Px(v) if (v - ml).abs() < 0.5),
            "<{tag}> margin-left should be {ml}, got {:?}",
            s.margin_left
        );
    };
    // 1em = 16px（默认 font-size）。
    check("blockquote", em, 40.0, em, 40.0);
    check("figure", em, 40.0, em, 40.0);
    check("dl", em, 0.0, em, 0.0);
    check("dd", 0.0, 0.0, 0.0, 40.0);
}

/// R1691/R1697：短语元素 UA font-style/text-decoration 默认（≡ i/em，chromium UA）。
/// address/cite/var/dfn → italic；u|ins → underline；s/del/strike → line-through。
#[test]
fn phrase_elements_get_ua_italic_and_decoration() {
    use crate::property::types::TextDecorationLineValue;
    use zero_css_parser::values::FontStyleValue;
    let doc = parse_html(
        "<body><address>a</address><cite>c</cite><var>v</var><dfn>d</dfn>\
             <u>u</u><ins>i</ins><s>s</s><del>x</del><strike>k</strike></body>",
    );
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    for tag in ["address", "cite", "var", "dfn"] {
        let id = doc.get_elements_by_tag_name(tag)[0];
        let s = styles.get(&id).unwrap_or_else(|| panic!("{tag} styled"));
        assert!(
            matches!(s.font_style, FontStyleValue::Italic),
            "<{tag}> should be italic from UA, got {:?}",
            s.font_style
        );
    }
    // R1697：ins 与 u 同组 underline（chromium UA `u, ins { text-decoration: underline }`）。
    for tag in ["u", "ins"] {
        let id = doc.get_elements_by_tag_name(tag)[0];
        let s = styles.get(&id).unwrap_or_else(|| panic!("{tag} styled"));
        assert!(
            matches!(s.text_decoration_line, TextDecorationLineValue::Underline),
            "<{tag}> should be underline from UA, got {:?}",
            s.text_decoration_line
        );
    }
    for tag in ["s", "del", "strike"] {
        let id = doc.get_elements_by_tag_name(tag)[0];
        let s = styles.get(&id).unwrap_or_else(|| panic!("{tag} styled"));
        assert!(
            matches!(s.text_decoration_line, TextDecorationLineValue::LineThrough),
            "<{tag}> should be line-through from UA"
        );
    }
}

/// R1692：small/sub/sup UA font-size/vertical-align（chromium smaller≈0.83em）。
#[test]
fn small_sub_sup_get_ua_font_size_and_vertical_align() {
    use crate::property::types::VerticalAlignValue;
    use zero_css_parser::values::LengthValue;
    let doc = parse_html("<body><small>s</small><sub>b</sub><sup>p</sup></body>");
    let mut system = StyleSystem::new();
    system.set_viewport(800.0, 600.0);
    let styles = system.compute_styles(&doc, &[]);
    // 0.83em × 默认 16px ≈ 13.28px。
    for tag in ["small", "sub", "sup"] {
        let id = doc.get_elements_by_tag_name(tag)[0];
        let s = styles.get(&id).unwrap_or_else(|| panic!("{tag} styled"));
        assert!(
            matches!(s.font_size, LengthValue::Px(v) if (v - 13.28).abs() < 1.0),
            "<{tag}> font-size should be ~13.28 (0.83em×16), got {:?}",
            s.font_size
        );
    }
    let sub = styles.get(&doc.get_elements_by_tag_name("sub")[0]).unwrap();
    assert!(
        matches!(sub.vertical_align, VerticalAlignValue::Sub),
        "<sub> should be vertical-align:sub"
    );
    let sup = styles.get(&doc.get_elements_by_tag_name("sup")[0]).unwrap();
    assert!(
        matches!(sup.vertical_align, VerticalAlignValue::Super),
        "<sup> should be vertical-align:super"
    );
}

/// R1693：code/kbd/samp/tt → font-family:monospace（chromium UA）。
#[test]
fn code_family_gets_monospace_from_ua() {
    let doc = parse_html("<body><code>c</code><kbd>k</kbd><samp>s</samp><tt>t</tt></body>");
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    for tag in ["code", "kbd", "samp", "tt"] {
        let id = doc.get_elements_by_tag_name(tag)[0];
        let s = styles.get(&id).unwrap_or_else(|| panic!("{tag} styled"));
        assert!(
            s.font_family.iter().any(|f| f.eq_ignore_ascii_case("monospace")),
            "<{tag}> font-family should contain monospace from UA, got {:?}",
            s.font_family
        );
    }
}

/// R1698：caption → UA text-align:center（chromium UA `caption { text-align: center }`）。
/// caption-side 上下定位由 layout 独立处理（R1653），此处只验默认水平居中。
#[test]
fn caption_gets_text_align_center_from_ua() {
    use crate::property::types::TextAlignValue;
    let doc = parse_html("<body><table><caption>t</caption><tr><td>x</td></tr></table></body>");
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    let cap = styles
        .get(&doc.get_elements_by_tag_name("caption")[0])
        .expect("caption styled");
    assert!(
        matches!(cap.text_align, TextAlignValue::Center),
        "<caption> should be text-align:center from UA, got {:?}",
        cap.text_align
    );
}

/// R1699：ul/ol → UA list-style-type（chromium UA `ul{disc}` / `ol{decimal}`）。
/// list-style-type 继承、CSS initial=Disc，故 ul 隐式 Disc 正确；但 ol 旧也继承 Disc
/// → 渲染圆点而非序号（BUG）。li 经继承得父 list-style-type（ul 下 li=Disc / ol 下 li=Decimal）。
#[test]
fn ul_ol_get_list_style_type_from_ua_and_inherit_to_li() {
    use zero_css_parser::values::ListStyleTypeValue;
    let doc = parse_html("<body><ul><li>u</li></ul><ol><li>o</li></ol></body>");
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    let ul = styles.get(&doc.get_elements_by_tag_name("ul")[0]).expect("ul styled");
    assert!(
        matches!(ul.list_style_type, ListStyleTypeValue::Disc),
        "<ul> list-style-type should be Disc, got {:?}",
        ul.list_style_type
    );
    let ol = styles.get(&doc.get_elements_by_tag_name("ol")[0]).expect("ol styled");
    assert!(
        matches!(ol.list_style_type, ListStyleTypeValue::Decimal),
        "<ol> list-style-type should be Decimal (not inherited Disc), got {:?}",
        ol.list_style_type
    );
    // li 继承父 list-style-type：ul 下 li=Disc，ol 下 li=Decimal。
    let li_in_ul = styles
        .get(&doc.get_elements_by_tag_name("li")[0])
        .expect("li in ul styled");
    assert!(
        matches!(li_in_ul.list_style_type, ListStyleTypeValue::Disc),
        "<li> in <ul> should inherit Disc, got {:?}",
        li_in_ul.list_style_type
    );
    let li_in_ol = styles
        .get(&doc.get_elements_by_tag_name("li")[1])
        .expect("li in ol styled");
    assert!(
        matches!(li_in_ol.list_style_type, ListStyleTypeValue::Decimal),
        "<li> in <ol> should inherit Decimal, got {:?}",
        li_in_ol.list_style_type
    );
}

/// R1700：HTML4 `<ol/ul/li type>` 属性 → list-style-type（CSS2 App D 表现提示）。
/// ol type: 1/a/A/i/I → decimal/lower-alpha/upper-alpha/lower-roman/upper-roman；
/// ul type: disc/circle/square；li type 覆盖父 list-style-type。
#[test]
fn list_type_attr_maps_to_list_style_type_hint() {
    use zero_css_parser::values::ListStyleTypeValue;
    let doc = parse_html(
        "<body>\
<ol type=\"A\"><li>a</li></ol>\
<ol type=\"i\"><li>r</li></ol>\
<ul type=\"circle\"><li>c</li></ul>\
<ol><li type=\"a\">alpha-li</li></ol>",
    );
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    let ol_a = styles.get(&doc.get_elements_by_tag_name("ol")[0]).unwrap();
    assert!(
        matches!(ol_a.list_style_type, ListStyleTypeValue::UpperAlpha),
        "<ol type=A> → UpperAlpha, got {:?}",
        ol_a.list_style_type
    );
    let ol_i = styles.get(&doc.get_elements_by_tag_name("ol")[1]).unwrap();
    assert!(
        matches!(ol_i.list_style_type, ListStyleTypeValue::LowerRoman),
        "<ol type=i> → LowerRoman, got {:?}",
        ol_i.list_style_type
    );
    let ul_c = styles.get(&doc.get_elements_by_tag_name("ul")[0]).unwrap();
    assert!(
        matches!(ul_c.list_style_type, ListStyleTypeValue::Circle),
        "<ul type=circle> → Circle, got {:?}",
        ul_c.list_style_type
    );
    // li type 覆盖父 ol（默认 decimal）：li[type=a] → LowerAlpha。
    let li_alpha = styles.get(&doc.get_elements_by_tag_name("li")[3]).unwrap();
    assert!(
        matches!(li_alpha.list_style_type, ListStyleTypeValue::LowerAlpha),
        "<li type=a> overrides parent → LowerAlpha, got {:?}",
        li_alpha.list_style_type
    );
}

/// R1710：HTML4 `<img border=N>` → border:Npx solid（CSS2 App D §13.7.3）。
/// 隔离实测 border 单独 net 改善（fixture 24 0.79%→0.71%）；hspace/vspace defer。
#[test]
fn img_border_attr_maps_to_border_hint() {
    use zero_css_parser::values::LengthValue;
    let doc = parse_html("<body><img src=\"x\" border=\"3\"></body>");
    let body_img = doc.get_elements_by_tag_name("img")[0];
    let hints = collect_presentational_hints(&doc, body_img);
    assert!(
        hints.iter().any(|(p, v)| p == "border" && v == "3px solid"),
        "<img border=3> → border:3px solid, got hints: {hints:?}"
    );
    // border="0" 不加边框（HTML4 border=0 显式抑制，img 默认无边框故 no-op）。
    let doc0 = parse_html("<body><img src=\"x\" border=\"0\"></body>");
    let img0 = doc0.get_elements_by_tag_name("img")[0];
    let hints0 = collect_presentational_hints(&doc0, img0);
    assert!(
        !hints0.iter().any(|(p, _)| p == "border"),
        "<img border=0> should not add border hint, got: {hints0:?}"
    );
    // computed: border=3 → border-top-width 3px solid。
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    let img = styles.get(&body_img).expect("img styled");
    assert!(
        matches!(img.border_top_width, LengthValue::Px(v) if (v - 3.0).abs() < 0.01),
        "border=3 → border-top-width 3px, got {:?}",
        img.border_top_width
    );
}

#[test]
fn heading_gets_ua_font_size_and_weight() {
    let doc = parse_html("<body><h1>Title</h1><h2>Section</h2></body>");
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    let h1_id = doc.get_elements_by_tag_name("h1")[0];
    let h2_id = doc.get_elements_by_tag_name("h2")[0];
    let h1 = styles.get(&h1_id).expect("h1 styled");
    let h2 = styles.get(&h2_id).expect("h2 styled");
    assert_eq!(h1.font_size, zero_css_parser::values::LengthValue::Px(32.0));
    assert_eq!(h2.font_size, zero_css_parser::values::LengthValue::Px(24.0));
    assert!(matches!(h1.font_weight, zero_css_parser::values::FontWeightValue::Bold));
}

/// `<pre>`/`<xmp>`/`<listing>`/`<plaintext>` 必须从 UA 样式表继承 `white-space: pre`
/// （HTML 渲染规范）。R1658：ZW default_impl white_space 默认 Normal，pre-family 此前折叠
/// 空白/换行（真 bug）；css-text 全 1644 oracle A/B net-0（零回归）+ legacy fixture 26/35
/// 小幅改善（whitespace 保真）。monospace 字体独立 A/B 切片（font-wall 高方差）。
#[test]
fn pre_family_gets_white_space_pre_from_ua() {
    let doc = parse_html("<body><pre>p</pre><xmp>x</xmp><listing>l</listing><plaintext>t</plaintext></body>");
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    for tag in ["pre", "xmp", "listing", "plaintext"] {
        let id = doc.get_elements_by_tag_name(tag)[0];
        let style = styles.get(&id).unwrap_or_else(|| panic!("{tag} styled"));
        assert!(
            matches!(style.white_space, WhiteSpaceValue::Pre),
            "<{tag}> should inherit white-space:pre from UA stylesheet, got {:?}",
            style.white_space
        );
    }
}

/// R1685：`<mark>` 高亮文本，HTML 渲染规范 UA `mark { background-color: yellow; color: black }`。
/// ZW 默认无 → <mark> 渲成普通 inline（无高亮）。本测试钉死 UA 注入的黄底黑字。
#[test]
fn mark_gets_yellow_bg_black_color_from_ua() {
    use zero_css_parser::values::ColorValue;
    let doc = parse_html("<body><mark>hi</mark></body>");
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    let id = doc.get_elements_by_tag_name("mark")[0];
    let style = styles.get(&id).expect("mark styled");
    assert!(
        matches!(style.background_color, ColorValue::Rgba(255, 255, 0, 255)),
        "<mark> should get background-color:#ffff00 from UA, got {:?}",
        style.background_color
    );
    assert!(
        matches!(style.color, ColorValue::Rgba(0, 0, 0, 255)),
        "<mark> should get color:black from UA, got {:?}",
        style.color
    );
}

/// R1659：`<input>` 是 void inline-block（无子节点），缺固有尺寸时 ZW 把 auto 宽度当全容器宽
///（fixture 37 实测 784×6）致 `<label>` 换行重叠。本测试钉死 UA 按类型注入的固有 width/height：
/// 文本类按 `size` 属性（默认 20）估宽 + 15px 内容高；checkbox/radio/color 固定 13px 方框；
/// submit/reset/button 按 `value` 字符数估宽。select/textarea 仍 width:auto（按内容测宽）。
#[test]
fn input_gets_intrinsic_sizing_from_ua_by_type() {
    use zero_css_parser::values::LengthValue;
    let doc = parse_html(
        "<body>\
             <input type=\"text\" value=\"Alice\">\
             <input type=\"password\">\
             <input size=\"10\">\
             <input type=\"checkbox\">\
             <input type=\"radio\">\
             <input type=\"submit\" value=\"Send\">\
             <input type=\"reset\" value=\"Clear\">\
             <select><option>x</option></select>\
             <textarea>t</textarea>\
             </body>",
    );
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    let inputs = doc.get_elements_by_tag_name("input");
    // 文本类（type=text/password）：默认 size=20 → 20*7+8 = 148px，height 15px。
    for &i in &inputs[0..2] {
        let s = styles.get(&i).expect("text input styled");
        assert!(
            matches!(s.width, LengthValue::Px(w) if (140.0..=160.0).contains(&w)),
            "text input default-size width ~148px, got {:?}",
            s.width
        );
        assert!(
            matches!(s.height, LengthValue::Px(15.0)),
            "text input content height 15px, got {:?}",
            s.height
        );
    }
    // 显式 size=10（无 type → 文本类）→ 10*7+8 = 78px（窄于默认 20）。
    let sized = styles.get(&inputs[2]).expect("size=10 input styled");
    assert!(
        matches!(sized.width, LengthValue::Px(w) if (74.0..=84.0).contains(&w)),
        "size=10 input width ~78px, got {:?}",
        sized.width
    );
    // checkbox / radio：固定 13px 方框。
    for &i in &inputs[3..5] {
        let s = styles.get(&i).expect("check input styled");
        assert!(
            matches!(s.width, LengthValue::Px(13.0)),
            "checkbox/radio width 13px, got {:?}",
            s.width
        );
        assert!(
            matches!(s.height, LengthValue::Px(13.0)),
            "checkbox/radio height 13px, got {:?}",
            s.height
        );
    }
    // submit value="Send"（4 字符）→ 4*7+14 = 42px；reset value="Clear"（5）→ 49px。
    let submit = styles.get(&inputs[5]).expect("submit styled");
    assert!(
        matches!(submit.width, LengthValue::Px(w) if (38.0..=48.0).contains(&w)),
        "submit value=Send width ~42px, got {:?}",
        submit.width
    );
    let reset = styles.get(&inputs[6]).expect("reset styled");
    assert!(
        matches!(reset.width, LengthValue::Px(w) if (45.0..=55.0).contains(&w)),
        "reset value=Clear width ~49px, got {:?}",
        reset.width
    );
    // textarea：自 R1681 起按 cols（默认 20）/rows（默认 2）注入 UA 固有尺寸。
    // <textarea>t</textarea>（无 cols/rows）→ width=20×7+8≈148px / height=2×19≈38px。
    let textarea_id = doc.get_elements_by_tag_name("textarea")[0];
    let ts = styles.get(&textarea_id).expect("textarea styled");
    assert!(
        matches!(ts.width, LengthValue::Px(w) if (140.0..=160.0).contains(&w)),
        "textarea default cols=20 width ~148px, got {:?}",
        ts.width
    );
    assert!(
        matches!(ts.height, LengthValue::Px(h) if (34.0..=42.0).contains(&h)),
        "textarea default rows=2 height ~38px, got {:?}",
        ts.height
    );
}

#[test]
fn textarea_uses_cols_rows_intrinsic_size() {
    // R1681：cols/rows 属性驱动 UA 固有尺寸（≡ R1659 input size 谱系）。
    // cols=10 rows=4 → width=10×7+8≈78px / height=4×19≈76px。
    use zero_css_parser::values::LengthValue;
    let doc = parse_html("<body><textarea cols=\"10\" rows=\"4\">txt</textarea></body>");
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    let ta = doc.get_elements_by_tag_name("textarea")[0];
    let s = styles.get(&ta).expect("textarea styled");
    assert!(
        matches!(s.width, LengthValue::Px(w) if (72.0..=84.0).contains(&w)),
        "textarea cols=10 width ~78px, got {:?}",
        s.width
    );
    assert!(
        matches!(s.height, LengthValue::Px(h) if (70.0..=82.0).contains(&h)),
        "textarea rows=4 height ~76px, got {:?}",
        s.height
    );
}

#[test]
fn select_suppresses_options_and_gets_intrinsic_width() {
    // R1679：option/optgroup UA display:none（ZW_SELECT_SUPPRESS_OPTIONS default-on）+ select
    // 固有宽 = 最宽 option 标签宽 + chrome。本测试跑在 default env（feature on）下。
    use zero_css_parser::values::{DisplayValue, LengthValue};
    assert_eq!(
        ua_default_display("option"),
        Some(DisplayValue::None),
        "option suppressed to display:none (R1679)"
    );
    assert_eq!(
        ua_default_display("optgroup"),
        Some(DisplayValue::None),
        "optgroup suppressed to display:none (R1679)"
    );

    // 两个 option：5 字符 "Volvo"（selected）+ 8 字符 "Mercedes"（最宽，决定宽）。
    let doc = parse_html(
        "<body>\
             <select>\
             <option value=\"v\" selected>Volvo</option>\
             <option value=\"m\">Mercedes</option>\
             </select>\
             </body>",
    );
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    let select = doc.get_elements_by_tag_name("select")[0];
    let s = styles.get(&select).expect("select styled");
    // 8 字符 × 7 + 24 chrome = 80px（允许 ±10 容差）。
    assert!(
        matches!(s.width, LengthValue::Px(w) if (70.0..=90.0).contains(&w)),
        "select width = widest option (Mercedes 8ch) ×7 + chrome ≈80px, got {:?}",
        s.width
    );
    // option 子应 display:none。
    for opt in doc.get_elements_by_tag_name("option") {
        let os = styles.get(&opt).expect("option styled");
        assert_eq!(os.display, DisplayValue::None, "option {opt:?} display:none");
    }
}

#[test]
fn select_intrinsic_width_uses_label_attribute() {
    // R1679：option `label` 属性优先于 text content，optgroup 内 option 也计入。
    let doc = parse_html(
        "<body>\
             <select>\
             <optgroup label=\"Swedish\">\
             <option label=\"Volvo-car\">v</option>\
             </optgroup>\
             <option>Short</option>\
             </select>\
             </body>",
    );
    let select = doc.get_elements_by_tag_name("select")[0];
    // 最宽标签 "Volvo-car"（9 字符）× 7 + 24 chrome = 87px。
    let w = select_intrinsic_width(&doc, select);
    assert!(
        (77.0..=97.0).contains(&w),
        "select intrinsic width uses widest option label (Volvo-car 9ch), got {w}"
    );
}

#[test]
fn font_element_presentational_hints() {
    let doc = parse_html("<font color=\"#990000\" face=\"Arial, Times New Roman\" size=\"5\">txt</font>");
    let font = doc.get_elements_by_tag_name("font")[0];
    let hints = collect_presentational_hints(&doc, font);
    assert!(
        hints.iter().any(|(p, v)| p == "color" && v == "#990000"),
        "font color: {hints:?}"
    );
    assert!(
        hints
            .iter()
            .any(|(p, v)| p == "font-family" && v.contains("Arial") && v.contains("\"Times New Roman\"")),
        "font face (quoted multi-word): {hints:?}"
    );
    // SIZE 暂未启用（见 html_font_size_to_em 注释 + master.md R808）。
    assert!(!hints.iter().any(|(p, _)| p == "font-size"), "size disabled: {hints:?}");
}

#[test]
fn center_element_text_align_hint() {
    let doc = parse_html("<center><p>x</p></center>");
    let center = doc.get_elements_by_tag_name("center")[0];
    let hints = collect_presentational_hints(&doc, center);
    assert!(
        hints.iter().any(|(p, v)| p == "text-align" && v == "center"),
        "center text-align hint: {hints:?}"
    );
}

#[test]
fn font_size_mapping_matches_html5_scale() {
    // 七级绝对字号（HTML5 §10.4 非线性刻度，基准 3 = 1.0em）
    assert_eq!(html_font_size_to_em("1"), Some(0.63));
    assert_eq!(html_font_size_to_em("2"), Some(0.82));
    assert_eq!(html_font_size_to_em("3"), Some(1.0));
    assert_eq!(html_font_size_to_em("4"), Some(1.13));
    assert_eq!(html_font_size_to_em("5"), Some(1.5));
    assert_eq!(html_font_size_to_em("6"), Some(2.0));
    assert_eq!(html_font_size_to_em("7"), Some(3.0));
    // 相对值从基准 3 解析
    assert_eq!(html_font_size_to_em("+2"), Some(1.5)); // 3+2=5
    assert_eq!(html_font_size_to_em("-1"), Some(0.82)); // 3-1=2
    assert_eq!(html_font_size_to_em("9"), None); // 超范围
}
