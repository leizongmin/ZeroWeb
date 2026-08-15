# ZeroWeb Storage (`zero-storage`)

> 浏览器端存储后端 — 提供 localStorage、sessionStorage 和 IndexedDB 的 Rust 实现。

## 概述

`ZeroWeb Storage` (`zero-storage`) 实现了 Web 标准中的客户端存储机制，包括 Web Storage（localStorage/sessionStorage）和 IndexedDB。作为 ZeroWeb 渲染管线的存储层，它为上层引擎提供按源（origin）隔离的键值存储和结构化数据库能力，同时支持配额管理和多种键类型的索引查询。

## 主要功能

- **Web Storage** — localStorage（持久）和 sessionStorage（会话），支持配额限制、按键索引、容量估算
- **IndexedDB** — 结构化数据库，支持 Object Store 的创建/删除、记录的增删改查、自增主键、复合键排序
- **存储管理器** — 按源（origin）隔离管理多个 localStorage/sessionStorage 实例，支持按源清除和批量清除
- **Cache API** — 缓存 Request/Response 对，支持按方法（GET、POST 等）与 URL 匹配、缓存命中查询和删除
- **Service Worker 注册表** — 管理 Service Worker 的注册与生命周期状态机（Registered → Installing → Installed → Activating → Activated → Redundant），支持 Fetch 拦截
- **错误处理** — 统一的 `StorageError` 枚举，涵盖配额超限、无效键、仓库不存在、序列化失败等场景

## 使用示例

```rust
use zero_storage::{StorageManager, StorageType, IdbDatabase, IdbKey};

// Web Storage — 通过 StorageManager 按源管理
let mut manager = StorageManager::new();
let storage = manager.local_storage("https://example.com");
storage.set("theme", "dark").unwrap();
assert_eq!(storage.get("theme"), Some("dark"));

// IndexedDB — 创建数据库和 Object Store，进行增删改查
let mut db = IdbDatabase::new("mydb", 1);
db.create_object_store("users", Some("id"), true).unwrap();

let key = db.add("users", serde_json::json!({"name": "Alice", "age": 30}), None).unwrap();
let record = db.get("users", &key).unwrap();
assert_eq!(record.value["name"], "Alice");
```
