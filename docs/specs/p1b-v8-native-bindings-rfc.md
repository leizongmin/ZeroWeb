# RFC：P1b V8 原生绑定——替换 polyfill 字符串桥

**版本**：v0.1（草稿）
**日期**：2026-08-08
**作者**：ZeroWeb rally（自主推进）
**状态**：v0.1 已批准 S0 PoC（2026-08-09 用户决策）；**S0 PoC 验证完成（R3095, 2026-08-09）—— TBD-1/TBD-2 阻塞性验证通过**；**S1 dom_bindings 生产化完成（R3096, 2026-08-09）—— 首组原生 getter（nodeType/tagName）+ NodeId↔对象映射 + bench（native ~15.6x）+ kill-switch**；**S2 生产接线完成（R3097, 2026-08-09）—— 原生绑定接通 webview 真实页面沙箱（`Sandbox::install_native_bindings` escape-hatch + `WebViewConfig.native_dom`，默认关 → 零回归，webview 集成测试通过）**；S3+ 按 §4 逐片 land（详见 §6 S0/S1/S2 结论）

> **与字体栈 RFC 同级**：本 RFC 是 `docs/goal/zero-web/master.md` 反复标注「P1b（V8 原生绑定）需独立 RFC，与字体栈 RFC 同级对待」的落地。字体栈 RFC（`docs/goal/rendering-compat/fontdue-replacement-scoping.md`）解决「看起来对不对」（渲染一致性），本 RFC 解决「能不能用」（JS 性能与 Web Components 正确性）。

---

## 0. 执行摘要

- **一句话目标**：把页面 JS 与 Rust DOM 的桥接从「polyfill 字符串序列化」（`register_callback` 以 `String` 编解码参数 + 7157 行 JS shim）迁移到「rusty_v8 原生绑定」（`FunctionTemplate`/`ObjectTemplate` 包装 Rust DOM 对象，原生值传递），以解决 SPA 性能、customElements 正确性、类型保真、维护体量四个硬限制。
- **本期范围**：仅产出设计文档（RFC）。**不在本期范围**：任何代码改动（实施需用户审批本 RFC 后，按 §7 分阶段切片单独推进）。
- **明确排除**：不替换 V8 引擎本身（rusty_v8 仍是唯一页面 JS 引擎）；不改 QuickJS 沙箱（script-sandbox 的扩展脚本沙箱与本 RFC 无关）。
- **核心约束**：
  1. **增量、可回滚**——polyfill 桥与原生绑定长期共存，每个 API 独立迁移，任一切片可单独 revert，不允许「大爆炸」式重写。
  2. **测试不退化**——14137+ 测试 + WPT 1341 用例 + reftest 在迁移期必须保持绿；行为对照（polyfill vs native）是每切片的落地门。
  3. **GC 安全**——V8 持有的 Rust DOM 对象引用须经 weak handle + 根集协调，不得悬垂或泄漏。
  4. **单文件 ≤ 2000 行**——原生绑定代码按 DOM 子域拆模块。
- **推荐方案**：**方案 C 混合（Hybrid DOM-Node）**——原生「DOM node 对象」直接暴露 Rust `NodeId`（element/text/document），原生值传递；高层 Web API（事件、Fetch、Observer、FontFaceSet 等）保留 shim 但改为调用原生 node 方法（去字符串 ser/deser）。渐进、低风险、SPA 热路径先受益。
- **首个落地步骤**（审批后）：切片 0——建 `crates/engine/src/dom_bindings.rs` 骨架 + 一个 PoC 原生 `Element.nodeType`/`tagName` getter（直接读 Rust DOM，不经 shim），加 bench 对照（polyfill vs native 单次读取），验证 GC + 值传递管线可用，零行为变更。

---

## 1. 背景

### 1.1 当前架构（polyfill 字符串桥）

页面 JS 经 V8 执行，DOM 操作经**字符串序列化**桥接：

```
JS (shim js_dom_shim.js, 7157 行)
  → __zw_<op>(args: String[])  // register_callback 注册的全局函数
  → host_callback_invoke (v8_runtime.rs)
  → Box<dyn Fn(&[String]) -> String>  // Rust 回调，参数/返回值均为 String
  → 操作 Rust DOM (zero_dom) / 收集 DomMutation
```

- `zero_script_sandbox::V8Sandbox::register_callback(name, callback)` 把 Rust 闭包挂为 V8 全局函数，但参数与返回值都是 `String`（`v8_runtime.rs:272` 经 `FunctionTemplate` 但值经 String 编解码）。
- `js_dom_shim.js`（7157 行）在 JS 侧把 `document.getElementById`/`setAttribute`/`appendChild`/`querySelector` 等翻译成 `__zw_*` 字符串调用，并用 Proxy 模拟 element 对象（`_makeProxy(sel, handle)` 经 selector 或 handle 标识元素）。
- `js_dom_bridge.rs`（3674 行）是 Rust 侧：`register_dom_callbacks` 注册 30+ `__zw_*` 回调 + `DomMutation` 变体 + `apply_mutations_to_html` + 各 Bridge（Fetch/Timer/Rect/ElementFromPoint/FontLoad）。

### 1.2 痛点（驱动本 RFC）

| # | 痛点 | 根因 | 影响 |
|---|------|------|------|
| P1 | **SPA 不可用** | 每个 DOM 操作 O(μs) 级 String ser/deser（参数拼串 + 返回值解析），React/Vue 的 reconciliation 在 1000+ 节点遍历时累积成秒级卡顿 | 浏览器能渲染静态页，但跑不动现代 SPA 框架（Done Criteria §1.3「执行页面 JavaScript + 基础 DOM 操作」对 SPA 不达标） |
| P2 | **customElements upgrade 阻塞** | element 是 JS Proxy（非 class 实例），自定义元素 `class X extends HTMLElement` 的构造器无法在 Proxy 上 `new`（class ctor 不能 `.call()`，`Reflect.construct` 产生独立对象非 Proxy） | Web Components（lit/stencil/fast）核心需求不可用；R2813 registry 仅 define/whenDefined，upgrade 显式 defer |
| P3 | **类型保真差** | String marshaling 丢失类型：数字↔字符串、对象经 JSON、二进制（字体/图片字节）经 String 损坏 | 已踩坑：browser 单进程 FontFace.load() 因 fetch handler 返 String body 致二进制字体字节损坏（R2949 follow-up）；属性值 round-trip 边角多 |
| P4 | **维护体量大** | 7157 行 JS shim + 3674 行 bridge + 30+ `__zw_*` 回调 + DomMutation 变体持续增长（R2945–R2953 每轮新增） | 每个新 Web API 需双端（shim JS + bridge Rust + DomMutation）实现，边角多、回归面广 |
| P5 | **持久 Context 稳健性** | 字符串桥 + 持久 Context 下未捕获异常会中毒 Isolate（R2945 修了一个 latent bug，靠 classic 脚本 try-catch 包装规避） | 页面脚本抛错曾废掉其后所有脚本；包装是症状治疗，根因是字符串桥缺乏原生异常传递 |

### 1.3 目标

- **业务**：让 ZeroWeb 能跑现代 SPA（React/Vue/Svelte）与 Web Components（customElements）——从「能渲染静态页」到「能跑交互式应用」。
- **技术**：DOM 操作热路径（属性读写、子树遍历、查询）桥开销从 O(μs) 降到 O(100ns) 量级（目标 ≥10x），并支持 class 实例化的 HTMLElement。

### 1.4 范围边界

- **在范围内**：页面 JS ↔ Rust DOM 桥接的原生化；DOM 核心类型（Node/Element/Document/Text/Attr/EventTarget/Event）的原生绑定；迁移路径与共存策略；GC 集成。
- **不在范围内**：V8 引擎替换；QuickJS 扩展沙箱（script-sandbox）；渲染管线（rendering-compat 赛道）；网络栈原生化（net crate 行为不变，仅 JS 侧 fetch 桥受益）。


---

## 2. 设计选项

### 2.1 方案对比

| 维度 | 方案 A：全原生（Ladybird 式） | 方案 B：增量热路径原生 | 方案 C：混合 DOM-Node（推荐） |
|------|------------------------------|------------------------|------------------------------|
| 实现复杂度 | 🔴 高（重写全部 DOM API 为原生） | 🟢 低（仅热路径） | 🟡 中（原生 node + shim 复用） |
| SPA 性能 | 🟢 最高（全原生） | 🟡 中（仅热路径受益，高层仍字符串） | 🟢 高（node 操作全原生，高层去 ser/deser） |
| customElements | 🟢 支持（真 class 实例） | 🔴 不解决（仍 Proxy） | 🟢 支持（node 是原生对象，可被 class extend） |
| 迁移风险 | 🔴 高（大爆炸，14000+ 测试需重验） | 🟢 低（局部） | 🟡 中（node 模型引入，shim 渐进改写） |
| 可维护性 | 🟢 好（无 shim） | 🟡 一般（双模型长期共存） | 🟢 好（node 单一权威，shim 萎缩） |
| 成本 | 🔴 高（多 session） | 🟢 低 | 🟡 中（分阶段，每切片可 land） |
| 回滚 | 🔴 难 | 🟢 易 | 🟡 中（按 API） |
| **推荐度** | ⭐ | ⭐⭐ | ⭐⭐⭐ |

**图例**：🟢 优秀 | 🟡 一般 | 🔴 较差

### 2.2 推荐方案：C 混合（Hybrid DOM-Node）

**理由**：
1. **增量可land**：每个 DOM API 独立迁移，polyfill 与 native 长期共存，每切片可单独验证 + revert（满足约束 1「增量可回滚」）。
2. **SPA 热路径先受益**：node 操作（属性、子树、查询）原生化即覆盖 React/Vue reconciliation 主体；高层 API（Fetch/Observer）保留 shim 但调用原生 node 方法（去 String ser/deser）即消主要开销（痛点 P1）。
3. **解 customElements**：原生 node 对象可被 `class extends HTMLElement` 继承（真 class 实例），upgrade + lifecycle 可实现（痛点 P2）。
4. **测试连续性**：node 模型与现有 selector/handle 标识可桥接（NodeId ↔ selector/handle 映射），既有测试逐步改写而非重写（约束 2）。

---

## 3. 详细设计（方案 C）

### 3.1 核心抽象：原生 DOM Node 包装

Rust DOM 已有 `NodeId`（opaque，`crates/dom/src/node.rs`）。原生绑定把 `NodeId` 包装为 V8 对象：

```
Rust: struct DomNodeHandle(NodeId);  // 经 v8::Local<v8::Object> + internal field 存 NodeId
V8:   element 对象 = ObjectTemplate 实例，internal slot[0] = NodeId（u32）
      getter/setter/method 经 FunctionTemplate 直接调 Rust（原生值，不经 String）
```

- **标识桥接**：迁移期保留 selector/handle（既有 shim + 测试用）↔ NodeId 双向映射；node 对象可 `__selectorFor()` 暴露 selector 供旧 shim 互操作，旧 `_makeProxy(sel)` 可经 `__zw_node_for_selector` 升级为原生 node。
- **GC**：V8 持有的 NodeId 不 own Rust 节点（Rust DOM 是权威）。node 对象用 weak-persistent 模式：V8 GC 回收 node 对象时不影响 Rust DOM；Rust DOM 节点移除时不主动通知 V8（node 对象变 stale——getter 时校验 NodeId 仍存在，否则返 null/throw，spec detached 行为）。

### 3.2 模块拆分（单文件 ≤ 2000 行）

```
crates/engine/src/dom_bindings/
├── mod.rs              # 注册入口：install_dom_bindings(scope, dom)
├── node.rs             # Node 基类（nodeType/parentNode/childNodes/...）
├── element.rs          # Element（tagName/attributes/getAttribute/setAttribute/...）
├── document.rs         # Document（createElement/querySelector/...）
├── text.rs             # Text/Comment 节点
├── event_target.rs     # EventTarget（addEventListener/dispatchEvent）
├── collections.rs      # NodeList/HTMLCollection/NamedNodeMap（原生 live 集合）
└── gc.rs               # NodeId ↔ V8 对象映射 + stale 校验
```

### 3.3 值传递（原生，去 String）

| JS 值 | 当前（String 桥） | 原生绑定 |
|-------|------------------|----------|
| number | `String::from(n)` ↔ parse | `v8::Number` ↔ f64/i32 直接 |
| string | `String` | `v8::String` ↔ `to_rust_string_lossy` |
| boolean | `"0"/"1"` | `v8::Boolean` |
| element | selector/handle String | `v8::Object`（NodeId internal slot） |
| array (NodeList) | `\|`-joined String split | `v8::Array` 或原生集合对象 |
| null/undefined | `""` | `v8::Null`/`v8::Undefined` |

### 3.4 异常传递（解痛点 P5）

原生绑定直接经 V8 `scope.throw_exception(type_error(...))` 传递类型错误（如 `element.appendChild(non-node)`），不经 String 编码 + try-catch 包装。持久 Context 不再因字符串桥的 pending exception 中毒（R2945 包装的根因消除）。

### 3.5 customElements 集成（解痛点 P2）

原生 `HTMLElement` 是 V8 真实 class（FunctionTemplate 构造）。`class MyEl extends HTMLElement` 经 `Reflect.construct` 产生继承 HTMLElement.prototype 的实例，internal slot 存 NodeId。`customElements.define` + createElement('my-el') 实例化自定义 ctor（ctor body 的 `this.appendChild` 操作原生 node）。lifecycle（connectedCallback）经原生 appendChild hook 触发。R2813 的 upgrade defer 解除。

### 3.6 高层 Web API 共存

Fetch/Observer/FontFaceSet/事件循环 等高层 API 保留 shim（js_dom_shim.js），但：
- 操作 element 时改用原生 node 方法（去 `__zw_*` String 调用）。
- 事件 target / mutation target 用原生 node 对象（非 selector String）。
- shim 体量随核心 DOM 迁移逐步萎缩。


---

## 4. 实施计划（分阶段切片，每切片独立 land）

每个切片：kill-switch（env `ZW_NATIVE_DOM=<scope>`）+ 行为对照门（polyfill vs native A/B）+ bench 对照。切片顺序按「热路径优先 + 风险递增」。

| 切片 | 范围 | 风险 | 验证门 | 预期收益 |
|------|------|------|--------|----------|
| **S0 骨架 + PoC** | `dom_bindings/mod.rs` + gc.rs + 一个原生 `Element.nodeType`/`tagName` getter（NodeId internal slot）+ NodeId↔selector 映射 + bench 对照 | 🟢 低（纯增量，默认关） | 单测 + bench 报告 native 单次读取开销 | 管线可用（GC + 值传递） |
| **S1 只读属性族** | element.tagName/nodeName/nodeType/attributes（read-only）原生绑定，shim 改调原生 | 🟢 低 | WPT DOM + 既有 element 单测全绿 | 热路径读取 ~10x |
| **S2 写入 + 子树** | setAttribute/removeAttribute/appendChild/insertBefore/removeChild/childNodes/children 原生 + DomMutation 经原生路径 | 🟡 中（mutation 路径核心） | reftest（DOM 变更驱动重渲染）+ WPT | reconciliation 主体原生化 |
| **S3 查询** | querySelector/querySelectorAll/getElementById 原生（消费 zero_dom 选择器引擎，复用既有） | 🟡 中 | WPT selectors + 既有查询测试 | 高频查询原生化 |
| **S4 EventTarget** | addEventListener/removeEventListener/dispatchEvent 原生 + 事件 target 用原生 node | 🟡 中 | 既有事件测试 + 生命周期事件 | 事件派发去 selector 匹配 |
| **S5 HTMLElement class + customElements upgrade** | 原生 HTMLElement class（可被 extends）+ customElements upgrade + connectedCallback | 🔴 高（class 实例模型） | 新 customElements 测试 + Web Components 兼容 | 解 P2（Web Components） |
| **S6 高层 API 改写** | shim 的 Fetch/Observer/FontFaceSet 等改调原生 node 方法，shim 萎缩 | 🟡 中 | 既有 R2945–R2953 测试 + WPT | 高层 API 去 ser/deser |
| **S7 收尾** | 移除 polyfill 桥死代码（`__zw_*` 无调用方）+ shim 删减 | 🟢 低（清理） | 全量回归 | 维护体量降（P4） |

**里程碑**：S0–S2 = M1（SPA 热路径原生化，React 可跑）；S3–S5 = M2（customElements + Web Components）；S6–S7 = M3（收尾，shim 萎缩）。

### 实施交接（Implementation Handoff）

#### 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险 |
|----------|------|------|------|
| `crates/engine/src/dom_bindings/`（新） | 新增 | 原生绑定模块 | 新代码，需 GC 设计验证 |
| `crates/engine/src/js_dom_shim.js` | 渐进改写/萎缩 | 高层 API 改调原生；S7 删减 | 既有 14137 测试依赖 |
| `crates/engine/src/js_dom_bridge.rs` | 渐进改写/萎缩 | `__zw_*` 逐步被原生替代 | 同上 |
| `crates/script-sandbox/src/v8_runtime.rs` | 可能小改 | 暴露 raw scope/handle 供原生绑定（若 register_callback 不够） | 引擎层，谨慎 |

#### 推荐修改顺序
1. **S0 骨架**——建模块 + gc.rs + PoC getter + bench，验证管线（首个落地步骤）。
2. **S1 只读**——低风险热路径，建立对照门工作流。
3. **S2 写入/子树**——mutation 核心路径，reftest 守。
4. 依序 S3→S7。

#### 首批提交建议

| 提交 | 范围 | 预期结果 | 验证 |
|------|------|----------|------|
| Commit 1（S0） | dom_bindings 骨架 + PoC nodeType/tagName + gc + bench | native 单次读取可用，默认 kill-switch 关 | 单测 + bench 对照（native vs polyfill 开销） |

---

## 5. 风险与缓解

| 风险 | 等级 | 缓解 |
|------|------|------|
| **GC 悬垂/泄漏**——V8 node 对象引用已移除的 Rust NodeId，或 Rust 移除节点后 V8 对象泄漏 | 🔴 高 | weak-persistent + getter 时 NodeId 校验（stale → null/throw）；S0 专项验证（长生命周期 + 节点增删压力测试） |
| **行为不一致**——原生路径与 polyfill 路径语义细微差异（边角） | 🟡 中 | 每切片 A/B 对照门 + WPT 全量；kill-switch 允许即时回退 polyfill |
| **持久 Context 异常语义**——原生异常传递与现有 try-catch 包装交互 | 🟡 中 | S0 验证 throw_exception 不中毒 Isolate；保留包装直至 S6 |
| **性能门禁**——迁移期 perf-gate（`make bench-gate`）须记录新基线 | 🟢 低 | S0 capture 新基线（JS→DOM 桥开销微基准），后续切片走 perf-gate |
| **rusty_v8 API 边角**——FunctionTemplate/ObjectTemplate 的 internal slot / inherited prototype 在 rusty_v8 150.x 的具体 API | 🟡 中 | S0 PoC 先验证可用 API；标 TBD（§6） |

---

## 6. 待定项（TBD）

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|----|------|--------|----------|--------|
| TBD-1 | rusty_v8 150.x 的 ObjectTemplate internal-slot + inherited-prototype 具体 API（FunctionTemplate 继承链） | ✅ 基础 API 已验证（S0）；阻塞 S5（class） | **已验证**（S0 PoC）：`ObjectTemplate::set_internal_field_count` + `Object::set/get_internal_field` + `v8::External::new/value` + `.into()`(Local→Local\<Data\>) + `data.cast::<External>()` 均可用；PoC round-trip NodeId 经 External 存/取通过。FunctionTemplate 继承链（S5 class）仍需 S5 专项验证。 | S5 前专项验证 class 继承（inherited prototype） |
| TBD-2 | GC weak-persistent 在持久 Context 的具体行为（V8 GC 触发时机、weak callback） | ✅ 已验证（S0 GC 设计可推进） | **已验证**（S0 PoC）：`v8::Weak::new(scope, &global)` + `weak.is_empty()`；强引用释放 + `Isolate::low_memory_notification()`（host GC，无需 `--expose-gc`）后 weak 变 empty。`Weak::with_finalizer`/`with_guaranteed_finalizer` 签名已验证（best-effort / guaranteed 语义）；`request_garbage_collection_for_testing` 需 `--expose-gc`（生产避免，PoC 用 low_memory_notification）。 | S1 gc.rs 用 weak + getter 时 stale 校验 |
| TBD-3 | NodeId ↔ selector 双向映射的性能（迁移期每次 node↔proxy 转换开销） | 重要 | 需 bench | S0/S1 bench 含映射开销 |
| TBD-4 | customElements upgrade 的 lifecycle 触发点（原生 appendChild hook 是否影响渲染管线增量更新） | 重要（S5） | 需与渲染管线增量更新协调 | S5 设计子文档 |
| TBD-5 | 是否保留 QuickJS 路径的原生绑定（或仅 V8） | 可选 | QuickJS 是扩展沙箱非页面引擎，本 RFC 默认仅 V8 | 用户确认 |

### S0 PoC 验证结论（2026-08-09，R3095）

**S0 阻塞性 TBD 全部验证通过**，P1b 可进入 S1（dom_bindings 生产化）。PoC 位于 `crates/script-sandbox/src/dom_bindings.rs`（engine 现无直接 v8 访问，经 Sandbox trait；S0 PoC 置 script-sandbox 有 v8，零行为变更、默认不接管线）：

- **TBD-1（internal-slot 值传递）**：`poc_internal_field_round_trip(node_id)` — ObjectTemplate + internal_field_count(1) + External(NodeId) 存 internal slot[0] + `get_internal_field` + `cast::<External>` 读回。Round-trip 12345/0/u32::MAX 通过。证明 NodeId 经 internal slot 传递管线可用（不经 shim 字符串桥）。
- **TBD-2（weak-handle GC 安全）**：`poc_weak_handle_becomes_empty_on_gc()` — Object + Global + `Weak::new` → 强引用释放 + `low_memory_notification` → `weak.is_empty() == true`。证明 Rust 持 weak handle 不阻止回收 + 对象 GC 后 weak 反映 empty（stale 检测基础）。

**关键 API（rusty_v8 150.2.0）**：`v8::scope!` 宏 + `ContextScope` + `ObjectTemplate::new/set_internal_field_count/new_instance` + `Object::set/get_internal_field` + `External::new/value` + `Local::cast/try_cast`（`.into()` upcast）+ `Global::new` + `Weak::new/with_finalizer/with_guaranteed_finalizer/is_empty` + `Isolate::low_memory_notification`。

**S1 架构决策（待）**：engine 现无直接 v8 访问。S1 dom_bindings 生产化需 ① engine 加 v8 dep（feature-gated）直接操纵，或 ② 绑定托管 script-sandbox 经扩展 Sandbox trait 暴露。S0 PoC 在 script-sandbox 验证 API 可行；S1 选型随首切片（PoC 原生 `Element.nodeType`/`tagName` getter 接管线）定。

**注**：本 RFC 标 **草稿**——TBD-1/TBD-2 阻塞性已 S0 验证（上）；TBD-3/4/5 非阻塞，随 S1+ 推进。rally 无人值守下不暂停。

### S1 dom_bindings 生产化结论（2026-08-09，R3096）

**架构决策（Option A 落地）**：engine 加 feature-gated `v8` dep（与 script-sandbox 同版本 150.2.0，Cargo 去重）+ `script-sandbox::ensure_v8_initialized` 提为 `pub`（engine 自建 Isolate 前确保平台初始化）。原生绑定置于 `crates/engine/src/dom_bindings/`（engine 拥有 DOM：`Document`/`NodeId`），getter 经线程局部 DOM 源（`gc.rs`）读真实 DOM。**不选 script-sandbox 托管**：script-sandbox 为通用 JS 沙箱（无 `zero-dom` 依赖），耦合 DOM 内部将反转自然所有权；engine 拥有 DOM，DOM 绑定归属 engine。

**S1 交付**（`crates/engine/src/dom_bindings/{mod.rs, gc.rs}` + bench + tests）：
- **首组原生 getter**：`nodeType`（Element=1，`v8::Integer`）/ `tagName`（HTML 大写，`v8::String`）经 `ObjectTemplate` accessor getter，从 internal slot[0] 读 NodeId → `Document` 直读（**不经 shim 字符串桥**）。
- **NodeId ↔ V8 对象身份映射**（`gc.rs`）：`NodeId`(ffi u64) → `v8::Global<v8::Object>`，同 NodeId 返同对象（spec identity），stale 校验（节点移除 → getter 返 undefined，spec detached）。
- **NodeId↔u64 编解码**：slotmap `KeyData::as_ffi`/`from_ffi`，internal slot 经 `v8::External` ptr 值（无堆分配）。
- **全局工厂** `__zw_native_element_for_id(idStr)`：`get_element_by_id` → NodeId → 创建/查找 native element 对象。
- **kill-switch**：`ZW_NATIVE_DOM` env（默认关 → 零回归）；`install_dom_bindings_if_enabled` 为生产入口，`install_dom_bindings` 为直装（bench/单测）。

**bench 结果（§4 S0 gate 达成）**——native 直读 live Document vs polyfill 重解析 HTML 快照：
- `native_node_type` ~193 ns / `native_tag_name` ~215 ns。
- `polyfill_tag_name` ~3.36 µs（真实 `__zw_get_tag` 路径：每次 `parse_html(dom_html)` 重解析 + `find_by_selector`，P1 根因）。
- **native ~15.6x 快于 polyfill**（215 ns vs 3.36 µs）——超越 RFC §1.3 目标 ≥10x，量化 P1 痛点。

**下一切片（S2 接线）**：S1 已证明 native 管线（值传递 + GC + 真实 DOM 读）可用并量化收益，但**未接 run_page_scripts 生产管线**——发现架构分层 gap：当前 script 执行路径（webview `run_page_scripts_impl`）仅持有序列化 `dom_html` 串，无 live `Document`（`Document` 在 `RenderPipeline.cached_doc`，单独一层）。生产接线需 ① webview 持 live `Document`（`Arc<Mutex<Document>>` 共享或 parse 复刻）+ ② V8Sandbox escape-hatch（`with_context` 暴露持久 Context 的 raw scope 供 install）+ ③ `Box<dyn Sandbox>` 路径取 concrete `V8Sandbox`（或 trait 加 escape-hatch 方法）。中-高复杂度，独立切片。其后 S1 只读属性族（nodeName/attributes）→ S2 写入/子树（reftest 守）。

**关键 API（rusty_v8 150.2.0，S1 新用）**：`ObjectTemplate::set_accessor(key, getter)`（无 scope 参）+ `AccessorNameGetterCallback`（ZST `fn(&mut PinScope, Local<Name>, PropertyCallbackArguments, ReturnValue<Value>)`，状态经线程局部）+ `PropertyCallbackArguments::holder()`（实例对象，读 internal slot）+ `ObjectTemplate::new_instance` + `FunctionTemplate::builder(fn).build().get_function`。

### S2 生产接线结论（2026-08-09，R3097）

**S1 dom_bindings 模块接通 webview 真实页面沙箱**（闭合 S1 「未接 run_page_scripts 生产管线」限制）。原生绑定现可经 `WebViewConfig.native_dom=true` 在 `run_page_scripts` 时安装到页面持久 V8 Context，与 polyfill 桥共存（页面脚本可直接读 `__zw_native_element_for_id('a').nodeType/.tagName`）。

**接线机制（escape-hatch）**：
- **`Sandbox::install_native_bindings(Box<dyn FnOnce(&mut PinScope, Local<Context>)>) -> bool`**（cfg v8，trait 方法，默认 `false`）：通用 escape-hatch，QuickJS 降级 no-op。
- **`V8Sandbox::with_context`**（私有）：进入持久 Context（镜像 `execute` 的 isolate.enter + scope! + resolve_context + ContextScope），调闭包——故安装的模板/全局对后续 `execute` 可见。
- **`install_dom_bindings_from_html`**（engine）：封装 `parse_html`，避免 webview 直接依赖 `zero_dom`。
- **`WebViewConfig.native_dom` + `WebViewBuilder::native_dom`**：kill-switch（默认 `false` → 零回归）。`run_page_scripts_impl` 在 `register_dom_callbacks` 后、脚本执行前，经 escape-hatch 安装。

**验证**：webview 集成测试（v8）`test_native_dom_bindings_wiring_r3097`——`native_dom=true` 页面脚本读 `nodeType=1`/`tagName=DIV`/`SPAN` + 对象身份 `=== true`；`test_native_dom_disabled_by_default_r3097`——默认关 → 工厂 `typeof === 'undefined'`（v8+quickjs 双通过）。全量 `make test` 16128 全绿。

**已知限制（记录，后续切片）**：① **read-only 快照**——`run_page_scripts_impl` re-parse `cached_html` 为独立 `Document`，**不随页面 mutation 同步**（JS 经 polyfill 改 DOM 后，native 读仍是初值）；nodeType/tagName 等稳定属性无碍，写入路径（S3+ mutation 经 native）后续。② **仅接线 `run_page_scripts_impl`**（`dispatch_event` 一次性事件派发路径不接，事件不读 native getter）。③ **QuickJS 后端 no-op**（`install_native_bindings` 默认 `false`；quickjs 无 v8 escape-hatch）。④ 线程局部 `gc.rs` 状态在 webview 单沙箱生命周期内有效（`reset_context` 接入导航重置为后续）。

**下一步**：S3 = selector→NodeId 解析接 full selector 引擎（`querySelector` native，消费 `zero_dom` 选择器，复用 `find_by_selector`）；或 S1 只读属性族扩展（`nodeName`/`attributes`/`getAttribute` native getter）。每切片 kill-switch + make test 零回归。

---

## 7. 与字体栈 RFC 的关系（同级，独立推进）

- 字体栈 RFC（`docs/goal/rendering-compat/fontdue-replacement-scoping.md`）：渲染一致性（reftest 47%→95%）。
- 本 RFC：JS 性能与 Web Components（SPA 可用性）。
- 两者**工作面不重叠**：字体栈改 render-foundation/font 栈；本 RFC 改 engine DOM 桥。可并行推进。
- 优先级：均为 P1b 级（master.md 标注），等用户审批后启动实施。

---

## 8. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1 | 2026-08-08 | 初稿（rally 自主产出，待用户评审） |
