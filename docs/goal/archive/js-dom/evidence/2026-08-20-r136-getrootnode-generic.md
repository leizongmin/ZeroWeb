# R136 — dom/nodes getRootNode 泛型 + composed shadow root（5F→0F）

**日期**: 2026-08-20
**里程碑**: M4（WPT dom 上游基线建立与扩展）
**Driving 用例**: `dom/nodes/rootNode.html`（5 subtest，WPT 上游）
**运行入口**: `make testharness-dom FILTER=rootNode`（polyfill）/ `make testharness-dom-native FILTER=rootNode`（native）

## 背景

R135 轮 master.md「下一步计划 (a)」候选簇之一（rootNode 5F）。上轮 session 429
中断，工作区遗留 part03/04/06 三个 `js_dom_shim` 未提交改动（本轮开头核对后确认
为本轮 WIP：getRootNode 泛型的主体实现）。本轮补 native 路径缺口 + 单测 + 全量
验证 + 收口。

## 根因（可达性盲区 + shadow 链断裂 + native 原型链断链）

1. **旧实现仅 proxy 元素可达**——`getRootNode` 只在 part04 proxy get trap 的
   sel 版分支（`_ancestorChain(sel)`），handle-only 元素
   （createElement/createTextNode/createProcessingInstruction/
   createDocumentFragment 产物，sel=null）、document、fragment、text 均
   `getRootNode is not a function`（WPT 5F 全此形态）。
2. **composed 选项不支持**——spec `getRootNode({composed: true})` 须返
   shadow-including root（穿 shadow host 继续上行），旧实现无 options 参数。
3. **shadow root 的 parentNode 语义缺失**——spec ShadowRoot 不在树
   （parentNode 恒 null）；composed 路径经 `.host` 上行不依赖 parentNode，
   但旧 `_parentNodeFor` 对 shadow 容器可能返非 null 值干扰普通 root 判定。
4. **shadow innerHTML 解析子链断裂**——`shadowRoot.innerHTML = '<div>'` 经
   `_zwFragmentAdded` 建的 `_zwMEl` 解析树顶层子 parentNode 指向
   `_zwMBuildBodyTree` 的**内部 body 快照**（tagName=BODY 的 plain object，
   非任何可达节点）——`shadowChild.getRootNode()` 应为 shadowRoot 却沿链走到
   假 body 断裂（普通 root 断言也炸）。
5. **native 叠加路径原型链断链（本轮补）**——native HTMLElement.prototype 是
   FunctionTemplate 产物，原型链直连 Object.prototype（**不经** polyfill
   Node.prototype——native 不注册 Node ctor，`_zwBuiltNodeChain=false` 时 shim
   不建链）。shadow innerHTML 解析子 `_zwMEl` 的原型链经
   HTMLDivElement.prototype → native HTMLElement.prototype，
   `getRootNode`（定义在 polyfill Node.prototype 上）不可达 → native 路径
   "shadowChild.getRootNode is not a function"（polyfill 5P / native 4P+1F
   的分叉根因）。

## 修复（四处，全部 `js_dom_shim` JS 侧）

1. **part03 `Node.prototype.getRootNode` 泛型**——沿 parentNode 链上行到根
   （receiver 分派：proxy 元素经 `_parentNodeFor`、plain 节点经 R84
   defineProperty 反链、document/无父返 this）；环守卫 4096 跳；composed
   选项到根后经 `.host` 继续上行（shadow-including root，无 host 等价普通
   root——best-effort）。
2. **part04 get trap 分支改委托**——`getRootNode` 返
   `globalThis.Node.prototype.getRootNode`（泛型沿 parentNode 上行天然覆盖
   sel 与 handle 双形态）；同文件 shadow root 的 parentNode 恒 null
   （`_shadowHandles[handle]` 判定）。
3. **part03 `_zwFragmentAdded` 顶层子 parentNode 重指宿主容器 proxy**——
   `_wrapHandle(hostHandle)` 幂等缓存（identity 稳定），盖 `__zwFragHostHandle`
   印章后同步重指；解析子沿链上行到 shadow root / fragment 容器即止。
4. **part03 native 原型链幂等补挂**——`!_zwBuiltNodeChain`（native
   HTMLElement 已注册）时 `Object.defineProperty(HTMLElement.prototype,
   'getRootNode', ...)`（own 已有则不动——R130 XMLDocument 常量同款模式；
   polyfill 自建链路径 own 已有零改动）。
5. **part06 `document.getRootNode()`**——document 自身是 root，返自身。

## A/B 验证

- **rootNode**：5F→**0F（5P 双路径 100%**，polyfill + native 同步——native
  需上述第 4 处补挂，首跑 1F 定位原型链断链后修复）。
- **dom/nodes 全量**：polyfill 8459→**8464P（+5，精确对应 rootNode 5
  subtest）** fail 187→188；native 7679→**7684P（+5 同步）**。fail +1 归因：
  MutationObserver-document "parser insertion mutations"（3F/1P 与 R134 轮
  记载的 41P MO-attributes 域不同文件；隔离复跑同结果 3F——**非本轮改动面**
  （本轮零 host 回调 / 零 mutation 路径改动），判并行流（service-worker/layout
  e4e87c2bf 等 5 commits）或 wpt-data 波动，记 R137 复核候选）。
- **跨域回归**：events 423P/28F（复跑；首轮 421 为超时抖动）、collections
  49P/0F、traversal 1589P/15F 与 R135 逐项一致。
- **单测**：engine `test_get_root_node_generic_r136`（11 断言段：四形态
  detached 自根 + fragment 子沿链 + document 自根 + 挂载元素到 document +
  shadow 子无 composed 返 shadowRoot / composed 穿 host 到 document +
  shadow root parentNode 恒 null），首跑即过。

## 教训

1. **proxy get trap 分支是 handle-only 元素的系统性盲区**（R134 `!sel` 短路
   同族）——Node 接口方法优先上 `Node.prototype` 泛型（receiver 分派覆盖双
   形态），get trap 分支只做委托。
2. **解析快照树的 parentNode 指向内部哨兵对象**是沿链 API（getRootNode/
   compareDocumentPosition）的隐雷——`_zwMBuildBodyTree` 的 body 快照非任何
   可达节点，顶层子挂宿主 proxy 是通用修复面（R123 `__zwFragHostHandle`
   印章的 parentNode 版）。
3. **native 叠加路径的原型链断链是常态**——native FunctionTemplate prototype
   直链 Object.prototype，polyfill Node.prototype 上的接口方法对 native 链
   对象不可达；幂等 defineProperty 补挂（own 已有不动）是既定模式（R130
   XMLDocument 常量、本轮 HTMLElement.prototype）。
4. **全量 nodes 计数与账面对比要留并行流漂移余量**——非本轮改动面的 ±1
   波动先隔离复跑 + 归因到 commit 面，不动本轮结论。
