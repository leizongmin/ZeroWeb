# M2-S180 XHR credentialed-open 根修——URL 凭证 send 层提取为 Basic Authorization，s179 挂账红钉转绿

## 入场定刀位

s179 挂账唯一真实 bug：iframe XHR `open()` 凭证内嵌 URL userinfo →
iframeFetch 经 globalThis.fetch → M2-S3 构造校验「Request URL cannot
include credentials」reject → 网络错误 DONE + ''。红钉
`test_iframe_service_worker_messages_dispatch_to_iframe_container`
（part23.rs，s179 账本 engine 2F 之一）。

## 勘根（共享路径定位，非 iframe 特有）

链路三层：

1. **open() 凭证入 URL（合规）**：part03.js `open()` username/password
   经 URL setter 写入 userinfo（xhr.spec §the open()-method 步骤
   14-15「set the username/password, given request's URL」）——open 面
   spec 一致，不动。
2. **send 三出口同源**：sync 主发（`__zw_fetch('__zwxhr:sync', …,
   self._zwXhrUrl, syncHeaders)`，headers 由 `_headersToWire(
   self._zwXhrHeaders)` 构建）+ sync preflight OPTIONS + async
   `fetchFn(self._zwXhrUrl, {headers: self._zwXhrHeaders, …})`——全部
   读 `_zwXhrUrl`/`_zwXhrHeaders` 两个共享槽。
3. **运输层撞构造校验**：主窗 async XHR `fetchFn = globalThis.fetch`
   （part03.js:956）、iframe XHR `_zwXhrFetch = iframeFetch`
   （part05.js:4223，最终亦 globalThis.fetch）——**任何**带凭证 open 的
   XHR 自 M2-S3 起恒拒。iframe 红钉只是语料唯一覆盖；主窗 async 同面。

**spec 归位**：fetch() API 层的凭证 URL 校验（fetch §HTTP fetch /
Request 构造）不适用于 XHR 运输；凭证的消耗在 fetch §HTTP-network
fetch——「request's URL includes credentials」提取臂转 Basic
Authentication（RFC 7617）。shim 的网络层就是 host `__zw_fetch` 前的
这一段，转换应落在此处。

## 施工（1 shim 文件 + 1 测试文件）

- part03.js `send()` 状态守卫后、sync/async 分叉前单点归一：URL 含
  `@` → `new URL` 解析 → username/password 非空时：作者
  authorization 头缺席则 `_zwXhrHeaders['authorization'] =
  'Basic ' + btoa(user + ':' + pass)`（part01b btoa，RFC 7617
  Latin-1 域）；setter 清空双凭证字段（WHATWG URL 序列化器双空省略
  userinfo，node 实证）→ `_zwXhrUrl = href` 无凭证分派。三出口共用
  两槽自动覆盖；重 send 幂等（URL 已无凭证）；path 含 `@` 的正常 URL
  双空门天然放行；作者 Authorization 优先不覆写（认证入口已有凭据）。
- part23.rs 红钉断言改写（Class B 陈旧契约→根修语义）：原断言钉
  「凭证保留在 wire URL」（M2-S3 前契约）；改钉 credential-free URL
  分派 + `authorization\x1eBasic YWE6YmI=`（base64('aa:bb')）落
  headersWire（回调记录扩第 4 字段 headersWire）。

## 验证

- **红钉转绿**：定向 `test_iframe_service_worker_messages_dispatch_to_
  iframe_container` 1P（旧版红证据：s179 账本 engine_lib_v2.log
  3633P/2F 同名红）。
- **engine lib 3635P/1F**（3636 总测；唯余 ua_display_li——渲染流在册
  非 XHR 面）。
- renderer lib **361P/0F**；browser bin tab_js_worker **39P/0F**。
- shim 装配序拼接 node --check SYNTAX_OK（113975 行）。
- forms/interactive corpus join 免跑：改动执行面（credentialed XHR
  send）于两语料不可达（无 XHR 凭证调用），无凭证 XHR 走双空门 no-op
  ——同 shim 全文已经 3636 条 lib 桥测实测（net-api XHR 家族全绿）。

## 归因与边界

- 修复以机制正确性 + 红钉红→绿证据承载（s179 账本在案旧红），未逐轮
  bisect（根因链 s179 已定谳：M2-S3 校验 × R2778 parser 链后此红显形）。
- btoa 非 Latin-1 凭证抛 InvalidCharacterError → 整块 try/catch 吞 →
  URL 仍 strip、无认证头分派（RFC 7617 默认 ISO-8859-1 域外的病态
  凭证优雅降级；UTF-8 Basic 不在 spec 域内）。
- Basic 头非 CORS-safelisted → 跨源凭证 XHR 将触发 preflight（sync
  preflight 臂既有逻辑自动生效）——spec 一致，无独立验证（语料无此形态，
  FIXME 不新记：机制为既有 preflight 逻辑的组合面）。

## 下一步候选（本流）

engine lib fetch 族已清零（唯余 ua_display_li 渲染流）；forms 全树
402F 家族分析；interactive 44F 归因复读。

## 中间产物

/tmp/s180/：engine_lib.log（首跑 tee 目标目录缺失，重跑全量在案）、
renderer_lib.log、browser_bin.log；/tmp/s180_concat.js（shim 拼接）。
