# R203 Evidence — Range setStart/setEnd crossing 重设 + 边界点序比较（M4）

**日期**: 2026-08-23
**切片**: M4——Range-set 主簇语义修复：**30144P/24395F → 35453P/19085F（净 +5309P/-5310F 零新增失败文件；Range-set 4153→9297P、collapse +54P）；native 35454P fail 集逐字节相同**
**改动面**: `part06.js`（`_zwRangeBpAfter` + `_zwTreeDepth` 新 helper + setStart/setEnd crossing 重设）+ `part21.rs`（单测）

## 一、spec 依据与修复

spec `range-set-start` 步骤 3 / `range-set-end` 镜像：新 start 在当前 end **之后**
（边界点比较，含跨文档形态）时 end 一并设为 (node, offset)；setEnd 向前穿 start
时 start 一并重设。旧 setStart/setEnd 只写自身侧——WPT Range-set 的 "must set
the end node to node too" 6767F 族根因。

## 二、`_zwRangeBpAfter` 边界点序比较（三段判定）

| 分支 | 判定 |
|------|------|
| 同容器 | `offsetA > offsetB` |
| A 是 B 祖先 | (cA,oA) 指向第 oA 个子——索引 < oA 的子树在边界点**前**；B 的「cA 直接子」索引 childB < oA ⇒ A 点在 B 后 |
| B 是 A 祖先 | A 的「cB 直接子」索引 >= oB ⇒ A 在 B 边界点后（该子及其后子树） |
| 其余 | **深度感知双 climb**：深侧先 climb 到同深，再同步上行至共同父，双方直接子索引定序 |
| 跨文档/无共同根 | 恒 after（spec position-of 异树不可比；WPT "or in different document" 变体要求重设触发） |

**算法坑（单测当场抓回）**：首版逐父同步 walk——深浅不一时（t1 in p0 深 2 层 vs
p2 兄弟容器深 1 层）走一步后 xA=p0 vs xB=div 触发 isAncestor 短路**误序**，
`setStart(p0.firstChild, 0)` 错误重设了 end。修正为深度对齐后共同 climb。11 个
规范形态 standalone 全绿 + shim 单测（crossing/backward/non-crossing 三场景）。

## 三、A/B 与全量

- 全量 polyfill **35453P/19085F/21T** / native **35454P**（fail 集逐行相同）
- vs R202：**零新增失败文件**，净 +5309P（Range-set 主簇 + collapse 连带）
- zero-engine 2343 单测全绿（含新 `test_range_set_start_end_crossing_r203`）；
  lit/vue e2e 全绿；fmt/clippy 干净；make test 除 XOpenDisplayFailed 环境项全绿

## 四、剩余簇（R204 输入）

1. **compareBoundaryPoints 9313F**：两形态——`how` 参数 WebIDL enum 转换抛
   NotSupportedError（5272）+ "Creating context/argument range threw"（context
   range 建立路径异常，4041）
2. **foreign-tree Maximum-call-stack ~1009**：foreignDoc 树上 range 操作递归
   （Range-selectNode 整文件死于此）
3. identity 纯形态（expected "[object Object]" vs got 同串不可辨）已大幅收缩——
   主因是 crossing 语义而非 wrapper 基建

## 五、commit

`9ebf34754`
