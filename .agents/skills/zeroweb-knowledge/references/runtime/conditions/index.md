# 跨域环境与构建分支

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查环境解析、Cargo feature 和平台/产品路径如何共同影响页面执行、网络存储与像素交付。
单一开关清单或一次 workspace 构建不能说明这些维度的组合行为。

### Decision Entry

产品环境名称与解析从 `crates/runtime-config/src/lib.rs` 查，依赖传播从 workspace 和
消费者 Cargo 文件查，实际启动及交付从 browser、Android 与 compositor 入口查。
实际验证参见 [功能地图](../../../../zeroweb-verify/features/index.md)，本次未运行配置矩阵。

## Branching Table

| 维度 | 默认/主路径 | 替代条件及差异 | 来源 |
|---|---|---|---|
| 环境布尔解析 | 默认开关仅0/false关闭 | opt-in只认1/true，兼容开关只认精确0关闭 | `crates/runtime-config/src/lib.rs` |
| 脚本选择 | renderer 默认V8 | QuickJS-only 与并集受 cfg/not(v8) 影响 | `apps/renderer/Cargo.toml`, `crates/script-sandbox/src/lib.rs` |
| browser 页面执行 | 发布使用 renderer peer | test/test-support 可编译进程内 worker | `apps/browser/src/main.rs`, `apps/browser/Cargo.toml` |
| 网络与持久化 | 普通 owner 构造时取配置 | private/test owner 使用内存；已有实例不自动重建 | `apps/browser/src/process_backend.rs` |
| GPU 交付 | Linux 有共享帧及 dma-buf 路径 | 非 Linux 门控关闭，Linux 也需当前帧满足导出资格 | `crates/protocol/src/frame_shm.rs`, `apps/compositor/src/lib.rs` |
| Android renderer | JNI 可运行共享角色 | renderer 还要求 android-renderer 链接，版本或线程启动不证明首帧 | `apps/android-browser/rust/src/lib.rs`, `apps/android-browser/rust/Cargo.toml` |

## Affected Scope

| 模块 | 影响 |
|---|---|
| `crates/runtime-config/src/lib.rs` | 解析与默认值 |
| `apps/browser/src/main.rs` | 产品、测试与窗口路径 |
| `apps/renderer/Cargo.toml` | 页面脚本引擎传播 |
| `crates/script-sandbox/src/lib.rs` | 并集及无引擎导出 |
| `apps/browser/src/process_backend.rs` | 网络/存储 owner 初始化 |
| `apps/compositor/src/lib.rs` | GPU 帧资格与回退 |
| `crates/protocol/src/frame_shm.rs` | 平台共享帧能力 |
| `apps/android-browser/rust/src/lib.rs` | JNI 链接、角色与 FD |

## Gotchas

- 环境默认、Cargo feature 和实际消费者是三个判断面；例如协议 GPU 环境变量默认开不证明当前 OS 已编译实现，renderer 引擎默认也不能由 browser 的 default feature 推断。 (`crates/runtime-config/src/lib.rs`, `crates/protocol/src/frame_shm.rs`, `apps/browser/Cargo.toml`, `apps/renderer/Cargo.toml`)
- 集中环境清单不是所有诊断变量的动态注册器，局部读取和 OnceLock 仍存在；修改变量名或默认值需查消费者生效时机。 (`crates/runtime-config/src/lib.rs`, `crates/protocol/src/transport.rs`)

## Architecture

- 产品配置在消费者初始化、模块选择或当前帧交付时生效，不能把静态配置值直接展示为已运行角色状态。 (`apps/browser/src/process_backend.rs`, `apps/compositor/src/lib.rs`, `apps/android-browser/rust/src/lib.rs`)
- 进程内测试、发布 IPC 与设备 role 复用部分契约，但跨平台权限、异步等待及帧句柄需要各自核验。 (`apps/browser/src/headless/session.rs`, `apps/android-browser/rust/src/lib.rs`, `crates/protocol/src/frame_shm.rs`)

## Child Knowledge Nodes

- [运行时配置的入口、默认值与生效时机](runtime-environment.md) — 默认值、解析差异、owner 与 client 生效时间。
- [脚本后端、无引擎构建与 feature 并集](script-features.md) — V8/QuickJS、可选依赖、单包与 workspace 差异。
- [平台、发布路径、测试宿主与 GPU 交付分支](platform-and-product-modes.md) — 桌面/Android、headless/GUI、线程与 GPU 帧交付。
