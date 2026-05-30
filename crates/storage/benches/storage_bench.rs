//! 存储 crate 性能基准测试。

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use zero_storage::{IdbDatabase, IdbKey, StorageManager};

/// 基准：localStorage 批量写入
fn bench_local_storage_write(c: &mut Criterion) {
    c.bench_function("local_storage_write_1000", |b| {
        b.iter(|| {
            let mut mgr = StorageManager::new();
            let store = mgr.local_storage("https://example.com");
            for i in 0..1000u32 {
                let _ = black_box(store.set(&format!("key_{i}"), &format!("value_{i}")));
            }
        })
    });
}

/// 基准：localStorage 批量读取
fn bench_local_storage_read(c: &mut Criterion) {
    c.bench_function("local_storage_read_1000", |b| {
        let mut mgr = StorageManager::new();
        let store = mgr.local_storage("https://example.com");
        for i in 0..1000u32 {
            store
                .set(&format!("key_{i}"), &format!("value_{i}"))
                .unwrap();
        }
        b.iter(|| {
            for i in 0..1000u32 {
                black_box(store.get(&format!("key_{i}")));
            }
        })
    });
}

/// 基准：IndexedDB 批量写入
fn bench_indexeddb_write(c: &mut Criterion) {
    c.bench_function("indexeddb_write_100", |b| {
        b.iter(|| {
            let mut db = IdbDatabase::new("benchdb", 1);
            db.create_object_store("items", Some("id"), false).unwrap();
            for i in 0..100u32 {
                let key = IdbKey::Number(i as f64);
                let val = serde_json::json!({"name": format!("item_{i}"), "value": i});
                let _ = black_box(db.add("items", val, Some(key)));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_local_storage_write,
    bench_local_storage_read,
    bench_indexeddb_write,
);
criterion_main!(benches);
