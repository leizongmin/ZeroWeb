# 事件循环与异步回调 spec 化 — 运行时控制面板（master.md）

**入口文档**: [../event-loop-spec.md](../event-loop-spec.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-11（M1 切片 1 完成——IO/RO WPT 基线 29.7%）

---

## 当前状态

**专项定位**：父目标 P1a 遗留面收敛（microtask checkpoint 简化版 = DC2 缺口②、host 侧
MO 通知端死路、IO/RO WPT 覆盖为零）。rAF 帧驱动切片已落地不重做；本目标三线 = IO/RO
WPT 基线 → MO host 触发（方案 C 设计已存在）→ checkpoint spec 化（kill-switch）。

**与兄弟 goal 的边界**：
- rendering-compat — crate 层面零重叠（渲染流域活跃面是 css-parser/style-system/
  layout-engine/render-foundation）；engine 属共享面，碰前 `git log` 核对
- webdriver — apps/renderer 共享：该流只碰 Automation 消息处理段，本流只碰 tick 排布段
  （page_scripts.rs/runtime.rs/js_worker.rs）；发现要碰对方段即暂停记入本表
- web-components — part01.js 无共享段（该流主力 part03/part04/part05 + dom_bindings），
  但同属 engine 共享大文件池，碰前互相 `git log` 核对

## 实测基线（2026-09-07 立项时）

### 现有实现

- ✅ rAF 帧驱动切片（kill-switch `ZW_RAF_FRAME_DRIVEN`，默认 OFF 同步 stub；js_worker.rs
  L40/L584/L680 + part01.js L3049-3070）——不重做，reftest 同步 stub 约束有效
- ✅ IO/RO B-gen 生产实现（part01.js L2568-2927）：observe 时 initial notification +
  `__zw_observers_tick` post-render 持续跟踪（threshold 越界/size-diff）；apps/renderer
  tick 接线完成（page_scripts.rs L353 + runtime.rs L708）
- ✅ 几何反馈基建：rect_bridge.rs 493 行同步 `__zw_getBoundingClientRect` + DOMRect
  真原型链（R3319）
- ✅ MO 双轨：polyfill MO（part01.js L742+ Proxy trap）观测 JS 驱动 mutation；dom 层
  `mutation.rs` + `pending_mutations` 记录端可用
- ⚠️ microtask checkpoint 简化版：v8_runtime.rs `perform_microtask_checkpoint`
  （L422/L486）仅 execute 末整批排空，非 spec per-task；无显式 task queue
- ⚠️ MO 通知端死路：engine 全仓零调用 `process_mutations`/`take_mutation_records`——
  host 驱动 mutation 不可观测
- ⚠️ MO 方案 C（hybrid 共享注册表 + host hook + NodeId↔handle 身份桥）设计完成
  （`zero-web/p1b-mutationobserver-host-trigger-design-2026-08-10.md`）未实施
- ⚠️ IO/RO WPT 覆盖为零（wpt-data 无 observer 目录、imported-tests.txt 零命中；现有
  测试全自写）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | IO/RO WPT 用例覆盖为零（fetch 脚本 + 导入 + 基线） | ✅ 2026-09-11（基线 29.7% 落 evidence/） |
| P2 | 事件循环时序差距清单（对照 spec 逐条）未建立 | ⬜ M1 |
| P3 | MO host 触发未实施（通知端死路） | ⬜ M2 |
| P4 | checkpoint 简化版（无 task queue、无 per-task checkpoint） | ⬜ M3 |

## 已完成切片

### M1 切片 1 — IO/RO WPT 基线（2026-09-11，零源码改动纯资产）✅

- `tests/wpt-runner/scripts/fetch-observers-subset.sh`（WPT `3159769338` pin，110 IO +
  40 RO top-level .html 全量 + resources 全目录；.window.js 包装形态不拉取）
- runner 入口 `make testharness-intersection-observer` / `make testharness-resize-observer`
  （`observers_case_skipped` 内容规则筛减：ref 页 / source 含 `<iframe` / v2 / resources；
  内容规则而非名字规则——document-scrolling-element-root.html 名字无 iframe 但内容用）
- **基线**：IO 81 cases / 217 subtests / **30.9% Pass**；RO 33 cases / 56 subtests /
  **25.0% Pass**（合计 29.7%）。明细 + 失败聚类：
  `evidence/2026-09-11-m1-observers-wpt-baseline.{md,json}`
- 账本：114 用例记入 `imported-testharness.txt`（`EVLOOP-M1-observers-baseline`）
- 关键归因：IO 最大失败簇（65×）= 初通知后动态跟踪面不可达——testharness runner 不调
  `__zw_observers_tick` 且无渲染循环；runner 环境限制已记入 evidence，切片 3 修

## 下一步计划

1. **M1 切片 2**：事件循环时序差距清单（v8_runtime.rs checkpoint 调用点盘点 +
   HTML spec 事件循环算法逐条对照）
2. **M1 切片 3**：IO/RO 语义轻量修复队列，从基线失败聚类取序：
   - 候选 a：runner 侧 observer tick 接线（对齐 renderer `tick_observers` 语义，
     解锁动态跟踪面——参照 `ZW_TESTHARNESS_RAF_FRAME_DRIVEN` opt-in 先例）
   - 候选 b：IO 构造器异常校验（threshold 范围 / rootMargin 语法 throw，零几何依赖）

**碰撞管理**：碰 engine 前先 `git log --since="14 days ago" -- crates/engine/
crates/script-sandbox/` 核对渲染流域活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线 + 时序差距清单 | 🔄 切片 1 ✅（切片 2/3 待做） |
| M2 — MutationObserver host 触发 | ⬜ |
| M3 — checkpoint spec 化 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿（`make test` / `make reftest` 入口，经 test-guard 包裹；
  禁止裸跑 cargo test）
- IO/RO 用例面：无基线（未导入/未建）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  时序变更必须 kill-switch + 全量 A/B 零回归；渲染相关门禁（product-smoke/bench-gate）
  在 tick 排布变更轮按 run-rules §12 判断是否需要
