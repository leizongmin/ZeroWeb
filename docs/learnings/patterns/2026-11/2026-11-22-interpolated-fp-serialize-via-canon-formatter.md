---
date: 2026-11-22
modules: engine
---

# 幂律插值的 fp 残差必须走 CSSOM 序列化口径——裸 Display 破坏规范形比较

## 问题描述

刀 FAM-F aspect-ratio 几何插值（css-values-4 §combine-ratio：r0^(1−t)·r1^t）钉测
第二轮失败：`interpolate_aspect_ratio("0.5", "2", 0.5)` 期望 `1 / 1`，实得
`1.0000000000000002 / 1`——0.5^0.5 × 2^0.5 的数学恒等在 f64 下留 fp 残差；
`("1 / 2", "2 / 1", 0.5)` 的 √(0.5×2) 同形。端点 t=0/t=1 是精确值，唯独中途点
（插值语义的主场）几乎必然带残差。

## 根因分析

插值输出用 `format!("{prefix}{} / 1", x)` 直印 f64：Display 打满 17 位有效数字。
而比较对手（WPT harness 断言期望、gCS 回读、钉测期望值）全部是 CSSOM serialize
口径——6 位有效数字去尾零（`format_num`，与 Chromium 一致，
`computed_style::aspect_ratio_str` 同口径）。同一数值两条序列化路径在幂律中途点
必然分叉：残差在第 17 位，Display 逐字比较即炸，6 位有效数字口径下不可见。

## 解决方案

任何产生 fp 中间值的插值器，输出一律走与 gCS 相同的序列化函数
（`crate::js_dom_bridge::computed_style::format_num`），不手写 Display/rounding
——口径恒同是规范形比较（`x / 1` 规范形、去尾零、6 位有效数字）成立的前提。
旁证：同文件兄弟插值臂（width/length 族）早已如此，新臂照抄即可。判断准则：
输出若被 WPT 断言逐字比较或与 gCS 回读比对，则序列化路径必须唯一。
