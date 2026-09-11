# 事件循环与异步回调 spec 化 — 运行时控制面板（master.md）

**入口文档**: [../event-loop-spec.md](../event-loop-spec.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-11（M1 切片 3a+3b——IO/RO 基线 29.7% → 38.8%）

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
| P2 | 事件循环时序差距清单（对照 spec 逐条）未建立 | ✅ 2026-09-11（evidence/2026-09-11-m1-event-loop-gap-list.md） |
| P2.5 | IO/RO 语义修齐第一批（tick 接线 + threshold 越界 + 构造器校验/getter） | ✅ 2026-09-11（切片 3a+3b，基线 29.7% → 38.8%，evidence/2026-09-11-m1-slice3-observers-wiring-validation.md） |
| P3 | MO host 触发未实施（通知端死路） | ⬜ M2 |
| P4 | checkpoint 简化版（无 task queue、无 per-task checkpoint） | ⬜ M3 |

## 已完成切片

### M1 切片 3a+3b — IO/RO 动态跟踪接线 + 语义修齐第一批（2026-09-11）✅

- 切片 3a：runner probe 循环接 `__zw_observers_tick`（对齐 renderer `tick_observers`）
  + IO threshold 越界判定 spec 化（`_thresholdIndex` + `_crossed` OR 臂——thresholdIndex
  或 isIntersecting 任一变化即派发；旧双侧比较在默认 [0] 下永不判越阈 = 65× 簇根因）
- 切片 3b：IO 构造器校验（threshold RangeError/TypeError、rootMargin SyntaxError
  DOMException）+ `root`/`thresholds`/`rootMargin` getter + IO/RO `observe()` 非 Element
  TypeError（nodeType 判据——第一版 handle/selector 判据误伤 shadow/detached 元素已修正）
- 基线：IO 67→87（30.9%→40.1%）、RO 14→19（25.0%→33.9%）、合计 29.7%→38.8%（+9.1pp）
- 质量门禁：make test 19,134P/0F + clippy 零警告 + fmt 干净；明细与翻绿清单落
  `evidence/2026-09-11-m1-slice3-observers-wiring-validation.md`
- 残留取序（3c+）：几何真值簇（transform/zoom/scroll-offset——多数属渲染流域，碰头
  协调）、runner viewport 校准（800×600 vs 1280×800，**待用户决策**，牵动全部套件）、
  documentElement.clientHeight viewport 化、zero-area ratio=1 + edge-inclusive、
  target-is-root、detached-doc 初通知抑制

### M1 切片 2 — 事件循环时序差距清单（2026-09-11，纯文档）✅

- `evidence/2026-09-11-m1-event-loop-gap-list.md`：checkpoint 调用点精确盘点
  （v8_runtime.rs:422/486 唯一两点）+ HTML spec 事件循环算法逐条对照 + 差距→里程碑映射
- 关键发现：① 常规路径（每 `<script>` / 每异步回调独立 execute）边界已吻合 spec；
  ② 违反点集中在**批量派发**——runner `__zw_fire_due_timers`（N timer 一个 execute）、
  renderer `tick_observers`（IO/RO/rAF 全部回调一个 execute）；③ part01.js 行号已漂移
  （MO 现 L2113+、IO L2866+、rAF L3388+，契约 9/7 快照过时）
- 待用户决策清单新增登记：requestIdleCallback 真实 idle 时序（已在 goal 范围外条款）

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

1. **IO/RO 语义轻量修复第二批（切片 3c，从残留聚类取序，全 in-shim）**：
   - documentElement.clientWidth/Height 对根元素返 viewport 尺寸（empty-root-margin）
   - zero-area target intersectionRatio=1（spec §2.2.11-12）+ edge-inclusive isIntersecting
   - target-is-root（root==target skip-to-step-11 臂 → 不相交）
   - detached document 初通知抑制（ownerDocument≠主文档 → observe 不派初通知，
     adopt 后 tick 重算；与 takeRecords 排队模型一并评估）
2. **M2**：MO host 触发（方案 C 实施）
3. **M3**：task queue + per-task checkpoint（kill-switch → A/B → default-on）

**待用户决策清单**：
- runner viewport 校准（1280×800 → 上游 WPT 校准 800×600）：牵动全部 testharness
  套件的绝对几何期望（一次性大重校准），非轻量修复
- requestIdleCallback 真实 idle 时序（已在 goal 范围外条款）

**碰撞管理**：碰 engine 前先 `git log --since="14 days ago" -- crates/engine/
crates/script-sandbox/` 核对渲染流域活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线 + 时序差距清单 | ✅ 2026-09-11（基线 29.7% + 差距清单；语义修齐第一批 3a+3b 落地 → 38.8%，第二批 3c 取序中） |
| M2 — MutationObserver host 触发 | ⬜ |
| M3 — checkpoint spec 化 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿（`make test` / `make reftest` 入口，经 test-guard 包裹；
  禁止裸跑 cargo test）。2026-09-11 切片 3a+3b 后：make test 19,134P/0F
- IO/RO 用例面：基线 29.7%（IO 30.9% / RO 25.0%）→ 切片 3a+3b 后 **38.8%**
  （IO 87/217 = 40.1% / RO 19/56 = 33.9%），明细见 evidence/
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  时序变更必须 kill-switch + 全量 A/B 零回归；渲染相关门禁（product-smoke/bench-gate）
  在 tick 排布变更轮按 run-rules §12 判断是否需要（本切片仅 runner probe 循环加一次
  shim 调用 + JS 语义，未触渲染管线，product-smoke 不适用）
