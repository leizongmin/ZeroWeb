# M0：面单 + per-face 分母实测 + form 最小切片

日期：2026-10-29。通道：`make testharness-form-url-encoding`（本提交新增，
test-guard 包裹 `--per-proc-mem 4 --total-mem 8 --time-limit 5400` +
`ZW_SCRIPT_TIMEOUT_MS=600000`，语料同源复用 31 号 fetch）。全量数据：
四族分批 `--json` 实跑（同 envelope），逐文件 TSV 见
[2026-10-29-m0-per-file-denominator.tsv](2026-10-29-m0-per-file-denominator.tsv)。

## 面单（DC-1 之 1）

68 文件 = form 46 + href 22，通道 skip 规则（`form_url_encoding_case_skipped`）
只放行 `-encode-form*` / `-encode-href*` / `*-encoder.html`（schinese inline
a.href 面）/ `big5-enc-ascii.html`，与 31 号 decode 通道 face 互斥。分簇：

| 族 | form | href | 计 |
|---|---:|---:|---:|
| japanese（euc-jp / iso-2022-jp / shift_jis） | 6+6+11 = 23 | 4+4+4 = 12 | 35 |
| korean（euc-kr） | 12 | 3 | 15 |
| schinese（gbk / gb18030 `*-encoder.html`） | 0 | 2 | 2 |
| tchinese（big5） | 11 | 4 + `big5-enc-ascii` = 5 | 16 |
| **计** | **46** | **22** | **68** |

`encoding/` 顶层 `iso-2022-jp-encoder.html` 不在 legacy-mb 目录扫描面（31 号
语料口径即不含），不在 68 之列。

## per-face 分母实测与两账口径钉死（DC-1 之 2 + 缺口 4）

四族分批实跑合计 **766,959 子测试 / 817 case（文件×variant）**，与 wpt.fyi
aligned stable chrome summary 的 encode 面逐位一致（wpt.fyi 按 variant 独立
计文件：japanese 357 / korean 254 / tchinese 204 / schinese 2 = 817 条目；
单文件抽查 `eucjp-encode-form.html` 8 variant = 7,392 同表）。逐 face：

| 族 | form | href | 族合计 | wpt.fyi |
|---|---:|---:|---:|---:|
| japanese | 203,690 | 129,825 | 333,515 | 333,515 ✓ |
| korean | 196,696 | 43,264 | 239,960 | 239,960 ✓ |
| tchinese | 149,668 | 43,513 | 193,181 | 193,181 ✓ |
| schinese | 0 | 303 | 303 | 303 ✓ |
| **计** | **550,054** | **216,905** | **766,959** | **766,959** ✓ |

**口径差钉死**（立项三账：DC-3 767,673 / coverage 余量 767,316 / 本面
766,959）：差额全部在 schinese 族——wpt.fyi 该族 1,017 = encoder 面 303
（gb18030 254 + gbk 49，本 goal 范围）+ `*-decoder.any.js` 面 714（window
357 + worker 357，30 号 JS API 通道域）。DC-3 表把 schinese 全族 1,017 记入
encode 面（多算 714）；coverage 余量 767,316 扣掉了已覆盖的 window 357
（多算 worker 357）。两账同收敛于 **766,959**：

- 767,673 − 714（decoder .any.js window+worker，非 encode 面）= 766,959
- 767,316 − 357（worker variant，非 encode 面）= 766,959

后续 M1–M3 与 DC-4 对账统一用 766,959 口径。

## 基线首跑（通道产数）

766,959 子测试 = **125 Pass / 766,833 Fail / 1 Timeout**（0.016%）。非零
Pass 均为真实语义通过，非同源假通过：`big5-enc-ascii` 123 案（该测断言
`search == 期望字节序列 || search == 输入原文` 的双臂容差，ASCII 下 UTF-8
序列化同形）+ schinese 两 encoder 的 "very basic"（`encode("s","s")`）2 案。
Fail 主形态 = 预判的编码缺面（期望 euc-jp 字节 `%8E%DA`、实际 UTF-8
`%EF%BE%9A`），href 面（`a.search` rust-url UTF-8）与 form 面
（`_encPair` 硬编码 UTF-8）同构。

Timeout 1 案：`iso2022jp-encode-form-errors-stateful`（单子测试
promise_test——form 提交到 `data:text/html;charset=iso-2022-jp` action +
named iframe，依赖 data: URL iframe 脚本执行 + 子→父 postMessage 回传；
runner 侧 data: iframe 不走脚本执行通道，promise 恒不决，看门狗 600s 收
止）。该案不属 M0 断言链，M1 capability 落地时一并核对（届时若 data:
iframe 脚本面仍缺，按契约挂账）。

## form 最小切片（DC-1 之 3）

`eucjp-encode-form.html` 全 8 variant（7,392 子测试）端到端：`setup()` 建
2 iframe + 2 form（GET / `/common/blank.html` / `acceptCharset=euc-jp` /
`target=frame-N`）→ 灌 400 码点 `form.submit()` → named iframe 导航（s138
⑤b 分支）→ `iframe.onload`（R288 微任务派发）→
`contentWindow.location.search` 读回（iframeLocation 读 `doc._zwURL`）→
逐码点断言。**断言链全可达，无断点**；失败集中且单一 = 核心被测语义缺面
（acceptCharset 零消费 → UTF-8 序列化）。断言消息样例：
`expected "%8E%DA" but got "%EF%BE%9A"`（U+FF9A）。

## envelope 预评估（M1 依据）

- 单文件独跑（eucjp-encode-form.html，8 variant 7,392 子测试）：墙钟 64s。
- 分族墙钟（test-guard 5400s envelope 内，korean 与 japanese 两批并发、
  另有并行流通道同机，数字偏保守）：japanese ~9 min（含上述 600s timeout
  案）/ korean ~7 min / tchinese ~3 min / schinese <1 min。
- 结论：单族远在 5400s envelope 内，全量 767k 子测试 <20 min 量级；31 号
  M1 式「`ZW_SCRIPT_TIMEOUT_MS` 提额」已按先例内置（600s），无额外 envelope
  缺口。缺口清单 5（envelope）就此销账。

## A/B 对账（零回归）

- 引擎/shim 零变更（本轮仅 runner + Makefile 增量通道；31 号 decode 通道
  规则 `encoding_legacy_mb_case_skipped` 逐字未动）。
- decode 面烟测：`make testharness-encoding-legacy-mb FILTER=eucjp-decode-errors`
  全 Pass（重编后二进制实证 31 号通道行为不变）。
- 25 号 form-submission / 20 号 url 基线：本轮无引擎变更，无回归面；M1 起
  每轮 shim 变更带基线对账（master.md 碰撞管理节承诺不变）。

## M0 判据核对（DC-1）

- 68 文件分簇清单 ✓（本文件 + TSV）
- per-face 分母实测 + 口径差钉死 ✓（766,959，两账收敛）
- form 最小切片 euc-jp 1 案断言链端到端跑通 ✓（Fail 但链路可达，预期形态）
