# R1095（2026-07-06）：FreeType 三位一体 metric-placement 协调改动 net-negative -42·font-wall feature-on 仍成立·已回退·零源码

承 R1094（freetype-raster 全 corpus +232 实测 + tuning 缺口诊断：renderer 忽略 bitmap.y_offset，metric 放置未用）。R1094 提出 forward 选项 ① 多 session 协调改动攻 metric-placement 墙。本轮**单 session 实施该协调改动并 A/B**，结果 net-negative，墙成立。

## 假设（R1094 诊断导出，8 次实验未测的新角度）

feature-on（FreeType bitmap 带 proper bitmap_top）+ 三位一体协调改动应能用真 per-glyph ascent 放置，突破 font-wall（超越 +232）：
1. **store-gate 放开**（`inline_finalization.rs:776`）：non-Ahem 也存 valign-aware baseline_y
2. **paint 发 baseline**（`text.rs:1570-1582` use_stored 全分支）：`v_offset = baseline_y_abs − frag.y`（GLYPHPROBE 已验 glyph.y = absolute_baseline）
3. **renderer 应用 bitmap.y_offset**（`cpu/mod.rs:494/514`）：位图顶 = `baseline − bitmap_top`（真 per-glyph ascent）

feature-off 经 `#[cfg(not(feature="freetype-raster"))]` 逐字复现原行为（4759 基线完整保留）。

## 实现（已回退）

- 3 源码 cfg-gated 改动（store-gate + paint v_offset + renderer 2 blit 站点）
- 3 Cargo.toml feature 转发链（layout-engine marker / engine→render-foundation+layout-engine / wpt-runner→engine），统一 `--features freetype-raster` 激活
- feature-on clippy 干净 + feature-off default clippy 干净（`cfg_attr(allow(dead_code))` 处理 PaintFragment.height/is_ahem_font + ahem_uses_embox_position fn）

## A/B 结果（css-text，feature-on 无三位一体 → 三位一体）

| 指标 | feature-on 基线（+232） | 三位一体 | Δ |
|---|---|---|---|
| oracle-pass | 383 (23.2%) | **341** (20.7%) | **−42** |
| credible | 370 | 328 | −42 |
| strict | 88 | 83 | −5 |
| near | 295 | 258 | −37 |

**per-subdir Δ（−42 分布）**：line-breaking −20（68→48）/ white-space −7（48→41）/ hanging-punctuation −4 / hyphens −2 / text-fit −2 / text-transform −2 / text-indent −1 / word-break −1 / word-spacing −1。

★ **−42 广泛分布**（非 non-stored 双移集中）——stored 路径（use_stored，三位体主目标）同样回归。排除「non-stored 路径双移主导」的可能，证 stored 路径本身也负。

## 结论：font-wall 是启发式平衡，feature-on 也不破

**当前渲染是一个精调的启发式平衡**：baseline_y 计算 / strut / half-leading / glyph 放置四轴已被 R990/R953/R817/R841 等共同调谐，在 fontdue（feature-off）和 FreeType（feature-on +232）两种光栅化下都恰好接近 chromium——**尽管每轴单独看都"不正确"**（0.8/font_size 启发式 ascent、忽略 per-glyph bitmap_top）。

三位一体把"放置"一轴改为"几何正确"（per-glyph bitmap_top），打破平衡——其他三轴仍是旧启发式，与新放置不匹配 → 广泛回归。这印证 R876「三方补偿平衡」：单轴"更正确"不产正 yield，须**所有轴同时重调**。

**这是第 9 次 font-metric 单点实验 net-negative**（R834/R836/R849/R875/R1052/R1056/R1067/R1090 + R1095），且**首次在 feature-on（FreeType bitmap）下证伪**——R1090 是 feature-off + 常数 0.928 公式，R1095 是 feature-on + 真 per-glyph bitmap_top 应用。两者皆负 → **font-wall 在 feature-on 也成立，per-glyph metric 放置非 yield lever**。

## 裁决

- 按协议（负 yield → revert）`git checkout` 6 文件回退（3 源码 + 3 Cargo.toml），零 net 源码，default build 绿。
- metric-placement 墙（R1094 诊断的 renderer 忽略 bitmap.y_offset）**确认无法用单 session 协调改动突破**——须 retune baseline/strut/placement 全轴（真多 session spec-rfc 工程量）。
- +232（R1094 实测）仍是 C-dep 的决定性 yield，不受影响。
- **forward 修订**：R1094 的选项 ①（多 session 协调改动）经本轮单 session 尝试确认为真·多 session（非"协调一下就好"）；剩余 forward = ②翻 default-on（CI 6-target 验证 + 计费）或 ③停在 +232。

**门禁**：纯实验（代码已撤），零 net 源码，feature-off default clippy 绿，css-text A/B 数据存档于此。
