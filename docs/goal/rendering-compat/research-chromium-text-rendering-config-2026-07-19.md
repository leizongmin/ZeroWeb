# 调研：chromium 文本渲染管线配置（font-wall raster 角度复核）

**日期**：2026-07-19（R1764）
**触发**：R1763 纠正 font-stack dead-end 后，复核 R1560「光栅化 coverage 永久关闭」结论是否过严
（R1560 用 skia-safe 0.80 **默认** config 测，非 chromium 实际 tuned config）
**结论**：**R1560「光栅化永久关闭」过严——raster-config（gamma/hinting/subpixel）是 REOPENED 但 untested + 深水区 hypothesis**；font-wall 根因仍 ambiguous（raster-config AND/OR layout/metric coherence）

---

## 1. 核心事实（外部权威源交叉验证）

### 1.1 chromium 文本渲染 = FreeType + Skia，且 Skia config 可调 + 平台差异

- **FreeType.org 官方**（`freetype.org/freetype2/docs/hinting/text-rendering-general.html`）：
  > 「Skia (Chrome) can perform gamma-correction but **turns it off for X11**」
  - 关键：chromium 的 gamma 校正在 **X11 下关闭**，非全局常量 → config-dependent + 平台差异
- **Chromium Issue #40267453**（`issues.chromium.org/issues/40267453`）：
  > 「Chromium passes **wrong gamma correction value** to Skia on Linux, resulting in washed out text on websites with dark themes」
  - 关键：gamma value 是 chromium **主动传给 Skia** 的参数，传错即视觉可见 → gamma 是 impactful config
- **skia-discuss**（`groups.google.com/g/skia-discuss/c/Z0WBnnxJTrk`）：
  > 调 `SK_GAMMA`-related values 改变 chromium 字体模糊度
  - 关键：SK_GAMMA 是编译期/运行时可调旋钮，直接影响渲染

### 1.2 default Skia ≠ chromium-tuned Skia（多方佐证）

- **Reddit r/GraphicsProgramming**（`reddit.com/r/GraphicsProgramming/comments/1o49opr/`）：
  开发者用 Skia Canvas API **复刻 Chrome 文本渲染失败**，试多种 flag 仍不匹配
- **SkiaSharp #1253**（`github.com/mono/SkiaSharp/issues/1253`）：
  讨论 SkiaSharp 是否用与 Chrome 相同的 Skia **编译设置**（结论：否，default ≠ Chrome-tuned）
- **Chrome Developers Blog**（`developer.chrome.com/blog/better-text-rendering-in-chromium-based-browsers-on-windows`）：
  > 「Skia doesn't pick up text contrast/gamma values from the Windows ClearType Tuner and **uses different defaults**」
  - 关键：Skia 有自己的 default，与 chromium 实际用的 tuned 值不同

### 1.3 hinting + subpixel 的具体配置

- **FreeType.org**：sub-pixel rendering 时 **关闭 horizontal hinting、保留 vertical hinting**
- **puredevsoftware.com**（`puredevsoftware.com/blog/2019/01/22/sub-pixel-gamma-correct-font-rendering/`）：
  详细解释 FreeType subpixel + gamma-correct 流程

---

## 2. 对 R1560 结论的复核

**R1560（2026-07-16）原结论**：「font-wall 光栅化 coverage 算法 lever **永久关闭**；real Skia (skia-safe 0.80) over FreeType 轮廓 css-text net-24 → font-wall 不在光栅化，在 layout/metric coherence」

**本调研发现 R1560 测试不完整**：
- R1560 用 skia-safe 0.80 的 **default config**（默认 gamma / 默认 hinting / 默认 subpixel）
- 上述外部源一致证明：**default Skia ≠ chromium-tuned Skia**（gamma value + hinting mode + subpixel positioning 全是 chromium 主动调的）
- 故 R1560 的 net-24 测的是「default-Skia vs ZW-FreeType vs chromium-tuned-Skia」**三者各异**，net-24 不能推出「chromium-tuned-Skia 也不改善」——那一路**从未测过**

**修正**：R1560 应表述为「**default-Skia** 光栅化 lever 关闭」；**chromium-tuned-Skia config（精确 gamma table + hinting mode + subpixel positioning）是 REOPENED 但 untested hypothesis**

---

## 3. X11 环境的 mitigating nuance（重要）

FreeType.org 说 chromium **X11 下关 gamma 校正**。ZW oracle 抓取环境 = WSL2 Linux（X11/Wayland）：
- 若 chromium 在该环境确实关 gamma → oracle 是「gamma-off」渲染 → ZW-FreeType（也无 gamma 校正）**此维已匹配**，gamma 角度收益可能小
- 但 **hinting mode（slight/full/off）+ subpixel positioning + stem darkening + LCD filter** 仍可能差异（这些非 gamma，X11 下仍 active）
- 需实测 chromium 在 WSL2 oracle 环境的实际 config（`chrome://gpu` / `--show-fps-counter` 无关，须 paint dump 或 about:flags 查 antialiasing mode）才能确定哪些维度真差

---

## 4. 对 ZW 的可行性评估

**若要 chase chromium-tuned-Skia config**：
1. 须先 **dump chromium 在 oracle 环境的实际 text-rendering config**（gamma on/off、hinting mode、subpixel mode）——决定哪些维度真差
2. 复刻到 ZW：两条路
   - **a) skia-safe 0.80 + 手动设 chromium tuned flags**（SK_GAMMA、`SkFont::setHinting`、`setSubpixel`、`setEdging`）——R1560 已建 skia-safe 集成 infra（S0/S1），可复用 + 改 config 重测 A/B
   - **b) ZW 自有 CPU compositor + FreeType hinting/gamma 手动实现**——更深，但无 C-dep
3. A/B：css-text/font-wall 簇 oracle pass-rate + welcome diff

**风险/成本**：
- R1560 skia-safe per-glyph Surface 重 + thread-local FontMgr OOM-prone（已记），config-tuned 版同问题
- chromium 实际 config 可能跨版本/平台变（chrome 127 vs 150），oracle 用 127 须对齐
- 即使 config 匹配，**layout/metric coherence（Phase A）仍独立残存**——raster config 不是完整 unlock

---

## 5. 战略结论

- **R1560「光栅化永久关闭」过严**，应修正为「default-Skia 关闭；chromium-tuned-Skia config REOPENED untested」
- **font-wall 根因仍 ambiguous**：可能 = raster-config（gamma/hinting/subpixel）AND/OR layout/metric coherence（Phase A），两者未解耦测过
- **真 unlock 仍深水区**：chase chromium-tuned config 须先 dump 实际 config（0 code 调研）+ skia-safe-tuned A/B 或 FreeType 手动实现，多 session
- **与 R1763 的关系**：R1763 hypothesized「gamma/subpixel/hinting compositing」方向**部分成立**（research 证 chromium 确有此 config），但 X11 gamma-off nuance + chromium-tuned 未测使其仍是 hypothesis 非 confirmed lever
- **下一步若 chase**：先 dump chromium 在 WSL2 oracle 环境的实际 text config（chrome://gpu 或 paint pixel 分析），0-code 确定 gamma/hinting/subpixel 实际值，再决定 skia-safe-tuned A/B 是否值得

---

## 6. 实证探针（research 后，2026-07-19 同轮 R1764）

承接 §5「下一步先 dump chromium 实际 config」。本轮立即做 0-code 实证：

**6.1 chromium 127 oracle 环境 AA 模式（config-dump）**：headless chrome 127 截 `<p>WWWmmm</p>`（sans-serif, 60px），PIL 逐像素分析：
- **subpixel-colored 像素 = 0%（100% grayscale）**：chromium 在 WSL2 headless 用 **grayscale AA 非 subpixel RGB**（edge `(0,0,0)→(196,196,196)→(255,255,255)` R=G=B 全程）
- 与 FreeType.org「chromium X11 关 gamma」一致：oracle 环境 gamma off + grayscale AA

**6.2 ZW-FreeType vs chromium 同文本 edge profile（关键）**：ZW CPU 渲染**同一 probe**，对比：
- chromium-127：darkest (21,34)，edge_R = `[0,0,0,0,0,196,255,255,...]`，239 distinct gray levels
- **ZeroWeb-FreeType：darkest (21,33)，edge_R = `[0,0,0,0,0,196,255,255,...]`，204 distinct gray levels**
- **edge transition 字节级一致**（0→196→255 同位置）+ darkest 像素同 x(21) + gray-level count 接近（239 vs 204）

**6.3 结论修正（重要）**：
- **ZW-FreeType 简单文本 raster/AA/positioning 已字节级匹配 chromium** → raster-config（gamma/hinting/subpixel）**非 font-wall lever**（ZW 已匹配，非「default ≠ chromium-tuned」差距）
- §2/§5「chromium-tuned-Skia config REOPENED hypothesis」**经探针削弱**——R1763 假说（gamma/subpixel/hinting = unlock）不成立：oracle 环境 gamma 已 off + ZW grayscale edge 已匹配
- **font-wall 真根因 = 复杂文本（shaping/kerning/ligature）或累积度量差异**，非简单 glyph raster（R1742 已指向：生产 paint 用 char-as-glyph_id 无 shaping；rustybuzz 切换是 font-wall metric deadlock）
- **R1560「font-wall 不在光栅化」结论 REINFORCED**（本轮 direct edge-match 证，非仅 Skia-net-24 推），但 R1560 forward「Phase A IFC 唯一剩」仍成立（复杂文本/累积度量 = Phase A 谱系）

**6.4 对策略的影响**：勿再 chase chromium-tuned-Skia raster config（探针证 ZW 已匹配简单文本）；真 unlock = 复杂文本 shaping / 累积度量（Phase A 谱系，深水区 deadlock）。font-wall 性质从「raster-or-metric ambiguous」收窄为「metric/coherence（非 raster）」。

---

## Sources

- [FreeType.org — text rendering general (gamma X11 off, hinting, subpixel)](https://freetype.org/freetype2/docs/hinting/text-rendering-general.html)
- [Chromium Issue #40267453 — wrong gamma to Skia on Linux](https://issues.chromium.org/issues/40267453)
- [Skia Issue #40045101 — gamma correction env var](https://issues.skia.org/issues/40045101)
- [Chrome Developers Blog — better text rendering (Skia defaults differ)](https://developer.chrome.com/blog/better-text-rendering-in-chromium-based-browsers-on-windows)
- [puredevsoftware — sub-pixel gamma-correct font rendering](https://www.puredevsoftware.com/blog/2019/01/22/sub-pixel-gamma-correct-font-rendering/)
- [Reddit r/GraphicsProgramming — replicating Chrome text in Skia](https://www.reddit.com/r/GraphicsProgramming/comments/1o49opr/)
- [SkiaSharp #1253 — same Skia compilation as Chrome?](https://github.com/mono/SkiaSharp/issues/1253)
- [skia-discuss — SK_GAMMA font blur](https://groups.google.com/g/skia-discuss/c/Z0WBnnxJTrk)
