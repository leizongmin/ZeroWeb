# R249 Evidence — 幽灵 P 写入者栈捕获 + iframe 工厂元素 own removeChild（WPT 净 0）

**日期**: 2026-08-25
**切片**: M4——R249(a) 幽灵 P 的写入者定位与修复
**改动面**: `part05.js`（_zwIframeCreateElement 产物 own removeChild）+ `part23.rs`（r249 单测）
**commit**: 见 master.md 本轮记录

## 一、栈捕获定位（R249-iframe 内联探针 + Error().stack）

R248 遗留问题「surround 期零 removeChild 调用/零 setter 写入而
p0.parentNode 终值 null」——R249 修正探针（run() 时点捕获初始值 + stack）：

1. **run() 时点**：`p0.p@run=DIV, tdKids=7, p0InTd=true`——p0 父链
   **健康**（setup 正常 append），推翻 R248「setup 期已 null」猜想。
2. **surround 期唯一一次写入**：`pSET=null STACK= set(parentNode) ←
   removeChild(<anonymous>:1740) ← Node.prototype.remove(11521) ←
   surroundContents(40403)`——**写入者是 Node.prototype.remove →
   Node.prototype.removeChild 的数组分支**（part03:1740 的
   `if (child.parentNode === this) child.parentNode = null;`）。
3. **机制**：testDiv（iframe 工厂元素）无 own removeChild →
   Node.prototype.removeChild 数组分支执行：`indexOf(child)` 命中后
   `this.childNodes.splice(...)`——但若 `childNodes` 读取返回**视图副本**
   （getter 每次返新数组），splice 落在**副本**上（源列表未动），
   而 `child.parentNode = null` 持久生效——**单向断链**：父链空、
   子列表残留（DIV 6 P vs sim 5 P 的「幽灵 P」）。

## 二、修复

`part05.js` `_zwIframeCreateElement` 产物补 **own removeChild**（与
`_zwMEl` removeChild 同款）：单次读列表 + identity indexOf + **就地
splice** + 父链置空 + 迭代器通知 + NotFoundError/TypeError 校验。

修复后栈验证：`removeChild(<anonymous>:27734)` = 新 own 实现（part05
本地 1310 行），remove 链路走 own 实现。

## 三、验证链（vs R245/R248 基线）

| 项 | 基线 | R249 | Δ |
|---|---|---|---|
| Range-surroundContents | 1806P/34F | 1806P/34F | 净 0（subtest diff=0——13/14,x 仍 Fail：own removeChild 修好 splice，但该形态 host/sim 的树比较另有残余分歧，待下轮探针） |
| ranges 全量 | 40080 行 | 40080 行 | set-diff **0/0** |
| Range-insertNode | 1841P/0F | 1841P | 100% 保持 |
| dom/nodes 失败集 | 57 | 57 | 逐条一致 |
| native surround | 1806P | 1806P | A/B 同值 |
| engine 单测 | 2392 | 2393 | 全绿（新增 r249：splice 就地 + 保序 + 父链 + NotFound/TypeError） |

- fmt/clippy（`-D warnings`）干净；make test 1F 为 XOpenDisplayFailed
  环境项；wpt-data 探针全部还原（R249 标记 0）。

## 四、land 依据

WPT 净 0 但修复真实栈捕获的引擎 bug（单向断链——父链置空而子列表
残留，任何依赖 childNodes 一致性的比较/遍历都被污染）。13/14,x 的
完整翻转还需下轮在 own-removeChild 生效后的新树形态上重新 dump 首差。

## 五、R250 靶点

- 13/14,x 残余：own removeChild 生效后重新 dump 双树首差（幽灵 P 已
  修，新分歧点待定位）。
- 17,x "[object Object]"（isEqualNode 对工厂对象的深层比较）。
- 16,x startOffset 11F。
