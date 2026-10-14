# 系统字体、下载字体、匹配与 WOFF

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖平台字体顺序、TTC face、CSS 别名、宽度匹配、下载字体和 WOFF 容器。字体看起来不对、粗体失效、网页字体加载后仍回退时读这里；字距、整形与位图缓存读同级整形主题。

### Key Entry Points
- `load_platform_fonts` in `crates/render-foundation/src/font/system.rs`：跨角色平台字体表。
- `FontLoader::load_font_at_index` / `build_font_resolver` in `crates/render-foundation/src/font/loader.rs`：face 注册与 CSS 查询表。
- `font_face_aliases` / `resolve_font_faces` in `crates/render-foundation/src/font/face_match.rs`：宽度优先匹配。
- `AsyncPageLoad` in `crates/webview/src/async_load.rs`：抽取与异步获取 CSS 字体资源，向宿主交付结果。

## Gotchas
- 平台字体必须跨 browser、renderer、compositor 同序加载；Regular 重复载入会占据 family_map 的第二位置，使自动生成的 `:700` 指向重复 Regular，当前实现按已载路径去重。 (`crates/render-foundation/src/font/system.rs`, `crates/render-foundation/src/font/loader.rs`)
- 平台 generic serif/sans/mono face 只加入 family_map，不加入缺字 fallback chain；改变 generic 匹配与改变 CJK fallback 是两件事。 (`crates/render-foundation/src/font/system.rs`)
- `load_font_at_index` 先验证指定 collection face，再登记原始字节与空 fontdue 槽；`get` 首次访问才展开轮廓，提前遍历所有 `get` 会破坏大型 CJK 字体的延迟解析收益。 (`crates/render-foundation/src/font/loader.rs`)
- `load_font` 根据字体内部 name 表注册族名，不根据 CSS 声明注册；下载字体必须额外 `register_family_alias`，否则声明的别名无法命中 resolver。 (`crates/render-foundation/src/font/loader.rs`, `crates/webview/src/async_load.rs`)
- `FontLoader::find(FontDesc)` 当前只是 family 的首 ID，并不做完整 weight/style/stretch 匹配；CSS 消费者应走 `resolve_font_faces` 而非把 `find` 当 CSS Fonts 实现。 (`crates/render-foundation/src/font/loader.rs`, `crates/render-foundation/src/font/face_match.rs`, `crates/layout-engine/src/font_resolution.rs`)
- WOFF 解码失败会把原始容器交给 sfnt validator 返回解析错误；`ZW_WOFF2=0` 只关闭 WOFF2 解码，不关闭 WOFF1 或裸 sfnt。 (`crates/render-foundation/src/font/loader.rs`, `crates/render-foundation/src/font/woff.rs`)

## Architecture
- CSS face alias 用十分之一百分比整数编码 stretch；normal 宽度同时保留 legacy weight/style alias，非 normal 只注册宽度键，调用方不能自己拼一套不兼容名字。 (`crates/render-foundation/src/font/face_match.rs`)
- 匹配先选择宽度再按现有粗体/斜体 suffix 顺序回退；normal 请求先查 legacy 键，失败才扫描宽度，大小写不同则仍有不区分大小写的慢路径。 (`crates/render-foundation/src/font/face_match.rs`)
- 同 family 的 `:face=N` 列表保留多个 unicode-range face；字符选择还要经过范围过滤，范围注册会同时失效 shape、hmtx 和 fallback-metrics 三类缓存。 (`crates/render-foundation/src/font/loader.rs`, `crates/render-foundation/src/font/loader/unicode_range.rs`)
- `FontFace.load()` 回调只向共享队列投递 family/src/resolve_id，fetch、字体注册与 Promise 完成由 runtime 消费；不应在脚本回调中直接改 loader。 (`crates/engine/src/font_load_bridge.rs`)

## Patterns
- `ZW_CJK_FONT_PATH`/`ZW_CJK_FONT_DIR` 的配置候选先于平台 fallback；配置 face index 默认 2，常规非 collection 字体候选用 0，TTC 中另尝试 face 7 提供 monospace family。 (`crates/render-foundation/src/font/system.rs`)
- 字体卸载同时移除 alias、fallback、位图和描述符并清相关缓存，但不回收数字 ID；duplicate 的 Arc 字节仍可保活旧 registry，不能据 remove 推断所有消费者立即释放。 (`crates/render-foundation/src/font/loader/release.rs`, `crates/render-foundation/src/font/loader.rs`)

## Dependencies
- WOFF1 用 zlib 表解压并重建 tag 排序、4 字节对齐的 sfnt；WOFF2 用 wuff 解压，后续两者均进入同一 face 验证路径。 (`crates/render-foundation/src/font/woff.rs`, `crates/render-foundation/Cargo.toml`)
