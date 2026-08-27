# R301 Evidence — MO move-record 兄弟字段（childList Range 双套件全解，23P/2F→25P/0F 100%）

**日期**: 2026-08-27
**切片**: M4——R301(a) MO 剩余 6F 首两件（extractContents/surroundContents childList）
**改动面**: `part04.js`（appendChild move 路径旧父 record 补 prev/next——R294 removeChild 同款判据）+ `part06.js`（surroundContents `_rmSnap` 顺序移除语义）+ `part24.rs`（+1 单测）

## 一、根因两处（sandbox 微任务 flush 探针）

### (a) appendChild move 的旧父 record 无兄弟字段（extractContents）

WPT "Range.extractContents: child and data removal"（n81 形态：CHANN + NNN +
NGED，range 削中段）期望 record `prev=firstChild / next=lastChild`。中段子的
move 经 `_r211frag.appendChild(_r211c)` → appendChild trap 的 **move 分支**——
旧父 record（part04:3510/3513）只发 `{removedNodes:[child]}` **无兄弟字段**。
探针实证 record `pv=null/nx=null`。

**修**：move 分支摘除前从旧父融合 childNodes 定位（identity 优先 + handle/data
内容键回退——R294 removeChild 同款）补 `previousSibling/nextSibling`。**这是
move 路径的通用修复**（所有 appendChild/insertBefore 移动语义的旧父 record
受益，非 extract 专用——首版在 extract 循环内显式 notify 与既有触发器**双重
record**，回退后统一修触发源）。

### (b) surroundContents 快照一次性取移除前兄弟（顺序语义错）

WPT "Range.surroundContents"（n100 = s1+s2，全范围包围）期望三 records：
s1 移除（pv=null, nx=s2）/ s2 移除（**pv=null**——s1 已移除后无左邻）/ added。
旧 `_rmSnap` 一次捕获全部移除前兄弟 → record2.pv 停留 s1。

**修**：prev 沿 previousSibling 链跳过**本移除集内**节点（顺序移除后不可见）；
next 保持原值（后续移除在本 record 时刻仍在树上，spec 顺序语义）。

## 二、验证

| 套件 | 基线 | R301 | Δ |
|---|---|---|---|
| **MutationObserver-childList** | 23P/2F | **25P/0F（100%，双跑稳定）** | +2P/-2F |
| MutationObserver 全族 | 112P/8F | 114P/6F | 恰 -2（剩 cross-realm/disconnect/document×3/inner-outer 各 1，独立域） |
| Range-mutations（move record 消费方） | 342P/5F | 342P/5F | 持平 |
| Range-insertNode / surroundContents | 1841/1840P 0F | 同 | 持平 ✓ |
| MutationObserver-inner-outer | 1P/1F | 1P/1F | 持平（wrapper identity 项，本轮记档不碰） |
| engine 单测 | 2438 | **2439**（r301 单测：extract 中段 record 兄弟齐 + surround 三 record 顺序语义） | +1 |
| make test | — | 1F = XOpenDisplayFailed 环境项 | 持平 |
| fmt / clippy | — | 干净 | — |

## 三、记档

- **MO 剩余 4F 归属**：cross-realm-callback 1F（iframe realm 的
  `frames[0].MutationObserver is not a constructor`——iframe win 的构造器面，
  R295 per-realm 构造器家族后续）；disconnect 1F（record 丢弃语义）；
  document 3F（parser insertion——parse-time MO 基建，深结构已记档）；
  inner-outer 1F（addedNodes wrapper identity）。
- **教训（首版回退）**：在调用方循环内显式补 notify 前必须先确认**既有触发器**
  是否已发同 record（探针 rmId 双 NNN 实证双发）——修触发源优于修调用方叠加。
