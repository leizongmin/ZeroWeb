# kc5r120 · `<dashed-function>` typed 求值域刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 404 → 417（+13）**＝judge VERDICT PASS＋verify
VERIFY-PASS（预注册硬码一次性通过）＝**P2F=0、GONE=NEW=0**。套件域账，
引擎总账不动（页外域零扩散）。

## 刀形

css-mixins-1 §3.1 typed 求值域＝eval 12F＋parameter-types 1F（13 键，
k119 收账 held 池逐案解码归簇：A 簇 typed 强制 10＋C 簇块产物 2＋typed
local IACVT 1）＝style-system `custom_function.rs` 单触面（写面 k118
已置位、engine 仅加指纹；k119 作用域穿线全承）：

**类型强制层**——`eval_typed_value`（stage1 env+var+random 预代入；
stage2 `parse_type_spec`：`type(...)` 剥壳、`+` 后缀 multi、五单型
`<length>/<number>/<angle>/<time>/<color>`，未知型 None 保守透传零
P2F）＋`typed_component`：Length=`parse_length` 先行、None 回落
`parse_calc`→`eval_calc_with_context` calc 化简＋px canonical（整数
无小数点）；Angle=`parse_angle_to_radians` 归一 deg（1turn→360deg）；
Time=`parse_time_token` 归一 s（1000ms→1s）；Number/Color 校验透传
不 canonical（零漂移——现有 Pass typed 形 plain token 恒同）。
`type(*)` 万能型豁免（保 'Untyped parameter with calc()' Pass）。

**三失败臂**——typed 实参强制失败≡IACVT 实参：缺省可解则兑缺省（缺
省内 var() 引用再解析）再失败或无缺省→SCOPE_IACVT 遮蔽标记，不整调
用 invalid（「Arguments are defaulted on type mismatch」族）；typed
局部失败→IACVT 遮蔽走回退（「Invalid value for typed local becomes
IACVT」）；typed 返回失败→调用 invalid `Ok(None)`（非环语义不冒
Cycle）。**块产物门**——`product_has_top_level_block`（dp/db/dc 三深
度计数，顶层 `{}` 块）→`Ok(None)`（「Passing {} as argument」expected
""——实参块字面透传绕过消费声明侧同门的不对称即修复面；list 形
`--f({1px, 2px})` 保留）。

**指纹**：`css_wire.rs` `KC5R120_TYPED_EVAL_FACE`（ZW-KC5R120-TYPED-
EVAL-FACE，#[used] 防 DCE）＋CARRY-K119 线承。

## 钉测（11 净增＋1 改断言）

`block_arg_unwrap` 旧钉升级：`--f({{}})`/`--f({a}{b})` 断言改
`DashOutcome::Invalid`（WPT 判据 expected "" 站 Invalid 侧，与块产物
门一致）。11 新钉：typed result calc 化简（13px）＋typed result
mismatch→Invalid（12s 喂 `<length>`）＋typed 实参 calc 化简（101px）
＋typed 实参单位归一矩阵（101px/360deg/1s）＋typed 实参 var 先解再化
简＋typed mismatch 兑缺省（"1 2 3"）＋缺省含引用再解析（"5px 6px"）
＋typed mismatch IACVT 遮蔽双形＋typed result 拒 CSS 宽关键字（
revert-layer→Invalid）＋typed local invalid→IACVT（"3" 喂
`<color>`→PASS）＋type(*) 零漂移透传矩阵（100px/1/calc 原样）。

## 门禁（三代腿）

- 首轮 fmt=1（折行漂移纯格式）→cargo fmt 落盘复检 0。
- 门禁腿：css-parser lib **3415P/0F**＋style-system lib **2664P/0F**
  （2653 基＋k120 净增 11）＋engine lib **4306P/3F**＝既有红精确对号
  （s330/expando/p99s，名单与 k119 腿恒同——非本刀引入）＋clippy
  **0**＋fmt **0**。

## 探针定谳（一代）

gen13（md5 `b689e90feadaf91343237cf7071d5b2f`，指纹
ZW-KC5R120-TYPED-EVAL-FACE=1＋CARRY-K119=1）全语料单腿（43 页 521 键）
对 k119 b1 终态逐键差分：**F2P=13、P2F=0、GONE=NEW=0**＝
dashed-function-eval **F2P=12**（typed 返回 calc 化简/mismatch 拒收/
typed 实参族 7＋块产物门 2＋result CSS 宽关键字拒收 1）＋
function-parameter-types **F2P=1**（typed local IACVT）。registry 13
键程序化生成（f2p-registry-r120.py，防转写漂移）。探针窗 404→417。
random Pass 键零个在语料（typed 改动零 random 漂移）。

## 电池双腿

- b0＝k119 归档 bin `4812766de73f14b0337e6f4a8ec8d836`
  （ZW-KC5R119-FN-EVAL-SCOPE）；rc0=0。
- b1＝gen13；指纹=1＋CARRY=1；rc1=0。
- **P0=404 pristine 双证**：b0 重跑 JSON 与 k119 收账 b1 **逐键恒同**
  （程序化全量比对 True）；b1 JSON 与探针产出**逐键恒同**（同 gen13
  bin 两独立轮全等 True）；P1=417 净 +13。
- status census：b0 {Fail 116/Pass 404/Timeout 1} → b1 {Fail 103/Pass
  417/Timeout 1}；43 页 521 cases 双侧恒同；Timeout 键恒同。

## 判官（judge-r120.py 预注册硬码）

- G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0。
- G2 P2F total=0 out-of-exemption=0（豁免集空）；held-F 6/6 持
  （shadow DOM 3＋invalidation 3，两臂皆 Fail）；红线 4/4 持
  （apply-top-level/mixin-basic/shadow 隔离/mixin-cycle）。
- G3 F2P observed=13 knife=13/13 missing=0 out-of-registry=0（registry
  内插）。
- G4 non-exempt pages diff=0；NET +13==+13 恒等式。
- **VERDICT PASS**（一次性通过，预注册零修订）。

## verify（verify-r120.py 独立复核）

恒等式 P1−P0==F2P−P2F（+13==13−0）＋关系预测 (13,13,0) 逐位命中＋
presence 521==521＋pages 43==43＋status 值域 census 净＋Timeout 恒同＝
**VERIFY-PASS**。

## 新基与挂账

新基 css-mixins P=417。刀域残 held-F＝eval 6（result 描述符 CSS 宽
关键字六关键字 left unresolved——inherit/unset/initial/revert/
revert-layer/revert-rule 产物字面透传由元素级 cascade 处理，需级联
次优声明回溯架构）＝**k121 cascade 域候选**；parameter-types 残 5
（typed 参数局部注册语义 4＋if() 查询注册自定义属性 1）＝k121+。
挂账池更新：conditionals 17F＋attr 10F＋cycles 4（真循环检测域：
property cycle 观察形）本刀未触全 held（G4 SET-IDENTICAL 证明页级
零扰）＋mixin-invalidation 3（mutation 生效形 held）＋
mixin-shadow-dom 3（carry，shadow 级联面缺位）＋@apply 嵌套块
FIXME＋其余 carry：from-import-media 条件假形＋m2c/m8/m15＋gCS
IACVT ''/absent 统一刀＋animations 24＋narrowing＋DC-3 对账周账
12-01。

脚本/结果驻 `.acceptance/wptfyi-css/k120/`（gitignore 不入库；bins；
/tmp/zw-k120/ 同灶＋probe-b1-full/results-k120-b{0,1}＋battery-k120.sh/
probe-r120.sh/judge-r120.py/verify-r120.py/f2p-registry-r120.py/
build-r120.sh/gates-r120.sh）。
