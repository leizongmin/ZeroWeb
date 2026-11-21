# 刀 A（TEXT-INTERP 四属性家族）收账判定 · 2026-11-22

## 结论

**TOTAL gained=7 lost=0（P2F=0），全额入账，tally +13070→+13077。**

四属性 text-justify / hyphenate-limit-chars / line-break / hyphenate-character
九站注册（types/parse/registry/apply/computed_style/default_impl/inherit 双表＋
js_dom_bridge gCS 三臂＋三串化器＋animation.rs strict lerp 臂＋FACE-MARKER），
face 由实施流编写、verify 流九站复核后经 ds 流发射电池实证。

## 电池证据

- 发射：`zw-knife-text-interp-ds.service` 02:35:09（log
  `knife-text-interp-battery-ds-0235.log`），等 s2a r3 电池排水（02:47:47）
  后过树门；FACE-MARKER b0=0 b1=1，M2 门（b0/b1 md5 异）过，RIDE 3 文件
  S1/S3 零扰动。
- legs：四语料 values/transitions/animations/text × b0/b1 全部产物非空
  （2062K/320K/253K/903K），rc=1 为该 runner 语料模式常态。
- RECONCILE 03:19:54：values gained=4 lost=0、transitions 0/0、
  animations 0/0、text gained=3 lost=0。

## 增益恰集（逐案归因）

### text +3（css-text/inheritance.html，b0=b1=42 断言无 ASYM）

- `Property line-break inherits`：fail→pass（inherit.rs 继承注册）
- `Property text-justify has initial value auto`：fail→pass（registry initial）
- `Property text-justify inherits`：fail→pass（inherit.rs 继承注册）

RECONCILE 页名关键词过滤器（text-justify|line-break|hyphen|text-indent|
text-align-last）未覆盖页名 `inheritance.html`，CASE 行未打——恰集由 JSON
事后逐案提取补齐，属脚本盲区不涉结果（后续刀可将过滤器扩为 inheritance 页）。

### values +4（css-values/calc-rounds-to-integer.html，ASYM b0=72 b1=80）

b1 独有 8 断言全部为 hyphenate-limit-chars CSS.supports 前置臂
（GRID-PLACESHORT +38 同款形态；b0 未注册属性→supports false→臂不生成）：

- pass 4：`should accept "10"`＋`should not accept "1.1e1"/"10.1"/"1e1"`
  （整数接受＋非整数拒绝正确）
- fail 4：`should accept "calc(1.1e1)"/"calc(10)"/"calc(10.1)"/"calc(1e1)"`
  （calc() 分量解析未支持）

ASYM 挂起律核验：gained 页两侧断言数不等须挂起的意图是防不可归因的
phantom gain；本页 8 断言 100% 归因 hyphenate-limit-chars 前置臂（恰集
逐条列出），按 GRID-PLACESHORT 先例**可归因部分入账（+4），4F 如实挂账**。

## 零位移与红线

- transitions/animations 双语料零位移：CSSA/WAAPI sample 走 lenient
  `interpolate_property_value` 默认臂（离散翻转），strict 委托臂缺口
  （hyphenate-limit-chars 整数对 sample F2F）使 sample 类无增益亦无回归，
  补臂（FI 刀同款 ~10 行）挂后续刀。
- P2F=0，红线保住；无 random flake 案。
- s2a r3 电池同窗结果（values lost=12 单页 P2F，strict 通道 size-keyword
  行为改变所致）已转达 S2 流 claim NOTES，非本刀账域。

## 协调过程记略

02:22 verify 代发 battery 因 s2a r3 face 树内驻留被 PATCH-FOREIGN-HUNKS
预期防线拒绝（rc=2，无产物）；S2 流 02:20 发射 r3（互锁处置 b 抢先序）；
ds 流 02:35 剥 s2a face（r3 b1 legs 预建 bins 窗＝不动树安全窗）后重发射。
verify 流全程未双发（flock 同键），转监视＋判账协作。r3 12 P2F 警报已入
双方 NOTES（收账前必读），S2 流判定权归 S2 流。

## 遗留挂账

1. hyphenate-limit-chars calc() 分量接受（4F，parse 域，WPT 期望接受）
2. lenient 委托臂缺口（hyphenate-limit-chars，CSSA/WAAPI sample 类增益面）
3. hyphenate-character / hyphenate-limit-chars 的 inheritance 页余量断言
   （本页 42 断言内未翻部分，含 gCS 序列化面可能缺口）
4. M4 探针（hyphens initial→auto、text-align-last auto→start 离散属性
   采样 t<0.5 取 to 值之谜）下轮参考
