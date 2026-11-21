# 刀 OM-17 · typed-om 残池负值/越域数值族 37 靶（2026-11-22）

## 认领与边界

- 池＝OM-16 电池 b1r css-typed-om.json 残池（99F/50 页普查
  `results-knife-om17-census/census.json` + cluster.py 聚类）。本刀恰集
  **38F/19 页 → judge 修正 37 靶**（height -3.14em 幻影靶剔除——validate
  白名单对 height 负 length 本就放行，b0 已 Pass，恰集程序化导出误含）。
- 三族：**store 拒收面 20F**（width/min/max 尺寸负值、font-weight
  [1,1000] 外、z-index 非整数、时长负 ms、font-size-adjust/font-stretch/
  text-size-adjust 越域、opacity var()）＋**computed 弃置面 12F**（height
  -3.14em→'auto'、perspective -3.14em→'none'、background-size/
  column-width/flex-basis/tdt→keyword、text-underline-offset calc→keyword）
  ＋**number 钳位面 6F**（flex-shrink/animation-iteration-count 负数弃置、
  stroke-miterlimit 负数保形——computed 合同 clamping_number 期望
  Unit(0,'number')；font-weight 0/-3.14 期望钳 Unit(1,'number')）。
- 残池余 ~61F 不入本刀：column-rule-width 5F（unsupported-class 刀）、
  perspective 0px→'1px' 引擎 bug（engine lane）、关键词 unsupported 族
  （break/clip-path/offset-rotate/pointer-events/scroll-snap-align/
  transform-style/writing-mode/text-overflow/overflow-wrap/break-spaces/
  color/flex-basis fit-content/perspective-origin none 等）、
  get-auto-min-size 6F、transform/序列化/IDL 族、animation-duration
  'is auto' 1F（engine lane）。

## 根因（探针 zzom17probe.html 三轮实证）

1. **N1 store 拒收（20F）**：part03 setProp 内嵌 `validate_decl_value`
   （style-system/property/apply.rs:2913）白名单值域门拒收越域声明——
   CSSOM setProperty 合同（赋值忽略保留原值），但 typed-om `set` 无逐
   属性值域门（css-typed-om-1 §set：值已按注册 syntax 结构化解析，store
   步不再跑文法白名单）→ 指定值面 `Specified value must be a
   CSSStyleValue` 20F。
2. **opacity 归一重写（3F）**：store 归一 %→number（OM-3 挂账原文）→
   specified echo 失败（0%/3.14%/-3.14% 期望 Unit(n,'percent') got
   Unit(n/100,'number')）。
3. **N2 computed 弃置（12F）**：已入库越域声明被引擎计算链弃置回
   keyword/默认 → computed 面回 CSSKeywordValue 12F。
4. **number 钳位缺臂（6F）**：引擎对 flex-shrink/animation-iteration-count
   负数弃置回 '1'、stroke-miterlimit 负数保形回显。

## face（part03.js +~27/−2 · part10-typed-om.js +~185/−0，patch md5 `cbf0ac0fa8c098d212ebc2b4181512cb`）

1. **part03 直推通道**：`_styleProxy` 增 `__zwSetPropUnchecked(name, value)`
   （双下划线名避开属性枚举/for-in 消费面——`__zwRawDecl` 先例）＋闭包
   双旗标 `_zwSetPropSkipValidate`/`_zwSetPropSkipNormalize`（同步置位-
   清位 try/finally 包 setProp 单调用）——值域拒收门前置短路一处＋归一
   早退原文透传（opacity %→number 重写槽保形回显：归一门重写会使
   specified echo 失配，探针 census 'expected 3.14 got 0.0314' 实证）。
   归一/简写展开/raw 影子/`_mo_notify`/CE 派发全链保留（OM-16 操作窗
   单派发合同不变）。CSSOM setProperty/IDL 写路径不接此通道（零漂移）。
2. **part10 四表三臂**：
   - **Arm A（mapSet 读回对账）**：setProperty 前后 read-back，
     `_afterRaw !== text && _zwOmStoreUncheckedSlots[p] && (_afterRaw ===
     _prevRaw || p === 'opacity')` → `__zwSetPropUnchecked(p, text)`；
     读回等式天然守门——echo/归一等形不触发。
   - **Arm B（computed NegRange 臂）**：face getPropertyValue 链 OM-11
     整数族臂后增 `_zwOmNegRangeSlots[p]` 臂——声明文本 calc/严格数值形
     门；ForceDecl 槽严格数值形直返；非 ForceDecl 槽引擎串非数值形
     （keyword 弃置）才返声明、词表单关键词直返（css-typed-om-1 §computed
     关键词原样）；GridClampPct 槽 % 形直返（裸 % token 与纯 % calc 入臂，
     混族 calc/多值列表回落引擎）。
   - **表扩充**：`_zwOmStoreUncheckedSlots` 12 槽；`_zwOmNegRangeSlots`；
     `_zwOmForceDeclSlots` 8 槽；`_zwOmClampNegNumProps`
     {animation-iteration-count, flex-shrink, stroke-miterlimit,
     font-size-adjust}；`_zwOmGridClampPctProps` += {width, min/max-width,
     height, min/max-height, background-size, flex-basis, font-stretch}；
     `_zwOmOpacityClampNumProps` += opacity（负 % 引擎计算串 '-0.0314'
     须按 css-color-4 §opacity 钳 0；正形引擎已钳化 no-op）；`_zwOmIntProps`
     += z-index；`_zwOmNonNegativeProps` += {animation-duration,
     transition-duration}（specified Sum 包裹）＋mapGetAll font-weight
     [1,1000] 外包 Sum 臂。
   - **canonicalize 新臂**（Unit + Sum 折叠尾同型，承 tab-size/OM-11 先例）：
     时长族 ms→s(/1000)＋负→Unit(0,'s')；font-weight <1→1 >1000→1000；
     `_zwOmClampNegNumProps` 负 number→0。

## 电池与返修链（承 knife-om16 配方）

- b0r（face 剥除＝HEAD 树）/b1r（face 重放）双臂，patch md5 断言＋双向
  校验＋trap 归一（b1 face 态即工作态）＋test-guard 包裹＋M2 异 md5＋
  FACE-MARKER（SetPropUnchecked part03 token b0r=0/b1r=7、NegRangeSlots
  part10 token b0r=0/b1r=2）。腿 {css-typed-om 主靶, css-values,
  css-sizing, cssom} × 双臂（css-sizing＝尺寸族重消费哨，cssom＝CSSOM
  setProperty 拒收合同哨，css-values＝typed-om 读径哨）。judge：37 靶恰集
  F2P＋GRAND P2F=0＋new/gone=0＋恰集外 F2P 出列打印。
- **v1**（battery-20261121-2212.log 22:24）GATE FAIL：TARGET-GAIN 34/38、
  P2F=0、css-sizing/cssom SET-IDENTICAL。4 靶未翻逐案定谳：①幻影靶
  height -3.14em（恰集修正 38→37）；②height/width -3.14% computed 臂
  「引擎串数值形回落」误判——引擎 gCS 对 % 形解析 used px（'3.14%'→
  '25px'、'-3.14%'→'-784px'、负形钳 '0px'）非 keyword 弃置形，Unit(px)
  对 clamping_percentage 期望 Unit(clamp(p),'percent') 失配；③tUO
  calc(0% + 0%) percent→px 绝对化折叠尾自摆乌龙（语料 propertymap 默认
  合同期望 percent 回显）。另 v1 css-values new=431 定谳**批中语料漂移**
  非 face 效应（random 20 页 22:18 由 rshare salvage2 从 Z2 拷入，跨
  b0r/b1r 腿窗；b1r random-invalid 39P≡rshare b0 39P）。
- **v2**（v2b-battery-r2.log 00:00）GATE FAIL：TARGET-GAIN **37/37 全翻**、
  GRAND F2P=47、new=0 gone=0（漂移域双臂语料恒同复核干净）、三哨
  SET-IDENTICAL，唯 **GRAND P2F=4＝font-weight 四关键词**
  （'bolder'/'normal'/'lighter'/'bold'）——v2 关键词直返臂置位先于
  ForceDecl 分流，font-weight 关键词声明被 keyword reify，而该族
  computed 合同为数值化（引擎 echo '700'→Unit(700,'number')，v1/v2 b0
  臂双绿实证）。恰集外 F2P 10 靶全数兑现预注册（width/height {0%, 3.14%,
  calc(0% + 0%)} ×2 页＝6＋'auto' ×2＋flex-basis 'fit-content' ×1）＋1
  计划外 computed.tentative.html（增益保留）。
- **v3 收窄**（patch md5 `cbf0ac0f…`）：关键词直返臂移入非 ForceDecl
  分支；ForceDecl 子集恢复 v1 原样（仅严格数值形直返）。judge 不变。

## 判读（v2c 电池 00:06 build → 00:18 GATE DONE，**OM17-GATE-PASS** rc=0）

二进制绑定 b0r=`941d58a1ccda1e210ee4cb0905feb203` / b1r=
`fa1b4283367f9a07b7787a5948d8da57`（M2 异 md5）：

- [css-typed-om] 12454 keys **P 12337→12384（+47）**：F2P=47 P2F=0
  new=0 gone=0——37 靶恰集＋恰集外预注册 10 靶全兑现＋计划外
  computed.tentative 'Computed StylePropertyMap contains computed values
  and not resolved values' 增益保留（归因 %/keyword 直返链）；
- [css-values] 1385 keys P 982→983 F2P=1：`random-computed.tentative.html
  :: Shared globally: random(--identifier, a, b)` **定谳 flake 非 face
  效应**——runner random() doc_seed＝文档指针地址（style-system/src/
  lib.rs:3273 `doc_seed: std::ptr::from_ref(doc) as u64`），b0r/b1r 两次
  独立调用进程地址不同 → random() 采样不同值；v2 电池同页
  SET-IDENTICAL 佐证非 face 面；不入账；
- [css-sizing] 5700 keys SET-IDENTICAL（P 3239→3239）——尺寸族 CSSOM
  写面未接 bypass 通道核；
- [cssom] 1630 keys SET-IDENTICAL（P 888→888）——CSSOM setProperty 拒收
  合同零漂移核；
- GRAND F2P=48 P2F=0 new=0 gone=0；TARGET-GAIN om17-negrange-v2 **37/37**。

## 预注册对账

- 37 靶全 F2P ✓；GRAND P2F=0 ✓（v2 font-weight 四关键词回绿）；
  new=0/gone=0 ✓（v1 漂移域复核）；恰集外 F2P 预注册 10＋计划外 1
  保持 ✓；FACE-MARKER 双 token 增量兑现（7＋2）✓。

## 工件（本 clone .acceptance/wptfyi-css/，未入库）

v1/v2/v3 patch（knife-om17-negrange{,-v2,-v3}.patch）、电池脚本
knife-om17-negrange{,-v2b,-v2c}-battery.sh、judge knife-om17-negrange-
v2-judge.py、认领 NOTES-knife-om17-negrange-claim.md、results-knife-
om17-negrange{,-v2,-v3}/ 双臂 JSON、电池日志 battery-20261121-2212.log
/ v2b-battery-r2.log / v2c-battery-r1.log。

## 残池（typed-om 通道后续候选另刀）

~61F：column-rule-width 5F（unsupported-class 刀）、perspective 0px→
'1px' 引擎 bug（engine lane）、关键词 unsupported 族、get-auto-min-size
6F、transform/序列化/IDL 族、animation-duration 'is auto' 1F（engine
lane）；另 part10 余池 gCS typed 通道 ~69F、marker 18F 维持挂账。
