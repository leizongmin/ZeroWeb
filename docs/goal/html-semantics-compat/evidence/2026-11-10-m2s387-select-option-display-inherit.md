# M2 切片 387（s387）——select-option-display-inherit 施工收口：option `display:inherit`→contents + closed select 盒面抑制（3F→3P，域 corpus 红 152F+2T→149F+2T）

日期：2026-11-10（承 s386 下一步候选：select-option-display-inherit 勘根切片——流内唯一明确 style 面无门案）
域：html/semantics/forms/the-select-element（driving 案 select-option-display-inherit.tentative.html，3F）
性质：**施工轮——上游决议勘根 + 三面施工 + 全树验收对账**（跨 429 中断两轮完成：上轮遗留未提交实现，本轮核验、保真修补与验证收口）

## 勘根（bugzil.la/2028130 评论链全文 + whatwg/html#3748）

Emilio 修复链定谳（bugzilla 2028130 comments 1–21，本地存档 /tmp/s385/bugzilla-2028130-comments.json）：

1. Firefox 149 把 select 改 UA shadow tree 后，option 的**样式继承父是 slot 而非 select 元素**——author `display:inherit` 在 option 上解析为「继承 slot 的 computed display」，Chrome 恒 slot 化、Firefox 2026-04 picker 落地后同；Emilio bug 内实测 Chrome classic 的 `gCS(option).display` 亦返回 `contents` 而非 `none`。
2. option 不渲染的面由** picker wrapper 的 display:none 子树**承载（"Adds a wrapping element instead of hiding the <slot> directly, matches other browsers"）——与 author display 声明无关，无论声明为何都无盒。
3. WPT 案即上游 wpt PR #59231（Emilio 同修附案），driving 断言两面：`display:inherit` → computed `"contents"` ×2（single/multiple select）+ closed 单选 option 零 gBCR（width/height/top/left 全 0）。

期望值由此定格：gCS 面 = contents（slot 继承父），盒面 = 恒无（picker-none 面）。ZW 无 picker 物化（customizable-select A2 簇立项面，s386 分诊），两级语义需**分面补**：style 面改写显式 inherit 声明的胜出值，layout 面把 select 内 option/optgroup 摘出盒面。

## 施工（3 文件 3 面 + 1 gate，同 `ZW_SELECT_SUPPRESS_OPTIONS` gate，default-on，`=0` 回退）

| 面 | 文件 | 内容 |
|---|---|---|
| style 面（gCS ×2 子测） | `crates/style-system/src/lib.rs` `compute_element_style_internal` 9.2 钩子 | option 且级联胜出 display 值为显式 `inherit` → resolved display 改写 `Contents`。**作用域限 select 祖先**（谓词与 layout 侧同判据，本轮保真修补——select 之外的 option 继承父是真实父节点，`inherit` 语义不变；无 author 声明时 UA none 胜出路径不变）。谓词挂在 tag+级联值双门之后，非热路径。 |
| 盒面（零 gBCR 子测） | `crates/layout-engine/src/tree.rs` `build_subtree` 汇聚点臂 + R3848 hoist 臂 | select 内 option/optgroup（祖先链含 `<select>`，经 optgroup 间接嵌套两形态均命中）→ `Display::None` leaf / 跳过，镜像上游 picker-none 子树面。select 之外 option（datalist 等无语料钉面）不在谓词内。 |
| IFC 收集面 | `crates/layout-engine/src/inline/collect_items.rs` 收集 walk 抑制臂 | 同谓词跳过收集与下降（Path A `!styles.is_empty()` 臂；**Path B 行为不变**——空 styles 时既有 `ua_hidden_without_styles` 恒排除 option/optgroup（含在 R1679 口径内），两通道合流后 option 内容恒不入 IFC，标签绘制仍由 `paint_select_value` 显式承担）。 |
| gate | `crates/layout-engine/src/inline/runtime_flags.rs` | `select_options_suppress()`（LazyLock 进程级单读，R5346 纪律：env 不入 per-node 热路径）。与 R1679 option UA 抑制同 gate，`ZW_SELECT_SUPPRESS_OPTIONS=0` 一键回退三面。 |

**跨 429 承接说明**：tree.rs/collect_items.rs/runtime_flags.rs 三面为上轮遗留未提交实现（勘根已按 s386 候选口径完成），本轮核验其与决议原文一致后，补两处保真：①style 钩子补 select 祖先限定（上轮实现漏限定，自由 option + `display:inherit` 会误报 contents，与自身注释口径矛盾）；②rustfmt。s387-probe-contents-box.html 探针（gitignored 本地语料）为上轮三角定位资产（display:contents 盒物化五变体 gBCR 导出），保留单列记账。

## 验证

- **driving**：`make testharness-html-semantics FILTER=select-option-display-inherit` **3/3 Pass**（patch 前 s385 基线同文件 3F——F→P 证据链两臂齐全），修补后复跑仍 3/3，exit 0。
- **clippy**：`make guarded-clippy` workspace `-D warnings` 全绿（修补前后两轮）。
- **fmt**：`cargo fmt --all -- --check` 干净（首轮 tree.rs 一处换行修为 rustfmt 落盘）。
- **产品 smoke**：`make product-smoke` PASS（welcome struct 0 issues + morning-work struct 0 issues）；`make product-smoke-legacy` **42/42 fixtures struct PASS**（37-form-controls PASS 3.85% 口径维持——option 抑制臂不波及 legacy 表单结构面）。
- **全树验收 join**：无 FILTER 全语料 sweep **8184P/265F/38T（8487 行，8484 唯一 (key,state)）** vs s385 基线 8198P/270F/37T（8505 行）。状态位移 join 恰 **3 键全 F→P** 且全部落在 driving 文件，零意外翻转零新增红。ΔP/ΔF/ΔT 逐键闭合：ΔP −14 = 切片 +3 − on-blur 截断 17；ΔF −5 = 切片 −3 − home-end 截断 3 + 新探针 +1；ΔT +1 = on-blur 文件级 Timeout 新观测。键集 −20+2=−18 恰合 8505→8487。
- **截断伪影归因**（两文件 × 基线/本轮 × solo 采样矩阵）：①focus-dynamic-type-change-on-blur（the-input-element 域，与切片零交集）基线 80P 无 T → 本轮 sweep 63P+1T；**solo 复跑 ×2 全 80P/0T exit 0**（同二进制跨跑状态翻转=s292 矩阵判据）→ 8-job 并行负载时序 flake，且该文件在档已有「79/80 行截断族」史（1275 行记录）；②select-home-end-pagedown-pageup.optional（簇 A 键盘面）两轮均文件级 Timeout（红态不变），子测行 3F→0F 为完成深度截断非状态翻转。两案均记 flake 不入红账位移。
- **make test**：workspace 全量在 zero-engine lib 处被 **7 组合态在册红**阻断（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_service_worker_messages_dispatch / response_body_readable_stream_r2967 / t8k_websocket ×2，均 img settle/fetch/SW/websocket 时序族，与本流 layout/style 改动零交集）。**stash A/B 定谳非本流**：本流改动 stash 后同套件基线复跑同 7F（1457P/7F 完全同形）；其中 img/svg 对 editing-compat 控制面在档已录（2026-11-07 checkout A/B：015172475 即红、其^ 与刀体均绿——他流窗口红）。受影响 crate 套件定向全绿：`test-guard -- cargo test -p zero-layout-engine -p zero-style-system` **exit 0 零失败**（2457P+2505P 及全部集成套件）。make test 全绿恢复归 engine 属流（并行纪律：main 是组合态，各流只写自己的控制面）。

## 口径更新

- the-select-element 域 corpus 口径：**152F+2T → 149F+2T**（簇 C 17F → 14F：display-inherit 3F 销案；簇 A 135F+2T 立项挂账维持；s325 探针 22F + s387 探针 1F = **23F 单列**）。本轮 sweep 域观测 169F+2T = corpus 148F+2T（home-end 3F 截断未观测）+ 探针 23F，逐项恰合。
- 全树键级基线更新为 8484 唯一 (key,state)（s385 8505 行基线的键级继承，-18 均已归因：伪影 20 出 + 探针/Timeout 2 入）。

## 下一步

- select-multiple-resize 勘根（s386 三分余下的待勘根 1F）。
- A2 簇主体立项征询维持待用户决策（六子面分期建议不变）。
