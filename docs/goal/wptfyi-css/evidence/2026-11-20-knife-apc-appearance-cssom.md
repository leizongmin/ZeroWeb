# EV · 刀 APC（appearance 族 CSSOM 四臂）— 2026-11-20 rally-cont

靶面 = css/css-ui/appearance-cssom-001.html 341 断言 13P/328F（corpusexp r3 10:21
HEAD bin 实测；css-ui corpus 1380F 最大单页刀位 24%）。

## 探针定谳（zwprobe/appearance-cssom-probe.html PROBE-0..13，零污染 --wpt-data 镜像发射）

- **PROBE-1**（setProperty('-webkit-appearance','none')）：按原名存储（cssText
  `-webkit-appearance: none`），specified 面 IDL 双形读通（idl_U WebkitAppearance=
  'none'/idl_l webkitAppearance='none'/gp_dashwa='-webkit-appearance' getPropertyValue
  ='none'——part03 :31868 `_stylePropName` webkit 特例既有）＝**写面与 specified
  读面本已通，唯一缺口 = computed 面**。
- **PROBE-2/7**：canonical appearance specified+computed 全通（13P 构成）。
- **PROBE-3/5**：invalid 值原文 store（appearance 'bogus-button' 与
  '-webkit-appearance' 'bogus-button' 均落 cssText）＝写面无值域门（CSSOM-1
  §setproperty 声明忽略缺失）。
- **PROBE-4/11**：`-ms-appearance: none` 原文 store＝厂商前缀形无拒收。
- **PROBE-6**：'-webkit-appearance' 写后 computed 面 gp_prop=''-（
  getPropertyValue('-webkit-appearance') 落空）而 gp_app='none' ✓＝**computed
  通道 webkit 名缺失**：gCS proxy `_camelToKebab('WebkitAppearance')`→
  'webkit-appearance'（无前导横）→ host 查询落空；computed serialize 无
  '-webkit-appearance' 别名臂。
- **PROBE-8**：computed initial＝'button'（button 元素上下文 UA 默认）。
- **PROBE-9/10**：IDL setter（style.WebkitAppearance=/style.webkitAppearance=）
  specified 落地 '-webkit-appearance' 名 ✓。
- **PROBE-12**（逐字节复制页序列）：r1=specified 'none' ✓ / r2=gCS().WebkitAppearance
  ''✗＝页 13F 全系 computed 面第二次 assert_style_for_prop（computed 组）。
- **PROBE-13**（跳过 gCS-first 变体）：r1/r4='textfield' 全 ✓＝gCS 调用无毒化，
  页失败纯系 computed 断言。

## 328F 分解 → face 四臂映射

| 簇 | F | face 臂 |
|---|---|---|
| computed -webkit 读（Values 4+Compat 9 的 webkit 形） | 13 | ③④ |
| Invalid 131×2prop（specified 回显 ''+computed 保持 initial） | 262 | ① |
| Prefixes 15 厂商形拒收 | 15 | ② |

预期满贯 341（值域门拒收 → specified '' + computed initial 天然成立）。

## face 四臂（knife-apc-face.patch，4 文件 +70/−3，静态 patch）

1. **写面值域门+前缀拒收**：callbacks.rs `__zw_appearance_normalize_decl`
   （kebab 门 appearance/-webkit-appearance 外返 "null"；CSS-wide 5 keyword
   小写放行；`zero_css_parser::values::parse_extended_visual::parse_appearance`
   14 值单源 Some→保形 echo/None→"null" 拒；non-v8 回落 "null"）+ part06.js
   `_zwNormalizeDeclValue` appearance 臂（`/(?:^|-)appearance$/` 命中即委托，
   尾部 !important 先剥；'null' 返 null=赋值忽略）。
2. **computed 别名臂**：computed_style.rs serialize
   `"appearance" | "-webkit-appearance" => appearance_to_css(...)`。
3. **computed webkit 名特例**：part01.js gCS proxy has/getPropertyValue 双 trap
   `/^webkit([A-Z])(.*)$/` → '-webkit-' + lower + kebab（CSSOM-1
   §camel-cased-attribute，与 specified 面 part03 `_stylePropName` 对称）。

规范锚 = css-ui-4 §appearance-switching（aliasing）+ CSSOM-1 §setproperty/
§camel-cased-attribute；WPT 页 = appearance-cssom-001.html（常驻语料回归）。

## 编译预验（13:38，发射前）

face apply → cargo check -p zero-engine（test-guard 6/14/600s）Finished 3.76s 绿
→ revert（marker grep=0、status 19 行复原）。precheck 只验树不验 face 的链缺口
当轮补救。

## 电池

knife-apc-battery.sh（OUT=results-knife-apc；主靶 css-ui + 哨兵 masking/shapes/
typed-om × b0/b1 = 8 腿；FACE-MARKER `__zw_appearance_normalize_decl` b0=0/b1=3；
M2 md5 异；RIDE 11 文件 S1/S2/S3 三点零扰动——md5 存档续开轮复验全 OK）。

发射链 census 排队（CS-1 salvage 让位）：zw-apc-launcher 13:30 入队 →
B0 BUILD 14:30:40（44.33s，b0-runner md5 `0acf014b`）→ B1 BUILD 14:31:25
（33.72s，b1-runner md5 `810b8a9a`）→ 8 腿 14:31:59-14:40:16（rc=1＝含
预期失败，产物完整）→ RECONCILE 14:40:16：

- css/css-ui/appearance-cssom-001.html: P 13->175 (**+162**)
- css/css-ui/appearance-initial-value-001.html: P 1->2 (**+1**)
- **TOTAL gained=163 lost=0 regressed=0 pages_gain=2 pages_loss=0**

TRAP-FACE-REVERTED state=done rc=0（恢复发射态=树净）。续开轮 JSON 独立复算
（verify_ab.py，685 页）与 RECONCILE 逐数一致、零 asym 页（双侧断言数全等，
零 phantom gain）；预期满贯 341 未达（175+2 vs 13+1＝+163），残面见下。

## 收账

- **净 +163 零 lost 零 regressed 零 phantom**。归属：appearance-cssom-001
  +162＝328F 三簇按 face 四臂兑现（值域门 invalid 262F 面 specified ''+
  computed initial 天然成立、prefixes 15F 拒收、computed webkit 读 13F 别名
  臂+名特例）；appearance-initial-value-001 +1 同根因顺带翻面。
- 残面（后续刀候选）：175P/341 断言＝残 ~166F，形态与 PROBE 期差异需专项
  取证（疑非 <compat-auto> 14 值覆盖外的 assert_in_array/枚举面 + computed
  UA initial 值面）——appearance-cssom-002/003 未在靶面（后续刀）。
- master.md 账头见「刀 APC 收账」行（face 提交 + 收账同轮）。

