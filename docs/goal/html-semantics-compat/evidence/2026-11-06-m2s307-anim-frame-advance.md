# M2 s307 — animated GIF/WebP 帧推进：animated-image-update 双页 ✗2.08%→✓0.00%（多帧解码+虚拟动画时钟+GIF/WebP 装载三层收口）

日期：2026-11-06。切片号 s307（承 s306 下一步候选①）。

## 勘根

**页面机制**（animated-image-update.tentative.html / animated-webp-update.tentative.html
同刀）：`<img src="resources/animated.gif|webp">` + reftest-wait；load 后
rAF→setTimeout(150ms)→takeScreenshot。资产两帧同构：红 100ms → 绿 10s
（GIF GCE delay=10cs/1000cs；WebP ANMF duration=100/10000ms），150ms 截图
时刻活跃帧 = 绿；ref = 静态 green.png。HTML spec：img 渲染动画当前帧
（https://html.spec.whatwg.org/multipage/images.html#the-img-element）。

**三层根因**（s306-full-tree 双页 ✗2.08%=10000/480000 整幅缺失）：

1. **harness 装载臂缺口（直接根因）**：reftest 通道 build_image_cache
   只有 PNG/JPEG/SVG 装载——**GIF/WebP 文件从不入缓存** → img 空白
   （两页同差幅 2.08% 的由来；并非「首帧红 vs ref 绿」）。
2. **解码单帧无时间线**：decode_gif_bytes 只解首帧（R34xx drawImage 语义，
   首帧红）；decode_webp_bytes read_image=首帧；无帧时序表。
3. **无动画时钟**：reftest 通道 shim setTimeout 经 microtask 即时消费
   （150ms 等待不产生真实时间）——即使有多帧解码，墙钟推进也越不过 100ms
   首帧边界。真浏览器语义 = 截图时刻动画已推进 timer 等待时长。

**爆炸半径盘点**：语料引 .gif/.webp 图片面仅 7 文件——自动化通道恰 2 目标页
（background-animated-001 无 ref 不入通道、cursor-023/024 cursor 面不上
帧缓冲、2 manual 页不入通道）；canvas drawImage 首帧语义不经 ImageCache
（桥接期一次性消费，R34xx 面零接触）。

## 施工

**render-foundation（crates/render-foundation/src/image_cache.rs）**：

1. `AnimationData`（帧快照 + start_ms 时间线 + total_ms 回绕 + 条目墙钟零点
   + active 游标）+ `frame_index_at`（rem_euclid 回绕，末帧闭到 total）。
2. `decode_gif_animation_bytes`：全块流帧循环（GCE 透明/延迟/处置逐帧、
   LZW 逐帧合成快照）；首帧路径与旧单帧实现逐字节同构；首帧后结构错误降级
   「保留已解帧」；帧数上限 4096 病态内存防线。FIXME 三项：处置码 3 降级
   leave、交错存储按序解码、NETSCAPE 循环计数忽略恒回绕。
3. `decode_webp_animation_bytes`：image-webp `read_frame` 顺序迭代（解码器
   内部完成 dispose+alpha blending，返回帧时长）；0 时长钳 100ms；静态
   WebP 委托既有路径 anim=None。
4. `decode_image_animation` 分派（GIF/animated WebP 多帧，PNG/JPEG/SVG 恒
   None）；`decode_gif_bytes` 收口为帧 0 薄委托（canvas R34xx 契约不变，
   content_hash 顺带由 0 哨兵转真值）。
5. `CacheEntry.animation` 字段（与条目同生命周期，GC 免同步）+
   `insert_with_animation`；`get()` 推进臂：虚拟时钟优先、缺省条目墙钟，
   活跃帧变化才换写 `Arc`（R3254-M2 像素不可变不变式保持，GPU 纹理缓存按
   content_hash 自然换新）。
6. `set_anim_virtual_elapsed` 虚拟动画时钟 thread-local + RAII
   AnimClockGuard（adopted-css 同款防 rayon 跨案泄漏）。

**runner（tests/wpt-runner）**：

7. resources.rs `build_image_cache` 增 GIF/WebP 装载臂（.gif/.webp 扩展名
   门 + `load_anim_file` 魔数分派；动画条目 `insert_with_animation`，单帧
   `insert_with_key` 零行为面外移；ImageKey 键面零变化）。
8. reftest_scripts.rs `apply_scripted_dom_mutations` 尾读 shim
   `__zw_anim_clock`（execute_json，无 timer 页恒 0）入返回值六元组；
   reftest.rs 两渲染入口 set_anim_virtual_elapsed RAII 注入。

**shim（crates/engine/src/js_dom_shim/part01.js）**：

9. setTimeout fallback 臂（无 host timer，全通道现行火路）记账
   `__zw_anim_clock += min(clamped_delay, 60000)`——延迟被即时消费 = 虚拟
   时间前进（真浏览器 timer 到期后截图语义）；无消费者通道仅累加一个数，
   零行为外移。

**单测 4 新增**（testdata 内嵌 corpus 双资产）：GIF 帧序/时序/回绕、WebP
帧序/时序、静态 passthrough、缓存虚拟时钟推进换帧（含 content_hash 随帧
换新 + 回绕回帧 0）。

## 验证

- 单测：`cargo test -p zero-render-foundation --lib s307` 4/4 绿（首跑即绿）。
- solo：animated-image-update ✗2.08%→**✓0.00%（0.00% 差幅，真通过-可信
  100%）**（fedf2e6b）；animated-webp-update 同刀 **✓0.00%**（e31662b4）
  ——修后双页首跑即绿；修前失败证据 = s306-full-tree（07bceb80）双页
  ✗2.08%。
- 全树：**17012 案 = 15986P/1026F**（10252 可信/567 可疑/5167 近似/1026
  不一致）vs s306 key 级 join diff——**F→P×10**：animated-image-update +
  animated-webp-update（目标修复）+ end-001/end-002/middle-002/tao-127/
  tao-128/tao-153/intrinsic-size-008（s306 已裁 pool 同族弹回）+
  block-ellipsis-018（新面，solo 3× ✓0.00% 裁定，与图片域无机制关联——
  flake 弹跳）；**P→F×2**：tao-150/tao-154（pool 族，solo 3× ✓0.00%
  全数直接裁定）。**净真实 +2 F→P 零净 P→F**；1034+10−2=1026 精确闭合。
- clippy 全仓零告警；rustfmt 过。
- make test（guarded-test）：80 腿 test result: ok 全绿（含 render-foundation
  4 新单测腿）；workspace 腿止于 engine lib **3773P/5F 已知集逐键相同**
  （5 已知 CSSOM cssText 红；r5296 套件级 flake 本轮未触发）——s307 变更
  **零新增失败**。

## 证据

日志（evidence/logs/，md5）：
- s307-solo-animated-gif.log fedf2e6bb7cf745b6ed488022bf1abdf（修后 solo
  ✓0.00%）；s307-solo-animated-webp.log e31662b431c42cbd9e3b1e88553e19e8
- s307-full-tree.log 5df3a83127a22374aa32d385bb19ac3c（全树 17012 案 +
  key join 源）；s306-full-tree.log 07bceb80（修前基线双页 ✗2.08%）
- s307-adjud-3keys.log 6ae93818e5bdb9495389fe76998f4b08（block-ellipsis-018
  /tao-150/tao-154 solo 3× 全绿裁定）
- s307-make-test.log 94bf3fa24c3685bd0b527624e3d1fb34（10393 行，80 腿 ok
  +workspace 腿止于 engine lib 3773P/5F 已知集；guarded-test）

代码锚：crates/render-foundation/src/image_cache.rs（AnimationData+
decode_gif/webp_animation_bytes+decode_image_animation+get 推进臂+
set_anim_virtual_elapsed）；tests/wpt-runner/src/reftest/resources.rs
（GIF/WebP 装载臂）；reftest.rs（双入口虚钟注入）；reftest_scripts.rs
（六元组尾读）；js_dom_shim/part01.js（setTimeout 记账钩子）。

## 下一步候选

1. the-bdi-element 13 红页簇归属勘根（bidi 隔离，疑渲染域移交）。
2. 小簇排队（ol 6/textarea 4/meter 4/input 3/link 3/embed 3/object 2/
   option 1/iframe 1/br 1/wbr 1）。
3. 生产通道动画装载接线（engine 网络路径 decode_image_animation——本轮
   仅 reftest 通道；生产 img 动画 GIF 维持首帧静态，零回归面）。
