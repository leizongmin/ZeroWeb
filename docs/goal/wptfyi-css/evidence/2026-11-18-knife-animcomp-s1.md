# 刀 ANIMCOMP S1：animation-composition CSS 属性通道（写面文法门 + 注册链）

日期：2026-11-18。认领：`.acceptance/wptfyi-css/CLAIM-knife-animcomp-css-channel.md`
（21:2x，RP1 收口 fork 连任）。本文件为 S1 段收口账；S2/S3（CSS 动画 →
Animation 对象桥 + 组合机制）待桥面落库后接续（见Claim 依赖节）。

## 池与定谳（census 基线 results-knife-fv1/css-animations.b1.json，19F）

- parsing/animation-composition-invalid.tentative.html 4F＝裸收 round-trip
  （`auto` / `add replace` / `add, initial` / `initial, add` 原文透传）＝写门
  缺失直接证；
- parsing/animation-composition-computed.tentative.html 1F＝registry 无条目
  （supported 探测 false 先于串化）；
- composition + keyframes 12F + compositeOrder 2F＝CSS 声明动画未入
  getAnimations() 对象桥（`el.getAnimations()[0]` undefined）＝S2 前置，
  不在本刀。

## 码面（11 触点，animation-fill-mode 全模板镜像）

- style-system：apply.rs 白名单 65→66 + 分派臂（css-animations-2
  §animation-composition `<single-animation-composition>#`——replace|add|
  accumulate 逗号列表；`auto`/单项空白多 token/css-wide 列表成员拒；替换
  函数/转义形放行 store 原文——TP 臂同口径）+ registry 初始值 + 属性清单
  + types AnimationCompositionList 变体 + apply_advanced 解析/臂 +
  default_impl/inherit + computed 字段/vec_cap + matcher/supports 臂；
- css-parser：枚举 + parse_animation_composition；
- engine：bridge import/computed 臂/str fn。

## 门禁与钉测

- style-system lib 2571/0（钉测 `knife_animcomp_write_gate_verdicts` 首执
  抓红 1 处——empty=上游清除语义恒真，PIN-DOA 纪律修正后绿）；
- engine lib 4208/0；己方十文件 fmt 清；
- clippy：style-system/engine 桌面被 css-parser ATP 他流 WIP 3 错阻塞
  （at_rules/property_validate 非本刀面）——挂账树静后补。

## 电池账（results-knife-animcomp-s1，23:31 发射 23:37 GATE-DONE rc=0）

四腿：css-animations 主靶 + css-properties-values-api 哨兵 × b0/b1；
ride-along 他流 WIP 九文件双 md5 快照（S1/S2 窗内零扰动）。

- **css-animations：F2P=5 P2F=0 GONE=1 net=5**——F2P 逐条＝invalid 4 形
  （auto / add replace / add, initial / initial, add should-not-set）+
  computed 1 形（`replace, add, accumulate` supported+串化）＝S1 靶面 5 形
  精确兑现，零外溢零回归；
- **GONE=1 法证**：display-none-dont-cancel.tentative.html 第 5 测行 b1 侧
  被 `Timeout` 页级标记替换（CASE_WALL_S=60 截断伪影；b0/b1 该页均 5 测、
  前 4 测 F 全同态）——ΔP 中性不入刀账；
- **哨兵 pva：0/0/0**（F2P/P2F/GONE 全零）。

S2/S3 残面（composition 6 + keyframes 6 + compositeOrder 2 = 14F）待
Element.getAnimations() 桥刀接续；master 残面池已挂账。
