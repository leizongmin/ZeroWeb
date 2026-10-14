# 运行时配置的入口、默认值与生效时机

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次未修改环境或运行产品。 -->

## Module Structure

集中说明产品环境变量的解析、默认值和消费者生效边界。兼容 kill-switch 与诊断项仍有局部读取，不是所有环境变量都集中或动态热更新。具体网络/存储语义归 web-data，GPU 算法归 graphics。

### Decision Entry

`crates/runtime-config/src/lib.rs` 的 `ENVIRONMENT_VARIABLES` 和解析 helper 是产品清单；实际效果由 RenderMode、HTTP client、scheduler、存储 owner 和 compositor 初始化决定。`docs/runtime-environment.md` 是对外索引，代码中的真实 helper 及调用时机优先。

## Branching Table

| 维度 | 分支一 | 分支二及生效边界 | 来源 |
| --- | --- | --- | --- |
| 渲染模式 | 未设置返回 None，默认 Auto；auto 优先可用 GPU | cpu/software/soft 解析为 Cpu；非法或非 UTF-8 返回错误 | `crates/render-foundation/src/config.rs`, `crates/runtime-config/src/lib.rs` |
| HTTP/2 | ZERO_HTTP2 未设启用 | 0/false 仅 HTTP/1.1；新建/缓存 client 按值区分 | `crates/net/src/connect.rs`, `crates/net/src/client.rs` |
| 代理 | ZERO_NOPROXY 未设尊重代理 | 1/true 调 builder.no_proxy，其他字符串不启用 | `crates/net/src/connect.rs` |
| 并发预算 | per-origin=6、total=24 | 非正整数回默认；scheduler 构造时固化，队列上限再由 total 推导 | `crates/net/src/resource_policy.rs`, `crates/net/src/fetch_scheduler.rs` |
| 隐私 owner | 普通 browser 构造持久 IDB/Cache/SW owner | ZERO_PRIVATE=1/true 或 cfg(test) 构造内存 owner，不改变已有实例 | `apps/browser/src/process_backend.rs` |
| 存储路径 | 未设取平台 data dir，失败落相对目录 | ZERO_STORAGE_DIR 直接作 IDB 目录，CacheStorage/OPFS/SW 加子路径 | `crates/storage/src/storage_manager.rs` |
| 帧交付 | Linux shm/GPU_IMAGE/ZERO_COPY 等默认启用 | 非 Linux 包装器返回 false，不能用 env=1 激活 Linux 实现 | `crates/protocol/src/frame_shm.rs` |
| CPU 工作线程 | 产品默认开，仅精确 0 关闭 | 测试 helper 只认 1/true，未设默认直接光栅 | `crates/render-foundation/src/rendering_thread.rs` |
| compositor 沙箱 | env 剥离、seccomp、Landlock 各自默认开 | 关闭 env 总名不关闭其他两项；OS 过滤仅 Linux | `apps/compositor/src/sandbox.rs` |

## Affected Scope

| 模块 | 配置影响 |
| --- | --- |
| `apps/browser/src/process_backend.rs` | 子进程、隐私存储/SW owner 创建 |
| `apps/compositor/src/lib.rs` | GPU 预热、工作线程与帧交付 |
| `crates/protocol/src/frame_shm.rs` | 平台默认和 mailbox/dma-buf 条件 |
| `crates/net/src/client.rs` | client 配置键与缓存实例 |
| `crates/net/src/fetch_scheduler.rs` | 并发、排队上限初始化 |
| `crates/storage/src/storage_manager.rs` | 持久化路径布局 |
| `crates/render-foundation/src/rendering_thread.rs` | 产品/测试线程默认值 |
| `apps/compositor/src/sandbox.rs` | 过滤器安装与时间顺序 |

## Gotchas
- enabled_when_true 只认 1/true，enabled_by_default 只在 0/false 关闭，enabled_unless_zero 只认精确 0；“false”“空字符串”“yes”在三类 helper 中效果不同，不能统一 truthy 解析。 (`crates/runtime-config/src/lib.rs`)
- 运行变量名清单记录的是默认行为，不证明消费者已安装 OS 能力；renderer seccomp 在当前源码仍是剥离环境与占位日志。 (`crates/runtime-config/src/lib.rs`, `apps/renderer/src/sandbox.rs`)

## Patterns
- HTTP client cache 键包含 timeout、no_proxy、http2；切换后获取的是相应键的 client，不是自动修改全部已持有 client。 (`crates/net/src/client.rs`)
- ZERO_STORAGE_DIR 的 IDB 直接目录是兼容行为，其余子路径并不全同于平台默认布局；迁移路径必须核对每个 owner，不能统一追加 IndexedDB 子目录。 (`crates/storage/src/storage_manager.rs`, `docs/runtime-environment.md`)

## Branching Behavior
- 网络并发设置在 PerOriginFetchScheduler::new 读取并复制到字段，之后改 env 不会改已有调度器；传输和排队预算不是同一个值。 (`crates/net/src/fetch_scheduler.rs`, `crates/net/src/resource_policy.rs`)
- compositor GPU 开关在 run_role 启动决定预热与 GPU-aware sandbox；环境改变不能当作已运行进程的状态切换机制。 (`apps/compositor/src/lib.rs`, `apps/compositor/src/sandbox.rs`)
- optional_path 保留非 UTF-8 路径且把空值视为未配置，optional_string 要求 UTF-8；不能在集中配置中把两类 helper 混用。 (`crates/runtime-config/src/lib.rs`)
- ZERO_CACHE_DIR 影响 DiskHttpCache::open_default 选用的根目录，已打开 cache 的 root 字段不会随 env 自动移动。 (`crates/net/src/disk_cache.rs`)
- ZW_IPC_TRACE 是局部诊断开关，首次 trace 经 OnceLock 缓存目标文件；运行中改 env 不会重新打开追踪文件，也不应把它注册表缺失解释成不可用。 (`crates/protocol/src/transport.rs`, `crates/runtime-config/src/lib.rs`)
