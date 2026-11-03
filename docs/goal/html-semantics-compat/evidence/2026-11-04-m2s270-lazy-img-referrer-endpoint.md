# M2 s270 — referrer-checker-img.py 图片端点仿真（image-loading-lazy-referrerpolicy-change 1F→P 收口）

日期：2026-11-04。切片号 s270（承 s269 下一步候选①；s268 候选②
承接，三轮连承首勘）。1 文件 1 刀 1 钉 + 1 文件级常量（runner
单车道，零引擎零 shim 改动）。

## 勘根

1. **入场候选核账**：s269 账本候选②（modulepreload referrer 族）
   实态已绿——solo FILTER=modulepreload 3 文件 15 子测全 Pass
   （s124 echo-referrer.py 端点臂 + link modulepreload 面在册）；
   第 4 文件 modulepreload-referrer-check.html 未收录根因 =
   内容含 `<iframe` 命中 `html_semantics_case_skipped` iframe
   依赖面规则（runner 单文档通道外，归 navigation-compat M3 多
   frame 管线门控，the-iframe-element 排除先例同口径）——**候选②
   销案**（过时承接：s269 挂账⑤记 "echo-referrer.py 端点 +
   link modulepreload 面独立候选"，实为 s124 已收口余项 + 非本
   流施工面）。候选收敛到①。
2. **目标页结构**：`<img loading="lazy" referrerpolicy="unsafe-url"
   src="resources/referrer-checker-img.py?expected_referrer=">` 视口
   外（前 1000vh div）；window load 后 `referrerPolicy = "no-referrer"`
   IDL set + `scrollIntoView()`；断言 deferred 装载用**最新**
   referrerpolicy（无 Referer 头）→ onload resolve。失败签名
   `assert_unreached: The image request should successfully load`
   = onerror 到场 = 装载已触发但取回失败。
3. **引擎面核（三层全通，非引擎缺陷）**：
   - relevant mutations：s70 `__zwImgAttrMutated5277`
     referrerpolicy/crossorigin 臂（状态枚举比对 → 重验）已在；
   - lazy 视口门：s70 scrollIntoView 旗标（`_zw5277ViewportFlag`）
     + `_imHadPrev` 门（deferred 不启动、视口触发启动）已在；
   - 取回链：`__zwImgRunUpdate5277` → 页面 `fetch(_imSrc)`（s247
     no-cors 通道）→ `resp.ok` → blob → createImageBitmap →
     settle loaded → dispatch load → onload 属性处理器全通。
   onerror 到场即证前三层工作，缺口仅在取回目标。
4. **端点缺席定谳**：语料 the-img-element/resources/ 目录整体不
   在语料包（上游 wptserve handler 不随语料拉取，s116/s124/s268
   同形态）→ fetch handler 无臂 → 404 → error。SSH sparse clone
   上游原件定谳端点语义：`referrer = headers.get("referer","")`
   vs `expected_referrer = GET.first("expected_referrer","")`，
   匹配 → 200 image/png（image.png 字节）；不匹配 → 404 "Not
   found"（两分支 CT 恒 image/png）。
5. **可观察等价裁定**：runner fetcher 无 Referer 头建模（s116
   module 臂同简化面——真实 HTTP 头不合成）→ img 取回 actual
   Referer 恒 ""；语料唯一消费者（本页）期望值恒空串（"无
   Referer"）→ 恒匹配。页 spec 点（deferred 装载用最新策略）的
   可观察差异仅在 Referer 头有无比对，策略值在无头通道不可观察
   ——单刀端点臂收口，零引擎改动（ FIXME 真实头建模注记在臂）。

## 施工（1 文件 1 刀 1 钉 1 常量，testharness.rs）

1. fetch handler `referrer-checker-img.py` 臂（css.py s151 臂
   后）：`expected_referrer` 空或缺参（上游 GET.first 默认）→
   200 + CT image/png + `ZW_ONE_PX_PNG`（69 字节 1x1 RGB，自包含
   最小合法图——上游 image.png 11KB 不入语料，页断言仅 onload
   到场无尺寸面）；非空期望 → 404 + "Not found"（CT 两分支
   image/png 逐字对齐上游）。
2. 文件级常量 `ZW_ONE_PX_PNG`（s270 注记）。
3. 钉 `referrer_checker_img_endpoint_sims_s270`（3 断言：空期望
   200+PNG magic+CT / 缺参同 200 / percent-encoded 非空期望 404）。

## 验证

- **钉**：新钉绿；runner bin 全量 **286P/0F/76I**（s268 基线
  285 + 1 新钉；guard 4G 阈值下单进程超限——并流钉累积既有问题，
  6G/12G 放宽跑取基线态，挂账注记①）。
- **solo**：目标页 **Fail→Pass**（日志 evidence/logs/s270-solo.log）。
- **族 A/B**（FILTER=image-loading-lazy，stash 同二进制）：16P +
  既有红群（in-scroller ×5 / base-url ×2 / below-viewport-dynamic
  / different-crossorigin 等）双侧零位移，唯一差异 = 目标页
  F→P（日志 s270-family-{a,b}.log）。
- **全树 A/B**（stash 同二进制法，test-guard + RAF）：
  **7845P/431F/68T/11NR → 7846P/430F/68T/11NR**——(key,state) 对
  8352/8352 双侧同集，**唯一差异 = 目标页 F→P，REGRESSED=∅**（
  第十个连续完美 A/B；s269→s270 间键对 8349→8352 为语料收录漂移
  ——API 列目录波动恢复新收录 3 键，双侧同态不在归因范围，注记
  ③）（日志 s270-fulltree-{a,b}.log）。
- **make test**（test-guard）：engine **3724 passed + 5 failed**
  ——5 红为 R-t2qq adopted-stylesheets 既有（s265 账本同集），
  3722→3724 为渲染流并行提交加钉（R5410 流），本切不涉 engine
  零入侵。
- fmt/clippy：`cargo fmt --all` 干净；workspace guarded-clippy
  零告警。

## 挂账更新

- image-loading-lazy-referrerpolicy-change **销案**（s268 候选②/
  s269 候选① 三轮连承收口）。
- modulepreload referrer 族挂账⑤**销案**（勘根①定性：3 绿 +
  1 排除面归 navigation M3）。
- 新记录：
  ① **runner bin 全量钉单进程内存超 4G guard 阈值**（cargo test
  单进程跑全部钉，并流钉累积；--per-proc-mem 4 杀树，6G/12G 放
  宽可跑 286P）——后续轮次二分定位内存大户钉，guard 阈值放宽仅
  作基线态取证非常态口径；
  ② **img 取回 Referer 头真值建模**（fetcher 无头通道下
  referrerpolicy 策略值不可观察，本页以无头等价承载；若语料扩
  出非空期望钉面需先建通道 FIXME 注记在臂）；
  ③ 语料收录漂移 +3 键注记（双侧同态，非本切范围）。
- 承前不动：dynamic import() 元素级 referrerpolicy 穿参（零
  corpus face 无验证路径）；module map 缺席（s256/s267 注记）；
  lazy 族既有红群（in-scroller ×5 / base-url ×2 / below-viewport
  -dynamic / different-crossorigin 等——视口相交模型面，候选待
  勘根分型）；R-t2qq 5 红（归该流自查）。

## 下一步候选

1. lazy 族既有红群勘根分型（image-loading-lazy-in-scroller ×5
   + base-url ×2 等——视口相交/滚动容器/URL 基模型面，shim
   车道；s70 视口旗标近似为既有口径参照）。
2. dynamic import() 元素级 referrerpolicy 穿参（s268 挂账——零
   corpus face，随语料扩展补，低优先）。
3. runner 全量钉内存大户二分定位（挂账①，基础设施轮次）。
