# DC-4 coverage 复测收口（2026-10-29）

## 批量运行终态

- 54 通道（51 Makefile 镜像主批 + CI 对齐补跑 canvas/canvas-worker/constraints），
  顺序 `--json` 实跑，结果目录与台账在仓库外与仓库同级的 `zw-coverage-dc4/`
  运行目录（`status.tsv` 逐通道 rc+JSON 字节数、`driver.log` 全程轨迹、
  `results/*.json`；关键台账快照已随本提交存 evidence）。
- **收口判据核对**：`status.tsv` 全行 rc∈{0,1}（54/54；rc=1 = runner 语义
  「完成但有失败子测试」，JSON 完整即 coverage 合格输入）；
  `results/*.json` 54/54 可解析。判据原始文本见 goal 文档 v1.7 头部。
- 主批重试链按 v2 语义收敛：rc=1 为合格完成态不再重跑；dom（900s envelope
  偏紧）与 uievents 放宽时限手动补跑收口（driver.log MANUAL-RETRY 节）。

## canvas 通道 rc=101 定谳与修复

- 现象：canvas 通道 3m20s 处 rc=101、JSON 0 字节，stderr =
  `testharness.rs:5003:23: end byte index 1024 is not a char boundary; it is
  inside '（' (bytes 1022..1025)`。
- 根因：`sniff_document_encoding`（s122 文档编码嗅探，DC-1 引入）的 1024
  字节 prescan 窗按字节直切 `&html[..min(len,1024)]`；canvas 语料某文档
  byte 1024 落在全角字符字节中段 → panic 杀整通道。共享路径——任何通道的
  多字节文档语料都可能踩中，canvas 只是首踩。
- 修复：截断点回退到最近 char boundary（至多丢窗尾一个不完整字符，spec
  prescan 本按字节流截断、str 视图只能按字符边界收缩）；回归钉
  `sniff_document_encoding_window_multibyte_boundary`（跨界输入修复前 panic /
  现回退 None；边界对齐输入窗恒等 `for_label("euc-jp").name() == "EUC-JP"`）。
  修复与回归钉同提交。
- 复跑：test-guard 包裹（--per-proc-mem 4 --total-mem 8 --time-limit 2700），
  rc=1（合格完成态）、JSON 369,321 字节，status.tsv 行原位更新。

## coverage 复测数字（docs/compat/wptfyi/coverage-2026-10-29.md）

- 全量分母 125,445 文件 / 2,267,219 子测试；ZW 覆盖 8,627 文件（6.88%），
  通过 548,502 子测试（**24.19%**）——对比上一期周报头条 7.41%。
  本轮先出的 07:23 版报告（canvas 补跑前）为 24.10%/6,415 文件，canvas 通道
  入账后刷新为终稿数字。
- legacy-mb 四族对账（DC-4 判据主体，wpt.fyi 分母计入、通过率如实）：

| 族 | 分母子测试 | 通过 | 覆盖文件 | 未覆盖 |
|---|---:|---:|---|---:|
| encoding/legacy-mb-japanese | 447,723 | 114,208 | 125/482 | 333,515 |
| encoding/legacy-mb-korean | 410,448 | 170,488 | 181/435 | 239,960 |
| encoding/legacy-mb-tchinese | 268,916 | 75,735 | 79/283 | 193,181 |
| encoding/legacy-mb-schinese | 1,017 | 357 | 2/6 | 660 |

- **清零路径成立**：三族通过合计 114,208+170,488+75,735 = **360,431**，与
  M2 全量基线（decode 面）逐字对上；四族未覆盖余量 767,316 = M3 甄别归口的
  encode 面（form/href 可跑性甄别 not-run，非解码管道缺口）。原「0/482 文件、
  runner 文档解码管道缺失」形态消失。
- 顺带受益面记账：单字节遗留编码文档形态随 DC-1 管道自动解锁——
  net-api 通道 `websockets/constructor/016.html`（windows-1252 文档）由
  read-Err skip 转为如实可跑（M0 零回归对账实证）。

## 提交物

- `tests/wpt-runner/src/testharness.rs`：char boundary 修复 + 回归钉。
- `docs/compat/wptfyi/coverage-2026-10-29.{md,json}`：DC-4 全量对账报告。
- `docs/compat/trends/wpt-suites.csv`：encoding-legacy-mb 行转 2026-10-29
  数据行（24.19% 头条）。
- 本文件 + status.tsv 副本（`2026-10-29-dc4-status.tsv`）。
