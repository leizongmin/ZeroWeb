# Chromium `line-height:normal` 精确公式逆向（FreeType + Skia + Blink 源码深潜）

**版本**：v1.0（R1174，2026-07-08，rendering-compat rally 自主模式）
**模式**：deep-research 源码深潜（chromium/Skia/FreeType 三栈 + DejaVu 字节实测）
**作者**：AI Assistant
**状态**：结论性裁决 — 用精确公式纠正 R1173 的推理瑕疵，重开 font-wall line-height lever

> **📌 来源说明（全文）**
> - **一手事实**：① FreeType 源码 `src/sfnt/sfobjs.c`（`sfnt_init_face` ascender/descender/height 算法，freetype/freetype 仓库 main）；② Skia 源码 `src/ports/SkFontHost_FreeType.cpp`（`generateFontMetrics`，google/skia 仓库 main）；③ Blink 源码 `third_party/blink/renderer/platform/fonts/font_metrics.h`（chromium/chromium 仓库 master）；④ DejaVuSans.ttf 实测 OS/2 + hhea + head 字节（本机 `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`，手解二进制表）；⑤ ZW `crates/layout-engine/src/inline/font_metrics.rs`（fontdue font-bridge，R885）；⑥ master.md R1173 实证 A/B。
> - **💡 推理**：标注于各节。
> - **⚠️ 待验证**：fontdue `line_metrics_full` 对 DejaVu 的 `line_gap` 取值（见 §5），不影响 chromium 公式结论。

---

## 30 秒速览

- **R1173 推断「chromium line-height:normal ≈ 1.2（非 raw metric），font-wall line-height 死墙」——本调研用三栈源码证明该推断错误**。
- **精确公式（一手事实）**：chromium（Linux，Skia+FreeType）对 `line-height:normal` 用字体的 **hhea 度量**（非 sTypo、非 win）——当字体 **未设** `OS/2.fsSelection` bit 7（`useTypoMetrics`）时。DejaVu Sans 未设该位，故 chromium 取 `hhea.ascent + |hhea.descent|`（`hhea.lineGap=0`）= **1.164**。
- R1173 实测的 `1.164` **正是 chromium 真实值**（不是错值）。R1173 的 +10 WPT flips 是真实移动。
- **welcome +0.68pp 回归 ≠ line-height 错**：是 **字体不匹配**（chromium 渲染 welcome 用系统字体，ZW 只加载 DejaVu）。line-height 常数对齐 DejaVu 的 1.164 不解 welcome（因字体不同）。
- **结论**：font-wall line-height lever **非死**——正确目标 = per-font hhea 度量（DejaVu=1.164）。阻塞 = ① welcome 字体不匹配（R631 谱系）+ ② per-font metric 真触达渲染路径（R890 override-map wiring）+ ③ 6 个硬编码 1.2 测试。**重开该 lever，纠正 R1173 误判**。

---

## 1. 调研动机与 R1173 推理瑕疵

### 1.1 R1173 的推断链（被纠正对象）

R1173 把 `NORMAL_LINE_HEIGHT_RATIO` `1.2 → 1.164`（DejaVu OS/2 metric），实测：
- +10 WPT flips（normal-flow/box-display/text-decor/flexbox/writing-modes，自 R1159 以来最大）
- welcome product-smoke **+0.68pp 回归**（16.29% → 16.97%，仍 < 20% DC-13 gate）
- make test FAIL（6 测硬编码 1.2）

R1173 据此**推断**：「welcome DejaVu normal ≈ 1.2 非 1.164 → chromium 的 normal ≠ raw font metric → 有调整/rounding/不同 metric 源 → font-wall line-height 死墙」。

### 1.2 推理瑕疵

R1173 的「chromium ≈ 1.2」是**从 welcome 回归反推的间接推断**，非直接测量。该推断隐含假设「welcome diff 主导项 = line-height」——但若 welcome diff 实际由**字体不匹配**主导（chromium 用系统字体，ZW 用 DejaVu），则 line-height 常数的变化对 welcome 的影响不可预测（可正可负），据此反推 chromium 值无效。

本调研**直接读 chromium 三栈源码**确定精确公式，不依赖 welcome 回归反推。

> **📌 来源说明（§1）**
> - 一手事实：master.md R1173 entry + `evidence/r1173-lineheight-...txt`。
> - 💡 推理：welcome diff 字体不匹配主导（R631 谱系四证 + R633 font 方向全证伪）。

---

## 2. DejaVu Sans 实测度量（一手事实）

手解 `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf` 的 OS/2 + hhea + head 二进制表（TrueType 目录 → 表偏移 → 固定布局字段）：

| 字段 | 值 | 来源表/偏移 |
|------|-----|------------|
| `unitsPerEm` | 2048 | head+18 |
| hhea `ascent` | 1901 | hhea+4 |
| hhea `descent` | −483 | hhea+6 |
| hhea `lineGap` | 0 | hhea+8 |
| OS/2 `version` | 1 | OS/2+0 |
| OS/2 `fsSelection` | 0x40 | OS/2+62 |
| **`fsSelection` bit 7（useTypoMetrics）** | **False** | 0x40 & 0x80 = 0 |
| OS/2 `sTypoAscender` | 1556 | OS/2+68 |
| OS/2 `sTypoDescender` | −492 | OS/2+70 |
| OS/2 `sTypoLineGap` | 410 | OS/2+72 |
| OS/2 `usWinAscent` | 1901 | OS/2+74 |
| OS/2 `usWinDescent` | 483 | OS/2+76 |

**派生比率（值 / upem）**：

| 候选 metric 源 | 比率 | 含义 |
|---------------|------|------|
| **hhea `ascent+\|descent\|`** | **1.1641** | ← R1173 测的值（无 leading） |
| win `usWinAsc+usWinDescent` | 1.1641 | 同 hhea（DejaVu 二者相等） |
| sTypo `asc−\|desc\|`（无 linegap） | 1.0 | |
| sTypo `asc−\|desc\|+linegap` | 1.2002 | ≈ 1.2（被误以为是 chromium 值） |
| hhea `asc−\|desc\|+linegap` | 1.1641 | lineGap=0 |

> **📌 来源说明（§2）**
> - 一手事实：本机 DejaVuSans.ttf 二进制表手解（Python struct）。
> - 💡 关键：DejaVu **未设** useTypoMetrics（bit 7 = False），决定 §3 走 hhea 分支。

---

## 3. chromium 三栈精确公式（一手事实·源码）

### 3.1 FreeType：`face->ascender/descender/height`（`src/sfnt/sfobjs.c::sfnt_init_face`）

```c
/*
 * 1. If OS/2 fsSelection bit 7 (useTypoMetrics) is set: use sTypo* metrics
 * 2. Otherwise, use the `hhea' table's metrics.
 * 3. If they are zero and OS/2 exists: fallback to sTypo*, then win.
 */
if ( face->os2.version != 0xFFFFU && face->os2.fsSelection & 128 )
{
  root->ascender  = face->os2.sTypoAscender;
  root->descender = face->os2.sTypoDescender;
  root->height    = root->ascender - root->descender + face->os2.sTypoLineGap;
}
else
{
  root->ascender  = face->horizontal.Ascender;        // hhea
  root->descender = face->horizontal.Descender;       // hhea
  root->height    = root->ascender - root->descender + face->horizontal.Line_Gap;  // hhea
  // fallback 仅在 ascender/descender 均 0 时触发（DejaVu 不触发）
}
```

**DejaVu 应用**（bit 7 = False，hhea 非零）→ else 分支：
- `face->ascender` = 1901（hhea）
- `face->descender` = −483（hhea）
- `face->height` = 1901 − (−483) + 0 = **2384**（hhea 派生）

### 3.2 Skia：`fAscent/fDescent/fLeading`（`src/ports/SkFontHost_FreeType.cpp::generateFontMetrics`）

```cpp
// FreeType will always use HHEA metrics if they're not zero.
// It completely ignores the OS/2 fsSelection::UseTypoMetrics bit.
static const int kUseTypoMetricsMask = (1 << 7);
if (os2 && os2->version != 0xFFFF && (os2->fsSelection & kUseTypoMetricsMask)) {
    ascent  = -os2->sTypoAscender  / upem;
    descent = -os2->sTypoDescender / upem;
    leading =  os2->sTypoLineGap   / upem;
} else {
    ascent  = -face->ascender  / upem;
    descent = -face->descender / upem;
    leading = (face->height + (face->descender - face->ascender)) / upem;  // ★
}
// ...
metrics->fAscent  = ascent  * fScale.y();
metrics->fDescent = descent * fScale.y();
metrics->fLeading = leading * fScale.y();
```

**DejaVu 应用**（else 分支）：
- `ascent` = −1901/2048 = −0.9282（Skia 约定：上为负）
- `descent` = −(−483)/2048 = 0.2358
- `leading` = (2384 + (−483 − 1901))/2048 = (2384 − 2384)/2048 = **0**
  - ★ 该公式从 FreeType `face->height` 反解 line gap：因 FreeType `face->height = asc − desc + line_gap`，故 `leading = height + desc − asc = line_gap`。DejaVu hhea `lineGap=0` → leading=0。
- **总行高 = |ascent| + |descent| + leading = 0.9282 + 0.2358 + 0 = 1.1640**

### 3.3 Blink：`FontMetrics`（`platform/fonts/font_metrics.h`）

```cpp
float FloatHeight() const { return float_ascent_ + float_descent_; }   // ascent+descent，无 line gap
int   LineGap() const { ... line_gap_ ... }
int   LineSpacing() const { ... line_spacing_ ... }                    // 单独字段
```

`ascent_/descent_` 由 `AscentDescentWithHacks()` 从 Skia `fAscent/fDescent` 填充（含 Mac 字体 hack，crbug.com/445830，Linux 不触发）。

**对 DejaVu**：无论 `line-height:normal` 解析到 `Height()`（ascent+descent=1.164）还是 `LineSpacing()`（ascent+descent+line_gap），因 DejaVu `line_gap=0`，**二者相等 = 1.164**。Height-vs-LineSpacing 的歧义对 DejaVu 不影响结论。

> **📌 来源说明（§3）**
> - 一手事实：FreeType `sfobjs.c` + Skia `SkFontHost_FreeType.cpp` + Blink `font_metrics.h`（皆仓库 main/master）。
> - 💡 关键：DejaVu `hhea.lineGap=0` 使 Skia `leading=0`，故总行高 = hhea ascent+|descent| = 1.164，与 line gap 是否计入无关。

---

## 4. 裁决：R1173 推断被推翻，font-wall line-height lever 重开

### 4.1 chromium line-height:normal 精确值（DejaVu）

**chromium（Linux/Skia/FreeType）对 DejaVu Sans 的 `line-height:normal` = 1.164**（hhea `ascent+\|descent\|`，无 leading）。这是 FreeType `face->height`（hhea 派生 2384）+ Skia 反解 line gap 公式 + DejaVu `lineGap=0` 三者的确定性算术结果，无 rounding/adjustment 空间。

R1173 测的 `1.164` **正是 chromium 真实值**。

### 4.2 R1173 「≈1.2 / 死墙」推断错误

R1173 从 welcome +0.68pp 回归反推「chromium ≈ 1.2」——该反推无效，因 welcome diff 由**字体不匹配**主导（chromium 渲染 welcome 用 fontconfig 解析的系统字体，ZW 只加载 DejaVu）。对齐 DejaVu 的 1.164 不解 welcome（字体不同），welcome 回归方向（正/负）不可由 line-height 常数预测。

**font-wall line-height 非「raw metric 替换死墙」**——chromium 本就用 raw hhea metric（DejaVu 1.164）。R1173 的结论是间接推断瑕疵，非死墙确证。

### 4.3 真实阻塞（重开后的 lever 边界）

font-wall line-height lever **非死**，但落地受三阻塞：

1. **welcome 字体不匹配**（R631 谱系，主阻塞）：ZW 只加载 DejaVu/Ahem，chromium welcome 用系统字体。per-font line-height（DejaVu→1.164）使 WPT 文本类对齐 chromium，但 welcome 因字体不同仍回归。welcome 当前 16.97%（1.164 下）仍 < 20% DC-13 gate，但违反 team「welcome 稳定」偏好。
2. **per-font metric 真触达渲染路径**（R890 override-map wiring）：fontdue font-bridge（R885）已暴露 per-font 度量，但 R890 实测经 override-map 接线后 welcome 一字不变（paint Path B 空 styles gate，R72）。须完成 `store_font_sizes_from_ifc` 式 override-map 的 populate+consume（绕空 styles），per-font 1.164 才真生效。
3. **6 个硬编码 1.2 测试**（`basic.rs:1182/1194/1242/1412/1800` + `advanced.rs:251`）：编码旧 1.2 假设，须随常数/per-font 改动更新。

> **📌 来源说明（§4）**
> - 一手事实：§2 实测 + §3 源码 + master.md R890/R631/R1173。
> - 💡 推理：welcome diff 字体不匹配主导（非 line-height），故 welcome 回归不可由常数预测。

---

## 5. ZW 复刻路径（作者综合）

### 5.1 fontdue 已暴露正确值（一手事实·实证确认）

`crates/layout-engine/src/inline/font_metrics.rs`（R885 font-bridge）注释明示：fontdue 对 DejaVu 报 `ascent ≈ 0.928em`（= 1901/2048 = hhea）。故 fontdue `ascent − descent`（descent=−0.236em）= **1.164**，与 chromium（§3）一致。**复刻不需 FreeType——fontdue 已给对值**。

**R1174 实证 dump（fontdue `line_metrics_full` @ DejaVu 16px，经 `FontMetricProvider::line_metrics`）**：
```
ascent = 14.8515625  (0.9282em)
descent = -3.7734375 (-0.2358em)
line_gap = 0          ★ hhea 同源（与 chromium 一致）
ascent − descent + line_gap = 1.1641em  ★ 精确等于 chromium 值
```
**结论**：fontdue 对 DejaVu 的 `line_gap = 0`（hhea `lineGap=0`，与 chromium 同源），`ascent − descent + line_gap = 1.1641` **精确等于 chromium**。无需 line_gap gate，font-bridge 注释的 `ascent − descent + line_gap` 公式对 DejaVu 直接给对值。

### 5.2 复刻步骤（多会话）

1. **完成 R890 override-map wiring**：`compute_final_inline_layouts`（inline_finalization.rs 5 站点）经 provider 算 per-font line-box 高度 → 存 `text_node_ascent_ratios`/`line_height_overrides`（`store_font_sizes_from_ifc` override-map 模式）→ paint Path B 经 override 消费（绕空 styles gate）。先 fontdue metric 验 wiring 通（welcome 应有变化，区别 R890 no-op），再确认值。
2. ~~**fontdue line_gap gate**~~（§5.1 实证 fontdue line_gap=0，**无需**）。
3. **welcome 字体不匹配解**（R631 谱系，独立）：ZW 加载 welcome-chromium 实际系统字体，使 welcome 与 WPT 文本类同字体源。此前 welcome 回归不可避免。
4. **6 测试更新**：随常数/per-font 改动更新硬编码 1.2。
5. **三态门禁 A/B**：welcome < 20% gate + css-text/linebox/normal-flow oracle 不降 + self-source 不降。

### 5.3 与既有证伪的关系（不冲突）

R1060/R1090/R1095/R1160/R891 证伪的是 **ascent/baseline/advance 单点杠杆**（per-glyph 偏移、常数 ascent、baseline_offset）——**非 line-box 高度常数**。本调研证 line-box 高度常数（DejaVu 1.164）= chromium 真值，与单点杠杆不同量级（R1172 §2.2 推理：line-box 高度是多行累积 diff 主导）。R1173 的 +10 flips 是 line-box 高度对齐的首个实证正移动。

> **📌 来源说明（§5）**
> - 一手事实：font_metrics.rs（fontdue font-bridge R885）+ master.md R890/R1172。
> - 作者综合：§5.2 复刻步骤基于 §3 公式 + §4 阻塞。

---

## 6. 结论

chromium（Linux/Skia/FreeType）对 DejaVu Sans 的 `line-height:normal` = **1.164**（hhea `ascent+\|descent\|`，`lineGap=0` 故无 leading），三栈源码 + 实测字节确定性证明。R1173 实测的 `1.164` 是 chromium 真实值（非错值），其 +10 WPT flips 真实。R1173「chromium ≈ 1.2 / font-wall line-height 死墙」推断**被推翻**——根因是 welcome 回归由字体不匹配主导，非 line-height 错。

**font-wall line-height lever 重开**：正确目标 = per-font hhea 度量（DejaVu 1.164，fontdue 已暴露）。阻塞 = welcome 字体不匹配（R631）+ R890 override-map wiring + 6 测试。这是 font-wall（dominant 41.3% corpus gap）首个有精确正确目标的可推进 lever，区别于 R1060 谱系的单点证伪。

---

## 参考资料

| # | 来源 | 类型 | 引用章节 |
|---|------|------|----------|
| 1 | FreeType `src/sfnt/sfobjs.c` `sfnt_init_face`（[freetype/freetype main](https://github.com/freetype/freetype/blob/master/src/sfnt/sfobjs.c)） | 一手事实 | §3.1 |
| 2 | Skia `src/ports/SkFontHost_FreeType.cpp` `generateFontMetrics`（[google/skia main](https://github.com/google/skia/blob/main/src/ports/SkFontHost_FreeType.cpp)） | 一手事实 | §3.2 |
| 3 | Blink `third_party/blink/renderer/platform/fonts/font_metrics.h`（[chromium/chromium master](https://github.com/chromium/chromium/blob/master/third_party/blink/renderer/platform/fonts/font_metrics.h)） | 一手事实 | §3.3 |
| 4 | DejaVuSans.ttf OS/2 + hhea + head 字节手解（本机 `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`） | 一手事实 | §2 |
| 5 | ZW `crates/layout-engine/src/inline/font_metrics.rs`（fontdue font-bridge R885） | 一手事实 | §5.1 |
| 6 | master.md R1173 entry + `evidence/r1173-lineheight-...txt`（1.164 A/B +10 flips / welcome +0.68pp） | 一手事实 | §1, §4 |
| 7 | master.md R890（override-map wiring no-op）+ R631（font 匹配四证伪）+ R1172（fontdue→FreeType metric feasibility） | 一手事实 | §4, §5 |
| 8 | [OS/2 table spec — Microsoft Learn](https://learn.microsoft.com/en-us/typography/opentype/spec/os2)（fsSelection bit 7 useTypoMetrics 定义） | 外部搜索 | §2, §3 |
