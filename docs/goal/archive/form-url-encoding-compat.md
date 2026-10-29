# Form/URL 遗留编码序列化 — form-url-encoding-compat

> **立项（2026-10-29，用户决策）**：31 号 [encoding-legacy-mb](./encoding-legacy-mb.md)
> 把 decode 面 360,431 子测试收到 100% 后收口，其 M3 甄别把 encode 面 68 文件
> （form 提交编码 + href URL 序列化）归口为「runner form/URL-compat 能力缺口」。
> 本 goal 承接该归口。这是全量 WPT 通过率到 50%+ 的唯一未立项杠杆：
> 当前头条 24.19%，encode 面 767,673 子测试 = +33.9pp 空间。
> **31 号 decode 面基线 360,431 是零回归底线**（分母逐 subtest 对照）。

**版本**: v1.1
**日期**: 2026-10-29（立项）；2026-10-29（收口）
**状态**: Done——M0~M3 完成（2026-10-29 收口归档）。encode 面 68 文件
766,959 子测试 766,958P/0F/1T（99.9999%），唯一非 P = data: iframe runner
Timeout 契约挂案（重入条件见 [master.md](form-url-encoding-compat/master.md)
缺口清单 2）；coverage 复测全量头条 24.19% → 58.02%。
**执行模式**: WPT 验收（上游 encoding/legacy-mb encode 面 corpus）+ 能力建设
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——文档编码面）

## 基线事实（立项时实测与甄别结论）

- **分母**：encode 面 68 文件，DC-3 分母口径 767,673 子测试
  （coverage-2026-10-29 未覆盖余量口径 767,316，差 357 为两账口径差，
  M0 钉死单一口径）。构成：form 46（japanese 23 / korean 12 / tchinese 11）+
  href 22（japanese 12 / korean 3 / tchinese 4 + schinese `*-encoder` 2 +
  `big5-enc-ascii` 1）。per-face 子测试 split 待 M0 实测（按文件数比例估算
  form ≈ 2/3、href ≈ 1/3，不作为承诺数字）。
  甄别依据：31 号
  [M3 可跑性甄别](./encoding-legacy-mb/evidence/2026-10-28-m3-encode-form-href-runnability.md)。
- **语料零依赖**：68 文件全部本地在位（31 号 DC-3 对账上游 pin 315976933
  missing=0），`/common/blank.html` 在位——本 goal 无 fetch 依赖，M0 直接进
  通道与能力切片。
- **依赖现状（25 号流 s138 落地后，2026-10-29）**：31 号 M3 甄别的三层缺面中，
  「缺面 2：form.target → 嵌套导航」**已由 html-semantics-compat M2-S138 落地**
  （`_zwFormSubmitNavigate` ⑤b target 分支：formtarget > target 属性 → named
  iframe，GET query / POST body 经 `__zw_reload_iframe`；`_zwLoadIframeEntry`
  method/body/headers 透传；iframe load 事件管线；echo-content-escaped.py
  端点）。POST→iframe→contentDocument 断言链已被 urlencoded2 36 F→P 实证
  打通。剩余两层：
  1. **缺面 1（核心被测语义，form 面）**：`_encPair` 序列化硬编码 UTF-8
     （s138 已对齐 spec percent-encode set，编码仍 UTF-8），acceptCharset
     属性零消费。被测语义 = entry list 按 acceptCharset（回落文档编码）
     出非 UTF-8 字节序列（euc-jp `%A4%A2` 形态）。
  2. **缺面 3（href 面）**：「encoding-parse a URL」无消费点——shim URL/
     URL 组件与 Rust 侧 rust-url 全 UTF-8。被测语义 = href setter 以文档
     编码为 fallback encoding 解析，query 按文档编码出字节。
- **规模面**：每文件 ≈ 全 ranges 码点数个 async_test（euc-jp ~7,000），
  400 码点/chunk 双 frame 轮转——通道 envelope 按 31 号 M1 先例（本通道
  `ZW_SCRIPT_TIMEOUT_MS` 提额）预评估。
- **通过率预期依据**：同一 encoding_rs 解码表在 30 号 JS API 面（99.96%）
  与 31 号 decode 面（100%）均收敛，序列化是同表反向；风险集中在断言链
  基建（iframe.onload → `contentWindow.location.search`）与 href 面的
  URL 记录改造，不在线表本身。

## Mission

runner/engine 补齐两块能力，使 legacy-mb encode 面 68 文件可跑并收敛：

1. **form 提交编码序列化**：`_zwFormSubmitNavigate` 序列化路径消费
   acceptCharset（合法 label → 对应编码；非法/缺省回落文档编码），entry
   list 值经 encoding_rs 出目标编码字节序列，GET query 按 spec
   [urlencoded serializer](https://url.spec.whatwg.org/#concept-urlencoded-serializing)
   的 encoding override percent-encode。
2. **encoding-parse a URL**：URL 相关 setter（`a.href` 入口）以文档编码为
   fallback encoding 解析，query 组件按文档编码序列化
   （[spec](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#encoding-parse-a-url)）。

## Support Envelope

**范围内**：

- `encoding/legacy-mb-*` encode 面 68 文件（form 46 + href 22）。
- form 面：acceptCharset 解析与验证、序列化编码 override、`&#NNNN;` 形态
  无法映射字符回退（urlencoded serializer 的 encoder 状态语义）、lone
  surrogate USV 转换步。
- href 面：文档编码 fallback parsing 的最小消费面（以 19+3 文件断言链
  跑通为准，不做通用 URL 记录改造设计）。
- runner 通道：per-dir skip 规则改判（encode 面 skip → 分面启用）+
  Makefile target（test-guard 包裹，沿 31 号先例）。

**范围外（划界）**：

- 单字节遗留编码（windows-125x/iso-8859-x）文档与 encode 面不拉语料、
  不立项；能力落地后作为顺带受益面记账（沿 31 号先例）。
- text/plain、multipart enctype 编码器与 File entry 层归 25 号
  html-semantics-compat（s138 挂账 ①②，非编码 override 语义）。
- url 解析主干（20 号 net-api-compat 二期已收 url 94.1%）不动；本 goal
  只加「文档编码 override」消费点。
- `.py` 服务器端点不拉（lib.sh 惯例）；`.headers` sidecar 维持不消费
  （31 号 DC-3 实证 98/98 hdr==meta）。

## Done Criteria

- **DC-1（M0 面单 + 最小切片）**：68 文件按面分簇 + per-face 子测试分母
  实测落 `docs/goal/form-url-encoding-compat/evidence/`，钉死与 DC-3 /
  coverage 两账的口径差；form 最小验证集（euc-jp 1 案）
  iframe.onload → `contentWindow.location.search` 断言链端到端跑通。
- **DC-2（form 面能力）**：acceptCharset 序列化落地（编码 override +
  `&#NNNN;` 回退 + USV 转换），form 面 skip 解除、通道基线（文本 + JSON）
  落 evidence；25 号流 form-submission 基线与 31 号 decode 面零回归。
- **DC-3（href 面能力）**：encoding-parse 消费点落地（或甄别不可行按契约
  挂账重估），href 面 skip 解除、基线落 evidence。
- **DC-4（coverage 复测）**：coverage 对账 encode 面清零路径成立
  （not-run 余量转已覆盖、通过率如实单列）；全量头条增量入账。

## 里程碑

- **M0**：面清单 + per-face 分母实测 + form 最小切片（DC-1）。
- **M1**：runner 通道 + form 面能力（DC-2）。
- **M2**：href 面 encoding-parse（DC-3）。
- **M3**：基线收口 + coverage 复测（DC-4）。

## 依赖约束与碰撞管理（run-rules §9）

- **25 号 html-semantics-compat（活跃 M2 流）**：`_zwFormSubmitNavigate`
  是共享路径。s138 挂账 ④（windows-1252 accept-charset + `&#NNNN;` 实体
  回退 + lone surrogate USV，urlencoded2 4 案）归口本 goal M1，25 号流
  不重复施工；本 goal 每轮变更 A/B 对账带 25 号 form-submission 基线。
- **20 号 net-api-compat**：url 域基线（94.1%）零回归对账每轮带；URL 面
  变化核对 `url` crate 与引擎共享路径边界。
- **31 号 encoding-legacy-mb**：decode 面 360,431 分母逐 subtest 对照，
  零回归底线；本 goal 不动 `decode_document_bytes` 管道。
- 引擎/shim 变化沿 25 号先例跑六通道 A/B 集合恒等对账。

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足，目标能力达到 production-ready 水平 | `DONE` | 见「DONE 允许条件」 |
| 进展仍可推进，还有未完成的工作 | `CONTINUE: <下一步>` | **默认输出** |
| 遇到真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |
| verify 发现未满足条件但进展仍可推进 | `CONTINUE: <下一步>` | 返回执行，不是 DONE |

### DONE 允许条件

同时满足：DC-1~4 全部满足有据；验证基于上游真实 WPT 用例（无内建 inline
充数）；`cargo build` + `cargo test` + `cargo clippy` 全过；master.md 内部
自洽、archive 建立。

### 禁止输出 DONE 的情况

- 仅通过内建 inline 用例，未用上游真实语料。
- 分母为子集非上游全量（68 文件口径不齐不算）。
- 通过率含同源假通过而未做非平凡性检查。
- 无实际代码/测试进度（仅有文档和计划）。
- encode 面通过率与 decode/JS-API 面基线存在未对账的回归。

## Document Control / Archive Policy

- **入口文档**（本文件）：Mission、Done Criteria、划界与协议。仅目标实质
  变化时修改；每轮进度不重写。
- **运行时控制平面** `docs/goal/form-url-encoding-compat/master.md`：唯一
  真实状态面板（里程碑表、缺口清单、碰撞记录、验证基线）。持续演进、
  各章节自洽、不允许无限增长。
- **归档区域** `docs/goal/form-url-encoding-compat/archive/`：里程碑级
  过程与历史证据，只追加不修改。
- **证据区域** `docs/goal/form-url-encoding-compat/evidence/`：通过率
  报告、失败分析、A/B 对账，持续追加。
