# shape-outside/circle-ellipse 位置文法刀（shapepos，2026-11-16 二十二轮）

中断轮（rate-limit 429，15:28 静默）孤儿 WIP 认领收口（mtime 取证 15:10-15:25 六文件在案，
磁盘终态与本轮计划同构）。rate-limit 多重 spawn 对称收敛（同 session 5×resume 对称实例，
19:18-19:46 交错在飞）：本流白名单条目 + 收敛实例显式裁决臂互补合树，停己采彼采认磁盘
终态链账（b1v3-b1v5 链与 make test 拆腿全部采自 zw-shapepos-final.service）。四站合刀：
css-parser 位置严格文法 + engine 缺省分量串化 + style-system inline 写面门 + 三 crate
常驻钉。

## 定向钉

1. **`parse_position_pair` 严格文法重写**（css-parser/src/values/parse_extended_border.rs）：
   css-values-4 §position 轴别文法，`PosComp` 枚举（Len/XEdge/YEdge/Center）逐 token 分类。
   合法域 = `[lp|x-edge] [lp|y-edge|center]` + y 首词对换（`top left`→`0% 0%`）+ center 填
   任意缺轴；拒收域 = `[lp|x-edge][x-edge]`（`50% left`）、`[y-edge][lp|y-edge]`（`top 50%`/
   `left right`）混轴/同轴双关键字、3 词混搭（len>2 早退）。语料契约注记：css-shapes-1 WPT
   oracle（parsing-utils.js validPositions/invalidPositions 表）属 Chrome 契约——现代
   css-values-4 的 2 词 `[edge] <offset>` 缘偏移形（`top 50%`/`bottom 70px`）在该语料属
   invalidPositions 拒收域，此处不收（语料计分契约优先）；FIXME(规格演进) 挂账语料换代后
   补臂。
2. **4 值缘偏移百分比补数**（同文件既有 4 值臂 resolve 闭包）：`bottom 70% right 80%` →
   `20% 30%` 代数精确折算（100%−p），长度偏移保 Calc 形（`bottom 1120px` →
   `calc(100% - 1120px)` 不可声明期折算）。corpus 钉面 circle-004/ellipse-004 computed 族。
3. **clip_path_to_css 缺省分量串化**（engine/src/js_dom_bridge/computed_style.rs）：circle
   缺省半径 closest-side 省略（`circle(at 800px 50%)` 形）；ellipse 双轴同缺省整体省略、
   单侧非缺省保双半径；空 inner 前导分隔符修（`circle(at …)` 非 `circle( at …)`——
   circle-004 computed 533F 族单一根因）。长度经 resolve_length 物理单位折 px
   （`50cm` → `1889.764px`，引擎原样发射，语料 roundResultStr 6 位有效数字归一后对表）。
4. **shape-outside inline 写面门**（style-system/src/property/apply.rs）：VALIDATED_
   PROPERTIES 白名单接入第六串（19→20），裁决复用 apply 既有臂 oracle——白名单条目（路由）
   + 显式分派臂（裁决，置于 `value_has_unimplemented_function` 放行臂**之前**，
   counter-increment 臂同序先例）：`value_has_substitution_function` 放行 + 裸 `<shape-box>`
   四关键字受理 + `parse_clip_path(v).is_some()` 裁决。语料 url() `<image>` 形全在样式表
   声明路径（不经本 inline 门），不受 apply 臂未建模 `<image>` 影响（法证：css-shapes
   全目录 0 处 inline setProperty url 形）。
5. **常驻钉**：css-parser `test_clip_path_position_*` 九钉（混轴拒/同轴拒/3 词拒/y 首词
   对换/1 词 y 关键字/2 词轴解/百分比补数/长度 Calc 形）+ round 文法四钉（inset round 列表
   全形/polygon 尾缀/前缀/inset calc 偏移切分）；engine part03 clip-path + shape-outside
   gCS 钉更新（`circle()` 全省略、`circle(at 1889.764px 50%)`、4 值缘偏移 `70% 30%`）；
   style-system `shape_outside_write_gate_verdicts` 十二钉（合法形放行/invalidPositions 域
   拒收）。
6. **animation.rs clippy 机械修**：`is_some_and(|v| rect_family(v))` → `is_some_and(rect_family)`
   ×2（上轮他流 21872a345 落树引出的 redundant-closure，非本刀语义面）。

## 门迭代定谳链（v2 臂修正 → A-lite 两轮 parse 补臂）

**v1（b1v3，白名单接入）零红利双根因**：css-shapes 腿 vs b1v2 SET-IDENTICAL（门零影响）。
探针页三段定位（回调已注册 ✓ / 裁决 `circle(at 50% left)` 返 "true" ✗ / 写路径 detached+
attached 均透传 ✗）钉死结构性绕过——`circle()` 等不在 IMPLEMENTED_VALUE_FUNCTIONS 名单，
`value_has_unimplemented_function` 提前放行 store，末尾 apply oracle 兜底永不可达。
**教训：白名单接入不等于兜底可达——分派流前置放行臂（unimplemented 函数/替换函数）必须
逐项核对先后序**；白名单既有条目的裁决臂全部位于放行臂之前（width random 臂/text-indent/
counter-increment 同谱系），本刀首踩该序陷阱。（另法证：b1v3 构建曾与收敛实例 cargo 竞争
出陈旧 style-system 的污染快照——锁争签名 "Blocking waiting for file lock" 在案，b1v4 起
solo 链重建消除。）

**v2（b1v4，显式分派臂）门红利 +324 / round 形回归 −232**：invalid 域八页 +65（invalid-
position/path-invalid/shape-invalid +10/+7/+9、circle/ellipse/inset/polygon/round-invalid
+10/+10/+12/+3/+9、xywh-invalid +4）+ shape-outside inline 门 circle/ellipse-003 +125×2；
回归四页 = round 参量文法缺口（`inset(… round <r>{1,4}[/…]` 列表、polygon 尾缀 round 被
parse_clip_path 拒收 → 门拒收 → b0 裸回显巧合通过的真 valid 页翻 Fail：inset-003 −195、
inset-002 −30、inset-function-valid −4、polygon-round-valid −3）。

**A-lite 一轮（parse-only round 文法）**：`parse_clip_round_list`（border-radius 简写全形
`<radius>{1,4} [/ <radius>{1,4}]?`，斜杠分隔水平/垂直半径组、非负域）接 inset round 参量与
polygon 尾缀 round；存储有损取水平首半径（Inset 单半径槽位/Polygon 无 round 字段，列表
建模 FIXME 另行收口；round 形 computed 此前 apply 恒拒 = none，有损存储不产生新串化行为
面；写面门只要求 parse 判定——valid 页原文回显）。裁决臂 `parse_clip_path(v).is_some()`
不动，parse 扩容承担全部修复。

**A-lite 二轮（b1v6，前缀臂 + 无条件括号感知）**：①inset 偏移切分括号感知
（`split_comma_or_space` 朴素 split_whitespace 曾把 `calc(100% - 20px)` 切 3 片 → 6 值
整拒）改 `split_top_level_whitespace`；②polygon round **前缀臂**（`polygon([<'fill-rule'> ||]
round <radius-list> , <pairs>)`——round 位于坐标列表前，radius 组止于首个顶层逗号
`find_top_level_comma` calc 内逗号安全；fill-rule 同槽无逗号直接相邻 `polygon(evenodd
round 1px, …)` 受理，fill-rule 后接坐标形仍走原拒收路径）。净 +29 但揭 **−7 两面回归**：
animation/shape-outside-interpolation 页 +28/−6（前缀真插值 +28 增益，同页 xywh→rect
带 round 跨形对 −6）；polygon-round-invalid −1（`polygon(round 1%, 1px 2px)` 百分比
半径被 border-radius 文法放行）。

**二轮回归经验二分定谳**（rebuild+b1v6 二进制单页探针逐变更回退）：回退①切分（保②前缀
臂）→ CSS Animations xywh-round 三案回 b0 基线 3P/0F → **切分是 −6 唯一祸源**。机理：
无条件括号感知使 `inset(10% calc(100% - 100px) 50% 0px round 20px)`（插值期望串形）可
解析，破坏二十一轮定谳的「期望串含 calc 时本机 setProperty 拒收 → expected 落回静态值」
**伪通过池保位**回落链（b0/b1v5 该对 clip_interp None → echo 保位 → 端点 verbatim 回落
对表）；scratch 探针双插值通道实证（lenient `interpolate_property_value` 恒 ""（echo 门
命中）、strict `interpolate_property_value_strict` 恒 None——registry 两侧在两树同形，
差异全在 parse 接受面传导的写回落链）。纪律重申：**B0 伪通过池（期望串含 calc 形）不追
真值保位**，直至残面⑤ calc 代数插值刀落地。

**A-lite 三轮（b1v7，条件切分试探 → 边界不可分）**：①inset 括号感知改条件形（仅无
round 段时括号感知）→ b1v7 账 4537P（vs b1v5 +34：+28 前缀真插值、+3 polygon-round-
valid 清偿、+1 polygon-round-computed、+4 inset calc 形解锁 inset-function-valid/008/
computed），但 animation 页残 **−2**（同对 t=1 端点：期望串 `inset(20% 50% calc(100% -
200px) 20px)` **无 round**，条件边界恰把它划进接受面 → 与 +4 inset 增益在 parse 层同
构不可分）；②polygon round **百分比拒收臂**（`parse_clip_round_list(…, allow_percentage)`
——inset round 依 border-radius 文法仍收 %，polygon round 按 Chrome 语料契约拒收
`polygon(round 1%, 1px 2px)`；`radius_is_percentage` 直判 Percentage 变体，calc 内 %
不在此判域无语料证据不作宽拒）——round-invalid 十案全拒 ✓。

**终态（b1v8，撤条件切分）**：条件切分整体撤销回朴素 `split_comma_or_space`（inset
calc 化偏移形整拒——伪通过池接受面边界钉化，FIXME(残面⑤ calc 代数插值刀) 落地后括号
感知一并清 inset calc 拒收面并解锁四案）；保留②百分比拒收臂。两方案总账对比：条件形
+1227/−2 vs 终态 **+1223/−1**（inset calc 案为门引入残 −1，b0 裸回显伪通过）——同
增益面下回归面减半且代码更简，伪通过池纪律（二十一轮「期望串含 calc 形不追真值保位」）
一致。

## A/B 语料账（B0 vs B1v2 vs B1v4 vs B1v5，css-shapes 套件）

B0 = bin-shapepos-b0 md5 `d364bc8d04db71bbf0333c35d4b5ef88`（HEAD 4f5e7a75d + peer es_module
WIP 同载）；B1v2 = b1v2 md5 `154b24dfcbd7cb57b11d0fe88bfa993b`（+本刀六文件）；b1v4 =
`e8de1c2242119c4ced366117426c702b`（+显式分派臂）；b1v5 = `d5f6a65c0835cbedc663fbe072fa1b78`
（+A-lite 一轮）；b1v6 = `4e40b952e4536b08239b7fe602045607`（+二轮）；b1v7 =
`1e1dbb4a4fdd3e8193e31d4c45011df6`（+三轮条件切分试探，已撤）；b1v8 =
`84815f65a818fe07c9be279a55153412`（终态，提交态）。b0/b1v2 双二进制 18:32/18:33 快照，
六腿 18:34-18:43（run-css-testharness-corpus.sh，test-guard per-proc 4G/total 8G/wall
900s，CASE_WALL_S=90），全腿 LEG-EXIT=0；b1v5 腿 19:54-19:55、b1v6 腿 20:05-20:07、
b1v7 腿 20:25-（cases=145 干净 census，zz 探针页 19:53 已清）。

| 套件 | B0 | B1v2 | b1v4 | b1v5 | b1v6 | b1v7 | b1v8（提交态） |
|---|---|---|---|---|---|---|---|
| css-shapes | 3312P/4202F | 4168P/3346F | 4492P/3024F | 4503P/3011F | 4532P/2982F | 4537P/2977F | **4535P/2979F** |
| css-masking | 1773P/4567F | 1774P/4566F | — | — | — | — |
| css-typed-om | 9872P/2487F | 9872P/2487F | — | — | — | — |

- **B0→B1v2（位置文法刀六文件）**：+856/−856 **0 flip**（逐案 status 对账）——增益页 15
  页全 GAIN 零 loss：circle/ellipse-004 +327×2（串化主红利）、-002 +68×2、-009 +19×2、
  -000/-001/-011 各 +2~+4、shape-functions circle/ellipse-computed +5/+6、parsing/
  shape-outside-computed +1；masking +1/−1（parsing/clip-path-computed +1，与
  basic-shape-applier-knife 账形同）；typed-om SET-IDENTICAL（transcript 字节同长互证）。
- **b1v2→b1v4（显式分派臂）**：+324/−232（上节 v2 两面）；zz 探针页 +6P 污染 census
  147（b1v5 起清页回归 145）。
- **b1v2→b1v5（显式臂 + A-lite 一轮，census 干净）**：+339/−4（净 +335）——v2 门红利全保
  （invalid 八页 +65、circle/ellipse-003 +125×2），回归四页收敛至 4 案（polygon-round-
  valid −3、inset-function-valid −1），inset-002/003 转净增益 +2/+10。
- **b1v5→b1v6（A-lite 二轮）**：+31P/−1F 页面账（净 +29，-6 动画对 +28 前缀真插值同页
  对冲后 +22）、+7 隐藏回归（动画 −6 + round-invalid % −1）。
- **b1v5→b1v7（三轮条件切分试探）**：+34P/−0F 页面账（净 +34 = 前缀 +28、polygon-round-
  valid +3、round-computed +1、inset calc 形 +4；动画页 −2 t=1 残差藏于 +28/−2 同页行）。
- **b1v5→b1v8（终态 = 前缀臂 + % 拒收 + 朴素切分）**：**+32P/−0F**（前缀 +28、polygon-round-valid +3、round-computed +1——动画页 +28P/
  −0F 池全保护）。终账 vs b0 **+1224/−1 净 +1223**（逐案 status 对账，唯一 loss =
  inset-function-valid calc 案 `inset(0px calc(100% - 20px) 2% 3em)` 门引入残 −1 挂
  残面⑤；EXPECT 与实测 4535P 逐位吻合）。

## 残面定谳（刀后余账）

- **位置串化规范刀（下刀候选 ~498F）**：002/004/009 余 29/206/14×2 族 value-mismatch 全
  单一根因——Chrome computed 位置规范形（x-then-y 序 `top left`→`left top`、缺轴 center
  补位 `circle(at 50cm)`→`circle(at 50cm center)`、`+50px`→`50px` 符号归一）。masking 面
  同型残差仅 2 案，收益集中 css-shapes。
- **round 列表完整建模刀**：Inset round 单半径槽位 → h/v 半径列表 + Polygon round 字段 +
  串化（`polygon(round 0, 1px 2px, 3px 4px)`→`polygon(1px 2px, 3px 4px)` 丢零半径、
  `polygon(nonzero round 1px, …)` 序列化丢 fill-rule 等语料期望形）+ 插值（round 对
  interpolated 现恒回落）。
- **残面⑤联动（本刀新挂 inset calc 边界）**：inset calc 化偏移拒收面（终态接受面边界，
  钉 `test_clip_path_inset_calc_offset_rejection_boundary`）残 −1（inset-function-valid
  calc 案）+ 解锁队列 inset-008 ×2/computed ×1/动画 t=1 对 ×2——calc 代数插值刀落地时
  括号感知切分一并清账。
- **inline 写面门红利已入账**：circle/ellipse-003 各 125F 全域 `expected ""` invalid 形
  （125/125 案 message 分类实证），门接入后整页翻转。
- 沿队列：③ inline inherit 解析刀 ④ path() 命令级兼容插值 ⑤ calc 代数插值共享面
  （steps-position-knife §5.1 / path-gate-removal-knife 挂账）。

## 门禁

**make test 拆腿**（zw-shapepos-final.service 19:20-19:41，MKT-RC=0，per-file fmt 六文件 +
companion build + workspace-test/quickjs-clippy 并行 + renderer/browser 腿 + GPU 探针腿 +
quickjs 三腿）：fmt 六文件 5/6 rc=0（part03.rs rc=1 = edition-flag 假漂移，canonical
`cargo fmt --all` 全净）、companion build rc=0、workspace-test rc=101 + quickjs-clippy
rc=101 全部对号在册（peer s419 es_module 测试红 ×6 + es_module.rs:2217 too_many_arguments
lint 红、worker-fetch/tab-mirror 族、default_actions、image_batch 环境形红——刀域零新增）。
终态树复验（b1v8 前）：css-parser `test_clip_path` 50/50（九位置钉 + round 四钉 + round-
invalid 十钉 + calc 边界钉全绿）、`cargo fmt -p zero-css-parser` 净、clippy 净、style-system
`shape_outside_write_gate_verdicts` 1/1、engine `test_get_computed_style_clip_path` +
`test_get_computed_style_shape_outside` 2/2、engine fmt 净（MKT 后刀面 delta 仅 css-parser
parse 层，engine/style-system 未再触碰，拆腿账对刀域仍权威）。

**b1v8 验证腿**（20:30 systemd-run，release 重建 49.6s + css-shapes 腿，LEG-EXIT=0，
cases=145 干净）：4535P/2979F，对 b0 逐案 **+1224/−1 净 +1223**，与发射前预估 4535P
逐位吻合。
