# M4 coverage 复测收口（2026-11-06）

CI run [42096051904](https://github.com/leizongmin/ZeroWeb/actions/runs/42096051904)
（weekly dispatch `run_wptfyi_coverage=true`，discover-shards 自动含 M3 五新面
catch-all 分片，全分片 success）→ aggregate 快照
[coverage-2026-11-05.json](../../../../compat/wptfyi/coverage-2026-11-05.json)
（bot 提交 b8800a45a，UTC 日期命名）。

## DC-4 核验

| 项 | 基线 11-02 | 复测快照 | 判定 |
|---|---:|---:|---|
| 全量头条 pass_rate | 67.54% | **68.10%** | ✓ 不回落（+0.56pp） |
| zw_passes | 1,530,796 | 1,544,578 | +13,782 |
| covered_files | 12,954 | 16,400 | +3,446（含本 goal 十二面 + 他流扩面） |
| 十二面 covered>0 | 0 面 | **12 面全 >0** | ✓ |

十二面 covered（快照分组，窗口可执行域）：quirks 3+3(unitless)、domparsing
25+23(tentative)、domxpath 32、SRI 1+2+2、cors 23、import-maps
15+5+7+9+9、mixed-content 2、referrer-policy generic 19 + css-integration 18、
input-events 19、preload 66、wai-aria 1+4+23(role)、storage 2 + buckets 1。

## 通道账 vs 快照分子口径注记

分面数值门（DC-1）定义在通道干跑账上（M1 365P + M3 297P，make target
可复跑复验）。快照分子按三家并集 universe 解析后记账，多数面与通道账逐位
一致（quirks 35、domparsing 151、domxpath 21、import-maps 34、input-events
188、wai-aria 2）；cors 52/92、SRI 15/32、preload 55/106 的差额为 universe
变体解析/交集对齐未命中部分（zw_only_files 609 同源机制），非通道回归——
同 corpus 同 runner 复跑通道账稳定可复现（M3 五面复跑逐位一致已验）。
缺口语义按 wptfyi-gap 口径文档（docs/compat/wptfyi-gap-analysis.md）。

`wpt-suites.csv` 已回填 `gap-batch1` 数据行（331 文件 / 3,811 子测试 /
662P = 17.37%，分面构成见行 note；trend_csv_check 契约单测过）。
