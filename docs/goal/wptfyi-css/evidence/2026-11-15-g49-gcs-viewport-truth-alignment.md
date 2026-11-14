# g49 GCS/媒体门视口真值对齐刀

2026-11-15。driving: css-values viewport-units-css2-001 单页 **84F**——
runner 视口 800×600（`WebViewConfig{width:800,height:600}` + shim 安装期
`set_pending_layout_viewport` + `__zw_user_resize`），期望值按
`documentElement.clientWidth`=800 派生（10vw 应 80px）；而 gCS 快照通道
四处硬编码 `set_viewport(1280, 800)` → `10vw` 折算 128px 全列错。

## 1. triage

- **两通道视口分叉**：渲染管线 `<style media>` 门在
  `pipeline/mod.rs::style_sheet_if_eligible` 也硬编码 1280×800（树内
  FIXME 在案）——同一门 GCS 侧若只修 gCS 会造成 gCS 判 media 通过而渲染
  仍按 1280 判（或反向），必须同刀双面统一。
- `<link media>` 已视口正确（shim s161 overlay 门把 link 包成 @media 流经
  style-system viewport），无需动。
- 视口真值源：`PENDING_VIEWPORT` thread_local（s421 引入，shim 安装时装
  一次；产品 resize 走 `__zw_user_resize`/`set_viewport` 不经此槽）——
  **cache 键无需视口分量**：槽内值 shim 安装后恒定，不构成代际输入。

## 2. 实现（engine 5 文件）

- `computed_style_cache.rs`：新增 `gcs_viewport_dims()`——读
  `PENDING_VIEWPORT`，0 值回落 `(1280.0, 800.0)`（未接线宿主与旧行为
  一致）。
- **gCS 四站**去硬编码：`compute_styles_for_doc`
  （computed_style.rs）、`computed_style_cache` 装配代、
  `length_to_css` 兜底臂、`calc_computed_to_css` CalcContext——全改走
  `gcs_viewport_dims()`；`GCS_VIEWPORT_W/H` 常量删除（孤置 doc 注释
  同步清）。
- **渲染管线链穿参**：`collect_stylesheets` /
  `collect_stylesheets_with_parsed_external` / `style_sheet_if_eligible`
  （media 门 `MediaContext::new(viewport.0, viewport.1)`，FIXME 换 g49
  注释）/ `collect_interleaved_external` 全链加
  `viewport: (f64, f64)` 参数；blob 路径与交错路径内部接线同步。
  shadow 内联链（`collect_inline_shadow_scopes`）签名免穿——函数头读
  `gcs_viewport_dims()`（compute_styles_for_doc 六调用点链路不扩散；
  resize 后 shadow 媒体门取 shim 安装期视口，可接受近似，文档面链路
  为准绳）。
- 六个管线调用点 + `pipeline_budget.rs` 两点传
  `(self.viewport_width as f64, self.viewport_height as f64)`（管线会话
  resize 后媒体门随动）；两处测试点传 `(800.0, 600.0)`（对齐 runner
  视口）。

## 3. 单测与门禁

- engine lib **4,026P/0F**（release，exit=0）——与上轮基线持平，含
  s421/s421-adjacent pending viewport 钉与 gCS 全量钉。
- fmt clean；clippy zero-engine + zero-wpt-runner `--all-targets
  -D warnings` 净（修一处删常量遗留的空行 doc 注释）。

## 4. 受控 A/B

B0 = results-g47-b2（css-images/css-values，g48 提交态二进制）+
results-g49-b0（css-anchor-position anchor 首覆盖，pre-knife 快照
05:42 = a3b89b387）；B1 = results-g49-b1（刀后 07:35 唯一内容差 =
本刀 5 文件）。ab-g42.py 三向：

| corpus | B0 | B1 | 净 |
|---|---|---|---|
| css-values | 4113P/1474F/6T | 4185P/1402F/6T | **+72P/−72F** |
| css-images | 3144P/71F | 3144P/71F | ±0 |
| css-anchor-position（anchor 首覆盖，265 页/12857 案） | 12061P/792F/4T | 同 | ±0 |

- grand fixed +72 regressed **0** net **+72**；案数恒同（5593/3215/
  12857）。逐页唯一变化页 = viewport-units-css2-001（F 84→12，fixed
  +72），其余 561 页逐案零位移——刀面精确命中预期（视口单位 gCS 面）。
- css-images 全持平确证残面均非视口依赖（object-fit/image-orientation/
  image() 等族），anchor 语料 12857 案零回归 = 渲染 media 门穿参无附带
  误伤。

## 5. 残面挂账

- viewport-units-css2-001 残 12F（160 案已过）——案级定谳（B1 JSON 逐案
  核对）：`border-spacing` × vw/vh/vmin/vmax 4 腿 + `vertical-align` ×4
  （含 computed 读回非零断言）8 腿，全部普通视口单位，无 svh/lvh/dvh
  动态单位亦无滚动条因子——两属性槽对视口长度丢弃/清零的承接缺口，另刀。
- viewport-units-invalidation 24F：**per-iframe viewport**（iframe 自身
  视口独立求值，本刀只对齐文档面单视口）——独立刀。
- viewport-units-scrollbars-mq-001 1F、scrollbars-root-properties-001
  1F（既有，非本刀回归）。
- 后续刀队列不变：calc-size 族 ~250F、signs-abs z-index/%-used-value
  94F、minmax 写门 ~94F、css-images 属性族 ~42F、url-request-modifiers
  76F、if() 38F。

## 6. 运行记录

- runner 重建 41s；corpus systemd 单实例（flock + 同树 cargo/rustc 守卫
  + 唯一 transcript；flock 拒绝了本 session 意外重复发射——上 session
  发射的 service 存活跨 session 接续）。
