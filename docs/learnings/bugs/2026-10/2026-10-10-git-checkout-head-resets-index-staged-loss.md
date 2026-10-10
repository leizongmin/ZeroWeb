---
date: 2026-10-10
modules: tooling
---

# `git checkout HEAD -- <paths>` 误用冲掉暂存区——dangling blob 全量恢复实录

## 问题描述

t8o 门禁轮做改动面二分诊断时，为「临时把工作区切回 HEAD 版本、保留暂存区」，执行了三轮
`git checkout HEAD -- <files>`（覆盖 8 个文件），诊断完成后用 `git checkout -- <files>`
恢复。结果：8 个文件的暂存版全部丢失（index 被重置为 HEAD），仅剩 3 个未动过的文件仍在
暂存区。若非及时发现，提交内容将静默缺 8 个文件、490 行减至 ~130 行。

## 根因分析

`git checkout HEAD -- <paths>` 的语义是**从 HEAD 恢复到 worktree 和 index 两处**——
它不只是切工作区，还会把 index 里这些路径的暂存内容一并覆盖为 HEAD 版。
「保留 index、只切 worktree」的正确命令是：

```bash
git restore --worktree --source=HEAD <paths>
# 或 git restore -W -s HEAD <paths>（旧 git：git checkout <paths> 前先确认语义）
```

而恢复用的 `git checkout -- <paths>` 是**从 index 恢复 worktree**——index 已被冲掉时，
它恢复出来的是 HEAD 版而非暂存版，形成「已恢复」的假象。

## 解决方案

`git add` 创建的 blob 对象即使被移出 index 也仍留在 `.git/objects`（无 gc 时），
可按内容特征找回：

```bash
git fsck --cache --no-reflogs --lost-found --dangling | awk '/dangling blob/{print $3}'
# 逐个 blob：git cat-file -s <blob> 看大小；git cat-file -p <blob> | grep -c <特征串>
#            + 与 HEAD 版尺寸对比（改动后 = HEAD 尺寸 + 插入量），双因子确认映射
git cat-file -p <blob> > <原路径>   # 逐个写回
git add <paths>                      # 重建暂存
```

恢复后用 `git diff HEAD --stat` 与事故前的 `git diff --cached --stat` 逐字节核对
（本例两者完全一致：11 files, 490 insertions, 8 deletions），再重跑钉测回绿确认。

## 教训与如何避免

- **诊断期临时回退源码时，永远用 `git restore --worktree --source=HEAD`**，不用
  `git checkout HEAD --`；前者只动 worktree，语义单一。
- 大改动（多文件、数百行）在关键操作（诊断、构建、审查）前先落一个 patch 备份：
  `git diff --cached > /tmp/<task>-staged.patch`——blob 恢复是最后手段，patch 秒级恢复。
- 恢复后不要只看文件存在，必须用改动前记录的 stat（文件数/行数）核对完整性，
  并重跑受影响的钉测。
- 多轮诊断刀叠加时，「每刀之后 index 里应该是什么」很容易算错；一次事故足够说明：
  诊断轮开始前把暂存状态固定下来（patch 或第二次 `git stash push --staged`）。
