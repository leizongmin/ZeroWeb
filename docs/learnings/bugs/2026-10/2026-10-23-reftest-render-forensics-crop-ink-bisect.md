---
date: 2026-10-23
modules: layout-engine, wpt-runner, engine
---

# reftest 渲染分诊：1x 全图目测不可靠——定量 ink + ≥4x 裁剪 + 删盒二分

## 问题描述

before-after-positioned-001（✗1.12%）分诊中，对 800×600 全图 PNG 的 1x 目测连错两次且方向相反：先判 ref 侧「稀疏无生成内容」，后又把富渲染误读为「文本齐全」——第二次误读是刚看过布局 dump 所致（心理预置：dump 说 img 在 x=22/226 32×32，就「看见」了不存在的文本）。两次误读各浪费数轮探针。

## 根因分析

1. 800×600 里 16px 文本约 8px 高，1x 下字形细节不可辨，大脑按预期模式补全——**预期即偏差源**。
2. --base-dir 是外链资源解析根：wpt-data 根 vs 文件目录直接决定 `<img src="support/...">` 是否加载，而 img 加载态会**全局翻转布局形态**（abspos auto 宽从 110px max-content 塌到 32px min-content）。探针迭代中改了 base-dir 而不自知，把两次不可比渲染当同一形态对比。
3. 同 binary 同输入出现**不同失败形态**（叠盖 vs 整体不绘制）——单盒探针复现不了完整文档的失败，因为失败形态依赖文档级上下文（前盒 abs span 的存在）。

## 解决方案

- **定量优先**：先分区算 ink 像素数（PIL crop + 阈值计数），不同区域数字对不上目测结论时信数字；任何形态结论必须配 ≥4x `Image.NEAREST` 裁剪图直接看。
- **变量固定**：base-dir、binary 代际、REFTEST_DUMP mtime 列为探针一等变量，每次渲染记录在案（本轮延续 R5233 的 mtime 教训）。
- **删盒二分**：从真文件逐级删除前置盒（sed/python 生成 k5/k4/k3 变体）定位「文档级上下文翻转失败形态」的触发盒——单元素最小探针（a/e/h/j）与文档级探针（k）是互补工具，前者定位机制、后者定位上下文依赖。
- **两侧证据都要拿**：布局 dump 几何全对 ≠ 绘制无恙（本轮 img x=22/226 正确落位但绘制全丢）；`ZW_DEBUG_IFC` 看 items 流可快速切分 collect/装配/绘制三段责任。

## 关联

- R5233 教训：dump PNG mtime 陷阱（未设 REFTEST_DUMP 的跑不刷新 dump）。
- R5230 教训：分诊工具保真度（Ahem 合成方块差异、--base-dir 语义）。
