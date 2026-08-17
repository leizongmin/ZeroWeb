# R34xx 第七批证据：arcTo 真切线弧（2026-08-16）

## 修复（WPT driving）

1. **arcTo 真切线弧**（path.rs flatten_to_vertices + raster.rs flatten_path_opts）：
   P0→P1→P2 切线点 T1/T2 + 弧展平（共线/半径过大 → lineTo(P1)）；**无子路径 →
   moveTo 首控制点不画线段**（"nothing is drawn up to it"）；负半径
   IndexSizeError（ctx + Path2D）
   - driving: 2d.path.arcTo.ensuresubpath.1/2、arcTo.negative
2. 子路径追踪（subpath_open——MoveTo 置位/ClosePath 复位）

## 状态

- path-objects 174/29（arcTo 全过；剩余 arc 形状/描边端帽/贝塞尔缩放/roundrect
  描边 ~29——stroke 几何深项）
- canvas 789 / engine 2152 / render-foundation 644 全绿；clippy 零警告
- 全目录：color-type 4/4、wide-gamut 12/12、filters 13/13、layers 30/30、
  pixel-manipulation 71/71（主 + worker 全绿）
