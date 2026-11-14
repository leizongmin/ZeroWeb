# g40：serialize-resolution 刀——`<resolution>` 维度 + image-set() calc canonical 折叠

日期：2026-11-14。承接 master.md「DC-3 v14 周跑对账」values 残余桶
「serialize-resolution 29 主体未动（image-set parse 前置刀，deferred）」。
上一轮 session 于 19:55 rate-limit 中断：g40 脚本（19:48）已备妥未发射、
Resolution WIP 已写全（mtime 19:43-19:46）、B0 预编辑二进制已建
（19:36:15，`zw-g40-b0-build.log`）。本轮按孤儿认领先例接手完成验证链。
（记账收敛注：本文件为双 roll 并行写账竞态后的合并定稿——先手 evidence
草稿两份归并不并行件保留，账目数字以 results 工件逐案对账为准。）

## 1. 恢复期事故与法证

- **B0/B1 时序法证**：B0 二进制（19:35:41-19:36:15 build）含对端 shim WIP
  常量（part01.js 14:22 / part01f.js 06:54——html-semantics 流对 C4 刀
  9d587f932 的在途回退），不含 Resolution WIP（19:43-19:46）；B1 二进制
  （21:19:52 build，exit=0，同树；21:20:38 0 秒 no-op rebuild
  fingerprint 复核）= 同基座 + 本刀。A/B 精确隔离本刀。
- **多实例多写竞态**：session 恢复边界重放叠加双 roll 发射，
  `run-g40-resolution.sh` 六次发射记录、两实例并发（log 三 build start
  头 21:19:12/21:19:51/21:20:38），同 `results-g40-b1` JSON 目录 +
  双 transcript 互焚（master.md 11-11 晚「anchor 零漂移对账三写竞态」
  同型事故）。处置：脏日志归档 `zw-g40-resolution.collide-212402.log`
  弃用，results 目录清空，双树按 PID 全杀，探针 CLI 形态修正
  （`css-values <page>` → `testharness-css-values <page> --json`，
  旧形探针全部打 usage 无产出）+ 脚本加 flock 互斥入口 + transcript
  唯一名（`$$` 后缀）后**单实例重跑采信**——终态
  `results-g40-b1/{css-values,css-images}.json` 为 21:26-21:29 带锁
  单实例产物（values 21:29:02 merge / images 21:29:27 merge，链
  exit=0），provenance 可审计。教训：无人值守脚本需加自实例锁防
  恢复边界重放（与既有 learning rally-double-spawn-race 同域）。
- 对端在途件不动：C4-revert shim WIP + staged html-semantics/rendering-compat
  控制面改写均为对端流 face（其 session 同殁于 rate-limit），保持树内常量，
  A/B 内部两侧等含。

## 2. 语义面（css-values-4 §6.4 + css-images-4 §image-set-nota）

- **`CalcExpr::Resolution(f64)` 新叶**（dppx 基）：`x`/`dppx` 恒等、
  `dpi ÷ 96`、`dpcm × 2.54 / 96`（1in = 96px = 2.54cm）；后缀判定 dppx
  先于 x（`1dppx` 若先剥 `x` 得 `1dpp` 恒败）；非有限字面拒（`infinity`
  走常量臂，无 `∞dppx` 形）。
- **`parse_resolution_calc` 独立入口**：`allow_resolution` 旗标同
  `allow_time` 口径——共享 `parse_calc` 入口保持拒 resolution 原子
  （length 槽不得因数值捷径接受 `calc(1dppx)`）；现无属性收裸
  `<resolution>`，消费方仅 image-set() 声明值内 calc 折叠。
- **`calc_expr_is_pure_resolution` 纯门**（镜像 `calc_expr_is_pure_time`）：
  裸 Number 叶拒（resolution±number 混型 spec invalid §10.1）、标量乘除
  `<resolution>×<number>` 合法、Mix/TreeCounting/Progress 保守拒。
- **型别表 `MTResolution` 行**：`number·resolution = resolution`、
  `resolution/number = resolution`；`resolution²` 无量纲拒。
- **全臂 exhaustive 扫**：counter 槽（helpers.rs）/ clip-path 槽
  （parse_extended_border.rs）fail-closed；% 叶扫描、`calc_expr_has_length`
  /`has_anchor`/`has_tree_counting`/`expr_has_time_leaf`/
  `find_divergent_nan_clamp`/`mask_tree_counting`/`calc_contains_percentage`
  （style-system）/`calc_expr_contains_percentage`（layout-engine）/
  `font_stretch_percentage_tree_valid`/`has_percentage_leaf`（apply.rs）全补
  Resolution 臂；lh 单位改写跳过；`eval_calc` Resolution 叶按 dppx 出数值；
  `tree_counting_slot_valid` MTResolution → false。

## 3. image-set() 折叠（fold_decl_specified "background-image" 臂）

`fold_image_set_specified`：仅 `image-set(` 头形且含 `calc(` 时扫描——
引号态机（含 `\` 转义）外定位 `calc(`（大小写不敏感）、配平右括号取
span（引号内括号不计深）、`parse_resolution_calc` + 纯门 + `scalar_value`
求值 + `dimension_suffix`（纯 resolution 树恒 `dppx`）→
`format_dimension_inner` §10.6 canonical 发射（有限 `<n>dppx`、NaN/±∞
`NaN|infinity * 1dppx`）；无变化/非 image-set/混型/未闭合整值 None 保原文
（fail-closed）；幂等（canonical 形二次折叠恒等）；多字节 URL 前缀
char-boundary 安全。嵌套内层 `calc(` 随外层 span 整体消费不重扫。
写读面接线零新增站——读视图单点 `js_dom_bridge.rs
style_from_mutations_lw`（r2320 既有）per property 调
`fold_decl_specified`，background-image 臂自动消费；mutation log 保留
原文（specified 折叠语义）。

## 4. A/B 权威账（B0 19:36:15 vs B1 21:19:52，同 shim 基座）

| corpus | B0 | B1 | 净 | REGRESSED |
|---|---|---|---|---|
| css-values（256 案） | 4012P/1575F/6T | 4041P/1546F/6T | **+29** | **0** |
| css-images（41 案） | 881P/2334F/0T | 887P/2328F/0T | **+6** | **0** |

- values +29 恰目标页 `calc-infinity-nan-serialize-resolution.html`
  **29/29 全页清零**（29F 桶 → 0F），UPS 全表无第二页；B0 失败消息
  形态全部为 `assert_equals '原始形' vs 'canonical 形'`——写面接受、
  读回原文未折叠（非写被拒），与读视图单点接线设计一致。
- images +6 全在 `image-set-parsing.html`（`calc(-1 * 1x)`/`calc(1dppx…)`/
  `calc(2x…)`/`calc(96d…)` 形经折叠 canonical 回读，写面接受+round-trip
  翻绿）——本刀红利，非目标页预期外增益。
- **批终净 +35 零回归。**
- json 工件：`.acceptance/wptfyi-css/results-g40-b0|b1/{css-values,css-images}.json`；
  逐案 diff 工具 diff-pages.py。

## 5. 探针残面（非本刀，分桶挂账）

- serialize-length 残 2F：`calc(1% * ±infinity)`——% 叶
  dimension_suffix 域（%-CB 形在册桶）。
- serialize-number 残 3F：`calc(1 * max(NaN, min(0,10)))` 嵌套 mix 形
  （在册 serialize 散形桶）。
- `image-set-computed.sub.html` 36F 维持：computed 值序列化通道 + 写面
  supports 拒 `calc(-1 * 1x)` 形（`is a supported value` false）——
  image-set computed/supports 通道另刀（本刀只折叠指定值）。
- `resolution-with-percentage-without-context.html`：0 子测试（语料页
  为 reftest 页 `rel="match"`，无 harness 断言产出，runner 一致跳过），
  账面不计；`@media (resolution … calc(sign(50%)*1dpi))` 求值面独立
  桶挂账。

## 6. 单测与门禁

- css-parser 新钉 4（`tests/basic.rs`）：单位归一 5 形 / NaN·±∞ 数学形
  6 断言 / 纯门+共享入口闭+型别表行 / fold 端到端 6 形含幂等·fail-closed
  三面·多字节安全。
- 三 crate 单测全绿（test-guard 包裹，exit=0）：css-parser 3212 单元
  （+4 新钉）+ 1 集成 / style-system 2538 / layout-engine 2515 单元
  （2 ignored）+ 20 集成。
- rustfmt --check 十文件 clean；clippy（css-parser/style-system/
  layout-engine）零告警。

## 7. 挂账

- % 叶 ±∞ 序列化（serialize-length 2F，%-CB 桶）。
- image-set computed 序列化通道 + write-face supports 拒面（36F）。
- serialize-number 嵌套 mix 3F（在册）。
- 对端在途件：C4-revert shim（html-semantics 流，未提交）——rebase 后
  组合态复验时注意其常量地位。
