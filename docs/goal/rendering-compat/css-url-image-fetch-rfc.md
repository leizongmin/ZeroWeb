# R1794 — CSS `url()` 图片抓取 + painter 查找一致性修复

**日期**: 2026-07-20
**状态**: Active（实施中）
**承接**: R1793（WebP 解码接入，关闭 goal doc line 76「WebP 解码未接入」）；本 RFC 关闭同条「残余缺口 = CSS `url()` 背景图未抓取」。

## 背景

goal doc `rendering-compat.md` Support Envelope「图片子资源与替换元素」（line 76）要求：
> 必须抓取 `<img src>`、CSS `url()`、favicon/metadata 中实际参与渲染的图片资源。

R318 已贯通 `<img src>`（PNG/JPEG/SVG via resvg，R1793 加 WebP）+ `@font-face` 字体 url。
**CSS `url()` 图片引用（`background-image` / `list-style-image` / `border-image-source`）目前完全不工作**——
painter 会为这三类 computed value 生成 `ImagePrimitive { image_key }`，但：

1. **无抓取路径**：`fetch_image_subresources`（webview.rs:371）只处理 `extract_img_srcs`（`<img src>`），
   不扫描 CSS 文本中的 `url()`；`image_cache` 从无这些图片的像素数据。
2. **查找 key 不一致（latent bug）**：painter 用 `simple_hash(url)` 哈希**原始 CSS url 字符串**
   （`effects.rs:121/143` background-image、`text_list.rs:110` list-style-image、
   `border.rs:182` border-image-source）；而抓取路径用 `image_resource_key(&abs, None)`
   = `simple_hash(resolve_document_url(document_url, url))` = **解析后的绝对 URL** 哈希。
   相对 url（`url(bg.png)`）两者 key 永不相等 → 即使抓了像素也找不到。

## De-risking 发现（本轮实证）

- CSS parser 存储原始 url：`BackgroundImageComputedValue::Url(url)` 的 `url` 是 CSS 原文里的字符串
  （`apply_advanced.rs:991-1009`；test `extended.rs:1409` 断言 `"bg.png"` 原样），**未在 parse 期解析为绝对**。
- `<img>` 路径**正确**：`build_img_intrinsic_sizes`（pipeline.rs:177）用
  `image_resource_key(&src, document_url)`，与抓取 key 一致。
- 唯一**正确**的 painter 图片查找是 `text.rs:322`（`<img>` 内容）用 `image_resource_key`。
- 4 处错误查找见上（effects×2 + text_list×1 + border×1）。

## 修复方案（两段，必须同 land 才端到端工作）

### Part A — painter 查找一致性（latent bug 修复）

4 处 `simple_hash(url)` → `image_resource_key(url, self.document_url.as_deref())`：

| 文件:行 | 用途 | 改动 |
|---------|------|------|
| `paint/painter/effects.rs:121` | background-image first_url_hash（intrinsic size 查找） | `image_resource_key(url, self.document_url.as_deref())` |
| `paint/painter/effects.rs:143` | background-image 每层 image_key | 同上 |
| `paint/painter/text/text_list.rs:110` | list-style-image | 同上 |
| `paint/painter/border.rs:182` | border-image-source | 同上 |

**零回归证明**：`image_resource_key(src, doc_url)` = `simple_hash(resolve_document_url(doc_url, src))`，
`resolve_document_url` 对 (a) 绝对 URL（`is_non_relative_href` true）原样返回，
(b) `doc_url=None` 时 `unwrap_or_else` 返回 `src` 原样。故：
- 绝对 URL → `simple_hash(abs)` = 改前；**字节不变**。
- `document_url=None`（现有所有 paint 单测）→ `simple_hash(src)` = 改前；**字节不变**。
- 相对 URL + document_url 设（真实导航）→ 改前 `simple_hash("bg.png")`（broken，永不命中）→ 改后 `simple_hash("http://x/bg.png")`（正确，命中抓取 key）。**只把 broken 改成 correct，无 working case 被破坏**。

### Part B — CSS `url()` 图片抓取

1. **`extract_css_image_urls(css: &str) -> Vec<String>`**（pipeline.rs，与 `extract_font_face_urls` 同模块同模式）：
   - 扫描 `url(...)` token（小写化定位 + 原文截取，处理 `"`/`'`/裸引号）。
   - **排除 `@font-face` 块内**的 url（字体由 `extract_font_face_urls` 单独处理，避免重复抓取）。
   - **排除 `data:` URI**（调用方识别，但这里也过滤以保持集合干净）。
   - **去重**（保留首次出现顺序）。

2. **HTML 内联 `<style>` 文本提取**：复用 `zero_dom` parse，收集所有 `<style>` 元素的 text_content，
   与外链 CSS 文本拼接，统一交 `extract_css_image_urls`。
   - inline `style="background-image: url(...)"` 属性暂**不在本切片范围**（较罕见，DOM 遍历 + 单属性解析成本高；
     外链 + `<style>` 块覆盖绝大多数真实页面，留 follow-up）。

3. **接线**（webview `prepare_page_subresources`）：
   - `resolve_external_css` 返回 external_css 后，组合 `external_css + inline <style> text`。
   - `extract_css_image_urls(combined)` → 与现有 `<img src>` 共用 fetch+decode 循环（同样的 base 解析、
     HTTP get、`decode_image_bytes`、`image_resource_key` 入 `image_cache` + `image_sizes`）。
   - background-image 不影响布局固有尺寸推导（`<img>` 才走 layout intrinsic），故入 `image_sizes`
     仅供 painter `get_image_size` 解析 background-size:auto 用——key 与 painter 改后查找一致即对齐。

## 验收标准

1. **单测**：
   - `extract_css_image_urls`：基础 url / 多 url / `@font-face` 内 url 被排除 / `data:` 被排除 / 去重。
   - painter 一致性：`background-image: url("rel.png")` + `document_url=Some("http://x/page")`
     → 生成的 `ImagePrimitive.image_key` == `image_resource_key("rel.png", Some("http://x/page"))`
     （改前 == `simple_hash("rel.png")`，改后对齐抓取 key）。
2. **回归门禁**：`make test`（render-foundation + engine + webview 相关）全绿；`cargo clippy --workspace --all-targets -D warnings` clean；`cargo fmt --check` clean。
3. **product-smoke A/B**：welcome / wintertc / morning-work 三 fixture 均不用 CSS `url()` 背景图
   → 结构检查全 PASS，字节级 byte-identical（确认 Part A painter 改动对不用 CSS url 的页面零影响）。
4. **goal doc**：关闭 line 76「CSS `url()` 背景图未抓取」残余缺口。

## 不在范围

- **`async_load.rs` 异步图片抓取路径**（line 384 `extract_img_resources`）：交互式浏览器 async page load 路径有独立 fetch 流；本切片只接 sync `fetch_url` → `prepare_page_subresources` 路径（覆盖 reftest harness + product-smoke）。async 路径的 CSS url() 抓取留 follow-up。
- inline `style=` 属性内的 `url()`（follow-up，需 DOM 遍历）。
- `content: url()` 生成内容图片、`cursor: url()`（抓取可覆盖但渲染路径未验证，留 follow-up）。
- 渐变 `image-set()`、多背景 `image()` 函数（ZW CSS parser 未支持，非本切片）。
- favicon 抓取（line 76 列出但不参与渲染，独立低优先级）。

## 风险与回退

- Part A 4 处改动均为 `simple_hash` → `image_resource_key`，已证零回归（见证明）。
- Part B 为纯新增抓取（不删除/不改现有 `<img>` 路径），失败降级 `tracing::warn!` 不阻断（同 `<img>` 语义）。
- kill-switch：本切片无 env gate（改动小且零回归已证）；若 product-smoke 出现回归则整片 revert。
