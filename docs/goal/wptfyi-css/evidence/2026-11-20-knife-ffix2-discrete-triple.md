# 刀 ffix2：discrete 离散域三性补臂 + font-family 过渡位（2026-11-20）

> 收账代际注记：03:08 电池发射、03:38 GATE-DONE 后原流静默未收账；
> 09:3x 由 FFIX2-TAKEOVER fork 按 results-knife-ffix2 工件接管收账落库
> （face 两文件 + evidence + master.md），NOTES-knife-waapi-fork-roles.txt
> 有接管法证与提交面 compile 闭包定谳。

## 池与 face

池＝fontcycle P2F 法证暴露的两类预存离散动画缺陷（Animations/WAAPI
t≥0.5 恒 from 值 8 行/性 + allow-discrete 不在列恒跳终值）：css-fonts
`discrete-no-interpolation.html` 主靶 42F（font-kerning/font-variant-alternates/
font-variant-numeric 三性离散面）+ font-family 过渡位 3F。face＝两文件：

1. `crates/engine/src/animation.rs`（单 hunk，`apply_single_property`
   三臂）：font-kerning（auto/normal/none keyword 严格解析，失败不动样式
   → canonicalize None 回落零回归方向）、font-variant-numeric
   （`parse_font_variant_numeric` 九变体映射）、font-variant-alternates
   （`parse_font_variant_alternates` 直赋 + features 清槽）。串化经 gCS
   回显臂与期望元素同通道（gCS 臂与值类型系 fontcycle ffix/echo 批
   `7d119d8ef`/`98d376f40` 先行落库，本刀不触）。
2. `crates/engine/src/js_dom_bridge/transition_registry.rs`：
   `DISCRETE_TRANSITIONABLE` 11→15，加列 font-kerning /
   font-variant-alternates / font-variant-numeric / font-family——
   animation-type: discrete（css-values-4 §animations），仅 allow-discrete
   可过渡（css-transitions-2 §transition-behavior-property）；不在列时
   过渡不启动恒跳终值（allow-discrete 6 行/性 + font-family 3 行根因）。
   承 ffs/fvs 同批配方。
   - https://drafts.csswg.org/css-fonts-4/#font-kerning-prop
   - https://drafts.csswg.org/css-fonts-4/#font-variant-numeric-prop
   - https://drafts.csswg.org/css-fonts-4/#font-variant-alternates-prop
   - https://drafts.csswg.org/css-fonts-4/#font-optical-sizing-prop（optical
     续刀另收，见 NOTES-knife-optical-prep.txt）

## 电池（zw-ffix2-battery，03:08:30 发射）

- b0 `99df8a5966b12c746e0ee4aacfcf3b06`（face 剥除）/ b1
  `84dd2e5f9bdc6a6e8dcffe6d9e17c2a5`（face 入树，35.9s 构建）。
- face marker 门缺席（face 注释 CJK，strings 不提取非 ASCII——r6 同型
  已知），收账以内容三角替代：树 diff 两文件 + 双臂构建 log + bin md5 +
  reconcile 唯一异页恰落 face 域（fonts discrete-no-interpolation）。
- RIDE census：12 文件（optical 预备 + 域外休眠 WIP，见 notes）S1/S2/S3
  md5 零扰动；电池窗内核对全 fork 零 crates/ 编辑（master.md :13 发射笔）。
- 腿：css-fonts（主靶）+ css-transitions/css-animations/css-properties-
  values-api/css-typed-om/css-shapes（五哨兵）×{b0,b1}，test-guard 包裹
  （leg 4/8/3000s + CASE_WALL_S=60），SCRIPT-RC=0。

## 收账（03:38:19 RECONCILE）

| 项 | 值 |
|---|---|
| 主靶 | css-fonts `discrete-no-interpolation.html` P 487→532（+45，F2P 全数） |
| TOTAL | **gained=45 lost=0 regressed=0 pages_gain=1 pages_loss=0** |
| 哨兵 | transitions/animations/pva/typed-om/shapes 五语料逐页零差异 |
| rc | 全腿 rc=1（harness 残 F 正常态），GATE-DONE + TRAP-NOOP done |

局部账累计 +7484 → **+7529**。

## 残面（预备账已在 NOTES-knife-optical-prep.txt，optical 续刀另收）

- font-optical-sizing 离散 14F（同页 8+6 反向两类）：animation.rs 无
  optical 应用臂 + registry 未列——optical 续刀 face ①②。
- decl 白名单缺口：font-kerning/font-optical-sizing/font-variant-numeric
  invalid 页 13F（白名单外放行 store 原文回显即 Fail；alternates 已在列
  15/15 全绿对照）——optical 续刀 face ③（apply.rs
  VALIDATED_PROPERTIES + 分派臂）。
- matcher keyword/parse 严格表 kerning/numeric 行已在 HEAD；optical 行
  预备 hunk 在树。
