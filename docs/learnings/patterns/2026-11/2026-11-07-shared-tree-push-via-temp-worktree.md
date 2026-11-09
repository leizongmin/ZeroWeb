---
date: 2026-11-07
modules: docs/goal/editing-compat
---

# 共享树多轮并行写作期的提交推送：临时 worktree + detached rebase + HEAD:main

## 问题描述

同一 clone 多个 Rally 轮并行时，工作区常驻他轮未提交改动（刀体脏文件、
在飞草稿）。本地有提交要推 origin，而 origin 每分钟级被他流推进：
`git pull --rebase` 要求干净树；`--autostash` 会把全树脏改动（含他轮在飞
字节）随 rebase 周期整体搬移，pop 窗口与他轮写入并发时整文件覆盖其在飞
编辑。

## 根因分析

autostash 的 stash/pop 作用于**整树**，无法按文件所有权分拣；并发写入者
没有 stash 协调，任何窗口重叠都是静默覆盖。历史先例（memory
rally-parallel-turns-same-session：双执行体 stash 型 A/B 撞车）同构。

## 解决方案

推送自身提交不触碰共享树：

```bash
git worktree add /tmp/push-wt origin/main   # detached，基点=远端顶
cp <我的文件> /tmp/push-wt/<路径>           # 或 cherry-pick 我的提交
cd /tmp/push-wt && git add <文件> && git commit
git push origin HEAD:main                   # detached HEAD 直推，勿推 main 分支名
git worktree remove /tmp/push-wt
```

注意：worktree 是 detached HEAD，`git push origin main` 推的是共享树分支
ref（未 rebase 的旧基）会被拒——必须 `HEAD:main`。origin 再推进时在
worktree 内 `git rebase origin/main` 重试即可。共享树的 branch ref 落后
origin 无需立即修复：他轮 pull --rebase 时对 patch 恒等的本地提交自动
跳过。
