# DC-2 · selection 扩面接线与 CI 首批 editing/run 实证（2026-11-11）

## selection 面：为什么 CI 只入账 31/231 文件

ui-edit 分片的 `fetch-wpt-selection` 一直指向旧版 `fetch-selection-subset.sh`
（显式 CASES/EDITING_CASES/TEXTCONTROLS_CASES 清单，~60 案），23 号 goal 脚本
的 deep 拉取（全 selection/ 树，2026-10-27 二期扩面到 101+ 文件）从未进 CI。
11-10 快照 selection 面 2,821 过 / 31-35 覆盖文件即旧清单的影子。

## 接线改动（2 文件）

1. `Makefile` `fetch-wpt-selection` → `goals/23-selection-compat.sh`
   （lib.sh deep 拉取 + 拉取完成标记幂等）。本地实测：补拉 2 文件后
   selection/ 184 案，标记写入，二跑零网络跳过。
2. `tests/wpt-runner/ci-shards.yml`：selection 移出 ui-edit 独立成
   `selection-face` 片（script_timeout 300s / case_timeout 300s /
   time_limit 7200）；fetch 带 `fetch-wpt-editing` 供 runner
   `SELECTION_TEST_SUBDIRS` 里 flat 扫描的 editing/ 与 editing/other。
   ui-edit 保留 uievents/touch-events/keyboard/pointerlock/dnd 五套件，
   time_limit 3600 不变。与 editing-run 11 片 / editing-other catch-all 的
   路径重叠由对账器按路径 max 语义去重（`merge_zw_results` 最完整一次
   执行胜出，不双记）。

## 本地全面 census（缺省预算基线，[紧凑 JSON](2026-11-11-dc2-selection-face-census.json)）

90s 脚本看门狗 + 30s 案超时（即旧 ui-edit 的 CI 预算），wall ~17 分钟：

| 面 | entries | subtests | pass |
|---|---:|---:|---:|
| selection/root | 92 | 35,652 | 22,307 |
| selection/contenteditable | 12 | 252 | 170 |
| selection/shadow-dom | 22 | 81 | 19 |
| selection/textcontrols | 9 | 78 | 76 |
| selection/caret + bidi + anonymous | 13 | 136 | 25 |
| editing/other（经 selection 通道） | 516 | 15,496 | 13,034 |
| 合计 | 664 | 51,695 | 35,631（68.9%） |

- 零空结果 entries；Timeout 只 4 案 18 子测试——case_timeout 300s 的放宽
  不改变面形态，预算无放大风险。
- selection 本体（root+子目录）本地 ~22.6k 过 vs 旧 CI 面 2.8k——扩面增量
  主要在 root addRange/collapse 矩阵族。
- CI 预算校验：17 min × ~3× CI 减速 ≈ 51 min ≪ time_limit 7200。

## CI 首批 editing/run 实证（run 42947347500，11/11 片全部成功）

11/11 片成功（[run](https://github.com/leizongmin/ZeroWeb/actions/runs/42947347500)）：

| 片 | 文件 | variants | subtests | pass | 过率 |
|---|---|---:|---:|---:|---:|
| er0 | multitest | 10 | 9,980 | 9,977 | 100.0% |
| er1 | forwarddelete 组 6 文件 | 13 | 7,858 | 7,595 | 96.7% |
| er2 | delete 组 8 文件 | 18 | 8,664 | 8,329 | 96.1% |
| er3 | insertparagraph 组 3 文件 | 10 | 9,439 | 9,233 | 97.8% |
| er4 | justifycenter/indent/superscript | 9 | 8,805 | 8,527 | 96.8% |
| er5 | formatblock/subscript/insertimage | 8 | 7,249 | 7,039 | 97.1% |
| er6 | justifyright/forecolor | 8 | 7,141 | 7,052 | 98.8% |
| er7 | justifyfull/fontname/fontsize | 11 | 9,409 | 9,283 | 98.7% |
| er8 | bold/inserttext/outdent | 10 | 8,607 | 7,828 | 91.0% |
| er9 | justifyleft/strikethrough/underline/italic | 12 | 8,870 | 8,020 | 90.4% |
| er10 | removeformat/lists/hr/linebreak/inserthtml | 10 | 10,262 | 9,984 | 97.3% |
| **合计** | | **119** | **96,284** | **92,867** | **96.5%** |

面覆盖贴近上游总量：96,284 vs 上游 96,745 子测试（差 461 = 随库探针与
undo-redo.html iframe 案不进片的口径差）。失败总量 **3,417**，按片
er9（850）> er8（779）> er2（335）> er4 = er10（278）> er1（263）>
er5（210）> er3（206）> er7（126）> er6（89）> er0（3）——DC-2 最大簇
是 er9/er8 两个 format 命令组，delete 组（原估"最重簇"）实际 96.1%。

er2 内部低过率小簇：undo-redo-after-mutation 0/7、
typing-text-in-direct-child-of-editing-host 6/16、
delete-to-make-editing-host-empty 4/12、insert-list-items-in-table-cell
83/112；delete.html 本体 7,437/7,663（97.1%，全片唯一 Timeout）。

**slow 簇未超预算，无需再拆**：er0 multitest 10 变体全出（仅 1 子测试
Timeout）；er1 forwarddelete（1 子测试 Timeout）；er2 delete.html 全片
唯一另一处 Timeout；er3 insertparagraph 组零 Timeout。案级
case_timeout 3600 / 片级 time_limit 10800 对最重案均有余量。

**本地标定勘误**：cal1 的 delete/forwarddelete >1800s 是并发负载污染的
虚高（cal1 delete 跑时 pipe-smoke/bold 校准同时在跑）——CI 实际
case_timeout 3600 内完成。cal2 的 5000s 重测同样有 selection 面并发，
只作上界参考。

## slow 簇本地标定（/tmp/zw-rally calibrate，300s 脚本预算，并发负载下偏大）

- insertparagraph.html：1,244s 完整跑完（7,195 子测试 8 变体，单文件级
  wall；er3 片 CI 零 Timeout 实证预算充足）。
- delete.html / forwarddelete.html：1800 帽口 rc=124——虚高（见上），CI
  实测在 case_timeout 3600 内完成。

## 遗留背景作业

- `/tmp/zw-rally/calibrate.sh`（上轮遗留）：slow 六案逐案墙钟，只写
  /tmp/zw-rally/，与语料树无写交互。
- cal2：delete/forwarddelete 5000s 帽口墙钟重测，输出
  /tmp/zw-rally/calibrate2.log。
