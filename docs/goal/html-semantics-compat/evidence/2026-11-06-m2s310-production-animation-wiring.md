# M2 s310 — 生产通道动画装载接线：webview 8 臂动画感知装载 + canvas G5 快照首帧 pin（img 动画 GIF/WebP 携带时间线，帧推进由 paint 侧 get() 墙钟承接）

日期：2026-11-06。切片号 s310（承 s309 下一步候选①，即 s307 ③ 生产通道接线，施工型切片）。

## 勘根/边界确认

- **现状**：s307 收口仅 reftest 通道（runner `build_image_cache` 装载臂）。生产通道
  4 网络解码+插缓存位（async_load `poll_lazy_images`/`poll_images`、webview
  `fetch_image_subresources`/`apply_runtime_image_syncs`）+ 4 data: 臂均
  `decode_image` + `insert_with_key` 单帧静态——动画 GIF 生产显示首帧。
- **隔离边界事实**：GIF 不在 [`is_raster_image_bytes`] 集（PNG/JPEG/WebP）→
  恒进程内解码，代理模式也多帧安全；PNG/JPEG/WebP 隔离模式经
  `ImageDecodeResultParams` 单帧 RGBA IPC——动画时间线不可跨进程，动画 WebP
  隔离模式维持首帧（`FIXME(protocol)`：IPC 帧表扩展后升级，不内联回落，
  IF-006 纪律同 `decode_image` spawn 失败面）。
- **canvas drawImage 首帧契约前置 pin**：img 源画首帧（R34xx drawing-images，
  2d.drawImage.animated.gif）。生产 G5 线 `__zw_get_image_wire` 经
  `snapshot_entries` 取像素，而 s307 `get()` 推进臂会把 `entry.data` 换写为
  活跃帧——接线后快照将随渲染返回活跃帧，违反首帧契约。s310 前置修：
  `snapshot_entries` 对动画条目固定 `frames[0]`（img 元素活跃帧与 canvas
  首帧语义在此分叉，各有单测锚定）。
- **dedup 重插冻结缺陷（接线必修前置）**：两同步臂（`fetch_image_subresources`/
  `apply_runtime_image_syncs`）缓存命中经 `get()` 后无条件 `insert_with_key`
  尾部重插——`animation: None` 覆写动画条目并冻结活跃帧。重插本就冗余
  （`get` 已刷 `ref_count`/`last_access_gen` LRU 面），s310 改 `from_cache`
  置位免重插（静态面行为不变：get 已刷新 LRU 状态）。
- **帧推进承接面**：生产 paint 按 `ImageCache::get(image_key)` 取像素（GPU
  renderer `prepare_image_resources` / CPU 路径）→ 动画条目按条目墙钟推进
  换帧。重绘为事件驱动，无连续动画重绘驱动——2026-08-08「Renderer 等待图片
  期间重复重绘」learning 为反面教材，本切片不引入驱动；帧随既有重绘演进。

## 施工

**render-foundation（crates/render-foundation/src/image_cache.rs）**：

1. `snapshot_entries` 动画条目首帧 pin + 文档注释（img/canvas 语义分叉说明）
   + 单测（虚拟时钟推进换写 data 后快照仍帧 0）。

**webview（crates/webview/src/image_decoder.rs）**：

2. `decode_image_with_animation`——分派与 `decode_image` 逐臂一致（代理+栅格
   → 单帧 `(img, None)`；否则 `decode_image_animation` 多帧）。
3. `decode_data_uri_animation`——data: 通道恒进程内（`decode_image` 契约），
   字节侧动画分派直解。
4. `insert_image_decoded` 装载 helper（Some→`insert_with_animation`，None→
   `insert_with_key` 零行为面外移）。
5. 单测 4：内联 GIF/WebP 动画时间线、隔离模式 GIF 仍内联动画、隔离+栅格
   （真实 image-decoder 子进程）代理首帧/spawn 失败错误面不回落、helper
   装载行为（动画条目推进 + 静态条目零推进面）。
6. 测试基建修复（本切片实测缺陷面）：`set_process_isolation` 触碰进程级
   旗标/env，cargo test 默认并行，既有「测试单线程」前提不成立——实测并行
   误导入代理路径致动画断言假红；加 `ISOLATION_TEST_LOCK` 串行化全部触碰方
   （3 既有 + 5 新增测试）。

**接线 8 臂（async_load.rs ×4 + webview.rs ×4）**：

7. async_load：img data:（R1987 臂改 `decode_data_uri_animation` 直解，免
   单帧+动画双解）、CSS url(data:)、`poll_lazy_images` 网络、`poll_images`
   网络。
8. webview：`fetch_image_subresources` data:/网络、`apply_runtime_image_syncs`
   data:/网络——动画条目 `insert_with_animation`；两同步臂 `from_cache`
   免重插（见上）。
9. 顺带清理：本接线致 `decode_data_uri`/`decode_image` 在 async_load/webview
   失去最后调用方，孤立 import 移除（本次修改造成，准则 4）。

## 语义边界（记录）

- 隔离模式动画 WebP 首帧静态（`FIXME(protocol)` IPC 帧表扩展挂账）。
- `duplicate_for_snapshot`（paint IPC）不携带动画（`animation: None`）——
  多进程 paint 快照为装载/快照时刻帧静态；跨进程 Instant 时间线与条目生命
  周期面未接线，挂账候选。
- 生产无连续动画重绘驱动：动画帧仅在重绘发生时演进（事件驱动）；驱动属
  渲染流动画时间线统一 RFC 面（draft 待批，本流不越界）。

## 验证

- 单测（test-guard 包裹）：render-foundation `--lib` 全量 **746/746**（含
  s310 新 1 + s307 族 5）；webview `--lib` 全量 **844/844**（含 s310 新 4 +
  image_decoder 模块 8/8）。
- clippy：两 crate default 特性零告警；`--all-features` 6 告警均在
  `video_registry.rs`/`video_registry_host.rs`（既有 video feature 门面，
  与本切片无关，只提及不修）。
- solo reftest（reftest 通道零回归锚）：animated-image-update ✓0.00%（真通过-
  可信）+ animated-webp-update ✓0.00%——[evidence/logs/s310-solo-animated.log](logs/s310-solo-animated.log)。
- make test（guarded-test）：80 腿 test result: ok 全绿；workspace 腿止于
  engine lib **3772P/6F——已知 5 键（CSSOM cssText 面）与 s307 基线逐键
  相同 + r5296_single_line_inflated 套件级 flake（s307 记录在案本轮首触发），
  solo 3× 全绿裁定**；key 级 diff vs s307-make-test.log 仅 +r5296 一键，
  **零新增失败**（见控制面状态行）。
- 行为证据链：修前生产动画 GIF 首帧静态 = s307 账本 ③ 记录 + 本次接线前
  代码（`decode_image` 单帧）即基线；修后动画条目装载 + 推进由单测锚定
  （`insert_image_decoded_wires_animation_s310`：虚拟时钟 150ms → 帧 1 绿）。

## 证据

日志（docs/goal/html-semantics-compat/evidence/logs/）：
- s310-solo-animated.log（双动画页 solo ✓0.00% ×2）
- s310-make-test.log（guarded make test 全腿）

代码锚：crates/render-foundation/src/image_cache.rs（`snapshot_entries`
首帧 pin）；crates/webview/src/image_decoder.rs（`decode_image_with_animation`
+ `decode_data_uri_animation` + `insert_image_decoded` + ISOLATION_TEST_LOCK）；
crates/webview/src/async_load.rs（4 臂）；crates/webview/src/webview.rs
（4 臂 + from_cache 免重插）。

注记：webview.rs（10001 行）/async_load.rs（2980 行）/image_cache.rs（4754 行）
均早已超 2000 行准则线（既有面，本切片增量 ~百行不触拆分，只提及）。

## 下一步候选

1. 小簇排队其余（textarea 4/meter 4/input 3/link 3/embed 3/object 2/
   option 1/iframe 1/wbr 1）逐簇勘根；br 1 勘根前先 solo 复核 editing-compat
   br 刀（fe760dc57）外溢现势。
2. 渲染流移交面跟进（s308 bidi / s309 R109 嵌套 block-in-inline）——待渲染流
   控制面状态，本流不越界。
3. （挂账）生产动画重绘驱动 / 隔离通道帧表 IPC 扩展 / paint IPC duplicate
   动画携带——渲染流 + 协议面协同候选，非本流域内施工。
