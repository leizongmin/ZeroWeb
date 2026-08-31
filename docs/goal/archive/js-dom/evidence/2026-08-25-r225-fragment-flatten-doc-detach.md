# R225 Evidence — insertNode 残余 23F 三根因修复（fragment 展平 + doc 级 detach + endOffset 同步）

**日期**: 2026-08-25
**切片**: M4——R225(a) insertNode 剩 23F：docfrag endOffset 5F + doc 级 PI/comment 位序 16F + 30,4
**改动面**: `part03.js`（`_zwMEl` appendChild/insertBefore fragment 展平 + `_makeDetachedDocument` doc 级 insertBefore 先摘除）+ `part05.js`（工厂元素 insertBefore fragment 展平）+ `part06.js`（`syncEnd209` handle-aware identity + fragment newOffset 语义）+ `part23.rs`（回归单测）

## 一、三根因（探针复刻 common.js setupRangeTests xmlDoc/foreignDoc 段实证）

1. **fragment 不展平**（0/4/8/10/15,20 的 endOffset expected 1 got 2/0）：
   `_zwMEl`（part03 5958）与工厂元素（part05 1280）的 insertBefore/appendChild
   把 DocumentFragment **本体**塞进 childNodes 占位（spec
   `concept-node-pre-insert` 对 fragment 是「逐子插入后清空」）——空 df 占一位
   使 tail 的 indexOf 多 1。appendChild 的 R212 展平只存在于部分站点，insertBefore
   全缺。
2. **syncEnd209 的 identity miss**：父 childNodes 视图元素是 `_wrapHandle` 包装
   proxy，与插入时的 raw 节点对象 `===` 恒 false（handle 形态）→ collapsed 插入后
   end 永不同步（endoff 0 形态）。
3. **doc 级 insertBefore 不摘除原父**（25/26/29/31,16/18 的树序分歧）：spec
   pre-insert 的 adopt 步骤「node 有父时先 remove」。`_makeDetachedDocument` 的
   insertBefore（7568）无摘除——xmlDoc 内 PI/comment 移位时**重复入列**
   （探针实证 `[10,7,1,7,8]` 双 PI；修后 `[10,7,1,8]` 单次）。

## 二、修法

1. `_zwMEl` appendChild/insertBefore（part03 5958/5980）+ 工厂元素 insertBefore
   （part05）补 fragment 展平分支（逐子递归 + 清空 childNodes）；
2. `syncEnd209`（part06）改 `_r225Same`（`__zwHandle` 相等判同一）+ fragment 的
   newOffset 语义（`ti225 + nodeLength(node) - 1` → setEnd(idx+1)）；
3. doc 级 insertBefore（part03 7568）入口先 `newNode.parentNode.removeChild(newNode)`
   （spec adopt 步骤）。

**回退记录**：`dom-range-insertnode` 的「node 是 referenceNode 时 ref 前移
nextSibling」步骤已实现并验证（30,4 +2P）但同时回退 28,0（-2P，testDiv sel 容器
proxy trap 对无-handle 节点的 insertBefore(p0, p1) 落 registry 分支使 p0/p1 互换）
——净 0 且引入新形态面，本轮回退，30,4 + 28,0 成对记入 R226（须 proxy trap 与
工厂/doc insertBefore 的 detach+advance 语义连贯实现）。

## 三、验证链（vs R224）

| 项 | R224 | R225 | Δ |
|---|---|---|---|
| Range-insertNode | 1817P | **1838P** | **+21** |
| dom/nodes | 12662P | 12660P | -2（±flake 带，F 数同 57） |
| dom/events | 579P | 578P | -1（±flake 带，R223 轮同款 577→579 波动） |
| dom/collections | 49P | 49P | 0 |
| dom/traversal | 1602P | 1602P | 0 |
| Range-surroundContents | 893P | 893P | 0 |

Range-insertNode 文件级 23F → **2F**（仅 30,4）。净 **+21P**。

- **engine 单测**：**2378 全绿**（新增
  `r225_insert_node_doc_order_and_fragment_flatten`——PI 单次移位 + 空 df
  endOffset=1 + df 不占位三断言）。
- **fmt / clippy**：零警告。

## 四、R226 靶点

- **30,4 + 28,0 成对**：insertNode 的 node-is-referenceNode 前移步骤须与 sel 容器
  proxy trap（part04 3723）的 detach/advance 语义连贯实现（本轮单独实现净 0 且回退
  28,0）。
- **surround ~350F 重聚类**（893P 基线）。
- 深项：customElements 多 registry / :scope query-root / lone-surrogate wire /
  MO-document parser 记录 / insertBefore 自旋。

## 五、commit

3a96d5d7a
