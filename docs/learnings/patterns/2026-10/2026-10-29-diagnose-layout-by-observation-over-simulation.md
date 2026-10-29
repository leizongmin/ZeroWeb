---
date: 2026-10-29
modules: crates/taffy-local,crates/layout-engine
---

# 布局根因排查：单点观测插桩定谳，不用手工推演开局

## 问题描述

R5331 排查 `flex-minimum-height-flex-items-029`（嵌套 column flex，auto 高外层容器应 100 实渲 500）时，先走手工推演：按 taffy `determine_container_main_size` 的三种 `available_space` 分支（Definite / MaxContent / MinContent）逐支模拟测量链，三度落空——没有一支能推出 500，多轮推导耗尽仍无法定位值来源。

## 根因分析

失败的不是推演的某一步，而是推演本身作为开局策略：taffy 布局是多趟递归（测量 ComputeSize / 完整 PerformLayout / 每节点缓存），同一个盒的最终尺寸由调用点组合决定。手工模拟需要同时命中正确的调用链、参数组合与缓存态，任何一支偏离真实执行，结论就整体作废。而真实执行只发生一次，直接观测它比枚举所有可能的它便宜且可靠。

## 解决方案

渲染/布局根因排查的定谳顺序固定为「先观测分界，再读代码收敛」：

1. **选分界点**：把管线切成可疑段（如 taffy compute 输出 → `extract_layout` 之后 → ZW postprocess 链之后），确定「值在这个分界点是错的」还是「分界后才错」。
2. **插桩观测**：在分界点临时插桩打印实际值（env 门控如 `ZW_R5331_DIAG`，便于 A/B；用完 `git restore` 回退，不留痕）。R5331 一处 extract 插桩就证明了 500 是 taffy 原生输出，排除整个 ZW 后处理域。
3. **读代码收敛**：确证值域后，只读分界点上游最近的处理路径找闭环根因——此时推演才有锚，用于假设收敛而非初始定位。

同域判例：R5330 的「检查点二分 C0-extract→C7-end」（027 根因定位在 `shift_siblings_after_ifc_grow`）是同一方法论的分段版。两条经验合并成一条规则：**对手工推演三度落空的信号要敏感——立即转观测，不要第四次推演**。
