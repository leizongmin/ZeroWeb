---
date: 2026-08-17
modules: zero-layout-engine 最终 IFC
---

# 昂贵子树扫描应先判断消费模式

## 问题描述

最终 IFC 为每个含文本容器递归扫描整个 DOM 子树，检查 `text-decoration` 和 `text-emphasis`。perf 中该扫描占 medium 页面约 1.46% CPU。

## 根因分析

扫描结果只用于 vertical writing-mode 的容器宽度兼容 gate，但旧代码在 horizontal 和 vertical 容器上都先扫描，再在后续条件中判断 writing mode。普通横排页面因此对每个文本容器执行无消费方的递归工作，嵌套结构还会重复访问后代。

## 解决方案

先判断 writing mode。默认 `ZW_DECORATION_SCAN_VERTICAL_ONLY=1` 时，horizontal 容器直接跳过扫描；vertical 容器继续执行原算法。`=0` 恢复旧的全容器扫描路径。

通用规则：昂贵分析若只服务特定布局模式，应先做模式判定，再构建分析结果。

## 验证

闭包计数测试证明 horizontal 不调用扫描、vertical 调用一次。两组反序 medium A/B 的 layout p95 均同向改善：`411→399ms` 与 `417→401ms`，约 3%。total 指标受共享主机 paint 波动影响，不作为收益主证据。
