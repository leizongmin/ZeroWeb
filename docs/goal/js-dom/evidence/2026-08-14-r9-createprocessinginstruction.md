# R9 — document.createProcessingInstruction polyfill 桥接 + DOMException identity 对等（M4 / DC-3）

**日期**: 2026-08-14
**轮次**: R9
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**commit**: 见 `git log`（feat(js-dom): polyfill createProcessingInstruction + DOMException identity parity）

## 背景

R8 真实化基线后，失败聚类显示 `document.createProcessingInstruction is not a function` 占 is-not-a-function 284 subtest 的 161（57%，最大单一可修缺口）。R7 已交付 **native** createProcessingInstruction API（dom_bindings factories.rs），但 **polyfill document（part06.js）无此方法**——而用例侧 `document` 始终是 polyfill document（即使 ZW_NATIVE_DOM=1，顶层 `globalThis.document` 仍是 polyfill shim 装的），故双路径用例都报 is not a function。

Document-createProcessingInstruction.html 用例（1P/11F）：8 个 invalid（assert_throws_dom INVALID_CHARACTER_ERR）+ 3 个 valid（target/data/ownerDocument/instanceof）。

## 改动

**polyfill document.createProcessingInstruction 方法 + host 桥 + DOMException identity 修复**（7 文件）：

1. **`js_dom_bridge.rs`**：新增 `DomMutation::CreateProcessingInstruction { handle, target, data }` 变体；`apply_dom_mutations` 加 arm（`doc.create_processing_instruction(target, data)`）；`query_inner_html_from_mutations` 等查询函数加 PI 分支（data 作 textContent 读回）。
2. **`js_dom_bridge/callbacks.rs`**：注册 `__zw_create_processing_instruction(target, data)` callback（镜像 `__zw_create_comment`，push CreateProcessingInstruction mutation）。
3. **`js_dom_shim/part06.js`**：document 加 `createProcessingInstruction(target, data)` 方法——spec 校验（target 须合法 Name production、data 不得含 `?>`，违则抛 InvalidCharacterError）+ 合法经 `__zw_create_processing_instruction` + `_piHandles` 标识。
4. **`js_dom_shim/part01.js`**：声明 `_piHandles`（存 {target, data}）。
5. **`js_dom_shim/part04.js`**：`_wrapHandle` 识别 PI（`_piHandles`）——nodeType=7、nodeName=target、target/data/nodeValue/length getter。
6. **`js_dom_shim/part03.js`**：ProcessingInstruction 构造器占位（挂 Node.prototype；instanceof 仍 false，记入 R8 instanceof 89 块缺口）。
7. **DOMException identity 对等修复**（顺带修 R3 createElement 既存对等 bug）：createElement/PI 校验抛错从裸 `new DOMException(...)` 改用 `new (globalThis.DOMException)(...)`（R6 identity 教训：native_dom 叠加路径下 = 原生 DOMException，避免 assert_throws_dom "wrong global"）。
8. **`js_dom_bridge_tests/part07.rs`**：新增 `test_create_processing_instruction_r9`（nodeType=7 / nodeName=target / target/data 读回 / mutation 记录 / spec 校验抛 InvalidCharacterError / 构造器占位）。

## 基线结果（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R8 | R9 | Δ |
|------|----|----|---|
| polyfill | 37.82% | **38.03%** | +0.21pp |
| native | 37.59% | **37.81%** | +0.22pp |

**双路径对等**: 差 0.22pp（polyfill 38.03% vs native 37.81%），≤ R8 的 0.23pp。

**PI 用例**: polyfill 1P/11F → **6P/6F**（+5）；native 1P/11F → **6P/6F**（+5，DOMException identity 修复后双路径对等）。

**subtest 总数**: 4490 → 4502（+12，此前 is-not-a-function 中断的用例现继续跑出更多 subtest，部分新 pass）。

完整 JSON 快照: `2026-08-14-r9-dom-nodes-polyfill.json` / `2026-08-14-r9-dom-nodes-native.json`。

## 为何 PI 双路径对等（关键发现）

用例侧 `document` 始终是 polyfill document（part06.js），即使 ZW_NATIVE_DOM=1。故 PI 必须在 polyfill document 实现才能让用例通过。native createProcessingInstruction（R7 dom_bindings）装在 native document template，但用例访问不到该 document。

DOMException identity：native 路径下 shim 用裸 `new DOMException(...)` 抛的异常，testharness `assert_throws_dom` 报 "wrong global"（词法作用域 part01b DOMException ≠ 全局 native DOMException）。改用 `globalThis.DOMException` 修复（R6 教训）。

## 验证

| 门禁 | 结果 |
|------|------|
| engine v8 单测 | ✅ 2076 passed（含新 PI 测试） |
| engine quickjs 单测 | ✅ 1407 passed |
| wpt-runner v8 单测 | ✅ 168 passed |
| wpt-runner quickjs 单测 | ✅ 103 passed |
| fmt / clippy（v8 + quickjs 双矩阵） | ✅ 零警告 |

## 下一步

- PI valid 那 3 个（instanceof ProcessingInstruction/Node）属 R8 instanceof 89 块原型链缺口，独立切片。
- 聚类 ROI：instanceof 原型链（89）/ iframe.contentDocument（createElementNS/case 等用例大头，390 subtest）/ cloneNode（10）。
