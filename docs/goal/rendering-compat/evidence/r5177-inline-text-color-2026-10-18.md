# R5177 inline 壳 color 折回归因拆分

日期：2026-10-18 ｜ 前轮：R5176（变换元素 step 8 绘制相位重排） ｜ 基线 corpus：1,078（R5176 ON 终态）

## 案形与域核

css-ui/outline-027（0.58%）：6 列 grid，列 5-6 案形 `.inline-text > .outline-container > span > div { display: inline; color: lime }`——inline 壳内无元素子的 inline 子（div 含纯文本）应继承自身声明色 lime 渲染，ZW 渲黑色（外层色）。

根因链（R4300 FLAT_CHILD_WALK 折回的 color 盲区）：`collect_flat_inline_children` walk 对「无元素子的 inline 子」一律折回 pending——text_content 并入外层、归因外层 elem_id（祖先吸收语义，border-color-012 `.text` 壳判例）。R358 per-fragment color 按 owner（frag nid，文本节点取父）styles 查 color → 查到外层色（黑），子元素声明的 lime 丢失。white-space 有 R4300d 判例（模式不一致的子不折回、`build_flatten_run_for_element` 子自身构造独立 run 归因子自身），color 无对应判定。

规范依据：CSS 2.1 §16.1 color 继承——子元素显式声明 color 覆盖继承值，其文本须用声明色渲染，与壳层归因无关。

## 实现（三处）

- **折回臂判定**（`crates/layout-engine/src/inline/collect_items.rs`）：`!gc_has_element_children` 分支 ws_same 判定前新增 color divergent 判定——`child_style.zip(style).map(|(c,p)| c.color != p.color)` 为真时不折回，flush pending 后 `build_flatten_run_for_element` 子自身独立 run（R4300d white-space 判例完全镜像）。双通道同值（R4300e 判例）：layout IFC 有 styles 直判 computed color；paint Path B（空 styles）读 `color_divergent_walk_nodes`，两臂同值不引入 IFC 分段分歧。样式查找复用 child_style 零额外开销。
- **存储信号**（R4310/R4312 模式）：`inline/mod.rs` IFC 加 `color_divergent_walk_nodes: NodeIdSet`；`types/mod.rs` LayoutBox 加 `inline_color_divergent_nodes: NodeIdSet`；`inline_metric_storage.rs` `store_font_sizes_from_ifc` 闭包内按 run owner 复算同一对比较（owner.color vs 父元素 color ≡ 折回臂直判的 gc.color vs walk 层 color）填充——「经 layout 同一判定门该元素必走 flatten 持有 run，覆盖完备」；paint Path B 据此复现「不折回归因外层」。
- **paint 补标**（`crates/engine/src/paint/painter/text.rs`）：IFC 构造注入 `with_color_divergent_walk_nodes(box_node.inline_color_divergent_nodes.clone())`；拆分归因后 frag nid 是内层元素，外层 inline 载体（has_direct_paintable_text 的 PHASEA_STORE_EXT 臂放行 paint_text）Path B 重跑会重复渲染——渲染循环前 `mark_split_owner_ancestor_chain_painted` 对拆分归因 owner 沿祖先链上溯 display:Inline 元素补标 painted_inline_nodes，dedup 拦住外层重跑（抑制面与旧折回行为等价）。

kill-switch：无独立开关（判定与 R4300d white-space 同构内联在折回臂），回退需还原三处。

## 迭代定谳（corpus A/B 同 binary）

| 迭代 | 结果 | 归因 |
| --- | --- | --- |
| ON v1 | outline-027 翻绿但 **4 glyph 双绘** | 拆分归因后 span 盒 Path B 重跑（PHASEA_STORE_EXT 放行 + dedup 不中）重复渲染子树文本——祖先链补标缺失 |
| ON v2 | 双绘消除，**1,078 假持平** | `./target/release/zero-wpt-runner` 是旧 binary（cargo test 只编 test profile 不重编 release）——重编后才见真值。**corpus A/B 必先 `cargo build --release -p zero-wpt-runner`** |
| ON v2 重跑 | **1,077 翻绿恰 1（outline-027 0.58%→0.00%）翻红 0** | 逐项 diff 干净 |
| ON v3（热点外提） | 仍 1,077，失败集与 v2 **逐项一致** | v2 宏内每 fragment 祖先上溯为真实热点（下节），外提为循环前预处理，行为零变化 |

## 性能热点与 bench 判别

三窗 bench 判别（同刻对照）：基线自身 2↔8 FAIL 摆动（机器负载漂移），但 ON 同刻系统性偏慢——incremental +32%、page/medium/paint 127ms 超预算 111ms。定位真嫌疑：render_fragment 宏内**每 fragment** 沿祖先链上溯（全页 O(fragments × depth) 无谓开销，普通 run 归因 id 即外层自身根本不需要上溯）。

优化：宏内上溯删除，改为渲染循环前一次 `mark_split_owner_ancestor_chain_painted`——`inline_color_divergent_nodes` 空集短路零开销（绝大多数页面无 color 差异拆分），非空时仅对 divergent owner 上溯。

第 4 窗（ON 优化后）vs 第 5 窗（stash 基线同刻）对比：

- perf-gate **双窗 PASS**（EXIT=0）；medium paint 104.02ms 回预算内（< 111ms，窗 3 超预算消失）。
- 微基准 7 项 p95 全 ±3% 内：incremental_vs_full **−8.4%**（窗 3 的 +32% 热点随宏外提消失）、paint_complex −2.8%、其余 ±1%。
- medium paint +12.9% **无法归因**：同窗未触碰的 morning **style** 阶段摆 +53.2%（8.76→13.41ms）——R5177 完全不改 style 阶段，噪声幅度 ≥53% 覆盖全部观测 delta（R5172-R5176 噪声判例延续）。零 relax。

## 验证

- unit：`r5177_inline_text_color.rs` ×2——(1) test_r5177_inline_div_text_color（outline-027 案形最小化 `#i { display: inline; color: lime }`，2 glyph 全 lime (0,255,0) 无双绘；修复前 2 黑、中间态 4 双绘）；(2) test_r5177_same_color_inline_no_split（span red > div inline red 对照：同色不拆分，2 glyph 全 red）。
- 既有钉维持：zero-engine lib **3,168 绿**；zero-layout-engine **1,990 绿**。fmt / clippy（双 crate --all-targets）干净。
- corpus 16,590 A/B 同 binary：基线 1,078 / ON **1,077**（翻绿恰 1 / 翻红 0，逐项 diff 仅 outline-027）；热点外提后第 4 窗与第 3 窗失败集归一化 diff **完全一致**。
- make reftest **712/712** EXIT=0。
- product-smoke **EXIT=0** 全 struct PASS。
- bench-gate：见上节，perf-gate 双窗 PASS，NEW=0。

## 挂账（新增/延续）

- **outline-028 仍红**：不同案形（floated descendants + max-content sizing），与本轮 color 折回归因无关，独立域核。
- **多层嵌套 inline 壳仅一层判定**：color_divergent 只比较子与直接父；隔层异色（span red > div green > 纯文本）中间层判定面待域核——现 walk 折回只吸收无元素子的 inline 子，隔层场景走元素子展开路径，预存行为未钉测试。
- backdrop-root 结构性挂账（R5176 tail 定谳，延续）：filter/opacity/mask/mix-blend-mode 4 案需「子树独立缓冲 + 边界出口合成 + 边界外内容躲父 filter」三层联动。
- 延续：dirty-rect 相位不一致、body-background 传播缺陷、preserve-3d 完整域、rotateY 域 reveal、apply_transform_post 下层恢复根修、perspective-origin-scroll 恒值簇、R5172 挂账组。
