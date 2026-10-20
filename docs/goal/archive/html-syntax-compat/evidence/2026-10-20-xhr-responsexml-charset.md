# XHR responseXML document charset 嗅探——html/syntax/charset 22F 清零（2026-10-20）

## 结果

`html/syntax/charset/xhr.html` **22/22 全绿**（本轮前 22F 全红，control plane 挂账
「responseType=document + characterSet 断言面」清零）。FILTER=xhr 单独验证
（guard 包裹，`make testharness-html-syntax FILTER=xhr`）。

## 归因

XHR「set a document response」须给 document 定编码：

- final encoding = override charset（overrideMimeType）→ 否则响应 MIME charset
  参数 → 均无则 prescan → 仍无则 UTF-8；
  https://xhr.spec.whatwg.org/#text-response（final charset 步骤）与
  https://fetch.spec.whatwg.org/#concept-request-response-tainting 无关——
  编码决定在 XHR 层。
- prescan/嗅探语义**不是** HTML spec 的 prescan（纯字节级）——shim 无字节级
  tokenizer，且 ncr 案（`&#119;` 于 windows-1251 文档）证伪纯字节 prescan。
  定谳 = WebKit/Blink 共享的 `HTMLMetaCharsetParser` 语义（真实 tokenizer
  状态机 + head 区段追踪 + 1024 字节停扫），22 案几何全部吻合：
  https://github.com/WebKit/WebKit/blob/main/Source/WebCore/html/HTMLMetaCharsetParser.cpp
  （bytesToCheckUnconditionally=1024；HTML spec prescan 对照
  https://html.spec.whatwg.org/multipage/parsing.html#prescan-a-byte-stream-to-determine-its-encoding ）

第二层根因（runner 侧）：静态文件服务对 `.html` 做
`from_utf8_lossy → inline_local_scripts → into_bytes()`，裸字节 0xE6 在
JS 解码前已被改写为 EF BF BD（probe 实证：`doc=[...43f,457,405]`=
windows-1251 解码 EF BF BD；`dec(E6,B6)=[436]` 证 host TextDecoder 本身正确）。

## 修复

- **part03.js**：`_zwXhrScanMetaCharset(bytes)`——tokenizer 感知扫描
  （plaintext/script/RCDATA/RAWTEXT 状态块内容不产 token；`<!--`/bogus
  comment 吞掉；head 区段 permitted 集 {script,noscript,style,link,meta,
  object,title,base}，越界标签终结区段；token 起点未过 1024 则照常处理，
  与 WebKit `!inHeadSection && consumed >= 1024` 对齐）；meta charset/
  content+http-equiv 双模式（WebKit encodingFromMetaAttributes）；属性值
  字符引用解码（ncr 案）。document 分支接 BOM → override/MIME charset →
  meta 扫描 → TextDecoder 解码 + `_zwEncoding` 槽。
- **part02.js**：`_zwParsedDoc` 补 `characterSet/charset/inputEncoding`
  getter（`_zwEncoding || 'UTF-8'`，
  https://dom.spec.whatwg.org/#dom-document-characterset ）；`_zwParseEl`
  补 `rel` IDL 反射（link/a/area，
  https://html.spec.whatwg.org/multipage/links.html#dom-link-rel ——xhr.html
  以 `link.rel == 'match'` 过滤）。
- **testharness.rs**（fetch 通道静态服务）：`.html` 仅在字节为合法 UTF-8 时
  走内联路径（往返恒等）；非 UTF-8 文档跳过脚本内联、原字节直供
  `body_bytes`（该类语料为纯标记，无相对 src 脚本；FIXME 内联待字节级
  tokenizer）。`.sub.js` 替换、其余类型原路径不变。

## 22 案几何（WebKit 语义吻合点）

- after-1kb / after-head-after-1kb(+crlf)：head 区段无终结标签→扫全文件
  found；`</head>`@1016→1024 停扫 not found（token 端点判）。
- after-bogus-after-1kb：`<bogus>`@1017 七字节 token 恰跨 1024→停。
- after-head-in-1kb(+crlf)：meta@1023 起 token 未过界→照常处理→found。
- in-template-after-1kb / in-noscript-after-template-after-1kb：template
  非 permitted→区段终结→1024 前停。
- in-script/in-style/in-title/document-write：RAWTEXT/RCDATA 块内 meta
  不可见。
- in-svg：svg 非 permitted→终结；in-svg-in-cdata：CDATA 按 bogus comment
  吞掉。
- ncr：tokenizer 解 `&#119;`→windows-1251（证伪纯字节 prescan）。
- non-ascii-in-comment/title-before：1024 前无终结标签→继续扫→found。
- CRLF 案按原始字节偏移。

## 常驻测试

- WPT `html/syntax/charset/xhr.html` 22 子测（in-repo corpus，
  testharness-html-syntax 通道常驻，与修复同提交）。

## 门禁

- `cargo fmt --check` / `cargo clippy -p zero-wpt-runner --all-targets`：clean。
- `make test`：全腿绿，2 环境红均非本轮归因——SW owner 测
  （process_backend::service_worker_owner_tests）单测隔离复跑 **12P/0F**（全
  通道并发跑时 multiprocess 时序抖动）；bookmark_context_menu_open_in_new_tab
  R5174/R5176 在案（stash 归零态基线同红）。
- 全通道复跑（rebase cb8f59044 后组合态）：**61255/61333 = 99.89%**
  （61255P/65F/13T；改动前同基线账 61233P/87F/13T——+22P/−22F 恰为本轮
  22 案，零外溢）。同日三独立跑结果逐位全等（含并发双 rally 实例重入，
  无文件冲突——确定性佐证）。
