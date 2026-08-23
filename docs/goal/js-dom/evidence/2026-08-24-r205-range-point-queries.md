# R205 Evidence — Range 点查询三方法的边界点比较重写（M4）

**日期**: 2026-08-24
**切片**: M4——isPointInRange/intersectsNode/comparePoint 三方法重写；**全量 47049P/7489F → 49534P/5005F（净 +2485P，零新增失败文件，intersectsNode + shadow 两文件全绿）**
**改动面**: `part06.js`（三方法边界点比较重写 + doctype 抛错 + collapsed 修正）+ `part21.rs`（单测）

## 一、三方法重写（复用 R203 `_zwRangeBpAfter`）

| 方法 | 旧实现 | 新实现 | 效果 |
|------|--------|--------|------|
| isPointInRange | 跨容器 best-effort true | point 在 start 前 / end 后返 false（树序比较） | 830→34F |
| intersectsNode | node 父索引直接比 range offset（坐标系错位） | node 占据 [(parent,i),(parent,i+1)]，与 range 区间交判定 | 186→1F |
| comparePoint | cDP best-effort + 方向位反 | before-start→-1 / after-end→1 树序比较 + doctype 抛错 | 496→124F |

**两个 spec 纠正**：
1. **intersectsNode 的 collapsed 前置 false 移除**——现行 spec 无此步骤：collapsed
   range 仍与**边界点所在节点**相交（node paras[0] + range collapsed 在其 text 子 →
   true）。R178 首版自加的步骤与 WPT 186F 全冲突。
2. **doctype 抛错的步骤序**（isPointInRange）——root 不同先返 false（foreign
   doctype 不抛），同 root doctype 才抛 InvalidNodeTypeError。首版 throw 前置使
   foreign doctype 也抛（214F 回归当轮抓回重排）。

## 二、A/B 与全量

- 全量 polyfill **49534P/5005F** / native **49534P**（fail 集逐行相同）
- vs R204：零新增失败文件；intersectsNode/intersectsNode-shadow 双文件全绿
- zero-engine 2345 单测全绿（含新 `test_range_point_queries_bptree_r205`——
  before-start/after-end/同点 + comparePoint 三态 + doctype 双方法 + collapsed 相交）
- fmt/clippy 干净；make test 除 XOpenDisplayFailed 环境项全绿

## 三、剩余（R206 输入）

- **surroundContents/insertNode 各 1840F**：`iframe.contentWindow.setupRangeTests
  is not a function` × 920 + `typeof Range ... undefined` × 920——srcdoc iframe
  子文档域（contentWindow 缺 common.js 求值）
- isPointInRange 剩 34F（`nodeB.compareDocumentPosition is not a function` 形态）+
  comparePoint 剩 124F 混合形态
- Range-set 剩 1369F、mutations 族残余

## 四、commit

`0242f8ffa`
