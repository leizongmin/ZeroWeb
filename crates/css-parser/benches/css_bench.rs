//! CSS 解析器性能基准测试。

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use zero_css_parser::{Parser, Tokenizer, selector};

/// 生成一个包含多个选择器和声明的基础 CSS 文档。
fn generate_base_css(rules: usize) -> String {
    let mut css = String::new();
    for i in 0..rules {
        css.push_str(&format!(
            ".class-{} {{ color: rgb({}, {}, {}); font-size: {}px; margin: {}px {}px {}px {}px; }}\n",
            i,
            i % 256,
            (i * 2) % 256,
            (i * 3) % 256,
            12.0 + (i % 20) as f64,
            i % 10,
            (i + 1) % 10,
            (i + 2) % 10,
            (i + 3) % 10,
        ));
    }
    css
}

/// 生成 ~100KB 的 CSS 文档。
fn generate_100kb_css() -> String {
    let mut css = String::new();
    // 基础样式规则
    css.push_str("/* 全局样式 */\n");
    css.push_str("*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }\n");
    css.push_str("html { font-size: 16px; line-height: 1.5; }\n");
    css.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; color: #333; background-color: #fff; }\n");
    css.push_str("a { color: #0066cc; text-decoration: none; }\n");
    css.push_str("a:hover { text-decoration: underline; }\n");
    css.push_str("img { max-width: 100%; height: auto; }\n");

    // 布局类
    css.push_str(".container { max-width: 1200px; margin: 0 auto; padding: 0 16px; }\n");
    css.push_str(".row { display: flex; flex-wrap: wrap; gap: 16px; }\n");
    css.push_str(".col-1 { flex: 0 0 8.333%; }\n");
    css.push_str(".col-2 { flex: 0 0 16.666%; }\n");
    css.push_str(".col-3 { flex: 0 0 25%; }\n");
    css.push_str(".col-4 { flex: 0 0 33.333%; }\n");
    css.push_str(".col-6 { flex: 0 0 50%; }\n");
    css.push_str(".col-12 { flex: 0 0 100%; }\n");

    // 填充到 ~100KB
    let target_size = 100 * 1024;
    let mut i = 0;
    while css.len() < target_size {
        css.push_str(&format!(
            ".component-{} {{ display: flex; align-items: center; justify-content: space-between; padding: {}px; background-color: #{:06x}; border-radius: {}px; }}\n",
            i,
            8 + (i % 16),
            (i * 12345) % 0xFFFFFF,
            4 + (i % 12),
        ));
        i += 1;
    }
    css
}

/// 生成复杂选择器 CSS。
fn generate_complex_selector_css(selectors: usize) -> String {
    let mut css = String::new();
    for i in 0..selectors {
        let selector = match i % 8 {
            0 => format!("div.container > .item-{} .text", i),
            1 => format!("#main .list-{} > li:nth-child(2n+1)", i),
            2 => format!("nav ul.menu-{} li:hover > a.active", i),
            3 => format!("article.post-{} header h2 + p", i),
            4 => format!(".sidebar .widget-{}:not(.hidden) .title", i),
            5 => format!("table.data-{} tbody tr:nth-child(odd) td:first-child", i),
            6 => format!("form.search-{} input[type=text]:focus", i),
            7 => format!(".modal-{} .content > .header ~ .body", i),
            _ => unreachable!(),
        };
        css.push_str(&format!(
            "{} {{ color: #{:06x}; font-size: {}px; display: block; }}\n",
            selector,
            (i * 7919) % 0xFFFFFF,
            12 + (i % 18),
        ));
    }
    css
}

// ── 基准 1: CSS 解析吞吐量 ──────────────────────────────────────

fn bench_css_parse_throughput(c: &mut Criterion) {
    let css_100kb = generate_100kb_css();

    c.bench_function("css_parse_100kb", |b| {
        b.iter(|| {
            let stylesheet = Parser::parse_stylesheet(black_box(&css_100kb));
            black_box(&stylesheet);
        });
    });
}

fn bench_css_parse_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("css_parse_by_size");
    for size in [100, 500, 1000, 5000] {
        let css = generate_base_css(size);
        group.bench_with_input(
            BenchmarkId::new("rules", size),
            &css,
            |b, css| {
                b.iter(|| {
                    let stylesheet = Parser::parse_stylesheet(black_box(css));
                    black_box(&stylesheet);
                });
            },
        );
    }
    group.finish();
}

// ── 基准 2: Tokenizer 吞吐量 ─────────────────────────────────────

fn bench_tokenizer_throughput(c: &mut Criterion) {
    let css = generate_base_css(1000);

    c.bench_function("tokenizer_1000_rules", |b| {
        b.iter(|| {
            let tokens: Vec<_> = Tokenizer::new(black_box(&css)).collect();
            black_box(&tokens);
        });
    });
}

// ── 基准 3: 复杂选择器解析 ──────────────────────────────────────

fn bench_complex_selector_parse(c: &mut Criterion) {
    let css = generate_complex_selector_css(100);

    c.bench_function("complex_selector_parse_100", |b| {
        b.iter(|| {
            let stylesheet = Parser::parse_stylesheet(black_box(&css));
            black_box(&stylesheet);
        });
    });
}

// ── 基准 4: Specificity 计算 ────────────────────────────────────

fn bench_specificity(c: &mut Criterion) {
    let css = "div#main.container > nav ul li.active a:hover";
    let stylesheet = Parser::parse_stylesheet(css);

    if let Some(rule) = stylesheet.rules.first() {
        if let zero_css_parser::ast::Rule::Style(style_rule) = rule {
            if let Some(sel) = style_rule.selectors.first() {
                c.bench_function("specificity_complex_selector", |b| {
                    b.iter(|| {
                        let spec = selector::specificity(black_box(sel));
                        black_box(spec);
                    });
                });
            }
        }
    }
}

criterion_group!(
    benches,
    bench_css_parse_throughput,
    bench_css_parse_by_size,
    bench_tokenizer_throughput,
    bench_complex_selector_parse,
    bench_specificity,
);
criterion_main!(benches);
