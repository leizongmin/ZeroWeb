# M5-S12 — XML 文档解析（responseXML）+ template-element 三腿销账（2026-10-14）

**背景**：master.md 下一步计划第 3 项「goal 内结构性挂账余量」——XML 文档解析
（template-element 3 腿，xhr 域唯一 goal 内可实施深水切片）。spec 面：xhr.spec
set a document response——XML essence 经 XML parser；responseXML getter 于
responseType ''/document 运行。

## 结果（六 corpus 受控 A/B，对照 M5-S11 树）

| corpus | 基线（M5-S11 树） | M5-S12 | Δ |
|---|---|---|---|
| fetch | 1504P/583F/2T | 1504P/583F/2T | 逐字节持平 |
| xhr | 575P/18F/1T | **578P/15F/1T** | **+3P/−3F** |
| url | 4639P/1426F | 4639P/1426F | 逐字节持平 |
| mimesniff | 1033P/865F | 1033P/865F | 逐字节持平 |
| streams | 905P/108F/5T | 905P/108F/5T | 逐字节持平（5T 先在） |
| eventsource | 35P | 35P | 逐字节持平 |
| websockets | 376P | 376P | 逐字节持平 |

xhr +3 腿（Fail → Pass，唯一变化页 template-element.html）：

1. template element parsing（XHTML ns template——childElementCount 0 + content
   fragment firstChild 'test'）
2. template element parsing 1（no-ns——childElementCount 1 + firstChild 'test'）
3. template element parsing 2（SVG ns——同 2）

回归防线实证：responsexml-invalid-type（rt document + text/plain → response/
responseXML null）与 overridemimetype-done-state（text/plain → null）定向复跑绿。

## 实施

### ① 最小 XML parser（part02.js `_zwXmlParseDoc`，置 DOMParser 区）

手写 tokenizer（~190 行）：prolog 跳过（`<?xml?>`/PI/`<!DOCTYPE>`/注释）+ 元素/
属性（双单引号形态）/自闭合/文本（五具名实体 + 十进制/十六进制数字实体）/CDATA
（不实体解码）+ **默认 xmlns 命名空间栈继承**（元素 ns = 自身 xmlns 或最近祖先）。
HTML spec：template 仅 HTML ns 有 content DocumentFragment（子元素移入，
`childElementCount` 0）；no-ns/SVG ns template 为普通元素。产物 `_zwXmlNode` 树：
firstChild/lastChild/childElementCount/textContent/accessor + getAttribute +
template `content`。容忍解析——well-formed 错误检查 / parsererror 文档 / 前缀
xmlns 绑定面挂账（语料无此形态）。

### ② fill 接线（part03.js）

- **rt 'document' 分支**：XML essence（`_zwXhrIsXmlMime`——text/xml、
  application/xml、+xml 后缀；application/xhtml+xml 由既有 HTML 分支先行消费）
  → `_zwXmlParseDoc` → response/responseXML 同置。**final MIME 无 CT 时不再回落
  'text/xml'**（spec 无 MIME → null——responsexml-invalid-type top.txt 面回归防线；
  旧回落值经 HTML 分支条件实际不可达，行为不变）。
- **rt '' 分支**（responseXML getter 面）：XML essence（override/CT 显式给出，
  无 CT 不猜）→ responseXML = XML doc，response 仍文本。

### ③ 探测记录

template-element 三腿分别要求 XHTML ns content 移入 / no-ns 普通 / SVG ns 普通——
HTML 解析管线（`__zw_parse_html_query`）对 xmlns 不敏感（HTML parser 忽略 xmlns，
`<test/>` 落 HTML ns）且 template content 语义不分 ns，无法承载 XML ns 判别——
独立 XML parse 域成立（非复用 HTML 管线）。

## 碰头核对（run-rules §9）

本切片改动面 = part02.js（XML parser，新增段置于 DOMParser 区）+ part03.js（fill
接线）——共享面 last-touch = 本流自身（4ac143f88 M5-S11），无并行编辑。域外挡门
延续 M5-S10/M5-S11 记录（渲染流 runtime.rs/postprocess.rs fmt+clippy，未处置）。

## 门禁

- `make test` 全绿 **20664P/0F**（与 M5-S11 持平——responseXML null 语义无存量断言
  翻新）。
- `cargo clippy -p zero-wpt-runner --all-targets -- -D warnings` 干净；`cargo fmt`
  干净（本切片纯 JS 变更，engine Rust 面零涉）。
- 六 corpus 受控 A/B 同树 stash 对照双跑（上表）。

## 残余（本切片后）

- xhr 域账面 15F = formdata 5（M5-S8 记账）+ overridemimetype shift-jis 8
  （encoding 域）+ permissions-policy 2（跨域）；cookies 页级 Timeout 1（跨域）。
- **goal 内可实施面就此全部清空**——余量全部为跨域记账（storage/security-hardening/
  encoding 邻域）或引擎能力深水（streams transfer/detach 需宿主 V8 detach）。
  XML parser 扩展面（well-formed 检查/parsererror/前缀绑定）挂账：语料无此形态，
  非本 goal 断言面。
