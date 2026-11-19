# 刀 ATP：@property 全链恢复道（css-properties-values-api-1）

**状态：电池已收账（VERDICT 兑现，见电池节）**
认领：CLAIM-knife-atp-recovery（孤儿 WIP 九文件恢复道，2026-11-18 深夜
429 断流遗产；恢复配方见 CLAIM 电池配方 v2）。本刀面＝CSSOM/规则通道
全链：css-parser 解析/序列化门（R1）→ engine 注册表接线（R2）→ 动画/
typedom 面（R3）。

## 面账与机制

上一轮 ATP 电池（陈旧二进制 b1787f8e 口径）：css-properties-values-api
F2P=77 P2F=7 net=+70；五类 P2F 根因全定谳（a 逗号门 / b calc 宽容链 /
c 注册表无 removal / d 无效名 prelude extent / e 由 d 衍生）。本轮六
fix 全落后重收账（分解电池 b0/b1 双腿，见电池段）。

## R1 css-parser 解析/序列化门

- `parser/property_validate.rs`（新模块）：syntax 文法门（`|` 分段、
  `<type>`(+`+`/`#` 乘子)、ident 字面量 CSS 宽关键字拒）、inherits 门、
  initial-value 分型门（universal 缺省合法；typed 必在且按联合文法可
  解析 + computational independence：var() 禁、font-relative 单位禁）。
- 序列化：`serialize_css_ident` C0/DEL hex 转义（`--tab\9 tab` 钉）+
  `@property` canonical cssText（描述符固定序、syntax 内容剥引号保形、
  缺 initial-value 段省略）。

### fix A：顶层逗号门改函数括号外计数

初版 `initial-value` 顶层逗号门 `toks.iter().any(Comma)` 把函数参数内
逗号也拒——`rgb(1, 2, 3)`（corpus 'Rule applied [<color>, rgb(1, 2, 3),
false]' 锚）与 `calc(10%, 10px)` 误拒。改 `count_top_level_commas`：
`Function/LParen` 增深、`RParen` 减深，深度 0 层 `Comma` 才计数；
`<color>#` 类逗号列表语法放行（`is_comma_list_syntax`），顶层
`1px, 2px` 列表形仍拒。

### fix A2：typed 臂函数形括号平衡（probe6 zzb2 定谳）

`<length>` initial `calc(10px)` 读回 ""：initial 门 all-token 检查
`Dimension|Number(0)|Function` 把 **RParen** 也卡（RParen ∉ 许可集 →
整值拒 → 规则不进 AST → 未注册 → 读 ""）；`<length-percentage>` 臂无
all-token 检查故同值绿。新增 `balanced_function_form`（括号深度非负且
终 0），length/percentage/angle/time/resolution/integer/number 八臂改
「裸原子 all 检查 ∨ 函数形平衡」双许可；值面可解析性仍由 engine 注册门
strict 分型裁决（parse 门放宽不改变注册语义）。font-relative/percent
排除检查保持 token 级全扫（函数内层同拒）。

### fix D：prelude 期失败消耗 extent

无效名 `@property foo { … }`（名非 `--` 前缀）初版 prelude 失败路径
`return None` 不消耗 extent——残余 body 被 parser 按 qualified rule
二次解析泄漏成 style 规则（cssRules 出幽灵条目，corpus 'Invalid
property name does not parse' 断言 cssRules 无条目面）。改两处失败分支
（名非 `--`、缺 `{`）都调 `skip_malformed_qualified_rule` 吞 prelude +
`{...}` 块/`;` 边界再返 None，规则整体消失（sheet.rules 无泄漏）。

## R2 engine 注册表接线

`js_dom_bridge/registered_properties.rs`：

- `note_stylesheets` 三呼叫点同挂（computed_style_cache 代际重建与
  cssa_registry 同形）；规则通道 syntax 剥引号走语法门、initial 走
  `validate_initial_strict` 严格门预解析计算形；note 侧失败跳过不注册
  ＝"Rule not applied" 语义。
- gCS `--` 分支 typed 串化/initial 兜底与动画插值 `--` 臂自此对 CSS 源
  注册生效；style-system 侧 initial 注入兜底（var() 解析面）不受影响。

### fix B：calc 宽容链缺口（canonicalize_scalar）

`--zzb3` 读 ""：declared 来源 = style-system 步骤 3 initial 注入
（declared=Some("calc(10px)")），canonicalize_scalar LP 臂 parse_lp 无
calc 臂 → None → unwrap_or_default ""。镜像 strict_scalar：先
`calc_eval_strict`（Len→px / Pct→% / LP 混合形保形原文），失败回退
parse_lp 原臂。

### fix C：注册表分表（removal 生命周期）

初版 CSS 规则通道写共享 REGISTRY 只增不删——规则删除后注册残留、
`@property` 离场语义失效；JS `CSS.registerProperty` 重名注册
InvalidModification 拒与 CSS 源 re-registration 后者胜两语义在单表互
斥。分表：JS 通道留 REGISTRY；规则通道新 RULE_REGISTRY，
note_stylesheets 每代整体重建（clear+重喂）——删除检测免费获得。
读面 `lookup` 规则通道优先合并双表（registration_of /
registration_known / syntax_of / names_json 全走）；register 重名检查
扩双表；clear_all 清两表。

## R3 动画/typedom 面

part06.js @property 面 hunk 随本刀提交（atomic 归 ATP face）；动画插值
`--` 臂消费 registration_of 合并视图（fix C 后自动覆盖规则通道）。

## 探针（zz-atp-probe*.html 六页，跑后归档出语料）

终账（bin `58cd6f0b`，六 fix 全落）：probe1 **5P/0F**、probe2 **4P/0F**、
probe3（calc 面）**6P/0F**、probe4 **3P/0F**、probe5 **2P/0F**、probe6
（syntax vs value bisect）**5P/0F**——修复前 probe6 4P/1F（zzb2
`<length>` calc(10px) 读 ""，fix A2 靶）。TB 侧 zz-tb-probe2（unset
in-flight 形）**1P/0F**（fix E 靶，账在 TB evidence）。

钉测：css-parser `at_property_gate` **11 passed**（钉①乘子括号外/②吞
后继恢复/③ident 全等/④transform font-relative 拒/⑤universal var 前置
拒/⑥C0 转义+cssText/⑦函数形整值保形/⑧prelude extent 消耗）+ lib
3312 绿；engine part145 `test_atp_note_stylesheets_registry_pins`
（①CSS 源注册 ②重声明后者胜 ③无效不注册 ④串 canonical ⑤universal
兜底 ⑥换代重建——后一代不含前代规则则前代注册消失，fix C 语义锁）
+ lib 4222 绿。

## 电池（results-knife-decomp，2026-11-19 09:40 GATE-DONE）

分解电池（zw-knife-decomp-battery，15 腿 + unit pins）：bin md5
b0=`54207468` b0s=`a473acbd` b1=`4ca80d11`；pva 双腿 rc=1（子测试级 Fail
存在即 rc=1，非腿失败）。S1-MISMATCH＝脚本 census bug（s2 剥除态混入
ALL_TRACKED，必然不等）；真仲裁窗 s3-vs-s1「face/ride-along 窗内零扰动」
——b1 账可全信。窗内 peer-3 editing 流并行腿共存（load 1.4-4.2）。

**css-properties-values-api b1 vs b0：F2P=80 P2F=0 net=+80，零回归。**

页级分布：at-property.html +39、at-property-cssom.html +32、
determine-registration.html +5、at-property-typedom.html +2、
registered-property-computation.html +1、at-property-animation.html +1。

对比上轮（陈旧二进制 b1787f8e 口径 F2P=77 P2F=7 net=+70）：P2F 7→0
（fix A2 括号平衡 + B calc 宽容链 + D extent 消耗兑现）、F2P +3。

哨兵：css-typed-om b1 vs b0 **0/0/0**（F2P/P2F/页级位移全零，recount 与
account 双口径一致）——@property 注册接线无 typed-om 侧泄漏。

VERDICT：兑现。css-parser 门 + engine 注册表接线双通道账实相符，
cssRules 无幽灵条目面（fix D）与 `<length>` calc 函数形（fix A2）由
钉⑧/钉⑦在电池 unit pins 腿复验（11 passed + 1 passed）。

## 残面

- pva 467F 余量不在本刀机制面（property 注册全链已通，余为值解析/
  animation 集成深水面）。
- css-transitions 腿 P2F=1（transitionevent-interface 单键 Init 形）
  归 TB/TS-RB 合成腿边缘翻转，见 TB evidence；与 ATP 面无关。
