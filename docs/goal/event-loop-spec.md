# 事件循环与异步回调 spec 化 — microtask checkpoint / host 侧 MutationObserver / IO-RO WPT 基线

**版本**: v1.0
**日期**: 2026-09-07
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（P1a 非阻塞 follow-up + DC2 缺口②「事件循环 microtask
checkpoint 时序简化」）

> **说明**
> 本文档是 ZeroWeb「事件循环与异步回调 spec 化」专项目标执行契约。rAF 帧驱动切片已落地
> （kill-switch `ZW_RAF_FRAME_DRIVEN`），P1a 的遗留面收敛为三件：① microtask checkpoint
> 是「每 execute 末整批排空」简化版而非 spec per-task checkpoint；② MutationObserver 只
> 观测 JS 驱动 mutation（polyfill Proxy trap），host/native 路径 mutation 的通知端是死路
> （dom 层 `pending_mutations` 记录端可用、`process_mutations` 零调用）；③ IO/RO 已有
> post-render tick 持续跟踪但 WPT 上游用例覆盖为零。本文定义 Mission、边界、Done Criteria、
> 执行协议和文档治理规则，供后续 `rally run` 会话作为稳定输入。日常进展、evidence、
> active milestone 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-09-07 用户决策）**：从父目标 P1a 遗留面拆出。理由：① microtask
> checkpoint 时序是 master.md Done Criteria 评估明确记录的 DC2 缺口②——交互式网站可用性
> 的时序基础；② host 侧 MutationObserver 设计文档已存在
> （`zero-web/p1b-mutationobserver-host-trigger-design-2026-08-10.md`，方案 C hybrid
> 已选型），只差实施；③ IO/RO 的 WPT 基线从零建立与①②同域（`part01.js` observers 段 +
> script-sandbox 执行时序），拆开会造成同一文件两流碰撞——**合并为一个 goal**；④ 改动域
> （js_dom_shim/part01.js、script-sandbox、apps/renderer 事件循环段）与 rendering-compat
> 渲染流域 crate 域不重叠（engine 的 shim 层非其活跃编辑面）。
>
> **▶ 基线事实（2026-09-07 实测）**：
> - **microtask checkpoint（简化版）**：`crates/script-sandbox/src/v8_runtime.rs`
>   `perform_microtask_checkpoint`（L422/L486）仅在每个 `execute()`/`execute_json()` 末尾
>   调用一次——「execute 退出时整批排空」，非 spec 的每 task 前后 + 每回调后 checkpoint；
>   无 host 侧 event loop、无显式 task queue（宏任务 = `ResolveAsyncCallback` FIFO 隐式）。
>   bridge 测试注释佐证此语义（js_dom_bridge_tests/part06.rs L81 等）。
> - **rAF（已落地帧驱动切片）**：`js_dom_shim/part01.js` L3049——kill-switch
>   `__ZW_RAF_FRAME_DRIVEN`（env `ZW_RAF_FRAME_DRIVEN`，apps/renderer/src/js_worker.rs
>   L40 判定 + L584/L680 注入）；ON = `_rafPending` 队列 + render 后 `__zw_raf_tick(ts)`
>   （L3067）；OFF（默认）= 同步 stub 预算执行（`_rafBudget = 64`/execute）。
>   **reftest 单渲染路径（`render_to_framebuffer`）必须保留同步 stub**（设计文档明确约束）。
> - **MutationObserver（双轨）**：polyfill MO（part01.js L742+，JS Proxy trap + `_defer`
>   microtask）只观测 JS 驱动 mutation；dom 层 MO（`crates/dom/src/mutation.rs` +
>   document/mod.rs 多处入 `pending_mutations`）**记录端可用、通知端死路**
>   （engine 全仓零调用 `process_mutations`/`take_mutation_records`）。设计文档
>   `p1b-mutationobserver-host-trigger-design` 推荐方案 C（hybrid：native observe 注册进
>   polyfill `__zw_mo_observers` 共享注册表 + host hook 投递 `_mo_notify`；唯一新增面 =
>   NodeId↔handle/selector 身份桥，R3106 已建关联）——**未实施**。
> - **IO/RO（部分实现）**：A-gen stub（`dom_bridge.rs` L1545-1615，永不触发）与 B-gen
>   生产实现并存（part01.js L2568-2927——observe 时 initial notification +
>   `__zw_observers_tick`（L2917）post-render 持续跟踪：threshold 越界/size-diff 派发）；
>   几何反馈基建就位（`rect_bridge.rs` 493 行同步 `__zw_getBoundingClientRect` +
>   DOMRect 真原型链 R3319）；apps/renderer `page_scripts.rs` L353 `tick_observers` +
>   `runtime.rs` L708 接线已完成。
> - **WPT 覆盖**：intersection-observer / resize-observer 目录在 wpt-data 中**不存在**，
>   imported-tests.txt 零命中；现有 IO/RO 测试全为自写（integration/dom_bridge_polyfill.rs、
>   js_worker.rs、js_dom_bridge_tests/part09.rs）。

---

## Mission

以 **WPT 真实用例 + HTML spec 事件循环算法定义为验证标准**，把异步回调时序从「execute
末整批排空」的简化版推进到 spec 形态：per-task microtask checkpoint、host 侧 mutation
通知接线、IO/RO 上游用例基线。分阶段里程碑校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **基线建立** | IO/RO WPT 用例导入 + 通过率基线；事件循环时序差距清单（对照 spec 算法逐条） |
| 中期 | **MutationObserver host 触发** | 方案 C hybrid 实施——dom 层通知端接活，kill-switch 切换 |
| 长期 | **checkpoint spec 化** | per-task microtask checkpoint（显式 task queue），kill-switch + A/B |

**关键约束**：
1. **rAF 帧驱动切片已落地不重做**；其 reftest 同步 stub 约束（`render_to_framebuffer`
   单渲染路径）继续有效，本目标一切时序变更不得破坏该路径。
2. 所有验证必须基于上游 WPT 真实用例（IO/RO 面）或 spec 算法逐条对照记录（事件循环面），
   不允许手写 inline 用例充数。
3. checkpoint spec 化是行为时序变更——必须 kill-switch 默认 OFF + 全量 A/B 零回归才允许
   default-on；A/B 有回归即回退并记录。

覆盖范围：

1. **microtask checkpoint** — 从 per-execute 整批排空到 spec「clean up after running
   script + 每 task 结束 checkpoint」形态（显式 task queue 结构）
2. **MutationObserver host 触发** — dom 层 `pending_mutations` 通知端接活（方案 C：
   共享注册表 + host hook），JS 驱动与 host 驱动 mutation 统一可观测
3. **IO/RO WPT 基线与语义收口** — 上游 `intersection-observer`/`resize-observer` window
   可执行面导入；root=元素、rootMargin、threshold 语义按 WPT 修齐
4. **task queue 基础** — setTimeout/rAF/rIC/MO/IO 回调的统一排队结构（为 checkpoint
   spec 化提供宿主）

执行方式：**WPT 基线先行** — IO/RO 导入是零源码改动的纯资产切片，先建立验收标尺再动时序。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| microtask 时序 | script-sandbox checkpoint 调用点重构、显式 task queue | kill-switch 默认 OFF |
| MO host 触发 | dom `mutation.rs` 通知端 + engine 接线（方案 C 设计已存在） | 照设计文档切片实施 |
| IO/RO 语义 | part01.js observers 段语义修齐（root/rootMargin/threshold） | 以 WPT 为准 |
| apps/renderer tick 链 | page_scripts.rs / runtime.rs / js_worker.rs 的 tick 排布 | 只动 tick 排布，不动渲染管线 |
| WPT 基础设施 | intersection-observer / resize-observer 用例导入 | 复用 tests/wpt-runner + `make import-wpt`；新增 fetch 脚本 |
| 单元测试 | 每项修复带单测 + 时序断言 | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **rAF 帧驱动重做** — 已落地（kill-switch 已存在），只修其约束下的时序配合
- **requestIdleCallback 真实 idle 时序** — part01.js 自述「基础可用实现」；真实 idle 判定
  需帧调度深化，记「待用户决策」
- **Web Workers 内部事件循环** — worker 事件循环属 SW/worker 域（已归档 goal 面），不碰
- **渲染管线 / 帧调度性能** — rendering-compat / layout-perf 流域，本流只消费 render 完成
  信号，不改渲染本身

### 依赖约束

- **与 rendering-compat 流边界（run-rules §9）**：本流改动域 = `crates/engine/src/
  js_dom_shim/part01.js`（observers/MO/时序段）+ `crates/script-sandbox` +
  `apps/renderer`（tick 排布段）+ `crates/dom/src/mutation.rs` + WPT 导入资产 + 本 goal
  控制面。渲染流域活跃编辑面是 css-parser/style-system/layout-engine/render-foundation
  与 reftest 资产——**crate 层面零重叠**；engine 属共享面，碰前 `git log --since=
  "14 days ago" -- crates/engine/ crates/script-sandbox/` 核对。
- **与 webdriver 流**：apps/renderer 共享——该流只碰 Automation 消息处理段，本流只碰
  tick 排布段；发现要碰对方段即暂停记入 master.md（碰头信号）。
- **与已归档 js-dom goal**：其 escape-hatch 收敛遗产（native CE 路径）依赖 MO host 触发
  作为必要伴随（设计文档明言）——本目标的 MO 实施即为该遗产补齐最后一环。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 用例导入与通过率基线

- [ ] 从上游 WPT 导入 `intersection-observer` / `resize-observer` window 可执行面真实用例；
      新增 fetch 脚本（照 indexeddb/cache-storage 先例）
- [ ] 建立分类通过率报告（文本 + JSON），记录基线
- [ ] 事件循环时序差距清单（对照 HTML spec 事件循环算法逐条标注现状）持久化到 evidence/
- [ ] 每项修复的 driving WPT 用例经常驻断言集并记入账本（`imported-testharness.txt`）

### DC-2: MutationObserver host 侧触发

- [ ] dom 层 `pending_mutations` 通知端接活（方案 C hybrid：共享注册表 + host hook 投递）
- [ ] host 驱动 mutation（native 路径 DOM 操作）可被页面 MO 观测（端到端测试：native
      appendChild → 页面 MO 回调收到记录）
- [ ] NodeId↔handle/selector 身份桥接（R3106 关联复用）
- [ ] kill-switch 门控 + JS 驱动路径零回归（polyfill MO 现有测试全绿）

### DC-3: microtask checkpoint spec 化

- [ ] 显式 task queue 结构落地（setTimeout/rAF/rIC/MO/IO 回调统一排队）
- [ ] per-task microtask checkpoint（kill-switch 默认 OFF）
- [ ] 全量 A/B 零回归后 default-on（A/B 有回归即回退并记录结论）
- [ ] reftest 单渲染路径（同步 stub 约束）保持有效

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + driving WPT 用例资产化
- [ ] `make reftest` 无回归（时序变更的渲染面守卫）

---

## 活跃里程碑

### M1 — WPT 基线 + 时序差距清单

**目标**：IO/RO 用例导入 + 通过率基线；事件循环差距清单（对照 spec 逐条）；失败聚类。

**切片建议**：
1. fetch 脚本 + IO/RO 用例导入 + 基线报告（零源码改动，纯资产）
2. 事件循环时序差距清单（v8_runtime.rs checkpoint 调用点盘点 + spec 算法对照）
3. IO/RO 语义轻量修复队列（rootMargin/threshold 等 WPT 驱动）

### M2 — MutationObserver host 触发

**目标**：方案 C 实施——dom 通知端接活 + 共享注册表 + 身份桥；kill-switch + A/B。

### M3 — checkpoint spec 化

**目标**：task queue 结构 + per-task checkpoint（kill-switch 默认 OFF → A/B 零回归 →
default-on）→ DC 全满足判定。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；验证基于上游真实 WPT 用例（无内建 inline 充数）；
`cargo build` + `make test` + `cargo clippy` 全通过；master.md 内部自洽，archive 已建立。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**checkpoint 调用点、MO 通知端、IO/RO 语义的确切差距
2. **自主导入** WPT IO/RO 用例，扩大覆盖范围
3. **自主实施** MO 方案 C 切片（照设计文档，kill-switch + 独立 land）
4. **自主验证**：`make test` + `make reftest` + clippy + WPT 通过率；时序变更必须 A/B
5. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：WPT 驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项（如 rIC 真实 idle、checkpoint default-on 时机）记
   「待用户决策」清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：engine 共享面碰前 `git log` 核对；apps/renderer 只碰 tick 排布段，
   不碰 Automation 段（webdriver 流活跃域）。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。当作当前任务的一部分修复，直到稳定可重复。
2. **时序回归**：A/B 出现新失败即回退本切片，记录结论，换下一切片——不硬推。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/event-loop-spec/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/event-loop-spec/archive/`：只追加不修改。
- **证据区域** `docs/goal/event-loop-spec/evidence/`：通过率报告、A/B 证据、差距清单，
  持续追加。
