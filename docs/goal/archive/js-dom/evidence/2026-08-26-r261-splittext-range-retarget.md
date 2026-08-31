# R261 Evidence — splitText live-range retarget（mutations-splitText 100%，+20P）

**日期**: 2026-08-26
**切片**: M4——R261(b) splitText 边界调整接线
**改动面**: part03（factory/detached 域 splitText 两段边界更新）+ part04
（proxy 域同款，handle 匹配）+ part23.rs（+1 回归单测）
**commit**: ef1f75fcd

## 一、replaceData 超时排查（R261(a)，归因收口未修）

- **预存项实证**：R255 时代日志（改动前）Range-mutations-replaceData 同样
  超时——**非 R260 调整引入**。
- **切片二分**：slice(0,200)/400/430 均通过（400→800 变体 <90s）；全量
  437（872 变体）超时；last-8 / 425+ 单独跑全过——**累积型慢**（非特定
  用例死循环）。setup 成本实测（iframe 域 100 轮 13ms）排除 setup；
  候选：registry 遍历 × 用例数的常数累积（874 ranges × 437 用例的
  sameNode 三键匹配）或 harness doTest 内其他 O(n²)。诊断窗口耗尽，
  记录归因后续轮再攻（低 ROI——其余 mutations 套件全 100%）。

## 二、splitText 两段边界更新（R261(b)）

WPT Range-mutations.js 的 testSplitText **逐字引证** spec
`dom-text-splittext` 末段，两段语义：

1. **replace-data 段**（无条件——split 等价 replaceData(o, len−o, '')）：
   边界在 (node, off) 且 `o < off ≤ len` → **收到 o**。
2. **split 段**（**仅 parent 非空**——spec「If parent is not null, run
   these substeps」）：**原始** off > o 的边界 → `(尾节点, off − o)`。

**两个中间红态的教训**：
- 首版无条件迁移（漏 detached 分支）→ detached 12F 反向回归——
  spec 的 parent 门是硬边界；
- 次版 split 段判定用**收缩后** off（两段链式）→ parented 8F——WPT
  算法明确用 `originalOffset` 判定与迁移（两段**非链式**）。

**实现**：part03 `_zwAttachCharacterDataMethods.splitText`（p = n.parentNode 门）+ part04 proxy splitText（`_r196PH` 门 + handle 匹配
边界容器）。

## 三、验证（vs R260 基线）

| 项 | R260 | R261 | Δ |
|---|---|---|---|
| Range-mutations-splitText | 96P/20F | **116P/0F** | **+20（100%）** |
| ranges 全量 set-diff | — | — | **+20 F2P / 0 P2F** |
| engine 单测 | 2400 | 2401 | +1 回归单测全绿 |
| fmt / clippy | — | 干净 | — |

## 四、R262 靶点

- **Range-mutations-removeChild**（2P/18F）：collapsed at (paras[0],0) +
  removeChild(paras[0]) → 边界应迁 (parent, index)——spec
  `concept-node-pre-remove` 末段的「removed node 是 boundary node →
  移到 (parent, index)」段（R183 _mode 已有部分现算路径，静态边界域）。
- extractContents 残余 32F / cloneContents 29F 重聚类。
- replaceData 超时（累积型慢，低 ROI 后续）。
