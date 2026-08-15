# M6 S0q — QuickJS 原生绑定骨架 PoC（R57）

**日期**: 2026-08-16
**commit**: `b781252f`
**里程碑**: M6 QuickJS 原生绑定移植（js-dom goal v1.1 DC-7 双引擎对等）首切片
**证据**: [evidence/2026-08-16-r57-quickjs-s0q-poc.json](../evidence/2026-08-16-r57-quickjs-s0q-poc.json)

## 目标

把「QuickJS 页面引擎 native = 真空」打破：镜像 V8 dom_bindings S0 的 PoC 面落一个
QuickJS（rquickjs）原生绑定骨架——rquickjs 原生对象持有 NodeId + 原生 getter 直读
Rust DOM + `Sandbox::install_native_bindings_quickjs` escape-hatch + webview
`native_dom=true` 生产接线。kill-switch 仍默认关 → 零回归 land。

## 实现

### 架构（镜像 V8，四个差异点）

| 维度 | V8 版 | QuickJS S0q 版 |
|------|-------|----------------|
| NodeId 承载 | ObjectTemplate internal slot[0]（External ptr） | 隐藏非枚举非可写 own property `__zwNodeFfi`（f64 Number） |
| 模板 | Global<ObjectTemplate> 线程局部缓存 | 无模板——工厂直接 Object::new + Accessor 注册 |
| 对象身份 | Weak + guaranteed finalizer（R3133） | **strong Persistent**（PoC；weak/finalizer 是后续切片） |
| 安装入口 | install_dom_bindings(scope, ctx, dom) | install_dom_bindings_quickjs(ctx, dom) |

- DOM 源：线程局部 `Rc<RefCell<Document>>`，getter 经 `with_dom`/`with_dom_mut`
  读真实 DOM（同 V8 gc.rs 模式；QuickJS Runtime 单线程，同线程派发安全）。
- 工厂 `__zw_native_element_for_id(idStr)`：**与 V8 同名同 wire 形态**——A/B
  对照门与测试双引擎复用同一调用脚本（miss → JS null 一致）。
- getter 面（PoC）：`nodeType`/`tagName`/`nodeName`/`id`(+setter)。
- `reset_quickjs_state()`：webview Drop 清线程局部（镜像 V8 R3334——QuickJS
  Runtime 随 WebView 销毁后 Persistent 悬垂，同线程第二个 native WebView 会
  UnrelatedRuntime）。

### rquickjs API 踩坑（本轮核心经验，S1q–S5q 全程受益）

1. **闭包「Ctx 参数 + 'js 值返回」有 HRP 生命周期困难**：闭包类型检查时 ctx 参
   （'1）与返回值（'2）两生命周期无法统一，且 `Value` 对 'js invariant（不可
   协变收缩）→ 编译失败。**解法：具名 fn**——签名 `'js` 显式统一，经
   `IntoJsFunc` trait impl 正确单态化。rquickjs 自身测试只展示了返回 u32 的
   Ctx 闭包（无 'js 值返回），返回对象/值时全部踩这个坑。
2. **Accessor getter/setter 的 this 经 `This<Object>` 参数接收**（`FromParam`
   实现，JS 调用侧不占实参位）。直接 `move |this: Object|` 闭包会以「缺 1 个
   实参」抛异常（getter 被 QuickJS 以 0 实参调用）。
3. `Persistent::save/restore` 消耗 self（用前 clone）；跨 Runtime restore 返
   `UnrelatedRuntime`。
4. Property builder flag 是**无参形态**（`.configurable()` 打开；默认关；无
   `configurable(bool)`）。
5. slotmap `KeyData::from_ffi(0).as_ffi() = 2^32`（version 强制 odd 的 `|1`）
   ——**任意值不自等**，`from_ffi` 只保证「`as_ffi` 产出值恒可逆」。f64 Number
   无损域 = version < 2^21（`ffi_f64_round_trip` 单测断言）。

### 接线

- `Sandbox::install_native_bindings_quickjs(installer)` trait 方法（cfg quickjs；
  默认 no-op false）。QuickJSSandbox 实现：`persistent_context: true` 时进持久
  Context（与 execute 共享）执行 installer；非持久返 false（绑定不可见，语义同
  V8 版「无持久 context 返 false」）。
- webview `install_native_dom_bindings` QuickJS 变体（native_dom gate +
  cached_doc_shared live 源 / cached_html re-parse 回落经
  `install_dom_bindings_quickjs_from_html` 封装——webview 不直接依赖 zero_dom，
  同 V8 `install_dom_bindings_from_html` 模式）。两处调用点 cfg 放宽到
  `any(v8, quickjs)`。

## 验证

| 矩阵 | 结果 |
|------|------|
| zero-engine quickjs | **1418 passed**（+2 新 PoC） |
| zero-webview quickjs | **552 passed**（+2 wiring 测试，与 V8 R3097 同断言面） |
| zero-engine v8 | 2153 passed（零回归） |
| zero-webview v8 | 599 passed（零回归） |
| zero-script-sandbox quickjs | 76 passed |
| clippy 双矩阵 | engine/webview/script-sandbox 零警告 |
| fmt | 无 diff |
| make test | 全 workspace 经 test-guard；唯一失败 `default_actions_work_without_javascript` 为**并行流既存**（clean HEAD `c220b746` 同败，表单导航域 html-compat，run-rules §9） |

PoC e2e 断言（engine 单测 + webview wiring 双层）：工厂命中 + 三 getter +
identity + miss→null + id setter 写 live Document 读回 + 隐藏 ffi 不可枚举。

## 与既有记录的关系（勘误 + 推进）

- **R57 开工时的切片改向记录**：master.md 下一步候选 (a)「M1 L2 live 视图最小
  切片复活」经本轮核实**不复活**——R43 诊断文档对 -10 根因的描述（「case.js
  iframe document 建元素」）与用例事实不符：case.js 实际在**主文档**
  documentElement 上建容器后查询、期望非空（live 合并方向本身正确），失配在
  `getElementsByTagName` 大小写匹配语义。即 R43 的结论「最小切片绕不开深改」
  仍成立（但根因记载不准），L2 待 M1 完整方案。转 master.md 候选 (h) +
  入口文档「M6 首切片选 S0q PoC」明示路径。
- DC-7 第一条（QuickJS rquickjs 原生绑定起步 + escape-hatch 实现）从 ❌ 真空
  → 🟡 骨架 PoC 已 land（S1q–S5q 是 M6 剩余）。

## 遗留（M6 后续切片输入）

1. **weak/finalizer 生命周期**（S0q 续）：当前 strong Persistent 会把已 GC 节点
   的条目留在缓存（泄漏）；镜像 V8 R3133 的 Weak + guaranteed finalizer +
   remove_node_listeners 对等物。rquickjs 侧对应物 TBD（`Persistent` 无 weak
   变体？需调研 rquickjs GC hook）。
2. S1q 只读属性族（className/attributes/classList/…镜像 V8 dom_bindings 既有面）。
3. bench 对照（RFC §4 S0 gate 的 quickjs 版）。
4. dom_bindings coverage 口径的 quickjs 参数化（现有脚本双 feature 可参数化，
   quickjs_dom_bindings 是新模块需纳入）。
