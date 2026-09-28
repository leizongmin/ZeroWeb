# M2-S1 — fetch scheme dispatch + data: URL processor + body MIME type

**日期**: 2026-09-28
**切片**: M2-S1（fetch 错误面「Should have rejected」簇 + data-urls 簇 + mimesniff Blob/File
.type 联动面）
**原始数据**: [fetch](2026-09-28-m2-s1-scheme-dispatch-fetch.json) ·
[xhr](2026-09-28-m2-s1-scheme-dispatch-xhr.json) · [url](2026-09-28-m2-s1-scheme-dispatch-url.json) ·
[mimesniff](2026-09-28-m2-s1-scheme-dispatch-mimesniff.json) ·
[streams](2026-09-28-m2-s1-scheme-dispatch-streams.json) ·
[eventsource](2026-09-28-m2-s1-scheme-dispatch-eventsource.json)

## 结果（六 corpus 分类通过率，对照 M1 基线）

| corpus | M1 基线 | M2-S1 | Δ pass |
|---|---|---|---|
| fetch | 480/1638（29.3%） | **772/1638（47.1%）** | +292 |
| xhr | 101/357（28.3%） | 102/435（23.4%*） | +1 |
| url | 337/504（66.9%） | 337/504（66.9%） | 0 |
| mimesniff | 124/1898（6.5%） | **1883/1898（99.2%）** | +1759 |
| streams | 253/590（42.9%） | 253/590（42.9%） | 0 |
| eventsource | 2/32（6.2%） | 2/32（6.2%） | 0 |
| **合计** | **1297/5019（25.8%）** | **3349/5065（66.1%）** | **+2052** |

\* xhr 分母 357→435（helper 装载面次生增长），pass +1 零回归。

## 修齐内容（crates/engine/src/js_dom_shim/part01.js + part02.js）

1. **fetch scheme dispatch**（`_zwFetchSchemeDispatch`，main fetch §4.1 step 12）：
   - **bad port**（§2.9，79 端口全表）：HTTP(S) scheme 显式 port 命中 → network error
     （fetch reject TypeError）。request-bad-port 0/83 → **83/83**。
   - **data: processor**（§6 + mimesniff parse/serialize + forgiving-base64）：
     data-urls base64/processing 2/154 → **154/154**；scheme-data **8/8**（含 HEAD
     null-body——main fetch §4.1 step 22）。URL parse 失败（`data://test:test/` 形态）→
     TypeError（host `__zw_parse_url` 注册时；未注册环境 lenient 跳过）。
   - **blob:**（scheme fetch blob 分支）：GET-only + 同源 + `_zwBlobStore` 命中 →
     200 + Content-Length/Content-Type；未命中/跨源/非 GET → network error。
     scheme-blob 0/18 → **18/18**。
   - **about:/file:/未知 scheme** → network error。scheme-about 0/7 → **7/7**、
     scheme-others 0/16 → **16/16**。
   - 位置：shim 侧共享路径（runner 与浏览器同行为，zero-net 协议栈零触碰）；http/https
     返 null 落 host 桥原路径（存量行为零变化）。
2. **MIME 基建**（同 IIFE 共享函数，data: processor 与 Blob/type 链共用）：
   `_zwParseMimeType` / `_zwSerializeMimeType`（mimesniff §4.4/§4.5 全分支：无 '=' 参数名
   丢弃、quoted-string 提取、token 值空串跳过、首胜参数、值非 token 引号包裹）、
   `_zwPercentDecodeBytes`（非法 % 序列字面保留、surrogate pair UTF-8）、
   `_zwForgivingBase64Decode`（whitespace 精确集 + padding ∈ {0, need} 规则——77 向量校准：
   `'ab'` 通过 / `'ab='` 失败 / `'ab=='` 通过）。
3. **Blob/File constructor type**（FileAPI）：type 经 MIME parse 失败 → 空串、成功 →
   **序列化形态**（`'x/x; bonus=x'` → `'x/x;bonus=x'`、`charset=" gbk"` 引号保留）。
   mimesniff mime-types.json Blob/File 面 124 → 全绿。
4. **Response/Request blob() MIME type**（Body mixin）：type = get the MIME type
   （extract a MIME type + serialize，非 raw header 值）——`x/x;\x01=x;bonus=x` 参数丢弃
   后 `bonus=x` 保留形态。mimesniff (Request/Response) 面随之上绿。

## 残余簇（下一步）

- mimesniff ×15：`new Request(..., {headers: [["Content-Type", value-with-\x00/\r/\n]]})`
  应抛 TypeError——**Headers append/set 值校验缺失**（header value 不得含 0x00/LF/CR、
  不得首尾 HTTP tab/space；name 为 field-name token）。归 M2-S2（headers 面簇：
  forbidden header 72 + headers 错误面 16 同通道）。
- fetch 错误面残余（47.1% → 更高）：`Should have rejected` 余量、response-stream-disturbed
  簇（disturbed/locked 语义）、integrity/credentials .py 依赖（fixture 通道 P7 记账）。
- eventsource 23 Timeout / fetch status-0 簇：runner fixture 通道（P7）未动。

## 验证

- `make testharness-net-api`（六 corpus 全量复跑，上表）；零回归域：url/streams/
  eventsource 逐 subtest 零 delta、xhr pass +1。
- `make test` 全绿（workspace 68+ 套，含 engine shim 全部回归）——见 commit 记录。
