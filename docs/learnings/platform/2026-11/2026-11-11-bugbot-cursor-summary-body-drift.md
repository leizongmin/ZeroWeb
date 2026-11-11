---
date: 2026-11-11
modules: platform
---

# Bugbot CURSOR_SUMMARY 注记 append 致 PR body 漂移：恢复协议 = edit 字节恒等 → 立即重跑 ready

## 问题描述

T55..T58 四个 PR（#294 等）交付链同坑：`gh pr ready` 通过后数十秒内，Cursor
Bugbot 向 PR body 尾部 append `<!-- CURSOR_SUMMARY -->...<!-- /CURSOR_SUMMARY -->`
HTML 注记块（数十至数百字节），下一步 deliveryCheck 的 body 字节恒等校验
（如 3796 vs 2917 字节）随之失败，`ready` 被拦——T58 期第 4 次复现
（`delivery/t58-ready-bugbot-intrusion.json` 存证，REAL_ERROR 'Remote PR
identity or body changed'）。

## 根因分析

GitHub 外部 App（Bugbot）按 commit 触发自动审查，审查完成即编辑 PR body
追加注记——时点恰落在「发布链校验 body → ready」与下一校验读 body 的窗口
里。deliveryCheck 的 body 恒等锚（发布时回读字节）被 App 单方面改写，任何
等待/重试都救不回：锚已经漂移，且注记会随下一次审查再 append。

关键机制：**Bugbot 按 commit SHA 审查，body 编辑不重触发审查**——这正是
恢复协议成立的前提。

## 解决方案

恢复协议（T55..T58 四次走通）：

1. `gh pr edit <n> --body-file <发布正文原文件>` 恢复字节恒等（以发布时
   body 文件为准，不含 CURSOR_SUMMARY 块）；
2. **立即**重跑 ready——窗口内 Bot 下一轮审查尚未完成，不会再 append；
3. 若再次被 append 拦下，重复 1-2（实测最多一次即过）。

不要走的弯路：改校验器放过注记（锚语义被稀释，且注记内容不受控）；
等 Bot「冷静期」（注记不消失，漂移不可自愈）。

## 如何避免

- 发布→ready 动作链压缩到同一脚本内连续执行，不留数十秒人工间隔；
- ready 失败时先查 body 字节差（`gh pr view --json body` 回读比对发布稿），
  尾部多出 `<!-- CURSOR_SUMMARY -->` 即按本协议恢复，不排查引擎/校验器侧；
- 校验器保持字节恒等口径不放宽，注记属外部写入，恢复正文即恢复锚。
