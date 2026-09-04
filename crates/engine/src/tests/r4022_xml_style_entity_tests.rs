//! R4022（XML #PCDATA 实体解码）：XHTML 文档（content_is_xml）的 `<style>` 内容
//! 按 XML 语义是已解析字符数据——`&gt;` 等实体须在进 CSS 解析前解码。
//! HTML tokenizer 对 <style> raw text 不解码实体（HTML 语义），ZW 此前致
//! `body &gt; div` 以实体串进入选择器（inline-table-width-001a/001b/002a/002b、
//! inline-block-zorder-005 族规则全丢）。

use crate::pipeline::decode_xml_char_references;

/// 基本五实体 + 数字/十六进制字符引用。
#[test]
fn r4022_decodes_named_and_numeric_references() {
    assert_eq!(decode_xml_char_references("body &gt; div"), "body > div");
    assert_eq!(decode_xml_char_references("a &lt; b &amp;&amp; c"), "a < b && c");
    assert_eq!(decode_xml_char_references("&quot;q&quot; &apos;a&apos;"), "\"q\" 'a'");
    assert_eq!(decode_xml_char_references("&#62; &#x3E;"), "> >");
    assert_eq!(decode_xml_char_references("&#x2603;"), "☃");
}

/// 无 `&` 的 CSS 原样返回（零分配快路径语义）。
#[test]
fn r4022_passthrough_without_ampersand() {
    assert_eq!(decode_xml_char_references("div { width: 10em }"), "div { width: 10em }");
}

/// 未知实体/缺分号原样保留（宽容——CSS 内容中字面 `&`（如自定义属性值）不受损）。
#[test]
fn r4022_unknown_entity_preserved() {
    assert_eq!(decode_xml_char_references("&nbsp; &x a & b"), "&nbsp; &x a & b");
}
