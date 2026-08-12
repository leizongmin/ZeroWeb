# canvas shadowBlur 极大值致 region padding i32 溢出 panic

**日期**: 2026-08-13（R3355）
**相关模块**: `crates/canvas/src/context/raster.rs`（`shadow_blur_geom`）+ `crates/canvas/src/context/context_impl.rs`（`draw_shadow_rect` / `draw_shadow_path`）
**调用链入口**: `crates/engine/src/js_dom_bridge/canvas.rs`（`setShadowBlur` op，从 JS 经 wire 解析任意 `f32`）

## 问题描述

`shadow_blur_geom(blur)` 计算阴影 box-blur 半径：

```rust
// 旧实现（raster.rs）
let r = ((blur / 2.0).round() as i32).max(1) as usize;
(r, (3 * r) as i32, 3)  // (radius, pad, passes)
```

当 JS 设 `ctx.shadowBlur = 1e30`（或 `Infinity`）时，`(1e30/2).round() as i32` 经 f32→i32 **饱和**到 `i32::MAX`，`pad = 3·i32::MAX as i32` 再饱和到 `i32::MAX`。随后 `draw_shadow_rect` 的 region padding：

```rust
// 旧实现（context_impl.rs:1113-1116）
let rx0 = (rect.left().floor() as i32 - pad).max(0);   // i32 减法溢出
let rx1 = (rect.right().ceil() as i32 + pad).min(cw);  // i32 加法溢出
```

`pad = i32::MAX` 时，`small ± i32::MAX` 在 i32 域加减法溢出 → cargo **debug profile（overflow-checks=true，cargo test 默认）panic**（`attempt to add/sub with overflow`）。

## 根因分析

三重故障链（同一根因 `pad` 失控）：

1. **region padding i32 溢出（确定性 debug panic）**：`draw_shadow_rect` / `draw_shadow_path` 的 `(coord as i32) ± pad` 溢出。修复前确定性复现于 `context_impl.rs:1115` `attempt to add with overflow`。
2. **box_blur_alpha 挂起**：`for dx in -r..=r`（`r = i32::MAX`）达 ~4.3e9 次迭代 × w × h → 几乎永久挂起（test-guard 墙钟兜底杀进程，但单次调用即卡死渲染线程）。
3. **box_blur_alpha `sum: u32` 溢出（确定性 debug panic）**：`sum += row[xx] as u32` 累加 `2r+1` 个 ≤255 值，r=i32::MAX 时 ~17M 次累加后 `sum` 超 `u32::MAX` panic。

**为何 debug 才爆 / release 藏**：i32 加减溢出 release 静默回绕（`small - i32::MAX` 回绕到正小值，`.max(0)` 后 region 仍合法；`small + i32::MAX` 回绕到负，`.min(cw)` 后 `rx1 <= rx0` 早返）——故 release 不 panic 但 region 错乱。而 cargo test（debug，overflow-checks=true）确定性 panic。**注意**：box_blur 的 `sum: u32` 溢出与挂起在 release 也存在（挂起 release 同样卡死；sum 溢出 release 回绕致错误模糊但 box_blur 在 region 内迭代，region 被 release 回绕钳到合法值故 sum 不会真到 17M——除非 region 大）。核心确定性 DoS 是 release 下的**挂起**（box_blur ~4.3e9 迭代）。

### f32→i32 饱和 vs 回绕

Rust `f32 as i32` 是**饱和**（`inf as i32 = i32::MAX`，`-inf as i32 = i32::MIN`，NaN as i32 = 0），**非**回绕。故 `pad` 不是回绕成小值（如 R3354 的 u32 乘法），而是**饱和到 i32::MAX**——一个仍合法的 i32，但后续 `± pad` 在 i32 域无空间。这是与 [[canvas-image-data-u32-size-overflow]]（R3354，u32 乘法回绕）不同的溢出家族：f32→窄整型饱和 + 窄整型算术。

## 解决方案

**双重修复**：

1. **`shadow_blur_geom` 半径封顶**（raster.rs）：`r = (...).min(SHADOW_BLUR_MAX_RADIUS)`（8192）。
   - 封顶后 `pad = 3·8192 = 24576` 远在 i32 内；
   - box_blur 窗口 `2·8192+1 = 16385` 窗样本，`sum` 上限 `16385×255 ≈ 4.2M`（u32 安全）；
   - 迭代量可控（不再挂起）。
   - 封顶值远超任何可见阴影软度（Chrome 实践上限同量级），W3C 无强制上限，属合理实现限制。

2. **region padding 改 saturating_add/sub**（context_impl.rs `draw_shadow_rect` / `draw_shadow_path`）：
   ```rust
   let rx0 = (rect.left().floor() as i32).saturating_sub(pad).max(0);
   let rx1 = (rect.right().ceil() as i32).saturating_add(pad).min(cw);
   ```
   - saturating 下溢钳到 0、上溢钳到 i32::MAX，再经 `.max(0)`/`.min(cw)` 规整到画布内——region 恒合法。
   - 双保险：即使未来半径封顶被移除，region padding 也不再 panic（仅 box_blur 侧仍有挂起风险，故封顶是主修复）。

`draw_shadow_stroke` 的 pad 为 f32 域（`blur_pad as f32 + half_lw`），`min_x - pad` 是 f32 减法 + `.floor() as i32` 饱和——不 panic，未改（rule 3 精准修改：只改真 panic 路径）。

## 如何避免

1. **从外部 f32/u32 推导出的「尺寸/偏移/半径」在使用前须封顶或有界**：`shadowBlur` / `lineWidth` / canvas 尺寸 / gradient 坐标等页面可控浮点，凡经 `as i32`/`as u32`/`as usize` 转窄整型后参与算术，要么封顶到合理上限，要么下游用 `saturating_*` / `checked_*`。
2. **`f32 as i32` 是饱和不是回绕**——别假设它「像 u32 乘法那样回绕成小值」。饱和到 `i32::MAX` 后，`x ± i32::MAX` 在 i32 域**必然**溢出。
3. **box-blur / 卷积窗口的累加器宽度**：窗口 `2r+1` 样本 × 每样本上限，须保证累加器类型容纳。`u8` 像素 × `2r+1` 窗 → 用 `u32` 需 `r < ~8.4M`；若 r 可达更大，封顶 r 或用 `u64`/`usize` 累加。
4. **cargo test（debug）与 release 行为差异**：debug overflow-checks 抓算术溢出 panic，release 静默回绕藏 bug。**不能**依赖 release「不 panic」判断无 bug——release 下挂起（巨大迭代）和静默错误值仍是真 DoS / 正确性 bug。本例 release 下 box_blur 挂起是确定性 DoS。

## 相关测试

`crates/canvas/src/context/tests/context_impl_coverage.rs`：
- `test_shadow_rect_huge_blur_no_overflow_panic_r3355`（fillRect→draw_shadow_rect，确定性 panic 复现）
- `test_shadow_path_huge_blur_no_overflow_panic_r3355`（fill→draw_shadow_path）
- `test_shadow_stroke_huge_blur_no_overflow_panic_r3355`（stroke→draw_shadow_stroke，回归锁）
- `test_shadow_blur_geom_caps_radius_r3355`（半径封顶行为锁：正常/极大/blur≤0）

关联：[[canvas-image-data-u32-size-overflow]]（R3354，canvas 整型溢出家族同根——页面可控尺寸推导）。
