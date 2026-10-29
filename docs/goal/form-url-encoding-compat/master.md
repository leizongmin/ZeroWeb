# form-url-encoding-compat master

**状态**: Active——M0 完成（2026-10-29，DC-1 判据全落）；M1 待启动
**入口**: [form-url-encoding-compat.md](../form-url-encoding-compat.md)

## 里程碑状态表

| 里程碑 | 内容 | 判据 | 状态 |
|---|---|---|---|
| M0 | 面单 + per-face 分母实测 + form 最小切片 | DC-1 | **完成**（[evidence](evidence/2026-10-29-m0-face-list-denominator-slice.md)） |
| M1 | runner 通道 + form 面能力（acceptCharset 序列化） | DC-2 | 未启动（通道已就位，差 shim 能力） |
| M2 | href 面 encoding-parse | DC-3 | 未启动 |
| M3 | 基线收口 + coverage 复测 | DC-4 | 未启动 |

## 验证基线

- **通道**：`make testharness-form-url-encoding`（2026-10-29 落地，commit
  `7ae6909cf`；test-guard 4/8G/5400s + `ZW_SCRIPT_TIMEOUT_MS=600000`，语料
  同源复用 31 号 fetch；filter 按路径子串，`FILTER=encode-form` 分面）。
- **encode 面基线（M0 首跑）**：766,959 子测试 / 817 case =
  125P / 766,833F / 1 Timeout（0.016%）。分母与 wpt.fyi aligned stable
  逐位一致；逐文件账
  [TSV](evidence/2026-10-29-m0-per-file-denominator.tsv)。
- **统一口径**：encode 面分母 766,959（立项 DC-3 767,673 与 coverage 余量
  767,316 的差额已钉死为 schinese decoder `.any.js` 面 714 = window 357 +
  worker 357，非本 goal 范围）。M1–M3 及 DC-4 对账用 766,959。
- 零回归参照：31 号 decode 面 360,431P/0F（本轮 decode 烟测 Pass，无引擎
  变更）；25 号 form-submission 与 20 号 url 基线 M1 起每轮带。

## 缺口清单

1. **form 面——序列化编码 override**（核心，M1）：`_encPair` 硬编码 UTF-8
   （part04.js:11289），acceptCharset 零消费。设计要点（M0 侦察）：shim 读
   form `accept-charset` 属性 → `_zwEncodingFromLabel` 逐 token 归一，首个
   合法者胜，全非法/缺省回落 `document.characterSet`（host 槽优先 + meta
   prescan 兜底，part06.js:5557）；非 UTF-8 编码需对称 host 桥
   （`__zw_text_decoder_*` 在 js_dom_bridge.rs 的 encode 方向）出目标编码
   字节 + urlencoded percent-encode set（`*-._` 与 alnum 字面量、空格→`+`）
   + 不可映射字符 `&#NNNN;` 回退 + lone surrogate USV 转换步（s138 挂账④
   的 urlencoded2 4 案即判据）。UTF-8 路径零变化（既有通道零回归面）。
2. **form 面——`iso2022jp-encode-form-errors-stateful` 特例**（M1 一并
   核对）：单子测试 promise_test，链路含 data: URL iframe 脚本执行 + 子→父
   postMessage 回传，runner 侧 data: iframe 不走脚本通道（M0 实测 Timeout
   600s）。acceptCharset 落地后若该面仍缺，按契约挂账。
3. **href 面——encoding-parse a URL 消费点**（M2）：`a.href` setter +
   `a.search` 读全 rust-url UTF-8；实现路径 = shim setter 以
   `document.characterSet` 为 fallback encoding 预编码 query 后交 rust-url，
   可行性甄别随 M2。
4. ~~口径差~~ 已销账（M0 钉死 766,959）。~~envelope~~ 已销账（单族
   ≤9 min，5400s envelope 富余；timeout 案见缺口 2）。

## 碰撞管理记录

- **2026-10-29 认领**：25 号 html-semantics-compat s138 挂账 ④
  （windows-1252 accept-charset + `&#NNNN;` 实体回退 + lone surrogate
  USV，urlencoded2 4 案）归口本 goal M1——同一能力（acceptCharset
  序列化）的单字节前哨，25 号流不重复施工。
- **边界**：20 号 net-api-compat url 主干不动；text/plain、multipart、
  File entry 层留 25 号（s138 挂账 ①②）。
- **零回归底线**：31 号 decode 面 360,431 逐 subtest 对照；M1 起每轮
  shim 变更 A/B 带 25 号 form-submission 基线与 20 号 url 基线；runner
  通道增量（M0）已带 decode 烟测。

## 下一步（M1 检查单）

1. shim：`_zwFormSubmitNavigate` 增「select a form submission encoding」步
   （accept-charset 解析 + document encoding fallback，缺口 1 设计要点）；
   UTF-8 路径零变化。
2. host 桥：`js_dom_bridge.rs` 增 encode 方向（encoding_rs encoder +
   percent-encode set + `&#NNNN;` 回退 + lone surrogate），沿
   `__zw_text_decoder_*` 先例。
3. 判据：urlencoded2 s138 挂账④ 4 案（windows-1252 前哨）P；encode-form
   面 Pass 大幅收敛（预期 euc-jp/big5/euc-kr 全族近 100%——encoding_rs
   编码表与 decode 面同表反向）；31 号 decode 面 + 25 号 form-submission +
   20 号 url 零回归对账。
4. 通道基线更新落 evidence，25 号/31 号/20 号对账数字同轮记账。
