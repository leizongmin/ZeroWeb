# 归档：M4 切片 7 / R7 — fetch .js 依赖（基线真实化）+ native createProcessingInstruction API

**日期**: 2026-08-14
**轮次**: R7
**Milestone**: M4（WPT dom 上游基线 + 按聚类驱动修复）
**切片**: M4 切片 7（.js 依赖补齐 + native PI API）
**基线**: `8d7a151b`（R6 land 后）

## 切片目标

R6 native 追平 polyfill 后，准备做 createProcessingInstruction（44 失败）。核实发现用例引用的 `.js` 测试体大量缺失（fetch 只拉了 .html），导致基线虚高。本轮：① 补齐 .js 依赖，基线真实化；② 实现 native PI API。

## 实现产物

### 1. fetch-dom-subset.sh 补齐 .js 依赖（基线真实化）
**`tests/wpt-runner/scripts/fetch-dom-subset.sh`**：
- `fetch_dir_html` 同时拉 `.html` + `.js`（用例引用同目录 .js 测试体，如 Document-createProcessingInstruction.js）
- 加拉 dom 根共享 `.js`（constants.js / common.js）+ `resources/testharnessreport.js`

拉到 28 个 dom/nodes .js + dom 根共享 .js。

### 2. native createProcessingInstruction API
- **`factories.rs`** `native_create_processing_instruction_invoke`：spec 校验（target 须 Name production → 复用 `is_valid_qualified_name`；data 不得含 `?>`）→ 非法抛 InvalidCharacterError；合法 `d.create_processing_instruction` → native 对象
- **`document.rs`** 注册 `createProcessingInstruction` 方法
- **`node.rs`** PI 专用 getter：`native_pi_target_getter`（PI.target）、`native_pi_data_getter`（PI.data）；修正 `node_name` PI 分支（`#processing-instruction` → PI.target，spec `dom-processinginstruction-target`）
- **`mod.rs`** Element 模板注册 `target`/`data` accessor（仅 PI 返值，非 PI → undefined）
- **`tests_dom_api.rs`** `native_document_create_processing_instruction_r7`（valid target/data/nodeName/nodeType + invalid target/data 抛 InvalidCharacterError）

## 验证证据

| 矩阵 | 命令 | 结果 |
|------|------|------|
| dom_bindings 全测（v8） | `cargo test -p zero-engine --features v8 --lib dom_bindings` | ✅ 199 passed（+1 PI 测试 +1 identity） |
| clippy v8 + quickjs | `cargo clippy ...` | ✅ 双矩阵零警告 |
| **polyfill dom/nodes 基线** | `make testharness-dom` | 56.45% → **51.12%**（真实化） |
| **native dom/nodes 基线** | `make testharness-dom-native` | 56.08% → **50.79%**（真实化，与 polyfill 对等差 0.33pp） |

## 关键洞察：基线真实化（非回归）

R7 前基线（56.45%/56.08%）**虚高**——因用例引用的 .js 测试体缺失，runner 加载失败→用例跳过/错误处理（不计为真实失败）。补齐 .js 后用例真跑，暴露真实 API gap（createProcessingInstruction/instanceof/attr_is 等）→ 基线降至 51.12%/50.79%。

**这是基线诚实化，非回归**：双路径对等（差 0.33pp）维持，且暴露的 gap 是真实可修的（PI/instanceof/attr helper）。此前 56% 是假象（跳过的用例不算分母里的失败）。

## 未完成（R7 发现，留下轮）

**PI 用例超时**：Document-createProcessingInstruction.html 外层 `test()` 嵌套多个 `test()` + `pi instanceof ProcessingInstruction`（构造器未装）→ testharness completion callback 未调 → 超时。native PI API（target/data/nodeName/校验）已就位，但需 ProcessingInstruction 构造器（instanceof + 解嵌套 test 超时）。

## 下一步（M4 切片 8 候选）

按剩余 ROI：① ProcessingInstruction 构造器（解 PI instanceof + 超时）② instanceof HTMLElement/Element 原型链（~88）③ attr helper（attr_is）④ 扩 dom/events。
