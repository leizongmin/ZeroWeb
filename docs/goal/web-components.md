# Web Components — Custom Elements / template / slot 的 WPT 驱动补齐目标

**版本**: v1.0
**日期**: 2026-09-07
**状态**: Active（一期：dom/engine 侧注册/升级/slot 分配；Shadow DOM 渲染级 composed
tree 属深结构，等用户点名后协调 rendering-compat——见 Support Envelope 排除）
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（M12「Web Components（Custom Elements + Shadow DOM）」
+ Tier 2「Web Components」列项）

> **说明**
> 本文档是 ZeroWeb「Web Components」专项目标执行契约。Custom Elements 已有较高程度的
> 实现（真 registry + define 自动升级 + connected/disconnected/attributeChanged 三回调
> 端到端，v8 native 与 quickjs 双路径），`<template>` JS 层可用但 Rust DOM 层是占位，
> `<slot>` 接近零（Rust DOM 有孤立数据结构、engine 零接线、JS 面无任何 API）。目标是以
> WPT 真实用例为验证标准补齐缺口。本文定义 Mission、边界、Done Criteria、执行协议和文档
> 治理规则，供后续 `rally run` 会话作为稳定输入。日常进展、evidence、active milestone
> 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-09-07 用户决策）**：从父目标 M12 拆出。理由：① Web Components 是
> M12/Tier 2 明确列项且无 goal 认领；② Custom Elements 底子好（lit-html 级 e2e 已有
> `e2e_lit_library.rs` 391 行），补齐 upgrade/whenDefined/adoptedCallback 是增量；③
> slot 是现代组件库（lit、stencil 产物）的硬依赖，template/slot 补齐对真实网站兼容性
> 杠杆最大；④ 改动域（dom_bindings、js_dom_shim part03/04/05、dom crate）与
> rendering-compat 渲染流域**一期零重叠**（不做渲染级 composed tree）。
>
> **▶ 基线事实（2026-09-07 实测）**：
> - **Custom Elements（程度较高，非桩）**：
>   - Rust lifecycle 桥：`crates/engine/src/dom_bindings/custom_elements.rs`（318 行）——
>     `notify_connect_after_insert`/`notify_disconnect_after_remove`/`collect_custom_subtree`
>     （spec 触发序）/`notify_attribute_change`（R3267 S5d）
>   - quickjs 路径：`quickjs_dom_bindings.rs` L1928 起 customElements 五件套——define
>     （含 `_ceUpgradeSubtree` 同步升级既有元素 R149）/get/getName/whenDefined
>     （**同步立即 resolve 的 PoC 简化**）/upgrade（**显式 no-op L2049**）
>   - v8 native 路径：`dom_bindings/factories.rs` L103/L132（S5b upgrade 分支 R3265，
>     `Reflect.construct` 复用 host NodeId）+ `html_element.rs` L349 + `element.rs` L747
>   - 测试：`dom_bindings/tests_ce.rs`（164 行）+ `tests/integration/src/
>     e2e_web_components.rs`（374 行 8 用例）+ `e2e_lit_library.rs`（391 行）
>   - 已知缺口：`customElements.upgrade()` no-op、`whenDefined` 同步简化、
>     `adoptedCallback` 无、form-associated 无
> - **`<template>`（JS 层可用，DOM 层占位）**：`crates/dom/src/parser.rs` L302-303
>   `get_template_contents` 返回目标节点自身（**内容内联为文档树子节点，非独立 inert
>     fragment**）；querySelector 例外规则（document/mod.rs L1629-1644 R145）规避误命中；
>   JS 层 `part04.js` L710+ 有完整 content fragment 视图（nodeType 11、childNodes/
>   children/cloneNode/getElementById/querySelector——lit-html 管线可跑）
> - **`<slot>`（接近零）**：Rust DOM 有孤立数据结构（`dom/src/document/shadow.rs` 207 行
>   `assign_slot`/`resolve_slots`/`assigned_nodes`）但 engine 侧**零调用**；JS 层
>   HTMLSlotElement 仅类名映射表（part03.js L824/L954）、slotchange 仅事件名表
>   （part06.js L3232/L3275）；无 `el.slot` IDL 属性、无 `assignedSlot`、无
>   `assignedNodes()`、无 slotchange 派发、无 flattened tree
> - **Shadow DOM（JS API 层基础）**：attachShadow open/closed 校验 + 重复 attach 报错
>   （part05.js L3381 R2926）+ shadowRoot getter + shadow 树内 DOM/查询 + 事件
>   composed/getRootNode retarget 基础（part20/part21/part24 测试佐证）；**不落**
>   `Document::shadow_roots`（engine 非测试代码零调用 Rust `attach_shadow`）、**不进渲染
>   管线**（无 composed tree 布局/绘制）、无 `:host`/`::slotted`
> - **WPT 覆盖**：`custom-elements`/`shadow-dom`/`the-template-element` 目录在 wpt-data
>   中**全部不存在**（html/semantics/ 下无 scripting-1）；imported-tests.txt 仅 1 行
>   R149 注记

---

## Mission

以 **WPT `custom-elements` / `shadow-dom` / `the-template-element` 真实用例通过率为
验证标准**，补齐 Web Components 缺口：Custom Elements 剩余 API、template 真 inert
contents、slot 全链路（IDL 属性 → 分配 → slotchange → assignedNodes）。分阶段里程碑
校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **基线建立** | 三目录 window 可执行面导入 + 通过率基线（现有实现的真水平标定） |
| 中期 | **Custom Elements 收口 + template 真实化** | upgrade/whenDefined/adoptedCallback + parser 层真 template contents |
| 长期 | **slot 全链路** | el.slot/assignedSlot/assignedNodes/slotchange + Rust 分配机制接线 + flattened tree 查询 |

**关键约束**：所有验证必须基于从上游 WPT 仓库导入的**真实用例**（不允许手写 inline 用例
替代或充数）。WPT shadow-dom 目录大量用例依赖渲染级 composed tree（`:host`/`::slotted`
匹配、shadow 树绘制）——依赖渲染管线的部分入 skip list 并注明「等用户点名 Shadow DOM
深结构专项」，不充数也不误排除。

覆盖范围：

1. **Custom Elements 收口** — `customElements.upgrade` 真语义、`whenDefined` Promise
   真等待、`adoptedCallback`、`observedAttributes` 边缘、form-associated 评估
2. **template 真实化** — parser 层独立 inert DocumentFragment contents（contents 不在
   文档树、克隆/实例化语义）
3. **slot 全链路** — `HTMLSlotElement`（name、`el.slot` IDL、`assignedSlot`）、
   `assignedNodes({flatten})`、slotchange 事件派发、Rust `resolve_slots` 接线
4. **Shadow DOM JS 语义增补** — 基础面内的查询/事件 retarget 与 WPT 对齐（渲染级排除）

执行方式：**交替推进** — 每轮同时扩展 WPT 导入范围和修复发现的缺口。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| Custom Elements | dom_bindings/ + quickjs_dom_bindings.rs 剩余 API | 双路径（v8 native + quickjs）同步 |
| template | dom/parser.rs get_template_contents 真实化 + document 查询语义 | R145 例外规则随之收敛 |
| slot | dom/document/shadow.rs 接线 + js_dom_shim part03/04 slot API + slotchange | 复用 Rust 已有分配数据结构 |
| Shadow DOM JS 面 | 查询/retarget 语义增补（非渲染） | part01/part05 现有存储模型上增量 |
| WPT 基础设施 | custom-elements / shadow-dom / the-template-element 导入 | 复用 tests/wpt-runner + `make import-wpt`；新增 fetch 脚本 |
| 单元测试 | 每项修复带单测 + e2e（照 e2e_web_components.rs 模式） | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **Shadow DOM 渲染级 composed tree** — shadow 树参与样式/布局/绘制、`:host`/`::slotted`
  匹配、`@scope`：属渲染深结构，与 rendering-compat 域交界，**等用户点名后专项立项**
  （届时按 run-rules §9 与渲染流协调），本目标不碰
- **Constructable Stylesheets / adoptedStyleSheets** — 依赖 CSSOM 深化，记「待用户决策」
- **form-associated custom elements 完整语义** — 只做评估记录，实施视 form 管线现状
- **HTML imports** — 已废弃的规范，不做

### 依赖约束

- **与 rendering-compat 流边界（run-rules §9）**：本流改动域 = `crates/engine/src/
  dom_bindings/` + `quickjs_dom_bindings.rs`（CE 段）+ `js_dom_shim/part03/04/05.js`
  （slot/template/shadow 段）+ `crates/dom/src/`（parser/shadow/query）+ WPT 导入资产 +
  本 goal 控制面。渲染流域 crate 域（css-parser/style-system/layout-engine/
  render-foundation）**一期零重叠**；engine + dom 属共享面，碰前 `git log --since=
  "14 days ago" -- crates/engine/ crates/dom/` 核对。
- **与 event-loop-spec 流**：同属 engine 共享大文件池（part01.js 是其主力，本流主力
  part03/04/05）——无直接共享段，但碰 `part01.js`（如 slotchange 事件名联动）前互相核对。
- **与已归档 js-dom goal**：其 native CE 路径（escape-hatch 遗产）是本流 Custom Elements
  收口的既有基座，只读消费不重建。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 用例导入与通过率基线

- [ ] 从上游 WPT 导入 `custom-elements` / `shadow-dom` / `the-template-element` window
      可执行面真实用例；依赖渲染级 composed tree 的用例入 skip list 并注明
- [ ] 新增 fetch 脚本（照 indexeddb/cache-storage 先例）
- [ ] 建立分类通过率报告（文本 + JSON），记录基线
- [ ] 每项修复的 driving WPT 用例经常驻断言集并记入账本（`imported-testharness.txt`）
- [ ] 通过率报告持久化到 `docs/goal/web-components/evidence/`，历史可追溯

### DC-2: Custom Elements 收口

- [ ] `customElements.upgrade(node)` 真语义（替换 no-op）
- [ ] `whenDefined` 真 Promise 等待（define 前调用不假 resolve）
- [ ] `adoptedCallback` 派发路径（document.adoptNode / appendChild 跨文档）
- [ ] v8 native 与 quickjs 双路径行为一致（同一测试面双跑）

### DC-3: template 真实化

- [ ] parser 层 `<template>` contents 为独立 inert DocumentFragment（不在文档树）
- [ ] `content` 属性真实 fragment（克隆/实例化语义；R145 querySelector 例外规则收敛）
- [ ] template 内资源不加载、脚本不执行（inert 语义，WPT 为准）

### DC-4: slot 全链路

- [ ] `HTMLSlotElement` 接线：`name`、`el.slot` IDL 属性、`assignedSlot`
- [ ] `assignedNodes({flatten})` 真语义（Rust `resolve_slots` 接线）
- [ ] `slotchange` 事件派发（分配变化时，microtask 时序）
- [ ] shadow 树内 slot 分配 + light DOM fallback 内容的基础 flattened tree 查询

### DC-5: 测试与质量不可退让

- [ ] `make test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + driving WPT 用例资产化
- [ ] `make reftest` 无回归（dom 结构变更的渲染面守卫）

---

## 活跃里程碑

### M1 — WPT 基线建立 + Custom Elements 收口

**目标**：三目录导入 + 基线；upgrade/whenDefined/adoptedCallback 三缺口修齐（双路径）。

**切片建议**：
1. fetch 脚本 + 用例导入 + 分类通过率基线（零源码改动，纯资产；现有实现的真水平标定）
2. `customElements.upgrade` 真语义 + `whenDefined` Promise 真等待
3. `adoptedCallback` 派发路径 + 失败聚类

### M2 — template 真实化

**目标**：parser 层 inert DocumentFragment contents + content 属性真实化 + R145 规则收敛。

### M3 — slot 全链路 + 收尾

**目标**：slot API 接线（IDL → 分配 → slotchange → assignedNodes）+ shadow-dom JS 语义
增修 → DC 全满足判定；Shadow DOM 渲染级专项的决策材料整理（若 WPT 基线显示缺口集中于此）。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~5 全部满足；验证基于上游真实 WPT 用例（无内建 inline 充数）；
`cargo build` + `make test` + `cargo clippy` 全通过；master.md 内部自洽，archive 已建立。
渲染级 composed tree 按排除条款明确记录为「等用户点名专项」，不算未满足 DC。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**CE 双路径、template parser、slot 数据结构的确切差距
2. **自主导入** WPT 三目录用例，扩大覆盖范围
3. **自主修复**，不等待用户逐步指令；每修 net≥0 即 land
4. **自主添加测试**，新修复必须有对应单元测试 + WPT 用例资产化
5. **自主验证**：`make test` + `make reftest` + clippy + WPT 通过率
6. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：WPT 驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项（Shadow DOM 渲染级、adoptedStyleSheets）记「待用户决策」
   清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：engine/dom 共享面碰前 `git log` 核对；part01.js 与 event-loop-spec 流
   互相核对。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。当作当前任务的一部分修复，直到稳定可重复。
2. **用例失败分析**：每个失败 case 必须分析根因（API 缺失？DOM 层占位？接线断？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/web-components/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/web-components/archive/`：只追加不修改。
- **证据区域** `docs/goal/web-components/evidence/`：通过率报告、失败分析等验证证据，
  持续追加。
