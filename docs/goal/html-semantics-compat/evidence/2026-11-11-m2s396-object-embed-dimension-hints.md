# M2 切片 396（s396）——object/embed width/height 内容属性 dimension hint 施工（#attr-dimensions 同口径，driving 4F→P）

日期：2026-11-11（承 s395 下一步：forms 树散单/分诊或按域 solo 分诊；本切片为 embedded-content 域 solo 施工候选）

域：html/semantics/embedded-content/the-object-element + the-embed-element（driving）

性质：**施工轮——style-system 单侧 presentational hint 面（collect_presentational_hints 一臂），零 shim/engine 改动**

> 交接说明：本切片源码 WIP 由上一 session（429 限流中断于 06:13）留下——style-system 一臂 + 单测已成形、clippy 已跑（/tmp/s394/clippy-s396.log 06:09 Finished clean），未及 solo/全树验收。本 session 接手后 rebase 到 origin/main（R5498 两笔 layout-engine 提交，非本流域）并完成全链验收。

## 勘根（基线 solo 失败签名入库）

HTML rendering §15.3.4 "Attributes for embedded content and images"（#attr-dimensions）：HTML 命名空间 `img/iframe/embed/object/video` 等的 `width/height` 内容属性按 rules for parsing dimension attribute values 映射为 CSS `width/height` presentational hints。style-system `collect_presentational_hints` 已有 `img`（s283 谱系）与 `table` 同型 `html_length_attr` 臂，`object/embed` 缺臂：

- driving 基线（pre-fix solo，/tmp/s396/solo-base-*.log）：
  - `the-object-element/object-attributes.html` **4F**：`object.width` expected "100px" got "auto"、`object.height` expected "50px" got "auto"（driving 对）；`object.contentWindow` ×2F 为独立在册红面（contentWindow 语义，与本 hint 无关，本切片不动）。
  - `the-embed-element/embed-dimension.html` **2F**：height/width 均 expected "100px" got "auto"。

## 施工（单臂 + 单测）

- `crates/style-system/src/lib.rs` collect_presentational_hints 增 `"object" | "embed"` 臂：`width/height` 属性经 `html_length_attr`（rules for parsing dimension attribute values 同口径，无单位数→px、百分比透传）映射 dimension hint，与 img/table 臂同型。
- `crates/style-system/src/presentational_hint_tests.rs` 增 `object_embed_width_height_attrs_map_to_dimension_hints`：object 100/50、embed 100/100、百分比透传（50% → width:50% 仅一条）、无属性零 hint 四面钉死。
- iframe/video 同属 #attr-dimensions 但语料无 driving F 面（the-iframe-element/the-video-element 目录无 getComputedStyle width 断言案），按需求内原则不扩臂。

## 验证（串行管线，/tmp/s396/ 各段旗标+全量日志）

- driving solo 修复后（/tmp/s396/solo-fix-*.log）：object-attributes 剩 2F（contentWindow 独立面，原样）；embed-dimension **exit 0 全绿**（2F→2P）。
- 全树 sweep join（基线 = s395 验收 sweep /tmp/s394/cur-s395.pairs unique 8463，(key,state) 对 0x1F 分隔 LC_ALL=C join）：
  - sweep1（sweep.log）：8466 行 = **8209P/221F/36T**，unique 8463；gone 5 = driving 4F + select-home-end-pagedown-pageup.optional.html Timeout；new 5 = driving 4P + 同文件 Fail。
  - **T/F 时序 flake 定谳**（s292 跨跑状态矩阵法）：该文件同二进制 sweep1=Fail、sweep2=Timeout 翻转=时序 flake；机制排除——文件内 object/embed 元素数 0、width 属性数 0，hint 臂无法触达；solo 直跑恒 Fail（flake-run1/run2 双跑一致，keyboard-nav 在册红真态）；s393/s394/s395 三代 sweep 该文件均 Timeout（/tmp/s394/base*.pairs + cur-s395.pairs）。
  - sweep2（sweep2.log，稳定复跑）：8466 行 = **8209P/220F/37T**，unique 8463；对 s395 基线 **gone/new 仅 driving 4 对（F→P），零意外位移、零 Timeout 位移**。在册红 224F+37T→**220F+37T**。
- 单元：`cargo test -p zero-style-system -j 2`（test-guard 包裹）**2516P/0F**（新增单测在列）；`cargo test -p zero-engine -j 2` **3835P/8F**——7F 与 s395 在册红集完全同形（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_service_worker_messages_dispatch / response_body_readable_stream_r2967 / t8k_websocket ×2），新增 1F=r5296_single_line_inflated 为渲染流 R5484 已在册的预存 env 竞态（s395 轮该测试通过、本轮竞态命中，非本变更面），零新增红。
- 产品 smoke：`make product-smoke` PASS（struct-check 0 issues，article 计数 1）；`make product-smoke-legacy` PASS（42 fixtures，struct-check failures: 0）。
- fmt/clippy：`cargo fmt --check -p zero-style-system` exit 0；`cargo clippy -p zero-style-system --all-targets`（test-guard 包裹）Finished clean（与中断 session clippy-s396.log 同形）。
- shim 零改动：本切片无 JS/part 变更，无 shim 拼装检查项。

## 残差与边界（钉死入账）

- `object.contentWindow` 2F 独立在册红面（object contentWindow 嵌套浏览上下文语义），与 dimension hint 无关，留 forms/embedded-content 散单池。
- select-home-end-pagedown-pageup.optional.html 为 sweep 负载下 T/F 边界时序 flake（真态=Fail 在册红），分诊口径按 Fail 计。
- r5296 env 竞态属渲染流在册（R5484），本流零债。

## 下一步

- 在册红 220F+37T 下一候选：forms 树散单/分诊推进或按域 solo 分诊（承 s392 方法论）；挂账池不变（`__zw_resolve_handle_selector` forward map 空返低优先勘根；渲染流 5 面→rendering-compat；form-submission-target 6T→navigation M3；sizes-auto lazy 半边待勘根；A2+D 簇立项征询待用户决策）。
