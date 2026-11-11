---
date: 2026-11-11
modules: tooling
---

# 复合命令里 `git stash pop` 不能无条件执行：push 失败后 pop 弹出陈年旧 stash

## 问题描述

做「改动是否为本切片引入」的基线对照时，用复合命令暂存本切片改动：
`git stash push <pathspec> >/dev/null && 跑基线测试 ; ... ; git stash pop >/dev/null`。
第一次 `stash push` 因 pathspec 含未跟踪文件直接报错退出（未跟踪文件不在
pathspec 匹配范围），但 `;` 分隔的尾部 `git stash pop` 照样执行——弹出的是
三周前另一分支遗留的 `stash@{0}`（render-residual T71 WIP），在当前分支上
落下两个文件的冲突态（UU），差点误判为切片自身问题。

## 根因分析

两个独立坑叠加：

- `;` 链不传播失败：前段 stash push 失败后，后段 pop 仍无条件执行。
- `stash pop` 操作的是栈顶 `stash@{0}`，与本次 push 无关联性保证；push 没成功
  时 pop 弹的是任意旧条目。stash 条目 applying 时按当前工作区做合并，跨分支
  基底必然可能冲突。

## 解决方案

- pop 只能挂在 push 成功之后：`git stash push ... && (测试; 前滚) && git stash
  pop`，或干脆不用 pop 回滚——未跟踪新文件本就不被 pathspec stash 收纳，
  改用「仅 checkout HEAD 恢复两个已跟踪文件 + 复制备份」的精确交换：
  先 `cp` 我的版本到运行目录，`git checkout HEAD -- <两个文件>`，测试后
  `cp` 回来，diff 核验逐字恒等。
- 冲突态善后：pop 冲突时 stash 条目自动保留（git 不丢栈），确认工作区
  pop 前该文件干净后，`git checkout HEAD -- <冲突文件>` 即精确撤销弹开
  残留，原 stash 不受损。
- 教训通用化：凡是「失败时后果不可逆」的命令（pop、drop、reset 类）一律
  不放进 `;` 复合链的尾部；链式执行用 `&&` 显式声明成功前置。
