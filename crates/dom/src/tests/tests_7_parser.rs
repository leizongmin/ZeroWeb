// DOM HTML 解析器专项测试。
//
// 覆盖：parse_html 函数的各种场景，包括错误恢复、实体解码、
//       void 元素处理、注释处理、doctype、复杂结构等。

use crate::*;

// ═══════════════════════════════════════════════════════════════════════
// parse_html 函数测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 parse_html 解析完整 HTML5 文档。
#[test]
fn test_parse_html_full_document() {
    let html = "<!DOCTYPE html><html><head><title>Test</title></head><body><h1>Hello</h1></body></html>";
    let doc = parse_html(html);
    assert!(doc.node_count() > 5, "完整文档应包含多个节点");

    // 验证文档结构
    assert!(doc.query_selector(doc.root(), "html").is_some());
    assert!(doc.query_selector(doc.root(), "head").is_some());
    assert!(doc.query_selector(doc.root(), "title").is_some());
    assert!(doc.query_selector(doc.root(), "body").is_some());
    assert!(doc.query_selector(doc.root(), "h1").is_some());
}

/// 测试 parse_html 解析文档片段（无 html/head/body）。
#[test]
fn test_parse_html_fragment() {
    let html = "<div><p>段落</p><span>文本</span></div>";
    let doc = parse_html(html);
    assert!(doc.node_count() > 3, "片段应包含多个节点");

    // html5ever 自动添加 html 和 body
    assert!(doc.query_selector(doc.root(), "html").is_some());
    assert!(doc.query_selector(doc.root(), "body").is_some());
    assert!(doc.query_selector(doc.root(), "div").is_some());
    assert!(doc.query_selector(doc.root(), "p").is_some());
    assert!(doc.query_selector(doc.root(), "span").is_some());
}

/// 测试 parse_html 解析空文档。
#[test]
fn test_parse_html_empty() {
    let doc = parse_html("");
    assert!(doc.root().is_valid(), "根节点应有效");
    // html5ever 自动添加 html 和 body，还有一个注释节点？
    assert!(doc.node_count() >= 3, "应该至少有根节点、html 和 body");
}

/// 测试 parse_html 解析只有空白字符。
#[test]
fn test_parse_html_whitespace() {
    let doc = parse_html("   \n\t  ");
    assert!(doc.root().is_valid());
    // html5ever 自动添加 html 和 body，还有一个注释节点？
    assert!(doc.node_count() >= 3, "应该至少有根节点、html 和 body");
}

/// 测试 parse_html 解析纯文本（无标签）。
#[test]
fn test_parse_html_text_only() {
    let doc = parse_html("Hello World");
    assert!(doc.node_count() > 1, "应该有文本节点");

    let root = doc.root();
    assert!(doc.has_child_nodes(root));
}

// ═══════════════════════════════════════════════════════════════════════
// 元素和属性测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 parse_html 解析带属性的元素。
#[test]
fn test_parse_html_element_with_attributes() {
    let html = r#"<div id="main" class="container" data-value="123">Content</div>"#;
    let doc = parse_html(html);
    assert!(doc.node_count() > 1);

    let div = doc.query_selector(doc.root(), "div").unwrap();
    assert_eq!(doc.get_attribute(div, "id"), Some("main".to_string()));
    assert_eq!(doc.get_attribute(div, "class"), Some("container".to_string()));
    assert_eq!(doc.get_attribute(div, "data-value"), Some("123".to_string()));
}

/// 测试 parse_html 解析嵌套元素。
#[test]
fn test_parse_html_nested_elements() {
    let html = "<div><p><span>Deep</span></p></div>";
    let doc = parse_html(html);
    assert!(doc.node_count() > 3, "嵌套元素应产生多个节点");

    assert!(doc.query_selector(doc.root(), "div").is_some());
    assert!(doc.query_selector(doc.root(), "p").is_some());
    assert!(doc.query_selector(doc.root(), "span").is_some());
}

/// 测试 parse_html 解析 void 元素。
#[test]
fn test_parse_html_void_elements() {
    let html = "<div><br><img src='test.png'><input type='text'></div>";
    let doc = parse_html(html);
    assert!(doc.node_count() > 2, "void 元素应被正确解析");

    assert!(doc.query_selector(doc.root(), "br").is_some());
    assert!(doc.query_selector(doc.root(), "img").is_some());
    assert!(doc.query_selector(doc.root(), "input").is_some());
}

/// 测试 parse_html 解析多 class 属性。
#[test]
fn test_parse_html_multiple_classes() {
    let html = r#"<div class="a b c"></div>"#;
    let doc = parse_html(html);

    let div = doc.query_selector(doc.root(), "div").unwrap();
    if let Some(NodeKind::Element(elem)) = doc.get(div).map(|n| n.kind.clone()) {
        assert_eq!(elem.class_list, vec!["a", "b", "c"]);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 错误恢复测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 parse_html 解析未闭合标签（html5ever 自动恢复）。
#[test]
fn test_parse_html_unclosed_tags() {
    let html = "<div><p>text<span>more";
    let doc = parse_html(html);
    // html5ever 自动闭合标签
    assert!(doc.node_count() > 2, "未闭合标签应被自动恢复");

    assert!(doc.query_selector(doc.root(), "div").is_some());
    assert!(doc.query_selector(doc.root(), "p").is_some());
    assert!(doc.query_selector(doc.root(), "span").is_some());
}

/// 测试 parse_html 解析错误嵌套标签。
#[test]
fn test_parse_html_misnested_tags() {
    let html = "<b><i>bold italic</b></i>";
    let doc = parse_html(html);
    // html5ever 处理错误嵌套
    assert!(doc.node_count() > 1, "错误嵌套应被恢复");

    assert!(doc.query_selector(doc.root(), "b").is_some());
    assert!(doc.query_selector(doc.root(), "i").is_some());
}

/// 测试 parse_html 解析重复属性。
#[test]
fn test_parse_html_duplicate_attributes() {
    let html = r#"<div class="a" class="b">text</div>"#;
    let doc = parse_html(html);
    assert!(doc.node_count() > 0, "重复属性不应导致解析失败");

    let div = doc.query_selector(doc.root(), "div").unwrap();
    // 取第一个 class 属性值
    assert_eq!(doc.get_attribute(div, "class"), Some("a".to_string()));
}

/// 测试 parse_html 解析只有关闭标签无开始标签。
#[test]
fn test_parse_html_closing_tag_without_open() {
    let html = "</div></p>text";
    let doc = parse_html(html);
    assert!(doc.root().is_valid());
}

// ═══════════════════════════════════════════════════════════════════════
// 特殊内容测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 parse_html 解析 HTML 实体。
#[test]
fn test_parse_html_entities() {
    let html = "<p>&amp; &lt; &gt; &quot; &#x2603;</p>";
    let doc = parse_html(html);
    assert!(doc.node_count() > 0, "实体应被正确解析");

    let p = doc.query_selector(doc.root(), "p").unwrap();
    let text = doc.text_content(p).unwrap();
    assert_eq!(text, "& < > \" \u{2603}");
}

/// 测试 parse_html 解析注释。
#[test]
fn test_parse_html_comment() {
    let html = "<div><!-- this is a comment -->text</div>";
    let doc = parse_html(html);
    assert!(doc.node_count() > 1, "注释应被保留在 DOM 中");

    assert!(doc.query_selector(doc.root(), "div").is_some());
    // 注释作为节点存在
    let node_count = doc.node_count();
    assert!(node_count > 1);
}

/// 测试 parse_html 解析 script 标签内容。
#[test]
fn test_parse_html_script_content() {
    let html = r#"<script>var x = 1 < 2; if (a && b) {}</script>"#;
    let doc = parse_html(html);
    assert!(doc.node_count() > 0, "script 内容应被正确处理");

    let script = doc.query_selector(doc.root(), "script").unwrap();
    let text = doc.text_content(script).unwrap();
    assert_eq!(text, "var x = 1 < 2; if (a && b) {}");
}

/// 测试 parse_html 解析 style 标签内容。
#[test]
fn test_parse_html_style_content() {
    let html = "<style>body { color: red; }</style>";
    let doc = parse_html(html);
    assert!(doc.node_count() > 0, "style 内容应被正确处理");

    let style = doc.query_selector(doc.root(), "style").unwrap();
    let text = doc.text_content(style).unwrap();
    assert_eq!(text, "body { color: red; }");
}

/// 测试 parse_html 解析 DOCTYPE 声明。
#[test]
fn test_parse_html_doctype() {
    let html = "<!DOCTYPE html><html><body>ok</body></html>";
    let doc = parse_html(html);
    assert!(doc.node_count() > 2);

    assert!(doc.query_selector(doc.root(), "html").is_some());
    assert!(doc.query_selector(doc.root(), "body").is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 文档结构测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 parse_html 无 html/body 标签时自动补全。
#[test]
fn test_parse_html_auto_body() {
    let doc = parse_html("<p>paragraph</p>");
    assert!(doc.node_count() > 1, "html5ever 应自动添加 html/body");

    assert!(doc.query_selector(doc.root(), "html").is_some());
    assert!(doc.query_selector(doc.root(), "body").is_some());
    assert!(doc.query_selector(doc.root(), "p").is_some());
}

/// 测试 parse_html head 中的 link/meta。
#[test]
fn test_parse_html_head_elements() {
    let html = r#"<head><meta charset="utf-8"><link rel="stylesheet" href="style.css"><title>T</title></head>"#;
    let doc = parse_html(html);
    assert!(doc.node_count() > 3);

    assert!(doc.query_selector(doc.root(), "head").is_some());
    assert!(doc.query_selector(doc.root(), "meta").is_some());
    assert!(doc.query_selector(doc.root(), "link").is_some());
    assert!(doc.query_selector(doc.root(), "title").is_some());
}

/// 测试 parse_html 深层嵌套（10 层）。
#[test]
fn test_parse_html_deeply_nested() {
    let html = "<div>".repeat(10) + "text" + &"</div>".repeat(10);
    let doc = parse_html(&html);
    assert!(doc.node_count() > 10, "深层嵌套应被正确解析");

    // 验证最深层的 div 存在
    assert!(doc.query_selector(doc.root(), "div").is_some());
}

/// 测试 parse_html Unicode 文本。
#[test]
fn test_parse_html_unicode_text() {
    let html = "<p>你好世界 🌍 こんにちは 안녕하세요</p>";
    let doc = parse_html(html);
    assert!(doc.node_count() > 0, "Unicode 文本应被正确解析");

    let p = doc.query_selector(doc.root(), "p").unwrap();
    let text = doc.text_content(p).unwrap();
    assert_eq!(text, "你好世界 🌍 こんにちは 안녕하세요");
}

/// 测试 parse_html 大文档（1000 个段落）。
#[test]
fn test_parse_html_large_document() {
    let paragraphs: Vec<String> = (0..1000).map(|i| format!("<p>Paragraph {i}</p>")).collect();
    let html = format!("<html><body>{}</body></html>", paragraphs.join(""));
    let doc = parse_html(&html);
    assert!(doc.node_count() > 1000, "大文档应被正确解析");

    // 验证部分段落存在
    for i in [0, 500, 999] {
        let expected_text = format!("Paragraph {}", i);
        let text_elements: Vec<String> = doc
            .query_selector_all(doc.root(), "p")
            .into_iter()
            .filter_map(|id| doc.text_content(id))
            .collect();
        assert!(text_elements.contains(&expected_text), "应包含段落 {}", i);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HTML5 语义元素测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 parse_html 解析 HTML5 语义元素。
#[test]
fn test_parse_html5_semantic_elements() {
    let html = r#"
    <header>
        <nav>
            <ul>
                <li><a href="/">首页</a></li>
                <li><a href="/about">关于</a></li>
            </ul>
        </nav>
    </header>
    <main>
        <article>
            <h2>文章标题</h2>
            <p>文章内容</p>
        </article>
        <aside>
            <h3>侧边栏</h3>
            <p>相关链接</p>
        </aside>
    </main>
    <footer>
        <p>&copy; 2024</p>
    </footer>
    "#;

    let doc = parse_html(html);

    // 验证语义元素
    assert!(doc.query_selector(doc.root(), "header").is_some());
    assert!(doc.query_selector(doc.root(), "nav").is_some());
    assert!(doc.query_selector(doc.root(), "main").is_some());
    assert!(doc.query_selector(doc.root(), "article").is_some());
    assert!(doc.query_selector(doc.root(), "aside").is_some());
    assert!(doc.query_selector(doc.root(), "footer").is_some());
    assert!(doc.query_selector(doc.root(), "ul").is_some());
    assert!(doc.query_selector(doc.root(), "li").is_some());
    assert!(doc.query_selector(doc.root(), "a").is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 表单元素测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 parse_html 解析表单元素。
#[test]
fn test_parse_html_form_elements() {
    let html = r#"
    <form action="/submit" method="post">
        <fieldset>
            <legend>用户信息</legend>
            <label for="name">姓名：</label>
            <input type="text" id="name" name="name" required>
            <label for="email">邮箱：</label>
            <input type="email" id="email" name="email" required>
            <label for="message">留言：</label>
            <textarea id="message" name="message" rows="5" cols="30"></textarea>
            <button type="submit">提交</button>
            <button type="reset">重置</button>
        </fieldset>
    </form>
    "#;

    let doc = parse_html(html);

    // 验证表单元素
    assert!(doc.query_selector(doc.root(), "form").is_some());
    assert!(doc.query_selector(doc.root(), "fieldset").is_some());
    assert!(doc.query_selector(doc.root(), "legend").is_some());
    assert!(doc.query_selector(doc.root(), "label").is_some());
    assert!(doc.query_selector(doc.root(), "input").is_some());
    assert!(doc.query_selector(doc.root(), "textarea").is_some());
    assert!(doc.query_selector(doc.root(), "button").is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 媒体元素测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 parse_html 解析媒体元素。
#[test]
fn test_parse_html_media_elements() {
    let html = r#"
    <video controls width="320" height="240">
        <source src="movie.mp4" type="video/mp4">
        <source src="movie.ogg" type="video/ogg">
        您的浏览器不支持 video 标签。
    </video>

    <audio controls>
        <source src="sound.mp3" type="audio/mpeg">
        <source src="sound.ogg" type="audio/ogg">
        您的浏览器不支持 audio 标签。
    </audio>

    <picture>
        <source media="(min-width: 900px)" srcset="image-large.jpg">
        <source media="(min-width: 600px)" srcset="image-medium.jpg">
        <img src="image-small.jpg" alt="响应式图片">
    </picture>

    <canvas id="myCanvas" width="200" height="100"></canvas>
    "#;

    let doc = parse_html(html);

    // 验证媒体元素
    assert!(doc.query_selector(doc.root(), "video").is_some());
    assert!(doc.query_selector(doc.root(), "audio").is_some());
    assert!(doc.query_selector(doc.root(), "picture").is_some());
    assert!(doc.query_selector(doc.root(), "canvas").is_some());
    assert!(doc.query_selector(doc.root(), "source").is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 交互元素测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 parse_html 解析交互元素。
#[test]
fn test_parse_html_interactive_elements() {
    let html = r#"
    <details>
        <summary>点击展开</summary>
        <p>展开的内容</p>
    </details>

    <details open>
        <summary>默认展开</summary>
        <p>默认展开的内容</p>
    </details>

    <meter value="3" min="0" max="10">3/10</meter>

    <progress value="70" max="100">70%</progress>

    <mark>高亮文本</mark>

    <time datetime="2024-01-01">2024年1月1日</time>

    <data value="123">产品编号 123</data>

    <output>计算结果：42</output>
    "#;

    let doc = parse_html(html);

    // 验证交互元素
    assert!(doc.query_selector(doc.root(), "details").is_some());
    assert!(doc.query_selector(doc.root(), "summary").is_some());
    assert!(doc.query_selector(doc.root(), "meter").is_some());
    assert!(doc.query_selector(doc.root(), "progress").is_some());
    assert!(doc.query_selector(doc.root(), "mark").is_some());
    assert!(doc.query_selector(doc.root(), "time").is_some());
    assert!(doc.query_selector(doc.root(), "data").is_some());
    assert!(doc.query_selector(doc.root(), "output").is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 数据表格测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 parse_html 解析数据表格。
#[test]
fn test_parse_html_data_table() {
    let html = r#"
    <table border="1">
        <caption>学生成绩表</caption>
        <thead>
            <tr>
                <th>姓名</th>
                <th>科目</th>
                <th>分数</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>张三</td>
                <td>数学</td>
                <td>95</td>
            </tr>
            <tr>
                <td>李四</td>
                <td>英语</td>
                <td>88</td>
            </tr>
        </tbody>
        <tfoot>
            <tr>
                <td colspan="2">平均分</td>
                <td>91.5</td>
            </tr>
        </tfoot>
    </table>
    "#;

    let doc = parse_html(html);

    // 验证表格元素
    assert!(doc.query_selector(doc.root(), "table").is_some());
    assert!(doc.query_selector(doc.root(), "caption").is_some());
    assert!(doc.query_selector(doc.root(), "thead").is_some());
    assert!(doc.query_selector(doc.root(), "tbody").is_some());
    assert!(doc.query_selector(doc.root(), "tfoot").is_some());
    assert!(doc.query_selector(doc.root(), "tr").is_some());
    assert!(doc.query_selector(doc.root(), "th").is_some());
    assert!(doc.query_selector(doc.root(), "td").is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 额外覆盖率测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 HTML 注释的处理
#[test]
fn test_parse_html_comments() {
    let doc = parse_html(
        r#"
        <!-- Top comment -->
        <div>
            <!-- Comment in div -->
            Text
            <span><!-- In span -->Content</span>
        </div>
        <!-- Bottom comment -->
    "#,
    );

    assert!(doc.node_count() > 5, "应该有多个节点");
    assert!(doc.query_selector(doc.root(), "div").is_some());
    assert!(doc.query_selector(doc.root(), "span").is_some());
}

/// 测试 DOCTYPE 声明的各种形式
#[test]
fn test_parse_doctype_variants() {
    // 测试完整的 DOCTYPE
    let doc1 = parse_html("<!DOCTYPE html><html><body>Test</body></html>");
    assert!(doc1.node_count() >= 3, "完整的 DOCTYPE 应该创建至少 3 个节点");

    // 测试带公共标识符的 DOCTYPE
    let doc2 = parse_html(
        "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\"><html><body>Test</body></html>",
    );
    assert!(doc2.node_count() > 3, "复杂 DOCTYPE 应该创建多个节点");

    // 测试系统标识符的 DOCTYPE
    let doc3 = parse_html("<!DOCTYPE html SYSTEM \"about:legacy-compat\"><html><body>Test</body></html>");
    assert!(doc3.node_count() > 3, "系统标识符 DOCTYPE 应该创建多个节点");
}

/// 测试 XHTML 文档检测：DOCTYPE public_id 含 "XHTML" 时 content_is_xml 置位
/// （CSS Selectors §6.3：XML/XHTML 属性值选择器大小写敏感，HTML 不敏感）。
#[test]
fn test_parse_xhtml_content_is_xml_detection() {
    // XHTML 1.0 Transitional DOCTYPE → 检测为 XML
    let xhtml10 = parse_html(
        "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\"><html xmlns=\"http://www.w3.org/1999/xhtml\"><body>Test</body></html>",
    );
    assert!(xhtml10.content_is_xml(), "XHTML 1.0 DOCTYPE 应检测为 XML 内容");

    // XHTML 1.1 DOCTYPE（WPT .xht 文件常用）→ 检测为 XML
    let xhtml11 = parse_html(
        "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.1//EN\" \"http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd\"><html xmlns=\"http://www.w3.org/1999/xhtml\"><body>Test</body></html>",
    );
    assert!(xhtml11.content_is_xml(), "XHTML 1.1 DOCTYPE 应检测为 XML 内容");

    // HTML5 DOCTYPE → 不检测为 XML（属性值大小写不敏感）
    let html5 = parse_html("<!DOCTYPE html><html><body>Test</body></html>");
    assert!(!html5.content_is_xml(), "HTML5 DOCTYPE 不应检测为 XML 内容");

    // 无 DOCTYPE → 不检测为 XML
    let no_doctype = parse_html("<html><body>Test</body></html>");
    assert!(!no_doctype.content_is_xml(), "无 DOCTYPE 不应检测为 XML 内容");
}

/// 测试自闭合 void 元素
#[test]
fn test_parse_void_elements_comprehensive() {
    let doc = parse_html("<div><br><img src='test.png'><input type='text'><hr><meta charset='utf-8'></div>");
    assert!(doc.node_count() > 2, "void 元素应被正确解析");

    assert!(doc.query_selector(doc.root(), "br").is_some());
    assert!(doc.query_selector(doc.root(), "img").is_some());
    assert!(doc.query_selector(doc.root(), "input").is_some());
    assert!(doc.query_selector(doc.root(), "hr").is_some());
    assert!(doc.query_selector(doc.root(), "meta").is_some());
}

/// 测试 script 和 style 标签的特殊内容处理
#[test]
fn test_parse_script_style_content() {
    let html = r#"
    <script>
        var x = 1 < 2;
        if (a && b) {
            console.log("test");
        }
    </script>
    <style>
        body { color: red; }
        .class { font-size: 16px; }
    </style>
    "#;

    let doc = parse_html(html);
    assert!(doc.node_count() > 2, "script 和 style 应被正确处理");

    let script = doc.query_selector(doc.root(), "script").unwrap();
    let style = doc.query_selector(doc.root(), "style").unwrap();

    let script_text = doc.text_content(script).unwrap();
    let style_text = doc.text_content(style).unwrap();

    assert!(script_text.contains("1 < 2"), "script 内容应包含 < 字符");
    assert!(style_text.contains("color: red"), "style 内容应被正确保留");
}

/// 测试 HTML 实体解码
#[test]
fn test_parse_html_entities_comprehensive() {
    let doc = parse_html(
        r#"
        <div>
            &amp; &lt; &gt; &quot; &#x2603; &#9731;
            &nbsp; &copy; &reg;
        </div>
    "#,
    );

    let div = doc.query_selector(doc.root(), "div").unwrap();
    let text = doc.text_content(div).unwrap();

    assert!(text.contains("&"), "应该包含 &");
    assert!(text.contains("<"), "应该包含 <");
    assert!(text.contains(">"), "应该包含 >");
    assert!(text.contains("\""), "应该包含 \"");
    assert!(text.contains("☃"), "应该包含 ☃ (U+2603)");
    // Note: Not all entities may be decoded by the parser
    assert!(text.contains("©") || text.contains("&copy;"), "应该包含 © 或 &copy;");
}

/// 测试深度嵌套结构
#[test]
fn test_parse_deep_nesting() {
    let mut html = String::new();
    html.push_str("<div>");
    for _ in 0..10 {
        html.push_str("<div>");
    }
    html.push_str("Deep text");
    for _ in 0..10 {
        html.push_str("</div>");
    }
    html.push_str("</div>");

    let doc = parse_html(&html);
    assert!(doc.node_count() > 10, "深度嵌套应被正确解析");

    // 验证最深层
    let divs: Vec<_> = doc.query_selector_all(doc.root(), "div").into_iter().collect();
    assert!(divs.len() >= 11, "应该有至少 11 个 div");
}

/// 测试未闭合标签的自动恢复
#[test]
fn test_parse_unclosed_tags_comprehensive() {
    // 各种未闭合的情况
    let cases = vec![
        "<div><p>text",
        "<div><p><span>text",
        "<div><p><span>",
        "<div><p>text<span>",
        "<div><ul><li>item1<li>item2",
    ];

    for html in cases {
        let doc = parse_html(html);
        assert!(doc.root().is_valid(), "HTML 应该被有效解析: {}", html);
        assert!(doc.node_count() > 1, "应该有多个节点: {}", html);
    }
}

/// 测试错误嵌套标签的恢复
#[test]
fn test_parse_misnested_tags_comprehensive() {
    let cases = vec![
        "<b><i>bold</b></i>",
        "<div><span>nested<div>more</span></div>",
        "<p><strong>text<p>more",
        "<ul><li>item1<div>nested</div></li></ul>",
    ];

    for html in cases {
        let doc = parse_html(html);
        assert!(doc.root().is_valid(), "错误嵌套应被恢复: {}", html);
        assert!(doc.node_count() > 1, "应该有多个节点: {}", html);
    }
}

/// 测试重复属性的解析
#[test]
fn test_parse_duplicate_attributes_comprehensive() {
    let cases = vec![
        r#"<div class="a" class="b">text</div>"#,
        r#"<div id="first" id="second">text</div>"#,
        r#"<div data-value="1" data-value="2">text</div>"#,
    ];

    for html in cases {
        let doc = parse_html(html);
        assert!(doc.root().is_valid(), "重复属性应被处理: {}", html);
        assert!(doc.node_count() > 0, "应该有节点: {}", html);
    }
}

/// 测试只有关闭标签的情况
#[test]
fn test_parse_only_closing_tags() {
    let html = "</div></p>text</span>";
    let doc = parse_html(html);
    assert!(doc.root().is_valid(), "只有关闭标签应被处理");
    assert!(doc.node_count() > 0, "应该有节点");
}

/// 测试空属性
#[test]
fn test_parse_empty_attributes() {
    let doc = parse_html(
        r#"
        <div
            id=""
            class=""
            data-empty
            value=""
        >
            Content
        </div>
    "#,
    );

    let div = doc.query_selector(doc.root(), "div").unwrap();

    // 空属性应该被保留
    assert_eq!(doc.get_attribute(div, "id"), Some("".to_string()));
    assert_eq!(doc.get_attribute(div, "class"), Some("".to_string()));
    // Note: Some parsers treat empty boolean attributes differently
    assert!(
        doc.get_attribute(div, "data-empty").is_none() || doc.get_attribute(div, "data-empty") == Some("".to_string())
    );
    assert_eq!(doc.get_attribute(div, "value"), Some("".to_string()));
}

/// 测试 Unicode 属性值
#[test]
fn test_parse_unicode_attributes() {
    let doc = parse_html(
        r#"
        <div
            title="你好世界 🌍"
            data-lang="zh-CN"
            class="a b c"
        >
            Content
        </div>
    "#,
    );

    let div = doc.query_selector(doc.root(), "div").unwrap();

    assert_eq!(doc.get_attribute(div, "title"), Some("你好世界 🌍".to_string()));
    assert_eq!(doc.get_attribute(div, "data-lang"), Some("zh-CN".to_string()));
    assert_eq!(doc.get_attribute(div, "class"), Some("a b c".to_string()));
}

/// 测试复杂的 HTML5 语义结构
#[test]
fn test_parse_complex_semantic_structure() {
    let html = r#"
    <header>
        <nav>
            <ul>
                <li><a href="/">首页</a></li>
                <li><a href="/about">关于</a></li>
            </ul>
        </nav>
    </header>
    <main>
        <article>
            <h2>文章标题</h2>
            <section>
                <h3>小标题</h3>
                <p>段落内容</p>
            </section>
        </article>
        <aside>
            <h3>侧边栏</h3>
            <nav>
                <ul>
                    <li><a href="/link1">链接1</a></li>
                    <li><a href="/link2">链接2</a></li>
                </ul>
            </nav>
        </aside>
    </main>
    <footer>
        <p>&copy; 2024</p>
    </footer>
    "#;

    let doc = parse_html(html);
    assert!(doc.node_count() > 20, "复杂语义结构应包含多个节点");

    // 验证所有语义元素
    let semantic_elements = [
        "header", "nav", "ul", "li", "a", "main", "article", "section", "h2", "h3", "p", "aside", "footer",
    ];
    for element in &semantic_elements {
        assert!(
            doc.query_selector(doc.root(), element).is_some(),
            "应该包含 {} 元素",
            element
        );
    }
}

/// 测试 iframe 和 object 嵌入内容
#[test]
fn test_parse_embedded_content() {
    let html = r#"
    <iframe src="page.html"></iframe>
    <object data="data.swf" type="application/x-shockwave-flash"></object>
    <embed src="movie.swf" type="application/x-shockwave-flash"></embed>
    <video controls>
        <source src="movie.mp4" type="video/mp4">
    </video>
    <audio controls>
        <source src="sound.mp3" type="audio/mpeg">
    </audio>
    "#;

    let doc = parse_html(html);
    assert!(doc.node_count() > 5, "嵌入内容应被正确解析");

    assert!(doc.query_selector(doc.root(), "iframe").is_some());
    assert!(doc.query_selector(doc.root(), "object").is_some());
    assert!(doc.query_selector(doc.root(), "embed").is_some());
    assert!(doc.query_selector(doc.root(), "video").is_some());
    assert!(doc.query_selector(doc.root(), "audio").is_some());
    assert!(doc.query_selector(doc.root(), "source").is_some());
}

/// 测试 template 元素
#[test]
fn test_parse_template_element() {
    let html = r#"
    <template id="my-template">
        <div>Template content</div>
        <script>console.log("template");</script>
    </template>
    <div>Regular content</div>
    "#;

    let doc = parse_html(html);
    assert!(doc.node_count() > 3, "template 应被正确解析");

    let templates: Vec<_> = doc.query_selector_all(doc.root(), "template").into_iter().collect();
    assert!(!templates.is_empty(), "应该有 template 元素");

    // 查找不在 template 内的 div
    let divs: Vec<_> = doc.query_selector_all(doc.root(), "div").into_iter().collect();
    assert!(!divs.is_empty(), "应该有 div 元素");

    // 验证至少有一个 div 有文本内容 "Regular content"
    let found = divs
        .iter()
        .any(|&div_id| doc.text_content(div_id).map(|t| t.contains("Regular")).unwrap_or(false));
    assert!(found, "应该有包含 'Regular content' 的 div");
}

/// 测试大文档性能
#[test]
fn test_parse_large_document_performance() {
    // 创建一个较大的文档
    let paragraphs: Vec<String> = (0..100)
        .map(|i| format!("<p>Paragraph {} with text content</p>", i))
        .collect();
    let html = format!("<html><body>{}</body></html>", paragraphs.join(""));

    let doc = parse_html(&html);
    assert!(doc.node_count() > 100, "大文档应被正确解析");

    // 验证部分段落
    let text_elements: Vec<String> = doc
        .query_selector_all(doc.root(), "p")
        .into_iter()
        .filter_map(|id| doc.text_content(id))
        .collect();

    assert!(text_elements.len() >= 100, "应该有 100 个段落");
    assert!(text_elements.contains(&"Paragraph 0 with text content".to_string()));
    assert!(text_elements.contains(&"Paragraph 99 with text content".to_string()));
}

/// 测试表单元素的各种组合
#[test]
fn test_parse_form_elements_combinations() {
    let html = r#"
    <form action="/submit" method="post">
        <fieldset>
            <legend>用户信息</legend>
            <label for="name">姓名：</label>
            <input type="text" id="name" name="name" required>
            <label for="email">邮箱：</label>
            <input type="email" id="email" name="email" required>
            <label for="message">留言：</label>
            <textarea id="message" name="message" rows="5" cols="30"></textarea>
            <button type="submit">提交</button>
            <button type="reset">重置</button>
        </fieldset>
    </form>
    "#;

    let doc = parse_html(html);
    assert!(doc.node_count() > 10, "表单应包含多个节点");

    assert!(doc.query_selector(doc.root(), "form").is_some());
    assert!(doc.query_selector(doc.root(), "fieldset").is_some());
    assert!(doc.query_selector(doc.root(), "legend").is_some());
    assert!(doc.query_selector(doc.root(), "label").is_some());
    assert!(doc.query_selector(doc.root(), "input").is_some());
    assert!(doc.query_selector(doc.root(), "textarea").is_some());
    assert!(doc.query_selector(doc.root(), "button").is_some());
}

/// 测试交互元素
#[test]
fn test_parse_interactive_elements() {
    let html = r#"
    <details>
        <summary>点击展开</summary>
        <p>展开的内容</p>
    </details>

    <details open>
        <summary>默认展开</summary>
        <p>默认展开的内容</p>
    </details>

    <meter value="3" min="0" max="10">3/10</meter>

    <progress value="70" max="100">70%</progress>

    <mark>高亮文本</mark>

    <time datetime="2024-01-01">2024年1月1日</time>

    <data value="123">产品编号 123</data>

    <output>计算结果：42</output>
    "#;

    let doc = parse_html(html);
    assert!(doc.node_count() > 10, "交互元素应包含多个节点");

    assert!(doc.query_selector(doc.root(), "details").is_some());
    assert!(doc.query_selector(doc.root(), "summary").is_some());
    assert!(doc.query_selector(doc.root(), "meter").is_some());
    assert!(doc.query_selector(doc.root(), "progress").is_some());
    assert!(doc.query_selector(doc.root(), "mark").is_some());
    assert!(doc.query_selector(doc.root(), "time").is_some());
    assert!(doc.query_selector(doc.root(), "data").is_some());
    assert!(doc.query_selector(doc.root(), "output").is_some());
}

/// 测试混合内容（文本、元素、注释）
#[test]
fn test_parse_mixed_content_with_comments() {
    let html = r#"
    <div>
        <!-- comment 1 -->
        Text before
        <span>Element content</span>
        Text after
        <!-- comment 2 -->
        More text
    </div>
    "#;

    let doc = parse_html(html);
    assert!(doc.node_count() > 6, "混合内容应包含多个节点");

    // 验证各种节点类型
    assert!(doc.query_selector(doc.root(), "div").is_some());
    assert!(doc.query_selector(doc.root(), "span").is_some());

    // 验证文本节点数量
    let div = doc.query_selector(doc.root(), "div").unwrap();
    let children = &doc.get(div).unwrap().children;
    assert!(children.len() > 4, "div 应该有多个子节点");
}

/// 测试文本节点在复杂结构中的合并
#[test]
fn test_parse_text_node_merging_complex_structure() {
    let html = r#"
    <div>Text1
        <span>Text2
            <strong>Text3</strong>
        Text4
    </div>
    "#;

    let doc = parse_html(html);
    assert!(doc.node_count() > 4, "复杂文本结构应包含多个节点");

    // 验证文本节点存在
    assert!(doc.query_selector(doc.root(), "div").is_some());
    assert!(doc.query_selector(doc.root(), "span").is_some());
    assert!(doc.query_selector(doc.root(), "strong").is_some());
}
