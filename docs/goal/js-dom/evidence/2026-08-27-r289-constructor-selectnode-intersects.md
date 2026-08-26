# R289 Evidence — Range-constructor 初始边界 + selectNodeContents node-length + intersectsNode 严格不等 + 叶子工厂 childNodes 统一（ranges 域收口）

**日期**: 2026-08-27
**切片**: M4——R289(a) `new Range()` 初始边界 (document,0) + (b) Range-selectNode 三层修复（textEl/parsed text 的 childNodes 字段 + selectNodeContents 的 node-length 语义 + doctype 抛）+ (c) intersectsNode 严格不等修正
**改动面**: `part05.js`（_wrapNodeEntry 工厂 childNodes/children）+ `part06.js`（Range 构造器初始边界、_zwRegisterTextEl 工厂 childNodes、selectNodeContents、intersectsNode）+ `part23.rs`（+3 单测）

## 一、修复内容（四个独立根因）

### (a) `new Range()` 初始边界（Range-constructor 1F 全解）

spec Range 构造器「set start to (document, 0), end to (document, 0)」。R183
的该初始化只落在 `document.createRange()`——`new Range()` 漏同步使
startContainer/endContainer 恒 null（WPT Range-constructor 六断言全挂）。

### (b) Range-selectNode 整文件 setup 崩溃 → 304P/0F 100%（三根因递进）

1. **叶子工厂 childNodes 字段缺失**（setup 崩溃根因）：common.js 的
   `nodeLength`/`testTree` 对**每个节点**（含 text/comment）读
   `node.childNodes.length`——两个叶子工厂（part05 `_wrapNodeEntry` 的
   parsed 文本/注释 + part06 `_zwRegisterTextEl` 的 textEl 包装）无
   `childNodes` 字段 → `undefined.length` TypeError 使整文件 setup 崩
   （declared tests: 0）。补 `childNodes: [], children: []`（与
   `_zwMText`/doc.createTextNode 两工厂统一——**四工厂叶子字段面全齐**）。
2. **selectNodeContents 的 endOffset 语义**（144F 簇）：spec
   `dom-range-select-node-contents` 步骤 2「endOffset = length of node」
   （`concept-node-length`：CharacterData = data.length）。旧版恒读
   `childNodes.length`（叶子恒 0）使 #text 112F + #comment 24F +
   somepi 8F 全部「expected N got 0」。
3. **doctype 抛 InvalidNodeTypeError**（12F 簇）：spec 步骤 1「node 是
   doctype → rethrow」缺失（current doc[0]/xmlDoc[0] qorflesnorf × 4
   range 域）。

### (c) intersectsNode 严格不等（intersectsNode-2 1F 全解，2359P 双套件 100%）

WPT Chromium crbug 822510 形态：range [div,0,div,1] 对相邻 s1（占据
[(div,1),(div,2)]）——边界**相接**不算相交。spec（测试引证原文）：true
iff `(parent, offset)` **严格 before** range end **且** `(parent, offset+1)`
**严格 after** range start。旧版否定条件用的是非严格（≥/≤）使相接形态
误判 true。修：`¬after(end, (parent,i))` 或 `¬after((parent,i+1), start)`
→ false（相等边界既非 before 也非 after）。

## 二、验证

| 套件 | R288 | R289 | Δ |
|---|---|---|---|
| Range-constructor | 5P/1F | **6P/0F（100%）** | +1 |
| Range-selectNode | 0P 整文件崩 | **304P/0F（100%）** | +304（解锁） |
| Range-intersectsNode/-2 | 2358P/1F | **2359P/0F（100%）** | +1 |
| Range-deleteContents | 125P | **129P/0F** | +4（childNode 字段连带解锁） |
| Range-extractContents | 187P | **192P/0F** | +5（同上） |
| Range-cloneContents | 187P | **191P/0F** | +4（同上） |
| insertNode/surround/compareBoundaryPoints/set/comparePoint/isPointInRange/collapse/cloneRange | 全 100% | 同 | 持平 |
| engine 单测 | 2423 | **2426** | +3（r289 三单测） |

## 三、dom 全量（单跑）

| 域 | R288 | R289 | Δ |
|---|---|---|---|
| dom 全量 | 53733P/104F | **54041P/101F** | +308 / -3 |
| dom/ranges | 38841P/39F | **39147P/36F** | +306（3 探针 F 持平） |
| dom/nodes | 12662P/57F | 12663P/57F | +1（flaky 回归） |
| dom/events | 578P/7F | 579P/7F | +1 |
| dom/traversal / collections | 1603P/1F / 49P/0F | 同 | 持平 |

ranges 域剩余失败面（全部 pre-existing，非本轮域）：
- `Range-in-shadow-after-the-shadow-removed` 2F：**variant 基建缺失**——
  `<meta name=variant content="?mode=closed/open">` 须按 query 串分别执行，
  runner 不支持 variant → `URLSearchParams.get('mode')` 返 null →
  attachShadow 抛 TypeError。dom/ 全树仅 2 文件用 variant（另一
  events/handler-count.html），独立基建切片。
- `Range-mutations-{insert,delete,append,replace}Data/dataChange` 5F：执行
  超时 90s 环境慢族（R261 已归因）。
- 本地诊断探针（zz-r54*/R222-probe）不计。

## 四、R290 靶点

- **(a) variant 基建最小支持**（runner 识别 `<meta name=variant>` 按
  content 串多趟执行 + location.search 反映 query——解锁 in-shadow 2F +
  events/handler-count）。
- **(b) mutations-data 5F 超时族**（低 ROI 备档）。
- **(c) nodes 域 57F 重聚类**（querySelector-All 4F / MutationObserver 8F /
  Element-remove 2F 等——ranges 域已 100% 收口，nodes 是最大剩余面）。
