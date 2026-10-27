//! T13/P29 切片①常驻回归：声明组合级联缓存（`CascadeCache`）正确性。
//!
//! 判定策略：kill-switch 双路径差分——`set_cascade_cache_enabled(false)` 走原始
//! expand+cascade 路径，`true` 走缓存路径；同一文档逐元素 [`ComputedStyle`]
//! 全等比对（`PartialEq` 为 incremental-rendering-oracle 的输出契约：全部字段
//! 均为可观察 CSS 值）。覆盖键四源（表现提示/匹配收集/内联/UA 注入）+ quirks
//! 位 + 伪元素 + `@layer` + 媒体查询 + 热缓存（跨帧）+ 跨文档键分离。
//!
//! 级联顺序依据 CSS Cascading and Inheritance Level 5
//! <https://www.w3.org/TR/css-cascade-5/>（origin/layer/specificity/order）。

use std::collections::HashMap;

use zero_css_parser::Stylesheet;
use zero_dom::{Document, NodeId};
use zero_style_system::{ComputedStyle, StyleSystem};

fn parse_sheets(css: &str) -> Vec<Stylesheet> {
    vec![zero_css_parser::parser::Parser::parse_stylesheet(css)]
}

/// DFS 收集全部元素 NodeId（Document 顺序遍历用队列，保持文档序）。
fn collect_elements(doc: &Document) -> Vec<NodeId> {
    let mut out = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(doc.root());
    while let Some(id) = queue.pop_front() {
        if matches!(
            doc.get(id),
            Some(zero_dom::NodeData {
                kind: zero_dom::NodeKind::Element(_),
                ..
            })
        ) {
            out.push(id);
        }
        for c in doc.child_nodes(id) {
            queue.push_back(c);
        }
    }
    out
}

/// 收集含指定 class 的元素（文档序）——定位断言目标，跳过 html/body 包装节点。
fn collect_by_class(doc: &Document, class: &str) -> Vec<NodeId> {
    collect_elements(doc)
        .into_iter()
        .filter(|id| {
            matches!(
                doc.get(*id),
                Some(zero_dom::NodeData {
                    kind: zero_dom::NodeKind::Element(el),
                    ..
                }) if el.class_list.iter().any(|c| c == class)
            )
        })
        .collect()
}

/// 同一文档在缓存开/关两路径下分别计算，返回按文档序排列的元素样式向量。
fn compute_paths(html: &str, css: &str) -> (Vec<ComputedStyle>, Vec<ComputedStyle>) {
    let sheets = parse_sheets(css);
    let mut runs = Vec::new();
    for enabled in [true, false] {
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(1280.0, 720.0);
        sys.set_cascade_cache_enabled(enabled);
        let styles: HashMap<NodeId, ComputedStyle> = sys.compute_styles(&doc, &sheets);
        runs.push(collect_elements(&doc).iter().map(|id| styles[id].clone()).collect());
    }
    let disabled = runs.pop().expect("disabled run");
    let enabled = runs.pop().expect("enabled run");
    (enabled, disabled)
}

fn assert_paths_equal(scenario: &str, enabled: &[ComputedStyle], disabled: &[ComputedStyle]) {
    assert_eq!(
        enabled.len(),
        disabled.len(),
        "[{scenario}] 元素数不一致：enabled {} vs disabled {}",
        enabled.len(),
        disabled.len()
    );
    for (i, (e, d)) in enabled.iter().zip(disabled.iter()).enumerate() {
        assert_eq!(e, d, "[{scenario}] 第 {i} 个元素：缓存路径与原始路径样式不一致");
    }
}

/// 差分矩阵：四源 + 伪元素 + @layer + 媒体查询 + var() + important/specificity。
/// 缓存路径必须与原始路径逐元素全等。
#[test]
fn kill_switch_differential_matrix() {
    let scenarios: Vec<(&str, &str, &str)> = vec![
        // 组合复用（命中热路径）：同 class 多元素 + 同 inline 组合多元素。
        (
            "combo-reuse",
            "<div class=\"k\">1</div><div class=\"k\">2</div><div class=\"k\" style=\"margin:4px\">3</div>\
             <div class=\"k\" style=\"margin:4px\">4</div><div class=\"k\">5</div><p>tail</p>",
            ".k { color: navy; padding: 1px 2px; } p { margin: 0; }",
        ),
        // 表现提示源差异：img width/height 属性、table border 属性。
        (
            "presentational-hints",
            "<img src=\"x\" width=\"100\" height=\"50\"><img src=\"x\" width=\"200\" height=\"80\">\
             <table border=\"2\"><tr><td>a</td></tr></table>",
            "td { color: gray; }",
        ),
        // UA 注入源差异：hidden 属性、标题族、pre、链接、表单控件。
        (
            "ua-injection",
            "<div hidden>h</div><div>v</div><h1>t</h1><h6>u</h6><pre>  p  </pre>\
             <a href=\"l\">link</a><input><button>b</button><hr>",
            "a { text-decoration: underline; }",
        ),
        // 匹配源差异 + specificity/important 交叠。
        (
            "matching-specificity",
            "<div class=\"r\" id=\"x\">a</div><div class=\"b\" id=\"y\">b</div>\
             <div class=\"r b\">c</div><div class=\"r b i\">d</div>",
            ".r { color: red; } .b { color: blue; } div { color: black; }\
             .i { color: green !important; } #x { font-weight: bold; }",
        ),
        // @layer：layered vs unlayered 声明（级联层顺序，CSS Cascade 5 §8）。
        (
            "cascade-layers",
            "<div class=\"l\">a</div><div class=\"m\">b</div>",
            "@layer base { .l { color: green; } .m { font-size: 10px; } } .l { color: olive; }",
        ),
        // 媒体查询（viewport 1280：min-width 1000 命中）。
        (
            "media-query",
            "<div class=\"mq\">a</div><div>plain</div>",
            "@media screen and (min-width: 1000px) { .mq { color: teal; } } div { display: block; }",
        ),
        // 自定义属性与 var()（缓存之后阶段的交互：custom properties 继承链）。
        (
            "custom-properties",
            "<div class=\"th\" style=\"--accent: purple\"><span class=\"use\">a</span></div>\
             <div class=\"th\"><span class=\"use\">b</span></div>",
            ".th { --base: 2px; } .use { color: var(--accent, black); padding: var(--base); }",
        ),
        // 伪元素（::before/::after 经同一缓存：collect_pseudo_declarations 进 matching 源）。
        (
            "pseudo-elements",
            "<div class=\"p\">a</div><div class=\"p q\">b</div><div>plain</div>",
            ".p::before { content: \"<\"; color: red; } .p::after { content: \">\"; }\
             .q::before { content: \"!\"; font-weight: bold; }",
        ),
    ];
    for (name, html, css) in scenarios {
        let (enabled, disabled) = compute_paths(html, css);
        assert_paths_equal(name, &enabled, &disabled);
    }
}

/// 热缓存（跨帧）与跨样式表键变化：同一 StyleSystem 实例第二帧全命中，
/// 结果须与首帧及关缓存基线一致；样式表变化改变 matching 源（键随输入变化）。
#[test]
fn warm_cache_and_stylesheet_change() {
    let html = "<div class=\"k\">1</div><div class=\"k\" style=\"color:sienna\">2</div><p>p</p>";
    let css_a = ".k { color: navy; } p { margin: 0; }";
    let css_b = ".k { color: teal; padding: 3px; } p { margin: 1px; }";
    let doc = zero_dom::parse_html(html);
    let order = collect_elements(&doc);

    let baseline = {
        let sheets = parse_sheets(css_a);
        let mut sys = StyleSystem::new();
        sys.set_viewport(1280.0, 720.0);
        sys.set_cascade_cache_enabled(false);
        sys.compute_styles(&doc, &sheets)
    };
    let baseline_b = {
        let sheets = parse_sheets(css_b);
        let mut sys = StyleSystem::new();
        sys.set_viewport(1280.0, 720.0);
        sys.set_cascade_cache_enabled(false);
        sys.compute_styles(&doc, &sheets)
    };

    let mut sys = StyleSystem::new();
    sys.set_viewport(1280.0, 720.0);
    let cold = {
        let sheets = parse_sheets(css_a);
        sys.compute_styles(&doc, &sheets)
    };
    let warm = {
        let sheets = parse_sheets(css_a);
        sys.compute_styles(&doc, &sheets)
    };
    let changed = {
        let sheets = parse_sheets(css_b);
        sys.compute_styles(&doc, &sheets)
    };
    for id in &order {
        assert_eq!(cold[id], baseline[id], "冷缓存帧与关缓存基线不一致");
        assert_eq!(warm[id], baseline[id], "热缓存帧与关缓存基线不一致（跨帧键不漂移）");
        assert_eq!(
            changed[id], baseline_b[id],
            "样式表变化后须按新输入重算（键含 matching）"
        );
    }
}

/// quirks 位在键内：同一实例先后计算 quirks 与 standards 两文档（同标记同 CSS），
/// 各自须与独立关缓存实例一致——跨文档不串键。
/// （CSS1 quirks 下级联合法性判定不同：cascade.rs `is_cascade_value_valid` 传参。）
#[test]
fn quirks_bit_separates_documents() {
    let html_quirks = "<div class=\"k\">a</div><div class=\"k\" style=\"width:9px\">b</div>";
    let html_strict = format!("<!DOCTYPE html>{html_quirks}");
    let css = ".k { color: maroon; line-height: 1.5; }";

    let compute = |html: &str, enabled: bool| -> Vec<ComputedStyle> {
        let doc = zero_dom::parse_html(html);
        let sheets = parse_sheets(css);
        let mut sys = StyleSystem::new();
        sys.set_viewport(1280.0, 720.0);
        sys.set_cascade_cache_enabled(enabled);
        let styles = sys.compute_styles(&doc, &sheets);
        collect_elements(&doc).iter().map(|id| styles[id].clone()).collect()
    };
    let quirks_baseline = compute(html_quirks, false);
    let strict_baseline = compute(&html_strict, false);

    // 同一实例：先 quirks 后 standards（缓存跨文档保留）。
    let mut sys = StyleSystem::new();
    sys.set_viewport(1280.0, 720.0);
    let doc_q = zero_dom::parse_html(html_quirks);
    let sheets = parse_sheets(css);
    let quirks_mixed: Vec<ComputedStyle> = {
        let styles = sys.compute_styles(&doc_q, &sheets);
        collect_elements(&doc_q).iter().map(|id| styles[id].clone()).collect()
    };
    let doc_s = zero_dom::parse_html(&html_strict);
    let strict_mixed: Vec<ComputedStyle> = {
        let styles = sys.compute_styles(&doc_s, &sheets);
        collect_elements(&doc_s).iter().map(|id| styles[id].clone()).collect()
    };
    for (i, (m, b)) in quirks_mixed.iter().zip(quirks_baseline.iter()).enumerate() {
        assert_eq!(m, b, "quirks 文档（混算实例）第 {i} 元素与基线不一致");
    }
    for (i, (m, b)) in strict_mixed.iter().zip(strict_baseline.iter()).enumerate() {
        assert_eq!(m, b, "standards 文档（混算实例）第 {i} 元素与基线不一致——quirks 位串键");
    }
}

/// 键四源各自必须区分：仅单一源不同的元素对，样式必须不同（缓存不得把
/// 不同输入折叠为同键）。配对差分同时校验双路径一致。目标元素统一标记
/// class="s" 以跳过 html/body 包装节点。
#[test]
fn distinct_sources_are_distinguished() {
    let html = "<img class=\"s\" src=\"x\" width=\"100\"><img class=\"s\" src=\"x\" width=\"200\">\
                <div class=\"s\" style=\"color:red\">r</div><div class=\"s\" style=\"color:blue\">b</div>\
                <div class=\"s\" hidden>h</div><div class=\"s\">v</div>\
                <div class=\"s cr\">c1</div><div class=\"s cb\">c2</div>";
    let css = ".cr { color: crimson; } .cb { color: royalblue; }";
    let doc = zero_dom::parse_html(html);
    let sheets = parse_sheets(css);
    let run = |enabled: bool| -> Vec<ComputedStyle> {
        let mut sys = StyleSystem::new();
        sys.set_viewport(1280.0, 720.0);
        sys.set_cascade_cache_enabled(enabled);
        let styles = sys.compute_styles(&doc, &sheets);
        collect_by_class(&doc, "s")
            .iter()
            .map(|id| styles[id].clone())
            .collect()
    };
    let enabled = run(true);
    let disabled = run(false);
    assert_paths_equal("sources", &enabled, &disabled);
    // 对内必须不同（若键漏源，两元素共享错误命中 → 样式相等 → 此处红）。
    // 依次：表现提示（width 属性）、内联 style、UA 注入（hidden）、匹配（类选择器）。
    for (a, b) in [(0, 1), (2, 3), (4, 5), (6, 7)] {
        assert_ne!(enabled[a], enabled[b], "第 {a}/{b} 元素样式相等——单一源差异被缓存折叠");
    }
}

/// 同组合一致性：相同输入的多元素样式必须互相同（缓存不得损坏同键命中）。
#[test]
fn same_combo_elements_consistent() {
    let html = "<div class=\"k\">1</div><div class=\"k\">2</div><div class=\"k\">3</div>\
                <div class=\"k\">4</div><div class=\"k\">5</div>";
    let css = ".k { color: navy; padding: 1px 2px; font-weight: bold; }";
    let doc = zero_dom::parse_html(html);
    let sheets = parse_sheets(css);
    let run = |enabled: bool| -> Vec<ComputedStyle> {
        let mut sys = StyleSystem::new();
        sys.set_viewport(1280.0, 720.0);
        sys.set_cascade_cache_enabled(enabled);
        let styles = sys.compute_styles(&doc, &sheets);
        collect_by_class(&doc, "k")
            .iter()
            .map(|id| styles[id].clone())
            .collect()
    };
    let enabled = run(true);
    let disabled = run(false);
    assert_paths_equal("combo", &enabled, &disabled);
    for (i, s) in enabled.iter().enumerate().skip(1) {
        assert_eq!(s, &enabled[0], "同组合第 {i} 元素与首个元素样式不一致");
    }
}
