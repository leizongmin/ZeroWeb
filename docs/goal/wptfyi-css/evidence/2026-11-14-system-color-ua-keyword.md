# g32：system-color-consistency 27F 全清——UA 表单系统色关键词化（S1' 值自洽端到端）

日期：2026-11-14。前置 triage：
`.acceptance/wptfyi-css/triage-system-color-consistency.md`（11-14 探针实证重写版）。

## 页与桶

`css/css-color/system-color-consistency.html`：`documentElement.style.colorScheme
= 'light'|'dark'` 双轮，每控件读自家 gCS（border-color/background-color/color），
`test_computed_value('color', '<SystemColor>', 控件读数)` 自洽比对。原账 35P/27F
（g3/g4/fd2 三轮恒定）分三桶：A ButtonBorder 8F / B ButtonFace 8F / C
ButtonText 5F + FieldText dark 6F。

## 原假设证伪（探针定谳，见 triage §关键机制）

- 原 S1「关键词身份回声」与 system-color-compute 页「doesn't compute to itself」
  钉面冲突，弃。本页是真值自洽比对，须**计算值语义**成立（css-color-4
  §resolving-value：计算时间解析为绝对色，scheme-aware）。
- 控件 border-color 读数原为空串（bridge 无简写臂）→ computed-testcommon
  `if (!computed) computed = specified` 回落字面——「回声通路存在」是空串回落假象。
- div inline 系统色 dark 轮不翻转是 **Rust dark 面板缺键**（buttonface 等落到
  named 静态值），非 scheme 通路缺失。

## 修法三面（工作树 S1'，三文件）

1. **engine `js_dom_bridge/computed_style.rs`**：`border-color` 简写聚合臂——四边
   独立 currentcolor 解析（与 longhand 臂同通道）+ CSSOM 最短形去重（1/2/3/4 值）。
   此前简写读数缺位返 ''。
2. **css-parser `values/color.rs`**：`parse_color_with_scheme` dark 面板扩键——
   buttonface/threedface/buttonhighlight→(43,43,43)、buttontext/buttonshadow→白、
   buttonborder/threeddarkshadow/threedhighlight/threedlightshadow→(117,117,117)、
   field→(32,33,36)、fieldtext→白（与 engine shim `_zwCCSystemDark` 既有近似对齐；
   buttonborder 无既有钉面取暗底描边中灰，页两侧同走本解析自洽不依赖具体值）。
   light 面板 = named_colors 静态值不变。
3. **style-system `lib.rs`** R1396 UA 默认关键词化：button → color ButtonText /
   background ButtonFace / border-color ButtonBorder（longhand 后置覆写 border
   简写色槽，几何 2px solid 不变）；input 按钮族（type=button|submit|reset|color，
   Chromium html.css 同组判定，大小写不敏感）→ bg ButtonFace / border-color
   ButtonBorder；文本族 input/select/textarea → bg Field。文本族不加 color 声明
   （继承 html 默认 light 黑/dark 白已与 FieldText 两轮同值，显式声明反覆写作者
   继承面）。

## 零漂移边界（triage §零漂移边界全守）

mark/marktext/linktext/canvastext 各行不触（UA mark bg 仍字面 #ffff00）；shim
`_zwCCSystem` 空串策略不动（div inline 读数走 Rust cascade 非 shim part11 管线）；
deprecated-sameas-001..023 相对匹配走 light 静态不受 dark 扩键影响。

## 验证账

| 项 | 结果 |
|---|---|
| solo 探针（release，system-color-consistency 单页） | 35P/27F → **62P/0F 全页清零**（results-g32-syscolor/system-color-consistency-solo-62p.json） |
| 探针读数钉面 | 控件四钮 bg 全 240 / border 128、dark 43/117；文本族 bg 白→Field、dark (32,33,36)；div inline ButtonBorder dark 117==控件 border dark、ButtonFace dark 43==控件 bg dark |
| make test 全量（unit zw-g32-mktest） | **3932P/7F**，7F 全为在册既有组（img/svg settle、fetch matrix、iframe SW、readable stream、websocket ×2），零新增 |
| rustfmt | clean（三 crate --check EXIT=0） |
| css-color 全量 | **10,972 → 10,999 净 +27 逐页零回归**（唯一增益 system-color-consistency 35→62；基线含 2 页 0P 探针页 zz-probe-forensic/zz-probe-none 已按卫生规则清出，不计账） |
| css-values / css-typed-om / cssom / 四小漂移锚 | **values 3,935 精确持平、cssom 1,968 逐页字节级一致、四小（transitions 200/animations 238/transforms 229/easing 51）Pass 级零漂移**；css-typed-om 首轮 9,870（−2 漂移）→ 豁免刀后 9,872（见下节） |
| clippy | 零告警（css-parser/style-system/engine --all-targets） |
| product-smoke | **EXIT=0**：welcome.html 375/320 双宽 struct PASS（0 issues + sibling-overlap + count[.card=4]） |
| product-smoke-legacy | **42 fixture struct-check failures: 0**（37-form-controls PASS；trend-only pixel 为 font-wall 基线数据） |
| make reftest | **725 项 0 失败**（真通过 559 + 近似通过 54，Result=success，unit zw-g32-reftest PATH 注入重跑） |

## 补刀：g27/g28 写面门替换函数豁免（typed-om −2 回归根因）

g32b 六套件对账暴露 css-typed-om 对 v14 探针基线 −2（三页漂移，全为 g27/g28/g29
批晚于基线所致，非本批修法面）：

- `counter-increment` var() 拒收 −1：g27 写面门缺替换函数豁免（g24 洞 A 同型）。
- `text-size-adjust` var() + 负 % −2：g28 门同理；负 % 为 **WPT 两页张力**——
  css-size-adjust invalid 页断言写面拒收（parsing-testcommon round-trip 空），
  typed-om percent 案为 `CSSUnitValue(-3.14,'percent')` 走 grammar-only 校验
  （shim `_zwOmValidateSet` unit family 臂，Chrome 同款两层：typed 值集不经
  prose 有效性门），我们两路共用宿主 `validate_decl_value` 单门。
- `font-stretch` 0% +1：g29 修复红利（Fail→Pass）。

修法（`property/apply.rs`）：counter-increment 与 text-size-adjust 两门头部补
`value_has_substitution_function` 豁免（var()/env()/attr() 放行 store 原文，
css-values-3 §3.3，有效性 computed 期重裁）；负 % 字面形保持写面拒（css-size-adjust
invalid 页 round-trip 断言依赖），typed-value 两层校验差异挂账 1F。单测 +6 钉
（两门 var()/attr() 混合形）。style-system lib 2,533 全绿。

**g32c 复验**（results-g32c-exempt）：typed-om 9,870→**9,872**（var 两腿回位，
对 v14 基线恰归零：font-stretch +1 抵 −3.14% 残挂 −1）；css-lists counter-increment
invalid 13P/0F、computed 8P/2F（g27 收口态逐页精确一致零漂移）；css-size-adjust
invalid 4P/0F、valid 7P/0F、computed 6F+interpolation 1F（g28 收口态精确一致——
写面拒收保持）。

## 风险与审计点

- UA 关键词化可见像素漂移（有意、spec 对齐）：button bg 239→240、text 8→0、
  border 118→128；input 按钮族 bg 白→240（light 实变）；dark 轮表单控件全面板化。
  reftest/表单视觉冒烟按门禁跑（product-smoke-legacy 42 fixture、product-smoke
  DC-13 welcome 阈值 20%）。
- dark 面板扩键影响所有 `parse_color_with_scheme(dark=true)` 调用方：css-color
  全量零回归核对覆盖。
- 语料数据目录探针卫生：zz-probe-syscolor.html 与 parsing/zz-probe-forensic/
  zz-probe-none 探针页已清出（canary 留痕）；zz-caprobe 两页基线亦有，diff
  apples-to-apples。
