# kc5r119 · `<dashed-function>` 求值端会话作用域刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 363 → 404（+41）**＝judge VERDICT PASS＋verify
VERIFY-PASS（预注册硬码一次性通过）＝**P2F=0、GONE=NEW=0**。套件域账，
引擎总账不动（页外域零扩散）。

## 刀形

css-mixins-1 §3 substitution／§2.3.1 shadowing／§4 执行模型求值端＝
eval 55F 靶面清 37＋cycles 8F 靶面清 4（51F→41F2P，held 22）＝
style-system 三触面（css-parser/engine 零改——写面 k118 已拒收非法形，
本轮全形合法可达求值）：

**作用域穿线（`custom_function.rs` 重构）**——
`substitute_dashed_functions` 参数 customs→scope 改名：调用方作用域
动态穿线（嵌套调用以 caller 已解析作用域展开＝「Substitute local from
outer scope」判例族）；args 预扫描递归传 scope；evaluate 返回
`(product, inner_scope)` 产物重扫以 inner_scope 为基座；未注册名仍保留
原文。三层视图：`view`（原始链解析视图，序无关）／`raw_scope`（if()
会话 k112 语义保持）／`scope_out`（已解析产物）。

**绑定期实参解析（M2）**——4.5 var 站不解 custom 属性（读期才解）→
实参 var() 须绑定期调用点代入（锚=consuming_prop，对 incoming）；
IACVT 实参→缺省→SCOPE_IACVT 标记（不 invalid）；缺席实参→缺省→
无缺省→`Ok(None)`＝整调用 invalid（`DashOutcome::Invalid` 新变体）。
块实参整体单块解包（css-values-5 component-function-commas，首块配对
闭括号须在末位才剥壳）。缺省在 param_scope 解析＋CSS 宽关键字语义
（`classify_css_wide_keyword`：initial=IACVT 遮蔽走回退；inherit=
调用方作用域取值；unset/revert/revert-layer/revert-rule=invalid）。

**局部语义**——CSS 宽关键字局部：initial=移除回落参数层（scope_out
同名参数绑定回插）＋inherit=调用方取值＋其余=SCOPE_IACVT 遮蔽。
局部解析管线：typed→if() 保持→var 锚定代入→call-scan 穿线→关键字
收尾；以全量原始视图为 var 链基础（声明序无关）。

**SCOPE_IACVT 遮蔽标记（`computed.rs`）**——`\u{0}zw-iacvt\u{0}`
（NUL 边界）：`resolve_var_reference_at` 截臂——标记键按缺席处置
（fallback 或 None）但**遮蔽外层同名**（不沿链下探），与键真缺席
分叉。标记只存在于函数会话作用域，元素级计算表不含。

**站点（`lib.rs` 4.755 站）**——`DashOutcome::Cycle | Invalid` 同臂
→fn_cycle_props→消费声明 IACVT。

**指纹**：`css_wire.rs` `KC5R119_FN_EVAL_SCOPE_FACE`（ZW-KC5R119-FN-
EVAL-SCOPE，#[used] 防 DCE）＋CARRY-K118 线承。

## 钉测（12 净增）

style-system custom_function 测试模块：2 旧钉升级 Invalid 语义改断言
（`missing_arg_without_default_kept_and_blank_args_dropped`→
`untyped_param_missing_is_invalid` 族）＋10 新钉——缺 result 描述符
Invalid＋嵌套调用 sees caller local＋隔帧局部可见/帧隔离＋双序全量
原始视图基座＋IACVT 局部遮蔽跨函数链＋绑定期实参解析（M2 var 代入
锚=consuming_prop）＋IACVT 实参兑缺省＋标记遮蔽 fallback 胜＋块实参
解包＋CSS 宽关键字局部语义 4 关键字矩阵＋关键字缺省语义矩阵。

## 门禁（三代腿）

- 首轮 fmt=1（折行漂移 10 处纯格式）→cargo fmt 落盘复检 0。
- 门禁腿：css-parser lib **3415P/0F**＋style-system lib **2653P/0F**
  （2642 基＋k118 钉 1＋k119 净增 10）＋engine lib **4306P/3F**＝既有
  红精确对号（s330/expando/p99s，名单与 k118 腿恒同——非本刀引入）
  ＋clippy **0**＋fmt **0**。

## 探针定谳（一代）

gen12（md5 `4812766de73f14b0337e6f4a8ec8d836`，指纹
ZW-KC5R119-FN-EVAL-SCOPE=1＋CARRY-K118=1）全语料单腿（43 页 521 键）
对 k118 b1 终态逐键差分：**F2P=41、P2F=0、GONE=NEW=0**＝
dashed-function-eval **F2P=37**（M1 穿线 8＋M2 绑定期 4＋M3 无效调用
3＋M4 块解包 6＋M5 遮蔽 2＋M7 关键字 14——M 簇预测精确全翻）＋
dashed-function-cycles **F2P=4**（预测外——逐键机制归因成立属求值域
内：`Cyclic defaults` 缺省互环→param_scope 解析 Cycle→IACVT 回退；
`Cycle through unused local` 未用局部自调用→call_stack Cycle→
Invalid→var fallback；`Cyclic outer --b shadows custom property`
局部自引 invalid→遮蔽标记截臂；`Local shadowing cyclic property
--x` 局部遮蔽元素环键→会话内 fallback）。registry 41 键程序化生成
（f2p-registry-r119.py，防转写漂移）。探针窗 363→404。

## 电池双腿

- b0＝k118 归档 bin `3155ad3da3a916da8c89f6e0edbdea3a`
  （ZW-KC5R118-DFN-VALID-FACE）；rc0=0。
- b1＝gen12；指纹=1＋CARRY=1；rc1=0。
- **P0=363 pristine 双证**：b0 重跑 JSON 与 k118 收账 b1 **逐键恒同**
  （程序化全量比对 True）；b1 JSON 与探针产出**逐键恒同**（同 gen12
  bin 两独立轮全等 True）；P1=404 净 +41。
- status census：b0 {Fail 157/Pass 363/Timeout 1} → b1 {Fail 116/Pass
  404/Timeout 1}；43 页 521 cases 双侧恒同；Timeout 键恒同。

## 判官（judge-r119.py 预注册硬码）

- G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0。
- G2 P2F total=0 out-of-exemption=0（豁免集空）；held-F 6/6 持
  （shadow DOM 3＋invalidation 3，两臂皆 Fail）；红线 4/4 持
  （apply-top-level/mixin-basic/shadow 隔离/mixin-cycle）。
- G3 F2P observed=41 knife=41/41 missing=0 out-of-registry=0（registry
  内插）。
- G4 non-exempt pages diff=0；NET +41==+41 恒等式。
- **VERDICT PASS**（一次性通过，预注册零修订）。

## verify（verify-r119.py 独立复核）

恒等式 P1−P0==F2P−P2F（+41==41−0）＋关系预测 (41,41,0) 逐位命中＋
presence 521==521＋pages 43==43＋status 值域 census 净＋Timeout 恒同＝
**VERIFY-PASS**。

## 新基与挂账

新基 css-mixins P=404。刀域残 held-F＝eval 18（M6 typed 求值 12：
calc 化简/单位归一/type check＋M8 元素级 CSS 宽关键字 6）＋cycles 4
（真循环检测域：property cycle 观察形）＝k120+ 候选。conditionals
17F＋attr 10F 本刀未触全 held（G4 SET-IDENTICAL 证明页级零扰）。
挂账池更新：mixin-invalidation 3（mutation 生效形 held）＋
mixin-shadow-dom 3（carry，shadow 级联面缺位）＋@apply 嵌套块
FIXME＋其余 carry：from-import-media 条件假形＋m2c/m8/m15＋gCS
IACVT ''/absent 统一刀＋animations 24＋narrowing＋DC-3 对账周账
12-01。

脚本/结果驻 `.acceptance/wptfyi-css/k119/`（gitignore 不入库；bins；
/tmp/zw-k119/ 同灶＋probe-b1-full/results-k119-b{0,1}＋battery-k119.sh/
probe-r119.sh/judge-r119.py/verify-r119.py/f2p-registry-r119.py/
build-r119.sh/gates-r119.sh）。
