//! 样式系统性能基准测试。

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use zero_css_parser::Parser as CssParser;
use zero_dom::parse_html;
use zero_style_system::{StyleSystem, cascade, matcher, property};

/// 生成一个包含 N 个同级元素的 HTML 文档。
fn generate_html(elements: usize) -> String {
    let mut html = String::from("<html><head></head><body>");
    for i in 0..elements {
        html.push_str(&format!(
            "<div class=\"item item-{}\" id=\"el-{}\"><span class=\"text\">hello</span></div>",
            i % 10,
            i,
        ));
    }
    html.push_str("</body></html>");
    html
}

/// 生成一个包含 N 条规则的 CSS 样式表。
fn generate_css_rules(rules: usize) -> String {
    let mut css = String::new();
    css.push_str("div { display: block; color: #333; font-size: 16px; }\n");
    css.push_str("span { display: inline; color: inherit; }\n");

    for i in 0..rules {
        css.push_str(&format!(
            ".item-{} {{ padding: {}px; margin: {}px; background-color: #{:06x}; }}\n",
            i % 10,
            4 + (i % 8),
            2 + (i % 4),
            (i * 54321) % 0xFFFFFF,
        ));
    }
    css
}

/// 从文档中收集所有元素节点 ID。
fn collect_elements(doc: &zero_dom::Document) -> Vec<zero_dom::NodeId> {
    let root = doc.root();
    let mut result = Vec::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        if let Some(node) = doc.get(id) {
            if matches!(node.kind, zero_dom::NodeKind::Element(_)) {
                result.push(id);
            }
            // 子节点逆序入栈以保持顺序
            let children = doc.child_nodes(id);
            for child in children.into_iter().rev() {
                stack.push(child);
            }
        }
    }
    result
}

// ── 基准 1: 级联算法 ──────────────────────────────────────────────

fn bench_cascade(c: &mut Criterion) {
    let mut group = c.benchmark_group("cascade");

    for count in [10, 50, 100, 500] {
        group.bench_with_input(BenchmarkId::new("declarations", count), &count, |b, &count| {
            // 值字符串先收集为 owned，再借用构造 CascadedDeclaration（字段借用声明）。
            let owned: Vec<_> = (0..count)
                .map(|i| {
                    (
                        format!("#{:06x}", i * 11111 % 0xFFFFFF),
                        cascade::CascadeOrder {
                            origin: cascade::Origin::Author,
                            layer_index: None,
                            specificity: (i as u32 % 2, i as u32 % 3, i as u32),
                            position: i,
                            important: i % 10 == 0,
                        },
                    )
                })
                .collect();
            let decls: Vec<_> = owned
                .iter()
                .enumerate()
                .map(|(i, (value, order))| cascade::CascadedDeclaration {
                    property: if i % 3 == 0 {
                        "color"
                    } else if i % 3 == 1 {
                        "font-size"
                    } else {
                        "margin"
                    },
                    value,
                    order: order.clone(),
                })
                .collect();

            b.iter(|| {
                let result = cascade::cascade(black_box(decls.clone()), false);
                black_box(&result);
            });
        });
    }
    group.finish();
}

// ── 基准 2: 样式计算（小页面）──────────────────────────────────────

fn bench_compute_styles_small(c: &mut Criterion) {
    let html = generate_html(50);
    let doc = parse_html(&html);

    let css = generate_css_rules(20);
    let stylesheets = vec![CssParser::parse_stylesheet(&css)];

    c.bench_function("compute_styles_50_elements", |b| {
        b.iter(|| {
            let mut system = StyleSystem::new();
            let styles = system.compute_styles(black_box(&doc), black_box(&stylesheets));
            black_box(&styles);
        });
    });
}

// ── 基准 3: 样式计算（中等页面）─────────────────────────────────────

fn bench_compute_styles_medium(c: &mut Criterion) {
    let html = generate_html(200);
    let doc = parse_html(&html);

    let css = generate_css_rules(50);
    let stylesheets = vec![CssParser::parse_stylesheet(&css)];

    c.bench_function("compute_styles_200_elements", |b| {
        b.iter(|| {
            let mut system = StyleSystem::new();
            let styles = system.compute_styles(black_box(&doc), black_box(&stylesheets));
            black_box(&styles);
        });
    });
}

// ── 基准 4: 选择器匹配 ──────────────────────────────────────────────

fn bench_selector_matching(c: &mut Criterion) {
    let html = generate_html(100);
    let doc = parse_html(&html);

    let css = r#"
        div { color: #333; }
        .item { padding: 8px; }
        .item-0 { background: #ff0000; }
        #el-0 { border: 1px solid #ccc; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let elements = collect_elements(&doc);

    c.bench_function("selector_matching_100_elements", |b| {
        b.iter(|| {
            for &el in &elements {
                for rule in &stylesheet.rules {
                    if let zero_css_parser::ast::Rule::Style(style_rule) = rule {
                        for sel in &style_rule.selectors {
                            let _ = matcher::matches_selector(black_box(&doc), black_box(el), black_box(sel));
                        }
                    }
                }
            }
        });
    });
}

// ── 基准 5: ComputedStyle 默认值生成 ──────────────────────────────

fn bench_computed_style_default(c: &mut Criterion) {
    c.bench_function("computed_style_default", |b| {
        b.iter(|| {
            let style = property::ComputedStyle::default();
            black_box(&style);
        });
    });
}

criterion_group!(
    benches,
    bench_cascade,
    bench_compute_styles_small,
    bench_compute_styles_medium,
    bench_selector_matching,
    bench_computed_style_default,
);
criterion_main!(benches);
