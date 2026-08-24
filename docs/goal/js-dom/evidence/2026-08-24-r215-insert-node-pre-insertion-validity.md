# R215 Evidence — insertNode 的 ensure-pre-insertion validity 前置（P→DIV→P 环根因）

**日期**: 2026-08-24
**切片**: M4——R215(a) restoreIframe 累积循环定位收口（真根因是 **parentNode 环**）+ insertNode HRE 336F 簇
**改动面**: `part06.js`（insertNode 校验四件 + surround 叶子路径抑制序）+ `part21.rs`（回归单测）

## 一、根因（upwalk 探针一行暴露）

R214 记的「restoreIframe 累积循环」（探针 300 轮复刻 clean）不是累积——真根因在
**单轮**：8,9 形态（range=detachedPara1.firstChild, node=detachedDiv）——host
insertNode 旧版 splitText 路径无校验直接 `parent_.insertBefore(node, tail)`：
`para.insertBefore(div, tail)` 使 `div.parentNode = para` 而 `div` 仍含 `para`
为子 → **P→DIV→P→DIV parentNode 环**（upwalk 探针 101 hops + chain dump
`#text→P→DIV→P→DIV…` 实证）。后续 sim 的 `isInclusiveAncestor` 上行 walk 在环上
栈溢出——表现为「Maximum call stack」。

sim 侧同形态 `ensurePreInsertionValidity` 检出 `div` 是 `para` 的 inclusive
ancestor → HRE。host 缺整个校验族（336F HRE 簇同根因）。

## 二、实现

**insertNode 校验四件**（spec `dom-node-pre-insert`，splitText 变更**前**执行）：
1. parent 非 Element/Document/DocumentFragment → HRE
2. **node 是 parent 的 inclusive ancestor → HRE（环检测，上行 walk guard 128）**
3. Text/CDATA 入 Document → HRE
4. Doctype 入非 Document → HRE

**surroundContents 叶子 newParent 路径的序保持**：sim 在步骤 3（extract 变更树）
之后步骤 5 才抛——R215 校验会拦在变更前，经 `_r215NoValidate` 帧标志对本路径
抑制（恢复 R212 先变更后抛序；insertNode 直接调用的校验不受影响）。首版
「显式 extract 后抛」过度（829→748），帧标志版精确（829→865）。

## 三、验证链

- **单文件**：insertNode **628P→902P（+274！）**；surround 829→**865P（+36）**；
  extract 86→**98**（+12）；clone 137→**149**（+12）；delete 58→56（-2）；
  mutations 456 不变
- **全量（polyfill）**：R214 基线 51302P/3737F/20T → **51575P/3463F/21T
  （净 +273P，P2F=0 纯增）**——F2P 274 全在 Range-insertNode
- **全量（native 对照）**：**51576P/3463F/20T**——flips 仅 1 既存 flaky
  （insertBefore-iframe-crash）
- **engine 单测**：2354 全绿（新增
  `test_insert_node_pre_insertion_validity_r215`——祖先自插入 HRE 不建环 +
  upwalk 有界 + Text 入 Document HRE 三断言组）
- **fmt / clippy**：零 diff / 零警告
- **make test**：（见 master.md R215 行）

## 四、教训

「累积循环」假设被 300 轮循环复刻推翻——**先复刻再定位**；upwalk chain dump
（前 12 站）一行暴露环结构，比堆栈推断快。

## 五、commit

（落盘时待填）
