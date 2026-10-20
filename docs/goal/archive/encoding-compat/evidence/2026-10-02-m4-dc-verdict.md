# M4 / 收口判定 — DC-1~DC-4

**日期**: 2026-10-02
**判定**: **目标 Completed**——DC-1~DC-4 全满足。

## DC 逐项判定

| DC | 判定 | 证据 |
|---|---|---|
| **DC-1** corpus 导入 + 分类基线 + CSV 转数据行 | ✅ | encoding/ 212 文件拉取（top/resources/streams/legacy-mb 七编码，pin 315976933870b34d6ea30e3f6643403edae678ba）；runner `testharness-encoding` 通道 + Makefile 双 target；基线 268/4403 = 6.1% 落 evidence/2026-10-01-m1-baseline.md + JSON；wpt-suites.csv 数据行（语料缺角 FORCE=1 补齐记账于 M2 证据口径注记） |
| **DC-2** labels 全表 + legacy 解码修齐、通过率可追踪 | ✅ | encoding_rs host 全表（labels 匹配/单字节 28 编码/GBK/GB18030/Big5/Shift_JIS/EUC-x/ISO-2022-JP 状态机/replacement/UTF-16）；6.1% → 98.4% 可追踪（CSV 三行轨迹）；legacy-mb `.any.js` 357/357、api-invalid-label 3421/3421、single-byte 336/336、gbk 82/82、gb18030 275/275、iso-2022-jp 34/34（evidence/2026-10-02-m2-labels-legacy-decode.md） |
| **DC-3** TextEncoder + BOM/fatal/ignore + 往返 | ✅ | utf-8 spec 逐字节状态机 + BOM 拆分嗅探 + fatal/ignoreBOM + encode 默认参数/孤立代理 U+FFFD + encodeInto spec 语义 + streams 93.9%；12070/12075 = **99.96%**（evidence/2026-10-02-m3-textencoder-modes-streams.md） |
| **DC-4** make test 全绿 + clippy + fmt + reftest 零回归 | ✅ | make test 68 suites 全绿（2026-10-02，M3 后）；cargo clippy --workspace --all-targets -D warnings 零告警；cargo fmt --check 零 diff；**make reftest 700/700、不一致 0**（2026-10-02 复跑——100.0% 文本面，与守成恒值同值） |

## 残差定稿（5/12075 = 0.04%，不阻收口）

- **transfer 结构面 ×4**（streams/decode-utf8 transferred chunk ×2、encodeInto detached
  output ×1、textdecoder-copy SharedArrayBuffer ×1）：detached ArrayBuffer/SharedArrayBuffer
  构造与 postMessage transfer 管道——workers/structured-clone 域基建，非 encoding 语义；
  重入条件 = transfer/structured-clone 管道落地（navigation/workers 流域）。
- **encoding_rs 契约 ×1**（textdecoder-mistakes fatal stream: iso-2022-jp）：spec 要求
  iso-2022-jp fatal 错误后 ESC 模式机跨错误保持（唯一不清状态编码）；encoding_rs 错误
  契约要求错误后弃用 decoder——上游契约边界，随 encoding_rs 升级重评。

## 挂账定稿（P5）

文档级编码嗅探（`<meta charset`/BOM 嗅探 → 文档解码、`document.characterSet`）按立项
划界**不属本 goal**，已双向记账转入 **html-syntax-compat** 控制面缺口清单 P5
（2026-10-02，其 M1 corpus 批次可一并基线 encoding/ 域已拉取的 bom-handling/eof-*/
utf-32*/sniffing 案面）。本 goal 运行面 skip 规则（encoding_case_skipped）与 fetch
脚本头注释已固化该划界。

## 共享面核对（run-rules §10）

- net-api 六 corpus 逐域同值零漂移（M2/M3 各复跑一次：9319/11688，streams 905/1020
  恒值、xhr 545→557 正漂移）。
- 渲染流（并行 stream）：reftest 700/700 恒值零回归；本 goal 未触渲染域文件。

## 交接清单

- 测试资产：host 单测 11（text_encoding.rs）+ shim 集成测 1（part02.rs）随 make test
  常驻；WPT corpus 走 `make testharness-encoding`（test-guard 包裹）可复跑。
- 后续数字维护：wpt-suites.csv 每次全量跑后追加行（生成器取最新行）。
