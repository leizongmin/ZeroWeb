# M1b 首帧上屏 e2e 证据（2026-09-01）

## 通路（R3268 canvas 同款两段式）

```
tests/fixtures/media/sample-webm-vp9.webm（真实文件）
  → zero-media VideoDecoder::open_webm_vp9 + next_frame()（首帧 RGBA 320x240）
  → wpt-runner load_video_first_frames 注入 ImageCache（ImageKey(simple_hash(url))）
  → pipeline extract_image_metrics（video srcs 扫描）→ set_image_sizes（固有尺寸）
  → layout-engine apply_replaced_element_sizing（video 白名单：NodeId 固有尺寸 sizing）
  → painter paint_video_element → ImagePrimitive（key = image_resource_key(src)）
  → render_full_scene CPU 光栅化 → 帧缓冲像素
```

键对齐链：harness 注入用 `simple_hash(url)`；painter 发图元用
`image_resource_key(src, document_url)`（同源 `simple_hash` + URL 解析）——本地相对
src 下二者一致。

## 验证

- engine 单测 4 件（`video_frame_display_tests`）：有解码尺寸 → ImagePrimitive（1:1
  320x240 rect）；无解码 → 无图元（占位零回归 gate）；无 src → 无图元；固有尺寸进
  布局（video 盒 320x240）。
- wpt-runner e2e 双件（`reftest/tests.rs`）：
  - `m1b_video_first_frame_renders_to_framebuffer`：真实 fixture 首帧上屏，帧区 RGB
    均值锚点 138-168（testsrc2 纹样 ≈153.5，同 zero-media M1a ffmpeg 参照锚点）；
    帧区外保持背景白（无越界绘制）。
  - `m1b_video_undecodable_src_stays_placeholder`：非 webm src（mp3）解码失败 → 占位
    白底——正负例共同锁定「像素来自真实解码」。
- 质量门禁：make test 66 套件 18577 全绿；clippy -D warnings 零警告；fmt 干净。

## reftest-upstream A/B（13951 → 13950 / 16730，唯一净 delta）

- **replaced-element-003（css-sizing/aspect-ratio）：false-pass unmask（-1）**。
  该案 `<video src="2x2-green.webm" style="aspect-ratio:2/1;width:100px;
  background:green">` vs `ref-filled-green-100px-square`（CSS green #008000），
  fuzzy `0-30;0-5000`。基线 video 不上屏 → 恰好显示背景 #008000 命中 ref（假通过）。
  M1b 真实帧上屏：容器元数据 `color_range=pc` + `color_space=gbr`（identity 矩阵，
  ffprobe 实测）→ 帧为纯 #00ff00，越出 fuzzy 0-30 通道预算 → 100x50 帧区 5000 px
  全差（1.04% 贴线 fail）。**chromium 真实行为同样绘制帧内容**——本案由「占位假通过」
  转「真值揭示」，与 R3866/R2478「false-pass unmask」先例同性质。
- box-shadow-overlapping-003 / replaced-element-003 在并行全量跑中的偶现翻动经 3 次
  standalone 复跑确认为并行负载噪声（与本次改动无关）。

## 范围注记

- 色度元数据精化（WebM Colour 元素 / limited-range YUV / BT.709 矩阵自适应）——M2
  解码层后续项（本案揭示的真值差距面）。
- 生产侧渲染线程注入（webview/async_load 媒体字节 → 解码 → ImageCache）——M2a
  player 模块实施项；M1b 通路已在 harness 侧完整打通并常驻断言。
