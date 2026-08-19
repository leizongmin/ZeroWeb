# M4 切片：Document-getElementById 动态 id 语义（R125）

**完成日期**: 2026-08-19
**里程碑**: M4 — WPT dom 上游基线建立与扩展
**Driving WPT**: `dom/nodes/Document-getElementById.html`（18 subtest，双路径 100%）
**证据**: [../evidence/2026-08-19-r125-nodes-getelementbyid.md](../evidence/2026-08-19-r125-nodes-getelementbyid.md)

## 摘要

上轮 429 中断的 WIP 由本轮收口。getElementById 的查询源（host 快照 + pending-ID 索引）
不反映同 execute 内的 id 变更与子树移除——八面修复（覆盖表 / in-doc 门 / 祖先判定 /
innerHTML·outerHTML 剔旧 / handle outerHTML= / lenient append / 解析节点原型链接 /
`_zwMEl` namespaceURI）。原型链接连带修复 DOMPurify r3019 回归（`instanceof Element`
变真后 `_checkValidNamespace` 消费 `element.namespaceURI`，undefined 误杀元素）。

## 结果

- Document-getElementById：6P/12F → **18P/0F（双路径 100%）**
- dom/nodes：polyfill 7825→**7838P**；native 6107→**6120P**（+13 同步）
- 回归面（traversal/collections/events/classlist/MO/NodeIterator/TreeWalker/contains）
  与 R124 同值零回归
- DOMPurify r3018/r3019 双绿；`make test` 全绿（双矩阵）；fmt/clippy 干净

## 关键决策

- sel-based id 变更走 **latest-wins 覆盖表**（`_zwIdOverrides`）而非重放 mutation 到
  快照——同批 mutation 在 execute 结束才 apply，JS 侧覆盖表是唯一即时权威。
- append 父 selector miss 改 **lenient no-op**（child 仍由 handles 表登记不断链）——
  spec 对 detached 元素 appendChild 合法，旧硬错中止整批 mutation 使页面脚本全挂。
