---
date: 2026-10-26
modules: .agents/skills/zeroweb-site-optimizer
---

# busy main 下集成核验用 contains 语义；轮询比较截断位数单点定义；抖动网络窗口重试封装

## 问题描述

P17c 合并段一日三坑：①预合并轮询打印 `headRefOid[:9]` 而 case 模式写 10 字符前缀，
永不匹配，30 轮空转后 PREMERGE_BLOCKED；②轮询 CLEAN 后 3 秒内并行任务合入 main，
GitHub 以新 head 为合并提交第一父，回执 `parents == {base, head}` 断言失败——合并
实际已成功，账本卡在 merging；③集成核验 `origin/main == merge commit` 等式在
origin/main 持续被并行任务推进的情况下反复失效。叠加背景：当天 github.com 页面/
附件面波浪状抖动（api.github.com 正常），呈现层活体附件探活随波浪超时。

## 根因分析

并行任务机器向 main 合并是本运行的常态而非异常：合并提交的第一父是「合并瞬间
main 的 head」，与 PR 记录的 base 可以不同（PR base 仍是祖先）；等式断言把
「包含任务工作」误表达成「main 停在合并提交上」。轮询坑是同一信息两处手写、
截断位数不一致；探活超时则是把传输层抖动当成证据失效处理。

## 解决方案

- 集成核验用 contains 语义：merge commit 是 origin/main 祖先、任务 head 是
  origin/main 祖先、交付 base 是 origin/main 祖先、合并提交第二父 = 任务 head、
  `contains_merge: true`——不要求 main 等于合并提交。
- 轮询/比较里同一 OID 只在一处定义截断（打印与模式共用同一变量派生）。
- 合并竞速的处置按先例：核验 first parent 的改动与任务文件交集为 0 后吸收，
  回执记 `concurrent_base_advance`（first_parent、主题、交集数、处置）。
- 抖动网络期校验封装成「窗口探测（3×200）+ 执行 + 失败冷却 60s 重试」，
  单次失败无状态损伤（快照写盘在校验前、安装在校验后）可安全重跑；
  校验耗时长的环节（>60s）要按「窗口必须撑住校验时长」估计探测轮数。

避免方式：合并后立即以远端 readback（state=MERGED、mergeCommit）为事实源写回执，
本地祖先断言只做 contains 面核验，不做等值面断言。
