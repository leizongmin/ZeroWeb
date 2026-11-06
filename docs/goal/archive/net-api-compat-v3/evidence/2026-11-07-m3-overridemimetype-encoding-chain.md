# M3（六）：overridemimetype 族 + 响应解码链收口——6F/1T → 0，+21P 零回归（13,232 → 13,253P）

日期：2026-11-07。前置：M3（五）send-redirect-to-cors fixture 批（全量基线
**13,232P**/1,444F/16T，分母 14,692 行）。本批四刀：shim 两文件（part03.js
open() 去 override 重置 + `_zwXhrDecode` 落 get a final encoding 链；part02.js
`_zwXmlNode` 补 `tagName` getter）+ fixture 一处（testharness.rs status.py
content 字节保真）。在账标签修正：控制面「overrideMimeType Shift-JIS 1F」为
M0 时点口径——M1a `.htm`/`.window.js` 解锁后该族实际 **5F + 1T**（unsent/
headers-received shiftjis 2F + invalid-mime-type 2F + edge-cases 1F +
open-state-force-xml 1T），本轮全数清零。

## 一、归因方法（规范定谳 + 探针页）

xhr.spec 全文逐节核对（curl + 解析）+ 临时探针页（breadcrumb 经故意
assert_equals 失败透传，收口前已删）。

1. **open() 不清 override**——spec open() 步骤明注「Override MIME type is
   not overridden here as the overrideMimeType() method can be invoked
   before the open() method」。旧 shim 于 open() 重置 override MIME/charset：
   unsent 态 shiftjis 案（open 前设 override）open 后失效落 UTF-8 lossy
   （got `�e�X�g`）。edge-cases「not reset by open()」两案
   旧绿系 accident（重置后落 UTF-8 lossy 恰得期望的 U+FFFD 串）。
2. **get a final encoding 链**——spec：label 初值 = **response** CT 的
   charset 参（CT 提取失败按 text/xml essence，无 charset 即 null），override
   MIME 的 charset 参**覆写** label；spec 明注本步不用 get a final MIME type
   （override essence 不参与编码定夺——invalid-mime-type bogus → octet-stream
   无 charset → response charset 续用）。旧 shim 只认 override charset、完全
   无视 response charset（edge-cases text/xml 案 expected `Âð` got U+FFFD×2；
   invalid-mime-type expected `ÿ` got U+FFFD）。
3. **status.py content 字节保真**——fixture `wpt_query_value`（UTF-8 lossy）
   后再 percent-decode 的**双重解码**把非 UTF-8 载荷（`%83%65…`、`%FF`、
   `%FE%FF`）lossy 成 U+FFFD 字节。headers-received shiftjis 案 got
   `�ｽe�ｽX�ｽg` 即 Shift-JIS 解码「UTF-8 处理过的字节」的指纹
   （engine 侧 override 链该案本已生效），锚定 fixture 是根因而非 engine。
   修法：改走 `wpt_query_value_bytes`（逐字节 Latin-1 映射）+ char→u8 还原。
4. **open-state-force-xml 1T**——探针示 `responseXML` 已设、textContent 正常，
   `documentElement.tagName` undefined（`_zwXmlNode` 无 tagName getter）→
   断言抛异常 → done() 永不到 → harness 报 Timeout。补 DOM spec tagName
   getter（元素节点返回限定名，XML 解析保源 case）。

## 二、改动清单

1. part03.js `open()`：删除 override MIME/charset 重置（2 行 + spec 引注）。
2. part03.js `_zwXhrDecode()`：落 get a final encoding 链（response CT
   charset → override charset 覆写 → replacement 面 → BOM 嗅探 → 表外回落
   UTF-8）。
3. part02.js `_zwXmlNode.prototype` 补 `tagName` getter（nodeType 1 →
   nodeName；模板面/XML 树共用——responseXML 文档断言面）。
4. testharness.rs status.py：content 解码改 `wpt_query_value_bytes` +
   char→u8（字节保真，文本视图仍 `from_utf8_lossy`）。

## 三、定向与全量 A/B

- 定向：`make testharness-net-api FILTER=xhr/overridemimetype` **族内 13/13
  全绿**（blob 77P 与基线持平；FILTER=xhr/responsexml Fail/Timeout 19 行
  基线持平——该族为在账外存量缺口，本轮不动）。
- 全量：`make testharness-net-api`（test-guard 包裹）对 M3（五）基线逐
  subtest A/B（分母 14,692 行 / 14,621 键不变）：**changed 21 = F→P 20 +
  Timeout→Pass 1，零 P→F，零新增 T，added/removed 0**。
  - 目标族 6 行：overridemimetype unsent/headers-received shiftjis、
    invalid-mime-type ×2、edge-cases「original continues」、
    open-state-force-xml（T→P）。
  - 链外搭车 15 行逐行溯源同根因（fixture 字节保真 + 解码链另一半）：
    responsetext-decoding.htm 13 行（windows-1252 CT charset、%FF 单字节、
    UTF-16 BOM 载荷——status.py content 字节保真后才可能命中）+
    send-receive-utf16.htm 1 行（CT charset=UTF-16 label 链）。
  - 13,232P/1,444F/16T → **13,253P**/1,424F/15T。
- 日志：基线 `/tmp/netapi-m3-redirect-fixture-full-cur.log`；当前
  `/tmp/netapi-m3-om-family-full-cur.log`；对照脚本 `/tmp/m3-auth-ab.py`。

## 四、挂账

M3 余项：xhr 在账失败簇收窄为 formdata/submitter 4F + cookies.http.html 1T +
redirect-keepalive 1T（permissions-policy 2F 策略面排除不变；
responsexml 族 19 行为在账外存量缺口，未计入门）→ M4 coverage 复测收口
（DC-4）。
