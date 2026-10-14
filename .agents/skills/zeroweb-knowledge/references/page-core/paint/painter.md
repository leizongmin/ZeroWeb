# 绘制顺序、文本、控件与视觉效果

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位页面图元生成、层叠错位、背景传播、文本染色和控件外观。`Painter::paint` 消费 LayoutBox 与计算样式，不负责 GPU 呈现；物理度量见 [字体整形与光栅化](../../graphics/render-foundation/fonts/shaping-and-raster.md)，最终 IFC 复用见 [布局定稿](../layout/inline/finalization-and-reuse.md)，filter 隔离见 [SVG 与过滤链](svg-and-filters.md)。

### User Task Navigation
- 顺序、坐标、效果窗口：`crates/engine/src/paint/painter/mod.rs`；文本、边框、控件分别在同目录 `text.rs`、`border.rs`、`controls.rs`。
- 验证参考：`crates/engine/src/paint/tests/css_remaining.rs` 守护空子树 transform 和背景传播，`tests/wpt-runner/src/reftest.rs` 消费图元与 canvas_images；见 [验证入口](../../../../zeroweb-verify/SKILL.md)，初始化资产已在 d0f281d509de3bf4f9dcd7e8dd01dac3464a663b 提交，本域未实跑。

## Gotchas
- 画布背景传播须有正 viewport 宽高及 DOM：html 背景优先，否则透明 html 借用 body 背景，传播节点自己的背景随后跳绘以避免图像相位重复；单独构造 Painter 不设置 viewport 会漏掉此路径。 (`crates/engine/src/paint/painter/mod.rs`, `crates/engine/src/paint/tests/css_remaining.rs`)
- 空子树不能发射 transform 像素后处理，否则已绘画布背景和兄弟可能被清白、重采样；空判定除 PrimitiveCounts 的七类外还检查 filters/blend_modes，backdrop-filter 本身算可见输出。 (`crates/engine/src/paint/painter/mod.rs`, `git:4f6eee749db719c97f9531c81be7d1fe545b4317`)
- 有实色背景也不必出现 TransformPrimitive：满足单 fill、无其他视觉图元等门控的叶盒可直接矩阵映射四角生成 path_fill；只统计 transforms 数量会误判变换丢失。 (`crates/engine/src/paint/painter/mod.rs`, `crates/engine/src/paint/tests/css_remaining.rs`)
- 纯 translate 只平移自子树新增图元，不走全场景像素后处理；复合变换直绘已含平移时不再重放 translate，避免祖先/兄弟被移动或子树双移。 (`crates/engine/src/paint/painter/mod.rs`)
- clip:text 的恒色栈会在宿主退出时弹出，而部分层叠上下文后代随后才 flush；文本绘制额外沿 DOM 祖先查最近恒色 clip:text 上下文，不能删成只查当前 paint 栈。 (`crates/engine/src/paint/painter/text.rs`)
- clip:text 当前是恒色背景的字形染色近似，不是真正任意背景 mask；多色渐变或未获纯色缓存的 URL 不激活，URL 还须默认 repeat/size/attachment，透明背景也不激活。 (`crates/engine/src/paint/painter/text.rs`)
- progress/meter 的条必须晚于文本绘制，用后绘覆盖 fallback；缺失或非有限 value 不绘条，progress 无 value 的条纹动画尚未实现。 (`crates/engine/src/paint/painter/controls.rs`)
- color_value_to_render 没有元素上下文，currentColor 回落黑色；背景、边框等应走 resolve_color_current，Mix 与 RelativeColor 也须在当前元素色上下文递归解析。 (`crates/engine/src/paint/color.rs`)
- control-chars-00B 是 test 必须异于空白 ref 的 mismatch 案，相同像素是失败；R5118 历史报告记录伪元素注入 VT 不可见、0/7 失败，并归因于 R644 Cc 占位框未覆盖该路径。当前 text.rs 有 Cc 方块分支仍不足以证明伪元素链路经过它；该提交仅更新报告，根因与当前像素需定向复核，不能写成已修复或本轮实跑。 (`docs/goal/rendering-compat.md`, `crates/engine/src/paint/painter/text.rs`, `git:3b548ed7804ecf0e050da00557ee8adb654e615e`)

## Architecture
- positioned 元素、真实 stacking context 和根是收集 scope：负 z、普通流/float、z=0、正 z 分段绘制；真实 SC 必须自成 scope，才能让 opacity/filter 等新增图元窗口覆盖 positioned 后代。 (`crates/engine/src/paint/painter/mod.rs`)
- positioned 且 z-index:auto 的 pseudo-SC 在自身装饰前 flush 负 z 后代，真实 SC 与根则在装饰后处理负 z；不能用统一 per-node z 排序替代这两个分支。 (`crates/engine/src/paint/painter/mod.rs`)
- 延迟 positioned 项携带中间祖先裁剪，flush 后只裁该项新增图元；absolute/fixed 的包含块在中间 overflow 祖先上方时可免该祖先裁剪，relative/sticky 不同。 (`crates/engine/src/paint/painter/mod.rs`)
- backdrop-filter 在自身内容前采样已绘背景，普通 filter 在子树后应用，再做 transform、opacity、mask 和 blend；这个发射顺序是语义，不是可任意交换的视觉增强。 (`crates/engine/src/paint/painter/mod.rs`, `crates/engine/src/paint/painter/effects.rs`)
- 子滚动 selector 在 paint 时解析为 NodeId；html/body 根滚动在此跳过，页面整体移动归宿主，不能再给根内容减一次滚动偏移。 (`crates/engine/src/paint/painter/mod.rs`)

## Decisions
- opacity 目前按新增图元 alpha 衰减而非普遍离屏组透明度；clip:text 恒色上下文的非零透明度后代字形恢复原 alpha，opacity=0 不豁免，避免错误降低露出的背景亮度。 (`crates/engine/src/paint/painter/mod.rs`)
- promote_compositing_layers 按 opacity、fixed 或非 auto z-index 提升并独立排序，未检查 transform/filter 等所有 SC 条件；它不是 Painter scope 顺序的等价实现，也不是 GPU 层分配已完成的证明。 (`crates/engine/src/composite/mod.rs`, `crates/engine/src/paint/painter/mod.rs`)

## Patterns
- select 按钮标签单独绘制，取首个 selected option，否则首 option，优先非空 label；ZW_SELECT_SUPPRESS_OPTIONS 同时约束 option 抑制、尺寸和绘制，不只是一处 paint 开关。 (`crates/engine/src/paint/painter/controls.rs`)
- fieldset 顶边以首 legend 几何挖缺口，其余边框下移半个 legend 高；不能给整个盒直接画普通四边，否则顶边穿过 legend。 (`crates/engine/src/paint/painter/border.rs`)
