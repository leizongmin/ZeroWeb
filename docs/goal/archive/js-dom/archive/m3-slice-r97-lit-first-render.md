# M3 R97 — lit 首渲染落地（walker 跨树重定位 + fragment 视图插入）

**日期**: 2026-08-17
**Commit**: `42f303db2`
**Milestone**: M3（真实 SPA / Web Components 端到端验收）
**前置**: R95（lit e2e 首切片 + 异步 update 链诊断）、R96（get trap 原型链污染收口）

## 背景与目标

R95 land 了真实 lit 库 e2e 首切片（bundle 求值 + ctor 桥 + template.content），
诊断出异步 update 链阻塞：`await this._$ES` 恢复后首渲染不落地（shadow root 恒空）。
R96 闭合了 hasOwnProperty/Object.prototype 表查污染层。本轮（R97）接续：定位剩余
阻塞并把 lit 首渲染真正打通。

## 探针推进（staged probes，全部已转正或删除）

1. **探针 1 修正**：R96 遗留探针的 `LitElement.prototype.__proto__.prototype`
   多一层 `.prototype`（undefined 上 getOwnPropertyDescriptor 抛 TypeError 整段崩）。
   修正后首读数：`eu-own:true`（R96 修复生效）+ `pending:true|rr-kids:0`。
2. **探针 2（post-drain 时机）**：`__litReport` 改在第二次 execute 读——整条
   update 链全通（performUpdate→shouldUpdate→update→render→_$AE→updateComplete:true），
   缩小到「render 产物不 commit」。
3. **探针 3**：lit-html 核心机制单点——全局 TreeWalker `P.currentNode = fragment`
   后 `nextNode()` 返回 `HTML>HEAD>BODY...`（重定位完全失效）+ `insertBefore`
   marker 后容器 kids:0。
4. **探针 4**：lit-html `render()` 直测（不经 LitElement）逐步收窄到 marker-only
   （fragment 插入 no-op）。

## 四重根因与修复

| # | 根因 | 修复 |
|---|------|------|
| 1 | TreeWalker currentNode 跨树重定位：order 快照不含 root 外节点，nextNode 从 document 头走。lit-html 用**单个全局 TreeWalker**（root=document）经 `P.currentNode = fragment` 遍历 template parts | part06 `nextNodeOffOrder()`：`orderPos<0 && relocated` 时导航式步进（firstChild/nextSibling/parentNode 状态机——上行后 descend=false 防重入子树；REJECT/0 剪子树按 walker 类型分叉） |
| 2 | `insertBefore(node, null)` handle 父不记 registry（appendChild 有 `_recordHandleChild`）——marker 插入后容器 childNodes 漏子 | 补 `_recordHandleChild`（对齐 appendChild） |
| 3 | 无 handle fragment 视图（template.content 派生）插入静默 no-op——lit commit 的 `marker.parentNode.insertBefore(importedFragment, endNode)` 永不落地。子节点也是 `_zwMEl` 解析对象（无 handle） | appendChild/insertBefore 各加无 handle fragment 分支：子节点展开入 registry（handle 子记反链；_zwMEl 子直接 push；带位按 registry 位置 splice） |
| 4 | `_zwMEl` 缺 `hasAttributes()`/`getAttributeNames()`——lit Template 属性 parts 提取 TypeError | part03 `_zwMEl` 补两方法 |

**A/B 捕获的同轮回归**：`insertBefore(b, b)` 自引用（spec no-op）被新 registry
分支重复插入——WPT Node-insertBefore "before itself" A/B 捕获（expected [b,c]
got [b,b,c]），入口 `newNode === refNode` 早退修复。

## 验收

- **lit e2e（常驻资产，`make test` 内）**：
  - `lit_first_render_lands`（新）：post-drain `pending:false|hasUpdated:true|rr-kids:2|p-tag:P|p-class:greet|p-text:Hello, ZeroWeb!`
  - `lit_html_render_direct`（新）：`kids:2|p:P|text:Hello, World!|ins:1,true,true`
  - R95 既有两组件维持 Pass
- **WPT dom A/B（clean-HEAD 二进制对照）**：traversal 1593→**1595P**（TreeWalker-currentNode ×2 转 Pass）；nodes 6673=base + fail **-24**；events 236 / collections 48 逐项一致
- **单测**：engine v8 **2205** / quickjs **1431** / integration **779** 全绿；`make test` 65 套件 **18086** passed exit 0
- **质量**：fmt 无 diff；clippy 双矩阵零警告；pre-commit-guard PASS

## 教训

1. **post-drain 读数时机**：异步框架链的验收读数必须在 microtask 排水后（第二次
   execute）——同步快照会把「尚未跑」误判为「没落地」。
2. **框架级 TreeWalker 消费模式**：lit-html 的全局单 walker + currentNode 重定位
   是 WPT 用例覆盖不到的真实路径——框架 e2e 是独立验证面。
3. **导航式步进的状态机陷阱**：上行到祖先后再走 firstChild 会重入已遍历子树
   （EM→text→上行 EM→又 firstChild=text 死循环）——上行后必须只横移。
4. **make test 的 product-version 陈旧缓存**：跨午夜构建时 `zero-product-version`
   的昨日 rlib 残留（build.rs 未声明日期变化 rerun）使 embedded version 断言失败，
   `cargo clean -p zero-product-version` 恢复——独立可复现缺陷，与本切片无关，留档。

## 下一步

- lit 响应式更新链（property set → requestUpdate → 二次 render diff commit）
- M3 SPA 面评估（React / Vue 之一代表性页）
