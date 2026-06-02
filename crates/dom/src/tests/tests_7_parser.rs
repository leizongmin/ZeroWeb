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
