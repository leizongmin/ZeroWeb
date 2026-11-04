# M2 s275 — deferred lazy 图冻结请求语义（型 B base-url ×2 双绿，2 文件 4 刀）

日期：2026-11-04。切片号 s275（承 s274 下一步候选①：型 B base-url ×2 勘根
——parse-time base URL 冻结面）。2 文件 4 刀（engine shim 单车道，零 runner
改动）。

## 勘根

1. **探针触发链定谳**（zz-s275-probe，验后删）：patch
   `__zwSelectImageSource5277`/`__zwImgRevalidate5277`/`fetch` 后跑 base-url
   同构页——链序实证 `base changed → revalidate → select ->
   https://wpt.test/invalid-url-where-no-subresources-exist/image.png →
   fetch → 404 → EVENT onerror → unreached`。scrollIntoView 唤醒面经 sweep
   → revalidate → **select 用变更后 base 重解析** → 坏 URL 404 → onerror →
   `below_viewport_img.promise.catch(unreached_func)` Fail。spec 语义（页
   面注释原文）：「the next step in the #updating-the-img-data algorithm is
   to fetch the request that has already been set up」——唤醒面 fetch
   **defer 时点 set-up 的冻结请求**，不重选源（HTML §updating-the-image-data
   lazy 分支：current request 的 request URL = 当时 selected source）。
2. **语义调和（两断言面不冲突）**：relevant mutations（crossorigin/src/
   srcset/sizes 等）按 spec 重跑 update-the-image-data → lazy 分支重新
   set-up 请求（最新 attrs/URL）——crossorigin-change「deferred img 用最新
   crossorigin」面；而 `<base href>` 变更 / pushState 文档 URL 变更**非**
   img relevant mutation，冻结请求存活——base-url ×2 面。刀形据此把冻结
   消费限定在**视口触发路径**（scrollIntoView 旗标/sweep/drain），mutation
   路径维持重选（defer 分支重新登记 = spec 重设请求）。
3. **settle defer 通路补登记**：settle lazy 分支远/近两 defer 臂均不选源
   （s273/s274 遗留形态——静态解析 img 的初始 defer 在 settle，mutation
   通路 defer 在 runupdate）——settle defer 图无冻结记录，唤醒时同样走
   重选坏 URL。settle 臂须选源一次登记冻结（纯本地解析，不 fetch 不阻
   load）。base-url 页图走 settle defer（探针 log 首条 select 即 settle
   臂，patch 后可见）。

## 施工（2 文件 4 刀）

1. **part03 runupdate lazy 门 defer 分支登记冻结请求**：
   `_zwImgDeferredReq5277[_imKey] = { url: _imSrc, density: _imDensity }`
   （defer 时点 set-up；mutation 重跑再入本分支重新登记 = spec 重设请求
   ——最新 attrs/URL）。spec 链接与页面注释原文入注。
2. **part03 fetch 臂启动即消费冻结记录**（`_zwImgFetching5277` 登记点后
   delete）——此后该图 mutations 恢复正常 restart 语义（重选源重设）；
   在飞/已终态图由视口 helper 判据兜底。
3. **part03 新增 `__zwImgViewportRevalidate5277`**（视口触发面专用重验）
   ：终态（`_resourceStates`）skip → 在飞 skip → 冻结命中直接
   `__zwImgRunUpdate5277(sel, handle, 冻结url, 冻结density)`（不选源）→
   无冻结回落既有 `__zwImgRevalidate5277`。三调用点换装：s70
   `__zwImgScrollIntoView5277` 旗标臂 / s273 sweep / s274 drain（sweep/
   drain 既有判据保留，helper 再查兜底）。s70 注释「按最新选中源 fetch」
   表述同步修订（该面由 mutation 重设承接）。
4. **part04 settle lazy 分支顶部选源一次登记冻结**（两 defer 臂共享——
   select 纯本地解析 + `_zwImgDeferredReq5277` 登记，continue 走向不
   变）；settle 循环开头 s224/s274 代际/队列重置点补
   `globalThis._zwImgDeferredReq5277 = {}`（persistent context 跨案键
   隔离）。

## 验证

- **solo base-url ×2 双绿**（image-loading-lazy-base-url{,-2}.html 各单
  子测 Pass，unreached 消除；日志 s275-solo-base-url.log）。
- **探针全链翻绿**（base-url 同构页 + instrument）：settle 臂冻结登记
  （select 正确 base URL ×3——含 s220 记录半两次）→ base 改坏 →
  `fetch .../resources/image.png?zz-s275 status=200`（冻结 URL，非
  invalid）→ onload → 主断言 Pass。dump 断言恒假为设计（导日志面）。
- **族归一**（FILTER=image-loading-lazy，39 结果行）：REGRESSED=∅，
  FIXED = base-url ×2（Fail→Pass）；余 5 红零位移（multicol /
  different-crossorigin / below-viewport-dynamic / in-viewport-dynamic /
  multiple-times 挂账全集）（日志 s275-family.log）。
- **全树 A/B**（stash 同二进制法）：A/B 8347 全键（(state,path) 归一
  sort -u），REGRESSED=∅，FIXED = base-url ×2 主键（日志
  s275-fulltree-{a,b}.log）。
- **make test**（后台 setsid）：70 test-result，engine 3724 passed +
  5 failed——R-t2qq adopted-stylesheets 既有 5 红同集（s265–s274 账本
  记档），本切零位移；错误 101 同源（日志 s275-maketest.log）。本轮
  零 Rust 改动（.js shim 单车道），fmt/clippy 不适用（make test 并行
  clippy 臂零 error 完成）。

## 挂账更新

- 型 B **base-url ×2 收口销账**（2 文件 2 子测 Fail→Pass）。
- 承前不动：multicol 窗序反向面（解除面 = runner trickle/async 慢流时序
  ）+ multicol 上游主干状态待补查；型 E different-crossorigin（avail 元
  组面）；below/in-viewport-dynamic Chrome-only 口径；multiple-times 主
  干双红销案维持；runner 全量钉内存大户二分（s270 挂账①）；root margin
  净速分档（s271 挂账①）；dynamic import() referrerpolicy 穿参。

## 下一步候选

1. 型 E different-crossorigin 勘根（avail 元组面——list of available
   images 元组键比对，s274 候选②顺延）。
2. below/in-viewport-dynamic Chrome-only 口径勘根（s272 挂账延续——
   inflight 阻塞链调度 vs spec delay load 字面冲突面）。
3. multicol 上游主干状态补查（wpt.fyi API 恢复时——两轮网络超时未决）。
