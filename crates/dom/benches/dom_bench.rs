//! DOM crate 性能基准测试。

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zero_dom::{Document, NodeId, parse_html};

/// 生成一个包含指定数量嵌套 div 的 HTML 文档。
fn generate_nested_html(depth: usize) -> String {
    let mut html = String::from("<html><body>");
    for _ in 0..depth {
        html.push_str("<div>");
    }
    html.push_str("Hello World");
    for _ in 0..depth {
        html.push_str("</div>");
    }
    html.push_str("</body></html>");
    html
}

/// 生成一个包含指定数量同级元素的 HTML 文档。
fn generate_wide_html(width: usize) -> String {
    let mut html = String::from("<html><body>");
    for i in 0..width {
        html.push_str(&format!("<div class=\"item\" id=\"item-{}\">text {}</div>", i, i));
    }
    html.push_str("</body></html>");
    html
}

// ── 基准 1: DOM 树构建（10k 节点）──────────────────────────────

fn bench_dom_tree_construction(c: &mut Criterion) {
    let html = generate_wide_html(10_000);

    c.bench_function("dom/tree_construction_10k_nodes", |b| {
        b.iter(|| {
            let doc = parse_html(black_box(&html));
            assert!(doc.node_count() > 10_000);
        });
    });
}

// ── 基准 2: querySelector（1000 元素）─────────────────────────

fn bench_query_selector(c: &mut Criterion) {
    let doc = parse_html(&generate_wide_html(1000));
    let root = doc.root();

    c.bench_function("dom/query_selector_1000_elements_by_tag", |b| {
        b.iter(|| {
            let result = doc.query_selector(black_box(root), black_box("div"));
            assert!(result.is_some());
        });
    });

    c.bench_function("dom/query_selector_1000_elements_by_id", |b| {
        b.iter(|| {
            let result = doc.query_selector(black_box(root), black_box("#item-500"));
            assert!(result.is_some());
        });
    });

    c.bench_function("dom/query_selector_1000_elements_by_class", |b| {
        b.iter(|| {
            let result = doc.query_selector(black_box(root), black_box(".item"));
            assert!(result.is_some());
        });
    });
}

// ── 基准 3: 批量 appendChild（1000 次）────────────────────────

fn bench_batch_append_child(c: &mut Criterion) {
    c.bench_function("dom/batch_append_child_1000", |b| {
        b.iter(|| {
            let mut doc = Document::new();
            let root = doc.root();
            let parent = doc.create_element("div");
            doc.append_child(root, parent).unwrap();

            for i in 0..1000 {
                let child = doc.create_element("span");
                doc.set_attribute(child, "data-index", &i.to_string());
                doc.append_child(parent, child).unwrap();
            }

            assert_eq!(doc.child_nodes(parent).len(), 1000);
        });
    });
}

// ── 基准 4: HTML 解析吞吐量 ─────────────────────────────────

fn bench_html_parsing_throughput(c: &mut Criterion) {
    let small_html = generate_nested_html(50);
    let large_html = generate_wide_html(5000);

    c.bench_function("dom/parse_html_nested_50", |b| {
        b.iter(|| {
            parse_html(black_box(&small_html));
        });
    });

    c.bench_function("dom/parse_html_wide_5000", |b| {
        b.iter(|| {
            parse_html(black_box(&large_html));
        });
    });
}

// ── 基准 5: getElementsByTagName（性能对比 getElementById）───

fn bench_get_elements_by_tag_name(c: &mut Criterion) {
    let doc = parse_html(&generate_wide_html(1000));

    c.bench_function("dom/get_elements_by_tag_name_1000", |b| {
        b.iter(|| {
            let divs = doc.get_elements_by_tag_name(black_box("div"));
            assert_eq!(divs.len(), 1000);
        });
    });

    c.bench_function("dom/get_element_by_id_indexed", |b| {
        b.iter(|| {
            let result = doc.get_element_by_id(black_box("item-500"));
            assert!(result.is_some());
        });
    });
}

criterion_group!(
    benches,
    bench_dom_tree_construction,
    bench_query_selector,
    bench_batch_append_child,
    bench_html_parsing_throughput,
    bench_get_elements_by_tag_name,
);
criterion_main!(benches);
