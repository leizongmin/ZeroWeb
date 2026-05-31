//! net crate 基准测试。

use criterion::{Criterion, criterion_group, criterion_main};
use zero_net::{CookieStore, NavigationHistory, parse_url};

fn bench_parse_url(c: &mut Criterion) {
    c.bench_function("parse_url", |b| {
        b.iter(|| parse_url("https://example.com:8080/path?query=value#fragment"))
    });
}

fn bench_navigation(c: &mut Criterion) {
    c.bench_function("navigation_navigate", |b| {
        let mut nav = NavigationHistory::new(100);
        let mut i = 0;
        b.iter(|| {
            nav.navigate(&format!("http://example.com/page{i}"), None);
            i += 1;
        })
    });
}

fn bench_cookie_parse(c: &mut Criterion) {
    c.bench_function("cookie_parse", |b| {
        b.iter(|| {
            CookieStore::parse_set_cookie("session=abc123; Path=/; Domain=example.com; Secure; HttpOnly; SameSite=Lax")
        })
    });
}

criterion_group!(benches, bench_parse_url, bench_navigation, bench_cookie_parse);
criterion_main!(benches);
