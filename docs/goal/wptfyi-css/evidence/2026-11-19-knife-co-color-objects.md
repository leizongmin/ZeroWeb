# 刀 CO：css-typed-om color objects 构造器验证 + CSSColorValue.parse（engine JS shim 域）

日期：2026-11-19 · 域：`crates/engine/src/js_dom_shim/`（part05/part10-typed-om/part11-color，+236/−14 行）· 电池 `knife-co-battery-2309.log`（ZeroWeb-3 clone，23:09–23:34）

## 定谳

css-typed-om stylevalue-subclasses 8 color 页 **143F** 根因 = 色值对象构造器无通道
验证（非法单位/类型静默接受而非抛 SyntaxError/TypeError）、无归一化（plain number
通道不转 percent）、hue `undefined` 不产 `CSSKeywordValue('undefined')`、alpha 缺省
缺失（应 percent(100)）、且 `CSSColorValue.parse` 全缺（cssColorValue 20/20 全红）。

判据逐条钉自页源 + `testhelper.js` `assert_color_channel_approx_equals`（通道语义：
unit 相等 + 数值近似，percent ×100 口径）：

- **通道 kind 六型**（共享验证器 `__zwOmColorChannel`）：`numPct`（RGB r/g/b 与
  各类 alpha——plain number → percent ×100；`CSS.number` → SyntaxError，页
  `new CSSRGB(0,0,0,CSS.number(1))` 钉）、`pctOnly`（HSL s/l、HWB w/b、Lab/LCH/
  OKLab/OKLCH l/c/alpha——拒 number/角度单位）、`numPctKeep`（Lab/OKLab a/b——
  plain number 保 `CSS.number`）、`hueHsl`（HSL h——数 → deg、角单位工厂保值、
  `undefined` → `CSSKeywordValue('undefined')`、px/em → SyntaxError）、
  `hueStrict`（HWB h——**裸数 TypeError**、角单位保值）、`hueNumAng`（LCH/OKLCH
  h——数 → deg、`CSS.number` 拒）。math 值（`.values` 数组）递归核叶单位。
- **构造器侧**：part05 CSSRGB/CSSHSL 具名构造器重写 + part10 `_zwOmColorCtor`
  升 `[field, kind]` 对（CSSHWB/CSSLCH/CSSOKLCH/CSSLab/CSSOKLab 五类）——ctor
  实参与 setter 同走验证器；alpha 缺省 `new CSSUnitValue(100, 'percent')`。
- **`CSSColorValue.parse`**（css-typed-om-1 §csscolorvalue-parse，接 part11-color
  IIFE 内 `_zwCParse` 解析结果）：rgb/rgba → CSSRGB（unitless 通道 `CSS.number`、
  `%` 保值、显式 alpha 数 ×100）；hex → CSSRGB（a<1 → percent ×100，=1 走缺省）；
  named → `_zwCNamed` 三元组 CSSRGB；transparent → CSSRGB(0,0,0,pct 0)；系统色/
  currentcolor → CSSKeywordValue（toString 小写）；hsl/hsla → CSSHSL（h 数/角
  单位、s/l `%` 保值或 unitless 走构造器 ×100——legacy 逗号形 unitless s/l 由
  `_zwCParse` invalid-hsl 门拒，与现代空格形分流）；hwb → CSSHWB（h 预包
  CSS.deg——hueStrict 面）；lab/oklab → L/C/L 百分比同标尺（unitless L →
  percent 原值，css-color-4 §9.1）+ a/b `CSS.number`；lch/oklch → c 同 L 标尺、
  h 裸数（`_zwCParse` 已折 deg）；calc/none 通道与 CSS-wide 关键词 →
  SyntaxError（FIXME(calc-channel) 挂账 math 通道臂）。

## 电池账（A/B 逐页 Pass 集比对，ab-compare.py）

b0 = HEAD 剥 face（md5 `b81a5b70bbdd…`）/ b1 = HEAD + face（md5 `9e91246680f2…`），
M2 双侧异 md5 过；RIDE 空（porcelain 仅 3 face 文件）；五语料 × 双侧 10 腿全 rc∈{0,1}
（Fail 计数合法退出码），leg 段 test-guard per-proc 4/total 8/3000s + CASE_WALL_S=60。

| 页（stylevalue-subclasses/） | b0 | b1 | Δ |
|----|----|----|----|
| cssColorValue | 0/20 | **20/20** | +20 |
| cssHSL | 9/24 | **24/24** | +15 |
| cssHWB | 3/21 | **21/21** | +18 |
| cssLCH | 1/17+1 | **18/18** | +17 |
| cssLab | 1/15 | **15/15** | +14 |
| cssOKLCH | 1/18 | **18/18** | +17 |
| cssOKLab | 1/15 | **15/15** | +14 |
| cssRGB | 23/51 | **51/51** | +28 |
| **合计** | 39 | **182** | **+143/143 满贯** |

哨兵（b0 → b1，逐页 Pass 集比对零漂移）：

- css-properties-values-api：908/1371 → 908/1371（GAINED 0 / LOST 0）
- css-animations：123 页 GAINED 0 / LOST 0
- css-transitions：164/662 基线，GAINED 0 / LOST 0
- css-masking（pin 子集 62 页）：15/77 基线，GAINED 0 / LOST 0

`== TOTAL: gained=143 lost=0 regressed=0 pages_gain=8 pages_loss=0`，GATE-DONE rc=0。

## 门禁与验证

- node 冒烟 32 断言全 PASS（拼装 shim 全 25 part + 桩 host：parse 全形 + 六 kind
  构造器/setter 验证语义；含 legacy hsl unitless s/l 拒、HWB 裸数 TypeError）；
  concat 25 part `node --check` 过（单文件片段结构不可独立 parse，HEAD 同态）。
- 电池 build 段 test-guard per-proc 6/total 14/1800s；transitions 腿与 ZeroWeb-2
  knife-g b1 腿并行窗 ~3min（16 核 load 4.5，非编译争抢场景）——哨兵双零漂移
  实证无扰。

## 余面挂账

- calc/none 通道（`rgb(calc(...), ...)`、`hsl(120 50% none)`）parse 臂 → SyntaxError
  挂 FIXME(calc-channel)；上游页有 calc 通道期望 CSSMathValue 形的子测未在本语料
  pin 面，随实证面扩。
- part11 `color()`/`color-mix()` 形未建模（同拒面）；OKLCH 长注记 hue 通道与
  serialization 面（`toString()` modern 形）属后续 serialization 刀。
