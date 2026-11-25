# kc5r126 · 函数求值端 attr() 替换＋attr-security taint 闭环刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 458（b0 实测）→ 467（+9）**＝judge VERDICT PASS＋
verify VERIFY-PASS＝预注册通过 P2F=0、GONE=NEW=0。净 +9＝
function-attr **6 键**（本刀主域 css-values-5 §attr-notations：result 直
形 untyped/typed ×type(<length>)/type(*) 4＋「attr() in default
parameter value」1＋「attr() in local variable」1，全
`expected "42px" but got "784px"` 形）＋local-attr-substitution **3 键
连带真收益**（「attr() fallback substitutes locally」／「attr() cycle
through unused fallback in local」／「attr() cycle through
function」——attr() 产物 `var(--x)` 经既有 var 链递归落地／attr 缺席
invalid 沿调用链上传走 var fallback，与刀域同源语义）。43 页 521 键恒
同、Timeout 恒同。

## 刀形

css-values-5 §attr-notations × §attr-security ＝函数体三域 attr() 文本
级替换＋taint 传播闭环：

**三域 attr 臂（custom_function.rs，替换序 var→attr）**——①参数缺省域
`resolve_default_value`（签名加 attr_reader、返回 (String, bool)，var
链后 attr 臂；文法 invalid → SCOPE_IACVT 遮蔽）；②局部声明域（untyped
链 var 后，**文本级替换**——嵌套 dashed 调用实参内形一并覆盖；None →
IACVT 遮蔽 continue）；③result 域（var 链后、块产物门前；None →
Ok(None) 调用 invalid，k119 同向）。「attr() cycle through function」
键定谳：`--g()` result `attr(data-f type(*))` 属性缺席无 fallback →
调用 invalid → `--local` 嵌套 Invalid → IACVT → result var Err →
`--tmp` 表移除 → `var(--tmp, PASS)` 走 fallback——invalid 上传链与
WPT 期望逐环吻合。

**taint 通道（DashOutcome::Value(String, bool)）**——css-values-5
§attr-security：attr() 产物携带 url → 污染，自三域替换点逐层 OR 合并
上传（实参递归 args_taint＋evaluate 返回三元组 eval_taint＋产物重扫／
局部嵌套展开合并＋尾返回累积）；evaluate 签名加 args_tainted（实参文
本经 var(--a) 引用进产物）。

**顶层消费臂（lib.rs 4.755 站）**——`Value(sub, fn_tainted)` 解构，
`fn_tainted && attr_target_fetches_url(prop)`（background/-image/
border-image/list-style-image/cursor）→ fn_cycle_props 级联移除＝
IACVT 继承回落——与 4.75 attr 重裁站同口径。url-tainted 8 键守成：
k125 观测＝产物原文残 attr 词残臂回落（级联移除→继承回落
Url("parent")），k126 换真替换＋taint 拦截（产物 url(cat.png)→taint
臂→同回落），观测同向零翻。custom property 消费域 taint 穿透（var()
间接消费重裁）FIXME 挂账。

**k125 残臂收窄**——taint 臂插 `prop.starts_with("--")` 之后、残臂之
前；残臂（`has_attr_word || has_dashed_function_call`）余域＝不可解残
形（未注册名嵌套残等），k125 挂账句（taint 随函数内 attr 求值刀）兑
现摘除。

## 门禁

- css-parser **3415P/0F**；style-system **2717P/0F**（2709 基＋
  function_attr_substitution 新模块 8 钉＝result 直形 3＋default
  parameter＋local variable＋url-tainted IACVT 继承回落＋嵌套调用实参
  ＋argument 直形守成）；engine **4310P/3F**（既有对端红名单恒同
  s330/expando/p99s）；clippy 0＋fmt 0（首轮 3×E0599＝Cow::Owned 解构
  String 冗余 into_owned＋fmt 折行——`>file; RC=$?` 直取收敛）。
- 钉测单跑 `cargo test --lib function_attr` → 8 passed。

## 探针与电池（一代）

gen19 v1＝**d659b9cf7c604cd00ea32aac3c14cb93**（指纹 ZW-KC5R126-FNATTR-
FACE=1＋CARRY-K125=1，首建即定档）。探针 43p/521k {Pass 467/Fail 53/T 1}
vs k125 b1 终态差分 **F2P=9 P2F=0 GONE=NEW=0**（registry 程序化生成，
judge/verify 预注册按定谳更新 F2P=9/HELD_F=14）。电池 b0＝k125 归档
gen18 v3 d683c575 rc0=0（**b0≡k125-b1 逐键恒同 True** status+message
521 全等 pristine 458）＋b1＝gen19 rc1=0（**b1≡probe 逐键恒同 True**
521 含 message）＝census b0 {P 458/F 62/T 1}→b1 {P 467/F 53/T 1}。
judge G1-G5 全门（G1 legs 0/0；G5 521==521 GONE=NEW=0；P2F=0；held-F
14/14＝k125 carry 10＋local-attr 残 4 新列；红线 4/4；G3 F2P 9/9 精确
±0 集外零；G4 SET-IDENTICAL 零扰）＋verify (9,9,0) 逐位＝净恒等式
+9==9−0。

## 挂账池（非本刀域）

- local-attr-substitution 残 4（「var() in attribute value substitutes
  locally」×3＝attr() 产物残 var 词需**替换循环二趟 var** 域＋
  「attr() cycle through local」＝attr 自指环局部 scope_out 解析口径
  ——产物 `var(--x)` 文本入表，环检测在 var 代入时触发走调用 invalid
  而规范期望 `var(--y, PASS)` fallback 生效）
- if() 域 taint 穿透（if_function.rs sub_operand attr 臂 `_tainted`
  弃用）——判据页无 if()+attr 组合，FIXME
- custom property 消费域 taint 穿透（var() 间接消费重裁）
- typed 局部域 attr 替换缺位（eval_typed_value 直接强制失败 → IACVT
  遮蔽，安全向同观测）
- resolve_default_value inherit 分支产物不补 attr 重裁
- 其余 carry：invalidation 4（CSSOM 域）＋eval 残 2（revert-layer/
  revert-rule 真级联回溯域）＋shadow-dom 3＋cycles 4＋local-if-substitution
  inherit 残 1＋m5 裸声明覆盖序＋@container 裸 not 前缀＋animations 24
  ＋narrowing＋DC-3 对账周账 12-01
