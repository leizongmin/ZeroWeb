# M2 s304 — the-img-element reftest 簇勘根+施工收口：mutation 沙箱同步 fetch 桥与 page_url 真实化（运行时插入图 2 页 F→P）＋ build_image_cache wpt.test 绝对 URL 面＋语料 4 资产补齐（净 +2 F→P、P→F 全 pool 裁定绿）

日期：2026-11-06。切片号 s304（承 s303 下一步候选①）。s303 预告「8 红页簇
勘根」在本轮升格为勘根→施工→验证完整切片：8 红页逐页定谳后，其中 2 页
同根（mutation 沙箱无 fetch 桥）且根在本通道 runner 侧，当轮施工收口。

## 勘根（s300-b2 基线 8 红页逐页定谳）

1. **available-images.html ✗1.46% → ✓0.00%（本轮修复）**：runner
   reftest 通道 `apply_scripted_dom_mutations` 脚本沙箱只注册 DOM 回调
   （callbacks.rs:572 `register_dom_callbacks`），**未注册 `__zw_fetch`**；
   shim fetch 落 `__zw_fetch_error:no-handler` 同步settling（part01.js
   :6063 R34xx 契约）→ 页内 img 臂 `if (!resp.ok && !resp.status)
   { _imFail(); }`；叠加沙箱 `page_url=about:blank` 使相对 URL 语义失真。
   可用图列表页的 detached `new Image()` 预载链（fetch→blob→
   createImageBitmap→onload）永不派发 → **运行时插入的 i2 不入
   mutated_html** → 与 ref 少一只图。
2. **img-with-containment-and-size.html ✗4.17% → ✓0.00%（本轮修复）**：
   同根运行时插入面（`i.src="image.png"` 由脚本赋值后插入）——**非**
   containment 渲染面（初勘假设否证：containment 盒在载入期已正确）。
3. **image-compositing-change.html 基线 ✓**：本轮 page_url 真实化引入
   回归后同轮修复（见施工 c）——C 轮 ✗1.17%（修复前二进制，确定性）→
   D 轮 ✓ + solo ✓✓✓ 0.00%。净零。
4. **adopt-from-image-document.html ✗16%**：image document 合成
   （`img.src` 指向 .html 文档→img 加载行为=文档 body 合成渲染）引擎
   全域零代码（grep image_document 无产品面）——本域最大剩余施工候选。
5. **animated-image-update.tentative.html / animated-webp-update
   .tentative.html ✗2.08%×2**：GIF/WebP **帧推进**缺失——
   render-foundation `decode_gif_bytes` 仅首帧（s300 先例：image_cache.rs
   面允许本域施工）；ref 为 resources/green.png 静帧。s300-b2 基线即
   ✗2.08%（语料资产缺席时同为 2.08%——失败模式从占位形态变为可观测
   首帧差，态未翻）；solo 复测 ✗2.08%（10000/480000，threshold=
   1.00%/5ch）定谳。
6. **svg-img-with-external-stylesheet.html ✗2.08%**：SVG-as-img 内
   xhtml `<style>` 被 usvg 忽略 → SVG 页透明方（100×100/480000 整）；
   **SVG CSS 消费面归 rendering-compat 移交**（非本通道）。
7. **responsive-image-select-print.html ✗8.33%**：print 仿真臂 +
   srcset w 描述符（`red.png 1w, green.png 200w`）选择语义；语料缺
   resources/red.png（本轮补齐）后剩余 print 面未深探。
8. **image-srcdoc-relative-uri-print.html ✗1.04%**：print 仿真 +
   srcdoc 相对 URI 基面，未深探。

## 施工（runner 侧三文件）

**a. reftest_scripts.rs `apply_scripted_dom_mutations`**：
- 沙箱 `page_url` 由 about:blank 换真实页面 URL——`wpt_root`×`base_dir`
  双 canonicalize 后 strip_prefix 还原 `/html/...` 相对路径，落
  `https://wpt.test/<rel>/`；root/dir 不同基无法还原时回落 about:blank
  维持旧行为（E0515：rel 归属化在 and_then 闭包内完成）。
- 注册同步 `__zw_fetch` 桥：`crate::testharness::
  reftest_mutation_fetch_handler(wpt_root)`（复用 testharness 通道
  fetch handler，无 stash 变体）→ 沙箱闭包内 `fetch_bridge` 解码
  method/url/headers/body（body_bytes raw 优先）→ `FetchRequest`
  （credentials/mode/redirect 按 arg 7/8/9）→ handler 同步
  `serialize_response` / `__zw_fetch_error:{e}`——shim R34xx 同步契约
  （part01.js:6063）使 fetch 立即 settle，img onload 链得以续跑。
**b. testharness.rs**：`pub(crate) fn reftest_mutation_fetch_handler`
  无 stash 包装（`wpt_data_fetch_handler_with_stash(root, None)`，进程级
  静态 stash 回落；不加 pub 于私签名防 ServeCustomResponseState 泄私）。
**c. resources.rs `build_image_cache`**：wpt.test 绝对 URL 接管——
  mutation 沙箱 page_url 真实化后脚本间 IDL 传值（`image.src=
  otherImg.src`）以绝对形态入 mutated html（image-compositing-change
  的 `original.src` 读回即此面）；剥
  https/http×wpt.test(:8000) 四前缀按站点根解
  `tests/wpt-runner/wpt-data/<path>`（query/fragment 剥除+percent-
  decode），**存在才接管**、落空回落既有解析；ImageKey 仍用原始 url
  哈希，painter 键面对齐零变化。

## 语料修复（4 资产，gitignored 本地数据）

- `the-img-element/resources/animated.gif` 292B（md5 ceb7ff0068d19fbe
  f657cc0f09434e44）、`animated.webp` 286B（md5 fe2ba7e9a9e024a91ee24e2
  d6def97b2）、`red.png` 100×100 PNG 510B（md5 b8ae7929d842f20ada510d
  963b741e1c）——上游 sparse clone pin `efcaa5bb87`（2026-11-03 master）
  取件；green.png 91B 为 11-02 既有非本轮。
- `common/reftest-wait.js` 5616B（md5 fbe764e11b37f04436f6a470ebc418e8）
  ——本地 common/ 为绕过 update-wpt-data.sh 补齐路径的手动填充残留（14
  文件独缺此件）；html+css 树 **354 页** `<script src="/common/
  reftest-wait.js">` 依赖，404 → takeScreenshot 未定义告警刷屏、wait
  机制全树失效。补后 0 告警。注意：脚本 `fetch_common_resources` 走
  unpinned master，该件内容随上游漂移，与 WPT_DATA_REF pin 语料有版本
  错位风险——已入 update-wpt-data.sh 挂账块（升级新 ref 后按条补入）。
- 4 资产入挂账：scripts/update-wpt-data.sh 已知套件打包缺口块 s304 条目
  （含 md5/provenance/「升级新 ref 后需重新补」）。

## 验证

- 门禁：cargo fmt 干净（首跑 make test 被 T58 fmt 前置门禁快速失败拦下
  ——当时 reftest_scripts.rs 未格式化，日志存证
  s304-make-test-fmtgate-abort.log，格式化后重跑）；clippy -p
  zero-wpt-runner --all-targets -D warnings 干净。
- 单测：`cargo test -p zero-wpt-runner reftest::` 102P/0F +
  `testharness::` 80P/0F（test-guard 包裹）。
- solo：solo-availimgs-fix ✓0.00%（available-images）；animated-gif
  solo ✗2.08% 定谳（10000/480000）；image-compositing-change 修复后
  solo ✓✓✓ 0.00%。
- 全树：**B**（语料补齐后/码前）15973P/1039F——vs s300-b2 全翻态可解释
  （1 R5430 二进制面 + 7 pool 键）→ **语料修复零净翻态**；**C**（码
  中间态）15979P/1036F；**D**（终态二进制）15979P/1036F。
- B→D key 级 join diff：**F→P×6**（available-images、img-with-
  containment 两真实 + tao-127/128/146/148 pool）；**P→F×3**（tao-149/
  tao-152/insert-inline-…-middle-002 **solo 3× ✓0.00% 全数直接裁定**，
  middle-002 另有同族 middle-003 三轮 ✓✗✓ 弹跳佐证）。**净真实 +2 F→P、
  零净 P→F**。
- C→D 同码翻态 6 键含 image-compositing-change ✗→✓（build_image_cache
  修复生效证据）+ 5 pool 族对向弹跳（净零）——族时序面与 s292 状态矩阵
  协议一致。
- D 轮含 3 zw-probe 诊断页键（p1/p4/p5 全 ✓，勘根用，已从语料删除，
  全树统计不受影响）。
- make test：终局 9958 行——workspace 腿照旧止于 engine lib 套件
  （**6F = 5 已知 CSSOM cssText 红 + r5296_single_line_inflated_lh_
  span_bg_box_level_content_anchored 1 例套件级 flake**）；r5296 solo
  filter 3× ✓ + engine lib 全套 solo 复跑 3763P/5F（r5296 ✓）裁定
  非确定性、非本 diff（runner-only 三文件，engine 测试与 runner crate
  无依赖）——engine paint 面**跨流注记**归 rendering-compat（rule 9
  记录不追），其余通道零新增红。
  **跨流注记（rendering-compat 输入）**：r5296 单测模块
  （paint/tests/r5296_inline_bg_ml_gate_tests.rs:80）套件级并发态
  flake，s300 全套 ✓→s304 全套 ✗→solo/全套复跑 ✓；s300→s304 间
  engine 套件 3763→3768（渲染流 R5428/R5429/R5430 落地伴随），归因
  需渲染流自勘（疑并发态污染，本域不追）。

## 证据

日志（evidence/logs/，md5）：
- s304-reftest-full-b.log a58944539e0017d1fc3aab2769fcc494（语料后/码前）
- s304-reftest-full-c.log 383c8d8f3bbc4cb8c74c3574eba9ca9b（码中间态）
- s304-reftest-full-d.log af74247d5fe4a0b4760b1ade4df8d535（终态）
- s304-reftest-solo-availimgs-fix.log 3f9c8d8138abbc1df366a7876b0e6585
- s304-reftest-solo-animated-gif.log 6f3d4a626721e34a6dd8d179fb023fc3
- s304-solo-adjud-{1,2,3}.log 0bb2d39bace5936b9d91ab9e4f8b1dfa /
  bee600c24de04372a906d254a508585b / 87189b5f532ebba8559015b4a138b3ee
  （FILTER=table-anonymous-objects 三连跑，tao-149/152 双绿）
- s304-solo-adjud-middle002-{1,2,3}.log 8980c52ec27e62d5483b2e3342ed0ddb
  / c560dc2f783d3ba24acf5ca80583da71 / 60aa0143dce8f35c1a2092fa33ecb3ed
  （FILTER=…-middle-002 三连跑 ✓0.00%×3）
- s304-make-test-fmtgate-abort.log 2f4ae071688da79a03ca1b03a94d449b
- s304-make-test.log bc4fd9343826b12506a5c976ddfefc57（终局 9958 行）

代码锚：reftest_scripts.rs（page_url 派生+__zw_fetch 桥注册）、
testharness.rs（reftest_mutation_fetch_handler 包装）、resources.rs
（build_image_cache wpt.test 接管）、shim part01.js:6063（R34xx 同步
契约）/:6155（fetch 实现）/:10670（reftest-wait no-op）、part03.js
:3596-3604（_zwAvailImgs/__zwImgSyncRuntimeTable 调用点）、
js_dom_bridge/callbacks.rs:572（register_dom_callbacks 无 fetch 桥）、
fetch_bridge.rs:83（FetchHandler）、webview.rs:4018/:4156（webview 自身
同步 __zw_fetch 对照面）。

## 下一步候选

1. **adopt-from-image-document 图文档合成**（本域 16% 最大钉面；
   `img` 加载 .html 文档→body 合成语义，涉 engine 页面管线新面，勘根
   立钉后按例施工）。
2. **animated GIF/WebP 帧推进**（render-foundation decode 域，
   s300 image_cache.rs 先例允许本域施工；两页 2.08% 同刀）。
3. print 仿真 faces（responsive-image-select-print 8.33% +
   image-srcdoc-relative-uri-print 1.04%；print 媒体仿真+srcset w
   描述符选择语义）。
4. the-bdi-element 13 红页簇归属勘根（bidi 隔离，疑渲染域移交）。
5. 小簇排队（ol 6/textarea 4/meter 4/input 3/link 3/embed 3/object 2/
   option 1/iframe 1/br 1/wbr 1）。
移交（rendering-compat 输入，随本轮注记）：svg-img CSS-in-SVG face
（usvg 忽略 xhtml `<style>`，svg-img-with-external-stylesheet 2.08%）。
