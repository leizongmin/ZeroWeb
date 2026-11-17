# 刀 N 三期：font 简写 specified 重组 + supports 注册门 + computed 防逐出（knifen3 A/B）

日期：2026-11-18 · 流：css（wptfyi-css）· bin：b1 `ba82502a`（02:22 建，含刀 O 注册门 3da1e3bbd）→ b2 `7f6c7bb7`（三回归修复后重建）
b0：`results-knifen2-b1`（二期终账在册产物，bin 8833233e = origin/main 03444181b 同码）

## 1. 四站通道定谳（b1 前诊断）

| 页面 | 缺口 | 通道 |
|---|---|---|
| system-fonts.html 588F | interpolation-testcommon.js:396 `CSS.supports('font', from)` 前置门恒 false | supports 门：known_properties 注册表无 `font` → `decl_supported` 尾门 contains 短路 |
| font-computed.html 309F | `is a supported value for font` 前置门 + computed 面无防逐出/组合 | supports 门 + computed 面（computed_style.rs `font_shorthand_to_css`） |
| font-valid.html 306F | inline IDL specified 面回显 raw 原文（readDeclValue 先命中），页面断 canonical | specified 读面（readProp 旁路重组） |
| font-shorthand-serialization-prevention.html 107F | serialization-prevention 规则全集未实现（防逐出 + 覆写合并） | specified 读面重组 + computed 防逐出 |

## 2. 码面（b1，6 文件）

- `style-system/shorthand/mod.rs`：expand_font 位解析抽出 `parse_font_shorthand_parts`（原 212 行文法循环 verbatim 搬运）+ `FontShorthandParts` 位结构；expand_font 缩为 css-wide 臂 / system 六关键字臂 / 位解析臂。
- `engine/dom_bindings/css_style_declaration.rs`（+184 行）：`font_shorthand_reconstruct_from_style`——声明串解析（!important 剥离、last-wins）、system/css-wide 原样回显、位解析、覆写合并（core 位替换 + font-variant 简写位解释 + reset 长手防逐出门）、variant/stretch 位门、canonical 组合（normal 位省略、非 normal lh ` / ` 形）。
- `engine/js_dom_bridge/callbacks.rs` + `js_dom_shim/part03.js`（两处 readProp 顶部）：`font` 旁路臂 → `__zw_style_shorthand_reconstruct(readRaw(), 'font')`。
- `engine/js_dom_bridge/computed_style.rs`：`font_shorthand_to_css` 头部防逐出门（size-adjust≠none / kerning≠auto / ligatures 非 normal / position·numeric·alternates·east-asian≠normal / feature·variation-settings≠normal → ''）+ caps 非 small-caps 位拒 + stretch % 形防逐出。
- `engine/js_dom_bridge.rs`：`decl_supported` 尾部 `font` 直调臂（注册表刀 O 之外的双保险，`apply_property_value_with_quirks` Some((1,1))）。
- 刀 O（同 bin 复账）：`known_properties` + `font` + initial_value 占位臂；apply_initial_value / inherit_property `font` 委托臂——`font: initial`/`inherit` 从静默 no-op 转 spec 正形。

## 3. 探针三态实证（b1 vs b1 前代）

- supports 门：`CSS.supports('font', …)` false → true（knife O 生效）。
- inline canonical：`medium sans-serif` / `italic xx-large / 0px fantasy` raw 回显 → canonical 重组 Pass。
- 防逐出：size-adjust 非 none 覆写 → got ''；覆写合并：font-family 覆写 → got '20px serif' 形。

## 4. 门禁

- style-system lib：**2563/0**（fresh 相对上轮账 2562/0 有 ±1 计数漂移，重构不加测试，待 evidence 记录；本轮四计数钉 13→16 后 2563/0 复绿）。
- style-system clippy --all-targets：rc=0；fmt clean。
- engine clippy --lib：仅剩挂账（peer es_module 在飞 `too_many_arguments` 8/7）；engine lib **test** 目标因 peer 在飞删 part116_c4_block_boundary_pins.rs 断编译，挂账 peer 流清偿。

## 5. b1 六腿首跑：交付确认 + 三回归定位（02:22 bin）

reconcile TOTAL：gained=991 lost=103 regressed=166 pages_gain=10 pages_loss=2。

**靶面交付**（fonts 净 +636）：font-valid 9→315（**+306**＝定谳满额）、font-computed 6→238（+232）、serialization-prevention 109→212（+103）、system-fonts 0→192（knife O 门开，余 396 需 font 长手插值机制）、serialization-font-stretch +1、variant +1、font-invalid +2、variations/font-shorthand +3。

**三回归（我的面，b2 全修）**：

1. `test_font_family_parsing.html` **−102**（全 `font: … (setter)` 形）：页面 harness `def` 捕获于设默认 `16px onelittlepiggywenttomarket` 之后；同组 font-family setter 测试先行落库的旧 `font-family` 原文被重组面覆写合并吃进 → `def` 恒等于 `16px <旧 family>`，与下一 subtest canonical 同形即假失败。根因：重组无声明序仲裁（attr 串序即声明序，font 前的长手声明应被简写 set 屏蔽——CSSOM 简写 set 复位子属性语义）。修：段序仲裁（`idx <= font_idx` 跳过）。
2. `serialization-prevention` −1（font-stretch normal 阻断）：stretch 位门八关键字表无 `normal` → `font-stretch: normal`（可表达，normal 位省略）误防逐出；emit 处 normal 未省略。修：门加 `normal` + emit 省略。
3. `subproperties-reset` −1（font-feature-settings）：页面 live gCS 断言 `computed.font != ''`；knife O 开 apply 面後 font 简写展开缺 feature-settings/variation-settings/size-adjust 复位槽 → 计算面 ffs 非 normal 触发我方防逐出 → computed.font ''。css-fonts-4 §font-prop："All subproperties of the font property are first reset to their initial values"。修：expand_font 三臂（css-wide/system/bits）加 `font-size-adjust: none`/`font-feature-settings: normal`/`font-variation-settings: normal`；FIXME 注明 synthesis 族 + font-width 复位待 apply 面验证。

**peer 挂账分隔（非我账）**：masking 腿 mask-image-interpolation +101/−60、mask-position-interpolation +50/−2——transition/animation 面，bin 02:22 扫入同 clone 对端在飞半程码（transition_registry.rs `TRANSITIONABLE_PROPERTIES` 65≠73 + animation.rs 幽灵 `apply_property_value`）所致，记对端流账；gaps/multicol/typed-om/shapes 四对照腿 SET-IDENTICAL（零位移）。另 `css/css-masking/animations/zz-knife-a1-probe.html`（02:16 生成、gitignored 遗留探针，页内断言恒 false）混入 b1 masking 页集（63 vs 62），恒 Fail 页对 reconcile 惰性（零 Pass 进出），未污染账面。

## 6. b2 探针实证（bin `7f6c7bb7`）

- stale-family 序（fontFamily 先落 → font 后设）：def='16px onelittlepiggywenttomarket'、fp='16px quite simple'——序仲裁生效。
- `font-stretch: normal` 覆写 → '16px serif'（阻断解除，condensed 形不变）。
- 页面口径（attached 元素 gCS）：font 设后 computed.font='20px serif' 非空——复位槽解防逐出。

## 7. b2 六腿 A/B 终账（02:45-02:53，unit zw-knifen3b-ab）

reconcile TOTAL：**gained=1114 lost=0 regressed=30**（regressed 全在 masking＝对端账，见 §5 分隔）。

**fonts 腿（我账）：净 +844，零 P2F**：

| 页面 | b0 → b2 | 增量 |
|---|---|---|
| parsing/font-valid.html | 9 → 315 | **+306**（定谳满额） |
| parsing/font-computed.html | 6 → 238 | +232 |
| animations/system-fonts.html | 0 → 192 | +192（knife O 门开；余 396 需 font 长手插值机制） |
| font-shorthand-serialization-prevention.html | 109 → 213 | +104（b1 的 font-stretch normal −1 已清） |
| font-shorthand-subproperties-reset.html | 8 → 10 | +2（复位槽反哺：size-adjust、variation-settings 双翻绿） |
| variations/font-shorthand.html | 4 → 7 | +3 |
| font-shorthand-serialization-font-stretch.html | 0 → 2 | +2（b1 +1，stretch normal 修复再 +1） |
| parsing/font-invalid.html | 10 → 12 | +2 |
| parsing/font-shorthand-variant.html | 0 → 1 | +1 |
| test_font_family_parsing.html | 2232 → 2232 | **0**（b1 −102 回归全清） |

四对照腿 gaps/multicol/typed-om/shapes：SET-IDENTICAL（零位移）。masking 腿 mask-image +104/−30、mask-position +166 无回归——对端在飞码在 b1(02:22 bin) 与 b2(02:39 bin) 间仍在演进（regressed 60→30、mask-position +50→+166），独立性旁证＝非本刀面。

**入账口径**：fonts 净 +844 记本刀（三期 + 刀 O 同 bin 复账）；masking 位移记对端流（transition/animation 面在飞），不进本刀账。
