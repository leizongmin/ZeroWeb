# M3-c 拆面：半透明帧复用 × 部分脏区重绘——只读静态审计（DC-4 半透明子项）

**运行身份**：审计基线 HEAD `80cf302e1`。**只读审计切片：零代码变更**。修复面 `apps/compositor` 为并行流近期活跃面（4b899ad52 2026-10-22 触及 compositor lib.rs present 管线 +381 行），按 run-rules §9 本切片只读不修，发现列跨流协调。

## 结论

**发现真实分歧（潜伏面，非默认配置可达）**：S3 部分脏区重绘路径对半透明图元存在 src-over **二次混合**——区域渲染直接在 `copy_front_to_back` 复制的旧帧像素上叠画，而全量路径每帧以 memset 白底重建。半透明图元在脏区内与自身上一帧的渲染结果再混合，颜色逐帧向饱和方向漂移，脏区边界出现可见接缝。

**默认生产配置不可达**：partial 路径唯一入口 `is_partial`（apps/compositor/src/lib.rs:563）要求 `!compositor_scroll_transform_enabled()`，而 `ZW_COMPOSITOR_SCROLL_TRANSFORM` 默认 enabled（runtime-config/src/lib.rs:150-153，`enabled_by_default` 语义：非 `0`/`false` 即开，lib.rs:194-198）→ 默认下 `is_partial` 恒 false，每帧全量重绘于新白底。可达配置 = 该 flag 显式置 `0`/`false`（受支持 kill-switch，已在 runtime-config 登记）。DC-4 默认路径像素语义由全量重绘保证，与 [m3-a](../2026-10-23-m3-a-postprocess-final-output/ANALYSIS.md) 结论一致。

## 证据链（五环闭合）

| # | 环节 | 位置 | 事实 |
|---|---|---|---|
| 1 | 全量路径每帧新白底 | crates/render-foundation/src/cpu/mod.rs:199-213、:238 | `render_full_scene_region` → `render_full_scene_region_on_background([255,255,255],…)`，`FrameBuffer::new_filled` 一次 memset 构白底 |
| 2 | partial 路径不清区、在旧帧上叠画 | apps/compositor/src/rasterize.rs:101-143；copy 决策 lib.rs:563-565 | `copy_front=true` 时注释明示「caller 已 copy_front_to_back；若未复制则清底」——拷贝**替代**了清底；随后 `render_full_scene_region_into(back,…)` 直接写 back |
| 3 | 区域渲染无清区语义 | cpu/mod.rs:356-384 | `render_full_scene_region_into` 是「画到既有缓冲」的内层函数，按 draw_order/分桶只绘制与 region 相交的图元，无任何打底 |
| 4 | 半透明图元 src-over 叠加 | cpu/mod.rs:795-829（fill_rect 半透明走 `blend_pixel`）、:1171-1189（`out = src·a + dst·(1−a)`） | 不透明 set_pixel 直写（安全），半透明与既有内容合成 |
| 5 | 半透明全视口图元生产存在 + 脏区接线 | crates/engine/src/paint/painter/mod.rs:1530-1534（画布传播背景全视口 fill，`resolve_color_current` 保留 alpha）；apps/renderer/src/runtime.rs:1415 → paint_export.rs:641 → crates/protocol/src/paint_snapshot.rs:601 | 引擎 dirty_rects 生产接线到 IPC → compositor partial 判定 |

## 具体分歧数值（`html { background: rgba(255,0,0,0.5) }`，Color a=128 → src_a≈0.502）

- 帧 1（全量）：白底 (255,255,255) + 50% 红 → G = 0·0.502 + 255·0.498 = 127 → **(255,127,127)**。
- 帧 2（paint-only 变更，脏区 = 变更节点盒 + 1px 外扩）：`copy_front_to_back` 后脏区内重画画布 fill：G = 0·0.502 + 127·0.498 = 63 → **(255,63,63)**；期望（全量语义）仍为 (255,127,127)。
- 逐次 partial 帧按 ~½ 衰减向饱和红 (255,0,0) 漂移；脏区外维持 (255,127,127) → 边界接缝。
- 无作者背景页同样中招：默认画布透明（白底语义仅由 memset 承担），页内任一半透明元素（如 rgba 遮罩/卡片）相交脏区即自我再混合。

## 覆盖面核实

- **GPU 臂同病**：`try_rasterize_partial_into_back`（apps/compositor/src/gpu_raster.rs:40-89）按脏区 clip 直 blit 到 back（caller 须已 `copy_front_to_back`），fill+glyph 支持场景含半透明 fill；无区底重铺。
- **既有测试不矛盾**：apps/compositor/src/rasterize_tests.rs 三个 partial 用例（:71/:117/:197）全部使用**不透明**红/蓝 fill，「脏区外保留旧帧」断言对不透明内容正确；半透明 partial 用例零覆盖。
- **调用面唯一**：partial 仅 lib.rs:573/:600（is_partial 门控）与 :612；:757（视区重采样）`copy_front=false` 走全量白底，安全。
- **触发条件（精确）**：`ZW_COMPOSITOR_SCROLL_TRANSFORM=0/=false` 且某帧 `dirty_rects` 非全视口，且脏区相交的图元中存在半透明 fill/rounded_rect，且其下方在脏区内无不透明图元重铺。

## 修复候选（修复面流决策，本切片不实施）

部分路径在区域渲染前对每个归一化脏区重铺白底（与全路径 memset 语义精确对齐；`back.clear(255,255,255,255)` 已是 `!copy_front` 臂的既有兜底，语义先例一致），或区域渲染改「新底渲染 + blit」。GPU 臂需同规则（scissor 区 clear 或 GPU 侧等价）。附带收益：worker patch 贴片（4b899ad52 新增呈现链）若未来与 partial 组合，同规则先行核对。

## 余项（控制面维持）

多次导航**运行时**对照（A→B→A′）与命中/像素在 engine 面（跨流协调）；本分歧修复在 compositor 面（跨流协调）；修复落地时应附「半透明 partial == 全量」常驻等价测试（rasterize_tests 同文件）。
