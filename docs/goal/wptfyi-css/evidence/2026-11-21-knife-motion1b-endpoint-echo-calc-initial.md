# 刀 MOTION-1b：offset-rotate 端点原文 echo＋calc 两词化＋offset initial-value 臂（2026-11-21）

## 刀位与动机

MOTION-1 收账遗留三残面：①r2 P2F ×2＝bare-`reverse` at(1) 端点期望作者原文
`reverse`、MOTION-1 lerp 出 canonical `auto 280deg`；②calc 复合角端点
`auto calc(90deg - 0.5turn - 300grad + 0rad)` 被 split_whitespace 碎词 3+
token 计数门整拒（supports 门 F，Verbatim 死代码，r1 分析预记估 +30~40）；
③MOTION-1 known_properties 注册 offset 简写缺 apply_initial_value 臂。

## 码面（face 3 文件）

- **engine animation.rs**：`offset_rotate_endpoint_echo_raw` 判定＋双臂
  （transitions 非 strict + strict）`t == 1.0` 端点原文 echo——裸关键词
  （bare `reverse`）、乱序复合（`100grad auto` 角先行）、calc 角端点回作者
  原文（css-values-4 §8.5.6 端点＝端点计算值，语料 at(1) 原文钉）；
  canonical 复合（关键词先序＋plain deg 角）与纯角走既有 lerp——at(1.5)/
  at(2) 外推采样与 block10 canonical 折叠期望不直通。
- **style-system apply_advanced.rs**：parse_offset_rotate_value 改
  split_paren_aware 括号感知两词化——calc() 角实参含空白不再碎词整拒，
  calc 复合形收进 Verbatim；MOTION1B FACE-MARKER
  `MOTION1B endpoint-echo-calc-paren-aware-initial-arm`（代码字面 ASCII）。
- **style-system inherit.rs**：offset 简写 apply_initial_value 臂——整簇回
  初始（path None/距离 0px/Auto/Normal，css-motion-1 §offset）。

## blanket 首发失败归档（r1，results-knife-motion1b-blanket-r1）

首版 `t >= 1.0` blanket 直通：motion −26（F2P 18 / P2F 44）＋composition
−3。根因＝t≥1 吞外推采样——at(1.5)/at(2) 期望 lerp 外推值（如
`100deg→180deg at(2)` 期望 `260deg`），直通 echo 端点原文翻转 44 断言；
t==1 仅 1 真损失（neutral→20deg WAAPI underlying 组合对）。修正＝收窄为
`t == 1.0` ＋端点形判定，blanket 结果目录整体归档不入账。

## A/B 电池（knife-motion1b-battery.sh r2，log knife-motion1b-battery-r2-0057.log）

- b0 = HEAD + ride（face 剥除）md5 `0d54b68f642879e6e4d77b8c06e6194b`；
  b1 = HEAD + ride + face md5 `6f0f735334621d2cbea02b2c73cb4777`（M2 异 ✓）。
- 门禁全绿：TREE-COMPILABLE＋FACE-MARKER b0=0/b1=1＋S1/S2/S3 ride census
  （8 文件含对端 s419 es_module.rs，md5 恒）＋TRAP-NOOP rc=0；12 腿
  （motion/borders/masking/shapes/ui/typed-om × b0/b1）JSON 全非空零
  LEG-EMPTY；01:02:29 GATE-DONE。
- 发射窗负载 ~4.3（mykernel qemu 已退，远低于 r2 伪红窗 14.45）。

## 收账（b0→b1，独立重算 motion1b-collect.py ＋断言名级枚举）

- **motion 净 +22**（2891→2913）：F2P 22＝calc 复合块 19
  （offset-rotate-interpolation 18：3 臂 × at(-1)/0/0.125/0.875/1/2 ＋
  math-functions 1：`calc(sign(20rem - 20px) * 180deg)` at(0)）＋端点原文
  echo 3（bare-`reverse` at(1) ×2——r2 P2F ×2 反转回执＋乱序 `100grad
  auto` at(1) ×1）；P2F **0**；断言数双侧对称 4955→4955 零 ASYM，45 页恒。
- **typed-om +1**：stylePropertymap offset-rotate
  `calc(0rad + 0deg)` set 门（parse 两词化连带，真实机制非 flake）。
- 哨兵：masking/shapes **SET-IDENTICAL**；ui +1＝outline-017（载敏 flake
  断言，双向不入账先例）；borders −15＝corner-shape-interpolation 127
  flips 双向同页＝CS-1c ±17~20 flake 带，双向不入账。
- RECONCILE stdout 口径 gained=24 lost=15（含 ui +1/borders −15 flake）；
  入账口径 face 净 **+23**（motion 22＋typed-om 1）。

## 单测

- engine：strict 臂 +2 钉（外推 `auto 100deg→reverse at(2)`＝
  `auto 260.00deg`、calc 端点 at(1) 原文 echo）＋transitions 臂 +2 钉
  （纯角外推 `260.00deg`、block10 canonical 折叠 `reverse -170deg→auto
  50deg at(1)`＝`auto 50.00deg`——原文直通会翻转该面）＋既有 5 断言
  （bare-reverse/乱序/calc/0.875 lerp/跨类 None）；offset_rotate 过滤 3P。
- style-system：calc 复合两词化 3 断言（`auto calc(...)` Verbatim/
  `reverse calc(...)` Verbatim/真 3+ 分量仍拒）＋offset initial-value 臂
  1 测（整簇回初始）；offset 过滤 14P。

## 残面挂账（MOTION-2 候选不变）

path() 命令插值+绝对化 canonical（001-004 656F）；anchor/position
<position> 对插值（266F）；shape() arc 命令（008 204F）；offset-path
composition add（84F）；parsing 页 bare-reverse gCS canonical（'auto
180deg' 期望——与 interp 页端点原文 echo 语义分层）；ray↔url animations
臂残面；calc 复合 composite 残。
