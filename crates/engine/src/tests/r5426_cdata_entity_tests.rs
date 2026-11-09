//! R5426（CDATA 段内实体不解码）：XHTML `<style>` 内容的 CDATA 段按 XML 1.0
//! §2.7（CDATA Sections）不识别 markup——字符引用按字面文本保留；段外实体
//! 照常解码（R4022）。此前 strip_cdata 先剥整体壳、R4022 臂全量解码，段内
//! `&#x46;` 等被误解码（content-177：`content:'TEST &#x46;...'` 误成 "TEST
//! FAIL"——CSS content 字符串本就不解 HTML 实体，CSS syntax-3 §String）。

use crate::pipeline::decode_xml_style_css;

/// 整体单壳段内字符引用字面保留（content-177 语义：CSS content 串不解码）。
#[test]
fn r5426_cdata_inner_references_stay_literal() {
    assert_eq!(
        decode_xml_style_css("<![CDATA[\n   .test:before { content: 'TEST &#x46;&#x41;&#x49;&#x4c;'; }\n  ]]>"),
        "\n   .test:before { content: 'TEST &#x46;&#x41;&#x49;&#x4c;'; }\n  "
    );
}

/// 整体单壳：壳剥除、段内字面保留、段外实体解码。
#[test]
fn r5426_whole_wrap_stripped_and_mixed() {
    assert_eq!(
        decode_xml_style_css("<![CDATA[ p::after { content: '&#x46;' } ]]> body &gt; div { margin: 0 }"),
        " p::after { content: '&#x46;' }  body > div { margin: 0 }"
    );
}

/// 段外实体照常解码（R4022 回归护栏：`body &gt; div` 选择器族语义不回退）。
#[test]
fn r5426_outside_cdata_still_decoded() {
    assert_eq!(
        decode_xml_style_css("body &gt; div { margin: 0 }"),
        "body > div { margin: 0 }"
    );
}

/// 多段 + 段中段（非整体单壳形态，strip_cdata 旧路径剥不掉的写法）。
#[test]
fn r5426_multiple_and_mid_text_sections() {
    assert_eq!(
        decode_xml_style_css("/* <![CDATA[ */ a &amp; b /* ]]> */ c &#62; d"),
        "/*  */ a &amp; b /*  */ c > d"
    );
    assert_eq!(
        decode_xml_style_css("x &#65; <![CDATA[y &gt; z]]> w &#66;"),
        "x A y &gt; z w B"
    );
}

/// 未闭合段：整个剩余文本按段字面保留（标记不识别）。
#[test]
fn r5426_unclosed_section_literal_to_end() {
    assert_eq!(
        decode_xml_style_css("p { color: &gt; } <![CDATA[ q &#x42;"),
        "p { color: > }  q &#x42;"
    );
}

/// 无段标记：直通 R4022 全量解码。
#[test]
fn r5426_no_section_direct_decode() {
    assert_eq!(decode_xml_style_css("a &gt; b"), "a > b");
}
