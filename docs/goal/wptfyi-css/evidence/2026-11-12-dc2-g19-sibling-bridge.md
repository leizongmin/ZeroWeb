# DC-2 g19 批：sibling-index()/sibling-count() DOM 求值桥 + 写面文法/串化收口

日期：2026-11-12　刀：g16 TreeCounting 叶（parse 期 opaque、探针 (1,1) 哨兵）
之上的 DOM 桥注入求值三面切片——css-parser eval/串化/counter 变体（Part A）
+ style-system sibling 线程/style_cache 禁用/写面门（Part B）+ engine gCS
三通道 sibling 注入（Part C）。两轮探针：probe1 49P/96F 暴露 cache-hit 通道
fail-soft '' 与 parse-fail 放行两洞，修复后 probe2 **56P/89F** 恰中预测。

## 1. 实现面

- **求值契约**（types.rs）：`CalcContext.sibling_ordinal/sibling_count:
  Option<u64>`——有 ctx 出值、无 ctx fail-closed None（g16 语义不变）。
- **Part A（css-parser）**：`calc_expr_has_tree_counting`（Mix 臂 false——
  serialize 走 fold_mix_dimension 原口径）；`tree_counting_slot_valid` 槽族门
  （MTNumber→scale/opacity/z-index/rotate，MTLength→width/margin-left/
  text-indent/transform-origin-z/left，MTTime→transition/animation 时族，
  MTAngle→rotate）；`parse_counter_list_with_sibling`（counter-reset/increment
  整数 token 字面或 math eval）；serialize `fold_tree_counting_calc`（乘法链
  因子展开、单 opaque 因子殿后、静态积 opaque→1 掩码求值——Angle deg
  provenance/Time 秒/Number 6dp、Length 位单静态原子保维度 canonical）。
- **Part B（style-system）**：`sibling_position(doc, element)`（元素子 1 基序
  +计数，文本/注释不计、根 (1,1)）；`apply_property_value_with_quirks` 尾参
  sibling（order/z-index/兜底臂真值求值）；写面门 `validate_decl_value`
  sibling 臂（白名单检查之前——表外槽 left 同收口）：math 形（calc 前缀/裸
  函数前缀）parse 失败整拒（带参/非时槽 time 叶），parse 成功走槽族门，非
  math 形放行（`content: "sibling-index()"` 字符串字面合法）；probe (1,1)
  哨兵接线 cascade/is_cascade_value_valid + supports matcher + CSS.supports。
- **style_cache 毒化禁用**：`stylesheet_has_tree_counting` prescan（@keyframes
  不扫——keyframe 页挂账）→ cache_safe AND 禁用（StyleKey 同键兄弟共享
  ComputedStyle 毒化）；每元素 sibling 计算由 `cascaded_has_tree_counting`
  （var() 代入后检测，覆盖 inline/UA）门控——零新 StyleSystem 字段。
- **Part C（engine gCS 三通道）**：by-selector（computed_style.rs :275）+
  computed_style_cache（:979）+ callbacks lazy 通道（:2424）均带
  `sibling_position(doc, node)`；`serialize_computed_property_with_sibling`
  inset 槽（left/top/right/bottom）Calc 树含 sibling 叶 → 注入 ctx 求值串化，
  其余委托主通道零漂移。

## 2. 两轮探针与修复（probe1 → probe2）

probe1（family 23 页 filter=tree-counting）：49P/96F（+18 对 g16 基线
31P/114F），两洞暴露：

- **gCS cache-hit 通道 fail-soft ''**：callbacks per-selector map 缓存
  ComputedStyle 无 sibling 位——populate 次（miss→lazy 通道）带 ctx、复用次
  （hit→`:2403` serialize_computed_property 裸通道）无 ctx——tree-counting
  页 #test 连续 4 次查询仅首查带 ctx，`left: calc(10% + 100px *
  sibling-index())` 得 ''（原注释「sibling 叶形态此处不可达」假设证伪）。
  **修复**：map 值附 `Option<(u64,u64)>` sibling 位（populate 时
  sibling_position 一次，复用次 O(1) 透传）。
- **写面门 parse-fail 放行洞**：带参 `sibling-index(100px)`/非时槽
  `calc(1s * sibling-index())` 经共享 calc 入口 parse 失败 → 原门放行 →
  shim 存原文（parsing 页 6F）。**修复**：math 形（calc 前缀/裸函数前缀）
  parse 失败整拒；非 math 形（引号字符串字面）放行防 content 误拒。

probe2（修复后同 filter）：**56P/89F（净 +25 对 g16 基线，零回归）**——

- calc-sibling-function-parsing 25P/22F → **45P/2F**（+20；残 2F =
  `color(srgb calc(1px * sibling-index()) 0 0)`/`oklch(...)` 实参槽——
  color() 型别门挂账）
- calc-sibling-function 0P/7F → **5P/2F**（+5：#test z-index '2'/
  counterIncrement 'foo 5'/left 'calc(10% + 200px)'/root z '2'/root order
  '2'；残 2F = 伪元素/嵌套伪元素断言——shim gCS 忽略 `_pseudo`（除
  ::backdrop），Face 3 挂账）
- 其余 21 页（keyframes 10/descriptors/invalidation/container-query/shadow/
  tree-scoped/gradient/trig）逐位持平挂账

## 3. 验证面

- 单测：css-parser lib 3,146 全绿（sibling_tree_counting_tests 10 组：eval
  真值/无 ctx fail-closed/Angle 静态积 4 形/Time 规范/Length 原子回显/空白
  归一/边界保原文/槽族门矩阵/fold_decl 分派）；style-system 2,525 + 5 全绿
  （sibling_position 序/根 (1,1)/树计数页端到端 z_index Integer(4)+order 10；
  t54 写面门随 parse-fail 拒面更新——带参/Time 叶翻拒、非 math 形放行钉）；
  engine lib 3,903 过（10 失败经 stash 基线法证为**既有失败**与本批无关：
  websocket 2/fetch 2/execcommand 2/img settle 2/cssom t49/iframesw 1）。
- workspace guarded-clippy 零告警（collapsible_if 一处修复后）。

## 4. 权威账（results-g19，六套件全量）

- **values（对 results-g17b 基线）**：3,032 → **3,072 净 +40，零回归**。
  增益 4 页：tree-counting/calc-sibling-function-parsing +20、
  tree-counting/calc-sibling-function +5（§2 两页）+ **calc-rounds-to-integer
  +12**（counter-increment/reset `foo calc(10.1|1e1|...)` 与 order
  `calc(10.1)` 整数槽 math 形——Part A counter 变体/order 臂整数舍入外溢
  增益，§tree-counting 之外页面）+ **hypot-pow-sqrt-computed +3**
  （`sqrt(sibling-index())`=2/`sqrt(pow(sibling-index(),2))`=4/
  `hypot(3, sibling-index())`=5——嵌套数学函数 sibling 叶 gCS 注入真值）。
  对账文件 results-g19/diff-values-vs-g17b.txt。
- **五套件零漂移（对 results-g16 基线）**：color 10,972 ±0、transitions
  200 ±0、animations 238 ±0、easing 51 ±0、transforms 229 ±0——页级零
  回归、无未解释漂移。对账文件 results-g19/diff-{color,transitions,
  animations,easing,transforms}-vs-g16.txt。
- 二进制 provenance：probe2/全量账同轮二进制（epoch 1794450666，g19 全
  改动含 cache-hit/parse-fail 拒面两修复；build log zwg19b-build.log）。
  probe1 出自 epoch 1794449975（两洞修复前，对账对 probe1 保留）。


## 5. 挂账桶更新

- color() 实参槽型别门（parsing 页 2F + color 通道挂账桶）
- shim gCS 伪元素通道（Face 3）：主页伪元素 2F/嵌套伪元素 marker 断言
- keyframe 家族 10 页（sibling-index-keyframe-*-dynamic + registered-
  properties 20F）：@keyframes 页内 sibling 求值——stylesheet prescan 不扫
  keyframes、keyframe apply 通道未接
- invalidation/container-query/descriptors/media-query/shadow-dom/tree-scoped
  /gradient-gcs/trig 各页：DOM 变更后重算/容器查询/@规则求值面
- 范围门哨兵假阳（font-weight 100-900 probe (1,1) 假阳——在册）
