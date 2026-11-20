# 刀 R2 扩面写面白名单门（clip / clip-rule / background-clip / transition-timing-function）

日期：2026-11-18。认领：仲裁 NOTES-knife-r-swarm-arbitration 分工第 2 条
（CLAIM 流 fork）；工件 r2-face-wip-notes.txt；前置刀 R core face
（clip-path，活号 `f55b09a3e`，3861aa9dc 被 autostash rebase 重写）落库后接续。

## 面与机制

四属性白名单接入（VALIDATED_PROPERTIES 60→64）激活树上既有臂（随刀 R
提交搭车入库的 inert 扩面臂）：

| 属性 | 文法 | 臂裁决单点 | driving |
|---|---|---|---|
| clip | `auto \| rect(<top>, <right>, <bottom>, <left>)`（4 项逗号分隔，`<length>`含负值`\|auto`；% 非法） | 臂内联（rect 解析 + parse_length_or_math） | css-masking/parsing/clip-invalid 4F |
| clip-rule | `nonzero \| evenodd` | 臂内联 matches! | css-masking/parsing/clip-rule-invalid 2F |
| background-clip | `<bg-clip>#`（border/padding/content-box + b4 扩 border-area/text 与空白组合特形两序等价） | 臂内联（括号感知逗号列表 + matches!） | css-backgrounds/parsing/background-clip-invalid 5F |
| transition-timing-function | `<easing-function>#`（parse_timing_function 严门 + linear() 停点 + math/树计数嵌套放行） | values::parse_timing_function + linear_stops_valid | css-transitions/parsing/transition-timing-function-invalid 25F |

替换函数形（var()/attr()）全臂放行 store 原文（computed 期级联替换后重裁，
counter 臂同口径）。

## 基线（b0 探针，probe-r2-b0/，pre-face bin md5 8b3eafeb）

css-masking（62 页，2026-11-18 21:15）：

| 页 | b0 | R2 预期 / v1 电池实测 |
|---|---|---|
| clip-invalid | 0P/4F | 4F→P（F2P；实测兑现） |
| clip-rule-invalid | 0P/2F | 2F→P（F2P；实测兑现） |
| clip-valid | 3P/0F | 持平（P2F 护栏；实测持平） |
| clip-rule-valid | 2P/0F | 持平（P2F 护栏；实测持平） |
| clip-computed | 0P/4F | 持平（canonicalization 缺口另账；实测持平） |
| clip-rule-computed | 0P/2F | 持平（同上） |
| clip-path-invalid/valid/computed | 0P/48F、54P/0F、4P/17F | 刀 R 面另账（实测 +48）；R2 零触碰 |
| clip-path-shape-parsing | 24P/20F | 刀 R 面另账（实测 +20——residue-survey 22F 挂 css-parser 刀，刀 R face 实际翻转 20，残 0） |

css-transitions（123 页，2026-11-18 21:15–21:23）：

| 页 | b0 | R2 实测（v1 电池） |
|---|---|---|
| transition-timing-function-invalid | 0P/25F | 25F→P（F2P；auto/`ease-in ease-out`/cubic-bezier 域门形/steps 整数与 jump-none 门形/css-wide 混列表形） |
| transition-timing-function-valid | 22P/0F | 持平（b0 透传已绿，门接入零位移） |
| transition-timing-function-computed | 15P/7F | 持平（computed canonicalization 另账） |

css-backgrounds（124 页，2026-11-18 21:23–21:27）：

| 页 | b0 | R2 实测（v1 电池） |
|---|---|---|
| background-clip-invalid | 0P/5F | 5F→P（fill/margin/stroke-box/view-box/`border`） |
| background-clip-valid | 8P/1F | 持平（镜像序形 `text border-area` 残 F＝非门层失败，见残面） |
| background-clip-computed | 9P/2F | 持平（computed canonicalization 另账） |

勘误（22:0x）：本表早版曾把 valid 页误报为全 F（runner JSON status
`Pass/Fail` 首字母大写，`=='PASS'` 匹配零命中假 0P）——R2 收益域精确
为 invalid 页 +36F，无 valid 页 F2P 潜力；预期性归因（镜像序 +1）未在
电池兑现，不转正（[[verify-red-classification-before-ledger-transcribe]]）。

## 电池（union A/B 双建，knife-r-union-battery-2127）

b0 = `cc8b6708e` apply.rs（pre-knife-R，刀 R 与 R2 双面同缺）/
b1 = HEAD `f55b09a3e` + r2-union-face.patch（白名单 60→64 + 四臂 +
linear_stops_valid + 钉测）。刀 R（clip-path）与 R2（四属性）文法面属性
不相交 → F2P 按属性分派归因无歧义：clip-path 页 F2P 归刀 R core face，
clip/clip-rule/background-clip/transition-timing-function 页 F2P 归 R2。
ride-along 双侧同态相消：es_module.rs+part115（他流 s419/s424d）+
callbacks.rs/registered_properties.rs/part10-typed-om.js（TD1 fork WIP，
s1/s2 md5 法证）。M2 双 bin md5 必异；P2F/NEW/LOST 零容忍。

五语料双腿：css-masking / css-shapes / css-transitions / css-animations /
css-backgrounds（CASE_WALL_S=60，test-guard 4/8/3000 包裹）。

## union r1 权威账（21:27:43→21:55:18 GATE-DONE RC=0 零 LEG-FAIL，
b0 `8b496720` / b1 `38d4b216`＝本提交精确面，results-knife-r-union-r1/）

| 语料 | b0 | b1 | ΔP | F2P | P2F |
|---|---|---|---|---|---|
| css-masking | 4703P | 4777P | +74 | 74 | 0 |
| css-shapes | 5915P | 5967P | +52 | 56 | 4 |
| css-transitions | 823P | 848P | +25 | 25 | 0 |
| css-animations | 516P | 516P | 0 | 0 | 0 |
| css-backgrounds | 3491P | 3495P | +4 | 5 | 1 |
| **合计** | | | **+155** | **160** | **5** |

逐页 F2P：clip-path-invalid 48 + clip-path-shape-parsing 20（residue-
survey 曾标 22F 归 css-parser 刀，core face 实际翻转 20 残 0）+
path/rect/shape-function-invalid 2/14/40（core face 域 124）；
clip-invalid 4 + clip-rule-invalid 2 + ttf-invalid 25 + bg-clip-invalid 5
（R2 域 36＝认领口径精确兑现）。LOST/NEW 全部墙边陲/异步漂移
（animations 页 b0 timeout 形、color-mix b0 fail 漂移）ΔP 中性。

P2F 零容忍分诊：

- css-shapes circle-ellipse-serialization 4 例（子测 2/5/25/26）＝
  basic-shape `<position>` 内嵌 calc()——core face parse_clip_path 位语
  缺 math 建模，b0 透传恒 P、b1 set 拒收空串即 F。**f55b09a3e 在库即存
  在的缺口，union r1 首次暴露**；挂账 css-parser 位语修复流（21:5x NOTES
  移交笔），不入 R2 账（R2 域 P2F=0）。
- css-backgrounds background-computed 1 例＝background-position-y calc
  多值形，fv1 serialize 域漂移窗噪第六例（既有签名，不入刀账）。

css-parser 热修 A/B 旁证（v2，b1 `b3083474`＝HEAD+R2+parse_position_pair
括号感知切分，results-knife-r-union-v2/）：4 例 serialization P2F 复位 ✓
+ shape-outside 新 F2P 5，但 interpolation 新 P2F 12
（clip-path-interpolation-001 + shape-outside-interpolation 各 6——推进
冻结在 from-value，合成对含位语形失效）＝4 旧 P2F 换 12 新 P2F 净 −2，
不可按现样入库；移交热修流收窄（NOTES 22:1x 移交笔）。

## 门禁

- fmt：`cargo fmt -p zero-style-system -- --check` CLEAN（钉测短断言
  一轮 rustfmt 形修正后）。
- 定向 clippy（test-guard 1800s 包裹，`--all-targets -- -D warnings`）：
  零告警。
- style-system lib 全量：2568 passed / 0 failed（基线 2567 + 钉测
  knife_r2_expansion_gate_verdicts 腿——树上测试名，1 passed 首执验证；
  clip 百分比断言首执抓红=parse_length <length-percentage> 兼容面误放，
  已补显式 Percentage 拒收门后复绿——钉测 DOA 纪律反例：首执抓红=
  验证有效）。三腿均在最终组合树（R2 face + css-parser 热修 WIP 在树
  各归各提交面）复跑定谳；21:2x 首 clippy 腿与电池 checkout 窗 ~10s
  重叠歧义作废重跑。

## 残面与边界

- transition-behavior 23F：transition 简写 canonicalization 域（allow-
  discrete 重排序序列化），R2 不含——归 shorthand 池刀。
- mask-valid.sub 36F：mask 简写域，归 mask 简写池刀（21:14 段分类同意）。
- linear() 停点 `&&` 两序文法收窄为语料锚定序（FIXME 在臂注释）；steps()
  内嵌 math 文法细裁挂账。
- clip/clip-rule computed canonicalization 缺口（computed 页 0P/4F、
  0P/2F）不在本刀写面门域。
- R2 域 computed 残面：ttf-computed 7F + bg-clip-computed 2F（computed
  canonicalization 层，电池实测持平零位移）。
- bg-clip-valid 镜像序 `text border-area` 1F：臂两序均接受仍 F＝失败在
  序列化/computed 层非写面门（电池实测零翻转；早版 +1 预期不转正）。

## 执行位让位与事实修正（21:2x–21:3x 窗）

上表"随刀 R 提交搭车入库的 inert 扩面臂"表述已被树上事实推翻：刀 R
face `f55b09a3e` 仅含 clip-path 门；四扩面臂 + linear_stops_valid + 钉测
`knife_r2_expansion_gate_verdicts` + 白名单 60→64 均为兄弟 CLAIM 流
fork 21:2x 窗落 **工作树**（未入库），union 电池 b1 由工作树卷入构建
（b1 md5 `38d4b216bf`）。本流恢复件 knife-r2-face-wip.patch 自此作废
不 apply（树上面已含等价 % 守卫，重复 apply 会双加臂）。

## b0 探针 DOA 预核（本流独立证据，21:24–21:30）

逐值对照 probe-r2-b0 三语料 JSON 与钉测草案/树面钉测：

- css-masking：clip-invalid 4F（none/三参 rect/百分比项/混合分隔）逐形
  过臂拒收集；clip-rule-invalid 2F（auto/1）同；valid 页 3P+2P 全在接受
  集；clip-computed 4F 为 computed 序列化面，接受集不变不翻。
- css-transitions：ttf-invalid 25F 逐形核（cubic-bezier 域/arity/infinite
  字面/steps 整数与 jump-none 门/病态分隔/列表含 css-wide keyword）全落
  拒面；ttf-valid 22P（linear() 停点五形/steps 八形/sibling-index() 放行/
  逗号列表）与 computed 15P 全在接受集。
- css-backgrounds：bg-clip-invalid 5F（fill/margin/stroke-box/view-box/
  border）齐；valid 8P 全过。

零 DOA、零 P2F。账面修正一笔：background-clip-valid `text border-area`
（镜像序形）当前 F——臂两序均接受 → 预期 F2P +1，R2 扩面预期账
**+37F**（ttf 25 + clip 4 + clip-rule 2 + bg-clip-invalid 5 + 镜像序 1）。

## union 电池地面真值与预判修正（21:55–21:57 旁证复账）

五语料 union 电池 BATTERY-RC=0（21:55:18），JSON 独立复账：
F2P +160 / P2F -5 / flip 5（全蒸发向）/ 净 +155。分账 core face
clip-path 家族 ≈ +124（clip-path-invalid 48 + shape-parsing 20 +
shapes 四函数页 56）；**R2 扩面切片 = +36F 零 P2F**（ttf 25 + clip 4 +
clip-rule 2 + bg-clip 5）。

上文 +37F 预判被电池否决一笔：background-clip-valid `text border-area`
镜像序形 b0/b1 均 F——门两序均接受但回读序列化不等，翻转需 store 端
serialization canonical 配合（归序列化面残池），非门层可收。电池事实
为准，v1 认领口径 +36F 兑现。

-5 P2F 分解：shapes `basic-shape-circle-ellipse-serialization` 4 形
＝calc() 嵌 shape position/radius 槽（circle/ellipse at right calc(10%
+ 5px) 等）——core face parse_clip_path 无 calc 位置支持，setProperty
拒收回读空串；修法约束核清（三 invalid 语料页零 calc 形，放行零回翻）。
`background-computed` background-position-y calc 列表形 -1＝fv1
serialize 域漂移窗噪家族第 5 例（master.md 既有归档）。flip 5 案
（transitions color-mix 2 + animations 3）子测试蒸发向，归 TD1 收账方
复现。全细节见仲裁 NOTES 21:55/21:57 两节。
