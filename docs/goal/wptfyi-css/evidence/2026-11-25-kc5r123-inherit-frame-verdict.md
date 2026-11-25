# kc5r123 · 会话内 inherit() 父栈帧替换刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 425（b0 实测）→ 431（+6）**＝judge VERDICT PASS＋
verify VERIFY-PASS（重注册版）＝**P2F=0、GONE=NEW=0**。净 +6＝
local-inherit-substitution **5 键**（本刀主域）＋function-parameter-types
**1 键**（k123a 帧表修正——k122 遗留非确定雷修复兑现）。套件域账。

## 刀形

css-values-5 §inherit-notation × css-mixins-1 栈帧语义（local-inherit
5 键全清）＝custom_function.rs 会话域 inherit() 记法单机制刀：

**判据解码**——`inherit(--name[, fallback])` 取**父栈帧**（调用方已解
析作用域）值：
- 元素父帧：顶层调用 incoming＝gather 表（`--f()` 内 inherit(--x) 读
  元素 --x——本帧局部 FAIL2 不遮蔽父帧读取）
- 调用方函数帧：嵌套调用 incoming＝调用方 scope_out（--g 内 inherit
  读 --f 帧局部 PASS——k119 穿线基建直接可用）
- guaranteed-invalid 父帧（var 缺席/var 自环 → SCOPE_IACVT 遮蔽标记/
  缺席）→ 有 fallback 取 fallback（原样拼接，下游 var 站续解）、无
  fallback → Err（result 站 → 调用 invalid；decl 臂 → 遮蔽 IACVT）
- typed 参数绑定经 k120 类型强制 canonical 化（42.0px→42px）经父帧读
  取透传
- 非 `--` 首参（标准属性查询）保持字面（FIXME 级联面）；拼接值不重扫
  （拼接内容形似体保字面——零重入面）；接线点＝decl 臂 stage1 后＋
  result 站 env 前
- 元素站（4.76）不触——元素域 inherit() 保持字面（现行为零变）

**指纹**：`css_wire.rs` `KC5R123_INHERIT_FRAME_FACE`（ZW-KC5R123-
INHERIT-FRAME-FACE，#[used] 防 DCE）＋CARRY-K122 线承。

## k123a 帧表修正（k122 遗留雷定谳——本轮最重要发现）

k122「params 全局联合」型别表对**同名跨函数参数**（`--g(--c <color>)`
vs `--f(--c type(*)`)）HashMap 迭代序不定＝非确定型别：「Universally
typed parameter can shadow other parameters」键 `lime≡#0f0`（同色
Rgba(0,255,0)）——序取 `<color>` 时解析比较误判相等 → 错取 FAIL3。
**k122 收账窗两轮（b1/probe）同序侥幸躲过＝名义账 426 是铜面样本**；
k123 电池 b0 臂（gen15 重跑）撞出 Fail（50% 铜面）→ judge 首轮 P2F=1
拦截。in-process census 复现：200 StyleSystem 实例 **{FAIL3: 109,
PASS: 91}**（同 bin 无刀两态铁证）。

**修正**：per-frame 型别表 `frame_types_of(caller, rule)`＝调用方帧型
别 ∪ 本函数参数型别（同名本函数胜，精确归属——值出自哪层帧型别随哪
层帧）＋帧链穿线（product splice 递归传被调方帧表；args 递归同帧）；
lib.rs 4.755 顶层帧表收窄为 registered 表（params 联合删除）；k122
FIXME（per-frame provenance）摘除。修正后 census **{PASS: 200}**
（200/200 确定性）。锚钉 universal_type_star_shadow_frame_deterministic
（20 实例循环断言）。

**k122 名义账修订**：k122 名义 426＝含该键序依赖 Pass 的铜面样本；确
定性账＝425（Fail）→ k123 face 431。下一轮电池 b0 锚＝gen16（431）
恢复洁净。

## 钉测（7 净增）

style-system tests/inherit_substitution.rs 新模块 6 钉：元素父帧（局
部不遮蔽）＋调用方函数帧＋guaranteed-invalid 父帧 fallback＋var 环父
帧 fallback＋typed 参数 canonical 透传（42.0px→42px）＋缺席无 fallback
调用 invalid（--tmp 表移除）＋typed_style_query.rs 追加 1 锚钉（帧归
属 20 实例循环）。前提勘误（首轮门禁）：E0631（`first()` 双引用
map(str::trim) 形）＋mod 字母序 fmt——落盘复检 0。

## 门禁（三代腿）

- 门禁腿：css-parser lib **3415P/0F**＋style-system lib **2692P/0F**
  （2685 基＋k123 净增 7）＋engine lib **4308P/3F**＝既有红精确对号
  （s330/expando/p99s，名单与 k122 腿恒同——非本刀引入）＋clippy
  **0**＋fmt **0**。

## 探针与电池（两代）

- **一代（雷暴露前 gen16 首建 bin）**：探针 F2P=5 P2F=0 registry 正
  确；电池 b1≡probe False（同 bin 两轮一 Fail 一 Pass）→ judge FAIL
  P2F=1（Universally typed）——**预注册门正确拦截非确定回归**；普查
  复现定位根因（上节）→ 帧表修正重建。
- **二代（修正后 gen16 重建 md5 `b1d40280cf829906bfef248eff560b4e`，
  指纹=1＋CARRY-K122=1）**：探针逐键差分（vs k122 b1 名义基）F2P=5
  P2F=0 GONE=NEW=0；registry 5 键（BASE sed 漏改坑再现——首跑误指
  k121 基出 10，grep 核出修正；后判定锚改 b0 实测，argv 顺序误传一次
  方向反（F2P=0/P2F=1）——核语义改正）。
- 电池双腿（锁键 zw-k123-battery.lock）：b0＝k122 归档 gen15
  `326125ebb4e57754905ac8d0b93366fa`（指纹=1）rc0=0——重跑撞出遗留雷
  铜面（唯一 DIFF＝Universally typed Fail；余 520 键与 k122-b1 逐键
  恒同）；b1＝gen16 rc1=0，**b1≡probe 逐键恒同 True**（gen16 确定性
  两独立轮全等）。
- **收账口径（重注册）**：P0＝b0 实测 425（gen15 铜面，注记）；registry
  判定锚＝b0 实测 6 键；status census b0 {F 95/P 425/T 1} → b1 {F
  89/P 431/T 1}；43 页 521 键恒同；Timeout 恒同。

## 判官（judge-r123.py 重注册硬码）

- G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0；G2 P2F=0（豁免集空）
  held-F 6/6 持＋红线 4/4 持；G3 F2P observed=6 knife=6/6 ±0
  （registry＝b0 实测锚 6 键）；G4 non-exempt diff=0；NET 425→431
  +6==6 恒等式。**VERDICT PASS**。

## verify（verify-r123.py 独立复核）

恒等式 P1−P0==F2P−P2F（+6==6−0）＋关系预测 (6,6,0) 逐位命中＋
presence 521==521＋pages 43==43＋status census 净＋Timeout 恒同＝
**VERIFY-PASS**（P0=425 铜面口径注记）。

## 新基与挂账

新基 css-mixins **P=431**（gen16 确定性锚；下一轮电池 b0＝gen16）。
挂账池更新：**eval 残 2（revert-layer/revert-rule——@layer 层序裁决＋
runner-up 声明回溯架构＝真级联回溯域，k121 时误判「继承臂同观」已勘
误——判据为 @layer one/two 形）**＋**local-inherit 残 2**（
dashed-function-standard-property「inherited longhand」＋
local-if-substitution「CSS-wide keywords locally (inherit)」——b0/b1
恒 F 取证非本刀域）＋conditionals 17F＋attr 10F＋cycles
4＋invalidation 3＋shadow-dom 3＋@apply 嵌套块 FIXME＋from-import-media
＋m2c/m8/m15＋gCS IACVT ''/absent 统一刀＋animations 24＋narrowing＋
DC-3 对账周账 12-01。

脚本/结果驻 `.acceptance/wptfyi-css/k123/`（bins）＋/tmp/zw-k123/ 同灶
（probe-b1-full/results-k123-b{0,1}＋battery-k123.sh/probe-r123.sh/
judge-r123.py/verify-r123.py/f2p-registry-r123.py/build-r123.sh/
gates-r123.sh/NOTES-k123.md）。
