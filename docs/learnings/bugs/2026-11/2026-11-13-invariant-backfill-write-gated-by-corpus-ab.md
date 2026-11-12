---
date: 2026-11-13
modules: layout-engine
---

# 不变式补写类修复的最终形态必须由 corpus 全量 A/B 定谳——四门全绿仍拦出确定性真回归

## 问题描述

R5510 修「行/行组盒高由表格 pass 直写而 content_height 留 taffy 旧值」的 backfill
不变式缺口（单格态 tr 60+18=78 复利）。首轮实现＝4 写点**无条件** `content_height =
height` 补写，常驻锚 3/3、全包 release 2657P/0F、make reftest 725/725、product-smoke
6/6——四门全绿。随后 corpus 全量 A/B（16594）翻出 table-anonymous-objects-087/088
确定性真回归（solo×3 双二进制：base ✓0.00%×3 / 修复后 ✗2.68%×3、✗3.49%×3）。

## 根因分析

tao-087/088（infer-cells-2）畸形表生成让 `display:table-row` span 行盒含 **unwrapped
Inline 子**（基线上悬 y=−9.6、嵌套 TableRow），行子树 max_bottom（138.4）≫ 网格行高
R（46）、taffy content_height 陈旧 −0。base 世界里 `backfill_r109_anon_block_heights`
arm② 的 delta（maxB − 0）膨胀行高后，**其兄弟位移臂把 unwrapped Inline 推到与参照
网格文本对齐的 y**——即 arm② 恰为该两案绿面的承担者（#29「绿案可能是另一条
回填/位移 pass 间接撑住」+#33 self-source 自洽联合活例）。无条件补写把 delta 缩为
maxB − R，兄弟位移差一行高，文本带移位即失配。

这是「恢复不变式」类修复的普遍陷阱：被恢复的不变式可能正是某些页面最终几何的
**隐性输入**——凡 content_height/height 槽位，下游消费方不止校验者想象的那一个
（arm② delta、兄弟位移、溢出裁剪、行组 Σ 等各自读同一槽位）。局部锚点只锚
修复者想象中的消费方语义，锚不到未知的消费方。

## 解决方案

收窄门控补写到已证域：`|taffy ch − 网格行高| > 0.5 ∧ 行子树含 r109 拆分载体` 才
补写。三段语义：①ch 一致＝no-op（常见路径零开销）；②不一致∧含载体＝唯一已证
驱动案（R5509 单格态），补写后 arm② 不触发、tr=td 精确；③不一致∧无载体＝维持
base 世界原 delta，绿面自洽不扰。深修共享 backfill pass 的兄弟位移语义被
R1163 broad 臂 welcome +12.57pp 先例否决（blast radius 不可控）。

## 避免方式

- 「补写/回填/对齐不变式」类布局修复：无条件形态先跑 corpus 全量 A/B 再定稿；
  锚/包测试/reftest 内测套件/product-smoke 全绿**不构成**跳过 A/B 的理由。
- A/B 翻红的 mover 必须 solo×3 双二进制定谳后再定性（#35 采样协议）；真回归先
  画双臂几何对照找「谁是绿面承担者」，再选收窄门控 vs 深修共享 pass。
- 收窄门控顺带零开销化：一致态不触发子树遍历。
