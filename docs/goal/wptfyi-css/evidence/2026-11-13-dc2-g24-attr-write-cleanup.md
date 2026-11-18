# DC-2 g24 批：attr() 值支持 + CSSOM cleanup null 臂根因修复

**日期**：2026-11-13
**范围**：css-values-5 §attr-notations parse/替换/级联门 + cssom-1 setProperty null 语义
**提交**：9114bef7e（rebase 于 peer a298e4ca2 之上）

## 改动面

1. **css-parser `values/attr.rs`（新增，1033 行）**
   - `scan_attr_calls`/`attr_calls_valid`：调用扫描与文法裁决（空名/`!`/`&`/尾管
     union/未知 `<T>`/类型 token 双节/三实参 invalid；乘数须紧贴分量——
     `type(<length> +)` 文法拒、`type(<length>+)` 合法，语料 174 钉）。
   - 型别选择子：DefaultString（缺省带引号串）/Unit（`attr(x px)` 裸数字归一
     number+unit）/Syntax（`*`/`<length>`/`<percentage>`/`<number>`/`<integer>`/
     `<angle>`/`<time>`/`<frequency>`/`<string>`/`<color>`/`<custom-ident>`/
     `<url>`/裸 ident/union `|`/乘数 `+`）。Angle/Time 维度值臂接
     `parse_dim_unit` + math 折叠（`fold_math_value` Angle/Time）。
   - `eval_attr_call` + `substitute_attr_values(_checked)`：compute 期元素级
     文本替换；属性缺失/型不合且无 fallback → `None`（IACVT）；taint 标记
     （替换产物含 `url(` token 或带 scheme 串）。
   - **fallback 型校验**：维度型（length/percentage/angle/time/frequency）
     拒 numberish（裸 f64 或 `fold_math(Number)` 成功——语料
     `type(<length>), 30`/`type(<length>), calc(10 + 20)`/`type(<angle>),
     calc(10 + 20)` invalid 钉面）；其余原文透传（`type(<percentage>), 10px`/
     `type(<angle>), italic`/`type(<custom-ident>), 3px` 钉面）——不合形交
     cascade 按属性值文法裁决（解析失败与 IACVT 同为 unset 语义）。
   - `attr_target_fetches_url`：§attr security 取 URL 属性清单
     （background/background-image/border-image/list-style-image/cursor）。
2. **style-system 预级联块（lib.rs）**：全源声明（hints/matching/host_matching/
   slotted_matching/inline_decls）attr() 替换闭包 + 取 URL 属性污染丢声明；
   reader = `doc.get_attribute(element, name)`（compute 期活性）。
3. **cascade.rs `is_cascade_value_valid`**：含 attr() 调用值——文法 valid 放行
   （延迟 compute 期裁决）、invalid 级联丢（写面 validate_decl_value 对未实现
   函数原文放行，此门收口）。
4. **CSSOM cleanup 根因修复（本批最大杠杆）**
   - shim `_styleProxy` sel/handle 臂 `setProperty`（part03.js）：`String(null)`
     → `"null"` 四字符 → 值校验拒 → **声明永移除不掉**。修为
     `String(value == null ? '' : value)`（WebIDL nullable `CSSOMString? = ""`，
     cssom-1 setProperty 空值臂 ≡ removeProperty）。
   - native `native_style_set_property_invoke`（css_style_declaration.rs）同病
     （`string_arg` v8 ToString(null)），改用既有
     `local_value_to_string_null_as_empty`。
   - `_zwMStyleProxy` 物化面原有守卫，无需改。
5. **写面门机制（尝试后撤回）**：中期曾实现两阶段写面门
   （`__zw_attr_decl_names` 报名单 → shim 拼属性包 → `__zw_attr_decl_gate`
   裁决拒写 no-op），插桩取证（[ZWDBG gate] 155 行）定谳：
   - 门读数准确（无 stale read——158 bag=`10`/161 bag=`10 ms` 全对）；
   - 但 32F 广泛回归：`type(<time>, <length>)` 逗号多型语法 scan 失败 → bag
     空 → 误拒（159/161）；值级 mismatch（文法合法）拒写打破 accept-then-drop
     期望（56/20/22/23/42/44/50）；
   - **语义定谳**：spec 区分 parse-time invalid（文法错 → 拒写 no-op）vs 值级
     IACVT（文法合法值不合 → 存 raw → cascade 丢 → unset）；语料
     test_invalid_attr 捕获写前值 + 尾部 `setProperty(property, null)` cleanup
     ——cleanup 修复后每测试起点干净，两语义收敛于 clean-state，无需写面门。
   - 机制整体撤回（callbacks.rs 两回调/part03.js 两臂+helper/css_style_
     declaration.rs 分支/attr.rs `attr_decl_names`+`attr_decl_write_gate`），
     净回退零残留。

## 探针证据（diag 页，不入库，跑后删除）

`attr-zw-state-probe.html` 链式取证（修复前 → 后）：
- 修复前：`setProperty('width', null)` 后 computed 仍 10px（p2b Fail）、
  `removeProperty` 正常（p3b Pass）、`setAttribute('style','')` 正常
  （p5b Pass）——三角定位 shim setProperty null 臂。
- 修复后：p2b Pass（清理生效）；p1/p2… attr-state 打印断言 Fail 为设计
  （expected 形恒不匹配以输出实际值；p1 的 `getAttribute('style')` null 是
  shim R3199 LW replay 读面独立 quirk，与本批无关，挂账）。

## 五面语料核验（vs g24attr4 基线，零回归）

| face | 修复 | 回归 | 残余 |
|------|------|------|------|
| attr-all-types | +14（58/62/74/128/130-136/155/163/174） | 0 | 19（12/14/15/16/24/26-30/59/60/73/77/78/81/126/175） |
| attr-cycle | +7（1/4/5/7/8/19/24） | 0 | 15 |
| attr-security | 0 | 0 | **0（23/23 满血维持）** |
| attr-css-wide-keywords | 0 | 0 | 14（css-wide keyword fallback 面，未实现特征） |
| attr-IACVT | 0 | 0 | 1 |

残余 49F 全部为 g24attr4 轮已有（fixed/regressed 对账零交叉），归类：
- attr+var 延迟组合（attr 值内 var() 需代入后重裁——73/74/77/78/81 + cycle
  多页）；
- EOF 括号恢复（`attr(data-foo type(<color>` 未闭合——59/60）；
- DefaultString font-family computed 通道（12/14/16——computed 显示 raw 串，
  替换未达 font-family 计算面）；
- 单位型合法性（`attr(data-foo xx, 3px)` 未知单位应 fallback——126）；
- css-wide keyword fallback 面（css-wide 14F）；
- `--x` type(<url>) IACVT 面（175）。

门轮全量对照（g24gate，已撤回）：attr-all-types 145/177（32F）——本批
153/177，净回撤门机制 +18。

## 单测与门禁

- css-parser lib 3,184 全绿（attr 106，含 fallback 矩阵/乘数空白/verbatim
  透传新钉）；style-system lib 2,528 全绿。
- clippy -D warnings 零告警（css-parser/style-system/engine）；rustfmt clean
  （收口 attr.rs/mod.rs 机械漂移）。
- lei-pre-commit-guard PASS（6 文件 1184 行，安全发现 0，文件提醒 0）。

## 全量对账（values 256 页，vs g23b 基线）

**values face 3,553 → 3,735 净 +182，零回归页**（256 页逐页对账：12 页
全涨 / 50 页持平 / 0 页回落；测试总数恒 5,593）。JSON
`.acceptance/wptfyi-css/results-g24full-values/`，transcript
`/tmp/zw-g24-full-values-*.log`。

增益页清单（cleanup null 修复的共享路径红利实证）：

| 页 | 增量 | 归因 |
|----|------|------|
| attr-all-types | +86（72→158） | attr() 全链 + cleanup |
| calc-dimension-serialization-order | +44（0→44） | cleanup null（页 harness setProperty(prop,null) 逐断言清理） |
| calc-serialization-002 | +17（2→19） | cleanup null 同上 |
| attr-cycle | +16（1→17） | attr() 全链 + cleanup |
| attr-pseudo-elem-invalidation | +6（0→6） | cleanup null |
| attr-css-wide-keywords | +3（4→7） | cleanup null |
| attr-argument-grammar | +2（11→13） | cleanup null |
| attr-length-specified | +2（0→2） | attr() + cleanup |
| attr-security-transition-interpolated | +2（1→3） | cleanup null |
| if-cycle | +2（6→8） | cleanup null |
| attr-invalidation / attr-serialization | +1/+1 | cleanup null |
