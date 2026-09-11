# Spec + RFC：page-wasm M3 — host function / importObject 宿主重入

**版本**: v1.0
**日期**: 2026-09-12
**状态**: 草稿（Rally 交棒件——下一轮按 §7 实施交接执行）
**父目标**: docs/goal/page-wasm.md M3（master.md「下一步计划」切片 1）
**规范依据**: https://webassembly.github.io/spec/js-api/ （WebAssembly JavaScript API）

---

## 0. 执行摘要

- **一句话目标**：让 embedder/execute 路径（`execute_script_with_dom` → `__WASM_BRIDGE__` →
  zero-wasm-sandbox）的 `instantiate(module, importObject)` 中 **JS 函数真作 wasm import**——
  wasm 执行调用 import 时同步回调 JS 并取回返回值（spec 语义），而非当前的「importKeys
  只收集不消费」。
- **本期范围**：①桥协议补 import 函数表传递；②host 侧按模块声明的 import 签名构建
  `LinkerConfig`；③HostFn 内同步重入 JS sandbox 执行回调；④值编组复用 M1 切片 2 线格式。
- **明确排除**：wpt-runner/testharness 路径（该路径跑 V8 原生 WebAssembly，importObject
  天然完整，本切片不触及）；iterator-protocol 级多返回值细节（v1 支持 Array 形式）；
  `WebAssembly.Tag/Exception` 相关 import。
- **核心约束**：
  1. 重入只发生在 host 排空点（`process_wasm_calls`），此时不在 V8 执行栈内——嵌套
     `sandbox.execute` 是 V8 同线程合法操作；
  2. import 回调内的 wasm export 调用走既有异步队列协议，天然无同步递归（无需深度
     防护，v1 仍加 1 层重入断言防回归）；
  3. import 函数签名以**模块声明为准**（host 侧 `Module::imports()` 反查），JS 侧只传
     函数身份不传类型；
  4. js-dom_shim/生产页面路径（V8 原生 WA）零改动。
- **推荐方案**：同步重入（§8.6 方案 A）。
- **首个落地步骤**：wasm-sandbox 补 `import_signatures()`（镜像 `export_descriptors()`，
  按 `Module::imports()` 反查），独立提交可先行落地并通过既有 205 测试。

---

## 1. 背景与目标

### 1.1 背景

page-wasm goal 的 polyfill 桥（execute 路径）自立项起就把 importObject 的键名收集进
`importKeys` 传给 host（`crates/engine/src/dom_bridge.rs` instantiate 段），但 host
`handle_wasm_instantiate_bridge` 只消费 `bytes`，**import 面从未接线**：带真实 import
的模块在 `module.instantiate(&sandbox)` 时因 import 缺失失败（M2 切片 3 已把该失败分类为
`LinkError`）。M1 基线证实 embedder 路径是 polyfill 的唯一消费面（wasm_bridge 测试族 +
外部 embedder），WPT window 面走 V8 原生 WA 不受影响。

wasm→JS 的同步回调是 M3 的核心难点：HostFn 在 `WasmInstance::call()` 的 wasmi 执行栈内
被调用，此刻 wasm 语义要求拿到 JS 返回值才能继续——队列式异步协议（导出函数调用的现行
机制）无法表达。但 host 排空点（`process_wasm_calls`）运行在 webview 宿主代码中，
**不在 V8 执行栈内**，此刻直接 `js_sandbox.execute(...)` 是同线程嵌套进入 V8——V8 支持
（HandleScope 可嵌套），这是方案 A 的可行性根基。

### 1.2 目标

- 业务目标：页面 WASM 与 JS 互操作在 embedder 路径达到 spec 语义（goal M12 的完成面）。
- 用户目标：embedder（含 wasm_bridge 测试族）可实例化带 JS 函数 import 的 wasm 模块，
  wasm 调用 import 时 JS 函数同步执行、返回值按类型编组回 wasm。

### 1.3 范围边界

- **在范围内**：`instantiate(buffer, importObject)` 与 `instantiate(module, importObject)`
  两形态；函数 import（`ExternType::Func`）；i32/i64/f32/f64 参数与返回值编组；JS 异常
  → wasm trap 传播；LinkError 分类维持。
- **不在范围内**：Global/Table/Memory 形态的 import（模块声明了则维持 LinkError 失败，
  v1 不接）；wpt-runner/testharness 路径；QuickJS 后端（v1 V8 路径启用，QuickJS 见 TBD-1）；
  `WebAssembly.Tag/Exception` import；ESM integration。

---

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|------|---------|------|
| 业务需求 | 是 | goal M12「页面 WASM 与 JS 互操作」完成面 |
| 用户需求 | 是 | wasm_bridge 测试族 / 外部 embedder 场景 |
| 解决方案需求 | 是 | M3 切片 1 范围收窄决策（master.md） |
| 功能需求 | 是 | 本文档 §3 |
| 非功能需求 | 是 | 本文档 §4 |
| 接口需求 | 是 | 本文档 §5（桥协议 + wasm-sandbox API） |
| 过渡需求 | 否 | 无迁移面（纯增量，旧协议回落保留） |

---

## 3. 功能需求

### FR-001：importObject 函数表传递
- **描述**：当 JS 侧 `WebAssembly.instantiate(bytes, importObject)` 的 importObject 中
  出现函数成员时，polyfill 必须为每个函数分配稳定 id、注册到 `WebAssembly._importFns`，
  并把 `[{module, name, id}]` 列表随 `__WASM_BRIDGE__` 载荷传给 host；host 必须按
  模块声明的 import（`module.imports()`）逐项建立链接，而非按 importObject 的键集合。
- **优先级**：必须
- **来源**：M2 切片 3 观察到的 importKeys-only 现状（本切片起点）

**验收场景**：

```
场景: 函数 import 正常链接
  假设 wasm 模块声明 (import "env" "add" (func (param i32 i32) (result i32)))，
    JS 传入 importObject = { env: { add: function(a, b) { return a + b; } } }
  当 经 execute_script_with_dom 发起 instantiate 且 host 排空实例化桥
  那么 实例化成功；__wasm_errors__ 无该实例条目；实例导出可入队调用
  验证: webview wasm_bridge 测试族新增 e2e（wasm-sandbox HostFn + 重入编组全链）

场景: importObject 缺函数（缺失链接）
  假设 wasm 模块声明 import "env"."add"，JS 未提供该键
  当 instantiate 排空
  那么 __wasm_errors__[id] 为 WebAssembly.LinkError 实例（M2 切片 3 既有分类维持）
  验证: 既有 test_wasm_bridge_error_classification 保持全绿

场景: import 签名不匹配
  假设 wasm 模块声明 import "env"."add" (param i32 i32)，JS 提供同名函数但模块
    实际声明的类型由 host 从模块反查（JS 不传类型）
  当 host 构建链接时
  那么 以模块声明为准构建 HostFn 签名；JS 返回值按声明结果类型解编（无法解编时
    该次调用按 trap 处理，外层 call 返回 Err）
  验证: wasm_bridge e2e——JS 返回值与声明类型不符（如声明 i32 返回字符串）时
    call 返回错误且不 panic
```

### FR-002：HostFn 同步重入 JS 回调
- **描述**：当 wasm 执行调用已链接的函数 import 时，host 必须在 HostFn 内同步调用
  `js_sandbox.execute` 执行 `WebAssembly._invokeImport(id, argsWire)`，JS 函数的返回值
  必须按模块声明的结果类型解编后写入 wasmi results 缓冲，使 wasm 侧同步观察到该值。
- **优先级**：必须
- **来源**：spec js-api import 语义；M3 切片 1 范围收窄决策

**验收场景**：

```
场景: wasm 调 import 取回返回值
  假设 模块 export call_add：调用 import "env"."add"(local.get 0, local.get 1) 并返回其结果
  当 JS 侧 add = function(a,b){ return a*b; }，实例化后调用 call_add(6, 7)
  那么 host 排空后 _callResults 中该调用的结果为 42（JS 返回值生效）
  验证: wasm_bridge e2e——import 返回值参与 wasm 运算

场景: JS import 抛异常
  假设 import 函数体 throw new TypeError('boom')
  当 wasm 调用该 import
  那么 HostFn 返回 Err（消息含异常标记），外层 call 返回 Err，_callResults 注入 null
    （不 panic、不毒化后续调用）
  验证: wasm_bridge e2e——异常路径 + 同实例后续调用仍可用
```

### FR-003：值编组复用 M1 切片 2 线格式
- **描述**：wasm→JS 方向：WasmValue 按声明参数类型转 JS 字面量（i32 → number、
  i64 → BigInt 字面量、f32/f64 → number），经 JSON 载荷传给 `_invokeImport`；
  JS→wasm 方向：返回值按声明结果类型解编（i32 → i32、i64 → 十进制字符串线格式、
  f32/f64 → number；多结果 → Array 按序解编）。
- **优先级**：必须
- **来源**：M1 切片 2 类型化协议（`parse_wire_arg`/`js_result_literal` 既有实现复用）

**验收场景**：

```
场景: i64 import 编组（>2^53 精度）
  假设 模块声明 import (func (param i64) (result i64))，JS 函数为 n => n + 1n
  当 wasm 以 i64 9007199254740993 调用该 import
  那么 JS 侧收到 BigInt 9007199254740993n，wasm 收回 9007199254740994
  验证: wasm_bridge e2e——BigInt 双向精度保持
```

### FR-004：非函数 import 维持 LinkError
- **描述**：当模块声明的 import 为 Global/Table/Memory 形态时，本切片不接值传递——
  host 必须以 LinkError 失败实例化（消息注明形态不支持），不得静默成功。
- **优先级**：必须
- **来源**：范围边界（§1.3）——失败显式化优于半支持

**验收场景**：

```
场景: Memory import 失败显式化
  假设 模块声明 (import "env" "mem" (memory 1))，JS 提供任意 importObject
  当 instantiate 排空
  那么 __wasm_errors__[id] 为 LinkError 且消息含 "memory" 形态说明
  验证: wasm_bridge e2e——非函数 import 显式失败
```

### FR-005：旧协议回落不变
- **描述**：当 JS 侧未传 importObject（或为空对象）时，现行实例化路径必须零变化
  （无 imports 的模块实例化、_start 自动执行、导出面注入、错误分类全部维持既有
  测试语义）。
- **优先级**：必须
- **来源**：回归保护——wasm_bridge 测试族 15+ 既有用例为回归基线

**验收场景**：

```
场景: 既有用例零回归
  假设 本切片落地后的代码树
  当 跑 make test
  那么 既有 wasm_bridge 测试族全部保持通过（其中 test_wasm_bridge_instantiate、
    test_wasm_bridge_start_auto_execution、test_wasm_bridge_error_classification
    为直接回归面）
  验证: make test 全绿
```

---

## 4. 非功能需求

### NFR-001：重入安全
- **描述**：HostFn 内嵌套 `sandbox.execute` 必须限定深度 1（import 调 JS 一次）；
  import 回调内对 wasm export 的调用经队列协议（异步）不得同步递归进入 wasmi。
- **测量标准**：Rust 侧重入守卫（计数断言）；递归构造用例（import 内直接调
  `call_wasm_export`——宿主 API 不暴露给页面 JS，用单测模拟）确认不 panic。
- **优先级**：必须

### NFR-002：性能回归护栏
- **描述**：本切片不得使 `make test` 全套件时长出现非正常退化（>20%）；无 wasm 活动的
  execute 路径新增开销必须为零（imports 构建只在 instantiate 桥触发时发生）。
- **测量标准**：既有 make test 时长对照（本轮基线 ~15 分钟量级）；perf-gate 按需
  （ZERO_WEB_BENCH_CRATES=zero-wasm-sandbox,zero-webview make bench-gate）。
- **优先级**：应该

---

## 5. 接口需求

### IF-001：桥协议扩展（`__WASM_BRIDGE__` 载荷）
- **类型**：JS ↔ host 协议（engine dom_bridge ↔ webview）
- **规格**：载荷新增可选字段 `imports: [{module: string, name: string, id: number}]`；
  `WebAssembly._importFns[id]` 保存 JS 函数引用；新增 `WebAssembly._invokeImport(id,
  argsJson)` 供 host 注入脚本调用——执行函数并返回 JSON 线格式结果（`{v: <wire>}`，
  异常时返回 `{e: message}`）。
- **错误处理**：JS 函数缺失（id 未注册）→ `_invokeImport` 返回 `{e: "import fn missing"}`；
  host 侧按 trap 处理。
- **默认动作**：无 `imports` 字段（旧协议载荷）时 host 走现行无链接实例化路径。
- **交叉引用**：值线格式定义见 FR-003。

### IF-002：wasm-sandbox import 签名反查 API
- **类型**：Rust API（zero-wasm-sandbox）
- **规格**：`WasmModule::import_signatures() -> Vec<ImportDescriptor>`；`ImportDescriptor`
  复用 `ExportDescriptor` 结构（name 字段 = "module.name" 拼接或拆分字段，以实现简取），
  仅收录 `ExternType::Func` 形态。三后端一致实现（wasmi 实查 / wasmtime 镜像 / stub 空）。
- **错误处理**：无 import → 空数组。
- **默认动作**：stub 后端恒空（既有 stub 语义）。
- **交叉引用**：结构定义复用 `ExportDescriptor`（crates/wasm-sandbox/src/types.rs）。

---

## 6. 约束与假设

### 6.1 必须约束（Must）
- 重入仅限 host 排空点发起（`process_wasm_calls` / instantiate 桥内调用），不得在
  `run_page_scripts` 的 V8 执行栈内新增任何重入点。
- import 签名以模块声明为唯一权威（`WasmModule::import_signatures()`）。
- 值编组必须复用 M1 切片 2 的线格式实现（`parse_wire_arg` / `js_result_literal`），
  不得引入第二套编组。
- 既有 wasm_bridge 测试族（FR-005 场景所列直接回归面）语义零变化。

### 6.2 禁止约束（Must Not）
- 不得改动 `run_page_scripts` / js_dom_shim / wpt-runner testharness 路径。
- 不得为通过测试静默吞掉 import 调用错误（错误必须可见：trap 或注入错误通道）。
- 不得在 HostFn 内持有跨 `sandbox.execute` 的 V8 句柄缓存（每回调独立 execute）。

### 6.3 已定决策
- 方案 A 同步重入（§8.6）：host 排空点直接嵌套 `sandbox.execute`。
- v1 仅函数 import；Global/Table/Memory import 显式 LinkError（FR-004）。
- 默认 feature 组合（V8 + wasmi）为验收组合；QuickJS 后端延后（TBD-1）。

### 6.4 技术约束
- `js_sandbox: Option<Box<dyn Sandbox>>` 与 `wasm_instances: HashMap<u64, WasmInstance>`
  为同结构体不相交字段——HostFn 闭包需同时持有两者访问能力，实现上须用字段级借用
  拆分或 `Rc/Arc` 化沙箱句柄，禁止 `unsafe` 绕过。
- 桥载荷经 JSON 字符串，BigInt 不可直接 JSON 化（i64 走十进制字符串线格式，M1 切片 2
  既有约束）。

### 6.5 假设
- V8 嵌套 execute（同线程、非执行栈内发起）可用且看门狗独立生效 — 状态：**待验证**
  （首个落地步骤中用 5 行探针测试确证，失败则升级 §8.6 方案 C）。
- rquickjs 同线程嵌套 execute 行为未知 — 状态：待验证（TBD-1 的内容之一）。
- wasmi `Module::imports()` 提供 module/name/ExternType::Func 访问（`ImportType`
  API 与 exports 对称）— 状态：待验证（实现时确证，API 命名偏差不阻塞设计）。

### 6.5A 实现来源说明

| 能力/行为 | 来源类型 | 具体来源 | 备注 |
|----------|----------|----------|------|
| import 签名反查 | 复用现有模块 | wasmi `Module::imports()`；镜像 `export_descriptors()` 模式（crates/wasm-sandbox/src/wasmi_backend.rs） | API 访问器命名实现时确证 |
| HostFn 机制 | 复用现有模块 | `LinkerConfig`/`HostFunction`/`instantiate_with_linker`（crates/wasm-sandbox/src/wasmi_backend.rs 既有完整实现） | 本切片不新增 wasm-sandbox 执行机制，只接线 |
| JS 值编组 | 复用现有模块 | `parse_wire_arg`/`js_result_literal`/`escape_js_string`（crates/webview/src/webview.rs） | FR-003 |
| JS 回调执行 | 复用现有模块 | `Sandbox::execute`（crates/script-sandbox，V8 后端） | 重入可行性见 §6.5 假设 1 |

### 6.6 代码变更边界
- **允许修改**：`crates/wasm-sandbox/src/**`（import_signatures + 测试）、
  `crates/engine/src/dom_bridge.rs`（polyfill WebAssembly 段——`_importFns`/
  `_invokeImport`/载荷组装）、`crates/webview/src/webview.rs`（wasm 段：
  instantiate 桥 imports 接线 + HostFn 闭包）、`crates/webview/src/tests/wasm_bridge.rs`
  （新用例）、`docs/goal/page-wasm/**`、`docs/specs/page-wasm-host-function-spec-rfc.md`。
- **禁止修改**：`crates/engine/src/js_dom_shim/**`、`crates/script-sandbox/**`（重入
  探针失败需升级方案时先停下按 §8.9 处理，不顺手改沙箱）、`tests/wpt-runner/**`
  （WPT 面不属本切片）、渲染流域 crate（run-rules §9）。

---

## 7. 优先级与里程碑建议

| ID | 需求 | 优先级 | 理由 | 里程碑 |
|----|------|--------|------|--------|
| FR-001 | importObject 函数表传递 | 必须 | 链接的前置 | M3-1a |
| FR-002 | HostFn 同步重入 | 必须 | spec 语义核心 | M3-1b |
| FR-003 | 值编组复用 | 必须 | 一致性 | M3-1b |
| FR-004 | 非函数 import 显式失败 | 必须 | 失败显式化 | M3-1a |
| FR-005 | 旧协议回落不变 | 必须 | 回归保护 | M3-1c（收口） |

### 建议里程碑
- **M3-1a**：wasm-sandbox `import_signatures()`（三后端 + 单测）+ polyfill 载荷
  `_importFns`/`_invokeImport`（独立可落地提交）。
- **M3-1b**：webview instantiate 桥 imports 接线 + HostFn 重入编组 + e2e 用例。
- **M3-1c**：回归收口（make test + clippy + master.md 记账）。

### 实施交接（Implementation Handoff）

#### 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险/注意事项 |
|----------|------|------|---------------|
| `crates/wasm-sandbox/src/types.rs` | 修改 | 补 `ImportDescriptor`（或复用 ExportDescriptor + 来源标记） | 结构最小化 |
| `crates/wasm-sandbox/src/wasmi_backend.rs` | 修改 | `import_signatures()`（imports() 反查） | API 访问器命名实现时确证（§6.5 假设 3） |
| `crates/wasm-sandbox/src/wasmtime_backend.rs` | 修改 | 镜像实现 | feature 关闭，编译验证靠 CI |
| `crates/wasm-sandbox/src/stub_backend.rs` | 修改 | 恒空实现 | 维持 stub 语义 |
| `crates/engine/src/dom_bridge.rs` | 修改 | `_importFns` 注册 + `_invokeImport` + 载荷 `imports` 字段 | 幂等安装守卫已就位，新字段随桥对象安装 |
| `crates/webview/src/webview.rs` | 修改 | instantiate 桥：imports 接线 → LinkerConfig 构建 → HostFn 闭包（重入 execute + 编组） | 借用拆分（§6.4）；禁改 wasm 段外区域 |
| `crates/webview/src/tests/wasm_bridge.rs` | 修改 | FR-001~FR-004 e2e 用例 | 复用 wat 构造模块 |

#### 职责映射

| 模块/文件 | 职责 | 依赖/被依赖 | 验证方式 |
|----------|------|------------|----------|
| wasm-sandbox | import 签名 + 链接执行 | 被 webview 消费 | crate 单测 |
| engine dom_bridge | JS 侧函数注册与回调执行面 | 被 webview 注入脚本调用 | wasm_bridge e2e |
| webview wasm 段 | 桥协议消费 + LinkerConfig 组装 + 重入 | 依赖 wasm-sandbox + script-sandbox | wasm_bridge e2e + make test |

#### 新能力来源对照

| 能力/需求 | 实现承载位置 | 来源类型 | 验证方式 |
|----------|--------------|----------|----------|
| import 签名 | wasmi_backend（imports() 反查） | 复用现有模块 + 仓内自实现镜像 | wasm-sandbox 单测 |
| 重入回调执行 | webview HostFn 闭包 → Sandbox::execute | 复用现有模块 | wasm_bridge e2e（FR-002） |

#### 推荐修改顺序

1. **探针先行**：5 行 wasm_bridge 探针测试确证 V8 嵌套 execute 可行（§6.5 假设 1）——
   `instantiate 后 drain 内执行 execute("1+1")` 已被 M2 各切片隐式验证过（每 execute
   尾注入脚本即在非栈内执行），此处只需确认 **HostFn 栈内**（`instance.call` 期间）
   发起 execute 的形态。失败 → 停止，按 §8.9 升级方案 C，不硬改 script-sandbox。
2. wasm-sandbox `import_signatures()`（M3-1a）——独立提交，crate 单测过即可 land。
3. polyfill `_importFns`/`_invokeImport` + 载荷字段（M3-1a 同提交或次提交）。
4. webview 桥接线 + HostFn（M3-1b）——先 FR-001 正常链接用例，再 FR-002/FR-003，
   最后 FR-004。
5. 回归收口（M3-1c）：make test + clippy + master.md 记账。

#### 首批提交建议

| 提交/批次 | 范围 | 预期结果 | 验证 |
|----------|------|----------|------|
| Commit 1 | 探针测试（可并入 Commit 2） | 假设 1 确证或升级决策 | wasm_bridge 单测 |
| Commit 2 | wasm-sandbox import_signatures + polyfill 注册面 | 旧协议零回归，新 API 可用 | cargo test -p zero-wasm-sandbox -p zero-engine |
| Commit 3 | webview 接线 + e2e 全用例 | FR-001~004 全绿 | make test 全绿 |

---

## 8. 技术设计（RFC）

### 8.1 现状分析
- **当前架构**：JS `instantiate(bytes, imports)` → polyfill 收集 importObject 键名为
  `importKeys`（`mod.fn` 字符串数组）随载荷传 host → host 忽略之，`module.instantiate`
  无链接执行 → 带函数 import 的模块 InstantiationError（→ LinkError 注入）。
- **问题/痛点**：①importKeys 只有名字没有函数体，host 无法执行；②即使有函数体，现行
  队列协议无法在 wasmi 执行栈内同步取回 JS 返回值；③`LinkerConfig`/`HostFunction`
  在 wasm-sandbox 层已完整实现（含参数/结果类型声明）却零消费方。
- **相关代码**：`crates/engine/src/dom_bridge.rs`（instantiate 段 importKeys 收集）、
  `crates/webview/src/webview.rs`（`handle_wasm_instantiate_bridge`）、
  `crates/wasm-sandbox/src/wasmi_backend.rs`（`instantiate_with_linker` 完整实现）。

### 8.2 目标状态
- **提议架构**：JS 侧注册函数 → 载荷传 id 表 → host 按 `import_signatures()` 组装
  `LinkerConfig`（每 import 一个 HostFunction）→ HostFn 闭包同步 `sandbox.execute`
  执行 `_invokeImport` → JSON 线格式往返 → 按声明类型编组进 results。
- **关键变更**：polyfill WA 对象新增 `_importFns`/`_invokeImport`；载荷新增 `imports`；
  webview instantiate 桥新增 LinkerConfig 组装与借用拆分；wasm-sandbox 新增
  `import_signatures()`。

### 8.3 影响范围分析
| 影响项 | 影响程度 | 说明 |
|--------|----------|------|
| execute 路径（wasm_bridge 消费面） | 高 | 新能力主战场，既有用例回归面 |
| run_page_scripts / WPT 路径 | 无 | V8 原生 WA，不经过 polyfill |
| wasm-sandbox API 面 | 低 | 纯增量（新方法），既有 205 测试不变 |
| 生产页面 | 无 | polyfill 不装在该路径（M1 基线结论） |

### 8.4 详细设计

**时序（一次带 import 的 instantiate）**：

```
JS execute #1（polyfill 已装）
  instantiate(bytes, {env:{add}})          ── polyfill
    _importFns[7] = add                    │ 函数注册（id 自增）
    _pendingBridge = __WASM_BRIDGE__:
      {id, moduleId, bytes, imports:[{module:"env",name:"add",id:7}]}
execute #1 尾（host，非 V8 栈内）
  handle_wasm_instantiate_bridge
    sigs = module.import_signatures()      │ 模块声明为权威
    linker = LinkerConfig; for each sig:   │ 按 sig.module/sig.name 匹配 imports 条目
      linker.define(HostFunction{ sig.params, sig.results, func = |params, results| {
        args = params.map(js_literal)      │ wasm→JS 编组（复用 js_result_literal）
        out = sandbox.execute("JSON.stringify(_invokeImport(7, JSON.stringify(args)))")
        r = parse(out)                     │ {v:...} 成功 / {e:...} 异常
        results[i] = parse_wire_arg(...)   │ JS→wasm 解编
      }})
    instance = module.instantiate_with_linker(sandbox, &linker)
JS execute #2（调用导出）
  exports.call_add(6,7) → 队列
execute #2 尾（host 排空）
  instance.call("call_add") ── wasmi ──> 调 import ──> HostFn ──execute──> JS add(6,7)=42
    ──> results=[i32(42)] ──> _callResults[callId]=42
```

**数据模型**：
- `WebAssembly._importFns`（JS，id → 函数引用，随 WA 幂等安装持久）。
- 载荷 `imports: [{module, name, id}]`（JSON）。
- `ImportDescriptor`（Rust）：`{module: String, name: String, params, results}`——
  与 `ExportDescriptor` 拆分定义（import 需双段命名，不复用单 name 结构）。

**JS `_invokeImport` 伪代码**：

```
_invokeImport(id, argsJson):
  fn = _importFns[id]
  if (!fn) return JSON.stringify({e: "import fn missing"})
  args = JSON.parse(argsJson)            // 按声明类型已还原（i64 为 string，函数内 BigInt(n)）
  try { v = fn(...args)                  // JS 异常 → catch
        return JSON.stringify({v: toWire(v)}) }   // i64 → String(v)，多结果 → 数组
  catch (e) { return JSON.stringify({e: String(e && e.message || e)}) }
```

**借用拆分**：`handle_wasm_instantiate_bridge` 内 `let (sandbox_js, instances) =
(&mut self.js_sandbox, &mut self.wasm_instances);` 字段级不相交借用；HostFn 闭包以
`Arc<Mutex<...>>` 或「先取出 instance、组装完再放回」的 take/restore 模式避免双可变
借用（实现取简，不引入 Arc 开销——take/restore 即可，实例化点实例尚未入缓存）。

### 8.5 安全考虑
- **权限控制**：`_invokeImport` 只执行 `_importFns` 内注册引用，不接受动态源码字符串。
- **数据保护**：跨桥载荷均为 JSON 线格式数字/字符串，无原始指针/句柄泄漏；JS 异常
  消息经 `escape_js_string` 转义后注入（SEC-08 既有纪律）。
- **潜在风险**：恶意 wasm 高频调用 import → 每次一次嵌套 execute（脚本看门狗
  `script_timeout_ms` 90s 兜底）；无同步递归通道（队列协议天然异步，NFR-001）。

### 8.6 替代方案

#### 方案对比表

| 维度 | 方案 A：同步重入 | 方案 B：队列挂起 | 方案 C：放弃函数 import |
|------|------------------|------------------|--------------------------|
| 实现复杂度 | 🟡 中（借用拆分 + 编组） | 🔴 高（wasmi 无法暂停栈） | 🟢 低（现状 + 文档化） |
| spec 语义 | 🟢 真（同步返回值） | 🟡 部分语义 | 🔴 无（LinkError） |
| 可靠性 | 🟢 无栈改动 | 🔴 需改执行引擎 | 🟢 高 |
| 可维护性 | 🟢 复用既有 LinkerConfig | 🔴 侵入 wasm-sandbox 核心 | 🟢 好 |
| 成本 | 🟡 中 | 🔴 高 | 🟢 零 |
| **推荐度** | ⭐⭐⭐ | ⭐ | ⭐⭐（作为假设 1 失败时的降级） |

**最终选择**：方案 A；假设 1（V8 嵌套 execute）被证伪时降级方案 C（FR-004 面已就位，
失败路径零新代码）。

**理由**：
1. wasmi 执行栈无法暂停（方案 B 物理不可行，除非换执行引擎——超出本 goal 边界）。
2. host 排空点不在 V8 栈内，嵌套 execute 是 V8 支持的形态，无需改 script-sandbox。
3. LinkerConfig/HostFunction 既有完整实现，方案 A 是「接线」而非「重建」。

### 8.7 实施计划
1. 探针确证（§7 修改顺序 1）→ 2. import_signatures 三后端 → 3. polyfill 注册面 →
4. webview 接线 → 5. e2e 用例逐 FR 落 → 6. 回归收口。

### 8.8 测试策略
- **单元测试**：wasm-sandbox `import_signatures()`（func-only 收录、module/name 拆分、
  空 import 恒空、stub 恒空）。
- **集成测试（wasm_bridge e2e）**：FR-001 正常链接/缺失/类型不符、FR-002 返回值生效/
  JS 异常不毒化、FR-003 i64 BigInt 双向、FR-004 Memory import 显式 LinkError、
  FR-005 既有用例全绿。
- **端到端**：make test 全绿 + clippy -D warnings。

### 8.9 回滚计划
- 全部新能力为增量路径：载荷无 `imports` 字段 → 走现行无链接路径（默认回落）。
- 回滚 = revert 本切片提交；无数据/格式迁移，`__WASM_BRIDGE__` 载荷向后兼容。
- 假设 1 证伪时：保 Commit 2（签名 + 注册面，无害），放弃 Commit 3，FR-004 面
  显式化维持（现状即 LinkError），目标回 M3 待定——记 master.md 待用户决策。

---

## 9. Spec Lint 报告

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要存在性 | ✅ Pass | §0 存在，含目标/范围/排除/约束/方案/首步 |
| 场景存在性 | ✅ Pass | FR-001~FR-005 均含 ≥1 验收场景（§3） |
| 异常路径覆盖 | ✅ Pass | FR-001 缺失+签名不匹配、FR-002 抛异常、FR-004 显式失败——异常场景数 ≥ 正常 |
| 测试绑定 | ✅ Pass | 每场景标注 wasm_bridge e2e / make test 验证命令 |
| UI 对齐 | ⏭️ Skip | 无 UI 面 |
| TBD 清零 | ⚠️ Warning | TBD-1（QuickJS 后端）为「重要」级非阻塞，已在 §6.3 声明延后 |
| 约束覆盖 | ✅ Pass | §6.1 四条 Must 分别由 FR-003（编组复用）、FR-005（零回归）、NFR-001（重入）、FR-001（签名权威）覆盖 |
| 实施交接完备 | ✅ Pass | §7 文件清单/职责映射/修改顺序/首批提交四件齐备 |
| 首步可执行性 | ✅ Pass | §7 修改顺序 1 给出探针测试形态与失败处置 |

### 语言精确性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 模糊动词 | ✅ Pass | FR 均用「必须/不得 + 具体行为动词」（注册/反查/编组/注入） |
| 无量化描述 | ✅ Pass | NFR-001 限定深度 1；NFR-002 量化 >20% |
| 非确定性措辞 | ⚠️ Warning | §6.5/§8.6 中「待验证」「可行」为假设级表述——按规则属假设区，非 FR 措辞，接受 |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §1.3 排除项（Global/Table/Memory import、QuickJS、Tag/Exception）与 FR-004（显式失败）自洽——排除即显式失败而非静默支持 |
| 约束冲突 | ✅ Pass | §6.1 与 §6.2 无矛盾（重入点唯一 vs 不改沙箱：方案 A 在 webview 层发起，不触碰 script-sandbox） |
| 方案漂移 | ✅ Pass | 方案 A 不引入 §1.3 排除项或 §6.2 禁止项依赖 |
| 外部事实保守化 | ✅ Pass | wasmi imports() 访问器命名、V8/QuickJS 嵌套行为均降级为 §6.5 假设（待验证），未写入 FR |
| 未验证细节泄漏 | ✅ Pass | FR 断言均为本仓可控行为（错误类型/编组值/回归面），无外部命名断言 |
| 实现来源闭合 | ✅ Pass | §6.5A 四行来源表覆盖签名反查/HostFn/编组/回调执行 |
| 来源-测试联动 | ✅ Pass | FR-003 复用来源（parse_wire_arg 等）在 §6.5A 第 3 行指明 |
| 类型分层清晰 | ✅ Pass | 需求（§3/§4）/决策（§6.3）/假设（§6.5）/待定（TBD-1）分区 |
| 优先级完备 | ✅ Pass | FR/NFR 均标必须/应该 |
| 代码边界完备 | ✅ Pass | §6.6 允许/禁止双列（禁止含理由） |
| 重复失控 | ✅ Pass | 线格式定义主区在 FR-003，IF-001 只交叉引用 |

**汇总**：23 Pass / 2 Warning / 0 Fail / 1 Skip
**门禁判定**：Fail = 0 → 允许确认（Rally 交棒制下，确认 = 下一轮实施时按 §7 交接执行；
两条 Warning 均为假设级注记，无阻塞）。

---

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|----|------|--------|----------|--------|
| TBD-1 | QuickJS 后端的 importObject 支持 | 重要 | rquickjs 同线程嵌套 execute 可行性 | M3-1 落地后单独立探针验证；不可行则 feature-gate 下 LinkError 显式失败（FR-004 同型） |
| TBD-2 | iterator-protocol 多返回值 import | 可选 | spec 值协议细节工作量 | v1 Array 形式；WPT 子集无此断言面，按需另切片 |

---

## 11. 修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v1.0 | 2026-09-12 | 初始版本（M3 切片 1 设计，Rally 交棒件） |
