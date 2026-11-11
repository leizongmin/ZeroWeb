# DC-2 g13：serialize canonical 折叠刀2 + 通用色通道 cq/sign 面 + round B=±∞ 策略表

日期：2026-11-12。基线：HEAD `12ece659e`（g12b 五套件入账后）。
本批三提交：`dcb81aad7`（css-parser serialize 刀2 + clamp NaN 毒化）、
`cc22f08e1`（engine part11 通用色通道）、`3351e0577`（round B=±∞ 策略表，
见 §3）。

前轮双 session（knife2 dev + e1 姊妹）在同 5 小时 rate-limit 窗口双双中断，
工作树遗留两批完整 WIP hunks；本 session 按认领笔记（.acceptance/wptfyi-css/notes-e1-color-claim.md，本地未跟踪）
分工分别验证、分开提交（两批 pathspec 互不卷入），并在提交态 HEAD 统一
重建二进制重录权威账（results-g13，消除前轮 knife2 values 轮与 e1c 重建
同 target 竞争的 provenance 疑点——g13 values/color 数字与前轮逐字节一致）。

## 1. serialize 刀2（css-parser，dcb81aad7）

勘根笔记：.acceptance/wptfyi-css/notes-g12-knife2-rooting.md（本地未跟踪）。
css-values-4 §10.11 specified 常量折叠从 Number/Angle 扩 Length/Time：

- `MathFoldKind::Length/Time`：width 与 transition/animation duration/delay
  四属性入 `fold_decl_specified` 白名单；Time 槽经 `parse_time_calc` 重
  parse（共享 calc 入口拒 time 原子），plain 指定值 `1s` 不折叠。
- 非有限 canonical 发射（§10.6）：`calc(NaN|±infinity * 1<unit>)` 因子归一，
  单位取树首维度叶（物理单位 parse 期已归一 px、ms 归一 s）；min/max 含
  NaN 塌缩 `calc(...)`；clamp 任一实参 NaN 且实参 canonical 单位分歧时
  结构外提逐 arg 规范化（`clamp(NaN * 1em, NaN * 1px, NaN * 1px)` 形）。
- 两级求值仅对非有限值穿透：无 ctx → 默认字体 16px（em/rem NaN/∞ 与字体
  无关）→ parent=0（% 式）；有限 em/% 式布局依赖不折叠（Chrome 同口径）。
- angle 位 ±∞ 由保守拒折改 canonical 放行（§10.10 无穷合法非 invalid）。
- 连带修 `eval_calc_with_context` clamp 臂 NaN 丢毒：`f64::min`/`max` 链式
  折叠忽略 NaN，`clamp(NaN, 0, 10)` 曾误产 0（§10.6 任一实参 NaN → NaN，
  先行毒化短路；min>max 退化与不 panic 语义保留，R3344 在册）。

门禁：css-parser lib 3117→3118 全绿（`infinity_nan.rs` +3 案：clamp 毒化、
serialize canonical 全形态表、round ∞ 表）；style-system lib 2516 全绿；
clippy `--all-targets -D warnings` 零告警。

## 2. 通用色通道 cq/sign 面（engine，cc22f08e1）

勘根：master.md 挂账「calc-container 2F（part11-computed-color 通用通道
prop=background-color 不经本入口）」+ color-mix `sign(100em - 1px)` 语料形。

- part11-color 导出 `__zwContainerInlineSize`（cqw 容器基座批既有的最近
  container-type ≠ normal 祖先探针）；`__zwComputedColor` 入口按 el 置入
  `_zwCCqwBaseG`，resolve 后 finally 清场。
- sign eval 词法器补 cq 水平轴（cqw/cqi 按 N·W/100，cqh FIXME → NaN
  fail-closed）与 em/px 叶（em 16px 与 part11-color evalAtoms 同口径）。
- 通道解析器补 `sign()`/`abs()` 函数臂（§sign-related：任意型 → number，
  sign(NaN)=NaN、sign(±0)=0）；前置外送门撤除（解析失败仍回退基座保底）。
- color-mix lead/trail calc 权重臂改括号平衡扫描认嵌套形
  （`red calc(50% + (sign(100em - 1px) * 10%))`）；contrast-color 声明面接
  `color(calc 槽)` 严格形内嵌 calc 规范化 + α 前导点补 0（§10.9 canonical
  order 数前重排由既有 termText 落）。

门禁：css-color 语料 10,970 → **10,972 净 +2 零回归**（对 results-g10
逐页 diff：color-computed-color-mix-function +1、
contrast-color-function-calc-container +1，余 64 页逐位一致）。
姊妹 session 中间轮记档：e1 首轮 +0（编辑中途）、e1b 二轮 -7（迭代
中间态，sign/abs 门切换半成品）、e1c 三轮 +2 = 终态（本批权威）。

## 3. round 族 B=±∞ 策略表（css-parser 回归修复，本 session）

clamp NaN 毒化门落地后 g13 语料捕获 **signed-zero -5 回归**
（`clamp(-1, 1/sign(round(nearest|up|down, 0|1, infinity)), 1)` 五行
P→F）。勘根：round 族 IEEE 直算 `(av/bv).round()*bv` 在 `0×∞` 处产 NaN
丢语义，而基线由 clamp 的 `NaN.min(max_v)` IEEE 洗毒（`f64::min` 忽略
NaN 返回另一参）意外掩盖——毒化门按 §10.6 正确传播后缺口暴露
（master.md 在册 FIXME types.rs:1420「round step ±∞ 族」兑现）。

修复按 round-function/signed-zero 语料钉面实现策略表（§round-func）：
B=±∞（A 有限）→ nearest/to-zero 给 ±0 签号随 A；up 正 A 给 +∞、余 ±0；
down 负 A 给 −∞、余 ±0；B 符号无关（`round(5,±∞)` 同 +0）；A=±∞ 有限步
→ A 原样（IEEE 路径既有）；A=±∞ 且 B=±∞ → NaN（既有）。爆炸半径仅
css-values（其余五套件语料零 `round(...∞)` 形，grep 定谳）。

单测钉 `test_round_infinite_step_strategy_table`（round-function 16 行 +
signed-zero clamp 端到端 8 形）；css-parser lib 3118 全绿、clippy 零告警。

## 4. 权威账（results-g13，HEAD cc22f08e1 单二进制，systemd unit
zw-g13-batch-0345，test-guard --per-proc-mem 4 --total-mem 8 --time-limit 3540，
transcript 唯一目录）

- **css-values（对 results-g12b 批前基线）**：2,646 → **2,787 净 +141，
  页级与 test 级零回归**（9 页增益 +141/0 losses）——g13 前驱轮（round 修复
  前）2,747 净 +101、signed-zero -5（§3 缺口暴露面）；g13b 复核轮（round
  策略表落地，results-g13b）对 g13 **净 +40 零回归**：signed-zero 全页
  翻绿 +20（-5 收复 + sign ±0 行 15 翻正——NaN 洗毒时代 A=−1/−0 的
  sign 行本就失败，策略表后 ±0 签号直达）、round-function +16
  （nearest/to-zero/up/down ∞ 步长行）、round-mod-rem-computed +4
  （`round(up, 1, ±Infinity)` → `calc(±Infinity)`）。增益明细：
  calc-infinity-nan-serialize-length +39（width Length 槽 canonical）、
  serialize-time +29（四属性 Time 槽）、calc-catch-divide-by-0 +21
  （÷0 → ±∞/NaN canonical）、round-function +16、signed-zero +15、
  minmax-time-serialize +11（`min(1ms)` → `calc(0.001s)` ms 归一）、
  round-mod-rem-computed +4、serialize-angle +3 与 serialize-number +3
  （clamp 毒化翻正——基线 `NaN.min(10)` IEEE 洗毒误产 10）。
- **css-color（对 results-g10 基线）**：10,970 → **10,972 净 +2 零回归**
  （§2 两页）。
- **css-transitions 对 results-g12**：200 → 200 逐页零漂移。
- **css-animations 对 results-g12**：238 → 238 逐页零漂移。
- **css-transforms 对 results-g12b**：228 → 228 逐页零漂移（angle ±∞
  canonical 与 clamp 毒化对 transforms 语料零扰动——rotate/scale 槽
  parsing 页非有限形零命中）。
- **css-easing 对 results-g12b**：51 → 51 逐页零漂移。

**批终五套件合计净 +143（values +141、color +2、余三套件 ±0），零回归。**
对账文件：results-g13/diffs-vs-baselines.txt、logs/g13b-diff-vs-g13.txt、
logs/g13b-diff-vs-g12b.txt。

## 5. 前轮中断跑对照

results-g12-knife2（前轮中断，仅 values 完成）：2,646 → 2,747 净 +101、
signed-zero -5——与 g13 values 轮逐字节一致（provenance 竞争疑点消除）。
增益 6 页 +106：serialize-length +39、serialize-time +29、calc-catch-
divide-by-0 +21、minmax-time-serialize +11、serialize-angle +3（clamp 毒化
翻正）、serialize-number +3（同）。
