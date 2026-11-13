# M2 s420 收口账——createPattern video 源臂 + ImageBitmap 污染旗标传播（security.pattern.fillStyle.sub 余 8F 清账）

- 日期: 2026-11-13；基线: HEAD d4d0e58a8（s419 收口），驱动: s419 挂账新记「video
  pattern ×6 + ImageBitmap ×2 新暴露面」。
- 驱动案: embedded-content/the-canvas-element/security.pattern.fillStyle.sub.html
  余 8F——forEachCanvasSource 三 video 变体（直跨源 / 同源 redirect.py 跳跨源 /
  跨源 redirect.py 跳同源，各 ×2 块 DOM canvas + OffscreenCanvas）+ unclean
  ImageBitmap ×2。

## 勘根两层

1. **video ×6——createPattern 无 video 源臂**: video 元素掉进通用 `!wire → return
   null` → pattern 为 null → fillStyle 设值无图案 → 画布不污染 →
   toDataURL/getImageData 不抛 → Fail。引擎 canvas 面无任何 video 像素表面
   （drawImage 也无 video 臂；`_zwImageBitmapSourceToWire` 无 video 分支）——
   runner 媒体面为**文件读模拟**（testharness.rs register_source 直读语料文件，
   `std::fs::read(wpt_root.join(path))`）：既无 HTTP fetch（无重定向概念）也无
   解码帧（帧像素表面为结构性缺位，FIXME 记档；drawImage(video) /
   createImageBitmap(video) 同缺不在本轮扩面）。
2. **ImageBitmap ×2——污染旗标不消费**: createImageBitmap 已捕获源 origin-clean
   （img 源按 src 判跨域 + canvas 源继承 ctx 旗标，`_zwBitmapUnclean` 写点
   part05 s247 在案），但 createPattern 位图臂 `patUnclean` 恒 false → pattern
   对象 clean → fillStyle/strokeStyle 设值不污染 → Fail。

## 修复（part05.js 两处）

- **video 源臂**（G5 img 臂后新增）: `tagName==='video'` + readyState >= 2
  （HAVE_CURRENT_DATA，无当前帧落通用 null 路径——spec 视频无帧 → 无位图）→
  以 **1×1 透明 wire**（'1:1;0,0,0,0'，无帧内容断言面、不伪造帧形状）建 host
  pattern → pattern 对象 `_zwPatUnclean = _zwCanvasSrcCrossOrigin(src 请求 URL)
  || _zwCanvasSrcCrossOrigin(数据 URL)`。数据 URL = redirect.py location 参数
  静态解析（corpus `/common/redirect.py?location=<URL>` 契约：pathname 命中
  redirect.py → query 取 location= 至 `&` → decodeURIComponent → 相对 base 解析
  绝对化）。双 URL **OR 联合**逐面命中（HTTP-redirect fetch 语义：请求 URL 经跨
  源重定向即污染 + 终跳数据 URL 跨源亦污染）——face1 直跨源请求面命中；face2
  同源 redirect.py 请求面 clean 但数据面跨源命中；face3 跨源 redirect.py 请求面
  命中（终跳同源不豁免——跨源重定向起点即污染，spec 一致）。
- **ImageBitmap 旗标传播**（位图臂一行）: `patUnclean = !!image._zwBitmapUnclean`
  （createImageBitmap 既有捕获点的消费端补全；spec：源 not origin-clean → 位图
  not origin-clean → pattern 使用面污染画布）。

## 验证

- solo 驱动文件: **14/14 全 Pass**（8F 全清 + 既有 6P 零扰动）。
- 周边 pattern 面 solo ×7 全 Pass 零扰动（create.cross / cross.cross /
  create.redirect / canvas.timing.cross / image.fillStyle.redirect /
  image.fillStyle.cross / image.strokeStyle.cross——img/canvas 源臂、重定向面、
  timing 面未受 video/bitmap 新臂影响）。
- sweep ×2 join 对 s419 双基线（\x1f pairs、(key,state) 语义）:
  - my1 vs b1: **stable=8511 state-diff=10 new=0**——8 driving F→P（恰为本轮
    修复面：video ×6 + bitmap ×2）+ select-home-end 双文件 file-key F→T（簇 D
    族在册 T↔F 漂移，红集不变）。
  - my2 vs b2: **stable=8512 state-diff=9 new=0**——8 driving F→P + optional
    文件键 F→T（簇 D）。
  - my1 vs my2: stable=8520 state-diff=1（detailed 文件键 T→F 单轮闪变，簇 D
    族）——两轮终态全同（213F+14T）。
- make test 全量: **155 腿 154 绿** + 仅 engine lib 在册红腿 **3911P/7F** 失败清单
  逐名 IDENTICAL 零位移（对 s419 收口轮同名单 diff 空：img_runtime_deferred_settle_r5275
  / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix /
  iframe_service_worker_messages / response_body_readable_stream_r2967 /
  t8k_websocket_error_nonterminal / t8k_websocket_event_flow）；clippy 腿
  （渲染流 13 crate -D warnings）零告警。过程注记：本轮链接段与本机并行构建叠加
  load 峰值 210-277（16 核；本树 workspace 链接 39 rustc/lld 进程为主、ZeroWeb-2
  /-3 各 1 例），compile ~75 分钟，测试段于链接完成后运行，无时序污染面（155 腿
  计数含 1 条 cargo 并行输出打断的 ok 行——"test result: " 与 ". 5 passed" 被同行
  Checking 行插入，实际该腿 5P/0F 通过）。

## 口径

- 在册红 s419 收口 raw 222F+13T（归一簇 D 面 221F+14T）→ 终态 raw
  **213F + 14T**（−8F 清账 video pattern ×6 + ImageBitmap ×2；+1T 系簇 D
  optional 文件键 F→T 翻回——本轮簇 D 双文件 optional=T、detailed=F，随轮
  T↔F 闪变红集恒定）——恰为 s418 官方基线 213F+15T 除账驱动文件级 T 后的账面。
- 终态账面: 213F + 14T。14T 构成: 6 form-submission-target M3 门控 + 4 links
  target_blank M3 门控 + 1 select-home-end 簇 D（本轮 .optional 呈 T 面；detailed
  呈 F 面计入 213F——簇 D 双文件随轮 T↔F 闪变，红集恒定）+ 1 button-click flake +
  1 sizes-auto F5 用户门 + 1 style-error-01 慢性（s417 分型在案）。
- 下一步: 挂账池承 s416/s417/s420（渲染流 6 面 / sizes-auto F5 用户门 / 簇 D
  select 立项征询 / A2+D 簇 / checkable parent-label 2F / video 帧像素表面
  FIXME——drawImage(video)/createImageBitmap(video) 与 2d.pattern.video.* 功能面）
  勘根首位。
