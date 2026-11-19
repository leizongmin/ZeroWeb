# 刀 font-cycle：font 单位循环 style-system 子集（ch 兜底分裂 + 三槽 math 臂，2026-11-19）

## 目标与收窄

residue survey 候选 #3（font-property-unit-cycles.tentative 79F +
unit-cycles 23F）recon 后按机制域收窄：本刀取 font-property-unit-cycles
79F 的 style-system 子集 ~19F；registered 形 60F 让渡 ATP fork regprops
域（同根因同源修复），unit-cycles.html 23F 让渡 registered-property
computation 簇认领候选（survey 批二刷新归组——var() cycle 检测 engine 域）。
收窄记见 NOTES 17:3x 节。

## 79F 四形分解（pristine b1 失败消息逐条）

1. **plain ch 4F**：font-size/font-size-adjust/font-weight/font-stretch
   `calc(1px * tan(atan2(1ch, 1px)))` expected 30 got **15**——ch 无度量
   兜底因子 0.5 vs WPT 基线 1.0。
2. **plain NaN 15F**：font-style（oblique 角度位）/font-feature-settings
   （"tag" 整数位）/font-variation-settings（"tag" 数值位）三槽无 math
   eval 臂，`calc(1 * tan(atan2(1em, 1px)))` 形解析恒败 → computed
   空/原文 → parseFloat NaN。
3. **registered+no-cycle NaN 60F**：registered custom property
   `--custom-length: calc(1px * tan(atan2(1em,1px)))` computed 恒空
   （含 25 个无循环腿）——registered property computation 求值路径缺
   font 相对 calc 求值；与 ATP fork 认领的 registered-property-computation
   32F「computed 空值 vw/vmin/calc/lh 全形」同根因。
4. **16 pass**：四 length/number/% 槽 plain 非 ch 腿（em/ex/cap/ic）——
   font 槽数学 eval 基建（font_slot_font_px/eval_font_slot_math_number，
   dependency-cycles 批先行）已兑现。

## 根因①（ch 兜底分裂）

WPT 基线自指：`computeUnitInPixels(container, unit)` 设 container
marginBottom=1<unit> 读 gCS——margin 直解路径 1ch@30px → 30px（factor
1.0）。而 calc 内 ch 叶走 CalcContext：

- `resolve_length_with_font_metrics` 的 `LengthValue::Ch` 直解臂
  `font_metrics.map_or(1.0, …)`（css-shapes computed `50ch`@16px →
  800px 钉面批已修——注释明言「旧 0.5em 兜底恰 ×0.5 系统性偏小」）；
- **同函数 Calc 臂 ctx `map_or(0.5, …)` 未同步**——`font-size: calc(…ch…)`
  解析 15px 与直值路径 30px 同元素分裂；
- style-system `font_slot_calc_context` `ch_width: font_size * 0.5`
  同病（其注释「期望 ch 15」系钉面误录——WPT 实测 expected 30）。

## 根因②（三槽 math 臂缺）

- font-style：`FontStyleValue::Oblique(Option<f64>)` 平值存储，
  `oblique calc(…)` 落 f64 parse 恒败。角度叶在 CalcParser parse_atom
  期弧度 Number 化（`parse_angle_to_radians`）→ math 求值结果为弧度，
  需双 provenance 转度（rotate 槽 parse_angle_or_math 同形：反三角/
  atan2 产出节点或顶层角度单位叶）。
- font-feature-settings：整数位 `suffix.parse::<u32>()` 恒败；
  **engine gCS 无 font-feature-settings longhand 回显臂**（双门——即使
  解析成功 computed 亦空串，parseFloat NaN）。
- font-variation-settings：坐标位 `parse::<f32>()` 恒败。

## 码面（7 文件 3 crate；apply.rs 与 face-C fork WIP 零行交叠）

1. `style-system/computed.rs`：Calc 臂 ctx ch 兜底 0.5→1.0（与直解臂
   口径对齐）。
2. `style-system/property/parse.rs`：font_slot_calc_context ch 0.5→1.0
   + 钉面注释更正（expected 30 非 15）。
3. `style-system/property/apply.rs`：font-style 臂追加 math 形回退
   `eval_font_slot_math_oblique`——型别门容 number/angle 两族
   （`atan2(1em,1px)` 型别 <angle>、`tan(·)` <number>），font 相对实参
   按父级字体基准（font_slot_font_px），弧度→度双 provenance。
4. `css-parser/values/serialize_math.rs`：span_has_top_level_angle_unit
   pub(crate)→pub（style-system oblique 臂跨 crate 消费）。
5. `css-parser/values/parse_layout.rs`：
   parse_font_feature_settings/parse_font_variation_settings 拆
   `*_with_font` 字体基准变体（原签名 None 委托零漂移）+ 整数/数值位
   math 求值回退（§10.5 math function 可用于对应文法位；整数位 round）
   + font_settings_calc_context（ex/cap 0.8、ch/ic 1.0 同合成口径）。
6. `style-system/property/apply_advanced.rs`：两 settings 臂换
   *_with_font + font_slot_font_px(style)。
7. `engine/js_dom_bridge/computed_style.rs`：gCS "font-feature-settings"
   longhand 回显臂（`"tag" <integer>` 列表，Normal→"normal"）。

## 消费方风险核查

- ch 0.5→1.0：resolve Calc 臂全属性面（calc+ch 叶）——直解臂已 1.0 口径
  在库，直值/calc 两路径由此归一；无既有通过钉面依赖 0.5（pva/fonts/
  shapes 哨兵腿电池定谳）。
- 三槽 math 臂：仅既有解析恒败形可达（新回退臂），合法字面形解析零漂移；
  *_with_font 原签名委托 None 保持纯字面（supports/哨兵等无基准调用方
  不变）。
- gCS 新回显臂：font-feature-settings 此前恒空串——无「期望空串」断言面
  （fonts 哨兵腿 P2F 电池定谳）。

## A/B 电池（results-knife-fontcycle）

21:21:50 发射（zw-fc-battery-2123），b0/b1 构建 1m21s/1m17s，legs
21:24:30–21:49:37，GATE-DONE rc=0。b0 md5 `bc43363fd` / b1
`ae57e492`（M2 双 bin 异 md5 ✓）。ride census 7 文件：S1/S2 零扰动；
S3-MISMATCH 定谳＝唯一变动件 `docs/goal/wptfyi-css/master.md`（doc-only，
不进编译输入）——账可收。patch 按 hunk 行域拆分保 face-C fork 在飞 WIP
（apply.rs 残 +23）与 ATP-R2 RegCtx hunks（computed_style.rs 残 +63），
git apply -R 剥除不用 git checkout。

**RECONCILE：gained=30 lost=14 regressed=17 pages_gain=12 pages_loss=1。**
剔除探针件（zzz-fontslot-probe，gitignored 仪表，不入账）后真账：

- pva `font-property-unit-cycles.tentative.html` 20→30 **+10**（ch×6 +
  cap/em/ex/ic×4，全数 "calc() with \<unit\> uses parent \<unit\>" 形＝
  ch 兜底 1.0 + Calc ctx font 槽度量兑现；tan/atan2 三角腿残 49F）。
- css-fonts 11 页 **+20**（font-style-parsing +6、font-variation-settings-calc
  +3、parsing/feature+variation-settings-computed +3、font-parse-numeric
  +2、serialization/inheritance/calc-in-variation/sign-function 各 +1）。
- css-fonts `discrete-no-interpolation.html` **−14**（见 P2F 法证）。
- shapes/typed-om/transitions 三哨兵零差异 ✓。
- **净 +16**（+30 − 14）。

## P2F 法证（14 行全数 discrete-no-interpolation.html font-feature-settings）

**未遮蔽预存红（unmasked-vacuous），非本刀行为回归。** 机制链：

1. 上游 harness `interpolation-testcommon.js:383-388` 的 expected 由
   **本引擎自身 gCS 往返**派生（把原始串 set 到探针元素再读
   getComputedStyle）；测试名 `should be ["smcp"]` 显示原始串。
2. b0（无回显臂）：expected=`""`、actual=`""`——**空串对空串虚过**。
3. b1（回显臂入库）：expected 变真值 `"smcp" 1`，actual 侧暴露两类
   **预存**离散动画缺陷——CSS Animations/WAAPI 在 t≥0.5 仍产 from 值
   `normal`（8 行）；transition-behavior:allow-discrete 在 t<0.5 产
   to 值 `"smcp" 1`（6 行，同页 font-kerning/font-variant-\* 同形
   预存真红旁证）。
4. 本刀 plain 路径解析行为零变化（`*_with_font(None)` 纯委托）——
   actual 侧机制未动，P→F 全由 expected 侧变真导致。

**处置**：净 +16 入账（指标口径 −14 为诚实记账，这 14 行在 weekly 账面
本属虚过）；兄弟 fork **ffix r2** 在飞（transition_registry
DISCRETE_TRANSITIONABLE +font-feature-settings，收口 allow-discrete 6 行）；
残 **Animations/WAAPI 8 行**＝动画 keyframe discrete 翻转机制域新刀候选
（与 transitions 事件系统鲸面不同机制）。

**ffix r2 并行验证结果（22:04 收，b0 vs r2b1＝face+ffix 组合态）**：
css-fonts 单语料 **gained=146 lost=0 regressed=0**——本刀 14 行 P2F 全数
回绿（registry 入列后离散翻转臂对 Animations/WAAPI 同样生效，8 行一并
收口），另 font-variation-settings-interpolation +113、composition +11、
zzz-fc-diag +2 归 ffix/animation-WIP 域（其流自账）。落库序：本刀
`98d376f40` 先行，ffix 随后，fonts 侧合计 +30 零 P2F。

## 仪表读数（zzz-fontslot-probe，不入账）

b0→b1 三行值变化：oblique calc 两形 computed 由 `normal`→`oblique 10deg` /
`oblique 29.99999999999993deg`（≈30＝容器 30px 基准正确，浮点噪声注记）；
`"tnum" calc(…)` declared 已解析但 computed `""`→`normal`（apply 路径
math 形仍落 Normal 兜底——settings math apply 臂残面）。

## 残面（让渡记）

- registered 形 60F → ATP fork regprops 域（registered-property-
  computation 32F 同根因，其修复顺带本页 registered 腿）。
- unit-cycles.html 23F + var-reference-unit-cycles 2F → computation 簇
  认领候选（var() cycle 检测 engine 域）。
