//! browser-shell 性能基准测试。
//!
//! 测量浏览器 Shell 数据模型关键操作吞吐量：
//! - 标签页创建/切换/关闭
//! - 书签 CRUD
//! - 历史记录搜索
//! - 自动补全建议
//! - 下载管理器操作

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use zero_browser_shell::{Autocomplete, Bookmarks, BrowserShell, DownloadManager, History};

/// 基准 1: 标签页创建吞吐量。
fn bench_tab_creation(c: &mut Criterion) {
    c.bench_function("tab_creation_100", |b| {
        b.iter(|| {
            let mut shell = BrowserShell::new();
            for _ in 0..100 {
                shell.new_tab(Some("https://example.com"));
            }
        })
    });
}

/// 基准 2: 书签批量添加。
fn bench_bookmark_bulk_add(c: &mut Criterion) {
    c.bench_function("bookmark_add_1000", |b| {
        b.iter(|| {
            let mut bm = Bookmarks::new();
            for i in 0..1000 {
                bm.add(
                    black_box(&format!("Page {i}")),
                    black_box(&format!("https://example.com/{i}")),
                    None,
                );
            }
        })
    });
}

/// 基准 3: 历史记录搜索。
fn bench_history_search(c: &mut Criterion) {
    let mut history = History::new();
    for i in 0..1000 {
        history.record(&format!("https://example.com/page/{i}"), &format!("Page {i}"));
    }
    c.bench_function("history_search_1k", |b| b.iter(|| history.search(black_box("example"))));
}

/// 基准 4: 自动补全建议。
fn bench_autocomplete_suggest(c: &mut Criterion) {
    let mut history = History::new();
    let mut bookmarks = Bookmarks::new();
    for i in 0..500 {
        history.record(&format!("https://example.com/page/{i}"), &format!("Example Page {i}"));
    }
    for i in 0..100 {
        bookmarks.add(&format!("Bookmark {i}"), &format!("https://example.com/bm/{i}"), None);
    }
    let ac = Autocomplete::new();
    c.bench_function("autocomplete_suggest_500history", |b| {
        b.iter(|| ac.suggest(black_box("example"), &history, &bookmarks))
    });
}

/// 基准 5: 下载管理器批量操作。
fn bench_download_manager(c: &mut Criterion) {
    c.bench_function("download_manager_100", |b| {
        b.iter(|| {
            let mut dm = DownloadManager::new();
            for i in 0..100 {
                let id = dm.start_download(
                    black_box(&format!("https://example.com/file/{i}")),
                    black_box(&format!("file{i}.bin")),
                );
                dm.update_progress(id, 500, Some(1000));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_tab_creation,
    bench_bookmark_bulk_add,
    bench_history_search,
    bench_autocomplete_suggest,
    bench_download_manager,
);
criterion_main!(benches);
