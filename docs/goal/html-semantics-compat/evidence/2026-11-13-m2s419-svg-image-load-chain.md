# M2 s419 收口账——SVG `<image>` 载入链接通（插入臂 + createPattern SVG 源面）+ security.pattern.fillStyle.sub 文件级 T 解挂

- 日期: 2026-11-13；基线: HEAD 55aad5b37（s418 收口），驱动: s417 挂账池新记「SVG `<image>` 载入链（canvas 13-hang 面）勘根首位」。
- 驱动案: embedded-content/the-canvas-element/security.pattern.fillStyle.sub.html——forEachCanvasSource 7 factory × 2 块（DOM canvas + OffscreenCanvas）= 14 子测；factory #2（cross-origin SVGImageElement：createElementNS(SVG,'image') + externalResourcesRequired + xlink:href + appendChild → onload resolve）promise 永不 settle → promise_test 串行队列其余 13 子测永不启动（NotRun 不成行）→ 文件级 Timeout 在册。

## 勘根三层

1. **载入链缺失**（挂点）: engine shim 无 SVG `<image>` 插入触发载入面——SVGImageElement 接口环/decode()（s69/part86）与 createPattern 错误臂（R34xx errSrc href 读）均存在，但插入后既不 fetch 也不派 load/error。HTML img 的载入由 src IDL setter/update-the-image-data 核驱动（SVG image 无 src 语义），引擎无逐元素插入钩子先例 → 复用 iframe/object/embed 插入臂机（part04 appendChild/append 两路 walk + `_zwStartConnected*`，s405 共享单次 `_realTag` 读）。
2. **createPattern 错误臂误判**（第二层）: SVG image 载入后 createPattern 仍返 null——host `__zw_get_image_size`（webview.rs）**snapshot-only**（无运行时 fetch 兜底），SVG image 无 naturalWidth（归一 0）+ 运行时键快照恒 miss → `nw<=0 && !errDims` 错误臂提前 return null，G5 wire 臂从未到达。
3. **no-CORS 通道缺失**（第三层，探针定谳）: 臂初版裸 `fetch(abs)` 跨源被拒（s153 通道旗标协议）——img 运行核以 `_zwImgNoCorsChannelDepth` 包裹 fetch 调用开 no-cors 通道（part03:3792 形态），SVG image 无 crossorigin 属性恒走该通道。

## 修复（三件套）

- **插入臂**（part01.js，embed 臂后新增 `_zwStartConnectedSvgImage`）: tag IMAGE + `_nsHandles[handle].namespace === SVG-ns` 双门（HTML `<image>` 是 img 历史别名——createElementNS 产物经 part06 登记表；parser 产物不登记，静态面同 FIXME(object-static-events) 族记档）→ href 优先/xlink:href 回落 → `_defer` 异步 fetch（**同闭包 `_zwImgNoCorsChannelDepth++` 包裹**）→ blob → createImageBitmap → settle 写 available-images（同 img 键：width/height/finalUrl 重定向记账/xo:'unset'）→ `__zwImgMakeEvent5277` 派 load；失败臂全链 error。externalResourcesRequired 忽略（SVG2 已废弃）。spec SVG2 §embedded https://www.w3.org/TR/SVG2/embedded.html#ImageElement 。
- **part04 两路 walk 接线**: appendChild 循环（s399/s405 同排后）+ append（`_appendVariadic` 路）各加一行同排调用（复用 tagHint，零新增宿主往返）。
- **createPattern SVG 源面**（part05.js）: ①错误臂加 `siAvOk` 已载入门——SVG image 的 href 解析键命中 available-images（width>0，插入臂 settle 写点）即跨过错误臂（svgimage.nonexistent 抛面零变化：插入臂 error 终态无 avail 条目）；②G5 臂 imgSrc2 提前计算 + isSvgImg 时 href/xlink:href 回落读 + 门 `naturalWidth>0 || (isSvgImg && imgSrc2)` 扩展 → wire 经既有 host `__zw_get_image_wire`（快照 miss 现场抓取+解码，webview.rs R56h 面）；清洁度 `_zwCanvasSrcCrossOrigin(imgSrc2)` 同 img 键（finalUrl 消费）。

## 探针定谳（zzz-s419-probe/probe2，已删）

- 教训在案: 初版探针 p2/p3 断言空心（t.done 无条件，「Pass」无证据力）——载入链有真断言（unreached_func）但 pattern 面全靠 DIAG dump；dump 未定时派发（explicit_done:false 自动收口早于 setTimeout）→ 不可采信。重写为真断言镜像 + DIAG 强制 dump。
- DIAG 序列（修复后）: `onload → avail:{width:100,height:50,finalUrl:http://www1.wpt.test:8000/images/red.png,xo:'unset'} → cp:obj,unclean=true → td:SecurityError → gi:SecurityError`——载入、settle、pattern 创建、跨源污染、双读拒绝全链闭合。
- 初版裸 fetch 探针: 跨源臂 error（unreached_func 触发）同源臂过 → no-CORS 通道缺失定谳。

## 验证

- solo 驱动文件: 文件级 Timeout 消除，14 子测全部成行——**6P/8F**。factory #2 双子测（DOM canvas + OffscreenCanvas）全 Pass。
- 余 8F 分型（新暴露面，非本轮驱动，挂账池新记）: cross-origin/redirected video ×6（video 源 → createPattern unclean 传播面）+ unclean ImageBitmap ×2（createImageBitmap 脏源旗标传播面）。
- sweep ×2 join 对 s418 双基线（(key,state) 语义、\x1f 分隔；过程注记：sweep.sh
  模板残留 s417 输出路径，首轮写到 /tmp/s417/ 且覆盖其旧 sweep 原始 log（已消费
  可再生工件，pairs 基线文件未受损；本轮 log 已抢救回 /tmp/s419/，脚本已改路径））:
  - my1 vs b1: **stable=8397 state-diff=0 new=124 gone=3**——驱动案文件级 Timeout
    消除 + 恰 14 new（6P/8F，SVGImageElement ×2 全 Pass）；另 new=select-event
    (102P) + focus-dynamic (8P) 两文件 b1 侧 file-T flake 残留本轮全绿（s418
    账本四轮矩阵在册负载 flake 族，同族漂移非新红）；gone=3 个文件级 T 键（驱动 +
    上两文件）。
  - my2 vs b2: **stable=8507 state-diff=1 new=13 gone=1**——驱动案 new=13 + stable=1
    （b2 在 Timeout 前已有 cross-origin HTMLImageElement pattern 子测 Pass 成行，
    本轮同态 Pass 计 stable）+ gone=1（文件级 T 键）；state-diff ×1 =
    select-home-end-pagedown-pageup.optional.html 文件键 T→F——簇 D 族在册 T↔F
    漂移（s415/s417/s418 在案，红集不变）。
  - **确定性**: my1 vs my2 直接 join stable=8521 state-diff=0 new=0 gone=0，两轮
    完全同态（222F + 8286P + 13T）；驱动文件 14 行两轮逐行一致。
- make test 全量: **155 腿 154 绿** + 仅 engine lib 在册红腿 **3911P/7F** 失败清单
  逐名 IDENTICAL 零位移（对 s418 收口轮同名单 diff 空：img_runtime_deferred_settle_r5275
  / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix /
  iframe_service_worker_messages / response_body_readable_stream_r2967 /
  t8k_websocket_error_nonterminal / t8k_websocket_event_flow）；clippy 腿
  （渲染流 13 crate -D warnings）零告警。过程注记：本轮 make test 链接段与本机
  并行构建叠加 load 峰值 240-330（16 核；自身 workspace 链接 16 rust-lld/cc 为主，
  ZeroWeb-2 构建次之，另有 ZeroSeed/.dev registry 无关构建），compile 59m20s，
  测试段于链接完成后运行，无时序污染面。

## 口径

- **基线勘误**: s418 账本「在册红 212F+15T」与 pairs 文件真值差 ±1——b2 实数
  **213F + 15T**（b1 为 214F+16T，含 select-home-end F 面 + select-event /
  focus-dynamic 两 file-T flake 残留；b1↔b2 三处差均为在册漂移族）。本轮起官方
  基线以 pairs 真值为准（213F+15T）。
- 在册红 213F+15T → 终态 raw **222F + 13T**（my1≡my2）；分解：+8F 新记余面子测
  （video pattern ×6 + ImageBitmap ×2，NotRun 阻塞解除后首次成行）+1F/−1T 簇 D
  select-home-end 文件键本轮 F 面（家族 T↔F 漂移，红集不变）−1T 除账
  security.pattern.fillStyle.sub 文件级。归一簇 D 漂移面（按 T 计）后本轮净变化
  = **221F + 14T**。
- 下一步: 挂账池承 s416/s417/s419（渲染流 6 面 / sizes-auto F5 / 簇 D 立项征询 /
  A2+D 簇 / checkable parent-label 2F / video pattern ×6 + ImageBitmap ×2 新面）
  勘根首位。
