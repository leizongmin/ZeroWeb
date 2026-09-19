# R4528 证据：background-clip 逐层（per-layer）——border-on-top 翻绿（clip-border-area 316→317 谱系）

- 日期：2026-09-19（R4528 C 轮——R4527 蓝图 1-6 步实施）
- 谱系：`clip-border-area-*` 多值 clip 声明族。

## 实现

1. `computed_style.background_clip: Vec<BackgroundClipComputedValue>`（R4527 蓝图①）。
2. `parse_background_clip_layers`（css-parser，镜像 parse_background_image_layers：逗号
   分段、任一段非法整条丢弃）+ apply_advanced kill-switch `ZW_PER_LAYER_CLIP=0`（回退
   单值解析 = 多值声明丢弃旧行为）。
3. `ComputedStyle::background_clip_for_layer(i)`（`i % len` 循环补齐，CSS 多层 cyclic
   规则）+ `bg_clip_text_solid()`（任一层 Text + 实底 + 无 image）。
4. painter 消费点语义：
   - paint_background **背景色 = 最后一层 clip**（css-backgrounds-3）；R4526 环带臂
     以 color_clip 判定；
   - rounded corner-override 臂以 color_clip 判定；
   - 画布传播环带（prop_style）= any-layer BorderArea；
   - clip:text 彩字（bg_clip_text_active/stack gate/paint_text helper）= any-layer Text；
   - paint_bg_image **逐层 clip** slice-2 未实施（image 层仍读 clips[0]——多图像层谱系
     multiple-backgrounds/complex 仍 fail，挂账 slice-2）。
5. js_dom_bridge getComputedStyle 序列化逗号列表。

## A/B

- corpus fail-list diff = **唯一 clip-border-area-border-on-top 移除**（4.17→0.00；
  2 层：layer0 green-100 image ring + layer1 none + 色橙=末层 border-box 全盒 ✓）+
  box-shadow-003 既有 flake 相位。corpus 1475（15119/16594 = 91.10%）。
- clip-border-area 族：border-on-top 翻绿；multiple-backgrounds/double/complex/
  box-decoration-break 仍 fail（slice-2 image 层挂账）。
- 全门禁：make test 19,352P/0F、fmt、clippy 双 feature 组 -D、product-smoke welcome
  15.40% 同值 struct PASS、bench-gate 定向 zero-engine GATE PASS（26 指标）。

## slice-2 挂账

paint_bg_image_in_origin 逐层 clip rect/ring（fn 签名缺元素盒几何——需 caller 预计算
逐层 clip rect 传入或传元素盒；multiple-backgrounds/double/complex/box-decoration-break
+ border-shape ×3 域）。
