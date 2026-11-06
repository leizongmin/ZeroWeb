# M2 s306 — picture/source 密度选源 + source 维度缺省：responsive-image-select-print ✗8.33%→✓0.00%（engine 选源语义 + runner 图片缓存两层修复）

日期：2026-11-06。切片号 s306（承 s305 下一步候选②）。

## 勘根

**页面机制**（responsive-image-select-print.html）：`<picture><source width="200"
srcset="./resources/red.png 1w, ./resources/green.png 200w"><img></picture>`，
`@page { size: 300px; margin: 0 }`；ref = green 200×200 div。spec
select-an-image-source（https://html.spec.whatwg.org/multipage/images.html#select-an-image-source
）：picture source 通过 media/type 门后按 sizes 换算 w 描述符密度，dpr=1.0
headless 基线下 200w 密度 200 > 1w 密度 1 → 选 **green.png**，且 source
width=200 作 img 维度缺省 → 200×200。

**两层根因定谳**（探针矩阵，probe-reftest/reftest2/h 三轮）：

1. **engine 层**：无选源语义——painter/pipeline 用 effective_img_src
   首候选近似（legacy 取 img srcset 首候选/red.png），无密度折算、无
   source 维度缺省。
2. **harness 层（真根）**：engine 修正后 solo 仍 ✗8.33%。REFTEST_DUMP
   test 页全白；REFTEST_DEBUG 图片通道 **images=0**；LAYOUT_DUMP picture
   盒 w=200 **h=0**（img 塌陷）——engine 已正确选 green.png（diag 单测
   证明产出 (8,8,200,200) green 图元），但 runner 本地 extract_img_srcs
   字符串扫描器只收 img-srcset 首候选/首个 source 首个 URL，
   **green.png（200w 非首候选）从未入 ImageCache** → ImageKey miss →
   200×0 空盒。修 runner 缓存覆盖 → ✓0.00%。

## 施工

**engine（crates/engine）**：

1. extract.rs：`parse_srcset_descriptor` 共享 w/x/h 文法核（w=纯数字 0→错、
   x=HTML float '+' 前缀非法非有限拒绝、h 遗产须伴 w、单候选 w/x 互斥）；
   `source_type_supported` type 白名单镜像 shim；`source_media_matches`
   media 门；`select_source_element` picture 走查（门过+srcset 非空→中，
   img→止）；`pick_srcset_candidate_url` 密度折取（dedup 保首、default-src
   按 `!hasD1 && !hasW` 追加、≤dpr 取最大否则取最小）；`img_selected_source_url`
   spec 序：picture source → img 自身 srcset → img src（文档注明与 legacy
   近似的差异及 FIXME）。
2. pipeline/mod.rs：build_img_intrinsic_all 选源 + `or_else` 回落
   effective_img_src（**broken-image 语义保真**：全 srcset 非法无 src 时
   选源 None → 密度 +Inf → dim 0，r5386 面不回归）；新增
   `apply_source_dimension_defaults`（source width/height 逐轴缺省，
   img 自有属性优先，Auto→Px），接线 4 条渲染路径。
3. painter/text.rs：img 臂 src 取选源 URL。
4. 单测 6 新增（文法核/白名单/密度折/门走查/source sizes/维度缺省+选源
   渲染 pin `test_img_source_dimension_default_and_density_selection_s306`）。

**runner（tests/wpt-runner）**：resources.rs `extract_img_srcs` 委托
`zero_engine::extract_img_srcs`——缓存覆盖全部声明候选（picture source w
描述符多候选、img src+source 并取、img 自身 srcset 非首候选）；删
`find_picture_source_srcset`（旧扫描器专用，R5056 先例）；tests.rs 换
`picture_page_collects_all_declared_candidates_s306` 契约测试。

## 验证

- solo：engine 修后两轮仍 ✗8.33%（probe-reftest/reftest2）→ harness 缓存
  修复 → **✓0.00%（0.00% 差幅，真通过-可信）**（after2）。
- 契约：`cargo test -p zero-wpt-runner --bin zero-wpt-runner` 契约测试 +
  engine 6 单测全绿（编译契约定味，行为由 solo/全树 reftest 定谳）。
- 全树：**17012 案 = 10245 真通过-可信 / 567 可疑 / 5166 近似 / 1034
  不一致**；vs s305-a（93e23082）key 级 join diff——**P→F×7 全数 flake**
  （end-001/end-002/insert-inline-in-blocks-n-inlines-middle-002/tao-127/
  tao-128/tao-153/intrinsic-size-008；6 键今日 solo ✓0.00% 裁定，
  middle-002 s304 已 solo 3× ✓0.00% 且今日补跑 ✓0.00%，同族 ✓✗✓ 弹跳
  先例）；**F→P×4**：responsive-image-select-print（目标修复）+
  grid-with-dynamic-img（真实 F→P）+ insert-block-in-inlines-middle-001/
  tao-154（s305 P→F 同键弹回，flake）。**净真实 +2 F→P 零净 P→F**；
  1031+7−4=1034 精确闭合。
- 引擎 lib 套件（guarded-test）：**3773P/5F**——5F 与 s304/s305 已知集
  逐键相同（5 已知 CSSOM cssText 红：p56_adopted_snapshot…/p99s_rule_style…
  /stylesheets_write_r2809/rule_style_r2810/add_rule_remove_rule_r3276；
  r5296 套件级 flake 本轮未触发）——s306 选源变更**零新增失败**
  （3773P=s305 3766P+7=s306 新增 6 单测+1 渲染 pin 全绿）；此前 80 腿
  test result: ok 全绿。
- 合流态复验（pull --rebase 合入 7a75de463 net-api M1b 后，HEAD 移交于
  make test 之后）：同 crate 交叠面（zero-wpt-runner）+引擎 shim 面
  （part01.js→engine lib）靶向复跑——engine lib **3773P/5F 已知集逐键
  相同**（零新增失败）、wpt-runner bin **288P/0F** 全绿；文件集与
  M1b（part01.js/testharness.rs/net-api doc）不相交，合流风险收口。
- 移交：image-srcdoc-relative-uri-print ✗1.04% 属 srcdoc-iframe 相对 URI
  解析面，非图片面——记 navigation-compat M3 待办。

## 证据

日志（evidence/logs/，md5）：
- s306-probe-reftest.log 99c644c4 / s306-probe-reftest2.log 842d2793 /
  s306-probe-h.log 4081a581（勘根探针矩阵三轮；zw-probe-* 语料 13 文件
  已删）
- s306-solo-responsive-print.log add8e407 / s306-solo-responsive-print-after
  .log b278f4a9（engine 修后两轮 ✗8.33%）
- s306-solo-debug.log 466f4b44（REFTEST_DUMP/DEBUG/LAYOUT_DUMP 三诊断 +
  diag 单测定谳 harness 层）
- s306-solo-responsive-print-after2.log 2a014e0c（修后 solo ✓0.00%）
- s306-full-tree.log 07bceb80（全树 17012 案分类 + key join 源）
- s306-adjud-batch.log c217f647（6 键 solo 批裁定）；
  s306-solo-middle002.log c6a08530（middle-002 今日补跑 ✓0.00%）
- s306-solo-srcdoc-print.log 407a863a（srcdoc 面 ✗1.04%，移交凭据）
- s306-make-test.log b47291b08d0293e97a1cf9fbdfda76c2（9730 行，80 腿 ok
  +workspace 腿止于 engine lib 3773P/5F 已知集；guarded-test）
- s306-confluence-englib.log 3b751a3069c3b6315364588533fbcb18 /
  s306-confluence-wptrunner.log 3682be74dbf8af7868af13b0bf8be26e
  （合流态靶向复验，guarded-test 包裹）

代码锚：crates/engine/src/pipeline/extract.rs（选源五件套+文法核）、
pipeline/mod.rs（build_img_intrinsic_all 选源+回落、
apply_source_dimension_defaults 四路接线）、paint/painter/text.rs（img 臂）、
tests/pipeline.rs（s306 pin）；tests/wpt-runner/src/reftest/resources.rs
（委托收口）、reftest/tests.rs（契约测试）。

## 下一步候选

1. animated GIF/WebP 帧推进（render-foundation decode 域，两页 2.08%
   同刀；s300 image_cache.rs 先例）。
2. the-bdi-element 13 红页簇归属勘根（bidi 隔离，疑渲染域移交）。
3. 小簇排队（ol 6/textarea 4/meter 4/input 3/link 3/embed 3/object 2/
   option 1/iframe 1/br 1/wbr 1）。
