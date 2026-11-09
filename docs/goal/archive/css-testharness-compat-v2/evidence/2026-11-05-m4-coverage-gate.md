# M4 / DC-4：coverage 复测收口（2026-11-05）

DC-4 收口件：css topdir 数值门复测与全量头条对账。前置台账：M3 十六目录
零回归对照（[2026-11-05-m3-sixteen-zero-regression](2026-11-05-m3-sixteen-zero-regression.md)）
与 M1/M2 四份批次接线基线。

## 口径

- **输入**：60 suite 逐案 JSON = 一期十六目录（16）+ 44 批次目录（44），
  合并投喂 `wptfyi_gap.py coverage`（结果目录
  `.acceptance/css-v2-m3/m4-merged/`，诊断留档不入库）。同一文件跨套件
  由工具按最完整一次记账（不双记），本轮跨套件重复 0 条。
- **runner**：冻结二进制 sha256 `1e082639ea844bea…`（构建自已提交树
  ff38013d7，含修复 6be1aae94），每目录 sequential，test-guard
  `--per-proc-mem 4 --total-mem 8 --time-limit 3600` 与常驻通道同参。
- **分母**：wptfyi cache universe（与 coverage-2026-11-02 快照同源），
  本轮实测 css topdir 195,553 子测试（快照 195,723，上游滚动 −0.09%；
  全量 2,265,464 对快照 2,266,480）。门按绝对子测试数判，不受漂移影响。
- **执行备注**：44 目录采集由自足链脚本完成 34 目录后，因会话竞态清理
  波及脚本进程树而中断；余 10 目录（css-position ~ css-writing-modes）
  由 `run-m4-resume.sh` 同参补齐（10×rc=0，逐目录 Pass 计数与批次 4
  基线一致，确定性复核通过）。

## 结果

| 面 | 目录数 | zw passes（coverage 口径） |
|---|---:|---:|
| 一期十六目录 | 16 | 20,149 |
| 批次 face-a | 12 | 4,102 |
| 批次 face-b | 19 | 2,731 |
| 批次 face-c（B3 13 套件） | 13 | 2,532 |
| **css topdir 合计** | **60** | **29,514** |

- face-a + face-b = B1 + B2 31 目录（6,833），face-c = B3（2,532）。
- 批次合计 9,365 对照 M1/M2 干跑基线 compare 口径 9,642（B1 4,608 +
  B2 2,369 + B3 2,665），差 −2.9% 为两口径计数语义（harness 级 Pass
  计入与否）与 B3 全量跑时长差吸收，逐目录无异常。
- **DC-4 门：css topdir ≥ 20,500 → 实测 29,514（门的 1.44 倍），通过**。
  css topdir 9.16%（17,923/195,723，coverage-2026-11-02）→ **15.09%**
  （29,514/195,553 本轮实测分母；按快照分母折 15.08%）。
- **全量头条不回落**：按快照同口径换算（1,530,796 − 17,923 + 29,514）/
  2,265,464 = **68.08%**（基线 67.54%，+0.54pp）。换算说明：本轮输入为
  css-only（其他域组计 0），全量绝对数以 weekly CI 全量为最终口径；
  coverage 报告留档 `.acceptance/css-v2-m3/m4/coverage-css.{md,json}`。

## DC-4 判定

**通过**：css topdir 绝对子测试 29,514 ≥ 门 20,500；全量头条换算 68.08%
不回落；`wpt-suites.csv` 数据行同提交回填。DC-1..DC-4 全部满足，
本 goal 收口。
