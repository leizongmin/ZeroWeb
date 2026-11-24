---
date: 2026-11-24
modules: engine,dom,docs
---

# 旧基 worktree 驻留多轮后 wholesale 提交＝跨流碰撞；法证三步与修复分工

## 问题描述

wptfyi-css 流（96ee337b4）提交 shim ③ 刀时，工作树停在一天前的旧基（pre-CE），wholesale 落库把 dom 流刚落地的 CE 源面（a3e299907）、part149 钉测、rendering-compat/wptfyi-dom/wptfyi-html 三个流的当日账本与证据一并回退/删除。受害流在源面自修（73f4d5dcf 整文件恢复＋HAS-NEW 合成 blob），但文档/证据欠账留在 origin tip 上无人认领。

此前三轮（r80c1/r81b/r81c）收账时都已注意到树上挂着 42 条非本流 staged blob（「peer 42 条 staged blob 恒等」如实记在案），选择绕行不碰——绕行躲得过本轮提交，躲不过下一次 wholesale。

## 根因分析

1. worktree 在某次操作后停在旧代（文件 mtime 早于当日批次），index 与 worktree 同代；后续轮次只加新内容不清理旧残留。
2. `git commit -a`/无 pathspec 提交以 worktree 现态为准，旧代文件对 HEAD 形成"删除/回退"差异，随刀一并入库＝静默跨流回退。
3. 逐轮绕行的判据（blob 恒等、mtime）只证明"这次不会误伤"，不消除残留本身。

## 解决方案

**树态法证三步**（判定残留性质后再动手）：

1. `git write-tree` 取 index 树 hash，与全历史提交树比对——不匹配任何提交＝混合代残留（部分文件新、部分旧）。
2. 对可疑文件取 index blob OID，`git log --all --find-object=<oid>` 定代：每个 blob 都能落到历史提交＝纯旧树残留零在途新内容；落不到＝真在途工作，必须保留。
3. 残留整包 `git stash push -m "<法证结论>"` 留档（不 pop），随后 FF/merge 到目标基线，worktree 与 index 同步复位。

**修复分工**：源面（crate 代码、钉测）由受害流按 PURE-STALE/HAS-NEW 分类恢复；账本/证据文档由**肇事流**反向修复——纯删除件从碰撞前提交 checkout，改写件取 `git diff <碰撞前> <碰撞提交> -- <file>` 反向 `git apply -R`（dry-run 零 offset/fuzz＋语义 grep＋与碰撞前提交 diff=0 逐位恒等三重验收）。受害流可在自己账本记「对端欠账照读」，但不代修他流文档。

**升格硬门**：任何提交前，porcelain status 全量必须与预期改动面逐一对照；非本流域或非本轮内容的 staged/unstaged 条目先法证后处置，不得带病落库。绕行记账不消除残留，残留必然等来下一次 wholesale。

## 验证

- FF 后恢复面 8 文件 vs 73f4d5dcf diff=0；shim ③ 豁免指纹恰 1、CE 符号回 a3e299907 水平、part149 在位。
- 文档修复：两 master vs db6d5e0fd diff=0；批三十六/三十七标记 grep=3；7 件证据文件复位。
- `git status` 净（除本轮有意修复集）。
