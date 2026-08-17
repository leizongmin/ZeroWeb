# 媒体元素 — 运行时控制面板（master.md）

**入口文档**: [../media-elements.md](../media-elements.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-17（立项——M1 待启动）

---

## 当前状态

**专项定位**：媒体方向三拆之一（可立即启动）。HTMLMediaElement 非解码语义面（状态机/
事件序列/API 行为），WPT media-0 真实用例驱动。**不被解码选型 RFC 阻塞**——headless
近似驱动先行，兄弟目标建成后替换驱动源。

**与兄弟 goal 的边界**：
- media-playback — 解码/帧渲染归其管（RFC 门控）；本目标的 readyState 真实驱动源由其
  供给（接口契约记录于两流 master.md）
- media-audio — 音频输出归其管；volume/muted 本目标只做语义（真增益归其接线）
- js-dom — 媒体反射段（part01.js R3040）共享，`git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ 属性反射：R3040 autoplay/controls/loop/muted/playsInline + 布局占位渲染
- ⚠️ load 算法/readyState/networkState 状态机缺失
- ⚠️ 事件序列（loadstart/canplay 等 20+ 事件）未实现
- ⚠️ canPlayType 未核实（M1 摸底）
- ⚠️ 元数据面（duration/currentTime/volume）stub 或缺失
- ⚠️ track 面缺失
- ⚠️ WPT media-0 未导入，无基线

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| M1g | WPT media-0 用例覆盖为零 | ⬜ M1 |
| M2g | load 算法 + 状态机缺失 | ⬜ M2 |
| M3g | 事件序列缺失 | ⬜ M2 |
| M4g | canPlayType/异常/元数据/track 面缺失或未核实 | ⬜ M1/M3 |

## 下一步计划

1. **M1 切片 1**：WPT media-0 用例导入 + 基线（零源码改动）
2. **M1 切片 2**：失败聚类 → 反射面/状态机/事件面已有 vs 缺失清单
3. **M1 切片 3**：canPlayType + 基础反射深化（preload/crossOrigin/controlsList）

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT media-0 基线 + 摸底 | ⬜ 待启动 |
| M2 — 状态机与事件序列 | ⬜ |
| M3 — API 语义 + track 面 + 播放层衔接 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿；clippy 零警告
- WPT media-0 面：无基线（未导入）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
