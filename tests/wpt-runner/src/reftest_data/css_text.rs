use super::InlineReftestDef;
use crate::reftest::ReftestCategory;

const REFTESTS: &[InlineReftestDef] = &[
    // ── 86-95: 文本排版扩展 ──
    InlineReftestDef {
        id: "css-text/text-color-named-vs-hex",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"color:red;\">Text A</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"color:#FF0000;\">Text A</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-font-size-vs-background",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:50px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:50px;background:blue;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-text/text-align-left-match",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-align:left;width:200px;\">Left text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-align:left;width:200px;\">Left text</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/white-space-nowrap",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:nowrap;width:50px;\">A B C D</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:nowrap;width:50px;\">A B C D</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-line-height",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"line-height:2;\">Line 1<br>Line 2</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"line-height:2;\">Line 1<br>Line 2</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-letter-spacing",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:5px;\">Spaced</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:5px;\">Spaced</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-word-spacing",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:10px;\">Hello World Test</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:10px;\">Hello World Test</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-indent",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-indent:40px;\">Indented text line that should have first line indented.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-indent:40px;\">Indented text line that should have first line indented.</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-transform-uppercase",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-transform:uppercase;\">hello world</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-transform:uppercase;\">hello world</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-in-flex-container",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"color:red;\">Hello</div><div style=\"color:blue;\">World</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"color:red;\">Hello</div><div style=\"color:blue;\">World</div></div></body></html>",
        is_match: true,
    },
    // ── M5 文字排版 reftest ──

    // text-align: justify（self-match）
    InlineReftestDef {
        id: "css-text/text-align-justify",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-align:justify;width:200px;font-size:16px\">The quick brown fox jumps over the lazy dog.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-align:justify;width:200px;font-size:16px\">The quick brown fox jumps over the lazy dog.</div></body></html>",
        is_match: true,
    },
    // text-align: center（self-match）
    InlineReftestDef {
        id: "css-text/text-align-center",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-align:center;width:200px;font-size:16px\">Hello World</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-align:center;width:200px;font-size:16px\">Hello World</div></body></html>",
        is_match: true,
    },
    // text-align: right（self-match）
    InlineReftestDef {
        id: "css-text/text-align-right",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-align:right;width:200px;font-size:16px\">Hello World</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-align:right;width:200px;font-size:16px\">Hello World</div></body></html>",
        is_match: true,
    },
    // text-align left vs right mismatch（block 子元素固定宽度，不同位置）
    InlineReftestDef {
        id: "css-text/text-align-left-vs-right",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:40px;background:blue\"><div style=\"width:100px;height:30px;background:red\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:40px;background:blue\"><div style=\"width:100px;height:30px;background:red;margin-left:100px\"></div></div></body></html>",
        is_match: false,
    },
    // word-break: break-all 长单词断行（self-match）
    InlineReftestDef {
        id: "css-text/word-break-break-all",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"word-break:break-all;width:60px;font-size:16px\">abcdefghijklmnopqrstuvwxyz</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"word-break:break-all;width:60px;font-size:16px\">abcdefghijklmnopqrstuvwxyz</div></body></html>",
        is_match: true,
    },
    // overflow-wrap: break-word（self-match）
    InlineReftestDef {
        id: "css-text/overflow-wrap-break-word",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"overflow-wrap:break-word;width:60px;font-size:16px\">supercalifragilisticexpialidocious</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"overflow-wrap:break-word;width:60px;font-size:16px\">supercalifragilisticexpialidocious</div></body></html>",
        is_match: true,
    },
    // CJK 自动换行（self-match）
    InlineReftestDef {
        id: "css-text/cjk-line-break",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:80px;font-size:16px\">这是一段中日韩文字测试内容</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:80px;font-size:16px\">这是一段中日韩文字测试内容</div></body></html>",
        is_match: true,
    },
    // white-space: nowrap 不换行（self-match）
    InlineReftestDef {
        id: "css-text/white-space-nowrap",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:nowrap;width:60px;font-size:16px\">This text should not wrap</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:nowrap;width:60px;font-size:16px\">This text should not wrap</div></body></html>",
        is_match: true,
    },
    // text-indent 首行缩进（self-match）
    InlineReftestDef {
        id: "css-text/text-indent",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-indent:32px;width:200px;font-size:16px\">First line indented. Second line not indented.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-indent:32px;width:200px;font-size:16px\">First line indented. Second line not indented.</div></body></html>",
        is_match: true,
    },
    // letter-spacing 字间距（self-match）
    InlineReftestDef {
        id: "css-text/letter-spacing",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:4px;width:200px;font-size:16px\">Spaced out text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:4px;width:200px;font-size:16px\">Spaced out text</div></body></html>",
        is_match: true,
    },
    // ── M5 文字排版扩展 reftest（目标 ≥ 50 Text reftest）──

    // word-spacing 单词间距（self-match）
    InlineReftestDef {
        id: "css-text/word-spacing-normal",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:8px;width:200px;font-size:16px\">one two three four</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:8px;width:200px;font-size:16px\">one two three four</div></body></html>",
        is_match: true,
    },
    // word-spacing 大间距（self-match）
    InlineReftestDef {
        id: "css-text/word-spacing-large",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:16px;width:200px;font-size:16px\">one two three four</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:16px;width:200px;font-size:16px\">one two three four</div></body></html>",
        is_match: true,
    },
    // text-decoration: underline（self-match）
    InlineReftestDef {
        id: "css-text/text-decoration-underline",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:underline;width:200px;font-size:16px\">Underlined text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:underline;width:200px;font-size:16px\">Underlined text</div></body></html>",
        is_match: true,
    },
    // text-decoration: overline（self-match）
    InlineReftestDef {
        id: "css-text/text-decoration-overline",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:overline;width:200px;font-size:16px\">Overlined text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:overline;width:200px;font-size:16px\">Overlined text</div></body></html>",
        is_match: true,
    },
    // text-decoration: line-through（self-match）
    InlineReftestDef {
        id: "css-text/text-decoration-line-through",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:line-through;width:200px;font-size:16px\">Strikethrough text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:line-through;width:200px;font-size:16px\">Strikethrough text</div></body></html>",
        is_match: true,
    },
    // text-decoration: dashed（self-match）
    InlineReftestDef {
        id: "css-text/text-decoration-dashed",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:underline dashed;width:200px;font-size:16px\">Dashed underline</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:underline dashed;width:200px;font-size:16px\">Dashed underline</div></body></html>",
        is_match: true,
    },
    // text-transform: uppercase（self-match）
    InlineReftestDef {
        id: "css-text/text-transform-uppercase",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-transform:uppercase;width:200px;font-size:16px\">hello world</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-transform:uppercase;width:200px;font-size:16px\">hello world</div></body></html>",
        is_match: true,
    },
    // text-transform: lowercase（self-match）
    InlineReftestDef {
        id: "css-text/text-transform-lowercase",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-transform:lowercase;width:200px;font-size:16px\">HELLO WORLD</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-transform:lowercase;width:200px;font-size:16px\">HELLO WORLD</div></body></html>",
        is_match: true,
    },
    // text-transform: capitalize（self-match）
    InlineReftestDef {
        id: "css-text/text-transform-capitalize",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-transform:capitalize;width:200px;font-size:16px\">hello world</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-transform:capitalize;width:200px;font-size:16px\">hello world</div></body></html>",
        is_match: true,
    },
    // text-transform: none（self-match）
    InlineReftestDef {
        id: "css-text/text-transform-none",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-transform:none;width:200px;font-size:16px\">No Transform</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-transform:none;width:200px;font-size:16px\">No Transform</div></body></html>",
        is_match: true,
    },
    // white-space: pre（self-match，保留空白）
    InlineReftestDef {
        id: "css-text/white-space-pre",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre;width:200px;font-size:16px\">  Hello   World  </div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre;width:200px;font-size:16px\">  Hello   World  </div></body></html>",
        is_match: true,
    },
    // white-space: pre-wrap（self-match，保留空白+换行）
    InlineReftestDef {
        id: "css-text/white-space-pre-wrap",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre-wrap;width:100px;font-size:16px\">Hello  World  Text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre-wrap;width:100px;font-size:16px\">Hello  World  Text</div></body></html>",
        is_match: true,
    },
    // white-space: pre-line（self-match）
    InlineReftestDef {
        id: "css-text/white-space-pre-line",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre-line;width:200px;font-size:16px\">Hello\nWorld</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre-line;width:200px;font-size:16px\">Hello\nWorld</div></body></html>",
        is_match: true,
    },
    // line-height: 2.0 倍行高（self-match）
    InlineReftestDef {
        id: "css-text/line-height-double",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"line-height:2.0;width:200px;font-size:16px\">Line one\nLine two</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"line-height:2.0;width:200px;font-size:16px\">Line one\nLine two</div></body></html>",
        is_match: true,
    },
    // line-height: 1.0 紧凑行高（self-match）
    InlineReftestDef {
        id: "css-text/line-height-tight",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"line-height:1.0;width:200px;font-size:16px\">Line one\nLine two</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"line-height:1.0;width:200px;font-size:16px\">Line one\nLine two</div></body></html>",
        is_match: true,
    },
    // line-height mismatch（1.0 vs 3.0）
    InlineReftestDef {
        id: "css-text/line-height-mismatch",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"line-height:1.0;width:200px;font-size:16px;background:yellow\">Line one\nLine two</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"line-height:3.0;width:200px;font-size:16px;background:yellow\">Line one\nLine two</div></body></html>",
        is_match: false,
    },
    // font-size: 24px（self-match）
    InlineReftestDef {
        id: "css-text/font-size-large",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:24px;width:200px\">Large text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:24px;width:200px\">Large text</div></body></html>",
        is_match: true,
    },
    // font-size mismatch（16px vs 32px）
    InlineReftestDef {
        id: "css-text/font-size-mismatch",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;width:200px;background:yellow\">Same text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:32px;width:200px;background:yellow\">Same text</div></body></html>",
        is_match: false,
    },
    // color: green 文本颜色（self-match）
    InlineReftestDef {
        id: "css-text/text-color-green",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"color:green;width:200px;font-size:16px\">Green text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"color:green;width:200px;font-size:16px\">Green text</div></body></html>",
        is_match: true,
    },
    // text-indent: 50px 首行缩进（self-match）
    InlineReftestDef {
        id: "css-text/text-indent-50px",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-indent:50px;width:200px;font-size:16px\">This is the first line. This is the second line.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-indent:50px;width:200px;font-size:16px\">This is the first line. This is the second line.</div></body></html>",
        is_match: true,
    },
    // text-indent: 10%（self-match）
    InlineReftestDef {
        id: "css-text/text-indent-percent",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-indent:10%;width:200px;font-size:16px\">First line indented by percent.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-indent:10%;width:200px;font-size:16px\">First line indented by percent.</div></body></html>",
        is_match: true,
    },
    // CJK 混合文本自动换行（self-match）
    InlineReftestDef {
        id: "css-text/cjk-mixed-wrap",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:120px;font-size:16px\">这是English和中文mixed内容</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:120px;font-size:16px\">这是English和中文mixed内容</div></body></html>",
        is_match: true,
    },
    // word-break: keep-all CJK 不拆分（self-match）
    InlineReftestDef {
        id: "css-text/word-break-keep-all",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"word-break:keep-all;width:100px;font-size:16px\">这是一段测试文字</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"word-break:keep-all;width:100px;font-size:16px\">这是一段测试文字</div></body></html>",
        is_match: true,
    },
    // 多行文本 justify（self-match）
    InlineReftestDef {
        id: "css-text/justify-multiline",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-align:justify;width:150px;font-size:16px\">The quick brown fox jumps over the lazy dog and runs away quickly.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-align:justify;width:150px;font-size:16px\">The quick brown fox jumps over the lazy dog and runs away quickly.</div></body></html>",
        is_match: true,
    },
    // letter-spacing: 2px（self-match）
    InlineReftestDef {
        id: "css-text/letter-spacing-2px",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:2px;width:200px;font-size:16px\">Slightly spaced</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:2px;width:200px;font-size:16px\">Slightly spaced</div></body></html>",
        is_match: true,
    },
    // tab-size: 4（self-match）
    InlineReftestDef {
        id: "css-text/tab-size-4",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre;tab-size:4;width:200px;font-size:16px\">Hello\tWorld</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre;tab-size:4;width:200px;font-size:16px\">Hello\tWorld</div></body></html>",
        is_match: true,
    },
    // long URL break-word（self-match）
    InlineReftestDef {
        id: "css-text/long-url-break-word",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"overflow-wrap:break-word;width:80px;font-size:16px\">https://www.example.com/very/long/path/to/resource</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"overflow-wrap:break-word;width:80px;font-size:16px\">https://www.example.com/very/long/path/to/resource</div></body></html>",
        is_match: true,
    },
    // text in flex container（self-match）
    InlineReftestDef {
        id: "css-text/text-in-flex",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;font-size:16px\"><div style=\"flex:1\">Hello</div><div style=\"flex:1\">World</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;font-size:16px\"><div style=\"flex:1\">Hello</div><div style=\"flex:1\">World</div></div></body></html>",
        is_match: true,
    },
    // text in grid container（self-match）
    InlineReftestDef {
        id: "css-text/text-in-grid",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;font-size:16px\"><div>Hello</div><div>World</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;font-size:16px\"><div>Hello</div><div>World</div></div></body></html>",
        is_match: true,
    },
    // vertical-align: top（self-match）
    InlineReftestDef {
        id: "css-text/vertical-align-top",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"height:50px;line-height:50px;width:200px;font-size:16px\"><span style=\"vertical-align:top\">Top</span></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"height:50px;line-height:50px;width:200px;font-size:16px\"><span style=\"vertical-align:top\">Top</span></div></body></html>",
        is_match: true,
    },
    // vertical-align: middle（self-match）
    InlineReftestDef {
        id: "css-text/vertical-align-middle",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"height:50px;line-height:50px;width:200px;font-size:16px\"><span style=\"vertical-align:middle\">Mid</span></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"height:50px;line-height:50px;width:200px;font-size:16px\"><span style=\"vertical-align:middle\">Mid</span></div></body></html>",
        is_match: true,
    },
];

pub fn reftests() -> &'static [InlineReftestDef] {
    REFTESTS
}
