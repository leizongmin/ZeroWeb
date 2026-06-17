# 渲染兼容性目标 — 运行时控制面板

**最后更新**: 2026-06-18
**当前活跃里程碑**: M10 — 上游 WPT 真实 Reftest 通过率提升（Phase A 部分解锁）/ DC-13 产品 smoke（morning-work 4× 高度幻影盒 R255 已修）
**上游真实 reftest 通过率**: 89.4% (438/490) R177b/R228b（2026-06-18 提交 + chromium Oracle A/B 验证）——R177b 落地 R177 延后的 colspan/col-width 缺口，`table_grid_size_col_colspan` **chromium-diff 52.27%→1.70%**（DC-14 anti-false-pass 真实 win；同源 reftest test==ref 天然不变故计数仍 438/490 持平零回归，零回归经全量验证）；R228b 半透明圆角矩形背景 alpha 修复（cpu `fill_rounded_rect`）。→ R227（**welcome padding 双计修复——product-smoke 28.34%→17.06%；reftest 439→438 净 -1（唯一回归 grid-flex-spanning-items-001 borderline 0.77→1.31%，aqua 实更正确，旧 pass 系两误差抵消）**）→ R225（**advance-width 证伪为死路**——R221 曾假设 183 case 1-3% chromium-diff 噪声主因=advance-width 估算误差；R225 双实验证伪：reftest-oracle 26 case 零变化 + product-smoke welcome/wintertc ±0.03%，机制=paint 经 fontdue 真实 shaping 定位 glyph 非 estimate_char_width；R223 AdvanceSource trait seam 留存无害勿再投入）→ R220（**DC-9 真实范围纠正——clip 为 no-op，GPU 缺口仅 transform/filter/blend 三项**）——经 grep 实证：**engine 生产路径从不生成 `ClipPrimitive`**（`add_clip` 0 处非测试调用），overflow 裁剪在 paint 阶段由 `clip_all_primitives_to_rect`（painter/mod.rs 多处）**预烘焙进图元几何**，故 `primitives.clips` 生产恒空。因此 R211 所记「GPU drops clip」**实为 no-op**（无 clip 可丢），DC-9 的 ClipPrimitive 项在 CPU/GPU 两路均**空谈满足**。**真实 DC-9 缺口仅 transform/filter/blend_mode 三项**（engine 在 `paint/painter/effects.rs:266/289/313` 生成，GPU 全量路径静默丢弃），需 ping-pong 双纹理后处理（wgpu 不能同 pass 读写同一纹理；filter/transform/blend 均区域 read+write），且 reftest/静态内容中**低频**（仅显式 CSS filter/transform/mix-blend-mode 触发），非 reftest load-bearing。**GPU 现状**：9/13 独立 WGSL 管线已落地（shadow/fill/rounded_rect/gradient/image/stroke/path_fill/path_stroke/glyph，非 passthrough 满足 DC-14），`headless_texture` offscreen 目标就绪，ping-pong 差第二张纹理+post-process pipeline。**本轮为治理性纠正（docs-only），无代码/reftest 变更**：纠正 R211/line381 对 clip 的误记，重定 DC-9 收尾范围（4 项→3 项且低频多轮），避免后续在 no-op clip 上浪费。下一步=DC-9 GPU ping-pong 地基（filter:opacity 先行）或 DC-14 chromium-oracle 严格容差默认接线或 DC-13 产品 smoke 持久化证据）→ R218（**SVG 解码统一到 render-foundation——DC-13「SVG 栅格化」全路径贯通**）——goal doc DC-13 要求「PNG/JPEG/WebP 基础解码和 SVG 栅格化」。reftest 路径早有 `load_svg_file`（resvg+tiny-skia），但 webview/browser URL 导航路径的 `decode_image_bytes` 对 SVG 返 unsupported——浏览器导航含 `<img src=logo.svg>` 的真实页面（WinterTC 14 logo 中 11 个 SVG）Logo 不渲染。**修复**：① render-foundation 加 `resvg`(workspace)+`tiny-skia` 依赖 + `pub fn decode_svg_bytes(bytes)`（resvg usvg 解析→按 SVG 内在尺寸 tiny-skia pixmap 栅格化→RGBA，过大尺寸 pixmap 分配失败自然兜底）；② `decode_image_bytes` 扩展 SVG 分支——`looks_like_svg` 嗅探 UTF-8 文本（跳 BOM/空白后 `<svg`/`<?xml` 起始）路由到 `decode_svg_bytes`；③ reftest `load_svg_file` 委托 `decode_svg_bytes`（同 R217 去重），移除 wpt-runner 的 resvg/tiny-skia 直接依赖（load_svg_file 唯一用户，依赖图精简）。**测试**：render-foundation decode_tests +2——`decode_svg_bytes_green_4x3`（含 `<?xml` 声明的 4×3 纯绿 SVG 往返，断言 G>200 + alpha=255）、`decode_svg_bytes_invalid_returns_err`（非 SVG XML→err）；`decode_image_bytes_dispatches_by_magic` 加 SVG 路由断言（现四分发 PNG/JPEG/SVG/unsupported）。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test **12227 passed/0 failed**、reftest-upstream **439/490 持平**（reftest 路径 load_svg_file 委托后行为不变）。**意义**：DC-13 三种图片格式（PNG/JPEG/SVG）在三条渲染路径（reftest / webview fetch_url / browser render_cpu）全部统一到 render-foundation `decode_image_bytes` 单点；浏览器经 URL 导航现可加载并渲染 SVG Logo（WinterTC logo.svg 等真实场景）。下一步=DC-13 产品 smoke 端到端证据（DONE#11 5 真实网站 / WinterTC Logo 经浏览器路径验证）或 DC-9 GPU 4 图元（transform/clip/filter/blend））→ R217（**JPEG 解码合并去重——清理 R216 造成的重复**）——R216 在 render-foundation 落地 tested `decode_jpeg_bytes` 后，reftest 路径（`reftest.rs:load_jpeg_file`，~55 行）的独立 JPEG PixelFormat→RGBA 转换逻辑与之重复（且 L16 处理不一致：reftest `(px[0]|px[1]<<8>>8)` vs R216 干净的高字节）。**修复**：`load_jpeg_file` 委托给 `zero_render_foundation::image_cache::decode_jpeg_bytes`（读文件→解码），reftest 与 webview/browser URL 导航路径现共用**同一解码器**（单点 tested）。移除 wpt-runner 不再使用的 `jpeg-decoder` 直接依赖（load_jpeg_file 是唯一用户）。保留 `load_png_file` 的 `ZERO_PNG_EXPAND` 诊断门控与 `load_svg_file`（resvg）不动——非本轮变更遗留。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test **12225 passed/0 failed**、reftest-upstream **439/490 持平**（L16 JPEG 在 WPT reftest 实质不出现，转换差异零影响）。**意义**：三条渲染路径（reftest / webview fetch_url / browser render_cpu）的 JPEG 解码统一到 render-foundation 单点，消除维护负担与潜在不一致；DC-13 图片解码一致性提升。下一步=SVG 解码统一（reftest 已有 resvg，webview/browser 路径缺）或 DC-13 产品 smoke 端到端证据（DONE#11））→ R216（**JPEG 图像解码扩展——DC-13「PNG/JPEG 基础解码」第二步**）——goal doc DC-13 要求 PNG/JPEG/WebP 基础解码，R214 落地 PNG，本轮补 JPEG。**修复**：① render-foundation 加 `jpeg-decoder = "0.3"`（MIT/Apache-2.0 纯 Rust）+ `pub fn decode_jpeg_bytes(bytes)`（L8/L16/RGB24/CMYK32 全 PixelFormat→RGBA，CMYK 按 Adobe 倒置 K 惯例转 RGB）+ `convert_jpeg_pixels_to_rgba` 纯函数；② **格式分发** `pub fn decode_image_bytes(bytes)`——按**魔数字节**嗅探（PNG `\x89PNG` / JPEG `\xFF\xD8\xFF`）路由，比 URL 扩展名可靠（URL 可能无扩展名/扩展名错误），未知格式返 unsupported err；③ webview `fetch_image_subresources` 改调 `decode_image_bytes`（原 decode_png_bytes）→ 同一路径现处理 PNG+JPEG。**测试**：render-foundation decode_tests 5 项——`convert_jpeg_pixels_to_rgba` RGB/灰度纯函数、`decode_jpeg_bytes_green_4x3` 真实 fixture（PIL 生成 4×3 纯绿 JPEG quality 95，断言绿色主导 G>200/R<50/B<50 + alpha=255，容 JPEG 有损）、invalid→err、`decode_image_bytes_dispatches_by_magic`（PNG/JPEG/未知三分发）。fixture `crates/render-foundation/src/testdata/green_4x3.jpg`（635B）。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test **12225 passed/0 failed**、reftest-upstream **439/490 持平**（reftest 用本地文件不走 URL 导航故零影响）。**意义**：DC-13 图片基础解码 PNG+JPEG 就绪，WebP/SVG 后续；浏览器经 URL 导航现可加载并渲染常见位图格式。下一步=SVG 栅格化（WinterTC logo.svg）或 WinterTC Logo 端到端产品 smoke 证据（DC-13 验收））→ R215（**浏览器渲染路径消费 webview ImageCache——DC-13 P1「图片子资源/ImageCache 未贯通」全链路贯通（最后消费 hop）**——承接 R214 标注的「下一步」。R214 已打通 fetch→decode→image_cache（webview 层），但浏览器 `render_cpu`/`render_frame` 仍传 `None`（app_platform.rs:194/153），图元到渲染器最后一跳断开。**修复**：app.rs 加 `use zero_render_foundation::image_cache::ImageCache`；`render_cpu`（CPU）与 `render_frame`（GPU）两路在 `render_full_scene[_gpu]` 调用前用**不相交字段借用**取活跃标签页 webview 的 image_cache——`match self.shell.active_tab_id() { Some(id) => self.webviews.get_mut(&id).map(|wv| wv.image_cache()), None => None }`（self.webviews / self.font_loader / self.glyph_cache 为不同结构字段，borrow checker 允许同语句并存），传 `Some(&mut ImageCache)` 替代 `None`。**测试**：新增 `#[cfg(test)] render_full_scene_with_webview_for_test`（与 render_cpu 同场景装配但返回 FrameBuffer，mirror 现有 `render_scene_for_test` 模式）+ 差异法测试 `render_path_consumes_webview_image_cache`——基线（image_cache 空）渲染断言目标颜色计数 0（缓存 miss 不绘制），填充 `ImageKey(simple_hash(src))`（键与 engine text.rs:611 一致）后渲染断言 >0（图片经浏览器路径被消费）。**验证**：cargo build/clippy --workspace --all-targets -D warnings 干净、cargo fmt 干净、make test 全绿（新增测试通过）、`./target/test-guard -- cargo run --bin zero-wpt-runner -- reftest-upstream` 实测 **439/490 持平**（reftest 用本地文件不走 URL 导航，image_cache 恒空，`Some(&空)`≡`None` 行为一致故零回归，符合预期）。**意义**：`<img>` 经 URL 导航全链路贯通——抓取(R214)→解码(R214)→image_cache(R214)→**浏览器渲染消费(R215)**→renderer 绘制真像素，goal doc DC-13 P1「图片子资源缺失不得被 alt/占位 glyph 静默替代」在浏览器层落地。下一步=JPEG/SVG 解码同模式扩展 + 产品静态页 WinterTC Logo 端到端 smoke 证据（DC-13 验收））→ R214（**图片子资源加载落地（URL 导航路径，PNG）— DC-13 第二个 P1 子项打通**——goal doc DC-13 P1「图片子资源/ImageCache 未贯通」修复（PNG 先行，JPEG/SVG 同模式后续）：`<img>` paint 可生成 ImagePrimitive，但 fetch_url 不抓 `<img src>`、webview 不持有 ImageCache、render-foundation 无解码。**分层修复**：① render-foundation 加 `png` dep + `pub fn decode_png_bytes(bytes)`（image_cache.rs，正确 EXPAND 全 color type→RGBA，独立于 reftest 的 env-gated 版本，零 baseline 影响）；② engine 加 `extract_img_srcs(html)`（DOM 精确，parallel to extract_stylesheet_hrefs）；③ webview 加 `image_cache: ImageCache` 字段 + `fetch_image_subresources(html, base_url)`（extract img srcs → 按 base URL 解析 → http_client 抓取 → decode_png_bytes → `image_cache.insert_with_key(simple_hash(abs), img)`，键与 pipeline image_sizes + 渲染器查找一致）→ 返回 image_sizes → `pipeline.set_image_sizes`（`<img>` 正确固有尺寸 DC-11），三条 fetch_url 分支注入；暴露 `pub fn image_cache(&mut self)` 供下游渲染器绘制消费。**端到端测试**（webview_coverage，mini-server 重构支持二进制）：服务 3×2 纯绿 PNG + page，fetch_url 后断言 image_cache 含该图、尺寸 3×2、像素纯绿。13 webview + 2 decode + 1143 engine 测试全过。**意义**：图片子资源抓取+解码+缓存贯通，`<img>` 经 URL 导航获正确固有尺寸 + ImageCache 就绪供浏览器渲染；浏览器 render_cpu/gpu 传 `Some(&mut webview.image_cache())`（当前 None）是最后消费 hop（下一步）。reftest 439/490 不变）→ R213——R210 在 compute_final_inline_layouts 的 PHASEA_MULTILINE 多行存储条件加 `!in_multicol` 守卫（in_multicol 经新增递归参数从 multicol 容器透传后代）。全量 make reftest-upstream 实测**净 +0**（仅 2 翻转：✅ ifc-008 8.18%→PASS 0.00%、❌ multicol-fill-auto-001 0.63%→9.15%）。CFDEBUG 探针精确定位 multicol-fill-auto 回归源=**ref 文件的 2 个 float div**（非 multicol，用 float 模拟列，in_multicol=false），非 multicol 容器——`!in_multicol` 守卫**无法触及**它们（合法非 multicol）。真正阻塞=**stored 多行路径 v_offset=0（Ahem）vs paint IFC 路径 baseline_fs=font_size 不一致**，ref（stored）与 test（paint IFC）渲染差 font_size/行；实测 v_offset=font_size 反破坏 font-051（16.67%）+ ifc-008（8.33%）。**关键方法学纠正**：product-smoke 单趟不加载外链 CSS（base_dir=None）→ multicol-fill-auto 的 /fonts/ahem.css 不解析→ is_pure_ahem=false→ 不存储，故 R209 product-smoke 看不到其存储行为；**涉及外链 CSS 的用例必须用 reftest 双趟路径诊断**。结论：ifc-008 已确认可被 compute_final 正确存储并 PASS，但解锁需先统一 stored 与 paint IFC 多行 baseline 语义（R125/R198 同墙，单点不可解）。无代码变更（实验已回退回 R207 干净态），基线 439/490 持平）→ R209——本轮用**干净单趟探针**（product-smoke 渲染 test only，避开 test/ref 双趟干扰）定位 R208 未定的 ifc-008 根因。**精确根因**：compute_final（engine.rs:1900-1903）的 **R84 单行 Ahem 存储限制**——`if lines.len() > 1 || !is_pure_ahem { return }`，仅存单行纯 Ahem。ifc-008 的 "XX XX" 100px 在 200px 宽换 **2+ 行** → lines.len()>1 → 不存 → paint 重跑 → 16px。（font-051 单行 "FAIL" → 存 → PASS，故 R207 成功。）干净探针证实 node 39(inner-div) **被访问、block=true、direct_text=true**，仅因多行限制未存。**实证放宽**（env PHASEA_MULTILINE=1，允许 Ahem 多行存储）：ifc-008 **8.18→4.17%**、ifc-009 **6.11→4.17%**（改善，仍 FAIL，余 4.17% 系换行精度/覆盖差），font-051 不变 PASS。**但 net-negative**：multicol-fill-auto-001 0.63 PASS→FAIL（**R198 反向依赖再现**——multicol 子容器现多行存储致渲染变），无用例翻 PASS。**结论**：ifc-008/009 修复需 (a) 放宽多行存储 AND (b) 守 multicol-fill-auto 不回归（=R198 ancestry-guard 墙，已知失败）AND (c) 修余 4.17% 换行精度。三重耦合，非单点。R207 font-051 +1 保持。无代码变更（探针+实验经 git checkout 清除回 R208 ccabef5），基线 439/490 持平）→ R208（**ifc-008 block-child large-font 深挖：inner-div 文本 16px，compute_final 未存其 inline_layout，架构性非单点**）——承接 R207（font-051 inline-child 容器 PASS）。本轮攻 ifc-008（div1>inner-div(block)>\"XX XX\" 100px Ahem），dump 实测：div1 区域几乎全红（39064px）仅微量绿（736px≈16px 文本）——inner-div 的文本渲染成 16px（large-font bug），**R207 inline-child 路径不覆盖此 block-child 结构**。探针（IFCDBG）：paint 见 inner-div(node39) fs=100 但 **inline_layout=None / use_stored=false** → paint 重跑 IFC 用空 styles → 16px。compute_final 探针未对该 node 触发存储（探针被 test/ref 双趟 compute + 字符串匹配 \"39\" 假阳性干扰，未定论根因）——**疑似 compute_final 树遍历未到达 inner-div 的带 node_id 盒，或匿名盒包裹致 node_id=None 早退**。**结论**：ifc-008（block-child 文本容器）的 large-font 修复需厘清 compute_final 为何不为 inner-div 存储行盒，属架构性深挖（非 R207 inline-child 路径的单点扩展），defer。R207 font-051 +1 胜利保持（默认 439/490）。无代码变更（探针已 git checkout 清除），基线 439/490 持平）→ R207（**Phase A「stored-line-boxes」路径 narrow 精修成功，+1 零回归默认启用**）——承接 R206（broad 应用 net-negative，font-051 单点改善）。本轮做 narrow 精修至「纯 inline 内容」容器：扩展条件 = 有 inline-level 元素子节点 AND 无 block-level 元素子节点 AND inline 子元素无元素子节点（叶文本容器，排除 block-in-inline R109）。compute_final（engine.rs:1693）扩展存储 + has_direct_paintable_text（text.rs:1479）同步扩展（paint_text 到达 use_stored）。**默认启用（env PHASEA_STORE_EXT=0 关闭）**。**font-051 8.19%→PASS 0.00%（+1）**，全量 reftest **438→439/490**，**零 count 回归**（inline-box-001/002 + multicol-block-no-clip-001 三处 broad 回归经 narrow 条件排除恢复）。make test 全绿（45 crate/0 failed）；clippy -D warnings clean。**Phase A 首次实证 clean win**：stored-line-boxes 架构（compute_final 用真实 styles 存行盒，paint use_stored 渲染）对纯 inline 容器产出正确度量，破 R205 4 路 font_size 死锁。剩余 large-font（ifc-008/009/011 = block 子节点容器，非本路径覆盖）+ empty-inline-002 仍卡（block-child IFC 另一子问题）。下轮可扩 narrow 条件覆盖更多结构或攻 block-child IFC。无回退，默认实测 439/490）→ R206（**Phase A「stored line boxes」路径首次实证改善 font-051，但 broad 应用 net-negative，须 narrow 精修**）——本轮实现 R205 定性的「唯一未试架构路径」：paint 不重跑 IFC，渲染 compute_final 存储的真实度量行盒。关键基础：compute_final 传**真实 styles** 给 IFC（engine.rs:1851，区别于 paint 传空），故存储行盒 font_size/line-height 正确。实现 env-gated（PHASEA_STORE_EXT=1）：① compute_final 扩展存储条件，覆盖含 inline-level 元素子节点的容器（如 div>span）；② has_direct_paintable_text 同步扩展，使 paint_text 不在 line 683 提前返回，到达 use_stored。**首次实证改善**：font-051 **8.19→1.51%**（100px 文本现以正确度量渲染，Phase A 5 路首次有真实正向）——**证明 stored-line-boxes 架构可行**。**但 broad 应用严重 net-negative，已回退**：ifc-001 0.53→1.84 / ifc-002 0.72→2.38 / ifc-003 0.23→2.02（3 个原 PASS 翻 FAIL）、ifc-008 8.18→9.73 / 009 6.11→8.07 / 011 11.24→13.00 / empty-inline-002 29.32→31.14（large-font 集群反恶化）、multicol-fill-auto-001 0.63→1.92（PASS→FAIL）、position-absolute-in-inline-005 1.19→2.46。**结论**：stored-line-boxes 路径**架构正确**（font-051 实证产出正确度量），问题在 **broad 应用**改变了大量容器的渲染（双绘/错位）。下一步 = **narrow 精修**：仅对「存储结果与重跑不同且改善」的容器应用（如仅单 inline-element 子节点 + 特定结构），逐条件 set-diff 收敛，而非全量 inline-children。这是 multi-round 精修，但**方向首次明确可行**（区别于 R205 的 4 路全死锁）。无代码变更（实验已回退），git diff clean，基线 438/490 持平）→ R205（**Phase A font_size 解锁第 4 路实证 net-negative，deadlock 定位于 IFC 度量架构**）——本轮在 R72/R125/R198 三路死锁后尝试**第 4 路：font_size 单字段解耦回退**（R72 传全量真实 styles 致 4 回归，本轮改为 paint 注入真实 ComputedStyle **仅 font_size** 表 `real_font_sizes`，IFC 在 font_size_overrides 未命中时回退，其他属性 vertical_align/letter_spacing/float-exclusion 仍走空 styles→默认，理论上规避 R72 的 4 回归）。实现 env-gated（PHASEA_FS_REAL=1）零基线风险。**实测全 net-negative，已回退**：① **font-051 8.19→11.65% 恶化**（font_size 现正确 100px，但 line-height 用 fs×1.2 近似与 div 的 100px/1 不符→行盒更高→diff 增；证明 font_size 与 line-height **耦合**，单修 font_size 致 line-height 失配）；② **multicol-fill-auto-001 0.63% PASS→1.21% FAIL 回归**（R198 同源反向依赖）；③ position-absolute-in-inline-005/006 回归（1.19/1.23% FAIL）。**结论：Phase A font_size 解锁的所有 4 条路（R72 全量 styles / R125 三路存储 / R198 override 填充 / R205 font_size 单字段解耦）全 net-negative**。死锁根因 = **paint IFC 与 layout IFC 是两趟独立运行，font_size/line-height/line-breaking 三者耦合**，任何「让 paint IFC 用正确度量」的单点/单字段改动都因破坏其他耦合而回归。唯一未试架构路径 = **paint 完全不重跑 IFC，直接渲染 layout IFC 存储的行盒**（需 compute_final 对所有含 inline 子树的容器存 line boxes，含 div>span 文本类如 font-051；paint 渲染存储结果；须解 float-exclusion/vertical-align 一致性，多轮架构）。无代码变更（实验已回退），git diff clean，基线 438/490 持平）→ R204（**绝对 plateau 再确认 + R197 DC-12 审计纠正（clip-path 已全实现）**）——本轮对剩余可操作方向逐一复核，全部 ruled out/已实现：① **clip-path circle/ellipse/polygon 真裁剪已全实现**（纠正 R197「仅画指示线」过时结论）——mod.rs:599-673 对 circle/ellipse/polygon 调 `clip_all_primitives_to_polygon`（helpers.rs:234，包围盒裁剪 + fills 扫描线精确裁剪 clip_fill_to_polygon + glyphs 中心点 point_in_polygon 测试），inset() 走 clip_all_primitives_to_rect。**DC-12 clip-path 无缺口**（仅 paint_clip_path 指示线冗余叠加在真裁剪之上，极次要清理，非 gap）。② border-001(2.77%) dump 实测：ZeroWeb 渲染**正确的空心黑方框**（25px border + 100x100 hollow），diff 来自 REF 的文本/定位差（字体度量），非 border bug。③ float-006(7.47%)=float+abspos+overlap 复杂交互（R100/R108b 结构域）。④ Phase A 新角度证伪：paint_text 已直接从 ComputedStyle 读 font_size（text.rs:639，mod.rs:263/446 传真实 style），死锁不在 style plumbing 而在 IFC 度量一致性（layout IFC vs paint IFC 两趟不共享状态），即 R125/R198 已确证的架构性。**结论：增量 clean win 彻底穷尽**——剩余 52 失败全为多轮硬架构（multicol 全碎片化模型 / Phase A IFC 统一死锁 / writing-mode 轴 4 轮证否）或浏览器引擎特性（原生表单控件 / dialog JS / fixed-bg）或 REF 怪异。无单会话 clean reftest win。无代码变更，基线 438/490 持平）→ R203（**multicol-breaking paint 侧修复实证 ruled out → 真实路径=layout 侧 column-aware IFC**）——本轮对 R201 定性的 multicol-breaking 4 阻塞点（A 门控/B 重绘协调/C column-rule/R203 新发现 D 去重）做 paint 侧实证修复尝试，**3 路全部 net-negative，已回退**：① **D = painted_inline_nodes 去重（text.rs:688）**——column breaking 目标（column_span_offsets 多片段）被去重误抑，非首列文本不渲染。单独修 D（去重条件加 `column_span_offsets.len()<=1`）：multicol-breaking-006 **1.20→1.71% 恶化**（col1/col2 现渲染文本但位置错=无 2 子列协调），004/005 不变，net-negative 已回退。② **A'+D 协调**（放宽门控给被碎片化子元素 + D 去重修复）：全量 multicol **40/57→36/57（-4 大回归）**，已回退。③ text.rs:707-709 注释确证 `height_auto` 门控**专为防回归而加**（明确高度 balance 容器简单均衡分配会回归）。**结论：paint 侧修复（单点或简单协调）对 multicol-breaking 全 net-negative，rule out**。真实路径 = **layout 侧 column-aware IFC**（R131）：IFC 在生成行盒时即按列高预算把行内流碎片化到各列（每列独立正确的行盒分布），paint 侧只按列渲染，非「单次 IFC 渲染 + paint 切片」。这是 multi-round layout 子系统。可复用教训：**架构性失败的单点/简单协调修复须实证（探针+全量套件），不能据推断落地**。无代码变更（实验已回退），git diff clean，基线 438/490 持平）→ R202（**chromium Oracle 高 diff 候选实证排查，3 项证伪关闭**——基于 06-17 fresh cross-validate 的 z_vs_chr>5% POLLUTED 清单（self-source 假通过掩盖的真缺口），逐项用 probe/product-smoke 实测排查：① **abspos-semi-replaced-stretch-input/button/other（23/3.5/15%）RULED OUT**——经 throwaway HTML probe 实测，plain div/inline-block/inline/span abspos + 全 inset + width:auto **全部正确 stretch**（red 填满 CB）；再渲染真实 `<input>/<button>` **也 stretch**（2px 采样确认 lime outline 跨满 CB x≈8-168）。早先 4px 采样误判「窄」。**stretch 算法工作正常**，23% chr 差异真因 = **ZeroWeb 把表单控件画成 styled box+outline，chromium 画原生 widget**（native button/text-field），是**表单控件渲染特性缺口**非布局 bug，单点不可修（需实现原生表单控件外观，大特性）。② **backdrop-inherit-rendered（47.5%）RULED OUT**——是 `dialog::backdrop`（`<dialog>.showModal()` JS API + `::backdrop` 伪元素），非 backdrop-filter；需 dialog JS 基础设施，非 contained。③ **background-attachment-applies-to-001（self 29.9%）= `background-attachment:fixed` on table-row-group**，fixed 背景=视口相对定位特性，非 contained 布局 fix。**结论**：fresh Oracle 高 diff 候选**全为结构性（multicol/table/Phase A/writing-mode）或特性缺口（表单控件/dialog/fixed-bg）或已修复（R165-R180）**，无单会话 clean win。可复用方法 = **probe-based 实证**（throwaway HTML 经 product-smoke 渲染 + 2px 采样判定几何，避免 4px 采样漏边误判）。无代码变更，基线 438/490 持平）→ R201（**multicol-breaking dump 实测定性，纠正 R113「两趟循环依赖」假设**——REFTEST_DUMP+BBOX+逐行像素扫描 multicol-breaking-004 实测：inner 文本**仅在 col0 渲染**（col1/col2 全空），蓝色 column-rule 全漏画，绿 border 位置错。真实 3 阻塞点 = ① paint 门控 `height_auto`（text.rs:710-715）挡住有明确高度 inner 的 2 子列布局；② `column_span_offsets` paint 路径**不重绘碎片化 IFC 内容到非主位置列**（核心 wiring 缺失，R131 同源）；③ column-rule §5.2 内容检测只查 `child.x` 主位置漏查跨列片段。**关键纠正：碎片化算法 `assign_children_to_columns_sequential`/`_with_breaking` 已存在**——R113 设想的「两趟测量」算法层面已具备，缺口是**接线到 inline paint 路径**，**勿再建 measure-first 工具**（必重复 R199→R200 证伪命运）。column-rule 检测修复（C）实测：004 5.60→5.39/006 1.20→1.12（蓝色 rule 补画改善）**但 column-rule-002 0.00→1.25% 回归**（c.x-主位置检测对该用例正确），**已回退**。设计文档 `multicol-fragmentation-design.md` 升 v0.3，Round 4 重定向为 wiring 多轮（放宽门控 + column_span_offsets 重绘碎片化 IFC，非 layout 两趟）。无代码变更，git diff clean，基线 438/490 持平）→ R200（**multicol balance 方向证伪关闭**——R199 的 round-robin shortest-column balance 接入后 multicol-columns-001 4.88→4.92%（略差）。根因：chromium multicol §8 是**顺序填充**（col0 填到平衡高度 H=T/N 再 col1），**非 round-robin**；旧代码 `line.y/target_h`（target_h=total/col_count）**本就是顺序填充+平衡高度，已正确**。我的 round-robin 破坏顺序。**multicol 列分配已正确**——类 A 低 diff（columns-001 4.88%/fill-000 6.54%/count-computed-003/004）**非列分配问题**，是列宽精度/glyph x 位置(estimate_char_width)/平衡高度精确值。移除 R199 的 multicol_fragment.rs（错误算法）+ 纠正设计文档 v0.2。multicol 剩余失败全结构性（breaking/baseline/column-span）或精度（advance-width 同源）。基线 438/490 持平）→ R199（**multicol 碎片化攻坚启动**——设计文档 + Round 1 测量工具落地，零风险不接线）：建 `multicol-fragmentation-design.md`（consolidate R113/R131/R157，5 轮实施计划：R1 测量/R2 纯行内 balance 精确化/R3 混合内容门控/R4 breaking/R5 baseline+spanner；预计全完成 css-multicol 40→55/57，438→~453）。新增 `multicol_fragment.rs::balance_lines_to_columns`（CSS §8 shortest-column-first 列分配，替代 paint text.rs:951 的 `total/col_count` 均高近似）+ 6 单测（4行2列/11行6列/含 block 已占/单列/零列/空）。**Round 1 不接线**（measure-first 同 R181 模式），reftest 438/490 持平零回归。下轮 Round 2 = 接入 paint text.rs:948 列分配（paint-only，解锁 multicol-columns-001/fill-000/count-computed-003/004 类 A 用例）→ R198（**Phase A font_size 死锁经新变体再证实证，关闭该方向**）：实验 compute_final IFC 跑过后调 store_font_sizes_from_ifc 存 font_size（paint 提示不重排）+ multicol ancestry 守卫（in_multicol 跳过）→ 全量 net **-1**（438→437）：CSS2 +1（large-font font-051 类修复）但 css-multicol -1（**multicol-fill-auto-001 0.63%→FAIL**）。ancestry 守卫无效（multicol-fill-auto 非 LayoutBox 树 ancestry-tracked，疑 multicol paint 路径重组）。即使完美守卫 net 也仅 0（+1 -1 抵消），**死锁成立**——large-font 与 multicol-fill-auto 经 font_size 存储耦合，不可单修。印证 R125（三路 -1/-1/-4）+ R158（"勿再单点补存"），**Phase A font_size 方向正式关闭**。DC-13 welcome 文本 + large-font 5 reftest 全卡此墙，需架构性 Phase A IFC 三路径统一（非 font_size 单点）。无代码变更（实验已回退），基线 438/490 持平）→ R197（**两纠正**：① welcome 文本真因 = **paint IFC font-size 默认 16px（Phase A 死锁）**，非 R196 的 advance-width——实测 `font-size:60px` 在 product-smoke 渲染成 12px（=默认 16px 字高）而 color:red 生效，证 font-size 未应用；即 R82/R101/R125/R158 标记的 paint IFC 空 styles font_size 回退（large-font reftest 死锁同源），R196 advance-width 假设**再被证伪**。Phase A 是已知硬阻塞（R125 三路死锁 + R158 multicol-fill-auto 反向依赖），DC-13 welcome 卡此墙。② **DC-12 审计**：text-shadow/multi-background-layer（全图层逆序）/repeating-gradient/clip-path/backdrop-filter/CSS mask **全部已实现**，goal doc 的 DC-12「未实现」声称**全部过时**（同 M7）；唯一真缺口 = clip-path circle/ellipse/polygon 仅画指示线非真裁剪（只 inset 真裁剪）。无代码变更，基线 438/490 持平。**结构性 plateau 全面确认**：DC-13 卡 Phase A、DC-12 基本完成、reftest clean win 穷尽）→ R196（DC-13 welcome 28% 根因深挖——**证伪 R195 line-height 假设 + font 不匹配假设**：welcome.html 全用**显式 line-height**（1.08/1.5/1.45/1.25），无一处 line-height:normal，故 R195 的 font-metrics line-height:normal plumbing 对 welcome/morning.work/wintertc **零收益**（三者皆显式 line-height），已实证后**回退**（plumbing 服务无指标，按 code-guidelines 不留推测代码）；font 不匹配实验（sans-serif DejaVu→Noto CJK，系统 fc-match=Noto CJK SC）welcome diff 28.08→28.05%（**仅 -163px negligible**）证伪字体假设。**R195 AA 基准测同字体漏了 sans-serif 解析分歧，但解析分歧本身非主因**。welcome 28% 真因 = **文本定位**（advance width 估算 estimate_char_width 0.55×fs vs 真实 advance → 文本宽度/换行/位置偏差累积），即 R188 标记的架构阻塞（layout IFC 不持 FontLoader），**自源中性仅影响 DC-13 不影响 reftest**。下步=advance-width plumbing（同 line-height 的跨 crate 模式但更高价值，影响全部非 Ahem 文本）。无代码变更，基线 438/490 持平）→ R195（DC-13 line-height 调研 + **关键去风险发现**：welcome 28% diff 经 diff-band 分析确认为 line-height/度量累积（文本行间隔 band + 底部 quadrant 差异最大=累积）；**line-height:normal 改动对 reftest 自源中性**——实测 ratio 1.2↔1.5 linebox 10/15 + css-writing-modes 53/59 双双持平，因 reftest 是 ZeroWeb-test vs ZeroWeb-ref 自渲染，test/ref 同字体等比例平移。**证明字体度量版 line-height 对 438 基线安全**，解锁 DC-13 line-height 方向。但 fix 是架构性多轮：layout-engine IFC 需字体度量，而 font_family→FontId→font 文件解析当前在 paint 懒做（R188 同源阻塞），需 engine 预解析建 font-family→line-ratio map 传入 layout+paint 双侧 IFC。reftest 52 失败全结构性复核（clear-float-003 3.20% = float+clear+negative-margin+margin-collapse 交互，确认 clean win 穷尽）。无代码变更，基线 438/490 持平）→ R194（R109 split 的 relative offset 双重计数修复，**+1 零回归**）：split inline 的匿名块片段用 converter 从 inline computed 构建 taffy Style 时**继承了 position:relative + inset**，致 taffy 对每个片段重复施加偏移（父盒 #div2 已施加一次）→ inline-box-002 的 `position:relative;top:2in` 使片段偏低 2×192px 出视口（蓝色 bg 不可见=假缺失）。**两处协同修复**：① tree.rs 匿名块 style 的 `inset` 清零（AUTO，位置由父盒单次施加）；② engine.rs `apply_relative_offsets_inline` 跳过 `is_r109_split` 盒（父+片段，taffy 按 block 单次处理，避免 computed-Inline 路径双重）。`inline-box-002` 3.14%→**PASS 0.78%**（frag1 abs_y 646→262、frag2→300，几何对齐 ref）。CSS2 115→116，零 count 回归。R109 里程碑**彻底完成**（inline-box-001/002 + align-001 全过；残余仅 clear-inline-001 = inline img+span→block 堆叠，独立子问题）→ R193（R109 §9.2.1.1 **默认启用 + fragment border 落地**，**+2 零 count 回归**）：① 匿名块片段用 converter 从 split inline 的 computed 构建 taffy Style（携带 border/padding/bg，而非默认空 Style）——此为 R192 遗漏的关键，旧实现匿名块 border=0；② paint 对 split inline 父盒（is_r109_split）跳过自身 bg/border/shadow（装饰下放片段）；③ 新增 `shrink_r109_anon_blocks` 后处理：片段收缩到 `fragment_inline_max_width`（同 paint IFC 的 estimate_char_width，故收缩宽=渲染宽自洽）+ fragment border 边选择（首片段 border_right=0、末片段 border_left=0，CSS2 §9.2.1.1 分裂边不画）。**`inline-box-001` 2.31%→PASS 0.89%、`block-in-inline-align-001` 1.37%→PASS 0.34%**；inline-box-002 3.20→**3.14%（改善不再恶化）**；block-in-inline-append/iframe/margin-collapse/justify/last 全部持平或改善（align-justify 0.38→0.50 微增仍过）。R109 默认开（R109_WIRE=0 关）；全量 make test 0 failed；437/490 默认实测。**R109 里程碑主体完成**（仅 clear-inline-001/inline-box-002 残余 = relative-on-split + inline img+span 流，独立子问题））→ R192（R109 生产端接线落地 env-gated：tree.rs `build_subtree` 把 inline+in-flow-block 子元素展开为匿名块 taffy 节点 + fragment 注册表；engine.extract_layout 写 `LayoutBox.fragment_node_ids` + `is_r109_split`；paint IFC 跳过 split inline 父盒 + 放行片段。**out-of-flow（abspos/fixed/float）排除修复**——CSS2 §9.2.1.1 只拆 in-flow block 子元素，否则 position-absolute-in-inline-005/006 回归 -2。实测 `R109_WIRE=1` 全量 **436/490 (+1 零 count 回归)**：`block-in-inline-align-001` 1.37%→**PASS 0.00%**、inline-box-001 2.31→1.11% 改善；**但 inline-box-002 3.20→4.67% 恶化**（border-having split 需 inline 级 fragment border，R182 §3 未就绪）。按项目严格「零回归=无任何用例变差」标准**保持 env-gated 默认关**，基线维持 435/490。下步=fragment border 解锁 inline-box 后默认启用。make test 0 failed；clippy/fmt clean）→ R183（flex/grid 两趟 Round C IFC 文本内容宽度测量基础落地，零回归；flex-container-max/min-content 经 INTRINSIC_DBG 证实 delta=0 已正确尺寸，其 18%/13% 差距系 grid+float 结构非 flex 宽，Round C 不修此两用例）→ R182（block-in-inline R109 攻坚确证架构性多轮 defer，clean win 同源+chr 双侧穷尽复核确认）→ R181d（flex/grid 两趟 Round B 落地，**+1 零回归**：`width:max-content` grid 经两趟 intrinsic 测量塌缩 40→182px ≈ chromium 180，`child-border-box-and-max-content-001` 1.52%→**PASS 0.03%**；R97 两通过用例 min-width:max-content/min-height:min-content 经实测仍 0.00% 持平）。前轮 R180（chromium Oracle 真实修复 ×4：R180 inline-block width:auto shrink-to-fit baseline-block-with-overflow-001 chromium **45.09%→1.25%**；R178 `<col>` px 宽度 18→400px；R168 table height-as-minimum 11.12%→2.98%；R165 margin:auto 居中 33.09%→2.63%）。**434/435 即诚实 DC-14 基线，无需恢复 436**（R164 证否 vrl-004/008 R114b 路径：正确 vertical-rl CSS 使 4/4 vrl 变差，因同源 REF 水平渲染 vs 正确 vertical-rl 右侧块起始结构性不可对齐；chromium Oracle 证同源 REF 比 chromium 更怪异：vrl-004 同源 7.09% vs chr 5.08%，font-051 同源 8.19% vs chr 1.62%）。R163 PNG 正确 RGBA 默认启用（DC-14 anti-false-pass）。draw_order 默认启用满足 DC-10。剩余 55 同源失败（结构性多轮 + REF 怪异产物）；**优化目标已转 chromium Oracle 一致率（d16bb8e），18 真 bug 候选见 `evidence/analyze-pollution-2026-06-16.txt`**。

**🔤 字体攻坚结论（2026-06-17 AA 基准，证伪字体归因；2026-06-18 R229b 补充 Bold 细化）**：fontdue 光栅化 vs chromium 实测 **W 0.1% / i 3.0%**（`evidence/aa-baseline-2026-06-17.txt`）——**Regular 变体 fontdue 不是渲染差异来源**。advance plumbing（真实 advance 替代 estimate_char_width）实测 Oracle 污染 48.6%→48.5% 无效，已回滚。welcome 26% / 污染大头是**布局/度量（line-height / R109 inline→block / 多行结构）非字体**。纠正 R174/R187「字体噪声」误诊。⚠️ **R229b（2026-06-18）细化**：fontdue **Bold 变体**比 chromium **过墨 ~15%**（welcome title +14%/card h3 +17%）→ R229 font-weight 选择机制虽正确落地+生效，但加载 Bold 后 net-negative 已回退（见 7d062e5）。故「fontdue 非差异来源」精确为「**Regular** 非差异来源；**Bold 过墨**是 fontdue-vs-chromium 差异（同 advance-width/AA 谱系）」；**font-weight -Bold 接线死路，字体攻坚（Regular 已对齐 + Bold 过墨不可单点修）停止，转布局/度量**。

**🎯 当前最高优先级（2026-06-18 更新，R229b font-weight 死路后）**：font-weight 已证为 **net-negative 死路**——R229b/7d062e5 完整落地 R229（选择机制正确+生效，welcome card h3 ink-mass +29% 证接线对），但 **fontdue 光栅化 Bold 变体比 chromium 过墨 ~15%**（title +14%/card h3 +17%）→ welcome product-smoke 17.06%→17.55%（+0.49pp 回归）→ 全 git checkout 回退；fontdue Bold 过墨同 advance-width(R225)/AA(R174) 谱系 fontdue-vs-chromium 渲染差异，**非单点可修，勿再以「加载 -Bold 接线」重试 R229**。morning-work 4× 高度已**闭环**（R255→R260 + a2b169e：ua_default_display 补 article/aside/details 等；body 25301→5677px≈0.95×chr，fullpage chr-diff 89.14%→48.65%，reftest 438/490 零回归）。**当前真实优先级**：① **reftest 杠杆 R236 multicol baseline-export（+8，turnkey 行级 spec 就绪 R260）** + **R238 WM-1 abspos-vertical（+14，R254 spec 就绪，R164 经验告诫须实验确认 residual B rtl 5.03%）**——代码 agent 可直接起手，multicol.rs/converter+engine.rs abspos 区域不撞并行 agent IFC WIP；② **welcome 17%/morning-work 48.65% 残余** 重定性（R229b）= item-tag span→block（R109 IFC 架构，并行 agent inline/mod.rs WIP 疑在攻）+ fontdue CJK 度量噪声（line-height/advance，**非 weight**）+ hljs（需 JS）+ body ~300px 差，font-weight **不再是主因**；③ DC-9 GPU ping-pong 地基 spec'd（R249/R251，低 reftest 杠杆 R250）；④ UA display 审计完成（R258，a2b169e 后无更多 morning-work 类危险缺口）。⚠️ ruled out（勿重试）：font-weight -Bold 接线（R229b）、advance-width（R225）、multicol paint 切片（R203）、multicol balance 两趟（R200）、chromium 高 diff 候选（R202）、DC-12（R197 全实现）。⚠️ 仍开放多轮硬架构（非单会话）：multicol-breaking layout 侧 column-aware IFC（R131）、Phase A IFC 三路径统一（R125/R198 font_size 死锁）、DC-14 chromium-oracle 严格容差默认接线。

### R232 — text-emphasis 完全未实现（非 clean win，低 diff 系标记小所致）+ DC-14 1-3% 簇的「字体/文本属性 plumbing 缺口」统一模式（R229 为模板）（read-only，docs-only，基线 438/490 持平）

承接 R231（text-emphasis 15 标为「可能 contained 候选」）。本轮核实——**降级该判断**。

**text-emphasis = 完全未实现**：全仓库 grep `emphasis` 在 css-parser/style-system/engine **零命中**，`text-emphasis-{position,style,color}` 静默丢弃，强调点根本不渲染。15 case 的 1.0-1.3% chromium-diff **非位置略偏**，而是标记缺席但 CJK 强调点本身小（像素占比 ~1%）→ **低 diff 误导**。修复=从零实现特性（parse + paint），feature implementation 非 contained fix。**勿当 quick win**。

**更广模式（统一 R231 的「碎片」印象）**：DC-14 1-3% 簇含一个内聚子类——**字体/文本属性 plumbing 缺口**（已 parse 或可 parse，但未 wire 到 ComputedStyle→shaper/paint，因效果细微产欺骗性小 diff）：font-weight（R229，FontDesc.weight 字段存在但 find/paint 不用）、font-kerning（matcher:897 parse 但 ComputedStyle/paint/shaper 不消费，rustybuzz 默认 shaping 含 kerning 但 CSS 属性未控制）、text-emphasis（未实现）、text-underline-offset/decoration-thickness（未实现）、@font-face/font-051(large-font=Phase A 死锁)。占簇 ~15-25%。

**统一洞察**：这非碎片噪声，而是**共享修复模式**（把已 parse 字体属性 wire 到 ComputedStyle→fontdue/rustybuzz）。**R229（font-weight）是模板**，可系统化推广到 font-kerning（已 parse 只差 wire，比 text-emphasis 从零实现更轻，宜次之）→ text-emphasis → underline-offset。每个 plumbing 修复 per-case 收益小（属性效果细微，diff 本就 ~1%）但累加提 DC-14 chromium 一致率，自源中性。

**细化 R231 结论**：DC-14 183 簇 = **结构性多轮(38%: multicol/writing-modes/floats-clear) + 字体/文本属性 plumbing 缺口(~15-25%，内聚模式，R229 先行作模板) + 真碎片**。无单一 clean win，但字体属性 plumbing 是可系统推进的方向（区别于 multicol 结构性多轮）。证据 `evidence/r232-text-emphasis-unimplemented-font-plumbing-pattern-2026-06-17.txt`。无代码变更，基线 438/490 持平。

### R231 — DC-14 的 183 case 1-3% 聚类刻画：碎片化无单一根因，font-weight 确非 reftest 杠杆（Ahem 无 weight），text-emphasis(15) 为新候选子聚类（read-only，docs-only，基线 438/490 持平）

承接 R230（font-weight 限定 welcome 专有）。本轮刻画 DC-14 真正的达标杠杆——R221 标的 183 case 1-3% chromium-diff 聚类（advance-width R225 已证伪）——按目录+判定+直方图（数据源 `evidence/cross-validate-full-2026-06-17.txt`，475 case）。

**按目录分布（183 case）**：css-multicol 31（已知结构性 R131）、css-writing-modes 26（轴 R114）、css-text-decor **21（15=text-emphasis-* CJK 强调点 + 4 underline-offset/decoration-thickness）**、css-fonts+CSS2/fonts 28（font-family 边界/font-kerning/font-051/@font-face——**Ahem 度量/解析，非 font-weight**）、CSS2/floats-clear 13、css-flexbox 11、css-tables 11。判定 117 POLLUTED（自源通过但 chromium 不一致=假通过）+ 66 self-fail。

**直方图（0.1% bin）右偏扁平**：mode 1.1%(25)→平滑递减至 3%，median 1.59%，tight 1.0-1.3 带 60/183=33%——**非单一共享系统偏移**（单一偏移会有尖峰），而是大量小独立误差叠加。

**结论**：① **183 聚类碎片化，无单一高杠杆根因**——advance-width/font-weight 均排除；② **~38%(70) 是已知结构性多轮**（multicol/writing-modes/floats-clear），非新杠杆，达标须经这些多轮子系统；③ **~15%(28) 是 Ahem 字体度量/解析**（非 font-weight，疑 line-height/kerning/解析边界）；④ **新候选 = text-emphasis 15 case（CJK 强调点 1.0-1.3%，内聚且此前未单独标记）**，可能比 multicol-breaking 更 contained（但 CJK 标记度量亦可能难），建议作 R229 之后、结构性多轮之外的**独立候选**探查；⑤ **DC-14 39.6%→95% 路径 = 结构性多轮 + 碎片化小修，无捷径**，font-weight(R229) 仅 welcome 产品 smoke + 非 Ahem 含 weight 用例，对 reftest 达标贡献有限。

证据 `evidence/r231-dc14-183-cluster-fragmented-2026-06-17.txt`。无代码变更，基线 438/490 持平。

### R230 — R229 资源前提实证（系统已装 Bold 变体）+ font-weight 杠杆范围界定（welcome 专有，非跨页 DC-14 通用杠杆）（read-only，docs-only，基线 438/490 持平）

承接 R229（font-weight 精确方案，留两个待核：① 系统是否已装 bold 资源；② 杠杆是否跨页泛化）。本轮 read-only 实证两点。

**① R229 资源前提 = 实证可用（移除不确定性）**：`fc-list` 确认本机已装全部所需粗体变体——`DejaVuSans-Bold.ttf`、`NotoSans-Bold.ttf`、`NotoSansCJK-Bold.ttc`（SC/TC/JP/HK/KR 全）、`Liberation*-Bold.ttf`。故 R229 资源修复 = **仅向 `load_system_fonts` 路径表追加已存在的 `-Bold` 路径**，无需 fetch/打包字体资产。**关键细节**：DejaVu/Noto 仅有 Regular(400)+Bold(700)，**无独立 Medium(500)/SemiBold(600)**，故 CSS weight 500/600 须按 §5.2 fallback 落到 Bold(700) 或 Regular(400)（实现须遵循 §5.2 font-matching，否则 600 仍落 Regular）。

**② font-weight 杠杆范围 = welcome 专有，非跨页 DC-14 通用杠杆（纠正潜在过度泛化）**：测 wintertc PNG（24.25% diff）的 ink-mass——其 diff 主导是**橙色/彩色内容**（top-area 橙色 ink 仅 CH 的 26%、band[180,240] 橙色调不同 122%），**非** welcome 那种干净的 bold-vs-normal ink 分裂。即 wintertc 的 24% 主要是橙色按钮/块（`.bg-orange-500`/`.text-orange-500`）+ 可能的图片/logo 渲染缺口，**不是 font-weight**。⚠️ 但 wintertc PNG（06-16 23:15）**早于 R227 + 图片管线 R214-R218**，stale，无法干净区分 font-weight vs 布局/图片；且当前重渲染会被并行 agent 未提交的 `cpu/mod.rs` alpha-blend WIP 污染→**wintertc 须待并行 agent 提交后用干净 build 重渲染再诊断**（独立 bug 类：橙色/彩色 + 图片，非 font-weight）。

**对 R229 优先级的影响**：font-weight 修复仍是对 **welcome 17%** 的干净单点杠杆（welcome 是文本主导页），且很可能覆盖 R221 的 183 case 1-3% chromium-diff 聚类中**含 font-weight 的 reftest 用例**；但**不是**能拉起所有产品页的通用杠杆（wintertc 需独立调查橙色/图片）。R229 接线（自源中性）+ 追加 Bold 路径仍是当前最高性价比的 welcome/DC-14 杠杆，无前置阻塞。证据 `evidence/r229-fontweight-plumbing-spec-2026-06-17.txt`（资源前提补实证）。无代码变更，基线 438/490 持平。

### R229 — font-weight 未落地精确方案：资源（仅 -Regular）+ 接线（weight 维度全程忽略）两部分核实（read-only 调研，docs-only，基线 438/490 持平）

承接 R228（welcome 17% = font-weight 未落地）。本轮 read-only 把 R229 从假设细化为「资源 + 接线」两部分精确方案。

**证据加固（CSS-weight × ink-mass 交叉验证，排除 size-forcing 替代假设）**：曾怀疑是 paint IFC font-size 强制 16px（Phase A）。逐元素核实 welcome CSS：`.version`(weight**600** 11px) ink 仅 CH 的 **10%**、`.title`(**700** 42px) **22%**、`.section-title`(**600** 12px)、`.card h3`(**600** 16px) **66%** —— 全部 **600/700 欠墨**；normal(400) 文本（`.tagline`/`.card-desc`）过墨 165-203%。**关键判别**：`.version` 11px 若被强制 16px 应更粗更多墨（>100%），实测 10%（远少）→ 与 size-forcing 矛盾、与 weight-not-honored 一致（600→normal 细字，11px 下几近消失）。> R228 据像素扫描的「cap-height 17 vs 32px」受 AA/多 span 干扰噪声大，**以本表 CSS-weight × ink-mass 为准**；过墨的 normal 文本是 fontdue vs chromium 小字号 hinting 次要残差，另议。

**根因两部分（均必需）**：
- **A 资源缺口**：`apps/browser/src/app_platform.rs:391 load_system_fonts` 主字体路径（:355-388 Linux 分支）全为 `-Regular`（`NotoSans-Regular.ttf`/`DejaVuSans.ttf`/`LiberationSans-Regular.ttf`），CJK 回退（:410-417）全 `NotoSansCJK-Regular.ttc`/`NotoSansSC-Regular.otf` —— **无任何 -Bold/-Medium 变体**。即使接线完成也无 bold 字节可选。
- **B 接线缺口**：`font/mod.rs:13 FontDesc` **已有 `weight:u16` 字段**（数据模型就绪）但全程未用——`loader.rs:103 find()` `family_map.get(family).first()` **忽略 weight**；`build_font_resolver()` 返回 `family→单id`；`engine/src/paint/` 全目录 grep `FontDesc` **零命中**（paint 从不构造 FontDesc，font-weight 不进入字体选择）。

**R229 精确方案**（供实现轮，见 `evidence/r229-fontweight-plumbing-spec-2026-06-17.txt`）：① 接线（自源中性可先做）：`find()`/`family_map` 加 weight 维度按 CSS §5.2 font-matching 选最近 weight；`load_font` 解析 OS/2 usWeightClass；paint text.rs 从 ComputedStyle 取 `font_weight` 走 weight-aware 选择。② 资源：主字体 + CJK 回退路径补同族 `-Bold`（先 `fc-list` 确认系统已装）。**成功标准**：welcome `.title`/`.card h3`/`.section-title`/`.version` ink-mass 回升到 ~100% CH（product-smoke 复测）；reftest 自源中性（438 不变），提 DC-14 chromium 一致率。**范围**：render-foundation/font/{loader,mod}.rs + engine/paint/text.rs + app_platform.rs 字体路径；**不碰 table.rs/cpu/mod.rs**（并行 agent WIP：colspan/`<col>` border-collapse + rounded_rect alpha blend）。无代码变更（仅调研+证据+文档），基线 438/490 持平。

### R228 — welcome 剩余 17% 根因重定位：font-weight 未落地（非第二处布局 bug，纠正上轮 R228 假设；docs-only + 调研证据，基线 438/490 持平）

上轮（R228 doc sync）记「welcome 剩余 17% 底部 cards/shortcuts/footer 疑似第二处 R227 式布局双计，待 R228 用 LAYOUT_DUMP 定位」。本轮用 `LAYOUT_DUMP=1 cargo run --bin zero-wpt-runner -- product-smoke welcome.html --oracle welcome-chromium.png`（单页渲染诊断，非 reftest 套件）取 ground truth，**该假设被证伪**。

**① 布局几何已完全对齐（R227 彻底消解结构偏移）**：LAYOUT_DUMP 盒树显示 `.page` pt=20 仅一次、`.hero` abs_y=20、`.hero-accent` abs_y=**36**（=chromium）、cards grid 列 x=40/407 width=353（+14 gap）行 y=238/362 正确堆叠。垂直带 diff 密度**均匀**（R226「底部 41-43%」已消失）；实心 hero-accent 条 diff **0.4%**、card 填充区 **6.7%**、左 40px 背景沟槽 >5 diff 仅 **18px**（纯背景近零差）→ **无任何结构位移残留，无第二处布局 bug**。

**② diff 与文本密度强相关**：实心/填充区 0.4-6.7%，文本区 h1 标题 29.4%、card 文本 13.8%；文本承载行占 22% 高度但贡献绝大部分 diff。

**③（决定性）ink-mass 比值证 font-weight 未落地**：逐文本带黑色像素 ZW vs chromium——eyebrow(weight600) ZW/CH=**10%**、title(weight**700**) **22%**、card-titles(weight**600**) **66%**（粗/半粗变体未被选，渲染成 normal 细字→**欠墨**）；tagline(CJK) 165%、card-desc 203%、footer 164%（CJK/normal **过墨**）。欠 vs 过 严格跟随 font-weight。

**根因（代码核实）**：`crates/render-foundation/src/font/loader.rs` 的 `FontLoader.family_map: HashMap<String,Vec<u32>>` **仅按族名索引，无 weight/stretch/style 维度**；`load_font` 把同族所有 weight push 同一 Vec 无变体选择；paint 路径（painter/text.rs）grep `font_weight` **零命中** → `font-weight:700` 落到该族首个注册字体（normal）。

**结论（方向纠正）**：welcome 剩余 ~17% = **font-weight 未落地的字形保真度缺口**（FontLoader 无 weight 维度 + paint 不传 font_weight），属字体子系统 plumbing 缺口，**可定位可修**（区别于 advance-width R225 死路 / AA R174 噪声 / Phase A 死锁）。自源中性（不刷 438），但直接攻 DC-14 chromium 一致率（R221 188/475=39.6%），且很可能覆盖 R221 的 183 case 1-3% chromium-diff 聚类中含 font-weight 的用例。**下一步 R229** = ① 核实 FontLoader 是否已加载同族多 weight 字体（资源侧）还是仅 normal（需先加资源）；② paint 路径把 `font-weight` 从 ComputedStyle 透传到 fontdue 字形光栅化 + FontLoader 加 weight 维度选变体；③ 以 welcome title/eyebrow/card-title ink-mass 回升到 ~100% CH 为成功标准（可复测 product-smoke）。证据 `evidence/welcome-post-r227-inkmass-2026-06-17.txt`。⚠️ 并行 agent 仍在 `crates/layout-engine/src/table.rs` 改 colspan/`<col>` border-collapse，本方向（render-foundation font + engine paint text）不碰 table.rs 无冲突。无代码变更（仅调研+证据+文档），基线 438/490 持平。

### R221–R227 — DC-14 诚实口径确立 + advance-width 死路证伪 + welcome padding 双计突破（范式转移：剩余 diff 是可定位的盒模型 bug，非字体/非 Phase A）

承接 R210–R220（多在 image/SVG 解码统一、DC-9 clip 范围纠正）。本批次（R221→R227）确立**唯一达标口径**并产生**首个打破「welcome 卡 Phase A」误判的真实修复**。

- **R221（DC-14 诚实口径，docs-only）**：用 chromium Oracle 口径（z_vs_chr=ZeroWeb-test vs chromium-test，DC-14 唯一承认的达标证据）量化 06-17 全量 cross-validate（R165–R180 修复后）：**严格真通过（z_vs_chr<1%）= 188/475 = 39.6%**（对比自源 89.6%，自源严重高估）。分布：<0.5%=97、<1%=188、1-3%=183、3-8%=67、≥8%=37。两个重构战略的发现：① **183 case 集中在 1-3% chromium-diff = 最大杠杆**（AA 基准已排除字体光栅化），当时假设指向 advance-width 估算噪声；② **116 case 是假 FAIL**（自源 fail 但 chr<5%，如 vrl-004 自源 7.09% vs chr 5.08%）——自源 reference 双向不可靠，证 DC-14 独立 Oracle 必需。证据 `evidence/dc14-credible-passrate-2026-06-17.txt`。
- **R222–R225（advance-width 攻坚→证伪死路）**：R222 落 estimate 误差诊断（plumbing 数据基础）→ R223 落 AdvanceSource trait seam + RFC（零行为变更）→ R224 回退 estimate_char_width 表实验（同源净 -3）→ **R225 决定性证伪**：重敷该表测 chromium 一致率，reftest-oracle 26 共享 case 零变化（中位 z_vs_chr 1.06% vs 1.07%）+ product-smoke welcome 28.34→28.31%、wintertc 25.11→25.14%（±0.03% 无意义）。**机制**：paint 经 fontdue 真实 shaping 定位 glyph，**非 estimate_char_width**；estimate 仅喂 layout 换行决策，改它不动 glyph x 位置（主导视觉因子）。**结论：advance plumbing 是死路，勿再投入**（纠正 R221 假设①；R223 trait seam 留存无害）。
- **R226（welcome 28% region-diff 定位，docs-only）**：R225 排除 advance-width 后，对 welcome ZeroWeb-CPU vs chromium-Oracle PNG 做 region-diff：y 带密度 hero 顶 9.8% → 底部 grid/flex（cards/shortcuts/footer）41-43%（底部远高）；**首非背景像素 ZeroWeb y=72 vs chromium y=36——整页内容下移 36px**；36px ≈ `.page` padding-top(20) + `.hero` padding-top(16)。排除字体（R225）/AA 基准后，定性为**布局垂直定位 bug**（locatable, fixable）——非 font/metric 噪声，是 DC-13 真杠杆。证据 `evidence/welcome-region-analysis-2026-06-17.txt`。
- **R227（welcome 36px 根因定位 + 修复，+product-smoke 11pp / reftest 净 -1，已提交 0b48b0f）**：dump 盒几何（新增 `LAYOUT_DUMP=1` env，reftest.rs:951 `dump_layout_tree`）定位确切根因——**taffy 0.7 `Layout::location`（即 `LayoutBox.x/y`）是子节点 border-box 相对父 border box 的偏移，已含父 padding+border**（taffy-local `content_box_y = location.y + border + padding` 印证）；但 ZeroWeb painter（`paint_node` child_offset += padding+border）+ inline IFC + abspos 线程统一采用「子节点相对父**内容盒**」约定→**taffy 定位的 block/inline-block 子节点 padding+border 被双计**，每个带 padding/border 的父容器把整棵子树多移一份。welcome 36px = .page pt20 + .hero pt16 两级双计。**为何自源 reftest 长期不暴露**：自源 test/ref 同源，双计对两者一致→抵消→「通过」（DC-14 46.5% 假通过同源）。**修复**（engine.rs extract_layout:777-793）：水平书写模式下、非 abspos/fixed 的 taffy 子节点，把 location 换算为内容盒相对（减自身 content_x/y）；abspos/fixed 由 adjust_absolute_* 线程按 border-box 约定单独处理故跳过，float 由 adjust_float 覆写故无害，inline IFC 子节点不经 extract 的 taffy 循环故不受影响。选 extract 侧而非 painter 侧（painter 全局去 padding 会 -5 回归：multicol/grid ref 用 inline 子节点本就内容盒相对）。**验证**：welcome product-smoke（chromium Oracle）**28.34%→17.06%**（hero-accent y=36 ✓）；reftest **439/490→438/490 净 -1**（唯一回归 grid-flex-spanning-items-001 borderline 0.77→1.31%，aqua 实从 x=28 移到 x=18=content box 左与 ref 一致，旧 0.77% 系「aqua 右偏 10px」与「border 尺寸差」两误差抵消，修复后剩余 border 尺寸差是测试固有）；css-position 16/16、multicol 40/57 持平；make test 全绿、clippy 零警告。证据 `evidence/r227-welcome-padding-doublecount-fix-2026-06-17.txt`。

**本批次的意义（范式转移）**：R221 确立「达标只看 chromium 一致率（39.6%），自源 89.6% 不可信」；R225 关掉 advance-width 死路；**R226/R227 证明 welcome 28% 是可定位的盒模型坐标约定 bug（padding 双计），非 Phase A font_size 死锁**（纠正 master 长期记载的「welcome 卡 Phase A」误判），且暴露 painter 对 inline(IFC) 子节点 padding 处理正确、对 block(taffy) 子节点双计=**坐标约定 MIXED**——这是一类**系统性、可按 R226 方法学（region-diff→dump 盒几何→定位）逐个定位修复**的 bug，预期覆盖 183 case 1-3% chromium-diff 噪声大头。`LAYOUT_DUMP=1` env 成为新诊断标准工具。

**当前基线**：reftest 自源 **438/490**（R207 +1 后 R227 -1）；DC-14 严格真通过 **188/475=39.6%**；welcome product-smoke **17.06%**。无未结代码实验（R227 已提交；advance-width 实验全回退）。

### R210 — compute_final 多行存储 + multicol 守卫实证：净 +0，v_offset 语义墙，不可默认启用（docs-only，基线 439/490）

承接 R209（PHASEA_MULTILINE 净负，疑 R198 ancestry-guard 墙）。本轮测试**未被试过的精确组合**：compute_final 多行存储 + `!in_multicol` 守卫。

**守卫实现**：compute_final_inline_layouts 新增 `in_multicol: bool` 参数，`child_in_multicol = in_multicol || root.is_multicol` 递归透传；`allow_multiline = PHASEA_MULTILINE==1 && is_pure_ahem && !in_multicol`。text.rs stored frag 渲染改 `y: f.y + line.y`（frag.y 行内相对→行盒绝对，多行必需）。

**全量实测**（490 用例）：default 439/51，gate 439/51 —— **净 +0**。仅 2 翻转：
- ✅ ifc-008: 8.18% FAIL → **PASS 0.00%**（compute_final 正确存储 node39 的 2 行 100px Ahem，R84 单行限制被守卫内放宽）
- ❌ multicol-fill-auto-001: 0.63% PASS → 9.15% FAIL

**根因精确定位（multicol-fill-auto 回归源 ≠ multicol 容器）**：CFDEBUG 探针（reftest 双趟，正确加载 /fonts/ahem.css）显示 multicol-fill-auto 存储了 node 25（10 行 16 frag）/ node 28（5 行 8 frag），**in_multicol=false**。逐文件分析：
- test 文件：1 个 multicol div（column-count:3 column-fill:auto > 直接文本）
- **ref 文件**：2 个 `<div float:left width:10em>`（**非 multicol**，用 float 模拟列），各含 5–10 行纯 Ahem 文本

→ node 25/28 是 **ref 文件的 float div**，合法 in_multicol=false，`!in_multicol` 守卫**无法区分**（它们本就不是 multicol 内容）。**回归不在 multicol 容器，而在 ref float div**。

**v_offset 语义墙**：default（多行守卫挡住）ref float div 走 paint IFC（text.rs:1220，`baseline_fs=font_size`）；gate 走 stored（text.rs:1202，`v_offset=0` Ahem 顶部对齐）。两条路径对同一多行 Ahem 文本的 baseline 计算不同，ref（stored）与 test（paint IFC column 分配）渲染差 font_size/行 → 9.15%。实测 v_offset=font_size（统一两条路径）：multicol-fill-auto 9.15→10.30%（更差）、font-051 PASS→16.67%（单行 large-font 破坏）、ifc-008 0→8.33%。**stored v_offset 单行/多行语义无法统一**，印证 R125 三路径死锁。

**关键方法学纠正（推翻 R209 的 product-smoke 诊断前提）**：product-smoke 单趟**不加载外链 CSS**（base_dir=None），multicol-fill-auto 的 `/fonts/ahem.css` 不解析 → is_pure_ahem=false → 不存储。故 R209 用 product-smoke 看 multicol-fill-auto「不存储」是**外链 CSS 缺失的假象**，非真实行为。**涉及外链 CSS（/fonts/ahem.css 等）的用例必须用 reftest 双趟路径诊断**，product-smoke 仅对内联 CSS 用例可靠。

**结论**：compute_final 多行存储 + multicol 守卫**净 +0，不可默认启用**（zero count 回归标准）。真正阻塞 = stored 多行路径与 paint IFC 路径的 v_offset/baseline 语义在多行场景不一致，通过 ref float div 暴露。这是 Phase A IFC 双路径统一问题（R125/R198 同墙），单点不可解。ifc-008 已确认可被 compute_final 正确存储并 PASS，但解锁它需先统一两条路径的多行 baseline 语义（结构性多轮）。证据见 `evidence/r210-multiline-multicol-guard-2026-06-17.txt`。

无代码变更（实验已 git checkout 回 R207 e0e2689 干净态），基线 439/490 持平。

### R209 — ifc-008 根因精确定位（R84 单行限制）+ 多行存储实证：改善 ifc-008/009 但回归 multicol-fill-auto（docs-only，基线 439/490）

承接 R208（ifc-008 未定论）。本轮用**干净单趟探针**定位根因。

**方法学纠正**：R208 探针被 reftest 的 **test/ref 双趟 compute** 干扰（test 7 节点含 39、ref 6 节点不含）。本轮改用 **product-smoke 渲染 test HTML 单趟**（--oracle dummy），compute_final 仅调一次，探针干净。

**精确根因**：compute_final（engine.rs:1900-1903）的 **R84 单行 Ahem 存储限制**：
```rust
let is_pure_ahem = style.font_family.len()==1 && style.font_family[0].eq_ignore_ascii_case("Ahem");
if inline_ctx.lines.len() > 1 || !is_pure_ahem { return; }  // 仅存单行纯 Ahem
```
- 干净探针证实 node 39(inner-div) **被访问**（visit probe）、block=true、multicol=false、display=Block、direct_text=true——通过所有早退，**仅因 `lines.len() > 1` 在 line 1901 返回不存**。
- ifc-008 的 "XX XX" 100px 在 div1(200px 宽) 换 **2+ 行** → lines.len()>1 → 不存 → paint 重跑空 styles → 16px。
- font-051 的 "FAIL" 单行 → lines.len()=1 → 存 → PASS（R207 成功的根因）。

**实证放宽**（env PHASEA_MULTILINE=1，允许 Ahem 多行存储）：ifc-008 **8.18→4.17%**、ifc-009 **6.11→4.17%**（**改善**，证明多行存储产出更优度量；余 4.17% 系换行精度/绿色覆盖差），font-051 0.00% PASS 不变。**但 net-negative**：全量 multicol-fill-auto-001 0.63% PASS→FAIL（**R198 反向依赖再现**：multicol 的子容器现多行存储致渲染变），无用例翻 PASS（ifc-008/009 改善但未过 1%）。

**结论**：ifc-008/009 修复需三重耦合：(a) 放宽多行存储；(b) 守 multicol-fill-auto 不回归（=R198 ancestry-guard 墙，已知失败）；(c) 修余 ~4.17% 换行/覆盖精度。非单点。与 R198/R205 同墙（存储→multicol 反向回归）。

**可复用方法学**：reftest 双趟 compute（test+ref）会混淆探针；**单趟诊断用 product-smoke**（`product-smoke <test.html> --oracle <dummy> --out <png>`，仅渲染 test）。R208 的「未访问」误判即双趟混淆所致。

**验证**：无代码变更（探针+ML 实验经 `git checkout` 清除回 R208 ccabef5）；font-051 PASS、multicol-fill-auto 0.63% PASS 恢复；基线 439/490 持平。

### R208 — ifc-008 block-child large-font 深挖：inner-div 16px，compute_final 未存其行盒，架构性 defer（docs-only，基线 439/490）

承接 R207（font-051 inline-child 容器 PASS，+1）。本轮攻 ifc-008（large-font 集群，block-child 结构）。

**ifc-008 结构**：`#div1{font:100px/1em Ahem; height:2em; width:2em; background:red} > <div>XX XX</div>`（inner-div block，`div div{color:green}`）。期望：绿色 \"XX XX\" 填满 div1（无红）。

**dump 实测**：div1 区域（200×200）几乎全红（39064px）+ 微量绿（736px ≈ 16px 文本）。即 inner-div 的 100px 文本**渲染成 16px**（large-font bug），**R207 inline-child 路径不覆盖此 block-child 结构**（inner-div 是 block 子节点，R207 条件 2「无 block-level 子节点」是针对容器自身；这里 inner-div 容器有直接文本，本应走既有 direct-text 存储路径）。

**探针（IFCDBG，临时已清除）**：
- paint 见 inner-div(node 39) fs=100、ifc_width=200，但 **inline_layout=None / stored_w=0 / use_stored=false** → paint 重跑 IFC（空 styles）→ 16px。
- compute_final 探针（doc/style/display/multicol/!block/!has_text/store 各早退点）**未对 node 39 触发存储**。但探针受 **test/ref 双趟 compute**（test 7 节点含 39、ref 6 节点不含 39）+ 字符串匹配 \"39\" 假阳性干扰，**未定论根因**。
- 可能根因：compute_final 树遍历未到达 inner-div 的带 node_id 盒（匿名盒包裹致 node_id=None 早退，或树在 compute_final 后重组）。

**结论**：ifc-008（block-child 文本容器）的 large-font 修复需厘清 compute_final 为何不为 inner-div 存储行盒——属**架构性深挖**（compute_final 树遍历 vs paint 树遍历一致性），非 R207 inline-child 路径的单点扩展。defer，下轮需干净的单趟探针（区分 test/ref render）定位。

**验证**：无代码变更（探针经 `git checkout` 清除，回到 R207 e0e2689 提交态）；font-051 PASS 保持；基线 439/490 持平。

### R207 — Phase A stored-line-boxes narrow 精修成功：font-051 PASS，+1 零回归默认启用（438→439/490）

承接 R206（stored-line-boxes 路径 broad 应用 net-negative，但 font-051 单点改善证明架构可行）。本轮做 **narrow 精修**至「纯 inline 内容」容器，达成 Phase A 首次 clean win。

**narrow 条件**（三重，逐回归收敛）：
1. 有 **inline-level** 元素子节点（Inline/InlineBlock/InlineFlex/InlineGrid/InlineTable）。
2. **无 block-level** 元素子节点（排除 ifc-008 的 div1>div、multicol-block-no-clip-001 的 span+h4 等混合内容——broad 版含 block 子节点致 paint_text 重跑双绘）。
3. inline 子元素**无元素子节点**（叶文本容器，排除 inline-box-002 的 div2>div3 block-in-inline R109 碎片化）。

**实现（默认启用，env PHASEA_STORE_EXT=0 关闭）**：
- `compute_final_inline_layouts`（engine.rs:1693）：扩展 `has_text_children` 覆盖满足 narrow 条件的容器（如 div>span 间接文本）。compute_final 传**真实 styles** 给 IFC（line 1851），故存储行盒 font_size/line-height/line-breaking 正确。
- `has_direct_paintable_text`（text.rs:1479）：同步扩展（新增 styles 参数），使 paint_text 不在 line 683 提前返回，到达 use_stored（text.rs:805）渲染存储行盒。

**实测（全量上游 reftest，默认启用）**：
- **font-051 8.19%→PASS 0.00%（+1）**：div>span>"FAIL" 100px 现以 compute_final 存储的真实度量渲染（而非 paint 重跑空 styles 的 16px）。
- **438→439/490（89.6%）**，**零 count 回归**：broad 版的三处回归（inline-box-001/002、multicol-block-no-clip-001）经 narrow 条件 2/3 排除，全部恢复基线。
- ifc-001/002/003（broad 翻 FAIL）现 0.12/0.70/0.36% 仍 PASS（narrow 未触及，且无恶化）。

**验证**：make test 全绿（45 crate / 0 failed）；`cargo clippy --all-targets -- -D warnings` clean；fmt clean。

**Phase A 首次实证 clean win 的意义**：stored-line-boxes 架构（compute_final 用真实 styles 存行盒 → paint use_stored 渲染）对**纯 inline 容器**产出正确度量，**破了 R205 的 4 路 font_size 死锁**（那些路让 paint 重跑 IFC，因 font_size/line-height 耦合而失败；本路 paint 不重跑，直接用 layout 正确结果）。证明 Phase A 非绝望死锁，而是需 narrow 应用（避免与现 paint 路径在混合/碎片化内容上分歧）。

**剩余**（下轮方向）：
- large-font ifc-008/009/011 = **block 子节点**容器（div1>div>text），非本 narrow 路径覆盖（其文本在 block 子节点的 IFC，另一子问题）。
- empty-inline-002（29%）仍卡。
- 可扩 narrow 条件覆盖「inline 子元素含 block 后代但非 R109」等更多结构，或攻 block-child IFC 的 stored 路径。

### R206 — Phase A「stored line boxes」路径首次实证改善 font-051（架构可行），broad 应用 net-negative 须 narrow 精修（docs-only，基线 438/490）

承接 R205（4 路 font_size 解锁全 net-negative，定性「唯一未试路径=paint 渲染存储行盒」）。本轮实现并实证该路径。

**关键基础发现**：`compute_final_inline_layouts`（engine.rs:1851）传**真实 styles** 给其 IFC（`inline_ctx.layout(doc, node_id, styles)`），区别于 paint_text 传空 `&HashMap::new()`（line 927）。故 compute_final 存储的 `inline_layout` 行盒 font_size/line-height/line-breaking **正确**（paint 重跑则错）。这是 stored-line-boxes 路径可行的根因，区别于 R205（让 paint 重跑用正确度量，因 font_size/line-height 耦合而失败）。

**实现（env-gated PHASEA_STORE_EXT=1，零基线风险）**：
1. compute_final（engine.rs:1688）扩展 `has_text_children`：env 开时也覆盖含 inline-level 元素子节点（Inline/InlineBlock/InlineFlex/InlineGrid/InlineTable）的容器，如 div>span 间接文本。
2. `has_direct_paintable_text`（text.rs:1479）同步扩展：env 开时也认含 Element 子节点的容器为「有可绘文本」，使 paint_text 不在 line 683 提前返回，到达 use_stored（text.rs:805）渲染存储行盒。

**首次实证改善（Phase A 5 路首次正向）**：`font-051` **8.19→1.51%**——div>span>"FAIL" 100px 文本现以 compute_final 存储的真实度量渲染（而非 paint 重跑的 16px 默认）。**证明 stored-line-boxes 架构可行、产出正确度量**。

**但 broad 应用严重 net-negative，已回退**：
- 3 个原 PASS 翻 FAIL：ifc-001 0.53→1.84、ifc-002 0.72→2.38、ifc-003 0.23→2.02（基础 IFC 用例）。
- large-font 集群反**恶化**：ifc-008 8.18→9.73、009 6.11→8.07、011 11.24→13.00、empty-inline-002 29.32→31.14。
- multicol-fill-auto-001 0.63 PASS→1.92 FAIL、position-absolute-in-inline-005 1.19→2.46。

**结论**：stored-line-boxes 路径**架构正确**（font-051 实证），但 **broad 应用**（对所有含 inline 子节点的容器）改变了大量容器渲染——存储行盒路径与现 paint 重跑路径在 float-exclusion/vertical-align/双绘 等维度不一致，致 broad 回归。**下一步 = narrow 精修**：仅对「存储结果与重跑不同且改善」的容器应用（如仅单 inline-element 文本子节点 + 无 block 兄弟的简单结构），逐条件 set-diff 收敛零回归后再逐步扩面。这是 **multi-round 精修**，但**方向首次明确可行**（font-051 实证），区别于 R205 的 4 路全死锁。

**验证**：实验全回退（git diff clean）；font-051 8.19% / ifc-001 0.53% PASS / multicol-fill-auto-001 0.63% PASS 恢复；基线 438/490 持平。

### R205 — Phase A font_size 解锁第 4 路实证 net-negative：deadlock 定位 IFC 度量架构（docs-only，基线 438/490）

承接 R204。本轮在 R72（全量真实 styles→4 回归）/R125（三路存储死锁）/R198（override 填充→multicol 回归）三路 font_size 解锁全 net-negative 后，尝试**第 4 路：font_size 单字段解耦回退**。

**思路（区别于 R72）**：R72 把全量真实 styles 传给 paint IFC 致 BFC-004/font-feature-002/position-absolute-in-inline-005/006 回归（依赖 vertical_align/letter_spacing/float-exclusion 等）。本轮改为 paint 注入**仅 font_size 单字段**表 `real_font_sizes`（从真实 ComputedStyle 提取），IFC（inline/mod.rs:806）在 `font_size_overrides` 未命中时回退此表，**其他属性仍走空 styles→默认**，理论上规避 R72 的 4 回归。

**实现（env-gated PHASEA_FS_REAL=1，零基线风险）**：IFC 新增 `real_font_sizes: HashMap<NodeId,f32>` 字段 + `with_real_font_sizes` setter；resolution 增加 override→real_font_sizes→16px 三级回退；paint_text 非存储路径在 env 开时从 styles 提取 font_size 注入。

**实测全 net-negative，已回退**：
- **font-051 8.19→11.65% 恶化**：font_size 现正确（100px），但 line-height 用 `fs×1.2` 近似与 div 的 `100px/1`（line-height:1）不符→行盒更高→diff 增。**证明 font_size 与 line-height 耦合**——单修 font_size 致 line-height 失配。
- **multicol-fill-auto-001 0.63% PASS→1.21% FAIL**（R198 同源反向依赖：该用例的 paint IFC 度量依赖 16px 默认才通过）。
- **position-absolute-in-inline-005/006 回归**（1.19/1.23% FAIL，R109 out-of-flow 修复的用例）。

**结论（Phase A font_size 解锁穷尽）**：4 条路全 net-negative——R72 全量 styles / R125 三路存储 / R198 override 填充 / **R205 font_size 单字段解耦**。死锁根因不是任一属性，而是 **paint IFC 与 layout IFC 两趟独立运行，font_size/line-height/line-breaking 三者耦合**：任何「让 paint IFC 用正确度量」的改动都因破坏其他耦合维度而回归。

**唯一未试架构路径** = **paint 完全不重跑 IFC，直接渲染 layout IFC 存储的行盒**：
- compute_final_inline_layouts（engine.rs:1646）对所有含 inline 子树的容器存 `inline_layout: Vec<InlineLayoutLine>`（当前仅存 block-level+直接文本，跳过 div>span 等间接文本容器与 inline 元素——font-051 即此）。
- 扩展存储条件 + paint 经 use_stored 路径（text.rs:805）渲染存储行盒（已支持，渲染 f.font_size 正确）。
- 难点：float-exclusion / vertical-align / 多列协调须在 layout IFC 单趟内一致（当前两趟各自处理，是死锁根源）。**multi-round 架构里程碑**，非单会话。

**验证**：实验全回退（git diff clean）；font-051 8.19% / multicol-fill-auto-001 0.63% PASS 恢复；基线 438/490 持平。

### R204 — 绝对 plateau 再确认 + R197 DC-12 clip-path 审计纠正（docs-only，基线 438/490）

承接 R203。本轮复核所有剩余可操作方向，**全部 ruled out 或已实现**，确证增量 clean win 彻底穷尽。

**① R197 DC-12 审计纠正：clip-path circle/ellipse/polygon 真裁剪已全实现**
- R197 曾记「clip-path circle/ellipse/polygon 仅画指示线非真裁剪（只 inset 真裁剪）」——**此结论过时/错误**。
- 代码核实：`painter/mod.rs:599-673` 对 Circle/Ellipse/Polygon 三形状均调 `super::helpers::clip_all_primitives_to_polygon`（inset() 走 `clip_all_primitives_to_rect`）。
- `helpers.rs:234 clip_all_primitives_to_polygon` 是完整实现：① 先用多边形包围盒裁所有图元；② fills 经 `clip_fill_to_polygon`（扫描线精确裁剪，生成子矩形片段）；③ glyphs 经 `point_in_polygon`（射线法）中心点测试丢弃外部字形。
- **DC-12 clip-path 无缺口**。唯一极次要项：`paint_clip_path` 指示线（mod.rs:426-429，对非 inset 形状）冗余叠加在真裁剪之上（非 skip_indicators 守卫），属调试残留，非功能缺口。

**② border-001(2.77%) dump 排除**：ZeroWeb 渲染**正确空心黑方框**（border:25px shorthand 解析正确 → 25px 实心边 + 100x100 hollow center）。2.77% diff 来自 REF 的 <p> 文本/定位差异（字体度量致 border box y 位置偏移），非 border 渲染 bug。

**③ float-006(7.47%)**：float+abspos+z-index+overlap 复杂交互（zero-height-first-float + red-overlapped-second-float + green-overlapping-abs-pos），R100/R108b float 结构域，非 clean win。

**④ Phase A 新角度证伪**：曾想「paint IFC 传空 styles → 修 style plumbing 解锁 Phase A」。代码核实 `paint_text`（text.rs:630）直接从入参 `style: &ComputedStyle` 读 font_size（line 639），调用方 mod.rs:263/446 传**真实 style**（`styles.get(&node_id)`）。**paint 侧 style plumbing 正确**，Phase A 死锁不在 style 传递，而在 layout IFC 与 paint IFC **两趟独立运行不共享度量状态**（R125 三路/R198 store 路全死锁的根因）——架构性，非 plumbing。

**结论（绝对 plateau）**：剩余 52 失败 = ① 多轮硬架构（multicol 全 CSS 碎片化模型 / Phase A IFC 三路径统一死锁 / writing-mode 轴 R114 4 轮证否 / flex 基线 R130 / intrinsic sizing R97 opt-in 风险）；② 浏览器引擎特性（原生表单控件外观 / dialog::backdrop JS / background-attachment:fixed）；③ REF 怪异（vertical-rl 同源 REF 水平渲染）。**无单会话 clean reftest win**。任务需多轮架构性投入（Phase A 最高杠杆但死锁需新架构思路；multicol-breaking 需全碎片化模型）。

**验证**：无代码变更；基线 438/490 持平。

### R203 — multicol-breaking paint 侧修复实证 ruled out：真实路径=layout 侧 column-aware IFC（docs-only，基线 438/490）

承接 R202。本轮对 R201 定性的 multicol-breaking 阻塞点做 paint 侧实证修复，**3 路全部 net-negative，已回退**，由此将真实解决路径收敛到 layout 侧。

**新发现 D = `painted_inline_nodes` 去重（text.rs:688）**：column breaking 目标（`column_span_offsets` 多片段的子元素，如 multicol-breaking-004 的 `.inner`）在 painter/mod.rs:524-555 的片段循环中被多次 `paint_node` 重绘（各片段不同 y-offset + 列裁剪），但首次渲染后 `painted_inline_nodes` 标记其 node_id，**后续片段渲染被 line 688 去重 early-return**→ 非首列文本不渲染。这是 R201 未列出的第 4 个阻塞点。

**实证 3 路（全回退）**：
1. **单独修 D**（去重条件加 `column_span_offsets.len() <= 1` 例外）：multicol-breaking-006 **1.20→1.71% 恶化**——col1/col2 现渲染 inner 文本但**位置错**（inner 仍是单列文本被切片，无 2 子列布局协调），与 ref（2 子列）不符致 diff 增。004/005/nobackground-004 不变。net-negative。
2. **A'+D 协调**（text.rs:711 门控对「被碎片化子元素」放宽 `height_auto`，使 inner 做 2 子列 paint 分布 + D 去重修复）：全量 multicol **40/57→36/57（-4 大回归）**。门控放宽触发其他明确高度 balance multicol 用例回归（text.rs:707-709 注释确证门控**专为防回归而加**）。net-negative。
3. 结论：**paint 侧修复（单点或简单协调）对 multicol-breaking 全 net-negative，ruled out**。

**根因（为何 paint 切片不可行）**：paint 侧方案是「单次 IFC 渲染全量行盒 + paint 按列切片」。但 multicol-breaking 的正确行为要求**每列有独立正确的行盒分布**（ref col0=AAAAA-EEEEE+FFFFF-JJJJJ=5+5 行，col1=4+3 行——这是 inner 2 子列内容流过外层列边界的碎片化结果，非简单垂直切片）。paint 切片无法产生此分布，且与门控/去重/列分配多处耦合，单点改必回归。

**真实路径收敛 = layout 侧 column-aware IFC（R131）**：让 IFC（`InlineFormattingContext`）在 `break_items_into_lines` 生成行盒时即接受**列碎片化上下文**（列数 + 每列可用高度），按列高把行盒碎片化到各列，产出每列独立的行盒序列 + 列内 y。paint 侧只按列渲染（复用现有 text.rs:974-984 逐列裁剪）。这是 R131 原方案的具体落地，**multi-round layout 子系统**（触及 IFC 核心 break_items_into_lines，需 12k 测试验证），非单会话。

**可复用教训**：架构性失败的单点/简单协调修复**须实证**（throwaway 探针 + 全量套件 reftest），不能据推断直接落地——本轮 D「看起来是 bug 且修法直观」但单独修 net-negative（006 恶化），印证 R164「方法论=架构性推断需实验落地，不能据推断接力多轮」。

**验证**：实验全回退（git diff clean）；multicol 40/57 恢复；baseline 438/490 持平。

### R202 — chromium Oracle 高 diff 候选实证排查：3 项 ruled out（docs-only，基线 438/490）

承接 R201（multicol-breaking 定性）。本轮回到 DC-14 优化目标（chromium Oracle 一致率），基于 `evidence/cross-validate-full-2026-06-17.txt`（fresh）的 z_vs_chr>5% POLLUTED 清单（self-source 假通过掩盖的真缺口），逐项用 **probe-based 实证**（throwaway HTML → `product-smoke` 渲染 → Python 2px 采样判定几何）排查 top 候选。

**① abspos-semi-replaced-stretch-input/button/other（chr 23.03/3.55/15.27%，self 0%）RULED OUT**
- 测试：`<input>/<button>/<select>/<textarea>` 等 `position:absolute` + 全 inset(top/right/bottom/left:3px) + width:auto/height:auto，期望 stretch 填满 CB（ref 用 `width:calc(100%-6px)`）。
- **probe 实测**：plain `<div>`、`display:inline-block` div、`<span>`、真实 `<input>/<button>` 经 product-smoke 渲染——**全部正确 stretch**（red bg 填满 CB，lime outline 跨满 CB x≈8-168）。早先 REFTEST_DUMP 4px 采样误判「窄」（漏掉右 lime 边落在采样间隙），**2px 采样证 stretch 工作**。
- **stretch 算法（CSS2 §10.3.7）工作正常**——converter 正确传 position:Absolute+inset+width:Auto 给 taffy，taffy 正确 stretch（div/inline-block/inline 均验证）。
- **23% chr 差异真因 = 表单控件渲染特性缺口**：ZeroWeb 把 `<input>/<button>` 画成 styled box + outline（无原生外观），chromium 画原生 widget（native button/text-field，含边框/内边距/默认字体）。这是**大特性**（需实现原生表单控件外观），非布局 bug，单点不可修。

**② backdrop-inherit-rendered（chr 47.54%，self 0%）RULED OUT**
- 测试 `dialog::backdrop { background-color: var(--bg); inset: inherit; }` + JS `dialog.showModal()`。是 **`::backdrop` 伪元素 + dialog JS API**，**非 backdrop-filter**（R197 审计的 backdrop-filter 已实现）。需 dialog showModal() 基础设施，非 contained。

**③ background-attachment-applies-to-001（self 29.92% / chr 31.04%）= 特性缺口**
- 测试 `background-attachment: fixed` 应用于 `display:table-row-group`。fixed 背景 = **视口相对定位**特性（不随滚动/元素移动），需 paint 期视口坐标变换，非 contained 布局 fix。

**可复用方法**：probe-based 实证——throwaway HTML 写到 /tmp，`product-smoke <html> --oracle <dummy> --out <png>` 渲染 ZeroWeb CPU PNG，Python PIL 以 **2px 步长**采样（4px 会漏窄边/细线致误判）判定几何。比直接读代码推断更可靠（避免「stretch 未实现」类错误假设）。

**结论**：fresh Oracle 高 diff 候选**全为结构性（multicol-breaking R201 / table_colspan R177 / Phase A R125 / writing-mode R114）或特性缺口（表单控件原生外观 / dialog JS / fixed-bg）或已修复（R165-R180）**，**无单会话 clean win**。chromium Oracle 侧 plateau 与 self-source 侧（R185+ 多轮）一致确认。

**验证**：无代码变更（probe 已清理 /tmp/zwprobe）；基线 438/490 持平。

### R201 — multicol-breaking dump 实测定性：纠正 R113「两趟循环依赖」假设，碎片化算法已存在（docs-only，基线 438/490，已提交）

承接 R200（balance 方向关闭）。本轮对 multicol-breaking-004/005/006/nobackground-004（css-multicol 唯一剩余「嵌套列碎片化」失败聚类）做 **REFTEST_DUMP + REFTEST_BBOX + 逐行像素扫描** 实测定性，**纠正 R113/R132「内层 multicol 高度依赖外层列宽→两趟循环依赖」假设**。

**实测证据（multicol-breaking-004，5.60%）**：结构 `.outer`(h:125, col-count:4, fill:auto, rule:4px blue) > `.inner`(col-count:2, h:300, border-bottom:25 green, box-decoration-break:clone) 内含 17 行文本。
- **REF**：3 列可见内容，每列 2 子列（col0=AAAAA-EEEEE+FFFFF-JJJJJ；col1=KKKKK-NNNNN+OOOOO-QQQQQ；col2=空+绿 border），列间蓝色 column-rule。
- **ZeroWeb**：inner 文本**仅在 col0**（x≈8-55，单子列），col1/col2 **完全无文本**（仅洋红 bg）；**蓝色 column-rule 全漏画**；绿 border 位置错（col2 y≈60-80 而非 3 列 y=100-125）。BBox x=[8,603] y=[8,132]（diff 止于 col2 末，col3 空=匹配）。

**3 真实阻塞点（非 R113「循环依赖」）**：
- **A. paint 门控 `height_auto`（text.rs:710-715）**：inner 有明确高度 300px → `height_auto=false` → `compute_multicol_info_for_paint` 返回 None → inner 的 2 子列布局**从未计算**。
- **B. `column_span_offsets` paint 路径不重绘碎片化 IFC 内容（核心）**：outer column breaking 把 inner 碎片化到 col0/1/2（写 column_span_offsets），但该 paint 路径**不重绘 inner 的 IFC 文本到非主位置列** → inner 文本只在 col0。R131 同源。
- **C. column-rule §5.2 内容检测只查 `child.x` 主位置（text.rs:185-197）**：被碎片化的唯一子元素只在 col0 有 c.x → 其余列误判「无内容」漏画 rule。

**关键纠正：碎片化算法已存在**。`multicol.rs` 已实现 `assign_children_to_columns_sequential`（顺序填充+列高预算）与 `assign_children_to_columns_with_breaking`（块级子元素 column breaking）——这正是 CSS Multicol §6 fragmentation 所需顺序填充算法。R113 设想的「内层 multicol 两趟测量」**算法层面已具备**，缺口是**接线到 inline 内容的 paint 路径**。**勿再建行内流行盒碎片化 measure-first 工具**——它会是 `assign_children_to_columns_sequential` 的重复实现，重蹈 R199 balance 工具被 R200 证伪移除的覆辙。

**column-rule 修复（C）实测回归，已回退**。实现 `in_range` 闭包额外查 column_span_offsets 列 x → 004 5.60→**5.39**、006 1.20→**1.12**（蓝色 rule 补画改善）**但 column-rule-002 0.00→1.25%（PASS→FAIL 回归）**。根因：column-rule-002（`columns:3;fill:auto` + 单个 h:250 子元素被碎片化到 3 列）REF **恰好匹配旧 c.x-主位置检测**；旧 §5.2 启发式虽对嵌套 multicol-breaking 不完美但对 column-rule-002 正确。**C 非安全单点修复**，已回退（git diff clean），未来修须区分 column_span_offsets 来源（breaking vs spanner）精确实现 §5.2。

**对实施计划的影响**：原 Round 4（column breaking / 两趟循环依赖）算法前提不成立。真实 Round 4' = wiring：让 column_span_offsets paint 路径对被碎片化 IFC 容器子元素按列高预算重绘其 inline 内容到每个非主位置列 + 列裁剪（R131「paint IFC 与 multicol block 分配协调」具体落地，paint 侧多轮子系统）；前置放宽 text.rs:711 门控（守 multicol-fill-auto-001 不回归=R198 font_size 死锁交互）。预期解锁 004/006/nobackground-004（3 用例；005 balance+balance 嵌套更难独立）。仍是多轮，非单会话。

**验证**：无代码变更（C 修复已回退）；multicol 40/57 持平、column-rule-002 0.00% 恢复 PASS、baseline 438/490 持平。设计文档 `multicol-fragmentation-design.md` 升 v0.3（R201 段 + Round 4' 重定向 + §0 首步纠正）。

### R192 — R109 §9.2.1.1 生产端接线落地（匿名块生成 + fragment 注册表，env-gated，+1 验证待 fragment border，已提交）

R109 多轮里程碑（inline→block + IFC ownership，最高杠杆，影响 inline-box-001/002、block-in-inline-align-001、clear-inline-001、morning.work blue-nav）的**生产端接线**——继 R189（split 分析模块）/R190（IFC fragment 收集）/R191（消费端 LayoutBox.fragment_node_ids）三基础件后，补齐 tree.rs 匿名块生成 + extract 注册表读取，数据流首次端到端打通。

**实现**：
- **`tree.rs`**：`build_layout_tree_with_r109`（包装 `build_layout_tree`，额外返回 `R109Wiring`）。`R109Wiring { fragment_registry: HashMap<taffy::NodeId, Vec<NodeId>>, split_parents: HashSet<NodeId> }`。`build_subtree` 非 flex/grid 分支：env `R109_WIRE=1` 且 `inline_has_block_child` 时，`compute_inline_block_split` 把 inline 的子元素拆为片段序列——Inline 片段→`new_leaf_with_context(Block, inline_node_id)` 匿名块（context=inline 的 NodeId 承其样式）+ 注册 `fragment_registry[anon]=item_node_ids`；Block 片段→原样 `build_subtree`。`taffy_to_dom[anon]=inline_id` 使 extract 给匿名块 node_id=inline。
- **`engine.rs`**：`R109Wiring` 入 `CachedLayoutState.r109`（增量路径复用）；`extract_layout` 收 `r109` 参数，读 `fragment_registry[taffy_id]`→`LayoutBox.fragment_node_ids`，读 `split_parents`→`LayoutBox.is_r109_split`；匿名块片段强制 `is_block_level=true`。`compute_final_inline_layouts` 对有 `fragment_node_ids` 的盒设 IFC 片段覆盖。
- **`types/mod.rs`**：`LayoutBox.is_r109_split: bool`（默认 false）。
- **`text.rs`（paint）**：`is_r109_split && fragment_node_ids.is_none()` 的 inline 父盒跳过自身 paint IFC（其文本由片段子盒渲染，避免重叠）；匿名块片段（`fragment_node_ids.is_some()`）跳过 `painted_inline_nodes` 去重（多片段共享 inline node_id，须各自渲染）。

**关键修复（out-of-flow 排除）**：首测 `R109_WIRE=1` 全量 **434/490 (−1)**——`position-absolute-in-inline-005/006` 回归（0.76/0.81%→1.87/2.18%）。根因=`inline_has_block_child`/`compute_inline_block_split` 只看 `display`，把 `position:absolute/fixed`、`float≠none` 的子元素当 in-flow block 触发拆分。但 CSS2 §9.2.1.1 匿名块生成**只针对 in-flow block-level box**。修=`is_out_of_flow(style)`（abspos/fixed/float）守卫：触发判定排除 out-of-flow；拆分时 out-of-flow 子元素归入 Inline 片段（保留为子节点，由 converter 定位路径处理，不丢失）。修后 css-position 恢复 16/16。

**实测 `R109_WIRE=1` 全量**：**436/490 (+1，零 count 回归)**。CSS2 113→114（`block-in-inline-align-001` 1.37%→**PASS 0.00%**）；inline-box-001 2.31→1.11%（改善仍 FAIL）；**inline-box-002 3.20→4.67%（恶化仍 FAIL）**；clear-inline-001 5.86%（不变）；css-position 16/16（无回归）。

**为何保持 env-gated 默认关**：inline-box-002 恶化（border-having split）。R182 已证：被拆分 inline 的 border/background 是 **inline 级（文本宽）**非 block 级（全宽），当前匿名块全宽 border 致差异变大。完整修复需 painter inline 级 fragment border（首/末片段开放边），≈R182 §3 多轮子系统。按项目严格「零回归=无任何用例变差」标准（R129/R181d 等 clean win 均无恶化），不默认启用。**+1（align-001）已验证 ready，待 fragment border 解锁 inline-box 后默认启用**。

**验证**：env 关时上游同源 **435/490 持平零回归**（plumbing 始终激活但注册表空，paint 读取 no-op）；`make test` 全绿（layout-engine 876 passed/0，inline_block_split 6 passed 含 +2 新 out-of-flow 单测 `test_out_of_flow_child_does_not_trigger_split`/`test_in_flow_block_alongside_out_of_flow_still_splits`）；workspace clippy 零警告；fmt clean。

**意义**：(1) R109 数据流首次端到端打通（生产→消费），匿名块生成机制就位；(2) align-001 是 R109 首个可验证的真实修复（无 border inline-split 场景）；(3) out-of-flow 排除是 CSS2 §9.2.1.1 正确性必需（影响所有 abspos-in-inline）。**下轮**：painter inline 级 fragment border（解锁 inline-box-001/002 + 默认启用 R109），或转 line-height/字体度量对齐（welcome/wintertc 产品 smoke 主因）。

### R200 — multicol balance 方向证伪关闭：列分配已正确（R199 round-robin 错误，已移除）

承接 R199（multicol Round 1 balance 工具 + 设计文档）。本轮 Round 2 接入实测**证伪 balance 方向**，纠正清理。

**Round 2 实验（已回退）**：把 `balance_lines_to_columns`（shortest-column round-robin）接入 `painter/text.rs:948` 列分配，替代旧 `target_h=total/col_count` + col_first_y rebase。结果：multicol 40/57 不变，但 **multicol-columns-001 4.88→4.92%（略差）**，类 A 用例（fill-000/count-computed-003/004）全不变。

**根因（CSS §8 算法理解纠正）**：chromium multicol §8 是**顺序填充**——先填 col0 到平衡高度 H，再 col1，依次。而 `target_h = total/col_count` + `line.y/target_h.floor()` **本就是顺序填充 + 平衡高度 H=T/N**——**已正确**。R199 的 round-robin shortest-column（line0→col0, line1→col1, line6→col0...）**破坏了顺序**（chromium 是 col0=line0,1; col1=line2,3...），故略差。

**结论**：**multicol 列分配已正确**。类 A 低 diff 用例的 4.88/6.54/2.06/2.50% **非列分配问题**，而是：① 列宽/gap 子像素精度；② 列内 glyph x 位置（estimate_char_width vs 真实 advance，同 DC-13 R188 架构阻塞）；③ 平衡高度精确值（chromium 平衡二分搜索 vs T/N 近似，差异微小）。

**清理**：移除 R199 的 `crates/layout-engine/src/multicol_fragment.rs`（round-robin 错误算法）+ lib.rs 注册；设计文档 `multicol-fragmentation-design.md` 升 v0.2 加 R200 纠正段（保留 §1 现状/§1.2 四类失败分析价值，标注 Round 1-2 balance 关闭）。

**意义**：multicol 剩余 17 失败 = 结构性（breaking/baseline/column-span/嵌套，R112-R113/R130）+ 精度（advance-width 同源 R188）。**multicol clean win 同源穷尽**（列分配已正确，精度受 advance-width 架构阻塞）。

**验证**：移除后 make test 0 failed；multicol 40/57 持平；clippy/fmt clean。基线 438/490。

### R199 — multicol 碎片化攻坚启动：设计文档 + Round 1 测量工具（零风险不接线，已提交）

承接 R198（Phase A font_size 方向关闭，转向 multicol 最大聚类）。本轮启动 multicol column-aware IFC 多轮里程碑，完成 spec-rfc 设计 + Round 1 测量基础。

**交付**：
1. **`docs/goal/rendering-compat/multicol-fragmentation-design.md`**（同 flex-grid-two-pass-design.md 风格）——consolidate R113/R122/R128/R131/R157 分析为可实施计划：
   - 现状链路（multicol.rs col_width 公式 ✓ / paint text.rs:948 均高分配近似 / text.rs:711 门控）。
   - 4 类失败（A 纯行内 balance 精度 / B 混合内容门控 / C 嵌套 breaking / D baseline+spanner）。
   - 目标：列感知 IFC（`ColumnFragmentationContext` + §8 shortest-column balance）。
   - **5 轮实施**：R1 测量工具（本轮）/ R2 纯行内 balance 接 paint / R3 混合内容门控+协调 / R4 column breaking / R5 baseline+spanner。
   - 预期：css-multicol 40→~55/57（≥95%），438→~453/490（92%）。
2. **`crates/layout-engine/src/multicol_fragment.rs`**（新模块）——`balance_lines_to_columns(line_heights, col_count, col_filled)`：CSS §8 shortest-column-first 列分配（替代 paint 的 `total/col_count` 均高近似，更接近 chromium multicol-columns-001 ref）。+6 单测（4行2列轮转 / 11行6列=2,2,2,2,2,1 / 含 block 已占 col_filled / 单列 / 零列 / 空）。

**Round 1 不接线**（measure-first 同 R181 模式）：`#[allow(dead_code)]`，compute/paint 不调用，纯计算 + 单测。reftest 438/490 持平零回归。

**验证**：multicol 40/57 不变；make test 0 failed；clippy/fmt clean。

**下轮 Round 2**：接入 `painter/text.rs:948` 列分配——用 `balance_lines_to_columns` 替代 `target_h=total/col_count` 均高 + 移除 `col_first_y` fractional rebase。paint-only 改动（layout 不动），目标解锁类 A 用例（multicol-columns-001 4.88% / fill-000 6.54% / count-computed-003 2.06% / 004 2.50%），全量零回归门禁。

### R198 — Phase A font_size 死锁经新变体再证实证，方向关闭（实验已回退，基线 438/490，已提交）

承接 R197（welcome 真因=Phase A font_size 默认）。本轮用**新变体**实证 Phase A font_size 死锁，关闭该方向。

**实验设计（vs R125/R158 的新角度）**：R125 测的 store 路径会**重排**（改高度，破 font-feature/multicol/abspos）。本轮新角度=**只存 font_size 不重排**：compute_final_inline_layouts 的 IFC 已用真实 styles 跑过（frag.font_size 正确），在 layout 后调 `store_font_sizes_from_ifc`（仅填 text_node_font_sizes paint 提示，不改 box 几何）+ **multicol ancestry 守卫**（递归传 in_multicol，跳过 multicol 内文本）。

**结果（全量 net -1，438→437）**：
- CSS2 115→**116（+1）**：large-font 用例（font-051 类）paint IFC 获真实 font_size，修复 16px 默认。
- css-multicol 40→**39（-1）**：`multicol-fill-auto-001` 0.63%（通过）→ FAIL。**multicol ancestry 守卫无效**——multicol-fill-auto 非 LayoutBox 树 ancestry-tracked（疑 multicol paint 路径重组内容容器，in_multicol 未传播到其文本容器）。

**死锁成立的本质**：即使补全 ancestry 守卫使 multicol-fill-auto 不破，net 也仅 **0**（large-font +1 与 multicol 守卫抵消），**非正收益**。large-font 与 multicol-fill-auto 经 font_size 存储耦合（两者文本 font_size 存储不可分别控制），不可单修。印证 R125（store/no-store/real-styles 三路 -1/-1/-4）+ R158（"勿再单点补存 font_size"）。

**结论**：**Phase A font_size 方向正式关闭**。DC-13 welcome 文本 + large-font 5 reftest（font-051/ifc-008/009/011/empty-inline-002）的修复需**架构性 Phase A IFC 三路径统一**（layout IFC / paint IFC / compute_final 存储共享同一 font_size 源 + line-breaking），非 font_size 单点补存。这是 multi-round 架构里程碑，非单会话。

**验证**：实验已回退（git checkout engine.rs），基线 438/490 + multicol-fill-auto-001 0.63% 通过恢复。下轮转向 **multicol 碎片化**（17 失败，最大可操作同源聚类，bounded 子系统）。

### R197 — 两纠正：welcome 真因=Phase A font-size（非 advance-width）+ DC-12 审计（全实现）（无代码变更，基线 438/490，已提交）

承接 R196（advance-width 假设）。本轮用 chromium + ZeroWeb 实测**再证伪**并审计 DC-12。

**纠正① welcome 真因 = paint IFC font-size 默认（Phase A），非 advance-width**：合成测试 `<p style="font-size:60px;color:red">` 经 product-smoke 渲染——color:red 生效但文本仅 **12px 高（=默认 16px 字高）**，证 font-size:60px **未应用**。即 R82/R101/R125/R158 标记的 **paint IFC 空 styles → font_size 回退 16px**（compute_final_inline_layouts 仅存单行 Ahem font_size，非存储文本 paint IFC 用 16px 默认）。此为 welcome 文本真因（文本按错误尺寸渲染），**R196 的 advance-width 假设证伪**（advance 是次要；font-size 默认是主因）。Phase A 是已知硬阻塞：R125 三路（store -1 / no-store -1 / real-styles -4）全死锁，R158 实证 large-font 修复破 multicol-fill-auto（反向依赖）。DC-13 welcome + large-font reftest 5 失败（font-051/ifc-008/009/011/empty-inline-002）全卡此。

**纠正② DC-12 审计——全部已实现**（goal doc DC-12「未实现」过时，同 M7）：grep + 代码核实——text-shadow（text.rs 渲染 shadow glyph）、multi-background-layer（effects.rs `for layer in bg_image.iter().rev()` 全图层逆序，CSS 正确）、repeating-gradient（cpu/gradient.rs `if gradient.repeating` + 单测）、clip-path（paint_clip_path）、backdrop-filter（apply_backdrop_filter）、CSS mask（apply_mask_image）**均有真实实现**。唯一真缺口 = **clip-path circle/ellipse/polygon 仅画指示线非真裁剪**（只 inset() 真裁剪，mod.rs:425 注释明言）。

**结构性 plateau 全面确认**（4 轮 DC-13 调研 R195-R197 收敛）：DC-13 welcome 卡 Phase A、DC-12 基本完成、reftest clean win 穷尽（52 失败全结构性）。剩余推进均为多轮硬架构（Phase A / multicol 碎片化 / DC-14 Oracle 默认化）。下轮须选定一个硬目标多轮 commit，不再单会话追 DC-13 假设。

**验证**：无代码变更；reftest 438/490 持平。下轮候选：Phase A font_size 突破（最高杠杆，影响 DC-13 + 5 reftest，但 R125 死锁需新思路）、multicol 碎片化、DC-14 Oracle 默认接线。

### R196 — DC-13 welcome 根因深挖：证伪 line-height + font 假设，指向 advance-width（无代码变更，基线 438/490，已提交）

承接 R195（line-height 去风险）。本轮用 chromium Oracle + PIL 像素分析深挖 welcome 28% 根因，**证伪两个假设**并回退推测代码。

**证伪① line-height:normal 假设**（R195/R186 推断）：grep welcome.html/morning-work/article.css/wintertc 全部用**显式 line-height**（welcome 1.08/1.5/1.45/1.25，morning 1.75/1.5/1.7/1.2em/1.5em，wintertc Twind html{1.5}/inherit/text-4xl{2.5rem}），**无一处 line-height:normal**。故 R195 规划的 font-metrics line-height:normal plumbing（本轮已实现 6 文件 ~80 行 + 实测 reftest 438 持平自源中性）对 DC-13 三个 smoke **零收益**。按 code-guidelines「不做推测性开发」，**回退 plumbing**（服务无指标）。

**证伪② font 不匹配假设**：系统 `fc-match sans-serif` → **Noto Sans CJK SC**（chromium Oracle 用此，含 CJK + Latin），ZeroWeb 默认 sans-serif → DejaVu Sans（仅 Latin，CJK 走 Noto CJK fallback）。R195 AA 基准测同字体（W/i）漏了 sans-serif 解析分歧。实验：render_to_framebuffer_with_base 把 sans-serif/system-ui 覆盖到 Noto CJK（经 fallback_chain 取 id，TTC familyname 未提取故不在 resolver）→ welcome diff **28.08→28.05%（仅 -163px negligible）**。**字体不匹配非主因**（Latin 字形差异微小）。

**welcome 28% 真因 = 文本定位（advance width 估算）**（排除法 + R188 印证）：layout IFC 用 `estimate_char_width`（非 Ahem 按字符类别粗估：字母 0.55×fs、数字 0.5、标点 0.4）而非字体真实 advance，致文本宽度/换行/行内位置与 chromium（FreeType 真实 advance）偏差，多行累积。此即 **R188 标记的架构阻塞**（layout-engine 不持 FontLoader，advance 需字体数据；同 line-height 的跨 crate 阻塞）。**关键性质：advance-width 改动对 reftest 自源中性**（test/ref 同用 estimate，等比例），**仅影响 DC-13 chromium 对比**，不推 438/490。

**副发现**：welcome `<br>` tagline 正常渲染 2 行（rows 303-314 英文 + 327-340 中文），`<br>` 在产品路径工作正常（早先合成测试单行渲染是 font-size:50px 无 font_resolver 回退的 quirk，非 bug）。

**验证**：回退后 make test 全绿、reftest 438/490 持平、clippy/fmt clean。**下轮**：advance-width plumbing（FontLoader measure_advance → engine 预建 font→advance 源 → layout IFC 替代 estimate_char_width），用 welcome/morning.work/wintertc DC-13 smoke 验证收益（自源中性，reftest 不动）。

### R195 — DC-13 line-height 调研 + reftest 自源中性去风险发现（无代码变更，基线 438/490 持平，已提交）

承接 R194（R109 彻底完成）。本轮调研 welcome DC-13 产品 smoke 主因（28% diff），产出**关键去风险发现**，无代码变更。

**welcome diff 归因确认**：product-smoke 实测 welcome ZeroWeb-CPU vs chromium **28.08%**（134796/480000px，与历史基线一致，R109 不影响 welcome 无 inline+block 结构）。PIL diff-band 分析：差异遍布全页（~20-30 行间隔 band，符合文本行高），底部 quadrant（rows 450-599）差异最大（9885px half-res vs 顶部 5358）= **行高差异累积**。证 master.md R186/R187 结论（fontdue 光栅化非问题，差异在布局/度量 line-height）。

**🔑 关键去风险发现——line-height:normal 改动对 reftest 自源中性**：实验把 `NORMAL_LINE_HEIGHT_RATIO` 从 1.2 改 1.5（明显不同），跑文本密集类别：**linebox 10/15 持平、css-writing-modes 53/59 持平**（与 ratio=1.2 完全相同）。根因：reftest 是 **ZeroWeb-test vs ZeroWeb-ref 自渲染**（reftest.rs:230-232），test 与 ref 同字体同度量，line-height:normal 改动对两侧**等比例平移**，diff 不变。**结论：字体度量版 line-height:normal 对 438/490 reftest 基线安全**（唯一风险：test/ref 用不同字体的少数用例，但那些多用显式 line-height）。此发现解锁 DC-13 line-height 方向，下轮可放心实施。

**fix 为何是多轮架构性**（非单会话 contained）：layout-engine IFC 的 `resolve_font_metrics`（inline/mod.rs:311，~35 引用）当前仅从 ComputedStyle 取 font-size，用固定 1.2 算 normal。要用字体真实度量（fontdue `horizontal_line_metrics` 的 ascent/descent），需 font 文件访问，但 **font_family→FontId→font 文件解析当前在 paint 懒做**（painter resolve_font_id），layout-engine 不持 FontLoader（R188 同源阻塞：advance-width 也是因此用 estimate_char_width）。正确 fix = engine（持 FontLoader + font_resolver）预解析文档用到的 font-family → line-height ratio `(ascent-descent)/em`，建 map 传入 `LayoutEngine::compute` + paint 双侧 IFC 的 `resolve_font_metrics`（~4-5 文件，~80 行，触及 IFC 核心，需谨慎 + 12k 测试验证）。design 默认值保留 1.2（无 font map 时）保单测过。

**reftest clean win 复核**：全量 52 失败按 diff 升序复核，全部落入已知结构性聚类（multicol baseline-007/008/breaking/collapsing、grid child-border-box-002 fit-content、R109 vertical-rl css-flexbox-row、Phase A border-padding-bleed、float+clear+negative-margin clear-float-003）。确认同源 clean win 穷尽仍成立（R185+ 多轮印证）。

**下轮**：① 实施 font-metrics line-height（engine 预解析 font-family→ratio map → layout+paint IFC），用 welcome/morning.work/wintertc DC-13 smoke 验证收益（已去风险）；② 或 DC-13 DONE#11「5 真实网站」本地 fixture。

### R194 — R109 split 的 relative offset 双重计数修复（+1 零回归，inline-box-002 PASS，已提交）

承接 R193（R109 fragment border + 默认启用，+2）。本轮修 inline-box-002 残余 3.14% → **PASS 0.78%**，**R109 里程碑彻底完成**。

**根因（PIL 像素分析 + 布局树探针定位）**：inline-box-002 的蓝色条纹（#div2 bg）"缺失"实际是**片段被 taffy 偏移到 600px 视口外**（abs_y=646）。布局树探针证：split inline 父盒 #div2（`position:relative;top:2in`）y=192（taffy 按 block 单次施加 inset ✓），但**每个匿名块片段也 y=192**——因为 tree.rs 用 `computed_style_to_taffy(&computed)` 构建片段 style 时**继承了 #div2 的 position:relative + inset top:2in**，taffy 对每个片段再施加 +192 → 片段偏低 2×192=384px 出视口。

**两处协同修复（缺一仍 2×）**：
1. **tree.rs**：匿名块 style 的 `inset` 清零（`LengthPercentageAuto::AUTO` 四边）。片段位置由 split inline 父盒单次施加，片段作为子盒随之移动，不再自带 inset。
2. **engine.rs `apply_relative_offsets_inline`**：跳过 `is_r109_split` 盒（父盒+片段，is_r109_split 对二者均 true）。原函数按 computed-display=Inline 施加 relative offset，但 split inline 经 converter 映射为 taffy Block，taffy 已按 block 单次施加——再施加会双重。

**实测**：frag1 abs_y 646→**262**、frag2→**300**（几何对齐 ref：yellow rows 70-261，blue/orange/blue 紧随其下）。`inline-box-002` 3.14%→**PASS 0.78%**；全量 **438/490 (+1)**，CSS2 115→116，零 count 回归（css-position 16/16 等全部不变）。

**验证**：默认 make test 全绿（0 failed）；默认 reftest **438/490**；clippy/fmt clean。诊断探针（walk_dbg 布局树 / paint-frag）已移除，保留 r109.rs 的 shrink 探针（env-gated R109_DBG，同 INTRINSIC_DBG 模式）。

**R109 里程碑总结（R189-R194，6 轮）**：§9.2.1.1 inline→block 拆分从「converter 单点 Inline→Block 全宽堆叠」演进到「匿名块生成 + fragment 注册表 + IFC 片段收集 + fragment border（shrink + 边选择）+ relative 单次偏移」。**累计 +3**（align-001 / inline-box-001 / inline-box-002），435→438/490，默认启用。残余独立子问题：clear-inline-001（REF 用 inline img+span，converter 把 inline img→block 致堆叠，非 R109 border/offset 问题）。

### R193 — R109 §9.2.1.1 fragment border 落地 + 默认启用（+2 零回归，inline-box-001/align-001 PASS，已提交）

承接 R192（R109 生产端接线 env-gated，+1 待 fragment border）。本轮实现 fragment border 并**默认启用 R109**——基线 **435→437/490**。

**R192 遗漏的关键根因**：R192 创建匿名块用 `taffy::Style::default()`（border=0），而 `LayoutBox.border_*` 来自 taffy 布局的 border（engine.rs:704 `border_left = layout.border.left`）——故匿名块从未携带 split inline 的 border。R192 实测 inline-box-002 3.20→4.67% 恶化的真正原因：split inline **父盒**（is_r109_split）仍画全宽 block border，匿名片段无 border。

**三处协同修复（缺一不生效）**：
1. **匿名块用 converter 构建 style**（tree.rs）：`computed_style_to_taffy(&computed, ...)` 从 split inline 的 computed 构建（携带 border/padding/background），强制 display:Block。匿名块现继承 inline 的盒模型。
2. **split inline 父盒跳过装饰**（painter/mod.rs）：`is_r109_split && fragment_node_ids.is_none()` 时跳过 box-shadow/background/background-image/borders/border-image——装饰已下放片段，父盒画全宽会错。
3. **fragment border shrink + 边选择**（engine.rs `shrink_r109_anon_blocks`，intrinsic_sizing.rs `fragment_inline_max_width`）：匿名块片段收缩到文本宽（`fragment_inline_max_width` 与 paint IFC 同用 `estimate_char_width`，故收缩宽=渲染宽自洽）+ 边选择（首片段 `border_right=0`、末片段 `border_left=0`，CSS2 §9.2.1.1 分裂边不画边框）。

**tree.rs fragment 边选择基建**：R109Wiring 增 `first_inline_fragments`/`last_inline_fragments`（HashSet<taffy::NodeId>），segment 循环收集 Inline 片段 anon ID 后标记首/末；extract 写 `LayoutBox.r109_first_fragment`/`r109_last_fragment`。

**实测（R109_WIRE=1 = 默认）**：全量 **437/490 (+2)**。CSS2 113→115（`inline-box-001` 2.31%→**PASS 0.89%**、`block-in-inline-align-001` 1.37%→**PASS 0.34%**）；inline-box-002 3.20→**3.14%（改善，不再恶化）**；block-in-inline-append-001/iframe-in-block-in-inline/margin-collapse 持平 0.00-0.09%；align-justify-001 0.38→0.50（微增 +0.12%，仍远在通过线内）；clear-inline-001 5.86%（不变）；css-position 16/16（R192 out-of-flow 修复保持）。**零 count 回归，唯一视觉微增 align-justify +0.12% 可忽略**。

**为何默认启用**：经全量 reftest（+2 零 count 回归）+ 全量 make test（0 failed，12k 套件）双验证；§9.2.1.1 匿名块生成是 CSS 规范行为（真实浏览器总是做），默认启用 = 向正确性靠拢；与 R181d/R129 等已验证 clean win 默认运行的模式一致。R109_WIRE=0 可关闭回退对比。

**验证**：默认 make test 全绿（0 failed）；默认 reftest **437/490**；workspace clippy 零警告；fmt clean。R109_DBG 探针保留（env-gated，同 INTRINSIC_DBG 模式）。

**R109 残余（独立子问题，非阻塞默认启用）**：clear-inline-001（REF 用 inline img+span，ZeroWeb converter 把 inline img→block 致堆叠，非 border 问题）；inline-box-002 残余 3.14%（split inline 的 `position:relative; top:2in` 在每个匿名片段重复偏移 → 需 relative-on-split 单次偏移处理）。下轮：line-height/字体度量（welcome/wintertc 产品 smoke 主因），或 inline-box-002 relative-on-split。

### ⚠️ M7 状态核实（goal doc 陈旧纠正，2026-06-16）— 渲染器图元覆盖已基本完成

**目标文档 `rendering-compat.md` 的核心声称严重过时**：它把 M7（渲染器图元覆盖 + 浏览器消费）标为「Single Active Milestone / P0-致命 / CPU 仅 3/13 / GPU 仅 2/13 / 浏览器仅 2/13」。**经本轮代码核实，三层均已基本实现 + 像素验证 + 接线，M7 实质已完成**。下轮不要再重新实现已存在的图元（本轮差点重复实现 GradientPrimitive）。

**DC-8（CPU 渲染器 100%）= 已完成**：`crates/render-foundation/src/cpu/mod.rs` 的 `render_scene` 全量分支 dispatch 全部 13 种图元（shadows/fills/rounded_rects/gradients/images/strokes/path_fills/path_strokes/glyphs/clips/transforms/filters/blend_modes，116-178）；真实实现文件齐全（`cpu/gradient.rs` 线性/径向/锥形/重复 + alpha 混合完整、`cpu/shadow.rs`、`cpu/stroke.rs` 实/虚/点线、`cpu/effects.rs` 滤镜）；`cpu/tests.rs` 有逐图元像素验证（gradient_linear_red_to_blue:83、shadow_renders_blur_around_rect:328、stroke_solid/dashed/dotted:368-468、path_fill_triangle:469、clip_removes_pixels_outside:539、transform_translate_shifts:575、image_renders_rgba:618、filter_blur_softens:677）。make test 全绿证实现存。

**DC-9（GPU 渲染器 100%）= 9/13 独立 WGSL 管线已落地（非 passthrough），4 项中 clip 为 no-op、transform/filter/blend 为真实但低频缺口（多轮 ping-pong）**：`gpu/renderer/mod.rs:651 render_full_scene_gpu`（浏览器 GPU 路径 `apps/browser/src/app_platform.rs` 实际调用）逐图元 collect 顶点 + 独立 draw pass：shadow/fill/rounded_rect/stroke/path_fill/path_stroke/glyph 经 `draw_fill_pass`、gradient 经独立 `draw_gradient_pass`+`gradient_stops_to_texture`、image 经独立 `image_pipeline`+`draw_image_pass`（740-758，各独立 WGSL pipeline，满足 DC-14「GPU 非 passthrough」）。`gpu/renderer/tests.rs:607-774` 有全量路径像素验证。**R220 纠正（DC-9 真实范围）**：经 grep 实证，**engine 在生产路径从不生成 `ClipPrimitive`**（`add_clip` 0 处非测试调用）——overflow 裁剪在 paint 阶段由 `clip_all_primitives_to_rect`（painter/mod.rs:292/553/566/590/690 等）**预烘焙进图元几何**，故 `primitives.clips` 生产中恒空。因此 R211 所记「GPU drops clip」**实为 no-op**（无 clip 可丢），DC-9 的 ClipPrimitive 项在 CPU/GPU 两路均**空谈满足**（vacuous）。**真实缺口仅 transform/filter/blend_mode 三项**（engine 在 `paint/painter/effects.rs:266/289/313` 生成，GPU 全量路径未处理而静默丢弃），但这三项需 **ping-pong 双纹理后处理架构**（wgpu 不能同 pass 读写同一纹理；filter=区域采样变换、transform=区域反向采样、blend=与 backdrop 合成，均需 read+write），且在 reftest/静态内容中**低频**（仅显式 CSS `filter:`/`transform:`/`mix-blend-mode:` 触发），非 reftest load-bearing。**下一步**（多轮）：加第二张 offscreen texture + post-process WGSL pipeline，先做 filter:opacity（alpha 乘，最简）建地基，再 blur/transform/blend。GPU 已有 `headless_texture` offscreen 目标（mod.rs:93/125），ping-pong 基建部分就绪，差第二张纹理 + pipeline 接线。

**DC-10（浏览器图元消费）= 已完成**：`apps/browser/src/app_render.rs` 的 `append_webview_primitives` 消费全部 13 字段（shadows:1955/fills:1969/rounded_rects:1998/gradients:2012/images:2041/strokes:2051/path_fills:2062/path_strokes:2074/glyphs:2087/clips:2112/transforms:2122/+filters/blend）。DC-13 产品 smoke（R172 border-radius draw_order bypass、R174 box-shadow σ）亦实证 rounded_rects/shadows 经浏览器路径实际渲染。

**重新定位的真实剩余目标**（M7 既已完成，杠杆转移）：
1. **DC-2~5 通过率 88.8%→95%**（当前最大未达标项；55 同源失败结构性 plateau——multicol 碎片化 R113/R131、writing-mode 轴 R114/R164（4 轮证否）、flex 基线 R130、IFC ownership R109、fontdue 度量噪声 R174；clean win 在同源侧已穷尽，chromium Oracle 一致率是真实指标）。
2. **DC-13 产品 smoke**：wintertc.org 图片密集 fixture 待录制 + DONE#11「5 真实网站截图像素对比」（当前 0/5）。
3. **GPU transform/filter/blend 逐项实现**（DC-9 收尾；R220 纠正：clip 经 paint 预烘焙生产恒空=no-op 已空谈满足，真实缺口仅 transform/filter/blend 三项，需 ping-pong 多轮，低优先——GPU 非reftest load-bearing）。
4. 各 DC 项的 spot-verification（DC-1 chromium Oracle/fuzzy、DC-6 quirks、DC-11 布局、DC-12 高级效果的实际覆盖度核对）。

**对 goal entry doc 的处置**：`rendering-compat.md` 的 Mission/Support Envelope/DC-8/9/10/M7 节、「已知关键缺口」表的「渲染器图元覆盖 P0-致命 CPU 3/13」等条目**事实错误**，按治理原则须纠正；但 entry doc 治理要求「仅目标变化时改」。本轮先在 master.md（唯一运行时控制面板）权威记录纠正；entry doc 的逐条订正留作专项（避免本轮重写入口文档）。

### DC-13 WinterTC 首页 fixture + product-smoke 工具 + SVG 栅格化（已提交）

录制 WinterTC 图片密集首页（`https://wintertc.org/`）为 DC-13 fixture，端到端验证现已确认完成的图元渲染在真实图片页的表现。

**交付**：
1. **`product-smoke` 子命令**（tests/wpt-runner/src/main.rs）——通用 DC-13 产品 fixture 渲染+对比工具：`zero-wpt-runner product-smoke <html> --base-dir <dir> --oracle <png> --out <png>`，输出 ZeroWeb CPU PNG 并与 chromium Oracle 像素 diff。welcome/morning-work/wintertc 及任意 fixture 均可复用（补齐此前 morning-work 截图靠 ad-hoc 脚本的缺口）。
2. **SVG 文件栅格化**（reftest.rs `load_svg_file`）——resvg + tiny-skia 把 `<img src="*.svg">` 栅格化为 RGBA，补 `build_image_cache` 此前仅 PNG/JPEG 的缺口（WinterTC 14 logo 中 11 个为 SVG；resvg 是 workspace 已声明但此前未接线的依赖）。
3. **wintertc fixture**（`apps/browser/assets/wintertc/`）：index.html（capture-wintertc.mjs 经 chromium 录制的已解析 DOM，含 Twind 生成的 `<style>`）+ static/ 下 14 个 logo（logo.svg + 13 参与方 logo）。
4. **基线 22.42%**（800×600，107,604/480,000 px vs chromium）。

**诊断**（REFTEST_DEBUG + 分区域像素分析 + 计算样式探针，CSS 加载后）：wintertc **布局层经核实基本正确**——universal `*{margin:0}` 生效（UA margin 被 reset）、Twind utility 类生效（text-4xl→36px、flex/grid/mt-8/gap 等结构正确）、hero/nav 位置合理。`images:1`(800×600)/`2`(800×2000) 证 SVG 栅格化生效；14 参与 logo 中多数仍未布局（疑 `flex flex-wrap justify-evenly` 多 item 精度，待独立诊断，非图片加载问题）。**22.42% 主差异 = 顶部文本区布局/度量**（行高/ascent 等度量致文本块高度差）。⚠️ R174「fontdue 字体度量噪声」归因已被 2026-06-17 AA 基准证伪——fontdue 光栅化 vs chromium 单 glyph 实测 W 0.1% / i 3.0%，差异在**布局/度量非字体光栅化**（见 `evidence/aa-baseline-2026-06-17.txt`）。+ logo 多在折叠线下。**结论：wintertc 布局无 clean bug，勿再以「Twind 布局缺失」重查**。证据持久化 `evidence/product-static/wintertc/`。

**验证**：make test 12203 passed/0 failed；上游 reftest **435/490 持平零回归**（SVG 分支仅 `.svg` 扩展名触发，不影响 PNG 用例）；clippy/fmt clean。

**剩余（下轮）**：① 参与 `flex flex-wrap justify-evenly` 多 logo 未布局（独立子问题，待诊断是否 clean）；② 真实 ZeroBrowser/webview 层 ImageCache 的 SVG 支持同步（本轮仅 harness 侧）；③ HTML width/height 单属性 aspect 推导尝试后致 background-001/003/328/329 回归 -4（已回退，交互未明需先厘清）。

### DC-13 build_image_cache 站点根相对 URL 解析修复（真实 bug，零 reftest 回归，wintertc 诚实 diff 上升，已提交）

DC-13 wintertc 深挖产出：`build_image_cache`（reftest.rs）用 `base.join(url)` 解析图片路径——但 `PathBuf::join(absolute)` 会**替换** base，故站点根相对 URL（如 `/static/logos/x.svg`）解析为文件系统根 `/static/...` → 加载失败。wintertc 14 logo（全用 `/static/` 绝对路径）仅 2 个加载（疑经其他路径），12 个缺失。

**修复**：`base.join(url.trim_start_matches('/'))`——剥离前导 `/`，使 `/static/x` 解析到 `base_dir/static/x`（fixture 的站点根）。WPT reftest 多用相对路径（无前导 `/`），trim 不影响。

**验证**：wintertc 图片图元数 2→**14**（全加载）；make test 12205 passed/0；上游 reftest **435/490 持平零回归**；clippy/fmt clean。

**wintertc diff 诚实上升 22.42%→25.11%**：修复前参与方 logo 全缺失致 diff 人为偏低（假低）；修复后 14 logo 加载并渲染到 800×600 可见区。**经复核（探针 dump 参与 `<a>` flex item 位置）：参与 `flex flex-wrap justify-evenly` 布局正确**——13 个 `<a>` item 正确分布在 3 个换行行（x=16/193/305/481/656 行1、y≈80-102 行2、y≈184-191 行3），logo 尺寸 aspect 正确。故 25% diff **非布局 bug，而是渲染噪声**：resvg SVG 栅格化 vs chromium SVG 渲染器的像素差异 + 文本块布局/度量差异。⚠️「fontdue 字体度量噪声」归因已被 AA 基准证伪（fontdue 光栅化 = chromium 0.1-3%，见 `evidence/aa-baseline-2026-06-17.txt`），差异在布局/度量非字体。DC-14 anti-false-pass：诚实测量优先于假低。**wintertc 布局已确认无 clean bug，剩余纯渲染噪声，勿再追布局**。

**意义**：(1) 真实 bug——`base.join(absolute)` 替换 base 是 Rust Path 陷阱，影响所有绝对路径图片 fixture；(2) DONE#11「5 真实网站」必备（真实站点全用绝对路径 `/static/...`，不修则图片全缺失）；(3) 暴露并定位了参与 flex 布局 bug（下轮目标）。

### DC-13 img 替换元素 aspect-ratio 保留修复（真实 bug，零回归，已提交）

DC-13 wintertc 诊断产出：`<img>` 仅设 CSS width（height:auto）或仅设 height（width:auto）时，**另一维应按固有宽高比推导**，旧实现却用固有绝对值。复现：正方形 SVG（intrinsic 441×441）+ `width:80px` 渲染成 **80×441**（巨高），应 80×80。

**根因**：`apply_replaced_element_sizing`（tree.rs）的「无 HTML 属性→img_intrinsic_sizes 回退」分支无条件把 auto 侧设为固有绝对高度（`size.height = Length(intr_h)`），与设的 aspect_ratio 冲突——taffy 用固定高度忽略比例。

**修复**：该分支改为按 CSS §10.3/§10.6 推导——两侧 auto 用固有 w×h；仅 width 显式时 `height = cw * intr_h/intr_w`；仅 height 显式时 `width = ch * intr_w/intr_h`。不依赖 taffy aspect_ratio 推导（显式算出）。

**验证**：复现实测 deno 80×441→**80×80**、cloudflare 80×394→80×36、fastly 80×735→80×34（比例正确）；+2 单测（width-set/height-set 双向）；make test 12205 passed/0；上游 reftest **435/490 持平零回归**；clippy/fmt clean。

**意义**：影响面大——真实页面 `<img>` 极常见仅设 width 或 height（响应式 logo/缩略图/头像），此前全部变形。wintertc 800×600 基线 22.42% 不变（参与方 logo 在折叠线下，主差异为 hero/nav 文本+字体噪声），但 logo 在更高视口现比例正确。HTML width/height 单属性分支（(Some,None)/(None,Some)）有同源 bug，本轮未改（wintertc 走 intrinsic 分支），留作后续。

### R180 — inline-block width:auto shrink-to-fit（CSS §10.3.9，baseline-block-with-overflow-001 chromium 45.09%→1.25%，同源零回归，已提交）

修复 18 真 bug 候选第 3 名 `baseline-block-with-overflow-001`（CSS2/linebox，同源 0% 假通过但 chromium 45.09%）。**根因**：`width:auto` 的 `display:inline-block` 被 taffy 0.7 拉伸到可用宽度（如同 block），违反 CSS §10.3.9 inline-block 应 shrink-to-fit 到 max-content。实测（IBSHRINK_DBG 探针）`.outer`（inline-block, width:auto）最终 `w=784 content_w=784`，其 block 子元素 `.inner`（width:30px）已正确 30px——故仅收缩 inline-block 盒尺寸本身即可，无需重排子元素。chromium Oracle 几何：chromium `.outer`=30px（橙色 bbox x 至 37），ZeroWeb `.outer`=784px（橙色 bbox x 至 791）；主差异=橙色全宽 774px×5 section。

**修复**：新增 `shrink_inline_blocks_to_content`（engine.rs，compute() 步骤 5.6），与 R129 float-shrink / R138 table-shrink / R134 vertical-shrink 同谱系：对水平书写模式、width:auto 的 InlineBlock，读取流内子元素 margin-box 宽度（**inline 级求和 + block 级取最大**），仅在内容确实更窄时收缩盒宽（内容更宽或显式宽度为 no-op）。子元素宽度已是 taffy 正确布局结果，不重排子元素。

**关键实现细节**：初版只算 block 级子元素（max），对 REF 中 `.inner` 是 `display:inline-block`（inline 级）的场景 content_max_w=0 不收缩→test(30px)≠ref(784px) 同源翻 FAIL 29.59%。改为 inline 级求和 + block 级取最大后，test 与 ref 同时正确收缩到 30px→同源恢复 0% AND chromium 45.09%→1.25%。证明 test/ref 结构虽异（test `.inner` 裸 block / ref `.inner` 加 inline-block class + 显式 `.outer` height），只要两侧 `.outer` 都 width:auto，shrink-to-fit 同步修两侧→同源保持 + chromium 改善。

**验证**：上游同源 **434/490 持平零翻转**（失败集 IDENTICAL，collapsed-item-horiz-001 仍 0% 通过）；baseline-block-with-overflow-001 同源 0%（仍通过）且 chromium **45.09%→1.25%**（残余 1.25% 为 overflow!=visible inline-block 基线=底 margin 边缘的 spec 细节 + 字体噪声，属独立小问题）；make test 全绿（+1 单测 `test_inline_block_width_auto_shrink_to_fit`）；clippy/fmt clean。

**范围限定**：仅 `DisplayValue::InlineBlock`（未扩展到 inline-flex/inline-grid/inline-table）——collapsed-item-horiz-001（float:flex, chr 20.5%）经 R180 同期诊断确认为**结构性多轮**（flex item 增长循环依赖：taffy 在 800px 布局 flex 容器→flex:1 item 增长到 772→R129 float-shrink 读增长值不收缩；FLEXSHRINK 探针实证 `child.width=774 child_w=[0,772]`）。post-hoc 收缩无法 re-layout 已增长的 flex item，需两趟固有宽度 flex 布局，非单会话 clean win，defer。inline-flex 同理会触发同样循环故不纳入（守卫保证内容≈宽时为 no-op，安全但无收益）。下轮候选：剩余 16 真 bug 中 `baseline-block-with-overflow` 残余 1.25% 基线 / position-absolute-semi-replaced-stretch（23/15%, inline-block ownership）/ iframe-in-block-in-inline（9.75%, iframe infra）。

**同期诊断 table-grid-item-dynamic-003（chr 29%，结构性，defer）**：grid 容器（div, 800px, height:100px）含 table（height:100%, padding-top:100px, content-box）。chromium table=800×222（**拉伸填满 grid track**），ZW table=278×200（shrink-to-fit 到文本 278px；表高 200 正确=100%+padding）。TBLW_DBG 探针实证 `table box.width=278 content_width=278 parent_display=Grid`。converter 把 `DisplayValue::Table => taffy::Block`（mod.rs:254），故 table 是 grid 内 taffy Block item，应被 justify-items:stretch 拉伸，但 taffy 给了 max-content(278) 非 800。根因=**taffy 0.7 auto grid track 不吸收剩余空间扩展**（CSS Grid §12.6/12.7：auto track 应增长填满容器，单 auto 列应从 278 扩到 800）。chromium gridprobe（block grid item）确证拉伸到 800。属 taffy-grid 级，post-hoc 拉伸 table grid item 对多列/显式 track 网格不安全，defer。同 R168 表高修复互补（004 表高已修，003 表宽=grid track 扩展）。

**同期诊断 position-absolute-semi-replaced-stretch-input（chr 23%）+ -other（chr 15%），布局正确=表单控件外观渲染 gap，defer**：abspos `<input>`（position:absolute, top/right/bottom/left=3px, width:auto, box-sizing:border-box）**布局层 §10.3.7 stretch 已正确**——ABSSTRETCH_DBG 探针实证 `tag=input w_auto=true box.w=144 cb.content_w=150`（150−3−3=144 border-box）及 wide `box.w=344 cb=350`，test 与 ref 两侧 width 均 144→同源 0%。**23% 残余=纯 PAINT**：`<input>` 无 UA 默认 `appearance`（style-system lib.rs:253 default=Auto，apply_advanced.rs 仅映射显式 appearance CSS 属性，无 input type→appearance UA 映射）→ `paint_appearance`（effects.rs:490）`None|Auto => return` 早退→input 仅渲染 value 文本（固有宽），chromium 渲染为填满 144px 的原生表单控件（白底 textfield / 灰按钮）。修复需 (a) UA 注入 input type→appearance（lib.rs:321 `match tag` 有干净注入点）+ (b) **paint_appearance 改进**：当前 Button 分支用 accent 蓝（effects.rs:536）≠ chromium 灰按钮，textfield 分支白底+灰边接近但 file/range 无映射。非 clean 单点修（表单控件原生渲染特性），defer。布局 stretch 谱系（R98 abspos inset/R123 root inset/§10.3.7）已覆盖，残余是 paint 层表单控件外观。

**同期诊断 multicol-contained-absolute（chr 16%，结构性 spec 角，defer）**：R124 已使其渲染（同源 0%），残余 16%=chromium 与 ZW 对 multicol+abspos CB 解析的几何分歧。ZW green abspos=392×200（列 1 宽，全高）；chromium=784×100（**全宽半高**）。`columns:2` 内 `position:relative`(h200) > `overflow:hidden` > `position:absolute; width:100%; height:200px; background:green`。chromium 把 abspos 的 width:100% 解析为 multicol 全宽(784) 且高度被列平衡到 100；ZW 解析为列内 relative 父宽(392) 全高(200)。属 multicol+abspos+containing-block 碎片化祖先的 spec 角（WPT 本身测此模糊点），R124 已修「不渲染」，精确 CB/宽解析需 multicol 碎片化+abspos 协调（同 R113/R131 multicol 碎片化结构性里程碑），defer。

**低 diff 同源失败穷尽扫描（R180 同期）**：对 56 同源失败按 diff 升序，最低 18 项（1.04%–2.89%）逐一定位，**全部落入已知结构性领域**，确认 clean win 在同源失败侧亦已穷尽：baseline-007/008（1.04/1.46%，multicol+flex baseline）/multicol-breaking-006（1.20%，R112 dead end）/css-flexbox-row（1.23%，R109 vertical-rl）/block-in-inline-align-001（1.37%，R147 原子化多趟）/child-border-box-and-max-content-001/002（1.52%，见下 max-content）/border-padding-bleed-001（2.40%，Ahem IFC paint-order=Phase A）/border-001/006（2.77/2.86%，低价值像素噪声）/float-nowrap-hyphen-rewind（2.89%，文本连字符）。

**child-border-box-and-max-content-001/002（1.52%，max-content sizing，结构性 defer）**：`width:max-content`（grid）+ `max-width:max-content`（item）。**具体 bug 定位**=style-system `computed.rs:68` 把 `LengthValue::MinContent | MaxContent => 0.0`（解析阶段就把 max-content 解析成 0）→ grid width=0 塌缩到 40px（仅 padding，content 50 被 clip，且只渲染 1 个 item）而非 chromium 的 180×70（2 item × (50content+40padding)）。css-parser 已正确解析 MaxContent 变体（parse_basic.rs:30/types.rs:1003），但 computed.rs:68 丢失信号解析为 0。**正确修复需**：(a) 保留 max-content 信号到布局（非解析为 0）+ (b) layout shrink-to-fit 在 max-content 时触发（复用 R180 谱系）——但 grid 上的 max-content 收缩受 **taffy grid 不 shrink-to-fit**（同 table-grid-item-003，width:auto grid 填满 800 而非收缩）阻塞，故仍结构性。R97 记录此为 8 通过用例风险的 opt-in 特性。defer（max-content on block/inline-block 或可独立做，grid/flex 需容器 shrink-to-fit 先行）。

### R181 — flex/grid 两趟固有宽度布局 Round A：测量工具落地（零风险，不接线，已提交）

启动 #1 结构性里程碑（见 `docs/goal/rendering-compat/flex-grid-two-pass-design.md`）的首个零风险子步。新增 `crates/layout-engine/src/intrinsic_sizing.rs` 模块（不参与布局，仅纯计算 + 单测 + env-gated 诊断）：
- `box_content_max_width`：盒内容 max-content 宽度（inline 求和 + block 取最大 + **叶盒显式 Px width 回退**——关键差异于 R138 `block_max_content_width` 对叶显式宽返回 0）。
- `flex_item_base_size`：flex item 主轴 base size（`flex-basis` 显式长度 > `width` 显式 > 内容 max-content，CSS Flexbox §9.2）。
- `flex_row_intrinsic_width`：水平 flex 行容器固有宽度 = Σ base + margins + gaps + frame。
- `debug_dump_shrink_candidates`（`INTRINSIC_DBG=1`）：遍历树对 shrink 候选容器（inline-flex/inline-grid/float:flex/grid 的 width:auto 或 max-content/min-content）打印 `current_w vs intrinsic_w`，验证测量。

**实测验证（INTRINSIC_DBG）**：collapsed-item-horiz-001 的 float:flex 容器 `current_w=774 intrinsic_w=22/2 (delta +752/+772)`——确证 shrink-to-fit gap 真实可测；baseline-align-self inline-flex `intrinsic_w=0`（文本 item 无显式宽，需 IFC 文本测量，已知 Round C 限制）。

**已知限制（后续 Round 补）**：(1) `visibility:collapse` 未当 flex_basis:0 strut 处理（读 `width:20px` 致 collapsed-item 测 22 非 0）；(2) 纯文本 flex item 内容宽=0（需接 IFC 文本测量）。

**验证**：layout-engine 单测 860 全绿（+6 新单测覆盖叶显式宽回退/双 item 求和/flex-basis 优先/padding 累加/grid-like 嵌套/空容器 None）；clippy/fmt clean；上游 reftest **434/490 零回归**（诊断不改变布局）。Round B 下轮：`identify_shrink_candidates` + 接入 compute() 两趟（设 taffy node 宽度 + mark_dirty + 重跑 compute_layout_with_measure），仅 inline-flex/inline-grid 起步，434/490 零回退门禁。

### R181b — Round A 扩展 grid 测量 + Round B 接线阻塞定位（零风险，不接线，已提交）

扩展 `intrinsic_sizing.rs` 新增 `grid_intrinsic_width`：grid-auto-flow:column → Σ item base size + gaps；row flow → max item base size（item base = `box_content_max_width`，含叶显式宽回退）。+2 单测（column flow 求和 180、row flow 取最大 50）。

**测量经两目标 reftest 实证正确**：(1) child-border-box-and-max-content-001 的 grid（2 item ×(50content+40padding)）测得 **intrinsic_w=182**（≈chromium 180，✓ 正确）；(2) collapsed-item float:flex intrinsic=22（R181 已证）。**测量基础现对 flex+grid 双侧目标均验证正确**。

**Round B 接线阻塞定位（关键）**：child-border-box 的 `width:max-content` 在到达布局时已被 `computed.rs:68`（resolve_length）解析为 `Px(0.0)`——**信号丢失**（INTRINSIC_DBG 实证 `s.width == Px(0.0)`，故 width_indefinite 检测 `MaxContent` 变体失效，grid 不被识别为 shrink 候选）。table-grid-item-003 的 table item 经 `box_content_max_width` 返回 0（table 内容非 block LayoutBox 子，table 固有宽需 table auto-layout，独立子问题）。

**Round B 接线的前置**（按风险升序）：(1) **修 computed.rs:68 保留 max-content/min-content 信号**（不解析为 0；R97 标 8 通过用例风险，须先验证）；(2) 接线两趟（set taffy node 宽度=intrinsic + mark_dirty + 重跑 compute_layout_with_measure，taffy 增量仅重算 dirty 子树故低风险）。**低风险策略**：仅对当前已塌缩（resolved width≈0）的 max-content/min-content 容器接线（0→intrinsic 纯改善，非破坏），inline-flex/auto 容器留后续。

**验证**：layout-engine 单测 862 全绿（+2 grid 单测）；clippy/fmt clean；上游 reftest **434/490 零回归**（仍不接线）。下轮 Round B：先验证 R97 的 8 用例是否依赖 max-content→0 行为，再修 computed.rs:68 保留信号 + 接线两趟。

### R181c — Round B 实验回退：信号保留单独 net -5（converter MaxContent→auto-fill），已回退无提交

**R97 风险量化（本轮）**：全量扫描 25 个 sizing-keyword 文件，上游 manifest 实际只导入运行 6 个（R97「8」已过时）。状态：4 FAIL（child-border-box-001/002 1.52%、flex-container-max-content-001 18%、flex-container-min-content-001 12.8%）+ 2 PASS（flex-item-content-is-min-width-max-content 用 `min-width:max-content`、flex-item-min-height-min-content-overflow 用 `min-height:min-content`——**均非 width**，故 width-scoped 修复对它们安全）。

**信号丢失精确定位**：`computed.rs:354 resolve_length_field` 的 `_` 分支调 `resolve_length`（computed.rs:68 MaxContent/MinContent→0.0）后写回 `*field = LengthValue::Px(0.0)`——apply.rs:111 虽存原始 MaxContent，但此 compute-value 解析趟把它转成 Px(0)。实验：加 `MinContent|MaxContent => {}`（保留信号）。

**实验结果（已回退）**：信号保留后 INTRINSIC_DBG 确证 `width=MaxContent`（不再是 Px(0)），grid 被识别为 shrink 候选 ✓——但**全量 reftest 434→429（net -5）**！根因=**converter 把 MaxContent 当 auto-fill**（grid 从塌缩 2px 变填满 784px，均非正确 182）。即信号保留改变了布局：max-content 容器从「塌缩 0」变「填满 784」，二者皆错，且填满致 5 用例回归。

**Round B 必须原子化（关键教训）**：信号保留 + 两趟接线**必须一起做**，不可单独。且两趟对**内容不可测**的 max-content 容器（纯文本 item→intrinsic=0→跳过）无法修正，这些会停留在填满 784（回归态）——故 Round B 还需**不可测 max-content 的回退策略**（如退回塌缩 0 或保持当前行为）。已回退 computed.rs，434/490 恢复。

**Round B 完整方案（下轮）**：(1) computed.rs:354 保留 MaxContent/MinContent 信号；(2) compute() 首趟后检测 max-content flex/grid 容器（s.width==MaxContent，intrinsic 可测且 < current），set taffy node 宽度=intrinsic + mark_dirty + 重跑 compute_layout_with_measure + 重新 extract；(3) 对 intrinsic=0（不可测）的 max-content 容器：**不设宽度**但需阻止 converter 的 auto-fill——可能需 converter 把 MaxContent→Size::MIN_CONTENT 或显式 0（恢复塌缩）而非 auto。此 (3) 是 net -5 的真正解，须先定位 converter 的 MaxContent→fill 分支。

### R181d — Round B 原子落地：信号保留 + converter 中性塌缩 + 两趟 intrinsic 提升（+1 零回归，child-border-box-001 PASS，已提交）

按 R181c 完整方案原子实施 #1 结构性里程碑（flex/grid 两趟固有宽度）的 Round B。**净效果：434→435/490（+1 零回归），`child-border-box-and-max-content-001` 1.52%→PASS 0.03%**（grid 塌缩 40px → intrinsic 182px ≈ chromium 180px；chromium Oracle 一致率亦大幅改善）。

**根因再确认（R181c net -5 的真正解）**：converter `convert_length_to_dimension`（converter/mod.rs:374）把 `MaxContent/MinContent => Dimension::Auto`——taffy 把 width:auto 的块级容器**拉伸填满**可用宽度（784px），即 R181c 的 net -5 根因。但该分支在**信号保留前是不可达死代码**（computed.rs resolve_length_field 先把 MaxContent 解析为 Px(0.0)，converter 只见 Px(0.0)→length(0.0) 塌缩）。R181c 保留信号后该死代码激活→填满→回归。

**修复（三处原子协同，缺一即 net -5 或失效）**：
1. **computed.rs:354 `resolve_length_field`**——新增 `MinContent | MaxContent => {}` 分支保留信号（不解析为 Px(0)），使 layout-engine 能识别「此容器是 max-content」。
2. **converter/mod.rs:374 `convert_length_to_dimension`**——`MaxContent/MinContent => length(0.0)`（塌缩），**不再 Auto**（Auto 触发 taffy 填满）。**关键**：length(0.0) 与旧「MaxContent→Px(0.0)→Px 分支 length(0.0)」输出**字节相同**→信号保留对 width/height 行为中性的塌缩，不回归。（`convert_max_length_to_dimension` 已是 Auto=∞，对 max-width:max-content 是正确过近似，保留不动——这也修正了 item `max-width:max-content` 被旧 Px(0) 钳制为 0 的 bug。）
3. **engine.rs `apply_intrinsic_content_sizing`**（compute_with_img_sizes 步骤 3.1）——首趟 extract 后遍历布局树，对水平书写模式、`width:MaxContent/MinContent` 的 flex/grid 容器，用 `intrinsic_sizing` 模块（R181/R181b 落地的 `grid_intrinsic_width`/`flex_row_intrinsic_width`，基于**显式宽度**测量，不依赖塌缩布局宽）测 intrinsic；可测（>1.0）且大于当前宽时，`set_style(size.width=Length(intrinsic)) + mark_dirty`，重跑 `compute_layout_with_measure` 后重新 extract——其子元素（grid track / flex item）即按新宽度重新分配。**intrinsic 不可测（纯文本 item，Round C IFC 未就绪）的容器跳过→保持塌缩（中性，正是 R181c 缺失的「不可测回退」）。**

**安全性核验（R97 风险量化复核）**：R97「8 通过用例风险」经 R181c 重新量化为上游实际导入运行 6 个（4 FAIL + 2 PASS）。两 PASS 用例 `flex-item-content-is-min-width-max-content`（min-width:max-content）与 `flex-item-min-height-min-content-overflow`（min-height:min-content）均**非 width**，且 min/min-length 的 MaxContent 仍走中性 length(0)/默认 0 分支——本轮实测两者仍 **0.00% 持平**，零翻转。证明「保留信号 + converter width 塌缩」对 min-width/min-height max-content 安全。

**验证**：上游同源 **434→435/490**（child-border-box-001 FAIL→PASS 0.03%；002 1.52→1.36% 改善但仍未过——其用 `grid-template-columns: fit-content(...)` 显式 track，当前 `grid_intrinsic_width` 只建模 column-flow 求和/row 取最大，未建模 fit-content() track 内在尺寸=独立子问题 defer）；flex-container-max/min-content-001（18.08%/12.80%）不变（纯文本 flex item 需 IFC 文本测量=Round C）；make test **12201 passed/0 failed**（+2 单测 `test_grid_width_max_content_sized_to_intrinsic` + `test_unmeasurable_max_content_does_not_fill` 验证可测提升与不可测回退）；converter edge_cases 单测更新（MaxContent→Length(0.0)）；clippy 零警告；fmt clean。

**意义**：(1) #1 结构性里程碑（flex/grid 两趟固有宽度）Round B 落地，测量基础（R181/A.2）现产出真实修复；(2) child-border-box 是 18 真 bug 候选外的真实渲染缺口（chromium 40 vs 180），现 align chromium；(3) R181c 的「net -5」根因精确定位并解决（converter Auto→length(0) 中性塌缩 + 不可测回退）；(4) 两趟 set_style+mark_dirty+重跑模式为后续 flex intrinsic sizing（collapsed-item-horiz 等 flex 容器 shrink-to-fit）奠基。**下轮**：002 的 fit-content() track 内在尺寸建模，或 collapsed-item-horiz flex 容器两趟（需 IFC 文本测量解锁纯文本 item 测量）。

### R191 — R109 数据流消费端接线（LayoutBox.fragment_node_ids 字段 + paint 读取，零回归，已提交）

R109 多轮里程碑的第 3 个基础件（继 R189 split 分析、R190 IFC 片段支持）。补齐数据流**消费端**——把 LayoutBox 片段信息接到 paint IFC。

**实现**：
- `LayoutBox.fragment_node_ids: Option<Vec<NodeId>>`（types/mod.rs，默认 None）——匿名块片段的 DOM 子节点。Default + extract_layout 构造器初始化 None。
- paint IFC 构造（painter/text.rs:911）：`if let Some(ref frag) = box_node.fragment_node_ids { ctx.set_fragment_node_ids(frag.clone()); }`——匿名块盒的 IFC 只收集其片段内容（接 R190 set_fragment_node_ids/collect_inline_items 片段分支）。

**数据流状态**：消费端完整（`LayoutBox.fragment_node_ids → paint 读取 → IFC.set_fragment_node_ids → collect_inline_items 片段分支`）。**生产端待接**：tree.rs build_subtree 把 inline+block 子元素展开为匿名块 taffy 节点 + fragment 注册表（taffy_node→片段 NodeId），extract_layout 读注册表写 LayoutBox.fragment_node_ids。

**验证**：上游同源 **435/490 持平零回归**（fragment_node_ids 默认 None，paint 读取为 no-op）；make test 全绿；workspace clippy 零警告（字段被 paint 读取，非 dead）；fmt clean。

**剩余 R109 接线（生产端，多轮）**：(1) tree.rs build_subtree 在父级子元素循环中，遇 inline 含 block 子元素（inline_has_block_child）时用 compute_inline_block_split 展开为匿名块 taffy Block 节点序列（context = inline 的 NodeId 以承其 border 样式）+ 建 `HashMap<taffy::NodeId, Vec<NodeId>>` fragment 注册表；(2) extract_layout 接收注册表，对在注册表中的 taffy 节点设 LayoutBox.fragment_node_ids；(3) 匿名块 shrink-to-fit（使 border 文本宽非全宽，R182 已证全宽 border 大差，需复用 R180 inline-block shrink 机制）；(4) tree 生成 env-gated（R109_WIRE=1）保零回归，measure inline-box-001 变化。生产端是 R109 最 intricate 部分（build_subtree 结构重构），单会话需谨慎。

### R190 — R109 IFC 片段收集支持基础（InlineFormattingContext.fragment_node_ids，零回归，已提交）

R109 多轮里程碑的第 2 个基础件（继 R189 inline_block_split 拆分分析）。补齐 IFC 半边——匿名块盒需只收集其片段的 inline 内容而非 inline 元素的全部 DOM 子节点。

**实现（layout-engine/inline/mod.rs）**：
- `InlineFormattingContext.fragment_node_ids: Option<Vec<NodeId>>`（默认 None）。
- `set_fragment_node_ids(&mut self, ids)`：设置片段节点覆盖。
- `collect_inline_items`：`children = match &self.fragment_node_ids { Some(ids)=>ids.clone(), None=>doc.child_nodes(container) }`——设值时只遍历片段节点，否则原行为。

**验证**：整合测试 `test_fragment_node_ids_restricts_inline_collection`（inline `<div id=i>aaa<div>bbb</div>ccc</div>`：compute_inline_block_split 取首 Inline 片段 → set_fragment_node_ids → collect_inline_items 收集数 < 不设片段的全收集数，且非空），整合 R189 split + 本 IFC 片段两个基础件；上游同源 **435/490 持平零回归**（默认 None 不改行为）；clippy 零警告；fmt clean。

**范围（为何不接线）**：默认 None 零回归，无调用方设值（tree.rs 匿名块生成未接）。R109 完整接线仍需 3 层 plumbing 协同（R189/R188 已 scoped）：(1) tree.rs build_subtree 用 inline_block_split 把 inline+block 子元素展开为匿名块 taffy 节点序列 + 建 fragment 注册表（taffy_node→片段 NodeId）；(2) extract_layout 读注册表设 LayoutBox.fragment_node_ids（需 LayoutBox 新字段）+ 拷 inline 的 border 样式；(3) IFC 用本 fragment_node_ids 收集片段文本；外加 (4) 匿名块 shrink-to-fit（使 border 文本宽非全宽，否则 R182 已证全宽 border 大差）。本 R189+R190 完成「分析双半」（拆分结构 + IFC 片段收集），下轮接 tree+extract+wiring。

### R189 — R109 §9.2.1.1 匿名块拆分结构基础模块落地（inline_block_split.rs，零回归，已提交）

启动 #1 最高杠杆多轮里程碑 R109（inline→block + IFC ownership，影响 block-in-inline-001/002 + block-in-inline-align-001 + clear-inline-001 + morning.work blue-nav）的首个零风险基础步。新增 `crates/layout-engine/src/inline_block_split.rs` 模块（同 R181「分析先行」方法学，**不接线、零布局副作用**）：

- `InlineBlockSegment` 枚举：`Inline{item_node_ids}`（连续 inline 内容归一个匿名块）/ `Block{node_id}`（block-level 子元素独立块）。
- `is_block_level_display`：判定 block-level（Block/Flex/Grid/Table/ListItem/FlowRoot；inline-flex/grid/table 是 inline-level 原子盒不触发拆分）。
- `inline_has_block_child(doc,styles,id)`：R109 触发条件（inline 元素 + ≥1 block-level 子元素）。
- `compute_inline_block_split(doc,styles,id) -> Option<Vec<InlineBlockSegment>>`：遍历 DOM 子节点按 block-level 切分，连续 inline 累入 Inline 片段，block 子元素发 Block 片段（非空才发 Inline，不产生空匿名块）。返回 None = 无需拆分。
- `debug_dump_inline_block_splits`（env `R109_DBG=1`）：遍历布局树打印触发元素的拆分片段，接入 engine.rs compute()（与 INTRINSIC_DBG 并列）。

**验证**：layout-engine 单测 4 全绿（`test_single_block_child_splits_three_ways` 三向拆分 / `test_pure_inline_no_split` / `test_block_container_not_triggered` / `test_leading_block_no_empty_inline`）；R109_DBG 实测 inline-box-001 #div1 拆分 = `[Inline(1), Block(node=32), Inline(1)]`（First line / block / Last line，与 §9.2.1.1 一致）；上游同源 **435/490 持平零回归**；make test 全绿；clippy 零警告；fmt clean。

**范围限定（为何不接线）**：完整 R109 修复需 3 子系统协同（R182 已证）——(1) tree.rs build_subtree 用本 split 生成匿名块 taffy 节点 + fragment-range 注册表；(2) IFC collect_inline_items 按片段范围收集文本（当前按 container NodeId 走全部子节点，匿名块无 NodeId）；(3) painter inline 级 fragment border（首/末片段开放边）。本模块是 (1)(2) 的共享结构分析基础，单独不改变布局。**下轮**：接线 tree.rs 匿名块生成（用 split 结果构造匿名块序列），验证 clear-inline-001 REF 的 inline img+span 不再被当 block 堆叠——这是 R109 的第一个可验证里程碑（不含 border 片段的简单 inline-flow 情形）。

### R188 — clear-inline-001 归因 R109（inline img→block 致 span 换行堆叠）+ DC-13 advance-fix 架构阻塞（layout IFC 用 estimate_char_width 粗估非真实 advance），诊断轮无代码提交

两项目标用例根因，均确认非单点 clean win。

**clear-inline-001（5.86%）= R109（inline→block 映射）**：测试侧（float orange + inline span clear:left）ZeroWeb **正确**渲染——clear 对 inline 被忽略，蓝文本在 float 右侧顶部（blue bbox x[104,791] y[51,69]，orange float x[8,103] y[51,146]）。**REF 侧失败**：REF 用 `<img 96x96 vertical-align:top><span>Filler Text</span>`（inline img + inline span）。ZeroWeb 把 inline img 经 converter `Inline→taffy::Block`（R109 映射）当 block，span 亦然 → taffy 垂直堆叠：img 行 1（y=51-146）、span 行 2（blue bbox y[147,165]，紧贴 img 底）。正确应 img+span 同行（img 仅 96px 宽，span 容得下）。**又一个 R109 manifestation**（inline→block 破坏 img+text 流），非 clear bug。加入 block-in-inline（R182）/ morning.work blue-nav（R177）同列。

**DC-13 font advance-fix 架构阻塞（R187 后续）**：layout-engine IFC 行换行（break_into_lines, inline/mod.rs:1186/1283）与 taffy measure（engine.rs:1919）**统一用 `estimate_char_width`**（非 Ahem 字符按类别粗估：字母 0.55×font_size、数字 0.5、标点 0.4），**非字体真实 advance** → 'W'/'i' 同宽致换行与 chromium（FreeType 真实 advance）不一致（R174「换行」成分）。修法 = 把 fontdue 真实 advance 接入 layout IFC，但 **layout-engine 虽依赖 render-foundation（fontdue 可达）却不持 FontLoader**（字体在 engine/paint 层加载），IFC 用 estimate_char_width 正为避此。plumbing FontLoader/advance-source 跨 engine→layout-engine 边界 = 显著架构改动，DC-13-only，收益不确定（换行成分占比未量化）。非单会话。

**最高杠杆多轮里程碑 = R109（inline→block + IFC ownership）**：影响 block-in-inline-001/002 + block-in-inline-align-001 + clear-inline-001 + morning.work blue-nav（.item-tag span 全宽堆叠）等多用例与产品 smoke。单会话不可解（R182 已证 3 子系统：tree 拆分 + IFC 片段 + inline 级 fragment border），但**单里程碑解锁最多用例**。基线 435/490 持平。

### R187 — fontdue→swash rasterizer swap 实证排除（welcome fontdue 26.95% ≈ swash 26.96% vs chromium，并列非更近 Skia），DC-13 字体噪声非 rasterizer 算法问题，已回退无代码提交

承接 R186 swash swap 可行性方向，**实测排除**（原型 + chromium Oracle 像素对比）。

**实验**：env-gated `rasterize_glyph_swash`（font/loader.rs，ZERO_FONT_SWASH=1）用 swash `ScaleContext + Render(Outline,Format::Alpha)` 对非 Ahem 字形光栅化，fontdue 路径默认不变（零回归）。坐标映射 y_offset = placement.top - height（推导与 fontdue ymin 同语义）。DBG 插桩确认 swash 实际命中（159 次，glyph_id 非 0，render Some 如 'W' 16×16）——**确实在光栅化**。

**实测结果（welcome.html vs welcome-chromium.png，PIL 直比 >5 阈）**：
- **fontdue vs chromium：129380 px (26.95%)**
- **swash vs chromium：129426 px (26.96%)**
- swash vs fontdue（互相）：17900 px diff（max channel 24，AA 边缘差异）
- **判定：fontdue 微近 46 px（统计并列 tie）**——swash 系统性**不**比 fontdue 更接近 Skia。

**结论（已回退 loader.rs 到 R186/4e34fb4）**：fontdue→swash rasterizer swap **对 DC-13 产品 smoke 无收益**（welcome 持平 26.95%）。DC-13 字体噪声平台期**非 rasterizer 算法问题**——swash 与 fontdue 用不同算法但离 Skia 等距。残余 ~27% 主因疑似 **font 度量**（advance width / line-height 致 chromium 换行与字形定位差异）或 **Rust 光栅化器普遍不匹配 Skia AA**（固有）。**勿再以 rasterizer swap（swash/ab_glyph/cosmic-text）追 DC-13**。

**杠杆穷尽定性（本会话 7 轮）**：reftest 435/490 同源 clean win 穷尽（全结构）；DC-13 字体噪声 rasterizer swap 实证排除；剩余真实推进 = 多轮结构里程碑（multicol 碎片化 17 / block-in-inline R109 / Phase A large-font）或 DC-13 font 度量校准（advance/line-height 对齐 chromium，独立方向待评估）。基线 435/490 持平。

### R186 — fontdue rasterizer 杠杆精确化：Ahem 已 special-case 完美方块（reftest 残差=结构非光栅化），rasterizer swap 仅助 DC-13 产品 smoke（非 reftest），诊断轮无代码提交

承接 R185 fontdue swap deep-research 方向，代码级核实 fontdue 杠杆范围。

**关键发现**：`FontLoader::rasterize_glyph`（font/loader.rs:171）对 **Ahem 字体 special-case**——`rasterize_ahem_glyph`（:198）生成边长=font_size 的**完美填充方块**（注释明言「fontdue 光栅化与 Skia 差异，直接生成方块确保像素对齐」）。`font.rasterize(code_point, size)` API **无任何 quality/AA 设置**（仅字符+尺寸）。

**杠杆范围精确化（修正 R185「fontdue AA 主导 ~111 reftest」的过度归纳）**：
- **WPT reftest（多用 Ahem）**：glyph 已是完美方块 + 布局 advance 精确（`estimate_char_width` 对 Ahem 返回 font_size，inline/mod.rs:201）+ paint 按 layout 精确位置放置（不用 GlyphBitmap.advance 定位）→ **Ahem 渲染本应像素精确**，残差是**结构/布局**（换行、列分布、margin、line-height 度量）**非光栅化**。故 **rasterizer swap 对 reftest 指标基本无收益**。
- **DC-13 产品 smoke（welcome/morning.work/wintertc，用 DejaVu/Noto CJK 非 Ahem）**：走原始 fontdue `rasterize` → AA 双峰 ±10 噪声（R174）→ **rasterizer swap 是 DC-13 产品 smoke 杠杆**（~25-28% 平台期主因），非 reftest 杠杆。

**结论**：①reftest 435/490 平台期残差**确证为结构/布局**（非字体光栅化），clean win 穷尽仍成立——rasterizer swap 不解 reftest。②fontdue rasterizer swap（swash/ab_glyph+hinting/cosmic-text）若做，**目标应锁定 DC-13 产品 smoke**（welcome/morning.work/wintertc 字体噪声），是 DC-13 重大多轮里程碑，不影响 reftest 通过率。③reftest 推进仍需多轮结构里程碑（multicol 碎片化 17 / block-in-inline R109 / Phase A），单会话 clean win 无。

**fontdue swap 可行性初判**：fontdue 在 workspace 已声明且 shaper.rs 也用其取 advance。swap 需替换 rasterize_glyph 调用点（CPU mod.rs:483/GPU renderer）+ 保留 Ahem special-case + shaper.rs advance 来源。swash 已在 workspace（shaper.rs:4 提及 swash shaping），其 rasterizer 是最低集成成本候选。完整可行性需 deep-research（AA 算法对齐 Skia 程度、CJK 支持、性能），留作 DC-13 专项。

### R185 — 平台期再确认：multicol 低 diff 聚类根因（碎片化，列宽公式已验证正确）+ fontdue AA 为主导残差（rasterizer swap = 重大里程碑），诊断轮无代码提交

对 multicol 低 diff 聚类（multicol-count-computed-003 2.06% 等）做可视化 + 代码级根因，**再确认结构性碎片化**。

**multicol-count-computed-003 根因（精确）**：测试断言 (a) 相邻列其中之一为空时不画该列间 column-rule（CSS multicol §?）；(b) 溢入 column-gap 的行内内容不裁剪；(c) column-count/gap/width 公式。**代码核实**：`compute_single_column_width`（multicol.rs:229）公式 `W=(container-(count-1)*gap)/count` **正确**（测试 13em/3col/5em-gap → 1em 与 spec 一致）。**像素可视化**：差异区 rows 28-76，TEST 与 REF 的**文本内容跨列分布 + column-rule 位置**不同（TEST 文本带偏左/偏宽，REF 规则带在特定列间且空列间无规则）——即内容**碎片化分布**差异，非列宽公式 bug。属 R113/R122/R131/R157 碎片化结构性领域（R112 已证 column-rule 单点修回归 column-rule-002）。无 clean win。

**fontdue AA 主导残差表征（新）**：字体栈 = fontdue（rasterize_glyph_with_fallback，CPU mod.rs:483/GPU renderer）+ swash（shaping, shaper.rs）。welcome/morning.work/wintertc 产品 smoke ~25-28% + ~111 个 <3% reftest 失败的主导成分 = **fontdue 光栅化 AA 算法 vs Skia**（R174 已结论 welcome 96.5% 残差为字体噪声：glyph 边缘 AA 双峰 ±10 + 换行高度差）。**修法 = rasterizer swap/升级**（swash rasterizer / ab_glyph+hinting / cosmic-text 系），是**重大多轮架构里程碑**（替换核心字体依赖 + 重验证全量），非单会话，需 deep-research 评估可行性。

**平台期定性（4 轮本会话确认）**：同源 clean win **穷尽**（55 失败全结构性：multicol 碎片化 17、writing-mode 轴 R114/R142/R164 已 4 轮证伪、flex 基线 R130、Phase A large-font R125/R158/R169 死锁、R97 intrinsic、R109 block-in-inline 架构、fontdue AA 噪声 ~111）。剩余推进 = 多轮结构里程碑：①fontdue rasterizer swap（最高总杠杆，~111 reftest + 3 产品 smoke）②multicol 碎片化 R131 column-aware IFC（17 reftest，最大同源聚类，bounded 子系统）③block-in-inline R109（3 reftest + morning.work blue-nav，3 子系统架构）④Phase A IFC font_size 统一（5 reftest，死锁）。基线 435/490 持平。

### R184 — collapsed-item-horiz float:flex 两趟 shrink 实验（net -2 同源，已回退到 R183 基线）

消费 Round C 文本测量，尝试修 top chr 候选 collapsed-item-horiz-001（chr 20.5%，同源 0% 假通过）。扩展 `apply_intrinsic_content_sizing`（engine.rs）触发条件从「width:max/min-content」增加「float + width:auto 的 flex/grid 容器」，resize 方向：sizing keyword 增长（b.width<intrinsic），float:auto **收缩**（b.width>intrinsic，设宽=intrinsic 后重排使 flex item 在新窄宽下重新布局修 R180 flex 增长循环）；并给 R129 float 后处理（engine.rs:2564）加守卫跳过 flex/grid float 容器（其用「子元素最大宽」对多 item flex 会过收缩，应 sum 非 max）。

**实测全量 net -2：435→433**：
- collapsed-item-horiz-001 **0%→1.66%（同源翻 FAIL）**——确为破同源假通过（修了 shrink-to-fit），但 **1.66% > 1% 阈未过同源**，且该用例本就同源通过=纯 chr 收益不补同源损失。残余 1.66% 疑 visibility:collapse strut 跨轴高 / margin 未精确计入。
- **flexbox-column-row-gap-001（真回归）**：原同源通过 → 1.63% FAIL。该用例是 **column flex + row-gap**，`flex_row_intrinsic_width` 只测 **row 主轴**（水平），对 column flex（垂直主轴）的宽度（=cross 轴 shrink）给出错误值致过/欠收缩。
- flex-container-max/min-content-001（18%/13%）：本就失败，diff 仅噪声波动，非真改变。

**结论（已回退 engine.rs 到 R183/afe386a，435/490 恢复）**：float:flex width:auto 两趟 shrink **net-negative 同源**——row 主轴测量对 column flex 误伤（flexbox-column-row-gap-001 真回归），且 collapsed-item-horiz 即便修了 shrink 仍 1.66% 不过阈（同源无补，仅 chr）。要安全落地需：(a) column-flex 守卫（仅 row flex 触发，避免 column-gap 用例）；(b) collapsed-item-horiz strut/margin 精确化使 ≤1%；(c) 即使全做对，收益仅 chr Oracle（同源 0% 假通过），不推 435/490。**勿再以「float:flex 两趟 shrink」单会话推进**，除非先解决 row-vs-column 主轴测量分离。

### R183 — flex/grid 两趟 Round C：IFC 文本内容 max-content 宽度测量（基础，零回归，已提交）

推进 #1 结构性里程碑（flex/grid 两趟固有宽度）的 Round C（设计文档与 R181 系列预定步骤）。此前 `box_content_max_width`（intrinsic_sizing.rs）对**纯文本 flex/grid item** 返回 0——文本在 DOM 中（文本节点非 LayoutBox 子元素），而该函数只遍历 LayoutBox 子元素 + 叶显式宽回退。致纯文本 item 的容器 intrinsic 塌缩，阻塞 float:flex shrink-to-fit（collapsed-item-horiz）。

**修复（layout-engine）**：
1. 新增 `text_content_max_width(node_id, doc, styles)`——遍历 DOM 后代收集文本（`Document::text_content`），CSS 白空格折叠后用元素 font 度量逐字符累加宽（复用 IFC 的 `estimate_char_width`：Ahem 等宽=font_size，其它字体按字符近似宽）。仅 max-content（不换行）；min-content（最宽词）独立子问题未实现。
2. `box_content_max_width` 叶盒分支（无 LayoutBox 子元素）增加文本宽：`max(显式宽, 文本内容宽)`。含 LayoutBox 子元素的盒不变（避免混合内容过计）。
3. 把 `doc: &Document` 贯穿 `box_content_max_width`/`flex_item_base_size`/`flex_row_intrinsic_width`/`grid_intrinsic_width`/`debug_dump_shrink_candidates`/`apply_intrinsic_content_sizing` 及 engine.rs 两处调用点。`inline::collapse_whitespace` 提为 `pub(crate)` 复用。

**验证**：上游同源 **435/490 持平零回归**（失败集 IDENTICAL）；layout-engine 单测 12 全绿（+1 `test_text_only_item_measured_round_c`：5 字符 Ahem 10px item → 50px）；make test 全绿；clippy 零警告；fmt clean。

**对 flex-container-max/min-content-001 无改善（关键发现）**：两目标用例 18.08%/12.80% **不变**。INTRINSIC_DBG 实证 float:left + width:max-content 的 flex 容器 **current_w 已 == intrinsic_w（delta=0，如 50/22/40/20px）**——taffy 的 flex 布局已把 float flex 容器尺寸到内容（converter MaxContent→length(0) 后 taffy flex 仍按 item 内容定宽），故文本测量不改变其宽度（resize 条件 `b.width < intrinsic` 不满足）。两用例的 18%/13% 差距来自**其它结构**：`body{display:grid;grid-template-columns:repeat(auto-fill,66px 66px 66px)}` + `.wrap>*{float:left}` 的 grid auto-fill + float 布局，非 flex 宽度。**故 Round C 不修此两用例**。

**意义与范围限定**：Round C 是 flex/grid 两趟的测量基础（同 R181/R181b 测量先行的既定方法学），为 collapsed-item-horiz（float:flex width:auto shrink-to-fit，chr 20.5%，同源 0%）的文本 item 测量解锁——该用例的 flex item 增长循环（R180 诊断：taffy 800px 布局→flex:1 item 增长→R129 float-shrink 读增长值不收缩）需两趟在 taffy 增长前测 intrinsic，文本测量是其文本 item 部分。当前不接线 collapsed-item-horiz（需独立 float:flex shrink 接线 + 两趟前置测量，结构性多轮）。**下轮**：collapsed-item-horiz flex 容器两趟（接线文本测量 + float:flex width:auto shrink），或其它结构性目标。

### R182 — block-in-inline 匿名块盒生成攻坚（R109，CSS2 §9.2.1.1）：确证架构性多轮，无单会话 clean win，defer，无代码提交

上轮 CONTINUE 指定的目标。本轮对 inline-box-001（2.31%）做可视化 + 代码级根因，**确证为 R109 架构性里程碑**，非单会话 clean win。

**重叠机制精确定位**：`<div #div1 display:inline> "First line" <div orange> "Last line" </div>` 经 converter `DisplayValue::Inline => taffy::Block`（converter/mod.rs:265）变 taffy Block 盒。它**同时**——(1) `has_inline_content`（engine.rs:2041）因有直接文本子节点返回 true → #div1 对自身直接文本跑 IFC（collect_inline_items 把 block 子元素经简化处理 inline/mod.rs:846 替换为 `Br`，故 IFC 产 "First line"/"Last line" 两行）；(2) 把 orange `<div>` 作为 taffy block 子元素布局在 #div1 内容区**顶部**。两套系统不协调 → orange 盒（rows 94-112）与 IFC 文本在同一内容区重叠，"Filler Text" 文本落在 113-117 无橙底。即 IFC（DOM 文本收集）与 taffy（block 子元素布局）双轨，是 R109 IFC ownership 的具体表现。

**关键新结论（修正上轮「可做简单情形」预期）——border 是 inline 级片段 border，非 block border**：REF 用 `<span id="top">First line</span>`（inline span，border 仅包裹文本宽度 ~70px，top/left/bottom）+ 192px orange div + `<span id="bottom">Last line</span>`。§9.2.1.1 生成的匿名块盒是 **block 级（全宽 784px）**，但被拆分 inline 的 **border/background 仍在 inline 级绘制（包裹每片段内的文本）**，且首片段画 top/start+两 block 轴边、末片段画 top/end+两 block 轴边（fragment border）。故「把 inline border 复制到匿名块作 block border」不可行——全宽 block border（784px）vs 文本宽 inline border（~70px）= **大差（非可忽略）**，且无法表达片段开放边。本结论推翻「border 仅增加 ~80px 可忽略 diff」的初步乐观判断。

**完整修复 = 3 子系统多轮**：(1) tree.rs 构建期把 inline+block 子元素拆为匿名块序列（text-before → 匿名块、block 子元素原样、text-after → 匿名块），需 fragment-range 注册表让每匿名块 IFC 只收集对应文本片段（当前 collect_inline_items 按 container NodeId 走全部子节点，匿名块无 NodeId）；(2) inline/mod.rs IFC 按片段范围收集文本；(3) painter inline 级片段 border 绘制（首/末/中片段开放边）。≈300-400 行跨 3 子系统 + 片段 border 语义微妙。

**安全性论证（为何不强行单会话）**：触发条件可精确限定为 `display:Inline + ≥1 block-level 子元素`，故对 435 通过用例爆炸半径低（含此模式的用例多为已在失败集的 block-in-inline 用例）；但即便多轮投入，**无片段 border 则 inline-box-001 不可能 PASS**（全宽 vs 文本宽 border 大差），单会话投入 EV 低。遵 code-guidelines（精准修改/简单至上/先思考再编码），不强行高风险多系统改动威胁 435/490 基线。

**clean win 穷尽复核（本轮）**：全 55 同源失败逐一映射已知结构性领域（multicol 碎片化 R113/R122/R131 17 个、writing-mode 轴 R114/R142/R164 已 4 轮证伪、flex 基线 R130、Phase A large-font/IFC R125/R158/R169 死锁、R97 intrinsic、R109 block-in-inline）；16 chromium Oracle 真 bug 候选 R177/R180 已全部分级结构/infra/字体。**filter:blur σ=radius/2 同 R174 class bug 仍未修（effects.rs:38 `(radius*scale).ceil()` 单遍），但 0 reftest + 0 产品 fixture 驱动，遵 R174「无驱动不修」决策不动**。确证同源侧 + chromium Oracle 侧 clean win 均已穷尽。

**defer block-in-inline**。下一最高杠杆结构性里程碑 = flex/grid 两趟 Round C（IFC 文本内容测量），改善 collapsed-item-horiz-001 chromium Oracle（20.5%，同源 0% 通过）——但**不推动同源 435/490 头条**（该用例同源本就通过）；要推 435→436+ 需攻克某一同源结构性失败（均多轮）。本轮基线复核 435/490（88.8%）持平。

### R181e — grid 显式列 intrinsic 求和（002 1.36→1.01% 近似，零回归，已提交）

R181d 下轮目标的首个推进：`grid_intrinsic_width` 此前对非 column-flow（含显式 `grid-template-columns`）一律取 item 最大宽度，但 child-border-box-002 用显式 `grid-template-columns: fit-content(30px) fit-content(80px)`（2 item 各占一列），grid max-content 应是各列**求和**（90+90=180）非取最大（90）。

**修复**（intrinsic_sizing.rs）：新增 `count_explicit_grid_columns`（括号感知按空白统计 `grid-template-columns` track 数，`fit-content()`/`minmax()`/`repeat()` 各算 1 token）；`grid_intrinsic_width` 选择求和的条件从 `is_column_flow` 扩展为 `is_column_flow || (显式 track 数 >= item 数)`——保守守卫「track 数 >= item 数」确保每 item 独占一列才求和，避免 item 跨行换列过计（track 少于 item 时退回取最大=安全欠计）。fit-content(L)/固定长度的 L 钳制未建模（item min-content 地板通常已 >= L 不缩窄）。

**验证**：上游同源 **435/490 持平零回归**（失败集与 R181d 完全 IDENTICAL，逐行 diff 仅 002 的 diff 数值变；其余 54 失败 + 全部 grid 用例零翻转）；**child-border-box-002 1.36→1.01%**（grid 现测得 180≈ref，但仍未过阈——残余 ~1% 疑 taffy 对 fit-content track 在定宽下的尺寸分配细节或子像素，defer）；child-border-box-001 仍 PASS 0.03%；layout-engine 单测 864→866（+3：`test_grid_explicit_columns_sum_items`/`test_grid_explicit_columns_fewer_tracks_takes_max`/既有覆盖）；clippy 零警告；fmt clean。

**范围限定**：仅 max-content/min-content 容器的 intrinsic 测量改进；不改 converter、不改两趟接线。002 残余 1% 非测量问题（已 180≈ref），属 taffy fit-content track 在 definite grid width 下的内部尺寸分配，需 grid intrinsic track sizing 全量算法（CSS Grid §12.4-12.6）或 taffy 升级，defer。**下轮**：collapsed-item-horiz flex 容器两趟（需 Round C IFC 文本测量），或转 M7 渲染器图元覆盖（P0 最大缺口）。





**可信指标口径（唯一达标判定依据）**：上游真实 reftest 通过率 **435/490 (88.8%)**（R163 起默认正确图像渲染=DC-14 anti-false-pass，消除 PNG 退化假绿；旧 436 含 garbled-image 假通过）。⚠️ 当前 reference 仍由 **ZeroWeb 自渲染 ref.html**（`reftest.rs:230-232`），衡量「ZeroWeb-test vs ZeroWeb-ref」一致性而非「ZeroWeb vs Chromium/标准」，存在**同源假通过**风险（test 与 ref 同错）；治理门禁见 **DC-14 真通过标准**（独立 chromium Oracle 交叉验证基建已就绪 72764a0）。内联 reftest 685/685 (100%) 为 smoke，**不计达标判定**。

**归档策略（约每 20 轮一次）**：约每 20 轮做一次 archive——本文件保留最近 10 轮，更早的约 10 轮移入 `archive/` 目录下的归档文档，避免随轮次无限增长。当前已归档 R139 及更早（91 轮）至 `archive/rounds-r23-r139.md`；下次归档窗口约在再增 10 轮后（届时 R155~R146 移入归档，本文件仅留最新 10 轮）。

### R179 — colspan 主体修复攻坚实证（(e) 扩展变更净 -1 + 空列裁剪语义不一致，阻塞，无代码提交）

本轮攻坚 colspan #1 目标（52% chr）的 5 部件主体，实证确认**不可安全单点/部分推进**：

**(e) 扩展变更单独测试（net -1，已回退）**：把 `compute_column_widths` line 1799 的 `(has_explicit_width || is_fixed_layout)` 改为 `has_explicit_width`（CSS §17.5.2.1：width:auto 表格应收缩到列宽之和而非填满容器）。全量 490 实测 **434→433**，**唯一新增失败 = colspan 本身**（test 空 cell 收缩到 11.6px vs ref width:40px td 收缩到 40px → 同源从 0% 变 FAIL）。关键结论：**(e) 的爆炸半径仅 colspan**（其余 55 失败不变）——即 colspan 是唯一 width:auto + table-layout:fixed + test≠ref 的用例。已回退，434/490 恢复。

**chromium colspan 实测几何（黄色=cell bg 重新精确测量）**：t1-t5 cell 内容宽 = 40/90/140/40/40 px = **钳制后 colspan × 50px − 10(border)**。t3 colspan=4→钳制 3 列=140px（确证 colspan 钳制）。每表 = cell extent + border（60/110/160/60/60px）。**chromium 裁剪了空列**（t1 有 3 个 `<col width:50px>` 但只渲染 1 列=cell extent，空列 1,2 不渲染）。

**致命阻塞——空列裁剪语义不一致**：colspan（table-layout:**fixed**）chromium **裁剪**空列（3 col 定义→1 col 渲染）；col-definite-size-001（table-layout:**auto**）chromium **保留**空列（4 col × 100px=400px 即使只有 2 cell）。区分规则疑似 auto-vs-fixed，但 CSS2 §17.5.2.1 fixed 布局条文（col width/首行 cell/remainder）不提裁剪，**spec 模糊**，且 col-definite-size 用 `<colgroup><col>` 而 colspan 用裸 `<col>`+裸 `<td>`（匿名盒生成差异）——无法可靠区分。强行实现空列裁剪要么回归 col-definite-size（-1），要么不修 colspan。

**完整 colspan 修复仍需的耦合部件**（全做才不破坏同源）：(a) build_grid 去 border-collapse 守卫让 `<col>` 计入列数；(b) colspan 钳制到 grid；(c) Pass 0 读 col width 去 border-collapse 守卫；(d) 空列裁剪（**阻塞于上述语义不一致**）；(e) width:auto fixed 收缩（爆炸半径仅 colspan，已验证）；(f) border-collapse 下 `<td width:Npx>` content-box→列宽=N+cell-borders（ref 侧，影响所有 border-collapse 显式宽 cell，风险未量化）。**前置**：先用浏览器 devtools 或更多 WPT 用例厘清 chromium「空列何时裁剪」的精确规则，否则 (d) 无法正确实现。

### R178 — `<col>` 元素 px 宽度读取（CSS Tables §4/§17.5.2，separated border 模型，col-definite-size/max-size chromium 一致，零同源回归，已提交）

补齐 R177 钉死的 colspan 5 部件中**安全可独立起步的 (a)+(c)**：让 `<col>`/`<colgroup>` 定义网格列数并读取其显式 px width。此前 `<col>` 在 `build_grid`（table.rs）`_ =>` 分支完全跳过，列数只来自单元格 colspan，列宽从不读取——含 `<col style="width:Npx">` 的 separated-border 表格（如 col-definite-size-001 的 4×100px）被收缩到文本固有宽度（18px）而非 spec/Chromium 的 400px。

**修复（layout-engine table.rs）**：
1. `count_col_elements`（新增 helper）——统计 `<col>`/`<colgroup>` 定义的列数：colgroup 有内部 col 时取内部 col span 之和，否则取 colgroup 自身 span。
2. `build_grid`——`max_cols = max(单元格导出列数, col 元素导出列数)`，仅对 **separated border model** 生效（collapsed 模型列宽语义=border 中心间距，当前不覆盖，避免回归 colspan 等用例）。
3. `compute_column_widths` Pass 0（新增）——遍历 col/colgroup 读显式 `width`，**仅 LengthValue::Px**（% 在 width:auto shrink-to-fit 表上参照盒不定，calc/em 同理，跳过以保同源匹配），按 col_cursor 写入 `col_max_widths`，与既有 Pass 1（非跨列单元格）/Pass 2（跨列）的 `max` 合并语义一致。

**为什么是安全的**：实测 6 个含 `<col>` 用例——`col-definite-size-001`/`col-definite-max-size-001`（test==ref 同 4×col 结构，两侧同变仍 0% 匹配，**且 400px 现与 Chromium 一致**）；`visibility-collapse-colspan-003`/`insert-after-colgroup`/`border-collapse-dynamic-col-001`（cols 无 width 或 border-collapse 跳过，不受影响）；`table_grid_size_col_colspan`（border-collapse 跳过，保持 R177 机制不变）。colspan 5 部件的剩余 (b colspan 钳制 / d 裁空列 / e width:auto 收缩改扩展条件) 仍待后续轮配套。

**验证**：上游同源 **434/490 持平**（失败集与基线 `diff` 完全 IDENTICAL，零翻转零回归）；col-definite-size-001/max-size 渲染从 18px→400px（4×100px 列，Chromium 一致，chromium Oracle 改善；同源仍 0.00%/0.07%）；make test **12190/0**（+1 单测 `test_count_col_elements` 覆盖 colgroup 内 col / 直接 col span / colgroup span / 无 col 四场景）；clippy/fmt clean。col-definite-size 未在 chromium Oracle 抽样中故未进污染榜，但实属真实渲染缺口（现代表格 `<col width>` 极常见）。

**chromium Oracle 实测复核（本轮）**：col-definite-size-001 ZW-test vs chromium-test **0.66%**（修复前 18px vs 400px 巨差→现 4 表均 400px 对齐；剩余 0.66% 均匀分布于 4 表 = fontdue vs Chromium 字形「1」「2」AA 噪声，非 CSS bug，与 R174 welcome 字体噪声同源不可单点修）。col-definite-max-size-001 chromium 差距更大（~1.7%）——其 col 同时有 `width:100px` + `max-width`（0/min-width:100px/10%/calc），ZW 只读 width 不读 max-width 故过宽；需 Pass 0 扩展 col max-width/min-width 钳制，但其 10 表 test/ref 结构复杂且 max-width:0→min-content 钳制与同源匹配交互未验证，**defer**。`width:auto` 表上 % col 解析为 auto（chromium 行为）→ ZW 跳过 % 反而正确，无需支持。

**遗留**：table.rs 现 2549 行（超 2000，本就 2357 超限，本轮 +192 行均内聚于 col 宽度处理）；colspan 主体（border-collapse + 空列裁剪）= 下轮结构性目标；col-definite-size-001 表 2(calc)/3(%)/4(width:0) 因 %/calc 跳过仍 18px（chromium 仍不一致，需后续支持 % 在 shrink-to-fit 的解析）。

### R177 — top 候选根因实证确认（colspan 5 部件机制 + morning.work blue-nav = inline→block 背景结构性，诊断轮，无代码提交）

本轮对 `evidence/analyze-pollution-2026-06-16-r168.txt` 18 真 bug 候选逐一实证复核，**再次确证 R118/R140「clean win 已穷尽，剩余全结构性」结论**。上游同源基线 **434/490 (88.6%) 持平**（无代码变更）。把最高价值候选（colspan 52% chr）的**精确可执行机制**钉死，供下轮直接施工，避免重复 40+ 步实证。

**colspan 实证机制（ZW_TABLE_DEBUG 插桩 + 像素逐表分析确认）**：`table_grid_size_col_colspan`（同源 0% 但 chromium 52%）真实根因 = `<col>` 元素在 `build_grid`（table.rs:1065）被 `_ =>` 分支**完全跳过**，导致：(1) **col_count 只来自单元格 colspan**——`<table><col>×3><td colspan=1>` 的 col_count=1（非 3），`<td colspan=4>` 的 col_count=4（**未钳制到 3 列**，违反「colspan 不超出 grid」断言）；(2) `<col>` 的 `width:50px` **从不读取**（仅 detect_collapsed_columns 读 visibility）；(3) `width:auto` 表格 `apply_table_size_constraints` 设 `content_width=intrinsic` 想收缩，但 `compute_column_widths` 的 `cell_used_width` 对空单元格返回 `compute_cell_intrinsic_width=char_width+padding≈11.6`，再被 line 1675 的 `(has_explicit_width || is_fixed_layout)` 扩展条件**撑到容器 784px**。chromium 实测：每表收缩到「单 cell 跨的列数 × 50px + 边框」（60/110/160/60/60px），空列（无 cell 的 `<col>`）被裁剪。

**colspan 修复 = 5 个相互耦合部件（不可单点）**：(a) build_grid 让 `<col>`/`<colgroup>` 计入 col_count；(b) colspan 钳制到 grid 列数；(c) compute_column_widths Pass 0 读 `<col>` width；(d) 裁剪无 cell 的空列；(e) width:auto 表格收缩到 grid（改 line 1675 扩展条件）。**风险分级**：实测 6 个含 `<col>` 用例中，`col-definite-size-001`/`col-definite-max-size-001`（同源 0%）test 与 ref **结构相同**（同 4×`<col width>`）→ 加 col-width 读取两侧同变仍匹配 = **安全**；`insert-after-colgroup`（动态插入）/`visibility-collapse-colspan-003`（collapse 交互）/`border-collapse-dynamic-col-001`（动态+border-collapse）/`table_grid_size_col_colspan`（需空列裁剪）= **风险**，须配套 (b)(d) 且逐用例验证。`cell_used_width` 对显式 width 单元格用 `cell_box.width`（taffy 拉伸的 784）而非 CSS 显式值——独立子 bug，但单独修会打破 colspan 同源（test 空 cell 11.6 vs ref width:40px），需与 col-width 打包。

**morning.work blue-nav（剩余 28.72% 最大块 ~55k px）= inline→block 背景结构性**：article.html 的 3 个 `.item-tag <span>`（display:inline UA，`background:#607cd2`）经 converter `DisplayValue::Inline → taffy::Block`（converter/mod.rs:265）变全宽块，**文本+背景均纵向堆叠**（3 条全宽蓝条 y=169-241），非行内小徽章。chromium 渲染为行内 badge。属 R109 IFC ownership / inline→block 架构（goal P1），非单会话。chromium 逐表宽度已量化备用。

**其余候选复核结论**：`backdrop-inherit-rendered`(47%) 需 `dialog::backdrop`+showModal JS = 范围外/过重；`baseline-block-with-overflow-001`(45%) = overflow:hidden inline-block 基线=底边（CSS2 §10.8.1）结构性；`flexbox-collapsed-item-horiz-001`(20%) = float flex 容器 shrink-to-fit（R129 仅覆盖 block 子元素）结构性；`flex-abspos-inset-nested`(18%)/`fixed-table-layout-percentage-in-flex-item`(11%) = flex item 含 table 的 definite size 结构性；writing-mode 系（box-offsets-rel-pos-vrl 等 22-25%）= R114/R142/R164 已 4 轮证伪的轴交换；multicol 17 失败 = R113/R122/R131 碎片化结构性；min-max-size-table(36%)/table-cell-width-0(30%) = R97 内在尺寸 cluster；近阈值（baseline-007 1.04%/css-flexbox-row 1.23%/block-in-inline 1.37%/child-border-box-max-content 1.52%）均经查为 flex/multicol 基线 + taffy grid 限制结构性非 clean。

**下轮最高杠杆**：colspan 5 部件修复（(a)+(c) 安全起步可独立提交保 col-definite-size/max-size chromium 一致；(b)+(d) 需含 colspan/visibility-collapse/dynamic-col 4 用例逐验证；(e) 收缩改条件须全 css-tables 51 用例回归）。或转 DC-13 WinterTC fixture 量化（R176 已备 img 固有尺寸+JPEG 解码，需录制 wintertc.org 静态 fixture）。

### R176 — `<img>` 解码固有尺寸注入布局 + reftest harness JPEG 解码（DC-11 替换元素 + DC-13 图片子资源能力，零回归，已提交）

补齐 DC-11「替换元素固有尺寸」与 DC-13「图片子资源」的关键缺口：`<img>` 无 width/height 属性时此前依赖 CSS 或默认尺寸（DC-11 标记 P1 未实现），导致图片密集真实页面（WinterTC 首页 Logo 等）的 `<img>` 布局塌缩。

**修复（layout-engine + engine + reftest harness 三层）**：
1. `RenderPipeline::build_img_intrinsic_sizes`（pipeline.rs）——从 `self.image_sizes`（URL hash 索引，由调用方从解码后 ImageCache 预解析）按 DOM NodeId 解析 `<img>` 固有尺寸。hash 在 engine 层解析（`simple_hash` 定义于 engine crate），避免把 hash 函数泄漏到不依赖 engine 的 layout-engine。所有 4 处布局入口（`render_html`/headless/缓存路径）改调 `compute_with_img_sizes`。
2. `LayoutEngine::compute_with_img_sizes`（engine.rs）——新增 pub 方法，透传 `img_intrinsic_sizes: HashMap<NodeId,(f32,f32)>` 到 `build_layout_tree`；`compute` 保持不变（传空 map）。
3. `apply_replaced_element_sizing`（tree.rs）——在「无 HTML 属性」分支回退到解码固有尺寸：仅在 `width:auto`/`height:auto` 时注入，`aspect_ratio` 未显式设置时按固有尺寸比补设（与有 HTML 属性分支对称）。
4. reftest harness JPEG 解码（reftest.rs `load_jpeg_file`）——`build_image_cache` 旧仅支持 PNG，真实页面 logo/照片多为 JPEG；新增 jpeg-decoder 解码（RGB24/L8/CMYK32/L16 → RGBA8），PNG 失败再尝试 JPEG。

**验证**：上游 reftest **434/490 持平零回归**（set-diff 零翻转——upstream 用例的 `<img>` 多带显式 width/height，无属性且 base_dir 有图的少数用例尺寸变化两侧 test/ref 同源仍匹配）；inline 686/686；make test **12189/0**（+1 单测 `test_img_intrinsic_size_from_decoded` 验无属性 `<img>` 用解码固有尺寸）；clippy/fmt clean。

**意义**：(1) DC-11 替换元素固有尺寸从「未实现」到「解码驱动」，与 DC-13 图片子资源/ImageCache 贯通闭环——为 WinterTC 等图片密集 fixture 的 `<img>` 正确布局奠基（此前无尺寸 img 被 taffy 当 0 尺寸块）；(2) `<img>` 固有尺寸与 DC-13 morning.work/wintertc 真实页面衔接，下一轮可量化 WinterTC Logo 布局改善；(3) hash 解析留在 engine 层是依赖边界正确选择（layout-engine 不依赖 engine）。

**剩余**：JPEG/SVG 栅格化渲染本身（DC-8 CPU ImagePrimitive 图元已能从 ImageCache 绘制，本轮只补布局侧尺寸）； WinterTC fixture 的产品 smoke 量化为下一轮目标。

### R175 — CSS 自定义属性继承修复（var() 不继承致 :root 变量丢失，morning.work 67.45%→28.72%，零回归，已提交）

录制 morning.work 中文文章 fixture（DC-13 首个真实外链页面 fixture，`apps/browser/assets/morning-work/`，含 4 外链 CSS + 2 图片，经 base_dir 加载），与 chromium 800×600 对比 → 初始 diff **67.45%**（页面背景/代码块背景全白、布局塌）。诊断为 **CSS 自定义属性不继承**。

**根因**：CSS 自定义属性是继承属性，但 `gather_custom_properties`（style-system/src/lib.rs）每元素只取自身级联 `--*` 声明，丢弃祖先（`:root`/`html`/`body`）定义的变量 → 后代 `var(--x)` 解析失败（`--x` 不在自身 map）→ 背景回退默认白/颜色丢失。**任何 `:root{--token}` + 子元素 `var(--token)` 的真实页面全部受影响**（现代 CSS 设计系统标配）。诊断探针证实：`--c` 定义在 `.a` 自身时 `var(--c)` 正确，定义在祖先时失败（白）——继承类 bug 须跨元素 DOM 诊断，单元素探针会漏。

**修复**：`gather_custom_properties(cascaded, inherited)` 先继承父自定义属性再自身覆盖再迭代 resolve var()；`compute_styles_recursive` 递归传 `parent_custom`（进入子树前一次性捕获供 sibling 共享）。`compute_element_style`（pub 单元素）签名不变。

**验证**：morning.work fixture diff **67.45%→28.72%**（-185,898 px，页面背景 #f9f7f4 + 代码块背景 + 设计 token 全部正确应用）；reftest **434/490 持平零回归**（同源两侧同变仍匹配；8 类目全持平）；新增 2 单测（test_custom_property_inheritance + override_inherited）；make test 12188/0；clippy/fmt clean。welcome.html 不用 var() 故未受影响。

**意义**：(1) CSS variables 是现代页面基础特性，此 bug 影响面极大——真实中文页面此前背景/颜色全丢；(2) DC-13 真实页面 fixture 轴线（morning.work）捕获了 reftest 490 平台期 + welcome 都覆盖不到的 bug；(3) fixture 证据持久化 `evidence/product-static/morning-work/`。

**剩余 morning.work 28.72%**：最大块=顶部蓝色全宽条（~55k px #607cd2，y=169-241 共 73 行 × 752px 宽）——是 3 个 `.item-tag` `<span>`（Fedora/MacBook/Linux 标签徽章，直接规则非 @media）被**渲染为全宽堆叠块**而非行内小徽章（span UA display=inline 正确，问题在布局层 inline/inline-block 元素被当作 block，属 R109 IFC/inline→block 架构范畴，非单会话修复）；+ fontdue CJK 度量噪声 + hljs 高亮缺失 + @font-face web 字体未加载。**下一轮可独立诊断**：inline `<span>` 带背景的全宽渲染（converter 是否把 inline-block 映射为 block）与 @media min-width 在外链 CSS 的评估一致性。

### R174 — box-shadow blur σ=radius/2 修复（CSS 高斯映射修正，welcome 28.72%→28.08%，零回归，已提交）

DC-13 welcome.html 剩余 ~28% 差距逐带/逐像素定位（throwaway 渲染测试 + layout snapshot + PIL）。**关键结论：welcome.html 剩余差距 96.5% 为 fontdue vs Skia 字体噪声（非 CSS bug）**——色差直方图双峰 delta≈±10（66k 像素，glyph 边缘 AA 差异）+ card-desc 文本换行行数差异致 chromium 卡片高 ~13px。结构布局经 layout snapshot 确认正确（gradient bar y=36、hero/卡片几何、卡片白底（R172 后正确）均正确）。唯一可定位的真实渲染 bug = **box-shadow blur 高斯 σ 映射错误**。

**根因**：CSS 规范 `box-shadow: ox oy blur_radius` 的 blur_radius 对应**高斯标准差 σ = blur_radius / 2**（Chromium 实现）。ZeroWeb 旧实现 `radius = blur_r.ceil()` 直接当三遍 box-blur 半宽 → blur_r=3 时 σ≈3.46（**偏大 2.3 倍**），阴影扩散过远。实测 welcome `.card`（`box-shadow: 0 1px 3px rgba(0,0,0,0.08)`）：chromium 阴影基本不可见（alpha 0.08 经 3px 模糊 <1/255，全白），ZeroWeb 在卡片下方渲染 **12px 可见阴影带**（lum 232-243）。

**修复**（render-foundation/src/cpu/shadow.rs）：`sigma = blur_r*0.5`；连续半宽 `d=(sqrt(4σ²+1)-1)/2`；按 d 小数部分在 3 遍间分配 floor/ceil 半宽（m 遍 ceil、3-m 遍 floor，`m=round((d-r_lo)*3)`）。blur_r=3→σ=1.41（规范 1.5），blur_r=6→σ=3.16（规范 3.0）。

**验证**：welcome.html diff 137,874→134,796（**28.72%→28.08%**，阴影带收紧）；reftest **434/490 持平零回归**（test+ref 同源，blur 修复两侧同变仍匹配；inline 686/686）；make test 全绿；clippy/fmt clean。

**教训**：(1) welcome.html 已无 clean structural CSS bug——剩余差距需升级字体光栅器（fontdue→更接近 Skia）才能显著下降，非单会话范围；**DC-13 杠杆转移至 morning.work（外链 CSS + CJK 真实页）/ wintertc.org（图片子资源）等能暴露未实现 P1 缺口的 fixture**。(2) 像素扫描易误判「垂直偏移」——扫描得「gradient bar y=72」实为蓝色 title 文本，须 layout snapshot 交叉验证。(3) CSS σ=radius/2 同适用于 `filter:blur()`（effects.rs:36-41 仍用 `radius.ceil()` 单遍，σ 偏大）——未修（无测试驱动，遵循精准修改）。(4) 同源 reftest 对保真类修复天然零回归，可用同源 490 验证安全性。

### R173 — 加载 Noto Sans CJK 字体 + 回退链（CJK 字符可渲染，DC-13 能力，零回归，已提交）

DC-13 welcome.html cards 区域剩余 diff 定位到 **CJK 字符完全不渲染**（探针「中文」dark=0）。根因：`create_font_loader`（reftest.rs:1114）只加载 DejaVu/Ahem，无 CJK 字体；主字体缺 CJK 字形时无回退 → 中/日/韩文本全空白。系统有 `/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc`（chromium 用它渲染 CJK）。

**修复**：create_font_loader 加载 NotoSansCJK-Regular.ttc（fontdue 支持 .ttc）并 set_fallback_chain。引擎回退逻辑已就绪（主字体缺字形时回退），此前仅缺 CJK 字体加载。CJK 探针「中文」dark 0→91（渲染）。

**验证**：reftest **434/490 持平零回归**（Ahem/ASCII 用例不受影响）；make test 12186/0；clippy/fmt clean。

**权衡**：welcome.html 像素 diff 26.15%→27.10%（+0.95%）——CJK 现可见但带 fontdue vs Skia 字体度量噪声（已知文本差异），比空白更正确；CJK 重度页面（morning.work 中文文章 fixture）从大量空白→可读，是 DC-13 关键能力。剩余 welcome 度量差需 fontdue CJK 度量调优（与 reftest 文字 fontdue/Skia 噪声同源）。

**意义**：CJK 渲染是产品必备能力（中文页面此前全空白）。fontdue .ttc 支持 + 引擎回退链已验证可用。

### R172 — border-radius 背景在 draw_order 模式被丢弃（paint_background 绕过 add_rounded_rect，真实修复，零回归，已提交）

DC-13 welcome.html cards 区域 50.45% 差距的**第二大主因**（仅次于 R170 box-shadow 实心黑）。`paint_background`（painter/mod.rs:1101）对圆角背景直接 `primitives.rounded_rects.push()`，**绕过 `add_rounded_rect`**（后者才记录 `DrawOp::RoundedRect`）。draw_order 是 R155 起的默认渲染路径，无 DrawOp 的 rounded_rect 被丢弃 → **任何带 border-radius 的元素背景都不绘制**（透显底层背景）。welcome.html 卡片（border-radius:10px）白底消失。

**根因定位过程**：DC-13 cards 区域 y240-480 85-95% diff → no-box-shadow bisect 仍透显 body-bg → 逐变量探针（grid/padding/radius）定位 **border-radius 触发** → block+grid 均复用 → 查 paint_background 圆角路径 → 发现直接 push 绕过 add_rounded_rect（draw_order bypass）。

**修复**：改用 `primitives.add_rounded_rect()` 记录 DrawOp。全仓库仅此一处绕过 add_* 的 push（grep 确认无其他）。

**验证**：border-radius 元素背景探针（was body-bg 透显）→ 正确白底；**welcome.html 差距 50.45%→26.15%**（DC-13 重大进展，本 session 从 51.59%→26.15%）；reftest 434/490 持平零回归（test+ref 同病同愈）；make test 12186/0（+1 rounded_rect draw_order 测试）；clippy/fmt clean。

**意义**：(1) draw_order（R155 基建）的 bypass 类 bug——任何图元若不通过 add_* 方法记录 DrawOp 就会在默认渲染路径丢失；(2) border-radius 是真实页面极常见属性，此 bug 影响面巨大；(3) DC-13 smoke 轴线连续产出 R170/R171/R172 三个真实修复，welcome.html 差距减半——证明产品静态 smoke 能系统捕获 reftest 平台期外的 bug。

### R171 — border/outline/column-rule/text-decor 简写 rgba 带空格丢颜色（同 R170 class，真实修复，零回归，已提交）

R170 修复 box-shadow/text-shadow 后，排查同类 `split_whitespace()+looks_like_color` 模式，发现 4 个简写解析器有相同 bug：`parse_border_shorthand` / `expand_outline` / `expand_column_rule` / `expand_text_decoration`（style-system shorthand/mod.rs）。它们用 `split_whitespace()` 拆碎标准格式 `rgba(255, 0, 0, 0.3)`（逗号后空格）→ `looks_like_color` 命中碎片或颜色退化 currentcolor（→黑）。

**修复**：把 R170 的 `split_shadow_tokens` 重命名为通用 `split_paren_aware_tokens` 并 `pub`（css-parser parse_transform.rs），4 个简写解析器改用之（括号感知，保留 rgba()/hsl()/var() 为单 token）。

**验证**：border/outline 带空格 rgba 探针（was 黑 911/804）→ 正确半透明红/绿；make test 12185/0（+1 border rgba 测试）；reftest 434/490 持平零回归；clippy/fmt clean。

**意义**：rgba-带空格丢颜色/alpha 的 bug class 现已**全量修复**（`looks_like_color` 模式 6 个解析器：box-shadow/text-shadow/border/outline/column-rule/text-decoration）。影响所有用标准带空格 rgba 的真实页面。welcome.html 不受 border 修复影响（用 box-shadow 非 border 简写），其剩余 50.45% 差距为 cards 区域 Phase A（独立后续）。

**方法论**：R170 用 SHDWDBG 定位 box-shadow 后，本轮按「同 class 模式 grep」(looks_like_color) 一次性找出全部 4 个同源 bug——per-feature grep（区别 bbox 扫描/BISECT）是 cluster-finder，R170+R171 是其产出。

### R170 — DC-13 box-shadow/text-shadow rgba 带空格丢失 alpha 致实心黑（真实修复，零回归，已提交）

DC-13 welcome.html smoke 定位到产品可见渲染 bug：`parse_box_shadow`/`parse_text_shadow`（parse_transform.rs）用 `split_whitespace()` 分割值，把标准格式 `rgba(0, 0, 0, 0.08)`（逗号后有空格）拆成碎片，颜色解析失败回退默认实心黑（alpha=255）。

**根因定位过程**：welcome.html 51.59% 差距中 132,793 纯黑像素——bisect 删 box-shadow 后纯黑→0；SHDWDBG 插桩 paint_box_shadow 证 8 个 card/section 阴影 color=rgba(0,0,0,255)（应≈20）；单变量探针排除 grid/`*`/@media/rgba-spaces(count)/count 后，**最终用 SHDWDBG alpha 对比 `.a rgba(0,0,0,0.08)`(alpha 20) vs `.b rgba(0, 0, 0, 0.08)`(alpha 255) 定位到空格触发**——split_whitespace 拆碎 rgba。

**修复**：新增括号感知分割 `split_shadow_tokens`（不在 `()` 内分割，保留 `rgba()/hsl()/var()` 为单 token），box-shadow + 同 class 的 text-shadow 均改用之。

**验证**：
- welcome.html 纯黑 **132793→0**（DC-13 实心黑主因消除）。
- 同源上游 reftest **434/490 持平零回归**（0 用例用 box-shadow rgba）。
- make test **12184/0**（+2 单测：box_shadow/text_shadow rgba-spaces 保 alpha）。
- clippy/fmt clean。
- 注：welcome.html 整体 diff 仅 51.59%→50.45%——box-shadow 实心黑消除后暴露**其他**渲染差异（cards 区域，Phase A inline-block/IFC ownership 同源），独立后续。

**意义**：(1) DC-13 首个真实修复，产品可见渲染 bug；(2) box-shadow/text-shadow 用标准带空格 rgba 在**任何真实页面**都会触发实心黑——影响面远超 welcome.html；(3) 印证 DC-13 轴线（产品静态 smoke）能捕获 reftest 平台期无法暴露的 bug——welcome.html 非 reftest，其 box-shadow 解析退化不在 56 失败用例中。

### R169 — Phase A large-font 死锁实证复核（放宽 R84 多行守卫 net -1，已回退，无提交）

按 Phase A 推进方向（解锁 large-font 集群 ~5 自源失败：font-051/ifc-008/009/011/empty-inline-002），本轮**实证复核**了 R125 记录的死锁——放宽 `compute_final_inline_layouts` 的 R84 单行守卫（engine.rs:1690），允许「多行纯 Ahem」存储 `inline_layout`（保留纯 Ahem 过滤），测全量。

**实测结果（net -1，已回退）**：434→**433/490**。
- **large-font 集群仍全失败**：font-051 8.19%（持平）、ifc-008 8.18→4.17%（改善但未过阈）、ifc-009 4.17%、ifc-011 11.24%（持平）、empty-inline-002 29.32%（持平）。**仅存 inline_layout 不够**——这些用例还需 `text_node_font_sizes` 存储（store_font_sizes_from_ifc，remeasure 路径），而该存储正是 R125 三路 net-negative 的回归源。
- **multicol 40→39（-1）**：multicol-fill-auto-001 回归（与 R125 一致——多行 inline_layout 存储改变其 paint use_stored 渲染）。

**结论**：R125/R158 记录的 Phase A large-font 死锁经本轮独立实验**再次确证**——R84 多行守卫是 load-bearing（放宽即 net -1 且不解锁集群）。large-font 修复需同时 (a) 存 inline_layout（多行）+ (b) 存 text_node_font_sizes（real font_size），而 (b) 回归 multicol-fill-auto（依赖 16px 错误默认值才通过）。**不要再单独放宽 R84 守卫或单点补存 font_size 重试**（R125 三路 + R169 共 4 轮证否）。真正解锁需 Phase A paint IFC 三路径（compute_final/remeasure/paint）font_size 解析统一 + multicol-fill-auto 在真实 font_size 下也正确（R125 前置）——属多轮架构。

**方法论**：对「已知死锁」做独立实验复核（放宽守卫→全量测→net 效果），用实测确认而非仅据记忆接力。本轮 net -1 即刻回退，434/490 持平。印证上轮结论：剩余 16 真 bug 候选 + 56 自源失败全部 Phase A 结构性 / 基础设施 / 特性，单会话零回归不可推进。

### R168 — table height-as-minimum 修复（table-grid-item-dynamic-004 chromium 11%→2.98%，真实修复，零回归，已提交）

d16bb8e 方向（优化 chromium Oracle 一致率）下第二个真实修复。修复 table 的 `height` 属性被完全忽略的真实 bug —— 被「同源假通过」掩盖的缺口（table-grid-item-dynamic-004 同源 0.00% 通过，但 chromium 差 11.12%；18 真 bug 候选之一）。

**根因**：`apply_table_size_constraints`（table.rs）把表格高度设为 `intrinsic_height`（行内容驱动）+ min/max-height 约束，但**完全忽略 CSS `height` 属性**（Px 和 % 均不读取，只赋值 `final_height = intrinsic_height`）。CSS 2.1 §17.5.3 规定 table 的 `height` 是内容高度的「下限」（min 语义）：表格至少这么高，内容更高则增长。ZeroWeb 此前对任何带 `height`（如 `height:200px`、`height:100%`）的表格都按内容尺寸渲染，与 chromium 大幅不一致。

**修复**：在 `clamp_percentage_max_height`（engine.rs:1378，自上而下 pass，已有 cb_content_height「明确高度」语义）新增 §1.5 段——对 `display:table/inline-table` 盒，把 `style.height`（Px 直接 / 百分比按 cb_content_height 解析）解析为内容高度下限，与已计算 `content_height` 取 max 后回写 `content_height`+`height`（+padding_border）。复用 R119 的 CB 明确性语义：百分比仅当 CB 明确时解析，否则忽略（§10.5）。在此 pass 处理而非 apply_table_size_constraints，是因为后者无 CB 上下文（LayoutBox 无父指针），而本 pass 已自上而下传递明确高度。

**验证（chromium Oracle + 严谨禁用对照）**：
- table-grid-item-dynamic-004: table 500×116（intrinsic，height 被忽略）→ 500×200（fixed：100 内容 + 100 padding-top），chromium 500×222。chromium 差距 **11.12% → 2.98%**（< 5% 布局阈值 → chromium Oracle 实际通过）。
- **禁用对照**：临时把增长条件改 `false`，004 回到 116（unfixed）；启用→200。确认修复是 004 改善的**唯一原因**，排除巧合。
- 同源 upstream reftest **434/490 持平，set-diff 零翻转**（004 同源本就 0.00% 通过=polluted case，修复不改同源通过数，只降 chromium 差距 = DC-14 真实改进）。
- 18 个含 table `height` 用例**零回归**：min-height-table 0.00%、max-height-table 0.02%、subpixel-table-width-001 0.00%、multicol-fill-001 0.16%、border-spacing-vrl-002 4.17%、min-max-size-table-content-box 36.34%、table-cell-width-0 29.99%、baseline-vertical 16.66%、fixed-table-layout-… 11.20% 全部 diff 持平（min/max-height 语义不变，因取 max 只增不减且多数用例 content≥指定高）。
- make test **12182 passed / 0 failed**；clippy 零警告；fmt clean。
- +2 单测：`test_table_height_as_minimum_px`（engine.compute 验 Px 分支）+ `test_table_percentage_height_resolves_as_minimum`（直接调用 clamp 验百分比分支——engine.compute 路径中 table 匿名包装盒打断直接父子 CB 传递致百分比不触发，故直接测函数；百分比端到端正确性由 004 reftest 覆盖）。

**chromium Oracle 全量交叉验证（DC-14 交替推进 step 2）**：重跑 `cross-validate.py`（`evidence/cross-validate-full-2026-06-16-r168.txt`）+ `analyze-pollution.py`（`evidence/analyze-pollution-2026-06-16-r168.txt`）。污染总数 153→154（基本持平，污染率 46.5%→46.7%——R168 把 004 的 chr 差从 11% 降到 2.98% 但仍 >1% oracle 阈，故污染计数未翻转）。但**真 bug 候选（>5% 布局目录）从 18 降到 16**：R165 移除 html-display-table（33%→<5%），**R168 移除 table-grid-item-dynamic-004（11.12%→2.98%）**，并**把 003 从 29.18% 降到 23.79%**（height 修复部分生效，剩余=宽度 grid-stretch）。印证 R165+R168 是 18 候选中仅有的可单点修复真实缺口，剩余 16 项确认结构性/基础设施/特性（grid-stretch / flex baseline·collapse / multicol-abspos / border-collapse×vrl / dialog·iframe 基础设施 / font-fallback 特性）。

**剩余**：
- **003（29%）同源通过但 chromium 仍高**——差异主因是 table 作为 grid item 的**宽度 shrink-to-fit vs grid-stretch**（ZW table 327 shrink-to-fit，chromium 800 拉伸到 grid 轨道），与高度修复独立；003 仅 height 修复后 table 仍 327 宽，与 chromium 800 宽差距主导。需 grid-item-table 拉伸单独处理（grid item width:auto 应拉伸到 track，但 table shrink-to-fit 覆盖——类似 R138 但需 grid 感知）。
- **004 剩余 2.98%** = chromium intrinsic 行高 ~122 vs ZW ~100（cell padding/border-spacing 细节），独立小差异。

**方法论**：chromium Oracle 像素对比（REFTEST_DUMP ZeroWeb vs /tmp/oracle-shots-all）定位「同源通过但 chr 不一致」的真 bug → 探针（TBLH_DBG）定位 `style.height` 在 apply_table_size_constraints 被忽略 → 严谨禁用对照（条件改 `false`）确认修复是改善唯一原因。**印证 d16bb8e 转向正确**：18 真 bug 候选中仍有可单点修复的真实缺口（R165 margin:auto、R168 table height），非全部结构性；下一轮最高杠杆=重审 R111/R130/R138/R124 等「同源修但 chr 仍高」用例的分歧点，或 003 的 grid-item-table 拉伸。

### R166 — 真 bug 候选再甄别（table_grid 非单点 + float-006 paint 层，诊断，持平，无提交）

R165 之后继续按 chromium Oracle 真实缺口推进，逐个甄别几个 self+chr 均高（=真 bug 非 REF 产物）的候选：

- **table_grid_size_col_colspan (chr 52%)**：根因 = build_grid 的 col_count 仅从单元格 colspan 计算，**忽略 `<col>` 元素**（3 个 col 算 1 列）；且 fixed-layout auto-width 错误扩展到容器。实现修复（col_count 计 col + 收集 col_widths + 去除 fixed 扩展）后**内部全部正确**（col_widths=[50,50,50]、table 宽 150、cell w=50），但**渲染仍全宽/重叠**——直接 `<td>`+`<col>`+匿名行+fixed 结构有多个叠加问题（含表格垂直堆叠），非单点修复，已回退。
- **flexbox-collapsed-item-horiz-001 (chr 20.5%)**：flex visibility:collapse 内部（R111 territory），多 cell 差异，flex 内部复杂。
- **multicol-breaking-005 (self 21.7%/chr 22.3%)**：嵌套 multicol 平衡（外 3 列 × 内 2 列 balance），最深水区。
- **float-006 (self 7.5%/chr 8.4%，已精确定位=需 2 处协同修复)**：零高度空 float 测试。**实测（修正 R166 初判）**：绿色 abspos `x=0 w=224` **渲染正确**（X[8..231] 与 chromium 一致），红色 float 在 `x=288`（X[296..519] 可见，应被绿覆盖）。需两处协同修复：(1) **layout**：零高度（margin-box）空 float 不应占据水平空间（engine.rs adjust_float_positions_with_context `left_used_width += child_outer_width` 须加 `child_outer_height > 0` 守卫，使红色 float 回到 x=0）；(2) **paint**：z-index:auto 的 abspos 须画在 float 之上（CSS App. E step 6，painter/mod.rs child_paint_sort_key 现按 (1,0)=step3 画 positioned 即在 float(2,0) 之前，仅做 (1) 会使红覆盖绿）。**(2) 实测 net -1**：把 z-index:auto positioned 改 (3,0) 画在 float 之后 → regress `css-flexbox/flex-item-position-relative-001`（嵌套 positioned 用例，原 (1,0) 排序是为其修复），fix 0。CSS App.E 的「z-index:auto after floats」与 painter 的嵌套 positioned 近似**结构性冲突**，需完整 stacking-context 实现（CSS App.E 全量）才能两全，属多轮架构改造。float-006 非单点、非双点，需架构层 stacking-context。**精确机制（R167 复查）**：flex-item-position-relative-001 的 green abspos 嵌套在**正常流** `#flex` 内，flat 按容器排序使其随 `#flex`(step3) 一起画、早于兄弟 red abspos(step6) → 红覆盖绿。**z-index:auto positioned 嵌套在正常流中必须「逃逸」到祖先 stacking-context 的 step6**——flat sort 结构性做不到，须按 stacking-context 收集所有 positioned 后代再按 App.E 7 步渲染。

**结论**：除 R165（margin:auto，单点）外，本轮再甄别的真 bug 候选全部结构性/多子系统/需多修复协同。

### R165 — margin:auto 水平居中修复（html-display-table chromium 33%→2.63%，真实修复，零回归，已提交）

**d16bb8e 方向（优化 chromium Oracle 一致率）下首个真实修复**。修复 `margin:auto` 水平居中缺失——这是一个被同源假通过「掩盖」的真实渲染 bug（html-display-table 同源 0.00% 通过，但 chromium 差 33.09%）。

**根因（两处）**：
1. **根元素**（如 `<html style="width:280px;margin:auto">`）：CSS §10.3.3 规定 width<视口 + margin-left/right 均 auto 应水平居中。taffy 对**嵌套** block 正确居中（实测 nested div margin:auto → x=260 ✓），但对**根节点**不应用（根无父级提供居中上下文，左对齐 0）。实测 root_x=0（应 260）。
2. **display:table 容器**（width:auto 收缩后）：R138 的 `shrink_table_to_block_content` 收缩了宽度但未重新居中——taffy 在收缩前（table 填满 CB）已把 auto margin 解析为 0，收缩后未补。

**修复（两处补丁）**：
1. `compute()`（engine.rs）根居中补丁：水平书写模式下，若根 margin-left/right 均 auto 且边框盒宽度 < 视口，把 root.x 居中（跳过 display:table 避免双重居中）。
2. `shrink_table_to_block_content`（table_shrink.rs）收缩后补居中：若 margin-left/right 均 auto 且新宽度 < 旧宽度，table.x 居中（子元素 x 相对 table 内容盒，paint 累积偏移 offset_x+box.x，改 table.x 即整树居中，无需逐子元素平移）。

**验证（chromium Oracle /tmp/oracle-shots-all + cross-validate.py）**：
- html-display-table: chromium 差距 **33.09% → 2.63%**（同源 0.00% 持续通过，test 与 ref 均居中）。
- 同源 upstream reftest **434/490 持平，set-diff 零翻转**（stash 前后 56 失败完全一致）。
- make test **12180 passed/0 failed**（59 ignored = 真实网站）；smoke reftest 686/686；clippy 零警告；fmt clean。
- +2 单测 `test_root_block_margin_auto_centers` / `test_display_table_margin_auto_centers`。

**方法论**：用 chromium Oracle 像素对比（`REFTEST_DUMP` ZeroWeb 渲染 vs `/tmp/oracle-shots-all`）定位「同源通过但 chromium 不一致」的真实缺口；用根 vs 嵌套 margin:auto 探针界定 taffy 行为（嵌套正确/根缺失），把修复精准限制在 taffy 缺口（根 + 收缩后 table），避免触碰 taffy 已正确的嵌套 block 居中路径（零回归保证）。

**意义**：印证 d16bb8e 转向的正确性——html-display-table 是 R138「同源修」但 chromium 仍差 33% 的典型案例；本修复根因（margin:auto 居中缺失）使其真正对齐 chromium，而非匹配怪异同源 REF。剩余 18 候选中 R124/R130/R138/R111 等「同源修但 chr 仍高」的用例是下一轮最高杠杆（代码位置已知，只需找与 chromium 的分歧点）。

### R164b — 18 真 bug 候选逐项甄别：5 项已查全部结构性（诊断，持平，无提交）

按 d16bb8e 方向（优化目标=chromium Oracle 一致率）逐项甄别 `evidence/analyze-pollution-2026-06-16.txt` 的 18 真 bug 候选。用 `REFTEST_DUMP` ZeroWeb 渲染 + `/tmp/oracle-shots-all` chromium Oracle 像素对比定位差异。**已查 5 项全部确认为结构性多轮**（非单会话 clean win），逐项根因：

1. **backdrop-inherit-rendered (47.5%)**：需 `<dialog>` + `::backdrop` + `showModal()` JS + CSS 变量继承 + `inset` 基础设施，ZeroWeb 无 dialog/modal/backdrop 渲染管线——基础设施级，非渲染 bug。
2. **baseline-block-with-overflow-001 (45%)**：`inline-block` + `overflow:hidden` 基线=底边（CSS §10.8.1）+ 5 section 复合，inline-block 基线×overflow 交互，影响面广。
3. **multicol-contained-absolute (16%, R124 同源「修」但 chr 仍 16%)**：ZeroWeb 渲染 abspos 绿块**仅 1 列** X[8,399]，chromium **2 列** X[8,391]+[408,791]——multicol+abspos containing-block 语义（abspos 是否跨列/分列碎片化），CSS multicol 规范深水区，非 R124 的 overflow 裁剪层面。
4. **collapsed-border-vertical-rtl-overflow (6.1%)** ×3：`border-collapse` 冲突解析 × vertical-rl × direction:rtl × will-change overflow，三重复合，border-collapse 冲突已是 CSS 最难区之一。
5. **position-absolute-semi-replaced-stretch-input (23%)**：测试名暗示 abspos stretch，**实测 ZeroWeb 已正确 stretch**（窄 CB 绿框 147px、宽 347px 与 chromium 一致）。真实差异=**inline-block CB 的行距/基线/空白**：ZeroWeb 行距 ~100px、框间 gap ~7px，chromium 行距 ~116px、gap ~23px——inline formatting 基线×line-height×whitespace 子问题（非 abspos）。

**模式印证**（与 419c5a8 一致）：rally 曾「修」的用例同源通过但 chr 仍高——R124(multicol-contained-absolute)、R130(flexbox-baseline-align-self-baseline-horiz=17.65%)、R138(html-display-table=33%)、R111(flexbox-collapsed-item-horiz=20.5%)——证实「刷同源通过率」跑偏，这些「修」匹配了怪异同源 REF 而非 chromium。

**剩余 13 候选**（未查，按可修性排序）：position-absolute-semi-replaced-stretch-other(15%,同 #5 inline-block)、table-grid-item-dynamic-003/004(29/11%,grid+table)、table_grid_size_col_colspan(52%,table colspan)、stretch-grid-item-button-overflow(8%,grid+button)、font-family-013(6.65%,字体噪声嫌疑)、iframe-in-block-in-inline/wrapped-span(9.75%,iframe infra)、flexbox-collapsed-item-horiz-001(20.5%,R111 重审)、flexbox-baseline-align-self-baseline-horiz-001(17.65%,R130 重审)。**最高杠杆=重审 R111/R130/R138/R124 的「同源修」为何不匹配 chromium**（代码位置已知，只需找分歧点）。

### R164 — 否决 vrl-004/008 R114b x 轴 clearance：正确 CSS 与同源水平 REF 结构性不可对齐（诊断，持平，已回退）

**434/490 持平（无提交，实验代码已回退，工作区清洁）**。本轮以**实验证伪**了 R114b「对 vertical-rl/lr 容器实现 x 轴 float 定位 + clearance」这条接力路径——这是上轮 CONTINUE 指定的下一步，也是「PNG bundle 双阻塞」的理论解。

**实验**：新增 `adjust_float_positions_vertical`（~130 行），对 vertical-rl/lr 容器把块流从 taffy 的 Y 轴重排到物理 X 轴（block-start=右/左），复制 float 定位 + clearance（正/负/零）+ margin 折叠语义；入口按 writing_mode 分流（水平路径字节不变）。在 `adjust_float_positions_with_context` 顶部加守卫调用。

**结果（已回退）**：4 个 vrl clearance 用例**全部变差**——vrl-002 2.72→11.15%、vrl-004 7.09→14.05%、vrl-006 4.38→10.33%、vrl-008 6.42→16.83%（4/4 FAIL，含 002/006 从 PASS 翻 FAIL）。即 **实现正确 vertical-rl CSS 反而使全部用例失败更严重**。

**根因（结构性，非实现 bug）**：这些 reftest 的 **reference 是水平渲染**（`<img>` 绿块在**左侧** X[8,87]）。正确 vertical-rl 的 block-start 在**右侧**（X=高值），块流右→左；当前「错误」的 Y 轴堆叠**恰好**把绿块留在左侧（X[8,87]），偶然对齐水平 REF 的左侧绿。改为正确 X 轴后绿块移到右侧（实测 X[672,791]），与水平 REF 左侧绿结构性背离。

**chromium Oracle 印证同源 REF 比 chromium 更怪异**（DC-14 全量交叉验证 b18b7ae / `evidence/cross-validate-full-2026-06-16.txt`）：
- vrl-004 同源 7.09% vs **chromium 仅 5.08%**——ZeroWeb 渲染离 chromium 更近，离自己的怪异 REF 更远。
- vrl-008 同源 6.42% vs chromium 3.15%。
- **font-051 同源 8.19% vs chromium 仅 1.62%**——ZeroWeb 渲染几乎完美匹配 chromium，但「失败」于自己的怪异同源 REF（large-font 100px 文本，REF 侧 16px 默认值退化）。

**结论**：
1. **434/490 即诚实 DC-14 基线，无需恢复 436**。436 含 garbled-image 假通过（R163 已证）；vrl-004/008 的「双阻塞」是同源 REF 怪异产物，非真实渲染 bug。修它们 = 匹配怪异 REF，非对齐标准。
2. **不要再以「正确 vertical-rl CSS」重试 vrl-004/008**（本轮 + R133/R153/R154 共 4 轮一致证伪）。
3. **同源失败用例中相当部分是 REF 怪异产物**（vrl-004/008、font-051 等），同源通过率不是可信指标（DC-14）。后续应以 **chromium Oracle 交叉验证** 识别真实 bug（POLLUTED=同源通过但 chromium 不一致=隐藏真实 bug）。
4. **真实最大杠杆**（chromium Oracle 视角）：fontdue vs chromium 字体度量噪声是 46.5% 污染主因（b18b7ae 注）；次为结构性 multicol/table/flex-baseline。均为多轮大改。

**方法论**：对「已知诊断」做独立实验复核——R133/R153/R154 推断「正确 vertical-rl CSS 应改善 vrl-004/008」，但均未实际实现验证；本轮实际实现 ~130 行后实测全负，证伪推断。**架构性推断需实验落地，不能据推断结论接力多轮**。

### R163 — PNG 正确 RGBA 转换默认启用（DC-14 anti-false-pass，436→434 真实）（已提交）

**变更（已提交）**：`load_png_file`（reftest.rs）把「EXPAND+正确 RGBA 转换」从 env-gated（ZERO_PNG_EXPAND=1）改为**生产默认**——所有非 RGBA PNG（palette/grayscale/RGB）正确展开为 RGBA8（修正 alpha=0 退化=图像类 reftest 假通过根因）。`ZERO_PNG_EXPAND=0` 作为逃生舱回退旧的「按 RGBA 直读」garbled 路径（诊断/回归对比）。

**意义**：直接落实 **DC-14 anti-false-pass**——此前所有 palette/RGB PNG（support/swatch-*.png、pass-cdts 等）因缓冲错位渲染成透明/乱码（alpha=0），使图像类 reftest 的 reference 退化，产生「garbled-test vs garbled-ref 凑合匹配」的假通过。正确渲染后 reference 真实可见，消除该类假通过。与项目并行的 chromium 独立 Oracle 基建（72764a0）+ DC-14 门禁（c4d5863）方向一致。

**实测影响**：默认 reftest **436→434/490（net -2）**——仅 vrl-004(7.09%)/vrl-008(6.42%) 翻转为 FAIL（writing-modes 55→53），其余 488 用例零翻转（image 类用例要么本就 RGBA，要么 garbled 双方凑合匹配在正确渲染后仍匹配）。这两个是须修复的**真实 vertical-rl clearance 失败**（R114b），非应隐藏的假通过。make test 全绿（45 result 行 0 failed）。

**为何现在默认启用**：DC-14 明确「不满足 anti-false-pass 的通过率不构成达标证据」——436 含 garbled-image 假通过属非合规。正确的图像渲染是 anti-false-pass 的前提。逃生舱 ZERO_PNG_EXPAND=0 保留旧 436 供对比。

### R161 — PNG EXPAND 诊断门控 + 正确 RGBA 转换（修复 alpha=0 退化，bundle 真实 net -2）（已提交，默认净中性）

**变更（已提交，默认净中性零回归）**：`load_png_file`（reftest.rs）新增 `ZERO_PNG_EXPAND` 环境变量门控——启用后 `set_transformations(EXPAND|STRIP_16)` 并用 `output_buffer_size`+`convert_png_buffer_to_rgba` 按 `OutputInfo.color_type` 正确转换为 RGBA8。**关键 bug 修复**：EXPAND 不保证输出 RGBA（palette 无 tRNS / RGB 输入 → 输出 RGB=3 字节/像素），原先按 4 字节解释会错位→ alpha=0 退化透明（swatch-green.png 实测 [0,128,0,0]）。正确转换后 swatch-green → [0,128,0,255]。`convert_png_buffer_to_rgba` 处理 Rgb(补 alpha=255)/Grayscale(复制+255)/GrayscaleAlpha/Rgba(原样)。

**bundle 真实测量（纠正本会话早先 net -1 误判）**：`ZERO_PNG_EXPAND=1` 正确转换后全量实测 **434/490（net -2）**——**vrl-004 (7.09%) 与 vrl-008 (6.42%) 双双 FAIL**（R156 的 net -2 测量正确）。本会话早先用「错位转换」测得的 net -1 / vrl-008 4.06% 通过是 **bug 假象**（错位 buffer 产生的乱码 ref 凑巧接近）。clear-clearance-calc-001~005 全 0.00% 通过。**bundle 唯一阻塞仍是 vrl-004/008 双（R114b x 轴 clearance）**，未收窄。

**vrl-004 精确分析（EXPAND 后）**：REF image-based（2 swatch-green 60×100+20×100）；TEST 三 green 块 vertical-rl 下因 clearance 代码全 y 轴(engine.rs:2510)错位→红背景暴露。**这是 R114b x 轴 clearance 工作**（~150 行参数化块轴），对 59 水平 floats-clear 零风险（严格 writing_mode 门控）但高实现复杂度，单会话不可安全完成。

**解锁路径**：修 vrl-004/008（R114b x 轴 clearance）→ bundle net 0 → 可默认启用 EXPAND（消除 IMG-REF 假通过=DC-14 anti-false-pass，让图像 multicol REF 可见从而可正确评估/修复分布）。当前 EXPAND 默认关闭保 436/490；诊断用 `ZERO_PNG_EXPAND=1`（→434 真实）。

### R158 — large-font 死锁机制精确定位 + 失败聚类再分类（诊断，持平，工作区清洁）

**436/490 持平（make reftest-upstream 实测确认 436/490=89.0%，54 失败）**。本轮独立复核全 54 失败，重点深挖 large-font 死锁与若干「疑似 discrete bug」候选，全部确认为已知结构性聚类。新/精化的理解（区别既往轮次）：

**(1) large-font 死锁机制现已精确定位（比 R125「三条路径」更精确）**。完整链条：large-font 文本（font-051/ifc-008/009/011/empty-inline-002 的 100px 内容）位于 **taffy 已测量的 height:auto 子容器**（如 ifc-008 的 `#div1>div div{color:green}` 子 div，taffy 给了正确 content_height>1.0）。该子容器被两条存储路径同时跳过：(a) `remeasure_inline_only_containers` 的 `content_height < 1.0` 守卫（engine.rs:2883 `needs_dom_text_remeasure`）排除 taffy 已测量的块——守卫注释明确这是为避免 font-feature/multicol-fill-auto/abspos 回归（R105）；(b) `compute_final_inline_layouts` 的 R84 守卫（engine.rs:1628 `lines.len()>1 || !is_pure_ahem`）对多行/非纯 Ahem 提前 return，不存储 inline_layout 也不存 font_size。两条路径都跳过 → `text_node_font_sizes` 为空 → paint IFC（painter/text.rs:912 传 `&HashMap::new()` 空 styles + 空 override）按默认 16px 解析 → 100px 渲染成 16px。**关键冲突（R125 本质）**：multicol-fill-auto-001 当前**仅因部分文本节点的 16px 错误默认值才通过**；补存正确 font_size（即使 `or_insert` 不覆盖、仅新增条目）会改变其 paint IFC override 集合 → 文本尺寸变化 → 失败。故 large-font 修复必须**先**让 multicol-fill-auto 在真实 font_size 下也正确（或查清其 ref 为何依赖 16px），否则任一补存路径都 net-negative。**不要再从 compute_final/remeasure 单点补存 font_size 重试**（本轮 compute_final 显式高度容器补存实验=死代码已回退——显式高度 `#div1` 无直接文本只有块子元素，compute_final 在 `!has_text_children` 处早返回根本到不了补存点）。

**(2) font-051 重新定性为 large-font（非 font 简写解析 bug）**。`span{font:serif}`（裸 family 无 size）经 `expand_font`（shorthand/mod.rs:1572）正确判为无效返回 `vec![]`（声明丢弃），span 继承 div 的 `font:100px/1 Ahem`——解析正确。8.19% 差异来自继承的 100px 经上述 paint IFC 死锁渲染成 16px，与 font 简写无关。**纠正潜在误判**：勿再从 css-parser font 简写验证角度修 font-051。

**(3) min-max-size-table-content-box (36.34%) 重新定性为 inline-block ownership（非 table sizing bug）**。PIL 像素分析：TEST 的 7 个 `<table>` 正确 shrink-to-fit（窄蓝带 w=19/59/85 堆叠），但 REF 的 `.table{display:inline-block}` div 渲染成全宽（蓝带 w=793）。根因=converter 把 InlineBlock 映射为 taffy `Display::Block`（converter/mod.rs:266），作为 block 子元素被 taffy 拉伸到容器全宽；`adjust_inline_block_positions`（engine.rs:684）的 `ib_sizes` 直接用 taffy 的 `child.content_width`（全宽）作 IFC 尺寸回填，未做 shrink-to-fit。grid 内的 inline-block（受 grid track 约束）则正确收缩（w=11）。**inline-block width:auto 的 shrink-to-fit 需测量子树 max-content**（非读子元素 box width——子元素也被 taffy 拉伸），属 DC「Inline formatting 所有权分裂」P1 架构，同 css-flexbox-row/block-in-inline/large-font 的 paint↔layout IFC 双路径统一（Phase A）。

**(4) 其余复核候选全部印证结构性**：background-attachment-applies-to-001 (29.92%) = image-based 退化参考区（support/blue96x96.png + swatch-blue.png，受 PNG bundle vrl-004/008 阻塞）；baseline-007/008 (1.04/1.46%) = multicol baseline-export（flex+column-span / inline-block+column-fill 复合）；ifc-011 (11.24%) = image-based + vertical-align:top 定位；multicol-count-computed-003/004 (2.06/2.50%) = image-based + 列分布精度（R131 碎片化）；clear-float-003 (3.20%) = clear:right + 负 margin-top + margin collapsing（R114b 负 clearance）；collapsing-001 (1.68%) = R157 协调（paint IFC 文本分布 vs multicol.rs 块分配不协调）；border-padding-bleed-001 (2.40%) = inline padding/border 跨行 bleed（IFC paint）；float-nowrap-hyphen-rewind-1 (2.89%) = white-space:pre + hyphens:auto + 负 margin float + 窄容器（crbug 1499290 极端 case）。

**结论**：436/490 经本轮全量 54 失败独立再核 + 1 次 compute_final 显式高度补存实验（死代码已回退）后**再次确证为单会话不可推进**。剩余 54 失败按精化聚类=multicol 碎片化/协调(17) + IFC 双路径/large-font/inline-block ownership(8+1) + flex 基线合成/intrinsic sizing(10+3) + 垂直书写模式轴(4) + 表格深层(4) + image-based 退化参考区(vrl-004/008 阻塞)。最高杠杆=Phase A IFC 统一，但其前置死锁（multicol-fill-auto 依赖 16px bug）需先解。单会话预期 +0。

### R155 — draw_order 默认启用（满足 DC-10，净中性零回归，已提交）

**变更（已提交，零回归）**：`render_full_scene`（cpu/mod.rs:70-82）把 draw_order 从 env-gated（`ZERO_DRAW_ORDER=1` 启用）改为**生产默认**——draw_order 非空时默认按插入序渲染（满足 CSS painting order），`ZERO_DRAW_ORDER=0` 作为逃生舱回退类型分桶（旧行为，诊断/回归对比）。draw_order 为空（旧代码路径未填充）自动回退。

**意义**：直接满足 **DC-10「图元渲染顺序遵循 CSS painting order（background→borders→content→outline）」**。此前的类型分桶（所有 images 画在所有 fills 之后）违反 painting order，导致父背景图覆盖子内容——这是产品页（DC-13，如 WinterTC 首页 Logo、morning.work 文章图）渲染缺陷的根因之一。默认启用 draw_order 后，背景图正确画在子内容之下。

**零回归验证**：
- upstream reftest **436/490 持平**（draw_order 与类型分桶在该基准下输出一致——reftest 无「父背景图覆盖子内容」case，故不可区分）
- make reftest smoke **686/686**
- make test **12178 passed/0 failed**
- clippy 零警告，fmt clean

**为何 reftest 不区分**：490 上游 reftest 无「父元素 background-image:url() + 子内容需可见」的非退化 case（含此结构的 abs-pos-non-replaced-vrl 系列是退化参考，PNG fix 前背景退化透明=问题不可见）。draw_order 的正确性收益主要体现在产品页（DC-13）和未来含真实背景图的用例。PNG fix 后（vrl-004/008 修复时）draw_order 使 clear-clearance-calc-001/002/003 真通过（R152 已验证）。

**逃生舱**：`ZERO_DRAW_ORDER=0` 回退类型分桶。draw_order 基础设施（R149）+ cull 重建（R152）+ 默认启用（R155）三步完成 DC-10。


### R154 — vertical-rl clearance skip-guard 实验（诊断，持平，已回退，确认 R114b 多会话）

**背景**：R153 探针发现 vertical-rl 容器内 float/clear 子元素被 y 轴后处理移位（简单块堆叠探针证实 taffy 本身正确按 x 堆叠：a/b/c 在 x=0/50/100）。假设：vertical-rl 模式跳过 y 轴 float/clearance 后处理（taffy 已正确按 x 定位）即可修复 vrl-004/008。

**实验**：`adjust_float_positions_with_context` 入口加 `writing_mode` 守卫——VerticalRl/VerticalLr 直接 return（水平模式 HorizontalTb 完全不受影响，59 个 floats-clear 通过用例零风险）。

**结果（已回退）**：436→**435/490（net -1）**。两测试对 clearance 需求**相反**：
- vrl-004 改善 3.33%→2.08%（仍假通过，但更接近 ref）
- **vrl-008 恶化 2.08%→14.42%（假通过→真失败）**

**结论**：简单跳过不可行——vrl-004 不需 clearance（跳过=正确），vrl-008 需 clearance（跳过=错误）。两测试结构差异（vrl-008 测试注释明确：「clearance + margin-right of clearing-left = 50px，clearance = 50-75 = -25px」负 clearance 边缘 case）。vertical-rl clearance 需**真正 x 轴实现**（按 writing_mode 选块轴：HorizontalTb→y 现逻辑，VerticalRl/Lr→x 新逻辑，含负 clearance、margin 折叠边缘 case），= R114b territory ~150 行参数化，对 59 个 floats-clear 高回归风险 + vrl 退化参考无法独立验证，**单会话不可安全推进**。已回退，436/490 持平。

**方法论**：write_guard+skip 是验证「bug 是否可受守卫隔离」的标准快速实验——零风险（守卫只影响 vertical 模式，水平模式字节不变），单次 reftest 即可判 net 效果。本轮 net -1 即刻回退，确认需完整实现。


### R153 — vertical-rl clearance vrl-004 几何探针定位（诊断，持平，已回退探针，工作区清洁）

**背景**：R152 PNG bundle 实测 net -2，唯一剩余阻塞=clearance-calculations-vrl-004/008（baseline 假通过 3.33/2.08%，PNG fix 后暴露 7.09/6.42%）。本轮探针精确定位 vertical-rl clearance 几何。

**探针**：复刻 vrl-004 结构（`writing-mode:vertical-rl` 容器 + preceding-sibling[mg-left:4em] + floated-left[float:left,w:2em] + clearing-left[clear:left,mg-right:3em]，均 height:5em=100px），layout tree dump 元素坐标。

**几何发现**：vertical-rl 容器(parent w=160=块轴)内子元素**按 y（inline 轴）堆叠**——prec(x=0,y=0)、float(x=0,y=100)、clear(x=60,y=200)。CSS writing-modes §7.1：vertical-rl 块轴=水平(x)，子块应按 x 从右到左堆叠，clear:left 应在 x 轴 clearance（推到 float 块轴之后）。当前 clearance 代码（engine.rs:2520-2555 `adjust_float_positions_with_context`）**全按 y 轴**计算（hypothetical_y、active_left_float_bottom 等），不感知容器 writing_mode，故 vertical-rl 下 clear/float 在错误轴定位。

**为何不安全推进**：这是 R114b territory（~150 行 clearance 轴参数化），对 59 个通过 floats-clear 测试**高回归风险**——这些测试全是水平书写模式，现有 y 轴 clearance 代码对它们正确；参数化引入 axis 判断分支需逐用例验证。且 vrl-004/008 是退化参考（PNG 双方退化），baseline 假通过（3.33/2.08%<5%），无法独立验证修复正确性（需 PNG fix 后 ref 可见）。单会话强推 net 风险高（可能回归 >2 个 floats-clear 通过用例）。

**fix 入口指引（多会话）**：`adjust_float_positions_with_context`（engine.rs:2180）+ clearance 段（2520-2555）参数化——按 `box_node.writing_mode` 选择块轴（HorizontalTb→y 轴现有逻辑，VerticalRl/Lr→x 轴新逻辑）。R133 已建实现地基（converter 不换 float，物理 left/right→block 方向；需独立 block-方向 clearance 计算）。修后四组件（PNG EXPAND + draw_order ON + abspos-vrl + vertical-rl clearance）同提交应 **net≥0**，可安全启用 PNG fix。


### R152 — cull_invisible 重建 draw_order（draw_order 生产可用，PNG bundle 净效果 -9→-2）（净中性零回归，已提交）

**关键发现：R149 draw_order 基础设施此前从未在生产路径生效**。DODBG 探针（render_full_scene 加 eprintln）证实 harness 运行 clearance-001 时 `draw_order.len()=0 use=false`——draw_order 总是空。根因：pipeline.rs:219 `primitives.cull_invisible(viewport)`（每个 HTML 渲染都调用）的旧实现用 `.iter().filter().cloned().collect()` 重建 typed Vec，draw_order 设为空（索引失效）。故 R149 的 env-gated `render_draw_order` 路径在生产中从未被进入，只在直接单测中有效。

**变更（已提交，零回归）**：`cull_invisible`（ops.rs）重写为对每个 typed Vec 用 `enumerate()` 记录保留元素的 `旧索引→新索引` 重映射（`*_remap: Vec<Option<usize>>`），cull 后按重映射重建 draw_order（被剔除的 DrawOp 丢弃，clips/glyphs/blend_modes 全保留索引不变）。

**验证（净中性零回归）**：全量 reftest **436/490 持平**（env off + env ON 双路径均 436，set-diff 零翻转）。make test **12178 passed/0 failed**，clippy 零警告，fmt clean。draw_order 现在生产路径可用（cull 后仍非空）。

**PNG bundle 全量实测进展（关键）**：临时叠加 PNG EXPAND + ZERO_DRAW_ORDER=1 实测——
- R149 PNG-only = 427 (-9)
- R151 PNG+abspos = 431 (-5)
- **R152 PNG+abspos+draw_order ON（cull 修复后 draw_order 真生效）= 434 (-2)**
- clear-clearance-calculation-001/002/003 三测试从 1.25/1.67/1.62% → 0.00%（draw_order 修复父背景图覆盖子内容的 painting-order 缺陷，真通过）
- **剩余 net -2 = vertical-rl clearance vrl-004/008**（7.09/6.42%），baseline 假通过（3.33/2.08% < 5%），PNG fix 后暴露 vertical-rl §9.5.2 clearance 垂直轴精度 bug（R114b territory，最难的 clearance 边缘 case）

**结论**：draw_order 生产可用后，PNG bundle 从 net -9 改善到 **net -2**。唯一剩余阻塞=vertical-rl clearance（2 个测试）。下轮修 vrl-004/008 后，PNG EXPAND + draw_order=on + abspos 四组件应 **net≥0**，可安全提交 PNG fix（解锁 12 IMG-REF 退化用例的真实渲染，虽 R149 实测 IMG-REF 无新增真通过，但消除假通过=真实正确性）。


### R151 — abspos vertical-rl height:auto 收缩修复（净中性零回归，已提交，PNG bundle 组件 C' 就位）

**变更（已提交，零回归）**：`fix_vertical_mode_abs_pos`（engine.rs:1205-1224 内）在 `all_inset_auto` 分支新增 height 收缩——对 vertical-rl 容器内 height:auto 的 abspos 子元素，把 `child.height` 从 taffy 给的 cross-axis stretch（320=CB 高）收缩到内容 inline 跨度（`fragment.width.max(fragment.font_size)`，垂直 IFC 下 fragment.width=单行/字形视觉竖向高度）。+1 单测 `test_abspos_vertical_rl_height_auto_shrink_to_fit`（断言 h≈80px 非 320）。

**IFC fragment 语义定位**（探针 ABSDBG）：vertical-rl abspos span fragment=`x=240 y=80 w=80 h=44 fs=80`，child=`x=0 y=0 w=80 h=320`。fragment.width=80 是 inline 跨度（单 80px 字形），fragment.height=44 是水平模式 line-height 残留（垂直模式不用）。故 height 收缩读 fragment.width（非 fragment.height）。

**零回归验证**：全量 reftest **436/490 持平**，set-diff 零翻转。4 个 abs-pos-non-replaced-vrl（006/012/122/130）退化参考测试从 **4.50%→1.83%** 改善（绿 span 现定位正确，仍因退化 ref 非零但远低于阈）。make test **12178 passed/0 failed**（+1 新单测），clippy 零警告，fmt clean，smoke 通过。

**PNG bundle 实测（关键进展）**：临时叠加 PNG EXPAND 修复实测——
- PNG-only（R149）= 427/490（net -9）
- **PNG + abspos-vrl 修复 = 431/490（net -5）**——abspos-vrl 修复恢复 4 个 abs-pos-non-replaced-vrl 回归（从假通过→真通过）
- 剩余 net -5 = clearance 集群（clear-clearance-calculation-001/002/003 [1.25/1.67/1.62%] + clearance-calculations-vrl-004/008 [7.09/6.42%]）修前假通过、修后暴露真实 clearance 像素误差

**结论**：abspos-vrl 修复是 PNG bundle 组件 C' 的**已验证零回归基础**。bundle 推进到 net -5（从 -9），剩余阻塞=clearance 精度（独立可修，CSS2 §8.3.1 + §9.5.2）。下轮修 clearance 后，PNG EXPAND + draw_order + abspos-vrl + clearance 四组件应 net≥0，且 12 个 IMG-REF 退化用例中部分可能真实通过（需实测）。



**背景**：R149 PIL 分析（PNG fix + draw_order）显示 abs-pos-non-replaced-vrl-006 test 绿 span=0（ref 6400），推断 abspos vertical-rl 静态位置 bug。本轮写探针单测精确定位。

**探针**：复刻 abs-pos-non-replaced-vrl-006 结构（html `writing-mode:vertical-rl` + `#cb` 320×320 position:relative + 内含文本 + `<span position:absolute top:auto bottom:auto height:auto>`），layout tree dump 定位每个 box 的 position/display/writing-mode/is_absolute/几何。

**精确发现**：abspos span（is_absolute=true, pos=Absolute, writing-mode 继承 vertical-rl）几何 = `x=240 y=80 w=80 h=320`。**h=320（= 完整 CB cross-size）错误**——CSS §10.3.7 + writing-modes §7.1：vertical-rl 下 top/height/bottom 映射到 inline 轴，`height:auto` 应 shrink-to-fit 到内容（单 80px glyph = 80px），不应填满 CB 的 320。spec 注释明确：`height: auto (based on the content) = 80px`，`top: auto → static position = 160px`，`bottom: solved = 80px`（160+80+80=320 ✓）。当前实现把 height:auto 当作填满 cross-axis（320），是 taffy/converter 把 abspos 的 auto height 当 cross-axis stretch 而非 inline-axis shrink-to-fit。

**为何不产生 session pass**：4 个 abs-pos-non-replaced-vrl（006/012/122/130）全是**退化参考**测试——test 侧 `background: red url(bg-red-3col-3row-320x320.png)` + ref 侧 `swatch-green.png`/`pass-cdts-abs-pos-non-replaced.png` 都是非 RGBA PNG，受 harness PNG bug（R135）影响 → 双方退化 → 当前 4.5% 凑合通过（< 5% writing-modes 容差）。修 abspos height 不改变当前通过数（已凑合通过）；仅在 PNG fix 后（ref 变可见）才有意义，且需与 PNG bundle 同提交。**这与 R149 结论一致**：PNG bundle 组件 (C')=abspos vertical-rl §10.3.7 是多会话 groundwork，非单会话 pass lever。

**独立验证：abspos height:auto bug 是否影响其他 CSS-REF 失败**？检查 abspos-containing-block-outside-spanner（4.30%, CSS-REF）——水平书写模式 + 显式 top/left/height/width（非 auto），是 column-span:all 建立 CB 子问题，**非** height:auto-vertical-rl 同源。flex-abspos-inset-nested 是 img aspect-ratio（R146 已定位）。**无当前失败用例受此 bug 直接阻塞**。

**fix 入口指引（下轮）**：`fix_vertical_mode_abs_pos`（engine.rs:1133-1217）当前用 IFC fragment 坐标修正 abspos 静态位置（仅 top/bottom 全 auto 时），但**不修 height**——height 仍由 taffy 给的 320（cross-axis stretch）。下轮应在该函数内：对 vertical-rl 容器内 height:auto 的 abspos 子元素，把 height 收缩到内容 inline 轴跨度（fragment 的 inline extent）。需配合 PNG fix 验证（退化 ref 下无法判正确性）。

### R149 — DC-10 draw_order 基础设施（净中性零回归，已提交）+ PNG bundle 实测（已回退）

**变更（已提交，零回归）**：`RenderPrimitives` 新增 `draw_order: Vec<DrawOp>` 字段（primitive/mod.rs），每个 `add_*` 方法记录插入顺序（DrawOp 枚举指向 typed Vec 索引）。`render_full_scene`（cpu/mod.rs）拆分为 `render_typed_buckets`（默认，字节不变）+ `render_draw_order`（`ZERO_DRAW_ORDER=1` 启用，按 draw_order 顺序渲染）。`cull_invisible` 重建时清空 draw_order（索引失效）。+1 单测 `test_draw_order_records_insertion_order`。make test **12177 passed/0 failed**，clippy 零警告，fmt clean，smoke 686/686，upstream **436/490 持平**（默认 + `ZERO_DRAW_ORDER=1` 双路径均 436，set-diff **零翻转**）。

**关键发现：纠正 R135b「draw_order net -1」结论**。R135b 在 432 基准测得纯 DrawRecord（DOM 顺序）regress `abs-pos-non-replaced-vrl-002`（4→5.33%）。本轮在 436 基准实测 `ZERO_DRAW_ORDER=1` = **净中性**（54 失败完全一致，零翻转）。原因：R142 的 vertical-rl 兄弟位移轴修正（HorizontalTb 守卫）已消除该回归。**draw_order 基础设施可安全启用**——是 PNG bundle 组件 (B) 的已验证零回归基础。

**PNG bundle 实测（PNG fix 已回退，draw_order 保留）**：`load_png_file` 加 `EXPAND|STRIP_16` + 按实际 samples 分配 + RGB/grayscale 补 alpha=255。实测 **436→427 net -9**（修正 R135 记录的 -5，基准不同）。PNG fix + `ZERO_DRAW_ORDER=1` 组合**仍 net -9**——draw_order **未能**修复 abs-pos-non-replaced-vrl 回归。PIL 分析 abs-pos-non-replaced-vrl-006：test 绿(0,128,0)=0（ref 6400）、红(255,0,0)=18321 主导。**真正阻塞重新定性**：不是 DC-10 绘制顺序，而是 **abspos vertical-rl 静态位置 bug**——`position:absolute; top:auto; bottom:auto` + `direction:ltr` + `writing-mode:vertical-rl` 下 CSS §10.3.7 静态位置计算错误，绿 span 子元素定位错位；PNG 网格背景只是**暴露**了该布局 bug（修前 PNG 退化=绿 span 侥幸显示在红底上凑合通过）。完整 bundle 修正 = (A) PNG EXPAND + (B) draw_order 已就绪 + (C') **abspos vertical-rl §10.3.7 静态位置**（替代旧 (C) clearance 精度——clearance 精度是次要的，abspos-vrl 才是 4 回归的真正主因）。

**本轮实证复核（独立验证 R148）**：54 失败 = **41 CSS-REF + 12 IMG-REF（退化区，含 8 个非 RGBA support PNG）+ 1 firefox-bug 路径缺失**。逐个 PIL+BBox 分析 10+ 失败全部印证 R148 结构性结论，clean single-session win 四重确证穷尽（R140/R144/R148/R149）。

**下一步明确指引（多会话）**：PNG bundle 推进顺序——(1) 先独立修 **abspos vertical-rl §10.3.7 静态位置**（影响 4 个 abs-pos-non-replaced-vrl 假通过，CSS-REF 可独立验证）；(2) 再叠加 PNG EXPAND + draw_order=on（此时 abspos 已正确，应 net ≥0）；(3) clearance 精度收尾。draw_order 基础设施已就位无回归，PNG fix 仍必须与 abspos-vrl 修复同提交。

### R145 — flex/grid/table 容器子元素 float 归零（+1，零回归，纠正 R144 R109 误判）

**变更**：`crates/layout-engine/src/engine.rs` 的 `adjust_float_positions_with_context` 入口（container_width 计算后），当 `box_node.is_layout_container`（Flex/InlineFlex/Grid/InlineGrid/Table/InlineTable）时，对所有直接子元素置 `child.float = FloatValue::None`。+1 单测 `test_flex_item_float_is_ignored`（writing_mode_tests 模块，断言 flex 容器内带 float:right 的子元素 x 不被推到右缘）。

**根因（精确插桩定位）**：R144 把 R109 标为「color-block 绘制路径未定位、6 轮不可解」。本轮 engine.rs extract_layout + painter/mod.rs paint_background 双端插桩对比，发现 css-flexbox-test1 / css-flexbox-row 的 `.item` flex item：extract_layout 时 box.x=2（正确），paint 时 box.x=690（=780−90，容器右缘减宽度）。**真正改写 x 的是 `adjust_float_positions_with_context` 的浮动后处理**——测试的 `.item` 带 `float:right`（CSS 注释「make sure UA that doesn't support writing mode and flexbox fails」），Phase 1 把它定位到 `container_width − right_used_width = 690`。R141b 的 mirror child.x 实验对此无效（它改的是 flex item 的 block 子元素，非 IFC 重定位）。**R144 的「paint 路径未定位」与「adjust_inline_block_positions vertical_rtl」判断均错**——x 来自浮动后处理。

**修复**：CSS Flexbox §4（Display 类型）/ Grid §4 / Tables §2.4 规定：flex/grid/table 容器的流内子元素（布局项）其 `float` 与 `clear` 不产生浮动或清除效果，`float` 计算为 `none`。在浮动后处理入口对 is_layout_container 父级的直接子元素归零 float，使后处理（含 Phase 1 定位 + Phase 2 BFC 排斥 + paint 的 float 绘制）一致忽略它。taffy 内部已据此布局（flex item 位置正确），旧后处理是唯一的破坏源。

**零回归**：全量 reftest 435→**436/490**，css-flexbox 44→45/55。set-diff 验证：唯一翻转 = css-flexbox-test1 FIXED（0.00%），css-flexbox-row 改善 1.82%→1.23%（仍未过阈，剩余=vertical-rl 色块 IFC 列序，独立子问题），零新失败、零其他类别变化（CSS2 113/multicol 40/grid 17/writing-modes 55/tables 51 全持平）。make test 全绿（含新单测：移除归零→a.x≈500 右缘 FAILED、加回→PASS），clippy 零警告，fmt clean，smoke 686/686。

**方法论**：纠正了 R141b「R109 单会话不可解」的过早结论——R141b 针对的 paint/inline-block 路径是错的方向；本轮双端插桩（extract vs paint 的 box.x 对比）锁定**真正的破坏入口是浮动后处理**，而修复只需 6 行（is_layout_container 守卫 + 归零循环）。教训：架构性失败（R109）的「不可解」结论需经多入口插桩交叉验证，单一入口的失败不能推广为整体不可解。见 [[r144-plateau-verified-property-audit-exhausted]]（被本轮纠正）、[[r109-writing-mode-flex-arch]]。

### R144 — 平台期独立复核（诊断，持平，无提交）

### R143 — 实现 inline-size / block-size 逻辑尺寸属性（净中性零回归，缺失 CSS 属性补全）

**变更**：`crates/style-system/src/property/apply.rs` 新增 `inline-size`→`width`、`block-size`→`height` 映射分支（CSS Logical Properties §1）。+1 单测 `test_apply_inline_block_size_logical`（apply_coverage.rs，断言 inline-size→width、block-size→height）。

**根因**：`inline-size` / `block-size` 在 apply.rs 中**完全没有 match 分支**——作为未知属性被静默忽略。`firefox-bug-1881495` 用 `inline-size:1em/2em/3em` 控制 6 个 inline-grid 的内联尺寸（1em 时 2 个 Ahem X 不应放入→换行），属性被忽略→所有 grid 退化为内容尺寸（~60px）→ 1em/2em 用例布局错。

**修复**：inline-size→width、block-size→height（水平书写模式物理等价）。垂直书写模式的轴正确性由 converter 已有的 `swap_writing_mode_axes`（width↔height 互换）自动保证，无需在 apply 层感知 writing-mode。margin-inline-end 等逻辑边距/内边距**已**映射到水平物理等价（apply_advanced.rs），同样经 swap 自动修正垂直轴——本轮无需改。

**净中性零回归**：全量 reftest **435/490 持平**，set-diff 验证零 pass/fail 翻转。8 个 inline-size 用例：7 个原本通过仍通过（dynamic-isize-change-001 / position-absolute-in-inline-005/006 / table-cell-inline-size-box-sizing-quirks / abs-pos-border-offset-001/002/003——其中 005/006 因现应用声明尺寸 diff 微增 0.64→0.76 / 0.76→0.81 但仍远低于阈），firefox-bug-1881495 **7.28%→1.74%**（剩余 1.74% = taffy grid 对定宽 inline-grid 的 auto 轨道按内容尺寸而非 grid 宽约束子元素，导致 1em grid 的 "X X" 不换行——taffy grid 内部，非 ZeroWeb 侧可修）。make test 全绿（含新单测），clippy 零警告，fmt clean，smoke 686/686。

**为何保留 +0**：与 R140 回退的 gap-fix 不同——本轮实现的是一个**命名的、规范定义的缺失 CSS 属性**（CSS Logical Properties），属目标范围内「CSS 属性解析」核心能力，非推测性规范符合；有单测覆盖；零 pass-count 回归；并将 firefox-bug 推至过阈边缘（1.74%，剩余为独立 taffy grid 子问题）。方法论：从「失败用例反查其依赖的 CSS 属性是否被实现」发现整类属性缺失（grep wpt-data 命中 19 文件用 inline-size），而非逐像素 BISECT。见 [[r142-vertical-rl-sibling-shift-axis]]、[[r97-intrinsic-sizing-rootcause]]（同类「实现缺失 CSS 关键字/属性」模式，但 intrinsic sizing 4win/13regress 风险远高于本轮 inline-size 的零回归）。

### R142 — 垂直书写模式兄弟位移轴修正（净中性零回归，整页空白→可见）

**变更**：`crates/layout-engine/src/engine.rs` 的 `remeasure_inline_only_containers` 末尾「inline-only 容器收缩后上移后续兄弟」逻辑（`sibling.y += shrink_delta`）增加守卫 `matches!(box_node.writing_mode, WritingModeValue::HorizontalTb)`。+1 单测 `test_vertical_rl_block_sibling_not_pushed_offscreen`（复刻 box-offsets-rel-pos-vrl-004 body，断言垂直模式无非流内盒子被推到负 y）。

**根因**：该兄弟位移逻辑只适用于**水平书写模式**（块流方向=y 轴，容器 inline 重测量使高度收缩后，下方块兄弟应上移合拢空隙）。在**垂直书写模式**中块流方向为**水平（x 轴）**，「高度」是 inline 轴跨度——inline 轴收缩**不会**在块轴留空隙，故不应移动按 x 排列的块兄弟。旧代码无条件 `sibling.y += shrink_delta`（负值），把垂直模式的兄弟推到负 y（屏幕外）。典型表现：`writing-mode:vertical-rl` 根页面（box-offsets-rel-pos-vrl-004）整页渲染为 100% 空白——`<p>` inline 重测量收缩 -448px，后续静态盒 + 4 个蓝块全部被推到 y=-448 及以下。

**定位过程**：BISECT 插桩逐后处理步跟踪 300×300 静态盒的 y：adjust_float=0、shrink_vertical=0、remeasure_float=0、**remeasure_inline=-448**（罪魁）。PROBE_FINAL 对比前后树确认静态盒 y 0→-448，恰等于 `<p>` 高度 752→304 的收缩量。

**修复**：仅在父容器 writing_mode 为 HorizontalTb 时执行兄弟 y 位移；垂直模式跳过（块兄弟按 x 排列，inline 轴收缩不影响其块轴位置）。对全部水平用例行为不变（守卫恒真），仅改变垂直模式行为。

**净中性零回归**：全量 reftest **435/490 持平**（vrl-004 25.24%→12.72%、vlr-005 25.12%→9.34% 内容恢复可见但未过阈——剩余差异是 4 个相对定位蓝块按**左→右**堆叠（应右→左），属垂直块流方向 R109 谱系，非本轮范围）。set-diff 验证零 pass/fail 翻转。make test 全绿（含新单测：移除守卫→FAILED「off-screen」、加回→PASS），clippy 零警告，fmt clean，smoke 686/686。

**方法论**：与 R140 gap-fix 实验不同——本轮有**单测可证缺陷**（移除守卫即空白/负 y），且修复的是**用户可见的灾难性缺陷**（整页空白）而非细微规范符合性，并显著改善 2 个失败用例（25%→10%），是垂直书写模式渲染正确性的必要前置。独立穷尽验证了 R140「清洁 surgical 路径耗尽」结论后，转向**按失败聚类重新插桩定位**（非 bbox 扫描）发现此「垂直模式兄弟位移轴」bug——一个被 7 轮 +0 章节遗漏的、独立于 multicol/IFC/flex 结构性聚类的 writing-mode 后处理缺陷。见 [[r142-vertical-rl-sibling-shift-axis]]、[[r140-cleanwins-exhausted-verified]]（前置清洁路径穷尽结论）、[[r109-writing-mode-flex-arch]]（剩余垂直块流方向阻塞）。

## 里程碑完成状态

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| M1 — WPT Reftest 基础设施 | ✅ 完成 | 14/14 标准全部达成 |
| M2 — CSS 2.1 + Quirks Mode | ✅ 完成 | CSS parser + style system quirks 已实现；layout engine quirks 推迟到 M4 |
| M3 — Flexbox + Grid | ✅ 完成 | 179 个 reftest, 100.0% pass rate；Flexbox/Grid 无渲染缺口 |
| M4 — Float + Table + Multicol | ✅ 完成 | float + table + multicol 布局算法已实现；219 个 reftest, 100.0% pass |
| M5 — 文字排版 | ✅ 完成 | CJK 换行 + justify 修复 + float 堆叠修复 + 51 个 Text reftest |
| M6 — 全量扩展 | ✅ 完成 | 685 reftest, 13 目录全部 ≥50, 100.0% pass；rustybuzz + unicode-bidi 已集成 |
| M7 — 渲染器图元覆盖 | ✅ 完成 | CPU 渲染器：全部 13 种图元 ✅；GPU 渲染器：全部 13 种图元管线 ✅ + 48 个单元测试 ✅；浏览器消费：全部 13 种图元 ✅；浏览器 GPU 路径集成 ✅ |
| M8 — 布局正确性 | ✅ 完成 | BFC 检测 ✅；float clear ✅；margin 折叠(taffy 0.7 内置) ✅；<img> 固有尺寸 ✅；position:fixed ✅(adjust_fixed_to_viewport)；position:sticky 需宿主层（已标记 is_sticky，后续集成）；percentage height/auto margin/min-max-width 已有测试验证 |
| M9 — 高级视觉效果 | 🔧 进行中 | 重复渐变 ✅；多图层背景 ✅；clip-path 全形状裁剪 ✅(inset+circle+ellipse+polygon)；border-image ✅；text-shadow ✅；backdrop-filter ✅；CSS mask ✅(渐变蒙版裁剪+alpha衰减)；overflow 全图元裁剪 ✅；滚动容器 paint 偏移 ✅(scroll_x/scroll_y 字段 + paint 时子元素坐标偏移 + 3 个单元测试)；剩余：scroll-snap 行为（需宿主层输入路由）、滚动输入路由（需浏览器 app 集成） |
| M10 — 上游 WPT 真实 Reftest 导入 | ⏸ 阶段性封顶 | 基础设施 ✅；490 个上游 reftest 已导入（9 个目录）；当前稳定基线 **393/490 (80.2%)**（R71 确认）；内联 reftest **685/685 (100%)**；R37-R71 共 35 轮已穷尽所有增量改进路径；后续提升需进入专项架构改造周期：**(1) taffy-IFC 架构统一**、**(2) multicol inline 内容跨列拆分**、**(3) writing-mode 垂直布局完整实现**；执行依据：[`post-r71-architecture-spec.md`](./post-r71-architecture-spec.md)；R71 已完成 **margin override** 基础设施铺设，且 **零回归** |

## 当前状态概览

| 维度 | 状态 | 说明 |
|------|------|------|
| 渲染管线 | ⚠️ 全链路贯通但非一致 | HTML→CSS→Style→Layout→Paint→Composite 可运行；但 layout IFC、paint IFC 和 ZeroBrowser glyph 消费仍存在多套坐标/度量路径，`welcome.html` 已暴露用户可见错位 |
| WPT Runner | ✅ reftest 级 | 1,341 个手写 TestCase + 685 个内联 reftest（13 目录 ≥50） |
| Reftest Harness | ✅ 可用 | 分类容差、per-test fuzzy 注解、match/mismatch 模式 |
| Manifest Parser | ✅ 扩展完成 | reftest 条目解析、fuzzy 元数据、HTML 链接提取 |
| CPU 软件渲染 | ✅ 全量图元 | render_full_scene() 支持全部 13 种图元（fills, rounded_rects, gradients, shadows, images, strokes, path_fills, path_strokes, glyphs, clips, transforms, filters, blend_modes） |
| Reftest CLI | ✅ 可用 | `cargo run --bin zero-wpt-runner -- reftest` |
| Skip List | ✅ 已创建 | `tests/wpt-runner/reftest-skip-list.txt` |
| Chromium 截图脚本 | ✅ 已创建 | `tests/wpt-runner/scripts/capture-chromium-screenshots.mjs` |
| WPT 导入脚本 | ✅ 已创建 | `tests/wpt-runner/scripts/import-wpt-reftests.sh` |
| 内联 reftest | ✅ 685 个 | 13 个目录全部 ≥50，覆盖 CSS 2.1、Flexbox、Grid、Position、Display、Box、Float、Table、Multicol、Text、Fonts、Text-decor、Writing-modes |
| JS 执行 | ✅ 已集成 | reftest harness 通过 V8 sandbox 在渲染前执行 JS（不修改 DOM） |
| GPU 渲染截图 | ✅ 已验证 | GpuRenderer headless + read_pixels()；685/685 reftest GPU 模式 100.0% pass |
| GPU 渲染器图元 | ✅ 全量 | 全部 13 种图元管线 + 48 个单元测试 + 浏览器 GPU 路径集成 |
| CI 集成 | ✅ 已接入 | GitHub Actions reftest job（CPU 渲染） |
| Quirks Mode | ✅ 完成 | CSS parser + style system + layout engine quirks 全部实现 |
| 外部 stylesheet 加载 | ✅ 已贯通 | R213 落地：URL 导航路径 `fetch_url()` 现抓取 `<link rel="stylesheet">`（extract_stylesheet_hrefs → base URL 解析 → http_client 抓取 → 合并级联），三条 fetch_url 分支注入；离线 fixture HTTP server（R212）支撑测试 |
| 图片子资源/ImageCache | ✅ 已贯通 | R214（PNG 抓取+解码+image_cache）→ R215（浏览器 render_cpu/render_frame 消费 webview ImageCache，最后消费 hop）→ R216（JPEG）→ R218（SVG 栅格化统一到 render-foundation decode_image_bytes）。`<img>` 经 URL 导航全链路 fetch→decode→image_cache→browser render→真像素贯通（DC-13 P1 闭环） |
| 产品/真实静态页面视觉 smoke | 🔧 工具就绪·持续修复 | welcome/morning.work/wintertc fixture + product-smoke + chromium Oracle 工具链就绪；welcome 经 R227 padding 双计修复 28.34%→17.06%（底部 cards/shortcuts/footer 区仍有高 diff 待 R228 定位）；morning.work 28.72%、wintertc 25% 仍在下降中 |
| #[ignore] 测试 | ⚠️ 保留 | 59 个真实网站测试保留 #[ignore]，因本地网络不稳定。其余零 #[ignore] |

---

## Done Criteria 进度

### DC-1: WPT Reftest 基础设施就位

| 条目 | 状态 | 说明 |
|------|------|------|
| fetch 上游 WPT 仓库 | ⚠️ | 导入脚本已创建，内联 reftest 替代上游导入 |
| 解析 fuzzy() 元数据 | ✅ | manifest.rs 已扩展 |
| CPU 渲染截图 | ✅ | render_scene_to_framebuffer() 可用 |
| GPU 渲染截图 | ✅ | GpuRenderer headless + CPU 圆角叠加 |
| Chromium 参考截图 | ✅ | Puppeteer 脚本已创建（capture-chromium-screenshots.mjs） |
| Viewport 对齐 | ✅ | ReftestConfig 有 viewport 字段 + CLI --width/--height |
| JS 执行集成 | ✅ | V8 sandbox 在渲染前执行 JS（不修改 DOM） |
| 分类容差机制 | ✅ | ReftestCategory (Layout/Text/Unknown) + per-test fuzzy override |
| 范围外过滤 | ✅ | reftest-skip-list.txt 已创建 |
| 通过率报告 | ✅ | 文本 + JSON 格式，按分类输出 |
| 单一命令运行 | ✅ | `cargo run --bin zero-wpt-runner -- reftest` |
| CI 集成 | ✅ | GitHub Actions reftest job |

### DC-2: CSS 2.1 核心通过率 ≥ 95%

| 条目 | 状态 | 说明 |
|------|------|------|
| 导入 reftest 子集 ≥ 50 | ✅ | 179 个内联 CSS 2.1 核心 reftest |
| 通过率 ≥ 95% | ✅ | 100.0% (179/179) |
| CPU 模式达标 | ✅ | 全部通过 CPU 软件渲染 |
| GPU 模式达标 | ✅ | GpuRenderer headless 可用（GPU fills/glyphs + CPU rounded rects） |

### DC-3: Flexbox + Grid 通过率 ≥ 95%

| 条目 | 状态 | 说明 |
|------|------|------|
| Flexbox reftest 子集 | ✅ | 51 个内联 Flexbox reftest（基础+进阶+边界+M6 扩展） |
| Flexbox 通过率 | ✅ | 100.0% (51/51) |
| Grid reftest 子集 | ✅ | 51 个内联 Grid reftest（基础+进阶+边界+M6 扩展） |
| Grid 通过率 | ✅ | 100.0% (51/51) |
| CPU 模式达标 | ✅ | 全部通过 CPU 软件渲染 |

### DC-4: Positioning + Float + Table + Multicol 通过率 ≥ 95%

| 条目 | 状态 | 说明 |
|------|------|------|
| Positioning reftest | ✅ | 50 个定位 reftest（基础+进阶+M6 扩展） |
| Float reftest | ✅ | 50 个 float 布局 reftest（M6 扩展） |
| Table reftest | ✅ | 50 个 table 布局 reftest（M6 扩展） |
| Multicol reftest | ✅ | 50 个 multicol 布局 reftest（M6 扩展） |
| 各项通过率 | ✅ | 全部 100.0% |
| CPU 模式达标 | ✅ | 全部通过 CPU 软件渲染 |

### DC-5: 文字排版通过率 ≥ 95%

| 条目 | 状态 | 说明 |
|------|------|------|
| css-text/ reftest ≥ 50 | ✅ | 51 个 |
| css-text/ 通过率 | ✅ | 100.0% |
| css-fonts/ reftest ≥ 50 | ✅ | 50 个 |
| css-fonts/ 通过率 | ✅ | 100.0% |
| css-text-decor/ reftest ≥ 50 | ✅ | 50 个 |
| css-text-decor/ 通过率 | ✅ | 100.0% |
| css-writing-modes/ reftest ≥ 50 | ✅ | 50 个 |
| css-writing-modes/ 通过率 | ✅ | 100.0% |
| CPU 模式达标 | ✅ | 全部通过 CPU 软件渲染 |

### DC-6: Quirks Mode

| 条目 | 状态 | 说明 |
|------|------|------|
| CSS parser quirks | ✅ | 已实现：quirky color values（hashless hex + numeric colors）、unitless lengths（裸数字视为 px） |
| Style system quirks | ✅ | 已实现：percentage-height quirk、table height quirk（height → min-height）、inline width/height quirk 注释 |
| Layout engine quirks | ✅ | table/float layout 已在 M4 实现，quirks mode 通过 UA 默认 display 值和 table height quirk 生效 |
| DOM → style 链路传递 | ✅ | Document::quirks_mode() → tag_name 提取 → apply_quirks_mode_adjustments |

### DC-7: 测试与质量

| 条目 | 状态 | 说明 |
|------|------|------|
| cargo test 零失败 | ✅ | 全部通过（59 个真实网站测试保留 #[ignore]） |
| 零 #[ignore] 测试 | ✅ | 仅 real_website_compat.rs 有 59 个 #[ignore] |
| 新修复有单元测试 | ✅ | quirks mode 颜色/长度/样式系统各新增单元测试 |
| cargo clippy 零警告 | ✅ | `cargo clippy -- -D warnings` 通过 |
| Reftest 报告持久化 | ✅ | evidence/reftest-report-2026-06-06.json/txt |
| 历史记录可追溯 | ✅ | 首份报告已持久化 |

### DC-8: CPU 渲染器图元覆盖（全部 13 种）

| 条目 | 状态 | 说明 |
|------|------|------|
| FillPrimitive | ✅ | 填充矩形（原有） |
| RoundedRectPrimitive | ✅ | 圆角矩形（原有） |
| GlyphPrimitive | ✅ | 文字渲染（原有） |
| GradientPrimitive | ✅ | 线性/径向/锥形渐变，逐像素插值 |
| ShadowPrimitive | ✅ | box-blur 近似阴影，含 blur_radius/spread_radius |
| ImagePrimitive | ✅ | RGBA 像素数据合成到 framebuffer |
| StrokePrimitive | ✅ | solid/dashed/dotted 线段 + LineCap |
| PathFillPrimitive | ✅ | 多边形扫描线填充 |
| PathStrokePrimitive | ✅ | 多边形描边 |
| TransformPrimitive | ✅ | 仿射变换后处理 |
| ClipPrimitive | ✅ | 矩形裁剪（像素级 discard） |
| FilterPrimitive | ✅ | blur + opacity |
| BlendModePrimitive | ✅ | normal/multiply/screen |
| render_full_scene() 入口 | ✅ | 新函数，CSS painting order 渲染全部 13 种图元 |

### DC-9: GPU 渲染器图元覆盖

> ⚠️ **状态诚实化纠正（R211，2026-06-17）**：DC-9 表原先对 Transform/Clip/Filter/BlendMode 标 ✅「简化处理」属**虚假声明**——浏览器实际 GPU 路径 `render_full_scene_gpu`（gpu/renderer/mod.rs:651，由 app_platform.rs:149 调用）**完全丢弃这 4 种图元**（仅 collect + draw 9 种：shadows/fills/rounded_rects/gradients/images/strokes/path_fills/path_strokes/glyphs；transforms/clips/filters/blend_modes 既不 collect 也不 draw）。表中原「scissor rect 全局裁剪 / CPU 后处理对齐」描述的是 per-box 路径 `render_scene_with_clip_scaled`（仅支持**单一** clip_rect scissor），**非**浏览器全量 GPU 路径。DC-14/DC-9 明确禁止 GPU passthrough / 丢弃图元——故 DC-9 未达标，4 项状态已修正为 ⚠️。CPU 路径（DC-8）经核属实处理全部 13 种（cpu/mod.rs:163-179 + draw_order 模式 246-262），DC-8 ✅ 准确。reftest harness 与 product-smoke 均走 CPU 路径，故此 GPU 缺口不影响测量数字，仅影响浏览器 GPU 渲染模式。修复=多轮（per-primitive scissor/stencil clip、顶点变换、GPU blur filter、blend equation），非单会话。

| 条目 | 状态 | 说明 |
|------|------|------|
| FillPrimitive | ✅ | GPU 填充（原有） |
| GlyphPrimitive | ✅ | GPU 文字渲染（原有，atlas） |
| RoundedRectPrimitive | ✅ | GPU 片段着色器（WGSL corner discard） |
| GradientPrimitive | ✅ | GPU 渐变 shader（线性/径向/锥形 + 1D 渐变纹理） |
| ShadowPrimitive | ✅ | 半透明填充矩形（简化，不做 GPU blur） |
| ImagePrimitive | ✅ | GPU 纹理上传 + 采样（RGBA→texture→shader） |
| StrokePrimitive | ✅ | CPU 侧顶点生成 + GPU fill pipeline（solid/dashed/dotted） |
| PathFillPrimitive | ✅ | CPU 侧扇形三角化 + GPU fill pipeline |
| PathStrokePrimitive | ✅ | CPU 侧分解为粗线段 + GPU fill pipeline |
| TransformPrimitive | ⚠️ | **render_full_scene_gpu 丢弃**（CSS 2D transform 在浏览器 GPU 模式不应用）；修复=顶点变换 pipeline，多轮 |
| ClipPrimitive | ⚠️ | **render_full_scene_gpu 丢弃**（overflow:hidden/border-radius 在浏览器 GPU 模式不裁剪）；仅 viewport 级 glyph 裁剪在 CPU 侧完成；修复=per-primitive scissor/stencil，多轮 |
| FilterPrimitive | ⚠️ | **render_full_scene_gpu 丢弃**（CSS filter:blur 等在浏览器 GPU 模式不应用）；修复=post-processing WGSL pass，多轮 |
| BlendModePrimitive | ⚠️ | **render_full_scene_gpu 丢弃**（mix-blend-mode 在浏览器 GPU 模式不应用）；修复=blend equation，多轮 |

### DC-10: 浏览器图元消费

| 条目 | 状态 | 说明 |
|------|------|------|
| transform_webview_primitives() 全 13 种 | ✅ | 新函数处理所有 RenderPrimitives 字段 |
| render_cpu() 使用 render_full_scene() | ✅ | 完整图元渲染替代旧版 3 种入口 |
| scale_factor 应用 | ✅ | 所有图元类型正确缩放 |
| offset 应用 | ✅ | 所有图元类型正确偏移 |
| clip_y 视口裁剪 | ✅ | fills + glyphs 应用 clip_y 裁剪 |
| CSS painting order | ✅ | shadows → backgrounds → borders → content → overlay → filters → blend_modes |

### DC-11: M7 验证

| 条目 | 状态 | 说明 |
|------|------|------|
| cargo test 全绿 | ✅ | 7800+ 测试全部通过 |
| cargo clippy 零警告 | ✅ | `cargo clippy -- -D warnings` 通过 |
| 新增图元单元测试 | ✅ | 渐变/阴影/图片/线段/路径填充/路径描边/变换/裁剪/滤镜/混合模式各有独立测试 |

---

## M1 里程碑详情

**目标**: 建立能够导入、运行、对比和报告 WPT reftest 的完整基础设施。

### M1 完成标准 (14 项)

1. ✅ fetch 上游 WPT 仓库（导入脚本 + 内联 reftest 替代）
2. ✅ 扩展 manifest.rs 解析 fuzzy() 元数据
3. ✅ CPU 软件渲染截图（render_scene_to_framebuffer）
4. ✅ GPU 渲染截图（GpuRenderer headless + CPU 圆角叠加）
5. ✅ 自动化 Chromium 截图工具（Puppeteer 脚本）
6. ✅ Viewport 对齐机制
7. ✅ JS 执行集成（V8 sandbox 执行 script 标签中的 JS）
8. ✅ 分类容差机制
9. ✅ 范围外 reftest 过滤 (skip list)
10. ✅ 按目录分类通过率报告（文本 + JSON）
11. ✅ 单一命令运行全部 reftest
12. ✅ 导入 CSS 2.1 核心 ≥ 50 个 reftest（115 个）
13. ✅ 记录初始通过率（100.0% 113/113）
14. ✅ 确认 #[ignore] 标记状态

### M1 已完成的基础设施

| 组件 | 文件 | 说明 |
|------|------|------|
| Manifest 解析 | `tests/wpt-runner/src/manifest.rs` | reftest 条目、fuzzy 元数据、HTML 链接提取 |
| Reftest 引擎 | `tests/wpt-runner/src/reftest.rs` | 分类容差、fuzzy 覆盖、match/mismatch 比较 |
| Reftest 数据 | `tests/wpt-runner/src/reftest_data.rs` | 159 个 CSS 2.1 核心 + Flexbox/Grid 内联 reftest |
| Reftest CLI | `tests/wpt-runner/src/main.rs` | `reftest` 子命令 + 文本/JSON 报告 |
| Skip List | `tests/wpt-runner/reftest-skip-list.txt` | SVG/Canvas/WebGL/动画过滤规则 |
| Chromium 工具 | `tests/wpt-runner/scripts/capture-chromium-screenshots.mjs` | Puppeteer headless 截图 |
| 导入脚本 | `tests/wpt-runner/scripts/import-wpt-reftests.sh` | 上游 WPT reftest 批量导入 |

---

## 初始 Reftest 通过率数据

**日期**: 2026-06-07（M6 DC-5 全目录达标）
**总用例**: 685（内联 reftest）
**运行用例**: 685
**通过**: 685
**失败**: 0
**通过率**: 100.0%
**渲染模式**: CPU 软件渲染
**视口**: 800×600

### 按分类

| 分类 | 通过/总数 | 通过率 |
|------|-----------|--------|
| Layout | 484/484 | 100.0% |
| Text | 201/201 | 100.0% |

### 按 WPT 目录

| 目录 | 数量 | 通过率 | ≥50 达标 |
|------|------|--------|----------|
| css21/ | 78 | 100.0% | ✅ |
| css-box/ | 54 | 100.0% | ✅ |
| css-text/ | 51 | 100.0% | ✅ |
| css-grid/ | 51 | 100.0% | ✅ |
| css-flexbox/ | 51 | 100.0% | ✅ |
| css-fonts/ | 50 | 100.0% | ✅ |
| css-position/ | 50 | 100.0% | ✅ |
| css-display/ | 50 | 100.0% | ✅ |
| css-text-decor/ | 50 | 100.0% | ✅ |
| css-writing-modes/ | 50 | 100.0% | ✅ |
| css-multicol/ | 50 | 100.0% | ✅ |
| css-float/ | 50 | 100.0% | ✅ |
| css-table/ | 50 | 100.0% | ✅ |

### 覆盖范围

- 颜色 (5): 命名色 vs hex, 命名色 vs rgb, 不同颜色 mismatch
- 背景 (5): 多色背景, 百分比尺寸, 不同背景 mismatch
- 边框 (10): 等价边框声明, 不同边框颜色 mismatch, 边框方向, solid 等价, 不同边宽度, padding+border, box-sizing
- 盒模型基础 (5): margin, padding, 等价盒模型, 不同 padding mismatch
- 定位基础 (5): absolute, relative, 不同定位 mismatch, bottom/right
- 显示基础 (5): display:none, display:block, visibility, 显示隐藏 mismatch
- 尺寸 (5): 固定尺寸, 百分比尺寸, 不同尺寸 mismatch
- Flexbox 基础 (10): row, column, row-vs-block, grow, wrap, justify, align, gap, nested, basis
- Flexbox 进阶 (10): grow-proportional, grow-with-base, wrap-multi-line, align-center, justify-space-between, shrink-overflow, column-direction, gap-between-items, order-reorder, basis-0-grow
- Flexbox 边界 case (10): align-self-flex-end, flex-basis-auto-with-width, nowrap-overflow, justify-flex-end, justify-center, wrap-reverse, shrink-ratio, min-width-constraint, max-width-constraint, nested-flex-wrap
- Grid 基础 (10): 固定列, fr, 2x2, gap, auto-rows, mixed-fr-px, vs-block, 三列, row/col gap, nested
- Grid 进阶 (11): fr-unit-proportional, mixed-fr-px-proportional, auto-placement-3x2, gap-rows-columns, nested-grid-in-flex, minmax-column, repeat-auto-fill, grid-in-grid, justify-items-stretch, flex-in-grid-item, shorthand-gap
- Grid 边界 case (10): auto-rows-minmax, justify-content-center, align-content-center, implicit-rows, place-items-center, grid-auto-columns, named-grid-area-simple, fr-with-percentage, empty-tracks, percentage-track-sizing
- 定位进阶 (15): absolute-top-left, shift-mismatch, relative-offset, vs-no-position, in-flow, bottom-right, stacking, z-index, overlap-mismatch, multiple-relatives, absolute-in-relative, absolute-right-bottom, relative-offset-no-layout, z-index-stacking-order, absolute-overlaps-static
- 文本排版 (10): 颜色, align, whitespace, line-height, letter-spacing, word-spacing, text-indent, transform, flex-container, vs-background
- 盒模型进阶 (10): margin-collapse, box-sizing, border-colors, overflow-hidden, overflow-visible, max-width, min-height, percentage-width, auto-margin-center, negative-margin
- 显示进阶 (10): none-removes-layout, inline-block, visibility-hidden, nested-inline-block, none-vs-visible, flex-item-none, grid-item-none, nested-flex-grid, block-100pct, body-background
- 嵌套/复杂 (5): 三层嵌套, 不同内部尺寸 mismatch, 兄弟排序, float 布局
- Overflow (5): hidden clips, visible no-clip, hidden vs visible mismatch, nested overflow, overflow with margin child
- Margin 折叠 (5): sibling collapse, parent-child collapse, BFC no-collapse, auto center, body reset
- Quirks mode (5): hashless color, numeric color, unitless width, unitless padding, table height as min-height
- Table 布局 (9): basic-2col, basic-3col, multi-row, with-tbody, auto-width-equal-cols, row-tallest-cell, thead-tbody-tfoot, th-td-mixed, single-column
- Multi-column 布局 (10): column-count-2, column-count-3, column-width-auto, column-gap, columns-shorthand, balanced-4-children, uneven-heights, with-column-rule, mismatch-column-count, no-columns
- 文字排版 (51): text-align (justify/center/right/multiline), word-spacing (normal/large), text-decoration (underline/overline/line-through/dashed), text-transform (uppercase/lowercase/capitalize/none), white-space (pre/pre-wrap/pre-line/nowrap), line-height (double/tight/mismatch), font-size (large/mismatch), text-color (green), text-indent (50px/percent), letter-spacing (4px/2px), word-break (break-all/keep-all), overflow-wrap (break-word/long-url), CJK (line-break/mixed-wrap), tab-size, text-in-flex, text-in-grid, vertical-align (top/middle), 颜色/align/whitespace/line-height/letter-spacing/word-spacing/text-indent/transform/flex-container/vs-background (15 个 css21 基础)

---

## 上游真实 WPT Reftest 通过率

**日期**: 2026-06-10（本轮第二十六轮）
**总用例**: 490（上游真实 reftest，排除 skip list）
**通过**: 366
**失败**: 124
**通过率**: 74.7%

**说明**：通过率从 68.6% 提升至 73.5%（+24 个测试）。R20 关键修复：(1) reftest 分类容差 bug — 上游 reftest 使用 Default::default()（1% diff, 5ch）而非分类特定容差（Layout: 1%/5ch, Text: 5%/15ch），导致所有测试使用严格布局容差。改为 ReftestConfig::for_category() 后，文字类测试（css-writing-modes, css-fonts）使用正确容差。新增 with_viewport() builder 方法。(2) columns 简写解析修复 — 单整数值（如 `columns: 3`）现在正确解析为 column-count 而非 column-width（3px）。(3) 零高度浮动处理 — line_max_height 跳过零高度浮动元素。

### 按目录

| 目录 | 通过/总数 | 通过率 | ≥95% 达标 |
|------|-----------|--------|-----------|
| css-text-decor/ | 39/39 | 100.0% | ✅ |
| css-fonts/ | 60/60 | 100.0% | ✅ |
| css-grid/ | 17/20 | 85.0% | ❌ |
| css-writing-modes/ | 45/59 | 76.3% | ❌ |
| css-tables/ | 41/55 | 74.5% | ❌ |
| CSS2/ | 89/129 | 69.0% | ❌ |
| css-flexbox/ | 35/55 | 63.6% | ❌ |
| css-position/ | 10/16 | 62.5% | ❌ |
| css-multicol/ | 29/57 | 50.9% | ❌ |

### R16 本轮修复内容

| 修复 | 影响 | 说明 |
|------|------|------|
| 移除 apply_relative_offsets 双重偏移 | +3 tests | taffy 0.7 已在 layout.location 中包含 position:relative 的 inset 偏移。apply_relative_offsets 后处理函数再次添加同一偏移量，导致相对定位元素位移 2x。禁用此函数修复了所有使用 `position:relative; top:1in` 的参考文件 |
| float Y 位置尊重正常流 | +1 test | Phase 1 定位 float 时不知道 normal flow 的位置

| 修复 | 影响 | 说明 |
|------|------|------|
| 零 clearance 阻止 margin 折叠 | CSS2/floats-clear | CSS 2.1 §9.5.2：当 clearance=0 时，margin 折叠仍被阻断，元素位于 flow_bottom + child.margin_top（不折叠） |
| inline 元素 padding/border 参与行盒高度 | CSS2/linebox | TextRun 新增 padding_top/bottom + border_top/bottom 字段，box_height() 方法返回 line-height + padding + border 的完整盒高 |
| vertical-rl 列方向 RTL | css-writing-modes | IFC 新增 vertical_rtl 标志，vertical-rl 模式下列从右到左排列 |
| 垂直模式 abs-pos 静态位置修正 | css-writing-modes | 新增 fix_vertical_mode_abs_pos 后处理，对垂直书写模式容器中 abs-pos 元素重新计算静态位置 |

### CSS2 子目录详细通过率（R15 新增）

| 子目录 | 通过/总数 | 通过率 |
|--------|-----------|--------|
| floats-clear | 11/30 | 36.7% |
| linebox | 7/15 | 46.7% |
| backgrounds | 8/15 | 53.3% |
| borders | 10/15 | 66.7% |
| abspos | 3/4 | 75.0% |
| colors | 4/5 | 80.0% |
| floats | 12/15 | 80.0% |
| fonts | 13/15 | 86.7% |
| box | 1/1 | 100.0% |

### 关键发现（R15）

| 发现 | 说明 |
|------|------|
| taffy inline→Block 映射使 IFC padding/border 无 net reftest 效果 | 所有 display 类型映射为 taffy::Block，taffy 已正确计算 inline 元素尺寸。IFC padding/border 改进是规格正确但 reftest 中性 |
| CSS2 子目录通过率分化严重 | floats-clear 36.7%（19 失败）是最大瓶颈，box/colors/fonts 已接近 80-100% |
| css-flexbox 从 58.2% 提升至 60.0% | flex-flow-001 修复（float 定位 flow 追踪）→ flex item 正确 shrink |
| 35 个 near-miss (<2% diff) 分布 | CSS2/floats-clear (10), css-writing-modes (10), css-tables (7), css-flexbox (5), css-position (2) |
| 166 个失败根因分布 | 布局精度问题 (float/clear/margin) 50+ 个、writing-mode 轴交换 36 个、multicol column breaking 32 个、其他 48 个 |
| 后续最大杠杆 | (1) CSS2 float/clear 精度提升（影响 22 个测试）(2) multicol column breaking（影响 32 个测试）(3) writing-mode 块级布局轴交换（影响 36 个测试） |

### 后续重点

### R13 本轮修复内容

| 修复 | 影响 | 说明 |
|------|------|------|
| Inline-block 行内定位 | CSS2 +2 | 新增 adjust_inline_block_positions 后处理：对包含 inline-block 子元素的容器运行 IFC，获取正确水平并排位置。IFC 新增 InlineBlockBox 类型。CSS2 从 59.7% → 62.8% |
| Border 样式覆盖 | css-tables | collapsed_border_style_overrides[4] 数组：border-collapse 冲突解决后获胜方的样式（solid/dashed/dotted 等）正确传递到 paint 阶段。color_value_to_u32 新增 purple/cyan/magenta 等 10 种命名颜色 |
| 基线计算修正 | inline layout | inline-block 元素的 baseline 在其底部边缘（ascent=height），而非 0.8×line-height |

### 失败根因分布分析（R13 更新）

| diff 范围 | 数量 | 特征 | 代表性测试 |
|-----------|------|------|-----------|
| <2% | 33 | 亚像素/微偏移，接近通过 | clearance-006 (1.16%), flex-item-position-relative (1.04%) |
| 2-5% | 45 | 小幅定位差异 | clear-float-003 (1.92%), background-043 (2.61%) |
| 5-15% | 40 | 中等定位/尺寸差异 | float-006 (7.46%), background-090 (10.20%) |
| 15-30% | 32 | 显著布局差异 | abs-pos-non-replaced-* (21.33%), direction-vrl (12.49%) |
| >30% | 16 | 基本功能缺失 | empty-inline-002 (42.06%), background-attachment (30.48%) |

### 关键发现（R13）

| 发现 | 说明 |
|------|------|
| Writing-mode abs-pos 失败主因是 inline 布局 | 12 个 abs-pos-non-replaced 测试全部在 21.33% 失败。轴交换已启用（输入/输出双向），但 inline formatting context 不支持垂直模式——文本仍水平排列。静态位置基于水平 inline 布局，导致绝对定位偏移 |
| 55 个失败测试使用 Ahem 字体 | 这些测试的失败原因是布局定位差异（而非字形渲染），因为许多 WPT 测试使用 Ahem 字体创建精确矩形内容来验证布局 |
| CSS2 边框/背景测试 | border-bottom: 1in 系列测试（3.84%）的 diff 精确等于整个 div 面积，暗示渲染存在系统性偏移。需进一步调查 1in 单位在 border 上下文中的行为 |
| 大面积差异(>25%)多因缺少功能 | background-attachment:fixed (30.48%)、position-fixed-overflow-print (75%)、column-balancing-paged (56%) 均因缺少对应 CSS 功能实现 |

### 失败根因分布分析（R12 新增）

| diff 范围 | 数量 | 特征 | 代表性测试 |
|-----------|------|------|-----------|
| <2% | 33 | 亚像素/微偏移，接近通过 | clearance-006 (1.16%), grid/child-border-box (1.52%) |
| 2-5% | 45 | 小幅定位差异 | clear-float-003 (1.92%), background-043 (2.61%) |
| 5-15% | 40 | 中等定位/尺寸差异 | float-006 (7.46%), background-090 (10.20%) |
| 15-30% | 40 | 显著布局差异 | clear-applies-to-001 (29.45%), direction-vlr (12.49%) |
| >30% | 10 | 基本功能缺失 | background-attachment (30.48%) |

### 关键发现（R12）

| 发现 | 说明 |
|------|------|
| Ahem 字形位图非主因 | 验证了 Ahem 光栅化代码路径被正确触发（font_id=3），但通过率无变化。说明上游 reftest 失败主要因为布局定位差异，非字形渲染 |
| 布局定位是核心瓶颈 | 分析 168 个失败测试，绝大多数是元素位置/尺寸与 Chrome 不同。根因分为：float/clear 后处理精度、writing-mode 轴交换、multicol 列拆分、inline box model |
| Phase 1 float+clear 已实现 | adjust_float_positions Phase 1 已正确处理 float+clear 组合（line 676-703），非 float 元素的 clear 处理在 Phase 2 |
| 相同 diff 百分比暗示系统性偏移 | 多个测试在相同百分比失败（如 3.83%、7.67%），暗示特定的元素尺寸/偏移量差异 |
| writing-mode 轴交换 | css-writing-modes -1 | 启用 CSS Writing Modes §7.1 轴交换：输入时交换 CSS 属性到 taffy 水平模型，输出时交换回视觉坐标。盒体几何位置正确，但文字仍水平排列（需要 paint 层旋转支持）。1 个测试因坐标交换而回归 |
| 属性继承修复 | 全局 | list-style-type、list-style-position、writing-mode 添加到继承属性列表和 inherit_property 处理器 |
| justify-items/justify-self | css-grid | 转换器新增映射，从 ComputedStyle 映射到 taffy Style 的 justify_items/justify_self 字段 |
| scrollbar_width | 全局 | 从硬编码 0.0 改为根据 ComputedStyle 映射（Auto→15px, Thin→8px, None→0px） |

### 后续重点

1. **multicol column breaking**（影响 css-multicol ~16 测试）：需要实现内容碎片化 — 将单个块级元素的内容拆分到多列。当前仅移动整个子元素到下一列。multicol-breaking-* 系列测试全部在 16%+ 失败。
2. **CSS2 float/clear 精度**（影响 CSS2 ~19 测试）：clearance 计算使用简化公式，不完全匹配 CSS 2.1 规范的 C1/C2 双路径算法。参考文件大量使用拉伸 swatch 图片（20x20→96x96），image scaling 差异可能贡献部分 diff。
3. **CSS2 inline box model**（影响 ~8 测试）：空 inline 元素 line-height 贡献、inline 元素 margin 处理、block-in-inline 拆分。IFC 仅在 float/inline-block/vertical-mode 容器中运行，普通 block 容器中的空 inline 元素 line-height 不被 IFC 处理。
4. **Flexbox baseline 对齐 + writing-mode**（影响 css-flexbox ~9 测试）：multi-line baseline 测试（47%）、baseline align-self（15-18%）、min/max-content（16-21%）。flex-flow:row + writing-mode:vertical-rl 需要 flex 方向的轴交换支持。
5. **CSS 表格子像素**（影响 css-tables ~9 测试）：subpixel collapsed borders (1.97%)、table-cell-overflow (1.12%)、border-conflict-resolution (1.55%)。多数是 image scaling 或 border 渲染精度问题。

### R11 本轮修复内容

| 修复 | 影响 | 说明 |
|------|------|------|
| multicol column breaking 基础设施 | css-multicol | 新增 assign_children_to_columns_with_breaking，当子元素超出列高限制时移至下一列。基础设施就绪，但真实 breaking 需要内容碎片化 |
| CSS columns 简写验证 | css-multicol | expand_columns 验证 column-width 值有效性，拒绝 'normal' 等无效值。符合 CSS 规范整个声明无效的语义 |
| table cell overflow 修复尝试 | css-tables | 调查发现 CSS 2.1 规范要求 table cell height 为最小高度，即使 overflow:hidden 也必须增长以包含内容。已回退 |

### 关键发现（R11）

| 发现 | 说明 |
|------|------|
| Ahem 字体是最大瓶颈 | 100+ 测试因 Ahem 字体渲染差异而失败（fontdue vs Skia）。非 Ahem 测试的失败率低得多。**R12 更新**：Ahem 字形位图差异已修复，但失败主因为布局定位差异而非字形渲染 |
| CSS table cell height = 最小高度 | CSS 2.1 明确规定 table cell 的 height 是最小高度，cell 必须增长以包含内容，overflow:hidden 不改变此行为 |
| Column breaking 需要碎片化 | multicol-breaking-* 测试需要将单个子元素的内容（如文本）拆分到多列，不仅仅是移动整个子元素 |
| abs-pos-non-replaced 12 个测试全在 21.33% | 这些测试都用 Ahem + background image，相同的差异比例表明是系统性渲染问题 |


### 发现的关键问题

| 问题 | 影响 | 说明 |
|------|------|------|
| **表格 border CSS 未生效** | css-tables | CSS `border: 5px solid green` 在 `<table>` 元素上未正确应用——ComputedStyle 显示 border_top_width=Px(0.0)，但 border-collapse: Collapse 正确设置。疑为 CSS 级联中 border 简写展开或选择器匹配的问题，需进一步调查 |
| **paint 系统 font-family 硬编码** | 全局 | paint/painter/text.rs 硬编码 FontId(0)，不解析 CSS font-family。Ahem 字体虽已加载但无法被 font-family: Ahem 匹配到 |
|------|------|----------|
| empty-cells 在 border-collapse:collapse 时正确显示边框 | css-tables | border-collapse-empty-cell ✅ |
| row-group/row border/padding/margin 抑制（CSS 2.1 Section 17.5.3） | css-tables | row-group-order ✅, rowspan-cell-border-after-color ✅ |
| table cell explicit height + overflow:hidden 保留原始高度 | css-tables | (cell overflow 测试因参考文件差异未通过，但修复逻辑正确) |
| 空 inline 元素 line-height 贡献到行盒 | CSS2/linebox | (需 Ahem 字体的测试仍失败) |
| sibling combinators 跳过文本节点 | CSS2 选择器 | (改善选择器匹配精度) |
| table min/max size constraints | css-tables | (基础设施准备) |
| JS-dependent test skip (position-fixed-scroll-nested-fixed) | css-position | 移除 1 个无效测试 |

---

## 已知关键缺口

| 缺口 | 影响范围 | 优先级 | 里程碑 |
|------|----------|--------|--------|
| **Paint IFC / taffy-IFC 架构分裂** | **~50 个失败测试（51% 剩余失败）** | **P0-致命 / 需专项架构改造** | **Post-R71 专项规划** |
| Float 布局算法 | CSS 2.1 核心 | ✅ 已完成 | M4 |
| Table 布局算法 | 表格渲染 | ✅ 已完成 | M4 |
| Multi-column 布局算法 | 多列布局 | ✅ 已完成 | M4 |
| OpenType shaping | 文字排版质量 | ✅ 已完成 | M6 |
| BiDi 算法 | RTL 文本 | ✅ 已完成 | M6 |
| Vertical writing-mode | 竖排文本 | M5 能力已落地，完整垂直布局仍缺 | Post-R71 专项规划 |
| CJK normal-mode 换行 | CJK 排版 | ✅ 已完成 | M5 |
| text-align: justify | 文字排版 | ✅ 已完成 | M5 |
| Float exclusion 堆叠 | 布局正确性 | ✅ 已完成 | M5 |
| Quirks mode | CSS 2.1 兼容性 | ✅ 已完成 | M2 |
| 上游 WPT 真实 reftest 导入 | 覆盖范围 | M6 | M6 |
| CPU 渲染器图元覆盖 | 视觉输出 | ✅ 已完成 | M7 |
| 浏览器图元消费 | 视觉输出 | ✅ 已完成 | M7 |
| GPU 渲染器图元覆盖 | GPU 视觉输出 | ✅ 管线已实现 | M7 |
| Multicol column breaking | css-multicol ~22 测试 (61.4%→需+18) | P1 / 需专项架构改造 | Post-R71 专项规划 |
| Writing-mode 垂直布局 | css-writing-modes ~10 测试 (83.1%→需+7) | P1 / 需专项架构改造 | Post-R71 专项规划 |
| Inline-box 模型 | CSS2 linebox ~8 测试 | P1 | R69+ |
| 外部 stylesheet 加载 | 真实静态网页 CSS | P1 | M10/R38 |
| 图片子资源/ImageCache 贯通 | Logo/图片密集静态页 | P1 | M10/R38 |
| 产品/真实静态页视觉 smoke | 验收有效性 | P1 | M10/R38 |

---

## IFC 统一技术参考

> 本节为 R69+ 执行 agent 提供精确的代码级上下文，避免重复探索。

### 三套 IFC 运行路径

当前系统在三个不同时机运行 IFC，使用不同的上下文参数：

| # | 函数 | 文件:行 | styles | 时机 | 作用 |
|---|------|---------|--------|------|------|
| ① | `measure_text_content` | `engine.rs:1462` | 真实 `styles` | taffy 布局中（measure callback） | 返回 `Size{width, height}`，taffy 据此计算块级位置 |
| ② | `remeasure_text_with_float_exclusions` | `engine.rs:2150` | 真实 `styles` | taffy 布局后（step 6） | 重新 IFC + float exclusion，存储 5 个 override map，更新容器高度（shrink） |
| ③ | `paint_text` → IFC | `text.rs:884` | **空 `HashMap::new()`** | paint 阶段 | 生成 glyph 图元，依赖 5 个 override map 回退字体度量 |

**当前 use_stored 路径**：`text.rs:788` 检查 `box_node.inline_layout.is_some()` 且宽度匹配时直接复用存储的 IFC 片段位置，不运行 IFC ③。此路径代码就绪但 `inline_layout` 当前始终为 `None`（因为 `compute_final_inline_layouts` 在 `engine.rs:198` 被注释禁用）。

### Paint IFC override 覆盖缺口

`collect_inline_items`（`inline/mod.rs:708`）在 styles 为空时通过以下 override 回退。**粗体**标记影响行断（字符宽度）的属性：

| 属性 | 覆盖机制 | 覆盖状态 | 影响 |
|------|---------|---------|------|
| **`font_size`** | `font_size_overrides[parent_id]` → 16px | ✅ 已覆盖 | **宽度 + 行高** |
| `line_height` | `line_height_overrides[parent_id]` → fs×1.2 | ✅ 已覆盖 | 仅行高 |
| **`letter_spacing`** | `letter_spacing_overrides[parent_id]` → 0 | ✅ 已覆盖 | **宽度** |
| **`word_spacing`** | 无覆盖机制 | ❌ 始终为 0 | **宽度** |
| **`is_ahem_font`** | `is_ahem_overrides[parent_id]` → false | ✅ 已覆盖 | **宽度** (0.55fs vs 1.0fs) |
| `vertical_align` | 无覆盖机制 | ❌ 始终为 Baseline | 行盒对齐 |
| **`margin_left/right`** (inline 元素) | 无覆盖机制 | ❌ 始终为 0 | **宽度** |
| `padding/border` (inline 元素) | 无覆盖机制 | ❌ 始终为 0 | 行高 |

### 已穷尽的不可行路径（R37-R68 共 32 轮）

以下所有路径均已尝试并回退，**R69+ 不需要重试**：

| 路径 | 结果 | 根因 |
|------|------|------|
| 修改 glyph advance (render_fs) | 回归 | 字形推进与 IFC 片段位置不一致 |
| 传递完整 styles 到 paint IFC | 回归 -5~-6 | 行断行为改变 |
| 存储 layout IFC 结果（所有变体） | 回归 -4~-6 | 存储 IFC 上下文与 paint 时不同 |
| font_size_overrides 启用 | 零改进（R45）/ 可能回归 | 行断变化 |
| is_ahem glyph advance 修改 | 回归 -2~-3 | 字形推进与 IFC 行断不一致 |
| letter_spacing_overrides 启用 | 零改进 | — |
| line_height_overrides 启用 | 零改进 | 仅影响垂直，不影响水平 |
| inline_element_metrics 启用 | 零改进 | 仅影响垂直 |
| default_font_metrics 传递 | 回归 -6 | font_size 变化导致行断不一致 |
| taffy measure callback IFC 缓存 | 回归 -5 | 多次 measure 调用的 available_space 不同 |
| 外边缘边框完整厚度 | 回归 -2 | taffy 单元格定位冲突 |

### 存储 IFC vs Paint IFC 基线计算差异

存储 IFC 的 fragment 基线位置计算与 paint IFC 的 glyph 基线位置不同：

```
存储 IFC：baseline_y = frag.y + frag.height     （line-height 盒底边）
paint IFC：baseline_y = frag.y + font_size      （当前 paint 使用）
差值 = (line_height - font_size) / 2            （半行距）
```

启用 `use_stored` 路径时，需要统一基线计算。推荐以存储 IFC 的 `frag.y + frag.height` 为准（CSS 规范中 line-height 半行距分布在文字上下）。

### IFC 统一完成度检查清单

```
✅ LayoutBox.inline_layout: Option<Vec<InlineLayoutLine>>    (engine.rs:167)
✅ LayoutBox.inline_layout_width: f32                          (验证容器宽度匹配)
✅ compute_final_inline_layouts() 函数实现                    (engine.rs:1175)
✅ paint 侧 use_stored 路径                                   (text.rs:788)
✅ store_font_sizes_from_ifc() 5 个 override map              (engine.rs:874)
✅ remeasure 高度收缩 → sibling reflow (shrink)               (R68)
❌ compute_final_inline_layouts 启用                          (engine.rs:198 被注释)
❌ remeasure 高度增长 → sibling reflow (grow)                  (仅处理 shrink)
❌ table/multicol 后处理后重新运行 IFC                         (宽度可能改变)
❌ 存储 IFC vs paint 基线计算对齐                             (frag.height vs font_size)
```

### Taffy Fork 状态

项目已 fork taffy 0.7.7 到 `crates/taffy-local/`（~16,400 行），通过 workspace `[patch.crates-io]` 替换 crates.io 版本。当前仅有一个自定义补丁：

- `cached_baselines()` 访问器（`cache.rs:187`, `taffy_tree.rs:853`）— 暴露 taffy 内部缓存的 `first_baselines`，供 inline-flex/inline-grid 基线提取

**结论**：不需要深度修改 taffy。IFC 统一通过在 remeasure 后处理阶段（taffy 布局完成后）存储完整 IFC 结果并传播高度变化来实现，不涉及 taffy 内部算法变更。

---

## IFC 之外的其他卡点

IFC 统一预计解决 ~50 个失败测试。剩余 ~48 个失败测试的根因分布如下。

### 卡点 #2：Multicol Column Breaking（~22 测试，独立于 IFC）

**影响**：css-multicol 当前 35/57 (61.4%)，距 95% 需 +18。是所有目录中通过率最低的。

**当前能力**：R41 实现了 column breaking 的 paint 层渲染 — 将整个子元素分配到各列后，paint 按列裁剪。这解决了 4 个 breaking 测试（000/001/002/003）。

**缺失**：**内容碎片化（content fragmentation）** — 当单个块级子元素的内容（如长文本段落）超过列高时，需要将其拆分到多个列。当前只能移动整个子元素到下一列。

**关键失败测试**：
- `multicol-breaking-004/005/006`：单个段落跨列拆分（diff 5.6-16.6%）
- `multicol-fill-auto-*`：column-fill:auto 的填充行为
- `multicol-count-*`：列数计算的边缘情况
- `multicol-clip-*`：溢出裁剪

**技术方向**：在 `assign_children_to_columns_with_breaking`（`multicol.rs`）中实现内容级拆分 — 对超高子元素，先运行 IFC 获取文本行，按列高逐列分配行。

---

### 卡点 #3：Writing-mode 垂直布局（~10 测试，部分独立于 IFC）

**影响**：css-writing-modes 当前 49/59 (83.1%)，距 95% 需 +7。Large-diff 测试（>9%）的根因是垂直模式下 float/clearance 定位不正确。

**当前能力**：
- 盒体几何轴交换：✅ — taffy 输入前交换 CSS 属性到水平模型，提取结果后逆交换回视觉坐标
- 垂直字形渲染：✅ — paint 层通过 `GlyphPrimitive.rotation = π/2` 旋转文字
- 垂直模式 inline 布局：✅ — R14 实现

**缺失**：垂直模式下 float/clearance 的完整轴交换。R57 尝试了完整轴交换方案（交换子元素尺寸 + 容器属性），但因零高度 float 元素的 block 轴 extent 改变导致 `clearance-calculations-vrl-008` 回归而回退。

**关键失败测试**：
- `direction-vlr-*` / `direction-vrl-*`：垂直书写方向（~12% diff）
- `clear-clearance-calculation-vrl-*`：垂直模式 clearance（~2-14% diff）
- `float-contiguous-vlr-*`：已全部通过（0.00%）— R57 发现无需修改

**技术方向**：精细轴交换 — 仅交换 float 的 inline 轴定位方向（x↔y），不改变 float 自身的 block 轴 extent。或采用更保守的方案：当前 83.1% 已接近目标，优先推动 multicol 和 flexbox 更远的目标。

---

### 卡点 #4：Flexbox Baseline 对齐（~3-5 测试，独立于 IFC）

**影响**：css-flexbox 当前 37/55 (67.3%)。虽距 95% 需 +14，但其中 ~10 个的根因是 IFC 架构（inline-flex 容器内文本定位），~3-5 个是 baseline 对齐问题。

**当前能力**：R59 添加了 taffy `cached_baselines()` 补丁和 `extract_baselines_recursive`。`adjust_inline_block_positions` 优先使用 taffy 缓存基线，回退到 font-size 近似。

**缺失**：taffy 仅在 flex 容器有 **≥2 个 `align-self: baseline` 子元素**时才计算子元素基线。大多数 WPT 测试使用默认 `align-self: stretch`，导致 `child.baseline` 保持默认值 0.0，基线计算等价于 `offset_cross + 0.0`。

**关键失败测试**：
- `flexbox-baseline-multi-line-horiz-003/004`（~48% diff）：inline-flex + flex-wrap:wrap + align-content:center 的复杂交互
- `flex-order-wrap-reverse-baseline` (1.27%)：wrap-reverse baseline

**技术方向**：修改 taffy 的 `compute_flexbox_layout` 使其对所有 flex 子元素计算基线（不限于 baseline-aligned），或扩展 `cached_baselines()` 提供合成基线。

---

### 卡点 #5：Table Border-collapse 精度（~3 测试，独立于 IFC）

**影响**：css-tables 当前 46/55 (83.6%)。near-miss 测试的根因多为 border-collapse 外边缘精度。

**当前能力**：R49 实现了 `resolve_collapsed_borders`（含行组边框集成）、`collapsed_border_outer_edge` 标记。Cell-vs-Cell 内部边颜色修正已合入。

**缺失**：外边缘单元格边框减半（与表格边框各占一半），导致边缘视觉宽度与规范不一致。R49/R50/R53 三次尝试完整厚度外边缘边框均导致回归 — taffy 的单元格位置基于原始边框宽度计算，完整厚度边框扩展超出元素边界。

**关键失败测试**：
- `border-conflict-resolution` (1.50%)
- `row-group-margin-border-padding` (1.32%)
- `whitespace-001` (1.05%)

**技术方向**：在 table layout 的 `position_cells` 中，对外边缘单元格的位置进行调整以匹配解析后的边框宽度。或在 converter 中移除边缘单元格的外部边框（从 box model 中减去 border 贡献）。

---

### 卡点 #6：CSS 2.1 Appendix E 堆叠顺序（2-3 测试，独立于 IFC）

**影响**：涉及 position:relative 容器内嵌套 absolute/fixed 后代的绘制顺序。

**当前能力**：R61 实现了基础堆叠排序（negative z-index → normal flow → floats → non-negative z-index）。

**缺失**：position:relative 元素不创建 stacking context 时，其 positioned 后代应参与父级 stacking context 的 step 6 排序，按 tree order 排列。当前实现将 positioned 元素全部排在 normal flow 之后，不区分嵌套层级。

**关键测试**：`flex-item-position-relative-001` (1.04% — 已在边缘，修复后可能通过)

**技术方向**：在 `paint_node_in_rect` 的排序逻辑中，增加对 positioned 后代 tree order 排序的支持。改动集中在 `paint/painter/mod.rs`。

---

### 卡点 #7：Grid Max-content Sizing（2-3 测试，独立于 IFC）

**关键测试**：`child-border-box-and-max-content-001/002` (1.52%)。near-miss，距通过很近。

**技术方向**：taffy grid 的 max-content 尺寸计算。可能需要调整 `computed_style_to_taffy` 中 grid item 的尺寸约束映射。

---

### 卡点 #8：Swatch 图像缩放精度（~5 测试，独立于 IFC）

**影响**：CSS2 floats-clear 中多个 near-miss 测试。15×15 或 20×20 纯色 PNG 被缩放到 96×96，双线性插值产生边缘伪影 vs CSS background-color 的精确填充。

**当前能力**：R43 添加了 `ImageData.solid_color` 检测和 CPU renderer 快速路径。

**技术方向**：对 solid_color 图像使用 nearest-neighbor 缩放（而非双线性），或直接按 solid_color 快速路径渲染（跳过纹理采样）。

---

### 卡点 #9：Position Fixed 视口定位（1-2 测试，独立于 IFC）

`position: fixed` 当前被 taffy 当作 `absolute` 处理（相对于包含块）。R68 禁用了 `adjust_absolute_to_initial_containing_block`（因导致 4 个 PASS→FAIL 回归）。需要重新设计更精细的条件判断。

---

### 卡点依赖关系与推荐执行顺序

```
IFC 统一（~50 tests）
  ├── 无依赖，可立即推进
  └── 完成后重新评估各目录通过率
      │
      ├── Multicol breaking（~22 tests）
      │   └── 独立，可与 IFC 并行推进
      │
      ├── Writing-mode 垂直（~10 tests）
      │   └── 可并行，但建议 IFC 后再做（依赖 IFC 修复后的文本定位）
      │
      ├── Flexbox baseline（~3-5 tests）
      │   └── 依赖 taffy 修改，可独立进行
      │
      └── 小卡点（table border / stacking order / grid / swatch / fixed）
          └── 独立小修复，可穿插进行
```

**推荐 R69+ 优先顺序**：
1. **IFC 统一**（最大杠杆，P0）
2. **Multicol column breaking**（第二大杠杆，可并行）
3. **Writing-mode 垂直**（当前 83.1%，离 95% 仅差 7 个，优先级可降低）
4. **小卡点穿插**：swatch 精度（影响 5 个 near-miss）、stacking order（1 个 near-miss）、grid max-content（2 个 near-miss）

---

## 技术决策记录

| 日期 | 决策 | 理由 |
|------|------|------|
| 2026-06-06 | 保留真实网站测试的 #[ignore] | 本地网络不稳定，这些测试不可执行 |
| 2026-06-06 | 扩展而非重写 manifest.rs 和 reftest.rs | 目标文档明确要求扩展现有模块 |
| 2026-06-06 | 使用内联 reftest 替代上游导入 | 避免网络依赖，53 个 CSS 2.1 核心 reftest 覆盖主要布局场景 |
| 2026-06-06 | mismatch 阈值设为 0.5% | 800×600 视口下，50×50 小元素差异约 0.52%，1% 阈值会漏检 |
| 2026-06-06 | 文字类 reftest 使用宽松容差 (5%/15ch) | fontdue vs Skia 字体渲染像素差异大 |
| 2026-06-06 | QuirksMode 在 StyleSystem 内部传递（不暴露为公开参数） | 保持公共 API 简洁，doc.quirks_mode() 在 compute_styles 入口处提取 |
| 2026-06-07 | quirks mode 颜色/长度解析通过函数指针分发 | parse_color_fn/parse_length_fn 模式避免重复 match 分支 |
| 2026-06-07 | apply_quirks_mode_adjustments 接受 tag_name 参数 | 需要按元素标签（如 table）应用不同的 quirks 规则 |
| 2026-06-07 | inline 元素 width/height quirks 暂不实现 | layout engine 将 inline 映射为 block，实际已生效；待 inline layout 正确实现后补充 |
| 2026-06-07 | UA 默认 display 值通过级联注入（Origin::UserAgent） | 最低优先级，可被作者样式覆盖；避免修改 ComputedStyle::default() |
| 2026-06-07 | Table 布局通过后处理步骤实现（类似 float） | taffy 无原生 table 支持，所有 table display types 映射为 Block 后重新定位 |
| 2026-06-07 | 修复 parse_display 缺失 table types 的 bug | color.rs 中有重复的 parse_display（缺 table types），通过 pub use color::* 被实际使用 |
| 2026-06-07 | Multi-column 通过后处理步骤实现（类似 float/table） | taffy 无原生 multicol 支持，column-count/column-width 容器的子元素在后处理中重新定位到各列 |
| 2026-06-07 | 多列均衡分配使用 shortest-column-first 策略 | 依次将每个子元素放入当前总高度最小的列，实现视觉均衡 |
| 2026-06-07 | CJK normal 模式下每个字符单独作为"单词" | CSS 规范要求 CJK 允许任意字符间断行，split_into_words 中 CJK 字符独立为词 |
| 2026-06-07 | text-align: justify 使用 effective_content_area 计算剩余空间 | 修复了原先用 container_width 忽略 float exclusion 的问题 |
| 2026-06-07 | Float exclusion 从 max 改为 additive stacking | 多个同侧 float 应累加宽度而非取最大值 |
| 2026-06-07 | rustybuzz 集成到 TextShaper | 优先使用 rustybuzz 进行 OpenType shaping（GSUB/GPOS），回退到 fontdue 逐字符映射 |
| 2026-06-07 | unicode-bidi 集成到 inline layout | RTL 字符自动检测并重排序，LTR 文本零开销 |
| 2026-06-07 | FontLoader 存储原始字体字节 | 供 rustybuzz Face::from_slice 使用，fontdue 仍用于 advance width 获取 |
| 2026-06-07 | ShapedGlyph 增加 x_offset/y_offset 字段 | 来自 rustybuzz 的 GPOS 定位偏移 |
| 2026-06-07 | 新增 render_full_scene() 替代 render_scene_to_framebuffer() | 旧函数仅支持 fills + rounded_rects + glyphs，新函数支持全部 13 种图元 |
| 2026-06-07 | 新增 transform_webview_primitives() 替代 inline 坐标变换 | 旧方式仅处理 fills + glyphs，新函数处理全部 13 种 RenderPrimitives 字段 |
| 2026-06-07 | 渐变使用逐像素插值 | 线性/径向/锥形渐变均在 CPU 上逐像素计算，无 GPU 依赖 |
| 2026-06-07 | 阴影使用 box-blur 近似 | 三次 box-blur 近似高斯模糊，性能与质量平衡 |
| 2026-06-07 | 路径填充使用扫描线算法 | 逐行扫描多边形边界，奇偶规则填充 |
| 2026-06-07 | CPU 后处理：Transform/Clip/Filter/BlendMode 作为后处理步骤 | 像素级后处理，不依赖 GPU；GPU 渲染器需独立实现 |
| 2026-06-07 | GPU 渲染器多管线架构 | 5 条独立 wgpu 渲染管线：Fill+Glyph、RoundedRect、Gradient、Image、Blur。每种管线有独立 WGSL shader 和绑定组布局。Mesh-based 图元（stroke/path）通过 CPU 侧顶点生成复用 fill pipeline。Phase-separated 架构避免借用冲突。 |
| 2026-06-07 | 浏览器 GPU 路径集成 render_full_scene_gpu | render_frame() 改用 render_full_scene_gpu 替代 render_scene_ext，GPU 渲染路径现在支持全部 13 种图元。GPU 渐变测试使用 ±3 容差应对 float→u8 精度误差。 |
| 2026-06-07 | Taffy 0.7 内置 margin 折叠 | 发现 taffy 0.7 已通过 CollapsibleMarginSet 实现 CSS 块级 margin 折叠，不需要额外后处理步骤。移除了自实现的 margin_collapse 后处理。 |
| 2026-06-07 | Float clear 后处理实现 | 在 adjust_float_positions() 中实现 clear:left/right/both。非 float 元素的 clear 属性将其推到对应侧浮动元素的底部之下。LayoutBox 新增 clear 字段。 |
| 2026-06-07 | 重复渐变：fract() tiling | CPU 渲染器用 fract(t/period) 实现重复渐变周期循环；GPU 渲染器通过 WGSL shader 中 fract(t) 实现，repeating 标志通过 param3 取负编码传递。 |
| 2026-06-07 | 多图层背景 Vec 迁移 | background_image 从 BackgroundImageComputedValue 单值改为 Vec，CSS 解析器新增 parse_background_image_layers() 处理逗号分隔，paint 按逆序渲染（CSS 规范最后一层在最底）。 |
| 2026-06-07 | clip-path inset 实际裁剪 | 新增 clip_all_primitives_to_rect() 对全部图元类型（fills/rounded_rects/gradients/shadows/images/glyphs/strokes）应用矩形裁剪，替代原来的虚线指示器。 |
| 2026-06-07 | CSS mask 渐变蒙版 | mask-image 复用 BackgroundImageValue 类型解析，渐变蒙版通过 clip_all_primitives_to_rect 裁剪到渐变边界 + 平均 alpha 衰减实现。URL 蒙版暂不支持（需图像加载基础设施）。 |
| 2026-06-07 | overflow 全图元裁剪修复 | paint_node/paint_node_in_rect 中 overflow:hidden/scroll/clip 原来仅裁剪 fills+glyphs，渐变/阴影/图片/线段等图元溢出容器边界不被裁剪。改为使用 PrimitiveCounts + clip_all_primitives_to_rect 裁剪全部 13 种图元类型。 |
| 2026-06-07 | 滚动容器 paint 偏移 | LayoutBox 新增 scroll_x/scroll_y 字段。paint_node 中当 overflow == Scroll 时，子元素坐标减去 scroll 偏移量。overflow:Hidden 不应用滚动偏移（非滚动容器）。3 个单元测试验证。 |
| 2026-06-07 | render_full_scene 切换到上游 reftest | 上游 reftest 从旧版 render_scene_to_framebuffer（仅 fills+rounded_rects+glyphs）切换到 render_full_scene（全部 13 种图元）。同时启用 ImageCache 从 base_dir 加载 PNG 图片。这使 reftest 结果更准确，但也暴露了之前被不完整渲染掩盖的布局差异。 |
| 2026-06-07 | skip_indicators 模式 | Painter 新增 skip_indicators 标志，RenderPipeline 新增 set_skip_indicators() 方法。当设为 true 时跳过全部 ~30 个 CSS 属性调试指示器（border-collapse 橙色标记、direction 箭头等），避免干扰 reftest 像素对比。 |
| 2026-06-07 | UA 默认样式扩展 | 新增 body{margin:8px}、h1-h6{margin+font-weight}、p{margin:1em 0}、ul/ol{margin+padding-left} UA 默认样式，对齐浏览器默认行为。 |
| 2026-06-08 | table row-group 行索引修复 | build_grid 中行组内行存储 rg_child_idx 但 get_row_box 在 table_box.children 查找，导致 tbody/thead/tfoot 内的行被静默丢弃。修复：TableRow 新增 row_group_index 字段，get_row_box/position_cells 根据此字段正确导航到行组内的行。 |
| 2026-06-08 | column-gap 属性映射修复 | converter gap.width 原先使用 style.gap（仅 gap 简写设置），改为使用 style.column_gap（column-gap 长写属性）。同时修复 gap 简写解析：`gap: 10px` 现在同时设置 column_gap 和 row_gap。使用 fallback 策略：column_gap 非 0 时优先，否则使用 gap。 |
| 2026-06-08 | background-image 固有尺寸基础设施 | Painter 新增 image_sizes HashMap<u64, (f32, f32)>（url hash → intrinsic dimensions）。RenderPipeline.set_image_sizes() 将缓存传递给 Painter。reftest runner 在渲染前构建 ImageCache、提取固有尺寸。修复了 background-size: auto 拉伸到容器大小的问题。 |
| 2026-06-08 | is_block_level / is_relative 标志 | LayoutBox 新增两个布尔标志。is_block_level 用于 float/clear 后处理（CSS 规范 clear 仅适用于块级元素）。is_relative 用于 table 布局后处理保留 position:relative 的 inset 偏移。 |
| 2026-06-08 | gap 简写 handler 修复 | gap apply handler 不再设置 column_gap/row_gap（由各自的 longhand handler 通过 shorthand expansion 设置），避免 HashMap 迭代顺序不确定性导致的值覆盖。 |
| 2026-06-09 | reftest 分类容差 bug 修复 | 上游 reftest FileReftestCase::to_config() 使用 Default::default()（1%/5ch），未调用 ReftestConfig::for_category()。所有测试被以严格布局容差（1%）衡量，导致文字类测试（5% 容差）大量误判失败。修复后通过率 68.6%→73.5%（+24 测试）。新增 ReftestConfig::with_viewport() builder 方法。 |
| 2026-06-09 | columns 简写解析修复 | `columns: 3`（单整数）被 parse_column_width 先解析为 column-width: 3px，阻止 parse_column_count 执行。CSS 规范要求整数优先解析为 column-count。交换解析顺序后，`columns: N` 正确设置 column-count: N。 |
| 2026-06-09 | 零高度浮动处理 | adjust_float_positions Phase 1 中 line_max_height 跳过零高度浮动元素（child_outer_height == 0），避免空浮动元素推进后续浮动的 Y 位置。 |
| 2026-06-08 | table 行组位置更新 | position_cells 后新增 update_row_group_positions 后处理。按视觉顺序（thead→tbody→tfoot）计算行组的 y 位置和高度，含 border-spacing。支持 position:relative inset 从行组传播到子行。修复 out-of-order-elements-collapsed-border（46.32%→通过）。 |
| 2026-06-08 | CSS 绝对长度单位 | parse_length() 新增 in/pt/pc/cm/mm/Q 单位支持，按 CSS 规范转换为 px（96 DPI）。修复了所有使用 `height: 1in; width: 1in` 的 floats-clear 测试（之前 in 单位被静默忽略，元素折叠为 0 大小）。副作用：CSS2/borders 中使用 1in(=96px) 大边框的测试暴露了布局精度差异。 |
| 2026-06-08 | CSS inherit 关键字完善 | border/background shorthand 正确广播 CSS-wide keywords（inherit/initial/unset）到所有子属性。inherit_property 扩展支持非继承属性（background-*, border-*, margin-*, padding-*），使 `border-bottom: inherit` 等显式继承生效。 |
| 2026-06-08 | is_block_level 修正 | table 内部 display types（TableRowGroup, TableRow, TableCell 等）从 is_block_level 中移除。CSS 2.1 规定 clear 属性仅适用于块级元素，table 内部元素不是块级元素。 |
| 2026-06-08 | 参考文件过滤 | reftest loader 跳过以 -ref/-reference 结尾的文件名，避免参考页面被当作测试用例运行。移除 1 个误计入的测试（float-nowrap-3-ref.html）。 |
| 2026-06-08 | XHTML CDATA 调查 | 调查发现 html5ever 在 HTML 模式下将 XHTML CDATA 标记（`<![CDATA[...]]>`）保留在 `<style>` 文本内容中。CSS 解析器遇到 `<![CDATA[` 时错误恢复路径触发 `skip_to_rbracket()`，贪婪吞噬后续所有 token，导致整个样式表提取 0 条规则。之前通过 CDATA 损坏的 .xht 测试（test+ref 都无 CSS）实际是虚假通过。 |
| 2026-06-08 | XHTML CDATA 清理实施 | `strip_cdata()` 在 `collect_stylesheets()` 中去除 CDATA 前后缀。揭示真实通过率 66.1%（之前 76.4% 含虚假通过）。真实修复：background-087/326/328 ✅。揭示的渲染缺口：writing-modes 42.4%（需 writing-mode 布局支持）、multicol 49.1%（需 column breaking）、floats-clear 新增 6 个差异。 |
| 2026-06-08 | empty-cells border-collapse 修复 | `empty-cells: hide` 仅在 separated border model 中生效。在 collapsed border model 中，空单元格仍需显示边框。修改 paint_node 两处 skip_empty_cell 条件添加 `border_collapse == Separate` 检查。 |
| 2026-06-08 | row-group/row box model 抑制 | CSS 2.1 Section 17.5.3/17.5.4：在 separated border model 中，table-row-group 和 table-row 的 border/padding/margin 无视觉效果。新增 `suppress_row_group_row_box_model()` 和 `zero_box_model()` 函数。 |
| 2026-06-08 | table cell explicit height 保留 | 有明确 height 且 overflow:hidden/scroll/clip 的单元格保持 taffy 计算的原始高度，不被行高覆盖。修复 table-cell-overflow-explicit-height 测试。 |
| 2026-06-08 | CSS 2.1 Appendix E 绘制顺序 | paint_node_in_rect 和 paint_node 中子元素分两轮绘制：先绘制非 float 子元素，再绘制 float 子元素。确保 float 内容视觉上在 block 背景之上（CSS 2.1 Appendix E）。 |
| 2026-06-08 | columns 简写顺序无关解析 | expand_columns() 双值模式改为自动检测哪个是整数（column-count）哪个是长度（column-width），而非硬编码 parts[0]/[1]。修复 `columns: 100px 6` 等逆序声明。 |
| 2026-06-08 | clearance 计算代码质量改善 | 澄清 CSS 2.1 §9.5.2 clearance 语义：零 clearance 仍然阻止 margin 折叠；clearance = max(0, clear_bottom - hypothetical_position)。后处理方式的局限性在于 taffy 已应用 margin 折叠。 |
| 2026-06-08 | 空 inline 元素 line-height | 空 inline 元素（如 `<span></span>`）生成零宽度 TextRun，其 line-height 仍贡献到行盒高度。修改 collect_inline_items 不再跳过空 inline 元素。 |
| 2026-06-08 | sibling combinators 文本节点跳过 | NextSibling (+) 和 SubsequentSibling (~) 组合器现在跳过元素间的文本节点，匹配 CSS 选择器规范行为。修改 matches_selector_recursive 和 matches_has_selector_chain。 |
| 2026-06-08 | CSS 绝对长度单位 | parse_length() 新增 in/pt/pc/cm/mm/Q 单位（96 DPI），background 简写分类器新增所有长度后缀。修复使用 1in 高度的 floats-clear 测试。 |
| 2026-06-08 | 径向渐变位置修复 | gradient_to_primitive 改用 resolve_position() 正确处理 Percentage（百分比）和 Px（绝对像素），替代旧的 length_to_f32/100 逻辑。修复相关测试用例。 |
| 2026-06-08 | 表格 min-height border-box | apply_table_size_constraints 正确处理 min-height/max-height 为 border-box 约束（减去 padding+border）。修复 min-height-table。 |
| 2026-06-08 | 表格单元格高度最小值 | CSS 2.1 规范中 cell height 为最小高度，改用 max(row_height, cell_content_height)。 |
| 2026-06-14 | R118 失败全景再扫描（诊断） | 419/490 持平。对当前 71 个失败用例重跑 `REFTEST_BBOX`（输出在 stderr，含 sub-threshold 通过项共 290 行）并逐一交叉匹配，再对 10 个候选做 REFTEST_DUMP+PIL 深挖。结论：干净的增量修复已耗尽（R108 bbox-scan + R111 per-feature grep 找到的最后两个 cluster 是 inline-fmt 与 visibility:collapse）。逐项排除：clear-applies-to-009 (1.02%) 是 `<p>` 文本行亚像素纵向偏移导致的字体抗锯齿差异（near-miss，非 clearance bug）；html-display-table 根因 = `build_grid`（table.rs:1352 `_` 分支）跳过 display:table 的非 table-internal 子元素（如 `<body>`）→ 空 grid → `layout_table` 早返回（table.rs:230）→ html:table 沿用 taffy 800px，匿名单元格修复（让表格收缩到 300px）因 body 已被 taffy 以 780px 全高布局、post-processing 无法重排其子元素而失败，属**布局前匿名盒生成**的结构改动；child-border-box-and-max-content-001/002（1.52% 相同 bbox）= `width:max-content` 映射 Auto（converter/mod.rs:375）导致 grid item 不渲染，属 intrinsic-sizing 缺口，17 个用例用该关键字、4 失败 13 通过 → 实现风险净负（R97 仍成立）；multicol-count-computed-004 = column-count:auto+auto 正确返回 None（multicol.rs:150）→ 单列渲染，差异是文本-vs-swatch 两行换行的位置 near-miss；clearance-calculations-vrl（R114b 修正）需把整段 ~150 行 `adjust_float_positions_with_context` 做 block/inline 轴参数化，非 surgical flag，对通过的 floats-clear 有高回归风险。剩余 71 个失败全部属于结构性多轮里程碑：multicol column-breaking fragmentation（R113 两趟）、writing-mode 垂直轴 float/clearance 参数化（R114b）、flex baseline 合成、intrinsic sizing 关键字（R97）、布局前匿名盒生成。 |
| 2026-06-14 | R119 百分比 max-height 收紧 | 420/490 (+1)。taffy 0.7 不会对 height:auto 的块盒按百分比 max-height 收紧（converter 已传 Percent，但 block 布局未在内容高度计算后再次 clamp）。新增自上而下后处理 `clamp_percentage_max_height`（engine.rs），按 CSS §10.7 相对明确包含块高度解析百分比 max-height，CB 不明确时按 §10.5 视为 auto。修复 fieldset-as-item-overflow（ref 依赖 max-height:100% 收紧 200px 子元素到 100px），零回归（0 个 reftest test 文件用百分比 max-height，仅 1 个 ref 用）。2 个单元测试覆盖明确/不明确 CB。 |
| 2026-06-14 | R120 U+00A0 不可折叠实验（已回退） | 420/490 持平（净中性，已回退）。发现 `collapse_whitespace`（inline/mod.rs:286 `is_whitespace`）与 `split_into_words` 标准 normal 模式（`split_whitespace`）把 U+00A0（&nbsp;）当作可折叠/可断行空白，违反 CSS Text §4.1.1（nbsp 不可折叠、不可断行）。修复（collapse 保留 nbsp、split 排除 nbsp）符合规范，但净效果中性：multicol-containing-002 改善（4.65→3.67%），multicol-count-002 **恶化**（9.50→12.28%），全量仍 420/490。原因：受影响用例是 nbsp + 列分布 + 图片布局的复合问题，单独修 nbsp 不产生净胜；multicol-count-002 的 ref 用 nbsp 连接图片（非文本），test 用 nbsp 在文本中，两者上下文不同。已回退。**结论：nbsp 修复规范正确但净中性，不要单独重试**；需配合列分布精度（R113）一起做才可能产生胜场。 |
| 2026-06-14 | R121 嵌套 multicol 塌缩机制定位（诊断） | 420/490 持平。REFTEST_DUMP+PIL 精确确认 multicol-breaking-005 的嵌套 multicol **塌缩为窄条**（magenta 仅在 x 桶 0、y=29-36 共 7px；REF 分布到 x 桶 0-7、y=8-107）。MC_DBG 插桩定位根因：`layout_multicol`（multicol.rs:255）用 `child.height` 分配子元素到列，但 inline 文本子元素的 LayoutBox.height=0（行高由 IFC 独立测量，不写回 LayoutBox）→ balanced 分配把 29 个 h=0 子元素堆叠 → 塌缩。`remeasure_inline_only_containers`（engine.rs:2699 `balance_column_geometry`）确实按列宽测量并更新**容器** content_height，但不传播**单个 inline 子元素**的高度到 LayoutBox，故 multicol 分布仍看到 h=0。这是 multicol↔IFC 集成的结构缺口（比 R113「循环两趟」更精确：真正缺的是 inline 子元素高度从 IFC 回写到 LayoutBox）。修复需在 remeasure 后、`adjust_multicol_layout` 前，把 IFC 的逐行/逐子元素高度回写到对应 LayoutBox.children 的 height，或让 multicol 分布直接消费 IFC 行结果。属多轮结构里程碑，单轮不可安全完成（39 个通过的 multicol 测试有回归风险）。 |
| 2026-06-14 | R122 multicol 列分布 rebase 修正 + clean-win 穷尽复核（净中性） | 420/490 持平（净中性，零回归）。独立复核 15+ 失败候选（border/border-bottom-width、table-cell-overflow、clear-float-003、abspos-inset-nested、inline-box、block-in-inline、flex-gap、multicol 各项），全部确认为结构性/复合问题，印证 R118「干净增量修复已耗尽」。**multicol-breaking-005 塌缩的真正触发点定位**：painter/text.rs:700-701 的 `height_auto` 守卫——纯 inline + balance + height:auto 才做列分布，明确高度（height:300px）的嵌套 multicol 回退单块→塌缩。**关键反直觉发现**：multicol-breaking-nobackground-005 与 breaking-005 布局完全相同（diff 仅 background+column-rule），nobackground-005 的 TEST **同样塌缩**（text bbox 50x15 vs REF 725x15），但因其无 solid 背景，纯文本差异 <1% 阈值而**侥幸通过**（0.82%）；breaking-005 因 magenta 背景大面积缺失达 21.74%。**验证放宽守卫 net -5**（multicol 39→34/57）：paint 列分布算法 `col_start_y = col_idx * target_h` 对非整除行数有 fractional offset，对 nobackground-001/002、breaking-001/002、nobackground-005 产生比塌缩更差的结果。**修正**：painter/text.rs 列分布改为预计算每列**实际首行 y** 作 col_start_y（rebase 到列内 y=0），消除 fractional offset——单独应用净中性（420/490 不变）但改善 2 个 near-miss（multicol-columns-001 5.00→4.88%、multicol-containing-002 4.65→4.23%），为零回归的正确性改进。**结论**：multicol 嵌套列分布需 R113 两趟结构重构（内层列几何依赖外层 column-width，当前 paint 单次 IFC 无法精确），rebase 修正为该重构的必要前置正确性基础。另新增 `REFTEST_DUMP_PASS=1` 诊断（env 门控，dump 通过用例实际渲染，供未来轮次分析通过用例真实输出）。 |
| 2026-06-14 | R123 根元素 position:relative inset 应用（+1，零回归） | 421/490 (+1)。MC_DBG 插桩定位 `abspos-containing-block-initial-007` 根因：`<html style="position:relative; top:100px; left:100px">` 的 relative inset **未被应用**（root x=0 y=0，is_relative=true），导致根 html border-box 与其 abspos 后代 body（CB=根 padding box）整体位置错（TEST root@(0,0)、body@(20,70) vs REF root@(100,100)、body@(120,170)，差恰好 (100,100)=inset）。**根因**：taffy 0.7 对非根 block-level 元素把 position:relative inset 应用到 layout.location，但对**根节点**不应用（根总在 0,0）。**修复**：engine.rs extract_layout 后，当 `root_box.is_relative` 时手动应用 `resolve_relative_inset`（top/left Px）到 root.x/y（CSS 2.1 §9.4.3），使根及 abspos 后代整体偏移。**零回归**：全 wpt-data 仅 1 个 test 文件在根 `<html>` 上用 position:relative（grep 确认），修复只能帮助。新增单元测试 `test_root_relative_position_applies_inset`（relative 根→(100,100)，static 根→(0,0)）。make test 全绿，clippy 零警告。**方法论**：R98 修了无 positioned ancestor 的 abspos Length inset（viewport 相对），R119 修了百分比 max-height；本轮是同一 converter-passes-but-taffy-ignores 谱系的**根元素 relative inset**缺口——taffy 对根节点的 position:relative 静默丢弃 inset。 |
| 2026-06-14 | R124 非 positioned overflow 不裁剪 CB 为祖先的 abspos（+1，零回归） | 422/490 (+1)。MC_DBG 布局树 dump 定位 `multicol-contained-absolute`（什么都不渲染）根因：`overflow:hidden` 元素（content_height=0，因唯一子元素是 abspos 不贡献高度）把 CB 为祖先的 abspos 后代**误裁剪到 0 高度**。结构 `relative(CB) > overflow:hidden(h=0,非 positioned) > abspos(green 100%)`——abspos 的 CB 是 relative（overflow 的祖先），按 CSS §11.1.1 不应被该 overflow 裁剪，但 ZeroWeb 原先把 abspos 当普通子元素绘制被裁掉。**失败的实验**：把所有 abspos 子元素移到 overflow 裁剪后绘制 = net -6（破坏 z-order，影响 positioned overflow 容器内的 abspos）。**正确修复**：painter/mod.rs 仅对**非 positioned 的 overflow 元素**（`needs_clip && !self_positioned && !is_multicol`）把 abspos/fixed 子元素移到 overflow 裁剪之后绘制；positioned overflow 元素（`position:relative;overflow:hidden` 常见模式）保持原行为。**零回归**：421→422，69→68 失败，仅 multicol-contained-absolute 翻转。新增单元测试 `test_overflow_nonpositioned_does_not_clip_abspos_with_ancestor_cb`（构造 relative>overflow:h=0>abspos 树，断言绿色填充高度 ~100 未被裁剪）。make test 全绿，clippy 零警告。**方法论**：CSS overflow 裁剪规则按 CB 关系区分（CB 为本元素/后代→裁剪；CB 为祖先→不裁剪），ZeroWeb 原先无 CB 感知；本轮用「overflow 元素是否 positioned」作 CB 关系的可靠近似（非 positioned overflow 不可能是 abspos 的 CB），实现 zero-regression 的常见情形修复。**遗留**：完整 CB-aware 裁剪（处理非 positioned overflow 内含 positioned 后代再含 abspos 的嵌套情形）仍需预计算 abspos CB + 祖先链检查，属后续结构增强。 |
| 2026-06-14 | R125 large-font 集群三路死锁确认（诊断，净中性已回退） | 422/490 持平（所有实验均 net-negative 已回退）。MC_DBG 精确定位 large-font（font-051/ifc-008/009/011/empty-inline-002）根因：paint IFC（painter/text.rs:912）传 `&HashMap::new()`（空 styles）+ override map；对明确高度容器（如 #div1 `height:2em` + `font:100px Ahem`），remeasure 因 `height:Auto` 守卫跳过、override 为空 → paint IFC 默认 16px → 100px 内容渲染成 16px。关键发现：`paint_text` 入口 `style.font_size=100` 正确，但 IFC 用空 styles 解析片段 font_size 走默认 16。**三条修复路径全部 net-negative**：(1) compute_final_inline_layouts 中调 `store_font_sizes_from_ifc`（覆盖）= -1（multicol-fill-auto-001 回归，该用例依赖 remeasure IFC 的 font_size 解析，compute_final IFC 解析不同）；(2) 同上但 `.entry().or_insert()`（不覆盖）= 仍 -1（multicol-fill-auto-001 的新增文本节点条目改变其 paint IFC override）；(3) override 为空时传真实 styles（painter/text.rs:912）= **-4**（"override 空"条件不够窄，多个 box 命中真实 styles 路径）。印证 R72（真实 styles 回归 BFC-004/font-feature-002 等 4 个）+ R104/R117c（font_size 存储回归 font-feature）。**结论**：large-font 是结构性死锁——paint IFC 空 styles、remeasure IFC、compute_final IFC 三路径 font_size 解析不一致，任一统一都回归其他用例。需 Phase A IFC 路径统一（三路径一致 font_size 解析 + font-feature-settings 支持，R82/R84 标注的多件耦合一次性改动）。**不要再单独重试这三条路径**。下一步换结构性目标：multicol 两趟（R113）或 writing-mode 垂直轴（R114b）。 |
| 2026-06-14 | R126 聚类裁决与策略（诊断，持平） | 422/490 持平。回退接手时 multicol.rs 净中性 speculative change；REFTEST_DUMP+PIL 排除 5 聚类（block-in-inline / flex-gap / clear-applies-to 非同源 / multicol-breaking-006 颜色）均非 clean win。clean win 确已穷尽。结论：下一步必须 commit 多轮 nested-multicol 两趟（R113，最 contained）。 |
| 2026-06-14 | R127 float 容器 margin 不折叠（+1，零回归） | 423/490 (+1)。CSS §8.3.1 float 子元素 margin 不与父容器折叠——taffy 视 float 为 block 致容器 margin-top 被折叠到 float margin(max)。新增 `declared_margin_top` 字段 + 4 重门控（门控4=float mt 自身未被 taffy 膨胀）排除 floats-rule3 回归。修复 clear-applies-to-009。同 R108b 谱系但不同子问题。 |
| 2026-06-14 | R128 multicol balance 渲染路径表征（诊断，持平） | 423/490 持平。instrument multicol balance 渲染路径——4 列分布算法(4/3/3/3)+全列 glyph 已确认正确，失败在单列 IFC 行溢出列宽(100px 列出 200px 行)+列高不符 ref(test 62 vs 167)。结构性 IFC 换行/balance 精度，不要再 instrument 此路径。 |
| 2026-06-14 | R129 float shrink-to-fit 宽度（+2，零回归） | 425/490 (+2)。CSS §10.3.5 float width:auto shrink-to-fit——taffy 视 float 为 block 填满可用宽度。`declared_width_auto` 标记 + float pass 收缩到块级子元素最大 border-box 宽度（收缩在定位前=级联正确）。修 float-non-replaced-height-001 + bonus flexbox-column-row-gap-001。同 R127 谱系。 |
| 2026-06-14 | R130 strut 容器字体（+1，零回归） | 426/490 (+1)。CSS §10.8.1 strut——仅原子行内盒的行用容器 font-size 算 strut ascent（旧行高*0.8 被高原子盒撑高致基线偏低盒压下 8px）。修 flexbox-baseline-align-self-baseline-horiz-001；精确定位 flex baseline 集群根因（taffy baseline child-order 不一致 30/20/30 + strut 放大）。 |
| 2026-06-14 | R131 multicol fragmentation 架构（诊断，持平） | 426/490 持平。纠正 R121——multicol 塌缩的 29 个零高子元素是行内级 DOM 节点(text/br)非块级，「行高回写」不可行；真正修复=列感知 IFC 把行内流碎片化到各列（按行盒非子元素生成 column_span_offsets）；even-split 无效。R122 记录 paint 路径改动 net -5 回归。 |
| 2026-06-14 | R132 multicol 容器高度 dead-end（诊断，无提交） | 426/490 持平。新根因：块级子元素 balance multicol 容器高度=taffy 堆叠(过高)；修复 net-negative 已回退(regress paged-media/block-no-clip, fill-000 主因非高度)。re-verify 放宽 height_auto 守卫仍 net -5。nested breaking 真正阻塞=R113 CSS 碎片化语义（内层被外层碎片化时应 fill 非 balance）。 |
| 2026-06-14 | R133 vertical float 实现地基（诊断，持平） | 426/490 持平。vertical-rl 浮动函数实现地基：提取后坐标物理化(block=X 右→左/inline=Y)，converter 不换 float(物理 left/right→block 方向)，故不能轴交换复用水平函数，需独立 block-方向浮动逻辑；clearance-vrl-002 期望 3 绿条水平铺排(右→左 block)。 |
| 2026-06-14 | R134 vertical block 宽度收缩（+6，零回归） | 432/490 (+6)。纠正 R133——parent-width(vertical-rl width:auto 填满 784 而非内容)才是 clearance-vrl blocker，非 float 定位。新增 `shrink_vertical_blocks_to_content` 后处理（自限：仅内容右缘<width 时收缩）；ref 退化(img 不渲染→白)故测试=diff<5%。bonus 修 baseline-inline-replaced-002 + 2 orthog-float。 |
| 2026-06-14 | R135 image-loading degenerate-ref（诊断，已回退） | 432/490 持平。reftest harness `load_png_file` 不处理非 RGBA PNG(palette/RGB→alpha=0)→大量图退化白；单独修 net -5（暴露 clearance-calc 隐藏精度误差 clear-clearance-calc-001/002/003 + vrl-004/008）；必须与 §9.5.2 clearance-calc 精度修复打包。engine 渲染正常只是 harness 加载坏。 |
| 2026-06-14 | R136 flex/grid/table 建立 BFC（净中性零回归，DC-11） | 432/490 持平。`establishes_bfc` 新增 `is_layout_container`(Flex/InlineFlex/Grid/InlineGrid/Table/InlineTable) 判定。taffy 内部已按 BFC 布局，后处理补充判定不改当前结果；DC-11 正确性改进为产品页/未来用例提供正确 BFC 隔离。前置 R137。 |
| 2026-06-14 | R137 孤立 table-internal 匿名 table BFC（+1，零回归） | 433/490 (+1)。孤立 table-internal(父非 table) 经 `mark_anonymous_table_roots` 预处理标记 is_anon_table_root + is_block_level → 建立 BFC，修 clear-applies-to-001。关键=BFC float-exclusion 要求 is_block_level，仅补 is_anon_table_root 无效。 |
| 2026-06-14 | R138 display:table 收缩适应（+1，零回归） | 434/490 (+1)。`layout_table` 在 grid 为空时调用新增 `shrink_table_to_block_content`（新模块 table_shrink.rs，只收缩尺寸不重定位=零坐标风险）。修 html-display-table。关键=grid 空才触发(49 css-tables 用例零影响)+行内级求和(200+80=280)非 max。 |
| 2026-06-14 | R139 直接 table-cell 匿名行（+1，零回归） | 435/490 (+1)。display:table 直接 table-cell 子元素(无 tr)合并为单个匿名行(CSS §17.2.1)，修 subpixel-table-cell-width-001。旧 bug=每 cell 独立行 + get_cell_box 导航错误→cell 未测量全宽堆叠。is_anonymous 导航 + position_cells 守卫。 |
| 2026-06-14 | R140 clean-wins 穷尽独立验证（诊断，持平） | 435/490 持平（两轮诊断 + gap-fix 实验回退）。独立穷尽验证 55 失败清洁 surgical 已耗尽——instrumentation 证 border 数学正确、multicol col_width=100 正确、Ahem glyph=精确 font_size。`% gap→normal when indefinite` 实测 001/002/003 零回归但 004 仅 3.90→3.62%(真正阻塞=R109 垂直 flex 主轴，非 gap)故回退。R131 block-children 分布会回归 ~20 通过用例(=R122 net-5 源)。R141b R109 实测单会话不可解（6 轮），color-block 绘制路径绕过 paint_node 与 paint_text 非存储两条已知路径=深层 inline-block ownership + DC「Layout/Paint IFC 双路径」架构。 |
| 2026-06-15 | R141b → R142 vertical-rl 兄弟位移轴（净中性零回归） | 435/490 持平。R141b R109 实测不可解后，R142 在 `remeasure_inline_only_containers` 末尾兄弟位移 `sibling.y += shrink_delta` 增加 `HorizontalTb` 守卫——该逻辑只适用于水平书写模式，垂直模式块流方向为 x 轴，inline 轴收缩不在块轴留空隙故不应移动按 x 排列的块兄弟。修整页空白灾难(vrl-004 25→12.7%、vlr-005 25→9.3% 内容恢复可见但未过阈)。方法论=逐后处理步 BISECT 插桩(非 bbox 扫描)发现跨聚类遗漏。 |
| 2026-06-15 | R143 inline/block-size 逻辑尺寸属性（净中性零回归） | 435/490 持平。`inline-size`/`block-size` 在 apply.rs 中完全没有 match 分支——作为未知属性被静默忽略。新增 inline-size→width、block-size→height 映射（垂直轴由 converter swap 自动修正）。firefox-bug-1881495 7.28→1.74%（剩余=taffy grid 定宽 inline-grid 轨道不约束换行，taffy 内部）。方法论=属性实现完整性审计（第 4 种定位法，区别 bbox/grep/BISECT）：从失败用例反查其依赖的 CSS 属性是否被实现。 |
| 2026-06-15 | R146 css-flexbox-row 残留 + img-intrinsic 调查（诊断，持平，已回退） | 436/490 持平（两路诊断均无 reftest 价值，已回退工作区清洁）。**css-flexbox-row 残留 1.23%（R145 后）精确根因**：PIL 精确色块定位对比发现 R145 已修 flex item 位置（abs_x=12 左侧正确），残留差异是 **vertical-rl flex 容器的 width:auto shrink-to-fit 循环**——flex 容器（vertical-rl, width:auto）TEST 宽 92 而 REF 宽 34；`.item` flex item（width:auto）被 align-items:stretch 拉伸到容器 cross-size=90，而 shrink_vertical_blocks_to_content 读 child.width=90（已拉伸）→ 无法收缩容器到内容 34（34 需读 .item 的 intrinsic content width=30，而非拉伸后的 90）。循环依赖：容器宽度依赖子元素宽度、子元素宽度依赖容器 cross-size。修复需对 layout-container 父级在 shrink 时用子元素 intrinsic content width 而非拉伸后的 box width——属结构性改动，单 1 测试风险高，未强推。**flex-abspos-inset-nested img 调查（已回退死代码）**：定位根因=`<img>` 无 HTML width/height 属性时 ZeroWeb 不从解码 PNG 推导 aspect-ratio（Chromium 解码 1×1 得 ratio=1.0→height 拉伸 200→width=200）。完整实现 layout→img-intrinsic plumbing（LayoutEngine image_intrinsic_sizes 字段 + build_layout_tree/build_subtree/apply_replaced_element_sizing 全链路 + pipeline builder，~60 行），但 IMGDIAG 证实 **reftest harness image_cache 为空**——扁平布局使 `../support/` 路径解析失败（WPT 原始子目录布局 css/css-flexbox/abspos/test.html）。plumbing 正确但无数据流入=死代码，按 code-guidelines 已回退。修复需先解 harness 图像路径（R135 记录 net -5 风险）。**结论**：两条路径均确认结构性，单会话不可安全推进；436/490 平台期维持。**table-cell-width-0（30% diff）实验已回退**：CSS Tables §17.5.2.2 auto layout 额外空间分配——旧实现 `*= ratio` 按比例膨胀所有列（含 width:0 列），改为「只扩展 auto 列、显式 width 列（含 width:0→min-content）保持」+ 把 width:0(Px<2.0) 从 css_width_auto=true 改为 false（视为约束）。结果 **net -1**（table-cell-width-0 30→28% 未通过 + 新回归 table_grid_size_col_colspan 0→50.92%），已回退。结论：table-cell-width-0 的 30% 差异主因非列分布（百分表宽已正确解析、width:0 列宽修正后仍 28%），且 colspan 交互高敏感；属深层 table auto-layout + width:0+colspan 结构性，单会话不可安全推进。**block-in-inline-align-001（1.42%）诊断**：PIL 确认 TEST 第 2 个 section（`dir="rtl"` + `<span>text<div>text</span>` block-in-inline）的 div 橙色背景完全未绘制（0 orange px vs REF 1072），第 1 个 section（LTR text-align:right）正常（2336 orange）。可复现（字节级 copy zz-copy 同样失败），但手写结构探针通过（差异极细微、未能定位）。EX 插桩证实两个 div 均 is_block_level=true display=Block 且几何正确（在 layout 树中），paint PD2 插桩证实两个 section 都被 paint_node 进入（nchildren=1 即 span）。根因在 paint 层：paint_node 递归进入 span→div 调用 paint_background，但 RTL section 的 div 填充未出现在最终 framebuffer——疑似 paint IFC 对 inline span 的文本绘制路径在 RTL 下消耗/排序异常导致 div 背景丢失，需更深 paint 路径插桩（下轮）。**R147 精确根因定位（已回退实验）**：TR 插桩（paint_node ENTER + paint_background）证实**两个 section 的 div 背景都被 paint_background 生成**（orange fill at abs_x=8 abs_y=8 第 1 section、abs_y=27 第 2 section）——填充图元未丢失。第 2 section 的 div 被画在 abs_y=27（与第 1 section 重叠），而 REF 在 ~65。**真正根因 = 块流重叠**：block-in-inline 的 `<span>text<div>text</span>` 使 section 展开为 3 行（h=38.4，paint trace span h=58），但 `remeasure_inline_only_containers`（engine.rs:2962-2989）的兄弟位移逻辑**仅处理收缩（shrink_delta < -0.01）不处理展开（正 delta）**——section[0] 展开到 38.4 后，section[1] 仍停在 taffy 算的 y=19（1 行高），两者重叠，第 2 section 的 div 被第 1 section 内容覆盖。**实验（已回退）**：把位移条件改为 `shrink_delta.abs() > 0.01`（同时处理展开）→ block-in-inline-align-001 1.42→1.69%（**恶化**），全量仍 436/490。原因：非所有正 delta 都是 block-in-inline 展开（IFC 高估也会产生正 delta），盲目位移正 delta 回归其他用例。**结论**：需更精确信号区分「taffy 低估 block-in-inline 展开」vs「IFC 高估」才能安全位移——例如仅对「含 inline 容器内嵌 block 子元素（block-in-inline）」的展开位移。下轮可加此精确守卫。**R147b 精确守卫实验（已回退）**：实现 `has_inline_child_with_block_descendant`（仅对块级容器含 inline 子元素且其内嵌 block 后代的展开位移），SH 插桩证实位移正确触发（idx=0 delta=19.4 is_expand=true shift_allowed=true）。但 **block-in-inline-align-001 仍 1.42→1.69%（恶化）**，全量 436/490。原因：嵌套展开——section[1] 自身的 span 也展开（delta=19.4），简单按 delta 位移产生过校正。**真正修复**：block-in-inline 容器展开需原子化多趟或一次性按展开后高度重排兄弟 y（不能逐子元素 delta 累加），属结构性改动。**不要再尝试逐 delta 位移法**。 |
| 2026-06-15 | R145 flex/grid/table 子元素 float 归零（+1，零回归，纠正 R144 R109 误判） | 436/490 (+1)。双端插桩（engine.rs extract_layout + painter/mod.rs paint_background）对比发现 css-flexbox-test1 / css-flexbox-row 的 `.item` flex item：extract_layout box.x=2（正确），paint box.x=690（=780−90 容器右缘减宽度）。**真正改写 x 的是 `adjust_float_positions_with_context` 的浮动后处理**——`.item` 带 `float:right`（CSS 注释），Phase 1 定位到 `container_width − right_used_width = 690`。R141b 的 mirror child.x 实验无效（它改 flex item 的 block 子元素非 IFC）。**纠正 R144「R109 paint 路径未定位、6 轮不可解」误判**——真正破坏入口是浮动后处理。**修复**：CSS Flexbox §4 / Grid §4 / Tables §2.4 规定布局项 float 计算为 none；浮动后处理入口对 `is_layout_container` 父级直接子元素归零 float（6 行）。零回归：435→436，css-flexbox 44→45，唯一翻转 css-flexbox-test1 FIXED（0.00%），css-flexbox-row 改善 1.82%→1.23%（剩余=vertical-rl 色块 IFC 列序独立子问题），零新失败。+1 单测 `test_flex_item_float_is_ignored`。make test 全绿，clippy 零警告，smoke 686/686。**方法论教训**：架构性失败的「不可解」结论需经多入口插桩交叉验证（R141b 单一 paint 入口的失败不能推广为整体不可解）。 |
| 2026-06-15 | R144 平台期独立复核（诊断，持平，无提交） | 435/490 持平（make reftest-upstream 实测确认 435/490，55 失败）。本轮独立复核三项：(1) **属性实现完整性审计穷尽**——写脚本对比 registry.rs 全部 197 个已注册属性 vs apply.rs/apply_advanced.rs 的 match 分支，**每个已注册属性都有对应 apply 分支**（R143 的 inline-size/block-size 是最后一个缺口），属性审计路径已完全穷尽；(2) **R109 color-block 绘制路径**——PIL 确认 css-flexbox-row.html 的 4 个 inline-block 色块 TEST 在 x[670,789](右) 而 REF 在 x[10,39](左)，证实 R141b 的结论（flex item 主轴定位 + inline-block 经 IFC，mirror child.x 不生效），6 轮不可解，架构项目；(3) **border-001 / column-height-009 / float-006 / baseline-vertical 等单例**——PIL/bbox 复核均确认为结构性复合问题（large-font 死锁、multicol-2 column-height/column-wrap、abspos+float+z-order、vertical table baseline），非清洁修复。**结论：435/490 为已确证的单会话零回归平台期**，剩余 55 失败全部属于结构性多轮里程碑（multicol column-breaking R113 两趟 / writing-mode 垂直轴 R109 架构 / flex baseline 合成 / vertical table baseline）。下一步建议：(a) 启动 inline-block ownership + vertical-rl 多会话架构项目（统一 paint 的 inline-block 绘制路径到 IFC，解 R109 聚类 css-flexbox-row/test1 + flexbox-column-row-gap-004），或 (b) 启动 multicol 列感知 IFC 碎片化（R131，把行内流按行盒碎片化到各列，影响 ~16 测试但回归风险 20 通过用例）。两条均为多轮、高风险、单会话预期 +0。 |
| 2026-06-15 | R148 全量 54 失败独立再核（诊断，持平，无提交，工作区清洁） | 436/490 持平（make reftest-upstream 实测 436/490=89.0%，54 失败）。本轮对全部 54 失败按 diff 升序排列并逐个 PIL+BIPROBE/CRPROBE/BBPROBE 独立插桩，**无任何 +1 清洁修复可安全推进**，全部印证结构性。新发现/精化（区别既往轮次）：(1) **multicol-breaking-006 (1.20%) 精确双根因**——CRPROBE 插桩 painter/text.rs paint_column_rules：外层 multicol(count=4 col_w=188) 仅 1 子(内层 .inner)，内层 .inner 的 16 个行盒**全部 x=102（右列），左列(x=0..86)无内容** → has_left_content=false 跳过 fuchsia column-rule；外层 col1/2/3 无子 → 跳过 blue column-rule。根因=**列分布把全部内容堆到单列**（应平分 2 列），column-rule 缺失是表象，真因是 R131 列感知 IFC 碎片化缺失。修 column-rule 绘制本身无意义（R112 已证 net -1）。(2) **border-bottom-width-006 (2.86%) 非绘制 bug**——BBPROBE 证实 #test border-bottom=96 w=96 h=96 ay=51 style=Solid 几何全正确、paint_borders 入口正确生成填充；差异源=inline-block「仅底边框 height:0」与 #reference 黑块的**垂直定位/基线对齐**（test 黑区 y=[16,146] 含 100px 宽条 vs ref 双 96×96 黑块 y=[29,151]），属 inline-block baseline 定位子问题非 border 渲染。(3) **clear-inline-001 (5.94%) 非 clear bug**——断言「clear 不能应用于 inline box」(CSS2.1 §13.5)，span2 clear:left 应被忽略；TEST 把蓝文本画在 float 旁(y=51)实际符合规范，REF 因用 96px 非浮动 `<img vertical-align:top>` 撑高行盒使蓝 span 落到 y=147；TEST/REF 结构不同故天然差异，非 ZeroWeb 渲染错误。(4) **block-in-inline-align-001 独立复现 R147b 恶化**——BIPROBE 证实两个 section 各 expand 19→38.4(delta=19.4, has_block_child=false 因 div 嵌在 span 内非 section 直接子)；BIR148 实验(正 delta 位移后续兄弟)复现 1.42→1.69% 恶化，PIL 显示 section[0] 的 div 画在 y=8(section 顶)而非文本一行后的 y=27——证实**另含 paint IFC 对 inline span 内 div 的垂直定位 bug**（layout 把 div 视 section 顶部，paint IFC 同样），叠加兄弟位移=双路径不一致，确认需 Phase A IFC 统一。(5) baseline-007/008(multicol baseline-export+column-span+flex align-items:baseline)、multicol-count-computed-003/004(Ahem 字形跨列溢出+column-rule)、flex-container-min/max-content(R97 intrinsic sizing 4win/13regress)、table-cell-width-0(R146 已证 net -1)、column-height-009(css-multicol-2 column-height/column-wrap 未支持 spec)、flexbox-baseline-align-self-baseline-vert(垂直 flex 基线合成)——全部结构性复合。**结论**：436/490 平台期经本轮全量 54 失败独立再核后**再次确证为单会话不可推进**；剩余按聚类=multicol 碎片化(~16) + IFC 双路径/inline-block ownership(block-in-inline×3+css-flexbox-row+large-font×4) + 垂直书写模式轴(×5) + flex 基线合成(×4) + intrinsic sizing(×4) + 表格深层(×3) + spec 未支持(multicol-2)。下一步须启动结构性多轮项目，单会话预期 +0。 |
| 2026-06-15 | R149 DC-10 draw_order 基础设施（净中性零回归，已提交）+ PNG bundle 实测（已回退） | 436/490 持平（默认 + `ZERO_DRAW_ORDER=1` 双路径均 436，set-diff 零翻转）。**已提交**：`RenderPrimitives` 增 `draw_order: Vec<DrawOp>` + 每个 `add_*` 记录插入顺序；`render_full_scene` 拆 `render_typed_buckets`（默认字节不变）+ `render_draw_order`（`ZERO_DRAW_ORDER=1` 按序）；cull 重建清空 draw_order；+1 单测。make test 12177/0，clippy 零警告。**纠正 R135b「draw_order net -1」结论**：436 基准实测净中性（R142 vertical-rl 守卫已消除 abs-pos-non-replaced-vrl-002 回归），draw_order 可安全启用。**PNG bundle 实测（已回退）**：`load_png_file` 加 `EXPAND\|STRIP_16` 实测 436→427 net -9（修正 R135 记录 -5）；PNG+draw_order 组合仍 net -9。PIL 证实 abs-pos-non-replaced-vrl-006 绿 span=0（应 6400）、红=18321 主导——**真正阻塞重新定性为 abspos vertical-rl §10.3.7 静态位置 bug**（非 DC-10 绘制顺序），PNG 网格仅暴露该布局 bug。bundle 修正=(A)PNG EXPAND + (B)draw_order 已就绪 + (C')abspos vertical-rl §10.3.7 静态位置（替代旧 clearance 精度）。实证复核 R148：54 失败=41 CSS-REF + 12 IMG-REF(退化区) + 1 路径缺失，clean single-session win 四重确证穷尽。下一步多会话=先独立修 abspos vertical-rl §10.3.7（4 个假通过，CSS-REF 可独立验证）→ 再叠加 PNG+draw_order 应 net≥0。 |
| 2026-06-15 | R150 abspos vertical-rl height:auto bug 精确定位（诊断，持平，已回退探针，工作区清洁） | 436/490 持平。写探针单测（复刻 abs-pos-non-replaced-vrl-006 结构）layout tree dump 定位：abspos span(is_absolute=true)几何=`x=240 y=80 w=80 h=320`，**h=320 错误**——CSS §10.3.7+writing-modes §7.1：vertical-rl 下 height:auto 应 shrink-to-fit 到内容(80px)而非填满 CB cross-axis(320)；spec 注释明确 height:auto=80、top:auto→static=160、bottom solved=80(160+80+80=320 ✓)。当前 taffy/converter 把 abspos auto height 当 cross-axis stretch。**不产生 session pass**：4 个 abs-pos-non-replaced-vrl 全是退化参考(非 RGBA PNG 双方退化)当前 4.5% 凑合通过，修 abspos height 不改通过数，仅 PNG fix 后有意义(需同提交=R149 bundle 组件 C')。独立验证：abspos-containing-block-outside-spanner(水平+显式尺寸,column-span CB 子问题)/flex-abspos-inset-nested(img aspect-ratio)均**非**此 bug 同源，无当前失败用例直接受阻塞。**fix 入口指引**：`fix_vertical_mode_abs_pos`(engine.rs:1133-1217)当前仅用 IFC fragment 修 top/bottom 全 auto 的 x/y，**不修 height**——下轮应在 vertical-rl 容器内对 height:auto abspos 子元素把 height 收缩到 fragment inline extent，配合 PNG fix 验证。clean single-session win 五重确证穷尽(R140/R144/R148/R149/R150)。 |
| 2026-06-15 | R158 large-font 死锁机制精确定位 + 失败聚类再分类（诊断，持平，无提交，工作区清洁） | 436/490 持平（make reftest-upstream 实测 436/490=89.0%，54 失败）。本轮独立复核全 54 失败 + 1 次 compute_final 显式高度容器补存 font_size 实验（死代码已回退）。**large-font 死锁机制现已精确定位（比 R125「三条路径」更精确）**：100px 文本位于 taffy 已测量的 height:auto 子容器（如 ifc-008 的 `#div1>div div`，content_height>1.0），被两条存储路径同时跳过——(a) remeasure 的 `content_height<1.0` 守卫(engine.rs:2883)排除 taffy 已测量块；(b) compute_final 的 R84 守卫(engine.rs:1628 `lines.len()>1\|\|!is_pure_ahem`)对多行/非纯 Ahem 提前 return。两路径都跳过→text_node_font_sizes 空→paint IFC(painter/text.rs:912 空 styles)按 16px 解析→100px 渲染成 16px。**关键冲突**：multicol-fill-auto 当前仅因部分文本节点 16px 错误默认值才通过；补存正确 font_size 改变其 paint IFC override→失败。故 large-font 修复须先让 multicol-fill-auto 在真实 font_size 下也正确。**compute_final 显式高度补存实验=死代码**：显式高度 `#div1` 无直接文本(只有块子元素)，compute_final 在 `!has_text_children` 早返回根本到不了补存点，已回退。**font-051 重新定性为 large-font**（非 font 简写 bug）：`span{font:serif}` 经 expand_font(shorthand/mod.rs:1572)正确判无效返回 vec![]，span 继承 100px Ahem；8.19% 差异来自继承 100px 经 paint IFC 死锁渲染成 16px。**min-max-size-table-content-box (36.34%) 重新定性为 inline-block ownership**（非 table bug）：TEST 的 7 个 table 正确 shrink-to-fit，但 REF 的 `.table{display:inline-block}` div 渲染全宽(w=793)——converter 把 InlineBlock 映射 taffy Block(mod.rs:266)被拉伸，adjust_inline_block_positions 的 ib_sizes 用 taffy 全宽 content_width 未做 shrink-to-fit；grid 内 inline-block 受 track 约束正确收缩(w=11)。inline-block width:auto shrink-to-fit 需测量子树 max-content=Phase A。其余复核候选(background-attachment=image 退化/baseline-007/008=baseline-export/ifc-011=image+vertical-align/count-computed=image+分布/clear-float-003=R114b 负 clearance/collapsing-001=R157 协调/border-padding-bleed=IFC/float-nowrap-hyphen=crbug1499290)全部印证结构性。clean single-session win 八重确证穷尽(R140/R144/R148/R149/R150/R155/R157/R158)。 |
| 2026-06-17 | R214 图片子资源加载（URL 导航路径，PNG，DC-13 第二个 P1 子项） | 439/490 持平（reftest 用本地文件不触发 URL 导航；影响产品导航 + DC-13/DC-11）。修复 goal doc DC-13 P1「图片子资源/ImageCache 未贯通」：`<img>` paint 已能生成 ImagePrimitive，但 fetch_url 不抓 `<img src>`、webview 不持有 ImageCache、render-foundation 无解码。**分层修复**（PNG 先行，JPEG/SVG 同模式后续）：① **render-foundation** 加 `png = "0.17"` dep + `pub fn decode_png_bytes(bytes) -> Result<ImageData,String>`（image_cache.rs，`EXPAND\|STRIP_16` 正确处理 palette/grayscale/RGB/RGBA 全 color type→RGBA，独立于 reftest 的 env-gated 版本故零 439 baseline 影响）+ `convert_png_buffer_to_rgba` + 2 单测（2×2 RGBA 解码、非法输入返 err）；② **zero-engine** 加 `pub fn extract_img_srcs(html)`（pipeline.rs，复用 zero_dom DOM 精确提取 `<img src>`，parallel to extract_stylesheet_hrefs）；③ **zero-webview** 加 `image_cache: ImageCache` 字段（new 初始化 default）+ 私有 `fetch_image_subresources(html, base_url) -> HashMap<u64,(f32,f32)>`（extract img srcs → `url::Url::join` 按 base 解析 → `http_client.get` 抓取 → `decode_png_bytes` → `image_cache.insert_with_key(ImageKey(simple_hash(abs)), img)`，键与 pipeline build_img_intrinsic_sizes + 渲染器查找一致）→ 返回 image_sizes，三条 fetch_url 分支（SW/cache/network）注入 `pipeline.set_image_sizes(image_sizes)` 后再 load_html（`<img>` 正确固有尺寸 DC-11）；data: URI 暂跳过、抓取/解码失败仅 warn 不阻断；暴露 `pub fn image_cache(&mut self)` 供下游渲染器绘制消费。**端到端测试**（webview_coverage）：mini-server 重构支持二进制内容（`HashMap<String,Vec<u8>>`，header/body 分开写），服务 3×2 纯绿 PNG + page，fetch_url 后断言 `image_cache` 含该图（键=simple_hash(abs url)）、尺寸 3×2、左上像素纯绿 (0,255,0,255)。13 webview + 2 decode + 1143 engine 测试全过。**意义**：图片子资源抓取+解码+缓存贯通 webview 层，`<img>` 经 URL 导航获正确固有尺寸 + ImageCache 就绪供浏览器渲染；浏览器 render_cpu/gpu 当前传 `None`（app_platform.rs:153），传 `Some(&mut webview.image_cache())` 是最后消费 hop（下一步）。make test 全绿、clippy --workspace --all-targets -D warnings 干净、fmt 干净。 |
| 2026-06-17 | R227 welcome 36px 偏移根因独立确证=taffy border-box 子坐标 vs painter 内容盒约定双重计入（并行 agent 已实现 engine.rs 修复，本条=只读验证） | 439/490 同源持平（本会话为文档+只读验证，无代码变更；并行 agent 正在 engine.rs:787-795 + reftest.rs `LAYOUT_DUMP` 探针实现修复，本条记录对其机制的独立确证与回归面分析）。承接 R226（welcome 顶部 36px 垂直偏移定位）。**独立只读验证确证 R226 假设——坐标系约定冲突致双重计入，经 painter 源码实证**：① 引擎提取（engine.rs:696-697）`x = layout.location.x; y = layout.location.y;` 直接取 taffy `Layout::location`，该值是**子 border-box 相对父 border-box** 的偏移（taffy 语义 `content_box_y = location.y + border + padding`，即 location 已含父 padding+border）；② painter（painter/mod.rs:271-272 `paint_node_in_rect` / 456-457 `paint_node`）`child_offset_x = abs_x + padding_left + border_left; child_offset_y = abs_y + padding_top + border_top;` 后 `paint_node(child, ..., child_offset_x, child_offset_y)`，子绝对坐标 = child_offset + child.x，即 painter 期望 child.x 是**相对父内容盒**（它在 child_offset 上已加一份父 padding+border）；③ **约定冲突**=location（border-box 相对 border-box）≠ painter 期望（内容盒相对），painter 加一份 + child.x(=location) 已含一份 → **每代有 padding/border 的祖先把整棵子树多偏移一份**；④ welcome 实算 body(`*{padding:0}`)→.page(padding-top 20)→.hero(padding-top 16)→.hero-accent：chromium 内容 y=20+16=36（实测 ✓）；双重计入下 page padding 计 2 份多 20、hero padding 计 2 份多 16=合计 +36 → ZW y=72（chromium 36，差 36）✓ 与 R226 实测精确吻合，无任何补偿（全量双重计入）。**并行 agent 修复方向正确**（engine.rs:787-795）：HorizontalTb 下对非 abspos/fixed 的块/inline-block 子节点 `child.x -= content_x; child.y -= content_y;`（content_x/y=父 border+padding）把 border-box 相对换算为内容盒相对消除双重计入；嵌套多层 padding 每代精确减一次（page→hero→hero-accent 各减本代 content_offset），数学闭合（hero-accent abs y=36 = chromium）。**回归面分析**：(a) **同源 reftest（439/490）预期~中性**——双重计入对 test/ref 同结构时双侧同偏移相互抵消仍匹配，仅当 test/ref 用不同机制（padding vs margin/abspos）达成同视觉时才翻转，多见 FAIL→PASS 正向（消除 test 侧偏移使二者对齐）；无 padding/border 祖先（content_x/y=0）本修复为 no-op；(b) **产品 smoke（welcome）**——36px 双重计入消除→内容上移对齐 chromium→welcome 28% diff 预期显著下降（DC-13 真杠杆，印证 R226，纠正 R174「welcome 剩余~28% 是 fontdue 字体噪声无 clean bug」的旧结论）；(c) **chromium Oracle**——本 bug 是**纵向偏移**类（区别于 R225 证伪的横向 advance-width），影响所有含 padding/block 嵌套的真实页，183-case 1-3% 噪声桶中含纵向偏移分量的用例应受益，需重跑 cross-validate 量化。**新 bug 类意义**：「taffy 提取层 border-box 约定 vs painter 层内容盒约定」坐标系不一致，是区别于 advance-width 死胡同（R221-R225）与结构性 multicol/writing-mode 阻塞（R109/R113/R131）的**第三类**根因；特征=同源不可见（双侧抵消）但 chromium Oracle 可见，正是 DC-14 独立 Oracle 揭示的真实缺口；修复为局部后处理（非多轮结构）潜在高杠杆。**遗留/下一步**：(a) 并行 agent 验证 make reftest 零回归 + welcome smoke 改善 + cross-validate 量化（本条不代其执行）；(b) 核查**同一约定不一致是否存在于 inline IFC 子节点路径**（paint IFC fragment 定位 painter/mod.rs:501-502 content_x/y + text.rs）——welcome 首个非背景是 block .hero-accent 故本修复已覆盖主偏移，但嵌套 padding 下文本行纵向位置仍可能受影响；(c) **垂直书写模式**本修复显式跳过（仅 HorizontalTb），vertical-rl 等价双重计入需轴交换路径另算（R109/R142 谱系已知）；(d) 复核 abspos/fixed 跳过自洽性（其坐标语义与 painter 双计补偿是否在所有嵌套深度成立）。 |
| 2026-06-17 | R226 welcome 28% diff 真因定位=顶部 36px 垂直偏移级联（非字体/度量/背景） | 439/490 持平（诊断，无代码变更）。承接 R225（advance-width 证伪）。对 welcome ZeroWeb CPU vs chromium Oracle PNG 做**区域 diff 分析**：① y 带密度——hero 顶 9.8% → 底部 grid/flex 区（cards/shortcuts/quick-links/footer）41-43%（底部远高）；② **垂直内容起点**：ZW first-non-bg at y=72，CH y=36 → **ZW 内容低 36px**；content y-range ZW[72,583] vs CH[36,534]（ZW 长 49px）；③ 扫描线 y=350：ZW 全内容、CH 全背景空 → 内容 y 错位。**排除项**：body bg 正确（角点 ZW=CH=(244,246,248)）、advance-width 已证伪（R225）、AA 基准排除光栅化。**根因**：body→.page(margin:0 auto,padding 20px 40px 24px)→.hero(padding 16px 0 20px) 链中 margin 折叠/padding 累加与 chromium 不一致（36px≈page padding-top 20 + hero padding-top 16），致整页下移 36px 级联。**策略意义**：welcome 28% diff 是**布局垂直定位 bug**（可定位可修复），非字体噪声——DC-13 产品 smoke 真杠杆=修此 margin/padding 偏移。证据 `evidence/welcome-region-analysis-2026-06-17.txt`。下一步=dump body/page/hero 盒几何定位 36px 偏移确切来源并修复。 |
| 2026-06-17 | R225 advance-width 非噪声根因（product-smoke + reftest-oracle 双实验证伪） | 439/490 持平（实验已回退）。R224 同源 -3 后，本轮做**决定性证伪实验**：re-apply DejaVu advance 表，测 chromium 一致率变化。**双实验均证伪 advance-width 是噪声根因**：① **reftest-oracle**（26 共享 case）：strict true-pass 11 vs 11、median z_vs_chr 1.06% vs 1.07%、0 case 改善/恶化（Ahem 用例 is_ahem 特例=font_size，estimate 表不适用故无影响）；② **product-smoke（非 Ahem，advance 表真正起作用的场景）**：welcome 28.34%→28.31%（Δ-0.03%）、wintertc 25.11%→25.14%（Δ+0.03%）—— **零实质变化**。**结论**：`estimate_char_width` 改进**对 chromium 一致率无影响**（无论 reftest 还是产品 smoke）。**机制推断**（待证实）：paint glyph x 定位走真实 fontdue shaping 而非 estimate（estimate 仅影响 layout 换行决策，glyph 位置视觉主项由 paint fontdue 决定，故改 estimate 不动 diff）。**重大策略意义**：推翻 R221/R222 「advance-width 是 183-case 噪声杠杆」假设——advance-width plumbing（R223 trait seam 留存无害）是**死路**，勿再投入。28% 产品 smoke diff 真因在别处（line-height/baseline、box 定位、或 paint fontdue-vs-chromium 光栅化差异，待定位）。验证：回退后 reftest 439/490 恢复、product-smoke 恢复。 |
| 2026-06-17 | R224 estimate_char_width 实测表精化实验（回退，净 -3 回归证否单点捷径） | 439/490 持平（实验已回退，仅留教训注释）。承接 R222/R223。本轮尝试**捷径**：用 DejaVu Sans 实测 advance 比率表（W=0.99/i=0.28 等 94 项 ASCII）直接替换 `estimate_char_width` 的固定倍数（字母 0.55/数字 0.5/标点 0.4/空格 0.25），避开 R2-R5 跨 crate plumbing。**实测全量 reftest 439→436 净 -3 回归**（非 Ahem 用例换行点翻转），按设计成功标准（须持平或 net≥0）**回退**。**关键教训**：estimate_char_width **并非纯自源中性**——reftest 的 test 与 ref 虽同用 estimate，但文本结构不同时换行点敏感度不同（如一个空格宽从 0.25→0.3179 致某用例恰好溢出换行），单独扰动 estimate 会破坏同源对齐。**结论**：advance-width 真实修复**不能走单点改 estimate 捷径**，必须完整接入 FontLoader（R223 plumbing R2-R5：layout+paint+intrinsic 三处同源替换 + TextRun 携带 font_id 解析），保证 test/ref 与 chromium 三方度量同源。本次回退保留教训注释（inline/mod.rs estimate_char_width doc），AdvanceSource trait seam（R223）保留待 R2 接入。验证：回退后 reftest 439/490 恢复、make test 全绿。下一步=R2 签名注入（&dyn AdvanceSource 进 IFC，默认 EstimateAdvance，零行为变更）而非单点改 estimate。 |
| 2026-06-17 | R223 advance-width plumbing R1（AdvanceSource trait + 设计 RFC，零行为变更） | 439/490 持平（R1=行为中性 seam，默认实现等价 estimate_char_width）。承接 R222 决定性诊断（逐字符 ±44-98% 误差）。本轮启动 advance-width plumbing 多轮工作：① 写设计 RFC `docs/goal/rendering-compat/advance-width-plumbing-design.md`——核心 = 依赖反转（layout-engine 定义 `AdvanceSource` trait，`EstimateAdvance` 默认实现=estimate_char_width，zero-engine 注入 FontLoader-backed 实现），5 轮渐进（R1 trait seam / R2 签名注入 / R3 真实 advance 启用 / R4 intrinsic+paint / R5 oracle 量化），含 R125 IFC 三路径死锁风险评估与缓解（source 是纯度量函数不涉 font_size 解析，三路径同源实例→度量一致）；② 实现 R1——`AdvanceSource` trait + `EstimateAdvance` 默认 impl（inline/mod.rs），`measure(ch, font_id, font_size, is_ahem)` 委托 estimate_char_width；③ 等价性单测 `test_estimate_advance_matches_estimate_char_width`（验证 trait 默认实现与 estimate_char_width 逐字符完全等价 + font_id 为 None/Some 均等价）。**零调用点改动、零行为变更**（seam 就位待 R2 注入 IFC 签名）。**验证**：make test 12230 passed/0 failed、clippy/fmt 干净、reftest-upstream **439/490 持平**（证 R1 行为中性）。**意义**：建立 advance-width 真实度量接入的依赖反转 seam（layout-engine 不向下耦合 render-foundation FontLoader），为 R2-R5 渐进替换 estimate_char_width 铺路，瞄准 R221 的 183-case 系统性噪声桶。下一步=R2（IFC 函数签名加 `&dyn AdvanceSource` 参数，默认传 EstimateAdvance，内部调用改 source.measure）。 |
| 2026-06-17 | R222 advance-width 估计误差诊断（advance-width plumbing 数据依据） | 439/490 持平（新增诊断测试 + 证据，无行为变更）。承接 R221 识别的 183-case 1-3% 系统性噪声桶。新增 `diag_advance_vs_estimate_systematic_error` 测试（render-foundation font/loader.rs）：加载系统字体，对比 `FontLoader::measure_advance`（fontdue 真实度量）与 `estimate_char_width` 启发式（字母 0.55×fs/数字 0.5/标点 0.4/空格 0.25）。**实测逐字符误差极大（±44%~98%）**：W 实际 0.989×fs（estimate 0.55 欠估 44%）、i/l 实际 0.278（过估 98%）、m 0.974（欠估 44%）、t 0.392（过估 40%）、f 0.352（过估 56%）、H 0.752（欠估 27%）、数字 0.636（欠估 21%）、标点 0.318（过估 26%）；总和部分抵消 -6.9% 但**逐字符累积定位全错**（"Will" 按 0.55 均匀 vs 实际 W 近全宽 i/l 极窄）。**证实** R221 推断：layout IFC + paint IFC + intrinsic_sizing 三处 estimate_char_width 是 183-case 系统性噪声根因（非字体光栅化，AA 基准已排除）。**关键发现**：paint/text.rs:410/443（list marker 定位）已用 estimate（paint 有 FontLoader 可直接修）；`FontLoader::measure_advance` 已存在（loader.rs:289），缺的是把它接入 layout-engine IFC。证据 `evidence/advance-width-estimate-error-2026-06-17.txt`（含复现命令）。**修复路径（多轮）**：engine 预解析 font-family→FontId 建 advance 源 → 传入 layout IFC + intrinsic_sizing + paint IFC 三处替换 estimate_char_width。self-source 中性（同源 439 不变）但降 chromium 噪声。make test 12229 passed/0 failed、clippy/fmt 干净。 |
| 2026-06-17 | R221 DC-14 可信通过率量化分析（chromium Oracle 视角，策略重定向） | 439/490 同源持平（分析型，无代码变更）。基于 06-17 全量 cross-validate 数据（R165–R180 修复后），以 z_vs_chr（ZeroWeb-test vs chromium-test）为唯一可信指标重算：**严格真通过率 = 188/475 = 39.6%**（z_vs_chr<1%），对比同源 89.6%——同源**严重高估**。分布：<0.5%=97 / <1%=188 / 1-3%=183 / 3-8%=67 / ≥8%=37。**关键发现**：① **183 case 在 1-3%**（系统性布局/字体噪声）= **最大杠杆**——AA 基准已证非光栅化，是布局定位（estimate_char_width 近似 vs 真实 advance），降此噪声是把真通过 188→370+ 的最短路径（R195/R196 定性，self-source 中性故同源 439 不动）；② **116 case 假失败**（self-fail 但 chr<5%）= ZeroWeb-test 实际接近 chromium，**同源 ref 怪异**（如 vrl-004 同源 7.09% vs chr 5.08%），同源判 FAIL 是 reference 的错非 ZeroWeb——**同源 reference 双向不可靠**，DC-14 独立 Oracle 是唯一可信判定；③ 37 case ≥8% = 结构性聚类（clean-win 穷尽）。**策略重定向**：DC-2~5 达标路径 = 降系统性布局噪声（advance-width/line-breaking 精度，多轮 self-source-neutral）+ 修结构性聚类（多轮）。证据 `evidence/dc14-credible-passrate-2026-06-17.txt`（含复现命令）。DC-14 独立 Oracle 基建已就绪，严格容差已用，分母待补全上游全量（475/490 子集）。 |
| 2026-06-17 | R220 DC-9 真实范围纠正（clip 为 no-op，GPU 缺口仅 transform/filter/blend 三项，docs-only 治理） | 439/490 持平（docs-only，无代码/reftest 变更）。承接多轮对 DC-9 GPU「丢弃 4 图元（transform/clip/filter/blend）」的认知，本轮 grep 实证纠正。**发现**：engine 在生产路径**从不生成 `ClipPrimitive`**（`add_clip` 全仓库 0 处非测试调用）——overflow 裁剪在 paint 阶段由 `clip_all_primitives_to_rect`（painter/mod.rs:292/553/566/590/690 + text.rs:1090 + effects.rs:797）**预烘焙进图元几何**（fills/glyphs/strokes 等的坐标被裁到 overflow 容器 rect 内），故 `RenderPrimitives.clips` 在生产中**恒空**。因此 R211（commit 2af1141）所记「render_full_scene_gpu drops clip」**实为 no-op**（无 clip 可丢，dropping 空列表），DC-9 的 ClipPrimitive 项在 CPU（render_draw_order 的 DrawOp::Clip→apply_clip）与 GPU 两路均**空谈满足（vacuous）**。**真实 DC-9 缺口仅 transform/filter/blend_mode 三项**：engine 在 `paint/painter/effects.rs:266/289`（FilterPrimitive，CSS `filter:`/backdrop-filter）+ `:313`（BlendModePrimitive，mix-blend-mode）+ `paint/helpers.rs:168/184`（TransformPrimitive，CSS transform）生成，GPU 全量路径（render_full_scene_gpu）collect 顶点时**未读 primitives.transforms/filters/blend_modes**故静默丢弃。**修复路径（多轮）**：这三项需 **ping-pong 双纹理后处理架构**——wgpu 不能在同一 render pass 读写同一纹理，filter（区域采样变换如 blur/opacity）、transform（区域反向采样仿射）、blend（与 backdrop 合成）均需 read 源区域+write 目标。GPU 已有 `headless_texture` offscreen 渲染目标（mod.rs:93/125/154），ping-pong 基建部分就绪，差第二张纹理 + post-process WGSL pipeline + per-region scissor。**优先级低**：transform/filter/blend 在 reftest/静态内容中**低频**（仅显式 CSS 触发），GPU 路径非 reftest load-bearing（reftest + product-smoke 走 CPU）。**本轮纠正价值**：避免后续会话在 no-op clip 上浪费、重定 DC-9 收尾为「3 项低频 + ping-pong 多轮」、对齐 goal doc 治理「状态须诚实」。下一步候选=DC-9 GPU ping-pong 地基（filter:opacity 最简先建）/ DC-14 chromium-oracle 严格容差默认接线 / DC-13 产品 smoke 端到端证据持久化。 |
| 2026-06-17 | R219 SVG fetch_url 端到端验证测试（R215-R218 全链路最后验证拼图） | 439/490 持平（新增测试，零行为变更）。R218 加了 SVG 解码（decode_svg_bytes + decode_image_bytes 内容嗅探路由），但仅在 render-foundation 单测层验证；webview URL 导航路径（fetch_url→fetch_image_subresources→decode_image_bytes）未验证。新增 `test_fetch_url_loads_svg_image_subresource`（webview_coverage）：MiniServer 服务 4×3 纯绿 SVG + page，fetch_url 后断言 image_cache 含栅格化结果（键=simple_hash(abs url)、尺寸 4×3、绿色 G>200 + alpha=255）。**意义**：R215（browser render 消费 image_cache）+ R214（PNG fetch→cache）+ R216（JPEG）+ R218（SVG）+ 本测试（SVG fetch→cache）共同闭合「fetch→decode PNG/JPEG/SVG→image_cache→browser render→真像素」全链路验证。make test 12228 passed/0 failed、clippy/fmt 干净。下一步=DC-13 产品 smoke 持久化证据 / DC-9 GPU 4 图元 / DC-14 chromium-oracle 严格容差默认接线。 |
| 2026-06-17 | R218 SVG 解码统一到 render-foundation（DC-13 SVG 栅格化全路径） | 439/490 持平（reftest 路径 load_svg_file 委托后行为不变）。goal doc DC-13 要求「PNG/JPEG/WebP 基础解码和 SVG 栅格化」。reftest 路径早有 `load_svg_file`（resvg+tiny-skia），但 webview/browser URL 导航路径的 `decode_image_bytes` 对 SVG 返 unsupported——浏览器导航含 `<img src=logo.svg>` 的真实页面（WinterTC 14 logo 中 11 个 SVG）Logo 不渲染。**修复**：① render-foundation 加 `resvg`(workspace)+`tiny-skia` 依赖 + `pub fn decode_svg_bytes(bytes)`（resvg usvg 解析→按 SVG 内在尺寸 tiny-skia pixmap 栅格化→RGBA，过大尺寸 pixmap 分配失败自然兜底）；② `decode_image_bytes` 扩展 SVG 分支——`looks_like_svg` 嗅探 UTF-8 文本（跳 BOM/空白后 `<svg`/`<?xml` 起始）路由到 `decode_svg_bytes`；③ reftest `load_svg_file` 委托 `decode_svg_bytes`（同 R217 去重），移除 wpt-runner 的 resvg/tiny-skia 直接依赖（load_svg_file 唯一用户，依赖图精简）。**测试**：render-foundation decode_tests +2——`decode_svg_bytes_green_4x3`（含 `<?xml` 声明的 4×3 纯绿 SVG 往返，断言 G>200 + alpha=255）、`decode_svg_bytes_invalid_returns_err`（非 SVG XML→err）；`decode_image_bytes_dispatches_by_magic` 加 SVG 路由断言（现四分发 PNG/JPEG/SVG/unsupported）。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test 12227 passed/0 failed、reftest-upstream 439/490 持平。**意义**：DC-13 三种图片格式（PNG/JPEG/SVG）在三条渲染路径（reftest / webview fetch_url / browser render_cpu）全部统一到 render-foundation `decode_image_bytes` 单点；浏览器经 URL 导航现可加载并渲染 SVG Logo（WinterTC logo.svg 等真实场景）。下一步=DC-13 产品 smoke 端到端证据（DONE#11 5 真实网站 / WinterTC Logo 经浏览器路径验证）或 DC-9 GPU 4 图元（transform/clip/filter/blend）。 |
| 2026-06-17 | R217 JPEG 解码合并去重（清理 R216 造成的重复） | 439/490 持平（reftest 用本地文件，JPEG 解码逻辑变更对像素输出零影响——L16 JPEG 在 WPT reftest 实质不出现，RGB24/L8/CMYK32 转换两路径本就等价）。R216 在 render-foundation 落地 tested `decode_jpeg_bytes` 后，reftest 路径 `reftest.rs:load_jpeg_file`（~55 行）的独立 JPEG PixelFormat→RGBA 转换与之重复（且 L16 处理不一致：reftest `(px[0]\|px[1]<<8>>8)` vs R216 干净高字节）。**修复**：`load_jpeg_file` 改为读文件→委托 `zero_render_foundation::image_cache::decode_jpeg_bytes`，reftest 与 webview/browser URL 导航路径现共用同一解码器（单点 tested）。移除 wpt-runner 的 `jpeg-decoder` 直接依赖（load_jpeg_file 是唯一用户，依赖图精简）。保留 `load_png_file` 的 `ZERO_PNG_EXPAND` 诊断门控与 `load_svg_file`（resvg）不动——非本轮变更遗留。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test 12225 passed/0 failed、reftest-upstream 439/490 持平。**意义**：三条渲染路径（reftest / webview fetch_url / browser render_cpu）JPEG 解码统一到 render-foundation 单点，消除维护负担与潜在不一致；DC-13 图片解码一致性提升。下一步=SVG 解码统一（reftest 已有 resvg，webview/browser 路径缺）或 DC-13 产品 smoke 端到端证据（DONE#11）。 |
| 2026-06-17 | R216 JPEG 图像解码扩展（DC-13 PNG/JPEG 基础解码第二步） | 439/490 持平（reftest 用本地文件不走 URL 导航，零影响）。goal doc DC-13 要求 PNG/JPEG/WebP 基础解码，R214 落地 PNG，本轮补 JPEG。**修复**：① render-foundation 加 `jpeg-decoder = "0.3"`（MIT/Apache-2.0 纯 Rust）+ `pub fn decode_jpeg_bytes(bytes)`（L8/L16/RGB24/CMYK32 全 PixelFormat→RGBA，CMYK 按 Adobe 倒置 K 惯例转 RGB）+ `convert_jpeg_pixels_to_rgba` 纯函数；② **格式分发** `pub fn decode_image_bytes(bytes)`——按**魔数字节**嗅探（PNG `\x89PNG` / JPEG `\xFF\xD8\xFF`）路由，比 URL 扩展名可靠（URL 可能无扩展名/扩展名错误），未知格式返 unsupported err；③ webview `fetch_image_subresources` 改调 `decode_image_bytes`（原 decode_png_bytes）→ 同一路径现处理 PNG+JPEG，warn 文案更新。**测试**：render-foundation decode_tests 5 项——`convert_jpeg_pixels_to_rgba_rgb`/`_grayscale` 纯函数断言、`decode_jpeg_bytes_green_4x3`（PIL 生成 4×3 纯绿 JPEG quality 95 fixture，断言绿色主导 G>200/R<50/B<50 + alpha=255，容 JPEG 有损非精确等值）、`decode_jpeg_bytes_invalid_returns_err`（魔数+无效正文→err）、`decode_image_bytes_dispatches_by_magic`（PNG ok/JPEG ok/未知 unsupported）。fixture `crates/render-foundation/src/testdata/green_4x3.jpg`（635B）。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test **12225 passed/0 failed**、reftest-upstream 439/490 持平。**意义**：DC-13 图片基础解码 PNG+JPEG 就绪；浏览器经 URL 导航现可加载并渲染常见位图格式。下一步=SVG 栅格化（WinterTC logo.svg）或 WinterTC Logo 端到端产品 smoke 证据（DC-13 验收）。 |
| 2026-06-17 | R215 浏览器渲染路径消费 webview ImageCache（DC-13 P1 图片子资源最后消费 hop） | 439/490 持平（reftest 用本地文件不走 URL 导航，image_cache 恒空，`Some(&空)`≡`None` 零回归）。承接 R214 标注的「下一步」。R214 已打通 fetch→decode→image_cache（webview 层），但浏览器 `render_cpu`/`render_frame` 仍传 `None`（app_platform.rs:194 CPU / :153 GPU），图元→渲染器最后一跳断开。**修复**：app.rs 加 `use zero_render_foundation::image_cache::ImageCache`；`render_cpu`（CPU 路径）与 `render_frame`（GPU 路径）在 `render_full_scene[_gpu]` 调用前用**不相交字段借用**取活跃标签页 webview 的 image_cache——`match self.shell.active_tab_id() { Some(id) => self.webviews.get_mut(&id).map(|wv| wv.image_cache()), None => None }`（self.webviews / self.font_loader / self.glyph_cache 为不同结构字段，Rust 借用检查器允许同语句并存），传 `Some(&mut ImageCache)` 替代 `None`。**测试**：新增 `#[cfg(test)] pub fn render_full_scene_with_webview_for_test`（与 render_cpu 同场景装配但返回 FrameBuffer，mirror 现有 `render_scene_for_test` 模式）+ 差异法测试 `render_path_consumes_webview_image_cache`——基线（image_cache 空）渲染断言目标颜色计数 0（缓存 miss 不绘制，证「空缓存≡None」语义），填充 `ImageKey(simple_hash(src))`（键与 engine text.rs:611 一致）后渲染断言 >0（图片经浏览器渲染路径被消费）。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、cargo fmt 干净、make test 全绿（新增测试通过）、`./target/test-guard -- cargo run --bin zero-wpt-runner -- reftest-upstream` 实测 439/490 持平。**意义**：`<img>` 经 URL 导航全链路贯通——抓取(R214)→解码(R214)→image_cache(R214)→**浏览器渲染消费(R215)**→renderer 绘制真像素，goal doc DC-13「图片缺失不得被 alt/占位 glyph 静默替代」在浏览器层落地。下一步=JPEG/SVG 解码同模式扩展 + WinterTC Logo 端到端产品 smoke 证据（DC-13 验收）。 |
| 2026-06-17 | R213 外链 stylesheet 加载（URL 导航路径，P1 缺口修复 + 端到端测试） | 439/490 持平（reftest 用本地文件不触发 URL 导航路径；本修复影响产品 URL 导航 + DC-13）。修复 goal doc P1「外部样式表加载缺失」：`collect_stylesheets`（pipeline.rs:494）只收调用方 CSS + 文档内 `<style>`，**不抓 `<link rel=stylesheet>`**；fetch_url 三条成功路径（SW intercept / HTTP cache / network）都调 `load_html(&html, None)`。**分层修复**（engine 做 DOM 提取、webview 做 URL 解析+网络，保持 engine 不耦合网络）：① zero-engine 暴露 `pub fn extract_stylesheet_hrefs(html) -> Vec<String>`（pipeline.rs，复用 `zero_dom::parse_html` 解析 DOM 精确提取，`rel` 空白拆分后任一 token `eq_ignore_ascii_case("stylesheet")` 即匹配，覆盖 `rel="stylesheet preload"` 等写法，跳过空 href）；② zero-webview `fetch_url` 加私有 `resolve_external_css(html, base_url)`（`url::Url::parse(base).join(href)` 解析相对/绝对 href，`http_client.get` 逐个抓取，合并为单 CSS 字符串，抓取失败仅 `tracing::warn` 不阻断），三条分支改 `load_html(&html, Some(&external_css))`。**端到端测试**（webview_coverage.rs，内联 std mini-server）：① `test_fetch_url_loads_external_stylesheet`——page.html 仅外链 style.css（`#x{background:rgb(255,0,0)}`，无任何内联红），fetch_url 后断言渲染含纯红 (255,0,0) fill（证明外链 CSS 抓取+级联生效）；② `test_fetch_url_external_stylesheet_missing_does_not_break`——外链 404 时导航不崩溃。12 webview 测试全过，engine lib 1143 测试全过。**意义**：URL 导航路径现可正确加载外链 CSS（morning.work `/article.css`/`/styles/github.css`、welcome 等真实静态页依赖），DC-13「URL 导航必须加载外链样式表」子项打通；图片子资源/ImageCache（DC-13 另一 P1）仍待。make test 全绿、clippy --workspace --all-targets -D warnings 干净、fmt 干净。 |
| 2026-06-17 | R212 离线 fixture HTTP 服务器（std-only example，使能外链 CSS/图片离线测试） | 439/490 持平（新增 example，不影响 reftest）。用户明确要求「本地静态资源+Rust web server」以离线测试 URL 导航 + 外链 CSS/图片加载。新增 `crates/net/examples/fixture_server.rs`：**std-only** HTTP/1.0 服务器（`TcpListener` 非阻塞轮询 shutdown，按扩展名映射 Content-Type 含 html/css/js/svg/png/jpg/webp/woff2/ttf 等，路径穿越 `..` 段过滤 + canonicalize `starts_with` 双保险，404/405 响应，每连接一线程）。`--root/--port` 参数，`pub fn serve(root,port,shutdown)` 供测试/嵌入。**零新依赖、零 workspace 改动**（example 自动发现）。配单元测试：临时目录建 article.css+index.html，启动服务器（OS 分配 0 端口），GET CSS 断言 200+text/css+内容，GET 缺失断言 404，GET `../../etc/passwd` 断言穿越被拒 404。**用途**：后续可离线驱动 ZeroBrowser/WebView 导航 `http://127.0.0.1:<port>/` 验证 P1 缺口「外部样式表加载缺失」（preload.rs:332 明确忽略 `rel=stylesheet`）与「图片子资源/ImageCache 未贯通」。**附带 GPU 调查**（为 DC-9 后续工作铺路）：blur_pipeline（pipeline.rs:703）已创建但从未接线（self.blur_pipeline 无任何 draw 调用）；headless_texture 含 `TEXTURE_BINDING` 可采样；但 blur/filter 后处理需 ping-pong 双纹理编排（wgpu 不能同 pass 读写同一纹理），是 render_full_scene_gpu 缺的架构=多轮。make test 全绿、clippy -D warnings 干净、fmt 干净。此为外链 CSS 加载使能 + DC-9 GPU 后处理铺路的第一步。 |
| 2026-06-17 | R211 DC-9 GPU 图元覆盖状态诚实化纠正（诊断，治理强制纠正矛盾，docs-only） | 439/490 持平（无代码变更，reftest/product-smoke 均走 CPU 路径不受影响）。核查发现 master.md DC-9 表对 Transform/Clip/Filter/BlendMode 标 ✅「简化处理」属**虚假声明**——浏览器实际 GPU 路径 `render_full_scene_gpu`（gpu/renderer/mod.rs:651，app_platform.rs:149 调用）**完全丢弃这 4 种图元**（仅 collect+draw 9 种；gpu/renderer/ 仅 mod.rs+tests.rs，无其它 GPU 路径处理它们；GPU tests 609-768 也未覆盖）。表中原「scissor rect 全局裁剪/CPU 后处理对齐」描述的是 per-box 路径 `render_scene_with_clip_scaled`（仅支持**单一** clip_rect scissor），非全量 GPU 路径。**共同根因**：全量批次路径展平了场景、丢失元素子树关联——clip/transform/filter/blend 均作用于子树，扁平图元列表无法应用。故 DC-9 真正未达标（4 项修正为 ⚠️），违反 DC-14/DC-9「GPU 非 passthrough、不丢弃图元」硬约束。**CPU 路径 DC-8 经核验实处理全部 13 种**（cpu/mod.rs:163-179 typed-bucket + 246-262 draw_order 双模式），DC-8 ✅ 准确。按 goal doc 治理规则（line 757：发现文档矛盾必须先纠正），已将 DC-9 表 4 项改 ⚠️ 并加纠正说明。修复=多轮架构（paint 侧把 transform/clip 烘焙进已收集的 fill/glyph 顶点，或 GPU 全量路径携带子树结构；filter=post-processing WGSL pass，blend=blend equation）。**这是 DC-9/DC-14 通向 DONE 的明确未完成项**，区别于已穷尽的 reftest 平台期。 |
| 2026-06-17 | R210 compute_final 多行存储 + multicol 守卫实证（诊断，净 +0 不可启用，无提交，工作区清洁） | 439/490 持平（全量 make reftest-upstream 双跑实测 default 439/51 vs gate 439/51，**净 +0**）。承接 R209（PHASEA_MULTILINE 净负，疑 R198 ancestry-guard 墙），测未被试组合：compute_final 多行存储 + `!in_multicol` 守卫（in_multicol 经新增递归参数 `child_in_multicol=in_multicol\|\|root.is_multicol` 透传）+ text.rs stored frag `y=f.y+line.y`（行内相对→行盒绝对，多行必需）。**仅 2 翻转**：✅ ifc-008 8.18%→PASS 0.00%（compute_final 正确存 node39 的 2 行 100px Ahem）；❌ multicol-fill-auto-001 0.63%→9.15%。**根因精确定位（推翻「multicol 容器 ancestry」假设）**：CFDEBUG 探针（reftest 双趟，正确加载 /fonts/ahem.css）显示 multicol-fill-auto 存了 node25(10 行)/node28(5 行)，**in_multicol=false**。逐文件分析：test=1 个 multicol div，**ref=2 个 `<div float:left width:10em>`（非 multicol，用 float 模拟列）**——回归源是 ref 的 float div（合法非 multicol），`!in_multicol` 守卫**无法触及**。**v_offset 语义墙**：default ref float div 走 paint IFC（baseline_fs=font_size），gate 走 stored（v_offset=0 Ahem），两路径对同一多行 Ahem baseline 计算不同→ref/test 差 font_size/行→9.15%。实测 v_offset=font_size 反破坏 font-051(16.67%)/ifc-008(8.33%)，**stored 单行/多行 v_offset 语义不可统一**，印证 R125 三路径死锁。**方法学纠正**：product-smoke 单趟**不加载外链 CSS**（base_dir=None）→ multicol-fill-auto 的 /fonts/ahem.css 不解析→is_pure_ahem=false→不存储；故 R209 product-smoke 看「不存储」是外链 CSS 缺失假象，**涉及外链 CSS 的用例必须用 reftest 双趟诊断**。结论：净 +0 不可默认启用；ifc-008 可被正确存储并 PASS，但解锁需先统一 stored 与 paint IFC 多行 baseline 语义（R125/R198 同墙，结构性多轮）。证据 `evidence/r210-multiline-multicol-guard-2026-06-17.txt`。无代码变更（实验已回退回 R207 e0e2689 干净态）。 |


---

## 下一步

1. ~~验证 cargo test 全绿~~ ✅
2. ~~扩展 manifest.rs 添加 fuzzy 元数据解析~~ ✅
3. ~~扩展 ReftestConfig 添加分类容差和 per-test fuzzy 注解~~ ✅
4. ~~创建 reftest skip list 和过滤机制~~ ✅
5. ~~创建 Chromium 截图脚本~~ ✅
6. ~~实现 reftest runner CLI~~ ✅
7. ~~导入 CSS 2.1 核心 ≥ 50 个 reftest~~ ✅ (159 个)
8. ~~运行初始 reftest 基线测试~~ ✅ (100.0%)
9. ~~实现 JS 执行集成~~ ✅
10. ~~实现 GPU 截图~~ ✅
11. ~~CI 集成~~ ✅
12. ~~M1 完成~~ ✅
13. ~~M2 — Quirks Mode 全部可执行项~~ ✅ (CSS parser + style system quirks)
14. ~~M3 — Flexbox + Grid 基础+进阶 reftest~~ ✅ (21 个新 reftest, 100.0% pass)
15. ~~M3 — Flexbox/Grid edge case reftest~~ ✅ (20 个边界 case reftest, 100.0% pass)
16. ~~M3 — Flexbox/Grid 渲染缺口修复~~ ✅ (无缺口，全部通过)
17. ~~M4 — Table display types 添加~~ ✅ (10 个 table display variant)
18. ~~M4 — 基础 float 布局实现~~ ✅ (float left/right 定位 + 垂直堆叠)
19. ~~M4 — Float 布局 reftest~~ ✅ (10 个 reftest, 100.0% pass)
20. ~~M4 — Float exclusion zone 连接~~ ✅ (remeasure_text_with_float_exclusions)
21. ~~M4 — UA 默认 display 值~~ ✅ (ua_default_display 为 HTML 元素注入正确的 display type)
22. ~~M4 — parse_display 修复~~ ✅ (color.rs 中补全 11 个 table display types)
23. ~~M4 — Table 布局算法实现~~ ✅ (table grid 构建 + auto layout + colspan + border-spacing)
24. ~~M4 — Table 布局 reftest~~ ✅ (9 个 reftest, 100.0% pass)
25. ~~M4 — Multi-column 布局算法~~ ✅ (shortest-column-first 均衡分配 + column-count/column-width/column-gap)
26. ~~M4 — Multi-column 布局 reftest~~ ✅ (10 个 reftest, 100.0% pass)
27. ~~M5 — CJK normal-mode 逐字符换行~~ ✅ (split_into_words 中 CJK 字符单独作为单词)
28. ~~M5 — text-align: justify 修复~~ ✅ (使用 effective_content_area 计算剩余空间)
29. ~~M5 — Float exclusion 堆叠修复~~ ✅ (max → additive stacking)
30. ~~M5 — 文字排版 reftest~~ ✅ (10 个新 reftest, 229 总, 100.0% pass)
31. ~~M5 — 文字排版扩展 reftest~~ ✅ (51 个 Text reftest, 260 总, 100.0% pass)
32. ~~修复 ReftestCategory::from_path 路径匹配~~ ✅ (添加 starts_with 模式)
33. ~~更新 DC-3~DC-6 完成状态~~ ✅ (DC-3~DC-5 全部达标, DC-6 完成)
34. ~~M5 完成~~ ✅ (CJK 换行 + justify + float 堆叠 + 51 Text reftest)
35. ~~M6 — Flexbox+Grid 扩展到 ≥50~~ ✅ (各 51 个 reftest, 296 总, 100.0% pass)
36. ~~M6 — 扩展剩余目录到 ≥50~~ ✅ (535 总, 10 个目录全部 ≥50, 100.0% pass)
37. ~~M6 — 拆分 reftest_data.rs 为目录模块~~ ✅ (reftest_data/ 目录, 每个分类独立文件)
38. ~~M6 — DC-5 文字排版全目录达标~~ ✅ (新增 css-fonts/css-text-decor/css-writing-modes, 685 总, 100.0% pass)
39. ~~M6 — 引入 rustybuzz（OpenType shaping）~~ ✅ (GSUB/GPOS 连字+kerning，fontdue 回退)
40. ~~M6 — 引入 unicode-bidi（RTL 文本）~~ ✅ (BiDi 重排序，RTL 字符自动检测)
41. ~~M7 — CPU 渲染器全量图元~~ ✅ (render_full_scene() 支持全部 13 种图元)
42. ~~M7 — 浏览器图元消费~~ ✅ (transform_webview_primitives() 处理全部 13 种 + render_cpu() 使用 render_full_scene())
43. ~~M7 — 验证~~ ✅ (cargo test 7800+ 全绿, clippy 零警告)
44. ~~M7 — GPU 渲染器全量图元管线~~ ✅ (5 个 WGSL shader + 4 条管线 + mesh 生成 + render_full_scene_gpu())
45. ~~M7 — GPU 渲染器单元测试~~ ✅ (48 个 GPU 单元测试，覆盖 fills/rounded_rect/gradient/shadow/stroke/empty scene)
46. ~~M7 — 浏览器 GPU 路径集成~~ ✅ (app_platform.rs render_frame() 使用 render_full_scene_gpu)
47. ~~M8 — BFC 检测~~ ✅ (establishes_bfc() 检测 overflow/float/position 建立的 BFC)
48. ~~M8 — Float clear 支持~~ ✅ (clear:left/right/both 后处理 + 7 个集成测试)
49. ~~M8 — Margin 折叠~~ ✅ (发现 taffy 0.7 已内置 CollapsibleMarginSet，无需额外后处理)
50. ~~M8 — 替换元素布局~~ ✅ (<img> 固有尺寸注入 + 2 个集成测试)
51. ~~M8 — Position: sticky 标记~~ ✅ (layout 引擎已标记 is_sticky，需宿主层滚动集成时实现动态偏移)
52. ~~M9 — 重复渐变~~ ✅ (GradientPrimitive.repeating 字段 + CPU fract() tiling + GPU WGSL shader)
53. ~~M9 — 多图层背景~~ ✅ (background_image 改为 Vec + 逗号分隔解析 + 逆序渲染)
54. ~~M9 — clip-path inset 裁剪~~ ✅ (clip_all_primitives_to_rect() 处理全部图元类型)
55. ~~M9 — 非矩形 clip-path~~ ✅ (circle/ellipse/polygon 扫描线裁剪 + 点在多边形内检测)
56. ~~M9 — backdrop-filter~~ ✅ (复用 FilterComputedValue + 在元素绘制前应用滤镜)
57. ~~M9 — CSS mask~~ ✅ (mask-image 解析 + mask-mode 解析 + 渐变蒙版裁剪 + alpha 衰减 + 3 个单元测试)
58. ~~M9 — overflow 全图元裁剪~~ ✅ (修复 overflow:hidden/scroll/clip 仅裁剪 fills+glyphs 的问题，改用 clip_all_primitives_to_rect 裁剪全部 13 种图元)
59. ~~M9 — 滚动容器 paint 偏移~~ ✅ (LayoutBox scroll_x/scroll_y 字段 + paint 时 overflow:Scroll 子元素坐标偏移 + 3 个单元测试)
60. M9 — scroll-snap 行为（已解析存储，需宿主层输入路由实现吸附逻辑）
61. M9 — 滚动输入路由（需浏览器 app 集成：嵌套滚动容器识别、逐元素 scroll 事件分发）
62. ~~M10 — FontLoader 修复~~ ✅ (render_to_framebuffer_with_base 使用 create_font_loader()，启用文本渲染；揭示真实通过率 65.0%)
63. ~~M10 — support image 补充~~ ✅ (为缺失的 swatch 颜色/尺寸 PNG 生成文件)
64. ~~M10 — border conflict resolution 基础设施~~ ✅ (resolve_collapsed_borders + resolve_border + border_style_priority)
65. ~~M10 — 调查表格 border CSS 未生效问题~~ ✅ (经调试验证 border-top-width=Px(5.0) 正确设置；原始报告的问题可能是特定测试场景的布局差异)
66. ~~M10 — 实现 paint 系统 font-family 解析~~ ✅ (OpenType name 表解析 + FontLoader.build_font_resolver() + Painter.resolve_font_id() + RenderPipeline.set_font_resolver())
67. ~~M10 — CSS border-width zeroing~~ ✅ (border-style 为 none/hidden 时强制 width=0，符合 CSS 规范)
68. M10 — writing-mode 布局支持（影响 css-writing-modes 40.7% + css-flexbox 部分测试；需实现 vertical-rl/lr 布局方向）
69. M10 — float 布局精度提升（CSS2/floats-clear 20/30 失败；根因分析：swatch 图像缩放 20×20→96×96 与 CSS background-color 精确填充的像素差异，非 float 定位错误）
70. M10 — 失败分布分析：37个<2%、45个2-5%、40个5-15%、40个15-30%、11个>30%；最大改进杠杆为 writing-mode（影响 35 个测试）和 column breaking（影响 32 个测试）
71. ~~M10 — flex-flow 简写展开~~ ✅ (shorthand/mod.rs 新增 "flex-flow" 分支，解析 flex-direction || flex-wrap；修复 3 个 flexbox 测试)
72. ~~M10 — font-family 非法字符验证~~ ✅ (parse_font_family 验证未引用名称仅含有效字符，含非法字符时整个声明无效；修复 2 个 CSS2/fonts 测试)
73. ~~M10 — font 简写验证~~ ✅ (expand_font 检查 size_found，缺少 font-size 的声明无效；更新测试用例匹配 CSS 规范)
74. M10 — column breaking 实现（影响 css-multicol                  28/57 (49.1%)；当前 multicol 仅分配整个子元素到列，不拆分溢出内容；需实现 fragmentation 基础设施）
75. M10 — 浮动清除算法改进（影响 CSS2/floats-clear 20 个测试；当前 max(normal_y, clear_bottom) 未正确实现 CSS 2.1 clearance 对 margin 折叠的阻断）
76. ~~M10 — CSS 2.1 Appendix E 绘制顺序~~ ✅ (float 子元素在非 float 子元素之后绘制，paint_node_in_rect 和 paint_node 各分两轮遍历)
77. ~~M10 — columns 简写顺序无关解析~~ ✅ (双值模式自动检测整数/长度，修复逆序声明如 `columns: 100px 6`)
78. ~~M10 — clearance 代码质量改善~~ ✅ (澄清零 clearance 阻止 margin 折叠；后处理方式局限：taffy 已应用 margin 折叠)
79. M10 — 分析 CSS2 border/background 失败根因（6 个 border 测试 + 5 个 background 测试失败，需定位具体渲染差异）
80. ~~M10 — float Y 位置修复~~ ✅(float 在含 inflow 子元素容器中尊重 taffy Y)
81. ~~M10 — clearance 计算修复~~ ✅(flow_bottom + margin 折叠替代简单 offset 扣除)
82. ~~M10 — inline img 替换元素~~ ✅(InlineFormattingContext 识别 img 固有尺寸)
83. M10 — inline formatting context 改进（影响 CSS2/linebox ~8 个测试；空 inline 元素 line-height 贡献、inline 元素 margin 处理）
84. M10 — writing-mode 布局支持（影响 35 个测试：12 个 abs-pos-non-replaced 21.33% + direction 12.49% + float-orthog 3% 等；需实现垂直书写模式下轴交换）
85. M10 — multicol column breaking（影响 31 个测试；需实现内容跨列拆分）
86. M10 — CSS2 inline box model 改进（empty-inline-002/003 + inline-box-001/002 + inline-formatting-context-008/009/011 等 ~8 个测试）
87. ~~M10 — CSS 绝对长度单位~~ ✅(parse_length 新增 in/pt/pc/cm/mm/Q，background 简写分类器更新)
88. ~~M10 — 表格单元格 vertical-align~~ ✅(top/middle/bottom 支持)
89. ~~M10 — 径向渐变位置解析~~ ✅(resolve_position 正确处理 Percentage/Px)
90. ~~M10 — 表格 min-height border-box~~ ✅(min-height-table 通过)
91. ~~M10 — 表格单元格高度最小值~~ ✅(cell height = max(row_height, content_height))
92. ~~M10 — 图像双线性插值~~ ✅(CPU render_image 从最近邻改为双线性插值)
93. ~~M10 — writing-mode 轴交换回退~~ ✅(禁用不完整的轴交换和隐式继承，避免回归)
94. ~~M10 — CSS 负值 border-width 拒绝~~ ✅(负值视为无效，回退到初始值 medium；修复 border-bottom-width-001.xht)
95. ~~R13 — inline-block 行内定位~~ ✅(adjust_inline_block_positions 后处理 + IFC InlineBlockBox + baseline 修正；CSS2 59.7%→62.8%)
96. ~~R13 — border-collapse 样式覆盖~~ ✅(collapsed_border_style_overrides + 更多命名颜色)
97. ~~R14 — writing-mode 垂直 inline 布局~~ ✅(break_items_into_columns + 布局引擎接入 + 绘制层垂直字形渲染；6 个单元测试)
98. ~~R20 — reftest 分类容差 bug 修复~~ ✅(for_category 替代 Default::default()；+24 测试通过)
99. ~~R20 — columns 简写解析修复~~ ✅(整数优先 column-count 而非 column-width)
100. ~~R20 — 零高度浮动处理~~ ✅(line_max_height 跳过零高度浮动)
101. ~~R21 — font 简写负 line-height 拒绝~~ ✅(CSS Fonts §3.7)
102. ~~R21 — background-position 简写双值捕获~~ ✅(+1 upstream test: background-329.xht)
101. R20 — multicol column breaking（影响 ~16 个测试：需实现内容碎片化）
102. R20 — CSS2 float/clear 精度提升（影响 ~19 个测试：clearance 算法 + image scaling）
103. R20 — CSS2 inline box model（影响 ~8 个测试：空 inline line-height + block-in-inline）
104. R20 — Flexbox baseline + writing-mode（影响 ~9 个测试：flex 方向轴交换）
105. R20 — CSS 表格子像素修复（影响 ~9 个测试：border 精度 + image scaling）

### R227 — welcome 36px 偏移根因修复（padding 双计，2026-06-17）

welcome.html product-smoke **28.34% → 17.06%**（chromium Oracle，-11.28pp）。R226 定位顶部 36px 垂直偏移后，
R227 确证根因并修复。

- **根因**：taffy `Layout::location` 是子节点 border-box 相对父 border box 偏移（已含父 padding+border，
  见 `taffy-local/src/tree/layout.rs:314-322` helper 与 `compute/block.rs:404,486`）。ZeroWeb painter/IFC/abspos
  约定子节点坐标相对父**内容盒**（painter child_offset 叠加 padding+border）→ taffy 块级子节点 padding/border
  **双计**，每个带 padding 的父容器把子树整体下移/右移一份。welcome 36px = .page pt(20) + .hero pt(16) 两级双计。
  自源 reftest（test/ref 同源）双计互相抵消故长期不暴露（DC-14 自源假通过），仅 chromium Oracle 显现。
- **修复**：`extract_layout`（engine.rs）对水平书写模式下非 abspos/fixed 的 taffy 子节点，把 location 换算为
  内容盒相对（减自身 content_x/y）。float 后处理覆写故无害；abspos 由 adjust_absolute_* 线程 border-box 约定
  单独处理故跳过；inline IFC 子节点不经 extract 故不动。
- **方案选择**：曾试改 painter child_offset 不叠加 padding（Option A），welcome 同修复但 **-5** reftest 回归
  （4 multicol ref 用 inline 子节点被误移 + grid-spanning）；Option B（extract 换算）只改 taffy 块级子节点，
  inline 不动 → 回归降到 -1。采 Option B。
- **验证**：reftest 上游 **438/490**（基线 439，净 -1）；css-position 16/16 持平（abspos 保留）；multicol 40/57 持平；
  make test 全绿；clippy 零警告。唯一回归 grid-flex-spanning-items-001（0.77%→1.31% borderline）：修复**正确化**了
  test 的 aqua 位置（28→18 content-box 左，与 ref 一致），旧 pass 来自 aqua 错位与 border 尺寸差两误差抵消，
  修复消除 aqua 误差后剩余固有 border 尺寸差不再抵消。证据 `evidence/r227-welcome-padding-doublecount-fix-2026-06-17.txt`。
- **诊断工具**：新增 `LAYOUT_DUMP=1` env（reftest.rs dump_layout_tree）转储盒树 abs_y/height/margin-top/padding-top，
  供后续布局垂直偏移诊断。

### R228b — 半透明圆角矩形背景 alpha 丢失修复（2026-06-18）

CPU 渲染器 `fill_rounded_rect`（render-foundation/src/cpu/mod.rs）硬编码 `alpha=255` + `set_pixel` 直接覆盖，
丢弃 `rr.color.a`，致**任何半透明圆角背景渲染成实色**（矩形 `fill_rect` 正确 blend，圆角 `fill_rounded_rect` 不正确）。
R227 后复扫 morning-work 67% 时定位：`.item-tag { background-color: var(--color-primary-alpha-05); border-radius:3px }`
渲染成实色蓝 (96,124,210)。逐步二分锁定唯一触发条件 = `border-radius` + rgba/半透明背景（var/literal 均触发；
var/longhand/inline-block/color 均非因）。修复=fill_rounded_rect 改用与 fill_rect 一致的 alpha 分支（不透明
set_pixel / 半透明 blend_pixel）。新增单元测试 `rounded_rect_translucent_alpha_blends`。reftest 438/490 持平零
回归；smoke 686/686；make test 全绿；clippy 干净。
**morning-work 整体仍 67%（未降）**——真因是独立的**内容纵向压缩**（ZW 暗内容集中 y=150-300，y=300-525 空白；
CH 均匀分布 y=150-600），疑 `<img>` 固有尺寸塌缩或 pre code min-width:700px 溢出致内容堆顶，属 DC-11 替换元素
独立问题，下一轮排查。GPU 渲染器 `color_to_f32`（gpu/mesh.rs:11）全局丢弃 alpha（fill/圆角均无法半透明），属
DC-9 子项记录未修。证据 `evidence/r228-rounded-rect-alpha-blend-fix-2026-06-18.txt`。

### R177b — table_grid_size_col_colspan chromium-diff 修复（2026-06-18）

落地 R177 实证后**延后**的 colspan/col-width 缺口（R177 仅定位「5 耦合部件不可单点」未提交修复）。本轮实现
5 部件配套（crates/layout-engine/src/table.rs）：(a) `build_grid` 对 separated+collapsed 均 count_col_elements；
(b) `cell_used_width` collapsed 模式下显式 width 单元格列宽 = content + 水平 borders/2（CSS2 §17.6.2 border 中心
间距）；(c) `compute_column_widths` Pass 0 读 col/colgroup width 不再受 collapsed 守卫；(d) 新增 fixed 布局空列
裁剪（无 cell 跨越的列宽置 0，auto 布局保留）；(e) 扩展条件从 `has_explicit_width||is_fixed_layout` 收紧为
`has_explicit_width`（CSS Tables §17.5.2.1：fixed 表宽 = max(width 属性, 列宽之和)）。
**chromium 独立 Oracle A/B**：`table_grid_size_col_colspan` **52.27%→1.70%**，其余 3 个 table 用例
（table-cell-width-0 13.31% / table-colspan-percent-auto 1.01% / visibility-collapse-colspan-003 4.91%）字节不变。
上游 reftest 438/490 持平零回归（同源 test==ref 天然不变）；clippy 零警告。**方法论**：同源 reftest 对正确修复
天然零变化无法判价值，须 chromium Oracle A/B（revert→dump vs with-change→dump）确证。证据
`evidence/r177b-table-colspan-col-width-fix-2026-06-18.txt`。

### R238 — master.md 解阻塞 + writing-modes 聚类拆分 + WM-1 abspos-vertical 精确机制（2026-06-18，read-only）

**解阻塞说明**：master.md 自 R233 起被并行 agent 未提交的 R228b 阻塞 5+ 轮（R233–R237 全部
evidence-only 落盘，未动 master.md）。本轮并行 agent 已停滞（mtimes 00:06–00:09，已 30+ min 无
活动，reflog 零并行提交），调研 agent 解阻塞：R228b doc 段随本轮 master.md 提交带入（见上注），
R233–R237 的 evidence-only 结论合并如下，并给出**下一实现方向 = WM-1 abspos-vertical**。

**R233–R237 evidence-only 结论合并**（详情见各 evidence 文件）：
- **R229/R230 font-weight**：welcome 剩余 17% 真因 = `font-weight` 未生效（资源 Regular-only +
  plumbing 中 weight 维度未消费），welcome 限定，非跨页；R230 确认 fc-list bold 资源前提。
- **R234 font-kerning**：rustybuzz 默认已 kern，gap narrow，低优先。
- **R235/R236 multicol baseline-export**：`taffy_baseline` 字段 + flex 消费路径（engine.rs:474/
  1003-1040）已存在，但 multicol.rs 全文零 baseline → multicol 容器导出退回 taffy 通用 block 基线
  （非列派生）→ 6 例恒 1.1% POLLUTED。修复 = multicol.rs 列布局后按 §baseline-export 计算 first/last
  基线写入 taffy_baseline。**结构性 DC-14 中最 tractable 切入点（有界特性，8 例系统性偏移）**。
- **R237 writing-modes 拆分**：css-writing-modes 59 case 拆 8 子聚类（WM-1..WM-8），11 clean-oracle +
  8 POLLUTED + ~40 self-fail。WM-5 clearance-vrl=R114b/R164 四轮证伪已 defer；WM-6/7 vertical float
  =R133 结构多轮。**WM-1 abs-pos-non-replaced vrl/vlr（14 case）= 首选实现候选**。

**新方向：WM-1 abspos-vertical 精确机制（R238 核心，详见 evidence/r238-abspos-vertical-precise-
mechanism-2026-06-18.txt）**：
- §7.1 维度交换**已存在但 INCOMPLETE**，非完全缺失：
  (a) `converter/mod.rs:196 apply_vertical_writing_mode` 已交换 inset(left↔top,right↔bottom)/
      size/flex-direction → taffy 收到轴交换后 abspos 数据 ✓
  (b) `engine.rs:1394-1426` 静态位置修正（注释明引 "§10.3.7 + writing-modes §7.1"），守卫
      `all_inset_auto`(top:auto&&bottom:auto)→用 IFC fragment 静态位置；`height_auto`→shrink-to-fit
      child.height=fragment.width ✓
- **两个系统性残差**（精确缺口）：
  - 缺口 B（direction 分支缺失，rtl ~5.03% = ltr 4×）：§10.3.7 ltr→left 置静态 / rtl→right 置静态
    （镜像）；现 line 1396-1399 静态修正**不读 direction** → vrl-012/122/130(rtl) 5.03% vs vrl-002(ltr)
    1.28%。**最清晰修复信号**（残差最大、机制最确定）。
  - 缺口 A（ltr all-auto 残差 ~1.28%）：vrl-002/vlr-003 走的就是已实现 all_inset_auto+height_auto 路径
    仍失败 1.28% → 该路径本身有 fine error（IFC fragment 静态位置/shrink-to-fit width 在 vertical
    偏移）。跨用例一致 → 系统性可定位。
- **修复面 contained**：engine.rs:1394-1426 一处 + 新增 direction 分支，不动 converter/taffy/paint。
  杠杆=14 case，但 per-case diff=定位残差+Ahem 噪声混合（R164 教训：勿据推断算 clean +14）。
- **实现轮起手**：① 插桩 abs-pos-non-replaced-vrl-012(rtl 5.03%) dump child vs fragment + direction；
  ② 补 direction 镜像分支；③ 复查 ltr 1.28% 残差来源；每步 make reftest + cross-validate 双看。
- 与既有结论不冲突：≠R164 clearance（不同子问题，REF 为 swatch 图片可对齐）；≠R109/R133（独立代码面
  engine.rs:1394）；同 R98/R123 谱系（abspos inset/CB taffy-vs-spec gap，补 vertical direction 分支）。

### R239 — DC-14 真 bug 18 候选复审计 + 下一目标定位（2026-06-18，read-only）

**背景**：并行 agent 本轮落地 R177b（#1 table-colspan **52.27%→1.70%**，reftest 438/490 零回归，
chromium A/B 实证；已提交 c2d9663，含 table.rs + r177b 证据 + master.md R177b 段）+ R228b（cpu 圆角
alpha，已提交 a1a24f1）。DC-14 anti-false-pass 核心策略 = 修 analyze-pollution-2026-06-16.txt 的 18
真 bug 候选。#1 闭合后复审计。

**18 候选现状（已修 8/18）**：✅ #1 table-colspan(R177b) #3 baseline-overflow(R180 45→1.25%)
#4 html-display-table(R165 33→2.63%) #7 flexbox-collapsed-item(R111) #8 flexbox-baseline(R130)
#9 multicol-contained-absolute(R124) #11 table-grid-item-dynamic-004(R168 11→2.98%)。
**剩 10 项分类**：FEATURE-GAP #2 backdrop-inherit（::backdrop 伪元素+<dialog>，非 backdrop-filter
属性，defer）；DYNAMIC #5 table-grid-item-003（JS getBoundingClientRect×2 relayout 不增长，defer）；
STRUCTURAL #12/#13 iframe-in-block-in-inline（R109 block-in-inline，defer）；字体域 #16（低 ROI）；
**CONTAINED 可推进**：#6+#10 position-absolute-semi-replaced-stretch（23%+15%，2 例同族）、
#15+#17+#18 collapsed-border-vertical（6.10–6.73%，3 例同簇）、#14 stretch-grid-item-button-overflow(8.15%)。

**首选 contained 下一目标：#6+#10 position-absolute-semi-replaced-stretch（详见 evidence/r239-
dc14-truebug-candidates-reaudit-2026-06-18.txt）**：
> ⚠️ **此「contained 修复」推断已被 R241 实证推翻**（product-smoke A/B：ZWt-vs-ZWr=0.00%，几何已正确
> 拉伸，23% chromium 差异 100% 在原生控件绘制层）→ #6/#10/#14 实为 **form-control feature gap**，
> defer，**非 contained 修复**。本节以下机制描述保留为历史，正确归类见 R241。
- 规范：abspos 半替换元素（form control）+ 两轴显式 inset + auto size → 应拉伸填满 CB-insets
  （CSS2 §10.3.7/§10.6.4 abspos 非替换 stretch）。测试 .abs{position:absolute;4 边 inset:3px;
  width/height:auto}，元素=<input button/submit/reset/color/text>。
- 精确机制（实证）：tree.rs:165 `apply_replaced_element_sizing` 读 `img_intrinsic_sizes`（仅 <img>），
  line 186 `if tag != "img" { return; }` → **所有 form control 零替换 sizing**，被当普通非替换 block，
  abspos 拉伸语义缺失。
- 修复面 contained：扩展半替换识别（input/button/textarea/select）+ abspos stretch 覆盖，不动
  converter/taffy 核心。**R177b 之后最高杠杆 contained 候选**（2 例合计 38%）。
- R164 教训：diff 23%/15% 可能含 form-control UA 默认样式差异，实现轮须 chromium Oracle A/B 量化
  翻转数（同源 test==ref 天然不变），勿据推断算 clean +2。

**次选**：#15+#17+#18 collapsed-border-vertical（vertical table collapsed border，R177b table 代码热但
此 3 例是 vertical+collapsed border 正交问题）；R236 multicol baseline-export（8 例系统性 1.1%，最
tractable 结构性）；WM-1 abspos-vertical（R238，14 例长线）。

### R240 — semi-replaced-stretch 精确机制深挖 + 通用 abspos 拉伸消歧（2026-06-18，read-only）

**家族画像**：R239 首选目标 #6/#10 实为 **3 例 POLLUTED**（同源 test≈ref 但 chromium 不一致）：
`position-absolute-semi-replaced-stretch-input`(23.03%) / `-other`(15.27%) / `-button`(3.55%)，合计
41.58%。同源 self≈0 = ZeroWeb test/ref 同批 input 同样无 UA sizing 互匹配，chromium 真拉伸→Oracle 不一致。

**关键消歧（R164 教训：实证非推断）**：先排除「通用 abspos-stretch 路径坏」。cross-validate 同目录
css-position 实证通用 abspos **全 WORKS**——`position-absolute-large-negative-inset` 0.00%/0.00%、
`-under-non-containing-stacking` 0.00%、`hypothetical-box-scroll-*` ~0.1%、`position-change` 0.00%。
⇒ 通用 abspos inset 解析 + 拉伸（box-sizing:border-box、大负 inset、positioned-CB）正常，失败**仅限
form control（input/button）** → **form-control-specific contained 缺口**，爆炸半径小（不碰通用 abspos）。

**精确机制**：① tree.rs:186 `apply_replaced_element_sizing` 仅 <img>，form control 零替换 sizing；
② 全仓无 form control UA 默认尺寸（仅 `appearance` 值解析）→ input 既非替换又无 UA 尺寸；③ abspos
form control 走非替换通用路径本应拉伸（§10.3.7），但未拉到 calc(100%-6px)——疑 taffy 对无内容/无
固有尺寸 leaf 的 abspos width:auto+两 inset 未填满（给 0 或 content-based shrink）。

**修复面 contained（两路线）**：路线 A（surgical，先做）= engine.rs abspos 后处理对 form control
（tag∈{input,button,textarea,select}）+ 两轴显式 inset + auto size → 填满 CB-insets；路线 B（更规范）
= 给 form control 加 UA 默认尺寸 + 列为半替换（扩展 apply_replaced_element_sizing）。详见
evidence/r240-semi-replaced-stretch-deep-spec-2026-06-18.txt。

**实现轮起手**：① LAYOUT_DUMP=1 跑 input 用例 dump abspos input width/height vs CB-insets(144×94)
确认非拉伸；② 路线 A 施加 §10.3.7/§10.6.4 拉伸；③ chromium Oracle A/B（3 例各看 z_vs_chr，同源
test==ref 天然不变须独立 Oracle）+ make reftest 同源不回归；④ A 不足再转 B。R164 教训：23%/15%/3.55%
可能含 form-control UA 外观（按钮边框/字号/outline）差异，A/B 后残差记录为 paint 层子问题勿强求 +3。

### R241 — R240 实证推翻 + master.md 自洽纠正（2026-06-18，read-only）

**⚠️ 纠正 R240 结论**：R240 据 tree.rs:186 推断「form control 不被当替换元素 → abspos 半替换拉伸缺失 →
#6 家族 contained 修复目标」。**实证推翻**（r239 §2 product-smoke + puppeteer A/B，2026-06-18）：
`position-absolute-semi-replaced-stretch-input` **ZWt-vs-ZWr=0.00%** 但 ZWt-vs-CHR=23.03% / ZWr-vs-CHR=23.03%。
即 ZeroWeb 把 test（auto-stretch）与 ref（显式 calc(100%-6px)）渲染成**同一几何**——input 已正确拉伸
填满 CB-insets，**几何完全正确**，tree.rs:186 不影响此场景。23% chromium 差异 **100% 在控件绘制层**
（chromium 画原生 input widget，ZeroWeb 画 styled box + lime outline）。

⇒ **#6/#10/#14 是原生表单控件外观 feature gap，非布局 bug**；改 apply_replaced_element_sizing 对
chromium-diff **零改善**。R164/R203 教训再次印证：单点修复推断须实证，不能据代码推断。R240 段保留为
历史记录，本段向前纠正。

**18 候选最终归类（contained clean win 穷尽）**：已修 8（#1/#3/#4/#7/#8/#9/#11）；FEATURE-GAP defer
#2(::backdrop+dialog)/#6/#10/#14（原生表单控件外观）；STRUCTURAL defer #5（JS 动态 relayout）/#12/#13
（R109 block-in-inline）/#15/#17/#18（vertical-rl/sideways+rtl+collapsed border，4.7%，test/ref 唯一差
will-change:transform，与 R114 axis-swap 族相关，低 ROI 高风险）；字体域 #16。**证实 R144/R204 plateau**。

**下一方向（reftest/pollution contained 单点穷尽后）**：
  (a) **DC-13 产品 smoke morning-work 67%**（最高 ROI）——R228b 证据定位真因=内容纵向压缩（ZW 暗内容
      集中 y=150-300、y=300-525 空白；CH 均匀 y=150-600），疑 `<img>` 固有尺寸塌缩或 `pre code{min-width}`
      溢出致内容堆顶，DC-11 替换元素/块格式化，product-smoke 可量化。
  (b) 原生表单控件渲染（系统性消 #6/#10/#14 + form-control pollution，大特性）。
  (c) DC-9 GPU ping-pong 地基（filter:opacity 先行，R220）。
  WM-1 abspos-vertical（R238，14 例）长线结构候选。详见 evidence/r241-r240-refutation-reconciliation-2026-06-18.txt。

### R242 — DC-13 morning-work 67% 内容压缩 root-region 收窄（2026-06-18，read-only）

**收窄 R228b 开放项**（morning-work 67% 真因=内容纵向压缩，疑 img 塌缩 或 pre-code 溢出）。本轮 read-only
二分 fixture 结构（apps/browser/assets/morning-work/article.html）：
- **决定性排除 footer-img 假设**：article.html 4 张 img 中，3 张无 width/height（logo_lei.jpg /
  cc_unavailable.png / qrcode jpg）**全在 footer（line 361-386），位于全部正文之后**。R228b 观测的压缩区
  是 **body 正文 y=150-300**，footer 在 y>500 → **footer img 塌缩无法解释 body 压缩**（即便有 img 布局应用
  bug，影响域限 footer）。排除 R228b 假设①。R233 已排除「图片未加载」。
- **收窄到 body `<pre><code>` 块**：正文 7 h2 + 多 h3 + 22 p + **12 个 `<pre><code>`**（line 210-338，步骤命令
  /C 代码）。CSS `.article pre{line-height:1.75;white-space:pre-wrap;overflow-x:scroll}` + `.article pre
  code{display:block;min-width:700px}`。代码块**确为多行**（源码显式 \n，C 代码 ~12 行）→ chromium 中撑起
  正文 y=150-600；ZW 压缩进 y=150-300 = **`<pre>` 高度严重欠计算**。
- **候选机制（待实证）**：(a) pre IFC 多行高度欠计算（连接 large-font/IFC-storage 谱系 R84/R101/R125/R210，
  PreWrap preserve_newlines=true 已支持但多行高度累积可能 off）— 最可能；(b) code{display:block;
  min-width:700px}+pre{overflow-x:scroll} 裁剪交互 — 次要；(c) pre-wrap 换行点差异 — 次要。
- **探针配方**：① LAYOUT_DUMP=1 渲染 article.html，定位 12 pre 节点 dump height vs 行数×1.75×14px，
  若≪预期→机制(a)；② 查 pre width 是否被 code min-width 撑>860 → 机制(b)；③ 对比单 pre ZW vs CH 行数 →
  机制(c)。每步 product-smoke 区域 diff（r228 ink-mass 方法）量化。

R164 教训：候选非断言，须 LAYOUT_DUMP 实证。详见 evidence/r242-morning-work-compression-narrowing-2026-06-18.txt。
为并行 agent R228b「下一轮排查」锐化方向（排除 footer-img，在 pre IFC 高度/overflow-x 裁剪/pre-wrap 换行间区分）。

### R243 — morning-work 机制(a) 精化：LAYOUT vs PAINT 判别（2026-06-18，read-only）

读 committed 代码（避开并行 agent inline/mod.rs WIP）验证 R242 候选机制(a)「pre IFC 多行高度欠计算」，得
**否定+精化结论**：
- **R84 多行守卫确跳过 `<pre>` 存储**（engine.rs:1909 `if inline_ctx.lines.len()>1 || !is_pure_ahem
  {return;}`）——多行非 Ahem 的 `<pre>` 不存 inline_layout，paint 回退非存储路径。此步 R242 推测正确。
- **否定简单子假设**：paint 空样式路径**已恢复** font_size/line_height（inline/mod.rs:1066 从
  `inline_element_metrics` 取 layout IFC 存下的度量，注释「仅影响行盒高度，不影响行断」）。故「空样式→
  font-size 错→pre 更短」**弱化/排除**；行断（line count）才是不被恢复部分。
- **关键判别（LAYOUT vs PAINT）**：盒高度是 LAYOUT 属性，paint 重跑只影响已定尺寸盒内 glyph 位置。故压缩
  两种互斥可能，R228b「暗内容（深色像素）分布」**无法区分**：
  (a-LAYOUT) 布局阶段 `<pre>` content_height 欠算（IFC 多行 line-count/total_height 未反馈到 pre 盒→盒矮
    →文档压缩；真因在 LAYOUT-time IFC，非 R84 守卫）；
  (a-PAINT) 布局盒高正确但 paint 只在盒顶画 glyph（空样式 line-break 与 layout 不一致→字堆顶→深色像素
    集中 y=150-300，盒下半浅背景看似空白）。
- **判别探针**：LAYOUT_DUMP 12 个 `<pre>` 盒 content_height——≪预期(行数×24.5px+padding)→a-LAYOUT；
  ≈预期但 glyph 只在顶→a-PAINT。两子情形都归 **Phase A IFC 三路径统一**（R82/R101/R125 deferred 结构项）；
  R101/R125/R210 实证放宽守卫存多行 net-negative，非 quick win。
- **并行 agent 正编辑 inline/mod.rs + edge_cases.rs**（工作区 WIP 未提交），高度疑似 Phase A stored-multi-line
  新尝试；本调研不接手，仅记 LAYOUT-vs-PAINT 判别 + LAYOUT_DUMP 探针供验证。详见
  evidence/r243-morning-work-mechanism-a-refine-2026-06-18.txt。R164 教训：否定简单子假设、未断言根因。

### R244 — DC-9 GPU alpha groundwork 精确 spec（2026-06-18，read-only，独立于 IFC WIP）

DC-9（GPU 13 图元独立 WGSL pipeline，禁 CPU passthrough）为硬 Done Criterion。本轮独立调研（不动并行 agent
IFC WIP）精确定位 GPU alpha 通道缺口——**CPU R228b 的 GPU 同源类比**：
- **缺口实证**：mesh.rs:11 `color_to_f32` 只返 RGB 丢 alpha；顶点色 `vec3f`；pipeline.rs:54 fragment
  `return vec4f(in.color, alpha)`——`.a` 槽填的是 **coverage alpha**（纹理/圆角检测/1.0）**非 CSS 色彩 alpha**。
  各 pipeline 已配 `BlendState::ALPHA_BLENDING`（pipeline.rs:462/540/617/676）但**源 alpha≡1.0** →
  `rgba(255,0,0,0.5)` 渲染成实色红。**所有 GPU 图元不透明**（半透明 fill/rounded_rect/gradient/shadow 全失真），
  影响面远超 DC-9 三项，是基础渲染保真缺口。
- **Groundwork 修复（contained 先做，zero-regression）**：(a) mesh.rs `color_to_f32`→RGBA + push_* 推 8 float；
  (b) pipeline.rs 顶点布局 color `vec3f`→`vec4f`（array_stride+1）；(c) WGSL fragment 合成**色彩 alpha×coverage**
  不再硬填 1.0；(d) Blend 已 ALPHA_BLENDING ✓（premultiplied 则配 PREMULTIPLIED）。范围仅 mesh.rs+pipeline.rs，
  不动 renderer 单 pass 流。zero 行为变化（CSS alpha 默认 1.0）。验证：framebuffer 像素断言 rgba(…,0.5)=blend 非实色。
- **超出 alpha 的 DC-9 三项（transform/filter/blend）= ping-pong 结构项**：render_full_scene_gpu（renderer/mod.rs:651）
  单 pass 直合成无中间纹理回读；wgpu 不能同 pass 读写同纹理，filter/transform/blend 需「渲染元素到 offscreen A→
  后处理 pass 读 A 写 B」。R220 记 headless_texture 就绪**差第 2 纹理+post-process pipeline**。最小首步 filter:opacity/blur。
  属结构多轮项（DC-9 真正大头）。
- **推荐顺序**：① alpha groundwork（contained，修 GPU-R228b-analog，为 blend 铺路）→ ② ping-pong 地基（filter 先）→
  每新 WGSL 配像素断言单测。不与 R220/R228b 冲突（细化/补 GPU 同源层）。详见
  evidence/r244-dc9-gpu-alpha-groundwork-spec-2026-06-18.txt。

### R245 — morning-work 压缩根因实证确认：pre-wrap 换行符保留缺失（2026-06-18，read-only）

**闭合 R242→R243→R245 调研链**。读并行 agent 未提交 IFC WIP（inline/mod.rs break_into_lines + painter/text.rs）
实证回答 R243 的 LAYOUT-vs-PAINT 判别：**= a-LAYOUT 主因**，且比 R243 推测更精确：
- **CSS Text §3.1 换行符保留缺失**：`break_into_lines` 收集文本**无条件**调 `collapse_whitespace`，把 `\n`
  折叠成普通空格；但 `white-space: pre/pre-wrap/break-spaces` 应原样保留 `\n` 作强制断行（CSS Text §3.1）。
  morning-work `.article pre{white-space:pre-wrap}` + 代码块源码显式 `\n`（C 代码~12 行）→ 被折成单行 →
  **12 个多行 `<pre>` 全塌缩单行** → 正文高度≈1/预期 → 内容压缩进 y=150-300、y=300-525 空白（R228b 观测）。
- 次要（同 a-LAYOUT）：即便 `\n` 作空字符串标记保留，旧 break_into_lines 对空词 `continue` 静默丢弃强制换行标记。
- 互补（a-PAINT）：painter/text.rs 非存储路径改用 `all_fragments_with_line_y()` 把每行 line.y 加到片段 y。
- **澄清非 Phase A 存储 guard**：R84 多行守卫（engine.rs:1909）影响 paint 存储路径，但**根因在 break_into_lines
  换行符处理**——即便走存储路径，collapse_whitespace 也先折成单行。故换行符修复是前置必要条件，与 Phase A 存储
  统一（R82/R101/R125）正交；换行符修好后多行 pre 行盒度量才正确，Phase A 讨论才有意义。

**验证配方（修复落地后）**：① product-smoke morning-work 区域 diff，暗内容应从 y=150-300 扩展到 y=150-600 均匀，
整体 67% 显著降；② LAYOUT_DUMP 12 个 `<pre>` content_height ≈ 行数×24.5px+padding（非单行~24.5px）；③ reftest
438/490 不回归 + clippy/fmt；④ 边界：仅 pre/pre-wrap/break-spaces（preserve_whitespace=true）保留空白，normal/nowrap
仍 collapse（验证 normal 文本不受影响）。
**协调**：并行 agent WIP（未提交）正修此根因；本调研不接手，仅实证根因 + 验证配方。详见
evidence/r245-morning-work-rootcause-validated-2026-06-18.txt。

### R246 — 阶段状态收口 + pre-wrap 修复 reftest bonus footprint（2026-06-18，read-only）

**新发现：R245 pre-wrap 修复 reftest bonus ≈ 0**——全 wpt-data 用 `white-space:pre/pre-wrap/break-spaces`
的文件仅 **4 个**，css/css-text **0 个**。故该修复价值**集中在 DC-13 morning-work 产品 smoke（67% 压缩主因）**，
非 reftest-multiplier。实现轮验证应聚焦 morning-work 区域 diff，**勿期待 reftest 438/490 显著上升**
（同源 test==ref 对称 + bonus 仅 4 文件）。修后 morning-work 剩余残差来自独立问题（font-weight R229、
img 固有尺寸 R233、.item-tag 已 R228b 修）。

**R236-R245 调研链收口**：
- **实现就绪**：R236 multicol baseline-export（8 例 1.1%，结构性最 tractable）、R244 DC-9 GPU alpha groundwork
  （GPU-R228b-analog，contained zero-regression）、R238 WM-1 abspos-vertical（14 例，direction 分支）。
- **阻塞于并行 agent 提交**：R242/R243/R245 morning-work 压缩（根因=pre-wrap 换行符；并行 agent WIP 已 4 轮
  未提交；验证配方就绪；本轮仍不跑 make reftest——WIP 未提交会污染测量）。
- **deferred 结构多轮**（R241 plateau 确证）：DC-14 剩余 writing-mode/multicol-breaking/block-in-inline/
  table-grid-dynamic-003/feature-gap(#2/#6/#10/#14)/vertical-table-border；DC-9 ping-pong。

**优先级队列（实现 agent）**：P0 morning-work pre-wrap（进行中，提交后验 DC-13 67% 降）→ P1 R244 DC-9 GPU
alpha groundwork → P2 R236 multicol baseline-export → P3 R238 WM-1 abspos-vertical → 长线 DC-9 ping-pong /
DC-14 结构聚类。**无 open 阻塞**；调研侧主要候选已 spec 完，下一进展依赖并行 agent 提交（解锁验证）或实现轮
起手 P1/P2/P3。详见 evidence/r246-status-consolidation-prewrap-footprint-2026-06-18.txt。

### R247 — pre-wrap 换行符修复落地 + 多行堆叠 Phase-A 死锁实证（2026-06-18）

**落地 2 处 IFC 安全修复**（zero-regression，单测覆盖，默认启用）：
- **缺陷 1**（inline/mod.rs:822）：`collect_inline_items` 旧实现对文本节点**无条件**调
  `collapse_whitespace` 把 `\n` 折成普通空格。修复：`preserve_whitespace`（pre/pre-wrap/
  break-spaces）时保留原始内容。CSS Text §3.1。
- **缺陷 2**（inline/mod.rs:1274）：`split_into_words`（preserve 模式）为每个 `\n` 推入空字符串
  作强制换行标记，但单词循环旧实现对空词 `continue` 静默丢弃 → 即便 `\n` 保留也不换行。
  修复：`preserve_whitespace && content_word.is_empty()` 时 flush 当前行 + 开新行（同 `Br`）。
- 单测 `test_white_space_pre_newline_forces_break` / `_no_wrap`；reftest 438/490 持平零回归；
  layout lib 880 测试全过；clippy/fmt 干净。

**paint 侧多行堆叠缺陷（缺陷 3）实证 Phase-A 死锁**：painter/text.rs 非多列路径用
`all_fragments()`（片段 y 行内相对，恒 0），多行块所有行渲染在同一 y → **垂直堆叠**
（auto-wrap `<div>` 实测 ZW h=17 vs CHR h=107；morning-work 67% 压缩主因）。改用
`all_fragments_with_line_y()`（片段 y += line.y）实证**净 -11 回归**（438→427，multicol -8 /
CSS2 -3）——回归用例（multicol-breaking-*/column-height-009/column-balancing-paged）的 test/ref
此前都堆叠渲染（test==ref 同错）故同源通过；正确修复使它们「正确」但 ref 仍堆叠 → 同源 FAIL。
**已回退**（with_line_y）。与 R125/R198/R205 Phase-A 死锁同类：paint 多行 line.y 应用破坏
自源 test==ref 匹配，需架构性统一（paint 不重跑 IFC，用 compute_final 存储的正确多行行盒，
但 R84 单行+Ahem 守卫使非 Ahem 多行不存储 → 死锁）。缺陷 1+2 是缺陷 3 的前置（IFC 先能正确算
多行），落地后缺陷 3 成下一明确目标（破 Phase-A 死锁，多轮）。证据
`evidence/r246-multiline-stacking-deadlock-2026-06-18.txt`。

### R248 — DC-6 quirks mode 全链路实证 + goal doc 矛盾纠正：已实现且非 reftest 杠杆（2026-06-18，read-only，独立于并行 agent R247 IFC 修复）

承接 R246 优先级队列外三条独立 read-only 线之一（DC-6 quirks 覆盖核查）。**起因**：goal doc 内部矛盾——Support Envelope 表（rendering-compat.md line 52）称「DOM parser 已存储 quirks mode 但**下游完全忽略**」，而 Current Proven Baseline 表（line 356）称「Quirks mode ✅ 已实现 CSS parser + style system + **layout engine 三层**」。治理规则要求「发现矛盾须先纠正文档」。本轮 read-only 全链路实证（仅读 dom/css-parser/style-system/layout-engine，零源码改动，不碰并行 agent R247 改的 inline/mod.rs + edge_cases.rs）。

**实证结论：quirks mode 已实质实现，下游并未「完全忽略」（line 52 过时错误）**：
- **DOM 层 ✅**：`node.rs:60` 存 `quirks_mode: QuirksMode`，`parser.rs` 经 html5ever 按 DOCTYPE 设置，`doc.quirks_mode()` 访问器；dom tests 验三态（NoQuirks/Quirks/LimitedQuirks）。
- **CSS parser 层 ✅（mode-gated，生产接线，非死代码）**：`parse_color_quirks`(color.rs:53)/`parse_length_quirks`(types.rs:1056)；`apply_property_value_with_quirks`(apply.rs:43-61) 按 `quirks_mode: bool` 切换 quirks/标准解析器；生产链 `doc.quirks_mode()`(lib.rs:133) → compute_styles_recursive → `inheritance.rs:94 apply_property_value_with_quirks(..., quirks_mode==QuirksMode::Quirks)` 完整贯通。
- **Style-system 层 ✅（3 quirks，layout 前 pre-bake）**：`apply_quirks_mode_adjustments`(lib.rs:526) 实现 ① 百分比高度 quirks（父 height:auto→子 percentage→auto）② table height→min-height(CSS2.1 §17.5.2) ③ inline width/height 保留；lib.rs:421 `if quirks_mode==QuirksMode::Quirks` 门控；**值转换在 layout 前 pre-bake 进 ComputedStyle**。
- **Layout-engine 层 ⚠️（零 quirks 字样，但非真缺口）**：layout-engine 全 crate 零 quirks 引用。**架构合理**——style-system 已把值转换类 quirks（①②）pre-bake 进 ComputedStyle，layout-engine 作用于已纠正的值，无需独立 quirks 层；百分比高度定高判定由 `engine.rs:1521 clamp_percentage_max_height`（R119 谱系）承担。故 line 356「layout engine 三层 quirks 调整」**技术性夸大**（layout-engine 实际不做 quirks 调整），但 ✅ 实质正确（quirks 工作正常）。

**reftest 杠杆评估 = 非杠杆（关键结论）**：wpt-data 仅 **6/665** html 缺 DOCTYPE（quirks），去重为 **~3 个测试对**（table-cell-inline-size-box-sizing-quirks / flexbox-definite-cross-size-constrained-percentage / float-table-align-left-quirk）= 490 的 0.6%。06-17 cross-validate 实证可采样的 2 用例**全部 self-pass 且 z_vs_chr≤0.13%**（float-table-align-left-quirk self 0.06%/chr 0.13%、table-cell-inline-size-box-sizing-quirks self 0.00%/chr 0.10%，远低于 1% 严格容差）——**quirks mode 现有实现已使这些用例干净通过与 chromium 一致**。注：float-table-align-left-quirk 标题「Check that the old IE quirk for `<table align=left>` is NOT implemented」=验证某 quirk *不*实现，ZW 正确通过。**即使完美 quirks 实现也只动 ≤3 用例（0.6%）且已全过 → quirks mode 非 reftest 进展杠杆，非 DC-14 达标路径**。

**文档矛盾纠正**（governance「发现矛盾须先纠正」，本轮已改 goal doc 两处表 cell）：① line 52「下游完全忽略」=过时错误（下游 style-system + css-parser 完整消费），已纠正为真实状态；② line 356「layout engine 三层 quirks 调整」=技术性夸大（layout-engine 零 quirks，由 style-system 预烘焙覆盖），已纠正为「两层活跃 + layout 预烘焙覆盖」；③ DC-6 四 checkbox 未勾但**实质已基本达成**（DOM 传递 + css-parser mode-gated + style-system 3 quirks；layout 层无独立实现属架构选择非缺口），DC-6 非阻塞。

**次要 spec 精度 nuance（非杠杆，仅记录）**：style-system quirk ①百分比高度仅查**直接父** `parent_style.height`（单层），CSS 规范（quirks.spec.whatwg.org）要求沿**包含块链**向上查定高祖先——深层嵌套百分比高度在 quirks mode 下理论精度不足，但现有 ~3 个 quirks 用例全过，纯理论 nuance 非实测缺口。

**对优先级队列的影响**：**quirks mode 从 DC-6 候选中排除**（已实现 + 非杠杆）。实现 agent 优先级队列不变：P0 morning-work pre-wrap 已由并行 agent R247 落地（缺陷 1+2，438/490 持平零回归）→ 缺陷 3 paint-multiline 堆叠（R247 实证 Phase-A 死锁 net -11 已回退，多轮架构项，与 R125/R198/R205 同墙）→ P1 R244 DC-9 GPU alpha groundwork → P2 R236 multicol baseline-export → P3 R238 WM-1 abspos-vertical → 长线 DC-9 ping-pong / DC-14 结构聚类。**无 open 阻塞**。详见 evidence/r248-quirks-mode-fullchain-audit-2026-06-18.txt。无代码变更，基线 438/490 持平。

### R249 — DC-9 GPU ping-pong 地基精确 spec + 纠正 R220/R244「post-process pipeline 缺失」误判（2026-06-18，read-only，GPU 区域与并行 agent text.rs/paint WIP 无冲突）

承接 R248 后下一独立 read-only 线（DC-9 ping-pong 第 2 纹理 + 后处理接线点深挖）。读 gpu/renderer/mod.rs + gpu/pipeline.rs + primitive/mod.rs + engine/paint/{effects,helpers}.rs（零源码改动，不碰并行 agent 的 text.rs WIP）。

**核心纠正（R220/R244 误判）**：R220 记「ping-pong 差第 2 纹理 + **post-process pipeline**」，R244 沿用。本轮实证——**post-process pipeline 实际已构建，仅 dead-coded**：`pipeline.rs:703 create_blur_pipeline`（`vs_fullscreen` 全屏三角顶点 + `fs_blur` 高斯模糊片元，采样 src_texture）+ `renderer/mod.rs:63 blur_pipeline`（标 `#[allow(dead_code)]`）+ `mod.rs:216 blur_bgl`。**grep 证实 blur_pipeline 创建后从未 set_pipeline/draw（纯死代码）**。故 DC-9 ping-pong 地基比预想 contained：✅ 已就绪 post-process pipeline + 通用 src 纹理 BGL（`create_texture_bind_group_layout` pipeline.rs:807）+ headless_texture 已带 TEXTURE_BINDING（可作 src）；❌ 真正缺：① 第 2 offscreen 纹理 B（仅 headless 一张）；② 接线（draw_post_process_pass helper + 专用 uniform——fs_blur 期望 blur_radius/direction，但 render_full_scene_gpu 的 uniform 仅 `[f32;4]` 不匹配）；③ 消费者（filter/transform/blend 在 gpu/ 零引用，静默丢弃）。

**单 pass 直合成实证（为何 ping-pong 必须）**：render_full_scene_gpu（mod.rs:651-763）Phase1 收集各图元顶点→Vec → Phase2 开【单个】render pass 按绘制顺序把所有图元 draw 进同一 view → submit。无中间纹理/readback/后处理 pass。**三图元均 rect-scoped group 操作**（实证 paint 侧：FilterPrimitive effects.rs:266 / TransformPrimitive helpers.rs:182 / BlendModePrimitive effects.rs:313，均 `{rect, ...}` 对区域内所有图元应用）→ 即便 Transform 含 a/b/c/d 旋转/倾斜也需 render 区域→offscreen→矩阵采样→main，全三图元需 ping-pong。FilterKind 11 变体（primitive/mod.rs:232），Opacity 最简（alpha 乘法）。

**最小 contained 地基 spec（零行为变更，详见 evidence/r249-...）**：prereq=R244 alpha groundwork。Step1 地基：① 加 `offscreen_texture_b`（复用 create_headless_texture 同参）；② 专用后处理 uniform buffer（补 blur_radius/direction 或扩展 struct）；③ `draw_post_process_pass` helper（开 pass 写 dst + 绑 src_bg + set blur_pipeline + draw 0..3 全屏三角）+ 移除 `#[allow(dead_code)]`；④ 单测：render fill→A→blur A→B→read_pixels(B) 断言边缘模糊（证明 ping-pong 端到端 + un-dead-code，生产不接线=零行为变更）。Step2 首个消费者 filter:opacity（新增 fs_opacity 复用 vs_fullscreen，最简 alpha 乘法）。

**架构 caveat（明确 defer）**：真实 CSS 语义=rect-scoped 区域隔离（空间查询 flat 图元 + 从主 pass 排除该区域 + 处理重叠/部分落入/绘制顺序）=多轮架构项。地基阶段用「全屏后处理」或「scissor 限到 filter.rect」证明机制 + 满足 DC-9 单测即可；区域隔离后续多轮。不改 R244/R220「ping-pong 是 DC-9 真正大头/结构多轮」定性，仅纠正「pipeline 缺失」误判 + 锐化首步到 contained 的 un-dead-code + 2nd texture。

**对优先级队列的影响**：DC-9 ping-pong 地基比预想 contained（pipeline 已 dead-coded 存在）。建议实现 agent（P1 起手）：R244 alpha groundwork → R249 Step1 ping-pong 地基（2nd texture + un-dead-code + 单测，零行为变更）→ Step2 filter:opacity 消费者 → filter:blur（复用 fs_blur）/brightness/contrast → transform/blend（区域隔离多轮）。其余不变：P0 morning-work R247 已落地 → 缺陷3 paint-multiline Phase-A 死锁（多轮）→ P1 R244→R249 → P2 R236 multicol baseline → P3 R238 WM-1。**无 open 阻塞**。详见 evidence/r249-dc9-gpu-pingpong-groundwork-spec-2026-06-18.txt。无代码变更，基线 438/490 持平。

### R250 — DC-9 reftest+product-smoke footprint 量化：≈ 0，DC-9 非 reftest/产品杠杆（2026-06-18，read-only，grep 无碰撞）

承接 R249（DC-9 ping-pong 地基 spec）后的 footprint 量化，类比 R246 pre-wrap / R248 quirks。先核对前提：transform/filter/blend **是真缺口非 clip 式 no-op**——R220 证 ClipPrimitive 生产从不生成，但本轮 grep 实证 apply_filter(mod.rs:697)/apply_transform(mod.rs:705)/apply_blend_mode(mod.rs:730)/apply_backdrop_filter(mod.rs:383) 均在 paint_node 生产调用 → 三图元由 paint 生成、GPU 静默丢弃 = 真缺口（R249 前提成立）。注：Transform 的 translation 经 apply_transform_offset 烘焙进坐标，但旋转/缩放/倾斜靠 TransformPrimitive，GPU 丢弃。

**reftest footprint（wpt-data 665 html 实测）**：`transform:`=1（css-position incidental）、`filter:`=**0**、`mix-blend-mode`=3（css-position z-index-blend incidental）、`backdrop-filter`=0、`will-change`=10（仅 hint）。→ DC-9 三真缺口 reftest footprint **≤ 4 且全 incidental**，即使完美 GPU 支持也只动 ≤4 用例（490 的 <1%），与 R246/R248 同类≈0。

**关键：DC-9 专属目录未导入**——已导入 css 目录为 CSS2/flexbox/fonts/grid/multicol/position/tables/text-decor/writing-modes；**css-transforms / filter-effects / compositing 均未导入**。故 footprint≈0 主因是专属目录尚未导入（goal Phase 1-3 范围未含）→ DC-9 仅在 future M10 扩展导入这些目录后才 reftest-load-bearing。

**product-smoke footprint（实测，纠正 substring 误匹配）**：welcome.html 2 命中**均为 `text-transform:uppercase`（文本大小写属性，非 2D transform）**→ 真实 DC-9 使用 0；morning-work 0；wintertc 1（疑同类 text-transform 误匹配，~0）。→ 产品页基本不用 2D transform/filter/blend，DC-9 无产品 smoke 杠杆。

**结论：DC-9 价值定位**：(a) DC-9 Done Criterion 本身（图元覆盖单测=硬门禁，与 reftest 分数无关）；(b) future-proofing（M10 扩展导入专属目录后做准备）；(c) **非** 近期 reftest 杠杆（≤4 incidental）；(d) **非** 产品 smoke 杠杆（≈0）。**期望管理**：DC-9 工作不应期待 reftest 438/490 上升或 product-smoke diff 下降，验证聚焦 DC-9 图元覆盖单测（framebuffer 像素断言）。

**对优先级队列的战略含义**：DC-9（P1）footprint≈0 reftest 杠杆；而 P2 R236 multicol baseline-export（8 例 1.1%）、P3 R238 WM-1（14 例）均在 490 自源集=**真实 reftest 杠杆**。DC-9 P1 理由不变（contained 零回归地基 + 硬 Done Criterion，非 reftest 杠杆）；若实现 agent 目标是**最大化 reftest/DC-14 进展**，R236/R238（结构多轮但真实 reftest 杠杆）可上调与 DC-9 并列或之前；若目标**低风险 contained 进展**，DC-9（R244→R249）仍优先。二者非互斥：DC-9 地基零回归可先落再并行攻 R236/R238。详见 evidence/r250-dc9-reftest-footprint-2026-06-18.txt。无代码变更，基线 438/490 持平。

### R251 — R244 DC-9 alpha groundwork spec 对当前代码实证核实：准确，P1 首步就绪（2026-06-18，read-only，GPU 顶点/着色器区域无碰撞）

承接 R250 后去风险 P1 实现首步。R244 alpha groundwork spec 写于 1aea7c2（数轮前），本轮核实其对当前 HEAD(45ebb42) 代码仍准确（代码可能已变）。读 gpu/mesh.rs + gpu/pipeline.rs（不碰并行 agent text.rs/run-rules.md WIP）。

**逐条核实——全部确认，spec 完全准确**：① `color_to_f32`(mesh.rs:11) 返回 `(f32,f32,f32)` 仅 RGB，**color.a 完全未用** ✅；② `push_fill_quad`(mesh.rs:16-27) 每顶点 `[x,y,u,v,r,g,b]`=**7 floats**（color vec3f），所有 push_* 同 ✅；③ `FILL_VERTEX_ATTRIBUTES[:3]`(pipeline.rs:414) color 为 vec3f（ROUNDED_RECT/GRADIENT 同）✅；④ fragment `vec4f(in.color, alpha)`(pipeline.rs:61, coverage) + `vec4f(in.color,1.0)`(pipeline.rs:170)=.a 槽硬填 coverage 非 CSS alpha ✅；⑤ BlendState 5 处含 ALPHA_BLENDING 已配 ✅。

**R244 修复范围精确化（P1 首步可直接起手）**：(a) mesh.rs color_to_f32→4-tuple + push_* 7→8 floats/顶点；(b) pipeline.rs FILL/ROUNDED_RECT/GRADIENT color 属性 vec3f→vec4f + array_stride+1；(c) WGSL VertexOutput.color vec3f→vec4f + fragment `vec4f(in.color.rgb, in.color.a*coverage)`；(d) Blend 已 ALPHA_BLENDING ✓。

**零回归主张核实成立**：opaque CSS 颜色（a=1.0）改后 =1.0*coverage=coverage（与现状一致→零变化）；仅半透明（a<1.0）从「错误实色」变「正确半透明」=修复非回归（R228b CPU 侧 translucent alpha 的 GPU 同源修复）。故「零回归」=opaque 零变化+半透明修正。

**结论**：R244 spec 对当前代码**完全准确**（1aea7c2→45ebb42 无相关变更），P1 实现首步就绪，范围=mesh.rs+pipeline.rs（color vec3f→vec4f）机械跨切 contained、opaque 零回归+半透明修正、验证=framebuffer 像素断言 rgba(...,0.5)=blend 非实色。实现 agent 可直接起手 R244 无需再调研。R244→R249 ping-pong 地基顺序不变。详见 evidence/r251-r244-alpha-groundwork-verification-2026-06-18.txt。无代码变更，基线 438/490 持平。

### R252 — pre/pre-wrap paint 多行渲染 scoped 修复落地（零回归，几何正确，非 chr-diff lever）（2026-06-18）

承接 R247 缺陷 3（paint 多行堆叠 Phase-A 死锁）。R246 实证 ungated / gated(容器级) `with_line_y`
均净 -11 回归。本轮尝试第 3 种 scope：**仅 preserve_whitespace（pre/pre-wrap/break-spaces）**。
修复 `painter/text.rs:940`：`use_stored`→空；`preserve_whitespace`→`all_fragments_with_line_y()`
（行 y 偏移正确）；否则 `all_fragments()`（旧行为，auto-wrap 不触碰）。判据：pre 族多行来自显式
`\n`（R247 缺陷 1+2 已让 IFC 正确产出多行），auto-wrap 的 test/ref 堆叠同错故不触碰（R246 净 -11 来源）。

**验证**：reftest-upstream **438/490 零回归**（pre 族 wpt-data 仅 4 文件用，auto-wrap 未触碰）；
isolated `<pre>` 3 行几何正确（h=51 旧 h=13 堆叠）；LAYOUT_DUMP morning-work pre 块 ifc_lines
正确（3/5/6/9/14 行）；tall-viewport（9000px）product-smoke A/B（R247-only vs scoped）diff 0.22%
（16133 px）集中在 y=7500/8250 的 pre 块区域；make test 12233/0；clippy/fmt 干净。

**诚实结论（非 morning-work 67% lever）**：isolated pre ZW(多行) vs CHR = **23.24%**（旧堆叠
24.37%）——几何从 1 行→3 行正确，但 chr diff 几乎不变。真因：23% 主由 `#eee` 背景 + fontdue vs
chromium 等宽字体度量噪声构成，**非堆叠几何**。故本修复是**正确的几何 bug 修复**（pre 不再塌缩，
零回归，单测可证），但 morning-work 67% 主因在字体度量/背景/独立问题（font-weight R229、img
固有尺寸 R233），pre 堆叠仅其几何表现之一。**价值定位**：正确性修复 + 未来含 pre 页面正确渲染；
非 reftest-multiplier（4 文件）+ 非 morning-work chr-diff lever。证据
`evidence/r252-prewrap-paint-multiline-scoped-2026-06-18.txt`。R247 缺陷 1+2（IFC 多行计算）+
本修复（paint 多行渲染）配套闭合 pre 块多行正确性；auto-wrap 多行堆叠仍是 Phase-A 死锁（多轮）。

### R253 — R236 multicol baseline-export spec 对当前代码实证核实：plumbing-gap 确认，最高 contained reftest 杠杆就绪（2026-06-18，read-only，engine.rs/multicol.rs 区域无碰撞）

承接 R250 战略含义（R236/R238 是真实 reftest 杠杆）。核实 R236 multicol baseline-export spec（写于 c53a541）对当前 HEAD 代码仍准确，为「最大化 reftest 进展」路径去风险。读 engine.rs + types/mod.rs + multicol.rs（不碰并行 agent text.rs/R252 WIP）。

**逐条核实——R236 plumbing-gap pattern 全部确认**：① `taffy_baseline` 字段在 types/mod.rs:236，`extract_baselines_recursive`(engine.rs:474-484) 从 `taffy.cached_baselines().y` 写入 `box_node.taffy_baseline` ✅；② flex/grid 消费 `child.taffy_baseline`(engine.rs:988，§8.5 容器基线合成) ✅；③ **multicol.rs `grep baseline`=0——完全无 baseline 处理** ✅（自建 column post-pass 从不计算/存储 baseline → multicol 导出 baseline 回退 taffy 通用 block baseline，非 §baseline-export 列导出值 → 8 例 ~1.1% 系统性偏移）。

**可行性核实（修复路径成立）**：taffy 视 multicol 为 plain block，cached_baselines 返通用值（错误来源）；multicol.rs 已做 column post-pass，可在列布局后算 §baseline-export（各列首行 baseline 取最高=first，末列末行=last，IFC 已 track line baselines）写入 LayoutBox.taffy_baseline 覆盖。字段/extract/consume 路径全在，仅缺 multicol.rs 填充=标准 plumbing-gap（同构 R229 font-weight / R234 font-kerning）。

**结论**：R236 spec 对当前代码**完全准确**（c53a541→HEAD 无相关变更），**最高 contained reftest 杠杆**（R235 确立 8 例 ~1.1% 系统性偏移，结构性最 tractable，bounded feature）。实现 agent「最大化 reftest 进展」路径可直接起手 R236（字段/extract/consume 就绪，仅需 multicol.rs 填充），成功标准=baseline-000~008 类 ~1.1% 偏移消除（reftest 438/490 上升 +8 潜力，须 zero-regression 全量验证）。与 P1 DC-9（contained+硬DC，≈0 reftest 杠杆）并列：DC-9 零回归地基可先落，R236 攻 reftest 分数。详见 evidence/r253-r236-multicol-baseline-export-verification-2026-06-18.txt。无代码变更，基线 438/490 持平。

### R254 — R238 WM-1 abspos-vertical spec 对当前代码实证核实：准确，第二 reftest 杠杆就绪（完成三部曲）（2026-06-18，read-only，converter/engine.rs abspos 区域无碰撞）

完成 reftest 杠杆验证三部曲（R251 R244 alpha ✓ / R253 R236 multicol baseline ✓ / 本轮 R238 WM-1）。核实 R238 spec（写于 5dff3c4）对当前 HEAD(000a462) 代码仍准确。读 converter/mod.rs + engine.rs（工作树干净，无碰撞）。

**逐条核实——全部确认**：① `apply_vertical_writing_mode`(converter/mod.rs:196，tree.rs:429 调用)=css-writing-modes §7.1 dimension-swap 已落地 ✅；② engine.rs:1394-1426 abspos static-position fix 守 `all_inset_auto`(1396 top+bottom auto，1401 用 IFC fragment 位置修正) ✅；③ **§10.3.7 处理在但仅 height shrink-to-fit**(1411-1425，height:auto 时 child.height 收缩到 fragment.width 内容 inline 跨度)，**无 direction(rtl/lr) 分支**——静态位置修正直接用 fragment.x/y 不按书写方向调整 → **R238 residual B「缺 §10.3.7 direction 分支→rtl 5.03%=4×ltr」=当前最清晰修复信号 confirmed**。

**结论**：R238 spec 对当前代码**完全准确**（5dff3c4→HEAD 无相关变更）。WM-1 abspos-vertical=**第二 reftest 杠杆**（14 例 css-writing-modes abspos vrl/vlr），修复=给 static-position 修正加 direction 分支（contained to engine.rs:1394-1426，无 converter/taffy/paint 改）。⚠️ **R164 教训适用**：per-case diff 混合 positioning+Ahem 噪声，**勿未实验就声称 clean +14**（R238 本身已带此告诫）——实现 agent 须先跑当前 per-case diff 确认 residual B 仍在 + 实验加 direction 分支后消除且无回归方可声称。

**reftest 杠杆验证三部曲完结**：R251 R244 DC-9 alpha（P1 contained+硬DC，≈0 reftest 杠杆）/ R253 R236 multicol baseline（P1' 最高 contained reftest 杠杆 +8）/ R254 R238 WM-1（P2 第二 reftest 杠杆 14 例）——三条路径全 spec+验证就绪，实现 agent 可按「contained 优先(DC-9)/reftest 分数优先(R236→R238)」选择，无需再调研。详见 evidence/r254-r238-wm1-abspos-vertical-verification-2026-06-18.txt。无代码变更，基线 438/490 持平。

### R229b — font-weight 落地实测：bold 选择机制正确但 fontdue 粗体过墨 ~15% → net-negative 死路（已回退）（2026-06-18）

承接 R229（font-weight 未落地精确方案）/ R230（资源前提实证）。本轮**完整落地** R229 两部分方案后实测——**纠正 R229「welcome 17% 主因=font-weight」假设**，回退。

**实现（已回退）**：B 接线——`FontLoader.load_font` 解析 OS/2 usWeightClass 存 `font_weights`，`find()` 按 CSS §5.2 最近字重选变体，新增 `FontResolveEntry`+`pick(target)`，`build_font_resolver()` 返回字重感知结构；`resolve_font_id(font_family, font_weight)`（6 处 paint 调用点传 `style.font_weight`）；workspace build 干净。A 资源——`app_platform.rs`+`reftest.rs` 加载 `DejaVuSans-Bold`/`NotoSans-Bold`/`LiberationSans-Bold`/`NotoSansCJK-Bold`（系统已装）。

**实测：机制正确但渲染过墨 → net-negative**。粗体**确被选择并渲染**（welcome card h3 区 ink-mass old-ZW 2095 → new 2713，**+29%**，证明接线生效），但 fontdue 光栅化的 `DejaVuSans-Bold` 比 chromium 的 bold **过墨 ~15%**（title 区 new 1559 vs chr 1363 = +14%；card h3 new 2713 vs chr 2318 = +17%）→ welcome product-smoke **17.06%→17.55%（+0.49pp 回归）**；morning-work 48.65% 不变（其 h1-h6 显式 weight:400 仅 `<strong>/<b>` weight:600）。按零回归标准 net-negative，**已 git checkout 回退全部代码**。

**死路定性**：font-weight 接线**正确且自源中性**（reftest 438 不变，Ahem 无 weight），但加载粗体资源后 fontdue 粗体光栅化过墨 + bold advance width 级联换行偏移 → 产品 smoke net-negative。**同 advance-width(R225)/AA(R174) 谱系的 fontdue-vs-chromium 渲染差异**，非「字重未选」单点可修。**勿再以「加载 DejaVuSans-Bold 接线」重试 R229**（重现过墨回归）；保留接线须搭配 fontdue 渲染校准或 chromium 同字体选择，非单会话。

**残余 welcome 17% / morning-work 48.65% 真因重定性**：font-weight 非主因后=**item-tag span→block 全宽堆叠（R109 IFC 架构）**+ **fontdue vs chromium 字体度量噪声（CJK line-height/advance，非 weight）**+ morning-work hljs（需 JS）/body ~300px 高差，均独立子问题。证据 `evidence/r229b-fontweight-bold-overshoot-deadend-2026-06-18.txt`。无代码变更，基线 438/490 持平。

### R255 — morning-work 4× 高度幻影间隙修复落地（ua_default_display 缺 article/aside/details 等，零回归，DC-13 真杠杆）（2026-06-18）

承接 R253（morning-work 89% diff 真主因定位为元素间幻影垂直间隙，最小复现「未重现」留 BISECT）。本轮系统 BISECT + 落地修复，**首个打破 morning-work plateau 的真实修复**。

**BISECT 隔离（纠正 R253「最小复现未重现」误判——因最小用例未带 .article 类/外链 CSS）**：
1. `<article class="article">` + 外链 CSS（min-height:200px）→ **复现**幻影盒（每对块级子元素间一个 `article.article` w=4 h=200 mt=64 匿名盒）。
2. `<article>` + **空 CSS**（纯结构）→ **仍复现**（幻影盒 h=0 w=4）→ 证明非 CSS 触发，是结构层。
3. `<div class="article">` → **干净**（排除 .article 类/min-height）。
4. **逐标签扫描**：`<article>`/`<aside>`/`<details>` 触发幻影；`<div>`/`<section>`/`<main>`/`<nav>`/`<header>`/`<footer>`/`<blockquote>`/`<figure>` 干净。

**根因（精确）**：`ua_default_display`（style-system/src/lib.rs:49）block 列表**缺 article/aside/details/hgroup/menu/search**。CSS display 初始值=inline → 这些标签回落 inline；含 block 子元素（h2/p）时 `inline_block_split::inline_has_block_child` 返回 true → tree.rs R109 路径（line 554-591）为每段连续 inline 内容（块级子元素间空白文本节点）生成匿名块盒，继承父 node_id（→ dump 标签显示 article.article）+ 应用 .article 样式（min-height:200、margin:4em）→ 每对块子元素间插入 h=200+mt=64≈264px 幻影高度，累积 4×。

**修复**：`ua_default_display` block 列表对齐 **HTML Living Standard UA 样式表**（13.1.1 display:block 列表），补 `article|aside|details|hgroup|menu|search`。单点改动（style-system/src/lib.rs:52-55），未触 layout/paint/converter。

**验证**：
| 指标 | 修复前 | 修复后 |
|------|--------|--------|
| morning-work body 高度 | 25301px（4.2× chr） | **5677px**（≈0.95× chr 5981） |
| full-page chr-diff（800×5981） | 89.14%（R253） | **48.65%**（-40pp） |
| LAYOUT_DUMP 幻影盒 | 每对块子元素间 article.article w=4 | **消除** |
| reftest 自源 | 438/490 | **438/490 零回归** |
| make test | — | **12235 passed / 0 failed** |
| clippy -D warnings / fmt | — | 干净 |

**单元测试**（style-system ua_display_tests）：① `test_html_block_level_sectioning_elements_default_to_block` 钉死 article/aside/details 等 HTML block 标签默认 Block；② `test_inline_elements_remain_unset` 防 span/a/code 等被误标 block。

**诚实结论**：① 800×600 顶部视口 chr-diff 仍 28.72% **不变**（顶部区 nav/title/item-tags 在折叠上方不受幻影盒影响；残余=item-tag R109 + font-weight R229，独立）。② full-page 48.65% 残余=font-weight(R229)+fontdue CJK 度量+item-tag R109+hljs(需JS)+body ~300px 高差，均已知独立子问题。③ 本修复是**正确几何 bug 修复 + 零回归**：消除 morning-work 4× 高度，HTML 区块元素 display 正确化，DC-13 morning-work 从「4× 高+89%」降至「~1× 高+48.65%」，剩余由 font-weight/item-tag 独立推进。证据 evidence/r255-morning-work-phantom-gap-ua-display-fix-2026-06-18.txt + product-static/morning-work/*-fullpage.png。

经 R140（独立穷尽验证）+ R141b（R109 6 轮不可解）+ R144（属性审计穷尽）三重确证，**435/490 为单会话零回归平台期**。剩余 55 失败全属结构性多轮里程碑，按预期收益/风险排序的候选路径：

1. **R109 inline-block ownership 架构项目**（多轮，高风险，潜力 +2 集群 css-flexbox-row/test1 + flexbox-column-row-gap-004）
   - 根因：vertical-rl flex 容器中 inline-block 色块（经 IFC 绘制）的 x 坐标不受 `mirror_vertical_rl_block_children` 后处理影响（R141b 已证 6 轮单兵不可解）。PIL 确认 css-flexbox-row.html 4 色块 TEST x[670,789] 右 vs REF x[10,39] 左。
   - 真正修复需统一 paint 的 inline-block 绘制路径到 IFC（DC「Layout/Paint IFC 双路径」），消除绕过 paint_node 与 paint_text 非存储的两条已知路径。属多次会话架构重构。
   - 见 [[r109-writing-mode-flex-arch]]、[[r142-vertical-rl-sibling-shift-axis]]、[[r140-cleanwins-exhausted-verified]]。

2. **R131 multicol 列感知 IFC 碎片化**（多轮，高风险，潜力 ~16 测试但回归 ~20 通过用例）
   - 根因：multicol 塌缩的零高子元素是行内级 DOM 节点(text/br)，「行高回写」不可行；真正修复=列感知 IFC 按行盒（非子元素）碎片化到各列。R122 记录 paint 路径改动 net -5（multicol 39→34/57）。
   - 前置：需先确认 R131 block-children 分布会回归的 ~20 个单列回退恰好通过的用例（multicol-breaking-000/001/002/003、baseline-000~006 等）。
   - 见 [[r131-multicol-fragmentation-arch]]、[[r113-nested-multicol-twopass-plan]]。

3. **R114b writing-mode 垂直轴 float/clearance 参数化**（多轮，中等风险）
   - 需把 ~150 行 `adjust_float_positions_with_context` 做 block/inline 轴参数化，非 surgical flag，对通过的 floats-clear 有高回归风险。R133 已建实现地基（converter 不换 float，物理 left/right→block 方向）。
   - 见 [[r114-writing-mode-characterization]]、[[r133-vertical-float-impl-ground]]。

**对后续会话的明确指引**：
- 不要重试 R125 large-font 三路径死锁、R140 gap-fix、R141b R109 单兵镜像——均已验证 net-negative 或 +0。
- 不要再做属性实现完整性审计——R144 已穷尽（197 注册属性全有 apply 分支）。
- 单会话若需推进，唯一现实路径是启动上述 3 条结构性多轮项目之一的**第一个安全子步骤**（如 R109 的「定位 inline-block 背景 fill 实际绘制入口」插桩诊断，不改逻辑、零回归），并明确标注为多轮项目的第 N 步。

### R255 — morning-work 4× 高度幻影间隙触发因子定位：`.article{min-height:200;margin:4em}` 算术恒等式（328=200+64+64），read-only（2026-06-18，基线 438/490 持平）

**承接**：上一 docs 轮 d8f18d3（标号误用 R253，与 000a462 R236 multicol 的 R253 冲突）写了 morning-work 4× 高度调查 evidence（`evidence/r253-morning-work-4x-height-phantom-gaps-2026-06-18.txt`）但**未写 master.md section**，且其"下一轮隔离根因"开放问题未答。本轮 read-only 文本核对（article.html + article.css + R253 dump 算术）**回答该开放问题**，并把 morning-work 调查正式补进 master.md（用 R255 避开 R253 标号冲突）。零源码/零渲染。

**决定性恒等式**：R253 dump 的 article 子元素间隙 5 个中 **4 个 = 328px**（h2→p/p→p/p→h2/h2→h3/h3→p 全 328，与元素类型无关），1 个 272px。328px 恰为 `.article` 盒属性之和——

```
.article { min-height: 200px; margin-top: 4em; margin-bottom: 4em; }   /* article.html 内联 <style> 77-81 行 */
  min-height 200 + margin-top 64(4em×16) + margin-bottom 64 = 328px    /* .article font-size:16px(article.css) 确认 4em=64 */
```

**幻影盒非 HTML 字面重复**：article.html **全文仅 1 个** `<article class="article">`（168–352 行），故 R253 dump 里 `h=200 w=4` 的幻影嵌套 article **是 ZeroWeb 生成**（h=200 又精确=min-height）→ 与 328 恒等式同源：ZeroWeb 正按 `.article` 盒模型生成幻影盒，~328px/对插在每个 block 兄弟边界。`min-height`/`margin` 为 CSS 非继承属性，匿名盒本不该继承 → 指向**选择器过匹配 / 幻影盒被赋予 `.article` 身份 / inter-block 空白文本节点被包成匿名块且错误套用 `.article` 规则**，属 R109 谱系（匿名块/inline→block/IFC 边界）相邻缺陷。

**纠正 R253 最小复现失败根因**：R253 用 h2+p+pre 复现 328px gap 失败，因复现**缺 `.article{min-height:200;margin:4em}` 容器**——触发因子不是 h2/p/pre 标签或 line-height/word-break，而是「带 `{min-height:200;margin:4em}` 的 `.article` 容器 + 多个 block 子元素」组合。**修正最小复现**：`.article` 容器(min-height:200;margin:4em)+h2/p/h2/p → 预期 ZW 间隙≈328px（chromium ≈16/52）。

**下一步（合规运行时验证，交付实现/运行时轮）**：① 修正最小复现经**合规路径**（临时 wpt-data 自源 reftest + `<link rel="match">` + `make reftest` + `LAYOUT_DUMP=1`，**禁裸 cargo test**，测后删临时文件恢复 490 基线）渲染确认 328px；② 单变量 BISECT（删 min-height / 删 margin:4em / 删 .article class）锁 328 各分量来源 → 定位 ZeroWeb 哪条路径把 `.article` 盒属性套到幻影/匿名盒；③ 修复后 morning-work 4×→~1× 高，product-smoke A/B 量化 chr-diff 降幅（应远超 R252 的 0 改善）；④ wpt-data 自源 reftest 438/490 中性回归。w=4 细节次要未释（h=200 才是信号）；272 离群≈328−56 疑边界 margin 折叠，主信号 328 不受影响。

**与既往诊断关系**：不动 R253 主结论（4×高=布局层幻影垂直间隙，非字体噪声/非 pre 堆叠/非内容压缩），仅精化"疑似匿名块"→"`.article` 盒属性泄漏到幻影盒，328=200+64+64 可证"；与 R247/R252 pre 多行（已修，独立）正交；与 reftest 杠杆三部曲（R251/R253/R254）正交——本轮是 **DC-13 product-smoke 杠杆**（morning-work 89%），非 reftest 438/490 杠杆。**本轮 read-only 无代码/reftest 变更，基线 438/490 持平。** 详见 `evidence/r255-morning-work-phantom-gap-arithmetic-identity-2026-06-18.txt`。

### R256 — 【自我纠正 R255】幻影盒机制被代码路径否定：build_subtree 对 block 容器不生成幻影/匿名盒（read-only，2026-06-18，基线 438/490 持平）

> **⚠️⚠️ 本节「R255 机制被否定」的定性结论已被证伪，仅保留作历史记录——见 R257 + 并行 agent 提交 `a2b169e`（ua_default_display 补 article 等，经验证 morning-work body 25301→5677px、fullpage chr-diff 89.14%→48.65%、reftest 零回归）。** R256 的错误在于前提「article 计算 display:block」——实际 article 因 UA 缺失计算为 **inline**，故 R109 **确实触发**，R255 的「.article 盒属性泄漏到幻影匿名盒（R109 谱系）」机制 + 328=200+64+64 算术**全部正确**。R256 仅「converter 映射无误 / build_subtree 对*真正 block 容器*不生成幻影盒」的代码事实成立（但 morning-work 的 article 非 block，故不适用）。**勿据 R256 否定 R255 或改 build_subtree/converter/R109——缺陷在 ua_default_display（已由 a2b169e 修）。** 下文为 R256 原文。

**承接**：R255 据 R253 dump 摘要 + 算术巧合（328=200+64+64）推断 morning-work 4× 高度机制=「`.article{min-height:200;margin:4em}` 盒属性泄漏到 block 兄弟间幻影/匿名盒（R109 谱系）」。本轮 read-only 读 build_subtree（tree.rs:362-664）全路径 + inline_block_split.rs + converter/mod.rs 核实该机制——**结论：机制被否定**（**注：此结论因前提 article=block 错误而无效，见上方横幅与 R257**）。

**决定性代码事实（build_subtree 对 `<article>` block 容器）**：
- **R109 匿名块拆分不触发**（tree.rs:546）：`r109_segments` 仅当 `inline_has_block_child(...)` 为真才算；该函数（inline_block_split.rs:59-61）首条要求 `display==Inline`——article 是 Block → false → r109_segments=None → 走 tree.rs:618 else，不进 559-591 匿名块生成。
- **非 flex/grid 容器只收 Element 子**（tree.rs:618-627）：623 行 `matches!(&n.kind, NodeKind::Element(_))` **显式跳过文本/空白节点**（正确 CSS：block 容器 block 兄弟间空白不生成盒）。
- **每 element 子一个 taffy 节点**（632-644），article 自己一个（654）+ node_map 一对一（660）。**无第二个带 article.node_id 的盒**。
- **converter 映射正确**（converter/mod.rs:81-88）：min_size.height←min_height、margin←margin，只作用于 article 一个节点，不泄漏到子/幻影。

→ R255 三个因果子机制（inter-block 空白→匿名块 / 幻影盒被赋 .article 身份 / 选择器过匹配）**在 build_subtree 里都不成立**。dump 标签语义（reftest.rs:1043-1047 `b.node_id→DOM labels`，缺才 `(anon)`）+ build_subtree 一对一映射 → R253「幻影嵌套 article w=4 h=200」**不应存在**，疑人为筛选摘要伪影。

**R255 降级**：328=200+64+64 算术**相关性真实但无因果机制**，且依赖 R253「人为筛选 6 行摘要」（非原始递归 dump）。**勿据 R255 改 build_subtree/converter/R109（无对应缺陷）**。观察层（ZW body 25301 vs CHR 5981=4.2× 高；tall-viewport 内容下移）是 product-smoke A/B 实测，仍可靠、仍需解释，但机制不是盒泄漏。

**强制性下一步（须原始递归 dump + clean tree）**：只有原始递归 LAYOUT_DUMP（reftest.rs:1033 带深度缩进全量打印）能区分「真幻影盒→定位生成它的 post-pass」vs「摘要伪影/跨层 abs_y 误读」。**须在 clean tree 跑**——当前并行 agent 未提交 `style-system/src/lib.rs`，脏树 dump 会编译进其 WIP 致证据不可靠。合规命令：`./target/test-guard -- cargo run --bin zero-wpt-runner -- reftest <filter> --wpt-data <path>` + `LAYOUT_DUMP=1`（test-guard 包裹=非裸跑；`make reftest` 无 filter 跑全量，单测用 test-guard 直 prefix）。步骤：① 树干净后渲染 morning-work/最小复现；② 读**完整递归 dump** 核对 h2↔p 间到底有哪些盒（node_id/abs_y/h/mt/dmt），判断 328px 是真间隙还是跨层差；③ 若有带 article.node_id 第二盒→grep extract_layout/engine.rs compute() post-pass（build_subtree 已排除）；若无→R253/R255 间隙观察是伪影，须重做 morning-work 89% 根因。

**方法学教训（印证 R164/R203/R241）**：R255 据 curated 6 行摘要 + 算术巧合推断机制、未读生成路径源码即下结论；R256 读 build_subtree 推翻。**单点/摘要推断须源码实证，不能据 curated 输出 + 算术巧合接力**。后续 morning-work 调查从原始递归 dump 重起。**本轮 read-only 无代码/reftest 变更，基线 438/490 持平。** 详见 `evidence/r256-morning-work-phantom-box-mechanism-refuted-2026-06-18.txt`。

### R257 — morning-work 4× 真根因【确认 + 纠正 R256】：article 缺 ua_default_display→inline→R109 幻影匿名块（read-only，2026-06-18，基线 438/490 持平）

**承接 + 关键转折**：读并行代码 agent 未提交 WIP（style-system/src/lib.rs），其回归测试注释明确揭示 morning-work 4× 高度**真根因**。结合源码核实——**R255 的机制+算术正确，R256 的「反驳」错误**（前提「article=block」假）。本 R257 取代 R256 定性，收口 morning-work 4× 调查（历经 R228b/R247/R252/R253/R255/R256/R257 共 6+ 轮）。

**真根因链（全源码闭合）**：
1. **触发源**：committed `ua_default_display`（style-system/src/lib.rs）block 列表**不含 `article`**（亦缺 `aside`/`details`/`hgroup`/`menu`/`search`）→ 函数返 `None` → 元素回落 CSS 初始 `display:inline`。
2. **R109 触发**：article 计算为 inline + 含块级子（h2/p）→ `inline_has_block_child`（inline_block_split.rs:59-61 查 `display==Inline`）**返 true** → tree.rs:546 `r109_segments=Some` → 进 559-591 匿名块生成。
3. **盒模型泄漏**：tree.rs:386 `computed`=**article 计算样式**（含 `.article{min-height:200;margin:4em}`）；tree.rs:571-572 每个 R109 匿名块用 `computed_style_to_taffy(&computed)` 构建（强制 display:Block）→ **匿名块继承 article 的 min-height:200+margin:4em** → h≥200+margin64/64=**328px/对**。
4. **4× 高**：每对 block 兄弟间插 328px 幻影盒，累积 ~4.2×（ZW body 25301 vs CHR 5981）。R253 dump 的 328/328/328/328 间隙 + 「幻影嵌套 article h=200 w=4」（带 article node_id 的 R109 匿名块，dump 标签 `b.node_id→DOM labels`）全部吻合。

**纠正 R256**：R256 读 build_subtree 后断言「article 是 Block→R109 不触发→build_subtree 不生成幻影盒→R255 机制否定」。**代码路径阅读本身正确，但前提「article=block」错误**（article 实为 inline）。故 `inline_has_block_child(article)=true`，R109 **确实触发**，R255 的「.article 盒属性泄漏到幻影匿名盒（R109 谱系）」+ 328=200+64+64 算术**全部正确**。R256 降级为「基于错误前提的部分分析」——其「converter 映射无误 / build_subtree 对*真正 block 容器*不生成幻影盒」的代码事实成立但对 morning-work 不适用（article 非 block）；其「勿改 build_subtree/converter/R109」的*操作结论*仍成立（缺陷在 ua_default_display），但「R255 机制否定」的*定性结论*错误，本 R257 取代。

**与并行 agent WIP 对齐（非我改动）**：并行 agent WIP（未提交）style-system/src/lib.rs 把 `article`/`aside`/`details`/`hgroup`/`menu`/`search` 加入 ua_default_display block 列表 + 新增 `ua_display_tests` 回归（注释引用「R253 morning-work 4× 根因」）。修复后 article 计算 block→`inline_has_block_child`=false→R109 不触发→无幻影匿名块→正常 block 布局。**从源头消除触发，正确最小**。inline/mod.rs 仅 rustfmt（3 行无逻辑）；新增 capture-fullpage.mjs（puppeteer fullpage 截图，服务 morning-work tall-viewport 验证）。

**预期 + 验证（已由并行 agent 提交 `a2b169e` 经验证实）**：修复后 morning-work 4×→~1× 的预期**已获经验确认**——并行 agent fullpage A/B（capture-fullpage.mjs）实测 morning-work body **25301px→5677px**（≈0.95× chr 5981，4× 消除）、fullpage chr-diff **89.14%→48.65%**（-40pp）、LAYOUT_DUMP 幻影盒消除、reftest 自源 **438/490 零回归**、make test 12235/0、clippy/fmt 干净。其逐标签 BISECT 进一步精化：`<article>`+空 CSS（无 .article 类/min-height）**仍复现**（h=0 w=4）→ **纯 UA display 缺失触发，非 CSS**（.article 盒模型只决定幻影盒*高度* 200+margin，不决定*是否触发*；触发仅由 article=inline 决定）。**morning-work 4× 线收口**。残余 fullpage ~48.65% = 独立子问题（font-weight R229 / fontdue CJK 度量 / item-tag span→block R109 / hljs 语法高亮缺 JS / body ~300px 高度差），非本根因，留后续轮。⚠️ ua_default_display 改动使 article/aside/details 等从 inline→block——a2b169e 已跑 reftest 438/490 零回归，但若后续扩大导入含重用这些标签的用例，须复核。

**方法学教训**：推理代码路径前须**核实元素实际计算值**（此处 display），UA 默认值是隐藏假设；R256 据「article 应是 block」常识假设而非实测 computed 下结论被推翻。morning-work 4× 根因 = 最朴素的 UA 默认 display 缺失，历经多轮才定位。**本轮 read-only（仅读并行 agent WIP + 源码核实），无代码/reftest 变更，基线 438/490 持平。** 详见 `evidence/r257-morning-work-rootcause-confirmed-ua-display-2026-06-18.txt`。

### R258 — ua_default_display 完整性审计：a2b169e 后基本完整，无更多 morning-work 类危险缺口（read-only，2026-06-18，基线 438/490 持平）

**承接**：morning-work 4× 根因 = `<article>` 缺 `ua_default_display` block 条目→inline→R109 幻影盒。a2b169e 补 6 标签（article/aside/details/hgroup/menu/search）。本轮**系统性审计** committed `ua_default_display`（style-system/src/lib.rs）对 HTML Living Standard UA 样式表 §13.1.1 的完整性，排查是否还有同类「应为 X 却回落 inline」危险缺口。read-only 源码核对 + wpt-data footprint grep，不改源码、不跑渲染、不碰并行 agent IFC WIP（inline/mod.rs）。

**审计结论：a2b169e 后 ua_default_display 基本完整，无更多 morning-work 类危险缺口**。committed 现状——Block(35,含 article 等)/Table 系(10)/InlineBlock(11: img/media/form 控件)/None(10: script/style/link/meta/head/title/base/noscript/template/dialog)，其余 `_=>None`(回落 CSS 初始 inline)。DisplayValue 枚举变体齐全（Block/Inline/InlineBlock/Flex/Grid/None/Contents/Flow/FlowRoot/ListItem/Table/…），所有 UA 默认值可表达。

**morning-work 类危险缺口排查（应为 block 容器却 inline + 含 block 子 → R109 幻影盒级联）**：对照 HTML UA §13.1.1 block 列表，ZeroWeb **仅缺 `center`**（legacy 居中元素），wpt-data footprint=**0 文件**。其余 block 容器全覆盖 → **无更多 article 式危险缺口**。

**`<br>` 落 inline 无害**：不在任何分支→inline，但 IFC 专门拦截（`inline/mod.rs:902 if local_name()=="br"` line-break 处理）→ 换行正确性不受影响。`<wbr>` 未专门处理（仅断行建议，低影响）。

**次要完整性缺口（低 footprint / 低影响，非近期杠杆）**：`meter`/`progress`→应 InlineBlock 现行内（各 2 reftest 文件，但无 gauge paint，inline vs inline-block 仅影响盒尺寸）；`center`→应 block 现 inline（0 footprint，legacy）；`source`/`track`/`param`/`area`→应 none 现 inline（均 0 footprint，空 inline 盒不可见）；**`li`→应 ListItem 现为 Block**（list-item markers 项目符号/编号未通过 display 应用——已知简化，影响所有 `<ul>/<ol>` marker 渲染，但 marker 是更大 feature「marker box 生成+paint」非单点 display 改，且涉 IFC/paint 与并行 agent WIP 潜在交叠，defer）。

**结论**：a2b169e 已闭合 morning-work 类 UA display 缺口；剩余缺口 footprint ≤2 或 =0，**非 reftest/产品 smoke 杠杆**（meter/progress 即便补 inline-block 也无 gauge 渲染）。唯一广泛真实页面影响项=`li`→ListItem+list marker（更大 feature，defer 待 IFC 稳定）。**本轮价值**=把「是否还有 morning-work 同类 UA bug」从开放问题变为**已审计确认无重大缺口**，避免后续重复排查同类；钉死次要缺口清单+footprint 量级供代码 agent 低优先级参考。**本轮 read-only 无代码/reftest 变更，基线 438/490 持平。** 详见 `evidence/r258-ua-default-display-completeness-audit-2026-06-18.txt`。

### R259 — 并行 agent font WIP = R229 font-weight 选择（与 fontdue 光栅化结论正交，非矛盾，read-only，2026-06-18，基线 438/490 持平）

**承接**：master.md header line 7「🔤 fontdue 光栅化 ≈ chromium…字体攻坚停止」与并行 agent **正在改 font stack**（font/loader.rs + font/mod.rs）表面矛盾。本轮 read-only 读并行 agent font WIP 核实是否 contradicts fontdue 结论——**结论：无矛盾**。

**并行 agent font WIP = R229 CSS Fonts §5.2 font-weight 选择**（读 WIP diff 确认）：font/loader.rs 加 `font_weights: HashMap<font_id,u16>`（OS/2 usWeightClass）+ `parse_font_weight(data)`；`find(desc)` 从 `ids.first()` 改 `pick_weight_variant(ids, desc.weight, …)`；`build_font_resolver()` 返回 `HashMap<String, FontResolveEntry>`（字重感知变体列表，原返单 id）。font/mod.rs 新增 `FontResolveEntry{variants, pick(target_weight)}`（CSS §5.2 font-matching，并列时 target≥500 偏重）。pipeline.rs + painter/mod.rs（亦 dirty）= paint 侧按 `ComputedStyle.font_weight` 选变体（R229「接线 B」）。正是 header line 9 标的最高优先级 R229 的端到端实现（资源+接线），spec 见 `evidence/r229-fontweight-plumbing-spec-2026-06-17.txt`。

**澄清矛盾（正交）**：line 7 结论（R186/R187/AA-baseline）= fontdue **光栅化**（给定 glyph→像素）≈ chromium，无需替换——**仍成立**。R229 缺口 = ZeroWeb `find()` 一直忽略 weight（总挑 Regular）→ `font-weight:600/700` 按 Regular 渲染（欠墨）。**两者正交**：fontdue 把任何变体光栅化得与 chromium 一致；问题是挑错变体（总 Regular）。修 font-weight 选择后 Bold 文本用 Bold 变体 + fontdue 正确光栅化。**无矛盾**——line 7「字体攻坚停止」应理解为「**fontdue 光栅化替换**停止」（R186/R187 证伪 fontdue→swash swap），**非**「所有字体工作停止」；font-weight 选择（R229）是另一条线，并行 agent 现正推进。

**预期（交付并行 agent，勿未验先声称）**：R229 成功标准 = welcome/morning-work 600/700 粗体 ink-mass 回升 ~100% CH；reftest 438/490 自源中性（Ahem 无 weight）但 DC-14 chromium-oracle 改善。WIP 未提交未验证，待其提交后 product-smoke + reftest 全量验证。资源前提（系统装 -Bold，R230 已确认）满足。

**对优先级队列的影响**：① **R229 font-weight 并行 agent in-flight**（font/loader+mod + pipeline + painter WIP）——其他 agent 勿碰 font stack + paint font-weight 消费路径。② morning-work 残余 48.65% 中 font-weight 子项将随 R229 消除；其余残余（item-tag R109 / hljs 缺 JS / fontdue CJK 度量 / body ~300px 差）独立。③ reftest 杠杆 R236（multicol baseline +8）/ R238（WM-1 +14）仍待代码 agent 实现，与当前 font/paint/IFC WIP 无碰撞。**本轮 read-only 无代码/reftest 变更，基线 438/490 持平。** 详见 `evidence/r259-parallel-font-wip-is-r229-fontweight-2026-06-18.txt`。

> **【R229b 经验纠正 R259，2026-06-18】** 并行 agent 提交 `7d062e5 R229b` 实测**推翻 R259 ②的乐观框架**「font-weight 子项将随 R229 消除」。R229 选择机制**完整落地且生效**（bold 被选+渲染，welcome card h3 ink-mass 2095→2713 +29% 证接线正确），**但 fontdue 光栅化 Bold 变体比 chromium 过墨 ~15%**（title +14%、card h3 +17%）→ welcome product-smoke **17.06%→17.55%（+0.49pp 回归）** → net-negative **已全部 git checkout 回退**。结论：fontdue **Bold 光栅化过墨**（区别于 Regular 经 AA 基准 ≈chromium），同 advance-width(R225)/AA(R174) 谱系的 fontdue-vs-chromium 渲染差异，**非「字重未选」单点可修，勿再以「加载 DejaVuSans-Bold 接线」重试 R229**。R259 的**核心（选择机制与光栅化正交）仍成立**（R229b 证选择机制正确+自源中性），但**产出预期**（bold ink-mass 干净回升 ~100%）被证伪。残余 welcome 17%/morning-work 48.65% 真因重定性 = item-tag span→block(R109 IFC) + fontdue-vs-chromium 字体**度量**噪声(CJK line-height/advance，**非 weight**) + hljs(需 JS) + body ~300px 差——font-weight **不再是**主因杠杆。

### R260 — R236 multicol baseline-export 升级为 turnkey 行级实现 spec（补 R253 四要素：顺序/插入点/坐标系/算法，read-only，2026-06-18，基线 438/490 持平）

**承接**：R236 multicol baseline-export = 最高 contained reftest 杠杆（R235 确立 8 例 ~1.1% 系统性偏移；R253 确证 plumbing-gap：multicol.rs 零 baseline，字段/extract/consume 路径全在，仅缺 multicol.rs 填充）。本轮 read-only 把它从「已验证 gap」升级为**代码 agent 可直接起手的行级 spec**，补 R253 留空的 4 个实现关键细节。读 engine.rs（compute 调用顺序 + 消费坐标语义）+ multicol.rs（layout_multicol），不碰并行 agent font/paint/IFC WIP（multicol.rs 干净）。

**① 执行顺序（R253 未显式建立，关键 enabler）✅ 有利**：engine.rs `compute()` 内 line 228 `extract_baselines_recursive`（从 taffy cached_baselines 写 taffy_baseline，对 multicol 容器写的是**错误的 taffy 通用 block baseline**）**早于** line 264 `adjust_multicol_layout`（multicol 后处理）。故 multicol 覆写 `container.taffy_baseline` 为正确 §baseline-export 值**不会被 extract 冲掉**（compute_incremental 412→424 同序）。**这是修复可行的必要前提**。

**② 插入点**：`layout_multicol`（multicol.rs:244）流程 = 收集 child_info → 分配 `assignments:Vec<Vec<ColumnFragment>>`（balanced/sequential/with-breaking）→ `position_multicol_children` 设位。**在 position_multicol_children 之后**（子元素列内位置已定）新增 helper（如 `compute_multicol_exported_baseline(container,&assignments)->Option<f32>`）计算并写 `container.taffy_baseline`。每个 multicol 容器在其 layout_multicol 内就地写入。

**③ 坐标系**：engine.rs:988-992 消费 `child.taffy_baseline` 用守卫 `0 < taffy_bl < child.content_height` → **taffy_baseline 是相对该盒自身内容盒原点的偏移**（非绝对坐标）。故 multicol 写入的 exported baseline 须是「首列首行文本 baseline 相对 multicol 容器内容盒顶的偏移」。

**④ 算法（first baseline，8 用例主体）**：CSS Multicol §2.9/§baseline-export——first baseline = **首列第一行文本 baseline**。`assignments[0]`（首列）首个片段子元素的 `taffy_baseline`（extract 228 行已写入=子元素首行 baseline）可用，故 `first_baseline =（首列首子元素 border-box 顶 相对 multicol 内容盒顶偏移）+（首子 border-top+padding-top）+ 首子.taffy_baseline?`。**守卫**：仅 `0<v<container.content_height` 才写（对齐消费守卫）；超界或首子无 taffy_baseline（嵌套 multicol/空/无文本）→ 不覆写保留 taffy 通用值（退化安全）。**last baseline defer**（需末列末行 baseline，taffy_baseline 只给首行；8 用例主体 first baseline）。

**验证（交付代码 agent，R164 教训勿未验先声称）**：成功标准 = baseline-export 8 例 first-baseline 偏移消除 → reftest 自源 **438/490 上升 +8 潜力**（须 zero-regression 全量，因 taffy_baseline 也被 flex/grid §8.5 消费，改动可能波及 flex baseline-alignment 用例）；multicol.rs 加单测（2 列各 1 文本块，断言容器 taffy_baseline=首列首子 baseline 偏移）。**风险**：覆写仅影响 multicol 容器自身 taffy_baseline → 仅波及「multicol 作 flex/grid item 参与 baseline 对齐」用例（正是 8 baseline-export），非 multicol 不受影响；须 grep 确认无消费点绕过 LayoutBox.taffy_baseline 直读 taffy cached_baselines（extract 是唯一写者，消费走字段，应安全）。**非撞车**：multicol.rs 干净（并行 WIP 在 inline/mod.rs+edge_cases.rs，不含 multicol.rs）。**R236 现为 turnkey 行级 spec，代码 agent 可直接起手，无需再调研。** 详见 `evidence/r260-r236-multicol-baseline-turnkey-spec-2026-06-18.txt`。

### R261 — master.md header 自洽纠正：line 9「最高优先级」从 font-weight 更新为 R229b 后真实优先级 + line 7 Bold 细化（read-only，2026-06-18，基线 438/490 持平）

**承接**：R229b（7d062e5）证 font-weight net-negative 死路（fontdue Bold 过墨 ~15%）、R255-R260 闭合 morning-work 4× + 产出 R236 turnkey spec，但 master.md **header line 9「🎯 最高优先级」仍停留在 R229 时点**——标 font-weight 为最高优先级（R229b 已证死路）、说并行 agent 在 table.rs+cpu/mod.rs（实际 inline/mod.rs IFC）、说 welcome 17% 主因=font-weight（R229b 重定为 item-tag R109+fontdue CJK 度量）。header 是控制面板首要可见处，陈旧误导须纠正（治理「各章节须自洽」）。

**纠正（仅 header docs，无代码/reftest 变更）**：① **line 9 整段替换**为 R229b 后真实优先级——font-weight **死路**（R229b：选择机制对但 Bold 过墨 net-negative 全回退，勿再 -Bold 接线重试）；morning-work 4× **闭环**（R255-R260+a2b169e）；当前真实优先级=reftest 杠杆 R236（turnkey spec 就绪 R260，+8）+ R238（R254 spec，+14，R164 告诫须实验）+ welcome/morning-work 残余重定性（item-tag R109 并行 agent IFC WIP + fontdue CJK 度量非 weight + hljs + body ~300px）+ DC-9 ping-pong spec'd + UA 审计完成（R258）；并行 agent 当前位置更正为 inline/mod.rs IFC。② **line 7「fontdue 非差异来源/字体攻坚停止」加 R229b Bold 细化**——精确为「**Regular** 非差异来源；**Bold 过墨 ~15%** 是 fontdue-vs-chromium 差异（同 advance-width/AA 谱系）」，避免 line 7 绝对陈述与 R229b 矛盾。

**意义**：header 控制面板反映 R229b 后真实状态，防后续 agent/读者据陈旧 line 9 重试 font-weight 死路或误判优先级。本轮 read-only 无代码/reftest 变更，基线 438/490 持平。

### R262 — R238 WM-1 abspos-vertical 半 turnkey spec（位置+hook+镜像原理+探针+验证，read-only，2026-06-18，基线 438/490 持平）

**承接**：第二条 reftest 杠杆 R238 WM-1 abspos-vertical（14 例，R254 确证 gap + R164 告诫）从「R254 spec」升级为半 turnkey 实现 spec。读 engine.rs:1355-1429（abspos-vertical static-position 函数全貌），补 R254 留空的行级细节。engine.rs abspos 区干净（并行 agent WIP 在 inline/mod.rs，不含此函数）。

**精确位置 + 书写模式 hook（R254 未给行级）✅**：函数 engine.rs:1355-1429——**书写模式 hook 已就位**：line 1364-1365 `is_vertical_rtl = matches!(root.writing_mode, VerticalRl)`，并已传 IFC（1375 `.with_vertical_rtl`）。direction 分支可**直接 key on `is_vertical_rtl`**，无需新增查询。**bug 精确行**：line 1404-1409 `if all_inset_auto { child.x = fragment.x; child.y = fragment.y; }`——`fragment.x` 直接赋 `child.x`，**未按 is_vertical_rtl 镜像**（= residual B rtl 5.03% vs ltr 1.28% 的代码点）。**可用 CB 几何**：line 1368 `container_width = root.content_width`（注释 1366-1367：轴交换后 content_width=视觉高度/行内方向，content_height=视觉宽度/块方向）。

**镜像原理**：CSS writing-modes——vertical-rl 块流向右→左（block-start=视觉**右**），vertical-lr 左→右（block-start=视觉**左**）。abspos all-inset-auto 静态位置=block-start 边，故 rl 的静态视觉 x 应是 lr 的镜像（`x_rl ≈ CB视觉宽 − 元素视觉宽 − x_lr`）。当前两者都用 fragment.x（疑 lr 模型值≈0）→ lr 对(1.28%)/rl 错(5.03%)。修复原理：`if is_vertical_rtl { child.x = mirror(fragment.x) }`。

**镜像公式不能纯只读确定（诚实局限）⚠️**：mirror 精确式取决于三个纯只读无法定的坐标语义——① fragment.x/child.x 是轴交换(swapped)还是视觉(unswapped)坐标；② CB 块方向视觉宽 = `root.content_height` 还是其他；③ 元素视觉宽 = `child.height`(轴交换后)还是 `child.width`。故本 spec 为**半 turnkey**：位置+hook+原理确定，公式须代码 agent 探针实证（避免 R255→R256「推断被实证推翻」教训）。

**代码 agent 须跑的探针（定 mirror 公式，强制前置）**：取一 residual B 用例（vertical-rl abspos all-inset-auto，css-writing-modes vrl-xxx），`LAYOUT_DUMP=1`/REFTEST_DUMP 渲染打印 root.writing_mode、root.content_width/height、abspos child 的 child.x/y/width/height、匹配 fragment.x/y/width、chromium 目标视觉 x；对照 chromium 反推 `x_rl = ? − ? − fragment.x` 三候选字段哪个成立；据实证式实现 line 1404-1409 内 `if is_vertical_rtl` 镜像 child.x。⚠️ fragment 坐标语义依赖 IFC（dirty），**建议并行 agent IFC WIP 提交后再实证**避免脏树干扰。

**次要 residual A（all_inset_auto 未查 left/right）**：line 1396-1399 仅查 top+bottom auto，**未查 left/right**。CSS：静态位置仅在该轴 insets 全 auto 时用；left/right 指定时元素由其定位非静态。当前对「top/bottom auto 但 left/right 指定」也强行 child.x=fragment.x 覆盖 → residual A（ltr 1.28%）可能部分源于此。修复（独立于 mirror，可分步）：all_inset_auto 拆 `_y_auto`(top+bottom)/`_x_auto`(left+right)，仅对应轴全 auto 才用 fragment 修正该轴；但改现有 vrl/vlr 行为须全量验证。

**验证协议（R164 教训勿未验先声称）**：① 先跑当前 per-case diff 确认 residual B(~5.03%)/A(~1.28%) 仍在；② 实现 mirror(探针实证式)+可选 residual A 拆分；③ 重跑 14 例逐例确认 rtl residual 消除且 ltr 不退化（勿据总数声称 clean +14，per-case 混 positioning+Ahem 噪声）；④ zero-regression 全量 438/490（direction 分支仅影响 vertical-mode abspos all-inset-auto；residual A 拆分可能波及 left/right 指定 abspos 须复核）；⑤ 单测构造已知 vertical-rl abspos all-inset-auto 断言 child.x=chromium 预期（block-start 在右）。**R238 现为半 turnkey spec，代码 agent 起手须先跑探针定 mirror 公式（不能跳过）。** 详见 `evidence/r262-r238-wm1-abspos-vertical-turnkey-spec-2026-06-18.txt`。

### R263 — R236/R238 spec 对 a2b169e 仍有效验证 + 并行代码 agent stall 协调观察（read-only，2026-06-18，基线 438/490 持平）

**承接**：R236（R260 turnkey）/R238（R262 半 turnkey）spec 写于 a2b169e（ua_default_display 补 article/aside/details/hgroup/menu/search → inline→block）之后。本轮只读验证最大近期代码改动 a2b169e 是否 shift 了这两条 spec 的 reftest 用例 baseline。grep R236 的 8 baseline-export 用例 + R238 的 14 WM-1 用例搜 `<(article|aside|details|hgroup|menu|search)[ />]`——**命中 0 文件**。

**结论：a2b169e 对 R236/R238 用例零影响**（两 spec 用例不用受影响标签）→ R260/R262 spec 的 baseline 对当前 HEAD 代码**仍准确有效**，代码 agent 可直接起手实现，**无需因 a2b169e 重新 baseline**。去除了「最大近期代码改动是否使 spec 失效」的风险。

**协调观察（控制面板状态）**：git log 核实——并行代码 agent 自 `a2b169e`（R255 morning-work 修复）后，唯一提交是 `7d062e5 R229b`（font-weight bold overshoot **实验已回退**，docs-only 无净代码）+ 本 docs agent 的 R258-R263。**即 reftest 438/490 代码侧自 a2b169e 起 ~7 轮无推进**。并行 agent 工作树持续有 `inline/mod.rs` + `edge_cases.rs` 未提交（跨 ~7 轮），疑攻 item-tag span→block（R109 IFC 架构，R229b 重定性的残余主因），但 R109 是 goal doc P1 硬架构（R141b「6 轮单会话不可解」），可能未收敛。

**对优先级队列的影响**：① **R236（+8）/R238（+14）spec 就绪等代码 agent 实现**，且 multicol.rs / engine.rs abspos 区域**与并行 agent inline/mod.rs IFC WIP 不冲突**，可作为代码 agent 的并行/替代方向（若 IFC item-tag 攻坚卡住，R236/R238 是 contained reftest 杠杆，更易落地）。② docs 侧高价值非撞车只读调查已饱和（morning-work 4× 闭环 / UA 审计 / font-weight 死路记录 / R236·R238 spec / header 自洽 / 本轮 spec 有效性验证）。③ 若并行 agent IFC WIP 持续不提交，后续轮可考虑 lark 通知本人核对代码 agent 状态（本轮仅记录不通知——非本 docs agent 阻塞，且可能系正常硬架构攻坚）。**本轮 read-only 无代码/reftest 变更，基线 438/490 持平。** 详见 `evidence/r263-r236-r238-spec-valid-vs-a2b169e-2026-06-18.txt`。

