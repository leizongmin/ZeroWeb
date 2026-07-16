# RFC：Skia C-Dep — FreeType→Skia 字形光栅化对齐（font-wall 唯一真实 unlock）

> **状态**：RFC 草案（待 feasibility spike 验证后实施）
> **日期**：2026-07-16（R1554）
> **承接**：R1553 font-wall scoping（metric/LoadFlag/gamma 三小切片全排除）
> **授权**：用户裁决包 #1（① 字体/C 依赖：许可证兼容 + feature-gate + A/B 验证；Skia 等大依赖须 **RFC + 小切片**）

---

## 1. 背景与动机

font-wall 是当前 rendering-compat **残余的主导根因**（R1552 证跨目录 near-pass 全 font-wall）。
R1553 实测排除三 Rust 侧小切片：
- **metric coherence**：DEAD（R1090/R1095/R1160/R1206 四轮 net-negative；R876 须三方同改）
- **LoadFlag tuning**：穷尽（R1069 DEFAULT 最优）
- **gamma-correct blend**：net-neutral（R1553 实测 writing-modes 134 vs 135）

★ 残余 = **FreeType coverage 算法 vs Skia coverage 算法**（位图生成本身，非 compositing）。
FreeType 自 R1068/R1159 default-on 已替换 fontdue 光栅，但 **FreeType 的 AA coverage 与
chromium（Skia）的 AA coverage 算法不同**——这是无法用 Rust 侧 flag/metric/blend 修正的
**光栅化器本质差**。唯一 unlock = 用 Skia 光栅化字形（对齐 chromium 算法）。

**预期收益**：font-wall 主干 unlock——bidi（wm 79）+ line-box（wm 50）+ 各目录 near-pass 带
（css-text/css-text-decor/selectors/...）en masse。保守估 +100~300 oracle（取决于覆盖范围），
主指标（aggregate ~54.8%）显著提升。

## 2. 现状 font-stack（R1553 scoping §1）

| 功能 | 实现 | 文件 |
|------|------|------|
| Shaping | rustybuzz（OpenType）+ fontdue 回退 | `crates/render-foundation/src/font/shaper.rs` |
| **Rasterization** | **FreeType**（default-on）→ fontdue 回退 | `crates/render-foundation/src/font/loader.rs:37-92` |
| Metrics | fontdue（ascent/descent/line-gap） | `loader.rs:418-443` |
| Composite | `blend_pixel`（straight sRGB，cpu/mod.rs:600） | `crates/render-foundation/src/cpu/mod.rs` |

光栅化入口：`FontLoader::rasterize_glyph` → FreeType `load_glyph(LoadFlag::DEFAULT)` +
`render_glyph(RenderMode::Normal)` → `GlyphBitmap{data: Vec<u8> 灰度 coverage, width, height,
bitmap_left, bitmap_top}`。`blit_glyph_bitmap` → `blend_pixel` 把 coverage 合成到 framebuffer。

## 3. 关键可行性发现：skia-safe 提供 **预编译二进制**

经调研（rust-skia repo + crates.io）：
- `skia-safe`（Rust safe 绑定）+ `skia-bindings`（C++ 桥）—— **大多数 feature 组合有预编译
  二进制**，匹配 crate 版本 + feature flag 时 **下载预编译 lib**（非从源码构建 Skia C++）。
- 从源码构建才需 GN + ninja + LLVM + Python + depot_tools（重型，~GB）。
- ★ **预编译路径 = 下载 lib（~50-100MB，网络）+ 编译 Rust 绑定**——本环境（WSL2 linux x86_64）
  **可行**（无需重型 C++ 构建）。

**许可证**：Skia = BSD-3-Clause（兼容 MIT）。✓ 满足裁决 #1 许可证要求。

## 4. 设计

### 4.1 集成点：替换/并行光栅化路径

新增 feature `skia-raster`（default-off，feature-gate 满足裁决 #1）于 `crates/render-foundation`：
- `FontLoader::rasterize_glyph` 内：若 `skia-raster` on，走 Skia 路径生成 `GlyphBitmap`
  （**保持 GlyphBitmap 接口不变**——coverage `Vec<u8>` + 几何），else FreeType（现状）。
- kill-switch env `ZW_SKIA_RASTER=0`（feature on 时仍可运行时关闭）。
- 其余 pipeline（shaping/metrics/composite/blit）**不变**——仅替换位图生成。

### 4.2 Skia 光栅化实现（首切片）

用 `skia_safe` 的 `FontMgr::new_from_data(font_bytes)` + `TextBlob` / `GlyphRun` 或
` Typeface::glyph(glyph_id)` + `Surface` 软件光栅化单字形 → 取 coverage 位图 → 转 `GlyphBitmap`。
关键：用 Skia 的 AA（`Paint::default()` + `anti_alias=true`）+ 同 size/位置约定。

### 4.3 坐标/度量一致性

Skia TextBlob 与 FreeType 的 `bitmap_top`/`bitmap_left` 约定不同——须在转换层对齐（`GlyphBitmap`
坐标约定见 `loader.rs:12-16`：`y_offset = bitmap_top - height`）。首切片仅验证 coverage 差，
坐标对齐在集成阶段处理。

## 5. 风险拆分与小切片（裁决 #1 要求）

| 切片 | 内容 | 验证 | 风险 |
|------|------|------|------|
| **S0 feasibility spike**（独立 scratch 项目） | scratch Cargo 项目加 `skia-safe`，预编译下载 + 编译 + 光栅化 1 字形 | 能否构建？coverage vs FreeType/chromium？ | 网络/下载失败；预编译不可用 |
| **S1 feature-gate 集成** | `skia-raster` feature in render-foundation，`rasterize_glyph` 双路径 + kill-switch | `cargo build --features skia-raster`；make test 零回归（feature off） | feature 相互依赖（v8/freetype） |
| **S2 A/B yield 验证** | feature on 跑 `make reftest-oracle` 多目录 | font-wall 簇 net≥0？welcome 字节稳定？ | yield 不达预期（revert feature） |
| **S3 default-on 决策** | 若 S2 net≥0 且无关键回归，翻 default | 全量 oracle + product-smoke + CI 7-target | CI 跨平台预编译覆盖 |

**紧急停止**：S0 若预编译下载/构建失败（网络受限或平台无预编译）→ 记录 blocker，转备选
（系统字体桥 / 接受 font-wall plateau / 重审 Phase A）。

## 6. 备选方案（若 Skia 不可行）

1. **系统字体桥**（fontconfig + 系统 FreeType 配置对齐 chromium Linux 栈）——可能缩小差但非根本。
2. **接受 font-wall plateau**，转 Phase A IFC 统一（P0 架构）或 multicol breaking（P1）。
3. **swrast/software Skia 替代**（如 `tiny-skia`，纯 Rust 软件光栅——但非 chromium 算法，yield 存疑）。

## 7. 开放问题

- 预编译二进制是否覆盖 linux x86_64 + 本仓库 feature 组合（需 S0 验证）？
- Skia 软件光栅的 coverage 是否真比 FreeType 更接近 chromium oracle？（S0/S2 验证——理论应一致
  因 chromium Linux 即 Skia+FreeType，但 Skia 的 AA pipeline 在 FreeType 之上）
- CI 跨平台（ubuntu/macos/windows）skia-safe 预编译覆盖 + 构建时间影响？

## 8. 下一步（本次执行）

**S0 feasibility spike**：scratch 独立项目验证 skia-safe 预编译下载 + 编译 + 单字形光栅化 +
coverage 与 FreeType/chromium oracle 对比。成功 → S1 集成；失败（网络/构建）→ 记录 blocker +
转备选。
