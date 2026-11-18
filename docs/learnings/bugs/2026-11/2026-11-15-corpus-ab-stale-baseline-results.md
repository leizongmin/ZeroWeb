---
date: 2026-11-15
modules: wpt-runner, testing
---

# corpus A/B 基线目录陈旧：跨刀复用旧 results 目录把已落库增益错记进本刀

## 问题描述

wptfyi-css offset-* 刀（CSS Motion Path）A/B 收口时，前一轮 session 用
`.acceptance/wptfyi-css/results-jsanim-b3`（三轮前的 B1 快照产物）作 B0 与
本刀 B1 逐页比对，得出「css-transforms +47 / css-values +7」的增益账。
对真基线（本刀前 HEAD：`results-flip-b1`）复比对：transforms/values/fonts/
backgrounds 四 corpus **全零差**，真增益全部落在目标面 css/motion（+363）
——+47 transforms 实为 composite-add 与 easing-flip 两刀**已提交**的增益，
被陈旧基线重复记进本刀。

## 根因分析

corpus A/B 的可比性要求 B0/B1 二进制分属「本刀提交态前/后」同一基座。快照
二进制（bin-*）与 results 目录（results-*）是一一配对资产：用旧 results
目录当 B0，等价于用旧二进制当 B0——多刀叠加期间，两代二进制的差 = 中间
已落库各刀的增益之和，逐页 diff 无法区分「中间刀」与「本刀」。

加重因素：连续多刀同日落地时，历史 results 目录多且命名相近
（results-jsanim-b3 / results-cadd-b1 / results-flip-b0…），
`ab-compare.py` 不校验 B0 新鲜度，错配静默发生。

## 解决方案

- 每刀 A/B 的 B0 必须**当轮重放**：stash 刀文件 → `cargo build --release
  --bin zero-wpt-runner` → 快照 `bin-<刀>-b0c` → stash pop → 重建 B1 快照
  （md5 对拍确认两二进制不同）。B0/B1 结果目录各开新 `results-*-clean`
  目录，不与历史目录混用。
- 若复用历史 results 目录（省腿时间），必须先做法证：核对该目录所用二进制
  的构建时间与内容装载面（快照 mtime vs 刀文件 mtime），并证明「B0 目录
  所用二进制 ≡ 本刀前 HEAD 行为」——通常只有**紧邻上一刀**的 B1 目录满足
  （其 B1 = 上一刀提交态 = 本刀前 HEAD）。
- 交叉验证定谳：对可疑增益页，用真基线目录复比对一次；两基线增益不一致
  即基线陈旧信号。
