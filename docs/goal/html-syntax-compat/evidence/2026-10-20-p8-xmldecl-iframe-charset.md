# P8: xmldecl 收编——iframe 子文档编码嗅探 + bogo-XML declaration

日期：2026-10-20 ｜ 前序：[P7 XHR document 编码嗅探](2026-10-20-xhr-responsexml-charset.md)

## 结论

`html/syntax/xmldecl` **88/88 全绿**（xmldecl-1/2/3 三 runner 页，88 断言子测；
xmldecl-1 全 40 Pass）。全通道回归见下文验证节。

四层能力落地：

1. **WHATWG prescan 算法**（`_zwFramePrescanCharset`，part01）：1024 字节窗
   meta 归纳 + 「get an XML encoding」兜底（bogo-XML declaration）。字节面
   （`__zw_bytes:` wire）与字符串面（合法 UTF-8 plain wire）双视图同一扫描器。
2. **iframe 子文档解码链**（`_zwFinishIframeEntry`）：BOM → transport charset
   （Content-Type 参数，confidence certain）→ prescan → 非 UTF-8 重解码；
   replacement 标签保留（TextDecoder 无 replacement decoder，解码降级
   windows-1252）。
3. **HTML 解析语义按 Content-Type 判定**：URL 扩展启发之外的动态端点
   （`trickle.py` 等 wptserve text/html 输出）按响应 MIME 走 HTML 文档语义。
4. **runner 通道**：`trickle.py` CGI 静态等价（wptserve trickle 语义无流式
   复现，输出逐字一致）；`.headers` sidecar Content-Type 覆盖（替换式）；
   `.htm` MIME 映射；`lib.sh` fetch_dir_html 增 `*.htm`/`*.headers`。

## 语义定谳

上游 xmldecl 三页验证「bogo-XML declaration 参与导航编码嗅探」（WHATWG
HTML 解析 §4.2.5.5 scripting 面之外的 encoding sniffing：prescan 与
get-an-XML-encoding）。fixture 全 88 断言面与 spec 算法逐案推演吻合：

- meta 命中即返回（xml-and-meta 案 1253 胜出——prescan meta 先行）；
- get-an-XML-encoding 只在 prescan 1024 窗耗尽/EOF 中止后触发（trailing
  body 案走 XML 声明、kilobyte-before/after 案走 meta/缺省——窗口界语义）；
- `encoding` 子串须在首个 `>` 之前、值须成对引号且无 ≤0x20 字节
  （gt-between/encodingencoding/one-around-label 等案落 1252 wrapper 正是
  XML-encoding 失败路径）；
- utf-16 系 label 归一 UTF-8（processing 步）、x-user-defined 归一
  windows-1252、ISO-2022 系归一 replacement。

https://html.spec.whatwg.org/multipage/parsing.html#prescan-a-data-state-based-on-the-byte-stream

## 修改明细

| 位置 | 内容 |
| --- | --- |
| part01 `_zwFramePrescanCharset` | prescan 扫描器：get-an-attribute 闭包（共享 position 游标）、meta 属性归纳（charset 命中即返回；content/http-equiv 显式略——spec prescan 只认 charset）、letter-tag 分支、`<!`/`</`/`<?` 滑过、getXmlEncoding 兜底、UTF-16 XML 声明字节序判定；`bt` 兼容 Uint8Array/string 双视图 |
| part01 `_zwFinishIframeEntry` | transport 提取（Content-Type 大小写不敏感）；`__zw_bytes:` wire 字节链（BOM/transport/prescan/解码）；plain wire 嗅探（`﻿` BOM → transport → text prescan）；kind CT override（`/^text\/html\b/i`） |
| part05 `_zwMakeIframeDoc` | charsetLabel 第三参 → R141 characterSet 三别名；leading bogus comment（`<?` 起始 → COMMENT_NODE，`?xml` data 保真——`fc.nodeValue.startsWith("?xml")` 断言面） |
| testharness.rs | trickle.py 静态等价（`body_bytes` 含 0xE6 → R3021 bytes wire）；`wpt_sidecar_content_type`（.headers sidecar）；sidecar 消费替换式；`.htm` MIME |
| scripts/goals lib.sh + 40-html-syntax-compat.sh | fetch_dir_html 增 `*.htm`/`*.headers`；xmldecl/support 目录入 fetch 清单 |

## 过程要点

**共享 position 指针死循环（90s V8 watchdog 现形）**：spec prescan 只有一个
position 指针——tag skip 与 get-an-attribute 必须同游标。首版 letter-tag
分支 getAttr 从陈旧 `_paPos` 起扫、null 落点早于 `i2`、`i2 = _paPos` 回跳，
baseline.htm 首个 `<p>` 即死循环。修法：skip 落点回写 `_paPos = i2` 后才进
属性循环。教训已沉淀 [learnings](../../../learnings/INDEX.md)（见
prescan-position-cursor 条目）。

**探针方法论**：`createElement('iframe')` 在本 runner 是无条件 skip
（上游 fixture 全用解析式 iframe），动态注入探针不可用；改用页内静态
`<iframe src>` + onload 断言的 probe-*.html 矩阵（probe-a 控制组→probe-g
同目录组）完成挂点二分——`__zw_bytes:` wire 且子目录 src 才挂，定位到
游标回跳。探针为临时产物已删除，不入正式通道。

**plain wire 盲区**：首版只对 `__zw_bytes:` wire 嗅探——非 UTF-8 字节
（0xE6/0xC8 等）才走 bytes wire；replacement.htm 纯 ASCII → 合法 UTF-8 →
plain wire，嗅探整体旁路。修法：plain wire 同走 BOM 字符/transport/text
prescan（ASCII 语义 code unit 等价字节面）。

## 评估关闭（两项，不入实现）

1. **XHR XML MIME final encoding 接线**（P7 余账）：corpus 内 XML MIME
   响应 document 化零断言，无验证面；实现无从对账，关闭。
2. **html/syntax/the-input-byte-stream 9 案复制**：上游 pin 为自述式测试
   （无标准 testharness 断言结构，依赖 wptserve 管道运行时回填结果），
   本 runner 管道不运行该机制；复制需自行发明 oracle，与「上游可复核」
   原则冲突，关闭。

## 验证

- `FILTER=xmldecl`：**88 Pass / 0 Fail**（2026-10-20，guard 600s 内 exit 0）。
- 全通道回归：见 master.md 当日账（本切片零回归为提交门禁前提）。
- 门禁：cargo fmt --check / clippy -p zero-wpt-runner 干净；shim 拼接
  `node --check` 通过；`git diff --check` 干净。

## 余账

- textharness R360 头比较对 `X-Zero-Final-URL` 仍精确大小写（未触及，
  runner 推送侧同形，无已知断言面）。
- UTF-16 面编码案（utf-16le-and-meta 等）已过，但 TextDecoder shim 对
  UTF-16 label 直建（不经 BOM 判定）的行为未单测覆盖——非本切片断言面。
