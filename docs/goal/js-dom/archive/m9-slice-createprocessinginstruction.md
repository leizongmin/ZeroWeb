# M4 R9 切片 — document.createProcessingInstruction polyfill 桥接 + DOMException identity 对等

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**前置**: R8（testharness 本地 .js 内联 + 基线真实化）
**commit**: 见 `git log`（feat(js-dom): polyfill createProcessingInstruction + DOMException identity parity）

## 背景

R8 真实化基线后（polyfill 37.82% / native 37.59%），失败聚类分析：`document.createProcessingInstruction is not a function` 占 is-not-a-function 284 subtest 的 161（57%），是最大单一可修缺口。

R7 已交付 **native** createProcessingInstruction API（dom_bindings factories.rs：target/data/nodeName/校验）。但核实发现：**用例侧 `document` 始终是 polyfill document（part06.js 的 globalThis.document），即使 ZW_NATIVE_DOM=1**。native document template（R7 装的 createProcessingInstruction）用例访问不到。故 polyfill document 必须自己实现 createProcessingInstruction，双路径用例才能通过。

## 改动（7 文件）

### host 桥（Rust）

- **`js_dom_bridge.rs`**：`DomMutation::CreateProcessingInstruction { handle, target, data }` 新变体（镜像 CreateComment）；`apply_dom_mutations` 加 arm → `doc.create_processing_instruction(target, data)`；`query_inner_html_from_mutations` 两处加 PI 分支（data 作 textContent）。
- **`js_dom_bridge/callbacks.rs`**：注册 `__zw_create_processing_instruction(target, data)` callback（镜像 `__zw_create_comment`）。

### polyfill shim（JS）

- **part06.js**：document.createProcessingInstruction(target, data) 方法——spec 校验（target 合法 Name production、data 不含 `?>`，违则抛 InvalidCharacterError）+ 合法经 callback + `_piHandles` 标识。
- **part01.js**：`_piHandles` 声明（存 {target, data}）。
- **part04.js**：`_wrapHandle` 识别 PI——nodeType=7、nodeName=target、target/data/nodeValue/length getter。
- **part03.js**：ProcessingInstruction 构造器占位（挂 Node.prototype；instanceof 仍 false，属 R8 instanceof 缺口）。

### DOMException identity 对等修复（顺带修 R3 既存对等 bug）

createElement（R3）+ PI 校验抛错从裸 `new DOMException(...)` 改 `new (globalThis.DOMException)(...)`（R6 identity 教训）。原因：native_dom 叠加路径下，词法作用域的 part01b DOMException ≠ 全局原生 DOMException，致 testharness `assert_throws_dom` 报 "wrong global"。

### 测试

- **`js_dom_bridge_tests/part07.rs`**：`test_create_processing_instruction_r9`（nodeType=7 / nodeName=target / target/data 读回 / mutation 记录 / spec 校验抛 InvalidCharacterError / 构造器占位）。

## 基线结果（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R8 | R9 | Δ |
|------|----|----|---|
| polyfill | 37.82% | 38.03% | +0.21pp |
| native | 37.59% | 37.81% | +0.22pp |

双路径对等差 0.22pp（≤ R8 的 0.23pp）。PI 用例双路径均 1P/11F → 6P/6F。

## 验证

engine v8 2076 / quickjs 1407 单测全绿；wpt-runner v8 168 / quickjs 103 全绿；fmt + clippy（v8 + quickjs 双矩阵）零警告。

## 关键决策

1. **spec 校验放 JS 侧（shim）而非 bridge callback**：polyfill 桥 callback 返字符串，难抛 JS 异常；JS 侧校验（复用 R3 `_zwIsValidQualifiedName`）与 createElement 一致，调用点同步抛。
2. **instanceof 不在本切片范围**：polyfill Proxy 节点 instanceof 需 getPrototypeOf 按节点类型返原型，是更深结构（R8 instanceof 89 块）。本切片加 ProcessingInstruction 构造器占位（不抛 TypeError），instanceof 留下轮。
3. **顺带修 R3 createElement DOMException identity**：同一 wrong-global bug，R3 在 R6 之前未应用教训；本切片一并修复（无害，为下轮 createElement 对等铺路）。

## 下一步

- instanceof 原型链（89 块，解 PI valid 3 个 + Element/HTMLElement）。
- iframe.contentDocument（createElementNS/case 等用例大头 ~390 subtest，html-compat 域，待评估）。
- 主线 M1 L2（polyfill-live 合一）。
