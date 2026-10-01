# M3 — TextEncoder/BOM/fatal/ignore 模式 + 编码往返 + streams 语义

**日期**: 2026-10-02
**套件**: `make testharness-encoding`（WPT 315976933870b34d6ea30e3f6643403edae678ba，全语料 36 案）
**原始数据**: 本轮 `/tmp/m3-run2.json`（复跑 `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- ./target/release/zero-wpt-runner testharness-encoding --json`）

## 结果

| 指标 | M2 | M3 |
|---|---|---|
| subtests | 11876/12075 = 98.4% | **12070/12075 = 99.96%** |
| encoding top | 11460/11604 = 98.8% | 11569/11604 = **99.7%** |
| streams | 59/114 = 51.8% | 107/114 = **93.9%** |
| legacy-mb `.any.js` | 357/357 | 357/357（恒值） |

面翻转：encodeInto 36/111 → **110/111**；streams/decode-ignore-bom 0/12 → **12/12**；
streams/decode-attributes 15/28 → **27/28**；streams/encode-utf8 3/19 → **19/19**；
streams/decode-bad-chunks 0/5 → **5/5**；streams/decode-non-utf8 9/13 → **13/13**；
streams/readable-writable-properties 0/2 → **2/2**；textdecoder-fatal 2/36 → **35/36**；
textdecoder-mistakes 78/87 → **86/87**；api-surrogates-utf8 1/6 → **6/6**；
textencoder-utf16-surrogates 1/7 → **7/7**；api-basics 5/6 → **6/6**；decode-eof/copy/arguments/
byte-order-marks/unsupported-encodings/streaming/fatal-streaming/ignorebom 全绿。

## 修齐面（M3 落地）

- **utf-8 解码 spec 状态机**（part02.js `_zw_utf8_decode_state`）：逐字节消费——非法续字节
  U+FFFD + 重处理（旧版前置整序列检查吞尾部 ASCII：`F0 9F 41` 只出 FFFD 丢 `A`；错误子
  部分合并）；不完整序列跨 chunk 驻留 state（替代 R3012 字节 tail carry）；fatal →
  TypeError（错误后重置——spec 非 stream decode 重置序列）。旧实现为 R3012/net-api
  M4-S21 增量修补版，本片按 spec #utf-8-decoder 重写。
- **BOM 前缀嗅探机**（`_zw_u8_bom_feed`）：EF BB BF 可跨 chunk 拆分（copy 案
  `[EF,BB]`+`[BF,40]` 面）、剥除后同调用余下字节内容直通（mistakes「repeats」双 BOM 面）、
  flush 未完成前缀退回为内容；ignoreBOM=true 不启用（BOM 解出 U+FEFF）。
- **TextEncoder**：encode 默认参数 ""（api-basics Default inputs）；孤立代理（高/低）→
  U+FFFD 字节（旧版 CESU-8 直编代理——api-surrogates/textencoder-utf16-surrogates 面）；
  encodeInto spec 语义——read/written 停在字符边界（字符永不劈开、destination 容不下
  整字符即停）+ destination 限 Uint8Array（Int8Array 等 → TypeError）。
- **TextDecoderStream**：options 转发（fatal/ignoreBOM IDL 反射）；label 默认 "utf-8" 而
  `''` 抛 RangeError（spec 构造器缺省与 TextDecoder 的 "" 不同）；chunk 类型门
  （非 BufferSource → TypeError）；transform/flush 同步抛经 TransformStream
  performTransform → TransformError（fatal malformed → readable error + 写侧 reject）。
- **TextEncoderStream**：跨 chunk UTF-16 边界驻留（尾部孤立高代理待下块成对）；尾部空
  chunk 忽略；chunk 全量 ToString 转换（undefined → "undefined"）。
- **WritableStream.prototype.getWriter** 委托（品牌访问面——readable 侧 net-api M4-S8
  已有，writable 侧此前缺）。
- **XHR final encoding + #decode BOM 嗅探序**（part03.js）：响应字节首部 BOM（UTF-8/
  UTF-16LE/BE）优先于 override/response charset 定解码器；表外 override 回落 UTF-8
  （unsupported-encodings：override=utf-32 + FF FE BOM → utf-16le 面）。
- **host 空块 guard**（text_encoding.rs）：encoding_rs 0.8.35 空输入调用会丢弃驻留 lead
  （big5 实证：hold 0x87 → 空 chunk → 0x40 得 `@` 而非 `䏰`）——空块非 flush 直通无输出，
  flush 面不受影响（回归测试锁定：stream_empty_chunk_guard）。

## 残差（5 Fail，记账）

| 案 | 形态 | 归属 |
|---|---|---|
| streams/decode-utf8 ×2（transferred Uint8Array/ArrayBuffer chunk） | detached ArrayBuffer——postMessage transfer 面缺结构化克隆/transfer 管道 | 结构面（workers/transfer 域，encoding 语义外） |
| encodeInto ×1（detached output buffer） | 同上（SAB/transfer 构造面） | 同上 |
| textdecoder-copy ×1（SharedArrayBuffer 变体） | `createBuffer('SharedArrayBuffer')` runner 无 SAB → buf undefined | 同上（SAB infra） |
| textdecoder-mistakes ×1（fatal stream: iso-2022-jp） | spec 要求 fatal 错误后 iso-2022-jp ESC 模式机驻留（唯一不清状态的编码）；encoding_rs 错误契约要求错误后弃用 decoder，无法表达跨错误状态保持 | encoding_rs 上游契约（上游 test 文件自引 encoding_rs#126 同族） |

## 门禁与共享面核对

- net-api 六 corpus 复跑：**9319/11688 逐域同值零漂移**（streams 905/1020、xhr 557/584、
  mimesniff 1898/1898、fetch 1497/2087、url 4429/6065、eventsource 33/34）。
- engine lib 2788 passed / 0 failed；host 单测 11 + shim 集成测 1（M2 落地）。
