# R5176 变换元素 step 8 绘制相位重排

日期：2026-10-18 ｜ 前轮：R5175（perspective used value clamp + perspective() 平面投影折叠） ｜ 基线 corpus：1,082（R5175 ON 终态）

## 域核与定谳

R5175 tail 定谳的挂账根因落地：`perspective-zero` / `perspective-zero-point-five`（各 2.08%）在 R5175 后变换数学全链正确（painter probe 单测绿、transform3d-perspective-006 全链翻绿），残卡纯为**绘制顺序**——css-transforms-1 §4 + CSS 2.1 Appendix E step 8：transform ≠ none 的元素建立堆叠上下文，与 positioned 元素同属 step 8 绘制组按 **tree order** 混排；ZW painter 此前把变换元素留在 steps 3/4/5 in-flow 相位 → DOM 序在后的 `#cover-me`（position:relative 红块）晚绘，盖住已正确缩放 (0,0)-(100,100) 的绿块。

规范依据：CSS 2.1 Appendix E step 8（positioned descendants + stacking context 成员同组 tree order）；css-transforms-1 §4（transform ≠ none 建立 SC）；css-transforms-2 §perspective-property（perspective ≠ none 建 SC 且 **applies to transformable elements**——display:inline 非原子/contents/none 无 principal box 不适用，R4070 contain 臂同款排除先例）。

## 实现

四处（crates/engine/src/paint/painter/mod.rs + crates/layout-engine/src/engine/sizing.rs）：

- **收集**（`collect_positioned_descendants`，新增第 7 参 `transform_phase`）：positioned 推入臂之后、`creates_stacking_context` continue 之前新增臂——`box_paints_in_transform_phase`（非 positioned + z_index==0 + transform ≠ none）的 in-flow/float 变换子收进所属 scope 的 z_index==0 桶，step 6 flush 与 positioned 项 tree order 混排施绘；**不下钻**（transform 自成 SC scope，其 positioned/变换后代由它自己的 collect/flush 在其效果窗口内收集，R505 效果覆盖同理）。裁剪矩形照 relative/sticky 口径整保留（变换元素 CB 仍是 static 父链）。
- **主循环排除**（paint_node_inner steps 3/4/5 谓词）：`!(transform_phase_defer && box_paints_in_transform_phase(child, styles))`——与收集同旗标同判（收集集 ≡ 排除集，否则双绘/漏绘）。旗标快照 `transform_paint_phase_enabled && multicol_fragment_depth == 0`。
- **multicol 片段领地**：scope collect 对 cso 子树整支跳过（片段内容无收集器可达），若主循环排除变换子则盒**永久丢失**——`multicol_fragment_depth` 计数器 gate 领地内 deferral（depth==0 才 defer）；片段内两处局部 collect 调用传 `transform_phase=false`（防双绘）。
- **perspective 建 SC**（sizing.rs `style_creates_stacking_context` 追加臂）：`perspective ≠ none（Px(0.0) 哨兵）&& transformable display` → SC。这是变换子相位重排的**前提配对**：R5174 父透视折叠要求变换子收集落在父 ctx 窗口内——perspective 父入 scope 后，其变换子 defer 到父 scope flush（paint_node 包裹窗口内），折叠不丢。painter 侧 `persp_pushed` 同步补 transformable display 门（ctx 窗口 ≡ scope 窗口两侧同判）。

kill-switch：`ZW_TRANSFORM_PAINT_PHASE=0` 回退相位重排（Painter::new 构造期单读）；`ZW_PERSPECTIVE_STACKING_CONTEXT=0` 回退 perspective-SC（OnceLock 进程级单读——R5108 per-node env::var +54% 判例；unit 不翻 env，R3858 教训，OFF 反证由 corpus OFF 轮承担）。

## 迭代定谳（corpus A/B 同 binary）

| 迭代 | 结果 | 归因 |
| --- | --- | --- |
| ON v1 | **翻红 1**：perspective-untransformable-no-stacking-context | perspective-SC 臂无条件——inline span（perspective:100px）误建 SC → z:-1 abspos 子在 span 背景前绘（红盖绿）。css-transforms-2 applies-to 语义：非 transformable 不建 SC 不适用 |
| ON v2（终态） | **1,078 翻绿恰 4 / 翻红 0** | display 门（Inline/Contents/None）落在 sizing SC 臂 + painter persp_pushed 两侧。翻绿：perspective-zero 2.08%→0.04%、perspective-zero-point-five 2.08%→0.04%、css-writing-modes/available-size-020、available-size-021 |

- **基线（stash 归零态）1,082** 恰等 R5175 ON 终态 ✓。
- **OFF-A**（`ZW_TRANSFORM_PAINT_PHASE=0 ZW_PERSPECTIVE_STACKING_CONTEXT=0`）fail 清单与基线**逐项相同**；**OFF-B**（仅 `ZW_PERSPECTIVE_STACKING_CONTEXT=0`）同样**逐项相同**——SC 开关零独立效果，4 案翻转全部来自相位重排本身，双 kill-switch 判别成立。
- 驱动族 probe：perspective-zero-3 0.04% 维持绿；perspective-zero-2 6.25% 维持红（preserve-3d 挂账域，本轮不涉）。

## 单测迭代（default display 陷阱）

painter `persp_pushed` display 门落地后 r5174 ×4 / r5175 ×1 单测翻红：`ComputedStyle::default().display == Inline`（CSS 初始值），手构样式的「块级父」被门误挡、persp ctx 不入栈。真实引擎无此面——UA 层 `ua_default_display` 给 div 显式 Block（R109 回归钉死），corpus 同窗翻红 0 佐证。修正：5 处测试父样式显式 `DisplayValue::Block`（4×r5174、1×r5175，测的本就是块级父），加注释说明门与 default 陷阱；新钉 r5176 布局侧测试同口径显式 Block。

## 验证

- unit +4：painter `r5176_transformed_paint_phase.rs` ×3——(1) transformed_after_positioned_sibling（perspective-zero 案形：fills 序 green>red + TP a=2 不变）；(2) perspective_parent_scope_keeps_ctx_pairing（perspective 父 scope + 变换子 defer 后折叠 s=2 不丢）；(3) inline_perspective_no_ctx（inline span perspective 不推 ctx、不折叠）。layout `r2302_isolation_stacking_context_tests.rs` +1——perspective-SC transformable 门（Block+perspective→SC、Inline+perspective→非 SC、none→非 SC）。
- 既有钉维持：zero-engine lib **3,154 绿**（含 r5174 ×10、r5175 ×11 全绿）；zero-layout-engine **1,990 绿**。
- corpus 16,590 A/B 同 binary：基线 1,082 / ON **1,078**（翻绿恰 4 / 翻红 0）/ OFF-A ≡ 基线 / OFF-B ≡ 基线。
- make reftest **712/712** EXIT=0。
- make test 全腿：workspace / renderer --bin+--lib / quickjs clippy 清 / gpu probe→**gpu 96P + surface 一致性 1P** / quickjs crates **1,905P** / quickjs renderer --bin 0P / quickjs renderer --lib **269P + 1 预存环境红**（r2940，R5174 起 stash 归零态基线同红在案，归因沿用）；browser --bin 腿 656P + 2 已知环境红（剪贴板前置 + `window_surface_present_smoke` 无 display，R385 在案）xvfb 前缀复跑 2P 绿；browser --tests 腿 655P + 3F——2 环境红同上，`bookmark_context_menu_open_in_new_tab_loads_url` **本轮 stash 归零态基线同红**（"应新建一个前台标签页" 1≠2，疑与并行流 dnd 点击抑制面相关，渲染域零归因，留待该流域核）。
- product-smoke：**EXIT=0** 全 struct PASS。
- bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-layout-engine,zero-engine`：ON **2 FAIL**（mb/block_layout_1000 3.41M vs 预算 2.94M、mb/flex_layout_1000 3.11M vs 3.09M）；**stash 归零态基线同窗复跑同 2 指标 FAIL 且更糟（block 3.66M、flex 3.10M，且多超 1 指标共 3 FAIL）**——环境负载噪声族，零 relax 零归因（R5172/R5174/R5175 判例延续）；page/* 族全 PASS 含 first_paint_wall 三案，NEW=0。

## 挂账（新增/延续）

- **dirty-rect 路径相位不一致**（新域，预存粗粒度）：`paint_node_in_rect` 无 positioned 收集，本轮相位重排不涉——脏矩形局部重绘下的变换/positioned 混排序与全绘不一致，挂账待域核。
- **atomic inline 替换元素 transformable 排除**：R4070 口径（display Inline 排除臂仅指非替换 inline）；替换元素 atomic inline 的 perspective/transform 适用性未单测钉，随 contain/transform 交叉域补。
- body-background 传播缺陷（延续，reftest ref 侧保真）。
- perspective-zero-2 preserve-3d 函数式 perspective 继承（延续，真 3D 管线域）。
- 非 px 单位/Calc perspective 绘制侧 <1px 未 clamp（FIXME 代码挂账延续）。
- 延续：preserve-3d 完整域、rotateY 域 reveal 缺陷、apply_transform_post 露出区域下层恢复根修、perspective-origin-scroll 2.08% 恒值簇、R5172/R5171 挂账组。
