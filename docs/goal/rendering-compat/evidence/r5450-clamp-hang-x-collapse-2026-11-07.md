# R5450：R5450 clamp trim 清零 run x 折叠——block-ellipsis-032 红带域收口（R5297 band 域幻影带）

日期：2026-11-07　域：`crates/layout-engine/src/inline/mod.rs`（apply_line_clamp_cap 尾部）
驱动案：css/css-overflow/line-clamp/block-ellipsis-032（hangable whitespace before ellipsis）
接续：R5449 修复切片（trim 落地后余量 ✗1.76% 三层挂账）的第 ① 层。

## 1. 归因（REFTEST_DUMP + REFTEST_DEBUG 图元级定谳）

R5449 挂账① 原表述「box-level span 背景自 span LayoutBox 几何施画（painter/mod.rs:3191/3198）」
**勘误**：REFTEST_DEBUG 图元 dump 实证红带 fill 全部来自 **R5297
`owner_line_band_extent_map` → R639 fragment bg 臂（whole_box）的 fills 通道**；
盒级（painter/mod.rs paint_background）在两态均未发出红色 fill。真实机制链：

1. R5449 trim 把末保留行 hangable 尾随空格 run 清零（text.clear + width=0），
   但 **x 保持自然递进位**（032 box0：8 空格挂 [200..264]，IFC 内容右缘 232）。
2. `owner_line_band_extent_map`（paint text.rs:6016）对每个 fragment 向祖先链
   union `(x, x+width)`——清零 run 仍贡献 `(x, x+0)`，band max_end = 264。
3. paint 侧 whole_box 臂按 band 域发红色 fill，**裁到容器内容右缘**：
   fill (209,69,32,19.6)＝IFC [200..232]（屏 [209..241]），即 R5449 取证所见红带。
4. 三盒红带同域实测（屏坐标）：box0 [209..240]、box1 [177..240]（right 对齐
   −40 位移后空格 [168..232]）、box2 [182..240]。

## 2. 修法（layout 层 x 折叠）

trim 生效分支追加：被剪除 run 的 x 折叠到剪除点（混合 run 新末端 `x+width` 或
首个纯空白 run 起点）。折叠后 band 退化为零宽点，whole_box 臂发出 **零尺寸
fill**（实测 ON 三盒红 fill 全部 0×0）——红带像素全消。保留内容 x 不动
（R1338 对齐口径已排除 hang 宽，位置中性）；与 trim 同门（`clamp_hang_trim`，
kill-switch `ZW_CLAMP_HANG_TRIM=0` 同关 trim＋折叠，R3858 独占进程教训）。

## 3. A/B 定谳（同二进制 kill-switch 双态）

- **032 solo**：ON ✗**1.30%** ×3 稳定 / OFF ✗1.83%（＝R5449 OFF 基线逐位同值）。
  净改善 −0.53pp；三盒红带 ON 全消（0 红像素），OFF 红带满域。
- **corpus 16594 双窗**：ON 698F / OFF 696F；ON-only 2 案
  （insert-block-…-end-002、table-anonymous-objects-148）＝在册互翻族，
  solo ON ×3 全 ✓0.00% 洗清＝**真态 REGRESSED=∅ FIXED=∅**（032 双态均 fail，
  案级改进不进 FIXED 列）。
- **block-ellipsis family**：双窗逐案全同（3 稳定红＝021/032/bidi-003）零 churn。

## 4. 剩余余量（032 ✗1.30% 组成）＝挂账②对齐参与域

红带层（原挂账①）本轮收口后，残差全部落 **right/justify 对齐参与**：
- 对齐在 `break_items_into_lines` 末尾（break_lines.rs:1416）先于 clamp cap
  执行，content_width 含 run 级悬挂空格宽（R1338 hang 豁免门是 IFC 级
  `preserve_whitespace`，032 容器 normal 不入门）→ box1 right 对齐 offset
  = 232−272 = **−40**（"This" 4ch 左裁，IFC dump 实证 This x=−40）；
- ellipsis 在对齐后追加（paint 侧 last_text_end_x 定位），ref 把 ellipsis
  计入行内容再对齐 → box1 ref 行起点 2ch（27ch 行右对齐 29ch 盒），ZW 行
  尾留白 ~1ch 级残差；box2 justify 同域（ref text-align-last:justify 含
  ellipsis 均布，ZW 对齐后追加）。
- 修法方向（下轮切片）：clamp 生效分支 trim 后对末保留行**再对齐**
  （right/center：Δ = line_limit − trim_unshifted − ellipsis_w −
  first_run.x 附加位移；justify：kept runs + ellipsis 重均布，cleared
  tail 不占 gap）。改 clamped 行对齐 → 须 corpus 全矩阵 A/B 预算。

## 5. 门禁矩阵

- fmt clean；clippy workspace `--all-targets -D warnings` 绿（新测试文件
  assign_op_pattern 一处顺手修正）。
- layout-engine 全腿 `--no-fail-fast`：lib **2432P/0F/2I**（R5449 2430 + 2 钉）
  ＋ 16 集成 target 零失败，合计 2471P/0F。
- make reftest **725/725**。
- corpus 双窗 A/B 详上（真态零开关确定性差异）。
- product-smoke 全 PASS（welcome 17.23% 与 R5444-R5449 逐字节同值＋struct
  全 PASS＋form-input-perf-gate）。
- bench-gate 定向 zero-layout-engine：见控制面补录（静窗纪律）。

## 6. 常驻测试

- `src/inline/tests/r5450_clamp_hang_x_collapse_tests.rs`（单元 2）：双纯空白
  run 清零后 x 全折叠到首个纯空白 run 起点（band 上界不越剪除点）／混合 run
  剪尾后纯空白 run 折叠到混合 run 新末端。
- `tests/r5450_clamp_hang_x_collapse.rs`（集成 2）：032 形态端到端（外部
  sheet 传入，r5342 口径）——clamp:1 清零悬挂片段 x 全折叠、band 上界不越
  剪除点／无 clamp 负向（空格片段 x 严格递进）。
- `tests/r5450_clamp_hang_x_collapse_kill_switch.rs`（独立进程 1，R3858
  模式）：`=0` trim 与折叠同门全关，悬挂片段保留且 x 自然递进。

## 7. 方法注记

- **stale dump 陷阱（本轮教训）**：`make reftest-upstream FILTER=…` 不带
  `REFTEST_DUMP=1` 时**不写 dump**，`target/reftest-dump` 残留上次带 dump 运行
  （可能跨会话/跨开关态）的 PNG——首轮像素取证把上一会话遗留 dump 当 ON 态
  分析，得出「box0 红带与开关无关」的假结论。 fresh 结论须 `rm` dump 目录后
  显式带 `REFTEST_DUMP=1` 重跑再取样。
- 图元级归因链：REFTEST_DEBUG（fills per-item rect+rgba）＞ 像素测量：
  ON 三红 fill 全 0×0 直接证施画者塌缩，绕开多 IFC 实例（layout pass 无位移 /
  paint pass 有位移——032 页 paint 重跑 IFC 带真实 styles）与 span 盒宽同步
  路径的推理泥潭。
- ZW_CLAMP_PROBE（R4414 临时门）扩展 fragment dump 为本轮取证入口，用后撤。
