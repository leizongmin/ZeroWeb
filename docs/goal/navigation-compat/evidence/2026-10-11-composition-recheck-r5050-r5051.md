# 组成态追认复核 — t8s1 + style-compat R5050/R5051 组成（2026-10-11）

**背景**：S9 复核（[2026-10-11-m2-s9-anchor-href-lw.md](2026-10-11-m2-s9-anchor-href-lw.md)）
覆盖的是 R5048/R5049 组成态；其后 main 又合入 engine-dom t8s1（DOMParser 快照 adopt，
PR #132）与 style-compat R5050（input/textarea 内容关键字 + 字体相对固有尺寸）、
R5051（::placeholder 伪元素作者样式）——均晚于上次组成态验证。R5050/R5051 触布局
几何，导航语料 scroll-to-fragid 族布局敏感，按 goal 复核惯例追认一轮。

## 运行

- **通道**: `make testharness-navigation`（test-guard 包裹，`--per-proc-mem 4
  --total-mem 8 --time-limit 1800`）
- **树态**: main `1430038a3`（R5051）工作区干净（仅 cdp-protocol 流遗留的未跟踪
  探针脚本，非本 goal 面）
- **二进制**: release `Finished in 0.18s` 零重编译 = 既有 release 即 HEAD 组成
  （上轮已建，规避 [A/B 基线臂 stale 二进制](../../../learnings/bugs/2026-10/2026-10-11-ab-baseline-rebuild-stale-release-binary.md) 坑）
- **墙钟**: ~19 min；原始输出 483 行见
  [2026-10-11-composition-recheck-r5050-r5051.txt](2026-10-11-composition-recheck-r5050-r5051.txt)

## 结果：逐行恒等，零漂移

- 行计数 **408 Pass / 35 Fail / 26 Timeout / 14 NotRun**，与 S9 evidence txt
  **语义 diff = 0**（空白归一后逐行恒等）。
- 上轮 rate-limit 中断前已在同组成跑过两轮（互为逐字节全同）；连同本轮 = 同组成
  **连三跑全同**，确定性成立。
- M2 收口态 **85.7%（408/476）** 在 t8s1 + R5050/R5051 组成上维持。

## 残余处置抽核（本轮顺带）

- the-iframe-element sandbox allow_top_navigation 2F：案体 = `window.open()` popup +
  子 iframe 顶导航 + postMessage + beforeunload——**popup/iframe/beforeunload 三重
  载体**，M3 门控挂账属实。
- the-history-interface 008 Timeout：案体经 `document.write` 拉
  `http://www.<host>/…` 跨主机脚本——runner 单主机形态不可达，P2 挂账定性属实。

## 结论

M2 收口态在当前 main 组成上锚定成立；残余 75 案处置经抽核维持。goal 唯一剩余项
仍为 M3 iframe 深结构切片（用户门控，维持挂起）。
