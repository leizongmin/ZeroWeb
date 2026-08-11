# Dependency Upgrade Backlog

待升级但存在 **API 破坏性变更** 的依赖项清单，需通过专项逐个处理。

> 最后检查日期：2026-08-12

## 概要

| 依赖 | 当前 | 目标 | 破坏性 | 风险 | 优先级 |
|------|------|------|--------|------|--------|
| [wgpu](#1-wgpu) | `24` | `29` | Major 5 个版本 | 高 | P1 |
| [winit](#2-winit) | `0.30` | `0.31` | pre-1.0 minor | 中 | P2 |
| [html5ever](#3-html5ever) | `0.29` | `0.39` | pre-1.0 minor ×10 | 高 | P2 |
| [reqwest](#4-reqwest) | `0.12` | `0.13` | pre-1.0 minor | 低 | P3 |
| [rusty_v8](#5-rusty_v8) | `0.32` | 上游最新 | 受 Deno 发布制约 | 高 | P3 |
| [wasmi](#6-wasmi) | `0.40` | `1.1` | Major 1.x（0→1） | 高 | P2 |

---

## 1. wgpu

- **当前**: `24`
- **目标**: `29`
- **涉及 crate**: `render-foundation`, `host-runtime`, `canvas`, `engine`
- **跳过原因**: 跨越 5 个主版本（24→29），wgpu 每个主版本都有渲染管线、资源绑定、surface 配置等 API 大改。
- **预计影响**:
  - `wgpu::SurfaceConfiguration` / `wgpu::Surface` 创建参数变化
  - 资源绑定组 API（BindGroupLayout、BindGroup）签名调整
  - 可能的着色器编译接口变更
  - `wgpu-hal` 内部 API 变化（仅影响直接使用 hal 的代码）
- **升级策略**:
  1. 先读取 wgpu [changelog](https://github.com/gfx-rs/wgpu/releases) 逐版了解 breaking changes
  2. 每次只升 1-2 个主版本，确保编译通过后再继续
  3. 重点关注 `render-foundation` 中的 GPU 管线代码
  4. 升级后跑渲染相关测试 + 手动验证浏览器窗口正常显示

---

## 2. winit

- **当前**: `0.30`
- **目标**: `0.31`（beta）
- **涉及 crate**: `host-runtime`
- **跳过原因**: `0.31` 仍处于 beta 阶段，窗口创建、事件循环 API 有调整。
- **升级策略**:
  1. 等待 `0.31` 正式发布
  2. 关注 `ApplicationHandler` trait 变化（0.31 重构了事件处理模型）
  3. `host-runtime` 是唯一消费者，影响面可控

---

## 3. html5ever

- **当前**: `0.29`
- **目标**: `0.39`
- **涉及 crate**: `dom`
- **跳过原因**: 跨越 10 个 pre-1.0 小版本，解析器驱动 trait（`TreeSink`）API 大概率有 breaking changes。
- **升级策略**:
  1. 检查 `html5ever` [changelog](https://github.com/servo/html5ever/releases) / 源码 diff
  2. `dom` 实现了 `TreeSink` trait，是核心适配点
  3. 升级后必须跑 DOM 构建 + CSS 选择器 + WPT reftest 全套测试
  4. 备选：评估是否迁移到其他 HTML 解析器

---

## 4. reqwest

- **当前**: `0.12`
- **目标**: `0.13`
- **涉及 crate**: `net`
- **跳过原因**: pre-1.0 minor bump，HTTP 客户端 API 可能调整（TLS 配置、连接池、`ClientBuilder` 等）。
- **升级策略**:
  1. 检查 `reqwest` release notes
  2. `net` crate 封装了 HTTP 能力，改动集中在 `net/src/` 
  3. 网络相关测试覆盖即可

---

## 5. v8（原 rusty_v8，已更名）

- **当前**: `150.2.0`（crate 已从 `rusty_v8` 更名为 `v8`，仓库不变，仍由 Deno 维护）
- **目标**: 上游最新
- **涉及 crate**: `script-sandbox`
- **跳过原因**: `v8` crate 的发布周期受 Deno 上游 V8 版本制约，API 变化频率高且不可预测。跨版本升级可能涉及 V8 isolate 创建、inspector、snapshot 等底层 API 变更。
- **升级策略**:
  1. 关注 [rusty_v8 releases](https://github.com/denoland/rusty_v8/releases)（crate 名已为 `v8`，仓库路径不变）
  2. 确保 CI 中相关平台（Linux x86_64/aarch64、macOS x86_64/aarch64、Windows x86_64）的 V8 archive 可用（`make setup-rusty-v8`）
  3. `script-sandbox` 是唯一消费者，升级后需跑 V8-backed 测试套件
  4. 同步更新 CI 中 Windows ARM64 的 V8 archive 可用性检查

---

## 6. wasmi

- **当前**: `0.40`
- **目标**: `1.1`
- **涉及 crate**: `wasm-sandbox`
- **跳过原因**: 0.40 → 1.1 是主版本升级，`wasmi::core` 模块变为私有，`ValType`、`TrapCode`、`HostError` 等类型导出路径变更。
- **具体 breaking changes**（基于编译错误）:
  - `wasmi::core::ValType` → 需通过公共 API 重新导出
  - `wasmi::core::TrapCode` → 同上
  - `wasmi::core::HostError` → trait 移动到其他位置
  - `wasmi::core` 模块整体变为私有，须通过 `wasmi::` 根路径访问
- **升级策略**:
  1. 查阅 [wasmi changelog](https://github.com/wasmi-labs/wasmi/releases)
  2. 适配 `wasm-sandbox/src/wasmi_backend.rs` 中的类型引用
  3. 升级后跑 wasm-sandbox 测试

---

## 升级流程建议

每次专项升级一个依赖时：

1. **创建 feature 分支**: `upgrade/<dependency-name>`
2. **修改 `Cargo.toml`** 中的版本约束
3. **`cargo check --workspace`** 定位编译错误
4. **逐文件修复** API 适配
5. **`cargo test --workspace`** 确保测试通过
6. **`cargo clippy --workspace --all-targets -- -D warnings`** 无新增警告
7. **提交 PR** 附带 changelog 链接和改动说明
8. 如测试/CI 未覆盖受影响路径，需手动验证
