# M2：href 面 encoding-parse（DC-3）

日期：2026-10-29。施工面：`text_encoding.rs` NCR 直出 + query `%` 归一 +
`part03.js` A/AREA 组件 getter encoding-parse 预编码。过程工件在
`/tmp/zw-form-url-m2/`（日志/JSON 不入库，本档记数 + per-file TSV 入库）。

## 施工面

- **host 桥**（`text_encoding.rs`）：
  1. **NCR 错误路径直出**——unmappable 的 HTML NCR 字节（`&#N;`）不经 set
     再判定，以预编码 `%26%23N%3B` 直接落输出（special-query set 不含 `&`
     `;`，set 路径在 query mode 会漏编界定符）。form mode 经 set 展开逐字节
     同形（等价路径，M1 form 面行为不变）。
  2. **query set 补 `%` 归一**——rust-url 保留裸 `%` 不归一 `%25`（Chrome
     消费侧归一，WPT iso2022jp-encode-href 期望依赖该归一），预编码源直接出
     `%25` 对齐（根因与排查见
     [learning](../../../../learnings/bugs/2026-10/2026-10-29-rust-url-preserves-bare-percent.md)）。
  - unit 6 案（`encode_percent` 过滤）全过：新案
    `test_encode_percent_query_ncr_direct`（EUC-JP/ISO-2022-JP query NCR 直出
    + form 等价锁 + euc-jp ¥ 0x5C 字面量 + ISO roman ¥ fresh + 裸 `%` 归一 +
    `&` len-1 guard）。
- **shim**（`part03.js` A/AREA 组件 getter，`__zw_parse_url` 调用点前）：
  `document.characterSet` 经 `_zwEncodingFromLabel` 归一非 UTF-8 + host 桥在位
  才分派；首 `?`（须先于首 `#`）切片，query 段
  `__zw_text_encode_percent(enc, q, 'query')`，`#` 起 fragment 原样透传
  （rust-url 既有 UTF-8 fragment 面）；解析失败回落仍用原始 aRaw。
  UTF-8/无桥/无 query 路径零变化；BASE/LINK 分支与组件 setter 不在消费面。
  href 面码域 0x80 起（ranges.js rangesAll），无 tab/newline 剥除交互。

## href 面收敛（DC-3 判据）

通道 `make testharness-form-url-encoding` 全量实跑：

| 面 | 子测试 | Pass | Fail | Timeout |
|---|---:|---:|---:|---:|
| form（46 文件） | 550,054 | **550,053** | 0 | 1 |
| href（20 文件） | 216,479 | **216,479** | 0 | 0 |
| href schinese encoder（2 文件） | 303 | **303** | 0 | 0 |
| big5-enc-ascii | 123 | 123 | 0 | 0 |
| **合计（=M0 分母 766,959）** | **766,959** | **766,958** | **0** | 1 |

- M1 基线 550,178P/216,780F/1T（71.7%）→ **766,958P/1T（99.9999%）**。
  唯一非 P 为既录 `iso2022jp-encode-form-errors-stateful` 单子测试 Timeout
  （runner data: URL iframe 脚本通道缺失，M0 缺口清单 2 挂账不变）。
- M1 挂的 gb18030/gbk-encoder 硬编码期望面 301F（`%81@`/`%949%DA3`/
  `%26%2358853%3B`/GB18030-2022 36 案）随 NCR 直出 + `%` 归一一并翻转全 P。
- iso-2022-jp href 面 43,275 全 P（errors-han NCR 期望 21,269 + 主面 7,392 +
  errors-hangul/misc）。
- 逐文件账：[2026-10-29-m2-per-file-post.tsv](2026-10-29-m2-per-file-post.tsv)。

### 首轮 211F 与修复

首轮（NCR 直出已落、`%` 归一未落）216,268P/211F，失败全集
`iso2022jp-encode-href.html`，断言消息 `Invalid code point NaN`：ISO-2022-JP
JIS 字节域 0x21-0x7E 含合法 `%` 字节（Ε U+0395 → `ESC $ B 0x26 0x25`），裸
`%` 被 rust-url 原样保留（Chrome 归一 `%25`），WPT normalizeStr 把尾部裸
`%` 解成 NaN 抛错。修复（query set 补 `%`）后定向复跑 43,275P/0F，全量见上。

## A/B 对账（DC-3 判据：零回归）

- **31 号 decode 面（零回归底线）**：`testharness-encoding-legacy-mb` 全量
  JSON 与 M1 基线**逐字节一致**（38,910,656 B，md5 `f73af333…`）——360,431
  P / 0 F 逐位不变。
- **25 号 form-submission-0**：`testharness-html-semantics
  FILTER=form-submission`，M1 post 基线 216 腿逐行比对**零翻转**；新增 16 腿
  全为语料新增文件（dialog-form-submission ×10 Pass +
  form/rel-*-target ×6 Timeout——runner 导航 target 既有挂账类，非编码语义）。
- **20 号 url 域**：`testharness-net-api FILTER=url`，pre（今日 20:10，M2
  改动前 binary）与 post（21:19）全量 leg 集**逐字节一致**（6,261 legs =
  6,116P / 145F / 0T，Fail 逐文件分布 diff 空）——零 P→F。注：较 M1 evidence
  记录（6,065 legs/141F）多 196 legs/4F，为两次运行之间语料刷新所致，pre/post
  同含，与 M2 改动无关。
- **附带 diligence**（契约外）：`testharness-encoding` 全通道 12,072P/3F，3F
  与 encoding-compat 2026-10-01 基线记载同类（streams/decode-utf8 2 +
  textdecoder-copy 1，M1 基线即记 decode-utf8 10/12），非 M2 引入。
- form 面 550,053/1T 不动（全量 run 内含）。

## 门禁

- `cargo fmt --all -- --check` 干净；`cargo clippy --workspace --all-targets
  -- -D warnings` 通过（systemd scope 24G/RuntimeMaxSec 3600 守卫）。
- 单测经 `./target/test-guard --compile-first` 包裹；通道经
  `target/test-guard`（4/8G/5400s）+ `ZW_SCRIPT_TIMEOUT_MS=600000`。

## M2 判据核对（DC-3）

- href 面 encoding-parse 消费点落地（getter 解析点预编码 + host 桥 NCR 直出
  + `%` 归一）✓
- href 面 216,479 全收敛（超出预期：encoder 301F 同轮翻转）✓
- 31 号 decode + 25 号 form-submission + 20 号 url 零回归同轮带；form 面
  550,053 不动 ✓
- 通道基线更新落 evidence（本档 + TSV）✓
- 遗留挂账：`iso2022jp-encode-form-errors-stateful` Timeout（runner data:
  iframe 管道，M3 收口按契约挂账或随 runner 能力顺带解锁）
