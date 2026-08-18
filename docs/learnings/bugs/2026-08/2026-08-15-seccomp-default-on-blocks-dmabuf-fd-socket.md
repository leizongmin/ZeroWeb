---
date: 2026-08-15
modules: apps/compositor/src/seccomp_linux.rs, crates/render-foundation/src/gpu/renderer/mod.rs, apps/browser/src/app_platform.rs, crates/webview/src/image_decoder.rs
---

# 184bbffc6 默认翻转 Linux 侧验证：四个修复与三个已知缺口（2026-08-15）

- 背景：commit `184bbffc6`（10 开关默认翻转）+ 后续 `3411df096`/`a42cec0d3` 在 WSL2（X11、lavapipe 软件 Vulkan、无 /dev/dri）接力验证

## 修复 1：seccomp 默认开阻断 dma-buf fd 交付通道（Linux 回归）

- 现象：`compositor_gpu_texture_export_dma_buf_round_trips` 等测试
  `fd socket connect 超时: /dev/shm/zeroweb-fd-*`；Windows 无法发现（Linux-only 路径）。
- 根因：`fd_socket_linux::publish_fd` 需 `socket(AF_UNIX)`/`bind`/`listen`/`accept4`/`sendmsg`，
  全在 seccomp 黑名单；GPU-aware 过滤器此前从未真实执行过。
- 修复：GPU 过滤器 AF_UNIX 域门控（`seccomp_data.args[0]`）——AF_UNIX 放行，
  inet/inet6/netlink 仍 EPERM；`bind`/`listen`/`accept*`/`sendmsg`/`recvmsg` GPU 模式放行
  （作用面被域门控限于 Unix socket）；`connect`/`execve` 维持阻断。
- 配套 `gpu_filter_execution_allows_unix_and_blocks_inet`：隔离子进程真实安装过滤器
  后探测 AF_UNIX 可用、AF_INET EPERM——BPF 必须真实执行验证，结构断言发现不了
  「黑名单误拦自家通道」。
- 经验：经典 BPF 仅前向跳转，非 socket 路径须用无条件 `jmp` 跳过参数检查段。

## 修复 2：GPU 光栅图片图元越界 panic（progressive paint × 异步解码）

- 现象：basic-function parity 场景（含 PNG 图片页）compositor 主线程 panic
  `img_resources[i..i+1]: range end index 1 out of range for slice of length 0`，
  compositor 进程死亡、整窗回退 legacy。
- 根因：`prepare_image_resources` 对 image_cache 未命中的图元直接 `continue`——
  资源列表与 `DrawOp::Image(i)` 索引错位（渐进绘制下图元先于解码 payload 到达；
  image-decoder 进程默认开后窗口更常见）。
- 修复：资源列表改与图元 **1:1 的 `Option` 占位**，绘制时跳过 `None`；
  未就绪图元等下一帧 payload。
- 经验：GPU 路径「未支持返回 false 回退 CPU」的契约不容 panic 破坏；
  混合就绪/未就绪图元的回归测试见
  `test_gpu_full_scene_skips_image_primitive_without_cached_data`。

## 修复 3：owned-present 首帧白屏（present 往返未完成时跳过本地合成）

- 现象：compositor 模式产品冒烟 `chrome region has too few colors: 1`——首帧整窗纯白。
- 根因：`ZW_COMPOSITOR_OWNED_PRESENT`+`ZW_COMPOSITOR_PRESENT` 默认开且 CPU 渲染时，
  `skip_local_composite_for_owned_present` 立即跳过本地合成等 compositor present 帧，
  而 present 请求-响应是异步的，首个稳定帧采集时像素未到 → 白屏。
- 修复：跳过条件追加「present 像素已就绪（尺寸匹配）」；未就绪前本地合成，
  就绪后无感切换（`owned_present_waits_for_present_pixels_before_skipping_local_composite`）。

## 修复 4：clippy 1.95 新 lint（collapsible_match）

`zero-style-system/src/lib.rs` 与 `zero-engine/src/js_dom_bridge.rs` 共 3 处 match 内
单语句 `if` 折叠为 match guard。教训：本地工具链升级后 clippy 新 lint 会让 CI 突然红，
`make test`（quickjs clippy 矩阵并行）应常跑。

## 已知缺口（本轮归因、未修，属结构性工程）

1. **多进程滚动视觉架构 gap**（Windows handoff 已列）：compositor 模式 browser 不重建
   primitives，页面内容唯一来源是回读位图（一帧高），滚动超一帧高度即空白。
   `a42cec0d3` 已回退 present bake 并把 SCROLL_TRANSFORM 翻回 disabled，present 捷径在
   滚动非零时跳过。终态需 compositor 平移光栅化或 renderer 滚动感知重绘。
   本机验证：滚轮小滚动跟手（54/64 采样变化）✓；gui-smoke 的 zoom 步骤同族缺口
   （基线同样失败，非本轮回归）。
2. **gpu-dmabuf 冒烟的自相矛盾**：dmabuf 导入成功 ⇒ 采纳帧 `gpu_direct`（无本地
   RGBA 位图）⇒ 冒烟的 headless GPU 捕获（独立渲染器实例，无 dmabuf 导入纹理）无页面
   像素 ⇒ `page region` 断言必挂。该模式（a48f58a1e 引入）在 dmabuf 真正生效的机器上
   从未绿过。修复方向：捕获路径导入 dmabuf 或 gpu_direct 帧保留 RGBA 影子。
3. **WSLg 环境限制**：`is_wayland()` 按 `WAYLAND_DISPLAY` 判定（WSLg 常驻设置），
   强制 `WINIT_UNIX_BACKEND=x11` 时仍判 Wayland → GPU 窗口渲染器禁用 → parity
   （硬性要求 GPU 呈现）超时。跑法：`env -u WAYLAND_DISPLAY`。曾尝试让 is_wayland
   尊重显式 x11 后端——但激活 GPU 渲染器后 Linux dmabuf 默认链使 gui-smoke 捕获
   变空白（缺口 2 的另一面），已回退；正确修法仍是缺口 2。
   另：WSL2 内核 landlock `landlock_add_rule` 对 /dev/shm 规则返回 EINVAL（fail-open
   继续，帧链路不受影响）；VERSION flag 查询也 EINVAL——Microsoft 内核 landlock 怪癖。

## 环境经验

- wpt-data 是 gitignore 独立 repo 副本，字体类测试失败先跑
  `bash tests/wpt-runner/scripts/sync-imported-resources.sh`（幂等补缺）。
- 无人值守长命令一律走 `make test`/`make reftest`/test-guard 包裹（OOM 防护）。
- 冒烟/parity 在 WSLg 需 `WINIT_UNIX_BACKEND=x11`（Wayland CSD/sctk_adwaita 会挂启动）。

## 追加（2026-08-15 下午）：parity #pic 缺图的完整根因链与部分修复

parity 采集（basic-function 场景）`#pic` 区域 100% diff 的逐层排查结论：

1. **不是**最初假设的「过期采纳位图」——settle 循环帧号正常推进、采纳位图为最新。
2. 采纳位图内容**竞态**：约半数运行含图片、半数缺图（转储位图像素证实）。
3. 竞态源头：`sync_webview_viewport` → renderer `SetViewport` →
   `try_republish_cached()` 立即重发布——该帧是「解码前」过渡帧；图片 final render
   随后到达与否取决于时序，且 `WebView::resize` 同尺寸也会**重建 RenderPipeline**
   （重解析+图片重解码，解码完成不保证触发重绘）——一旦回归到缺图帧可能永久停留。
4. 捕获侧 settle 判据「等第一个推进帧」恰好拍到过渡帧。

已落地修复（将 0% 通过率提升到 ~50%，且都属原则正确）：

- `parity_smoke.rs` settle 判据改为**帧序列静止**（帧号连续 3 次采样不前进，每次
  ~100ms 窗口），符合 skill 证据契约的「加载完成 + 采样一致」。
- `webview.rs` `resize` **同尺寸 no-op**：消除同视口重推导致的 Pipeline 重建回归。

**残余竞态（renderer 图片管线，待独立立项）**：load 期「图片解码完成 → 触发重绘」
的时序仍不确定——卡死运行中图片帧从不落地。需要 renderer 侧探针定位解码完成事件
与 repaint 触发链（导航流的 image batch 机制 vs resize/republish 路径）。

## 追加（2026-08-15 傍晚）：gpu-dmabuf 冒烟自相矛盾已修复（RGBA 影子 + is_wayland）

原「已知缺口 2/3」的修复落地：

- **gpu_direct RGBA 影子**：dmabuf 采纳时（process_backend）经 `map_linear_rgba`
  生成 CPU 影子入 tab image_cache；`commit_compositor_dmabuf` 以帧位图键存储。
  窗口渲染仍走 compositor_import（不双绘）；headless GPU 捕获
  （`get_webview_extra_primitives_for_capture` → `compositor_frame_primitives`
  允许影子回退）绘制影子获得完整页面像素。
- **is_wayland 尊重显式后端**：`WINIT_UNIX_BACKEND=x11` 判非 Wayland——WSLg 的
  `WAYLAND_DISPLAY` 常驻不再误禁 GPU 窗口渲染（parity 生产门禁与 gpu-dmabuf
  冒烟都要求 GPU 呈现路径）。此前单独启用该修复会让 gui-smoke 捕获空白
  （缺影子的 dmabuf 路径），影子补齐后两全。

验证：`make browser-compositor-smoke` 三模式（legacy/compositor/gpu-dmabuf）
首次全绿——`compositor_dmabuf_adopted` 事件出现且页面区域像素断言通过
（该模式自 a48f58a1e 引入后从未绿过）。
