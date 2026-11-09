---
date: 2026-11-06
modules: engine, tests/wpt-runner
---

# 响应头值是字节序列不是 Unicode 文本——fixture percent-decode 语义两态

## 问题

fetch redirect-location-escape 四行（tentative 案面）全部
`TypeError: Invalid header value` 后落 `__zw_fetch_error:throw` 错误对象
（fetch promise 不 reject 反 resolve status-0 响应）。案面 Location 头含
原始字节（`%E2%98%83` → 三字节、`%FF` → 单字节），期望终跳 URL 以
`%E2%98%83`/`%FF` 单字节转义呈现。

## 根因

两层语义错位：

1. **fixture 解码层**：`wpt_query_value` 用 `percent_decode_str(..)
   .decode_utf8_lossy()` 把查询参数解成 Unicode 文本——`%E2%98%83` 是合法
   UTF-8（☃ U+2603）、`%FF` lossy 成 U+FFFD（U+FFFD）。而上游 wptserve/
   浏览器链路上响应头值恒为**字节序列**：JS `Headers` 的 ByteString 语义
   逐字节映射 U+0000..U+00FF，charCode > 0xFF 即非法——U+2603/U+FFFD 的
   Location 头在 `_zwValidateHeader` 处抛 `Invalid header value`。丢字节的
   lossy 解码还会让 `%FF` 永远无法以 `%FF` 形态回到线上。
2. **shim 解析层**：字节映射后的 Location 字符（U+00E2 U+0098 U+0083）交
   `new URL()` 解析时 query 通道按 **UTF-8 重编**——U+00E2 变双字节
   `%C3%A2` mojibake。浏览器行为（whatwg/fetch#883，tentative）是重定向
   Location 的 ≥0x80 字节以 **%XX 单字节**转义进 URL（`%` 字面保留）。

次要发现：executor 的 `catch` 把 fetch 流程内同步异常 resolve 成
`__zw_fetch_error:throw` 错误对象（status 0）而非 reject——案面症状为
「`resp.url` undefined」而非异常栈，第一眼指向错误方向。

## 解决

- fixture 侧新增 `wpt_query_value_bytes`（percent-decode 逐字节 `b as
  char`），redirect.py 的 `location` 参数改用之；其余 UTF-8 文本语义消费
  者不动（改全局解码会外溢）。
- shim 侧 settleFetch 解析 Location 前对 0x80..0xFF 字符逐个转义
  `%XX`（大写十六进制，`%0X` 补零），ASCII 值逐字符零变化。

## 避免方式

- 任何把查询参数/文件内容**原样回写响应头**的 fixture（Location、echo
  类），解码必须走字节语义变体，不走 UTF-8 文本解码。
- JS 侧看到 `Invalid header value` / 错误对象症状时，先查头值的字符域
  （charCode > 0xFF 的字符只能来自解码层误把多字节序列合成了单个字符）。
- fetch executor 的 throw-swallow（resolve 错误对象）在排查时要先剥掉，
  否则异常根因被症状掩盖。
