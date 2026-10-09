---
date: 2026-10-09
modules: .agents/skills/zeroweb-site-optimizer
---

# 长时链运行期的工作树纪律：HEAD guard 与 log 追加语义的两件实证

## 问题描述

合并树集成链（八段：fmt/clippy×2/release/maketest/reftest/WPT/live，约 35 分钟）重跑时连出两件工作树/脚本语义事故：

1. **链中 HEAD 移动被拦截**：链运行期间控制器在同一工作树提交了 docs PR（commit 70d840e37），maketest 后的 HEAD guard 报 `TREE_GUARD_FAIL got=70d840e37 want=d955ee223`，按设计中止（reftest/WPT/live 未跑）。虽然 `git diff --name-only` 证明两 commit 间产品代码零触碰（仅 docs/learnings），守卫纪律仍不接受跨 HEAD 证据迁移——全链钉回原 SHA 重跑。
2. **stage log 跨重跑追加不截断**：链脚本 stage 函数写法为 `[ -s "$log" ] || echo 头 > log` + 输出一律 `>> log`——头行只在文件**首建**时写，输出永远追加。重跑时旧 log 已存在，于是 maketest log 混入首跑+重跑两段电池（142 腿 = 2×71，总通过数翻倍到 40,668P），若直接解析会得出翻倍的锚数。

## 根因分析

1. HEAD guard 的价值恰在于把「同树全链」变成硬约束；它拦下的是证据身份风险（运行中途树身份变化），不看变更内容是否无害——这是特性不是误报。
2. 追加语义的设计动机是保护链内续写（同一次运行的各段追加到同一 log），但「重入」与「链内续写」对该写法不可区分：跨重跑时旧内容被当作本次链的前段保留。

## 解决方案

1. **纪律**：长时链/终验运行期间，同一工作树禁止任何 git 写操作（commit/分支切换）；docs 类并行交付推迟到链尾。执行人开工即 `git rev-parse HEAD` 钉定并记录。
2. **重跑前清理语义**：重跑链之前把上一轮的 stage log 归档改名（或脚本改为每段 `> log` 后追加——但那会牺牲链内续写保护）；本例处置：按第二处配方行（`rustc -O scripts/test-guard.rs` 第二次出现）拆分，重跑段为正身（头行重建并在 run.md 披露），首跑段归档 `*-attempt1.log`。
3. **解析侧防线**：从 log 派生数字（腿数/通过数）时先断言量级合理（腿数与上一终局同数量级），翻倍/归零立即怀疑双段或截断——本次 142 腿断言（≥70）通过了但 +20,339 的 delta 异常才是真信号，量级对独立值的断言比下界断言更早抓到这类事故。
