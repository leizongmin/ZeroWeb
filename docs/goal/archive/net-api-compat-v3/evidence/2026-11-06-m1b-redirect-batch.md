# M1b：fetch/api/redirect 批次导入 + redirect.py fixture 升级 + fetch shim 重定向语义修复

日期：2026-11-06。前置：M0 对账甄别、M1a 枚举器窗体解锁、M1b 部分
（folder.txt + xhr 簇归因图，见
[前档](2026-11-06-m1b-folder-txt-and-xhr-cluster-map.md)）。

## 一、批次内容

1. **语料导入**：`fetch/api/redirect` 14 案入 `NET_API_CORPUS_SUBDIRS`
   （count/empty-location/keepalive×2/location/location-escape.tentative/
   method/mode/origin/referrer/referrer-override/schemes/to-dataurl/
   back-to-original-origin，pin `315976933870`）。
2. **redirect.py fixture 升级**（上游逐字语义）：全局原子计数废除，改
   per-token stash `{count,preflight}`（take→put）；OPTIONS 置
   preflight 位（无 `redirect_preflight` 时 200 不重定向）；Location 附
   全量原 query + `count=N`（scheme 认定按 urlparse 语义——首段
   `^[A-Za-z][A-Za-z0-9+.-]*:` 前缀，无前缀即空；仅 空/http/https 且非
   `?simple=` 时附加）；`?max_count=` 超限回 200 + `str(count-1)`；
   `?redirect_referrerpolicy=` 透传 Referrer-Policy。
   **stash 键 `redirect\x1f{token}` 命名空间化**——runner stash 为 flat
   单键（上游为 per-token 多键 dict），裸 token 键会被 preflight.py
   fixture 误读（首字段 count 被当作 did-preflight，曾致 cors-redirect
   20 案 P→F；根因与规避见
   [learnings](../../../../learnings/bugs/2026-11/2026-11-06-flat-stash-key-collision-between-endpoint-fixtures.md)），
   `clean-stash.py` fixture 同步清两键。
3. **新 fixture**：`redirect-empty-location.py`（302 + 空 Location）、
   `method.py`（x-request-method 等六头回显 + 体回显，`?cors` CORS 应答）。
4. **fetch shim 重定向语义修复**（`crates/engine/src/js_dom_shim/part01.js`
   settleFetch/finishFetch）：
   - `redirect: 'error'` + 重定向状态 → TypeError（先于 Location 解析）；
   - Location 头 trim 后为空 → TypeError（empty-location 面）；
   - 跳转目标 scheme 非 http/https（data:/mailto:）→ TypeError；
   - hopCount ≥ 20 → TypeError（不再落末跳 302）；
   - 301/302/303 仅 POST→GET 改写（修 303 HEAD 被误改写），改写落体时
     剥 body 衍生头（content-length/type/encoding/language/location）；
   - `response.redirected = hopCount > 0`（URL list > 1 语义）。

## 二、`.h2.` 形态案正式排除（M1b 甄别记账）

`net_api_case_skipped` 增 `name.contains(".h2.")` 通用排除：上游 `.h2.`
命名案经 wptserve h2 配置按 HTTP/2 交付，runner 无 HTTP/2 栈。影响 26
子测试移出分母：`fetch/api/basic/request-upload.h2.any.js` 12（10P/1F/1T，
h1 等价路径本可过）、`fetch/api/basic/status.h2.any.js` 7F、
`xhr/status.h2.window.js` 7F。非语料缺失，engine h2 支持落地后可回收。

## 三、全量 A/B（零回归门，DC-4）

`make testharness-net-api` 全量（test-guard 包裹），M1a 全量 log 对照
（Python `re.finditer` 解析，NUL 字节坑规避）：

| | Pass | Fail | Timeout | 案 |
|---|---:|---:|---:|---:|
| M1a 基线 | 12,655 | 1,533 | 16 | 543+ |
| M1b 本批 | **12,924** | 1,602 | 16 | +14 案 |

- **added 364**（264P/99F/1T，14 案）——redirect 批次净新增；
- **removed 26**（10P/15F/1T）——全部为 `.h2.` 排除（见上）；
- **changed 15，全部 Fail→Pass，零 P→F**：2 folder.txt 面（前档已记）+
  13 xhr send-redirect 面（301/302/303 POST→GET 改写、303 HEAD 保方法、
  body 衍生头剥离、bogus Location——shim 修复与 XHR 路径共享收益）；
- cors-redirect 20 对（前次中间轮 P→F 回归）经 stash 命名空间化后回到
  逐字节一致（Pass→Pass）。

redirect 批次自身：364 子测试 264P/99F/1T = **批次门 98.5% 校验基准
264/(264+99+1) = 72.6%——未达批次门**，残余失败为引擎语义簇（下节），
转 M3 收敛；本批按 DC-2 记账为导入完成 + 簇归因落档。

## 四、残余失败簇（M3 账，99F+1T）

| 簇 | 子测试 | 归因 |
|---|---:|---|
| referrer 族 | 63（override 51 + referrer 12） | referrer-policy 重定向面（Referrer-Policy 透传已接，referrer 计算/继承语义缺口） |
| redirect-mode | 11 | 跨源 no-cors/manual 矩阵（manual opaqueredirect 面 + no-cors follow 拒绝语义） |
| redirect-origin | 10 | 同源跳 Origin 头置发语义 |
| redirect-method | 9 | 307/308 GET 跳后 Content-Length:0、POST 体携带 |
| location-escape | 4 | 原始字节 URL 保真（`%0D%0A` 面等） |
| back-to-original-origin | 2 | 响应 type 污染（跨源往返后 type 判定） |
| keepalive | 1T | iframe 面挂起 |

## 五、后续

- M3：上表簇收敛（与 xhr redirect 引擎缺口合并——xhr send-redirect 面
  尚余 307/308/循环/bogus scheme 子测试在账）。
- M2：auth1-11 HTTP Basic、getResponseHeader 响应头控制面、CORS
  preflight 端点扩表、websockets 非 wss 面（interfaces 74 案）。
