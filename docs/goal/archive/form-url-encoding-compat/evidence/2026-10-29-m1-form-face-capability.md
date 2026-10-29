# M1：runner 通道 + form 面能力（DC-2）

日期：2026-10-29。施工 commit `da608f159`（rebase 后 HEAD）：
`text_encoding.rs` encode 方向桥 + `part04.js`「select a form submission
encoding」步 + callbacks 注册。过程工件在运行 session `/tmp/zw-form-url-m1/`
（JSON 100MB 级不入库，本档记数 + per-file TSV 入库）。

## 施工面

- **host 桥**（`crates/engine/src/js_dom_bridge/text_encoding.rs`
  `text_encode_percent(label, text, mode)`）：legacy 编码逐 scalar fresh
  编码（ISO-2022-JP 剥 per-scalar `ESC ( B` flush 尾巴）+ URL standard
  percent-encode after encoding 字节级 set——mode=form（urlencoded set +
  空格 `+`）/ mode=query（special-query set，M2 预留）；unmappable →
  HTML NCR；replacement/UTF-16 系输出编码 = UTF-8
  （get-an-output-encoding）。unit 5 案新测试过（模块 16/16）。
- **shim**（`part04.js` `_zwFormSubmitNavigate`）：accept-charset 逐 token
  `_zwEncodingFromLabel` 归一首个合法者胜，全非法/缺省回落
  `document.characterSet`；仅 urlencoded 面 `_encPair` 分派消费，非 UTF-8 +
  host 桥在位走 `__zw_text_encode_percent`，UTF-8/无桥路径纯 JS 零变化。
- **per-scalar fresh 模型**（非整串流式）：WPT encode-form 期望按逐码点
  独立预计算（encode-form-common.js split(separator) 逐段比），Chrome 同构；
  stateful 编码整串流式产出多出模式切换序列全 FAIL。定谳过程与指纹见
  [learning](../../../../learnings/patterns/2026-10/2026-10-29-wpt-encode-form-per-scalar-fresh-encoding.md)。

## form 面基线（DC-2 判据 1）

通道 `make testharness-form-url-encoding`（M0 落地），修复后全量实跑两轮
JSON 逐字节一致（`full-face.json` ≡ `m1-encode-full.json`，确定性复跑）：

| 面 | 子测试 | Pass | Fail | Timeout |
|---|---:|---:|---:|---:|
| form（46 文件） | 550,054 | **550,053** | 0 | 1 |
| href（20 文件） | 216,479 | 0 | 216,479 | 0 |
| href schinese encoder（2 文件） | 303 | 2 | 301 | 0 |
| big5-enc-ascii | 123 | 123 | 0 | 0 |
| **合计（=M0 分母 766,959）** | **766,959** | **550,178** | **216,780** | 1 |

- form 面 99.9998%：唯一非 P 为既录 `iso2022jp-encode-form-errors-stateful`
  单子测试 Timeout（data: URL iframe 脚本通道缺失，M0 缺口清单 2——**按契约
  挂账**，acceptCharset 能力已落地、卡点是 runner data: iframe 管道，非本
  goal 编码语义）。
- euc-jp 6 文件 58,059 P / 0 F（commit 验证轮，含 errors NCR 三变体）。
- 逐文件账：[2026-10-29-m1-per-file-post.tsv](2026-10-29-m1-per-file-post.tsv)。
- href 面 216,780 F 全部为 M2 缺面（期望目标编码字节、实际 UTF-8，与 M0
  基线形态同构），M2 起收敛。

## s138 挂账④等价面（25 号归口判据）

`zw-windows-1252-urlencoded-probe.html`（25 号流 s138 挂账④的等价探针，
probe-w1252.json）：**4/4 Pass**——windows-1252 mappable `%E9%80` / unmappable
NCR / lone surrogate USV 转换 / 混合串。

## A/B 对账（DC-2 判据 2：零回归）

- **31 号 decode 面（零回归底线）**：修复后全量 `decode-face.json`——
  385 case **360,431 P / 0 F**，与 31 号基线逐位一致。
- **25 号 form-submission-0**（共享路径 `_zwFormSubmitNavigate`）：pre/post
  全量 A/B（`ab-form-submission-pre.log` vs `ab-form-submission2.log`，
  216 vs 215 行）翻转 **11 legs 全部为挂账④等价面 F→P / Timeout→4P**：
  urlencoded2「characters not in encoding」×4、「lone surrogate」×2、probe
  文件级 Timeout → 4 子测试 P。**P→F 0**。（另一次 post 轮 15:21 因注册
  泄漏面少 23 行，set 与 15:22 全量轮在共有行上一致。）
- **25 号 html-semantics 全通道**（附加 diligence，`hs-full.json`）：
  940 case 4,432 P / 795 F / 108 T / 11 NotRun / 1 Unsupported，与 s148
  切片态量级一致（该面 base 线为 25 号流维护，本 goal 不改判）。
- **20 号 url 域**（每轮承诺，本轮 shim 变更不触 url 面）：全量
  FILTER=url 实跑 6,065 子测试 = **5,924 P / 141 F / 0 T**。收口档基线
  169F（url-setters 13 + USP BOM 28 + 单案面 128）；今日 141F = url-setters
  **13 同** + 单案面 **128 同** + USP BOM **28→0**（20 号流 M2-S17
  `f0e69f2fd` 已收口，归因闭合：169−28=141 逐文件分布吻合）。**零 P→F**。

## M1 判据核对（DC-2）

- acceptCharset 序列化落地（编码 override + `&#NNNN;` 回退 + USV 转换）✓
- form 面 skip 解除（M0 已放行）、通道基线落 evidence ✓（本档 + TSV）
- 25 号 form-submission 与 31 号 decode 面零回归 ✓（url 域同轮带 ✓）
- 遗留挂账：`iso2022jp-encode-form-errors-stateful` Timeout（runner data:
  iframe 管道，非编码语义；M3 收口时按契约挂账或随 runner 能力顺带解锁）
