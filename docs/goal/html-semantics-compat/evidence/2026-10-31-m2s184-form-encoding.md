# M2-S184 form 提交编码 text/plain + multipart 臂落地——12F 双根刀（accept-charset 消费 + text/plain iframe 字面渲染门），shim 2 + rust 2

## 入场定刀位

s183 账本「下一步候选」首位：form 编码替换 12（s181 分型定谳——
form-submission-0 text-plain.window.js 6 + multipart-formdata.window.js
6，全部 windows-1252 提交编码面）。

## 勘根（两层根因）

1. **根因一（提交编码未消费）**：enctypes-helper
   formSubmissionTemplate 造 form（acceptCharset = formEncoding，
   POST → /FileAPI/file/resources/echo-content-escaped.py，iframe
   target，contentDocument.body.textContent 经 helper unescape
   断言）。shim part04 text/plain 串位组装与 multipart
   `_mpPushStr` 均走 `_zw_utf8_encode` 直通——accept-charset 选出的
   提交编码（HTML §4.10.22.4/§4.10.22.5 encoding algorithm 消费
   「select a form submission encoding」产物）从不参与串位字节化。
   期望行为：mappable á → 0xE1；unmappable ə/U+FFFD/💩 → HTML NCR
   `&#601;`/`&#65533;`/`&#128169;` ASCII 字节（encoding_rs HTML
   error mode 同型）。
2. **根因二（text/plain iframe 字面渲染门缺位）**：首刀接线后
   targeted 仍 12F 但 got 由 mojibake `aÉb=cï¿½d\r\n` 变 charref
   解码态 `aəb=c�d\r\n`——**字节已是正确 windows-1252+NCR
   面**，错在回显消费端：part01 `_zwFinishIframeEntry` 只对
   `text/html` Content-Type 升 HTML 解析（URL kind 兜底 .py → xml
   落标记解析），回显体里的 `&#601;` 被解码成真实字符；真浏览器
   text/plain 文档走 spec navigate read text（body > pre、解码文本
   字面进树、不解析标记）。
3. **勘根方法学坑（两记）**：runner `html_semantics_case_skipped`
   静默跳过含 `createElement("iframe")`/`<iframe` 字面量的 case 源
   （cases 空 → exit 1 零输出）——探针内联 iframe 造帧须拆串
   （`"ifra" + "me"`）；helper `expectedBuilder` 仅收单参
   `serialized`（`expectedBuilder.bind(null, expected)` 预绑
   expected），探针直读 textContent 不经 helper unescape。

## 施工（shim 2 + rust 2）

- **text_encoding.rs**：`text_encode_form_bytes(label, text)`——
  `Encoding::for_label` 缺席返空串（shim 归一后方调用）；
  replacement/UTF-16 归 UTF-8；UTF-8 臂 `as_bytes` 直通；非 UTF-8
  逐 scalar fresh 编码（ISO-2022-JP fresh-scalar ESC `( B 伪迹剥除，
  text_encode_percent 同款模型），字节 csv 十进制返（crypto 模块
  wire 对称）。单测 `test_encode_form_bytes_windows_1252_ncr` 钉
  á→"225"、€→"128"、`aəb`→NCR 字节、U+FFFD→`&#65533;` 八字节、
  💩→`&#128169;`、ASCII 两编码同 csv、未知 label 空串。
- **callbacks.rs**：注册 `__zw_text_encode_form_bytes`（label +
  text 双参）。
- **part04.js**：text/plain 分支——body 组装后
  `_fSubmitEnc !== 'UTF-8' && typeof __zw_text_encode_form_bytes ===
  'function'` 门内 `_zwEncodeBytesPrefix(_encFormBytes(_body))` 字节
  wire（`__zw_bytes:` csv，host fetch_bridge 解码 body_bytes——
  multipart R3020 同通道）；multipart `_mpPushStr` 同门编码感知
  （name/filename/非 File value；File 字节体与 boundary/头名
  ASCII 不经桥）。
- **part01.js**：`_zwFinishIframeEntry` text/plain 门——
  `_zwIsTextPlain` 时 kind 升 'html' + body 包 HTML 转义 `<pre>`
  壳（`&`/`<`/`>` 转义后解析回头即字面文本；`\r\n` 经解析器输入流
  归一 `\n` 与真浏览器同型——helper unescape 首步 `\r\n?|\n →
  \r\n` 归一即为此设；NUL 交 tokenizer FFFD 替换）+
  `entry.doc.contentType = 'text/plain'` 回写。spec HTML
  document-lifecycle #read-text。

## 验证

- **勘根探针**（zz-s184-rt.window.js，收口前已删）：form 同构
  POST → iframe textContent 逐 charCode 倾倒——首刀后
  `a&#601;b=c&#65533;d\n` 字面（38,35,54,48,49,59…）钉死根因二；
  text/plain 门后同形稳定。
- **定向 form-submission-0：299P/3F**（12 靶案全 F→P；残留 3F =
  submission-checks.window.js `<form> cannot navigate` 三兄弟，
  连接性独立面 s183 前在册，非本刀面）。
- **forms 全树 join 4711/4711 键 F2P = 12 恰靶案（6+6
  text-plain/multipart）、其他位移 0**（4325P/375F/12T →
  4337P/363F/12T）。
- **interactive-elements 483 键零位移**（439P/44F bit-exact）。
- **engine lib 3636P/1F**（+1 新单测；1F = ua_display_li
  test_ua_display_li_list_item_canvas_inline_s24_flip 渲染流在册
  同键，s183 同报）。
- **shim node --check SYNTAX_OK**（114133 行）；cargo fmt/clippy
  -p zero-engine 全绿。

## 复验（2026-10-31 会话中断恢复后，rebase 并入 origin 7 提交——site-opt
## caret/paint + R5340 removeChild memo + R5341 runner harness 运行期读取——全数重跑）

- 定向 299P/3F（3F 同上三兄弟）、forms 全树 **4337P/363F/12T** 逐位
  一致、text-plain/multipart 两靶文件 62P/0F 全绿、interactive
  **439P/44F** bit-exact；engine lib **3641P/1F**（渲染流并入 +5 新
  测试，1F 同上在册键）；shim node --check SYNTAX_OK（114155 行，
  +22 = R5340 part04 行数）；cargo fmt --all --check 过 + guarded
  workspace clippy 零告警。

## 残留与边界

- submission-checks 三兄弟（连接性面）在册另刀。
- form 编码 utf-16 面：shim `_fSubmitEnc` 归一 utf-16→UTF-8
  （spec utf-16 提交编码即 UTF-8 同义面），语料无对钉案。
- enctypes-helper unescape `\r\n?|\n` 归一只还原解析器输入流
  归一；`\xNN` 还原对 `\` 先行 `\x5C` 转义防误吞——runner echo
  端点与上游 wptserve echo-content-escaped.py 语义同构（既有）。

## 下一步候选

form.elements 集合层 12（HTMLFormControlsCollection/RadioNodeList）；
disabled click 8 + click 消费激活 7（同 click 分派域可合刀）；
color-attributes input.alpha/colorSpace 2F 小反射面可搭车。
