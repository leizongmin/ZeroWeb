# DC-2 · P1 queryCommand 状态语义读面修复（2026-11-11）

## 修复面

六 inline format 命令（bold/italic/underline/strikethrough/subscript/
superscript）的 `queryCommandState`/`queryCommandIndeterm` 读面按 spec
state 语义重写：

- `part01c.js` 新增 `__zw_ce_format_query(cmd)`：effective command value
  （computed style 判定：font-weight/font-style 解析值、text-decoration
  逐元素 own 值走查、sub/sup inline 祖先链计数）+ 含域 formattable 全命中
  state + part-not-all Indeterm；activated values 按 Chrome 口径
  （bold 含 600 cutoff）。复用值面 `_zwValGetAllEffectivelyContained`。
- `part06.js` 三臂接线：`queryCommandIndeterm`/`queryCommandState` format
  命令改走本面（typing-state 兜底保留读序在前）；exec toggle 臂保持
  `_zwQueryFormatStateDom` 旧读不动（apply/unwrap 机制面）。

## 方法与数字

同树前后二进制对跑（test-guard 包裹，`ZW_SCRIPT_TIMEOUT_MS=300000`），
面 = 六 format 命令文件 + 4 个 state 探针（qsq/qv5/vsel/tstyle），
共 19 案 11,965 子测试。产物 `before/after` JSON 在本轮 transcript，数字：

| 文件 | before P/F | after P/F | ΔP |
|---|---|---|---:|
| bold | 2669/379 | 2673/375 | +4 |
| italic | 1807/266 | 1845/228 | +38 |
| underline | 1871/276 | 1894/253 | +23 |
| strikethrough | 1872/275 | 1895/252 | +23 |
| subscript | 1067/174 | 1080/161 | +13 |
| superscript | 1099/174 | 1112/161 | +13 |
| 合计 | 10420/1545 | 10534/1431 | **+114** |

- before 基线可信：italic 1807/2073=87.17%、underline 1871/2147=87.12%，
  与 [dc1-run-shards](2026-11-11-dc1-run-shards.md) 本地核对数一致。
- 无新失败族：innerHTML 族两轮恒 727（apply 面零扰动）；7 例
  `expected false but got true`（execCommand return value，
  contenteditable=false 门）与 zz-probe-tstyle F=1 两轮同在，均既有。
- 见证探针 [zz-probe-fmtstate.html](../../../tests/wpt-runner/wpt-data/editing/run/zz-probe-fmtstate.html)
  D1–D7（style span computed style / UA 元素表 / part-not-all indeterm /
  600 cutoff / sub 嵌套）：修复二进制 7/7 Pass，HEAD 基线二进制 7/7 Fail。

## 归因修正：残余 696 失败属 apply 面非读面

triage 的「~800 共享路径 = state 读语义」需拆分：读面可修的部分即上表
+114；残余 `Wrong result returned` 696（state 624 + indeterm 72）经诊断
探针实测为 **execCommand apply 缺口的下游读数**——多测期望按 spec apply
后的 DOM 录制，我们的 apply 未产生该 DOM，读面（现已 spec 正确）如实报
实际 DOM 的状态：

- 跨段落 `<p>[foo</p> <p>bar]</p>` exec bold：rc=true 但 DOM 零变更
  （跨段落 wrap 臂缺失）→ state 读 false（对实际 DOM 正确），期望 true；
- `foobar<b>bazqoz</b>quz` 选 bar+baz exec bold：DOM 零变更（跨边界
  merge 臂缺失）→ indeterm 读 true（对实际 DOM 正确），期望 false。

triage「er8/er9 纯 state 失败与 innerHTML 零共现 → DOM 已对」的推断
不成立——multitest 本不断言 innerHTML，零共现是断言面缺口不是 DOM 证据。
残余面归入 P2 apply 工作（跨段落 wrap / 跨边界 merge，与 innerHTML 长尾
的 bold 转换输出 197 同根）。

## 范围口径

本轮本地面为六命令文件 + 探针；multitest.html/misc.html 等 state 断言
案的 corpus 全量增量以下周 CI weekly 为准。
