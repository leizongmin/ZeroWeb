# M2-S161 动态元素 gCS 通道 + 外链 media 门三层

日期：2026-10-30。基线：s159 终案对账（`/tmp/zw-s159-full1.log` 7205P/793F/87T）+ s160 已收口 11F→P。

## 切片目标

控制面 s161 候选①拆两半施工：动态元素 gCS 读值通道（handle 视图）与外链 `media` 门（conditionally-block 2F + media-match-block-script 1F 的设计面前置）。

## 施工面

1. **handle 视图 gCS 通道**（`zero-engine`）
   - `computed_style_cache.rs`：新 `HANDLE_VIEW_CACHE` thread_local 10-tuple 槽（html/drain_gen/style_version/view_gen/applied/css/adopted/doc/persistent_nodes/styles）+ `computed_style_by_handle()`——`parse_html` + `apply_dom_mutations_full` 重放 `[applied..count)`（persistent_nodes 回填 ephemeral handles）+ `compute_styles_for_doc` + `apply_intrinsic_size_overrides`；`clear_generation_cache` 同步清槽。
   - `callbacks.rs`：注册 `__zw_get_computed_style_by_handle(handle, prop)`，读 snap/style_version/MUT_DRAIN_GEN/DOM_VIEW_GEN/MUT_APPLIED_CURSOR（min 钳制）。
   - `computed_style.rs`：`compute_styles_for_doc` 放宽 `pub(super)` 供 handle 视图共用。
   - `part01.js` gCS：query 失败后回落 `elt.__zwHandle` → 新回调（catch 返 `''`）；display/position/float 等全部走通。
2. **外链 media 门三层**
   - Rust 静态路径：`extract.rs` 新 `extract_stylesheet_links`（href+media 原文）；`webview.rs resolve_external_css` 与 `async_load.rs begin_stylesheet_fetch` 头插 media 门（非空非 `all` → s160 同款 `MediaContext::new(1280.0, 800.0)` evaluate，no-match 跳过不取回不入级联；视口 FIXME 记账）。
   - shim overlay：`part03.js _zwSsRecomputeExt` 对 `_zwExtSheetTexts` 逐表 wrap `@media …{…}`（ASCII ws 归一 + trim + lowercase；空串与 `all` 不 wrap）；`part04.js` setAttribute/removeAttribute 与 `part05.js` IDL reflected 三触发臂带 handle-only TAG 容忍（`_realTag` 对动态 link 回落 DIV——s150 注记在案，重算纯幂等下发误触发无害）。

## 验证

- `cargo check`（engine/webview/style-system）✓、`cargo fmt` ✓、`make guarded-clippy` workspace ✓、shim 14-part 拼接 `node --check` 111228 行 ✓。
- 探针（静态 link media=none → 翻 all）：门半（初始不应用）+ 翻转半（IDL `media="all"` 立即应用，无 refetch）双断言 Pass，实证 Rust 静态槽门与 shim overlay 翻转通道各就各位。探针为 gitignore 本地数据，验证后已删。
- 靶面 `stylesheet-media-change-no-fetch`：断言 1（动态 p gCS 读 red）翻 P——失败点移至断言 2（记账面，见下）；`the-link-element` 目录 113P/2F 双 F 均为既有（link-rel-attribute shadow DOM 变体 s159 同红、断言 2 本切片记账）；`the-style-element` bad-set 为 s159 bad-set 严格子集（s160 11F→P 保持）；`style_non_matching_media` P 保持。
- 全量案级 join（full3+full4 对 s159 终账，(key,state) 对 sort -u）：F→P 恰为 s160 的 11；P→F 唯一实质位移 **SVG noscript**（见下），修复后 `non-html-noscript` 复 P（终账 full4 7093P/803F/91T）。
- **回归修复（超出切片预期的质量面）**：`style-system/lib.rs` UA 默认 display 注入带 `is_html_ns` 门——HTML 渲染规范 UA 表在 real UA 经 `@namespace xhtml` 限定（Firefox html.css 先例，bug 1470150）；此前 localname 盲匹配，SVG/MathML 同名异命名空间元素误吃 `display:none`，动态 gCS 通道暴露真实计算值后翻红。修复即全局生效（静态渲染 + gCS 双通道同源）。
- `dialog-popover-closedby-complex` 5 子测 P→F 位移经 A/B 定谳为环境漂移非回归：带/不带本切片 style-system 改动 solo 跑同 16 bad，s159 自身同文件 12 bad，当日 full1→full4 呈 4→11→16→16 时序漂移（已录 flake 类）。

## 记账面（本切片不解决）

1. **动态 link 对 `__zw_query_all` 不可见**：R51c 查询视图故意不应用 handle 链 → overlay 重算无法枚举动态 link，级联 blob 通道对其缺口（事件通道有）；stylesheet-media-change 断言 2 当前根因即此，per-sheet 结构化通道 = s161 候选①的设计面，另立切片。
2. **外链 css 槽位级联序**：collect 注入序 = [color-scheme, 外部 css, 页面 style] → 同特异性页面 style 胜，真实浏览器按文档序 link 在后应胜；翻转需全语料 A/B 独立切片。
3. **跨批 handle 边界**：`computed_style_by_handle` 仅重放 `[applied..count)`，跨批句柄引用边界 FIXME 已注代码。
4. **layout-engine `collect_items.rs` 兜底 `ua_default_display` 调用未带命名空间门**：本切片只修 style-system 注入点（corpus 零位移面）；layout 兜底路径为潜在同构缺口，暂无失败证据。
