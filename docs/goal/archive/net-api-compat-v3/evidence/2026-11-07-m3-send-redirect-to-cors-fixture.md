# M3（五）：send-redirect-to-cors 八簇收口——8F → 0F，+8P 零回归（13,224 → 13,232P）

日期：2026-11-07。前置：M3（四）getAllResponseHeaders 排序簇（全量基线
**13,224P**/1,452F/16T，分母 14,692 行）。本批改 `tests/wpt-runner/src/
testharness.rs` 的 `xhr/resources/corsenabled.py` fixture 单文件两处，**零引擎
改动**。在账项「重定向终跳 sync cors-filter 同规」经逐跳探针归因修正：8F 根因
全部在 fixture 缺口，引擎侧（M2-S11 preflight 镜像 + M3（二）redirect 循环 +
M2 批 E6 sync 终响应 cors check）语义本已齐备。

## 一、归因方法（请求序列探针）

fixture handler 临时 eprintln 请求行（method/url/content-type/body）逐案对比 +
独立复刻页（最小 case 序列 + 跨案状态污染排查）。探针证据链：

1. 8 失败案的期望分支全部为「期望 200」；期望 0 的对称案（x-pony 无 safelist
   参）全绿——引擎 preflight enforcement 对无 ACAH 名单的自定义 CT 拒绝正确。
2. `?safelist_content_type` 案（5F）：探针示 `OPTIONS → 停`——preflight 发起且
   被 ACAH 名单拒。对照上游 corsenabled.py：该参存在时**追加第二条**
   `Access-Control-Allow-Headers: content-type`（同名多实例）——本仓 fixture 缺
   此分支，恒单 ACAH `x-test, x-foo`。引擎 `response_headers_get` 走
   `Headers.get` 合并语义（fetch spec get combine），多实例 ACAH 合并后名单即含
   content-type——engine 侧无需改动。
3. 体携带案（3F：307 post with string / 308 FOO text/plain /
   308 FOO multipart）：探针示终跳 200 送达但 `x-request-data` 回显空串——
   fixture 只读 `req.body_bytes`，而 fetch_bridge 契约**文本体在 `req.body`、
   二进制体在 `body_bytes` 互斥**（`__zw_bytes:` csv-decimal 才入 bytes）。
   上游 `request.body` 回显任意体——补 `or_else(|| req.body.clone())`。
   对照案 307 post with typed array（bytes 通路）本就绿，互证。

跨案状态污染假设（preflight cache / pending 泄漏）经最小序列复刻页排除：
隔离跑与序列跑行为一致，失败纯为逐案 fixture 缺口。临查中曾记录中间迭代
痕迹（探针与复刻页），收口前已全部移除，最终 diff 仅 fixture 13 行。

## 二、fixture 改动（testharness.rs 两处）

1. `?safelist_content_type` → `headers.push(("access-control-allow-headers",
   "content-type"))`（上游逐字等价补齐——上游源码 2026-11-07 wpt master 核对）。
2. `x-request-data` 回显 `body_bytes` 缺席时回落 `body`（文本体语义对齐上游
   `request.body`）。

## 三、定向与全量 A/B

- 定向：`make testharness-net-api FILTER=send-redirect-to-cors` **23/23 全绿**
  （原 8F 清零，15 既有绿维持）。
- 全量：`make testharness-net-api` 全量（test-guard 包裹）对 M3（四）基线逐
  subtest A/B（分母 14,692 行 / 14,621 键不变）：**changed 8 全 F→P、零 P→F、
  零 T 变化**——即目标 8 行，逐行可溯源。13,224P/1,452F/16T → **13,232P**/
  1,444F/16T。
- 日志：基线 `/tmp/netapi-m3-sort-full-cur2.log`；当前
  `/tmp/netapi-m3-redirect-fixture-full-cur.log`；对照脚本 `/tmp/m3-auth-ab.py`。

## 四、挂账

M3 余项：xhr 在账失败簇（formdata/submitter 4F + overrideMimeType Shift-JIS
接线 1F + cookies.http.html 1T + redirect-keepalive 1T；permissions-policy 2F
策略面排除）→ M4 coverage 复测收口（DC-4）。
