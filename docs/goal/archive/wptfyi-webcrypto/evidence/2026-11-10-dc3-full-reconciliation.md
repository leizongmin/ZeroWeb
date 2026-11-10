# DC-3 · 全量对账收账（2026-11-10）

本轮四件事：① 周跑 run 42667268541 收尾监控；② 聚合快照落库核验；③
净增账与邻域核验；④ 收口门禁与账面封账。

## 一、run 收尾

- run [42667268541](https://github.com/leizongmin/ZeroWeb/actions/runs/42667268541)（dispatch @ `4482feb56`）42 个 matrix job 全部回齐。crypto 分片
  success（10m40s 含 fetch/build，语料 38 号 v3 全量），产物逐 key 对本地
  r6 完全一致——70,627 keys / 0 漂移（核验见
  [DC-2 evidence 第六节](2026-11-10-dc2-generatekey-enforcerange.md)）。
- catch-all-8（subresource-integrity + test262，time_limit 5400s）runner
  杀树 → 分片红（exit 1，test262 单套件体量超单分片预算，归属
  wptfyi-runtime 流的分片预算课题）。聚合按 757f4db84 预案容错：红面按
  未跑同态记 0，不吞快照（`wptfyi-coverage-aggregate` job 全步 success，
  本 run 终态 completed:failure 仅由该红腿贡献）。

## 二、快照落库

bot commit `003efe86b`（2026-11-09T19:16:32Z）回写
[coverage-2026-11-09](../../../../compat/wptfyi/coverage-2026-11-09.md)，basis
= 三家 aligned stable `792c7512e3`（与 DC-1/DC-2 本地双 basis 试跑同一
revision）：

| 项 | 11-07 发布快照 | 新快照 |
|---|---:|---:|
| WebCryptoAPI 通过 / 分母子测试 | 26,779 / 70,872 | **67,877 / 70,920 = 95.71%** |
| WebCryptoAPI 覆盖 / 全过文件 | 137 / 137 | 326 / 294 |
| 全量通过子测试 | 1,551,067 | 1,603,003 |
| 全量文件分母 | 126,638 | 126,709（+71 上游漂移） |

WebCryptoAPI 数值与本地 r6 口径 coverage 预告值逐位一致（95.71%，DC-1
evidence 第六节同 basis 试跑）；zw_env_fails 0、可疑条目 2 条与前快照
持平。

## 三、净增账

**67,877 − 26,779 = +41,098 ≥ 31,000 达标线**（coverage-r7 本地双 basis
口径为 +41,898，见 DC-2 evidence 第四节）。

CI 账与本批 generateKey 修复的 +800 差额：run HEAD `4482feb56` 早于
`4cba77f7e`/`f3d81180c`，800 转正未入本快照，属预期；r7 本地终账
（70,627 keys，锚对照零回归）与 part53 契约已落 DC-2 evidence，随下次
周跑自然入账。不再单独 dispatch：达标线已过，且 weekly concurrency
cancel-in-progress 会砍在跑 run 尾巴。

## 四、邻域核验（compare-topdirs，`.acceptance` 诊断脚本不回库）

新快照对 11-07 发布快照逐 topdir 对照，WebCryptoAPI 之外全部变化归属
他流/上游漂移：

| 归属 | topdir 变化 |
|---|---|
| 本目标 | WebCryptoAPI **+41,098**（唯一 webcrypto 可达面） |
| runtime 流 | encoding +10,925（legacy-mb 语料接入） |
| css 流 | css +566 |
| 其他增益 | websockets +276、fetch +104、input-events +13 等零星 |
| 上游分母漂移 | mathml +134、reporting +42、url +22、web-monetization 新目录等 |

回退 5 项（无一 webcrypto 可达）：

| topdir | Δ vs 11-07 | 定性 |
|---|---:|---|
| IndexedDB −1 / editing −7 / web-animations −2 | 微量 | 前快照（16:12Z 落库的旧 11-09 账）已存在同值，纯携带 |
| dom | −105 | 前快照 −142，本 run 回升 +37 |
| shadow-dom | −981 | 前快照 −490，本 run 新增摆动 **−491** |

shadow-dom 摆动归因：covered_files 不变（172，同面同语料），pass 级
翻转；本 run HEAD 差（`a73228e22..4482feb56`）内非 docs 引擎提交 =
渲染流 s5482/s5483（c2880ee2c / 6e540963e）+ editing 流 T53
（1893d9d10）+ runtime 流 test262 折叠（2214db000，test262 域内）。
webcrypto 触面（part01b/part02 shim、crypto 分片 fetch/预算、part53
契约）无任何 shadow-dom 路径可达。

**产物级法证定谳（2026-11-10 并行席补充）：页超时截断摆动，零行为
翻转，不立流级挂账。** 以 16:12Z cron 快照（`8ba9975d9`）为近基线复
比 + 两 run `zw-results-*` 产物逐字节对拍：

- 组差全部落单文件：`shadow-dom/declarative/gethtml.html` 3,015 →
  2,524 子测试（Pass 同落 −491），前缀 2,523 状态逐位一致，两 run 尾部
  同为文件级 Timeout 条目（message "testharness page timeout completed
  the file with 1 test(s) unreported"）——runner 页超时预算内跑多少算
  多少，新 run html-sem job 26:01 → 28:38（hosted runner 速度方差）
  少跑 491。
- 同形态佐证：editing/run `zz-probe-f6bwin.html` 分片 −651（universe
  外探针页，不进组账面，仅证 runner 方差）、css-writing-modes
  `text-input-vertical-overflow-no-scroll.html` −1（7 测试未报截断）、
  dom/ranges `Range-mutations-replaceData.html` +37（正方向截断差，
  621 → 658）。四笔前缀均零状态翻转；html-semantics 产物两 run md5
  相同。
- 结论：第四节的渲染/editing 流待查项与 11-08 掉零前科、r5270↔r5296
  竞态形态同根——env 竞态（页超时截断）定谳，渲染/editing 流无债。

## 五、收口门禁

- 本轮仅 docs（本文件 + master.md）。engine crypto 面（shim part01b/
  part02、part53.rs）自 f3d81180c 零变化（git log 路径过滤空），DC-2
  evidence 的契约 **12/12 + 1/1**、clippy `-D warnings`、fmt check 证据
  在当前代码态持续有效。f3d81180c 后 engine 仅 f619446a1（shadow 选择
  器门，他流写面，晚于本 run HEAD）。
- 快照产物非空、命中真实路径与变体：WebCryptoAPI 覆盖文件 137→326
  （+189 = 分片路径收录 + 新族），全过文件 294；分片收录机制经 DC-1
  机制验证（covered +74 全为分片路径）。
- 目标三里程碑收账：DC-1（2026-11-09 evidence）、DC-2（2026-11-10
  evidence）、DC-3（本文件）。同 revision 路径级证据链完整：r6/r7 全量
  JSON（`.acceptance/wptfyi-webcrypto/`）+ CI 分片逐 key 对账 + 本快照
  三方一致；上游漂移另报（`792c7512e3` 对快照 basis +2 路径 +48 子
  测试，DC-1 evidence 第三节）。

## 六、遗留（不在本目标范围）

- test262 单分片预算（catch-all-8 5400s 杀树）：runtime 流课题。
- ~~shadow-dom −491 摆动~~：已定谳页超时截断 env 方差（第四节产物法证），
  渲染/editing 流无课题。
- crypto 分片 time_limit 3600（本地实测 238s）未再收敛：保守余量维持，
  CI 实测 10m40s（含 fetch/build）已留案。
