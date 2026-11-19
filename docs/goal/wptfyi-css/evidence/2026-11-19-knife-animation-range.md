# 刀 animation-range 族（css-animations/parsing，2026-11-19 12:4x 认领）

**认领**：NOTES 12:4x 条目；residue-survey 候选 #2。typed-om 域让位
togrid face B/C（其 evidence 明文全占）。a3 刀#2（keyframes-name-valid +
animation 简写）同语料不同页避让。

## 账（pristine css-animations.b1.json）

| 页 | P/F | 主形态 |
|---|---|---|
| animation-range-shorthand.html | 22/111 | 简写展开+双向序列化 |
| animation-range-start-computed.html | 0/30 | gCS 文本 |
| animation-range-end-computed.html | 0/29 | gCS 文本 |
| animation-range-start-valid.html | 21/5 | canonical 序列化 |
| animation-range-end-valid.html | 20/4 | canonical 序列化 |
| animation-range-start-invalid.html | 0/11 | 拒收 |
| animation-range-end-invalid.html | 0/14 | 拒收 |
| animation-range-shorthand-end-zero-offset.html | 3/3 | 边角形 |

style-system 全零支持（grep 无 animation-range）；part10 仅 list-valued
两行既有（'animation-range-end':1 @1861，只读不动）。

## 断言机制（parsing-testcommon.js）

- test_valid_value：纯 CSD——`style[prop]=v → getPropertyValue` 须等于
  canonical 串；再 set(readback) 往返一致。**无 typed-om 无 computed**。
- test_invalid_value：set 后 getPropertyValue 须 ''（拒收丢声明）。
- test_computed_value：set 指定值 → `getComputedStyle().getPropertyValue`
  等于期望（计算值语义）。

## 文法（scroll-animations-1 §view-timelines-ranges）

- 长手 `animation-range-start/end`：
  `normal | [ <timeline-range-name> <length-percentage>? ]#`
  名集 cover/contain/entry/exit/entry-crossing/exit-crossing。
  名必须在百分数**前**（'50% exit' 拒）；normal 互斥独占（'normal 10px' 拒）；
  单段内一名一数（'contain contain'、'cover 50% enter 50%' 拒）；
  'none' 拒。
- 简写 `animation-range`：`<start> <end>?` 逗号列表；单值=start、end
  默认 normal；'normal normal'→'normal'；'normal 100px' 拒。

## canonical 序列化规则（valid 页实证）

指定侧（CSD readback）：
1. `0` → `0px`（单位补全）
2. start 侧省略 0%（'cover 0%'→'cover'）；end 侧省略 100%
   （'contain 100%'→'contain'）
3. calc percent-first 排序：'calc(10px + 10%)'→'calc(10% + 10px)'；
   乘积同样（'calc(sign(100em - 1px) * 1%)'→'calc(1% * sign(100em - 1px))'）
4. calc 同族简化保壳：'calc(10% - 10%)'→'calc(0%)'（简写页指定形）
5. 逗号列表逐段规范化

computed 侧（gCS）：
1. em→px（'entry 1em'→'entry 10px'）
2. calc 同族折叠去壳（'calc(41% + 1%)'→'42%'；'calc(1em + 10px)'→'20px'）；
   百分+px 混族保 calc（'calc(10% + 10px)' 原样）
3. 省略规则同指定侧（'COVER 0%'→'cover'，大小写规范化）
4. 'entry calc(10% - 10%) entry calc(50% + 50%)' computed→'entry'
   （start 0% 省 + end 100% 省）

简写重组：逐段 start/end 配对，end 为 normal 默认时省略（'10% normal'→
'10%'）；start/end 皆默认→'normal'。

## 接线图（animation-direction 先例全链）

1. css-parser values：枚举 + parse fn（AnimationDirectionValue 先例
   parse_basic.rs:1247；新文件或 parse_misc.rs 域）
2. style-system types.rs DeclaredValue 变体（AnimationDirectionList 先例
   @1784；types.rs 已 2451 行超 2000 限——新增类型考虑 types/ 子目录或
   css-parser 侧承载，避免加剧）
3. registry.rs 初始值（@196 邻域）
4. inherit.rs 继承臂（@1447 先例）
5. matcher/supports.rs CSS.supports 臂（@71 先例）
6. apply.rs validate_decl_value 写门臂（@2243；apply.rs 4877 行）
7. engine js_dom_bridge/computed_style.rs gCS 文本臂（@1213
   enum_list_to_css 先例；简写重组 @1228 animation_shorthand_to_css 先例）
8. style-system shorthand/transition.rs animation 简写重组（@224/@307
   mk() 先例）——animation-range 简写在此或兄弟模块
9. parse.rs parse_animation_range_*（parse.rs 1686 行，+臂后 <2000）

## 风险与避让

- togrid face B/C 在飞（part10 12:39 M）；若扩散到 parse.rs/css-parser
  并发窗→碰头信号记 master.md 错峰。
- calc percent-first 排序/简化复用既有 calc 机制（css-parser calc 流
  已落库 parse_layout.rs face A）；fold 复用 serialize_math 既有函数。
- computed em→px 需字号因子（computed_style 现有 em 解析路径复用，
  font-size 16px 默认）。
- P2F 哨兵：typed-om/pva/masking/transitions 四语料（togrid 电池同面）。

## 计划

1. 深读接线点（transition.rs/computed_style.rs/inherit.rs/supports.rs/
   apply.rs 门/css-parser parse_basic 模板）
2. css-parser 类型+parse（含 canonical 序列化函数）
3. style-system 接线（types/registry/inherit/supports/apply 门/shorthand）
4. engine computed 文本臂
5. 钉测（style-system + engine lib，togrid 三钉模式）
6. 探针 zz-ar-* → test-guard 构建 → 单页验证
7. A/B 电池（css-animations b0/b1 + 四哨兵）→ evidence 回填 → pathspec
   commit → guard → master.md

## face A 探针定谳（13:05，bin md5 3295cee6）

实现面收敛为三件（比接线图 9 站更薄——文法/canonical 单源 css-parser，
style-system 只加 validate 门）：

1. `crates/css-parser/src/values/parse_animation_range.rs`（新）——
   `parse_animation_range_side`（split_top_level_commas + normal 独占门 +
   逐段 1/2 token 分派）、`canonical_offset`（calc→fold_math_specified
   Length 槽；sign 树乘积 narrow 兜底 reorder_percent_product_front——
   FIXME 通用 percent-first 乘积排序挂账 css-parser calc canonical 域；
   裸数补 px；`1em` 数/单位拆分状态机——'e' 贪指数会吞 'em'）、
   `animation_range_side_to_css`（侧别省略：start 0%/end 100%）、
   `fold_animation_range_specified`
2. serialize_math.rs fold_decl_specified 加 animation-range-start/end 臂
   （语料驱动注释）
3. apply.rs VALIDATED_PROPERTIES 66→68 + 写门分派臂（css-wide 早放行于
   门上游不受影响；var() 走既有 substitution 早放行）

钉测 `css-parser/src/values/tests/animation_range.rs` 5 枚：start_valid
（23 案）、start_calc（3 案含 sign 重排）、end_valid（18 案含 end 100%
省略）、invalid_reject（14 形双侧）、fold_none_on_invalid——5 passed
0 failed（test-guard 包裹 cargo test）。

探针腿（test-guard 构建 release bin + css-animations 全语料单腿 +
pristine b1 逐页 diff）：

| 页 | pristine | face A bin | Δ |
|---|---|---|---|
| animation-range-start-valid | 21P/5F | 26P/0F | +5 |
| animation-range-end-valid | 20P/4F | 24P/0F | +4 |
| animation-range-start-invalid | 0P/11F | 11P/0F | +11 |
| animation-range-end-invalid | 0P/14F | 14P/0F | +14 |

**TOTAL gained=34 lost=0 regressed=0**——四长手页全数归绿，shorthand/
computed 页零扰动（face B/C 域），全语料零 P2F。发射遇 togrid face B
在飞编译红（parse.rs:351 E0631，13:01 落）扫入→NOTES 碰头条目+
retry-on-green 守望（systemd zw-arfa-probe），其 13:0x 自愈后 attempt 1
即绿（13:05:18，构建 13:03:51 DONE）；bin provenance=HEAD+RIDE WIP
（es_module/part115 惰性面，face A 探针账用）+togrid face B 已自愈态
（grid 域与 css-animations 语料不相交，diff 表零扰动实证）。

face B 设计（已侦察，未实现）：ComputedStyle 存 canonical 指定文本
`Vec<String>` ×2，gCS 臂逐段 parse_animation_range_side→偏移
parse_length→`length_to_css(lv, font_size_px)`（computed_style.rs:2193
已含 % 保留/calc 同族折叠去壳/相对单位 resolve 全口径）→变换后重判
range_offset_elides；六站接线照 animation-direction（default/inherit/
apply_advanced/gCS 臂）。face C：shorthand 展开+读回重组待 face B 后。

## 探针判决（13:05，results-knife-animrange/fa-probe-run.log）

css-animations 全语料腿 vs pristine：**gained=34 lost=0 regressed=0**，
与 face A 预测分毫不差——

- animation-range-start-valid: P 21→26（+5）
- animation-range-start-invalid: P 0→11（+11）
- animation-range-end-valid: P 20→24（+4）
- animation-range-end-invalid: P 0→14（+14）

四长手页全动、其余页零差异（P2F 双保护实测成立：css-wide 早放行 +
fold None 零漂移）。探针 runner provenance：树态=face A+face B WIP 扫入
（bin md5 3295cee6 与 face B 电池 b1 同源树），但 animations 语料不含
grid/scroll-padding 属性面，face B 污染不可能入探针账；face B 面由
battery sentinel 腿（transitions/animations/pva/masking）独立覆盖。

## face B deputy 代跑与探针判决（14:04，results-knife-animrange/fb/）

原主 fork 冻结（NOTES 窗记 13:21 备妥未跑），deputy 认领代跑管线
（NOTES 13:52）。fb-probe 构建三处编译修（WIP 系 13:05 电池构建后新写
从未编译，13:05 b0/b1 构建日志无 css-parser 重编译可证）：

1. `parse_animation_range.rs:416` E0106——`split_halves` 加
   `<'a>(toks: &'a [&'a str])` 统一借用源签名（单一调用点）
2. E0004——kind 按构造只取 0/1/2，match 加语义等价 `_ => return None`
   兜底臂（穷尽性；不取 rustc 建议的 todo!()）
3. E0382——`HalfState` 无字段枚举补 `Clone, Copy`

修后探针（bin md5 7e351d42，构建 14:01:54→14:02:36）：css-animations
全语料 vs pristine b1 逐页——

| 页 | pristine | face B bin | Δ |
|---|---|---|---|
| animation-range-shorthand | 22P/111F | 57P/76F | +35 |
| animation-range-start-computed | 0P/30F | 30P/0F | +30 |
| animation-range-end-computed | 0P/29F | 29P/0F | +29 |
| animation-range-start-valid | 21P/5F | 26P/0F | +5 |
| animation-range-end-valid | 20P/4F | 24P/0F | +4 |
| animation-range-start-invalid | 0P/11F | 11P/0F | +11 |
| animation-range-end-invalid | 0P/14F | 14P/0F | +14 |
| animation-range-shorthand-end-zero-offset | 3P/3F | 6P/0F | +3 |
| Element-getAnimations.tentative | 2P/20F | 3P/21F | +1 |

**TOTAL gained=132 lost=0 regressed=0**（face A 34 + face B 增量 98 同窗
合计）。computed 页 59F 全归绿=engine computed 文本臂 length_to_css
全口径（em→px/calc 折叠/% 保留）生效；shorthand +35 但残 76F，残面
三分：R1 shim 简写 syntax 表缺 animation-range→CSS.supports 门 35F
（全可 F2P）/R2 非法形漏过通用写路径（时间单位/颜色/未知词）~17F/
R3 裸偏移段 end 侧 longhand 未双写 ~6F。Element-getAnimations
tentative +1P/+1F 候选波及面（getAnimations 读 range 侧别）——电池
A/B 实测零位移（见下节），改判归 face A 落库代/运行噪声，不入 face B 账。

## face B 电池收账（gen-1，14:33 GATE-DONE，bin c28757e3/400b78f9）

双腿电池（knife-arfb-battery.sh，systemd zw-arfb-battery）：b0=HEAD
（face A 已落库 `333e88683`）+ride；b1=face B 十一文件。钉腿双绿
（css-parser animation_range 8P/0F；engine side_computed 2P/0F）；
ride census S1/S3 零扰动；TRAP-RESTORE applied；BATTERY-EXIT-RC=0。

**账：face B 净 +97 零 P2F**，四页全数对齐探针预测：

| 页 | b0 | b1 | Δ |
|---|---|---|---|
| animation-range-shorthand | 22P | 57P | +35 |
| animation-range-start-computed | 0P | 30P | +30 |
| animation-range-end-computed | 0P | 29P | +29 |
| animation-range-shorthand-end-zero-offset | 3P | 6P | +3 |

start/end valid/invalid 页零位移=face A 已在 b0 侧，自洽；
pva/masking/transitions 全语料零位移，typed-om 除六页外零位移——
P2F 保护成立。97+53=150 与电池内置对账 TOTAL 闭合。

**+53 污染法证与仲裁**：b1 gen-1 构建窗（14:07:42→14:08:28）与
twin-B part10-typed-om.js 在飞编辑窗（首见 M 14:10:39、14:11:36
revert）重叠，include_str! 扫入——typed-om numeric-objects 六页
（add-two-types +26/arithmetic +8/create-a-type +8/cssMathInvert
+3/cssMathNegate +2/multiply +6）为 ARITH 刀靶面，与 AR face 零机制
交集。ARITH 流 NOTES 仲裁认领 +53（本流电池独立重验后入彼账）；
deputy 采认：face B 账=+97，+53 不入本流账。pva/masking/transitions
零位移证污染足迹窄，哨兵零位移可信。

r2 干净代重建取消：part10 现为 ARITH 在飞 WIP（禁动 peer 件）+
zw-facec-battery2 在飞（错峰）+ 机制面不相交+探针/电池/落库分解
三方一致（132=34+97），重跑无新增信息。gen-1 工件全保留
（results-knife-arfb/{b0,b1,bin}/、knife-arfb-battery-1408.log）。

## face B 电池定谳（14:33，results-knife-arfb/，KNIFE-ARFB-GATE-DONE）

双腿电池（b0=HEAD+ride，b1=全树）×5 语料，test-guard 包裹，钉腿
css-parser `animation_range` 8 passed 0 failed、engine `--lib
side_computed` 2 passed 0 failed；ride census S1/S3 全窗零扰动；
TRAP-RESTORE applied；BATTERY-EXIT-RC=0。

**AR face B 账面：css-animations b1-b0 gained=97 lost=0 regressed=0**，
恰 4 页——

| 页 | b0 | b1 | Δ |
|---|---|---|---|
| animation-range-shorthand | 22P | 57P | +35 |
| animation-range-start-computed | 0P | 30P | +30 |
| animation-range-end-computed | 0P | 29P | +29 |
| animation-range-shorthand-end-zero-offset | 3P | 6P | +3 |

与探针（bin 7e351d42，14:02 构建，先于 part10 污染窗 14:07:42）同 4 页
差分逐页相等——双 bin 交叉验证 face B 纯账。b0 已含 face A（HEAD
`333e88683` 落库，探针判读段所述「+132=face A 34+face B 98」为对
decomp-pristine 旧基线口径；本电池 A/B 差分=纯 face B）。

**污染定谳（不入账 +53）**：电池 headline gained=150 含 typed-om
+53（numeric-objects 算术族 6 页）——二进制 strings 集差分实证
b1-runner 含 `_zwOmTypeAdd/_zwOmTypeOfVal/_zwOmUnitType` 等 typed-om
算术 shim（b0 无、b1only 无 ride 泄漏、有 animation-range 签名 ×5）：
窗内 twin-B 的 part10-typed-om.js 编辑（14:07:42–14:10:39 在树，
14:11:36 被其 revert）被 b1 构建扫入。AR face 无 CSSNumericValue
通路，+53 全记 twin-B arithmetic 域（其对端 evidence
knife-arith-typedom-numeric.md 在飞）。哨兵 pva/masking/transitions
零位移成立；typed-om 除该 +53 外零位移。

残面（shorthand 页 57P/76F）：R1 shim 简写 syntax 表缺
animation-range→CSS.supports 门 ~35F（全可 F2P，随 arithmetic 流
part10 面带解）/R2 非法形漏过通用写路径（时间单位/颜色/未知词）
~17F/R3 裸偏移段 end 侧 longhand 未双写 ~6F/余 ~18F 中间带待判。

## face B 电池收官（14:33:56 GATE-DONE，results-knife-arfb/）

双腿 × 5 语料（test-guard 全程包裹；b0 build 14:06:56→14:07:41、b1
14:07:42→14:08:28；bin md5 b0=`c28757e3` b1=`400b78f9` M2 区分过）：

- b0 = HEAD + 域外 ride（HEAD 已含落库 face A `333e88683` + togrid
  face B `7e453b5fc`，十文件 face B WIP 剥除）
- b1 = face patch（`knife-arfb-full.patch`）重放全树
- RIDE（es_module.rs + part115_s423 测试）S1/S3 md5 census 双点零扰动
- 钉腿双绿：css-parser animation_range **8 passed** rc=0；engine
  side_computed **2 passed** rc=0
- 十语料腿全 rc=1（runner 常态退出）；TRAP-RESTORE applied rc=0；
  BATTERY-EXIT-RC=0

### 独立对账（分语料 ab-compare + 域归因）

| 语料 | gained | lost | regressed | 判 |
|---|---|---|---|---|
| css-animations | **+97** | 0 | 0 | AR face B 净账 |
| css-typed-om | +53 | 0 | 0 | part10 污染隔离（见下） |
| css-properties-values-api | 0 | 0 | 0 | 哨兵零位移 ✓ |
| css-masking | 0 | 0 | 0 | 哨兵零位移 ✓ |
| css-transitions | 0 | 0 | 0 | 哨兵零位移 ✓ |

animations +97 全落 4 页且均 shorthand/computed/zero-offset 域：
shorthand 22→57（+35）、start-computed 0→30、end-computed 0→29、
shorthand-end-zero-offset 3→6（+3）。零 P2F（lost=regressed=0）。
b0-vs-pristine 基线漂移 +64（face A `333e88683` 落库 +34 与
ANIMCOMP `917378f05` 落库后 mask-image-interpolation +30）——双侧共
享基线，不入 AR 账。13:05 段「bin md5 3295cee6 与电池 b1 一致」实为
同源树非同 bin（deputy 三修跨代），实际电池 b1=`400b78f9`。

### typed-om +53 污染裁定（零接触五点证据链）

b1-b0 二进制差只可能来自 10 face 文件 + part10（若落在 b1 构建窗）：

1. face patch 内容审计：`grep -cE "part10|CSSNumericValue|numeric|
   typed-om"` = 0，纯 10 Rust 文件零 shim 面——数值域位移不可能源于
   AR face
2. +53 全部落在 numeric-objects 6 页（arithmetic +8/add-two-types
   +26/create-a-type +8/cssMathInvert +3/cssMathNegate +2/multiply
   +6）= twin-B NOTES 认领的 arithmetic 域
3. part10 瞬态编辑窗 14:07:42–14:10:39（M 态 14:10:39 见、14:11:36
   revert）与 b1 构建 14:07:42→14:08:28 重叠
4. 电池窗后 twin-B 重建 target bin（md5 `17284bb3` ≠ b1 `400b78f9`）
   佐证其构建活动持续
5. 三哨兵（pva/masking/transitions）零位移——污染面恰好只有 twin-B
   域，与 shim include_str 扫入形态吻合

裁定：+53 = twin-B arithmetic WIP 快照（~14:08 时点）经 include_str
扫入 b1 bin，隔离出 AR 账；同时是 twin-B 免费实测——其 arithmetic
目标 ~73F 已 +53（快照时点），残留 add-two-types 分母差可对照。
AR face B 收官净账 = **animations +97 零 P2F，三哨兵零位移**。
