# DC-2 g15 批：calc-mix() 嵌套和串刀（外层 calc 树递归 canonical 化）

日期：2026-11-12　刀：serialize_math 树递归项化 + 单位排序（css-parser 单 crate 面）
前置：g14 批收口（batch-step evidence）；并行流 45b769764（写面文法门收
text-indent/transform-origin-z——invalid 6F 清零）同轮入 main，本轮单二进制
含两修，分页归因（invalid 页归文法门刀，serialize 页归本刀）。

## 1. 实现面

- `fold_mix_nest_sum`（serialize_math）：顶层非 Mix 但加法树含 Mix 叶——
  BinaryOp(Add/Subtract) 叶项化收集（符号沿 Subtract 右臂传播）：标量叶
  `scalar_value` 直读、裸 % 叶系数直读（scalar_value 无 ctx 盲区补面）、
  Mix 叶按 `mix_leaf_term` 塌缩项化（Expr 权重/布局依赖树/number 位
  Mix 不可项化 → None 原文透传）；同单位常量合并、% 项前置其余保序
  （FIXME(canonical)：完整单位序待全值域钉面）、单项落既有塌缩串。
  挂点 fold_dimension_calc Mix 臂后、% 叶门前——非 Mix 树 expr_contains_mix
  快速短路，零漂移。
- `mix_leaf_term` 抽臂（fold_mix_dimension 前两臂共用）：全零塌缩
  (0, 首项单位) / 标量塌缩 (值, 树首单位) 项化，行为与原臂逐位等价。
- 纯 % 值项字面权重混合符号解析：`calc-mix(10% 50%, 30% 50%)` 加权和
  (10·50+30·50)/100=20 保 % 维度（eval_mix 公式，used 值期才需基准）——
  scalar_value 对 % 叶两级 fallback 恒 None 的补面。
- `filled_literal_weights` helper：字面权重 clamp + 缺失填充表，归一化串臂
  preamble 去重共用。

## 2. 验证面

- 单测：calc_mix 12/12（新增 canonical 和串五形表逐字钉 `calc(0% + 10px)`/
  `calc(10%)`/`calc(10px)`/`calc(10% + 0px)`/`calc(20% + 10px)` + sibling
  守卫钉保留）；css-parser lib 3128 全绿、clippy 零告警。
- 语料：单页探针 serialize 66→76P（指定值 10 面全翻，computed 10 面留）、
  invalid 12→18P（45b769764 6F 清零对账验证）。全量账见 §4。

## 4. g15 权威账（results-g15，单二进制含 45b769764+本刀，test-guard 包裹）

- **css-values（对 results-g14b 基线）**：2,887 → **2,903 净 +16，页级与
  test 级零回归**（2 页增益/0 losses）。
  - calc-mix-serialize +10（本刀——外层 calc 嵌套指定值 10 面全翻：
    五形 × specified 双变体；computed 10 面留 §5）
  - calc-mix-invalid +6（45b769764 文法门收口对账验证——transform-origin-z/
    text-indent 两槽 × 三形，页内 18/18 全绿）
- 对账文件：results-g15/diff-vs-g14b.txt（本批 diff 输出存档）。

## 5. 挂账桶更新（serialize 残 52F + computed 页残 28F）

- sibling-index() 桶 37F（serialize 26 + computed 11）：需 DOM 桥
  sibling-index 求值——`5px * sibling-index() 0%` 权重 Expr 形 parse 期
  原样保留，求值期需兄弟序注入（跨 engine/css-parser 面）；
- computed/used 串面桶 43F（serialize 页 nest computed 10 + computed 直读
  串 16 + computed 页 17）：computed 通道不走存储文本透传（本轮实证——
  `calc(0% + 10px)` computed 期望原样也 Fail），需 style-system computed
  serialize 链（length_to_css 对 Calc 返空串域 + 单项 calc 解包 + 零项
  丢弃 + % 解析通道）——下一刀候选，style-system 面（apply/computed
  相邻，开工前核对并行流认领面）。
