# kc5r122 · style() 冒号等值类型感知比较刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 421 → 426（+5）**＝judge VERDICT PASS＋verify
VERIFY-PASS（预注册硬码一次性通过）＝**P2F=0、GONE=NEW=0**。套件域账，
引擎总账不动（页外域零扩散）。

## 刀形

css-mixins-1 §2.1 typed 参数＝局部注册语义（function-parameter-types
五 held 键全清）＝if_function.rs 冒号等值臂类型感知单机制刀：

**判据解码**——五 held 键全用 `if(style(--c:X))` 冒号等值查询：现
text_eq（空白归一字节等值）无类型感知，`--f(#f00)` 查 `red` 文本不等
恒 FAIL。刀：**型别表**（@function 参数型别全局联合∪@property 注册
语法回落，params 优先）查得已知型别且双侧解析成功 → **比解析值**
（`<color>` "red"≡"#f00"≡Rgba；`<length>` px 数值（em 基 γ1）；`<angle>`
rad／`<time>` s／`<number>` f64）。

- 五键形态：param retains type×1（`--f(--c <color>)` 实参 #f00 查
  red）＋param type as local registration×1（局部 `--c:#00f` 查
  blue）＋parent stack frame×2（`--f()` 无参查询经调用方作用域穿线
  见 `--g(--c <color>)` 参数）＋registered query×1（@property --c
  syntax `<color>`＋元素 `--c: red` 查 #f00）
- type(*)/未知型/联合型（parse_type_spec 不识）→ 回退裸 text_eq 不变
  （「Universally typed」现 Pass 键零扰）
- 任一侧解析失败 → 回退裸 text_eq（观测恒同——解析确定性 ⇒ 同文本
  同解析值，唯一翻转 False→True，**构造性零 P2F**）
- 元素站（lib.rs 4.76）type_lookup=None 行为零变；FIXME：per-frame
  provenance（同名局部注册遮蔽次序）未建模——全局联合保守面
- 穿线：IfEvalCtx 加 type_lookup＋font_base_px；custom_function.rs
  substitute_dashed_functions/evaluate/if_session_eval 加 types 参数
  threading（args 递归/产物 splice/局部展开/result 站全覆盖）

**指纹**：`css_wire.rs` `KC5R122_TYPED_QUERY_FACE`（ZW-KC5R122-TYPED-
QUERY-FACE，#[used] 防 DCE）＋CARRY-K121 线承。

## 钉测（11 净增）

style-system tests/typed_style_query.rs 新模块：WPT 五案形态钉（retains
type/local registration/parent stack frame×2/registered query）＋type(*)
豁免（text_eq 不变）＋untyped 不变（#f00 vs red FAIL）＋untyped 等值文
本照旧 PASS＋右侧解析失败回退 FAIL＋左侧解析失败回退（@property 注册
型别＋元素基值不可解析 → text_eq True——若误硬判 False 此钉红）＋
`<length>` px canonical 翻转（1em≡16px 刀前 FAIL 刀后 PASS）。前提勘
误：初版「typed 实参 oops 查询回退」钉前提错——typed 实参绑定期即被
k120 强制门 IACVT 走不到比较臂，改双侧回退对钉。

## 门禁（三代腿）

- 首轮 style-system 1F＝typed_parse_failure 前提错（上勘误）＋fmt 折
  行两处→修钉＋cargo fmt 落盘复检 0。
- 门禁腿：css-parser lib **3415P/0F**＋style-system lib **2685P/0F**
  （2674 基＋k122 净增 11）＋engine lib **4308P/3F**＝既有红精确对号
  （s330/expando/p99s，名单与 k121 腿恒同——非本刀引入）＋clippy
  **0**＋fmt **0**。

## 探针定谳（一代即中）

gen15（md5 `326125ebb4e57754905ac8d0b93366fa`，指纹
ZW-KC5R122-TYPED-QUERY-FACE=1＋CARRY-K121=1）全语料单腿（43 页 521
键）：探针对 k121 b1 终态逐键差分 **F2P=5、P2F=0、GONE=NEW=0**＝
function-parameter-types.tentative.html **F2P=5**（五 held 全翻、该页
held-F 归零）。registry 5 键程序化生成（f2p-registry-r122.py，BASE
指 k121 b1——仿制 sed 次序坑两处（gates log 名 r119 残留/build+
probe runner 名 r122-elem-kw 被 k121→k122 先行吃掉模式）grep 核出修
正，防转写漂移机制双向起效）。探针窗 421→426。

## 电池双腿

- b0＝k121 归档 bin `eb146ee6c6f957d848735c08c825406f`
  （ZW-KC5R121-ELEM-KW-FACE）；rc0=0。
- b1＝gen15；指纹=1＋CARRY=1；rc1=0。
- **P0=421 pristine 双证**：b0 重跑 JSON 与 k121 收账 b1 **逐键恒同**
  （程序化全量比对 True，521 键含 message）；b1 JSON 与探针产出**逐键
  恒同**（同 gen15 bin 两独立轮全等 True）；P1=426 净 +5。
- status census：b0 {Fail 99/Pass 421/Timeout 1} → b1 {Fail 94/Pass
  426/Timeout 1}；43 页 521 cases 双侧恒同；Timeout 键恒同。

## 判官（judge-r122.py 预注册硬码）

- G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0。
- G2 P2F total=0 out-of-exemption=0（豁免集空）；held-F 6/6 持
  （shadow DOM 3＋invalidation 3，两臂皆 Fail）；红线 4/4 持
  （apply-top-level/mixin-basic/shadow 隔离/mixin-cycle）。
- G3 F2P observed=5 knife=5/5 missing=0 out-of-registry=0（registry
  内插）。
- G4 non-exempt pages diff=0；NET +5==+5 恒等式。
- **VERDICT PASS**（一次性通过，预注册零修订）。

## verify（verify-r122.py 独立复核）

恒等式 P1−P0==F2P−P2F（+5==5−0）＋关系预测 (5,5,0) 逐位命中＋
presence 521==521＋pages 43==43＋status 值域 census 净＋Timeout 恒同＝
**VERIFY-PASS**。

## 新基与挂账

新基 css-mixins P=426。刀域残 held-F＝parameter-types **0**（页清空）
＝eval 2（result 描述符 revert-layer/revert-rule left unresolved——需
级联层/规则次优声明回溯架构）＝**k123+ 回溯刀候选**。挂账池更新：
conditionals 17F＋attr 10F＋cycles 4（真循环检测域：property cycle 观
察形）＋local-inherit 7（inherit() 函数记法栈帧语义，非关键字域）＋
mixin-invalidation 3（mutation 生效形 held）＋mixin-shadow-dom 3
（carry，shadow 级联面缺位）＋@apply 嵌套块 FIXME＋其余 carry：
from-import-media＋m2c/m8/m15＋gCS IACVT ''/absent 统一刀＋animations
24＋narrowing＋DC-3 对账周账 12-01。

脚本/结果驻 `.acceptance/wptfyi-css/k122/`（gitignore 不入库；bins）＋
/tmp/zw-k122/ 同灶（probe-b1-full/results-k122-b{0,1}＋battery-k122.sh/
probe-r122.sh/judge-r122.py/verify-r122.py/f2p-registry-r122.py/
build-r122.sh/gates-r122.sh）。
