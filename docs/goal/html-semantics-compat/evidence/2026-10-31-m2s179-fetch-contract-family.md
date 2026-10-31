# M2-S179 fetch 契约族 16 红勘定——M2-S3 构造校验链测试面三代际对齐 + Response.clone Set-Cookie 回归根修

## 入场定刀位

s178 记 renderer 余 3F（fetch 契约族）+ engine lib 10F 中 fetch 族 9 红——
三面合计 16 红（engine 9 + browser bin 4 + renderer 3）同族异象，s177
stash 对照已判「同签名既有红」非本流引入。本轮全族勘定归因后施工。

## 勘根（单根链 + 三分类）

**根因链**：net-api 二期 M2-S3（7c61c3f4c，2026-10-29）引入
`_zwRequestValidate`（part01.js）——fetch/Request 构造 URL 经 shim URL
ctor 解析绝对化；shim URL ctor（part02.js）自 R2778（867f97304）起强制
host 回调 `__zw_parse_url`。两条链在测试环境相交：

- **A 类（裸沙箱缺 parser）**：fetch 族测试自建 sandbox 未注册
  `__zw_parse_url` → URL ctor 抛「URL constructor requires a URL
  parser」→ fetch 构造期 reject。生产路径由
  `js_dom_bridge/callbacks.rs:459` 注册，测试面缺位——测试对齐修复。
- **B 类（陈旧断言 vs 刻意 spec 契约）**：M2-S8 关联克隆
  （`_zwAbortSignalFollow`，引 fetch spec「Signal state is cloned」）后
  `req.signal !== req2.signal` 且 abort 跟随；M2-S3 后 `Request.body`
  getter 返 ReadableStream（spec 正文一体）。两案断言仍锁旧值——断言
  对齐当前契约，WPT 语义在案。
- **C 类（about:blank cannot-be-a-base）**：测试文档 URL 为
  `about:blank`，相对 URL（`/api` 等）经 fetch 校验恒 `Invalid URL`
  reject（spec 一致，非 bug）——测试改绝对 URL 或补
  `__zw_get_page_url`。

**二层两 bug（一层修复后显形）**：

1. origin 测试大小写字面量：断言查 `origin\x1e` 小写，wire 首查实为
   title-case `Origin`（fetch spec §5.1 头名大小写不敏感；shim 追加
   title-case 合规）——探针 3 wire dump 实证后修字面量。
2. **Response.clone 丢 Set-Cookie（真实 shim 回归，根修）**：clone 以
   `headers: self.headers` init 重填，Headers-init 走公开迭代
   （Symbol.iterator），response guard 隐藏 Set-Cookie → clone 后
   `getSetCookie()` 空。M2-S3 headers-record 重构移除 `_h` 特判后引入。
   spec：clone 复制**内部头列表**（「a copy of response's headers」，
   fetch.spec.whatwg.org#dom-response-clone），guard 是视图过滤非数据
   删除。

**非本刀真实 bug（挂账钉红）**：iframe XHR `open()` 把 username/password
内嵌为 URL userinfo（part03.js:144-151），iframeFetch（part05.js:4073）
经 globalThis.fetch → `_zwRequestValidate` 抛「Request URL cannot include
credentials」→ 网络错误 DONE + ''。XHR spec：凭证入请求面（Authorization/
凭据模式），不入 URL 串。语料
`test_iframe_service_worker_messages_dispatch_to_iframe_container` 保持
红钉。

## 施工（7 测试文件 + shim 1）

- part10.rs：5 案——3 案补 `__zw_parse_url` 注册（复用
  `js_dom_bridge.rs:4500` pub(crate) 纯函数，part02.rs 先例）；signal 案
  `__reqSigSame` → `__reqSigDiff` + abort 后 `__reqSigFollows`（B 类）；
  r2968 案 `__qBody` → `__qBodyIsStream instanceof` + clone 前置 +
  microtask drain（B 类）；origin 案字面量 `origin\x1e` → `Origin\x1e`
  ×2 + 第三断言 `to_ascii_lowercase` 包裹（头名大小写不敏感语义）。
- part11.rs：body-readers 页 URL `about:blank` → `https://wpt.test/page`
  （C 类，相对 `/api` 需可解析 base）。
- part17.rs：fetch_sync_wire 补 parser 注册（A 类）。
- part23.rs：credentials 案补 `__zw_get_page_url` + parser，期望 URL 绝
  对化（A 类 + 页 URL 缺位）。
- renderer js_worker.rs 3 案 / browser tab_js_worker.rs 4 案：fetch URL
  绝对化（C 类，快照 URL about:blank），POST 期望 URL 同步。
- part01.js：`_zwCloneRawHeaders`——clone 双臂（stream tee / 非流）以
  `_h` 逐键 slice + `_hCase` 拷贝直填内部头列表，绕过公开迭代 guard
  过滤（R3222 钉注）。

## 验证

- **engine lib 3625P/10F → 3633P/2F**：fetch 族 8 转绿（一层 6 + 二层
  2）；余 iframe XHR（挂账钉）+ ua_display_li（渲染流在册）。
- **browser bin tab_js_worker 39P/0F**（4 红清，含 real_http 案）。
- **renderer lib 358P/3F → 361P/0F**。
- shim 装配序拼接（part01,01b,02-07,08a,08,08c,08b,08d）node --check
  SYNTAX_OK（113947 行）。
- forms/interactive corpus join 免跑：改动执行面 `Response.prototype.clone`
  于两语料页面不可达（无 fetch 调用），同 shim 全文已经 3633 条 lib 桥测
  实测。
- renderer lib 全量首跑 1F（`t27p35_post_nav_arrival_order`，POST /submit
  帧缺失）隔离跑绿 + 复跑全量绿——归因测试自述时序窗（200/400ms sleep
  余量）负载翻红一次性，非本刀引入（本刀改动面仅 js_worker fetch 测试
  URL 字面量，与 pinned runtime 实例无共享状态）。

## 归因与边界

- 三类处置均「测试面对齐既定契约」而非放宽校验：A/C 类是测试环境缺
  生产注册/不可解析 base，B 类是断言落后于 M2-S3/M2-S8 已落 spec 契约
  （各案注释引 spec 条款）。
- clone 根修不扩大：只补 clone 双臂内部头列表保真，Headers-init 公开
  迭代语义不动（response guard 对新建 Response 仍生效）。
- 失败时点未逐轮 bisect：根因链以 M2-S3/R2778 两提交机制定谳，16 红全
  族一次性勘定归因，不追认中间轮次。
- Node 直跑探针作废（undici fetch/Response 抢先 + console 覆写）——二
  层归因改用 in-V8 rust 探针（探针验后删净，git 无残留）。

## 挂账

**iframe XHR credentialed-open**（真实 bug，下刀位）：`open()` 凭证内嵌
URL userinfo → fetch 构造校验 reject。根修方向：凭证迁移请求面
（Authorization 头 / 凭证模式参数），URL 串保持无 userinfo，XHR spec
§4.3 open(url, username, password)——「if url includes credentials, set
them on the request」语义。在册红钉
`test_iframe_service_worker_messages_dispatch_to_iframe_container`。

## 下一步候选（本流）

⑥iframe XHR credentialed-open 根修；forms 全树 402F 家族分析；
interactive 44F 归因复读。

## 中间产物

/tmp/s179/：engine_lib.log（v1 一层后 3631P/4F）、engine_lib_v2.log
（终态 3633P/2F）、renderer_lib_v2.log（时序翻红样本）/v3.log（全绿）、
browser_bin_v2.log、shim_concat.js、probe_fetch*.js（作废 node 探针）。
