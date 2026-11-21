# 刀 CALCSIZE-CANON · calc-size() 指定值 canonical 串化（2026-11-21）

## 缘起与认领

calcsize b1 探针 census（results-knife-calcsize/b1/css-values.json）分池：
calc-size-parsing 残 6F＝serialize 桶 canonical 形断言。认领 NOTES
（NOTES-knife-calcsize-canon-claim 09:10）原含 interpolate-size-computed
2F；09:1x 修订让位——MOTION-3 线在本 clone 在途面（js_dom_bridge.rs
decl_supported interpolate-size 臂 + apply.rs 包装）即该 2F 修复面，
冲突认领→本线让位（NOTES 协议）。注：MOTION-3 电池（+690）收账清单
未含 interpolate-size-computed 翻账，该 2F 仍在其域挂账。

## 机制

face = css-parser 三文件：

1. **values/serialize_calc_size.rs 新模块**（~330 行）：入口
   `canonicalize_calc_size_specified`——单一顶层 `calc-size(basis, sum)`
   形拆解（大小写不敏感前缀 + 括号配平 + 闭括号后无残余；切片锚原文，
   小写副本只作探测）；basis 关键字保形/嵌套 calc-size 递归/绝对单位
   96 DPI px 归一（`2in`→`192px`）；sum 顶层 `+` 拆项（混合 +/- 序
   `None` 保形——符号迁移语义面未钉）、% 前置稳定排序（`50px + 30%`→
   `30% + 50px`，serialize_math §10.6 同口径）、积项规范形（数字因子
   前置 `size * 2`→`2 * size`、除法收倒数乘法 `size / 2`→
   `0.5 * size`）、多顶和内积项加括号（`30px + size / 2`→
   `30px + (0.5 * size)`）。非文法形（0/2+ 顶层逗号、多因子积、除零、
   非 calc-size 跨度）返 `None` 原文透传零漂移（fold 通道契约）。函数名
   保作者形（`CALC-SIZE(` 不重裁）；整括号包裹项先剥外括号再 canonical
   （重入幂等——join 已加括号不叠层）。单测 3 函数：六形钉面/幂等与
   拒形保形/相对单位保形。
2. **values/serialize_math.rs**：`fold_decl_specified` 新十三尺寸槽臂
   （width/height/min-max 变体/block-size/inline-size 族/flex-basis，
   与写面门 [`calc_size_decl_gate`] 同槽清单）——
   `canonicalize_calc_size_specified(v).or_else(fold_math_specified
   (Length))`：非 calc-size 形回落原 math fold，与旧臂行为逐字节恒等
   （旧 `width|…|height` 臂同 fallback，零漂移）。`max-width` 从后侧
   anchor 臂上移该臂（anchor-size 2F 同经 `.or_else` 兜底，行为不变）；
   新增 `canonical_number_text` 对外口径（format_number_6dp 单源复用）。
3. **values/mod.rs**：模块注册 + re-export。

通道实证：serialize 桶 `e.style[...]` 读路径走 `fold_decl_specified`
重放折叠（callbacks.rs sel 臂，sin-cos-tan-serialize 270P/0F 同通道
b1 基线在案）——css-parser 单点即达，无需 engine 接线（前稿 shim 钩子
假说不需要）。canonical 形回写经写面门重验安全（门以 parse_math_function
代裁同文法，`size` 叶替换后语法等价——GRAMMAR 刀已证）。

`#[used]` face marker `CALCSIZE canon-serialize`（代码字面 ASCII，
serialize_calc_size.rs，MATHFN/CALCSIZE grammar-gate 先例）。

## 电池（unit zw-knife-calcsize-canon-0916）

09:16 发射，09:24:22 GATE-DONE rc=0（wall ~8 分钟）。b0 = HEAD＋ride
剥 face（serialize_calc_size.rs 为 untracked 新文件——patch 外存在，
b0 剥 mod.rs 声明后成死文件不参与编译，FM b0=0 兑现）；b1 = HEAD＋
ride＋face；M2 双 bin md5 异（b1 `9edba8818ef2af32ef53dae0242af9e3`）；
FM b0=0/b1=1；RIDE 8 文件（motion3 WIP 三件＋CHMETRIC wpt-runner 三件
＋es_module.rs＋SKILL.md，对端在途），S1/S3 两窗 md5 零扰动；TRAP-NOOP
state=done（face 回树=发射态）。

发射 swarm 实录：本 session 冻结窗（motion3 电池在飞，共享树 trap
互踩协议禁双跑）内由 sibling fork 按 NOTES 协议发射；本 fork 09:20/
09:21 双发尝试被 flock 拦截（BLOCKED 双证），三发射一活两拒——
resume-swarm 收敛零双写。

产物：`.acceptance/wptfyi-css/results-knife-calcsize-canon/`（bin/b0/b1
＋ride-files.txt＋s1-md5.txt；日志 knife-calcsize-canon-battery-0916.log）。

## 结果（css-values b0→b1）

| 页 | b0 | b1 | Δ |
|---|---|---|---|
| calc-size-parsing | 88/94 | 94/94 | +6 |

净 **+6（F2P 6 / P2F 0）**，孤页恰集；全其余 values 页零位移（无
ASYM 行＝双臂页级断言数对称）。b0 6 形（`size * 2`/`size / 2`/
`30px + size / 2`/`50% + size / 2`/`50px + 30%`/嵌套 `2in`）全数翻账。

预期兑现核对：认领预期 calc-size-parsing 88→94（+6）——**全数兑现，
无偏差，无外溢**。

## 域界与挂账

- interpolate-size-computed 2F——MOTION-3 线域（本刀让位，其电池未翻
  仍挂账该线）；
- calc-size-typed-om 19F——part10-typed-om.js 域，OM-13 电池在
  ZeroWeb-3 clone 在飞（09:03 发射）；
- CALC-SIZE-RESOLVED ~112F（height 23/width 13/box-sizing 26/flex-basis
  50，used-value/gCS 深刀）——CANON 收账后另开认领；
- animation 插值臂 ~2563F——MOTION-3 线（其 Slice 1 +690 已收，
  余面续刀）。

## ride 说明

motion3 WIP 三件（apply.rs/js_dom_bridge.rs/part02.rs）随本刀电池双臂
同载（ride 相消不入本刀账），battery 窗内由其流提交 9f9bc0f2d
（+690）——本刀 b0/b1 与该 HEAD 内容一致。CHMETRIC wpt-runner 三件
＋es_module.rs＋SKILL.md 仍在途未提交，不入本刀 pathspec。
