# M6 S5q — QuickJS customElements 五件套 + lifecycle（R64）

**日期**: 2026-08-16
**commit**: `243811c6`
**里程碑**: M6 QuickJS 原生绑定移植（js-dom goal DC-7）第九切片
**证据**: [evidence/2026-08-16-r64-quickjs-s5q-customelements.json](../evidence/2026-08-16-r64-quickjs-s5q-customelements.json)

## 目标

S5q：customElements 五件套（define/get/getName/whenDefined/upgrade）+ lifecycle
（connected/disconnectedCallback）——QuickJS native 域的 Web Components 基础。

## 实现

- **Registry 存 Rust**（`CE_REGISTRY: tag → Persistent<ctor>`）：QuickJS native
  域无 polyfill `_ce_registry`，V8 版「复用 polyfill registry」路径不可镜像，
  Rust 权威 + JS 薄方法面。
- **五件套**：define（tag ASCII 小写规范化）/get/getName（反向 identity 查）/
  whenDefined（PoC 立即 resolve——`Promise::new` + resolve 调用；真 pending
  语义延异步域）/upgrade（PoC no-op）。
- **create 命中 upgrade（PoC 路径）**：generic native 元素
  `set_prototype(ctor.prototype)`——prototype 方法可达 + lifecycle 可派发；
  完整 ctor 执行（super() NodeId 注入链，镜像 V8 UPGRADE_NODE_ID 栈）延后续
  切片（rquickjs `construct` 路线）。
- **Lifecycle**：appendChild/removeChild 成功路径派发 connected/disconnected
  ——子树 DFS（tag 含 `-` fast-path，V8 R3271 镜像）+ CONNECTED_CUSTOM 状态
  （V8 镜像）+ 先标记后派发（防派发中再 mutation 状态错乱）+ this = native
  元素 + 回调缺失静默（spec 可选）。

## PoC 断言

define + get('MY-EL' 大写规范化命中) + getName 反查 + get miss → null；
create('my-el') 命中 → `greet() === 'hi-MY-EL'`（prototype 方法 + native
tagName getter 联动）；append → `conn:MY-EL`；remove → `conn:MY-EL|disc:MY-EL`。

## API 发现

- `Promise::new(&ctx)` 返 `(promise, resolve, reject)` 三元——立即调 resolve
  构造已解析 promise。
- `HashSet::new` 非 const fn（`HashMap::new` 是）——thread_local 的 const 块
  只对 HashMap 可用。

## 验证

engine quickjs **1419** / v8 **2153** 零回归；webview quickjs wiring 绿；
clippy quickjs 矩阵零警告；fmt 无 diff。

## M6 状态：S0q–S5q 全部有 land 实现

S5q 为 PoC 深度（完整 ctor 执行/upgrade 语义/attributeChangedCallback 延后）。
元素面 = 4 工厂 + customElements 五件套 + 13 属性（6 setter）+ 11 方法 +
lifecycle 派发。**M6 剩余 = PoC→生产深度补齐**：完整 ctor 执行链、
capture/bubble 祖先链、Event 构造器族、DOMException 基建、weak/finalizer、
S1q 复合对象（attributes/classList）。
