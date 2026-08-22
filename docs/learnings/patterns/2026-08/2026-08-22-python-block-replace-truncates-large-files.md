---
date: 2026-08-22
modules: js-dom, docs
---

# 用 python 块级 replace 编辑大文件把 441KB 控制面截断成 3.4KB

## 问题描述

R168 轮次更新 `docs/goal/js-dom/master.md`（441KB / 847 行）时，用 python 做了一次
「替换文件头到某锚点」的块级编辑：

```python
old_head = src[src.find('# JS/DOM'):src.find('**上轮**: R167')]
src = src.replace(old_head, new_head)
```

锚点 `**上轮**: R167` 在文件中的实际位置与预期不符（前面还有 `**本轮**` 行），
`src.find` 返回的边界把**文件主体整个包进 old_head**——replace 后文件只剩 3.4KB /
8 行。截断版随 docs commit 推到远端（`470b0e0e6`），下一个 session 开工读控制面
时才发现。

## 根因分析

1. **块级 replace 的边界由运行时 `find` 决定，不由作者视角决定**——锚点字符串在
   大文件里多次/错位出现时，切出的块远大于预期，且 replace 成功不报任何警告。
2. 大文件（400KB+）的 diff 在提交前**只抽查了头部**，没有核对行数/字节数的量级
   变化（847 行 → 8 行是两个数量级的丢失，任何一次 `wc -l` 都能拦住）。
3. 同一轮里已经因为「拼接字符串边界不可见」改了三次（先插错位置、再重复、再修）——
   每次修复都在引入新的边界假设。

## 解决方案

1. **从 git 历史恢复**：`git show <good-commit>:path > path`（上一轮 docs commit
   `6431f125f` 是完整版）。恢复后重做记录。
2. **改用行级编辑**：按行 split、用 `startswith` 精确匹配行、插入/替换单行、
   再 join——每一步操作的行数是显式的，不可能误吞文件主体。
3. **提交前量级守门**：对大文件（>10KB）的 docs 变更，`git diff --stat` 的
   insertions/deletions 必须与预期行数同量级（本轮 +30 行预期 vs +2/-845 实际，
   一眼可见）；CI 之外最便宜的防线是 commit 前跑一次 `wc -l` 对比。

## 如何避免

- 编辑 >1000 行的文件时禁用「块级 find/replace」——用行级定位（Edit 工具的
  old_string 唯一匹配，或 python 按行 split + startswith）。
- 多轮修复同一处编辑时停下来：第一次边界错就说明对文件结构的假设错了，
  继续叠加 replace 是在猜。
- 大文件变更提交前 `git diff --stat` 看量级，不是只看 `--check`（空白错误）。
