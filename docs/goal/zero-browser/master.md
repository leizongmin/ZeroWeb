# ZeroBrowser 运行时控制平面

**最后更新**: 2026-05-30
**当前活跃里程碑**: M7 ✅ 已完成 | 下一活跃：M8 — 多进程 IPC + M9 — Storage
**执行状态**: M1-M5、M7 全部完成。M6 (V8) 暂缓。继续推进 M8 和 M9 的可独立实现部分。

---

## 当前仓库事实

| 项 | 状态 |
|----|------|
| 仓库代码 | ✅ Cargo workspace + 18 个 crate（7 个已实现） |
| 编译状态 | ✅ `cargo build --workspace` 通过 |
| 测试状态 | ✅ `cargo test --workspace` 538 个测试全绿 |
| 覆盖率 | ✅ 所有已实现 crate 各模块 ≥83% |
| 性能基线 | ✅ 31+ 个 criterion 基准可运行 |
| CI | ✅ GitHub Actions 配置就位 |
| Clippy | ✅ 零警告（全 workspace） |

### 已实现 crate

| Crate | 测试 | 覆盖率 |
|-------|------|--------|
| dom | 82 | 85.45% |
| css-parser | 138 | 86.88% |
| style-system | 101 | ≥85%/模块 |
| layout-engine | 61 | ≥83%/模块 |
| engine-core | 39 | ≥98%/模块 |
| net | 32 | — |
| security | 24 | — |
| render-foundation | 53 | 53.30% |

---

## 里程碑状态

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| M1 项目骨架 + 渲染基础设施 | ✅ 完成 | 归档 |
| M2 HTML 解析 + DOM 树 | ✅ 完成 | 归档 |
| M3 CSS 解析器 + 样式系统 | ✅ 完成 | 归档 |
| M4 布局引擎 | ✅ 完成 | 归档 |
| M5 渲染管线集成 | ✅ 完成 | 归档 |
| M6 JavaScript 集成 (V8) | ⏸ 暂缓 | 需要 rusty_v8 二进制 |
| M7 网络栈 + 导航模型 | ✅ 完成 | 归档 |
| **M8 多进程架构** | 🔄 进行中 | protocol crate (IPC) |
| **M9 Canvas + Storage** | 🔄 进行中 | storage crate |

---

## 下一步计划

1. ~~M1-M5, M7~~ ✅ 全部完成
2. M8: 实现 `protocol` crate（IPC 消息序列化/反序列化）
3. M9: 实现 `storage` crate（localStorage、sessionStorage、IndexedDB 基础）
4. M6: V8 集成（待环境准备就绪）

---

## 归档记录

- **M1** ✅ → [archive/m1-skeleton-render-foundation.md](archive/m1-skeleton-render-foundation.md)
- **M2** ✅ → [archive/m2-dom.md](archive/m2-dom.md)
- **M3** ✅ → [archive/m3-css-parser-style-system.md](archive/m3-css-parser-style-system.md)
- **M4** ✅ → [archive/m4-layout-engine.md](archive/m4-layout-engine.md)
- **M5** ✅ → [archive/m5-rendering-pipeline.md](archive/m5-rendering-pipeline.md)
- **M7** ✅ → [archive/m7-network-security.md](archive/m7-network-security.md)
