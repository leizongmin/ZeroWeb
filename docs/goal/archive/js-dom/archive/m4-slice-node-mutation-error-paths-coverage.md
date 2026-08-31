# M4 R36 — dom_bindings coverage 提升（node.rs mutation 错误路径 + aria/role 反射）

**日期**: 2026-08-14
**轮次**: R36
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复）/ DC-4 coverage
**前置**: R35（eventPhase 反映 dispatch 阶段）
**状态**: ✅ 已 land（纯测试新增，零回归）

---

## 背景

R35 后评估下一切片：Event-dispatch 系列 document/window 入 dispatch chain 诊断为**深结构**（document/window/html listener key 三合一架构简化，part06.js:152/1265/1809 大量逻辑依赖 html key 共享，改之波及 postMessage/onerror/inline handler），按深结构护栏跳过。master.md 剩余聚类 ③「native namespaceURI getter 独立化」核实**已完成**（element.rs:61 native_namespace_uri_getter 闭合 R3163，master.md 过时记录需勘误）。

转 DC-4 coverage 提升（干净、低风险、纯测试新增、零碰撞面）。cargo-llvm-cov 逐文件明细：node.rs 91.6%（剩 49 行）/ element.rs 92.7%（剩 52 行）绝对提升空间最大。lcov 分析定位可达未覆盖分支：node.rs 的 appendChild/insertBefore cycle + removeChild/replaceChild NotFoundError 错误分支（`Some(Err(e))` → dom_error_exception → throw_dom_exception，R4 已修但单测只覆盖成功路径）。

## 实现

### R36 单测 1：node.rs mutation 错误路径（`native_node_mutation_error_paths_r36`）

5 个错误路径断言（经 try/catch 捕获 DOMException name）：
- `appendChild(parent)` cycle → HierarchyRequestError（WouldCreateCycle）
- `appendChild(ancestor)` cycle → HierarchyRequestError
- `removeChild(非 child)` → NotFoundError（NotAChild）
- `replaceChild(new, 非 child oldChild)` → NotFoundError
- `insertBefore(self, ref)` cycle → HierarchyRequestError

覆盖 node.rs `Some(Err(e))` 分支 + `dom_error_exception` 的 WouldCreateCycle/NotAChild 映射。

### R36 单测 2：element.rs aria/role IDL 反射（`native_aria_role_idl_reflection_r36`）

5 个 aria 反射断言（覆盖 `idl_to_attr` 全分支 + aria_reflected_getter/setter）：
- `ariaLabel` ↔ `aria-label`（idl_to_attr aria 分支：aria 前缀+大写→连字符小写）
- `role` ↔ `role`（idl_to_attr role 特殊分支）
- aria/role 缺省空串
- `ariaLabelledBy` ↔ `aria-labelledby`（多段驼峰小写）
- aria setter null → content 属性 "null"（非 LegacyNullToEmptyString，spec null→"null"）

## 验证

| 门禁 | 命令 | 结果 |
|------|------|------|
| R36 单测 1 | `cargo test -p zero-engine --features v8 --lib native_node_mutation_error_paths_r36` | ✅ 1 passed（5 错误路径） |
| R36 单测 2 | `cargo test -p zero-engine --features v8 --lib native_aria_role_idl_reflection_r36` | ✅ 1 passed（5 aria 反射） |
| engine v8 全量 | `cargo test -p zero-engine --features v8 --lib` | ✅ 2114 passed（R35 基线 2112 +2） |
| engine quickjs 全量 | `cargo test -p zero-engine --no-default-features --features quickjs --lib` | ✅ 1411 passed（零回归） |
| clippy v8 | `cargo clippy -p zero-engine --features v8 --all-targets -- -D warnings` | ✅ 零警告 |
| fmt | `cargo fmt --all -- --check` | ✅ 无 diff |
| dom_bindings coverage | `scripts/check-dom-bindings-coverage.sh` | 源码 94.27%→**94.35%**（+0.08pp）/ 全部 95.95%→96.01%；node.rs 91.6%→**92.3%**（+4 行） |

## 决策记录

- **为何从 Event-dispatch 转向 coverage**：Event-dispatch document/window 入 chain 是深结构（listener key 三合一架构），按护栏跳过；namespaceURI getter 核实已完成（master.md 过时）；coverage 提升是剩余最干净的低风险切片（纯测试新增、DC-4 持续提升硬指标）。
- **错误路径测试用 try/catch 捕获 e.name**：native DOMException 经 throw_dom_exception 抛出，run_script 用 `.expect("run")` 会 panic，故 JS 侧 try/catch 捕获 DOMException.name 验证 spec 合规（HierarchyRequestError/NotFoundError）。
- **aria 测试覆盖不增但保留**：idl_to_attr aria 分支可能已被既有 aria 测试部分覆盖（element.rs 662/714 计数未动），但 R36 aria 测试验证了 aria 反射行为正确性（aria*→aria-x 转换 + role 特殊 + null→"null" 语义），是 default-on 后 a11y 合规的正确性证据，保留。

## 勘误（master.md 过时记录）

- **「native namespaceURI getter 独立化（dom/nodes 双路径差 0.65pp）」已完成**：element.rs:61 `native_namespace_uri_getter` 闭合 R3163（mod.rs:131 注册），dom/nodes 双路径差 0.65pp 的根因非 namespaceURI（已实现）。剩余差归因 createElementNS 其他面或用例侧 document=polyfill（未解问题 #9）。

## 净影响

- DC-4（coverage 持续提升不退化）：dom_bindings 源码 94.27%→94.35%（+0.08pp）/ 全部 95.95%→96.01%；node.rs 91.6%→92.3%
- DC-4（每项修复有单测）：node mutation 错误路径 + aria 反射单测覆盖（cycle/NotFoundError spec 合规 + a11y IDL 反射正确性，default-on 后合规证据）
