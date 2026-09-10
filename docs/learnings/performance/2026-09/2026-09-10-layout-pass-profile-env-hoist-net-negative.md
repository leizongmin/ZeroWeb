---
date: 2026-09-10
modules: layout-engine
---

# 布局 pass 探针归因两坑：分段标记间隙误归因 + kill-switch env::var 提升反变慢

## 问题

R4214 轮用「分段计时探针」（`ZW_PROFILE_LAYOUT=1`，`Instant` 差分 + `zw_mark!` 宏）对
`block_layout_1000_elements`（criterion，基线 1.88ms / 实测 2.4-3.0ms 带）做 pass 级成本
归因，试图为 layout-perf-pullback-rfc 选第一刀。两处读数把归因带进沟里：

1. `shrink_mixed_control_forms` 探针读数 ~0.25ms（占实测 ~10%）→ 判定「无 form 页面白付
   全树递归」，加 DOM tag 预扫描门（`get_elements_by_tag_name("form").is_empty()` 早退）——
   门确认 fire（`ZW_GATE_TRACE` 实证 `has_form=false` 且立即返回），**读数一分不降**。
2. `adjust_float_positions_with_context` 探针读数 ~0.29ms → 归因于函数内 **15 处**
   per-node `std::env::var` kill-switch 读取（进程环境锁 + environ 线性查找）。把 13 个
   旗标提升为函数入口一次性快照（`FloatKillSwitches::from_env()`）随递归透传——
   交替 A/B（同机交错 3 轮）**一致变慢 ~0.27ms**（2.64ms 基线 vs 2.91-2.96ms 提升）。

## 根因分析

**坑 1（标记间隙误归因）**：`zw_mark!` 差分测量的是「上一个 mark 到本 mark 之间」的全部
时间，不是紧邻那条语句。`P_shrink_mixed_forms` 的 mark 前面隔着 section 12 的
`compute_final_inline_layouts`（真实 ~0.2ms 大头）+ multicol/backfill/relpos 一串 pass——
0.25ms 几乎全是间隙里的 `compute_final`，`shrink_mixed_control_forms` 本身仅 ~15µs。
**探针必须紧贴目标语句两侧成对放置**，或至少先按「语句清单 + mark 对位」核对一遍间隙内容。

**坑 2（env::var 提升反变慢）**：`adjust_float_positions_with_context` 是 ~1300 行的单体
函数（float 定位/收缩/clearance 全逻辑）。散布在冷分支里的 `std::env::var`（不透明外部
调用）客观上成了 **LLVM 优化栅栏**——抑制了热循环的激进内联/展开/大函数体合并，
I-cache/分支布局停在了一个「更幸运」的形态。提升成局部结构体后，13 个 bool 横跨全函数
存活期，寄存器压力 + 代码布局变化反致净负。教训：**「减少明显浪费」的机械优化在无
profile 逐点验证时可能负收益**——尤其对超大单体函数内的「浪费」，它可能正在承担意外的
codegen 锚点职责。

**背景**：本机 bench 对环境负载极敏感（同代码 2.43→3.03ms 带，±20%），单次读数不可信；
归因必须交替 A/B + 多段交叉核对（其他段持平才能定罪目标段）。

## 解决方案

- 两项「优化」均按净值纪律**回退**（净负/净零不挂账），探针仪器化代码同样回退。
- 保留有效产出：block_layout_1000 的 pass 级成本表（HEAD ~2.6ms @ load~1.3：
  tree build/converter ~0.92ms（35%，最大单杠杆）＞ float walk ~0.29ms ＞ intrinsic
  pass2 ~0.27ms ≈ taffy pass1 ~0.25ms ≈ compute_final ~0.2ms ＞ extract ~0.17ms），
  已记入 layout-perf-pullback-rfc §5 进度，converter（S1）为下一刀。
- 后续探针规范：mark 成对紧贴目标语句；每次归因先跑「门已 fire」的实证（trace/
  计数器），再对比读数；负结果同样入账。
