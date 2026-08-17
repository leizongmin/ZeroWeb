# ASCII 快路应先于 Unicode 属性查表

- 日期：2026-08-17
- 相关模块：`zero-layout-engine` 行内文本度量

## 问题描述

`estimate_char_width` 为识别零 advance 的 Unicode nonspacing mark，对每个字符调用 `unicode_bidi::bidi_class`。medium 页面以 ASCII 文本为主，perf 中 Unicode BiDi 二分表占约 2.35% CPU。

## 根因分析

ASCII 的 BiDi 类集合不包含 NSM，但旧路径仍对每个 ASCII 字母、数字、空格和标点进入完整 Unicode 属性表。文本宽度估算会在 intrinsic sizing、taffy measure、最终 IFC 和 paint IFC 中重复执行，单次小成本被多 pass 放大。

## 解决方案

先用 `char::is_ascii()` 排除 ASCII，再仅对非 ASCII 字符查询 `bidi_class`。`ZW_NSM_ASCII_FAST=0` 可恢复旧查询路径。Arabic U+0654/U+0670 等真实 NSM 继续查表并保持零 advance。

通用规则：当目标 Unicode 属性在 ASCII 子集中有确定答案时，先做 ASCII 分支，再进入 Unicode 范围表或二分查找。

## 验证

隔离 release 微基准交替运行三轮，旧路径为 `4.74–5.61s`，快路为 `1.18–1.25s`，约 4 倍提速，checksum 完全一致。medium 首组页面 A/B 的 layout/total p95 分别改善约 3.4%/2.7%；反序页面数据受共享主机负载漂移影响，不作为主证据。
