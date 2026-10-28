# M3：form-encode / encode-href 面可跑性甄别

日期：2026-10-28。划界来源：goal「划界」节——form 提交编码面（`*-encode-form-*.html`）
与 `<a href>` URL 序列化面（`*-encode-href-*.html`）不属本 goal（document 解码面），
M3 一并甄别可跑性、不强收。本文件即该甄别的结论与归口。

## 语料盘点（本地 136 html 口径）

| 面 | 文件数 | 分布 |
|---|---:|---|
| encode-form | 46 | japanese 23 / korean 12 / tchinese 11（schinese 未拉到） |
| encode-href | 19 | japanese 12 / korean 3 / tchinese 4 |
| 合计 | 65 | 占本地语料 136 html 的 48%——通道 skip 规则排除的主体 |

共享驱动：`encoding/resources/encode-form-common.js`（140 行）、
`encode-href-common.js`（57 行）。

## encode-form：不可跑（三层依赖缺面）

机制：setup 建 2 iframe + 2 form（`method=GET`、`action="/common/blank.html"`、
`acceptCharset=<meta charset>`、`target=<iframe name>`），每 chunk 400 码点灌入
input 后 `form.submit()`；断言在 `iframe.onload` 里读
`contentWindow.location.search` 拆分比对。被测语义 = spec form submission 的
entry list 序列化按 acceptCharset（回落文档编码）出**非 UTF-8 字节序列**
（euc-jp `%A4%A2` 形态）。

三层缺面（按断言链依赖序）：

1. **序列化编码缺面（核心被测语义）**：引擎 `_zwFormSubmitNavigate` 的
   `_encPair` 硬编码 `encodeURIComponent`（UTF-8 percent-encoding，
   `part04.js:11184`）；acceptCharset 属性全 shim 零消费。UTF-8 序列化下
   全部断言不可达。属 form 提交序列化面（非 document 解码面）。
2. **target=iframe 导航缺面**：`_zwFormSubmitNavigate` 默认分支只做顶层
   `__zw_request_navigate(_abs)`，无 form.target → 嵌套上下文（iframe name）
   解析；runner 单文档 harness 无子框架导航服务。`iframe.onload` →
   `contentWindow.location.search` 断言链不可达。`/common/blank.html` 本地
   文件存在（`wpt-data/common/blank.html`）且 testharness base（`http://wpt.test`）
   可解析，但导航管道不达子框架——卡的不是文件，是嵌套导航。
3. **规模面**：每文件 ≈ 全 ranges 码点数个 async_test（euc-jp ~7,000），
   400/chunk 双 frame 轮转；即便 1+2 就绪也需通道 envelope 评估。

**结论：不可跑。** 归口 = form 提交编码面（acceptCharset 序列化 + 嵌套
导航），记账为 runner form-compat 能力缺口；两项齐备前该面无判据意义，
划界「不强收」维持。

## encode-href：可跑但面缺（不入基线）

机制：同步面——`a.href = "https://example.com/?" + <码点>`，读 `a.search`
比对期望字节序列。无服务器/iframe/导航依赖，runner 通道跑得起来。

被测语义 = HTML spec「encoding-parse a URL」：href setter 以**文档编码**为
fallback encoding 解析 URL，query 组件按文档编码出字节（legacy 面，现代
spec 仍保留）。引擎实况：URL 面全 UTF-8——shim `new URL`/
`encodeURIComponent`（part02 URLSearchParams/URL 组件，5846 起按绝对 URL
重解析）、Rust 侧 `url::Url`（rust-url，UTF-8 percent-encoding）；全仓无
「文档编码 override」消费点。实跑预期近全量 Fail（期望 `%A4%A2`、实际
`%E3%81%82`）。

**结论：可跑但无判据意义**——缺的是 URL/HTML 集成面（encoding-parse），
非本 goal 管道。通道 skip 规则维持，理由由「未甄别」升格为「已甄别——面缺，
跑了只产红噪声掩盖解码面收敛信号」。归口 = runner URL-compat 能力缺口。

## 对账口径影响

- 通道 skip 规则不变；DC-3 全量基线（待语料补全）分母口径：encode 两面按
  skip 记 not-run 并单列（本盘点：form 46 + href 19，schinese 族语料齐后
  另计），与 decode 面（wrapper + errors）分开——「已解锁面通过率」不被
  「未解锁面」稀释。
- 若未来立项 form 提交面 / URL 面 goal，本文件即两面的依赖清单一（缺面 1/2
  与 encoding-parse）。

## 更正（2026-10-29）

语料盘点表按文件名模式（`*encode-form*`/`*encode-href*`）统计漏掉 3 个
同面文件：schinese `gb18030-encoder.html`/`gbk-encoder.html`（inline
a.href 面，本地在位）与 tchinese `big5-enc-ascii.html`。encode 面合计
**68** 文件（非 65），schinese 1,017 分母即出自 2 个 `*-encoder.html`。
结论不变：同属 href 序列化面（encoding-parse a URL），甄别归口维持。
详见 [2026-10-29-corpus-completeness-dc3.md](2026-10-29-corpus-completeness-dc3.md)。
