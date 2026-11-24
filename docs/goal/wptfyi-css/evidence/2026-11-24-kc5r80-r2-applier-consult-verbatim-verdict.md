# kc5r80 刀 R2 全链验证定谳（consult→canonicalize 往返写面补缺＋race 语义钉定）

- 日期：2026-11-24
- 位面：实施位（ZeroWeb-2 clone，wptfyi-css r80 轮刀 R2 收口）
- 判读车代际：
  - 插桩诊断车 `bins/r80r2-probe-runner`：`5fb7fc93dd87`（无 RET 探针）→ `d80549d95f90`（R80B-RET 探针代，diag2/diag3 用）
  - 电池净车 `bins/r80b2-runner`：`6d49949c2f84`（R2 全刀＋探针全剥，strings PROBE-FREE-VERIFIED）
  - b0 基线：`kc5r79/bins/r79-runner` `f45097a17eb4`（r79 b1 内容复用，免构建）

## 1. 根因定谳：canonicalize 往返写面缺口

consult 链 `query_core` 正确产出 eased-interp raw 后，`js_anim_registry::canonicalize`
（js_anim_registry.rs:479）克隆样式 → `AnimationClock::apply_to_computed_style` →
重串化，以 `ser1 != ser0` 为采纳门——`apply_single_property`（animation.rs:4790）
缺 border-*-radius ×4／column-gap／transform-origin／text-decoration-color／
box-shadow 五族臂 → apply 空转 ser1≡ser0 → None → gCS 回声 cascade 终值 =
properties-value 页「must not be target value after start」假败。

探针实证（R80B-RET 钩在 `__zw_get_computed_style_by_handle` 回调出口）：
REG 成功（REGISTER from=1px to=10px）→ CONSULT-HIT eased-interp raw=1.00px →
**RET ret="10px"（回声终值）**——丢弃点锁定在 canonicalize 内，非登记/插值端。

修复（mask/ui/background-position 委托臂同病同治先例）：

- LP 直写臂 ×5（border-*-radius、column-gap）：`parse_len_pct_val` 受理 strict 臂
  产形（"N.NNpx"/"N.NN%"）写 `LengthValue` 槽。
- text-decoration-color 直写色槽（`parse_color`，同 color 臂形）。
- transform-origin／box-shadow 委托 style-system
  `apply_advanced_property_value(style, name, value, None)`（双 LP 槽＋
  `parse_box_shadow_list` 全槽映射同文法单点）。
- strict/lenient shadow 臂拆分：box-shadow 独立门换 `parse_box_shadow_list`
  （gCS 串化形 `rgb(r, g, b) Xpx Ypx Bpx Spx` spread 恒显，text-shadow 文法无
  spread 必拒——此前 REG-STRICT 假 honest 全族）；lenient 臂五组件
  （ox/oy/blur/spread/color）逐层插值，blur 钳非负、spread 带符号、inset 失配
  层离散翻转。
- `interpolate_lp_token_strict` 纯 % 对分支补齐（px 系数恒 0 误走 px 形，
  transform-origin `0% 50%`→`100% 50%` 端点往返断裂修复）。

## 2. diag3 验证（002＋003，TEXT 通道，bin d80549d95f90）

- 002：values **8P/1F**（8 margin/padding % 对 F2P；vertical-align honest 1F）。
- 003：values **37P/24F**（diag2 基线 26P/96F）。column-gap ×8＋
  text-decoration-color＋transform-origin 全翻——consult eased-interp 逐点吻合
  （raw=7.90px at t=1.534/2 ≈ ease(0.767)，1+9×0.767 数学核）。
- remaining 24＝honest 12（预注册 ceiling 逐案对上：background-image ×3／
  background-size ×1／display ×2／position ×3／object-view-box ×3）＋race 12。

## 3. race 语义钉定（Tier-1 依赖，r82 站位）

harness `generalParallelTest.js:240` values[1]＝第二个去重采样值；引擎
`document.getAnimations()` 缺 CSSTransition 对象（全局缺口）→ `a.currentTime`
seek 惰性 → Tier-2 consult 只走墙钟。gCS 读成本 ~0.7s/次 × 5 同步采样 →
2s 窗内需两次 distinct 读：column-gap 1.211+1.908 双读窗内 ✓（values[1]=9.58px
≠to）；radius 1.534+2.217 第二读落窗外回 to（values[1]="10px" ==to →F）。
font-size-adjust ×2 同形翻转（diag1 窗内过、diag3 窗外挂，CONSULT-MISS＋
「while transitioning」形）＝race-class 非结构 P2F。确定性修复＝getAnimations
返 CSSTransition＋currentTime 桥接（Tier-1 域）。

## 4. 电池（门修订 3，预注册 NOTES §6；b0r2/b1r2 代际隔离目录）

- provenance：b0 `f45097a17eb4`／b1 `6d49949c2f84`；face md5 五文件＋part04
  发射日志（/tmp/zw-r80-battery2.log 头部）。shim part03/part04 mtime
  07:14/07:30 早于发射 10:12，mid-run 零 JS 变更。
- css-values：b0r2≡b1r2＝**6679P/3201F/2T 全等零扰动**（LP 臂重写＋锚定保留
  读＋R2 登记扩名的回归面洁净）。
- css-multicol：b0r2 P=1251/F=362（r79 锚逐字）；b1r2 P=1254/F=359——+3/-3。
  跨流仲裁（对向 verify 位 10:0x master 记录，先于本电池发射 10:12）：+3 系
  树内 KC-5-B（cis 简写/长手合并）域真增益非回归，判读锚换全量新 bin，
  键级归因见 §5。
- css-transitions：见 §5 键级判读。
- 电池 bin 与落库树的已知偏形：bin 构建后落了 5 处机械 clippy 修复
  （explicit_auto_deref/neg_cmp_op_on_partial_ord/let_and_return ×2/
  doc_lazy_continuation），语义中立（neg_cmp 臂 NaN 边界差异不可达——
  duration 经解析默认 0.0 非 NaN）。

## 5. transitions 键级判读（电池二轮，门修订 3）

全域：b0 `1049P/441F/19T` → b1 `1141P/348F/19T`；F2P=93，**P2F=1（fail-stop 触发）**，
only-b0=1（b1 报告缺 1 案，完备性不对称记录）。values 全等零扰动
（6679/3201/2，257 页）；multicol +3/-3＝KC-4 gap 三键（gap-animation-001/002/003，
第三代复现，跨流仲裁归 KC-4 账）。

F2P=93 机制分类（键级清单＝judge-r80b2.py 输出，机制逐族）：

- **纯 % 分支结构修复 ×8**（002 margin/padding-bottom/left/right/top %）：LP 对
  px 系数恒 0 误走 px 形致 gCS % 形端点往返断裂（interpolate_lp_token_strict 补
  纯 % 域串化）；diag3 探针臂 002 8P 逐案对照。
- **applier 五族臂 ×11**（003：column-gap ×8＋text-decoration-color＋
  transform-origin＋box-shadow(shadow)）：canonicalize 往返写面补臂，consult
  eased-interp 逐点数学核（§2）。
- **R2 锚保留起动域 ×72**（003 column-width/height/rule-width 各 ×8＋column-count
  ＋column-rule-color＋font-stretch＋transform(rotate)；implicit-001 ×22；
  inherit-003 ×22）：b0 旧序锚消费在登记失败/strict 路径毒化 from → 过渡不起动
  got=终值；R2 保留读起动后既有 interp 通道产出中值。race 界未逐键钉定，标
  结构域候选。
- **race ×2**（003 font-size-adjust number decimal/integer）：diag1/diag3 双窗
  实证（diag1 窗内过、diag3 窗外挂，CONSULT-MISS「while transitioning」形）＝
  Tier-1 依赖（getAnimations 缺 CSSTransition→currentTime seek 惰性→values[1]
  墙钟竞速），非结构。确定性修复＝r82 站位。

**P2F=1 案定谳（结构回归，非 race）**：transition-behavior.html
`transition-behavior:allow-discrete should animate for values fallback to discrete
animation`——b0 P=3/F=3 → b1 P=2/F=4 确定性 solo 复现。机制：R2 初版按登记成败
判锚——normal 位 strict-skip（clip-path discrete 值拒登记）写读**保留**旧锚，
锚滞留 from=`circle(50px)`；allow-discrete 位 pair 端点错代（旧 from × 新 to），
consult 回旧值 got=`circle(50px)` ≠ expected=`circle(farthest-side)`。
css-transitions-1 §2 before-change style 语义要求锚随每次样式变化事件前移，与
登记成败解耦。修复＝收口追刀 `style_change_advances_anchor`（读面判据：
`transition_property` 全空＝机制未确立读保留锚[K3a 克隆后代防线]；其余读含
登记失败一律前移）——View/Gen 两条 call site 同批换装（computed_style_cache.rs:
~932 / callbacks.rs ~2683）。电池三轮（r80c1）按预注册重验此案应 P3/F3 复原。

## 6. 结论（电池三轮 r80c1 判读，六门全过 → R2＋追刀 收账）

电池三轮 `battery-r80c1.sh`（unit zw-r80c1-battery，BATTERY-RC=0 12:13:18，
六腿全 RC=0；b0=kc5r79 r79-runner `f45097a1`，b1=`bins/r80c1-runner`
`351ba81f658a`＝R2 全刀＋追刀 style_change_advances_anchor，与 kc5r81b
r81b-b1-runner md5 恒等＝bin 内容恒等交叉验证锚）。face census 14 项 md5
记 log 头（NOTES §8）。判读 `judge-r80c1.py` case-name 键级差分，verdict PASS
exit 0：

- **G1 P2F=0 fail-stop**：values/multicol/transitions 三 corpus 全零 pass 丢失。
- **G2 multicol 锚逐字**：b0=(1251,362)、b1=(1267,346) 双命中——b1 形与 bin
  恒等的 kc5r81b b1 实测逐字同＝跨 battery 交叉验证点成立。
- **G3 multicol 外来键签名**：F2P=16 恰预注册 16 键（columns-interpolation
  inherit 族 13＋gap-animation-001/002/003 各 1＝KC-4 账），无其他移动。
- **G4 values SET-IDENTICAL**：6679→6680，唯一 churn＝
  random-computed.tentative.html `random(--identifier, a, b)` fail→pass
  （预注册 random 域豁免键，kc5r81a run2 同键先例）。
- **G5 追刀修复判据**：transition-behavior.html
  `transition-behavior:allow-discrete should animate for values fallback to
  discrete animations.` 键 b1r3=**pass**（§5 P2F=1 案复原）；页级三态
  b0/b1 均 P3/F3＝预注册 b0 形复原。§5 追刀处方（锚随样式变化前移、与登记
  成败解耦，View/Gen 双 call site 换装）实证生效。
- **G6 对上代 b1 净差**：b1r3 vs b1r2（r80b2 b1=6d49949c）键级 diff 恰为
  G5 键 fail→pass（F2P=1，extra=0，P2F=0，F↔T jitter=0）；only-r3=3＝
  event-dispatch.tentative.html presence 噪声族（b0→b1 双向 report-only，
  pass 计数中性，r80b2 判读先例）。

transitions 全域：b0r3 `1049P/459F/17T`（P 锚逐字＝r79 账；F+T 比 r80b2 b0
多捕获 16 案＝墙钟截断 presence 噪声，P 集稳定）→ b1r3 `1142P/350F/19T`
（对上代净 +1P＝追刀键）。F2P=93 机制分类沿用 §5（纯 % ×8＋applier 五族
×11＋R2 锚保留起动域 ×72＋race ×2）。

**收账**：R2 全刀＋追刀 style_change_advances_anchor 六门实证零回归，落库
commit `3e8cdfc12`（face 14 文件 temp-index plumbing，parent=0adcfd9de——
上游两波推进 b0f06491e→0adcfd9de 只碰 docs 域零 face 交集，合并件免重做；
part03.js/bridge 合并件保上游 dom 刀②续内容）；race ×2 维持 r82 站位；
WAAPI 7 键 from=[inherit] 解析域移交下一刀（js_anim_registry from 关键字）。


