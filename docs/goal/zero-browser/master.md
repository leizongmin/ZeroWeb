# ZeroBrowser 运行时控制平面

**最后更新**: 2026-05-30
**当前活跃里程碑**: M1 — 项目骨架 + 渲染基础设施迁移
**执行状态**: 进行中（M1 骨架已建立，待完成 wgpu 渲染 demo）

---

## 当前仓库事实

| 项 | 状态 |
|----|------|
| 仓库代码 | ✅ Cargo workspace + 16 个 crate 骨架 |
| 编译状态 | ✅ `cargo check --workspace` 通过 |
| 测试状态 | ✅ `cargo test --workspace` 38 个测试全绿 |
| 覆盖率 | 待测量（脚本就位） |
| WPT 通过率 | N/A |
| 性能基线 | ✅ `cargo bench` 5 个基准可运行 |
| CI | ✅ GitHub Actions 配置就位 |
| Clippy | ✅ 零警告 |

### 仓库结构

```
crates/           16 个 crate（dom, css-parser, style-system, layout-engine,
                  engine-core, canvas, render-foundation, host-runtime, net,
                  security, storage, protocol, script-sandbox, wasm-sandbox,
                  webview-api, browser-shell）
apps/             browser（占位入口）, webview-demo（占位入口）
tests/            wpt-runner, integration, benchmarks/results
scripts/          run-benchmarks.sh, check-coverage.sh
.github/          CI 管线（三平台 build + test + clippy + 基准）
```

### 文档控制平面状态

- [x] 入口文档（`docs/goal/zero-browser.md`）已就位
- [x] 运行时控制平面（本文件）已创建
- [x] 归档区域（`docs/goal/zero-browser/archive/`）已创建
- [x] Spec + RFC 文档已就位

---

## 活跃里程碑：M1 — 项目骨架 + 渲染基础设施迁移

### M1 交付物进度

| # | 交付物 | 状态 | 备注 |
|---|--------|------|------|
| 1 | 完整的 Cargo workspace 结构，所有 crate 骨架就位 | ✅ 完成 | 16 crate + 2 apps |
| 2 | `render-foundation` crate 从 OmniTerm 迁移并适配 | 🔄 进行中 | 核心抽象已建立，待完整 GPU 渲染器迁移 |
| 3 | `host-runtime` crate 支持 winit 窗口创建和事件循环 | ✅ 完成 | winit 0.30 ApplicationHandler |
| 4 | 可以在 macOS/Linux/Windows 上创建窗口，使用 wgpu 渲染文本 | 🔲 待开始 | "Hello ZeroBrowser" demo |
| 5 | 所有 crate 编译通过，`cargo clippy` 无警告 | ✅ 完成 | 零警告 |
| 6 | `render-foundation` 单元测试（≥20 个测试用例） | ✅ 完成 | 24 个测试用例（geometry:10, color:5, primitive:4, font:6, surface:7） |
| 7 | criterion 基准基础设施就位 | ✅ 完成 | render-foundation/benches/ |
| 8 | `render-foundation` 首批基准（≥3 个） | ✅ 完成 | 5 个基准（damage_tracker, glyph_cache, frame_buffer, primitives） |
| 9 | 覆盖率测量脚本就位 | ✅ 完成 | scripts/check-coverage.sh |
| 10 | CI 管线就位 | ✅ 完成 | GitHub Actions 三平台 |

### render-foundation 已实现模块

| 模块 | 内容 | 测试 |
|------|------|------|
| `geometry` | Point, Size, Rect, DamageTracker | 10 个测试 |
| `color` | Color (RGBA), hex 解析, sRGB→linear, premultiplied alpha | 5 个测试 |
| `primitive` | FillPrimitive, GlyphPrimitive, RenderPrimitives | 4 个测试 |
| `font/loader` | FontLoader (fontdue), 字体加载和 glyph 光栅化 | 5 个测试 |
| `font/cache` | GlyphCache, LRU 淘汰策略 | 6 个测试 |
| `surface` | SurfaceDescriptor, FrameBuffer (CPU RGBA) | 7 个测试 |

### M1 验收标准

- ✅ `cargo build` 在 Linux 上成功
- ✅ `cargo test` 全通过
- ✅ `cargo clippy` 零警告
- ✅ `cargo bench` 可运行并输出结果
- 🔄 `cargo build` 在 macOS/Windows 上成功（CI 待验证）
- 🔲 运行 demo 二进制可以看到窗口和渲染文本
- 🔄 render-foundation 覆盖率 ≥ 50%（待测量）

---

## 性能基线（首次记录）

| 基准 | 耗时 | 说明 |
|------|------|------|
| damage_tracker/add_100 | ~6.5 µs | 添加 100 个脏矩形 |
| damage_tracker/damage_all | ~3.8 ns | 全区域脏标记 |
| glyph_cache/insert | ~10.5 µs | 插入 256 个 glyph |
| frame_buffer/clear_1080p | ~762 µs | 清除 1920x1080 帧缓冲 |
| primitives/build_1000_fills | ~1.7 µs | 构建 1000 个填充图元 |

---

## 已确认的技术决策

| 决策 | 选择 | 状态 |
|------|------|------|
| 技术路线 | Route A — 自建内核 | 已确认 |
| CSS 解析方案 | 完全自建 | 已确认 |
| JS 页面引擎 | V8（rusty_v8） | 已确认 |
| JS 扩展沙箱 | QuickJS（feature-gated） | 已确认 |
| 布局基础 | taffy 扩展 | 已确认 |
| 渲染基础 | OmniTerm 复用 + wgpu | 已确认 |
| 进程模型 | 浏览器进程 + 多渲染进程 | 已确认 |

---

## 下一步计划

1. ~~初始化 Cargo workspace~~ ✅
2. ~~创建 render-foundation 核心抽象~~ ✅
3. ~~实现 host-runtime（winit 窗口 + 事件循环）~~ ✅
4. ~~建立 CI 管线~~ ✅
5. **创建 "Hello ZeroBrowser" wgpu 渲染 demo** ← 当前
6. 迁移 OmniTerm wgpu 渲染器（glyph atlas、vertex layout、shader）
7. 提交并推送代码

---

## 未解决问题

| ID | 问题 | 优先级 | 状态 |
|----|------|--------|------|
| TBD-1 | MSRV（最低支持 Rust 版本）策略 | 重要 | 待定 |
| TBD-2 | OmniTerm 代码复用许可证确认 | 重要 | 假设同团队可复用 |
| TBD-3 | V8 二进制分发策略 | 重要 | 待定 |
| TBD-4 | CSS 解析器性能目标 | 重要 | 待定 |
| TBD-9 | 浏览器 UI 框架选型 | 重要 | 待定 |

---

## 归档记录

无已完成里程碑。
