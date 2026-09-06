# 页面 WASM 深化 — WPT 驱动的 WebAssembly 页面可用性目标

**版本**: v1.0
**日期**: 2026-09-07
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（M12「页面 WASM 可以加载执行并与 JS 互操作」未勾项）

> **说明**
> 本文档是 ZeroWeb「页面 WASM 深化」专项目标执行契约。页面 WASM 桥**已有闭环底座**（JS
> polyfill → `__WASM_BRIDGE__` 协议 → webview host 经 zero-wasm-sandbox 编译/实例化/调用 →
> 结果回注），但导出面、参数类型、流式实例化均为 stub 或受限子集，且上游 WPT 用例覆盖为零。
> 目标是以 WPT 真实用例为标准深化到可用水平。本文定义 Mission、边界、Done Criteria、执行协议
> 和文档治理规则，供后续 `rally run` 会话作为稳定输入。日常进展、evidence、active milestone
> 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-09-07 用户决策）**：从父目标 M12 拆出。理由：① M12「WASM 支持（页面
> WASM 与 JS 互操作）」是 Done Criteria 中唯一无任何 goal 认领的未勾大项；② 底座已闭环
> （webview.rs 398 行 wasm_bridge.rs 测试 15 个全绿），深化路径清晰，不是从零起步；③ 改动域
> （wasm-sandbox、webview wasm 段、dom_bridge.rs WebAssembly polyfill 段）与 rendering-compat
> 渲染流域**零重叠**，可安全并行；④ WPT wasm 用例目录在 wpt-data 中为零，导入面完全空白 =
> 清晰的验收起点。
>
> **▶ 基线事实（2026-09-07 实测）**：
> - **Rust 执行底座（完整）**：`crates/wasm-sandbox/` 1392 行——wasmtime/wasmi/stub 三后端
>   feature gate；`WasmSandbox::compile` → `WasmModule::instantiate` → `WasmInstance::call`
>   + `read_memory`/`write_memory`/`set_fuel`；`WasmValue` 枚举（I32/I64/F32/F64…）、
>   `LinkerConfig`（host function 注入）、`SandboxConfig`（fuel）。
> - **JS polyfill（受限）**：`crates/engine/src/dom_bridge.rs` 行 361–530
>   `globalThis.WebAssembly`——`compile`/`instantiate`/`instantiateStreaming`（回退
>   arrayBuffer）/`validate`（仅 `\0asm` 魔术字节）；实例导出是 **stub**（`exports.memory.buffer`
>   固定 64KB、`grow` 恒返 1、`__host_backed__:false`）；调用协议 = `_callQueue` push →
>   host 排空执行 → `__wasm_call_results__` 回注；参数映射**仅 I32**。
> - **host 执行（闭环）**：`crates/webview/src/webview.rs` 行 4530–4611 探测
>   `__WASM_BRIDGE__:`/`__WASM_COMPILE__:` → `WasmSandbox::new()` 编译/实例化；行 4795–4840
>   排空 `_callQueue`；`crates/webview/src/tests/wasm_bridge.rs`（398 行 / 15 测试：加减、
>   负参、大参、invalid bytes、missing export、`_start` 自动执行等）。
> - **架构注记**：engine 自身不依赖 zero-wasm-sandbox（桥在 webview 层经协议字符串）；
>   `imported-tests.txt` 中 wasm 命中为零；wpt-data 无 wasm 类别目录、scripts 无 fetch-wasm
>   脚本。

---

## Mission

以 **WPT WebAssembly 真实用例（js-benchmark 之外的可执行面）通过率为验证标准**，把页面
WASM 从「能跑一个 add 函数的最小桥」深化到「典型 wasm 模块可用」水平：真实导出面、参数
类型扩展、流式实例化、链接与 import 语义。分阶段里程碑校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **基线建立** | 导入上游 WASM JS API 面可执行用例 + 通过率基线（当前 stub 导出下的表现即验收清单） |
| 中期 | **核心通路** | 导出面真实化（memory 真实映射、多类型参数）、instantiate importObject 语义 |
| 长期 | **互操作收口** | host function（JS 函数作 wasm import）、流式编译、Module/Instance 缓存语义 |

**关键约束**：所有验证必须基于从上游 WPT 仓库导入的**真实用例**（不允许手写 inline 用例
替代或充数）。上游 wasm 用例分两类：`wasm/jsapi`（JS API 语义，window 环境可执行）与
`wasm/spec`（引擎内核语义，多为多平台 harness）——本目标只收 `wasm/jsapi` window 可执行面，
`wasm/spec` 内核类入 skip list 并注明（内核正确性由 wasmtime/wasmi 上游保证，不在自建范围）。

覆盖范围：

1. **导出面真实化** — `Instance.exports` 真实函数表、`Memory.buffer` 真实映射（grow 同步）、
   Global/Table 导出
2. **参数与返回类型** — I32/I64/F32/F64 全映射（当前仅 I32）
3. **instantiate/compile 语义** — importObject 链接、LinkError/CompileError/RuntimeError
   分类、validate 完整校验
4. **host function** — JS 函数作 wasm import（LinkerConfig 已有底座）
5. **流式编译** — `instantiateStreaming`/`compileStreaming` 走真实 Response body（非回退）

执行方式：**交替推进** — 每轮同时扩展 WPT 导入范围和深化桥接面。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| JS polyfill 深化 | `dom_bridge.rs` WebAssembly 段：导出面/类型/错误分类 | 保持 `__WASM_BRIDGE__` 协议演进 |
| host 执行深化 | `webview.rs` wasm 段：类型转换全映射、import 链接、导出查询 | 消费 zero-wasm-sandbox 既有 API，不重建 |
| wasm-sandbox 补强 | 语义缺口（如 LinkerConfig 实际接线、fuel 默认策略） | 仅补页面互操作所需 |
| WPT 基础设施 | `wasm/jsapi` 用例导入、testharness 执行、通过率报告 | 复用 tests/wpt-runner + `make import-wpt`；新增 `scripts/fetch-wasm-subset.sh` |
| 单元测试 | 每项修复带单测（wasm-sandbox 级 + webview bridge 级） | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **`wasm/spec` 内核语义用例** — wasm 引擎内核正确性由 wasmtime/wasmi 上游保证；skip list 注明
- **非页面 WASM（插件沙箱）** — wasm-sandbox 在 script-sandbox/扩展面的应用不在本目标
- **WASM GC / Component Model / threads / SIMD 提案** — Tier 3+，记入「待用户决策」
- **WASM ESM integration**（`<script type="module">` 加载 wasm）— 依赖 ES Modules 深化，排除

### 依赖约束

- **与 rendering-compat 流边界（run-rules §9）**：本流改动域 = `crates/wasm-sandbox` +
  `crates/webview/src/webview.rs` wasm 段 + `crates/engine/src/dom_bridge.rs`
  WebAssembly polyfill 段 + WPT 导入资产 + 本 goal 控制面。渲染流域 crate 域**零重叠**；
  `dom_bridge.rs`/`webview.rs` 属共享大文件——开工前 `git log --since="14 days ago" --
  crates/engine/src/dom_bridge.rs crates/webview/` 核对，有活跃编辑则先做零碰撞面
  （wasm-sandbox 单测、WPT 导入）。
- **与 event-loop-spec 流**：无共享面（该流改 js_dom_shim/part01.js 与 script-sandbox
  事件循环；本流的 WebAssembly 段在 dom_bridge.rs，不重叠）。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 用例导入与通过率基线

- [ ] 从上游 WPT 仓库 `wasm/jsapi` 导入 window 环境可执行真实用例；`wasm/spec` 内核类
      入 skip list 并注明
- [ ] 新增 `scripts/fetch-wasm-subset.sh`（照 `fetch-cache-storage-window-subset.sh` 模式）
- [ ] 建立分类通过率报告（文本 + JSON），记录基线
- [ ] 每项修复的 driving WPT 用例经常驻断言集并记入账本（`imported-testharness.txt`）
- [ ] 通过率报告持久化到 `docs/goal/page-wasm/evidence/`，历史可追溯

### DC-2: 导出面与类型真实化

- [ ] `Instance.exports` 真实函数表（当前 stub `__host_backed__:false` 消除）
- [ ] `Memory.buffer` 真实映射（grow 后 JS 视图同步；不再固定 64KB）
- [ ] I32/I64/F32/F64 参数与返回值全映射（`WasmValue` 已有枚举，桥接层补齐转换）
- [ ] Global/Table 导出可查询（`get_global_export`/`has_table` 已有，接 JS 面）

### DC-3: 实例化语义

- [ ] `instantiate` importObject 链接语义 + LinkError 分类（import 缺失/签名不匹配）
- [ ] `validate` 从魔术字节升级为真实校验（或明确记录 wasmtime validate 接线）
- [ ] `compileStreaming`/`instantiateStreaming` 真实 Response body 路径（保持 arrayBuffer 回退为兼容路径）

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + driving WPT 用例资产化

---

## 活跃里程碑

### M1 — WPT 基线建立 + 类型扩展

**目标**：导入 `wasm/jsapi` 用例、记录 stub 基线；参数/返回类型 I64/F32/F64 全映射 +
导出函数表真实化。

**切片建议**：
1. `fetch-wasm-subset.sh` + 用例导入 + 分类通过率报告（零源码改动，纯资产）
2. `WasmValue` 桥接层类型转换全映射（webview wasm 段）
3. exports 函数表真实化（`exports()` API 接 JS 面）+ 失败聚类

### M2 — Memory/Global/Table + 实例化语义

**目标**：Memory 真实 buffer 映射 + grow 同步、Global/Table 导出、importObject 链接 +
错误分类；每步 kill-switch + A/B 零回归。

### M3 — host function + 流式 + 收尾

**目标**：JS 函数作 wasm import、streaming 真实路径、validate 接线、剩余用例修复 →
DC 全满足判定。

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

1. **自主探索**当前桥接面缺口的确切差距（stub 导出 vs WPT 期望）
2. **自主导入** WPT `wasm/jsapi` 用例，扩大覆盖范围
3. **自主运行**用例，分析失败原因（导出 stub？类型？链接？错误分类？）
4. **自主修复**，不等待用户逐步指令；每修 net≥0 即 land
5. **自主添加测试**，新修复必须有对应单元测试 + WPT 用例资产化
6. **自主验证**：`make test` + clippy + WPT 通过率确认修复有效
7. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：WPT 驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：碰 `dom_bridge.rs` / `webview.rs`（共享大文件）前先 `git log` 核对；
   有活跃编辑则转零碰撞面（wasm-sandbox 单测、WPT 导入）。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。当作当前任务的一部分修复，直到稳定可重复。
2. **用例失败分析**：每个失败 case 必须分析根因（导出 stub？类型映射？链接语义？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/page-wasm/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/page-wasm/archive/`：只追加不修改。
- **证据区域** `docs/goal/page-wasm/evidence/`：通过率报告、失败分析等验证证据，持续追加。
