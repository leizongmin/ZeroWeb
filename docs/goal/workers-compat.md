# Worker 并发兼容 — Dedicated Worker

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active（已立项，未启动——启动顺序由用户点名）
**执行模式**: WPT 驱动（上游 workers corpus 为验收标尺）+ 运行时落地；
照 event-loop-spec（微任务机制先例）/ script-sandbox（隔离实例先例）打法
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——并发面）

> **说明**
> 本文档是 ZeroWeb「Worker 并发兼容」专项目标执行契约。现代前端框架与重计算场景
> 依赖 dedicated worker；本目标把 Worker 从「构造器桩」推进到「真实运行时 + 消息
> 往返语义」。
>
> **▶ 拆分动机（2026-09-12 用户决策，四 goal 同批立项之一）**：① 并发面是真实
> 浏览器必备能力，当前完全缺位；② 消息底座现成——MessageChannel/MessagePort +
> structuredClone 已在 zero-web P1a 落地（shim 内 MessagePort 22 / structuredClone
> 27 处），本 goal 只差隔离执行实例；③ 与渲染流零 crate 重叠，可独立并行。
>
> **▶ 基线事实（2026-09-12 实测）**：
> - **Worker 构造器桩已存在**：`crates/engine/src/dom_bridge.rs:1839`（"Provides
>     the Dedicated Worker constructor"）——part14 测试实证 `new Worker(url)` 返对象、
>     postMessage/terminate 不抛，**但无真实子执行实例**（桩语义）
> - **消息底座现成**：MessageChannel/MessagePort（P1a，postMessage 目标源/targetOrigin
>     语义已落）+ structuredClone（27 处）——worker 往返序列化不缺
> - **隔离执行先例**：script-sandbox（V8/QuickJS feature gate + 宿主桥接）与
>     event-loop-spec（微任务机制，已归档）提供运行时组织范式
> - **运行时缺口**：独立 JS 隔离实例（v8 isolate / quickjs ctx per worker）+ 独立
>     微任务循环 + worker 全局作用域面（self/DedicatedWorkerGlobalScope/importScripts）
> - **WPT corpora**：`workers/` `dedicated-workers/`（testharness；modules 面启动时评估）

---

## Mission

以 **WPT workers 真实用例为验收标准**，落地 dedicated worker 真实运行时
（独立隔离实例 + 事件循环 + 全局作用域）与 postMessage 往返语义，使「重计算/
后台任务」类页面能力可用。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **classic 脚本先行**：module worker 类型启动时评估，不承诺
3. **运行时组织复用先例**：隔离实例照 script-sandbox 边界风格，微任务照
   event-loop-spec 先例，不另起炉灶

覆盖范围：

1. **Worker 生命周期** — new Worker(url)/terminate/worker 全局作用域
   （self/DedicatedWorkerGlobalScope）/onerror/onmessageerror
2. **脚本执行** — classic worker 脚本加载与执行（importScripts 语义）；
   module worker 评估记账
3. **消息往返** — postMessage/structuredClone 往返 + transferable
   （ArrayBuffer transfer 所有权语义）+ MessageChannel 跨端口

### 排除（明确不在范围内）

- **SharedWorker** —— 无页面共享场景地基，挂账
- **Service Worker 页面侧扩展** —— service-workers goal 已归档收口（storage 侧）；
  其拦截/生命周期扩展另行立项
- **Worker 内 DOM/Blob URL 全量** —— 规范即无 DOM；URL.createObjectURL 跨实例
  语义如触缺口记账
- **nested worker** —— 二期挂账

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine/dom_bridge | Worker 构造器从桩升级为真实实例派发 | 现 dom_bridge.rs:1839 桩位 |
| page-runtime | worker 实例归属与消息契约 | **先确认 zero-page-runtime 契约再动** |
| script-sandbox | 隔离实例组织范式复用（feature gate 风格） | 不改其对外边界 |
| WPT 资产 | workers/dedicated-workers 子集导入 | fetch 脚本 + 账本 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 zero-page-runtime**：页面运行时统一契约——worker 实例归属/消息通道须先对
  契约协商（WPT/TabWorker/renderer 面），确认后再动协议，不单方面改。
- **与 script-sandbox / wasm-sandbox**：隔离风格复用、边界不碰（其宿主桥接对外
  API 不改）。
- **与 event-loop-spec（已归档）**：微任务/事件循环先例复用，遗留问题记账回流。
- **与渲染流（rendering-compat 等）**：无共享 crate 面；worker 不触
  style-system/layout-engine/painter。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 导入与基线

- [ ] workers/dedicated-workers corpus window 可执行子集导入（fetch 脚本 + 通道 +
      imported 账本）
- [ ] 分类通过率基线落 evidence/，并回填 docs/compat/trends/wpt-suites.csv planned 行

### DC-2: Worker 运行时

- [ ] 独立隔离实例 + 独立事件循环 + worker 全局作用域最小面
      （self/navigator/location/onerror）
- [ ] classic 脚本加载执行 + importScripts；module worker 评估结论记账

### DC-3: 消息往返

- [ ] postMessage/structuredClone 双向往返 + MessageChannel 跨端口
- [ ] transferable（ArrayBuffer transfer 中断所有权）语义

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿零失败 + clippy `-D warnings` + fmt 干净（v8 + quickjs 双矩阵
      评估——隔离实现须两 JS 引擎下行为一致或差异记账）
- [ ] 每语义切片带单测/集成测试；`make reftest` 零回归

---

## 活跃里程碑

### M1 — WPT 导入与基线

**目标**：两 corpus fetch 脚本 + 导入 + 基线（纯资产零源码改动）；corpus fetch 用编号脚本 `tests/wpt-runner/scripts/goals/70-workers-compat.sh`（索引 README；幂等续拉，GitHub API 限流时部分目录跳过属预期）。

### M2 — Worker 运行时

**目标**：隔离实例 + 事件循环 + 全局作用域 + classic 脚本执行。

### M3 — 消息往返

**目标**：postMessage/structuredClone/transferable/MessageChannel 语义。

### M4 — 收口

**目标**：DC 逐项判定 + SharedWorker/nested/module worker 挂账定稿。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方「DONE 允许条件」 |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；验证基于上游真实 WPT 用例；`cargo build` +
`make test` + `cargo clippy` 全过（JS 引擎双矩阵差异如实记账）；master.md 自洽，
evidence 持久化；SharedWorker/nested/module 挂账定稿。

---

## Execution Protocol

### 自主执行原则

1. **基线先行**：M1 纯资产切片零源码改动
2. **契约先行**：M2 动协议面前先与 zero-page-runtime 对契约（§9 碰撞管理）
3. **逐簇收敛**：WPT 失败聚类 → 逐簇修齐 → 账本更新 → suites CSV 回填

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮
2. **双引擎差异**：v8/quickjs 行为不一致时以 spec 仲裁，差异记账不静默
3. **worker 内网络/存储子集请求**：如触 fetch/OPFS 跨域面，记账到对应 goal，不越界

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。
  仅在目标本身实质变化时修改；每轮执行不重写。
- **运行时控制平面** `docs/goal/workers-compat/master.md`：当前真实状态唯一控制面板。
- **归档区域** `docs/goal/workers-compat/archive/`：只追加不修改。
- **证据区域** `docs/goal/workers-compat/evidence/`：WPT 基线/修齐账本，持续追加。
