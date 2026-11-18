# rule-color 列表文法刀（二十四轮，刀 A）

日期：2026-11-16 深夜～11-17。提交：`8c3d4d5f7`（刀体+twin 简写 apply 臂）→
`970e19289`（撤回 WIP 携入的承重边界撤除）。终态码态 = `970e19289`。
对应设计稿：[gap-rules-knife-design](2026-11-16-gap-rules-knife-design.md) 刀 A。
（hash 注记：docs 收口 rebase 重写刀码哈希 ed3091d45→8c3d4d5f7、2de00fcad→
970e19289，本档活引用已换新哈希；A/B 节 B1 的 6f97346e2/0c15c48b 为 amend
前史+二进制 md5 配对 provenance，保留不换。）

## 刀体五站（css-gaps-1 §column-row-rule-color `<line-color-list>`）

- **css-parser**：`parse_rule_color.rs` 新档（292 行）。段模型
  `RuleColorSegment::{Plain, Repeat{count, colors}}`，计数
  `RuleRepeatCount::{Auto, N(≥1), Calc(原文)}`；括号感知顶层逗号切分
  `split_top_level_commas`；`auto` ≤1、repeat 内逗号颜色列表、system-color
  dark 口径；`rule_color_list_to_css` 对 calc 计数 parse_calc→eval_calc
  折叠整数。钉 4。
- **style-system**：`column_rule_color` ColorValue→`Vec<RuleColorSegment>`
  迁移 + `row_rule_color` 新字段；registry/inherit/apply/default_impl 四臂；
  白名单 +2（`row-rule-color`/`rule-color`）；VALIDATED_PROPERTIES 22→25
  （裁决臂置于 substitution-function 放行臂之后、未实现函数放行臂之前）；
  `expand_rule_color` 简写双 longhand 展开。twin amend 增补 rule-color 简写
  apply 臂（expand_shorthands+逐条递归 apply 全过才落）+21 行测试。
- **engine**：gCS `rule_color_list_to_gcs`（单段平色走既有 color_to_css
  保 multicol 纯色串化零漂移；calc 计数折叠整数）；`row-rule-color`/
  `rule-color` 读臂（row/column 全等→该值否则空串）；paint 首段首色
  （`rule_color_list_first_color`）保形。
- **残面⑤首片（后撤回）**：WIP 携入 `parse_clip_inset`/`parse_position_pair`
  括号感知切分、inset calc 偏移受理边界撤除——**守卫定谳 −78 后撤回**，见下。
- **验收债**：apply_initial_value 补 shape-margin/shape-image-threshold
  复位臂（二十三轮验收）。

## −78 守卫定谳与撤回（`970e19289`）

WIP 携入的 paren-aware 切分按「残面⑤已闭合」假设撤除了 b1v8 钉化的
拒收边界，但 calc 代数插值面并未全闭环。逐案对账实测（reconcile，B1 vs B0）：
css-masking clip-path-interpolation 三页 **−74**（xywh-rect −58 / 002 −10 /
001 −6）、css-shapes 净 **−4**（shape-outside-interpolation 混合净 −12：受理
解锁的 inset 中点 +8 但回落断链 −20 级；受理增益 +8 另散布 6 页：inset-function-
valid/computed、shape-outside-computed、circle/ellipse-011、inset-008）——
合计 **−78**（guard 过程初报 −90/−16 经逐案对账更正）。b1v6/b1v7 先例（动画
通道回落链断裂）第三次复现。按**伪通过池保位纪律**撤回至拒收边界态
（cf2d10ac9 两文件原状回植），clip_path 50 钉（rejection 形）全绿；撤回终态
b1v3（`e2bb5d29`）shapes/masking 对 B0 复测 SET-IDENTICAL（145=145/61=61，
0/0/0，final 链与 v3 确认腿双证）。残面⑤（calc 代数插值全闭环）继续挂队列，
是未来重撤该边界的硬前置。

## A/B 权威账链

- B0 = `dc1bab12`（cf2d10ac9 detach 重建，刀父）；B1 = `0c15c48b`
  （6f97346e2，六腿：gaps +139、multicol +2、typed-om 护栏 0/0/0）；
  twin amend `8c3d4d5f7`（简写 apply 臂 +7）。
- **双写者污染事故**：`results-rulecolor-b1v3` 目录被 unit+inline 多写者
  并发合并污染（gaps transcript 788 重复条目、并发加载窗内 4 页空产出丢失
  形 141→145 复并），目录废弃不作账；同目录毒化还含 b1v3g 链 unit cwd 毒化
  全案秒败产物（pass=0 形，已删 json×3/transcript×3）。
- **final 权威链**（`zw-rulecolor-final`，单写者独占窗，b1v3 保全二进制
  `e2bb5d29` = 970e19289 码态，五套件直测 B0，PAGE-ONLY 零容忍）：

| corpus | 页集合 | net | lost/regressed | 判定 |
|---|---|---|---|---|
| css-gaps | 75=75 | **+146**（359→505） | 0/0 | 刀 A 增益 + twin 简写臂 +7 |
| css-multicol | 91=91 | **+2** | 0/0 | 增益 |
| css-typed-om | 359=359 | 0 | 0/0 | SET-IDENTICAL 护栏 |
| css-shapes | 145=145 | 0 | 0/0 | SET-IDENTICAL（撤回复正） |
| css-masking | 61=61 | 0 | 0/0 | SET-IDENTICAL（撤回复正） |

**净 +148 / 0 lost / 0 regressed，五 corpus 页集合全等（PAGE-ONLY 零容忍满足）**。
工件：`.acceptance/wptfyi-css/results-rulecolor-final/`（b1v3 污染目录废弃）。
8c3d4d5f7 提交正文记 +141 = B1 态 gaps +139 + multicol +2（简写臂 +7 在
amend 内，合计 +146 与 final 账一致）。

## 单测与门禁

- css-parser 钉 4、style-system 2,547/0F、engine lib 4,076/0F 全绿；
  fmt/clippy 三 crate 净（唯一 warning = peer es_module WIP 在册连坐）。
- bench-gate 定向（zw-rulecolor-bench2，`ZERO_WEB_BENCH_CRATES=
  zero-css-parser,zero-style-system,zero-engine`，HEAD=970e19289）：microbench
  **3/3 PASS**（css_bench/engine_bench/style_bench），页面场景 welcome p95
  7.97ms / medium 281ms / morning 49ms、resource 峰值 RSS 171MB / startup
  39.7ms、form-input p95=0.014ms jank=0；`perf-gate` **rc=0 PASS**（form-input
  预算 + absolute page-total / retained-form 预算过；基线硬件错配 WARN 在册——
  基线录于 i5-13500H、本机 Ryzen 9700X，绝对预算口径过门）。
  过程注：test-guard 默认 180s 墙钟恰在 bench-report 收尾瞬间杀树（脚本续行
  echo 未达），perf-gate 对已落盘报告工件（benchmark_20261117_001523.json）
  单独补跑定谳。
- make test 拆腿（mkt5，es_module.rs peer WIP stash dance 净树跑，fmt 前置门
  过后 debug 全量在飞）：刀面三 crate 全绿（css-parser 3,280/0F、style-system
  2,517/0F、engine lib 4,076/0F），make 于唯一红止腿——
  `service_worker_manager::tests::module_update_compares_static_dependency_bytes`
  （page-runtime:3782），test-guard solo 复跑（纯 HEAD 净树，WIP 已 stash）
  复现 = 在册账 script-sandbox 流真实红，非本刀（mkt3/mkt4 死因 = fmt 门吃
  es_module 漂移，5 秒即止零测试产出——拆腿须净树前置）。
- make test 全腿收口（zw-rulecolor-mkt7，make -f 片段逐腿 LEGRC 记账
  no-fail-fast，00:33–00:49，es_module WIP 在树态）：workspace 腿 css-parser
  **3,280/0F**、style-system **2,517/0F**、engine lib **4,077/0F**（刀域零红）；
  es_module 二进制 441/**6F** = peer WIP 连坐（同一 WIP 使在册 module_update 转
  绿——与 mkt5 净树面互补实证两面互换）；qjs-clippy 101 = WIP 连坐
  too_many_arguments；红项全对号在册账：renderer-lib 101（worker-fetch）、
  browser-bin 101（**tab 镜像 + default_actions**，741P/2F，panic
  apps/browser/src/tests.rs:2945 实证）、browser-tests 101（**focus-governance
  六钉 stale-binary 环境形伪红** 0P/6F——panic 文本 "renderer binary is
  stale … rebuild zero-renderer"，overlay 无 cargo build -p zero-renderer
  前置+stash pop 抬源 mtime 触发，零语义红，全量 make test 自带前置不现形）、
  qjs-w 101（image_batch_e2e 环境形 775P/1F/59I）、qjs-rlib 101
  （worker-fetch）；GPU 双腿 0/0（wgpu adapter 可用）、qjs renderer bin 0。
  全 battery FAILED 17 行 census 与 6 个 FAILED 二进制逐一互证（journal
  逐行对账，红项零缺零多）。
  过程注：00:27–00:29 三电池并发碰撞（zw-rc-mkt5-legs / zw-rulecolor-mkt6 /
  mkt5b）按 notes 仲裁收束为 mkt7 单写者 canonical；zw-rc-mkt5-legs 死于
  rustc SIGSEGV（同 target 多 cargo 门禁并发 = 增量缓存损坏先例再证），
  途中账与 mkt5b 互证后废弃。

## 残面队列

- 刀 B：width-list + rule/row-rule 简写家族，~684F。
- color 真插值池 ~412F（列表插值器缺位）。
- 刀 D：inset-cap/junction 几何族，~1.7k 最大残面。
- 残面⑤：calc 代数插值全闭环（承重拒收边界重撤的硬前置）。

## 教训

1. 涉 clip-path/shape-outside 解析面的改动必须带 masking/shapes 守卫腿——
   gaps/multicol/typed-om 三腿全绿不构成零回归证据（−78 漏检先例）。
2. 共享 results 目录必须单写者独占窗——unit+inline 双写者并发写会合并
   出重复条目与截断 json，账目作废。
3. systemd-run 发射 runner 链必须显式 `-p WorkingDirectory=<repo>`（或脚本
   首行 cd）——runner wpt-data 相对解析基于 unit cwd，发射实例 shell cwd
   非 repo 根时全案秒败 pass=0（b1v3g VOID 先例，788 案 17.3M 峰值真空转）；
   判腿先核 corpus(json) 页数与 P>0 再采认。
4. 提交正文引用的证据文档路径必须与提交同轮落地（8c3d4d5f7 引用本档，
   本档为补账轮补齐）。
