---
date: 2026-09-30
modules: ""
---

# `gh pr edit --body-file` 二次编辑会覆盖 `--attach` 已重写的附件 URL

## 问题描述

用 `gh pr edit --body-file body.md --attach img.png` 创建带截图的 PR 后，gh 会把
正文中匹配 `--attach` 参数的本地图片引用改写为 GitHub 持久附件 URL
（`https://github.com/user-attachments/assets/<uuid>`）——但**只改远端正文**，
本地 `body.md` 文件不变。随后任何一次「拿旧本地 body 文件再跑
`gh pr edit --body-file`（不带 `--attach`）」（例如改个错别字）都会把远端已重写的
附件 URL 打回本地相对路径，PR 图片全部裂图。

## 根因分析

`--attach` 的 URL 改写是「编辑时一次性原位替换」，不是对本地文件的持久变换；
本地 body 文件与远端正文在上传后即分叉。任何基于过期本地文件的后续 `--body-file`
编辑都是整体替换，静默回退改写结果。

## 解决方案

- 需要再次编辑正文时，**先从远端拉最新正文**再改：
  `gh pr view <N> --json body -q .body > body-v2.md`。
- 或该次编辑**始终同传全部 `--attach` 参数**，让 gh 重新执行原位替换。
- 编辑后必须回读校验：正文中 `user-attachments/assets` 引用数 = 附件数、
  本地路径引用数为 0。

## 如何避免

把「PR 带附件」视为不可变组合操作：body 与 attach 必须同一次调用交付；
任何后续 body 编辑默认先回读远端。
