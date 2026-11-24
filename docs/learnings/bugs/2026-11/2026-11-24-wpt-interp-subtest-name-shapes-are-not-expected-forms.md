---
date: 2026-11-24
modules: engine, style-system, wpt-runner
---

# WPT interp 断言名内嵌的 from/to 声明串不是期望形——按名字形状分类期望值产生系统性假锚

## 问题描述

KC-5 sizing 动画 GT 回归（r71→r72 两轮）：r71 判读把 GT 键按「期望形」分类，得出 66 键期望 `calc-size(KW, Npx + size * t)`（MIX 串）、14 键期望离散端点串的硬锚表。r72 据此设计三刀：MIX 串合成咨询、calc-size 裸透传、width/height 钳。电池实测三刀中两刀全空——MIX 咨询 127 键发射 0 回收，裸透传直接打出 REG 35＋F2P 90（净 +111F，REDLINE）。键级复检：GT-89 的 expected 全量形族＝`10px`×48／`0px`×22／`100px`×10／`150px`×9，**零 MIX 串、零端点串**。

## 根因分析

失败发生在**分类层**而非机制层：`*-interpolation.html` 的 subtest 名内嵌 from/to 声明串（如 `from [calc-size(auto, size * 2)] to [50%] at (1) should be [0px]`），名字里天然出现 `size * t`、`calc-size(KW, …)` 等形状。对 fail message 做形状 grep 时，这些形状被归到「期望形」桶——但它们是**输入声明**，不是比较目标。真实 expected 永远经参考元素 static gCS 折叠（见 [前篇](2026-11-23-wpt-interp-harness-expected-folds-via-static-gcs.md)），只可能是引擎自身能产出的折叠形；消息里 `should be [...]` 是折叠前原串文本。对名字形状做「方向代数逐条核对」能自洽（残差样本全是名内声明串字面），假锚上的公式再准也回收不了任何键。

第二层教训是**对称破缺的代价量化**：b0 对折叠伪影键的 Pass 常由巧合对称承载（宽松 parse 塌 0px、echo 通道产折叠形）。任何绕开这些通道的改动——哪怕方向「语义更正确」（裸 calc-size 串比塌 0px 更忠实于声明）——都会按每键一票的速度翻红：r72 裸透传一刀就击穿 125 个 passing 键。判定「某通道可否绕开」的判据不是该通道产出是否语义正确，而是**expected 侧是哪个通道的产物**；expected 塌 0px 时塌 0 通道就是承载通道。

## 解决方案

- GT 键期望形分类只能用 fail message 的 `expected "..."` 字段（或 fold(expected)），**禁止**用 subtest 名、断言文本、名内声明串做期望形归类；键级清单脚本里对名字做形状 grep 属伪源。
- 改动触碰任何「产出引擎自身折叠形」的通道前，先按前篇方法双臂探针定 expected 产生通道；通道承载判定＝「expected 侧从哪条链折叠而来」，而非产出语义优劣。
- 字符串合成路线（按声明形状拼 got）与折叠通道结构不相交：合成串 ∈ 声明形域，expected ∈ 折叠形域，两域零交集时发射越多损失越大（r72 刀③ 127 发射 0 回收实证）。回收键的正确目标是让 got 落进 expected 的折叠域。
