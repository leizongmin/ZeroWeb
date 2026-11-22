# DC-2 · 批十四：CI 周跑 infra 缺档根因定谳＋62 脚本支撑文件补档

2026-11-22。11-21 周跑快照 `html/infrastructure zw_passes=82` 对本地批十三后
实测 674 的 8 倍差定谳（DC-3 对账前置勘误腿，无 Rust 码笔）。

## 方法

下载 run 44928234880 catch-all-3 作业产物 `zw-results-catch-all-3`
（retention 3 天内），逐键比对本地批十三后全量跑（批十三证据同源）。
三个先期假设（语料限流洞／TIME_LIMIT 截断／CI 端点缺位）全部证伪：

- fetch 步「失败 0」，62 脚本清单内文件全到位（pin 315976933）；
- 套件墙钟 106s，5400s 帽内，无截断；
- py 端点缺位只影响批十三新面（CI 检出为批九码，本地同码基线 550 已对齐）。

CI 事实：132 键全跑（与本地同键集）、386 子测试（本地 1191）、case 级
19 P / 30 F / 83 TO。

## 根因（goal 脚本域两条）

1. **分片语料缺 9 支撑文件**。CI 周跑按分片只跑本 goal 的 fetch；本地这些
   文件由 workers／encoding／html-syntax 等其他 goal 通道带入，分片作业拿
   不到。缺档形两种：
   - `load support "No such file or directory"`——messagechannel.any.js
     （本地 151 P）与 serialization-via-idb.any.js 加载
     `common/sab.js`、`IndexedDB/resources/support.js` 失败；
   - **`subsetTestByKey` 未定义 → 0 注册 TO**——query-encoding 5 基页 ×
     16 include ＝ 80 键全灭（`common/subset-tests-by-key.js` 缺档；
     resolve-url.js 全部注册走该入口），报
     `testharness completed without reporting registered tests (tests:0)`；
     worker 变体同链两键。
   发现方法：META script= 传递闭包扫描（159 文件，8 文件在 fetch 目录外）
   ＋ CI 逐键比对补全 HTML `<script src>` 直引（META 扫描不覆盖）。
2. **links/icon no-load-event.svg 本地同缺**（`/images/wpt-logo/…svg`，
   testharness 页消费），顺带补档；本地该键 tallies 补档前后逐项相同
   （runner 不取 icon 资源）。

另有 `select/options-length-too-large.html` 的 0 注册 TO：本地同形——批十三
before/after 与批十四复跑三份全量产物中该键均为同一 Timeout 消息（30s 案帽下
`options.length = 100001` 慢路径超限，engine select 域，同 page-load TO 面候选
形），不属缺档、亦非 CI 特异。本证据初稿「本地通过＝CI 速度敏感嫌疑」表述
有误，勘正如上（后经本地过滤单跑复现＋三份全量产物逐份核对）。

## 修复与验证

- `62-wptfyi-html-infra.sh` ＋10 `fetch_raw`（9 支撑文件 + svg），批十四
  注释块记根因；`bash -n` 过；本地拉取失败 0（svg 761B 落盘，余 9 文件
  已在盘跳过）。
- 本地无漂移证明：同 b13-after 二进制（HEAD 同码）复跑 infra 全量 132 键，
  131/132 键逐键 tallies 与批十三 after 逐项相同；唯一漂移键
  messagechannel.any.worker.js＝批十已记录抖动带键（48P/20F/1T/1PF，
  历史形态 102→29→115→48）。
- 归约口径注记：CI 子测试 121 vs 快照 82 的差为 wptfyi_gap.py variant 键
  归约，不另勘误。

## 待实证

44972755575（批十三码＋缺档语料，预期复现 82 形＝对照数据点）；本批
commit 后另派 coverage-only 周跑取新快照做 DC-3 对账（预期 infra 面
~600+）。

[逐文件 JSON](2026-11-22-dc2-batch14-ci-corpus-support-files.json) ·
[批十三证据](2026-11-22-dc2-batch13-query-encoding-py-fixtures.md) ·
[批十证据](2026-11-22-dc2-batch10-query-encoding-getsub.md)
