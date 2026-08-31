# R315 Evidence — regraft 后 handle 兄弟链 identity 归一（R314 缺口 1 正面修复，traversal 1F 收敛至 root 边界残余）

**日期**: 2026-08-28
**切片**: M4——R315(a) TreeWalker-walking-outside-a-tree（Acid3 6a）的 regraft 兄弟链修复
**改动面**: `part05.js`（`_recordHandleChild` 一处 identity 翻转）+ `part24.rs`（r315 六步回归 + r314 归因断言更新为期望值链）

## 一、根因链（探针七轮逐步实断）

WPT 序列：建树（doc>head,body / head>title / body>p）→ walker 绑 body → `doc.removeChild(body)`
→ `w.lastChild()` → `doc.appendChild(p)`（**regraft**）→ `p.previousSibling` 期望 `head`、
`w.previousNode()` 期望 `title`。

修复前实测 `pPrevSib=null / prevNode=null`。逐层 trace（B/W/ENTER-F/STEP0/F-OUT 打点链）：

1. `_recordHandleChild` 执行且 `_proxyCache['@h']` 命中（`cached=true`）→ 非 cache miss
2. sibling getter 进入 R79 分支、父/childNodes 视图正常（`W=obj,cn=2`）
3. **`F-OUT,f=-1`：`doc.childNodes`=[HEAD,P] 里 P 的成员与 `_makeProxy(null,'__n4')` 产物
   `===` 不等**（`k1eq=false`；成员 handle 相同 `k1h=__n4`、原型同、tagName 同）
4. `_recordHandleChild` 检测 `cached proxy !== child` 命中（`SPLIT:__n4`）→ **同 handle 双
   proxy 分裂实锤**

**完整机制**：R52 消零语义（GR3 泄漏修复）在 remove 后清 `_proxyCache['@h']`，其注释假设
「同 handle 不会再被访问（节点已消零）」。但 regraft 场景：remove 后任何属性读（探针的
`w.lastChild()` 等）触发 `_makeProxy` cache miss → 重建 proxy B 并入缓存；页面持有的旧
proxy A 从此与 B 分裂。重挂时页面把 A 传入 appendChild → registry/反链/live-NL 的成员
都是 A → 此后 `_makeProxy` 恒返 B → sibling getter 的 `kids[i] === self` 恒 miss。

## 二、修复（part05.js，一处）

`_recordHandleChild` 入口：缓存命中但 `!== child` → **以页面传入的 child（A）翻转缓存**。
正确性依据：页面视角 identity = A（registry/反链成员都是 A）；B 是 remove 窗口期的孤儿
重建（页面不持有）；live-NL 数组的 stale B 成员在下次 refresh 时按 `arr[i] !== view[i]`
覆写为 A，天然收敛。R52 的堆泄漏修复语义不受影响（消零仍清缓存；只有**重挂**才翻转——
重挂即节点复活，缓存本就该指向活对象）。

## 三、A/B

| 套件 | 修复前 | 修复后 | Δ |
|---|---|---|---|
| TreeWalker-walking-outside-a-tree | 0P/1F（prevNode=null 断言挂） | 五步推进（prevNode=TITLE ✓）| 残余 root 边界面见 §四 |
| TreeWalker.html | 761P | 761P | 持平 |
| NodeIterator 全族 | 795P/0F | 795P/0F | 持平 |
| Range-mutations 八套件 | 全 100%（splitText 116/insertBefore 76/appendChild 70/replaceChild 60/removeChild 20） | 同 | 持平（R52 性能域无回归）|
| Range-deleteContents/insertNode/surroundContents | 125/1840/1840 全 0F | 同 | 持平 |
| ParentNode-querySelector-All / Element-matches | 1975P / 669P | 同 | 持平 |
| Event-dispatch 全族 | 全 Pass | 同 | 持平 |
| MutationObserver | 4F（既存备档） | 同 4F | 持平 |
| engine 单测 --lib | 2453 | **2454** | +1（r315 六步回归）|
| quickjs 矩阵（clippy + test）| — | 全绿 | — |
| fmt / clippy（v8）| — | 干净 | — |

## 四、残余面（R316 候选）

WPT 六步的后三步（`p.appendChild(body)` 重挂 root 后 `nextNode` 期望 P→**BODY**、
`previousNode` 期望 null）实测 `nextNode2=null / prevNode2=TITLE`：root（body）被移入
p 子树后，walker 的 root 边界判定与 currentNode 位置语义在「root 不再是 currentNode
祖先」形态下的重检（R314 root 止步的对称面——当前实现按 root 子树归属剪枝，WPT 期望
按「root 一度到达」语义重走）。属 TreeWalker 导航 oracle 的边界精化，独立小切片。

## 五、教训

1. **「消零后不再访问」假设会过期**——R52（性能修复）的合法性前提被 R314 的 regraft
   形态打破。性能修复的假设要在后续功能修复时复核（同 R313「无变体注释会过期」教训
   的性能域变体）。
2. **identity 分裂的探针序**：`F-OUT,f=-1`（匹配失败）→ 成员 handle 相同（`k1h` 同）→
   `SPLIT` 检测（缓存对象 !== 传入对象）——三步定位双 proxy 分裂，比栈捕获更快。
3. **临时打点清理必须 git diff 复核**——本轮 python 截取脚本误删 part24.rs 既有测试
   （-1236 行），`git checkout` 恢复后重放（R308「改 shim 须重建 binary」同款的
   「脚本改文件须 diff 复核」教训）。
