# kc5r92 verdict——math-depth 属性刀（registry＋apply＋computed＋inherit＋gCS 五触点＋integer 取整小臂）（2026-11-25）

## 结论

- **刀面**（指纹 `ZW-KC5R92-MATHDEPTH-FACE`，b1 bin md5 f8b7b3d0）：
  1. registry.rs 三表：`known_properties()`（CSS.supports 门）＋
     `initial_value()`（`"math-depth" => Some(Integer(0))`）＋
     `is_inherited()`（css-math-3 §math-depth Inherited: yes）。
  2. apply.rs 求值臂（order 臂旁）：`auto-add | add( <integer> ) | <integer>`
     文法解析（add() 剥壳 trim、i64 parse→i32 clamp）＋顶层数学函数
     fallback（order 臂同口径 eval_int_math_value）。
  3. computed_style.rs 字段 `pub math_depth: i32`＋default_impl.rs 初值 0
    （auto-add 父链基 0 求值等价）。
  4. inherit.rs 两臂：`inherit_property` 父→子拷贝＋`apply_initial_value`
     复位。
  5. matcher/mod.rs `is_property_supported` 三形门（auto-add/add(int)/int）。
  6. engine js_dom_bridge/computed_style.rs gCS 串化臂
    `"math-depth" => style.math_depth.to_string()`（裸整数串）。
  7. lib.rs `#[used]` 指纹 static（r91 指纹并存）。
  8. **css-parser integer 取整小臂**（布防「随机集成零新码」预测落空）：
     `random_integer_typed("math-depth")`＋eval_random_call serialize 前
     `v.floor()`（css-values-5 §9.1.2 integer 位取整）。加法式清单判定，
     其余属性零触。
- **纯加法刀**：8 文件 +106 行（face 85＋取整臂 21）；hunk 区与 fmt 债区
  互斥（random.rs hunk :748/:956 < 债区 :1375+；lib.rs 唯一 hunk :4969 <
  债区 :3375 语义区）。

## 布防预测账 as-run（两处勘误如实记）

| 门 | 布防预测 | as-run | 判 |
|---|---|---|---|
| G-F2P | =1 精确（Shared by property math-depth） | **=2**（另翻 `Property math-depth value 'add(random(30, 10))'`） | 修订入账（见下） |
| held-F | swap 键持 F | **翻 P** | 翻红即查结案（见下） |
| 零新码 | 随机集成零新码 | **integer 取整小臂必需** | 布防机制勘漏 |
| G-P2F | =0 fail-stop | =0 | PASS |
| G4 | SET-IDENTICAL | 非 random 页 diff=0 | PASS |
| G5 | GONE/NEW=0 | 9882/9882 =0 | PASS |
| NET | +1 | **+2**（6718→6720） | 修订 |

### held-F 翻红即查结案

`add(random(30, 10))` min>max：布防假设 spec §9.3 swap（区间 [10,30] 随机
非定值 ≠ '30' 恒红）；**引擎既定语义＝钳制恒返 min**（`eval_random_formula`
`max := if max < min { min }` → span 0 → draw 恒 30），r88 语料钉面在案
（`random(30%, 10%)` 期望 `0.3`＝min 恒返，Pass）。页期望 '30' 恰为 min 值
→ 恒等 → F→P。归因闭合：b0 同引擎同页持 F（无属性 gCS ''）；同形兄弟
corner-shape `superellipse(random(3, 1))`／offset-path `ray(random(30deg,
10deg))` 双臂持 F（属性未实现对照）——math-depth 属性实现为唯一变量。
sentinel 4 键（color/vtn/font-palette calc/animation-timeline）全持 F ✓。

### 「零新码」落空机制勘漏

布防三路分解把路 B 实证绑在 font-palette calc 形键 F 的「全替换落级联」
归因上——该观测（gCS ''）在「替换成功落级联」与「替换产物浮点→apply 拒
→IACVT」两假说下不可区分。r92 探针 DUMP（值回显进 fail message）定谳：
`add(random(1, 30000))` 浮点 draw（如 `15478.28…`）越出 `add(<integer>)`
文法 → cascade dummy oracle（cascade.rs `is_cascade_value_valid` SENTINEL
替换后 apply 裁决）拒 → 声明丢 → gCS '0'。integer 位取整后全链通。

## 探针门（probe-impl 双臂，b0 345b13cd 指纹=0 / b1 f8b7b3d0 指纹=1）

| 门 | b0 | b1 | 判 |
|---|---|---|---|
| E4 定值形 '3'/'7'/'-5'/'9'/auto-add '0' | Fail | Pass | 双翻 |
| E4b camel/kebab 通道同值 | Pass | Pass | 恒形 |
| E2 auto 异元素异 draw＋[1,30000] 界内 | Fail | Pass | 双翻 |
| E3 property-index-scoped 同值 | Fail | Pass | 双翻 |
| E6 supports 三真两假 | Fail | Pass | 双翻 |
| E5 inline raw echo＋gCS '11' | Fail | Pass | 双翻 |

## judge/verify 双算

- judge-r92.py **VERDICT PASS**：G1 双腿 RC=0/0；G5 9882/9882 GONE/NEW=0；
  G2 P2F=0＋sentinels=4 持 F＋held-F 显式集 0（修订后）；G3 F2P=2 刀内
  精确 out-of-registry=0；G4 非 random 页 diff=0；NET 6718→6720 = +2。
- verify-r92.py **VERIFY-PASS**：presence 恒等＋恒等式 P1−P0=2 ==
  F2P−P2F=2−0。
- 独立恒等式：P 6718→6720（累计基线 P 计 6720，r91 收 6718）。

## css-cascade 预检（all-prop-revert-layer watch 项——flake 定谳）

math-depth 进 `known_properties()` → `all` 简写展开清单 +1（cascade.rs:470）
→ noop 页声明集变化。首跑配对 b0=98P/b1=96P（−2）触发预检；**6 run 复跑**
（b0×3＋b1×3）：b0 99/96/97、b1 98/100/97——两臂分布互叠＝**HashMap 迭代
序 per-process 随机采样的顺序敏感 flake**（同 order 声明稳定排序输入序随
机），非 math-depth 回归。该页属 css-cascade 语料（本 goal 电池域外），
布防 §三 watch 项维持 DC-3 周账归因；引擎级确定性缺陷（同输入异输出）
归 rendering-compat 域记录，本刀不修。

## 门禁

- clippy `-p zero-css-parser -p zero-style-system -p zero-engine
  --all-targets -- -D warnings`（systemd scope 24G 包裹）**RC=0**。
  清偿两处 engine 既有 clippy 债（kc5r86 刀 js_anim_registry.rs:165
  redundant closure＋刀 R2 cssa_registry.rs:262 if_same_then_else——1.99
  新 lint 报出；机械合并/引用替换，语义恒等）。
- fmt 10 处 diff 全既有债（css-parser/random.rs 8 处＝对端刀 10417f5e0
  域；engine computed_style.rs:32 use 折行；lib.rs:3375 r91 挂账同处），
  本刀 hunk 区零相交硬证在案。**fmt 清偿笔挂账**（10 处独立提交）。
- 发射事故如实记：探针页 DUMP 注入用 `str.replace` 全量命中 3 处 script
  闭合标签毁页 1 次（重写干净页恢复）；probe 首跑 cases=0（页缺
  testharness 字面引用被包含门跳过，补 link/script 头后过）。

## 残面移交

1. wrapper 族余 4 键刀序维持（vtn ident() ＞ font-palette ＞
   animation-timeline ＞ color 相对色）。
2. css-cascade all-prop-revert-layer-noop 顺序敏感 flake（域外记录——
   rendering-compat 归因窗）。
3. fmt 清偿笔（10 处既有债独立提交）。
4. DC-3 对账周账（12-01 周窗口）。

## 证据链（.acceptance/wptfyi-css/kc5r92/ 本地）

NOTES-kc5r92 §零-§四（布防）／probe-impl-r92.sh＋log（双臂 RC=0/0）＋
probe-impl-{b0,b1}-css-values.json（六门 5F/6P）／battery-r92.sh＋log
（指纹 B1=1/B0=0、双腿 RC=0/0、DONE 01:46:21、PROBE-PAGE-CLEAN 0）／
results-k92-{b0,b1}/css-values.json 满量 257 页/9882 case（b0 Pass=6718
与 r91 收账逐位一致）／judge-r92.py VERDICT PASS／verify-r92.py
VERIFY-PASS／cascade-precheck/（b0/b1 JSON＋RRUN-VERDICT 6 run flake
定谳）／bins/r92-mathdepth-runner（f8b7b3d0，MATHDEPTH 指纹=1＋VARIND
指纹=1 双代并存）；代账 kc5r89/bins/GENERATIONS.txt r92-mathdepth 行。
