---
date: 2026-11-09
modules: tests/wpt-runner,build-support
---

# 批脚本二进制特征串选串须先实测源文件单行连续存在

## 问题描述

F7 delete-merge-div 门禁批（batch-f7-delmerge-gate-v1，2026-11-09）跑满 60 分钟，probe 七腿、knife11、ilb/ip/ihtml/exec×3 零漂移、mt judge（F2P=6 恰机制预期、P2F=0）、calibrate 白名单、iol/iul、enginelib 名册、battery 七腿全绿，最后二进制身份腿 ABORT：`grep -aqF '壳在块内任一层' target/release/zero-wpt-runner` 未命中。刀体行为确在产物内（mt F2P=6 与 probe 全绿独立佐证），纯身份判定误报。

## 根因分析

sig 串选自提交标题措辞「幸存壳在块内**任一层**即占位」，但 part01f.js:501-502 的注释实际排版为：

```
// <font><br></font> 双占位）。⑤ F7① 扩面（2026-11-09）：壳在块内
// 任一层同样即占位——multitest flex/grid 4 键删净后逐迭代补块级 br
```

换行 + 注释前缀 `// ` 把目标字节序列劈成两段，连续串在源文件与二进制（include_str! 原文内嵌）中均为 0 次，`grep -F` 必败。三个诱因叠加：

1. 选串来源是 commit message / 设计文档措辞，不是源文件逐字摘录；
2. 刀体过 2000 行合规做过注释精简，注释排版在脚本定稿前后可能变_rewrite_；
3. 跨行注释的换行点在人读时不可见，人眼校对发现不了。

伴随坑：grep 二进制必须带 `-a`（配 `LC_ALL=C`）；无 `-a` 时 grep 对二进制文件的匹配语义会假零（同批诊断轮独立留档）。另注意批运行后脚本 mtime 变化≠运行中被改——本例 09:53:20 的修改是 ABORT（09:50:45）之后的补门修复，时序对账避免误判「bash 读到半新半旧脚本」。

## 解决方案

批脚本 sig 门选串三律：

1. **入脚本前先实测**：`grep -c '候选串' 源文件` 必须非零（单行连续存在），再 `grep -aqF '候选串' 产物` 预验；两步都在脚本定稿前做，不靠 commit message 措辞。
2. **优先非注释 token**：代码标识符（如 `shellExitDesc`）、构建期不变的字符串字面量优先于注释片段——注释会被精简/重排，标识符与行为绑定更稳。
3. **假 ABORT 处置**：替换串补跑单腿追加原批日志（`ALL-DONE` 标注 recovery-rerun），不整批重跑；判读/evidence 注明「措辞 bug 非引擎面」。多补一次会产生第二个 ALL-DONE，判读方需按日志段区分。
