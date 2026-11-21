---
date: 2026-11-21
modules: wpt-runner,testing
---

# corpus 腿判不了按需 fetch 未落库的页面——电池发射前必须核靶页在库

## 问题描述

knife-rshare 电池（random() 写面 setter 双门刀）四 corpus 腿（values /
css-typed-om / cssom / css-variables）双臂全绿哨兵、RECONCILE 零 flip，
face 被判「零效果」——但 face 的机制证据（探针 P17/P18 setter 拒收）明确指向
values 语料的 random 页。同一二进制换页级 filter 补跑后立刻翻出 net +3
（4 F2P ＋ 1 P2F）。

## 根因分析

`testharness-css-values` corpus 是按需 fetch 模式：上游 css-values 目录 672
页，只有被先前刀/摸底明确拉取过的页面才落在本 clone 的
`tests/wpt-runner/wpt-data/css/css-values/`（gitignored）。ZeroWeb-3 是新迁入
的电池 clone，只拉过 62 页，random 子域 20 页从未拉取——corpus 腿跑的是
「本 clone 在库页面」而非「上游语料域」，目标面根本不在跑测集合里。双臂
IDENTICAL＋哨兵全绿恰好是「空转哨兵」的形态，与「face 真零效果」不可区分，
若只看 RECONCILE 数字会直接误判假说被证伪。

叠加第二笔：corpus 名笔误 `testharness-values`（正确名 `testharness-css-values`，
目录名直映）使首跑 values 腿 `Unknown command` 秒退，23KB usage 文本冒充 JSON
产物，RECONCILE 首次 load 即 JSONDecodeError——错误产物形态也会掩盖真实腿况。

## 解决方案

- 子域刀电池发射前必须核靶面在库：`ls tests/wpt-runner/wpt-data/css/<corpus>/ |
  grep <靶页特征>`，缺页先补（从姊妹 clone 拷贝或 fetch 脚本拉取），并在电池
  判读里区分「corpus 面哨兵」与「页级靶面」两种账。
- corpus 命令名＝目录名直映（css-values→testharness-css-values），产物 sanity
  必须验 JSON 可解析＋页数>0，23KB usage 文本＝命令名错的指纹。
- 已构建的存档 bin（`results-*/bin/{b0,b1}-runner` 静态副本，shim/face 已烙入）
  与树态解耦——corpus 腿废后可用其重跑正确靶面（页级 filter 5s/臂），不必重
  构建双臂。
