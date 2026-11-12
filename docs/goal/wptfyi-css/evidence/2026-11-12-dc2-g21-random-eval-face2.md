# DC-2 g21 批：random() 播种求值全链（Face 2，compute 期元素级替换）

日期：2026-11-12　刀：g20 canonical 串化（Face 1）之上的求值兑现——CSS
Values 5 §9.4 Evaluating random()/random-item()：doc/element 双 seed sha256
PRNG base value + min/max/step 公式 + 指定值原位替换。g21pre 行为定谳
16P/169F → 探针两轮洞修复 **116P/69F**；六套件权威账 values 3,111 →
**3,214 净 +103 逐页零回归**（random-computed +95、random-item-computed
+5、random-item-nested +2、random-unresolvable-argument +1），五套件
Pass 级零漂移。

## 1. 实现面

- **求值器**（css-parser values/random.rs，+683 行）：`RandomEvalContext`
  {doc_seed: u64, element_seed: Option<u64>} + `SENTINEL` 常量（doc=0/
  element=None——支持性判定 seed 无关）+ `substitute_random_values` 全值
  扫描（引号状态机/词边界/括号配对；词假阳无害 Borrowed 透传）+
  `random_base_value`（cache name = doc_seed·element_seed·property·
  canonical key·ordinal 的 sha256 流 → [0,1) 均匀；`fixed <expr>` lite
  求值直取，element-scoped 无 element 视图 fail-closed None）+
  `eval_random_formula`（min/max/step 网格：`(max-min)·r` 网格吸附含
  FIXED_MAX epsilon 顶格、reversed 实参交换后 clamp、NaN/±∞ 关键字直通、
  step ≤0 忽略、step=∞ 退化 min）+ `serialize_random_result`（维度族
  canonical 串化；% 叶不可解析 → `random(fixed <R>, …)` **保留形**
  fixed-echo）+ `eval_random_item_call`（split_item_list 顶层逗号切分、
  ordinal 选中项整体替换、`:invalid` 形 None）。
- **cascade 写面/支持性门**（style-system cascade.rs + engine
  js_dom_bridge.rs decl_supported）：值含 `random` 词粗扫 → 哨兵上下文
  替换 → 按替换文本 apply/known_properties 裁决（合法性 seed 无关，
  g19 sibling 哨兵同型）；`random(fixed ` 保留形放行（compute 期求值
  承担）；None → 拒。
- **compute 期替换**（style-system lib.rs 步骤 4.7）：级联声明表逐项
  `substitute_random_values`（element 视图 seed）——全数值可解析 →
  原位改写级联文本（下游 apply 消费数值形，typed 字段照常承载）；
  保留形 → 存 [`ComputedStyle::random_computed`]（gCS 直读），级联保
  原文；custom property 跳过（§9.4 substitution 语义）。
- **style_cache 禁用 prescan**（`stylesheet_has_random`，g19
  tree-counting 同型）：random 声明页禁 style_cache——StyleKey 同键兄弟
  共享 ComputedStyle 而 seed 含元素标识，复用即 §9.4 sharing 毒化。
- **简写前 canonical 化**（`canon_random_inputs`）：omitted/auto key 以
  **原声明属性名**展开后再 expand_shorthands——longhand 视图展开会以
  longhand 名分键，`margin: V1 V2` 的 top/bottom 异 seed 破 §9.4
  sharing 属性分量同值语义（random-computed 语料 shorthand 桶钉面）。
  非简写声明幂等（单调用 ordinal 双路同 1）。
- **gCS 保留形直读**（engine computed_style.rs）：`style.random_computed`
  表优先回显（typed 字段未承载的 % 叶计算值唯一观察面；每元素独立）。
- 新依赖：css-parser `sha2`（workspace）、style-system `slotmap`
  （element as_ffi seed）。

## 2. 探针（results-g21pre/g21probe/g21probe2/g21probe3）

- g21pre（替换器未接线基线）：random-computed 13P/141F +
  random-item-computed 3P/28F = 16P/169F。
- probe1：92P/93F（+76）——暴露两洞：① gCS cache-hit 通道无
  random_computed 位（style_cache 禁用不彻底）；② 简写拆分后 longhand
  名分键（margin-top/bottom 异 seed）。
- probe2/probe3：**116P/69F**（对 pre 净 +100）——probe2 修复后稳定，
  probe3 复跑逐位一致（无顺序依赖）。

## 3. 验证面

- 单测：css-parser lib **全绿**（random 组新增：fixed 公式/step 网格与
  epsilon/reversed clamp/NaN±∞ 关键字/嵌套 fixed key/% 叶 fixed-echo/
  element-scoped fail-closed 与 sentinel/字符串字面免疫/嵌套 random-in-
  fixed 等 13 案）；style-system lib 全绿（+1：inline random-item color
  槽管线内最小重现）；engine lib 3,903 过（10 失败经 g19/g20 基线法证
  为既有：websocket 2/fetch 2/execcommand 2/img settle 2/cssom t49/
  iframesw 1，与本批无关）。
- clippy（css-parser/style-system/engine 定向，-D warnings）零告警；
  rustfmt 零漂移。

## 4. 权威账（results-g21，六套件全量，单二进制 provenance
zwg21resume-build.log 15:31 release 重建，service 模式 test-guard 包裹）

- **values：3,111 → 3,214 净 +103，零回归**（4 页增益）：
  - random-computed 13P/141F → **108P/46F（+95）**
  - random-item-computed 3P/28F → **8P/23F（+5）**
  - random-item-nested 0P/4F → 2P/2F（+2）
  - random-unresolvable-argument 0P/3F → 1P/2F（+1）
- 五套件 Pass 级零漂移：color 10,972 ±0、transitions 200 ±0、
  animations 238 ±0、easing 51 ±0、transforms 229 ±0。transitions 失败
  子测试计数差（+1139）为基线轮页面级超时截断伪影——
  properties-value-inherit-001/002 在 results-g20 各仅记 1 个 Timeout
  子测试（页超时未枚举），g21 全量枚举 560F+1T，两轮 Pass 均为 0、
  timeout 总数 21=21，非行为回退。

## 5. 残余桶（random 族挂账，供下批取用）

- random-computed 46F：跨元素同 key 等值/异值断言 ~14F（element-scoped
  seed 一致性细粒度）+ 非注册属性 CSS.supports 8F（offset-distance/
  shape-margin/stroke-dasharray/stroke-dashoffset/cx/cy/rx/ry——
  known_properties 未收 SVG 属性族）+ custom property random 2F + 散形。
- random-item-computed 23F：shim gCS 读面 raw 回显桶（`expected "5px"
  got "random-item(fixed 0, 5px, 15px)"`——写面存原文通道未经 compute
  替换的观察面缺口）+ color 槽 shim 黑值桶（`rgb(0,0,0)` 回退——shim
  plain 臂未接求值）+ color 单项 supports 2F。
- random-item-invalid 14F：写面校验不拒 malformed random-item（空参/
  auto 混入/括号畸形 `{`/`}` 残留——`random-item( )` 等原样存值）。
- random-in-if 6F / random-in-custom-function 8F / random-in-descriptors
  4F / random-in-media 2F / random-in-container-query 2F /
  random-item-in-container-query 1F（@规则 通道：media_text/descriptor/
  style query 未接 random 求值）/ random-in-animations 1F（页脚本
  TypeError）/ random-alias-property 1F。
- random-unresolvable 5F：unresolvable % → IACVT 期望 unset/initial
  黑值（现回落 inherited green / 50px）。
- D 桶 color 通道嵌套 random 3F（g20 挂账，随本刀不动）。
- 既有挂账不变：tree-counting 家族残 89F、anchor 953F、独立大桶
  （line-break-ch 194F/attr-all-types 103F/viewport-units 84F/
  calc-size 62F）、serialize-resolution 29F。
