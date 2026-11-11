# g14 批：CSS Values 5 calc-mix() 全链实现（css-values 语料轮 + 碰头记录）

日期：2026-11-12。刀：`6d3e3af16`（calc-mix 主刀）+ `c1a069a1c`（layout-engine 漏臂）+ `0f1c0639d`（minmax 回归修复，并行流收口含本流 fold 门）。

## 1. calc-mix() 实现面（6d3e3af16）

- **新模块 `values/calc_mix.rs`**：`CalcMixItem`/`CalcMixWeight` 类型、字面权重 token 界扫描（`<percentage [0,100]>` 范围门）、`expr_has_percentage_leaf` 权重文法门、`mix_math_type` §10.5 族一致（angle 叶 parse 期已 Number 化，以项值原文顶层角度单位补 provenance）、`eval_mix` 求值公式（clamp [0,100] / 缺失填充 `max(0,100-Σ)/n` / 分母 `max(Σ,100)`）。
- **types.rs**：`CalcExpr::Mix` 变体、`parse_calc_mix`/`parse_mix_args`、parse_factor 嵌套臂（depth 守卫）、eval/rewrite/lh/is_pure_time 臂；**parse_atom token 界补空白分隔**（calc-mix 值/权重对空格分隔，合法 calc 原子从不含内部空白，既有形态零漂移——单测回归面全绿实证）。
- **serialize_math.rs**：`fold_mix_dimension` 四级指定值折叠（Expr 权重原样保留 / 全零塌缩 `calc(0<首项单位>)` / 标量塌缩 / 归一化串——归一化除数按百分数口径 `w/(Σ/100)`）；fold_decl_specified `scale|opacity`(Number) + `width|text-indent`(Length) 槽接线。
- **style-system**：IMPLEMENTED_VALUE_FUNCTIONS +calc-mix（passthrough 转真文法门）；text_indent_length_is_valid Calc 臂收口 `calc_expr_is_length_percentage` 型别门（invalid 页 number/angle Mix 拒、`calc(5)` 误放行一并收口）；computed.rs calc_contains_percentage Mix 臂（仅项值含 % 阻塞布局期，权重纯系数不阻塞）。
- **单测**：css-parser +8（文法门/型别门/求值公式/折叠四级/挂账钉）、style-system +1（实现化文法门 + 既有 calc 形零漂移钉）。lib 3126+2517 全绿、clippy 零告警。

## 2. 漏臂与 provenance 事故（c1a069a1c）

layout-engine converter 两个 calc 谓词（contains_percentage/has_length）缺 Mix 臂 → zero-wpt-runner release 链编译失败。首跑 g14 误用旧二进制（构建任务管道吞退出码，`cmd | tail` 掩盖 cargo error），pass=2787 恰复读 g13b 基线——作废重跑。教训两条：①新增枚举变体后全链 `cargo check` 须覆盖 layout-engine；②后台构建禁管道吞退出码，`echo "exit: $?"` 落日志。

## 3. g14 语料首账（594 案全量，diff-vs-g13b.txt）

**2,787 → 2,882 净 +95**：calc-mix-serialize +51（128 案 66P，剩 62F 为挂账面：sibling-index() 形、外层 calc() 嵌套 % 排序）、calc-mix-computed +25（自一致比较全链通）、calc-mix-invalid +12（12 拒形全绿）、minmax-length-serialize +8（text-indent Length 臂翻正）。

**回归 -1**：minmax-length-percent-serialize 4 行 P→F（`min(20px, 10%)` 等指定值原样保留形）——根因：text-indent 新 fold 臂暴露 `eval_calc_with_context` Min/Max `filter_map` 静默丢不可解析实参（% 叶无 ctx → None 被丢 → `min(20,10)` 误可解析塌缩 `calc(20px)`）；基线因 text-indent 无 fold 臂偶然 Pass（width 槽同缺陷未被语料测到）。

## 4. 碰头信号与双修叠加（规则 9 记录）

本流在 fold_dimension_calc 加「有限 % 树不标量塌缩」防御门（scalar_value 层拦）；并行流同 clone 同期定位同一根因、修 eval 共享路径（Min/Max 全实参参与 `?` 传播）+ 回归钉（infinity_nan +18 行）并 staged。两修叠加 3127 全绿、语义互补（eval 语义正确化 + fold 防御纵深），并行流认领整批提交为 `0f1c0639d`（含本流 fold 门），本流不再重复提交。

单页复核（0f1c0639d 二进制）：4 行 specified P→F 全部翻回 Pass，minmax-length-percent 页回到基线形态（computed/used 残 Fail 为基线既有）。

## 5. 复核轮交接与 g14b 权威账

并行流已跑完 `zw-g14b-values`（css-values 重跑，594 案全量，0f1c0639d 单二进制）+ `zw-g14b-six`（六套件对账批）——权威账以 g14b 为准。

**values 权威账（results-g14b/diff-vs-g13b.txt）**：2,787 → **2,887 净 +100，
页级与 test 级零回归**（5 页增益/0 losses）——高于首账预期 +96 的 +4 为
minmax 修复红利：minmax-length-percent-serialize +3（`min(1px)`/`max(1px)`/
`min(1px, 2px)` specified 塌缩行，混 % 形回归同步清零）、minmax-length-
serialize +8→+9。calc-mix 三页与首账一致（serialize +51/computed +25/
invalid +12）。computed 页残 28F 二分：sibling-index() 形 ~10（并入 DOM 桶，
两页合计 ~36F）+ used-value serialize 空串域 ~18（`expected "0px" got ""`，
与 serialize 页 computed 串面同域，合计 ~34F）。serialize 残 62F 三分：
sibling-index() 形 ~26（DOM 桥求值）、
外层 calc() 嵌套 % 排序 ~20（树递归 serialize）、computed 串面 ~16（em 等
布局依赖项 computed 直读/全零形）。invalid 残 6F 精确归类：transform-origin-z/
text-indent 两槽 × number 族/angle 族/%+number 混族三形——scale 槽 12 拒形全绿
反证解析层门已生效，缺口在 shim CSSOM e.style 直设路径的属性槽型别门
（style-system apply 门被绕过），修复方向为 css-parser 槽位 fold 的 family
收敛（Length/Number/angle/time 槽各按 mix_math_type 比对）。
注：values JSON 事后被 six 批 values 段 merge，217 路径出现 ×2 重复条目、
transcript-css-values-g14b.log 被其段覆盖——对该 JSON 再 diff 必须按路径
去重取条目，权威数字以 diff-vs-g13b.txt（base 2787 / new 2887 / +100）
为凭。
六套件零回归对账由 g14b-six 轮收口。

## 6. g14b-six 六套件对账收官（Result=success，results-g14b/diffs-vs-g13.txt）

余五套件对 results-g13 批账逐页零漂移：color 10,972 ±0、transitions 200
±0、animations 238 ±0、transforms 228 ±0、easing 51 ±0——本批共享路径
改动（parse_atom 空白 token 界、Min/Max eval 全实参、fold % 叶树门）对
五套件语料零扰动，批终 **净 +100 零回归**。
