# DC-2 g16 批：tree-counting 叶接入（sibling-index()/sibling-count() + 符号零求值）

日期：2026-11-12　刀：css-parser TreeCounting 叶 + eval_mix 符号零（parse/求值/
串行/型别多臂扫面）；代码单独提交 ed76acefb（css-parser 7 文件 + style-system/
layout-engine 各 1，+212/-2）。
前置：g15 批收口。本轮权威账为**组合二进制**（ed76acefb + 他流 engine
computed 串化刀——已提交 9dbe26c80；results-g16 values 账出自其 06:50:17
前态二进制，取证与提交态复跑见 §4），按 g14 先例组合态入账 + 分桶归因；
本刀孤立贡献以单页探针为证（探针跑在重建前纯本刀二进制 06:42:51）。

## 1. 实现面

- `CalcExpr::TreeCounting(TreeCountingFn)` 新叶（types.rs）：CSS Values 5
  §tree-counting 兄弟序 `<integer>` 产出，parse 期 opaque；`try_parse_tree_
  counting` 无参文法（带参草案未决，拒绝）；parse_factor 挂 calc-mix 臂后，
  `parse_math_function` 裸函数链同收。
- 求值面 fail-open 于权重门、fail-closed 于值：`CalcContext` 无 DOM 视图，
  `eval_calc_with_context` 返 None（FIXME DOM 桥挂账）；`calc_expr_math_type`
  → MTNumber（integer 族，使 `1 * sibling-index()` 过 mix 族门）。
- **符号零**（eval_mix，本批唯一行为面）：权重 0 × 不可求值项贡献 0——语料
  注记 "0% so calc-mix() can still be resolved"（`calc-mix(1 0%, 3 0%,
  5 * sibling-index() 0%)` → `calc(0)`）；可求值项照常相乘，NaN·0 = NaN
  毒化链保持（IEEE 传播不丢）。
- 折叠链兼容（serialize_math）：dimension_suffix/find_divergent_nan_clamp
  → None、expr_has_time_leaf → false；Expr 形权重/嵌套和透传臂不变（语料
  行 322-336/364-378 原文透传保留）。
- 权重/包含门：expr_has_percentage_leaf、expr_contains_mix、calc_contains_
  percentage（style-system）、calc_expr_contains_percentage/has_length
  （layout-engine）→ false 臂补齐；fail-closed 槽 counter_calc_type、
  clip_path_calc_type → None。
- 编译器驱动臂扫：workspace guarded check 全绿，无遗漏穷举 match。

## 2. 验证面

- 单测：calc_mix 新增 3 组（parse+型别门含带参拒绝、符号零求值 4 形含
  NaN 毒化、fold 指定值语料全零塌缩/normalized identity/透传）+ 原嵌套
  守卫钉改 normalized 透传恒等（语料行 303 对齐）；css-parser lib
  3131 全绿（+3）、clippy 3 crate 零告警、fmt clean。
- 单页探针（probe-g16-calc-mix.log，纯本刀二进制）：serialize 76→**111P**
  （+35）、computed 25→**41P**（+16）、invalid 18/18 稳——本刀孤立贡献
  **+51**。全量账见 §4。

## 4. g16 权威账（组合态分层入账，test-guard 包裹）

- **中间账（06:50:17 二进制，9db 尾笔编辑前窗口）**：css-values（对
  results-g15 基线）2,903 → **2,917 净 +14，页级零回归**；test 级 16 增
  2 减——2 例 Pass→Fail 为 **g15 意外通过修正**（见下），非行为回归。
  - calc-mix-serialize +12 净（14 增 2 减；sibling-index 0% 权重行——符号
    零塌缩 `calc(0)`/`calc(0px)`/`calc(0%)` 全零桶 + normalized 恒等行）
  - calc-mix-computed +2（0% used-value 面同因）
- **2 例 Pass→Fail 归类（意外通过修正）**：g15 期 sibling 行 fold 失败 →
  声明值 raw 透传 → text-indent computed 通道回退初始值 '0px' 恰合期望
  （假通过）；g16 fold 成功 → 声明值 `calc(0px)`（specified 面转真实通
  过），computed 通道暴露 text-indent 对 length 族 calc-mix 一律串空串的
  既有缺口——与 20+ 无 sibling 同伴（`calc-mix(1px 0%)` 等，g15/g16 同
  败、报错逐字节同为 `got disallowed value ""`）汇入同一桶，即 g15 挂账
  computed 串面桶，归 9dbe26c80 后续落账面。
- 组合态归因注记：孤立探针 +51 → 组合账 +14；serialize 页组合态 88P 对
  探针 111P 差 -23 归他流 computed 串化刀（组合态仍处处 ≥ g15，无回归）。
  其面终态落账归其流；本刀贡献以 §2 探针为孤立证据。
- 二进制取证：values 账出自 06:50:17 二进制（9dbe26c80 前一态，06:52:30
  尾笔编辑未入）；**提交态复跑 results-g17/css-values.json 已落盘：2,903 →
  2,953 净 +50（两刀均提交态）**——8 页增益 +78（serialize +35/computed
  +16/minmax-length-serialize +12/border-radius 两页 +10/minmax-lp-serialize
  +3/letter-spacing +1/gradient +1）+ **2 页回归 -28**（minmax-length-percent-
  computed -26、hypot-pow-sqrt-computed -2——min/max/hypot %+px used-value
  等价面，归 9dbe26c80 computed 串化面：其三页探针未覆盖此二页；非本刀
  面——本刀行为臂仅 TreeCounting 叶存在时可达，该二页无 sibling 语法）。
  对账文件 results-g17/diff-values-vs-g15.txt；-28 修复归 computed 串面流
  挂账。results-g16 六套件扫账 values（07:08，bin-g16 冻结二进制）与 g17
  逐位一致（2,953P/2,640F），双录互证。
- 对账文件：results-g16/diff-values-vs-g15.txt。

## 5. 挂账桶更新（提交态口径）

- serialize 页 111/128 残 17F：% 宽槽 computed/used 解析行
  （`calc-mix(1% 0%) ` 族 + `calc(10% + calc-mix(...))` used 面，需布局
  分辨率）+ sibling 非零权重行（50% 混合，需 DOM 桥兄弟序求值）。
- computed 页 41/53 残 12F：DOM 桥 sibling 求值 + % 面。
- tree-counting 家族 23 页（keyframes/descriptors/invalidation/media-query
  等）：DOM 值面全残，挂 DOM 桥桶——`calc-sibling-function-parsing.html`
  parse 面 25/47 亦待文法扩展后重盘。
- getComputedStyle-calc-mixed-units 0/3·5/8·4/7 原样（used 值换算面）。

## 6. 六套件漂移对账（对 results-g14b 基线，零漂移门；bin-g16 冻结二进制
= 提交态 9dbe26c80 构建 07:04:58，sha 0e42b777d928de00，test-guard 包裹）

- color 10,972 → 10,972 **±0**；transitions 200 → 200 **±0**；
  animations 238 → 238 **±0**；easing 51 → 51 **±0**。
- transforms 228 → **229 净 +1**：transforms-support-calc
  "perspective supports calc" F→P（9db computed 串化刀 calc 支持面外溢
  增益）；回归 0。
- values 2,887 → **2,953 净 +66**（g15 +16 与本批 +50 累计，分层见 §4；
  test 级 -28 为 g15 双侧空串假通过修正、挂账 computed 串面流）。
- 六套件页级零回归；无未解释漂移。
- 对账文件：results-g16/diff-{color,values,transitions,animations,
  transforms,easing}-vs-g14b.txt。
