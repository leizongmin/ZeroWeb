# 共享图元转换与字体命名空间

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

`zero-paint-convert` 是 renderer 的 `PaintSnapshotParams` 到光栅器 `RenderPrimitives` 的公共映射。修改 IPC 字段或排查 browser、compositor、WebDriver 显示不一致时先读这里。转换不负责下载资源、导航 epoch 或窗口呈现；各消费者负责资源注入与自身状态。

## API Surface

| 真实 API | 输入与责任 |
|---|---|
| `to_render_primitives(PaintSnapshotParams) -> RenderPrimitives` | 按值消费快照，转换图元、绘制顺序、字形来源和 variation 表。 |
| `fonts::PaintFonts::new(Arc<FontLoader>)` | 基于平台字体表建立 surface/document 局部注册表。 |
| `PaintFonts::update(&[IpcFontPayload]) -> Result<(), String>` | 校验并原子替换完整下载字体集；成功变化递增 `revision`。 |
| `PaintFonts::remap` / `remap_glyphs` | 将下载字体的源 ID 改成本地 loader ID，平台 ID 保持不变。 |
| `PaintFonts::has_downloaded_fonts` | 判断是否需要独立字体命名空间。 |

## Usage Examples

browser 保留快照元数据，公共转换消费克隆；其后单独注入图片和保存字体 payload。来源：`apps/browser/src/paint_ipc.rs`。

```rust
let primitives: RenderPrimitives =
    zero_paint_convert::to_render_primitives(params.clone());
snap.font_payloads = params.font_payloads;
```

browser 借公共注册表校验信任边界，但之后另建与 chrome 共存的映射。来源：`apps/browser/src/app_page_fonts.rs`。

```rust
let (base, _) = Self::cached_system_fonts();
let mut validated =
    zero_paint_convert::fonts::PaintFonts::new(std::sync::Arc::new(base));
if let Err(error) = validated.update(fonts) {
    tracing::warn!(%error, "browser: rejected page font resources");
    return false;
}
```

WebDriver 从最近一帧转换，再调用 CPU 场景光栅器；此调用本身不导入下载字体。来源：`apps/webdriver/src/session.rs`。

```rust
let primitives = zero_paint_convert::to_render_primitives(paint.clone());
let font_loader = FontLoader::new();
let mut glyph_cache = GlyphCache::new(1024);
```

## Gotchas
- 转换不是资源同步：图片只转成 `ImageKey`，图片 payload 与字体 payload 不会自动注入缓存或 loader；只调用转换可得到几何却缺内容。 (`crates/paint-convert/src/lib.rs`, `apps/browser/src/paint_ipc.rs`)
- 当前 IPC 映射把 shadow 的 `inset` 固定为 false、`clip`/`clip_out` 固定为 None，把 image 的 `source` 固定为 None；不能把进程内阴影裁剪或 border-image 子矩形能力等同于 IPC 已贯通。 (`crates/paint-convert/src/lib.rs`)
- `glyph_text_runs` 中零 ID 被跳过，重复 ID 即使文本相同也会使该 ID 失效；缺失或非法 UTF-8 区间产生无 source 的 glyph，不应补造 cluster 身份。 (`crates/paint-convert/src/lib.rs`)
- variation 表中任一无效坐标使该表项变为空集，越界的 `font_variation_id` 被移除；保留 glyph 不代表其轴坐标有效。 (`crates/paint-convert/src/lib.rs`)
- `PaintFonts::update` 拒绝空字节、超预算、重复源 ID 和与平台 ID 冲突的 payload；失败不替换已有 loader，空集成功则释放文档下载字体。 (`crates/paint-convert/src/fonts.rs`)

## Architecture
- 字体更新是完整集合替换而非增量追加；同 ID、同 face index、同字节的帧复用旧 Arc 和 revision，变化时从 base duplicate 后全部导入再提交。 (`crates/paint-convert/src/fonts.rs`)
- 图元 `draw_order` 按原索引映射到 `DrawOp`；文本控件边界是非绘制元数据，供 browser 输入法和 caret 消费，不能从 GPU 像素推断其正确性。 (`crates/paint-convert/src/lib.rs`, `apps/browser/src/paint_ipc.rs`)
- renderer 每份完整帧重复发送其中真正引用的下载字体并按 ID 去重排序，避免 latest-wins 队列丢掉单独资源注册；消费端须把 payload 当完整当前集合，而非“一次性加载通知”。 (`apps/renderer/src/font_payloads.rs`, `crates/paint-convert/src/fonts.rs`)

## Decisions
- 公共映射在 R4181 从 compositor 提取，再让 browser 删除第二份实现；新增图元映射应定位公共层，browser 自身几何、hit-test 和缓存副作用仍留调用方。 (`crates/paint-convert/src/lib.rs`, `apps/browser/src/paint_ipc.rs`, `git:3b04866ebf25ea7334afb8c97b34619a4d26202e`, `git:0bcac5fc6c4a020b47c7ff4a4b0279bd1574acb4`)

## Consumer Analysis
- browser 的 `apply_paint_snapshot` 同时还原图元、图片、文档几何、脏区和 hit-test；`sync_page_fonts` 另外导入页面字体并清 glyph cache、GPU atlas 和 retained framebuffer，不能把 chrome 字体 ID 直接换成 renderer ID。 (`apps/browser/src/paint_ipc.rs`, `apps/browser/src/app_page_fonts.rs`)
- compositor 用公共转换结果进入 `rasterize_into_back`，切换字体 namespace 时 `font_resources::activate` 更换 loader、清 CPU/GPU 字形缓存并重建 raster worker；仅替换 loader 会留下旧页面字形。 (`apps/compositor/src/rasterize.rs`, `apps/compositor/src/font_resources.rs`, `apps/compositor/src/lib.rs`)
- WebDriver 的 `session_screenshot` 使用公共转换和 `render_full_scene`，但创建空 `FontLoader`，且没有 `PaintFonts` 导入；不能据“共用光栅器”声称下载字体截图与 compositor 等价。 (`apps/webdriver/src/session.rs`)
