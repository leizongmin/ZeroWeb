# R4511 证据：multicol-span-all-children-height-004a 精确带几何（chromium oracle vs ZW）

- 日期：2026-09-19（R4511 P 轮）
- 方法：`REFTEST_DUMP=1 make reftest-upstream`（单案过滤）dump PNG → python 逐像素带分析（x=50 块区 / x=160 列右 pink 区 / x=300、380 col1），两页（test / ref）双探。
- 用例：`css/css-multicol/multicol-span-all-children-height-004a.html`（28.09%）——article（column-count:2，lightgreen）> div.container（height:450，pink）> [block1 200 yellow][span1 50 lightblue][block2 200][span2 50][block3 200]。

## chromium 目标渲染（REF PNG 实测，y 绝对坐标，viewport 800×600）

| y 范围 | 内容 | 备注 |
|---|---|---|
| 8..108 (100) | 段1 内容带：yellow 100/col + pink 列余宽 | block1 200 → 平衡 100/col |
| **108..208 (100)** | **白（无 bg）** | 容器段1 预算余量 |
| 208..258 (50) | spanner1 蓝 | 位于**完整预算段之后** |
| 258..358 (100) | 段2 内容带 | |
| **358..458 (100)** | **白** | 段2 预算余量 |
| 458..508 (50) | spanner2 蓝 | |
| 508..533 (25) | 段3 内容带：yellow 25/col + pink 25 | block3 被剩余预算裁到 50 → 25/col |

- 容器 450 按段切 **200/200/50**（内容带 100/100/25 + 未涂余量 100/100/25）✓ 合计 450。
- **white gap 连 article green 都不涂**（chromium 白 = body 底色；article box 虽覆盖但不透绿——容器段盒不透明白 OR article 不覆盖，两说并存未定谳）。
- ref 页（无 spanner、container h=200）同规律：pink 只涂内容带 100，108..208 白——**单段容器同样只涂内容带**。

## R4507 蓝图修正（本轮核心产出）

R4507 mock 推导假设「段间暴露 **article bg（green 100/50）**」——**错**。chromium 实际 = **白 gap（无任何 bg）**。ZW 若按蓝图实现绿 gap，将引入与白的新 diff（两段 100×~200 带）。slice 1 的 bg per-section 面（蓝图②）须改定为：

> 段盒 = 完整预算（content + 余量），**bg 只涂内容带**；余量带不涂容器 bg 亦不透 article bg。

## ZW 现状（TEST PNG 实测）

| y 范围 | 内容 | 与 chromium 差 |
|---|---|---|
| 8..108 (100) | yellow+pink ✓ | 一致 |
| 108..158 (50) | spanner1 **紧贴内容带** | 缺 100 预算余量推进 |
| 158..258 (100) | 段2 ✓ | 一致 |
| 258..308 (50) | spanner2 紧贴 | 缺 100 推进 |
| 308..358 (50) | 段3 pink 50 | 应 25/col（block3 未按剩余预算裁剪） |
| 358..408 (50) | **green（article bg 透出）** | 应为内容带 25 + 白 |

- ZW 的 pink 内容带（R1359/R1535 per-column bg regions）s1/s2 已与 chromium 对齐 ✓——残差集中在三面：①spanner 定位缺预算余量推进（蓝图③）；②末段内容未按剩余预算裁剪（R1357 squeeze 的 c 值只 cap wrapper 总高，未裁末段 block 内容）；③段尾余量带 bg 抑制（现透 article green）。

## slice 1 实施面定位（下轮起）

- 主战场：`try_layout_nested_spanner`（multicol.rs synthetic 路径）+ R1357 cap / R1359+R1535 bg regions / spanner y 推进。
- ①③两面纯 advance/clip 数学（预算分配表 section_budget = min(内容和/列数? …) 待定 200/200/50 的分配律——本轮实测 450 = 200+200+50 即 s1=s2=内容带+余量(100)、s3=剩余 50，分配律 = 前段吃满「内容+余量」到下一段内容出现前）；②bg 抑制 = nested_spanner_col_bg regions 扩展为 (offset, w, content_h, budget_h) 双高（内容带涂 bg、余量带留白不透 article——须 paint 侧跳过 article bg 或容器段盒改为不透明白？**未定谳，需先做 chromium 单案 DOM 探针**）。
- 同族受益：004b（31.05%）、006（23.40%）、span-all-rule-002（27.67%）。
