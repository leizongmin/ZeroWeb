# M2 s246 — srcset 逗号 URL 候选切分勘根 + spec URL 收集口径补面（Rust 三函数单点，current-pixel-density 3 SVG + density-vs-zoom 3 density 面翻绿）

日期：2026-11-03。切片号 s246（承 s245）。根：s245 下一步候选 zero-engine
`build_img_intrinsic_all` SVG 固有尺寸面独立勘根轮（current-pixel-density
basic 3 SVG clientWidth 784×0 → 期望 10×10，需核对 extract→layout 消费契约）。

## 入场定刀位

s245 下一步候选：zero-engine build_img_intrinsic_all SVG 面（basic 3 SVG
clientWidth，需独立勘根轮核 extract→layout 契约）。

## 勘根（纯代码链逐环定谳，零探针——静态管道全绿与实测 784×0 矛盾定位单点）

- **extract→layout 消费契约逐环核**（六环节）：
  ① `build_img_intrinsic_all`（pipeline/mod.rs:746）img 臂 = `effective_img_src`
  → `image_resource_key(src, document_url)` 查宿主注入三 map
  （image_sizes/ratios/no_ratio）→ s219 密度折算 per-Node；
  ② 宿主注入（webview.rs:1713-1732）：SVG 分类链 `svg_intrinsic_kind`
  （image_cache.rs:2009）——BothAbs/ComputedIntrinsic → `image_sizes` +
  `no_ratio_intrinsic` → `image_no_ratio`；RatioOnly → 仅 `image_ratios`；
  ③ key 一致性：data: 经 `is_non_relative_href` 原样保留
  （resolve_document_url），webview data: 臂 `(src, None)` 与 pipeline
  `(src, document_url)` 同串同 hash ✓；
  ④ 解码链：Rust `percent_decode_bytes` 是 WHATWG 宽松语义（非法 % 原样
  保留——与 s245 JS 侧修复后同口径），fixture 仅 %20 → 解码通；三个
  basic fixture（viewBox '-1 -1 2 2' × width/height 双 20 / 单 width 20 /
  单 height 20）分类 BothAbs/ComputedIntrinsic(20,20)×2 → 均应落
  `image_sizes` 20×20，密度 2x → (10,10)；
  ⑤ layout 消费（tree.rs:1281 `_` 分支）：no_ratio 先命中即跳过 sizes
  （互斥分派），basic 3 fixture 无 no_ratio 条目走 sizes 臂 → intrinsic
  (10,10) 且无 containing-block 拉伸（s245 probe plain 变体 ground truth
  证实消费语义）；
  ⑥ **PNG 对照证管道通**：basic 同页 3 PNG expect-0 face（srcset-only）
  clientWidth=1 = 密度 +Inf 折算 → sizing.rs 1px 钳——PNG srcset-only 的
  快照/折算管道全通，SVG face 独缺上游条目。
- **单点定谳（srcset 逗号切分）**：fixture srcset 值
  `data:image/svg+xml,<svg%20…> 2x`——data: URL 的 header 分隔逗号
  （`svg+xml,`）裸露在属性里。Rust 三处 srcset 候选切分全用 `split(',')`：
  `srcset_first_url`（R2419）→ effective src 截断为 `data:image/svg+xml`；
  `push_srcset_candidates`（s70）→ 抓取清单推两个错切段；`parse srcset`
  密度镜像（s219）→ 一个候选错切成 [(None,None),(payload串,2x)]。截断
  URL `decode_data_uri` 空 payload 解码失败 warn → `image_sizes` 无条目
  → layout 无固有尺寸 → 拉伸盒 784×0。
- **spec 口径**：HTML #parse-a-srcset-attribute 的 URL 收集 = 首个非
  ASCII 空白序列（**URL 内逗号合法**），仅「URL 以逗号结尾」剥尾逗号；
  shim `__zwParseSrcsetAttr5277`（part03:2460）正是该口径（URL 只停空白、
  尾逗号 replace、括号分组描述符）——shim natural 面自 s245 起走
  payload 解析 fallback 链已通，Rust 快照链与 shim 查询键（`__zw_get_image_size`
  = resolve_document_url(完整 URL) 查 cached_image_sizes）因错切段而
  恒 miss、回落 payload 链。
- **shim 快照臂密度序核实（防修复翻红）**：`_zwImgNaturalDim`
  （part04:752）快照臂先取 `__zwSelectImageSource5277` 的 density 再查
  `__zw_get_image_size`，hit 后 `round(raw/density)` 折算返回——宿主键表
  修复后 naturalWidth 链从 payload fallback 收敛到快照主链且折算不丢
  （20/2x → 10 ✓），非 P→F 通道。
- **波及面封闭**：`effective_img_src`/`extract_img_srcs` 全消费者 =
  paint 键（painter/text.rs:917）、intrinsic 折算、fetch 清单三处同源
  同变，无独立第三方。corpus `srcset="data:"` 全扫 5 文件：basic（目标
  面）、svg-img-density-vs-zoom（3 density 面 → 见验证）、
  source-relevant-mutations/img-picture-ancestor/srcset-media-dynamic
  （`data:,sXXX` 空 header 形态——旧错切两候选 vs 新单候选，payload
  非图像解码失败恒无条目，密度选择同值 1.0 → 零位移）。

## 施工（一处文件一块 + 测试，shim 零改动）

- extract.rs 新增 `srcset_candidates`（spec URL 收集口径——URL = 首个非
  ASCII 空白序列、组尾逗号剥除、描述符区跳至顶层逗号（括号不感知同旧
  简化面），返 (url, 描述符区) 列表）；`srcset_first_url`/
  `push_srcset_candidates`/`img_selected_pixel_density` 候选循环三处换用。
- 测试：mod.rs `srcset_first_url_parses_first_candidate` 补 data: 逗号
  URL/`data:,a` 断言；tests/pipeline.rs 新增
  `test_img_srcset_comma_url_density_intrinsic_r5386b`（逗号 URL 单候选
  2x → 10×10、`data:,a` 快照无条目、尾逗号组切分次候选 1x 不折算）。

## 验证

- cargo check 干净（unused_mut 修一处）；单元三门绿：
  `srcset_first_url_parses_first_candidate`、
  `test_img_srcset_comma_url_density_intrinsic_r5386b`、
  `test_img_density_corrected_intrinsic_scaling_r5386`（s219 零回归）；
  runner 重建 31.8s。
- solo 三门：
  current-pixel-density **13F→10F**（3 SVG face 精确 Pass——width/
  height/clientWidth/clientHeight/naturalWidth/naturalHeight 六断言全绿；
  残 10F = 3 PNG 1px 钳 + 7 error alt 盒，s245 挂账渲染流车道维持）；
  svg-img-density-vs-zoom **7F→4F**（no-intrinsic/intrinsic-width/
  both 三个 density 面 F→P——no_ratio/sizes 分派与密度折算语义面精确
  兑现勘根推演；残 4F = 3 zoom 面（CSS zoom 几何未实现，渲染流车道）+
  intrinsicWidth baseline（修复前同态 Fail，零位移））；
  naturalWidth-naturalHeight-width-height **18F 精确维持**（240P，零
  位移——IDL 面走 shim resourceState 不涉 Rust 快照）。
- 全树 A/B（evidence/logs/s246-ab.log vs s245-ab.log）：见下节。

## 全树 A/B（evidence/logs/s246-ab.log vs s245-ab.log）

- 行级 7712P/503F/81T → **7719P/496F/81T**（ΔP=+7、ΔF=−7、ΔT=0）。(key,state)
  对（expected 尾截断 + LC_ALL=C sort/comm 同序）8293/8293，LOST 7 /
  GAINED 7 一一对应：**6 目标键精确 F→P**（current-pixel-density basic
  3 SVG face + svg-img-density-vs-zoom 3 density 面，逐键吻合勘根推演）；
  另 1 对 = checkbox `:active` 交互族 **F→P**（s245 轮同键曾 P→F 漂移，
  本轮漂回——同跑日志内 2F+7P 混态自证 flaky，s238/s239/s245 在册
  checkable-active 漂移族同形，非本切通道（本切仅 Rust srcset 切分，
  无交互面））——零负向位移，**第五个连续完美 A/B**（承 s241/s242/
  s243/s244/s245 线）。
- 改动臂单点零通道：srcset 切分仅 URL 含逗号形态生效（常规相对 URL
  候选输出与旧实现逐键等价），与 A/B 无其他位移一致。

## 残余挂账（本切不动）

- 渲染流车道：3 PNG expect-0 clientWidth 1px 钳（sizing.rs `w.max(1.0)`）、
  7 error.html 404 alt 盒、3 zoom 面（CSS zoom 几何）、18 naturalWidth
  rendered 渲染尺寸面——s245 面级归属维持。
- svg-img-density-vs-zoom intrinsicWidth baseline 1F（修复前同态，勘根
  未入本轮——独立挂账待勘）。
- s241/s240 存量维持。

## 下一步候选

customizable-select 177F（挂起待用户拍板）；UA modal dialog 居中 sheet
缺位（渲染流车道）；渲染流车道 img 面（拉伸/zoom/1px 钳/alt 盒）归
layout-engine 流水线。
