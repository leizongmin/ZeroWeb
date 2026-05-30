//! 安全 crate 性能基准测试。

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zero_security::{
    check_cors, is_simple_request, CorsPolicy, ContentSecurityPolicy, Origin,
};

/// 基准：Origin 解析
fn bench_origin_parse(c: &mut Criterion) {
    c.bench_function("origin_parse_1000", |b| {
        b.iter(|| {
            for i in 0..1000u32 {
                let url = format!("https://example{}.com/path?q={}", i % 100, i);
                black_box(Origin::parse(&url));
            }
        })
    });
}

/// 基准：同源判断
fn bench_same_origin_check(c: &mut Criterion) {
    let origin_a = Origin::parse("https://example.com").unwrap();
    let origin_b = Origin::parse("https://example.com/other").unwrap();
    let origin_c = Origin::parse("https://evil.com").unwrap();

    c.bench_function("same_origin_check_1000", |bencher| {
        bencher.iter(|| {
            for _ in 0..1000 {
                black_box(origin_a.is_same_origin(&origin_b));
                black_box(origin_a.is_same_origin(&origin_c));
            }
        })
    });
}

/// 基准：CORS 策略检查
fn bench_cors_check(c: &mut Criterion) {
    let policy = CorsPolicy::default();
    let origin = Origin::parse("http://example.com").unwrap();
    let headers: Vec<(String, String)> = vec![
        ("Accept".into(), "text/html".into()),
        ("Content-Type".into(), "text/plain".into()),
    ];

    c.bench_function("cors_check_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(check_cors(&policy, &origin, "GET", &headers));
            }
        })
    });
}

/// 基准：CSP 策略解析
fn bench_csp_parse(c: &mut Criterion) {
    let csp_str = "default-src 'self'; script-src 'self' https://cdn.example.com; style-src 'unsafe-inline'; img-src *; connect-src 'self' https://api.example.com";

    c.bench_function("csp_parse_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(ContentSecurityPolicy::parse(csp_str));
            }
        })
    });
}

/// 基准：CSP 资源检查
fn bench_csp_resource_check(c: &mut Criterion) {
    let csp = ContentSecurityPolicy::parse(
        "default-src 'self'; script-src 'self' https://cdn.example.com",
    );

    c.bench_function("csp_resource_check_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(csp.is_resource_allowed("script", "app.js"));
                black_box(csp.is_resource_allowed("script", "https://cdn.example.com/app.js"));
                black_box(csp.is_resource_allowed("script", "https://evil.com/bad.js"));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_origin_parse,
    bench_same_origin_check,
    bench_cors_check,
    bench_csp_parse,
    bench_csp_resource_check,
);
criterion_main!(benches);
