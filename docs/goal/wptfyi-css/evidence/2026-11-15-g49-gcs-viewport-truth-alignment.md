# g49 GCS 视口真值对齐刀

2026-11-15。driving: css-values viewport-units-css2-001 全页 84F——B0
（g47-b2）既有失败，视口相对长度（vw/vh/vmin/vmax）解析基准错位。

## 1. triage

语料 runner 视口 800×600（testharness.rs `WebViewConfig{width:800,
height:600}`，shim 安装期 `set_pending_layout_viewport(800,600)` +
`__zw_user_resize` 同步 innerWidth=800）。但 GCS 快照通道与渲染管线
`<style media>` 门各有一处硬编码 1280×800：

- **GCS length/% 解析**（computed_style.rs）：`sys.set_viewport(1280.0,
  800.0)` 常量 ×4 站（compute_document_styles、compute_styles_for_doc、
  length_to_css 兜底臂、calc_computed_to_css CalcContext）——`10vw`
  折算 128px，渲染面/innerWidth 为 80px，页面断言
  `documentElement.clientWidth` 派生期望全错。
- **渲染管线 `<style media>` 门**（pipeline/mod.rs style_sheet_if_eligible，
  FIXME 在案）：`MediaContext::new(1280.0, 800.0)`——`<style media>` 表
  应用判定与 GCS 门、shim innerWidth 三面分裂（css-conditional/
  mediaqueries：视口求值按布局 viewport）。`<link media>` 已正确（s161
  shim 侧 @media 包裹，随 style-system viewport 走）。

## 2. 实现（engine 5 文件，单点真值 `gcs_viewport_dims()`）

- `computed_style_cache.rs` 新增 `gcs_viewport_dims()`：读
  PENDING_VIEWPORT thread_local，0 值回落 1280×800（未接线宿主，g49
  前行为）。缓存键无需视口分量：该槽 shim 安装期一次性写入，产品
  resize 走 `__zw_user_resize` 不经此槽（doc 注释在案）。
- GCS 四站全部换读该真值；`compute_styles_for_doc` 折算 (vw,vh) 复用
  （collect 与 set_viewport 单次读取）。
- 渲染面 `<style media>` 门穿参：`collect_stylesheets` /
  `collect_stylesheets_with_parsed_external` / `style_sheet_if_eligible`
  / `collect_interleaved_external` / `collect_shadow_inline_sheets` 增
  `viewport: (f64, f64)` 形参，六主管线调用点 + blob/交错/shadow 内部
  链全部传 `(self.viewport_width as f64, self.viewport_height as f64)`；
  shadow 内联链读全局 `gcs_viewport_dims()`（签名免穿，
  `compute_styles_for_doc` 六调用点链路不扩散，doc 注释在案）。
- 双面同源：GCS 门与渲染门同一视口值，media query 判定不再分裂。

## 3. 单测与门禁

- engine lib（release）**4,026P/0F**，与刀前基线逐位一致（含
  part111/115 set_pending_layout_viewport 探针页）。
- fmt/clippy（-D warnings，engine + runner）净；孤置 doc 注释
  （GCS_VIEWPORT_W/H 常量遗痕）清除。

## 4. 语料 A/B（anchor B0=results-g49-b0 新建首基线 / B1=results-g49-b1）

B1 = 刀后 release 二进制（mtime 07:35 > HEAD a3b89b387 05:39）；
B0 = pre-knife 快照 /tmp/zw-runner-g49-b0（provenance = g47-b2 同源
build 05:42）。三语料 exit=0（07:36–07:44）。

| 语料 | B0 | B1 | Δ |
|---|---|---|---|
| css-values（256 页 5593 案） | 4113P/1474F/6T | **4185P/1402F/6T** | **净 +72，零回归** |
| css-images | 3144P/71F | 3144P/71F | ±0 |
| css-anchor-position（首跑 12,857 案） | 12061P/792F/4T | 12061P/792F/4T | 逐案一致，零位移 |

- viewport-units-css2-001 **84F→12F**（+72 FIXED，其余 255 页逐案
  不动）。残 12F：border-spacing ×4 + vertical-align ×8（含 gCS 非零
  px 断言）——per-property computed 串化面（表格内距/基线相对属性），
  非视口解析错位，本轮范围外。
- viewport-units-invalidation 24F 不动（B0=B1）——案级消息全为
  `win.getComputedStyle is not a function`：iframe
  `contentWindow.getComputedStyle` 未桥接（缺能力面，未走到视口求值），
  另行挂账。
- anchor 语料 B0=B1 逐案一致——刀对非视口面零位移。

## 5. 结论与下一步

视口相对长度解析基准归位（GCS/渲染/shim 三面同源 800×600）。残面
队列：calc-size 族（~250F）、signs-abs z-index/%-used-value（94F）、
minmax 写门（~94F）、url-request-modifiers（76F）、random/if-cycle、
css-images 属性族（object-fit/image-orientation 等 ~42F）、
viewport-units 残 12F per-property 串化 + invalidation 24F iframe
contentWindow gCS 桥。
