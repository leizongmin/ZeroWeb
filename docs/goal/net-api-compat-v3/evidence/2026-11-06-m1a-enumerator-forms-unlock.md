# M1a：通道枚举器窗体解锁（.htm + .window.js）

日期：2026-11-06。改动：`tests/wpt-runner/src/testharness.rs` 共享枚举器
`run_any_js_corpus_subdirs_with_helpers_variants` 增 `extra_forms` 门（默认关，
仅 net-api 通道开启）：`.htm`（xhr corpus 上游老式命名，同 `.html` 形态走
substitution + `run_testharness_html_inner`）与 `.window.js`（window 变体，
webmessaging/webstorage 通道既有先例，`run_any_js_window_case` 形态无关）。
skip 口径同步：`-manual.htm` 与 `-manual.html` 同交互面排除；websockets
`back-forward-cache-*.window.js` 显式排除（RemoteContextHelper 多窗口 bfcache
编排，runner 无多文档/bfcache 管道——M0 甄别档排除口径）。

## 收益（七域对照，逐行计数）

| 域 | M0 基线 | M1a 后 | 增量 |
|---|---|---|---|
| fetch | 1,900P/189F/2T | 1,912P/276F/2T | **+12P** +87F |
| xhr | 586P/7F/1T | 903P/346F/5T | **+317P** +339F+4T |
| websockets | 377P/0F/0T | 377P/0F/0T | 不变 |
| streams | 917P/96F/5T | 918P/96F/5T | **+1P** |
| url | 5,924P/141F/0T | 7,196P/216F/0T | **+1,272P** +75F |
| mimesniff | 1,296P/602F/0T | 1,296P/602F/0T | 不变 |
| eventsource | 35P/0F/0T | 57P/10F/4T | **+22P** +10F+4T |
| **合计** | **11,035P/12,078** | **12,659P/14,221** | **+1,624P** +508F+8T |

案数 543 → 684（+141：xhr +201、url 大面 .htm/.window.js、eventsource、
fetch 窗体余量）。新 Fail 面 = 解锁面真实缺口显影（access-control preflight
族 corsca.py 端点、h1-parsing 原始线缆端点、多窗口编排等）——M1b/M2 fixture
分批与 M3 失败簇归因的输入，不是回归。

## 零回归核验（DC-4 零回归门）

- 同机 A/B：M0 基线 log × M1a 全量 log 按 (case, subtest) 对比——交集
  **12,070 对，CHANGED 0，GONE 0**（逐对状态字节一致）。
- 其余共用枚举器通道（encoding / legacy-mb / form-url-encoding /
  navigation / console / html-syntax）`extra_forms=false`，`is_htm`/
  `is_window_js` 被 `&&` 短路为 false，枚举行为编译期等价——通道级隔离
  （run-rules §10），各通道既有账分母不动。
- 编译面：clippy `-D warnings` + rustfmt 干净。

## 口径记账

1. url/mimesniff v2 划界「不重开」指引擎语义面收敛归 v2 口径；本批 url 面
   +1,272P 是**通道枚举窗体解锁的增量入账**（此前未跑案，非引擎改动），
   零回归门以逐对字节一致判，不违划界。
2. 基线账更新：通道账基线由 11,035/12,078 换代为 **12,659P/14,221**（M1a
   后新常态）；后续批次 A/B 以同树重跑对照为准。
3. mimesniff/websockets 两域无新窗体文件，合计行不变——侧证解锁面无越界。
