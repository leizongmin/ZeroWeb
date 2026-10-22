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

## M3-c 追加（同日切片 2）：region 裁剪补全（render-foundation，本流域）

深入核对发现 S3 region 模式的裁剪**只完成了一半**——`render_draw_order` 中仅 Fill 有逐像素裁剪（`fill_rect_for_region`，cpu/mod.rs:749）：

| 图元 | 修复前 | 后果（region 消费方） |
|---|---|---|
| Glyph（页面文字） | **完全无门控**（cpu/mod.rs:648 原） | 每帧 region 渲染重画**全文档字形**；区外 AA 边在旧帧上逐帧二次叠加变深 |
| PathStroke | **完全无门控**（:638 原） | 同上（半透明描边整帧二次叠加） |
| RoundedRect/Shadow/Gradient/Image/Stroke/PathFill | 相交跳过、无逐像素裁剪 | 跨界图元整绘，越界部分叠上区外旧帧（不透明写同值无害；半透明/AA 漂移） |
| Filter/Transform/Clip | 无门控 | 同族 |

**本切片落地（render-foundation，725/725 全绿 + clippy/fmt PASS + compositor 70/70）**：Glyph 按保守包围盒门控（`glyph_primitive_bounds`，宁多勿少，跨界字形不裁剪防条带切字）+ PathStroke 按顶点 hull 外扩线宽门控（与 PathFill 对齐）；4 个新测试钉住：区外字形跳过（整盒扫描防镂空假通过）、跨界字形 region 内墨迹与全量逐位一致、**半透明 fill region==full 逐位等价**（DC-4 光栅级契约，既有等价测试只覆盖不透明）、区外 PathStroke 跳过。零默认路径变更（region=None 全量不受影响）。

**残留（未实施，列协调/后续）**：跨界图元越界叠加族——逐像素裁剪需对 7 个光栅器做 loop-clamp 手术（rounded rect 不能用「裁 rect 再画」——圆角相对原 rect，需 clamp 循环界），且消费方 stale-base（不清底）不除则区内二次混合仍在；与 compositor re-base 修复（上文候选）**必须配套落地**，拆开修任何一半都剩另一半的漂移。

**消费面修正**：上文「默认生产配置不可达」仅指 **compositor partial 路径**（`is_partial` 被 scroll-transform 默认开钉死）。核对发现 **apps/browser 条带复用路径**（app_platform.rs:638-647 `can_reuse`，`ZERO_SCROLL_BLIT` 默认开）也是 region-into 消费方且**默认活跃**：加载动画期每帧只重绘 chrome 条带（不清全帧）——同 stale-base 家族，底色不透明时表现为条带内 AA 边（spinner 弧线等）逐帧加深；chrome 半透明图元清单未逐项核查（列运行时观察项）。该文件属 zero-web 产品面且近期有提交（10-18），re-base 修复列跨流协调。

## M3-c 落地（同日切片 3）：compositor CPU partial 臂 re-base 修复

**协调门解除依据**：compositor-animation-continuity goal 2026-10-22 收口 DONE，apps/compositor 面空闲；bg02 流当日活跃面在 engine shim/protocol/renderer，与本切片（compositor rasterize + render-foundation surface）不重叠（run-rules §9）。goal M3 范围本含 compositor 消费者，本流承接。

**修复内容**：

1. `crates/render-foundation/src/surface.rs`：新增 `FrameBuffer::clear_rect(x0, y0, x1, y1, rgba)`——物理像素闭开区间、越界钳位、行主序按行填充；语义与整帧 `clear` 一致仅范围受限。
2. `apps/compositor/src/rasterize.rs` `rasterize_into_back` 的 `copy_front=true` 分支：渲染循环前对每个归一化脏区重铺白底（floor/ceil 外扩 + 视口钳位，与下方 region 构造及 blit_region 拷回范围一致）——CPU partial 路径与全量 memset 白底语义精确对齐；脏区外保留旧帧（未变内容）。
3. GPU 臂不改：主 pass `LoadOp::Clear(WHITE)` 已清整个目标再按 scissor 绘制（gpu/renderer/mod.rs:1109/:1132），无 stale-base，本行仅在注释中注明。

**回归测试**（rasterize_tests.rs `partial_dirty_semi_transparent_matches_full_render`）：帧 1 全量渲染半透明全视口红 fill（a=128）作为上一帧与期望结果；帧 2 `copy_front_to_back` 后以相同图元、局部脏区 partial 重绘（模拟 paint-only 变更）；断言 partial 帧与全量帧**逐位一致** + 脏区内探针 (255,127,127)（修复前二次混合为 (255,63,63)）+ 脏区外保留。**fail-first 已验证**：stash 修复后单测 FAILED（复现二次混合），恢复后 PASS。

**验证**：zero-compositor 71/71（含新测试）、zero-render-foundation 725/725、`cargo clippy --all-targets` 0 警告、rustfmt 干净。

**残留定性更新（切片 2「跨界越界族」影响面收窄）**：re-base 落地后，主案例（全视口半透明 fill）经 `fill_rect_for_region` 逐像素裁剪不越界，**完全修复**；残留仅「未逐像素裁剪族（RoundedRect/Shadow/Gradient/Image/Stroke/PathFill）跨界 + 区域内已重铺白底」组合下的区域边界有界伪影——且消费方若改 temp-buffer 方案（渲染到 scratch 再拷回）则该族无需手术。列观察项，不单独修。

**契约瑕疵记录（本轮不修，列 render-foundation 契约清理待办）**：`render_full_scene_region_into` 的 region 参数内部单位不一致——glyph 过滤按**物理像素**（cpu/mod.rs:387 `glyph.x * scale` 后与 region 比较），fill 裁剪按 **CSS 逻辑**（cpu/mod.rs:787 `fill.rect` 与 region 直接求交）。scale=1 时两义重合无影响；scale≠1 且调用方传物理坐标时 fill 裁剪错位。现网可达性：browser 条带两处调用均传 scale=1.0（app_platform.rs，构造性安全）、compositor partial 默认关闭 → **潜伏**。后续与 GPU clip 逻辑坐标契约（T34 已修 GPU 侧同族问题）对齐时一并收口。

## M3-c 后续（同日切片 4）：region 契约统一为物理像素（render-foundation，收口切片 3 记录的契约瑕疵）

**跨流红灯复绿确认**：`html_compat::dialog_popover_invalid_targets_leave_no_top_layer_state` 通过（7fd3ed736 把集成测试期望同步到 s32 lenient 语义）；html_compat 全模块 21/21 绿，挂约 4 轮的跨流红灯族清除。

**契约统一落地**：region 参数定约**物理像素**（与全部调用方现状一致——compositor rasterize/worker patch 传 `x*scale`、browser 条带传 scale=1.0、ui_glyphs 过滤本按物理）：

1. 两处 `in_region` 闭包（`render_typed_buckets`/`render_draw_order`）：图元 CSS rect 闭包内统一 ×scale 后与 region 求交（乘法逐位精确）。修前 CSS rect 直接与物理 region 比较，scale>1 过度包含（保守无害），**scale<1 会漏画区域内图元**（stale 像素，真实缺陷）。
2. `fill_rect_for_region`：region ÷scale 换算回 CSS 后与 `fill.rect` 求交（scale=1 与 2 的幂逐位精确；非 2 的幂除法亚像素舍入仅影响裁剪边界像素归属）。修前混合单位求交，scale≠1 时跨界 fill 按错误边界越界绘制。
3. 切片 2 的 Glyph/PathStroke 内联门控改走统一 `in_region`（保留 is_none 短路惰性）。
4. 文档修正：`render_full_scene_region`/`_into` 的 region 单位声明（原写「CSS 逻辑像素」与 glyph 过滤实现自相矛盾）。

**测试与 fail-first**：新增 `region_matches_full_and_clips_at_scale_2`（半透明全视口底 + 跨界半透明 fill + region 内字形，逻辑 region (0,0,100,100) → 物理 (0,0,200,200)；断言 region 内与全量逐位一致 + region 外保持白底）+ compositor `partial_dirty_semi_transparent_matches_full_at_scale_2`（HiDPI 端到端 partial==full）。**fail-first 已验证**：撤修复后 render-foundation 测试恰在 (200,0) 泄漏 (255,127,127) FAILED，恢复后 PASS。

**过程教训（已沉淀 [learning](../../../../../docs/learnings/bugs/2026/10/2026-10-23-manual-primitives-draw-order-vacuous-equivalence.md)）**：首版测试 fail-first 不红——手工 `RenderPrimitives` 构造的 `add_glyph` 只登记 glyph op，fills 不进 draw_order，渲染走 draw_order 分支时 fills 整体跳过，双缓冲全白等价断言空转；`cull_invisible` 对手工图元同样只重建出 glyph op。修正为手工 push 完整 `DrawOp` 序列后 fail-first 才成立。教训：等价类断言的绿灯不能区分「验证了修复」与「两边都没画」，fail-first 是唯一可靠检测。

**验证**：zero-render-foundation 726/726、zero-compositor 72/72、clippy/fmt 0 警告、定向 bench-gate PASS（render_bench 1/1 绝对预算过；硬件基线失配 WARN 为既有事实）。
