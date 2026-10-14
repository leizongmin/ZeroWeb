# Cache API、OPFS 与跨进程存储桥

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

读取网页 caches 的请求/响应恢复、缓存对象删除后仍存活、OPFS 写流与路径、owner 共享问题。CacheStorage 使用共享 StorageManager；OPFS 独立维护 per-origin 文件树。Service Worker 的 registration-local 缓存另见 SW 主题。

## Gotchas
- CacheResponse 不只保存 status/body，还保存 URL、statusText、response_type 和 headers；丢掉 opaque/opaqueredirect 或 final URL 会改变页面响应身份。 (`crates/storage/src/cache_api.rs`, `crates/page-runtime/src/cache_storage_host.rs`)
- 页面 CacheStorage 拒绝字符串 null 的 opaque origin；请求正文由 serde 解析，异常名称与字节 body wire 均校验，不能把 malformed body 当普通文本。 (`crates/page-runtime/src/cache_storage_host.rs`)
- 删除命名 cache 会移除名称映射但保留旧 cache_id 对应的 doomed cache，重开同名得到新实例；仅按 cache_name 操作会让旧 JS 对象错误指向新 cache。 (`crates/page-runtime/src/cache_storage_host.rs`)
- cache 名称携带 UTF-16 code units 编码，允许保留 Rust UTF-8 字符串不能直接表达的 DOMString 身份；解码校验四位十六进制分组，不能只用显示名称作键。 (`crates/page-runtime/src/cache_storage_host.rs`)
- OPFS handler 初始化持久层失败会静默降级内存，但后续 dirty flush 的 I/O 失败会返回错误；初始化成功或 API 可用都不足以证明数据已落盘。 (`crates/page-runtime/src/opfs_host.rs`)
- OPFS 持久交换格式只保存文件路径、内容与 lastModified，空目录不在扁平文件表中；无文件的树被视为空并删除持久文件，不能宣称空目录跨 owner 重建保留。 (`crates/storage/src/opfs.rs`, `crates/storage/src/opfs/persistence.rs`)

## Architecture
- CacheQueryOptions 的 ignoreSearch、ignoreMethod、ignoreVary 是独立语义并穿过宿主 wire；修查询选项要核对页面 host 与 SW manager 两条消费路径。 (`crates/storage/src/cache_api.rs`, `crates/page-runtime/src/cache_storage_host.rs`, `crates/page-runtime/src/service_worker_manager.rs`)
- CacheStorage 每 origin 一份哈希命名 .cache 快照，恢复校验 origin 与文件路径一致；写入使用临时文件、同步、替换和备份恢复，重复 origin 或损坏格式不是空缓存成功恢复。 (`crates/storage/src/cache_api/persistence.rs`)
- OPFS 句柄以路径段数组表达，同一路径代表同条目；写流持有独立 buffer，close 提交、abort 终止，删除有活动写者的目标或子树有单独错误语义。 (`crates/storage/src/opfs.rs`)
- OPFS 错误 wire 保存 Name|code|message，TypeError 与 DOMException 分开恢复；桥接层不能丢掉 legacy code 或把全部错误转成 UnknownError。 (`crates/page-runtime/src/opfs_host.rs`, `crates/storage/src/opfs.rs`)
- IndexedDbOwner 的 CacheStorage handler clone 同一 StorageManager，而 OPFS handler 从独立 root 构造；clone owner 不自动证明多个 OPFS handler 共享同一 live 文件树。 (`crates/webview/src/indexed_db_owner.rs`)
