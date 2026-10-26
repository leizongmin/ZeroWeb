---
date: 2026-10-26
modules: .agents/skills/zeroweb-site-optimizer
---

# 账本变更脚本深拷贝别名：改动落在被丢弃的旧对象上；done 任务与证据文件冻结面

## 问题描述

同族脚本 bug 一日三现（rev44/rev46/rev48 三个完成回执脚本）：`let wf = JSON.parse(read())`
后先 `const op = wf.operations.find(...)` 取引用，再 `wf = JSON.parse(JSON.stringify(wf))`
整体深拷贝替换，随后 `op.status = 'completed'` 改的是**深拷贝前的旧对象**——安装的新
revision 里操作仍是 intended，而校验器对该形状放行（publish 完成态不在其强制面内），
缺陷静默通过 pairwise 校验。后续连坐：修正后给已 done 任务的 evidence 文档原地改写
并重绑 ref，又连吃两级拒绝——「Evidence hash mismatch」（pairwise 重验上一 revision，
其绑定哈希指向被覆盖的旧文件）与「Completed task changed」（done 任务任何字段不可变）。

## 根因分析

reassign 变量不更新既有引用，这是 JS 基础语义，但「取引用 → 深拷贝 → 改引用」的
顺序在只为防突变而深拷贝的脚本里极易写反；校验器对 publish/integrate 完成态与
presented 的耦合核验不完整，使错误形状可过闸。证据面则是硬约束：被任一 revision
绑定过的文件字节不可变（evidence() 逐文件验哈希，且 pairwise 会重验前序 revision），
done 任务的全部字段（含 evidence ref 本身）进入冻结面。

## 解决方案

- 深拷贝**之后**重新查找引用，并在修改后按新拷贝断言落点：
  `wf = structuredClone(wf); const op = wf.operations.find(...); op.status = ...;
  assert(wf.operations.find(...).status === 'completed')`。
- 证据文件一经 revision 绑定即不可变；修正 = 新文件 + 后续记录，绝不原地改写。
- done 是终态冻结：发现已 done 任务的材料有误，接受「已知缺陷 + 修正版留盘 +
  run.md 记录」的前向修复，不回放历史 revision，也不为此换代（换代只用于结构性死锁）。

避免方式：完成回执类脚本统一走「bump(mutate) 高阶函数」模式（拷贝与版本号在函数内
单点处理，mutator 只见新副本）；每次写完账本快照立即用与产品无关的独立断言
（重新 find 后核对状态）自检别名。
