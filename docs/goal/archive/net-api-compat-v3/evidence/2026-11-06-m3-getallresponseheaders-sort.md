# M3（四）：getAllResponseHeaders legacy-uppercased 排序簇——1F → 0F，+1P 零回归（13,223 → 13,224P）

日期：2026-11-06。前置：M3（三）xhr auth 五簇（全量基线 **13,223P**/1,453F/16T，
分母 14,692 行）。本批改 `crates/engine/src/js_dom_shim/part03.js` 单刀
（getAllResponseHeaders 序列化排序），零 fixture 变化。

## 一、定谳（wpt.fyi「浏览器头序」注记落实为规范文本）

失败行（基线 cur8）：

```
Fail xhr/getallresponseheaders.htm :: XMLHttpRequest: getAllResponseHeaders()
  expected "also-here: Mr. PB\r\newok: lego\r\nfoo-test: 1, 2\r\n__custom: token\r\n"
  but got  "__custom: token\r\nalso-here: Mr. PB\r\newok: lego\r\nfoo-test: 1, 2\r\n"
```

上游语料（wpt master 逐字核对）：fixture `xhr/resources/headers.asis` wire 序
foo-TEST → FOO-test → __Custom → ALSO-here → ewok（本仓 testharness.rs 逐字嵌入
一致）；期望序 also-here → ewok → foo-test → __custom。既非 wire 到达序也非纯
byte 升序（`_`=0x5F 应排首）——根因在 xhr.spec §3.6.5 的**专用比较器**：

> Let headers be the result of sorting initialHeaders in ascending order, with a
> being less than b if a's name is **legacy-uppercased-byte less than** b's name.

legacy-uppercased-byte less than = 名先 **byte-uppercased**（仅 0x61-0x7A →
0x41-0x5A）再 byte 比较——下划线 0x5F 不属小写字母域、恒大于大写字母 0x41-0x5A，
`__custom` 因此排在小写字母名**之后**：ALSO-HERE(0x41…) < EWOK(0x45…) <
FOO-TEST(0x46…) < __CUSTOM(0x5F…) = 期望序，wpt.fyi Firefox 7/7 与各引擎绿色
由此贯通。
https://xhr.spec.whatwg.org/#legacy-uppercased-byte-less-than

与 fetch Headers 迭代面（forEach/entries，Fetch §5.1 纯 byte 排序，`__custom`
排首）是**两套口径**——headers-sorting 族断 `__custom` 排首的既有绿面不受扰动。

## 二、引擎改动（单刀）

`getAllResponseHeaders` 序列化前对存储表键显式排序：键收集 → `sort(比较器)` →
逐项 `name: value\r\n` 拼接。比较器按 legacy-uppercased-byte less than 内联实现
（`replace(/[a-z]/g, … - 0x20)` 后 `<`/`>` 比较）。多值合并与 forbidden
response-header 名排除维持既有通路不变（存储表由响应头 fill 构建——同源
guard='response' 过滤 set-cookie/set-cookie2、跨源 cors-filter 按名剔除，两路
皆不透传；多值经 Headers `_h` 数组 `, ` 合并）。

中间迭代留痕：第一刀曾改响应头 fill 直读 Headers `_h` 内部表（wire 到达序），
实验证伪「浏览器头序 = wire 序」假设后**回退**——序列化序由 getter 排序单独
决定，fill 通路与基线逐字节同型，最终 diff 单刀最小。

## 三、定向与全量 A/B

- 定向：`make testharness-net-api FILTER=getallresponseheaders` **10/10 全绿**
  （主断言序 also-here/ewok/foo-test/__custom + 单头 .asis 6 案 +
  after-reuse/cookies/status 3 邻案全维持）。
- 全量：`make testharness-net-api` 全量跑（test-guard 包裹）对 M3（三）基线
  cur8 逐 subtest A/B（分母 14,692 行 / 14,621 键不变）：**changed 1 全 F→P、
  零 P→F、零 T 变化**——即目标 subtest 自身。13,223P/1,453F/16T →
  **13,224P**/1,452F/16T。
- 日志：基线 `/tmp/netapi-m3-auth-full-cur8.log`；当前
  `/tmp/netapi-m3-sort-full-cur2.log`（最终 diff 全量跑）；对照脚本
  `/tmp/m3-auth-ab.py`。

## 四、挂账

M3 余项不变：重定向终跳 sync cors-filter 同规 + xhr 在账失败簇
（formdata/submitter 4F + overrideMimeType Shift-JIS 接线 1F +
cookies.http.html 1T + redirect-keepalive 1T；permissions-policy 2F 策略面
排除）→ M4 coverage 复测收口（DC-4）。
