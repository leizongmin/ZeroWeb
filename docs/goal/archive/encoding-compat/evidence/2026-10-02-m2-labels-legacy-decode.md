# M2 — labels 标签全表 + TextDecoder legacy 解码

**日期**: 2026-10-02
**套件**: `make testharness-encoding`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: 本轮 `/tmp/m2-run2.json`（全语料 36 案；复跑命令 `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- ./target/release/zero-wpt-runner testharness-encoding --json`）

## 结果

| 指标 | M1 基线 | M2 |
|---|---|---|
| 执行面 | 21 案（语料缺角——首拉超时后幂等快路径跳过整目录，`.any.js` 主形态未补齐） | **36 案（FORCE=1 补齐 top 24 .any.js）** |
| subtests | 268/4403 = 6.1% | **11876/12075 = 98.4%** |
| encoding top | 214/3932 = 5.4% | 11460/11604 = **98.8%** |
| legacy-mb `.any.js` | 7/357 = 2.0% | 357/357 = **100%** |
| streams | 47/114 = 41.2% | 59/114 = 51.8%（M3 面） |

口径注记：M1 与 M2 分母不可直接比——语料补齐（21→36 案）+ variant 并跑全量；
两轮均为「拉到什么跑什么」的诚实口径，缺角根因与补拉动作记账于此。

## 面翻转（M2 修齐项）

| 案面 | M1 | M2 |
|---|---|---|
| api-invalid-label（label 校验门） | 0/3421 | **3421/3421** |
| single-byte-decoder（28 单字节编码） | 168/336 | **336/336**（TextDecoder 半脸 0→168） |
| gbk-decoder | 0/82 | **82/82** |
| gb18030-decoder | 7/275 | **275/275** |
| iso-2022-jp-decoder（ESC 状态机） | 1/34 | **34/34** |
| api-replacement-encodings（replacement 构造拒绝） | 0/6 | **6/6** |
| textdecoder-labels / textdecoder-fatal-single-byte 等补拉案 | 未执行 | 全绿（textdecoder-labels 等零 Fail） |

## 架构（M2 落地）

- **host 面**：`js_dom_bridge/text_encoding.rs`（encoding_rs——WHATWG encoding 标准
  tables 的生成实现，zero-net 同款工作区依赖）三 native：`__zw_text_encoding_of`
  （labels 全表匹配 → 规范名）、`__zw_text_decoder_new`（有状态 decoder，
  `with_bom_removal`/`without_bom_handling` 承接 BOM/ignoreBOM 语义）、
  `__zw_text_decoder_decode`（字节 csv wire 进、JSON {text, err} 出）。
- **有状态 decoder 表**：spec decode 跨调用驻留半截多字节 lead 与 iso-2022-jp ESC
  模式机——纯字节 carry 无法表达（encoding_rs 消费 lead 后宿主侧已无该字节）。
  host 持 handle→Decoder 表（FIFO 封顶 4096 透明逐出）；shim 持 handle，非 stream
  flush 后 `_zwDone` 标记下一轮重建（spec：非 stream decode() 重置状态）。
- **shim 构造门**（part02.js）：label 经 host 查表；未知 → RangeError；replacement
  → RangeError（spec：replacement 不可经 decoder 构造）；utf-8 留守纯 JS 快路
  （零 wire 开销），非 utf-8 走 host；host 未注册（reftest/polyfill）降级 utf-8
  零回归。
- **XHR final-encoding 面**（part03.js）：overrideMimeType charset = replacement →
  非空输入整段单 U+FFFD（replacement-encodings.any.js 的 XHR 车辆）。
- **测试资产**：host 单测 10 条（tables/单字节/GBK/GB18030/iso-2022-jp/UTF-16+BOM/
  replacement/跨块 split/fatal/封顶逐出）+ shim 集成测 1 条（part02.rs——构造门
  RangeError 双脸 + 五编码真值 + fatal + ignoreBOM + 模式机跨调用）。

## 回归核对（共享面，run-rules §10）

- **net-api 六 corpus 重跑**：streams **905/1020 = 88.7%**（与 M4-S28 收口恒值精确
  同值）；xhr 557/584 = 95.4%（收口 545/583 → **+12**，legacy overrideMimeType 解码
  修齐正漂移）；六 corpus 合计 9319 pass（收口 9292 → +27，零回归）。
- **engine 全量 lib 测试**：2787 passed / 0 failed（含新增 11 条）。

## 残差（136 Fail → M3 簇）

| 簇 | 案面 | 归属 |
|---|---|---|
| TextEncoder/encodeInto spec 语义（read/written 拆分、destination offset） | encodeInto 36/111 | **M3** |
| utf-8 JS 快路 spec 状态机（逐字节 U+FFFD、fatal、EOF、BOM split/sticky） | textdecoder-fatal 2/36、mistakes 78/87、eof 0/2、copy 0/2 等 | **M3** |
| lone surrogate encode → U+FFFD 字节（现 CESU-8 直编） | api-surrogates 1/6、textencoder-utf16-surrogates 1/7 | **M3** |
| TextDecoderStream options 转发 + chunk 类型门 + fatal error 传播 | decode-attributes 15/28、bad-chunks 0/5、non-utf8 9/13 | **M3** |
| TextEncoderStream 跨 chunk 多字节缓冲 | encode-utf8 3/19 | **M3** |
| brand check 基建 + detached ArrayBuffer（transfer 面） | readable-writable-properties 0/2、decode-utf8 10/12 | M3/记账 |

## 下一步（M3）

1. utf-8 JS 快路按 spec #utf-8-decoder 状态机重写（含 BOM 前缀机跨 chunk + fatal 线程）
2. TextEncoder：默认参数 ""、lone surrogate → U+FFFD、encodeInto read/written spec 语义
3. TextDecoderStream/EncoderStream：options 转发、chunk 类型门、跨 chunk 编码缓冲、
   fatal error 传播
