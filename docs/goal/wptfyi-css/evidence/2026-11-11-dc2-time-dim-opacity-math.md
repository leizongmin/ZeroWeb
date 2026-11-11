# DC-2 g10：time 维度（CalcExpr::Time + allow_time 门控）+ <opacity> 数学函数臂

日期：2026-11-11（夜批）
基线：HEAD `9291e87ab`（g9 mod/rem 除零批后），ba 流 js hunks 在 stash@{0} 隔离。

## 结果

- **css-values 语料：2,464 → 2,516，净 +52，逐页零回归**（0 页失 pass / 增 fail；F 3,122 → 3,070）。
- **css-color 对照：10,970P 零漂移**（对 g9b 权威账同值）。
- make test 见下文「门禁」——首轮死在 fmt 门（上游 CSSOM 提交带入未格式化 `callbacks.rs`，非本批改动面），`cargo fmt -p zero-engine` 机械修复后重跑。

## 净增逐页归属（6 页，全部净增）

| 页面 | ΔP | 失败变化 | 归因 |
|---|---|---|---|
| minmax-time-computed.html | +19 | F 24→5 | min/max() 接受 time 实参（MTTime 型别推进） |
| round-mod-rem-computed.html | +12 | F 50→38 | round/mod/rem 的 time 实参形（`round(10s, 6s)` 族） |
| minmax-time-serialize.html | +11 | F 22→11 | minmax-time computed 同源的序列化形 |
| calc-time-values.html | +6 | F 10→4 | `calc(10s + 5s)` 等 time 维度算术 |
| signs-abs-computed.html | +2 | F 103→101 | `sign()/abs()` time 实参形 |
| hypot-pow-sqrt-computed.html | +2 | F 14→12 | 同上（time 实参数学函数） |

净 +52 远超批前预估 +15（transition-delay 12F + opacity 3F）：time 槽位解锁是
**全家族效应**——round/mod/rem/minmax/sign/abs/hypot/pow 的 time 实参形一并从
「dimension 不识别」桶翻绿。transition-delay 单页归属不变（其修复包含在家族内）。

## 改动面（9 文件，+204/-4）

- `crates/css-parser/src/values/types.rs`（+75）：
  - `CalcExpr::Time(f64)` 变体（秒为底，ms 入库时归一）。
  - `CalcParser` 增 `allow_time: bool` 门控：全部既有构造点补 `allow_time: false`，
    共享 calc 入口保持 time-closed（防 `calc(10s)` 经数值捷径在 length 槽变合法）。
  - 新 `parse_time_calc()`：time 槽专用入口（`allow_time: true`），全串消费校验。
  - 新 `calc_expr_is_pure_time()`：表达式纯度判定（递归 BinaryOp/BinaryMathOp/
    Min/Max/Clamp/UnaryOp），拒绝混型（time+number）。
- `crates/css-parser/src/values/parse_extended.rs`（+10）：`CalcMathType::Time`；
  `calc_expr_math_type` 的 Time 臂；Multiply 增 `(MTNumber, MTTime)|(MTTime, MTNumber)=>MTTime`、
  Divide 增 `(MTTime, MTNumber)=>MTTime`；`calc_expr_has_length` Time=>false。
- `crates/css-parser/src/values/parse_transform.rs`（+13）：`parse_time` 增数学函数臂
  （`parse_time_calc` → 纯 time 门 → `eval_calc` → `is_finite` 门；±∞ fail-closed
  挂 FIXME(types.rs:1420 同族) ∞ 管线桶）。
- `crates/css-parser/src/values/parse_layout.rs`（+25）：活版 `parse_opacity` 增数学函数臂：
  `parse_math_function` → `calc_expr_is_number` 门 → `eval_calc` → **NaN censor-to-zero
  （css-values-4 §10.12）** → ±∞ fail-closed → clamp(0,1)。评审修复：censor 判定
  必须先于 `is_finite` 门（NaN 不有限，顺序颠倒则 censor 不可达，单测钉住）。
- `crates/css-parser/src/values/tests/basic.rs`（+73）：4 组新钉：
  1. `test_parse_time_calc_math_over_time_units`：round/mod/rem 的 time 实参求值 + ms 混型。
  2. `test_parse_time_calc_rejects_mixed_and_shared_entry_stays_closed`：time+number 混型拒；
     共享 `parse_calc` 对 `10s` / `calc(10s + 5s)` 恒 None（time-closed 回归门）。
  3. `test_parse_time_math_forms_via_parse_time`：parse_time 端到端。
  4. `test_parse_opacity_math_function_forms`：round/mod 形、**`round(1,0)`→NaN censor 0.0**、
     clamp 越界 clamp(0,1)、`calc(1px)` 拒、`round(10s,6s)` 拒（opacity 槽 time-closed）、裸形零漂移。
- 非穷尽匹配补臂（E0004，`CalcExpr::Time` 新变体的全仓扫描收口）：
  `parser/helpers.rs` `counter_calc_type`、`parse_extended_border.rs` `clip_path_calc_type`、
  `style-system/computed.rs` `calc_contains_percentage`、`layout-engine/converter/mod.rs`
  `calc_expr_contains_percentage` + `calc_expr_has_length`——均 `Time(_) =>` 中性臂。

## 门禁

- css-parser lib：3,111 passed / 0 failed（含 4 组新钉）。
- workspace clippy（css-parser + style-system + layout-engine）：零告警。
- 语料跑：`run-g10-time-opacity.sh`（systemd-run --user `g10-corpus-run.service`，
  test-guard `--per-proc-mem 4 --total-mem 8 --time-limit 3540`；构建 43s，values 2m16s，
  color 35s）。权威账：`.acceptance/wptfyi-css/results-g10/{css-values,css-color}.json`。
- make test 四跑链：①mktest-g10 死 fmt 门（`cargo fmt --all -- --check`）——
  `js_dom_bridge/callbacks.rs` 为上游 CSSOM 提交（0a435f0a1/0ff1bc92c，另一流，
  g9b 基线之后入 main）带入的未格式化测试代码，非本批改动面；`cargo fmt
  -p zero-engine` 修复（+97/-22，剥空白全等比对：仅 8 处 struct pattern 尾逗号
  + 换行重排，零内容差异），独立 `style(engine)` 提交。②mktest-g10b（同名两跑：
  首跑死于 service 未注入 cargo PATH——rustc ENOENT 即退，日志被重跑覆盖）重跑
  编译 6m19s 后 fingerprint 写入 ENOENT——并发 make 入口的 target-disk-guard
  分级清理撞上在途 cargo 编译窗（共享树双门禁竞态，非仓库代码缺陷；
  target 85G>50G 阈值触发 tier-1 incremental 清理）。
  ③mktest-g10c `test-guard 无法启动测试 artifact`——R4323-F 在案瞬态（暖缓存
  + 并发 clippy 抢锁失稳窗，R4335 同例复跑全绿）。④mktest-g10d 权威账：
  **3,881 passed / 10 failed**（g9b 3,879/9 基线 +2P）。
- **r307_tree_order_identity_domains 新红归因探针**（engine js_dom_bridge 树序
  恒等，非本批面）：db74aad18（含 CSSOM 两提交、无 g10）worktree 单测复跑
  0 passed / 1 failed，而 g9b 基线 14b6ca724+g9 同测绿（3879P/9F 在册无此名）
  ——红窗精确钉在 CSSOM 两提交（0a435f0a1/0ff1bc92c），g10 豁免；+2P 同窗
  归因。账目挂 rendering 流碰头账，修复归彼流（r307-probe2.log）。

## 与预估差说明及残余

- 批前预估 +15 只算了 transition-delay 12F + opacity 3F；实际 +52 的差值来自
  round/mod/rem/minmax/sign/abs/hypot/pow 家族的 time 实参形——批前归类时这些
  被记在「dimension 不识别」泛桶，未单独计页。
- `parse_opacity` 数学臂 3F 落地（含 NaN censor）。censor 语料 inert 判定（g9 定谳）
  仍成立：宿主 face 今日走 shim 路径，shim 侧 NaN censor 臂是独立刀（3F，在册挂账）。
- 残余桶不变：scale ∞ 20F（FIXME types.rs:1420 族）、%-mixing 18F（CalcContext 族）、
  transform matrix 化 5F+6F、line-break-ch-unit 194F（ch 单位独立桶）、
  random-computed 141F（random() 独立桶）。
