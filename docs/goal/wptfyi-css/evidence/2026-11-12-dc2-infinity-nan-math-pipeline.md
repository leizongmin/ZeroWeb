# DC-2 g12：∞/NaN 数学求值管线刀1（§10.6 min/max NaN 毒化 + §10.12 computed 期 censor）

日期：2026-11-12
基线：HEAD `b94c97bc3`（g11 time 族写面文法门批入账后）。

## 勘根（刀1 范围定谳）

calc-infinity-nan-computed.html 56F 全分类后，本批兑现可恢复桶（deferred：
rotate/matrix 14F → transform matrix 化在册桶；%-CB 形 → % diverge 桶；
serialize 五页 123F → 刀2）：

1. **min/max NaN 毒化丢失**（types.rs eval arms）：IEEE `f64::min`/`max` 忽略
   NaN——`max(15px, calc(NaN * 1px))` 误产 15px，下游 censor 不触发。§10.6
   规定任一实参 NaN → 结果 NaN。
2. **length 求值 NaN 直通**（style-system resolve_length_inner Calc arm）：
   `eval_calc_with_context().unwrap_or(0.0)` 对 NaN 直通 → 序列化 "NaNpx"；
   §10.12 computed 期 NaN censor 为 0（±∞ 保留，序列化饱和覆盖 ≥1e6 断言）。
3. **time 族数学形写面误拒**：`parse_time` math 臂 `v.is_finite()` 门与
   `parse_animation_duration` 纯 plain 形（g11 挂账）把 NaN/±∞ 合法形当
   invalid 拒——§10.10 求值 NaN/±∞ 非 invalid，censor 属 computed 期。
   连带：`calc_expr_is_pure_time` 拒标量乘除（`calc(NaN * 1s)` =
   Multiply(Number, Time) 被 purity 门拒）——§10.1 型检查
   `<time> * <number>` / `<time> / <number>` 合法，标量侧按 §10.5 型别表判
   <number>（`sign(1em - 16px)` → number 放行）；em 求值需 parse 期默认字体
   基准（新 `eval_calc_with_default_font`：16px；Chromium parse 期不持字体，
   计算期真实字体重解析语义差异在册）。
4. **`<number>` 槽 ±∞ 误拒**（parse_finite_number_math）：`!is_infinite` 门
   拒 `scale: calc(infinity)`；§10.12 ∞ → 属性范围钳属 computed 期。连带
   该门 `calc_expr_has_length` 叶扫描误杀 `sign(<length>)` 合法形——改走
   `calc_expr_is_number` 结果型别门（§10.5：sign 恒 <number>）。
5. **engine time 序列化缺 censor**（computed_style.rs time_list_to_css）：
   duration 族 `[0s,∞)` 负值/NaN 直通 format_num（NaN→"NaNs"）。

## 改动面

- `crates/css-parser/src/values/types.rs`：Min/Max eval arms NaN 毒化；
  `calc_expr_is_pure_time` BinaryOp 乘除臂标量型别门；新增
  `eval_calc_with_default_font`（parse 期 em/rem 16px 基准求值，%/vw/vh 仍
  fail-closed）。
- `crates/css-parser/src/values/parse_transform.rs`：`parse_time` math 臂撤
  `is_finite` 门（NaN/±∞ 放行，None IACVT 仍拒）；`parse_animation_duration`
  数学形臂（纯 Time 叶门 + 默认字体求值；plain 门与 `infinite` 关键字拒收
  不动——animation-duration-invalid 语料钉）。
- `crates/css-parser/src/values/transform_helpers.rs`：
  `parse_finite_number_math` ±∞ 放行 + has_length 叶扫描换 `calc_expr_is_number`
  结果型别门（iteration-count 自门 `is_finite() && >= 0` 不受扰）。
- `crates/style-system/src/computed.rs`：resolve_length_inner Calc arm
  NaN→0 censor（±∞ 保留）。
- `crates/engine/src/js_dom_bridge/computed_style.rs`：`time_list_to_css`
  增加 `non_negative` 参数（duration 族负值钳 0；两族 NaN 钳 0；±∞ 保留）。
- 钉：css-parser `values/tests/infinity_nan.rs` 新文件 4 案（mod.rs 注册）；
  style-system `test_resolve_length_calc_nan_censors_to_zero`；engine
  `time_list_censors_nan_and_negative_duration_at_computed`。

## 门禁

- css-parser lib：3,115P/0F（+4）；style-system lib：2,516P/0F（+1）；engine
  lib time_list 案 P（3,892 filtered 中 1P）；clippy 三 crate 零告警。
- 语料五套件（systemd unit `zw-g12-corpus-run`，test-guard
  `--per-proc-mem 4 --total-mem 8 --time-limit 3540`，transcript/results
  唯一目录 results-g12，全五套件 exit=0）：
  - **css-values（主账，对 results-g11b 前驱基线）**：2,572 → 2,641
    **净 +69，页级与 test 级零回归**（test gains=69 losses=0），10 页增：
    calc-infinity-nan-computed 9→29（+20，NaN censor/max-min 毒化/duration
    NaN·∞/delay sign±∞ 形全翻）、round-mod-rem-computed 205→221（+16，
    round ±Infinity 求值放行）、signs-abs-computed 132→139（+7，sign em/rem
    形）、calc-infinity-nan-serialize-angle 14→19（+5）、minmax-time-computed
    19→24（+5，min/max 标量乘除）、signs-abs-serialize 12→16（+4）、
    round-function 169→173（+4）、calc-time-values 6→10（+4，int×time/
    time÷int）、calc-infinity-nan-serialize-number 23→25（+2）、
    exp-log-compute 13→15（+2，sign em 形）。serialize 三页随 eval/parse
    放行经既有 fold_math_specified NaN/∞ 规范化自发回账（刀2 serialize 桶
    部分自解）。
  - **css-transitions 对 g11b**：198 → 200 净 +2 零回归
    （transition-delay-computed 0→1、transition-duration-computed 2→3，
    engine time censor 直接翻账；inherit-height-transition 本轮落账 0P 新页，
    非既有 pass 流失）。
  - **css-animations 对 g11b**：236 → 238 净 +2 零回归
    （animation-delay-computed 2→3、animation-duration-computed 5→6）。
  - transforms/easing 对照（无 v3 基线）：页内断言自证 + pre-flip 二进制
    A/B 逐页 diff（results-g12pre），见下节。

## transforms/easing 对照 A/B（results-g12pre vs results-g12）

pre-flip 臂 = `zero-wpt-runner-g11pre`（01:46 独立落盘二进制）；post-flip 臂 =
results-g12（01:31 批构建二进制，五套件全部在 01:44:02 前跑完，早于 01:46
重建——全程单一二进制零污染）。逐页 diff（diff-g11-vs-v3.py）：

- **css-easing：51 → 51 字节级零差**（gains=0 losses=0）——step/linear()/bezier
  output 既有失败桶（引擎 easing 求值面，非 parse）对本批零扰动。
- **css-transforms：+3 gains / -40 losses**：
  - gains 全部为翻转驱动（transform-with-sign-function matrix(…sign(1em-1px)…)
    2 案 + scale-parsing-valid `calc(2 * sign(1em - 1px))` 1 案——sign(<length>)
    →<number> 型别门 + 默认字体求值兑现）。
  - losses 集中 rotate/scale/translate-parsing-computed 三页（7/21/12=40，
    其余 104 页零损失）——**归因 registry 缺臂，非本批回归**：pre 臂二进制
    （01:46 树态）含他流 `registry.rs` WIP（rotate/translate/scale 入属性
    清单，个体变换 gCS 序列化启用面）与 part11 shim WIP，post 臂（01:31 批
    构建）无之。丢失形态为整页含 `none`/`0deg` 裸值全灭（枚举/序列化级），
    非数学门语义形态；registry 批入账后三页在 main 复绿。
- 结论：本批对 transforms/easing **零回归**（easing 干净 A/B；transforms
  损失全部归因对照臂的 registry 差异维度）。

### g12c 干净复跑（权威账，results-g12c）

registry 差异维度消去后的单写者干净轮（当前树二进制 = 9017dcca1 + 他流
registry.rs WIP + 本批，transcript 唯一）：**transforms 223 → 228 净 +5，
test 级零回归**（gains=5 losses=0，3 页增）——全部为 sign(<length>) 形兑现：

- scale-parsing-computed 21→23：`calc(2 * sign(1em - 1px))`、
  `calc(1 + sign(1em - 1px))`
- scale-parsing-valid 8→9：`calc(2 * sign(1em - 1px))`
- transform-with-sign-function 0→2：matrix/matrix3d sign 形

即 A/B 段 losses=40 的三页在 g12c 全数复绿（7/23/12——scale 页超 pre 臂
+2），确认 registry 归因 + 本批净 +5。**五套件批终账：values +69、
transitions +2、animations +2、transforms +5、easing ±0，合计净 +78，
页级与 test 级零回归。**

### values 面 registry 补账（results-g12b/css-values.json，g12b values 轮）

registry 批（0a0c910df）在 values 面的独立增益（g12b values 轮 01:53，
registry WIP 在树，对 results-g12 前驱）：**2,641 → 2,646 净 +5 逐页零
回归**——calc-infinity-nan-computed 29→34，scale ∞ number 组 5 案翻账
（`calc(infinity)`/`calc(max(infinity, 10))`/`calc(-infinity * 10)`/
`calc(1 / sign(1em - 16px))`/`calc(1 / (-1 * sign(1em - 16px)))`）——
CSS.supports 前置门放行后经本批 ±∞ 放行 parse 链兑现（两批互补：registry
开门、本批放形）。values face 批终修正 2,426 → **2,646**，五套件合计净
**+83**（values +74、transitions +2、animations +2、transforms +5、easing
±0）。

### g12b transforms 轮 transcript flake 记档

g12b transforms 轮（01:54）merge exit=2 无 JSON 产物：composited-under-
rotateY-180deg.html 在 transcript 输出**两份** `[]` JSON 块（g12 主轮与
g12-ab 基线轮同页均单块）→ 聚合器 "Extra data: line 2" 解析败。同二进制
单页复跑不复现（非确定性双发射，疑 page 超时/重试双 dump 竞态）；runner
健壮性挂账：块体解析宜容忍/去重重发，非本批引入。

## 碰头与协作记录

- 远端入 3 提交（610bbae31 s407 embed / 13133a2f2 R5504 入账 /
  d1c3de2ae r5504 abspos 双翻）——文件面 engine js_dom_bridge_tests + shim
  part01/part04 + layout-engine postprocess，与本批五文件零交集；rebase 推
  迟至语料跑收尾后（跑中 rebase 毒化 wpt-data mtime/内容，跳树毒化在册）。
- 他流 color WIP（part11*）仍在树（g11 同例不触五套件通道）；批中他流新增
  style-system/property/registry.rs WIP（非本批触碰文件，零交集）——
  A/B 三页 40 losses 即 pre 臂含之、g12 批构建不含之的 registry known-props
  差异（rotate/translate/scale gCS 读面），g12c 复跑复绿定谳；registry 批
  彼流入账后三页在 main 复绿。
- 本 evidence 初稿由 rate-limit 中断的前轮 session 落盘（门禁段 TBD 占位），
  本轮语料跑完成后补全——孤儿文件先查内容再续写，未重复起草。孪生 session
  于 01:50 抢先提交同批六文件（9017dcca1，含语料账），双 session 对称收敛：
  代码账归彼（早提交），对照 A/B/g12c 与本文档收口归本 session。
- results-g12b 目录双写者碰撞（本 session transforms/easing 复跑 × 孪生
  registry-knownprops 验证轮同 JSON dir）：transforms merge 双双零产出，
  easing 三轮同值（51P）不受扰；教训续录——同 dir 并发 corpus 轮各自唯一
  transcript/dir。

## g12b 收尾增量（registry/known_properties 落树后 values 复跑）

- results-g12b/css-values.json（本流 g12b 轮，02:53 前驱 results-g12 对照）：
  **2,641 → 2,646 净 +5 逐页零回归**——增益全落 calc-infinity-nan-computed
  scale 组（29→34）：rotate/translate/scale 入 known_properties（0a0c910df，
  含 inherit initial-value 臂 + engine part02 supports 钉）解封
  `CSS.supports('scale', …)` 前置门后，`calc(infinity)`/`calc(max(infinity,
  10))`/`calc(-infinity * 10)` 等写面 case 前置断言翻真。master.md 头部
  values face 同步 2,641→2,646。
- 门禁：HEAD(0a0c910df) 推送后复跑 css-parser lib 3,115P/0F、style-system
  lib 2,516P/0F、engine time_list 钉 1P——终态绿。
