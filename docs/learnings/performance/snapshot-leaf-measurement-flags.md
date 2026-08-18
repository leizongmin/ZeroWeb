# Snapshot Leaf Measurement Flags

日期：2026-08-18

相关模块：`crates/layout-engine/src/inline_finalization.rs`、`crates/layout-engine/src/inline/runtime_flags.rs`

## 问题描述

R3444 已将部分 residual layout kill-switch 接入 process-lifetime snapshot，但 leaf measurement 仍在 `measure_text_content` 内按 taffy 文本测量回调 live 读取 `ZW_CONTENT_VISIBILITY` 与 `ZW_SHAPED_FALLBACK`。medium 页面会多次进入匿名 flex/grid 文本测量，这类 live getenv 会被文本节点数量放大。

同一区域还有两个 Ahem family 判断使用 `contains(&"Ahem".to_string())`，每次判断都会构造临时 `String`。

## 根因分析

R3444 主要覆盖了 postprocess、line metric 和 shaped advance trace/fallback 的 IFC runtime path，但 leaf measurement 是独立入口，没有复用 `inline/runtime_flags.rs` 的 residual snapshot。树构建阶段已有 content-visibility snapshot，文本测量阶段继续 live 读取会让同一策略在一个 layout pass 内反复扫描环境。

## 解决方案

将 content-visibility 和 shaped-fallback leaf measurement 判断接入 `inline/runtime_flags.rs`：

+ 默认使用 process-lifetime snapshot。
+ `ZW_LAYOUT_RESIDUAL_ENV_SNAPSHOT=0` 恢复 live lookup，保留调试回滚语义。
+ Ahem 判断改成 `iter().any(|family| family == "Ahem")`，保持原 exact/case-sensitive 语义，只移除临时分配。

## 验证方式

同一 release 二进制 residual snapshot off/on 的 medium layout p50/p95 为 `248.27/301.52→217.09/270.22ms`，total p95 为 `526.50→474.79ms`。style/paint p95 存在尾部漂移，不作为收益主证据。

局部验证覆盖 Ahem 路径与 layout-engine clippy；完整提交门禁覆盖 workspace clippy、`make test`、`make reftest`、`make product-smoke` 与 `make bench-gate`。
