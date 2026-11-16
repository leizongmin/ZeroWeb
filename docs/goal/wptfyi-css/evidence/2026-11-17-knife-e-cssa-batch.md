# 刀 E CSSA 声明式 gCS 咨询批（2026-11-17）：css-animations 同步读注册表 + inherit/junction/calc 三面修复

## 交付面（8 面）

1. **E1 junction 初值 0%**（`crates/style-system/src/property/default_impl.rs`）：
   `column/row-rule-inset-junction-start/end` 4 字段默认 `Px(0.0)` → `Percentage(0.0)`
   （WPT oracle：junction initial→-40% 期望 % 线性 lerp 自 0%；cap initial→10px 期望
   px 线性自 0px——cap 槽维持 0px）。
2. **E1b「rule-inset」简写复位臂 per-slot**（`inherit.rs` apply_initial_value）：
   原实现把 cap 默认复制进 junction 槽（E1 改默认后即毒化）——改 per-slot defaults。
3. **E2 rule-inset 8 longhand 显式 inherit 臂**（`inherit.rs` inherit_property）：
   css-cascade-4 §7.3（非继承属性显式 inherit = 复制父 computed，与 inherited-ness
   无关）——TRANS 通道 inherit→10px 块 from=20px 期望实证。
4. **E3 calc 混合插值**（`animation.rs`）：新 helper `blend_rule_inset_lp`
   （css-values-4 §8.5.6 interpolation-calc：px↔% 端点对逐单位系数线性，
   `calc(P% ± Xpx)`，px 系数零 → 纯 % 简化）接入宽松臂（tick 通道）+ 严格臂
   （transition 登记门放行跨单位对）。rule-inset 8 槽 168F/通道的根因修复。
5. **E4 shim `_zwUnsafe` 单位闸例外**（`part03.js` ~16500）：WAAPI 同属性跨帧
   单位 token 集不同 → 整动画不注册——px↔% 对全死根因。`_zwIsRuleInsetLonghand`
   判别（kebab/camel 双收 8 longhand）放行。shim provenance 指纹
   `zw-ruleinset-unitmix-2026-11-17` 注释（include_str 全量入二进制，strings 抽验）。
6. **E5 关键帧 keyword 解析**（`animation.rs` start_animation_with_underlying 开头、
   underlying 门之外——WAAPI 委托路径共享）：'initial'/'unset' →
   `ComputedStyle::default()` + `serialize_computed_property_with_sibling`（gCS
   串化同通道）。'inherit' 需父样式不解析（FIXME(kf-inherit)，残面 40F/页）。
7. **E6 cssa_registry**（新文件 `crates/engine/src/js_dom_bridge/cssa_registry.rs`）：
   CSS Animations 声明式 @keyframes 的 gCS 同步读咨询注册表（transition_registry
   十轮刀模式镜像）。thread_local `AnimationClock` + per-(World, identity) 条目；
   keyframes 经 `note_stylesheets` 喂入（**三处**：代际槽 tree_dirty 分支、全量
   兜底分支、**视图槽 compute_styles_for_doc 分支**——v2 补）；咨询四通道
   （callbacks.rs sel hit/miss 臂 + computed_style_cache.rs handle 链）；清理挂
   clear_generation_cache 同点；**喂入即清 ENTRIES**（同名换内容 harness 每 at
   重注入形态的失效防线）；per-identity 首咨询锚 start（config 从 ComputedStyle
   animation-* 字段读取，current_time=now → duration 100s/delay −50s 钉
   progress=0.5）；tick 产出过 `js_anim::canonicalize`（applier 未覆盖/恒等回落
   零回归方向）。渲染管线 AnimationClock/事件派发零触及。
8. **E7 钉测 6 个**：blend 代数（端点简化/负折叠/同单位 None）、宽松跨单位入口、
   junction/cap 初值 oracle、E5 initial/unset 解析（clock 端到端）、inherit 8 臂、
   简写复位 per-slot。engine lib 4,122/0F、style-system 2,550/0F。

## 两轮 A/B（B0=results-transall-knife-d-base，46684957 纯净二进制同源）

| 轮 | 二进制 | gaps | multicol | typedom | shapes | masking | 合计 |
|---|---|---|---|---|---|---|---|
| v1（代际槽喂入） | bin-cssa-e1 `5f978828` | +2037 | +24 | 0 | +18 | +36 | **+2115** |
| v2（+视图槽喂入） | bin-cssa-e2 `b69a15ce` | +2488 | +61 | 0 | +258 | +437 | **+3244** |

两页 interpolation 主池（rule-inset-interpolation ×2）：v1 P488/页 → v2 **P624/F48**
/页（残差：inherit 端点 40F + TRANS calc 端点形 8F）。

## PAGE-ONLY 零容忍对账（v2 vs base，subtest SET 全对比）

NEW-PASS **+3254** / GONE-PASS **10** / NEW-FAIL **0** / 五 corpus 页集合全等。
10 GONE 逐条归因 = **CSSA 真跑后揭穿的假绿**（A 侧 Pass = 动画零效果恰好断言
相等/断言未真跑 msg 空形；v1 账内同款已现形佐证），非语义回归，缺口入残面队列：

1. rule-color-interpolation-repeaters-001 ×1：`repeat(auto, …)` 双通道 vacuous
   （上游 typo，chrome 拒收非 parity 目标——刀 B3 外证 `652a04f66` 已闭环，持位）。
2. shape-margin neutral at(0) ×1：**neutral implicit boundary 缺口**——R5018
   resolve_implicit_boundaries 仅覆盖 individual transform 三属性，其余属性
   0%/100% 空帧合成缺 underlying 端点（FIXME 通用化刀面）。
3. shape-image-threshold inherit ×2：inherit 关键帧端点（需父样式通道，
   FIXME(kf-inherit) 池）。
4. clip-path-interpolation-002 ×4：clip-path 跨单位（px↔%）calc blend 缺口
   （clip_interp 无混合臂，同 E3 rule-inset 模型可复制）+ calc 端点形 quirk。
5. masking 其余 ×2：同 clip/neutral/inherit 族。

## 残差与刀后队列

- interpolation 两页 48F/页：inherit 40（CSSA+WAAPI inherit→10px 5 ats × 4 slots）
  + TRANS trans:all `-100%→1px` at(1) 8（calc(0% + Xpx) 端点形被折叠）。
- rule-inset 池外新亮面：shapes +258/masking +437/multicol +61 = cssa 咨询对
  全 corpus interpolation 页族的通吃增益（shape/mask/clip 属性同通道）。
- 刀后队列：R5018 neutral implicit 通用化、inherit 父样式线程、clip-path 跨单位
  blend、calc 端点形 quirk、cssa 声明式事件面。

## Provenance 与过程纪要

- A 基线 `bin-transall-knife-d-base` 46684957（刀 D 收口归档）；v1
  `bin-cssa-e1` 5f978828；v2 `bin-cssa-e2` b69a15ce（strings 指纹
  `zw-ruleinset-unitmix-2026-11-17` =1 双验）。es_module s419 peer WIP 在树窗口
  构建，二进制 provenance 以归档副本 md5 + 指纹为准。
- **视图槽漏喂教训**：interpolation corpus 元素 = createElement 动态形态 → gCS
  走 handle 通道 → 视图槽（HANDLE_VIEW_CACHE）独立重建（compute_styles_for_doc
  分支）——首刀只喂代际槽两处，CSSA 残余 156F/页全 got 底层值；补视图槽喂入
  后两页 P488→P624。**handle 通道 corpus 主形态的喂入面必须三处对齐**。
- systemd 发射两坑复账：`--scope` 模式随 client 进程组回收被杀（service 模式
  ——裸 systemd-run 默认 transient service——根修）；同 e1-full 脚本被并行流
  改造为 e2 五腿链双跑碰撞（bin/JSON_DIR 各归档副本隔离，采彼账停己腿零重跑）。
- 测试：engine lib 4,122/0F（含 v4 eased==1 例外钉）、style-system 2,550/0F、
  两 crate clippy 本刀文件过滤零告警；fmt 本刀八文件单文件核净（style-system
  registry.rs HEAD 域外既有漂移不卷入）。
- 腿链：`launch-cssa-e1-probe.sh`/`launch-cssa-e1-full.sh`（v1）+
  e2full 链（v2，并行流改造版）；JSON 在 results-cssa-e1-full、
  results-cssa-e2-{probe,full}；transcript 同前缀。残差分类与 PAGE-ONLY
  对账脚本输出在本文件同轮执行记录内。

## v2 五 corpus 链收尾账（shapes/masking 腿落盘，e2full 全链 GATE-DONE）

五 corpus v2 vs v1 增量：shapes **+240**（4848→5088，up 243/dn 3）、masking
**+401**（1961→2362，up 407/dn 6）、gaps +451、multicol +37、typedom 0——
与 v1 vs base 累加即控制面 v2 总账 +3244，两法交叉核账一致。

**v2→v1 翻转 18 dn 逐条归因（零 NEW/GONE 保持）**：
- 10 个 inherit 面（同面 inherit-up 24 对冲，净 +14）——FIXME(kf-inherit) 池。
- 8 个 neutral/implicit-keyframe 面（shape-margin/shape-image-threshold
  from-neutral at(0) ×2、clip-path from inset(20px) to inset(40%) ×5、
  rule-color repeaters ×1）——v1 里这些点靠 View 世界无咨询 echo 掩护通过
  （cascade 恰等期望），v2 视图槽喂入后咨询真跑、implicit 边界帧合成缺口
  （R5018 仅覆盖 transform 三属性）暴露为真失败。**非 v2 语义回归，是喂入
  面扩大后假绿揭穿**，与 10 GONE 同性质，缺口入残面队列（R5018 通用化刀面）。

## v3：interpolate_keyframes 末段选择修复（at(1) 撞末帧时停 [last-1,last] 段）

progress 落在/越过末帧 offset 时旧选择器把 from_idx 推到末帧（i+1 越界），
产出空段/持位形；修为 `if kf.offset <= progress && i + 1 < keyframes.len()`
——at(1) 落末段 [last-1,last] blend(1.0)，span=0 退化出 to 帧声明原样经
canonicalize 对齐 gCS 串化形（css-animations-1 §keyframes 段语义）。
bin-cssa-e3 `01225d12` 探针：css-gaps 两页 vs v2 **NEW 0 GONE 0
SET-IDENTICAL**（P624/F48 持位）——css-gaps 的 CSSA/WAAPI at(1) 面不经
interpolate_keyframes 末段（TRANS 通道走 transition_registry），修复保留：
css-animations/web-animations corpus 的 at(1) 面可受益，且段选择语义修正
独立成立。

## v4：transition echo 门 rule-inset 八槽例外（TRANS at(1) 真根因）

TRANS at(1) got `1px` 定谳：`transition_registry::query_core` eased==1.0
echo 短路门例外清单只有 rule-width/color 四属性——rule-inset 八槽不在内，
t=1 直接 `return None` → 回落级联 to 形 `1px`，**从未抵达**
`interpolate_property_value` 的 blend_rule_inset_lp 跨单位臂（v3 改的是
interpolate_keyframes，两通道不同插值入口——零位移的机理闭环）。修：例外
清单补八槽（同单位/关键字端点 t=1 经插值臂与 echo 数值恒等、canonicalize
零截尾归一同形，无翻转面），钉测 `test_eased_one_rule_inset_calc_mix`
（steps(1,start) 强制 eased==1.0，`-100%`→`1px` at(1) =
`calc(0% + 1px)`）。bin-cssa-e4 `90762cf3`，engine lib **4122/0F**（+1 钉）。
探针账（css-gaps 全 corpus，v4 vs v2）：**+16 up / 0 dn / NEW 0 GONE 0**（3753
→3769）——两 inset 页 P624/F48 → **P632/F40**，TRANS at(1) 16F 全数翻 Pass，
同单位/关键字端点零串化翻转（echo 门例外扩大无回路面）。残差定形：**40F/页
= inherit-keyframe 单面**（FIXME(kf-inherit) 池），TRANS 面清零。

## Provenance 增补

- v3 `bin-cssa-e3` 01225d12；v4 `bin-cssa-e4` 90762cf3（shim 指纹沿用
  zw-ruleinset-unitmix-2026-11-17）。腿链脚本 `launch-cssa-e4-probe.sh`
  （单腿 B）；**cwd 毒化复发一例**：systemd `-p WorkingDirectory=${ACC}`
  发射 → wpt-data 相对路径全断 788 页全 `No such file or directory`
  （knife-D 同款形），毒 JSON rm 重发、WorkingDirectory=$ROOT 根修。

## 刀后队列补充：mask-border 家族支持门定谳（11-17 数据定向，刀 F 靶面）

五 corpus 残余 10,081F 排行定谳：css-masking mask-border-\* 四插值页
（width 558 + slice 434 + source 294 + outset 168）+ composition 页 112 =
**1,791F 同一前置门**——`interpolation-testcommon.js:396`
`assert_true(CSS.supports(property, from), "'from' value should be supported")`
整页翻转，**非插值面**。门链：shim `CSS.supports`（part06.js）→ 宿主
`__zw_css_supports` → `decl_supported`（js_dom_bridge.rs:6183）→ css-wide
关键字门走 `PropertyRegistry::known_properties()` + 属性文法走
`apply_property_value_with_quirks`——**mask-border-{width,slice,outset,source,
mode,repeat} 全链缺席**（style-system 无字段/无注册、css-parser 无文法、
shim 无名单，border-image 同构先例仅 part10 Typed OM 两处）。

刀 F 两面批次（刀 D 级跨站）：
1. **支持门面**：css-parser 文法（width `<length-percentage>{1,4}`、slice
   `[<number>|<percentage>]{1,4} && fill?`、outset `[<length>|<number>]{1,4}`、
   source `<image>`、mode/repeat 离散关键字）+ style-system 注册/apply +
   decl_supported 走通——先解锁四页断言体真跑。
2. **插值面**：interpolate_property_value 三臂（width LP 列表同单位 lerp/
   跨单位 calc-mix 复用 blend_rule_inset_lp；slice number/% 分量同型 lerp +
   fill 双侧保持否则离散；outset len/num 同型 lerp 混型离散；source/mode/
   repeat 离散翻转）+ transition 注册关键字端点解析（E5 通道 analog：
   from/to 'inherit'/'initial'/'unset' 对 prev/new style 串化）。
