# headless JS-DOM 行为差异 backlog（R3323 WPT 行为锁实测定位）

- 日期：2026-08-12
- 相关模块：`crates/engine/src/js_dom_shim/`（B-gen shim）、`crates/engine/src/js_dom_bridge/callbacks.rs`（host 回调）
- 相关切片：R3323 js-dom/* WPT 用例行为锁（升级过程实测暴露）

## 问题描述

R3323 升级 js-dom/* WPT 用例为「行为完成→断言预期→失败抛」时，首版按 spec 全行为断言，**5 个用例失败暴露 headless shim 与真浏览器的真实行为差异**。这些差异在生产页面路径（`run_page_scripts` → `generate_js_dom_shim` + callbacks，wpt-runner/reftest 同机制）下可观察，是 P1b escape-hatch 收敛（native DOM 直改）需逐项闭合的目标清单。**记录此 backlog 供 P1b 用户点名后逐项消化**，避免下游重复探测。

注：[`headless-handle-childnodes-limit.md`](./headless-handle-childnodes-limit.md) 记录的是「handle 子挂普通父的 childNodes 读取限制」（R3316，单一根因）；本文件记录的是 R3323 新发现的 4 项**不同根因**差异，与之互补。

## 差异清单

### 1. innerHTML 写后读不回写（mutation-queue stale-read）—— `js-dom/innerhtml-outerhtml`

```js
var el = document.getElementById('content');  // selector-identity 元素
el.innerHTML = '<span>Replaced</span>';
el.innerHTML;  // 仍返旧值 "<p>Original</p>"，非 "<span>Replaced</span>"
```

**根因**：`__zw_set_inner_html(sel, val)`（callbacks.rs:1292）把变更 **push 到 mutations 队列**（`DomMutation::SetInnerHtml`），后续 `apply_dom_mutations` 批量应用到 `dom_html`；而 `__zw_get_inner_html(sel)`（callbacks.rs:1282）**读 `dom_html` 快照**（`html.lock()`）。同一脚本内 set→read 时，变更仍在队列未应用 → 读到 stale 快照。真浏览器同步应用，set→read 立即可见。

**闭合需**：set 时同步应用到 `dom_html`（或在 getter 读时合并 pending mutations）。代价：改 mutation 应用时序（生产变更应用路径，rule 11 风险）—— 属 P1b escape-hatch（native DOM 直改，读写同源）范畴。

### 2. selector-identity 父的 querySelectorAll 不反映 handle 子 —— `js-dom/document-fragment`

```js
var frag = document.createDocumentFragment();
for (var i=0;i<5;i++) frag.appendChild(document.createElement('li'));
document.getElementById('list').appendChild(frag);
document.getElementById('list').querySelectorAll('li').length;  // 0，非 5
```

**根因**：同 [`headless-handle-childnodes-limit.md`](./headless-handle-childnodes-limit.md)——handle 子（createElement/createDocumentFragment 产物）不在 parsed DOM 树，`querySelectorAll` 读 parsed 树故读不到。**容器侧 `fragment.childNodes.length` 可读**（R2927 `_handleChildren` registry 记录容器子），但挂到 selector-identity 父后，父的 selector 查询读 parsed 树不含这些子。

**闭合需**：appendChild 的 handle 子同步插入 host sel 对应的 DOM 节点（持久化 parsed 树），或 querySelectorAll 合并 pending mutations。同属 P1b 范畴。

### 3. shadow root 内容不经宿主 querySelectorAll 查询 —— `js-dom/shadow-dom-basic`

```js
var shadow = host.attachShadow({ mode: 'open' });
shadow.innerHTML = '<p>Shadow content</p>';
shadow.querySelectorAll('p').length;  // 0，非 1
```

**根因**：shadow root 经 `_shadowHandles`/`_fragmentHandles` registry 标识，`innerHTML` 写入 fragment 容器；但 shadow 内容**不渲染、不查询**（渲染管线走 flat `dom_html`，不遍历 shadow 树）。`shadow.querySelectorAll` 读 parsed 树（不含 shadow 子树）。真浏览器 shadow 内 `querySelectorAll` 应查 shadow 树。

**闭合需**：shadow root 的 querySelectorAll 走 shadow 子树（`_handleChildren` 递归），而非宿主 parsed 树。相对独立，可在 shim 内闭合（不触生产变更应用路径），但 shadow 子树多为 handle-only，依赖差异 #2 先闭合。

### 4. MutationObserver takeRecords 合并/部分捕获 —— `js-dom/mutation-observer`

```js
var obs = new MutationObserver(fn);
obs.observe(target, { childList: true, attributes: true });
target.textContent = 'Changed';       // childList
target.setAttribute('data-x', '1');   // attributes
obs.takeRecords().length;  // 1，非 2（spec 应 ≥2）
```

**根因**：headless MO 经 Proxy trap 同步捕获 mutation，但多次 mutation（textContent + setAttribute）可能在同一 checkpoint **合并为单条记录**或部分捕获（textContent 经 characterData/childList trap、setAttribute 经 attribute trap，时序/合并语义非 spec 严格逐条）。真浏览器每次 mutation 产独立记录。

**闭合需**：MO 记录生成改为逐 mutation 独立（不合并），按 spec 时序。shim 内可闭合，但需核实 trap 触发时序与 spec record 生成对齐。

## 影响

- **WPT 测试断言**：上述 4 差异使 js-dom/* 相关 WPT 用例**不能断言 spec 全行为**（R3323 已调准断言到 headless 真行为，仍锁可验证面）。
- **真实页面兼容性**：依赖「set 后立即读」（React state→render 同步读、Vue/lit 模板更新后立即查 DOM）的框架在 headless 生产路径下可能行为偏差；但多数框架经 microtask 批量更新（mutations 在下一 checkpoint 已应用），故实际影响有限。
- **P1b 收敛价值**：4 差异是 escape-hatch 收敛的**具体可量化目标**——P1b native DOM 直改后，读写同源（读写都走 live Document，不经 mutation 队列），差异 #1/#2/#3 自然消除；#4 需独立核实。

## 解决方案（当前：记录，未修）

当前结论：4 差异均记此 backlog，**不作为自主切片 land**——差异 #1/#2 触生产 mutation 应用路径（rule 11 风险），#3/#4 相对独立但依赖 #2 且价值需评估。等 P1b escape-hatch 用户点名（rule 11）后，作为 native DOM 直改的具体验收项逐项闭合。

## 如何避免

- 写 js-dom/* WPT 行为断言前，先核查目标 API 是否触上述差异（set→read 回写 / handle 子查询 / shadow 查询 / MO 记录计数）。若触，断言调准到 headless 真行为（容器侧计数 / 读侧验证 / 身份验证），不强断言 spec 全行为，避免假 fail。
- 实现「set 后同脚本读」类 API 时，确认 mutation 应用模型（队列式 vs 同步），README/doc 注明 headless stale-read 限制。
- 评估 P1b escape-hatch 收敛时，本文件 4 差异作具体验收清单（逐项写 js-dom 行为断言升级到 spec 全行为，全过 = 该差异闭合）。
