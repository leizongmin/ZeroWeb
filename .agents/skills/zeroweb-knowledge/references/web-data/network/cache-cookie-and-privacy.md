# HTTP 缓存、连接策略、Cookie 与隐私

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

读取缓存误命中、再验证、代理失败、Cookie 隔离与无痕落盘问题。HTTP 缓存不是网页 `caches` API；后者见存储索引。Cookie 网络底座与 JS document.cookie 语义也不是同一实现。

## Gotchas
- HttpCache::new 仅建内存缓存；open_persistent 才尝试打开磁盘层，ZERO_PRIVATE 启用时直接返回内存缓存，磁盘打不开也会降级而非终止导航。 (`crates/net/src/http_cache.rs`, `crates/net/src/private_mode.rs`)
- 非默认分区使用独立内存缓存，当前磁盘索引未携带 partition key，所以不能为提高命中率把这些分区接到共享磁盘层。 (`crates/net/src/resource_loader.rs`)
- 缓存去掉 URL fragment，但保留响应 Vary 的字段维度；请求头名查找不区分大小写，Vary 字段名在键里保留原样，不能把它误述为完全规范化键。 (`crates/net/src/cache_key.rs`)
- 磁盘读取校验元数据中的键与 body_len；键不符返回 miss，长度不符清除文件，不能只依据哈希文件名认定条目有效。 (`crates/net/src/disk_cache.rs`)
- 缓存年龄包含响应初始 Age/Date 与驻留时间；磁盘提升到内存时 TTL 已是剩余新鲜期，不能再次重复扣除同一初始年龄。 (`crates/net/src/http_cache.rs`, `crates/net/src/disk_cache.rs`)

## Architecture
- 资源入口只缓存无 body GET；其他方法走 HttpClient write-through，并在成功后失效相关条目，网页 Cache API 不受此 HTTP 策略自动管理。 (`crates/net/src/resource_loader.rs`)
- 普通和无痕标签由 TabFetchProxy 选择不同 ResourceLoader，无痕加载器使用独立 HttpCache::new 与 private 分区；这证明缓存隔离，不证明所有 Cookie 或存储路径都自动共享同一无痕边界。 (`apps/browser/src/fetch_proxy.rs`)
- CookieStore 区分 host-only 与 Domain Cookie，默认 SameSite 为 Lax；同名、同域、同路径替换保留创建时间，发送顺序需同时考虑路径长度和创建时间。 (`crates/net/src/cookie.rs`)
- 失败 URL 的共享负缓存默认冷却 30 秒、最多 1000 项；查询会清自身过期项，超限先清过期再淘汰最早失败记录，成功必须 mark_ok 清除失败记忆。 (`crates/net/src/negative_cache.rs`)

## Patterns
- blocking 客户端采用仅 IPv4 的 DNS resolver，默认启用 HTTP/2 并尊重代理；ZERO_HTTP2=0 与 ZERO_NOPROXY=1 分别改变协议与代理，不能混为同一连接回退。 (`crates/net/src/connect.rs`)
- 代理错误诊断只保留脱敏代理地址，普通网络错误只注明已设置的代理变量名；新增日志不能直接输出带 userinfo 的原始代理值。 (`crates/net/src/connect.rs`)
