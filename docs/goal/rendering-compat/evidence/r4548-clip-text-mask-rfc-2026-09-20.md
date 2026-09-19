# R4548 评估：background-clip:text mask 管线 RFC（P 轮 0 net code——clip-text 族 10 fail 定性 + 管线设计蓝图）

- 日期：2026-09-20（R4548 P 轮——R4547 下轮方向①）
- 谱系：R4525 clip-text 实底彩字 v1（color-only hack，bg image 非空不触发）→ 本轮
  全族评估。

## 现状定性（逐案探针，10 fail）

R4525 v1 = 「实底 bg + clip:text 时把 bg 色下探为字形色 + 本盒 bg fill 抑制」的
**color-only 等价渲染**，适用面极窄。探针实证的失效形态：

1. **bg image 非空**（multi-line：`linear-gradient(green,green)` + red bg）→ v1 禁用
   → 背景图全盒铺涂（绿矩形盖满，字形裁剪完全缺失）；
2. **descendants**（Block/Float/transformed/Table 子树文本）→ 字形色下探的
   context 栈在 float/transform/table-cell 子树断裂（字形以 text 色渲染、bg 实涂）；
3. **blend-mode / stacking-context-child / flex / inline / on-body-scroll /
   constrain-geometry** → 同族（栈传播 + 混合模式交互）。

共同缺口：**字形 mask 能力**——把盒的背景（color+image）只绘制在「该元素及其后代的
字形覆盖区」内。这是 R4525 记账的「mask 管线域」，color hack 无法覆盖（image 案必须
逐像素 mask）。

## 管线设计蓝图（CPU raster 路径）

新增绘制期离屏合成（painter 内，非新图元类型——避免 IPC/convert 面改动）：

1. **触发**：paint_background 检测 `background_clip == Text` 且（bg color 或 image
   非空）→ 进入 mask 模式（现有 `bg_clip_text_color` 栈改造）。
2. **离屏收集**：本盒子树（含后代）绘制期，把「背景层」重定向到 offscreen
   FrameBuffer（bg color + bg image 全盒绘制）；字形栅格写入第二 offscreen 作
   alpha mask（paint_text 现有 glyph 栅格路径复用，字形色 → 白色 alpha）。
3. **合成**：dst = bg_offscreen × mask_alpha（逐像素乘法），回贴主 framebuffer。
   blend-mode 案在合成步选混合函数（§css-backgrounds-4 与 mix-blend-mode 交互按
   chromium 实测对齐）。
4. **作用域**：offscreen 生命周期 = 本盒 paint_node 区间（栈式 push/pop，与
   `bg_clip_text_color` 栈同位）；后代 SC（stacking-context-child 案）按 chromium
   实测决定是否并入 mask（初版并入，回归再收）。

## 切片与预期

- slice 1（multi-line 三案 + inline/flex ≈5 案）：单元素 + 简单后代的 mask 合成；
- slice 2（descendants/stacking-context ≈3 案）：后代字形并入 mask + SC 边界；
- slice 3（blend-mode/on-body-scroll/constrain-geometry ≈2-3 案）：混合 + 滚动/
  几何约束边界。
- 预期合计翻绿 ~8-10 案（10 fail 中 1-2 案可能另有独立缺陷）。

## 风险

1. offscreen 合成的性能面（仅 clip:text 页触发，bench 基准不含该形态）；
2. 字形 mask 的亚像素抗锯齿与 ref（chromium glyph AA）的阈值差（fuzzy 容差
   0-5846px 余量充足）；
3. blend-mode × mask 的合成顺序语义需 chromium 探针校准；
4. GPU 路径（render-foundation GPU mesh）不在本 RFC 范围（reftest 默认 CPU）。

## 结论

可行（CPU 离屏合成路径清晰、无跨 crate 依赖），建议下轮起 slice 1 实施。关联：
clip-border-area 6 fail 为另一域（background-clip:border-area × border-shape/
blend/box-decoration-break），不在本 RFC。
