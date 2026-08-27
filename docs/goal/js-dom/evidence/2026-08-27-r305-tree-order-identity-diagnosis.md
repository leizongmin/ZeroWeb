# R305 Evidence — tree-order 4F 诊断轮（wrapper identity 域界定；R167/R296 桥的 iframe 域缺口）

**日期**: 2026-08-27
**切片**: M4/L2——R305(a) handle-append 融合评估（转入 tree-order identity 诊断）
**改动面**: 无代码 land（诊断轮——wpt-data 注入探针，跑完还原）

## 一、诊断数据（assert_unreached 注入，四上下文首分歧点）

WPT `ParentNode-querySelector-All` 的 tree-order 断言（`querySelectorAll("*")`
逐位 === `traverse(root)`（firstChild/nextSibling 递归）产物）：

| 上下文 | 首分歧 | 数据 |
|---|---|---|
| Document | idx 0 | `trav=HTML / res=META`——**结果的 html 元素缺失**（首项即 meta） |
| Detached Element | idx 1 | `trav=DIV / res=DIV / eq=false`（idx2 又 true——同 tag 兄弟间 identity 断） |
| Fragment | idx 0 | `trav=DIV / res=DIV / eq=false` 三连——**全 identity 不一致** |
| In-document | idx 306 | `trav=NULL / res=NULL / eq=false`（顺序对、对象异） |

## 二、域界定

四上下文全部构造在 **iframe contentDocument 工厂域**（`doc =
frame.contentDocument`；Detached/Fragment 的 root = `element.cloneNode(true)`
——工厂克隆树）：

- **Document 上下文**：R296 的结构桥（`_zwWrapCached` 的 html/body/head 直返
  doc 视图对象）只接了 **detached-doc 工厂**的查询包装——content iframe 的
  查询路径未过此桥（html 缺失形态待溯源：可能 `*` 结果枚举源不含 html 或
  桥 key 不匹配）；
- **Detached/Fragment 上下文**：traverse 读 `cloneNode` 产物树节点，查询走
  handle registry（`_handleQueryAll`）——两套对象的 identity 归一 = R291
  定性的「wrapper→视图归一」深结构域（R158/R171/R173 系列，R171 实测 902F
  依赖，须专用切片评估 blast radius）；
- **In-document idx306**：同 identity 族尾部形态。

**与 R304 的关系**：R304（innerHTML 同 turn 视图）已解 sel 容器形态；
tree-order 的缺口在**工厂/克隆域的查询包装 identity**——R220 族的第三个
子域（sel 同 turn ✓ / handle-append 融合 ○ / 工厂查询 identity ○）。

## 三、后续（R306 候选）

1. **最小可评估切片**：content iframe 域的 `*` 查询接入 R296 结构桥
   （html/body/head 三形态——Document 上下文 idx0 的 html 缺失若为桥 key
   问题则是低成本修复）；
2. **深结构主线**：cloneNode 产物树与 handle registry 查询的 identity 归一
   （评估 R171 桥的克隆域扩展 + 902F 依赖的现代形态重测）；
3. 或转 **M1 L2-read-only 主线首切片**（getElementById/querySelector 读
   live——R304 已铺同 turn 视图基础）。

## 四、验证

诊断轮无代码变更；wpt-data 注入已还原（`ParentNode-querySelector-All.js`
backup 比对一致）。
