# M3 切片 R90 — Web Components 端到端验收首切片（DC-2 开刀）

**日期**: 2026-08-17
**Commit**: `dc7775c9`

## 资产

`tests/integration/src/e2e_web_components.rs`——5 断言组经 WebView 真实页面脚本管线（load_html + run_page_scripts_strict）：

1. define + createElement 升级 + lifecycle（connected/disconnected + isConnected 前后）
2. observedAttributes + attributeChangedCallback 三参（null→gold / gold→null）+ connectedCallback 属性反射渲染
3. Shadow DOM attachShadow(open) + shadowRoot 读 + 子树 querySelector
4. registry get/getName 反查 + get-miss
5. CustomEvent dispatch + detail 传递

常驻 `make test`（zero-integration-tests --lib 772 全绿含 5 新增）。断言形态 = 页面侧收集 `__wcReport` 字符串逐组核验（可复现、可 diff）。

## 基线暴露的三缺口（全部修复）

| # | 缺口 | 修复 |
|---|------|------|
| ① | createElement 不查 CE registry（generic proxy 返回，升级路径断） | 命中注册 tag → `setPrototypeOf(el, ctor.prototype)` 立即升级（与 manual upgrade(root) 同型） |
| ② | getPrototypeOf trap 动态派发覆盖 setPrototypeOf（instanceof 仍 false） | trap 先查 CE registry（tag 命中返 ctor.prototype，先于 iface 表）——parser 建元素与升级元素同源 |
| ③ | connectedCallback 时刻 isConnected 读 false（旧实现依赖 layout 后的 rect） | 先沿 `_zwNodeParent` 反链上行（到 sel 节点即 connected），rect 探测降回落 |

## 已知限制（记档）

- **class ctor 体不可重放**：`B.call(el)` 抛 "cannot be invoked without 'new'"；`Reflect.construct` 新建 this 而非复用 el。升级 = 原型挂接 + connectedCallback 承载初始化（imperative WC 模式）。lit/stencil 的 constructor 内初始化面受限——后续可探索 Proxy-ctor 桥（new 时捕获 this 再移植）。
- **shadow 树内 isConnected false**：反链未串过 shadow 容器→host 边界（Node-isConnected-shadow-dom 2F）。
- whenDefined resolve 断言未闭环（微任务时序，下轮补 flush 后读）。

## 回归

- integration 772 全绿（+5）；engine v8 2187 / quickjs 1427 全绿
- dom/traversal per-case 不变（1593P/11F）；dom/events/collections 逐字节一致
- dom/nodes 净 +1（Node-isConnected ordinary-child 修复；name-validation flake 波动）
- fmt 无 diff；clippy 双矩阵零警告；pre-commit-guard PASS
