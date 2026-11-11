# R5506 inline-block shrink grow 腿自身 max-width used 值钳（2026-11-12）

## 一、归因链

**驱动案**：R5505 挂账新增①（探针案形，无 corpus 专案）——inline-block span（`max-width:120px`）内含定宽 inline-block 原子子（`width:300px`），span 自身 used-width 钳不生效：ZW 实测 300 应 120；文本形正常已证（120 ✓）。

**探针复现**（临时探针四变体，修复前）：
| 形 | 外层实测 | 期望 |
|---|---|---|
| 原子子（inline-block 300px） | 300 | 120 |
| 块级子（div 300px） | 300 | 120 |
| 文本形（宽文本） | 120 | 120 ✓ |
| 无 max-width 基线 | 300 | 300 ✓ |

**根因定位**：kill-switch 对照定谳——`ZW_IB_SHRINK_GROW=0`（R5348 grow 腿关闭）时原子子形外层=120（taffy 初布局的 max-width 钳保持）。元凶＝`shrink_inline_blocks_to_content` horizontal 臂 R5348 grow 补腿：`ib_grow_target` 取流内子已布局 margin-box 最大者后直接回写，无自身 max-width 钳；触发条件 `ib_grow_target > box_node.width` 中 `box_node.width` 已是 taffy 钳后值（120），grow 腿越钳回写 300。原子子与块级子两变体同路径（子外宽 300 > 当前 120）。

**域定性**：R5505 修的是测量域（`box_content_max_width` 子递归贡献钳）；本案是消费端 used-width 域（shrink pass 回写目标盒自身 used 宽）——同一 max-width 语义的双路径第三面（guidelines #6/#8 谱系）。

## 二、修复

`float_positioning.rs` grow 腿 else 臂（`ib_grow_target` 折叠后）：自身 definite max-width 钳——`cap = resolve_intrinsic_real_length(max_width) + frame`（Calc 混合式排除沿 R5368 `clamp_box_max_contribution` 同款，% 项伪值解析）；钳后目标 < 当前宽则 grow 触发条件自然不满足（min-width 地板场景保持，min 胜 max）。CSS2 §10.3.9/§10.4 used 值约束。

**有意维持不改**：
- shrink 腿（单向收缩，写值 ≤ 当前 ≤ cap，天然在钳内）；
- `width_keyword` 双向腿（`has_w_constraint` skip 已覆盖 definite max-width；% max-width 同域缺口见下）；
- vertical 臂（`has_w_constraint` 全排除 definite min/max，R4234/R4389 同款已保护）；
- % max-width 与混合 Calc 的 used 域钳（需 CB 宽上下文，本 pass 无父宽参数，FIXME 已注；无驱动案不扩）。

## 三、验证

| 门 | 结果 |
|---|---|
| 常驻锚 r5506_ib_grow_maxwidth_cap_tests 5 test | 全绿（含 min-width 冲突地板保持） |
| 包测试 cargo test -p zero-layout-engine | P=2642 F=0 |
| make reftest 725 | 725/725 全绿 |
| product-smoke | welcome desktop diff 17.23%<20%（R5501 基线同值）＋struct-check 全 PASS |
| corpus 全量双态 A/B（16594 案） | base 688F vs new 691F；mover solo×3 双态 30/30 全 PASS＝全噪声，真态 FIXED=∅ REGRESSED=∅ |
| bench-gate 定向 layout-engine | GATE PASS 25 指标 NEW=0 |
| clippy -p zero-layout-engine --all-targets -D warnings | 无告警 |
| rustfmt | clean |

**双二进制行为判别**（#28 纪律）：探针 HTML（蓝盒 max-width:120 含红定宽子 300）双 release runner 渲染 PNG 像素测量——base 蓝盒右缘 x=300（bug 在），new x=120（修复生效）；md5 互异＋base build 日志含 zero-layout-engine 重编译（stash 源回退态构建）。

**corpus A/B mover**（剥 % comm）：
- FIXED（base-only）：tao-153
- REGRESSED（new-only）：tao-145/148、insert-block-end-002、insert-block-middle-002

5 mover solo×3 双态 30/30 全 PASS——tao 互翻族（145-154 在册）＋insert 家族（R5498 insert-block-002/R5501 insert-inline 同族）全窗快照假动，真态零移动。净 +3F 掩于噪声、真值≈688F 等效。驱动案形无 corpus 专案（R5505 盘点已证该域唯一在册案 003/004 为测量域、已修）。

## 四、残留与挂账

- % max-width / 混合 Calc max-width 的 grow 腿 used 钳缺口（需 CB 宽 plumbing；无驱动案，FIXME 已注代码，挂账低优先观察）。
- 同域：`width_keyword` 双向腿对 % max-width 同缺口（`has_w_constraint` 对 % 返 false 不 skip），随上条一并观察。
