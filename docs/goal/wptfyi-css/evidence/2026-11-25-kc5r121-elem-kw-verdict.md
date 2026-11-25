# kc5r121 · `<dashed-function>` 产物元素级 CSS 宽关键字落地刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 417 → 421（+4）**＝judge VERDICT PASS＋verify
VERIFY-PASS（预注册硬码一次性通过）＝**P2F=0、GONE=NEW=0**。套件域账，
引擎总账不动（页外域零扩散）。

## 刀形

css-mixins-1 §3.1＋css-variables-1 §2 产物关键字落地＝eval 4F 靶面清
4（cheap-4；held 2 回溯域）＝style-system lib.rs 4.755 站 Value 臂单
站刀（custom_function.rs/computed.rs 零改——k119 会话分类只覆盖
locals/缺省，本刀补元素级）：

**判据解码**——WPT「left unresolved on result descriptor」族：求值端
对 result 产物关键字**字面透传不解**（k120 held 6 键归簇），消费元素
按 custom property 语义落地。替换产物**整值**为关键字时（trim 小写
判别）：

- `initial`＝guaranteed-invalid → 表移除兑现 var 回退/gCS ''
- `inherit`/`unset`＝父计算值代入（custom property 继承传递；缺席父
  值→同 absent）
- `revert`＝回退 author origin 声明后按继承性质取父计算值（**探针
  一代定谳修正**：初版 absent 臂预测 F2P=4 实翻 3——revert 非
  guaranteed-invalid，custom property 无 author 声明时值＝inherited，
  缺席父值才 guaranteed-invalid；改继承臂后二代探针 4 键全翻）
- `revert-layer`/`revert-rule` 需级联层/规则次优声明回溯架构，保持
  字面透传（held＝k122+ 回溯刀候选）

部分值关键字不认（整值才是关键字）；落表保 trim 原文（判别用小写，
大小写敏感产物零漂移）。注册属性 unset/revert=initial 值缺口：站内
无注册表，FIXME 随注册表接线刀对齐。

**指纹**：`css_wire.rs` `KC5R121_ELEM_KW_FACE`（ZW-KC5R121-ELEM-KW-
FACE，#[used] 防 DCE）＋CARRY-K120 线承。

## 钉测（10 净增）

style-system tests/custom_keyword_landing.rs 新模块：initial absent＋
revert 父值代入＋revert 父缺席 absent＋inherit 父值＋inherit 父缺席
absent＋unset 父值＋大小写不敏感（INHERIT）＋revert-layer/revert-rule
字面持＋部分值字面（pre initial post）＋作者直书关键字不触（无替换
路径保字面——套件域窄刀证词）。

## 门禁（三代腿）

- 首轮 engine E0308＝指纹字节数误标 22→23（字面实长 23）＋fmt 折行
  漂移→落盘复检 0。
- 门禁腿：css-parser lib **3415P/0F**＋style-system lib **2674P/0F**
  （2664 基＋k121 净增 10）＋engine lib **4308P/3F**＝既有红精确对号
  （s330/expando/p99s，名单与 k120 腿恒同——非本刀引入）＋clippy
  **0**＋fmt **0**。

## 探针定谳（两代）

gen14（md5 `eb146ee6c6f957d848735c08c825406f`，指纹
ZW-KC5R121-ELEM-KW-FACE=1＋CARRY-K120=1）全语料单腿（43 页 521 键）：
一代探针 F2P=3（revert 未翻）→ 揭穿预注册预测偏差 → revert 臂
absent→继承修正重建（md5 前代 `600b8e83472d5b00851298dbf295a125`
作废）→ 二代探针对 k120 b1 终态逐键差分 **F2P=4、P2F=0、
GONE=NEW=0**＝dashed-function-eval **F2P=4**（initial/inherit/unset/
revert left unresolved 全翻）。registry 4 键程序化生成
（f2p-registry-r121.py；首跑误指 k119 基线 404 出 17——BASE sed 漏
改揪出修正，防转写漂移机制双向起效）。探针窗 417→421。

## 电池双腿

- b0＝k120 归档 bin `b689e90feadaf91343237cf7071d5b2f`
  （ZW-KC5R120-TYPED-EVAL-FACE）；rc0=0。
- b1＝gen14；指纹=1＋CARRY=1；rc1=0。
- **P0=417 pristine 双证**：b0 重跑 JSON 与 k120 收账 b1 **逐键恒同**
  （程序化全量比对 True）；b1 JSON 与探针产出**逐键恒同**（同 gen14
  bin 两独立轮全等 True）；P1=421 净 +4。
- status census：b0 {Fail 103/Pass 417/Timeout 1} → b1 {Fail 99/Pass
  421/Timeout 1}；43 页 521 cases 双侧恒同；Timeout 键恒同。

## 判官（judge-r121.py 预注册硬码）

- G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0。
- G2 P2F total=0 out-of-exemption=0（豁免集空）；held-F 6/6 持
  （shadow DOM 3＋invalidation 3，两臂皆 Fail）；红线 4/4 持
  （apply-top-level/mixin-basic/shadow 隔离/mixin-cycle）。
- G3 F2P observed=4 knife=4/4 missing=0 out-of-registry=0（registry
  内插）。
- G4 non-exempt pages diff=0；NET +4==+4 恒等式。
- **VERDICT PASS**（一次性通过，预注册零修订）。

## verify（verify-r121.py 独立复核）

恒等式 P1−P0==F2P−P2F（+4==4−0）＋关系预测 (4,4,0) 逐位命中＋
presence 521==521＋pages 43==43＋status 值域 census 净＋Timeout 恒同＝
**VERIFY-PASS**。

## 新基与挂账

新基 css-mixins P=421。刀域残 held-F＝eval 2（result 描述符
revert-layer/revert-rule left unresolved——需级联层/规则次优声明回溯
架构）＝**k122+ 回溯刀候选**。挂账池更新：parameter-types 残 5（typed
参数局部注册语义 4＋if() 查询注册自定义属性 1）＋conditionals 17F＋
attr 10F＋cycles 4（真循环检测域：property cycle 观察形）本刀未触全
held（G4 SET-IDENTICAL 证明页级零扰）＋local-inherit 7（inherit() 函
数记法栈帧语义，非关键字域）＋mixin-invalidation 3（mutation 生效形
held）＋mixin-shadow-dom 3（carry，shadow 级联面缺位）＋@apply 嵌套
块 FIXME＋其余 carry：from-import-media＋m2c/m8/m15＋gCS IACVT ''/
absent 统一刀＋animations 24＋narrowing＋DC-3 对账周账 12-01。

脚本/结果驻 `.acceptance/wptfyi-css/k121/`（gitignore 不入库；bins；
/tmp/zw-k121/ 同灶＋probe-b1-full/results-k121-b{0,1}＋battery-k121.sh/
probe-r121.sh/judge-r121.py/verify-r121.py/f2p-registry-r121.py/
build-r121.sh/gates-r121.sh）。
