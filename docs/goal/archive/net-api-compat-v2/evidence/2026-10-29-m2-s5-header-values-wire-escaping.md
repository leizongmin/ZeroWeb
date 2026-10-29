# M2-S5 — header-values-normalize wire 转义 + fixture 方法/空值回显（2026-10-29）

**切片**: goal v2.7「后续可推进小片」第 1 位——`fetch/api/headers/
header-values-normalize.any.js` 36F（承 M2-S3c 记账 3）。**二期 M 收口后
重开小片**，不改动收口判定（M 收口 DC-4 终稿以 `netapi-m2s3c-r4` 为准）。

## 36F 三分解（M2-S3c 记账 3 → 本片归因）

| 子因 | legs | 根因面 |
|---|---:|---|
| B：XHR 全腿 `assert_unreached: XHR should not fail` | 30 | runner fixture 兜底方法拒绝（非引擎） |
| C：fetch strip-set 腿 `expected "" but got null` | 4 | fixture 空值头不回显 |
| A：fetch `\x1e`/`\x1f` 腿 `expected "\x1e" but got null` | 2 | wire 分隔符腐蚀 |

i=0（0x00）双腿（XHR SyntaxError / fetch TypeError）本就通过——引擎 0x00
拒绝面无缺口。

## 根因与实现

### 1. wire 分隔符腐蚀（fetch_bridge + shim 对称转义）

fetch 头 wire（shim↔host）以 `\x1e` 分隔 name/value、`\x1f` 分隔字段；
头值可含任意控制字符（本语料 0x01-0x1F 面），值内出现分隔符即被拆解腐蚀
——`\x1e` 腿请求值被截断、响应值错位 → 回读 null。

实现：ESC 方案对称转义（`\x1b`→`\x1bX`、`\x1e`→`\x1bE`、`\x1f`→`\x1bF`；
先转 ESC 保证逆变换无歧义；未知序列字面保留向后兼容）。头名恒为合法 token
不转义。

| 方向 | 生产（转义） | 消费（还原） |
|---|---|---|
| 请求 JS→Rust | `part01.js` `_headersToWire` / `_zwAddHeader` / part05 EventSource `__zwInternalHeadersWire` | `fetch_bridge.rs` `decode_headers` |
| 响应 Rust→JS | `fetch_bridge.rs` `encode_headers`（经 `serialize_response` 覆盖 async/sync/frame sidecar 全部响应 wire） | `part01.js` `_parseHeadersWire` / `_zwGetWireHeader` / `_zwPreNamesOf`（safelist 判定见真实值）/ frame sidecar `hdrs360` |

shim 内部 wire 再拼装点（preflight 名单重组、redirect 逐跳 kept wire 等）
拆分后原样重组，转义形不二次加工。part07 CacheStorage wire 为独立契约
（JSON 载荷内自洽），不在本边界，记账不动。

### 2. fixture 兜底方法拒绝（XHR 30F）

上游 `inspect-headers.py` 是 wptserve 普通 handler，**方法无关**（任意方法
回显请求头）；语料 XHR 腿用 `POST`（fetch 腿默认 GET）。runner
`wpt_data_fetch_handler` 兜底 `method != GET && != OPTIONS → Err` 在 fixture
分派之前 → POST 到 inspect-headers.py 直接网络错误 → XHR onerror 全灭。
实现：inspect-headers.py 豁免兜底拒绝（`testharness.rs`），其余端点维持。

### 3. fixture 空值头不回显（strip-set 4F）

值 strip 后为 `""` 的头仍应回显：上游按「头存在」判定（`in
request.headers`），runner 按 `!value.is_empty()` 判定把空值头静默丢弃 →
`x-request-val1` 缺席 → got null。实现：presence 判定（`hits.is_empty() →
continue`），空串值照常回显 `x-request-val1: `。

## 验证

- probe（`testharness-net-api header-values`）：header-values-normalize
  26/62 → **62/62**（0 Fail）——三因（转义/方法/空值回显）全数清偿。
- 全通道 A/B（`netapi-m2s5-r1.json` vs **M 收口参考**
  `netapi-m2s3c-r4.json`，逐 subtest）：**+39 增益 / 0 回归 /
  MISSING 0**（10843 → 10882 pass）。
  - 本片 +36（normalize 26→62 收口）+ 连带 +3：`header-values.any.js`
    all-valid ×2（合法值 POST 经 inspect-headers.py 回显解锁）、
    `request-headers.any.js` POST 腿 ×1（S3c 记账 2 方法面首腿清偿，
    余 22F 维持挂账）。
  - url / mimesniff 零变化（169F / 602F 持平）——wire 转义对既有全通道
    语义零扰动（同步 XHR / frame sidecar / EventSource 内部 wire 同一
    转义边界覆盖）。
  - fetch/api 域：334F → **295F**（−39；Timeout 1 两轮同为 harness 计数
    边界，非本片变化）。
- 构建无 warning（`build-m2s5-r1.log`）；fetch_bridge 单测 12/12（含新
  转义往返 `headers_wire_escapes_separator_bytes_in_values`）；运行
  stderr **0 panic**。
- 参考（vs `netapi-m2s3b-r3.json` 二期前基线）：+298 增益 / 1 Pass→Fail
  （SRI「CORS invalid integrity」显影记账，M2-S3c 记账 1，非本片变化）。

## 记账（未清偿）

1. **0x00 值的真实 HTTP 传输面**：本片后 `\x01-\x1F`（除分隔符）值全链路
   保真；0x00 在 Headers fill 拒绝（fetch TypeError / XHR SyntaxError），
   语料与规范一致，无传输需求。
2. **part07 CacheStorage wire**：同族分隔符腐蚀面（值含 `\x1e`），独立
   契约、语料无钉，挂账不动。
