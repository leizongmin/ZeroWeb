# CSS 子测试控制面

**2026-11-18 · Active**。目标净增 153,000 项（绝对线 182,392）。weekly 对账账面 91,520/198,899（46.01%，baseline 29,392 → 净 +62,128）；本轮 A/B 局部账累计净 +6220（刀 RP1 registerProperty 语法严格门 +89 + 刀 fv1 @font-feature-values 全链 +313 + 收敛轮 masking +62 + 刀 N 二期 fonts +624 + 刀 N 三期/刀 O fonts +844 + 刀 A1 masking +273 + 刀 A1b 动画残面 +156 + 刀 P typed-om +1125 + 刀 A1c +455 + 刀 B1 border-image 动画族 +939 + 刀 A1d bg-position 单轴列表 +499 + 刀 Z z-index 整数插值 +158 + 刀 A1e bg-position 残面 WAAPI shim/echo/轴交换 +46 + 刀 A2 vertical-align 插值 +143＝主靶 +127＋搭载 +16；全语料位移 +144 含等价页 +1 归窗噪，已落库 `79b70dce2`、重绑探针 Δ=0 闭合 + 刀 A3 text-shadow 阴影列表插值 +127；全语料位移 +128 含窗噪 +1 归 fv1 serialize 域漂移 + 刀 Q2 shape-outside computed 三修 +238；全语料位移 +241 含 typed-om deterministic favourable +2 按 CLAIM 判据不扩认领 + backgrounds 窗噪 +1 第三次归 fv1 serialize 域漂移 + 刀 A3b transitions 首帧量子剃刀 +12（r2 守卫全域形 P→F 两面 5 行定谳→R2 收窄 bezier-only）；全语料位移 +15 含 backgrounds 白拿 +3＝white-orange ε-face 双形 ×2 剃刀家族顺向 + bg-position-y 漂移窗噪 +1 第四次归 fv1 serialize 域漂移 + 刀 A3r text-shadow 残面 unset 级联向 + currentcolor 预解析 +9＝主靶 CSSA 族 9 行（transitions ΔP=+9 零 FOREIGN；animations/masking SET-IDENTICAL；全语料位移 +8 含 backgrounds bg-position-y calc 窗噪 −1 第五次归 fv1 serialize 域漂移——b1 bin solo 复测 Fail 同签名，r2 属性门缺席版 110 P→F 回归已修不入账，见刀 A3r 段） + 刀 fv1 @font-feature-values 全链接入 +313＝fonts +303（主靶 288 + alternates-invalid 15 + map 族 3）+ cssom +6，P2F=0 全域，异常三案法证全销（LOST=10 b0 幻影/inherit-001 预存挂起双 solo 复现/animations flake ΔP=0，见刀 fv1 段） + 刀 R clip-path 写面白名单门 core face +120＝124 F2P（clip-path-invalid 48 + shape-parsing 20 + shapes path/rect/shape-function 56）− 位语 calc 缺口 P2F 4（f55b09a3e 在库即存在缺口 union r1 首次暴露，挂账 css-parser 位语热修流） + 刀 R2 四属性扩面写面门 +36（clip 4 + clip-rule 2 + ttf-invalid 25 + bg-clip-invalid 5，R2 域零 P2F）＝union r1 五语料批净 +155（含 bg-position-y calc 窗噪 −1 第六例不入刀账），weekly 快照兑现待下轮 cron。**卡点**：fv1 已落库推送（`7effb5d25`+`d8dbe4217`+`daddfdfed`）；RP1 已落库 `4c5d59491`（rebase 前号 `1ed980e63`；3 face 文件 1359+/56−，gen3 电池账 +89 零 P2F 已入 evidence，docs 收口笔 `023b8ed5d`，见刀 RP1 段）；engine clippy `too_many_arguments` 归 s419 对端在飞 WIP——engine lib **test** 断编译已解（part116 工作树删除系被上游超越残留，恢复后 HEAD 双含 part116+part117 编译通，11-18 09:2x 树清窗兑现）——A2/A1/B1/Z 引擎钉测补跑全销账（A1/B1/Z 31P/0F＝`edb22edea`） + engine clippy `too_many_arguments` 8/7 挂账对端清偿（clippy 复核随 fv1 落账后同窗跑）；刀 B1 机制码已落库 `b410cbcb3`（11-18 hunk 级 index 精确提交：animation.rs 7 hunks 295+/4− + transition_registry 清单 74→77，零 A1d hunks 卷入）；刀 A1d 已收口（REG 24 三层根因修复净 +499，见刀 A1d 段）；刀 Z 机制码已落库 `b270f1bca`（见刀 Z 段）。**下一步：刀 R/R2 已收账（union r1 五语料 A/B 净 +155，见刀 R/R2 收口段）——css-parser 位语 calc 热修收窄（v2 旁证：serialization 4 形复位但 interpolation 12 新 P2F 净 −2，移交笔在 NOTES 22:1x 段）+ engine clippy 清偿；刀 Q/S 正式六腿 A/B 同窗补账（刀 Q p2 engine 面已收口见刀 Q2 段，其十四腿 A/B 已入账）；typed-om animation/transition-delay 秒值串化 +2 deterministic favourable（Fix C collateral）后续刀可显式认领；刀 A3b 残面池（CSSA unset/currentcolor 9F 已刀 A3r 收清见其段，残 WAAPI 通道 ~10F 归 FIXME(kf-inherit-waapi)、steps t=0 精确臂挂观测待 seek 精确化驱动）；三语料残面 ≥20F 页靶图已侦查落档（transitions 事件系统鲸面 ~857F=在飞件占用、composite 池 79F、animations animation-range 族 170F、masking clip-path 插值 ~778F=css-parser 占用，[evidence residue-survey](evidence/2026-11-18-residue-survey-transitions-animations-masking.md)；刀 R 同族白名单门扩面候选 +59F=timing-function-invalid 25/behavior 23/clip 4/clip-rule 2/background-clip 5）；clip-path-shape-parsing 22F（shape() 命令 canonical）css-parser 后续刀；刀 A1c 残面池（mask-image 静态 cascade 'inherit' 臂——A1b 核注根子、apply.rs transition-behavior known 白名单门、css-transitions parsing 页）；刀 A1d 残面池（transition 简写 behavior 槽 ~5-8F、TransitionEnd/CSSTransition 事件系统大刀 ~1000F+30F、vertical-align 40F（CSS2/linebox+flexbox 静态布局面；插值面 136F 已刀 A2 收清见其段，残 calc 混合 9F 挂 FIXME(va-calc-slot) 池）——bg-position WAAPI 列表 14F + transitions echo 4F 两面已刀 A1e 收清见其段）；刀 Z 残面池（WAAPI z-index 通道 8F：from[auto] 离散翻转 7F + neutral 面 1F；z-index.html typed-om set 门 4F 归刀 P P2 池）；刀 P 残面 P2 池候选（axes 简写 computed em/cm/calc、calc-% 引擎折算、block-size 引擎 gCS 臂、scroll/inset % 引擎形、typed-om set 门 z-index `<integer>` 缺表 4F——残 146F+4F，见刀 P 段）；候选 @font-feature-values 294F（fv1 对端在飞，避让）、system-fonts 插值机制（supports 门已开，需 font 长手 lerp）、composite:add accumulation（@keyframes `animation-composition` 通道 engine 全缺，underlying ⊕ delta 端点预累加——A1d own-b1 窗口 REG 2 源，composition 页 24 形）。
**刀 TD1 收口：StylePropertyMap 语义补全（registered properties × Typed OM）（11-18 22:1x 收账，认领 +166 零 P2F）**：池＝css-properties-values-api typedom.html 166F（RP1 残面主靶）。码面 3 文件：registered_properties.rs（`TypeKind::name` 逆映射 + `syntax_of` 注册 syntax 描述符串化（Combined 分量 `|` 并置乘子直挂、ident 保大小写）+ `names_json` 清单 + 钉测 test_syntax_of_roundtrip）+ callbacks.rs（`__zw_registered_property_syntax`/`__zw_registered_property_names` 两查询回调，thread_local 直读无 doc 借用）+ part10-typed-om.js 五族修复（①set：自定义属性分型值 TypeError（css-typed-om-1 §set 值解析步仅受理 string/CSSUnparsedValue）+ 非列表逗号门对 `--` 豁免；②append：registered custom property 恒 TypeError（css-properties-values-api-1）；③迭代：values/entries 产列表形、entries 对 `[name, list]`、Symbol.iterator 挂 entries（css-typed-om-1 §5.3 iterable<DOMString, FrozenArray>）、forEach 列表形；④computed registered 按 syntax 分型 reify：乘子分段（`#` 顶层逗号/`+` 空白）+ unit 族 CSSUnitValue（famWant 表 + unitless zero 宽限）+ `<custom-ident>` 与 ident 字面量 CSSKeywordValue ci 敏感 + `<color>/<url>/<string>/<transform-*>` opaque 基类 + `<image>` CSSImageValue 相对 url baseURI 绝对化 + calc 族树解析 `#` 混合形 CSSMathSum；`'*'` 与解析失败回落 unparsed、specified 面恒 unparsed；⑤computedStyleMap 枚举 inline declared + 注册表名单（缓存键 el/inline 长度/注册表 JSON）。探针定谳推翻「宿主枚举缺」初判（宿主 length/item 原生含 `--`，真因 shim 五机制）；首代 face 探针 321P/3F+1 hang 定谳两缺陷（`<custom-ident>` reify 臂缺席 + 枚举逐名 gCS×item 每下标重算二次方页超时）二代修清 **325P/0F**。**四语料 A/B**（knife-td1-battery-2156 systemd oneshot，21:56→22:07:35 GATE-DONE rc=0 零 LEG-FAIL，双建 55s/42s 增量，b0 `7daa3f68`/b1 `b3083474`，fp-count b0=0/b1=1，S2 ride-along 窗内零扰动）：pva 653P/718F→819P/552F **F2P=166 P2F=0 NEW=0 LOST=0**＝恰 typedom 池全清；typed-om/variables/cssom 三哨兵 SET-IDENTICAL；.err 全零、timeout b0/b1 对称（variables 11/11、cssom 1/1）零新增；本轮累计净 **+6231**（前账 +6065 + 本刀 +166）。门禁：fmt 清、clippy 零新增（在案域外 3 lint：animation.rs ×2 + part115 ride-along）、engine lib **4191 passed/0 failed** + 钉测单独复核 1 passed/4190 filtered。残面池：pva 552F（at-property-typedom 0P/2F＝@property 规则→运行时注册表接线缺口挂账、font-property-unit-cycles.tentative 79、unit-cycle 族 102、registered-property-computation 33…）、typed-om 1286F 头页 logical/grid-template/scroll-padding 归 layout 域池不变。详账见 [evidence knife-td1](evidence/2026-11-18-knife-td1-typedom-map-semantics.md)。swarm：q2 lineage fork 21:2x 正式让位（CLAIM 尾段在案）；电池曾排队等 knife-r-union-battery（21:27–21:55 在飞）——共享树 A/B face-flip 互毒禁并发，TD1 face 全程 applied 其两侧同享＝常量相消其账不受影响。
**刀 R/R2 收口：invalid 写面白名单门 core face + 四属性扩面（11-18 22:1x 收账，resume-swarm 多 fork 合流：core fork clip-path 面 `f55b09a3e`、CLAIM 流 fork R2 扩面工作树 face + union 电池发射收账）**：池＝刀 R 同族白名单门扩面候选（timing-function-invalid 25/clip 4/clip-rule 2/background-clip 5 + core face clip-path-invalid 48 + shape-function 族 56）。R2 码面 apply.rs 单文件：白名单 60→64（clip/clip-rule/background-clip/transition-timing-function 四属性 + 数组内注释）+ 四分派臂（clip＝rect() 逗号 4 项 `<length>|auto` 百分比显式拒收、clip-rule＝nonzero|evenodd、background-clip＝`<bg-clip>#` 三 box + border-area/text 两序组合、ttf＝`<easing-function>#` 复用 parse_timing_function 严门 + linear_stops_valid 停点文法 + calc/sibling-index 嵌套放行）+ 钉测 knife_r2_expansion_gate_verdicts（36 断言语料锚定）。**union r1 五语料 A/B**（21:27→21:55 GATE-DONE 零 LEG-FAIL，双建 42s/40s M2/S2 法证绿，b0 `8b496720`=cc8b6708e 面 / b1 `38d4b216`=HEAD+R2 本提交精确面）：F2P=160 P2F=5 **批净 +155**＝core face 域 124（clip-path-invalid 48 + shape-parsing 20 + shapes 56）+ R2 域 36 − core face 位语 calc 缺口 P2F 4（circle/ellipse at calc() 位语形，serialization 页——**f55b09a3e 在库即存在，union r1 首次暴露**，挂账 css-parser 位语热修流）− bg-position-y calc 窗噪 1（第六例）。R2 域零 P2F；valid/computed 页零位移实证残面在序列化/computed 层非写面门（ttf-valid 22P/0F 本就绿——早版「valid 页 +31F 潜力」系 status 大小写误报已勘误）。门禁：lib 2568/0 + fmt 清 + clippy -D warnings 零告警（组合树复跑定谳）；R2 提交面=apply.rs 单文件（css-parser 热修 WIP 两文件归其流提交）。残面池：transition-behavior 23F（shorthand 池刀）、mask 简写 36F、linear() `&&` 两序 FIXME、ttf/bg-clip computed 9F + bg-clip-valid 镜像序 1F、clip/clip-rule computed 6F。热修 A/B v2 旁证（parse_position_pair 括号感知）：serialization 4 形复位 + shape-outside 5 新 F2P，但 interpolation 12 新 P2F（推进冻结 from-value）净 −2 不可入库，移交收窄。详账见 [evidence knife-r2](evidence/2026-11-18-knife-r2-expansion-write-gates.md)；swarm 仲裁全谱系 NOTES-knife-r-swarm-arbitration.txt 在档。
**刀 RP1 收口：registerProperty 语法严格门（11-18 20:3x 收账，认领 +89 全语料位移 +91 零 P2F，resume-swarm 双 fork 合流收口）**：池＝css-properties-values-api（register-property-syntax-parsing 主靶 292 行语料全矩阵 + 五语料 A/B 注册门全域回归面）。码面 3 文件：registered_properties.rs（typed/universal 严格注册门 validate_initial_strict 族：calc 严格分型/font-relative 拒/custom-ident ci 禁宽关键字/乘子邻接/混合 run 同族门 + emoji strip 越界修 + 禁名 ci + number ±号 + **gen3 引号 url 续扫修**）+ callbacks.rs（syntax wire 保真，空串不再映射 None）+ part06.js（shim WebIDL 字典转换：syntax undefined→'*'/null→"null"/inherits 门按兑换后 syntax 判）。探针三代：首代 emoji panic→二代 241P/5F→三代 246P/0F，gen3 runner（md5 `619b7962`）复验零回归。**gen3 五语料 A/B**（20:23→20:36 GATE-DONE 零 LEG-FAIL，双建 31s/30s M2/S2 法证绿，b0 `d12d87bd`）：pva 主靶 562P/799F→653P/718F **ΔP=+91**＝F2P 89（syntax-parsing 87 + registered-property-initial 2）+ NEW pass 2（NEW=10 乘子展开 b1 独有形按 CLAIM 不扩认领）−P2F 0；typed-om/variables/cssom SET-IDENTICAL、animations NEW=1/LOST=1 墙边陲 flake ΔP=0（fv1 同页同款）。**gen2 曾 P2F=6 已修**：custom-property-animation-image/url 引号 `url("https://…")` 形拒收——valid_url_value 对 scan_url_after_open 引号 handoff 单发 `i==b.len()` 判定漏扫余段（语料只锚未引号 `url(a)` 故探针幸绿、universal 面因循环续扫幸绿），`valid_declaration_value` 循环体析出 `scan_declaration_tail` 共享面引号 handoff 续扫收尾 + 钉 3 枚引号形 pin，gen2 JSON 存档 gen2-p2f/。门禁：engine lib 4 钉测全过（含引号形）、fmt 清、定向 clippy 零新增（在案 lint 均域外：animation.rs unnecessary_cast ×2 既有提交态、part115 s424d doc_lazy_continuation、es_module s419 too_many_arguments 对端 WIP）。残面池：pva b1 718F/97 页（typedom.html 166、font-property-unit-cycles.tentative 79、at-property/CSSOM 族 95、unit-cycle 检测族 102、registered-property-computation 33…尾 85 页 246F 长尾），typed-om 语料 1285F 头页 logical/grid-template/scroll-padding 归 layout 域池。详账见 [evidence knife-rp1](evidence/2026-11-18-knife-rp1-register-property-syntax.md)。swarm 仲裁：NOTES-knife-rp1-swarm-convergence.txt 在档（429 限流后双 fork 同源 resume：回填 fork 完成 evidence A/B/残面/gen2 法证四段合流回填，收口 fork 发射 gen3 电池 + 收账 + 提交推送）。
**刀 fv1 收口：@font-feature-values 全链接入（11-18 16:2x 收账落库 `7effb5d25`——rebase 前号 `3a0a573d8`，认领 +313 零 P2F，在飞孤儿码树内接手收口）**：池＝候选 @font-feature-values 294F（master 前账），靶面 css-fonts 四页（test_font_feature_values_parsing 主靶 + map_iteration + map-live + sameobject tentative）。码面 12 文件三域：css-parser（at_rules 族名列表文法 + 畸形嵌套吞块 + 值定义个数上限；serialize.rs 新档 canonical cssText 单源裁决；parse_layout font-variant-alternates character-variant 列表形；钉测随码）+ style-system（apply.rs 白名单 58→59 接入 + FFV character_variant 列表消费）+ engine（callbacks `__zw_parse_ffv_rule` + css_wire FontFeatureValues wire + part06.js insertRule FFV 分支/CSSFontFeatureValuesRule 物化；接手补齐三缺口＝FFV map prototype 补链 Map.prototype（map_iteration/map-live 三 F 根因）+ set 标量归一 + computed_style character-variant 列表串化）。探针：四 FFV 页 solo 全绿（主靶 302P/0F canonical）+ alternates-invalid 15P + alternates-valid 3P。**七语料 A/B**（15:49→16:23 双建 44s/45s，M2/S2 法证在档）：css-fonts +303（5441P→5744P）+ cssom +6（CSSFontFeatureValuesRule）＝**F2P=313 P2F=0**；masking/shapes/backgrounds/transitions/animations 五腿 ΔP=0 或 flake 级微动；异常三案法证全销：主靶 LOST=10＝b0 幻影子测（b1 302=探针 canonical）、transitions properties-value-inherit-001 LOST=560＝**预存挂起形**（solo A/B 双侧同 90s 硬超时，HEAD 即挂 fv1 面外，归 transitions 事件系统鲸面同域挂账）、animations NEW3/LOST1＝墙边陲 flake ΔP=0。详账与残面池（css-fonts b1 2501F 头九页 ~1525F 全在 animations/interpolation 族）见 [evidence knife-fv1](evidence/2026-11-18-knife-fv1-font-feature-values.md)。
**刀 A3b 收口：transitions 通道首帧量子剃刀（11-18 13:2x 收账，认领 +12 全语料位移 +15 零 P2F）**：A3 残面 transitions 通道 12F（neutral/inherit/unset × at{0.3,1.5} × {plain,all}）根因＝harness 采样形 progress=0.5+δ/100s 微扰（cubic-bezier 对称中点斜率 0.5 → eased 偏 ~1.5e-5）打中输出 .5 半边界翻转 oracle ε 形期望串（at0.3 串 `178`≠179、at1.5 外推臂 `109`≠110）；旧「keyword 端点解析失败」假设被探针推翻（inherit-resolved==explicit 恒等）改判 ε 面（A3 残面池 §1 已同步改判）。码面 transition_registry.rs 单文件两臂：①首帧量子守卫 `FIRST_FRAME_QUANTUM_S=0.016`（css-transitions-1 §starting-of-transitions 帧界定格，elapsed<16ms→active=−delay 解析 seek 点）**R2 收窄 bezier-only**——r2 电池实证守卫全域形对非 bezier 翻转 oracle（after-change steps(2,start) 同帧读 −3P→F + currentcolor linear at(0.5) 半边界 −2P→F，solo ×3 稳定复现定谳非窗噪），收窄后 linear/steps 保持墙钟 ε 形、steps t=0 臂不可达零行为差；②entry_eased 对称中点解析臂（`t==0.5 && x1==0 && x2==1` → y(0.5) 解析求值，消除共享 cubic_bezier 二分 20 轮 ±7e-7 容差噪声恰在半边界掷硬币；at=0.5 走 linear 不入臂、其余参数域共享路径零扰动）。钉测 4 枚（at0.3/at1.5 精确串二分必挂形 + R2 回归锁 steps-start 同帧读/linear seek ε 形两枚）。**r3 八腿电池**（R2 收窄形 bins b0=f8e3022e…/b1=a6393f34…，13:02→13:26 GATE-DONE BATTERY-RC=0 零 LEG-FAIL；r2 电池 12:12 定谳 P→F 触发收窄；r3 scope 形被 systemd-run 调用方 10min 上限级联杀→service 模式 WorkingDirectory=/ 坑 cd 内层修复重发）：主靶 **+12/0**（text-shadow-interpolation 147P→159P 与 solo 双 bin 精确一致、composition 0/0、after-change 页 b0==b1 恒等 R2 消除实证）+ backgrounds 白拿 +3（white-orange ε-face 双形 ×2 剃刀家族顺向 + bg-position-y fv1 漂移 ×1 窗噪第四次不认领）+ animations/masking 双腿 SET-IDENTICAL；transitions only-b1 560＝properties-value-inherit-001 unlocked 展开 async 既有形 ΔP 中性。残面池：CSSA/WAAPI unset 端点解析 9-10F（fork currentColor 流同域）、currentColor 元素色上下文（fork 流认领中）、共享 bezier 二分容差挂观测（电池零 P→F 实证无触界）、steps(·,start) t=0 精确臂（守卫收窄后不可达；未来 seek 精确化覆盖 steps timing 须先修此臂）。卷批事件与修复：本流暂存批曾被对端裸 commit 卷批（transient `082968a9b` 未推送），对端 rebase 修复剥离后重建 `ddb053b01` 仅含其 q2 补笔——本刀 4 文件完整还原（机制码 160+/7−、两份文档、master 收口段），改由本流标题提交入库 `3e17845ad`（22a942c1e..3e17845ad FF 推送兑现），全程法证 .acceptance/wptfyi-css/NOTES-a3b-sweep-082968a9b.txt 在档。详账见 [evidence 2026-11-18-knife-a3b-quantum-razor](evidence/2026-11-18-knife-a3b-quantum-razor.md)。
**刀 Q2 收口：shape-outside computed 三修（11-18 13:1x 收账，认领 +238 全语料位移 +241 零 P2F 零 lost）**：主靶 css-shapes shape-outside computed 域 168F（刀 Q p1 探针分类：精度 ~88 + calc 非 Px 静态叶 ~40 + ch 度量 ~40）三机制修复——①engine computed_style.rs `format_num` 千分位→6 位有效数字（CSSOM serialize decimal 精度口径，129 调用点单源；`50pt`→`66.6667px`）；②`calc_computed_to_css` CalcContext 补 x_height/ch_width（css-values-4 §5.1.1/§5.1.2，Ahem 0.8em/1em）解 Ex/Ch 静态叶整串空置；③style-system computed.rs Ch 兜底 0.5em→1em（css-values-4 §5.1.2 '0' advance，Ahem 语料 1ch=1em；Q p1 探针 50ch 800→400 定谳旧兜底恰 ×0.5）。旧真钉测改判 10 处（engine part03 3：circle 1889.764→1889.76 ×2 + aspect-ratio 1.778→1.77778；style-system 7：ch=0.5 16→32 形）+ 新钉测 2 枚（calc Ex/Ch 折 px、format_num 6-sig 边界）。**门禁**：style-system lib **2566/0**、engine lib 复跑 **4184/0**（同窗含对端 A3r WIP 共树共绿；首跑 4F＝3 钉测旧形改判 + 1 处窗内瞬态插桩——`tmp_probe_ts_lerp_exact_orange` 经 HEAD/A3 提交/工作树三重 grep 定谳非入库泄漏零提交义务）、定向 clippy 本刀两面零 lint（在案 3 lint 归 fv1/2d3e71d19 挂账）、fmt 双清。**十四腿 A/B**（双建 patch 往返 M2/S2 法证双绿，bins 12:14/15；电池 knife-q2-battery-r2 12:39→13:12 GATE-DONE 零 LEG-FAIL——12:35-12:39 三方 fork 双发竞态按先手优先仲裁收束、链 C 独写者收账、shapes 双 JSON 145 页 parse-ok 单一终态写者，NOTES-knife-q2-race-arbitration.txt 在档）：shapes 靶腿 **+238**/0 lost（页级：circle/ellipse-004 +63×2 Fix C 精度、inset-001 +46/ellipse-005 +30/polygon-004 +18 Fix A calc 叶、circle-005/shape-margin +8 Fix B ch）＝**认领 +238**（CLAIM 判据范围）；transitions/masking/fonts 三腿 SET-IDENTICAL（fonts 对照腿实证 Fix C 全局对 font computed 串化零位移）；animations ±1 对称 Timeout 摆动；typed-om +2＝animation/transition-delay 秒值串化（gCS 走 format_num 单源，`expected -0.00314 but got -0.003`，solo 双 bin 双观测同向 deterministic）＝Fix C favourable 按 CLAIM 判据不扩认领挂备注；backgrounds +1＝bg-position-x calc 多层串化跨跑漂移（A2/A3 先例第三次）窗噪不认领。残面池：shapes 残 1599F 靶面外、layout 本地 ch=0.5 口径不一致挂池、`Rch` 兜底 0.5 待驱动、format_num ≥1e6 科学计数口径 FIXME。详账见 [evidence 2026-11-18-knife-q2-shape-outside-computed](evidence/2026-11-18-knife-q2-shape-outside-computed.md)。
**刀 A3 收口：text-shadow 阴影列表插值三通道接入（11-18 11:2x 收账，认领 +127 全语料位移 +128 零 P2F）**：主靶 `css/css-transitions/animations/text-shadow-interpolation.html` 168 forms 152F/16P（A2 态基线）根因＝text-shadow 全通道零插值覆盖（registry 无条目+animation.rs 三臂全缺，回落离散 echo）。码面 2 文件（css-parser 零编辑）：registry 81→82 + animation.rs 宽松臂（`parse_text_shadow_list` 双端点、层数 max 零阴影补齐 none≡空表、`lerp_color` 预乘空间、blur 非负域钳 0 offset 不钳、em/rem ×16、串化镜像 computed_style text_shadow_to_css color-first/format_num 契约）+ strict 双端点可解析门（非列表对 None＝不启动过渡，css-transitions-1 §starting-of-transitions，同 va 臂形）+ applier 臂（resolve_effect_length 穿线写 style.text_shadow，canonicalize apply→串化往返共因第四次验证）+ 钉测 test_text_shadow_lerp 8 断言（首跑 0 passed 教训：漏 #[test] 属性的 fn 不入测试；恰集核 4178 总数）。**A/B 定谳**（八腿电池 knife-a3-battery-r2 11:10→11:25 GATE-DONE 零 LEG-FAIL；r1 包装 10min 上限被杀、css-transitions b0 完整收档含 rc 标记行后 r2 续跑七腿）：双建同窗 fv1 WIP 双侧同态（patch 往返换面不碰 stash 栈、trap EXIT 崩溃自恢复、M2/S1=S2 md5 法证绿；b0 `80fe1329…`/b1 `50d0142f…`）：主靶 +122（16→138P）零 P→F + composition 页 +5（replace→add 塌缩形；add/accumulate 55F 归池）全落两 ts 页；**population unlock**＝properties-value-001/-inherit-002 页级 Fail→Timeout + 560×2 子测展开（properties.js:214 表项 'text-shadow' + generalParallelTest transitionend 挂死解除；ΔP=0 不计 P，properties-value 池旧估 260F 被实测取代）；漂移腿 css-animations/masking 双腿 SET-IDENTICAL、css-backgrounds +1＝background-computed.html bg-position-y calc 多层串化断言（battery b1 单点 Pass 而 b0-solo/battery-b0/b1-solo 三跑全 Fail＝跨跑双向漂移窗噪，探针 solo.background-computed.{b0,b1}.json 在档 ⇒ 归 fv1 serialize.rs 域不认领，A2 同页族 bg-position-x 先例第二次）。门禁：本刀 hunks fmt 清（4 处 rustfmt 形修复 + 余 13 处既有漂移不代修）、定向 clippy rc=0（两个 -A 各带域外法证：script-sandbox es_module.rs:2211 too_many_arguments 8/7 归 fv1 WIP 流、animation.rs:2434/2435 unnecessary_cast 归 2d3e71d19 提交态；本刀自有两处 no-op cast 当场修复并补跑钉测绿）、钉测 1 passed/4177 filtered。残面池：关键字端点（neutral/inherit/unset）未解析计算值 ~24F + currentColor Animations/WAAPI 通道缺元素色 ~6F + composite add/accumulate 累积池 60F。详账见 [evidence 2026-11-18-knife-a3-text-shadow](evidence/2026-11-18-knife-a3-text-shadow.md)。

**刀 A3r 收口：text-shadow 残面 unset 级联向 + currentcolor 预解析（11-18 13:5x 收账，认领 +9 全语料位移 +8 零可归因 P2F）**：残面簇 A/B 的 CSS Animations 通道 9 行（WAAPI ~10F 归 FIXME(kf-inherit-waapi) 不动；Transitions 关键字行 12F 归刀 A3b 域）。码面单文件 animation.rs：①keyframe 端点 unset 按 `PropertyRegistry::is_inherited` 分流（css-cascade-4 §default：inherited 取父 computed、其余 initial；非继承属性与刀 K1 逐位一致）②`serialize_text_shadow_resolved`——含 currentcolor 的 text-shadow 列表端点装载时以 underlying 元素色定死（css-color-4 §5.4；插值臂无元素上下文）+ **属性门 name=="text-shadow"**（parse_text_shadow 裸长度默认 CurrentColor，无门误判改写 background/mask-position、border/mask-image outset/width 的 "Npx Npx" 端点）。**r2 回归实录（不入账）**：属性门缺席版 bin 八腿暴露 P→F×110（backgrounds −58 + masking −52，机制同上；三正例钉测全绿仍漏跨属性污染——八腿普查兜住）；修复=属性门+回归钉 `test_keyframe_non_text_shadow_untouched_by_currentcolor_resolver`（bgpos 40px 140px→80px 80px at(0.5)=60.00px 110.00px），masking −52→+0、backgrounds −58→−1 全愈；r2 的 +9 不预支，认领以 r3 为准。**A/B（r3 八腿，bin-knife-a3r，patch 往返 M2/S2 法证绿，face=animation.rs 单文件，fv1+Q2 WIP 双侧同态相消）**：transitions 814P→823P ΔP=+9 全落 text-shadow-interpolation.html CSS Animations: 族 unset×5+currentcolor×4 零 FOREIGN（b1 侧行数 1531→3212=pv001 页双稳满展开全 Fail ΔP=0 免疫）/ animations SET-IDENTICAL / masking SET-IDENTICAL / backgrounds −1＝bg-position-y calc 行窗噪第五次归 fv1 serialize 域（b1 bin solo 复测 Fail 同签名，solo.background-computed.r3b1.json 在档）。门禁：钉测 3 passed/4184 filtered、clippy rc=0（两 -A 沿 A3 法证）、本刀 hunks fmt 清（animation.rs 13 处既有漂移经 knife-q2 流 12:02 全文件 rustfmt 清偿随本刀一并落库，机械零行为）。残面池更新：WAAPI 通道 unset/currentcolor ~10F（kf-inherit-waapi）+ composite add/accumulate 池 60F 既持。详账见 [evidence 2026-11-18-knife-a3r-text-shadow-residual](evidence/2026-11-18-knife-a3r-text-shadow-residual.md)。

**刀 Z 收口：z-index 整数插值三通道接入（11-18 08:2x 落账 `b270f1bca`，本刀净 +158 零 P2F 零 lost）**：主靶 `css/css-transitions/animations/z-index-interpolation.html` 166F/250（刀 A1d 收账基线）根因＝z-index 全通道零覆盖——`TRANSITIONABLE_PROPERTIES` 无条目（transitions 不启动）+ animation.rs lenient/strict/applier 三臂全缺，整数插值回落 to-echo（from[-2]→to[5] at(0) got "5" 含 t0 伪 through，B0 伪通过池同形）。码面 2 文件净 +61/−2：registry 79→80 加 `z-index`（strict 门 auto/小数端点 None 不登记，静态跳终值）+ lenient 臂两端 `<integer>` 严格解析（`parse::<i64>`，+/- 号与 css-syntax 符号法一致）lerp 后 **round half towards +∞**（css-values-4 §8.1.2；f64 `.round()` 是 half-away-from-zero −0.5→−1 不符，用 `(v+0.5).floor()`）auto/小数端点离散回落（§8.5.6）+ strict 臂整数同型门 + applier 臂离散 echo 串落 `ZIndexValue` 槽溢出 i32 保旧值（css-position-3 §z-index）+ 钉测 `test_z_index_lerp` 10 断言随码（engine lib test 断编译挂账，补跑随树清窗）。**两腿 A/B**（b0=results-knife-a1d-v2；b1=bin-knife-z md5 `41cfb19f` 同树同 provenance——fv1 在飞 WIP 两侧同扫）：css-transitions 365P→529P **+164/−164 零 lost 零 REG**——本刀净账 +158（z-index-interpolation 84→242P，166F→8F）；混窗增益 +6 窗口法证归 fv1 已落库刀 T/U（transition-timing-function-valid +4 timing canonical、transition-property-valid +1 property 归一小写、all-interpolates +1）；css-masking 4701→4703P +2/−2 零 lost（clip-path-shape-parsing 2 形＝刀 T arc small/rotate 缺省补刀兑现，归 fv1 账）。残 8F 全 WAAPI 通道挂残面池（from[auto] 离散翻转 7F + neutral 面 1F）；z-index.html 4F（`Can set` → `Invalid unit for property`）＝part10-typed-om.js:1768 CSSStyleValue set 门 `<integer>` 缺表，归刀 P P2 池。门禁：`cargo check -p zero-engine --release` 过（5.77s）、本刀 hunks fmt 清（animation.rs 余 12 处 fmt diff 全刀 B1 已落库区，不代修）。详账见 [evidence 2026-11-18-knife-z-z-index-integer](evidence/2026-11-18-knife-z-z-index-integer.md)。
**刀 A2 收口：vertical-align 插值三通道接入（11-18 09:2x 落账 `79b70dce2`，净 +144 零 P2F；提交标题保守口径 +143 双口径见下）**：主靶 `css/css-transitions/animations/vertical-align-interpolation.html` 182 forms 136F/46P（刀 Z 态 solo 基线）根因＝vertical-align 全通道零插值覆盖（registry 无条目+animation.rs 三臂全缺，回落离散 echo）。码面 2 文件：registry 80→81 + animation.rs lenient 臂（LP 线性、baseline 端点按 0 长度参与、关键字对/关键字↔LP 50% 离散翻转、px↔% 跨单位 px 系数在前的 calc 系数形专属 blend_va_lp——与 rule-inset % 前缀形槽序相反）+ strict **真实可插值门**（首版恒 Some 被 solo 推翻：语料含 test_no_interpolation（super→40%），恒 Some 致 transitions 通道按 50% 翻转 −6F；修正后非插值对恒 None＝无过渡立即跳终值，css-transitions-1 §starting-of-transitions，CSSA/WAAPI 通道不经此门）+ applier 臂 + 钉测 `test_vertical_align_lerp` 15 断言（补跑 **PASS**：part116 残留恢复后 engine lib test 编译解，定向腿 1 passed/4176 filtered）。**八腿 A/B 定谳**（battery `knife-a2-r1-084503`，08:45→09:08 GATE-DONE EXIT=0 零 LEG-FAIL）：**b0 存档 bin `6541c6d3` 被本刀 v1 污染**——主靶页 167P/15F 恰合 v1 恒 Some solo 形（subtest 指纹逐条闭合：8 calc 混合 + 6 super→40% 误注册 + 1 WAAPI at(1)），打包窗树内已含 A2 v1 hunks → 电池实量 v1→v2 段 +6P/−6F 全落主靶页；全貌对刀 Z 落库态＝46P/136F→173P/9F **净 +127P**（solo 136F→15F→9F 三端点 × 电池两端点指纹链锚定）；**全语料补账净 +144**（刀 Z bin `41cfb19f` 全语料 529P/980F→battery b1 673P/835F 逐页 diff 零域外页，results-knife-z × results-knife-a2）：搭载面 +17＝composition 页 +16（Compositing CSS/WA 通道随 lenient 臂搭载，battery b0 已含=v1 态得；残 24F 归 composite:add accumulation 池）+ all-interpolates 等价页 +1（机制假设＝registry 条目使 `all` 通配探测 va，具体可证；刀 Z 窗同类翻转曾归 fv1 T/U 为反证注记）——**双口径仲裁**（b-stream note 09:29）：提交标题保守 +143 剔此例归窗噪、账面以 +144 为准，机制标题不追改。**重绑探针闭合**：落库后重建 bin `aeba37e1c`（fv1-s3 provenance 8 文件 md5 在档）solo 主靶页 173P/9F **Δ=0**＝v2 终态逐数复现，回退臂未触发；apply.rs:1081 va 臂系 HEAD 既有臂，A2 提交集维持两文件。归其系漂移：css-animations +1F（display-none-dont-cancel 新增子测试＝s560 display 臂 4d82f45e4 窗内编辑先于 b1 构建，其自述 display 臂 P2F 观察哨同域）、css-backgrounds +1P（bg-position-x computed 串化＝fv1 serialize.rs 域）、css-masking 零位移；b1 md5 `e625eff5` 电池前后双证未漂移。门禁：本刀 hunks fmt 清（钉测 9 处 assert 折行已修；animation.rs 余 14 处全既有 HEAD 漂移不代修）、clippy 零新增（余 3 处全既有提交态：too_many_arguments 8/7 对端挂账 + A1d 2345/2346 f64 cast）。残面：calc 混合对 9F（LengthValue 无 Calc 变体 FIXME(va-calc-slot) 跨切面）+ CSS2/linebox flexbox 静态布局 40F（layout 域）。详账见 [evidence 2026-11-18-knife-a2-vertical-align-interpolation](evidence/2026-11-18-knife-a2-vertical-align-interpolation.md)。
**刀 A1e 收口（11-18 08:4x，A1d 残面池 bg-position 三面双修+轴修净 +46 零 P2F）**：三修一暴露面——①`part03.js` `_zwPosList` += background-position-x/y（css-backgrounds-4 §2.4 `<position-x>#` 同文法逗号列表；门④整串单位 multiset 对帧间层数失配 2层→3层 误杀，WAAPI x/y 页 7F×2 全 at 无覆盖；探针三级定谳：solo stderr 21 BUILD-ENTRY=前 3 blocks、multi-layer block 未到引擎 register＝shim JS 门非引擎链；引擎全链 probe_waapi_multilayer_chain 复跑 AST/mix/tick/canonicalize 全通）；②transition_registry echo 例外清单 += bg-position 族 4 名（A1b mask-position 同参数 REG：同形对 t=1 插值≡echo 经 canonicalize 归一恒等；main 6F→0、origin 22F→14、x/y 2F×2→0）；③animation.rs `mask_pos_layer` 单值臂垂直关键字（top/bottom/start/end）入 Y x=center（css-backgrounds-3 §3 单值形 `bottom`≡`center bottom`——原恒入 X 致 `center`→`bottom` 对轴交换 8F，修后暴露面）。battery：r1（bin `5c8924b0` echo+shim 态，08:25）bg vs a1d-v2 **+38/−0 恰四靶页**（x+9/y+9/main+6/origin+14）+ transitions/masking/typed-om 三腿 SET-IDENTICAL；r2（bin `6541c6d3` +轴修）solo 四页 **700/700 全清**（origin 280/280 +8 精确兑现）+ bg 腿 +8 恰 `center→bottom` 8 断言 + masking SET-IDENTICAL。净 **+46**（bg 3442→3488），四腿五跑零外溢。残面池 trim：A1d 池 bg-position WAAPI 通道 14F + transitions echo 4F 认领清账。详账 [evidence 2026-11-18-knife-a1e-bg-position-waapi-shim-echo](evidence/2026-11-18-knife-a1e-bg-position-waapi-shim-echo.md)。

**刀 T/U/V 码面落库（11-18 08:0x 三笔推送，css-parser 域，探针待树清补账）**：刀 T `f14dde052` shape() 物理单位形文本级缺省补刀——arc `small` 修饰词/零值 `rotate` 对、控制点缺省参考系（by 形 `from start`/to 形 `from origin`）丢省，物理单位 guard（AST 通道 parse 期折 px 丢单位）值走 parse 门裁形 + 文本手术，语料钉面 shape-function-valid 3F + clip-path-shape-parsing canonical 2F；刀 U `77afe9d65` transition 族声明值 canonical——新建 serialize_transitions.rs（循 serialize_gradient 每族一模块先例）：timing step-start/step-end 关键字展开 + steps(N,end|jump-end) 缺省位省略（per-component 列表处置）、property all/none 归一小写 custom ident 保形，语料钉面 transition-timing-function-valid 4F + transition-property-valid 1F；刀 V `39eb7cbe5` clip-path 文本 canonical 严格性——inset 裸 `round`/circle·ellipse 裸 `at`/ellipse 单半径拒收（invalid 形此前被静默修整为合法形：inset(0px round)→inset(0px) 等 4 形实证，AST 臂本就严格、文本通道单侧缺口）+ 半径数裁形（circle 0|1、ellipse 0|2）。门禁：zero-css-parser **3310/0**（+6 钉测累计）、clippy 零告警、fmt clean。**探针挂树清批**（runner 重建需 engine 树清——peer WIP 在场禁建防毒化）：T/U/V 六页探针 + 正式六腿 A/B（与刀 Q/S 同窗）。**census 新挂（bin-knifes-p3 只读探针 95F）**：clip-path-shape-parsing 20F「should not set」经 parse 臂逐条独立核验均拒（tok[1] to|by 闸/`close path` 长度闸/`via` 无 with 臂/未闭合括号/`nonzero,` 入口段裁形）＝纯 validate_decl_value 门形归刀 R；transition-timing-function-invalid 25F（bezier 参域/steps 参域/token 汤形）+ transition-property-invalid 15F（css-wide 混入/数字 idents）+ transition-invalid 5F（简写多槽畸形）＝同门族刀 R 扩面；transition-shorthand 18F + transition-valid 3F＝`transition` 简写多槽展开缺失（style-system apply.rs 树清批新项——`1s -3s cubic-bezier(...) top` 五长手全空）；transition-behavior 23F＝简写 canonical 序（property duration timing delay behavior）+ supported 门混族；@starting-style 1F（parser/at_rules.rs 对端在飞避让）、clip-path-invalid path() 引号形 4F（engine CSSOM 层）挂账。
**刀 A1d 收口：background-position 单轴多层列表 + canonicalize 往返臂三通道接入（11-18 07:49 收账，本刀净 +499 零 P2F 零 LOST）**：靶面三页 + parsing/inheritance 面（A1c b1 账 x/y-interp 28F×2「should be supported」+ main 186F 静态 to 三类）。根因＝canonicalize 往返缺 applier 臂（ser1≡ser0→None→回落级联基值）；首轮电池 x/y transitions from[inherit] REG 24 三层根因（①`mask_pos_comp` 无 Calc 臂→mix None 回落 to；②`bg_position_axis_to_css` Calc→空串→canonicalize 往返 Some("")→got ""；③inherit.rs 缺 x/y 独立继承臂→from[inherit] 落 initial 非父值）；bisect 插曲＝EdgeOffset 符号口径（right/bottom 缘＝100%−偏移、偏移项取负，css-backgrounds-3 §3.6——旧码 +20，mask 语料无 right/bottom 对故 A1 账面未暴露，INH bin 伪通过靠 canonicalize None 回落 cssa 掩盖；修后 origin 258 超旧上限）。码面 6 文件：apply_advanced 单轴多层列表臂（css-backgrounds-4 §2.4 cyclic 层重复、all-or-nothing 拒收保旧值）+ animation.rs 委托臂/lenient x/y（mask_pos 机制复用 LCM 循环 lerp）/strict +2/mask_pos_comp Calc 臂（css-values-4 §10.11）/EdgeOffset 符号 + transition_registry 77→79 + computed_style gCS x/y 臂/font_size_px 线穿三串化器/Calc canonical 臂（calc_computed_to_css % 项保留零 px 项丢弃）+ inherit.rs x/y 臂（只补丁目标轴父轴取模）+ style-system 钉测 +1。**四腿 A/B**（b0bg+b2 双对账；b1=bin-knife-a1d-v2 `de8977d4`；发射撞车让位——采 sibling fork `wptfyi-a1d-v2r2` 现轮 corpus 账一账两用，bin 同 provenance，自 fork r3 半腿产物已清）：bg gained=1437/lost=0/regressed=0（对端 B1 面 +938 逐页恰合其账本＋本刀 +499 逐页分解闭合零残差：main 10→190、origin 18→258、x/y-interp 84→103×2、composition 0→2、parsing computed +3、x/y parsing-computed 0→16×2、inheritance 51→55——solo 预言逐页命中）；masking +15/lost=0（对端 B1 +1＋fv1 刀 S +14 认领，本刀零漂移＝EdgeOffset 符号修对 mask 语料零扰动证成）；transitions REG=0/lost=0（gained=267 全落 corpus 页集覆盖扩展 pooled 不认领；properties-value 族 3 页外部墙钟 Timeout 记账不缺案）；typed-om SET-IDENTICAL。门禁：style-system lib **2566/0**（+1 钉测）；engine lib test 断编译挂账对端（js_dom_bridge_tests.rs:208 part116 include! 悬空，part117 迁移半程）；本刀 hunks fmt 清（animation.rs 余 12 fmt diff 全 B1 已落库区，不代修）。残面挂账：WAAPI bg-position 列表通道 7F×2、transitions 简写对 echo 展开位 2F×2、transition 简写 behavior 槽 C6（~5-8F）。详账见 [evidence 2026-11-18-knife-a1d-bg-position-lists](evidence/2026-11-18-knife-a1d-bg-position-lists.md)。
**刀 B1 账本落册：border-image 三长手（slice/width/outset）动画家族接入三通道（11-18 07:1x bin `eaecaaec` 账本，净 +939 零 P2F；机制码已落库 `b410cbcb3`——hunk 级 index 精确提交，钉测 4 枚随码在册）**：
**刀 S 收口：clip-path specified 声明值 canonical 串化（11-18 07:4x 落账 `2692ac1f0`，css-parser 域探针级 14F，正式 A/B 待树清）**：css-masking/parsing/clip-path-valid 14F/54 根因＝clip-path canonical 走既有 AST 往返串化，14 形恒等回落或剥除语义错向（geometry-box 两序页源钉面**全剥**非保形——`test_valid_value("clip-path","circle(7% at 8% 9%) border-box","circle(7% at 8% 9%)")`）。修复＝文本级 `canonicalize_clip_path_specified`（css-masking-1 §the-clip-path + css-shapes-2 §serializing-basic-shape）：geometry-box 前缀/后缀全剥、裸 0 补 px（inset/xywh/rect/polygon）、circle 单 closest-side 与 ellipse 全 closest-side 缺省半径省略、polygon nonzero/全零 round 组省略、path() nonzero 省略 + 数据 token 规范（逗号→空白、z→Z、单空格连接；数据串内含逗号不可逗号切分，引号前缀判定单/双段）、at 位置重排复用刀 Q canonicalize_at_position；identity/url()/ray()/shape() passthrough 回落既有 AST 通道零漂移。**探针账**（bin-knifes-p3 md5 `37c812e0`）：clip-path-valid 14F→**0F**/54 满贯；刀 Q 三页（circle-002/004、inset-003）零漂移、shape-function valid/invalid 零 P2F；clip-path-invalid 48F + shape-function-invalid 40F＝同一 `validate_decl_value` 白名单门（style-system/apply.rs 对端在飞，shape-outside 先例同款接入即启用）+ clip-path-shape-parsing 22F（shape() 命令 canonical + invalid 门）+ clip-path-computed 17F（engine 面）挂账待树清。门禁：zero-css-parser **3306/0**（+4 钉测）、clippy 零告警、fmt clean；推放入 rebase 窗（远端来件 rendering-compat 纯 docs 零重叠，autostash 干净）。详账见 [evidence 2026-11-18-knife-s-clip-path-specified](evidence/2026-11-18-knife-s-clip-path-specified.md)。
**刀 Q p1 收口：shape-outside specified 声明值 canonical 串化（11-18 07:0x 落账 `3788f6c99`，css-parser 域探针级 416F，正式 A/B 待树清）**：css-shapes `shape-outside/values/` inline 通道根因定谳＝`testInlineStyle` 经 SetStyleOnHandle → `style_from_mutations_lw` → `fold_decl_specified`，shape-outside 不在串化白名单原文回显 ≠ oracle canonical 形（b0 344F：circle/ellipse-002 位置补全 29F×2、circle/ellipse-004 位置组 143F×2、inset-003 slash 折叠 52F）。修复＝文本级单源 `canonicalize_shape_outside_specified`（css-shapes-1 §shape-outside-property + css-values-3 \<position\>）：单 token 补全、双关键字水平先行重排（含 center 混合形 `top center`→`center top`——探针 p1 余 4F 钉出缺口）、4-token edge-offset 对序、inset round 垂直组等值 slash 折叠；CSS-wide keyword/var()/attr()/shape-box 前缀 passthrough。文本级而非 AST 往返＝物理单位 parse 期已折 px、AST 往返丢原单位而 oracle specified 保单位形（先例 g47 object-position canonicalize_position_decl 同语义）。**探针账**（b0=results-knife-p-b2 vs p2 bin md5 `76b768e6` 提交态重建）：circle-002 29F→**0F**/136、ellipse-002 29F→**0F**/136、circle-004 206F→63F、ellipse-004 206F→63F、inset-003 292F→240F＝**inline 面全清**（逐页实测净清 396F，含镜像页重复计 416F），余 fail 全为 engine computed 域，探针实测分类定谳：精度 88F（format_num 3 位小数 vs oracle toPrecision(6)，pt/mm 形及 calc 串内同病）、calc 非 Px 叶 drop 40F（Ex/Ch 叶 unwrap_or_default 空串，oracle 折 px：70ex→896px）、ch 度量 40F（oracle 1ch=Ahem '0' advance=1em=16px、我方 fallback 0.5em=8px，50ch 800→400 恰 ×0.5）合计 **168F**——刀 Q p2 挂账（D/F 需联动修）；**vmax 假说撤销**（p1/p2 权威数据 circle/ellipse-004 各 60 vmax 子测全 Pass，旧 ×0.5 观察出自状态串误解析期伪账、实为 ch 面伪移植，无需插桩）；inset 多半径截断 198F（ClipPathValue::Inset 单 radius parse 期丢垂直组，AST 改型）与对端动画域 clip-path 插值高碰撞 PARKED。门禁：zero-css-parser **3302/0**（+4 钉测）、clippy 零告警、fmt clean；正式六腿 A/B 待 engine/script-sandbox 树清后同窗补账。详账与探针谱系见 [evidence 2026-11-18-knife-q-shape-outside-specified](evidence/2026-11-18-knife-q-shape-outside-specified.md)。
**刀 A1c 收口：transition-behavior 字段建模 + allow-discrete 离散对面 + bg/object-position 插值臂（11-18 06:1x 落账，本刀净 +455 零 P2F 零 LOST）**：三机制（style-system 6 文件 + engine 3 文件，css-parser 零编辑）——①`transition-behavior` 字段全链：ComputedStyle `Vec<String>` + initial/known 表 + inherit 复位 + `parse_transition_behavior_list`（normal|allow-discrete 大小写不敏感逗号列表，未知整条拒收保旧值）+ apply 臂 + engine gCS 串化臂（`string_list_to_css(…,"normal")`）+ `transition` 简写第 5 槽（css-transitions-2 §shorthand，behavior 关键字分支置于 property 位识别之前——两槽皆合法 ident；省略槽归一 normal；9 处既有 arity 钉 4→5 同步，style-system 2565 全绿）；②登记/求值离散门：`register_from_style_change` 逐 index 读本位 allow-discrete 绕 TRANSITIONABLE_PROPERTIES 清单门 + all 通配扩 DISCRETE_TRANSITIONABLE 8 属性探针；`register_property_if_changed` strict 门 `!allow_discrete && strict None` 才拒登记；**discrete 旗标**（v2 修复）：strict None 豁免对 entry 记 `discrete:true`，求值端纯 `eased≥0.5→to` 翻转——v1 窗 multicol REG 36 定谳为豁免对走 lenient 数值化产伪中值（column-count auto 0 数值化 lerp "10" vs 期望翻转 "20"），v2 bin 修复后 REG 36→0 GAIN 20→54，v3 全腿零 REG 定谳；③bg-position/object-position：TRANSITIONABLE_PROPERTIES 72→74 + lenient 臂复用 mix_mask_position（mask-position 臂形逐字，None 离散回落）。**七腿 A/B**（b0 六腿=results-knife-p-b2 `2faea6ed` 刀 P 落账后基线——排除刀 P typed-om 混窗；bg 靶腿 b0=bin-knifen2 `8833233e` 现跑 b0bg；b1=bin-knife-a1c-v2 `26153f17`）：backgrounds +5 REG 0（b0bg 2000→2005）、gaps +24、multicol +54、shapes +96、masking +276、typed-om 零漂移＝**本刀净 +455**；fonts +330/−2（净 +328）REG 6 全在 font-feature-values serialization/font-variant-alternates——本刀 13 码文件 diff 对该面 grep=0 零触及，窗口法证归 fv1 在飞系其落账自我裁决。写面零编辑实证：`style.transitionBehavior` 走 shim 通用 camel→kebab（part03.js:31059）入 decl store，本刀前已通、仅缺 cascade 臂。外来码面：fv1 WIP（apply.rs whitelist/FFV 门、css-parser preserve_decl_case+serialize.rs、shim/callbacks/css_wire/es_module）不入本刀 pathspec。引擎钉测 2 枚已入册（allow-discrete 登记/关键字端点翻转），编译待 fv1 part116 悬空解除后补跑。详账与 bin 谱系见 [evidence 2026-11-18-knife-a1c-anim-transition-behavior](evidence/2026-11-18-knife-a1c-anim-transition-behavior.md)。
**刀 P 收口：typed-om known 门 + toStringTag + radius opaque（11-18 04:1x 落账，typed-om 净 +1125 零 P2F）**：b0 定谳 9 页 ~942F 同根＝`_zwOmPropSyntax` known 门缺表（SET 全落 "Unsupported property"）。四机制单文件 shim 刀（part10-typed-om.js）：①表扩 +~86 条（物理+逻辑 margin/padding/inset/border width-color-style/corner radius/scroll-margin+padding；nonneg −inset 6 +border 轴 width 2 +逻辑 corner 4 +scroll-padding 8——首轮 grep 无 range 回调误剔被探针 "expected CSSMathSum" 证伪回补）+ computed 三臂（宽度关键字 1/3/5px、padding 负 % 钳 0%、currentcolor 保形）+ `_zwOmLogicalPhysical` 22 条物理槽位 gCS 翻译（轴简写不入表——R5 探针证 host 无展开、翻译毒化 equal 面）；②`Symbol.toStringTag` 子类 tag 全集（`assert_class_string` 按 toString tag 判类——b0 全 corpus 337 unsupported + em/cm/calc 族根因；F2P 页集 9→120 页 corpus 杠杆）；④radius 页契约——四物理角 computed `assert_is_unsupported`（css-backgrounds-3 双值对 L1 不支持→基类 opaque）declared 照常解析（`_zwOmComputedUnsupportedProps` + computedStyleMap 闭包 `__zwOmComputedFace` 置位分支；逻辑角 computed 默认 equal 期望不在列，零 P2F）+ `'border-radius': []` 空表 opaque 往返（margin 简写同款臂）。**P2F 定谳修复**：①宽度臂原泛 `/-width$/` 门劫持 scrollbar-width（css-scrollbars-1 computed 为 as-specified 关键字，decl 'thin'→1px→表无 `<length>` 族→opaque 基类）——b1（dda6ab89）P2F=1 A/B 网捕获、peer a1b 电池（其 bin 扫入本刀 WIP）同面独立复现；收窄正例族门 `border-*-width/outline-width/column-rule-width` 随 b2（`2faea6ed`）落账，b1 留档捕获证据。**六腿 A/B**（b0=knifen2-b1 `8833233e` → b2=results-knife-p-b2，systemd 双轮错峰 a1b 电池后）：typed-om 靶腿 F2P=1125/120 页、P2F=0、lost=0、页集合全等；TOTAL gained=2410 lost=0 regressed=0 pages_gain=153 pages_loss=0（ab-compare 与逐腿独立复算双口径一致）；对照腿 gained 全归已入账流（fonts 856=N/n3、masking 336=A1 273+A1b 63、gaps 54/multicol 22/shapes 17=A1b），本刀净账 **+1125** 不重账。peer 自然验证：a1b bin typed-om F2P=1089（含①②无③④），b1=1089+36（③④ radius 32+简写 4）差值逐面精确吻合——其账面 "+96" 与其 battery json 实测 1089 不符，以其 json/TOTAL=2374 为准已注记。残面挂账 P2 池：axes 简写 computed em/cm/calc（declared 回落形）、calc-% 引擎折算（same-unit specified fold + computed Incompatible units）、block-size 引擎 gCS 臂、scroll/inset % 引擎形——残 146F。详账与 bin 谱系见 [evidence 2026-11-18-knife-p-typed-om-logical](evidence/2026-11-18-knife-p-typed-om-logical.md)。
**刀 A1b 收口：动画通道残面 R1–R3 三站三刀（11-18 03:5x 落账，本刀净 +156 零 P2F）**：R3 shim `_zwUnsafe` 单位门 position 族（mask/background/object-position）改逐层单位种类比较（paren-aware 顶层逗号分割、层 signature 忽略层数，css-masking-1 §5.3）——层数失配误杀整动画登记 +7 mismatched WAAPI 全翻；R1 transitions `query_core` eased==1.0 例外清单 +mask-position（at(1) echo 欠混合形，initial/unset 期望 calc 系数形、mismatched 期望 LCM 展开形）+6 Transitions at(1) 全翻；R2 WAAPI 帧端点 'inherit' 改写——corpus 元素 handle-only（sel 链 `__zw_parent`/wrapper `__zwSelector` 槽双不可用，探针 R2C 实证）走 `_zwNodeParent[handle]` 反链（值为对象，按字段提取 parentHandle）→ `__zw_get_computed_style_by_handle` 父 computed 改写帧值，父不可达保持原文回落兜底 +7 WAAPI inherit 全翻。单页探针谱系 176→183→189→189→**196P/196T 全清**（bin-knife-a1b md5 499b4f2e→27317410→1689833c→2cddb000 终态，TEMP-DBG 摘净 strings 验证）。**六腿 A/B**（b0=knifen2-b1 `8833233e` → b1=results-knife-a1b-b1）：TOTAL gained=2374 lost=1 regressed=1 pages_gain=153——本刀五腿（masking/fonts/gaps/multicol/shapes）**lost=0 零容忍成立**；本刀净账 **+156**（masking 净 +63＝mask-position +20/clip-path +25/mask-border +21/mask-image −3、gaps +54、multicol +22、shapes +17；跨页抽验翻面行 100% 为 WAAPI from[inherit]＝R2 属性无关杠杆）；fonts +856 重测归刀 N/n3 账（较其 +844 账面 +12 漂移为其后续落码，不重账）；typed-om +96 与唯一 lost=1（scrollbar-width 'thin' 34/34→33/34）窗口法证归 knife-P（part10 WIP 03:37:24 扫入电池 bin 03:41:16，静态关键词面本刀无通道可达）。mask-image −3 核注：harness 期望侧静态 gCS 'inherit' 关键字解析缺陷（interpolation-testcommon.js:383-387 inline 设值读 computed 返 'none'，父链 .parent mask-image url(blue) 应返 blue）——刀 A1 时 actual（E5 initial 回落）与 expected 两侧同错同值恰 Pass，本刀 R2 修 actual 侧为 spec 正确 blue（CSSA 链同证）后两侧分歧转 Fail；vs b0 该三行恒 Fail 零容忍不受影响，根子 cascade 关键字通道非本刀回归。外来码面处置：transition_registry.rs ZW_TRANS_DEBUG 探针块（兄弟 fork 03:36:36）commit prep 摘除恢复原 return 形、diff 纯净 +6；js_anim_registry/part01f/es_module/part10 外来 WIP 不入本刀 pathspec。门禁：六腿电池 bin 编译运行即编译门（Rust delta 仅 matches! +1 臂无 lint 面，共享树外流 WIP 在飞 clippy 免跑避噪音归因）；transition_registry.rs 单文件 rustfmt --check。详账与 bin 谱系见 [evidence 2026-11-18-knife-a1b-anim-residuals](evidence/2026-11-18-knife-a1b-anim-residuals.md)。
**刀 A1 收口：mask 长手族动画 applier 臂 + transitionable 清单补齐（11-18 03:0x 落账，masking 净 +273 零 P2F）**：双通道同病定谳——①`AnimationClock::apply_single_property` 缺 mask 八长手臂 → js_anim/cssa canonicalize `ser1≡ser0` 恒 None（与 clip-path 臂 11-16 同病）；②`TRANSITIONABLE_PROPERTIES` 缺 mask 族 → transitions 双形不启动。伴生③：px↔px 插值空串之谜——`mask_pos_fmt` 纯 length 对发 `calc(0% + Xpx)` 系数形 → 存不可解析 `Calc` 轴 → 轴串化 `""` → 层 `" "` 被 canonicalize 采纳；修＝`MaskPosComp` 三元组 + 纯 length 对发纯 `Xpx`（css-values-4 length↔length 产物是 length；含 % 项端点保 calc 形——`initial`/`unset` 块期望 inline 同 calc 形同通道比对）。②′ mask-image 例外剔除清单（65→72）+③′显式 `transition-property` 登记臂补清单 contains 门（原只 `all` 臂过门——applier 臂激活后 mask-image 显式离散启动 REGRESSED=30 at<0.5 全灭；门后静态跳终值 B0 正形；allow-discrete 对面 54F 随 `transition_behavior` 字段建模挂 A1b）。**六腿 A/B**（b0=results-knifen2-b1；b1 谱系 95969b20→bb21cf2b→ae9458e0→fa17c8dc 终 bin，resume-swarm 双 fork CLAIM 全程仲裁、三次 cp 撞窗腿作废重验）：终账 TOTAL gained=1117 lost=0 regressed=0 pages_loss=0——本刀账 masking 净 **+273**（mask-position 10→176P、mask-image 240→347P）corpus 逐子测复算 273/0/0/0 全零；fonts +844 记刀 N 三期 b2 账（其 entry 已反向记本刀位移旁证）；gaps/multicol/typed-om/shapes SET-IDENTICAL。门禁：clippy `-p zero-engine` rc=0（本刀文件零新增）、cargo check 过。残面挂账：mask-position 页 20F 分解 R1 端点钳制 raw 采纳/R2 WAAPI inherit 端点/R3 WAAPI 多层 keyframe 门；clip-path 文法站/basic-shape 插值/mask-border composition/mask 简写 no-interp 原池不变。详账与 bin 谱系见 [evidence 2026-11-18-knife-a1-masking-animations](evidence/2026-11-18-knife-a1-masking-animations.md)。
**刀 N 三期收口：font 简写 specified 重组 + supports 门 + computed 防逐出（11-18 b2 落账，fonts 净 +844 零 P2F）**：四站通道定谳——system-fonts 588F＝`CSS.supports('font',…)` 前置门死码（known_properties 无 `font`）、font-computed 309F＝computed 面无防逐出/组合、font-valid 306F＝inline raw 回显非 canonical、serialization-prevention 107F＝防逐出规则全集缺。码面 6 文件：`font_shorthand_reconstruct_from_style`（+184：声明串解析/位解析/覆写合并/防逐出门/canonical 组合，位解析单源抽出 `parse_font_shorthand_parts`）+ computed `font_shorthand_to_css` 防逐出门 + readProp `font` 旁路 ×2 + `decl_supported` font 直调臂；刀 O 注册门 3da1e3bbd 同 bin 复账（b1 `ba82502a` 含两者）。b1 首跑 gained=991/regressed=166 露三回归全属我面：test_font_family_parsing **−102**（同组 font-family setter 先行落库旧长手被重组覆写合并吃进＝缺声明序仲裁）、serialization-prevention −1（stretch 位门无 `normal` 误阻断）、subproperties-reset −1（knife O 开 apply 面后暴露 expand_font 复位集缺 css-fonts-4 §font-prop 子属性）→ b2 `7f6c7bb7` 全修：段序仲裁（font 前覆写跳过）+ stretch 门加 normal/emit 省略 + expand_font 三臂加 size-adjust/feature-settings/variation-settings 复位槽（计数钉 13→16）。b2 六腿终账 fonts 净 **+844**（font-valid +306 满额/font-computed +232/system-fonts +192/prevention +104/reset 页 +2 反哺）零 lost 零 regressed；gaps/multicol/typed-om/shapes 四对照腿 SET-IDENTICAL；masking 位移（mask-image +104/−30、mask-position +166）＝对端在飞 transition/animation 码演进旁证，记对端账不进本刀（对端＝刀 A1，已收口见上）。门禁：style-system lib **2563/0**、clippy --all-targets 0、fmt clean；engine lib test 挂账对端（part116 在飞删）。证据：[2026-11-18-knife-n3-font-serialization](evidence/2026-11-18-knife-n3-font-serialization.md)。
**刀 O 在途：font 简写 supports 注册门（11-18 02:2x 码面落库 3da1e3bbd，corpus 账已并入刀 N 三期 b2）**：同根定谳——system-fonts 588F（interpolation-testcommon `CSS.supports('font', from/to)` 前置门，588/588 单形 `'from' value should be supported`）+ font-computed 309/315F（排列空间 `is a supported value for font` 前置门）＝`decl_supported` 尾门 `known_properties().contains(&prop) && apply_property_value_with_quirks` 对简写 `font` 在 contains 短路——一期在 apply.rs 铺的 font oracle 臂（expand_font 单源）永不可达。三站补齐 3 文件 +114 行（registry known_properties + "font" + initial_value 占位臂 OptionalString(None)（值空间=长手组合无独立标量）；apply_initial_value / inherit_property 各加 `font` 委托臂——按 expand_font 13 槽展开清单重置/拷父，`font: initial`/`inherit` 与 `all:` 关键字级联此前静默 no-op 现转 spec 正形；font 非继承属性 is_inherited 不动）；注册表完整性钉测（known×initial_value 全覆盖 + apply_initial_value handle 全属性）逼出缺臂即红＝护栏有效。钉测 +1（`test_font_shorthand_registered_for_supports_gate`：from 全形/message-box 拒 16px/16px inherit）。门禁：style-system lib **2563/0**、clippy rc=0、fmt clean。**行为变更面**：`font: inherit` 语义从 no-op 转真继承（级联序后者胜者清 13 槽）——P2F 风险点在册，六腿 A/B（b0=results-knifen2-b1）落账前不入总账。挂账：corpus 探针/A/B/evidence 待 peer 落码（见头部卡点）。

**刀 N 二期收口：font-family 文法写面 CSSOM 收口（11-18 02:0x，608F 全清 +624）**：主靶 `css/css-fonts/test_font_family_parsing.html` 608F/1624P 全「应收被收」双通道形（rule + setter 各 304）——通道定谳＝一期门挂 cascade 面（expand_font/apply.rs）而 WPT 页走 **CSSOM 写面**（insertRule body `_parseDeclarations` → `_zwNormalizeDeclValue` 分派枢纽 / inline setter part03 通用臂 `__zw_validate_decl_value` T54 白名单），原文落库回显即 Fail＝一期 fonts corpus 零位移根因。**文法单源三站**：parse.rs `font_family_list_has_css_wide_keyword`（六键 inherit/initial/unset/revert/revert-layer/default，段级——单段单 token unquoted 才命中，`default simple` 段内序列与 quoted 段不命中）+ 本体门（仅列表多段且任段 keyword → 整条无效，单段单 keyword 是整条 CSS-wide keyword 由 cascade 前置臂处理）；shorthand expand_font family 位 keyword 门（不问段数——`font: 16px inherit` 单段亦拒）；apply.rs validate_decl_value 白名单 +2（font-family/font）+ 分派臂置于未实现函数放行臂**之前**（`simple()` 形 function token 会被放行臂先行误放，counter-increment `reversed(` 同序先例）——escape 词法 valid 组（`\\s imple`/`\\1f4a9`）与替换函数形放行保形（FIXME font-family 转义词法挂账）、font 臂委托 font_shorthand_supported（expand_font 单源，system font 臂 :3046 既有）；shim part06.js `_zwNormalizeDeclValue` 顶部 font 臂（宿主校验 false → null 拒＝声明剔除/赋值忽略，尾部 `!important` 先剥再校验，合法保形零重写 reject-only）补齐 rule 声明/规则样式 setter 两通道（inline 独立臂幂等）。**伴生修复**：`extract_random_args` byte-slice panic——font-family 入白名单后通用 random 门对 valid 语料 `納豆嫌い`/简写形 `16px 納豆嫌い` 崩溃（`&lower[i..i+7]` str 切片窗口**终点**落多字节字符中间——起点 ASCII 守卫不够，二次探针实证 end=7 落 '納' bytes 5..8 内；修复＝byte 切片比较语义等价，`[u8]` 切片无 char boundary 约束；random-item 处既有 byte 切片本已安全未动）。**门禁**：style-system lib **2562/0**（+2 钉测 `parse_font_family_css_wide_keyword_list_rejected`/`font_family_list_css_wide_keyword_detector`）、clippy `-p zero-style-system --all-targets -D warnings` rc=0、fmt clean；主靶单页 b1 探针 608F → **0F**（2232P）。**A/B 六腿**（b0=results-conv-b1v2 在册 v2 终账产物 `4ff1057d` 直作基线侧——同 bin 同码同机 2h 窗；b1=bin-knifen2 `8833233e`；01:42:04–01:48:43 六腿 + reconcile）：fonts 3914→4538 **F2P=624/P2F=0**（主靶 +608 清零 + font-family-invalid +6 引号混排组 + font-invalid +10 非 normal 重复/双 system keyword 组——expand_font 严格校验经新通道生效）、masking 4074/gaps 3838/multicol 943/typed-om 9947/shapes 5125 五对照 **SET-IDENTICAL**；TOTAL 27841→28465 **净 +624**；`gained=624 lost=0 regressed=0 pages_gain=3 pages_loss=0`＝P2F/GONE/pages_loss 零容忍全满足、F2P 全落 css-fonts 靶面 3 页＝界内。详账 [evidence 2026-11-18-knife-n2-font-family-grammar](evidence/2026-11-18-knife-n2-font-family-grammar.md)。**残面挂账更新**：font-family escape 词法（放行臂收窄）+ font-computed 309F + @font-feature-values 294F + font-variant computed 长尾 + system-fonts 588F；style-attr 通道（`_zwStyleAttrDeclFilter`）font 臂未挂（靶页不经此通道，g44 先例可扩）；engine clippy 挂账待 peer es_module 落码。


**收敛轮收口：mask 六长手补全刀 MI + font 刀 N 一期 + typed-om 盖戳修复（11-18 01:1x 收账，429 死绝 resume-swarm 双 fork 收敛轮）**：三刀一修复全链 11 码面文件——**刀 MI**（css-masking-1 §5.3–5.6 六长手 position/size/repeat/clip/origin/composite：css-parser `MaskGeometryBox/MaskClip/MaskComposite` 三枚举 + `parse_mask_{geometry_box,clip_layers,origin_layers,composite_layers}` 四解析器（SVG 几何盒 fill/stroke/view-box §5.4）+ position/size/repeat 复用 background 同构 list parser；style-system @supports oracle 六臂 + inherit 显式拷父六臂（css-cascade-4 §inherit，Inherited: no）+ apply_advanced 写值通道六臂（position/size/repeat background_* 镜像 + clip/origin/composite masking 枚举）；engine gCS 六串化臂 + animation.rs mask-position 逐槽 lerp `mix_mask_position` + image/mode/repeat/clip/origin/composite/mask 离散臂 t≥0.5 + 严格臂一行委托 K1 契约同型；编译修复 2＝shorthand E0425 `parse_font_family` 全路径 + computed_style `FontVariantCapsValue` 导入）＋**刀 N 一期**（font 简写展开严格化：font-stretch 位九关键字/`||` 逐位至多一次/normal 占位略过/family 位委托 `parse_font_family` 单源且门改 `.is_empty()`（css-fonts-4 §font-prop `<family-name>#` 逗号列表，`font: 14px Arial, sans-serif` 合法）+ gCS font 简写串化（variant 位/stretch canonical 反查表 §font-width-prop 同源/family 多 ident 免引号）+ apply.rs font supports oracle 臂；钉测 error_cases 2 例更新为 spec 正形（family 必填整条拒收，chrome 拒收实证））＋**typed-om 盖戳修复**（sibling fork 先手，本 session 采彼——`part10-typed-om.js _zwOmParseWithText` 补 opaque 实例 `_zwOmAssociatedProp` 盖戳 +7 行，chrome oracle 对齐）＋**columns 刀死靶撤码**（CLAIM v2 前提"chrome 不实现 column-height"被本机 chrome 探针推翻——gCS('columns') height 非 auto 恒输出 slash 段四形逐项＝R2755 Chromium 150 钉测复证，试砍代码全回退；教训双条：页面硬编码 expect 是陈旧 oracle 非现行为，归因需实机探针；`apply_property_value_with_quirks` supports oracle 臂不可用于有直接 apply 通道的简写——columns 臂曾拦截 apply_advanced:1841 直接通道致 r3749/zero-is-width 两钉测红，font 臂无冲突因 apply_advanced 无 font 臂）。**门禁**：style-system lib **2560/0**、clippy `-p zero-css-parser -p zero-style-system --all-targets -D warnings` rc=0、fmt clean（registry.rs mask-origin 臂前刀遗留漂移顺带折叠）、engine clippy **挂账**（dep 链连坐 peer es_module s419 在飞 8 参 too_many_arguments，peer 流落码后其门禁消化）。**A/B v1 十二腿**（b0=`bin-mm-b1` md5 `c2424b73` 收账态——本树 23:24 release bin 被在飞 mask WIP 污染弃用：masking 实测 F2287≠收账 F2336 差值恰 WIP 增益；b1=`bin-conv-b1` `156fd30b`；/tmp/conv-ab-svc-003955.log 00:39–00:56 rc 全 0）：masking 4012→4074 **F2P=56/P2F=0/NEW-Pass=6/GONE=0**，翻转全落 mask-repeat/mask-size/mask-position 三靶页＝刀 MI 界内；fonts 3914→3914 **零位移**＝刀 N 实装子面无回收（CLAIM-knife-n-fonts-parsing 主靶 font-family 文法 608F 属 css-parser 面未入实装切片，仍挂账）；gaps/multicol/shapes SET-IDENTICAL；typed-om **P2F=1** 触发零容忍归因闭环（单页双 bin 探针定谳非 flake：b0 14P/b1 13P1F；chrome oracle 三探：`computedStyleMap.get('mask-position')` 返基类 CSSStyleValue 无 typed 分型、writable attributeStyleMap 跨属性 append/set 基类值恒 TypeError 同属性 set 放行＝关联属性语义；插桩探针四点定谳 ZW 侧 opaque 实例 `_zwOmAssociatedProp`=undefined＝`_zwOmValidateSet` 关联错配门空转；机制＝刀 MI 补出 mask-position 计算串暴露 typed-om 潜伏分歧非刀 MI 缺陷——刀 MI 前 get 返 undefined append 掷 TypeError 测试假绿）。**A/B v2 六腿清偿**（sibling `bin-conv-b1v2` `4ff1057d`＝v1+盖戳，b0 沿用 v1 六腿；/tmp/conv-ab2-010306.log 01:03–01:11 rc 全 0）：typed-om 9947→9947 **SET-IDENTICAL 恢复**（P2F=1 清偿）、masking **F2P=56/P2F=0 保持**、fonts/gaps/multicol/shapes SET-IDENTICAL、六 corpus TOTAL 27779→27841 **净 +62**、页集合全等 page-only/GONE 零容忍全满足＝「申报=实测」谱系第四十六例。**resume-swarm 收敛注记**：双 fork 同窗（本 session v1 发现链+探针、sibling 修复+v2 验证，先手修复被后到方运行天然验证一账两用）；本 session 插桩探针页曾入 wpt-data 于 sibling typed-om 腿起跑前移除（页集零污染）；仲裁与角色分工 [.acceptance notes](../../../.acceptance/wptfyi-css/NOTES-conv-round-arbitration.txt)；详账 [evidence 2026-11-18-conv-mi-font-knife](evidence/2026-11-18-conv-mi-font-knife.md)。**残面挂账**：font-family 文法严格化 608F（刀 N 二期，css-parser 面）+ font-computed 309F + @font-feature-values 294F + font-variant computed 长尾 + system-fonts 588F；mask 组合/多层形 FIXME(mask-shorthand-grammar) + allow-discrete ~224F + kf-inherit-waapi 半面；columns 形2 allow-discrete/形3 WAAPI 端点回落在册；typed-om set-face 同属性 opaque 放行 vs chrome（probe3 实测 set('mask-position',opaque) NO THROW 同形待 corpus 面确认）；engine clippy 挂账清偿待 peer。

**mask 简写刀收口（11-17 23:1x 收账，FIXME(mask-shorthand-reset) 挂账兑现轮）**：11 文件批次——css-masking-1 §3.10 mask 简写展开（单值 `<mask-reference>` 形 13 投影：mask-image 保形 + 七长手初值 + mask-border-* 五长手初值；组合/多层形 FIXME(mask-shorthand-grammar) 拒收）+ mask-* 六长手站（position/size/repeat/clip/origin/composite：PropertyValue 六臂 + MaskGeometryBox/MaskClip/MaskComposite 三枚举（fill/stroke/view-box/no-clip/add/subtract/intersect/exclude，与 background 侧独立）+ ComputedStyle 六字段 + 初值 + registry known/initial +8 + inherit reset 六臂）+ 裸 0 双文法消歧（parse_border_image_outset unitless 0 收 Number 槽，css-values-4 §5.2，判别器 `token.parse::<f32>()` 失败自然区分 "0px"）+ inherit_property 五臂（mask-border-* 显式 inherit 拷父，css-cascade-4 §inherit；apply_initial_value 与 inherit_property 双函数结构——初值臂置 default 正确，显式 inherit 走后者）。钉测 shorthand/tests/mask.rs 3 + tests/basic.rs probe6 转正 1，style-system 常驻 2559/0，fmt/clippy 净。**A/B 十腿**（bin-mm-b0/bin-mm-b1 md5 双证 `06919b91`/`c2424b73`，22:45–22:50 落账）：css-masking 3687→4011 净 **+324**（F2P=325/**P2F=3**——三例 mask-border-source-interpolation WAAPI 通道 from-inherit=FIXME(kf-inherit-waapi) 既有挂账面假绿揭真红：b0 双侧 inherit 均无臂同落 none 互证假绿，b1 期望侧 inherit_property 五臂正确解析父链 url(green.png)、实际侧 WAAPI 父链不可达仍 none，非本刀行为回归（json b0=Pass/b1=Fail 逐条 message 实证）；inheritance.sub 8P/28F→**13P/23F**、mask-shorthand-subproperties-reset 死态转活全 Pass）、余四 corpus SET-IDENTICAL 全零位移、TOTAL 23540→23864（本刀 A/B 账；含 mask-border 刀累计 22221→23864 净 +1643）；本树 release bin（23:01）masking Fail=2336 与 mm-b1 同账复证；css-backgrounds 语料 bin-mm-b1 vs 本树 bin 日志 diff **逐字节零差**（裸 0 修复在 css-backgrounds 中性，收益面=masking reset 页 outset 断言）。**5F 归因链**：七层假说（parse/tokenizer/expand 序/apply/matching/canon/cascade 胜者）逐层实证排除，真因两件套=裸 0 值形歧义（"0px"≠初值站 "0"，页面自比较恒等驱动 Fail）+ inherit 通道缺臂；探针页 assert_true(false,值) 形态 Fail message 即探针数据，探针红≠行为红。**残面挂账**：border-image-outset calc 空格容错（split_whitespace 拆碎 `calc(0.5em + 10px)` → supported false，outset-computed 2 断言双腿同红既有 FIXME(border-image-calc-tokenize)，sign() 通道另叠）、CSS.supports('mask') 臂缺（decl_supported 尾部）、六长手 apply 写值通道 FIXME(mask-longhands-unimplemented)（仅初值/reset 通道生效）、组合/多层形 FIXME(mask-shorthand-grammar)（mask-valid 多层形红 2336F 大头）、composition/allow-discrete 承接 mask-border 刀。**协调注记**：与 regprop 刀（下块）同树窗口——regprop 摆位（11 文件 pop 回树）后 mask-m 流二次发射成功；首次发射（22:17）死于 regprop stash 窗 E0583（js_dom_bridge.rs mod 声明在树而 registered_properties.rs 被收走，mod/文件配对完整性检查值得进 A/B 脚本前置位）。详账 [evidence 2026-11-17-mask-shorthand-reset-knife](evidence/2026-11-17-mask-shorthand-reset-knife.md)。

**registerProperty 刀落地（11-17 22:4x 收账，死页定谳轮续）**：css-properties-values-api-1 全链 6 文件——shim JS 桥 `CSS.registerProperty`（part06.js 字典转换 + `--inherit-test-3` 锚定 typed 缺 inherits TypeError）→ `__zw_register_property` 回调（callbacks.rs 六参 + `kind|message` 错误串协议）→ 运行时注册表 `registered_properties.rs`（新建 ~640 行：syntax-string 文法 `*`/`normal`/`<type>[+|#]` 组合 + ident 字面量、禁名=CSS 宽关键字+`default`+component-name 而 **'none' 合法**（语料 `<url>|none`/`<transform-list>|none` 两页锚定）、Typed/Universal/Normal 三形、TypeError/SyntaxError/InvalidModificationError 精确映射、重名拒注册）→ gCS `--` 分支两通道 `registration_known` 咨询（computed_style.rs：var() 替换先行再分型串化，`rgb(128, 128, 128)`/`150deg`/`150s`/`150dppx`/`calc(0% + 50px)` canonical 形）→ 插值通道（animation.rs `--` 臂分型插值 + 不可插值 t≥0.5 离散翻转）。单测 3/3。**A/B 十二腿**（b0=`06919b91` HEAD+peer cs hunks 态·peer 11 文件离树窗法证；b1=`52393e2d` 刀全码·clippy 收敛后重建）：pva **148→566 净 +418**（F2P=396/P2F=0 零容忍、页集合 111=111 全等、GONE 39F+1T=死页标记清零、NEW 22P+199F=复活页新增断言池）、五对照 corpus **SET-IDENTICAL 全零位移**（gaps 3838/multicol 943/typedom 9947/shapes 5125/masking 3687、F2P/P2F/NEW/GONE 全零）、TOTAL 23688→24106。门禁：fmt clean、clippy default 形态 `-p zero-engine --all-targets -D warnings` rc=0（quickjs 形态 2×E0433=peer 刀 D v8 门控双臂域外在飞红，es_module stash 窗法证在册）。**碰撞事故在案**：peer mask-m ab4 与本刀 stash 栈交叉（其 B0 段 `git stash pop` 弹走我插队 stash 致其 B1 build E0583 `MM-BUILD-FAIL`）——b0 bin 从其 bin-mm-b0/ 找回（md5 双证）、peer B1 期望树态已摆位（11 文件 pop 回树）、跨流 stash 栈无归属协议教训入 [acceptance notes](../../../.acceptance/wptfyi-css/notes-regprop-knife-ab.md)。残面：FIXME(regprop-inherit) inherits 祖先链传播（style-system gather 未接运行时注册表）、FIXME(regprop-fs) em/rem 端点按根字号 16 近似、`@property` at-rule CSSOM（CSSPropertyRule，at-property-cssom 32F+at-property 16F）、transition 通道注册属性面；pva 余 Fail 795 为 R2 深水面（transform-list 组合/zoom/typed 边界）。详账 [acceptance reconcile](../../../.acceptance/wptfyi-css/evidence-regprop-reconcile.txt)。

**死页类定谳：setProperty 崩溃自愈、registerProperty 成新刀面（11-17 22:0x，续开轮收尾探针）**：本代 runner（`3877b8bd`，mask-border b1 同值）本地双套件探针（`.acceptance/wptfyi-css/results-deadpage-probe/`，不入库）——① css-transforms 107 页 stub **0**、css-properties-values-api 111 页 stub **39**：CI 11-14 验尸 402 死页的 `Cannot read properties of undefined (reading 'setProperty')` 崩溃类**已随 runner 代次自愈**（weekly 44017408145 起快照死页回收兑现中）；② pva 39 死页签名**全部** `TypeError: CSS.registerProperty is not a function`（animation/custom-property-animation-* 族）＝**shim 缺口定谳**：registerProperty 零命中（shim/script-sandbox grep），而 `@property` at-rule 管线**已在**（css-parser ast.rs:181 `PropertyRule`+parser/at_rules.rs:439 `consume_property_rule`+style-system `registered_property` 钉测）——下轮刀面＝**shim JS 桥 `CSS.registerProperty`（descriptor→registered-property registry）+ `CSSPropertyRule` CSSOM 面**，与 peer mask-longhands 在飞面（style-system registry/apply/gCS + animation.rs 插值）零文件交集，可并行；pva 39 页 declared=0 质量未入账、unlock 额度待修复后首跑实测（另有 css-typed-om/animation 系潜在消费面）。**weekly 兑现注记**：我 dispatch 的 run 44092401586（@cd33cfa3a）被 concurrency cancel-in-progress 砍尾（completed/cancelled），**successor weekly 44092733559 @58a962fb in_progress**（含我 push 的三笔与 peer gap-rules 刀 e43f6a0c8 等 3 新提交，我流已落后 3 待 rebase）——selection 恢复+死页回收+mask-border +1319 首次入快照的验收对象转为该 run，不重发。
**mask-border 刀落地（11-17 21:09 收账，429 死绝续开轮）**：css-masking-1 §3.4–3.9 五长手（source/slice/width/outset/repeat）全链 12 文件——style-system 七站（computed_style 五字段复用 BorderImage*ComputedValue 同构类型/default/inherit/apply_advanced 五臂/shorthand `ImageBorderShorthandSpec` 共享内核重构+mask-border 简写/supports+registry）+ engine 四面（gCS 六串化臂含简写/transition 清单 60→65/插值宽松臂五属性+严格臂一行委托 K1 契约/apply 五臂）。**oracle 定谳修正**：四槽插值逐槽 type 对应一致才 lerp、任一失配或 slice fill 不对称整条离散（expectFlip 0.5）——早期「slice 跨域 %↔number 逐槽 lerp 单位随 from」为误读；slice/width/outset 非负钳 0 + 裸无单位 0 双态歧义归一（parser parse_length 先收 Px(0)，'0'→'20' at:0.3='6' 非离散，钉测抓住）；px↔% calc 混合走 lerp_len 刀 G 同款；钉测 7/7、zero-engine lib 全量 4158/0F。**A/B 十腿**（verify-mask-border-ab.sh：b0=`36291827` HEAD 干净态 stash 12 文件精确清单重建硬证——现 release bin provenance 不明弃考古；b1=`3877b8bd` 工作树）：css-masking **F2P=1319/P2F=0**、余四 corpus SET-IDENTICAL 全零位移、TOTAL 22221→23540 **净 +1319**、页集合全等 page-only 零容忍满足。NEW F5+GONE Timeout1 同页同机制＝mask-shorthand-subproperties-reset 页 Timeout 死态转活，暴露 mask 简写未重置五 mask-border-* 长手（css-masking-1 §3.10）——FIXME(mask-shorthand-reset) 5F 挂账非 P2F。残面：composition 页/allow-discrete 挂账维持。详账见 [evidence 2026-11-17-mask-border-knife](evidence/2026-11-17-mask-border-knife.md)。**提交 5dadeee2c**（pre-rebase `51fd00537`，21:2x，guard PASS 零发现/FH-004 evidence KEEP 处置）。**push 兑现＋门禁对齐笔（21:5x，续开轮收尾）**：51fd00537 rebase 后换代 5dadeee2c，连 gate 对齐笔 980f9f053、刀序普查 docs aed377204 三笔已推 origin/main（autostash rebase 干净、unstaged WIP 字节还原、push 后 behind=0）；980f9f053＝animation.rs rustfmt 折行漂移（max_width=120）＋unnecessary_cast f64→f64 ×4（width/outset 两 px 臂 708/778）清偿、border_shape.rs J2 7071c0b30 遗留漂移同笔顺带——engine clippy -p×2 -D warnings rc=0（es_module s419 WIP stash 窗法证前后指纹一致）、style-system clippy rc=0、lib 4158/0F 复验；es_module too_many_arguments=peer WIP 连坐在册（flow 惯例该 crate 出 clippy 清单）。**mask 族刀序普查（21:3x，CLAIM-knife-mask-shorthand 已落）**：mask-image/mask-mode 有臂（apply_advanced:2338+gCS:544，registry 名目缺）而 animations 段 408F+196F+42F 全红=插值臂缺失非站缺失；mask-position/size/repeat/clip/origin/composite/type 三面全缺站（apply/gCS/reg 零命中）；mask 简写全缺（shorthand map 无条目）。子刀序：mask-longhands（position/size/repeat 三站，bg_* 同构先例，主靶 mask-position-interpolation 196F+parsing ~90F）→ mask-shorthand（简写+§3.10 reset，依赖三站）→ mask-image-interp（408F 池）。
**三刀（J2/K/K1）收账落地（11-17 19:5x 收口，429 死绝后续开轮）**：chain3（4-bin×五 corpus，15:10–15:41）判读→四笔提交落树→提交后五 corpus 复验。**提交**：7071c0b30 J2 残面（border_shape+helpers paint 面；parse_extended_border 括号感知切分被 corpus 判据否决净 −49 **整体出树**，serialize 模块+mod.rs 声明同出，WIP 存 parser-face-j2-wip.patch 归刀 L 候选）/ 64085b147 registry.rs 纯 fmt（pre-rebase bfbb0596e） / 4ff161f72 K（九文件）/ 531e795e7 K1（六文件）；animation.rs hunk 拆分 K=6/K1=15（K 过滤器过时行号误收 5353 对端钉测 hunk，内容定谳归 K1；并集 21=全量、工作树反向 apply 字节闭环、两快照 cargo check --tests 绿、pre-commit-guard PASS ×3）。**bin 账**（reconcile-j23-*.txt）：K=multicol +176（F2P 192/P2F 16，column-width/count/columns-interpolation +86/+57/+32）余四 SET-IDENTICAL；K1=净 +98（gaps CSSA 半面 50 wins 主靶✓＋multicol 22/shapes 17/masking 14 正面外溢；回归 5 全落 WAAPI 通道端点=FIXME(kf-inherit-waapi)）；J2 parse 面=否决在案。**提交后复验**（bin-j23r3 `5005a82d`，构建于 pre-rebase 0b1d1cece 树，reconcile-j23r3-postcommit-vs-J2A.txt）：vs bin-j23J2A 五 corpus **SET-IDENTICAL 全零位移**（gaps 3838/multicol 943/typedom 9947/shapes 5125/masking 2368/TOTAL 22221=22221，F2P/P2F/NEW/GONE 全零，页集合全等）＝组合预测兑现：K +176 与 K1 +98 入账、J2 残面 corpus 中性实证、parse 否决面出树后 shapes/masking 回落 J2A 水平；provenance 注记：rebase（0b1d1cece→531e795e7）带入 editing 面 shim/pins（part01g/part06/part132-134）与 bins 及复验二进制同旧 shim 态，且 css corpus 无该面消费路径，对比双侧自洽。**探针**：J2 border-shape lens absolute-coords 靶页 ✗4.67%→✓0.39%（css-borders 44/49→45/49）、arc-miter-limit 0.44%→0.31% 仍 ✗；K multicol lens reftest 全套+upstream 双族零页级翻转（+176 全 testharness 断言面）。**观察与下一步**：① weekly 44017408145 聚合守卫拦回写——**selection 34041→0 全灭已定谳修复**（78d4f081f，pre-rebase 05dfd8d98 注记，11-17 20:1x，CI 执行面非 css 码面）：runner filter 分通道不一致——editing-run/test262 通道 `split('|').any`（fb30d4f08 pipe 组惯例）、selection 通道两收集器落单字面 `contains`，ef429d0e9（11-15 whitespaces 拆片）给 face 片写竖线联合 filter 后**恒零匹配恒 `[]`**；证据链：fetch 步正常（182 案在盘）→ runner 0.28s 空真绿退出（rc=1 用例集空）→ ws 片（单子串 filter）同 run 真账 423KB 对照 → A/B 双 bin 本地闭环（修复前 `5005a82d` pages=0 复现 / 修复后 `36291827` pages=2，collapse-00 2655/2655）；修复=selection 通道两收集器补 `split('|').any`（单子串零漂移，ws 片与本地 FILTER= 用法不变）；入 main 后下轮 weekly selection 组恢复入账（face+ws 并集 ≈34041+，ef429d0e9 扩面新文件首跑可升），详账 [evidence 2026-11-17-selection-face-filter-fix](evidence/2026-11-17-selection-face-filter-fix.md)；② mask-border 刀已落地 +1319（首段；残池=composition 页+mask-shorthand-reset 5F+allow-discrete）；③ CSS.supports columns slash 通道 222F 簇、allow-discrete ~224F、kf-inherit-waapi 半面 50F+F 池在册。
**三刀（K/K1/J2）收账接管·4-bin 对账链实录（11-17 15:10，rate-limit 死绝后单 session 接手；终局见上段）**：① 编排器 RC=20（style-system 10 红）根因=刀 K fork 三缺陷（bare `/` 段拒收、em 不解析致 paint/gCS 消费端静默丢失、14 个 arity 2 过时测试）——三处修复+测试翻新后三腿全绿：css-parser 3299/0F、style-system 2555/0F、engine 4148/0F（test-guard 包裹收账）；J2 面同窗收尾（sin_cos 解构反置致弧顶点转置 4147→4148/0F、serialize_shape_specified 六处 canonical 修复、`circle(at)` 空位置拒收补、inset calc 拒收边界按刀 G 残面⑤自记条件撤除翻新为括号感知接受面——corpus 归 A/B 定谳）；fmt/clippy 三 crate 净（css-parser 5 错清偿：dead fn 删、identical if 折叠、match→? ×3；engine 唯一 warning PathSeg 转 `#[cfg(test)]` use）。② 三刀联合 4-bin 原子窗建成（build-j23-bins.sh：B=全码面 `78abca73`/KA=K 六文件+computed.rs RC20 臂+K hunks 反向 `17f80e5d`/K1A=五独占+15 hunks 反向 `0cc73360`/J2A=J2 五文件反向 `d502b182`，快照指纹 pre/post 逐字节全等）。③ **双 spawn 毒化事故**：resume 恢复竞态两链并发（chain+r2）同写 transcript/results→块体交错 rc=2 零产出——全树清查停链、毒工件清除、launch 脚本加 flock 单例锁根修后 chain3 单写者重发（15:10:49 起，20 腿 4 bin × 五 corpus）。④ 链后：五 corpus reconcile（K 判据 multicol F2P 列页/K1 判据 gaps F2P≈50 仅 CSSA 半面/J2 判据五 corpus SET-IDENTICAL）→ reftest 探针（J2 border-shape 两靶页/K multicol 列页）→ evidence 回填 → per-knife pathspec 提交（animation.rs hunk 拆分，es_module/examples 一概不入）→ pre-commit-guard → pull --rebase（远端 d582d2ee9，9 提交 docs+editing 面、与本刀码面零冲突预判）→ push。
**观察面注记：mask-border 无主池＋K1 收账基建＋编排器自足性（11-17 12:58，verify 角色轮续段，零码面）**：① mask-border 属性族整池勘察落档 [notes-mask-border-family-knife-scope](../../../.acceptance/wptfyi-css/notes-mask-border-family-knife-scope.md)——css-masking 8 页 **1791F/0P 全灭**与 9693736da 规划节精确吻合、认领板无主；五站（解析→registry/shorthand→apply→gCS→动画）零接线定谳（grep 法证），插值 4 页 ~1454F 可回收为主、composition 336F 二期，下一刀候选。② K1 收账基建就绪：[verify-k1-expected-f2p.txt](../../../.acceptance/wptfyi-css/verify-k1-expected-f2p.txt)（results-cssa-h1-full 四 inherit 页逐断言基线，页级 FAIL=136 含 WAAPI 半面与既有面，判据仍按 claim「CSSA 半面 F2P≈50/P2F=0/余四 corpus SET-IDENTICAL」，evidence 模板旧账 100F 已注漂移）——编排器 RC=42 或 fork 死绝后任意后续轮可凭此收账。③ orchestrate-cssa-k.sh 核读＝**全自足链**：探测 60×120s 上限（RC=42 ~14:00），判据 css-parser 编译绿＋quiet 四条件（runner 腿/手术脚本/cargo/animation.rs 180s 静默），成功路径钉测三批→build-cssa-k-bins.sh 手术→B/A′ 链全自动发射，无需 fork session 在场——session 死绝不影响链落地，只影响 RC=42 后的隔离窗接管。④ fork 群活性：3×resume 进程存活爬行（CPU ~1%）、transcript ~23min 无增长、J2 css-parser 面 >1h 零触碰——**不轻判死**（前轮误判自纠在案），继承评估留给编排器 RC=42 事实。⑤ weekly coverage run 44017408145 in_progress（12:28 起）：checkout 早于本地 ad487ea89 落树＝本轮快照**不含**刀 H +145，push 欠账（本地 4 提交，behind 1 d08f8ef17 docs-only）在编排器停探＋树态 provenance 可证后再 rebase 带 push。
**刀 H·typed-om shim WIP 认领落地（11-17 12:44 收口，e2 孤儿在途码两轮对称消去验证，r1 +71 → r2 修复 +74）**：刀 I 碰头账披露靶面后按认领板认领（CLAIM-knife-h-typedom-round）。r1（原 WIP 181+/24−，camelCase 归一+set/append 校验门族+get 面扩容，A′ `6e60e480`/B `8303d45a` 十腿全 rc=0）：五 corpus SET-IDENTICAL ×4＋typedom 9873→9944 净 +71（F2P 74/P2F 3）＝刀 I 碰头账预测逐位兑现。**r1 3 键 P2F 定谳为真实面回归并 r2 清偿**——properties/{all,background,margin}.html "Can set ... to var() references" 失败消息 `expected "CSSUnparsedValue" but got "CSSStyleValue"`：harness set 输入是 CSSUnparsedValue 实例走 kind='unparsed' 分支不触 token 门、get shorthand 回落臂错返 opaque 基类，spec css-typed-om-1 §get 值归一要求 var() 引用文本恒 CSSUnparsedValue（node 复演定谳码面矛盾：字符串形不进本脸；margin-* 长hand "Unsupported property" 全家族 F 双侧同相为既有缺口不入账）。r2 同根因补两处（191+/24−，指纹 -r2）：mapGetAll 回落臂 var() 文本返 `CSSUnparsedValue(_zwOmTokenizeVar(text))`（harness `specifiedAlternateExpected` CSSWG#13792 备选形态命中）＋`_zwOmCheckShorthandTokens` 折叠占位 `@` 接受臂（折叠 regex 把 calc/var/url 折 '@' 后无接受臂＝字符串 var()/url() shorthand set 同根错杀，完成折叠作者意图；语料静默零额外位移实测）。r2 账（隔离窗构建：姊妹 css-parser serialize_shape_specified 半成品 5 编译错卡树 33+ 分钟，stash 两文件隔离 87s 窗 A′2→pop→B2→pop 逐字节恢复＋指纹法证 B=1/A′=0；A′2 `313394d1`/B2 `318b424d`，HEAD=84cb6ab00（发射时 9b50a92e7，landing rebase 换基），十腿全 rc=0）：五 corpus SET-IDENTICAL ×4＋typedom 9873→9947 **净 +74（F2P 74/P2F 0）**，三脸双侧逐键 Pass 直查，verify 角色轮独立复算确证「申报=实测」（同面 r1+r2 累计 +145）。A′2 基数较 r1 抬升（gaps +46/multicol +201/shapes +17/masking +13）＝未落地 peer WIP 组合态增量双侧对称抵消不入账。残面：margin-* longhand 缺口后续刀候选、_zwOmExpandShorthand all/background 留 FIXME。详账见 [evidence 2026-11-17-knife-h-typedom-wip](evidence/2026-11-17-knife-h-typedom-wip.md)。
**刀 H2 复算确证（11-17 12:38，verify 角色轮）**：H2 双链（A′ bin-cssa-haprime2 `313394d1` 12:25 链 / B bin-cssa-h2 `318b424d` 12:24 建，log-cssa-h2chain-122539）五 corpus 落盘后本方独立逐数复算——typedom 9873→9947 **净 +74**（F2P=74/P2F=0/NEW-GONE 断言全零，StylePropertyMap get/getAll/append/set/delete/clear/has/declared/disconnected 面）、gaps/multicol/shapes/masking 四 corpus P/F/T/pages 逐数恒等 SET-IDENTICAL＝「申报=实测」预期成立；与 H1 +71 同面累计 **+145**，收尾（evidence+master 主笔+part10 提交）归 H fork。**组合态注记**：A′(haprime2) 对 h1 基线揭示未落地 peer WIP 组合态增量——multicol 757→958 **+201P**（repeater/columns 流在飞面）、gaps +46、shapes +17、masking +13（typedom 恒等）＝各归属流落地时按五 corpus 工法自行认领，不入 H2 账。
**CI 执行面定谳＋coverage 重跑已 dispatch（11-17 12:35，rally 续开轮，观察姿态零码面）**：11-14 weekly run 43615261973 coverage 工件全量验尸（73 css 套件 JSON：7,081 页/114,088 子测/P 62,687；快照权威账 css zw_passes=62,896、covered_files 7,036/35,983、den_subtests 197,163——差 209=aggregate 合并含 face 分片外语料，方向健康）——**缺口 134,267 三分解**：① **死页 402**（每页恰 1 stub Fail `page script threw (declared tests: 0)`，如 masking clip-composition `TypeError: Cannot read properties of undefined (reading 'setProperty')`）＝11-14 CI 旧 runner 页脚本崩溃；**live 页零截断**——5 corpus CI live 子测和与本地五代 runner 逐数 IDENT（gaps 1768/multicol 341/typedom 12357/shapes 5291/masking 522）且页集合恒等（masking 61=61）＝无 fetch 缺口；死页本地 yield **P=6,158**（gaps 2,168/masking 2,173/shapes 1,198/multicol 619/typedom 0），余 68 套件 356 死页无本地账按均值 ~134 P/页上限估 ~54k（下限 6.2k，重跑实测）；② covered 页 F ~51.4k＝真实兼容缺口（刀群主战场不变）；③ **未拉取文件 28,947/35,983（80.4%，子测 ~83k）**＝DC-2/DC-3 corpus 扩面主杠杆。**行动**：weekly `run_wptfyi_coverage=true` 已 dispatch（run **44017408145**，runner 自 origin/main 当前代构建＝刀群修复自动入账），下轮核验快照 css 分子 62,896→预期 73k+ 与残余死页清单（当前 runner 仍崩类=下轮刀面线索）。预算闭合粗算：62,896＋死页回收（6.2k~54k）＋live F 51.4k×修复率＋未拉取 83k×yield——+153k 需三杠杆并用，机械杠杆（死页回收+扩面）优先。工件的 `-p` glob 翻页截断坑（per_page=30 漏看 v2 分片）与普查命令见 [evidence 2026-11-17-ci-face-autopsy-coverage-rerun](evidence/2026-11-17-ci-face-autopsy-coverage-rerun.md)。fork 群监视：watch-k-land 42min 无事件到期正常回收（三链未落地）；K orch unit（zw-cssa-k-orch-120100）在跑，让位纪律维持。
**战役态注记（11-17 11:59，rally rate-limit 续开轮，零码面观察姿态）**：前轮刀 K 遭 rate-limit ×10 中断（11:42）；用户侧手工恢复 5×resume fork 群（同 id b2dab9f0，transcript 共写），以 `.acceptance/wptfyi-css/CLAIM-*.txt` 认领板协调多刀并行（停己采彼仲裁）。让位重估＝对方编辑活跃（typed-om.js 11:51:23）＋多刀认领/树态实证在飞，**全码面继续不碰**，本轮推独立对账面：
- **刀 H·typed-om shim**（认领板 CLAIM-knife-h-typedom-round）：H1 双链 11:27 收账，本方独立复算 results-cssa-{haprime,h1}-full 五 corpus JSON 逐数确证 typedom 9873→9944 **净 +71**（F2P 74/P2F 3，StylePropertyMap case-insensitive/list-valued 面）、gaps/multicol/shapes/masking 四 corpus P/F/T/pages 全恒等 SET-IDENTICAL＝「申报=实测」；收尾（evidence+master+提交）归 H fork，其已转 H2 轮（orchestrator zw-cssa-h2r2-115324 探针轮询 PROBE-1..4@11:53-11:58 待树编译恢复）。part10-typed-om.js 树态已含 H2 增量（claim 报 181+/24−，现 diff +205），H1 收口提交需分代核差。
- **刀 K1·inherit-keyframe cssa 半面**（CLAIM-knife-k1-inherit 11:47 修订）：五独占文件（cssa_registry/callbacks/computed_style_cache/pipeline mod+budget）＋animation.rs hunk 级手术（与 repeater 流共文件，build-cssa-k1-bins.sh 带 180s 静默门＋反向补丁拆分）；WAAPI 半面挂 FIXME(kf-inherit-waapi)；bin 构建待上游编译绿。
- **J2·shape()/path() 命令串接几何**（树态实证认领，launch-cssa-j2-full/j2aprime 11:49 备射）：border_shape.rs+helpers（刀 J 留 FIXME scope）＋css-parser parse_extended_border/serialize_shape_specified.rs 新档同簇在飞；**11:52-11:57 k-compile-poll css-parser rc=101（E0004/E0277/E0382/E0599 五错）＝树暂不可编译**，K1/H2 bin 构建全队列等 compile 恢复。
- **未上板在飞面**（H claim 窗口注记口径）：es_module（长期 s419 WIP）/registry-shorthand/computed_style 族/examples 删集——fork 群 A/B 以对称消去吸收，本方零碰。
**继承预案**：fork 群若再遭 rate-limit 死绝，按认领板优先收口已验证增量（H1 +71 首位，其余按 CLAIM 文件＋树态 diff 分代接手）。远端 behind 1（d08f8ef17 邻域 docs-only）零文件冲突，本笔本地提交后随下轮 landing 流 pull --rebase 带 push。
**碰头注记（11-17 11:21，刀 K 关键字面让位）**：本方开展 keyframe 关键字面残余池（inherit/initial 136F，row-rule-color E5 站 currentcolor-form 等）普查后实施 `AnimationState.keywords_resolved` 字段方案；应用时发现 `animation.rs` 已被并发同流会话修改（11:21:05，刀 K1 `start_animation_with_underlying` 增 `parent: Option<&ComputedStyle>` 第四参 + start 站 inherit 解析臂 + WAAPI/cssa 通道 None + FIXME(kf-inherit-waapi)，cargo check 在跑＝活跃认领实证）＝同面撞车。按碰头纪律本方撤回自有编辑（字段方案无初始化点会破坏对方构建），核 `git diff` 余量 +85/−6 全为对方在飞工作后停手；让位判断随监视事件全程重估（对方落地前本方不碰 animation.rs 及其插值站：插值类页面 mask-border/columns/shape-outside-interpolation 族全在该站）。
**刀 J·rect/xywh vertices 回归修复已落地（11-17，rendering-compat 在册 border-shape-shadow 族 3 键确定性回归定谳清偿＝GONE 3 验证点兑现）**：根因＝07fcc9e0e basic-shape knife 为 `Shape|Path|Rect|Xywh` 四兄弟加空顶点 FIXME 臂——三页 `border-shape: xywh(0% 0% 100% 100%)` 解析落地后 border-shape 激活、空顶点致背景 `background_vertices()` 空 path_fill（绿底失）+ Stroke plan 空描边（黑边框失）、仅 box-shadow 独立通道留画。**交付面**：`border_shape_vertices` 补 Xywh 臂（x,y 起点 + w,h 尺寸 % 相对参考盒）与 Rect 臂（缘位置语义：top/left auto→0、right/bottom auto→参考盒右/下缘）+ `rect_shape_vertices` 共享发射（顺时针四顶点与 Inset 臂同 winding；round 槽 h/v 半径相对内矩形宽/高解析、单角钳制 min(iw,ih)/2 复用 `rounded_corner_polygon_in_box`，§5.5 重叠缩放留 FIXME）；**scope 决策**：Shape/Path 命令串接几何另属独立刀面保持收窄 FIXME（三靶页仅消费 Xywh；absolute-coords-shape 等 5 F 属独立键非本刀账）。**验证**：探针差 vintage 行为锚——A′ bin 三页 ✗ F 7.69/9.19/10.08%（与在册回归速率逐页吻合）/ B bin 三页 ✓ **P 0.00%**；**B↔A′ 五 corpus SET-IDENTICAL**（21907=21907，F2P/P2F/NEW/GONE 全零、页集合全等、十腿全 rc=0）＝修复零 corpus 扰动。engine lib **4,134/0F**（+3 钉；首版 offset_round 钉弧顶判据写错修正，几何实现正确）。**A/B 设计注记**：A′ bin 现建（stash border_shape.rs → build → pop）不复用 bin-cssa-i1——五 peer WIP 文件 mtime 10:23 晚于 i1 构建，现建保证对称消去无假设；实证 A′ 总账与刀 I A′ 逐 corpus 恒等＝10:23 peer 事件语义零差回溯定谳。本刀 1 文件提交以显式 pathspec 限定，peer WIP 一概留树。详账与 provenance（A′ `5ce2b39f`/B `5423a44f`）见 [evidence 2026-11-17-knife-j-rect-xywh-vertices](evidence/2026-11-17-knife-j-rect-xywh-vertices.md)。
**刀 I·important-tier 回归修复已落地（11-17，rendering-compat 在册 important-prop 确定性回归定谳清偿）**：根因四层全修——（A）`block_to_point` 多选择器折叠取首选择器致 `from,to` 丢 100% 帧、合成空边界帧被刀 F 通用臂填值漂移 → `block_to_points` 逐选择器展开（css-animations-1 §keyframes）+ `register_keyframes` flat_map；（C）关键帧 `!important` 无效声明（css-animations-1）经 `block_to_points` 丢弃——`border-color: red !important` 不再覆盖同块 `green`；（B2）动画施值级联重要度分层（css-cascade-4 §importance）——`ComputedStyle` 新字段 `important_won_props`（`CascadeOutcome` 组合缓存自 `cascade_with_orders` 既有 `orders` 旁表 `CascadeOrder.important` 派生，升序去重，零级联签名改动），施值门置 `AnimationClock::apply_to_computed_style` **单点**（渲染 stamping + js_anim/cssa gCS 咨询共享 `canonicalize` 三通道同覆盖；跳过即恒等返 None 回落级联），简写按写达 longhand 对位（`background`→`background-color` R3882 同源、`border-color`→四槽）；（D）applier 补 `border-color` 臂（四边 color 槽，先于本刀存在的静默丢弃缺口）。**验证**：reftest 探针差 vintage 行为锚——A′ bin ✗ F 2.08% / B bin ✓ **P 0.00%**（整面绿方无红，REFTEST_DUMP 对照）；**B↔A′ 五 corpus SET-IDENTICAL**（21907=21907，F2P/P2F/NEW/GONE 全零、页集合全等）＝回归修复零 corpus 扰动。engine lib **4,131/0F**（+6 钉）、style-system 2,550/0F + cascade_cache 5/0F。**A/B 设计注记**：g1 bin 不含 08:06 后进树 peer WIP——对称消去改法，先建 A′（同 WIP 态无本刀）再建 B（同 WIP+本刀 4 文件），差量=纯本刀；指纹法证 s419/刀H/刀F 标记串双 bin 同态（刀H 0/0→1/1 转正=typed-om WIP 对称入账实证），刀 I 无自有字面量以探针差替代。**碰头信号（A′↔g1 peer WIP 面定量化，非本刀账）**：typedom **+71**（F2P 74/P2F 3＝刀H typed-om WIP 靶面在账，归属流落地时认领）、masking **−59**（P2F 69）/shapes −2＝css-parser 括号 WIP 面现状净负——归属流落地前需自行 A/B 定谳；registry/shorthand rustfmt 折行=语义零差（arg 尾逗号语法糖）已对称入 B 账。本刀 4 文件提交（engine/animation.rs、style-system lib.rs/computed_style.rs/default_impl.rs）以显式 pathspec 限定，peer WIP 一概留树。详账与 provenance（A′ `3ddc6839`/B `fa4355e2`）见 [evidence 2026-11-17-knife-i-important-tier](evidence/2026-11-17-knife-i-important-tier.md)。
**刀 G clip-path 跨单位 calc blend 已落地（11-17，lerp_len px↔% 混合臂）**：`clip_interp.rs` `lerp_len` 新增两跨单位臂复用刀 E `blend_rule_inset_lp` 模型（逐单位系数 `(1-t)·from + t·to` → `calc(P% ± Xpx)`，px 系数零折叠纯 %，同单位守卫恰好兼容仍走既有线性臂）——css-values-4 §8.5.6 interpolation-calc，css-shapes-1 §basic-shape-interpolation 坐标/半径槽同文法，刀 E GONE 定谳第 4 条（clip-path-interpolation-002 跨单位 blend 缺口）靶面兑现；钉测 +1（inset 四边 mix + circle 半径/位置槽 + 零折叠纯 % + 关键词半径离散保持 + offset-path 同文法通道行为锁——`interpolate_offset_path` 复用 `interpolate_clip_path` 跨单位臂同步生效）。A/B（B=bin-cssa-g1 `c10cb92c` 对刀 F f1 基线，五腿 07:57:21→08:02:44 全 rc=0）：五 corpus **净 +5**（masking +5 全落 clip-path-interpolation-002 `inset(20px)→inset(40%)` 五档 t=0/0.3/0.6/−0.3/1.5，其余四 corpus SET-IDENTICAL）；**PAGE-ONLY 零容忍 F2P +5 / P2F 0 / NEW-GONE 全零 / 页集合全等**。engine lib **4,126/0F**（本轮续开 session 独立复验 16.59s）、clippy/fmt 本刀文件净（es_module transform_import 8 参 + css-parser dead_code 两 warning = peer WIP 连坐在册）。二进制指纹法证：s419 标记串 f1/g1 双 bin 同态＝抵消假设成立、刀 H 标记串（zw-typedom-listvalued）双 bin 0 处＝typedom 腿零污染。**树态碰头注记**：g1 构建后窗口三处非本刀在途变更留树未提交——css-parser 括号感知切分 WIP（靶面 inset-function-valid 三案 A/B 双侧 8P/3F 持位未动＝未进 bin 未验证）、js_dom_shim part10 刀 H typed-om WIP、examples/demo 删除集（无 evidence 归属）——待归属流自证或认领。上轮 87debb24 限流中断（08:12），本轮续开完成五腿账回填（flip 口径 Pass/Fail 大写定谳，首轮小写口径假全零陷阱复现）。详账与 provenance 见 [evidence 2026-11-17-knife-g-cross-unit-calc-blend](evidence/2026-11-17-knife-g-cross-unit-calc-blend.md)。
**刀 F·neutral 隐式边界帧通用化已落地（11-17，R5018 通用 gCS 串化臂 + compositor 双端点）**：`animation.rs` 新 `generic_underlying_value`——`underlying_individual_value` 逐属性手工臂（transform/clip-path/rule-width/rule-inset/offset/individual 三属性）miss 后一律落 `serialize_computed_property_with_sibling`（E5 同通道），css-animations-1 §keyframes neutral key = underlying value 的**全属性**兑现（空串未实现属性返 None 保既有回退；individual 三属性命中定制臂即返回语义保持）+ 共享路径语义翻转：compositor 导出 from-only opacity 缺口期冻结保守形 → 双端点 `[0.0,1.0]` 正常导出（`from_only_opacity_fills_underlying_and_exports` 新钉替代 rejected 旧钉，缺口注释清偿）。A/B 对双基线（gaps=v4 探针 3769、其余=v2 全账）：五 corpus **净 +47**（gaps +23 全落 rule-color/width 列表族 from-neutral 端点、multicol +4、shapes +20 neutral 面、typedom/masking 持位符合预期）；**PAGE-ONLY 零容忍 NEW-PASS +47 / GONE-PASS 0 / NEW-FAIL 0 / 页集合全等**，2 timeout（shape-outside-018 / mask-shorthand-reset）= A 侧同形持位 flake——即刀 E 揭穿的 neutral implicit boundary 缺口（GONE 定谳第 2 条 + 18 dn 面）兑现。与并行流认领的「刀 F」mask-border 批（9693736da 规划节，1,791F 支持门+插值批）撞名已做区分标注（本刀 = 刀 F·neutral 通用化，两刀靶面零交集）。engine lib **4,125/0F**（+2 钉 + 1 钉更新）、clippy/fmt 净；对账脚本 flip 口径修正记入 evidence（首跑漏 flip 账全零假象，corpus 行计数交叉核出）。详账与 provenance（B 064154a2）见 [evidence 2026-11-17-knife-f-neutral-boundary-batch](evidence/2026-11-17-knife-f-neutral-boundary-batch.md)。
**刀 E CSSA 声明式 gCS 咨询批已落地（11-17，cssa_registry 新档 + 六接线 + inherit/junction/calc/shim 四面）**：engine `js_dom_bridge/cssa_registry.rs`（transition_registry 模式镜像——thread_local AnimationClock + per-(World,identity) 条目，keyframes 经 note_stylesheets **三处喂入**：代际槽两分支 + **视图槽 compute_styles_for_doc 分支**（v2 主修复——corpus createElement 动态元素走 handle 通道，漏喂则 CSSA 残余 156F/页 got 底层值），喂入即清 ENTRIES 防 harness 同名换内容永不重锚，per-identity 首咨询锚 start 定格 progress=0.5）+ callbacks.rs sel hit/miss 两臂 + computed_style_cache handle 链咨询 + clear_generation_cache 同点清理；配套四面：junction 4 字段初值 0%（WPT oracle % 轴 lerp 自 0%）+「rule-inset」简写复位 per-slot 臂 + 8 longhand 显式 inherit 臂（css-cascade-4 §7.3）+ blend_rule_inset_lp 跨单位 calc 混合（css-values-4 §8.5.6，`calc(P% ± Xpx)`，px 系数零折叠纯 %；宽松臂+严格门双接入）+ shim `_zwUnsafe` 单位闸 rule-inset 例外（跨帧 px↔% 对注册全死根因）+ E5 keyframe 'initial'/'unset' 关键字经 gCS 串化解析（WAAPI/CSSA 两端共享，'inherit' 留 FIXME 池）。两轮 A/B（B0=knife-d 收口归档 46684957）：v1（代际槽两处喂入）五 corpus 净 +2115；v2（补视图槽）**净 +3244**（gaps +2488、multicol +61、shapes +258、masking +437、typedom 0 持位）；PAGE-ONLY 零容忍对账 **NEW-PASS +3254 / GONE-PASS 10 / NEW-FAIL 0 / 页集合全等**，10 GONE 逐条定谳全为「CSSA 真跑揭穿的假绿」非语义回归（rule-color repeat(auto) vacuous 持位＝刀 B3 外证同款、shape-margin neutral implicit boundary 缺口＝R5018 仅覆盖 transform 三属性、shape-image-threshold inherit 池、clip-path 跨单位 blend 缺口）——四缺口入刀后残面队列；rule-inset interpolation 两页 P488→624/F48/页；v3 interpolate_keyframes 末段选择修复（at(1) 撞末帧停 [last-1,last] 段）css-gaps SET-IDENTICAL 保留＋v4 transition echo 门 rule-inset 八槽例外定谳 TRANS at(1) 真根因（eased==1.0 短路在 blend 臂之前，改 interpolate_keyframes 零位移的机理闭环）→ 两页 **P632/F40**、css-gaps +16 up/0 dn/NEW-GONE 零、TRANS 面清零（残差 inherit-keyframe 40F/页单面）。engine lib **4,122/0F**（+7 钉）、style-system 2,550/0F、fmt/clippy 本刀域净（es_module peer 连坐在册）。systemd `--scope` 随 client 组回收腿死坑 service 模式根修、e1-full 脚本被并行流改造双跑碰撞采彼账停己腿零重跑。v2→v1 翻转 18 dn 逐条归因＝喂入面扩大后假绿揭穿（inherit 10 与 inherit-up 24 对冲＋neutral implicit 8＝R5018 通用化刀面）非语义回归。详账与 provenance（A 46684957/v1 5f978828/v2 b69a15ce/v3 01225d12/v4 90762cf3 + shim 指纹）见 [evidence 2026-11-17-knife-e-cssa-batch](evidence/2026-11-17-knife-e-cssa-batch.md)。
**刀 D rule-inset 全链批已落地（11-17，css-parser 新档 + style-system 8 longhand/15 shorthand + engine gCS/动画通道 + inline 站三回调五接线）**：css-gaps-1 §inset 槽语法单点 `parse_rule_inset_component`（`overlap-join | <length-percentage>`）+ 四槽/轴对/全值简写顶层解析；engine gCS `rule-inset` 跨轴等值模型（列/行各四槽重组后双轴全等 → 该值，否则空串——checkpoint 轮直通 column quad 的 REGRESSED=1 重写修复）+ transition/WAAPI 通道；**inline 站**（kd-inline-station）为 checkpoint 轮 A 侧 inset 页 0 PASS 定谳的根治——corpus 语料 `el.style[prop]=v` 全落 shim 字符串面、native 绑定展开代码 runner 从未执行，三回调（expand/reconstruct JSON 通道）宿主注册 + part03 T50/M face 五处接线（写面拆名单 longhand 简写键不占声明位、读面 raw 重组缺任一即空串 Chrome 口径、remove 家族一并清、回落零漂移；position-try 组 F inline 站同型先例）。v2 终态 A/B 对 head 基线（c775846d4 纯净二进制 46684957，B 侧不动）：**净 +1545 / 0 lost / 0 regressed / 28 增益页，页集合全等**（gaps +1474：parsing roundtrip 11 页 845 + bidirectional 家族 231 + computed 157 + animation 插值 2×240 + invalid 144；multicol +14；typedom/shapes/masking SET-IDENTICAL；checkpoint 轮 native 面 +841/1R 为中间账）。V8 探针 direct 回调三分法定谳两 bug：8 longhand 读重组 len match 缺 8 臂恒空串（全值形 10 REG，修复 quad 双轴重组 + part129 9 pin）、gCS 直通 column quad（target2 oracle）。engine lib **4,116/0F**（+3 钉 + 9 pin）、fmt/clippy 净（animation.rs duplicated `#[test]` + gap_rule_* 私有签名泄漏两自产 warning 修复；es_module peer 连坐在册）、bench-gate 定向 engine 1/1 PASS perf-gate rc=0、census 定向池 22 页 1377P/882F。systemd unit cwd 毒化 fail-all 形入账（pass=0 fail=788 五腿 41s merged JSON 照常落盘，单页探针双 cwd 对照定谳，脚本内 `cd $ROOT` 根修，毒 JSON rm 重跑）。详账与 provenance markers 法证见 [evidence 2026-11-17-knife-d-inset-batch](evidence/2026-11-17-knife-d-inset-batch.md)。
**gap-rules 刀 B 已落地（11-17 二十五轮，width/style 列表文法 + rule 简写家族 + csswg#12201 结构门）**：css-gaps-1 §column-row-rule-width/style `<line-width-list>`/`<line-style-list>` 三档同构段模型（Plain/Repeat{auto≤1,calc 计数折叠}，条目复用 multicol ColumnRuleWidth/StyleValue 保形）+ §rule-shorthand-properties `<gap-rule>` 联合简写文法（段内三维度 `||` 至多各一 + `*_raw` specified 原文保形——`hotpink` 不归一 rgb()）+ rule/row-rule/rule-width/rule-style 简写族全接线（style-system types/registry/shorthand `expand_gap_rule` 三 target 投影 + engine gCS 列表串化单段零漂移 + row 读臂 + `gap_rule_recomb_to_css` 三维 zip + paint 首段 + `<length [0,∞]>` 负 calc clamp 0）——rate-limit 多实例并发（四实例 resume 同 session）按认领板对称收敛：css-parser 两实例互补合流、engine 站停滞窗接手收口、animation 插值站 `interpolate_rule_width_value`（展开/广播/auto LCM/段数门 7 钉）同窗落地。两轮 A/B 权威账（PAGE-ONLY 零容忍，五 corpus 页集合全等）：B1 +259/−6 捕获六误配形回归——gCS 重组缺 csswg#12201「longhand 不对齐简写空串」结构门（B0 期 row-rule 未实现 vacuous 过、落地后平铺串泄出），修复 `gap_rule_lists_aligned`（段数/Plain-Repeat 同型/计数折叠口径/循环体长度全等）+ recomb 与 column_rule_shorthand_to_css 双臂失配空串 → B2 **净 +265/0 lost/0 flips/NEW-GONE 全零**（gaps +258、multicol +7、typed-om/shapes/masking SET-IDENTICAL；from-longhands 页 12 误配形全过、4 对齐形留 inline 站；B2−B1 delta +12 纯门修复归因无漂移）；单测 css-parser 3,293/0F、style-system 2,548/0F、engine lib 4,096/0F（含 animation/inline 双站在树新钉）、fmt/clippy 三 crate 净（唯一 warning=es_module peer WIP 在册连坐），归因细节与残面移交见
[evidence 2026-11-17-gap-rules-knife-b](evidence/2026-11-17-gap-rules-knife-b.md)。
**transition `all` 通配＋computed 值域＋WAAPI 校验批已落地（11-17，rate-limit 中断轮孤儿 WIP 认领收口，提交 `f4ca3f641`——rebase 前号 `f1ca328ca`）**：
engine 三文件一批（trans-all 展开 FIXME 清偿 48 项清单＋eased==1 rule-\* 四属性
例外回落插值臂；刀 C computed 串化 repeat 保形＋列表项/row 单值 px 取整、column
小数保形；WAAPI keyframes 非法值丢属性 hardening）。五 corpus A/B 对 head0
（`46c3df57a` 纯净二进制 `71851abb`，刀批＋es_module s419 WIP 全 stash 窗内构建）
**净 +474 / 0 lost / 0 regressed / 19 增益页，页集合全等**（gaps +198、multicol
+61、shapes +108、masking +107、typedom 0，增益面全落 trans-all 清单属性与
rule-\* 族）；head0 vs b3c 五 corpus SET-IDENTICAL＝editing ilb WSCANON 双修复
CSS 面零位移拆账。B3 残面 ~715F→348F（rule-\* 插值页族 11 页：trans 234→42
清偿 192＝trans-all 主增益、waapi 77→71、cssa 231 持位＝声明式 gCS 咨询独立
刀面）。engine lib 4,106/0F 八新钉、fmt/clippy 三 crate 净（es_module peer WIP
warning 在册连坐）、bench-gate 定向 engine 1/1 PASS perf-gate rc=0。刀 D inset
字段 scaffold（中断瞬间在编，不在批内）撤回存 patch
`.acceptance/wptfyi-css/wip-knife-d-inset-fields.patch`，完整刀需 parser/registry/
apply/gCS/paint 五站（inset 页池 102P/2157F）。systemd-run 发射 corpus 链新坑
入账：无 WorkingDirectory → wpt-data 相对路径全空 cases=N pass=0 形，必须
`cd $ROOT` 包进内层 bash -c。详账与 provenance markers 法证见
[evidence 2026-11-17-knife-c-transall-batch](evidence/2026-11-17-knife-c-transall-batch.md)。
**刀 B3 页级收账完成（11-17，twin 提交 `71b07c7d8` 五 corpus A/B 对 B2）**：
animation 插值器 + inline 简写展开 + apply 接线修复兑现——**净 +395 / 0 lost /
0 regressed**（gaps +305/−1、multicol +91、typed-om/shapes/masking
SET-IDENTICAL、五 corpus 页集合全等）；−1 定谳 WPT 页 expected 括号残缺
vacuous-pass 清算形（B2 假绿揭穿，非语义回归）——外证闭环（11-17，`652a04f66`）：
残缺串不进 animate() keyframes 通道（harness L210-224 只用 case 级 from/to），
chrome-160 276/276 全过而上游 typo 仍在 = Chrome 对 repeat(auto,…) 双通道
拒收双 vacuous（parser 缺口非 parity 目标），b3d 含终态 keyframes 校验仍
五 corpus 零翻转（修复惰性，−1 持位非语义回归）；双链 provenance 法证（b3 链
二进制 es_module WIP 扫入形 `e3a46906` 留参照、b3c 链钉定干净 `58ec871f` 单
写者权威，b3wip vs b3c 五 corpus SET-IDENTICAL = es_module peer WIP 对 CSS
corpus 零位移双证）；9 增益页残余 ~715F census 入残面队列（CSS Animations
keyframes 通道 ~302 最大、trans 简写+all ~241、WAAPI ~118），账目与机理见
[evidence B3 节](evidence/2026-11-17-gap-rules-knife-b.md)。
**rule-color 列表文法刀已落地（11-16 二十四轮，刀 A，gap-rules 设计稿首刀）**：css-gaps-1
§column-row-rule-color `<line-color-list>` 模型五站接线（css-parser parse_rule_color 新档
Plain/Repeat 段 + auto≤1 + calc 计数折叠；style-system column/row_rule_color 双
Vec<RuleColorSegment> + 白名单 +2 + VALIDATED 22→25 裁决臂序位 + expand_rule_color 简写；
twin amend 增补 rule-color 简写 apply 臂 +7；engine gCS 单段平色零漂移串化 + row/rule 读臂
+ paint 首段首色保形；验收债 shape-margin/threshold 复位臂两臂）——WIP 携入的 inset/position
paren-aware 切分按残面⑤未闭环定谳撤回（`970e19289`：masking clip-path-interpolation −74、
shapes 混合净 −4，合计 −78，b1v6/b1v7 回落链断裂先例第三次复现；拒收边界态恢复，clip_path
50 钉 rejection 全绿）。权威账链（final 单写者独占窗，b1v3 双写者污染目录废弃）：五套件
**净 +148 / 0 lost / 0 regressed，页集合全等**——gaps 75=75 +146（359→505）、multicol
91=91 +2、typed-om/shapes/masking 三护栏 SET-IDENTICAL。单测 css-parser 钉 4、
style-system 2,547/0F、engine lib 4,076/0F 全绿，fmt/clippy 三 crate 净；bench-gate 定向
（bench2，三 crate）microbench **3/3 PASS** + perf-gate **rc=0**（绝对预算过门，基线
硬件错配 WARN 在册）；make test 全腿收口：mkt5（净树 stash 窗）止腿于在册 SW
module_update（page-runtime:3782，纯 HEAD solo 复现定谳），mkt7（make -f 片段
全腿 no-fail-fast）workspace 腿净树互补面实证——WIP 在树则 module_update 绿而
es_module 6 钉红（连坐互换两面）；全腿红项全对号在册（es_module 6=WIP 连坐、
worker-fetch 双面×3 腿、tab 镜像+default_actions（browser-bin 741P/2F）、
focus-governance 六钉 stale-binary 环境形（browser-tests 0P/6F，零语义红）、
image_batch 环境形、
qjs-clippy 101=WIP 连坐），刀面三 crate 3,280/2,517/4,077 全 0F 零红，GPU 双腿
0/0（wgpu adapter 可用），详账见 evidence 门禁节。
残面：刀 B width-list ~684F、color 真
插值池 ~412F、刀 D inset-cap/junction ~1.7k、残面⑤ calc 代数插值（承重边界重撤硬前置），
见
[evidence 2026-11-16-rule-color-knife](evidence/2026-11-16-rule-color-knife.md)。
**shape-outside/circle-ellipse 位置文法刀已落地（11-16 二十二轮，rate-limit 中断轮 twin
孤儿 WIP 认领收口）**：css-values-4 position 轴别严格文法（PosComp 四类 + y 首词对换 +
4 值缘偏移百分比补数）+ engine 缺省分量串化（circle closest-side 省略/ellipse 双缺省
省略/空 inner 前导分隔符修/物理单位折 px）+ shape-outside inline 写面门接入（白名单第六
串 + 显式分派臂置于 unimplemented 放行臂之前——v1 b1v3 SET-IDENTICAL 定谳「白名单接入
≠兜底可达」序陷阱）+ round 参量文法四轮迭代（inset round 列表全形/polygon 前+尾缀臂/
fill-rule 相邻受理/polygon round 百分比契约拒收/inset calc 偏移拒收边界钉化——b1v6/b1v7
两轮实证括号感知破伪通过池后撤）。css-shapes corpus 净 **+1223 零新回归**（对 b0
+1224/−1：circle/ellipse-004 +327×2、circle/ellipse-003 +125×2、invalid 域八页 +65、
动画前缀真插值 +28、polygon-round-valid 清偿 +3、唯一残 −1 = inset calc 案挂残面⑤），
六代二进制 A/B 逐案对账全链、双 spawn 对称收敛法证与残面队列见
[evidence 2026-11-16-shapepos-position-grammar-knife](evidence/2026-11-16-shapepos-position-grammar-knife.md)。
**shape-margin/shape-image-threshold 五站小刀已落地（11-16 二十三轮）**：css-shapes-1
两属性全接线（ShapeMargin 类型站 + 非负 `<length-percentage>` apply 臂 + threshold
parse_opacity 语义复用 computed 钳 + 写面门白名单第七/八串 + decl canonical pct→number +
gCS/transition/animation 三插值器臂）+ supports/gCS 双向钉。corpus 净 **+188 零回归**
（css-shapes +187/0/0 十九页全 GAIN——interpolation +118、composition +15、parsing +16、
values +33、inheritance +2；masking SET-IDENTICAL、typed-om +1 stylepropertymap 转
真）——composition add 累加组（25F）与混单位端点对（margin 64F/threshold 58F）入残面
队列，见
[evidence 2026-11-16-shape-margin-threshold-knife](evidence/2026-11-16-shape-margin-threshold-knife.md)。
**基本形状插值 applier 臂刀已落地（11-16 二十一轮，twin 孤儿 WIP 认领收口）**：clip-path/
shape-outside 全 <basic-shape> 文法族经共享插值器进动画写回（clip_interp + AnimationClock
槽位臂 + echo 保留门伪通过池保位），七套件 corpus 净 **+519 零回归**（masking 六页
+8~+88、shapes 三页 +53~+71、motion 一页 +4，DOWNS 零触碰）——四轮迭代定谳链（v1
+503/0/29 四类定谳：keyword 半径 Fix 1 / edge-offset calc 归一层漏检 / inherit 伪通过池 /
path 失配）与残面三队列见下方二十一轮段与
[evidence 2026-11-16-basic-shape-applier-knife](evidence/2026-11-16-basic-shape-applier-knife.md)。
**Steps 臂修复验证收口（11-16 补测轮 b1v9，零码改）**：二十一轮被吸收入提交的共享 Steps
位置语义修复补测 **+222/0/2**（2 伪红=上游 2 页 at(1) 数据缺右括号定谳不回滚），残面②
关闭，见 [evidence 2026-11-16-steps-position-knife](evidence/2026-11-16-steps-position-knife.md)。
**path() 门臂回滚自证收口（11-16 二十一轮续，b1v10，提交 6615e43b9）**：steps-position-knife
挂账的 path 门撤除 WIP 认领自证——engine lib 4,074P/0F（+Steps 十钉）、corpus **vs b1v9
SET-IDENTICAL**（门臂语料中性，v6 中间账 ±8 按实测覆盖修正）、vs b1v4 基线 +741/0/2 两代
二进制复现零 flake，见
[evidence 2026-11-16-path-gate-removal-knife](evidence/2026-11-16-path-gate-removal-knife.md)。
**Transition 通道 gCS 同步读可见性刀已落地（11-16 二十轮，提交 3fbaaf26d）**：七套件
corpus 净 **+322 零回归**（filter-effects 八页 +17~+40、motion 七页 +4~+25，DOWNS
红线零触碰、echo 池全保护）——T 通道全死根因（同脚本轮次断言读恒先于泵首帧挂载）以
consult-only 两世界注册表清偿，三轮迭代定谳链与残面队列见下方二十轮段与
[evidence 2026-11-16-transition-channel-knife](evidence/2026-11-16-transition-channel-knife.md)。
11-14 发布
快照 **62,896 / 197,163（31.90%）**（coverage-2026-11-14，weekly cron
4214a625d），对验收线尚余 ~119,496——较 11-10 快照（52,488 / 26.70%）
四天净增 **+10,408**（[DC-3 刷新证据](evidence/2026-11-15-dc3-snapshot-refresh.md)）；
当前 css-images **3144P/71F**、css-values 4185P/1402F/6T（g49-b1）；
css-anchor-position anchor 首覆盖 12061P/792F/4T（265 页/12857 案）。
**`target` expando 刀已落地（11-15 六轮，提交 8ee02114e）**：part01
`_reflectedStringAttrOn` tag 门（真反射集 {a,area,form,link,base}）+
part05 set 守卫 / part04 get expando 回读优先（原始值 R3069 面不动）。
A/B（B0=results-texp-b0/B1=results-texp-b1 探针页剔除）：css-masking
126P→**615P 净 +489、0 页回归**（61/61 保序），21 崩页转真实子测、分母
与 chrome 逐页精确对齐（5,818 上游子测解锁）。见 [evidence
2026-11-15-target-expando-shim-knife](evidence/2026-11-15-target-expando-shim-knife.md)。
**逐组转化率实测完成（11-15 七轮）**：337 页 harness 面全量实测
（42 组、71 corpus invocation、同二进制零重建）——**335/337 解锁、
still-declared-0 清零**，解锁面 **6,691P / chrome 分母 55,861（12.0%）**，
相对刀前全崩净增 **+6,691P**（grid 33.9%/sizing 29.6%/anchor-position
57.4% 最高；motion/shapes/gaps/box 等 0% 组全卡 supports 门）。残差分桶
（50,731 fail）：**supports 覆盖门 49.5%**（shape() 5.5k / calc-size 1.9k /
mask-border 1.7k / offset-* 3.3k / backdrop-filter 943 / gaps 装饰 1.4k）、
**WAAPI 门 21.7%**（`'animate' in Element.prototype` 缺 Element.prototype
定义 + has 白名单臂，R2965 实现本身跑通已实证）、**插值语义 28.6%**。
**WAAPI 门刀已落地（11-15 八轮，提交 107bcd935）**：part03
`Element.prototype` 挂 animate/getAnimations（R351 `__zwSelector/__zwHandle`
反解 identity 委托 `_makeAnimation`）+ part05 has 白名单两名。探针门翻转
实证（proto-has/inst-has 双 true）；五组 A/B（B0=
results-texp-b1-{p1,p2}/B1=results-waapi-b1-{p1,p2}）**净 +184P 零回归**，
WAAPI 门子测 4071→**0**，2 页 25% 分母差应归一。提交前去冗余（part03
不可达残片块 + part05 同链重复条件），清理后探针复验一致。见 [evidence
2026-11-15-waapi-gate-knife](evidence/2026-11-15-waapi-gate-knife.md)。
**offset 簇刀已落地（11-15 十三轮，rate-limit 中断轮 WIP 认领收口）**：CSS
Motion Path offset-path/distance/rotate/anchor 四属性五站全链——style-system
类型/注册（OffsetRotateValue 四形 + known_properties 白名单 + spec 初始
none/0px/auto/auto）+ apply 文法（path() 首片、length-percentage 字号绝对化、
auto|reverse|<angle>|auto <angle> 归 deg、anchor 复用 parse_origin_xy）+
gCS 计算值四臂（缺臂即读缺省先例教训）+ WAAPI 插值链（underlying 四臂 +
插值四臂：同单位线性/混合单位离散 ≥0.5 翻转 css-values-4 §8.5.6 + apply
写回往返）。单测 4+3 钉全绿；**干净双二进制 A/B（B0c=HEAD stash 重建 /
B1c=WIP 重建，md5 异哈希 provenance）七 corpus（motion/transforms/values/
backgrounds/fonts + 白名单触面护栏 typed-om/cssom）净 +364 / 0 lost /
0 regressed / 17 页**——motion +363（interpolation 族 311 + parsing/
computed 25 + inheritance 4 + supports-calc 1）+ values random-computed
offset-distance % 槽红利 +1，四护栏 corpus 逐字节零差。**前轮 stale-B0
错账修正**：中断轮曾以 results-jsanim-b3（三轮前产物）为基线得「+47
transforms」——实为 composite-add/easing-flip 两刀已提交增益；对真基线
results-flip-b1 复比对 transforms/values/fonts/backgrounds 全零差，真增益
全在 motion 面。教训入册
[docs/learnings/bugs/2026-11/2026-11-15-corpus-ab-stale-baseline-results.md](../../../docs/learnings/bugs/2026-11/2026-11-15-corpus-ab-stale-baseline-results.md)。
残面（motion 45 页余 4,487F）：ray()/基本形状文法族 ~1,830（parse 首片
bail 后置）、offset-position 属性缺面 ~386（同型五站刀）、已解锁页内
插值中点/混合形残余 ~1,891（auto+angle 混合端点 458、path() 命令级兼容
插值 687、anchor/distance 混合单位等）、coord-box 144、composition
add/accumulate 140、math 函数角度 96。**门禁**（make test 拆腿，
zw-offset-mkt3.service）：12 腿全账红项全对号在册（peer s419 es_module 6 WIP
+ clippy 连坐、worker-fetch 三形态、tab 对 + default_actions、image_batch
负载形伪红），本刀域零红——engine 3 钉 workspace 腿日志绿 + style-system
4 钉定向腿 RC=0（fail-fast 截停补腿）。见 [evidence
2026-11-15-offset-motion-path-knife](evidence/2026-11-15-offset-motion-path-knife.md)。
**并行实例避让记（11-15 23:36 定谳，rate-limit 双 spawn 在册形态）**：本
clone 并存两 agent 实例（2614542 = 22:12 无 resume spawn；本实例 = 23:04
resume spawn）。offset 簇刀收口由本实例完成（6b84507fe）；offset-position
刀 23:31 起由实例 2614542 在树实施（style-system property 五站 + engine
animation/gCS offset 面，未提交），本实例零编辑让位（「停己采彼」）。
后续轮次开工前先 `git log --oneline -3` + `git status` 核该刀落地态：
已提交→按控制面推进下一残面刀（ray()/基本形状文法族 ~1,830 等属同文件
面，仍需先核对方在飞状态）；未提交且 mtime 活跃→继续让位勿重开刀；
mtime 静默 >30 分钟且无提交→查孤儿（journalctl + ps 祖先树）后按磁盘
终态仲裁。共享树纪律：两实例禁同时跑重型门禁（双 cargo 门禁缓存在册）。
**（11-16 00:15 更新）offset-position 刀已由实例 2614542 收口**（A/B
+108/0/0、门禁 12 腿零本刀域红，见下条目与当轮提交），避让记任务完成，
后续轮次按控制面推进残面刀。
**offset-position 刀已落地（11-15 十四轮，同日第二刀）**：`normal | auto |
<position>` 三形五站（OffsetPositionValue 枚举 + 白名单第五串 + spec 初始
normal + parse_offset_position_value 复用 parse_origin_xy + gCS 序列化臂 +
WAAPI underlying/插值/写回三臂）+ **parse_origin_xy 4 值 edge-offset 共享
扩展**（css-shapes-3 §position：`<edge> <offset>` 对，right/bottom →
Calc(100%−o)，transform-origin/offset-anchor 全消费方连带）。单测 5+7 钉
全绿。A/B（B0=62ae854f HEAD 重建 / B1=e2fe2a8d）motion+typed-om+cssom：
**净 +108 / 0 lost / 0 regressed / 5 页**——offset-position-interpolation
+68（312→244）、computed +13（15→2）、anchor-interpolation +24（4 词共享
连带）、anchor-computed +2、inheritance +1。首轮 B1 白名单漏串事故（initial
表已改、known_properties() 数组漏第五串，offset-position 四页零动）法证
补串重跑销账。残面（interpolation 余 244）：initial/unset/inherit 端点
组 108 = harness 全局关键字缺口（十三轮定谳跨 corpus 形）、CSS Animations/
Web Animations/transition:all 通道组 ~90 = 动画值进 gCS 链主矛盾面、
calc 代数插值 46（`left 480px`→`right -140%` 形）；composition add 56
（KeyframeEffect composite 共享刀）、calc 形 supports 2、offset 简写 3、
parsing-invalid 3（shim setProperty 拒收域）。见 [evidence
2026-11-15-offset-position-knife](evidence/2026-11-15-offset-position-knife.md)。
**ray()/基本形状 supports 门刀探查定谳（11-16 00:50，第三刀让位轮，零码改）**：
offset-path 六全灭页（005 502F/006 510F/008 614F/009 36F/coord-box 144F/
composition 84F，共 **1,890F**）死因定谳——全部死于 harness 前置门
`assert_true('from'/'to' value should be supported)`（interpolation-testcommon.js:396）：
`CSS.supports(prop, v)` 两参走 shim part06 → 宿主 `decl_supported`
（js_dom_bridge.rs:6171）→ Rust `apply_property_value_with_quirks`，而
offset-path 的 Rust parse 臂现只收 none|path() 首片，ray()/circle()/ellipse()/
inset()/polygon()/xywh()/rect()/shape() 全拒 → supports=false 整组 6 检查点
连坐（composition 页 underlying/add 值组同门实证）。**与 setProperty 通道
分叉实证**：parsing-valid 已 46P——setProperty 走 shim JS 验证器宽收，
circle(100px)/inset(100%)/ellipse()/polygon()/xywh()/rect()/url() 标准形全过；
supports 门与 parse 面不同源即刀核心。刀界（下轮实施）：`parse_offset_path_fn`
放宽全基本形状族首片接受 + gCS verbatim 臂 + WAAPI 离散翻转（现臂通用，无需
新增）；理论增益估 ~400-500P（端点检查点 2/6 × 六页 1,890 × 非全灭通道
~3/4，transition:all 通道组可能仍死；精确账以刀后 A/B 为准）。残面修正
两笔：①parsing-valid 余 24F = shim JS 验证器边角（rad 单位、`ray(0deg at
center center sides)` 乱序、无单位 0、`round 0px,` 逗号形、path `s` 命令、
`path(  "…"  )` 空白归一）独立通道挂账；②parsing-invalid 24F 全账红（P=0）
——setProperty 通道原样存 invalid 值读回非空所致（非本刀域）。001 页 179F
复核：60F got "none"（transition:all 通道丢值）、12F 小写 `m` path 输出
（疑 jsanim registry 侧另一插值臂，下轮探针）、余为 path 命令级插值期望
（后置域）。**让位记**：peer backdrop-filter 刀在飞（zw-bdf-ab.service
cssom 批 + editing-run 双链；registry.rs/es_module.rs/part02.rs WIP mtime
00:15），在跑链禁发重型入口，本刀不开工，下轮开工前先核该刀落地态。
**backdrop-filter 白名单刀已落地（11-16 十五轮，本流收口 peer 让位面）**：
一行门线刀——style-system `known_properties()` 白名单 `"filter"` 后补
`"backdrop-filter"`（2 文件 +12/−0：registry.rs 一行 + engine part02 R2785
supports 钉扩双向断言），机制 = supports 前置门 `decl_supported` 缺串短路假
（apply/gCS/WAAPI 插值/写回/初始五站 R2762 起全在，唯一缺口门线）。A/B
（B0=bin-bdf-b0 e2fe2a8d=posp-b1 快照 ≡ 落地 HEAD / B1=bin-bdf-b1 37eb8df2
WIP 重建）三 corpus：filter-effects **+253/0/0/6 页**（interpolation-001..004
+218、parsing/computed +25、composition-001 +10），css-typed-om/cssom
护栏 0/0/0 白名单纯增量零外溢。定向钉 1 passed（test-guard 包裹）；
`cargo fmt -p zero-style-system -p zero-engine -- --check` 净（27 处
part02 假漂移 = 裸 rustfmt 配置解析差异，cargo fmt 权威门作准）；clippy 两
crate 净（唯一 warning = peer es_module too_many_arguments 在册连坐）。
make test 拆腿门（zw-bdf-mkt.service）全腿归账在册红：workspace 10097P/1F
（same_origin_image_burst 负载 flake，R5288/R5292/R5385 同族 ≥4 先例，
peer editing-run 电池并发窗；**discharge 定谳** zw-bdf-discharge 00:50 solo
1P/0.61s，journalctl 证）、renderer-lib 371P/1F（worker-fetch renderer
形）、browser bin/tests 各 2F（tab 镜像 + default_actions 形）、
quickjs-clippy 101（peer es_module lint 在册）、gpu 双腿 0、其余腿见
[evidence 2026-11-16-backdrop-filter-whitelist-knife](evidence/2026-11-16-backdrop-filter-whitelist-knife.md)。
**同文件碰撞法证澄清（crule 提交时更正）**：01:14 crule 刀流
（zw-crule-ab.service 在飞）同两文件在途改写（registry 入 column-rule
行；**grep 法证：registry.rs:690 backdrop-filter 行在**——f80e4a8b8 已入
HEAD、crule diff +1 仅 661 行、插入点远离 689 区，part02 backdrop 断言
原样，「行被抹」系 stash 瞬态误读，复录可免；gap-rules 十六轮 45b4209b5
已账本侧互证本刀 A/B +253/0/0 + discharge 代跑）——按停己采彼/认领件
让位协议不动文件；crule 提交含本刀行/钉原样（HEAD 内容），本刀无需
独立复录 commit，按认领件核对后并入账。
残面：composition-001 余 ~90F = shim composite-add 函数列表 all-or-nothing
主矛盾面（十轮已挂账）、散页 ~690 在他 corpus 全量跑自然入账；同型 supports
桶定谳 mask-border/shape-outside/calc-size/row-rule 均真实现缺口后置。
**gap-rules 刀侦察+设计轮（11-16 十六轮，零码改——共享树编译破树风险让位）**：
本实例（监控线）全程让位 peer 三刀（offset-position 4427b121e +108/0/0、
backdrop 白名单 f80e4a8b8 +253/0/0、ray()/基本形状在飞 css-parser 四文件
+parse_basic_shape 新档），收口三笔：①knife-1 门禁归因收口（zw-offset-mkt3
12 腿红项全对号在册零本刀域红 + style-system/engine 定向钉补腿 5/5+3/3）；
②落地态核验三刀 commit pathspec 纪律（es_module 恒不入）+ A/B 复算互证
（posp +108、bdf +253 护栏 0/0/0）；③**css-gaps 面基线入账 + 下一刀设计定谳**
——results-gaps-b0（css-gaps P356/F3792 75 页 + css-multicol P237/F1376 91 页，
bin-posp-b1 ≡ 4427b121e）；**勘误上条「row-rule 真实现缺口」定性**：缺口为
css-gaps-1 把 rule 族 color/width **重定义为 repeat() 列表文法**（
`repeat(4, blue)`/`red, repeat(3, blue)`——gap-decorations-color-valid 页
实证，纯色=单段列表 multicol 兼容），非纯值 mirror 可达；两通道探查已完成
（setProperty shim 宽收 ✓ / gCS camelCase 通用路由 ✓ / 死因=supports 门
Rust parse 拒列表）。分刀 A color-list 模型（+600~900F）/ B width-list+简写
（+400~600F）/ C `column-rule` 简写白名单一行（multicol 全实现只差门线，
可独立先行）/ D inset-cap/junction 几何族（1.5k+ 最大残面，单长度插值
页内实证）。全案 [evidence 2026-11-16-gap-rules-knife-design](evidence/2026-11-16-gap-rules-knife-design.md)。
**column-rule 简写门线刀已落地（11-16 十七轮，刀 C 独立先行认领收口）**：
交接稿实况修正——「只差白名单一行 + 钉」不完整，白名单行中间态
（B1' 0e3d7ab6）对 b0 零 delta 实证，真缺口 = **apply 链无简写展开臂**
（展开在 cascade 层，apply.rs:2192 注记；`decl_supported` = 白名单 +
apply oracle 两 conjunct，简写后者恒假）。刀体 3 文件 +42/−0：registry
白名单 `"column-rule"`（column-rule-style 后）+ apply_advanced 新简写臂
（复用 cascade 同一展开器 shorthand::expand_shorthands→expand_column_rule，
`||` 文法/重复组件裁决单点——@supports 样式表路径 matcher/supports.rs
同判据；longhand 逐条递归 apply 全过才落 true，展开空即 false 非法值仍
拒）+ part02 R2785 钉扩 column-rule 五向断言（合法三形真/bogus/重复
color 假）。定向钉 1 passed；fmt/clippy 两 crate 净。A/B（B0=bin-crule-b0
c481ca1b ≡ HEAD 45b4209b5 / B1=bin-crule-b1 23ee88d5）：css-multicol
**+6/0/0**（column-rule-computed.html 6 子测翻转）+ css-gaps **+3/0/0**
（gap-decorations-rule-shorthand-computed.html column-rule 3 翻转，row-rule
页同步解锁——纯色单段列表兼容预判兑现）+ css-typed-om 护栏 0/0/0，
same_pass 与设计轮基线逐数吻合（237P/91 页、356P/75 页）。残面分解
（multicol 命名 F 347 全非本刀域）：插值三页 320F 语义桶、shorthand.html
12F = shim 声明对象 longhand 投影面、invalid 10F = shim 宽收漏拒、
valid 2F = canonical 序列化、width-computed 2F = calc() 门。**同文件
碰撞法证澄清**：bdf 账本「registry 本刀行被抹」不成立——registry.rs:690
backdrop-filter 行在（f80e4a8b8 已入 HEAD，本刀 +1 仅 661 行 column-rule）、
part02 backdrop 断言原样，「被抹」疑 stash 瞬态误读；复录可免，其「若
crule 提交已含本刀行/钉则并入账」分支适用。peer basic-shape/es_module
WIP 本轮持于 stash（crule-peer-window2）至门禁收口。门禁 zw-crule-mkt3
拆腿（01:34–01:51）刀域零新增红：companion/clippy 0、gpu 双绿，红项 9
全对号在册（workspace 14150P/1F=SW module_update、renderer-lib 371P/1F、
browser bin/tests 各 741P/2F、qjs-workspace 775P/1F=image_batch、
qjs-renderer-lib 369P/1F=worker-fetch 三形），全案
[evidence 2026-11-16-column-rule-shorthand-knife](evidence/2026-11-16-column-rule-shorthand-knife.md)。
**基本形状文法族刀已落地（11-16 十八轮，rate-limit 双 spawn 对称收敛收口）**：
shape()/path()/rect()/xywh() 共享文法五站全链——①css-parser
`parse_basic_shape.rs` 新档（shape 命令式 move/line/hline/vline/curve/smooth/
arc/close，to|by 双向，`with` 控制点，`of` 修饰序不敏感 + arc canonical
cw→large→rotate 缺省省略 + 裸数字补 px + 1 词位置补 2 分量；path 双引号
SVG data；rect 空白 4 值 + round；xywh slash 圆角；types.rs ClipPathValue
四变体 Box 载荷）+ parse_clip_path 四臂派发 + `serialize_math.rs`
fold_decl_specified clip-path canonical 臂（**物理单位守卫**
has_physical_length_unit：pt/pc/cm/mm/in/q 命中回落原文通道——types.rs
pt→px 折算使声明级 canonical 无法保原单位形，b0 verbatim 假过 1 例经守卫
复原）；②style-system offset-path apply 文法门放宽（ray() 全形 canonical
序 + 基本形状全族经 parse_clip_path 裁决 + coord-box 五后缀剥离 +
`none` 显式拒收）；③engine gCS（clip-path 四臂：shape 走 computed 级
`shape_to_css_computed`——关键字折 %、rotate 归 deg；path/rect/xywh 声明级
verbatim）；④engine transition get 臂（clip-path canonical 比较 +
offset-path none 回落）+ WAAPI `clip_interp.rs` 新档（clip-path 同构逐
分量插值 + offset-path ray/shape/path 逐命令插值 + 异构离散 ≥0.5 翻转）；
⑤paint border_shape 四变体 FIXME 空顶点。**resume-swarm 对称收敛**：本
clone 双实例（rate-limit 多重 spawn 在册形态）对 computed_style Shape 臂
同向接线，按磁盘终态采认合并；A/B 权威批由 twin 发射（b1v3），本实例
挂看门采认不重跑——一账两用。**A/B 权威账（B0=bin-shape-b0 6f551c72
HEAD 88b6ab519 / B1v3=bin-shape-b1v3 9c06ccb6 HEAD+全 WIP 终态，
zw-shape-b1v3-ab.service 04:51:41–04:58:15 四腿 EXIT=0 journal scope
记账 CPU 6m28s，transcripts 在盘）**：css-shapes **+54/0/0**（145 页，
computed +35/valid +17/rect +2）、css-masking **+327/0/0**（61 页，
clip-path 插值三页 +320）、motion **+558 净/UPS 570/DOWNS 12**（45 页
576→1,134，006 +203/008 +173/005 +104/coord-box +34/007 +25）、
css-typed-om 护栏 **0/0/0**——**合计净 +939/UPS 951/DOWNS 12/GONE 0/
NEW 0**。**12 DOWNS 定谳（伪影假说证伪）**：v2 批曾按 mtime 法证归因
「旧二进制伪影」（04:11:37 构建早于 04:17 修复）预期 v3 消除，实测 v3
含全部修复 DOWNS 恒 12（007 neutral↔none 端点对）——**真实残面**：
插值函数级探针 ✓ 但 shim/engine 对 neutral keyframe 页构造
（transitions:all 声明注入、动画值应用点）未建模，活体插桩挂下轮；
收口行红分类转抄前独立核机制条款直接兑现。**存量红三修顺带清偿**：
registry/inherit 补 column-rule 简写与 backdrop-filter initial_value 臂
（HEAD 白名单已解锁而 initial_value 表无臂的验收债两条，style-system
2544→2546 全绿）。门禁：css-parser 3,259P/0F（25 钉）、style-system
2,546P/0F、engine 4,057P/0F（插值钉 5）、clippy 三 crate 净、fmt 带配置
十五文件净；make test 拆腿（zw-shape-mkt.service 04:53–05:11 GATE-DONE）
红项六腿全对号预存（workspace 10923P/2F image_batch 双形、renderer-lib
371P/1F + qjs-renderer-lib 369P/1F worker-fetch 三形、browser bin/tests
各 2F tab_js_worker+default_actions、qjs-workspace 825P/1F image_batch
形），**刀域零新红**；bench-gate 定向全绿（zw-shape-bench2/3：css_bench 1/1→三 crate 3/3 微基准 + 绝对预算 PASS，硬件基线不匹配相对门禁 WARN 在册，报告两份入 tests/benchmarks/results/）。残面（下刀
队列）：shape-outside 全谱实测 **5,105F**/104 页（supports 门 1,955 +
串化 canonical 3,117——`circle(at 50cm)` 期望补 center/edge 序/calc 折算
+ assert_true 前置 33；设计稿
`.acceptance/wptfyi-css/next-knife-shape-outside-design.md`，registry 三串
门线 + 存储槽 + apply + gCS + WAAPI 五站，旧估 ~2,184 只数了门桶偏小）、
007 页 neutral 页构造活体插桩（**并发实例已在飞认领**，ZW_ANIM_DEBUG
插桩层 05:19）、rect/xywh
computed 折算、path() 段级 canonical（绝对化+空格归一）、shape() 命令级
兼容插值、<position> 3 词混合形、ray() && 乱序 supports 形。全案
[evidence 2026-11-16-basic-shape-grammar-knife](evidence/2026-11-16-basic-shape-grammar-knife.md)。
**Transition 通道可见性刀设计+认领（11-16 05:4x，轮换续跑实例）**：§7 追加定谳
（T 通道 corpus 级不可见）的直接后续——六套件 T 面 census **1,752P(echo 巧合池
全保护红线)/7,846F**（masking 2,416/motion 1,736/gaps 1,218/shapes 1,190/
filter 638/multicol 648）；F 键期望分类：expect-from t<0.5 **1,436** + expect-to
t≥0.5 **2,080** ≈ **3,522 直取池（可见性+discrete flip 零新插值器，jsanim flip 臂
先例）** + 真插值池 3,706（offset-path 405/clip-path 363/shape-outside 351/filter
154/backdrop 123 头部全属十八轮+刀 A 已有插值器属性复用）。刀体三站镜像
jsanim v3 已证模式（start_transitions before-change 真值捕获+TRANSITIONS
注册表+gCS consult）+ Phase 2 seek 语义 + Phase 3 transition:all 展开；
全案与开工条件（刀 A animation.rs/transition.rs/computed_style.rs 窗口收口后
零码改解除）见 `.acceptance/wptfyi-css/next-knife-transition-channel-design.md`。
**T 面 census 扩展（11-16 06:0x，同实例，刀 A 窗口等待期只读账本）**：flip-b1
快照十三套件补账（cadd-b1-comp 九套件实测零 T 键=composition 面，shapeout-b1
twin 在飞写面未触碰）——**T 键 10,898/T-P 2,928/T-F 7,970**（backgrounds
1,752F/transforms 1,660F/grid 652F/values 1,215F/fonts 912F 头部）；全谱合并
口径：**echo 红线 4,680P、T-F 15,816、Phase 1 直取池 ≈6,516**（直取 5,635+
钳制臂 881）、真插值池 8,396、CSS Animations seek 臂面 4,276F 首获定量
（transform 848/width 570/grid-template 652 等直取大头换位 backgrounds+
transforms）。逐套件明细与修正三点见
`.acceptance/wptfyi-css/notes-transition-census-extension.md`。
**T 通道根因细化+harness 语义解码（11-16 06:1x，同实例）**：只读侦察定谳
§7「零 transition tick」真根因=**同步读窗口**——机械全在（TransitionClock
生命周期/pipeline tick_animation_styles 4b+5b 接线/webview R342 before-change
两步基线），但 corpus T 测试同一脚本轮次内 setProperty→gCS 读断言，无泵帧
介入，基线 seed 后 start 要等下一帧 tick，断言读恒先发生。harness 语义：
transitionDelay=-50s/duration=100s 钉死 progress=0.5，at 编码进 timing
（createEasing：0→steps(1,end)/1→steps(1,start)/0.5→linear/其余
cubic-bezier(0,(8y−1)/6,1,·) 对称点 y(0.5)=at 精确）；engine
apply_timing_function 四形态全支持且对称 bezier 首采样即命中，at∉[0,1]
外推天然支持——**零新 timing 工作量，Phase 1=同步启动+gCS consult 两站**
（详图 `.acceptance/wptfyi-css/next-knife-transition-channel-design.md` §5）。
**基本形状插值 applier 臂刀已落地（11-16 二十一轮，rate-limit 中断轮续跑 + twin 孤儿 WIP
磁盘终态仲裁认领收口）**：clip_interp 共享插值器（none 恒等/Inset 四边+round/Circle/
Ellipse/Polygon 逐点/Shape 命令级/rect()/xywh() 分量/Path 段数门）+ apply_single_property
clip-path（parse_clip_path→style.clip_path 槽）与 shape-outside（None/Shape/裸 shape-box
Box 三形）两臂 + 宽松/strict 双通道同对臂。**四轮迭代定谳链**：v1 +503/0/29 四类——①18
keyword 半径对（002 页 T 通道族，同关键字误判恒等可插值）→ lerp_radius 任一端关键字整对
None Fix 1（css-shapes-1 §interpolation 关键字半径不参与插值）；②6 edge-offset 缘偏移对
（001×3+shapeout×3）——calc 化在 shape_axis_comp_resolved 归一层产出 `calc(100% - off)`、
原文无 "calc(" 子串（v2 引擎测试 1F 定谳文本检查结构性漏检）→ 结构化 clip_value_has_calc
逐槽位遍历；③4 inherit→rect/xywh 期望 calc 化 inset 不可达（setProperty 写面拒收→
expected 落静态值==刀前 echo，B0 伪通过双巧合）；④1 path 段数失配 at(1)（jsanim flip
时序面 shared Steps(Start) 位）→ **echo 保留门** `clip_pair_echo_preserve` 四臂（calc/
rect 族跨形/inherit∧rect/path 双端）命中返空串 apply 必败逐位还原刀前行为（伪通过池
保位不追真值，jsanim 十轮同款纪律）——v3 宽 css_wide 臂 b1v7 +499/0/0 过宽压 20 真离散
增益（initial/unset underlying 恒 none、期望端点裸形，逐子测差集 20 案全 initial/unset
定谳）→ v4 收窄仅 inherit，**b1v8 权威账 gained=519 lost=0 regressed=0 pages_gain=10
pages_loss=0**（B0=bin-trans-b1v4 md5 639f7bd1 码态采认/B1=bin-trans-b1v8 md5 afd7f247，
md5 链 b1v5→b1v6 7692f85a 带 bug→b1v7 319523b2 宽臂→b1v8 权威全可溯）：masking 001
+67/002 +83/shape-control-points +57/shape +88/xywh-rect +32/discrete +8，shapes
shapeout-interp +71/path +53/shape-shape +56，motion offset-path-009 +4，七套件其余页
零差（typed-om/gaps/multicol/filter-effects 护栏面零差）。门禁：engine lib 4,073P/0F ×2
（v3/v4 链独立复跑，4071+2 新测试含三保位形+三不进门反例钉）+ webview lib 852P/2F 双 F
逐名对号在册环境形红（invalid_host 代理形 env -u 五变量清空复绿实证 + csp_gate s348
env -u 仍挂）零新增 + clippy/fmt 净。残面三队列：①calc 代数插值共享面（edge-offset
位置/inset calc 分量 lerp_len Calc↔Calc + rect 族跨形→inset 归一整体插值 + setProperty
calc-inset 写面，整刀候选届时逐一拆保位钉）②jsanim flip Steps(Start) 共享位（独立刀
验证）③path() 命令级兼容插值。全案
[evidence 2026-11-16-basic-shape-applier-knife](evidence/2026-11-16-basic-shape-applier-knife.md)。
**shape-outside/circle-ellipse 位置文法刀（11-16 二十二轮，shapepos-knife）**：中断轮
（rate-limit 429 15:28 静默）孤儿 WIP 磁盘终态认领（mtime 取证 15:10-15:25 六文件）+
rate-limit 多重 spawn 对称收敛（同 session 5×resume，本流白名单条目 + 收敛实例显式裁决
臂互补合树，链账采 zw-shapepos-final.service 磁盘终态）。四站：①css-parser
`parse_position_pair` 严格文法（PosComp 枚举逐 token 分类，`[lp|x-edge] [lp|y-edge|center]`
+y 首词对换+center 补位；`50% left`/`top 50%`/`left right` 混轴同轴双关键字拒；现代
css-values-4 缘偏移形属语料 invalidPositions 域不收，FIXME(规格演进) 挂账）+ 4 值缘偏移
百分比补数（`bottom 70% right 80%`→`20% 30%` 代数折算，长度偏移保 Calc）；②engine
clip_path_to_css 缺省分量串化（circle closest-side 省略/ellipse 双缺省整体省略/空 inner
`circle( at …)` 前导分隔符修/circle-004 computed 533F 族单一根因；`50cm`→`1889.764px`
物理单位折 px）；③style-system shape-outside 写面门（白名单 19→20 + 显式分派臂置于
`value_has_unimplemented_function` 放行臂**之前**——v1 白名单条目末尾兜底臂被提前放行
结构性绕过，b1v3 SET-IDENTICAL 定谳「白名单接入≠兜底可达」，探针三段定位法证：裁决
`circle(at 50% left)` 返 true ✗）；④round 参量文法四轮：parse_clip_round_list（inset
round 列表全形 `<r>{1,4}[/ <r>{1,4}]?`）/polygon 尾缀+前缀臂（`find_top_level_comma`
calc 逗号安全+fill-rule 无逗号相邻受理）/polygon round 百分比契约拒收（Chrome round-
invalid 域 `polygon(round 1%,…)`，inset 仍依 border-radius 文法收）/inset calc 化偏移
拒收边界钉化。**A/B 六代二进制账**（b0 md5 d364bc8d→b1v8 md5 84815f65，全腿
LEG-EXIT=0）：B0→b1v2 位置文法刀 +856/−856 零 flip（circle/ellipse-004 +327×2 串化主
红利）；b1v2→b1v4 显式臂 +324/−232（round 文法缺口暴露：inset round 列表/polygon 尾缀
被拒 → b0 裸回显巧合通过的真 valid 页翻红）；b1v5 A-lite 一轮 → 残 −4（+1195/−4 对
b0）；b1v6 A-lite 二轮 → 净 +29 但揭两面回归（无条件括号感知 + 前缀臂同批）；b1v7
三轮条件切分试探 4537P 但动画 t=1 残 −2（条件边界把无 round 的 calc 化期望串划进接受
面，与 +4 inset 增益 parse 层同构不可分）；**b1v8 终态（朴素切分+前缀臂+% 拒收）
4535P，对 b0 逐案 +1224/−1 净 +1223**，与发射前预估逐位吻合，动画页 +28P/−0F 伪通过
池全保护（二十一轮「期望串含 calc 形不追真值保位」纪律两轮实证后回归）。门禁：MKT 拆腿
MKT-RC=0（fmt 六文件 5/6 rc=0 part03.rs 假漂移 edition-flag、workspace/quickjs-clippy
101 全对号在册 peer s419+worker-fetch 族零刀域新增）+ 终态钉测 css-parser 50/50（位置九
钉+round 四钉+invalid 十钉+calc 边界钉）/style-system 1/1/engine 2/2/fmt/clippy 净。
残面：round 列表完整建模（串化丢零半径/丢 fill-rule 形+插值回落）+ inset calc 边界四案
解锁队列（残面⑤ calc 代数插值刀联动）+ 位置串化规范刀（~498F：x-then-y 序/缺轴 center
补位/符号归一）。全案
[evidence 2026-11-16-shapepos-position-grammar-knife](evidence/2026-11-16-shapepos-position-grammar-knife.md)。
**Steps 位置语义共享臂修复验证收口（11-16 二十一轮补测轮，b1v9，零码改）**：58b3721ad 内
被吸收的共享 Steps 臂修复（提交时未实测——b1v8 二进制 14:01:50 构建早于修复 14:06:39
写入）补测收口：provenance 纯 HEAD 码态（CHAIN-START 14:13:56 仅 peer es_module WIP 在
册，工作树 animation.rs path 门撤除 WIP mtime 14:18:00 晚于 CHAIN-DONE 未进二进制），
engine lib 4,073P/0F、runner md5 447334b9、七套件 corpus **vs b1v8 净 +222/0/2、vs b1v4
基线 +741/0/2**（29 页 at(1) 编码族 echo 失败转通过，残面② jsanim flip Steps(Start)
共享位关闭）。2 REG 逐层定谳**改进暴露型伪红不回滚**：上游 WPT 数据 at(1) expect 缺右
括号恰 2 页（clip-path-interpolation-shape.html:64 / offset-path-interpolation-008.html:54）
→ setProperty 拒收 → expected 回落 `.target` 类规则值（interpolation-testcommon.js:385-387
往返 + createTargetContainer 模板克隆）；b1v8 伪通过=旧 floor eased 0 的 from 侧回落值恰同
污染 expected 双巧合，b1v9 got=to 端点（与测试名声明期望逐字吻合）反暴露假期望——合规引擎
同型恒红。新挂账：inline `clip-path: inherit` 疑未走继承链（from 侧输出=.target 类规则值
非 .parent 继承值，b1v8/b1v9 transcript 同证；修通后 inherit→shape 对 4/5 子测可真插值，
归 inline-style 全局关键字解析刀）；工作树 animation.rs path 门撤除 WIP（twin 在飞认领面）
本轮未触碰待其自证 A/B。全案
[evidence 2026-11-16-steps-position-knife](evidence/2026-11-16-steps-position-knife.md)。
**path() 门臂回滚自证收口（11-16 二十一轮续轮，b1v10，提交 6615e43b9）**：上段挂账的
path 门撤除 WIP（twin 流 14:18 写入）由本流认领验证：clip_pair_echo_preserve 撤
Path/Path 门臂回归离散翻转 + test_timing_function_steps_position 十钉（css-easing-1
§step-easing-functions 逐形态）+ gate 测试期望翻转；门禁三绿（fmt rc=0；默认特性 clippy
animation.rs 零命中——全图唯一错=peer s419 es_module WIP too_many_arguments 归因挂账，
落地后补全量；engine lib **4,074P/0F**）。b1v10 链（twin 流 14:39:56 发射，本流采认等待
停己链；runner md5 3ab4488a）七腿 rc=0 ×7：**vs b1v9 SET-IDENTICAL**（逐子测恒等——
path 门臂在七套件语料零二进制级影响，v6 中间账 ±8 未在控制变量双二进制对账复现，按实测
覆盖修正；撤除以语义理据保留——该族 corpus 不可见的机制：门的 echo 与旧
Steps bug 的 from 侧回落对该族产出同形 FAIL，37 path 对增益真根因是 Steps
修复解锁的 at(1) 翻转，非门臂；vs b1v8 双刀合计 +222（187 Steps 非 path +
37 path 对 + 2 伪红恒定））、**vs b1v4 基线 +741/0/2 第二次独立复现**（447334b9 与
3ab4488a 两代二进制同账，零 flake；2 REG 伪红定谳沿用；b1v9 纯度复核维持，
md5 差异=门臂在场 vs 缺席非混态）。残面队列不变（calc 代数插值共享面整刀
候选居首 + inline inherit 解析刀沿 steps-position-knife §5.1 + path() 命令级
兼容插值③沿队列）。全案
[evidence 2026-11-16-path-gate-removal-knife](evidence/2026-11-16-path-gate-removal-knife.md)。
**Transition 通道 gCS 同步读可见性刀已落地（11-16 二十轮，提交 3fbaaf26d）**：十八轮 §7 定谳
的 T 通道全死根因清偿（corpus interpolation-testcommon T 测试同一脚本轮次
setProperty(from)→gCS flush→写 transition-*+setProperty(to)→gCS 断言读，无泵帧介入
→start_transitions 挂泵首帧→断言读恒先发生→gCS 恒落 cascaded 终值 echo）——Phase 1
consult-only 两世界注册表（零 webview/pipeline 改动，jsanim 十轮刀已证模式镜像）：engine
新增 transition_registry.rs（World::{Gen,View} 条目表 + per-identity LAST_STYLE
before-change style 快照 + 登记五门含严格插值对探针 + query_core 咨询）+ callbacks.rs
Gen 三站（前值捕获/登记/cache-hit 与 miss 双 consult 位）+ computed_style_cache.rs
View 三站（corpus T 元素 querySelector('.target')||动态容器→handle→View 世界主战场探针
实证）+ js_anim_registry canonicalize pub(super) 提级复用 + animation.rs
interpolate_property_value_strict（~180 行，非插值对登记期即拒——echo 池结构性零扰动）。
**三轮迭代定谳链**：v1 零差根因=文档级槽 diff 被 harness 两阶段结构（全部 target 先写后
读）穿插写毁前值→per-identity LAST_STYLE 修（v2）；v2 at(1) REG 根因=共享
apply_timing_function Steps 臂未吃 StepPosition（steps(1,start)@t=0.5 误落 0）→注册表
本地 entry_eased（css-easing-1 §step-easing-functions，共享函数不动独立刀验证）；v3 残留
26 REG+8 DOWNS 两类定谳=at(1) 端点序列化形差（eased==1.0 恒等 to 回落 echo）+at(1.5)
[0,1] 四族量钳制（css-filters-1 §filter-functions：grayscale/sepia/invert/opacity 钳，
brightness/contrast/saturate/blur/hue-rotate/drop-shadow 无界不钳）——**整体 eased≥1→None
方案经全账扫描否决（63 例无界族 at>1 外推真增益全翻回）**。**v4 权威账 gained=322
lost=0 regressed=0，pages_gain=15 pages_loss=0**（B0=bin-shapeout-b1v4 md5 70f90c73
码态采认/B1v4=bin-trans-b1v4 md5 639f7bd1 七腿全量重跑 LEG-EXIT=0 ×7）：filter-effects
八页 +17~+40（backdrop-filter-001 70→110、filter-003 107→143）、motion 七页 +4~+25
（offset-distance 32→57），DOWNS 红线零触碰、echo 池 1,752P 全保护、offset-path 三页
v3 8 lost 全回收。门禁：registry 单测 8/8（6 基线+eased==1 回落钉+量钳制钉）+ engine
lib 4,068P/0F + webview lib 852P/2F 逐名对号在册环境形红（image_decoder spawn 缺件补件
复绿 + csp_gate s348 在册）零新增 + clippy/fmt 净；全量 make test 未跑挂账=HEAD 既有
fmt 漂移两文件（dom tests_9:455,462 / testharness:3755,3762 工作树干净反证在 HEAD）+
peer s419 WIP es_module:2040,2164 挡门（非本刀面，按质量门禁实际影响以定向腿覆盖）。
残面（下刀队列）：①clip-path shape() applier 臂（raw 插值全对 canon=None 恒回落——
masking clip-path-interpolation-shape 30 登记/30 None 实证）；②transition:all 逐
longhand 展开（FIXME(all) Phase 3）；③transition 离散翻转语义；④css-transitions corpus
帧驱动异步面（真 TransitionClock 管线，非本刀同步读机制覆盖）；⑤overshoot 长度外推的
共享插值器量域化精化（本刀注册表本地钳制的 Phase 2）。全案
[evidence 2026-11-16-transition-channel-knife](evidence/2026-11-16-transition-channel-knife.md)。
**shape-outside 门线刀已落地（11-16 十九轮，rate-limit 双 spawn 对称收敛第三例——twin 孤儿
WIP 认领收口）**：06:06 twin 实例 5 小时限额中断，WIP 静默 >2h 无提交，按磁盘终态仲裁认领
（孤儿核查无进程/无在飞 unit → WIP 完整性审读 11 文件连贯五站 → b1v2 二进制早于 06:03 追加
两臂 → 终态重建重跑权威账）。刀体三段：①**白名单五站**——style-system
`ShapeOutsideComputedValue` 三变体（None/Shape/Box）独立 wrapper（文法含裸 `<shape-box>` 形，
不用 ClipPathValue 别名不波及 clip-path 面）+ registry 白名单/initial_value 两处 +
存储槽/缺省/inherit + apply 文法门（`<basic-shape>` 复用 parse_clip_path 裁决单点 + 裸
shape-box 四关键字大小写不敏感；FIXME `<basic-shape> <shape-box>` 后缀组合形，corpus 现值
均裸形态）+ engine gCS `shape_outside_to_css` 三臂 + transition get 臂（clip-path canonical
同型）+ part03 钉（四关键字/非法拒收/缺省 none）；②**4 值 edge-offset 共享扩展**——css-parser
`parse_position_pair`（css-values-4 §position `<edge> <offset>` 对：首词定轴序、同轴双对
拒收、right/bottom → Calc(100%−o)，style-system parse_origin_edge_offsets 同构）+ circle at
三钉（直读/折算/同轴拒收；`<offset> <edge>` 乱序形 FIXME 后置，语料现值均 edge 首词形）；
③**附带臂 decl_supported 全局关键字**——`initial|inherit|unset|revert|revert-layer` 对
known_properties 恒真、**置顶于各属性文法早退之前**（CSS Cascade 5 §css-wide-keywords；
首建版被 counter-increment 文法门截胡恒假，置顶系行为修正故重跑权威批，已测五腿逐腿全同
证中性）——interpolation-testcommon `from [initial]`/`to [inherit]` 前置门整组连坐跨
corpus 形一次清偿（offset-position 刀残面「initial/unset/inherit 端点组 108」定谳）。
**A/B 权威账（B0=results-shape-b1v3 ≡ HEAD ff29d95be 采认不重跑 / B1=bin-shapeout-b1v4
md5 70f90c73 终态树重建；合成探针页 2 页批前出清 g41 先例——twin b1v2 曾卷入 +11P 假增益；
es_module s419 两侧同载零贡献）五腿净 +2,086 / 0 DOWNS / 页集合全等**：shapes **+1195**
（1894→3089/145 页，inset-001 +296/ellipse-005 +178 串化桶 + shape-interpolation 三页 +180
supports 前置门桶）、masking **+384**（995→1379/61 页，clip-interpolation keyword 端点组）、
motion **+354**（1134→1488/45 页，offset-*/position/anchor/rotate 端点组）、values **+153**
（4813→4966/256 页，B0=results-flip-b1 注：其后 values 面唯一已知增益 offset 刀 +1 echo
在案归因闭环——calc-size interpolate-size 152 + echo 1）、typed-om 护栏 **0/0/0**（359 页）。
两臂分解：五站刀体 ≈+1046 + 4 值 edge-offset +139 + 关键字臂 ≈+877；echo 红线（texp 转化率
轮巧合池）零触碰。门禁：engine 4060/0（part03 钉）+ style-system 2546/0 + css-parser 新钉
（E0425 作用域笔误修后重跑）；fmt 三 crate 净；clippy css-parser+style-system `-D warnings`
净、engine 唯一红 = peer es_module s419 在册连坐；make test 拆腿（zw-so2-mkt 08:45:15–09:03:08
GATE-DONE，12 腿）：绿五腿 companion-build/renderer-bin/gpu 双腿/
quickjs-renderer-bin 全 EXIT=0；红七腿全对号在册**零本刀域红**——workspace
15,651P/6F 全 es_module（s360/s361/s337/s269/reexport 族，peer s419 WIP
树态红）+ quickjs-clippy 101 es_module.rs:2217 too_many_arguments（同 WIP
lint 连坐）、renderer-lib 371P/1F + quickjs-renderer-lib 369P/1F =
worker-fetch（c6dcbce0a 在册镜像）、browser-bin 741P/2F + browser-tests
772P/2F = tab_js_worker fetch 镜像 + default_actions（在册对）、
quickjs-workspace 825P/1F = image_batch 负载形（在册；两腿数均含小目标
合计，非仅主二进制行）；cdp_focus 新鲜度 tripwire 本轮未现
（companion build 刷新二进制 mtime 顺带核销先例）。**bench-gate 全绿**
（zw-so2-bench 09:08:27–09:14:53 EXIT=0：微基准 3/3 css_bench/engine_bench/
style_bench + 页场景绝对预算 PASS——welcome p95 8.6ms/medium 307ms/morning
51ms、resource peak_rss 169MB、retained form input p95 0.018ms，报告
benchmark_20261116_090833.{txt,json}；dirty=true 系 es_module peer WIP
同载，A/B 零贡献同例）。残面（下刀队列）：shape 族真插值池（shape-outside-interpolation
660F/780、shape-interpolation 366F/546、path-interpolation 340F/478——门开插值可跑后的
中点期望残面，归 Transition 通道刀真插值池）、circle/ellipse-004 串化深形各 533F +
inset-003 302F + circle/ellipse-003 各 125F（declared 级 canonical：缺分量补 center/edge
序/calc 折算展开）、shape-margin/shape-image-threshold 两属性未入白名单各 120F 全灭页
（backdrop-filter 同型门线刀候选：shape-image-threshold `<number>` [0,1]、shape-margin
`<length-percentage>`）、composition 138F 在账。全案
[evidence 2026-11-16-shape-outside-whitelist-knife](evidence/2026-11-16-shape-outside-whitelist-knife.md)。
**地图校正（11-15 九轮收口探针定谳，改下一步方向）**：①前轮「@keyframes
apply 拒收全局关键字、keyframe-inherit-anim 证 from 帧被丢」为**探针伪账
更正**——同步 gCS 读不反映动画值：kw2 对照组（JS 建/static `<style>` +
`from{height:100px}→200px` 普通值）六个变体全 `auto`，waapi 探针
`animate([{opacity:0},{opacity:1}])` currentTime=500 读 opacity=**1** 非
0.5——Animation 对象链路通（playState/currentTime）、**值未进同步
computed 通道**。②插值 corpus 现有通过面走 CSS Transitions 离散路径
（grid-template-rows-interpolation 574P 实证多为 allow-discrete flip），
min-height-composition 0P/60F 证 additive composition 值同样不反映。
③真主矛盾 = **动画值进 gCS 同步读链**（WAAPI KeyframeEffect→引擎动画
时钟→computed 通道未接），锁死 WAAPI 面值域 + 语义桶 14.5k + 全局关键字
端点解析面——上限远超 supports 桶。CSS.supports 覆盖门仅在页级前置调用
（harness 本体只门 WAAPI isSupported 与 custom-props，`supportsValue` 对
css 方法恒 true）。engine 共享面共存记：ZeroWeb-3 流在 part05.js 施工
（M3-B/C 刀 staged 未提交，peer 提交 f82e60416 账面），本轮 engine 提交
已 rebase 干净、无文本冲突。
**composite-add 刀已落地（11-15 十一轮）**：shim `_makeAnimation` 对 add/accumulate
动画注册前预合成端点（endpoint' = underlying ⊕ endpoint 后按 replace 登记——U 恒定下
lerp(U⊕F,U⊕T,p)≡U⊕lerp(F,T,p) 数学恒等；可加域 `<number>`/px/`<angle>`/同 metric 对，
% 与混合单位/calc/关键字/函数列表 all-or-nothing 拒；KeyframeEffect 持原始 keyframes，
getKeyframes() JS 对象面零变化）+ 引擎 apply 十臂（font-size/font-weight/font-size-adjust/
letter-spacing/word-spacing/text-indent/min/max-sizing 严格解析，失败不动样式回落 B0
零回归方向）+ 插值三臂（font-weight 钳 [1,1000] css-fonts-4、font-size-adjust 同 metric
lerp、text-indent/min-max-sizing px 严格关键字离散）。A/B 双面（B0=bin-jsanim-b3 v3
快照 / B1=bin-cadd-b1，全程快照二进制）：五电池 **+75/0/0**（font-size-adjust-composition
+31 全纯 add 面、font-weight-composition 0→5、rotate-composition 4→9；values 与 B0
逐字节持平——border-*-width 伪通过池经空 underlying 天然过滤保位）+ 16 组
composition 定向双腿 **+197/0/0**（margin/padding 8 页、width/height/min-max 6 页各
0→12、letter/word-spacing 12/12、text-indent 5）——**合计 +272/0/0 零回归**，超前估
~146G（定向 16 组 box/sizing/text 面超原估口径）。过程事故：批脚本循环变量名
`GROUPS` 撞 bash 特殊只读数组（用户组 ID 列表）赋值被静默忽略 → `$GROUPS` 展开
"1000" 单组 corpus usage exit 2，改名 `_zwGroups` 重跑 32 腿全绿。见 [evidence
2026-11-15-composite-add-knife](evidence/2026-11-15-composite-add-knife.md)。
make test 门禁（zw-cadd-mkt2 systemd unit + test-guard，177 腿 18:06 收）：本刀域
全绿（animation 171 ok、engine 面零红）；唯一红腿 es_module s337/s345 两测经机制核
（WIP diff 明文替换 s345 旧 globalThis getter 形并同步改断言）归 script-sandbox 流
在途 s419 WIP 树态红，不入本刀账。
**剩余腿九腿账（zw-cadd-legs 单元 18:23 收，绿四红五，零新增本刀真实代码红）**：
绿四腿=renderer bin / GPU renderer / GPU consistency / quickjs renderer bin；红五腿
逐名对号——renderer lib 1F=worker-fetch（在册 c6dcbce0a reject 语义过时镜像）；
browser bin 2F=tab_js_worker 同根镜像+default_actions（在册三红账）；browser tests
6F=cdp_focus_governance 整套 0.00s panic，**新形伪红**：该套件 277 行新鲜度 tripwire
断言 zero-renderer 二进制 mtime 晚于 js_dom_shim 最新源 mtime——18:16 rebase re-pick
（内容恒等）刷源 mtime 而二进制未重建，六测未执行即 panic，机制由 panic 文本自证，
`build -p zero-renderer` 重建即翻绿（树被在途施工占用暂缓，下轮门禁顺带核销）；
quickjs crates 腿 zero-script-sandbox quickjs lib 280P/**47F es_module**——s419 WIP
在 quickjs 组合下扩面（v8 形仅 s337/s345 两红；失败集含 WIP 新增 s419 具名测，
stdout 见 `with (__zwImpH$)` 包装形），归 script-sandbox 流在途树态红不入本刀账；
quickjs renderer lib 1F=worker-fetch quickjs 镜像同 c6dcbce0a 根。全账真实代码红仍
SW 具名导入 getter 一处。碰头记：18:32:21 起 js_anim_registry.rs（AnimationClock
timing 面）现他流在途施工（E0382 中间态+`cargo test -p zero-engine --lib anim`
腿在跑），与本刀已提交 animation.rs 文件不重叠；九腿账采集窗 18:16-18:23 先于其
落地，provenance 由腿日志时间戳锚定。残面候选（下一刀）：easing 感知离散翻转采纳 ~88G、url 绝对化 ~9G、MULTI 值形
（transform 列表/背景对）逐函数合成。
**easing 感知离散翻转采纳刀已落地（11-15 十二轮）**：jsanim v2 fallback 检测器
把「开区间 tick 输出 ≡ 端点原文」一概判引擎 echo 回落 B0，把 spec 离散/步进的
合法恒端点输出全折（css-values-4 §8.5.6 不可插值对 transformed progress 0.5
翻转、step-start/end eased 钳 0/1 同形；round-10 账离散面 ~50 真增益 v1 内含、
v2/v3 清零）。放宽为 **eased 一致性采纳**：`flip_ok` 形态门（恰 2 帧、端点
offset 0/1 选择器域判定——数组序不信任、direction Normal）下重算
`eased = apply_timing_function(prog, entry.timing)`：输出 ≡to 且 eased≥0.5 /
≡from 且 eased<0.5 / 恒值对（from≡to 任何进度）→ 采纳；侧别失配/NaN → 维持
回落（零回归方向）。eased 同源：合成规则帧不携带 animation-timing-function
（`_zwWaapiSpecKeyframes` 丢帧级 easing 键）→ 引擎段内恒用全局 timing；span 1
归一化恒等使整体进度 ≡ 段进度，entry 级重算精确。A/B 双面（B0=bin-cadd-b1
十一轮 B1 快照 / B1=bin-flip-b1，全程快照二进制；marker `zw-easing-flip-adopt`
走 part03.js 注释入二进制——Rust const/static 不引用不落二进制三试教训）：
新面八任务双腿 **0/0/0**（六组 corpus 无 WAAPI 动画面 flip 链 inert——目标
选择事后剖析见 evidence）；五电池护栏 **+118/0/0 零回归**，经账目三桶拆分
（采样位置 × 期望端点形态程序化分类）归属定谳：**flip 离散直接 +21**
（transforms 007 matrix3d↔matrix 离散对 3 + values max-width/height
none↔20px 离散对 4 + fonts metric 混排离散对 14——interior 端点期望，
本刀设计目标面）、**cadd 臂迟达 +5**（font-size-adjust add 组合中点真插值
期望 ∉ 端点=插值臂需求，bin-cadd-b1 构建早于臂落树实锤）、**外来钳制面
+92 挂待归因账不记本刀**（B0 tick 在 prog≤0/≥1 无发射瞬态：主候选
animation.rs 17:43 构建瞬态提交前回退〔commit diff 不可见〕，次候选
es_module s419〔nm 符号集 76 vs 460 内容差实锤、module 编译域无 classic
路径〕；本刀三重排除＝shim verbatim blob 逐字节同 + 注册流纯增量 +
eval 钳制位恒等）。按属性分组 REGRESSED 空表，离散属性白名单收紧预案未
触发。**通道教训**：css-grid grid-template-rows-interpolation 失败面走
CSS Transitions 通道（allow-discrete）不经 js_anim_registry，本刀对其无效
（实测 0/0/0，归 transition 通道另刀）；「~88G」挂账实际落点 = transforms/
values/fonts 三域离散翻转面，背景族 discrete-no-interpolation ~119 候选未兑现
（引擎无该族 keyframe echo 输出，采纳链 inert）。单测 flip_eased_adopts 七钉 +
engine anim 123/123 全绿；tick 副作用纪律零交互（flip_ok 门直读 keyframes AST，
无新增 tick，与 AnimationClock.tick learning 无副作用路径约定同型）。见
[evidence 2026-11-15-easing-flip-adopt-knife](evidence/2026-11-15-easing-flip-adopt-knife.md)。
make test 门禁（zw-flip-mkt2 systemd unit + test-guard，19:29:38 发射 / 19:50:11
收；首轮 zw-flip-mkt fmt 全局步被 peer WIP 四文件漂移阻断〔animation.rs/
computed_style.rs=offset 刀、es_module.rs=s419、apply_advanced.rs=offset 刀〕，
按 composite-add 先例拆腿、fmt 只核本刀文件 FMT-SCOPE ✓）：companion ✓ /
GPU adapter+consistency ✓ / renderer bin ✓ / quickjs renderer bin ✓；红腿逐名
对号（**零本刀域红**，engine anim/registry 面全绿）——clippy 1F=script-sandbox
`too_many_arguments` 8/7（s419 WIP lint 连坐，peer 域）；workspace 测试
1F=image_batch_e2e 41≠36 负载峰值形（在册）；renderer lib 1F=worker-fetch
（在册 c6dcbce0a 镜像）；browser bin 3F=tab_js_worker 镜像+default_actions
（在册）+ gpu_compositor_path_dispatches_input_events_to_form_controls
（新面孔 IPC 帧头断裂形态，**solo 定谳 flake 不入账**——双会话合计 solo
×11=8 绿 3 红〔红侧签名=compositor「通道关闭，退出」后 signal 11 断帧；
flip 侧加密采样 3/8 红、门禁收后低载仍现，纯负载相位解释存疑〕，分类一致归
compositor 域后续查）；
browser tests 2F=tab_js_worker+default_actions（在册）；quickjs workspace
1F=image_batch（同负载形 quickjs 面）；quickjs renderer lib 1F=worker-fetch
quickjs 镜像（在册）。round-11 cdp_focus 新鲜度 tripwire 伪红本轮未现
（companion build 刷新二进制 mtime 顺带核销）；round-11 script-sandbox
quickjs 47F es_module 扩面本轮未现（s419 树态推进，peer 域）。
残面候选（下一刀）：url 绝对化 ~9G、MULTI 值形（transform 列表/背景对）逐函数
合成、CSS Transitions 通道离散翻转（grid-template 面）。
**动画值进 gCS 刀已落地（11-15 十轮，jsanim v1→v3 三代 A/B 收口）**：shim
`Animation` 经 `__zw_js_anim_register/seek/playpause/cancel` 把 keyframes/timing/
播放态登记到引擎侧 `js_anim_registry`（thread_local ENTRIES + 独立
AnimationClock 实例，渲染管线零触及），gCS selector/handle 两通道序列化前咨询
（apply 后过同一规范化序列化器）。三代形态：v1 裸奔 +586G 但 245 REG
（composite/calc-size/url/neutral 缺端点等离散 fallback 全翻红）；v2 +形状守卫
（composite≠replace、calc-size(/random(、裸关键字端点不登记）+fallback 检测器
（开区间 tick 恒等端点原文=未真插值→回落 B0 通道）+canonicalize None（序列化
恒等→回落 gCS 通道）→ 183G/0L/**9R**；v3 +端点合成（neutral 单帧臂 JS 侧注册前
读 underlying 补 0%/100% 帧，Web Animations §5.4.15）+混合单位端点门（px↔%/
px↔em 跨单位插值退化面不登记；无单位侧合法插值不误门——首版 none→X 陪葬
−21 事故已修）→ 五组 A/B（B0=results-jsanim-b0/B1=results-jsanim-b3，全程
快照二进制防混源）：**css-transforms +185/0/0、css-backgrounds +13/0/0、
motion/values/fonts 0/0/0，合计 +198/0/0 零回归**。教训三则：corpus A/B 前
必核双腿 page 集合相等（peer 施工目录瞬态缺席曾伪增 183 REG，补腿法
`CSS_CORPUS_ONLY_SUBSTR` 定向重跑）；`cases=` 汇总是 merge 侧 entry-页计数
非尝试数（库存核对以 transcript `==== ` 行为准）；B0 伪通过池（echo 恒等/
期望本身垃圾串）以守卫保位不追真值。残面增益候选（下一刀）：composite-add
建模 ~146G、easing 感知离散翻转采纳 ~88G、url 绝对化 ~9G。见 [evidence
2026-11-15-jsanim-gcs-value-knife](evidence/2026-11-15-jsanim-gcs-value-knife.md)；
背景与分桶见 [evidence
2026-11-15-texp-groups-conversion](evidence/2026-11-15-texp-groups-conversion.md)；
规格溯源（候选定谳/站点行级定位/探针矩阵）见 [evidence
2026-11-15-target-expando-interpolation-unlock](evidence/2026-11-15-target-expando-interpolation-unlock.md)。
**提交门禁 make test（11-15，mkt3/mkt6 双跑一致）**：fmt/clippy 绿；workspace
腿 135P+1F——`service_worker_manager::module_update_compares_static_dependency_bytes`
真实代码红非伪红（11-15 纠类；es_module 具名导入装 globalThis getter 撞
84a48833d 老钉 `globalThis.value` 赋值抛 TypeError，script-sandbox 导入绑定
面，与本刀零代码路径，修复归 script-sandbox 流；红窗 11-08 s345 打开，老钉
08-20 入树后因首败截断+缺件遮蔽在 make test 面从未执行过、17:03 首暴露）；
Makefile 补建 zero-browser 防
image_batch_e2e 缺件伪红（清库必现形态）。
**剩余腿九腿补跑账（11-15，zw-jsanim-legs 单元 17:31 收官）**：绿四腿
（renderer bin / GPU renderer / GPU consistency / quickjs renderer bin）；
红五腿全对号**零新增真实代码红**——renderer lib 1F=js_worker worker-fetch
（c6dcbce0a reject 语义过时镜像在册）；browser bin 2F=tab_js_worker 同根
镜像+default_actions（均在册三红账）；browser tests 3F=前两者重跑镜像+
bookmark_context_menu_open_in_new_tab_loads_url **回填定类=负载相位 flake**
（同测 17:20:49 过/17:23:45 挂跨腿翻转，曾落 editing WIP 70cb4d2f5 共树在途
落地 17:23:27 后 18s；发现流 solo ×3 全绿定谳——`--tests` 面 xvfb 前缀
EXIT=0×3（0.67s/0.06s/0.06s，/tmp/zw-jsanim-solo-b2.log），solo 二进制含
70cb4d2f5（HEAD 3fdbc3c53 编译），混码嫌疑同证排除，负载窗并发唯一余因）；
quickjs crates 1F=image_batch_e2e 42 fetch/36 图 6 重复 fetch=在册伪红
负载峰值形态第三现形；quickjs renderer lib 1F=worker-fetch quickjs 镜像
同 c6dcbce0a 根。全账唯一真实代码红仍是 SW 具名导入 getter 一处
（script-sandbox 面）。细目见 evidence jsanim 刀文档验证节。
**g50′ 完成（11-15，提交 7bc31687d，同树并发流落地）**：apply 期视口
thread_local 单源 + vertical-align `<length-percentage>` 三层
（parse/computed/inline 布局 %×line-height），css-values
viewport-units-css2-001 残 12F（border-spacing 4 + vertical-align 8）
目标面；calc-size 链 WIP（8 文件）在 stash@{0} 隔离待 A/B 后还原另刀。
**DC-3 证据补强（11-15 二轮，同日）**：①语料面定谳——ci-shards.yml 已接线
65 套件覆盖全部主要 css 组，剩余 ~119k 缺口 = 引擎能力面（未接线零散组仅
~1.1k，语料杠杆已耗尽，刀链是唯一主杠杆）；②快照树态 ∈ [g41, g42)（12 组
CI vs 本地对账 |Δ|≤33），当前 css 面真值 ≈ **64.8k**；③masking 6,281 缺口
再分解：chrome 65 载子测文件共 6,398 子测，本地镜像 573 文件零缺，但 22 页
setup 即崩（共享插值 harness `createTargetContainer` target `.style`
undefined 单根因）堵 **5,859/6,398（92%）**——单根因刀优先级显著上调。
该 crash 面已由 target expando 刀解除（见上）。见 [DC-3 刷新证据
§3/§4/B′](evidence/2026-11-15-dc3-snapshot-refresh.md)。
**g49 完成（11-15）**：GCS/媒体门视口真值对齐刀——GCS 快照通道四处与渲染
`<style media>` 门恒 1280×800，而 runner/宿主实视口 800×600（shim 安装期
`set_pending_layout_viewport`）→ viewport 单位（10vw→128px≠80px）与媒体门
求值双面失真。修法：`computed_style_cache::gcs_viewport_dims()` 单源读
pending 视图布局 viewport（未接线回落 1280×800）；GCS 四站去硬编码 + 渲染
collect 链（collect_stylesheets/with_parsed_external/
style_sheet_if_eligible/collect_interleaved_external）viewport 穿参至
`MediaContext`（FIXME 清偿），shadow 内联链签名免穿全局读。A/B（B0=
results-g47-b2 + results-g49-b0 anchor / B1=results-g49-b1）：viewport-
units-css2-001 84F→12F **fixed +72 零回归**（css-values 4113→4185P），
css-images/anchor 全持平——刀面精确命中。残面：css2-001 残 12F 案级定谳 =
border-spacing 4 + vertical-align 8（vw 族属性槽承接缺口，另刀）、
viewport-units-invalidation 24F = iframe `contentWindow.getComputedStyle`
未桥接（缺能力面，案级消息定谳）。见 [evidence
2026-11-15-g49-gcs-viewport-truth-alignment](evidence/2026-11-15-g49-gcs-viewport-truth-alignment.md)。
**g48 完成（11-15）**：gradient `at` 位置文法位 + canonical 序 + computed
折算刀——csswg#2140 后 `<position>` 行为定谳（oracle: gradient-position
三页）：3 值混搭形全禁（旧 3|4 值宽松 walk 误收 `at left 4px top` 五形）、
1 值缺省轴补 center（旧实现 `x=y=该值`——computed 18F 主根因）、4 值双缘
偏移对 right/bottom 百分比 100−x 折算（`right 30%`→`70%`）、canonical at
子句重排（`bottom 10% right 20%`→`right 20% bottom 10%`，标准序保形）。
`parse_position_pair` 重写 + `canonicalize_at_clause`（css-parser
parse_transform.rs 单文件，radial/conic 两接线）；computed 面零改动（折算
pair 流经既有 computed_head）。A/B（B0=results-g46-b2/B1=results-g48-b1
合并树）：gradient-position invalid 5→0 / valid 4→0 / computed 18→4
（**+23 零回归**）；合并树 grand 净 +49（css-images 3091→3127、css-values
4041→4054），其中 +34/−8 归并发流 g47 object-position 写面判定刀（其
position-valid.tentative 逻辑关键字 x-start/y-end −8 回归归 g47 轮收敛，
R2478 pin `left 50px center` 冲突同）。残面：computed 4F 长度锚
（`calc(100% - Npx)` 须缘锚定模型扩展）、object-position valid/computed
族、conic-calc-angle 8F。见 [evidence
2026-11-15-g48-gradient-position-grammar-canonical-computed](evidence/2026-11-15-g48-gradient-position-grammar-canonical-computed.md)。
**g45b 完成（11-15）**：attr 污点经自定义项传播刀——css-values-4 §attr
security 三页清偿。根因：预级联 attr 替换站把 `--s: attr(href)` 替换成
属性文本后词痕即逝，4.75 重裁站失察（g45 image-set 文法位放行前被文法拒
意外遮蔽）。修在预级联站：替换成功且 prop 为 `--` 项时登记污染名，替换
完成后取 URL 属性声明精确引用 `var(--名`（ident 级比对）→ 整条声明丢弃
（IACVT 回落初值 none）；跨元素继承链不在本站视野（FIXME 挂账）。单测
style-system 2,539（+正钉/负钉）全绿、fmt/clippy 净；A/B（B0=results-g46-b1
修前二进制 / B1=results-g46-b2 修后唯一内容差，touch 破 freshness 真重建）：
css-values 4038→**4041P 净 +3**（attr-security-animation/transition/
universal-selector 3 案 FIXED，全页 NEWFAIL=0），css-images 3091P 维持；
对 g44-b1 基线 css-values 4041P/1546F 回零。见 [evidence
2026-11-15-g45b-attr-taint-custom-prop](evidence/2026-11-15-g45b-attr-taint-custom-prop.md)。
**g46 完成（11-15）**：calc 负分辨率存储形回读 + content×image-set 刀——
探针定谳 `calc(-1 * 1x)` 得 "none" 根因在 canonical 存储形 `calc(-1dppx)`
回读（calc 一元负号维度字面 neg 重写落 `0 - x` 通用臂 → 纯 Resolution 门
拒裸 Number 叶）：共享解析器 neg 臂加 Resolution 保族重写
`Resolution(0) - x`（镜像 Length `0px - x`，字面混型 `calc(0 - 1dppx)` 仍拒）
+ computed 负值钳 0（css-values-4 §6.4，oracle `0dppx`）+ content `<image>`
域 ImageSet 臂（`parse_image_set_value` 单实现点，background 层臂同源）+
style-system 直通 + engine content_to_css 复用 image_set_to_css。A/B
（B0=results-g45-b1/B1=results-g46-b1）：css-images 3087→**3091P 净 +4**
（image-set-computed F 4→0 全页清零，image-set 双页收口 0F+0F），css-values
±0 零回归；单测 css-parser 3,227（+1 钉，账面更正 g45 3,227 实为 3,226）/
style-system 2,538/engine 4,013 全绿，fmt/clippy 净。残面：position-invalid
5F/conic-calc 3F 细形、content
image-set 渲染面 FIXME。见 [evidence
2026-11-15-g46-calc-negative-resolution-content-image-set](evidence/2026-11-15-g46-calc-negative-resolution-content-image-set.md)。
**g45 完成（11-15）**：image-set 解析与串化刀——image-set() 整值可解析
（css-images-4 §image-set-nota：`-webkit-` 别名剥除、负字面分辨率拒收、dpi/
dpcm→dppx 折算、calc 三态折叠 `fold_resolution_typed` 含符号归一重试）+
canonical 恒 Some 投影（url 引号化/缺省补 1x/calc §10.6/标准序）+ decl
invalid 判定扩接双写面（IDL part06 content/border-image-source/cursor 臂 +
setAttribute g44 通道）+ style-system 变体直通 + engine computed dppx 串化 +
painter 防御臂。A/B（B0=results-g44-b1/B1=results-g45-b1）：css-images
2941→**3087P 净 +146**（image-set-parsing F 114→0 全页清零 144/144 Pass、
image-set-computed F 36→4），css-values −3（attr() 污点暴露——B0 碰巧过，
拒 var() 为 spec 错误方向，正修独立刀）；单测 css-parser 3,227（+4 钉）/
style-system 2,538/engine 4,009 全绿，fmt/clippy 净。残面：computed 4F
（content×image-set 支持门 2F + calc 负分辨率折叠/钳制 2F，均 B0 既有）、
position-invalid
5F/conic-calc 3F 细形。见 [evidence
2026-11-15-g45-image-set-parse-serialize](evidence/2026-11-15-g45-image-set-parse-serialize.md)。
**g44 完成（11-15）**：色标文法位 + style-attr 写面拒收刀——color-stops-parsing
60F 双通道分修：parse 侧 hint 位约束（首位前/连续/尾悬挂拒收 + 空段拒收 +
单色标表合法）与 radial 首参单百分比不再误派发 head；写面新增
`_zwStyleAttrDeclFilter`（style 内容属性 setAttribute 通道逐声明 invalid 渐变
拒收，g43 判定器 'null' 哨兵复用，幸存声明原文保形，handle 世界 part04 +
物化世界 part03 双臂）。A/B（B0=results-g43-b1/B1=results-g44-b1）：css-images
2881→**2941P 净 +60**（color-stops-parsing F 60→0 全页清零 162/162 Pass）零
回归，css-values 4041P ±0；单测 css-parser 3,223（+1 钉）/engine 3,991 全绿，
fmt/clippy 净。事故：首录刀口落物化世界净 ±0（活跃文档实为 handle 世界），
补 part04 臂后兑现。残面：image-set 150F（C+D 刀）、position-invalid 5F/
conic-calc 3F 细形。见 [evidence
2026-11-15-g44-color-stops-style-attr-write-gate](evidence/2026-11-15-g44-color-stops-style-attr-write-gate.md)。
**g43 完成（11-15）**：渐变 invalid 写面拒收刀——292 形二桶分修：parse 严格性
（三型调用方 `in` 子句出现必带合法方法，畸形整渐变拒收，桶 B 10 形含极坐标缺
hue 后缀/`in` 后倒置）+ 写面拒收判定器 `background_image_decl_invalid`
（渐变前缀 + parse 失败 → invalid，回调 "null" 哨兵 + shim 拒收臂，桶 A 282 形
裸方法词不再回显原文）。A/B（B0=results-g42-b1/B1=results-g43-b1）：css-images
2584→**2881P 净 +297**（invalid 页 F 292→0 全清 + position/conic-calc 溢出
+5）零回归，css-values 4041P ±0；单测 css-parser 3,222（+2 钉）/engine 3,991
全绿，fmt/clippy 净。残面：position-invalid 5F/conic-calc 3F 细形、color-stops
60F、image-set 150F（6b 指派）。见 [evidence
2026-11-15-g43-gradient-invalid-write-gate](evidence/2026-11-15-g43-gradient-invalid-write-gate.md)。
**g42 完成（11-15 收口，11-14 深夜开工遇 429 rollover）**：渐变 computed 通道
canonical 串化刀——`ColorInterpolationSpace` 模型身份保留（Hsl/Hwb/XyzD50/
XyzD65 入枚举，渲染回退移交 paint map_interpolation）+ `ColorValue::ColorFunction`
未解析保留（color() 不再 parse 期坍缩 Rgba，canonical 空间名/分量域 0-1/
xyz→xyz-d65 别名，paint 回退 to_srgb_rgba）+ engine computed 双路串化
（specified 骨架 `gradient_specified_to_css`：配置头 canonical——linear 角词序
保真/radial size 单位归一 `40em`→`640px`+位置模型化+缺省省略/conic from 角度
归一；`in <method>` 冗省同 declared 则；`color()` 站保真 `color(srgb 1 0 0)`；
程序化构造回退 R2750 口径不变）+ radial size-only 首参派发 + 4-token 缘偏移
位对解析。A/B（B0=results-g41-b2/B1=results-g42-b1 quiescent 重录，首轮被
僵尸 disk-guard 清库焚毁作废）：css-images 1439→**2584P 净 +1145**（computed
F 881→0、valid F 264→0 两页全清、余页零漂移）零回归，css-values 4041P ±0；
单测 css-parser 3,220（+3）+ engine lib 3,991/all-targets 47 target 全绿，
fmt/clippy 净。残面：invalid 拒写 292F（A 刀文法门）、color-stops 60F、
image-set 114F+36F（6b 指派）。事故与手术记见 [evidence
2026-11-15-g42-gradient-computed-serialize](evidence/2026-11-15-g42-gradient-computed-serialize.md)。
**g41 完成（11-14）**：渐变声明值 canonical 串化刀——linear/radial/conic-gradient
写面 canonical（css-images-4 §serializing-gradients：配置前置/`in <method>`
后置/缺省法冗省（predefined color() 站→oklab、legacy 站→srgb、声明法==缺省
且 hue 缺省省整子句）/xyz→xyz-d65 别名/色标 as-written）；核心
GradientSpecifiedParts 文本承载 + `canonicalize_background_image_specified`
（非渐变/未识别面整值 None passthrough 零回归），写面单点
fold_decl_specified background-image 臂（g40 image-set 折叠回落）+ shim
回调双通道（grid G1 同型）。B0（results-g40-b1 g40 提交态）/B1
（results-g41-b2，探针页删除后同二进制干净重录）受控 A/B：css-images
887→1439P 净 **+552**（valid +549/computed +3）、css-values 4041P ±0 零
回归，UPS 552/DOWNS 空（初版 b1 卷入本地合成探针页 +4P/1F，跑后已删
不入库）。单测 css-parser 3217（+5 钉）+ engine 3959 全绿；
clippy/fmt 净。本轮双 spawn 竞态收敛（5 复苏位，写面采纳单点分派+双通道
收敛形态）与陈旧二进制事故定谳（44s 构建复用旧 engine rlib，B0 恒等作废）
见 evidence §3。残面：G2 computed 881F、invalid 拒写 292F、4-token 位置/
size-only radial 解析缺口、color-stops 60F、image-set 114F+36F（6b 指派）。
见 [evidence 2026-11-14-g41-gradient-canonical-serialize](evidence/2026-11-14-g41-gradient-canonical-serialize.md)。
**g40 完成（11-14）**：serialize-resolution 刀（上一轮 rate-limit 中断孤儿
认领完成）——`<resolution>` 维度（`CalcExpr::Resolution` dppx 基归一：
x/dppx 恒等、dpi÷96、dpcm×2.54/96，dppx 后缀先于 x 判定）+ image-set()
声明值 calc canonical 折叠（`fold_image_set_specified` 引号态机扫描 +
`parse_resolution_calc` 独立入口 allow_resolution 旗标同 allow_time 口径
——共享 parse_calc 保持 resolution 闭 + `calc_expr_is_pure_resolution`
纯门 + MTResolution 型别表行 + 全臂 exhaustive fail-closed 扫；读视图
单点 js_dom_bridge.rs style_from_mutations_lw 既有 fold_decl_specified
消费，零新增写站）。B0（19:36:15 刀前二进制）/B1（21:19:52 刀后）受控
A/B：css-values 4012→4041 净 **+29**（calc-infinity-nan-serialize-
resolution.html 29/29 全页清零，UPS 无第二页）+ css-images 881→887 净
**+6**（image-set-parsing.html calc 形红利）——双 corpus REGRESSED=0。
单测 css-parser 3212 全绿（+4 新钉：单位归一/NaN·±∞/纯门+型别表/fold
端到端幂等 fail-closed 多字节）+ style-system 2538 + layout-engine
2515+20；fmt/clippy 净。双 roll 并行写账竞态收敛合并定稿（工件逐案
对账为准）。残挂账：% 叶 ±∞ 序列化 2F、image-set computed/supports
通道 36F、serialize-number 嵌套 mix 3F。见
[evidence 2026-11-14-g40-resolution-image-set-fold](evidence/2026-11-14-g40-resolution-image-set-fold.md)。
**g38 完成（11-14）**：anchor-size() used 值布局期求值刀——六尺寸槽
（`ComputedStyle::anchor_size_slots`）+ 四 inset 槽双形承载，布局期
anchor_pos pass3 按锚几何回填 used 宽高（css-anchor-position-1 §4：
物理/含块/自身三轴映射 + 隐式缺省关键字 + fallback + replaced 均匀
缩放 + 普通值清槽防回灌）；anchor 语料对 HEAD 基线净 **+70**
（+77/−7 全 Fail↔Pass：anchor-size 四页 +52、anchor-name 家族
+18/−6、ident-function −1）；HEAD+g38 = 265 页 11,199P/1,654F/4T。
见 [evidence 2026-11-14-g38-anchor-size-used-value](evidence/2026-11-14-g38-anchor-size-used-value.md)。
**g39 账目更正（11-14，g38 轮定谳）**：上段 g39 组合态账
12,061P/796F（对 g37-final 真态净 +68、本刀 +3 零回归）撤销——系
provenance 过期 A/B 二进制所测（18:27:55 编译先于 18:39 终版提交；
留存日志实存 12,061P/792F 亦 Δ4F）。提交态逐案分解：账基线更正为
g37-final 工件基 **11,989P**/864F（旧线 11,993P 与工件 Δ+4P，原始
stdout 未存无法复源；本轮三 corpus 键集逐案同一全按工件计），g39
直接效应 +5、伪通过摘除 −865（writing-modes-001 864 + 探针页 1）、
净 **−860**——strut 门揭了先在 VRL in-flow 布局缺口（writing-modes-001
1296P 系 sizing 空转重布局副作用撑出的伪通过桶，g36 挂账疑定谳），
修复面另立 VRL in-flow 布局刀；g38 净 +70 独立计。同见 g38 evidence §4/§6。
**g39 完成（11-14）**：空绝位 auto 高 strut 门——abspos 塌 0 补测只对
有行内内容盒写行高近似（CSS2 §10.6.4），g36 'none' 形态挂账清偿；组合态
账 265 页 12,061P/796F/4T（对 g37-final 真态净 +68），本刀面 +3 零回归；
见 [evidence 2026-11-14-g39-abs-empty-strut-gate](evidence/2026-11-14-g39-abs-empty-strut-gate.md)。
**s424a 定谳回退（11-14，无代码落库）**：class 属性族门拓宽受控 A/B——
s423 selector rect 臂放行族拓宽到 classList/className（engine 三文件实验
补丁，part115 单测全绿），同基座 ± 门对照：**+3 真增益**（dynamic-001/004
单 classList 变异页 + fallback-invalidation 1 案）/ **−865 揭真**
（writing-modes-001 864 案伪过池 + 金丝雀 1 同步翻转——真缺口 = writing-mode
× anchor 组合未建模，归 VRL in-flow 布局刀挂账）→ 净 **−862** 违背指标
方向定谳回退；engine 三文件已回 HEAD 零残留，补丁与 A/B 工件留
`.acceptance/`。假设证伪面：anchor-name-mutation 15F 页变异经 template
cloneNode+append 结构性队列，`all()` 谓词否决门仍闭——清偿需 selector 臂
结构性放行（s421 无门全重放语义 + 等值性实证），挂账。见
[evidence 2026-11-14-s424a-class-gate-widening-experiment](evidence/2026-11-14-s424a-class-gate-widening-experiment.md)。
**s424b 定谳回退（11-14，无代码落库）**：selector 臂结构性放行受控 A/B——
s423 臂 family 谓词撤为 `style_version > applied` 单窗口判（callbacks.rs，
对齐 s421 handle 臂语义），同基座对照净 **−1065**（+4 同 s424a 增益面 /
−1069 = 揭真池 865 + try-tactic-position-area 131 等变异面新增暴露 204），
**目标 anchor-name-mutation 15F→15F 零清偿**——失败形态质变：基座 15 案
全 `got 0` 陈旧读、实验组全数 `expected X got X+1` 三轴恒 +1 且断言链前移
（变异重放已通到布局层、活值读生效，残账 = 重布局几何/读链恒 +1px，独立
挂账）；callbacks.rs 已回 HEAD 零残留，工件留 `.acceptance/`。**s423 谓词
两轮实验（s424a class 拓宽 −862 / s424b 结构放行 −1065）定谳收口：均证伪，
门维持收窄态**；揭真桶清偿前提 = VRL in-flow 布局 + position-try fallback
状态机建模落地后重开。双 session 写账竞态收敛取先手 evidence，补丁形态与
归因以 artifacts 对账更正（基座/实验组读值消息逐案比对），见
[evidence 2026-11-14-s424b-structural-gate-experiment](evidence/2026-11-14-s424b-structural-gate-experiment.md)。
**g37 完成（11-14）**：anchor()/anchor-size() calc 嵌入叶 + serialize
canonical 三修——anchor-parse-valid 4F/anchor-size-parse-valid 8F 全清 +
invalid-fallback 4 伪红修正，anchor 语料 11,977→11,993 净 +12 零回归；
comb3-x7 权威账 parse-valid 0F 经 HEAD 反证定谳伪账（详见 evidence）；
见 [evidence 2026-11-14-g37-anchor-serialize-canonical](evidence/2026-11-14-g37-anchor-serialize-canonical.md)。
**g36 完成（11-14）**：s423 门控收窄 + anchor_pos V 系坐标系统一——anchor
语料 11,870→11,977 全量净 +107（真语料 +88 零真回归，position-area-basic
+45/wm-dir +44），4 REG 定谳 last-successful 伪通过揭开挂账；见
[evidence 2026-11-14-g36-border-coord-s423](evidence/2026-11-14-g36-border-coord-s423.md)。
已落库未入快照增量逐批见 DC-2 各批账与里程碑表（typed-om/color/values/
grid/anchor 各批，至组F 布局刀3→g35 wm/对齐刀 anchor 语料 11,863→11,870（g35 轮扩面基线 11,869 +1 净）；本流组E color
face css-color 10,905→10,972，至通用色通道 cq/sign 刀；values face
css-values 2,426→2,893，至 serialize canonical 折叠刀2+round B=±∞ 策略表
+calc-mix() 全链刀（g14b 权威账净 +100 零回归，六套件对账零漂移）
+写面文法门收口（g14c invalid 残 6F 清零）
+嵌套和串刀（g15 净 +10 零回归，values face 2,903）
+tree-counting 叶/符号零刀（g16 净 +14 零回归，values face 2,917；提交态
组合权威账 g17 2,953，含 9db computed 串化刀）
+computed canonical 串化刀全套（9db 主刀 + be0 补臂，g17/g17b 两轮：
minmax/hypot 单参配对 28 案 g15 空串对空串伪通过转真通过，values face
2,917→**3,032** 净 +129，calc 族逐页零损失；分页归因与组合态污染分账见
evidence 2026-11-12-dc2-g17）
+sibling 求值桥刀（g19：CalcContext 注入求值 + gCS 三通道 + 写面文法门，
values face 3,032→**3,072** 净 +40 零回归，五套件 ±0；两轮探针 49P→56P
修复 cache-hit fail-soft 与 parse-fail 放行两洞，见 evidence
2026-11-12-dc2-g19-sibling-bridge）
+random canonical 串化刀（g20：§9.4.1 `<random-key>` parse 期 canonical
化挂 fold_decl_specified 头部单点接线 5 写面站，values face 3,072→
**3,111** 净 +39 零回归，五套件 ±0；random-serialize 42F 收口 39F 残
color 通道嵌套 3F 挂账，见 evidence 2026-11-12-dc2-g20-random-canonical）
+random 播种求值全链刀（g21：§9.4 Evaluating sha256 doc/element 双 seed
PRNG + min/max/step 公式 + compute 期元素级原位替换 + % 叶 fixed-echo
保留形 gCS 直读表 + style_cache prescan 禁用 + 简写前 canonical 化 +
supports 哨兵门，values face 3,111→**3,214** 净 +103 零回归——
random-computed +95/random-item-computed +5/item-nested +2/unresolvable
+1，五套件 Pass 级零漂移，探针两轮洞修复 16P/169F→116P/69F，见 evidence
2026-11-12-dc2-g21-random-eval-face2）
+random-item 写面文法门（g22：§funcdef-random-item 实参文法裁决——
key 节缺失/项表空/顶层 `!;]`/块残片/括号未闭合五类 invalid 形三写臂共用
门，values face 3,214→**3,228** 净 +14 零回归，random-item-invalid 14F
全页清零，五套件 Pass 级 ±0，见 evidence
2026-11-12-dc2-g22-random-item-write-grammar）
+min/max/clamp canonical 保持串化 + 写面 math 文法/槽族门 + clamp none
重写（g23：§10.6 canonical 排序保持串化 margin 槽接线 + §10.5 写面
22 数学函数文法×槽族型别门（LengthOnly/LengthPercentage/Number 三档 +
Invalid 除法二分）+ §comp-func clamp(none,…) parse 期重写 + shim math
函数形透传（calc/min/max/clamp 排除保 `_zwCComp` 折叠口径），values face
3,228→**3,553** 净 +325 零回归（invalid 桶 208+serialize 桶 ~117），五
套件 Pass/Timeout 双锚定零漂移，见 evidence
2026-11-12-dc2-g23-minmax-serialize-write-gate）
+attr() 值支持批 + CSSOM cleanup null 臂（g24：css-values-5 §attr-notations
parse/型别/替换全链 + 预级联全源替换 + §attr security 污染丢声明 +
cascade 文法门 + fallback 维度型拒 numberish；**根因修复 shim sel/handle 臂
setProperty(prop, null)**——String(null)="null" 四字符被值校验拒致声明永移
除不掉，native 同病同修；写面门机制尝试后撤回（插桩定谳值级 mismatch 须
accept-then-drop，cleanup 修复后两语义收敛）；五面 attr-all-types 103F→19F
净 +84 零回归/attr-cycle +7/security 23/23 维持；**values 全量 256 页对账
3,553→3,735 净 +182 零回归页**（12 页全涨——calc-dimension-serialization-
order +44/calc-serialization-002 +17 为 cleanup 共享路径红利），见 evidence
2026-11-13-dc2-g24-attr-write-cleanup）
+window.length [Replaceable] descriptor 修正（g25：shim part01.js
configurable:false→true + s400 桥测 L4 pin 同步修正——html-semantics 流
b6793f4ad 将 [Replaceable] 误读为 [LegacyUnforgeable]，spec 双源定谳（HTML
Window 接口 `[Replaceable] readonly attribute unsigned long length` +
Chromium window.idl line 63 + WPT unforgeable 钉测面恰四属性不含 length），
configurable:false 致顶层 `function length(){}` 走 ECMA-262 GDI
CanDeclareGlobalFunction step 4 抛 SyntaxError 整页崩——v14 cssom
serialize-values.html 533→0（页级崩非子测试败）；修复双面验证：s400 桥测
2 passed（L4 新 pin + 计数/bare-origin/error/移除面全保）+ release 探针
533P/164F 精确回位（697 总子测试=Chrome 分母，Chrome 同页 686/697）；语料
仅此一页声明顶层 length()，无同病面；跨流碰头信号入账，见 evidence
2026-11-13-dc3-v14-reconciliation §6）。
+trig 型别门 sin/cos/tan 放行 <angle> 实参（g26：css-values-4 §10.6
<https://drafts.csswg.org/css-values-4/#funcdef-sin>——calc_expr_math_type
型别表 Sin/Cos/Tan 仅收 MTNumber，atan2/asin 族与角度算术复合产出 MTAngle
被整式拒→声明丢弃→页面回退值伪装错算；font-property-unit-cycles 95F 全败
归因走双探针切面（plain calc 全对/字面角过/凡复合全丢——失败值全为页面
CSS/默认值回退非错算），修复后原页 0→5P（font-size 五腿 30/24/24/15/30
精确回位，css-properties-values-api −5 闭环）+ values 全量 256 页
3,735→3,933 逐页 0 回归（本批可归因 +4：atan2-computed +3/hypot-pow-sqrt
+1；line-break-ch-unit +194 为 g24 基线伪红修正——NO-GATE 二进制 A/B
定谳，页内零 trig，「line-break-ch 194F」独立桶实际已闭）；单测
css-parser 3,184/style-system 2,528 全绿，fmt/clippy 净；残余挂账：font-*
数值槽 calc 接入（30 腿）/registered custom property typed compute
（60 腿）/rotate 槽裸 math 函数与 serialize ctx（25F），见 evidence
2026-11-13-trig-typegate-font-unit-cycles）。
+counter-increment 文法门（g27：css-lists-3 §counter-increment 写面五件套
——parse_counter_list_with_sibling math 分支名 token 守卫（函数形必含括号、
custom-ident 不含，`myCounter otherCounter -8` 名后跟名形此前整拒）+
parse_counter_list_grammar 文法门（仅文法裁决不求值——supports/写面无容器
ctx，eval 延后 apply 站）+ decl_supported/validate_decl_value 白名单接入
（分派臂置未实现放行门之前防 `reversed(` 误放；转义 ident 保留放行）+
fold_decl_specified 隐式整数位补 1 canonical 串化（math 形原文保留恒等）+
is_counter_name default 保留排除；invalid 13F→13P/valid 2F→2P/computed
7→8P，li 页与 calc-sibling-function 钉面逐子测一致零漂移，css-lists −2 桶
闭账 +16P；单测 3,187/2,529 全绿；残余挂账：counter eval 容器参数腿
（apply_advanced 签名无容器 ctx——`sign(2cqw - 10px)` computed 2F），见
evidence 2026-11-13-counter-increment-grammar-gate）。
+text-size-adjust <percentage> 槽批（g28：css-size-adjust-1 §text-size-adjust
——tree_counting_slot_valid MTLength 臂收 % 槽（% 叶经 LengthPercentage 归
MTLength，`calc(10% * sibling-index())` 接受腿）+ fold_decl_specified 尺寸组
接线（`calc(10% + 5%)` → `calc(15%)` canonical 折叠，plain/sibling 形预验
零漂移）+ validate_decl_value 白名单 15th 写面门（`auto|none|<percentage>`
负 % invalid——invalid 页 4 形新收口）；valid 5→7P 两腿回位、invalid 4P，
size-adjust −1 桶闭环——四小回归全处置完毕；残挂账：gCS 通道无
text-size-adjust computed 串化臂（computed 6F + interpolation 1F），见
evidence 2026-11-13-text-size-adjust-percentage-slot）。
+font-* 数值槽 calc 接入批（g29：font-weight/font-size-adjust/font-stretch 三槽
math 形接入 + parent_font_size 父级字体基准（css-properties-values-api
#dependency-cycles-via-relative-units）+ **font-width 级联槽位分裂定谳修复**
——font-width 与 font-stretch 各占独立级联槽位写同字段 `style.font_stretch`，
继承填充循环把 font-stretch 判「未声明」用父值覆盖（font-width-computed 全腿
回初始值实证）；canonical_property_name 收 font-width→font-stretch + 填充循环
别名跳过（注册保留供 CSS.supports 尾门）。css-parser 侧：font-weight 连续
`<number [1,1000]>`、parse_font_stretch 0% valid、fold_decl_specified 三槽
接线；apply 侧：% 保持型 eval + Px 叶 Add/Sub 锚语义窄门 + calc 负 %
解析期不 clamp/computed 钳 0。css-fonts 全量 2,375→**2,411P** 净 +36
（font-width-invalid 8P 全页清零、font-width-computed 12→28P、
font-weight-invalid 2P 全清）；unit-cycles 单页 5→**20P**（G1 直置 35 腿
全绿）；单测 3,188/2,531 全绿；残余挂账：sign(20cqw) 容器实参+乘法重排
串化 6F、bolder/lighter 相对权重表 42F、font-size-adjust-invalid 写面门
57F、size-adjust 串化 14F、G2/G3 typed compute 75F、inline decl alias 腿
1F，见 evidence 2026-11-14-font-slot-calc-parent-basis）。
+font-size-adjust 写面文法门批（g30：css-fonts-5 §font-size-adjust-prop
——VALIDATED_PROPERTIES 19th + 文法门（整值 math 前置放行 / metric 前缀
空白剥离 + basis 复检 from-font|非负数|math，不进 calc 内部切分；inf/nan
is_finite 门拒；invalid 57 形——apply 链 parse splitn 结构虽拒，shim
CSSOM store 不经 apply 回显原文即 Fail）+ fold_decl_specified 指定值
串化臂（初始 metric ex-height 剥离、cap-height 等 metric 保持 + basis
math canonical 折叠，sign 树形保留原文）；invalid 0P/57F→**57P/0F 全页
清零**、valid 14→18P/1F（残 sign 乘法重排 1F）、computed 10P/10F 不变；
css-fonts 全量 2,411→**2,472P** 净 +61 零回归、unit-cycles 20P 零漂移；
单测 3,188/2,532 全绿；残挂账：computed 通道 from-font 解析刀（真实字体
度量 0.5 vs 相对单位合成比 0.8 口径分离）+ calc 负数 computed 钳 0 +
sign 容器实参，见 evidence 2026-11-14-font-size-adjust-write-gate；
animations 插值通道 3+ 页（font-size-adjust/stretch-interpolation 等
1F/页）并入桶 5 挂账）。同轮双 session 竞态记：对端 session 曾以
「g29 遗留认领」入口写重复 g30 条目并暂存在途件，本 session 收敛取
实现方账（evidence 同名文件），重复段已摘除——先手提交去碰撞。
+font-weight 相对权重表 css-fonts-4 边界（g31：resolve_relative_font_weight
旧 css-fonts-3 400/600/800 边界整表更替为 §5.2 #relative-weights 表
——bolder 350/550/900（≥900 保持原值）、lighter 100/550/750（<100 保持）。
font-weight-computed 48→**56** +8P，css-fonts 全量 2,472→**2,480P** 净 +8
零回归，单测 2,533 全绿。lighter-bolder 页 22F 定性转桶：页子元素经
cloneNode 克隆，gCS fontWeight 空串（探针四变异钉面——克隆节点样式合成
缺口，shim/桥面非 css face）；font-weight-matching 14F 转挂 @font-face
描述符匹配算法桶（g32 归因定谳：ExtractedFontFace 已带 weight 描述符、
face 请求已带 weightNum 注册，缺口在运行时字体键选择——font_load_bridge
R2417 按 ≥600 二值粗体键塌缩，无 face 级 css-fonts-4 §5.4 匹配
（font-weight-matching 页期望表 420→450 face 的 [400,500] 向上区间查找
钉面）；架构面批：faces 注册表 + §5.4 desired→face 选择接入
text_shaping/font_resolution，跨 3 crate）；installed-fonts 6F 宿主测试字体环境依赖定性不计缺口，
见 evidence 2026-11-14-font-weight-relative-table）。
+lighter/bolder JS plain 通道臂（g31 续：克隆子树 gCS host face miss
救回——lighter-bolder 页 22 腿 gCS 返空串，五轮探针切面定谳 cloneNode
批量物化的克隆子件不入 HANDLE_VIEW_CACHE.nodes 表、无 selector 可走
正置表回落，host 视图对该子树 face 盲；part01c `__zwPlainComputed`
新增 font-weight 臂（继承走查 + zwFwResolved 父链递归查 css-fonts-4
§5.2 表，invalid 声明视同无声明上行，css-wide 族回落 400，序列化对齐
宿主 font_weight_str）+ part01 gCS 直取形 plain 回落白名单补
font-weight 第 7 项（与 getPropertyValue 通用回落臂对齐，两臂对称）；
lighter-bolder **0→22P 全页清零**，css-fonts 2,480→**2,502P**，见
evidence 2026-11-14-bolder-lighter-relative-weight）。
+attr() font 槽写面拒收 + var 实参 deferred 形修复批（css-values 对
trigfix **−13** 回归全案：**对称失败真相**——trigfix 时代
parse_font_weight 域 [100,900] 百倍数使语料 attr helper expected 侧
同样不可解析、双侧 400 相等即过，g29 连续域让 expected 真出值打破
对称暴露 elem 侧缺陷；**洞 A** apply.rs font-*/font-size-adjust 写面
分支豁免集补 value_has_substitution_function（attr()/var() 替换文本
放行 store、computed 期重裁，此前只放行 math 形 → setProperty 整拒）
+ **洞 B** `attr(var(--x))` deferred 形预级联站当 IACVT 丢整条声明、
cascade.rs 所称 4.75 步重裁站从未建成——attr.rs
substitute_attr_values_deferring 变体（deferred 原文保留 + 实参部数
档案）+ lib.rs 4.75 attr 重裁站（resolve_var_in_cascaded 后重扫，invalid
/IACVT 丢声明、url 污染同 attr-security 口径）+ **结构冻结刀**
（css-values-5：attr() 实参部数 parse 期定型，var 代入只填槽不新增
实参分隔——AttrCall.arg_parts + substitute_attr_values_readjudicate
按档案冻结；首版全量重评使 argument-grammar sub 8 翻红定谳补刀）。
css-values 全量 3,920→**3,935**（+15 页级零回归：attr-all-types
150→159、argument-grammar 8→14 唯余预存 sub 9；对 trigfix 3,933 净
**+2** 反超）+ css-color 10,972 恒一 + css-fonts/四小套件页级零漂移
（transitions 200/animations 238/transforms 229/easing 51）；单测
css-parser 3,190/style-system 2,533 全绿；残挂账：argument-grammar
sub 9（custom property 站 attr 语义）、attr-all-types 18F（<color>
型别/type(*) 嵌套/non-existent 族），见 evidence
2026-11-14-attr-font-write-gate-deferred；运行事故记：target-disk-guard
三级清库撞在飞对账链（runner 二进制中途消失），三大件账完整落盘幸免，
四小套件经 zw-g31-small4 补跑零漂移闭账——重型 make 入口与在跑链
错峰纪律强化）。
+system-color-consistency 27F 全清批（g32：UA 表单系统色关键词化 S1' 值自洽
端到端 + 写面门替换函数豁免补刀）——原 triage「关键词身份回声」假设经探针
证伪（页为真值自洽比对：控件 gCS 读数 == inline 系统色计算值，scheme-aware
双轮），实修三面：engine computed_style.rs border-color 简写聚合臂（四边独立
currentcolor 解析 + CSSOM 最短形去重，此前读数缺位返 ''→ falsy 回落字面假象）
+ css-parser color.rs dark 面板扩键（buttonface/threedface/buttonhighlight→43/
buttontext·buttonshadow→白/buttonborder·threed 族→117/field→32,33,36/
fieldtext→白，与 shim _zwCCSystemDark 对齐，light 面板不变）+ style-system
R1396 UA 默认关键词化（button→ButtonText/ButtonFace/ButtonBorder；input 按钮
族 type=button|submit|reset|color→ButtonFace/ButtonBorder；文本族与
select/textarea→Field；border-color longhand 后置覆写 border 简写色槽）。
**solo 35P/27F→62P/0F 全页清零**；css-color 全量 **10,972→10,999 净 +27 逐页
零回归**，values 3,935 精确持平/cssom 1,968 逐页字节级一致/四小 Pass 级零漂移；
typed-om 首轮 −2 三页漂移全归因 g27/g28 写面门缺替换函数豁免（g24 洞 A 同型）
——**补刀 apply.rs counter-increment/text-size-adjust 两门头部
value_has_substitution_function 豁免**（typed-om 9,870→9,872 对 v14 基线恰归零：
font-stretch 0% g29 红利 +1 抵 −3.14% 负 % typed-value 两层校验残挂 −1；
css-lists g27/css-size-adjust g28 收口态逐页精确一致）。make test 3932P/7F
（7F 在册组）/clippy 零告警/fmt clean/product-smoke EXIT=0（welcome 双宽
struct PASS）+ product-smoke-legacy 42 fixture struct 0 失败（37-form-controls
PASS）；UA 关键词化可见像素漂移为有意 spec 对齐（button bg 239→240/text
8→0/border 118→128、input 按钮族 bg 白→240、dark 轮全面板化）；make reftest
725 项 0 失败；探针页
zz-probe-syscolor/zz-probe-forensic/zz-probe-none 已清出数据目录，见 evidence
2026-11-14-system-color-ua-keyword。
+progress() 三实参数学函数全链批（g33：CSS Values 5 §progress parse/型别/
求值/串化/写面门，13 文件 +501/-27；g32 HEAD 无 CalcExpr::Progress，progress
全链为本批新增）——核心实现 CalcExpr::Progress 变体 + try_parse_progress
（三实参顶层逗号 + no-clamp ident 边界 + 同族门 + 逐实参角度家族一致性门）
+ eval_progress 三段（全 eval → % 族 parent_length=100 符号基准重试 +
**V 不可求值时 S==E 部分求值臂**：clamped 恒 0 不依赖 V/no-clamp 按 V 比较
或 None，NaN 毒化部分臂同生效）+ specified 串化（可求值塌缩 calc(<number>)、
不可求值 canonical 保形含 no-clamp/±∞）；写面/门禁五修：serialize_math.rs
角度 provenance TRIG 表补 progress( + apply.rs strip_trig_argument_spans 同
补 + part11-color.js calc 包裹形保形透传（_zwCJoinRaw id 粘连存文畸形交宿主
fold 正规管道）+ apply.rs math_value_paren_unbalanced 字节级括号配平门入
通用 math None 臂（未闭合形双豁免误放行封堵）；配套 rotate3d 四臂
（parse_transform.rs：x/y/z 分量 f64::parse→parse_transform_number + 角度槽
parse_angle→parse_angle_or_math + eval_calc→eval_calc_with_default_font +
分词器 split_transform_value_args depth-0 修复（盲拆撕碎函数实参表，改括号
深度感知、逗号段判空语义保旧），R5192 口径）+ % 折叠屏蔽
expr_has_percentage_leaf_outside_progress +
mask_tree_counting/expr_contains_mix/computed/converter Progress 递归臂 +
part01.js gCS opacity 抢答门 regex；单测 progress_function_tests.rs 9 钉 +
apply.rs 文法槽族门断言集更新。**values face 对 g32 提交基线净 +74、250 页
逐页零回归**（python 逐页 diff REGRESSED 0）：progress-serialize 13→54P
（+41）、progress-computed **0→27P**（+27）、progress-invalid 33→35P 全页
清零（+2）、E 门 collateral clamp-integer-invalid +2/hypot-pow-sqrt-invalid
+2；探针 24 断言全绿（写面 7 + computed 3 + rotate3d 析因 8 + 复验 6，
rotate3d 真因 = transform 文法位裸 f64::parse/纯 parse_angle/eval_calc 无
字体基准/分词器撕参四洞，探针 12/12 全绿）；残桶挂账：
serialize 13F = S1 scale→matrix computed 串化缺口、computed 9F = lh 字体
度量族 6F + margin %·px converter 1F + cqw 宽 used 解析 1F + rotate3d
案 1F（**定谳上游陈旧期望不追**：css-values-5 Issue 11825 后默认 clamp
[0,1]，(21em−1rem)/(11em−1rem) 分子−分母 = 10e > 0 恒 ratio>1 → 规范值 1；
语料期望 2 与同页 line 35 '1'/serialize 页 matrix(0.5) clamp 钉面自相矛盾，
上游 master 逐字节一致非本地过期）；flaky 观察 1 案；对账首轮 values
4,021（含探针 12P，早于精修/分词器二进制）+ color 10,999 +
transforms 229 + motion 105/391 + typed-om 9,872 + cssom 1,968 六锚持平
（均首轮二进制），终值以终跑 + 补套件电池为准；make test **3,935P/7F**（在册
7F 零新增）/clippy 零告警/rustfmt clean（分词器修复后复验），见
evidence 2026-11-14-progress-function-g33。
**DC-2 g34 终账批完成（11-14，干净 HEAD 单二进制，仅文档）**：g33 验证账
遗留「终值以终跑 + 补套件电池为准」兑现——前轮 g34 链 /tmp 日志随会话结束
丢失，本链 systemd service 全量重录。values 全量 256 案 **3,935 → 4,009 净
+74 零回归**（python 逐页 diff REGRESSED=0，GAINED 恰五页与 g33 终跑账同构
——serialize +41/computed +27/invalid +2/clamp +2/hypot +2，g33 终值 4,009P
定谳为提交态真值）；电池六锚精确持平（color 10,999/typed-om 9,872/cssom
1,968/motion 105/easing 51 字节级一致 + transitions Pass+Timeout 双锚定），
transforms **229→231 +2 零回归**（scale-parsing-computed 23→24/
transform-with-sign-function 2→3，g33 rotate3d 四臂+分词器 depth-0 修复
红利），transitions 200→**201**/animations 238→**239**（各 +1 真实增益：
基线轮 Timeout 子测本轮完整枚举后真过，非伪影；F 总量差为在册超时截断
伪影族口径）。[证据](evidence/2026-11-14-g34-final-accounting.md)
**DC-2 g35 anchor 批完成（11-14）**：position-area wm/dir 映射 + §4.1 对齐
面——css-parser `AreaWmDir` 双坐标系单实现（CB 写作模式参照 plain logical、
self-* 参照 anchored 自身）+ layout-engine anchor_pos 区域 pass 接线与区域
默认表/显式 align/justify-self 折算/span-all→anchor-center（margin-box 口
径，显式尺寸只定位不覆写）；14 单测全绿。三轮 corpus 对账：triage 基线
11,869P/967F → AB1（inset-IMCB 面）11,805P 净 −64（in-position-try 64P 等
值页破：fallback 门跳过 #target 保持 left:200 而 #ref 介入对齐落点分叉，
单独建模 inset 必破等值语义）→ 撤除后 AB2 **11,870P/966F/4T 净 +1、case
级逐页 REGRESSED=0**（anchor-in-anchor-positioned +1 真实增益）。目标桶定
谳：position-area 族 236F 中 **~190F 为 CSSOM `style.positionArea` 声明层
缺口**（runner 未注册该属性，元素未被区域定位拦截，布局面无法翻绿，另
刀），余 ~46F 分桶挂账（scrolling 23/getComputedStyle inset 使用值 4/
@container anchored(fallback) 4/last-successful 3 等）。[证据](evidence/2026-11-14-g35-anchor-position-area.md)
**（g36 证伪更正，11-14）**：上根因表述错误——写面早已通（shim _styleProxy
链 position-area normalize+表外放行），真缺口 = s423 同 turn 读陈旧（写后
同 turn offsetWidth/gBCR 断言读到写前几何快照）；writing-modes-001 基线
1296P 全为陈旧读碰巧相等的**伪通过**（drain-split 探针实证），~190F 实为
渲染器真实缺口+读陈旧混合；定谳与 border 坐标系修复见
[evidence 2026-11-14-g36-border-coord-s423](evidence/2026-11-14-g36-border-coord-s423.md)。

**DC-1 完成**：四焦点目录本地重跑复现快照（typed-om 25 / color 1,293 逐位
一致），缺口三分类入账——运行面断言失败 ~139.5k（Chrome 同面过 95%）、
未接语料 ~8.1k+缺文件 ~10.5k、通道外形态余量。
**DC-2 首批完成**：Typed OM API 入口面（part10 ~1,810 行：CSSStyleValue
类族 + CSS 工厂 + StylePropertyMap + attribute/computed/rule styleMap +
225 属性语法表；part06 styleMap 挂钩）——css-typed-om 簇全量重跑
**Pass 25 → 9830**（365 页，65 页零失败），残余 2,383 归四桶（简写长槽
缺面/computed 颜色管线/computed transform/`.any.js` 通道）。
**DC-2 次批完成**：css-color declared/computed 通路（part11 色值管线接线 +
part06 null 即拒 + part01 computed `has` 陷阱/宿主抢答覆盖）——css-color 簇
62 页 **Pass 1,293 → 7,229**（+5,936），逐页不降核对 33 增/28 平/1 回归
（relative-color -14 记账，次批后半恢复）；T48 五钉 Blink 对齐翻正。
**DC-2 次批后半完成**：computed 真色彩数学管线（part11-computed-color.js
1,871 行接线：13 空间全精度矩阵/relative color 解析/color-mix 混合/none
策略/序列化）+ part03 raw 影子/part01c raw 读取分流 + NeedsPlain 关键字
排除——css-color 簇 63 页 **Pass 7,229 → 9,318**（+2,089），逐页不降
**9 增/54 平/0 回归**（前轮 -14 豁免页恢复持平，computed-relative 0→1,100、
computed-mix 0→930）。
**DC-2 再批完成**：declared 声明面归一（part11-computed-color.js 覆盖段
+527/-17：relative origin 折叠/color-mix method 与权重规范化/calc 规范形/
rest alpha 行为表/浅面基座委托）——css-color 簇 63 页 **Pass 9,318 →
10,530**（+1,212），逐页不降 **2 增/61 平/0 回归**（contrast 透传修复后
恢复）。
**DC-2 四批完成**：语法面收批（part11-computed-color.js +205/-3：relative
通道表槽类型校验/color-mix hue 方法空间门/alpha() 声明面/color-layers
声明面与 computed 入口）——css-color 簇 63 页 **Pass 10,530 → 10,756**
（+226），逐页不降 **5 增/58 平/0 回归**（cc7 校验过严 -20 回归修正后）。
残余 254 归三桶：computed 校验/序列化残余（relative 69、rgb 21 等）、
light-dark 32、system-color 55。
**DC-2 语料扩面批完成**：DC-1 缺语料桶 13 目录一次接线（41-sh DIRS 扩面 +
runner 13 wrapper/dispatch 臂/CLI 子命令 + corpus 脚本显式映射 + Makefile
守卫 target + ci-shards css-v2-face-d 分片，提交 0d66d7997），全量 missing=0
（pin 315976933，blob sha 逐文件复核）。基线记录轮 13 corpus **8,562 子测试
/ Pass 1,381**（transitions 185 / om-view 545 / geometry 209 / animations 225
等），失败分桶全为引擎能力缺口（startViewTransition / registerProperty /
WAAPI / 过渡插值 / 布局 overflow 几何），通道本身零缺陷账。
**卡点**：无阻塞。v14 对账完成（§里程碑表）：grid 精确一致、values/color/
anchor 达阈值；收尾挂账——typed-om `.any.js` 通道**已修**（fetch-b
GOAL_PULL_ANY_JS=1，待周跑显影）+ 四小回归：properties-values-api −5
**已由 g26 修复**（trig 型别门，5P 真实回位）；writing-modes −4（本地
13P+1T 反超 shard，瞬态/环境待定性）/css-lists −2 **已由 g27 修复**
（counter-increment 文法门，invalid 13F→13P/valid 2F→2P/computed +1 同页
对账，套件 +16P；残 2F 为 counter eval 容器参数腿挂账）/size-adjust −1
**已由 g28 修复**（text-size-adjust % 槽，valid +2P 两腿回位）——四小回归
全处置完毕。text-size-adjust-valid
2F：`calc(10% + 5%)` canonical 折叠 + `calc(10% * sibling-index())` 接受）
仍挂账；cssom −533 已由 g25 修复（下轮周跑回显）。g23 minmax/clamp serialize+写面门已落库（values 净
+325 五套件 ±0，invalid 桶清账）。g19 sibling 求值桥已落库（CalcContext 注入 + gCS 三通道
+ 写面门两洞修复；probe family 31P→56P 净 +25、values 净 +40 五套件 ±0），
tree-counting 家族残 89F 分桶挂账（color 实参槽 2F/伪元素 Face 3 2F/keyframe
10 页/invalidation/container/descriptors/media/shadow/tree-scoped/gradient/
trig）。random() 串化（g20 serialize 42F→3F，残 color 通道嵌套 D 桶）与
播种求值全链已落（g21：random-computed 141F→46F、random-item-computed
28F→23F）+ 写面文法门已落（g22：item-invalid 14F 清零）；random 残桶
分账挂账（跨元素 seed 一致性 ~14F/SVG 属性族 supports 8F/shim gCS raw
回显与 color 黑值 ~25F/@规则 通道 ~17F/unresolvable IACVT 5F，见 g21
evidence §5）。
**下一步**：g35 anchor 批已完成——~~后续刀 CSSOM `style.positionArea` 声明面
支持~~（**g36 证伪**：声明写面早已通，真缺口 = s423 同 turn 读陈旧 + border
坐标系错位——s423 门控收窄 + anchor_pos V 系统一修复已落地，anchor 语料
A/B 见 evidence g36；writing-modes-001 等伪通过桶揭开渲染器真缺口挂账另
刀）；v14 收尾两项——typed-om `.any.js` 周跑通道**已修**（fetch-b
Makefile 目标 GOAL_PULL_ANY_JS=1：lib.sh 默认排除 .any.js 致 CI 语料缺
numeric-objects 11 案；本地探针 9,872 精确对账 + 端到端 fetch 回补实证，
下轮周跑验 9,872 显影，见 evidence §9）+ 四小回归余账（properties-values-api
−5 已闭；writing-modes −4 瞬态定性/lists −2 已由 g27 闭账/size-adjust −1
已由 g28 闭账——四小全处置完毕）；g26 trig 型别门残余
三桶（~~font-* 数值槽 calc 接入 30 腿~~ **已由 g29 落地**——三槽 math 形 +
父级字体基准 + font-width 槽位分裂修复，css-fonts 净 +36/unit-cycles 5→20P，
残 sign(20cqw) 容器实参/~~bolder-lighter 权重表~~（**已由 g31+g31续
全闭账**——Rust 表 computed +8P + JS plain 通道臂 lighter-bolder
22F→22P 全页清零；matching 14F 转字体匹配桶、installed-fonts 6F 环境定性）/~~size-adjust invalid 写面
门~~（**已由 g30 闭账** 57P）等七桶见 evidence
2026-11-14-font-slot-calc-parent-basis §5/registered custom
property typed compute 60 腿/rotate 槽裸 math 函数 + serialize ctx 25F +
atan2 percentage 实参 eval）；之后组 F 布局桶（刀1 offset CSSOM + 刀2
position-area + 刀3 anchor()
inset 求值已落，anchor 语料 11,863P）：anchor 语料残余 953F 分桶——
position-area wm/dir + align/justify ~114F（非 htb 逻辑轴映射）、anchor()
复杂形/动态 ~98F（anchor-name-003/anchor-position-004/anchor-inside-outside/
anchor-name-mutation）、anchor-size() ~50F、position-area-basic 47F、fallback
布局 ~43F（溢出检查 + last-successful 状态机 + @position-try 应用引擎）、
inline 首片段 10F + 滚移感知 ~16F（刀3 双门撤条件）、apply.rs 数学函数残余
transform matrix 6 腿（并入 matrix 化桶）；values 残余桶（g23 后）：∞ 桶
（calc-infinity-nan-computed 残 ~31F：rotate 14 → matrix 化桶 + %-CB 形 +
unbalanced 形）、serialize 散形残余（~~resolution 29 主体未动（image-set
parse 前置刀，deferred）~~ **已由 g40 闭账**——29/29 全清 + image-set-parsing
红利 +6；残 length % 叶 ~2/angle ~8/number 嵌套 mix ~3）、signs-abs computed
残 ~94F + invalid 残 ~9F（g23 invalid 门已收 43）——`abs()` 空参文法、
font-weight 非白名单属性写面/serialize 4F、invalid 门表外槽族放行
（FIXME 逐 parser 族扩面）、
%-mixing CalcContext 族（calc-mix 主刀后残余：serialize 62F——sibling-index() 形已收（g19 DOM 求值桥）+ 外层 calc() 嵌套 % 排序 ~12F 树递归 serialize + computed 串面散形；computed 28F——% calc 保留形 length_to_css 空串等）、
tree-counting 家族残 89F（g19 后：keyframe 10 页、invalidation/container-query/
descriptors/media-query/shadow-dom/tree-scoped/gradient-gcs/trig 各页 + color
实参槽 2F + 伪元素 Face 3 2F——见 evidence g19 §5 分桶）、
独立桶 line-break-ch
194F（**已闭**——g24 基线该页伪红，NO-GATE 二进制 A/B 定谳 194P，
非真实缺口，见 g26 evidence §6）/ random 族残 ~86F（g22 后，见 g22 evidence §5）/ attr 残 49F（g24 后：
all-types 19 + cycle 15 + css-wide 14 + IACVT 1，attr+var 延迟组合/EOF 括号/
css-wide fallback 等未实现特征，见 g24 evidence §残余）/ viewport-units 84F / calc-size 62F /
progress-serialize 55F / position-computed 55F；其余散页；anchor-ident-function 求值面 12F
（`ident()` 串接需 calc/sign 求值管线，parse 接受面已通）；**DC-3 v14 周跑
完成对账**（run 43240558696，11-12 14:45Z 起，HEAD 74c4f245f 含 color/values/
grid/anchor/g13-g23 各批增量；产物 coverage-2026-11-12：css 60,545/196,806
净 +4,293；grid 精确一致、typed-om −43 通道缺口、cssom −533 已修复见 g25、
四小回归 −12 待归因；见 evidence 2026-11-13-dc3-v14-reconciliation）；
color 挂账桶（var-origin/pow 裸函数形/嵌套 oklab 真逆/hsl 嵌套 1F/
out-of-gamut 20F/opacity calc 20F/mix-out-of-gamut 2F 已清——组E 批；
余长尾 ~39F，系统色 consistency 27F **已由 g32 闭账**）；light-dark/system-color
建模残余（consistency 页闭、support/compute 页 dark FieldText 残挂账）；
grid G2b（taffy detailed_layout_info，架构面）。

**DC-2 g21 批完成（11-12，random 播种求值全链 Face 2）**：CSS Values 5
§9.4 Evaluating random()/random-item()——css-parser 求值器（`RandomEvalContext`
doc/element 双 seed sha256 PRNG + SENTINEL 支持性哨兵 + min/max/step 网格
公式（epsilon 顶格/reversed clamp/NaN±∞ 直通/step≤0 忽略）+ % 叶
`random(fixed <R>,…)` fixed-echo 保留形 + random-item 顶层逗号切分整体
替换）；style-system compute 期步骤 4.7 元素级原位替换（全数值改写级联
文本下游 apply 消费、保留形入 `ComputedStyle::random_computed` gCS 直读
表）+ `stylesheet_has_random` prescan 禁 style_cache（g19 同型——StyleKey
同键兄弟共享而 seed 含元素标识，§9.4 sharing 毒化）+ `canon_random_inputs`
简写前以原声明属性名 canonical 化（longhand 视图展开破 margin-top/bottom
同 key 同值语义）+ cascade 写面/engine decl_supported 哨兵替换裁决。
探针两轮洞修复（gCS cache-hit 位漏 + 简写拆分后 longhand 名分键）
16P/169F→**116P/69F**；六套件权威账 values **3,111 → 3,214 净 +103 逐页
零回归**（random-computed +95/random-item-computed +5/item-nested +2/
unresolvable-argument +1），五套件 Pass 级零漂移（transitions F 计数差为
基线轮页面超时截断伪影——inherit-001/002 各 1 Timeout 未枚举 vs 全量
枚举 560F+1T，Pass 均 0、timeout 21=21）。单测 css-parser 3,172 全绿
（random 求值组 13 案）/style-system 2,526 全绿（+1 inline random-item
管线重现）/engine 3,909 过（7 失败在册既有组——editing 流近期修复后
10→7，逐名核对）/rustfmt clean/定向 clippy 零告警。
[证据](evidence/2026-11-12-dc2-g21-random-eval-face2.md)

**DC-2 g24 批完成（11-13，attr() 值支持 + CSSOM cleanup null 臂根因）**：
CSS Values 5 §attr-notations 全链——css-parser `values/attr.rs` 新增（调用
扫描文法裁决含乘数紧贴钉 174/union/乘数形、型别选择子 DefaultString/Unit/
Syntax 十六形、Angle/Time math 折叠臂、compute 期元素级文本替换 IACVT/
taint 标记、fallback 维度型拒 numberish 其余原文透传）；style-system 预级联
块全源声明替换 + §attr security 取 URL 属性污染丢声明；cascade
`is_cascade_value_valid` attr() 文法门（valid 放行延迟裁决/invalid 级联丢）。
**根因修复**：shim `_styleProxy` sel/handle 臂 `setProperty(prop, null)`
String(null)="null" 四字符被值校验拒致**声明永移除不掉**——语料
test_invalid_attr 尾部 cleanup 全体受累（in-run 捕获值全为残留）；native
`native_style_set_property_invoke` 同病同修（WebIDL nullable
`CSSOMString? = ""`）。**写面门机制（两阶段属性包协议）尝试后整体撤回**：
插桩定谳门读数准确但语义错误——值级 mismatch（文法合法）须 accept 存 raw
走 cascade IACVT-drop（unset），仅文法 invalid 才 no-op；cleanup 修复后每
测试起点干净、两语义收敛，无需写面门。**五面权威账（vs g24attr4 中期基线
零回归）**：attr-all-types 19F（+14：58/62/74/128/130-136/155/163/174，
vs g23b 累计 103F→19F 净 +84）/attr-cycle +7/security 23/23 满血维持/
css-wide 14F/IACVT 1F 持平；残余 49F 全为中期轮已有（attr+var 延迟组合/
EOF 括号恢复/DefaultString font-family computed 通道/单位型合法性/css-wide
fallback——挂账特征清单见 evidence）。单测 css-parser 3,184 全绿（attr
106）/style-system 2,528 全绿/clippy 零告警/fmt clean/pre-commit guard
PASS（1184 行零发现）/**make test 全量 424 passed 0 failed**（systemd
service 形态，Result=success）；探针链定谳 cleanup 三角（setProperty(null) 挂/
removeProperty 过/setAttribute('style','') 过→shim setProperty null 臂），
diag 页跑后删除不入库。
[证据](evidence/2026-11-13-dc2-g24-attr-write-cleanup.md)

**DC-2 g23 批完成（11-12，min/max/clamp canonical 串化 + 写面 math 门 +
clamp none 重写，四刀 A-D）**：CSS Values 4 §10.6/§10.5/§comp-func——
刀 A serialize_math.rs：min/max/clamp specified 值 canonical 保持串化
（布局依赖树不塌缩时排序保持发射：多实参保序/单参 ≡ calc() 解包/同单位
合并/-0 归一/number→%→维度 ASCII CI 排序/节点殿后/乘积系数前置，除法·
复合形态 bail 保原文零回归边界），Length 槽白名单接线 margin-top/left；
刀 B types.rs：clamp() `none` 实参 parse 期重写（clamp(none,A,B)≡min、
clamp(A,B,none)≡max、双 none ≡ 裸值兜底臂，嵌套内层先归约）+
skip_whitespace 注释等价空白（css-syntax-3 §4.4）；刀 C apply.rs：写面
math 形通用文法+槽族型别门（22 函数名单 × parse_math_function 文法裁决 ×
槽族型别表 LengthOnly/LengthPercentage/Number 三档 + 角度 token 扫描补
number 槽 provenance + Invalid 除法二分放行单位幂抵消形，豁免三臂：
替换函数/双 calc 拐角/未知函数实参），validate_decl_value 白名单前接线
三写臂共用；刀 D engine part11-color.js：shim math 函数形透传
（round/mod/rem/hypot/pow/atan2/abs/sign/trig/calc-mix——calc/min/max/
clamp 排除，防绕过 `_zwCComp` canonical 折叠臂）。**首轮全量（g23 2230）
暴露刀 D 首版 -11 color 回归**（透传含 calc 族旁路折叠），收窄名单后
g23b 复跑归零。**六套件权威账 values 3,228 → 3,553 净 +325 零回归**
（invalid 桶 208：minmax-length +48/minmax-number +42/round-mod-rem +32/
clamp-length +23/signs-abs +43/hypot-pow-sqrt +10 等；serialize 桶
~117：clamp-length-serialize +45/clamp-partial +16/minmax 四页 +17 等；
computed 桶 clamp-length +9/clamp-integer +5），css-color 10,972 零漂移
（flip=0 reg=0），transitions/animations/easing/transforms Pass/Timeout
双锚定逐位一致（transitions 1,122 条 Fail->None 为 g21 在册超时页枚举
伪影族，Timeout 页集 21=21 恒同）。单测 css-parser 3,177 全绿（+4
serialize 表）/style-system 2,527 全绿（+1 槽族门矩阵）/engine 3,909 过
（7F 在册组）/定向 clippy 零告警/fmt clean（跑后收口两处机械漂移）。
[证据](evidence/2026-11-12-dc2-g23-minmax-serialize-write-gate.md)

**DC-2 g22 批完成（11-12，random-item 写面文法门）**：CSS Values 5
§funcdef-random-item 实参文法裁决（任意替换函数 parse 期仅实参文法，
属性族型别替换后承担）——css-parser `random_item_args_valid`（括号/
花括号三层配平 + key 节缺失即非法 + 项表非空 + 逐项 `{…}` 整包裹或
裸项、裸项顶层禁 `!;]` 块内豁免）挂 `eval_random_item_call` 头部 +
style-system apply.rs `random_item_calls_valid` 写面门（引号状态机
ident 边界扫描、调用未闭合 fail-closed、逐调用转文法裁决）接线
validate_decl_value 白名单检查之前（font-family 等表外槽同收）——
native CSD/shim writeDecl/sel-handle 三写臂共用单实现。探针
random-item-invalid 0P/14F→**14P/0F 全页清零**、random-item-valid 21P
零回归、家族 7 页 ±0；六套件权威账 values **3,214 → 3,228 净 +14 逐页
零回归**，五套件 Pass 级 ±0（transitions/animations F ±1 为页面超时
截断枚举差伪影，Pass/Timeout 双锚定）。单测 css-parser 3,173 全绿
（+1 文法矩阵 38 断言）/style-system 2,526 全绿/engine 3,909 过（7F
在册既有组，本树含 peer editing bold 批零交互）/定向 clippy 零告警/
fmt clean。
[证据](evidence/2026-11-12-dc2-g22-random-item-write-grammar.md)

**DC-2 g19 批完成（11-12，sibling 求值桥/写面文法收口刀）**：CSS Values 5
§tree-counting DOM 求值桥三面切片——CalcContext 注入求值契约（sibling_
ordinal/sibling_count Option，有 ctx 出值无 ctx fail-closed）+ Part A
css-parser（calc_expr_has_tree_counting/槽族门 tree_counting_slot_valid/
counter 列表 math 变体/fold_tree_counting_calc 乘链展开+静态积掩码求值：
Angle deg provenance/Time 秒/Length 原子保维度 canonical）+ Part B
style-system（sibling_position 元素子序、apply/order/z-index/兜底臂真值、
style_cache prescan 禁用 + 每元素 cascaded 检测门控零新字段、probe (1,1)
哨兵接线 cascade/supports/CSS.supports）+ Part C engine gCS 三通道
（by-selector/cache/lazy 带 sibling_position；inset 槽 sibling 叶注入串化）。
**两轮探针**（tree-counting 家族 23 页）：probe1 31P→49P 暴露两洞——gCS
cache-hit 复用通道无 sibling 位（#test 连续查询仅首查带 ctx，left 得 ''，
「不可达」注释证伪）+ 写面门 parse-fail 放行（带参/非时槽 time 叶 shim 存
原文）→ map 附 sibling 位 O(1) 透传 + math 形 parse 失败整拒（非 math 形
放行防 content 字符串误拒）。probe2 **31P→56P 净 +25 家族零回归**：
parsing 25→45P（残 color() 实参槽 2F 挂账）、主页 0→5P（残伪元素 Face 3
2F）。values 全量 **3,032 → 3,072 净 +40 零回归**（4 页：两 tree-counting
页 +25、calc-rounds-to-integer +12 counter/order 整数槽 math 形外溢、
hypot-pow-sqrt-computed +3 嵌套函数 sibling 叶真值）；五套件对 g16 逐位
±0（color/transitions/animations/easing/transforms）。单测 css-parser
3,146 全绿/style-system 2,525+5/engine 3,903 过（10 失败 stash 基线法证
既有与本批无关）/guarded-clippy 零告警/fmt clean。
[证据](evidence/2026-11-12-dc2-g19-sibling-bridge.md)

**DC-2 g20 批完成（11-12，random canonical 串化刀 Face 1）**：CSS Values 5
§9.4.1 Simplification of `<random-key>`——parse 期指定值 canonical 化挂
`fold_decl_specified` 头部单点接线（native setter 双臂 + bridge replay
3 站共 5 写面站自动覆盖，engine 侧零改动；custom property 跳过）。
canonical 机器（values/random.rs）：`parse_key_section` 结构化 key 节
（fixed 单 token/calc 包裹 lite 求值 [0,1) 无单位门、dashed 至多一最前 ×
element 至多一 × UA 域至多一组合）+ `canonical_section` 数字节（单 token
单位归一 ms→s/turn·grad·rad→deg 余原样保往返、多 token calc-sum lite
求值器——维度×维度/跨单位和差/混除/除零拒）+ `canonical_key_text` 序归一
（auto/省略展开 `element-scoped ua-<property>-<index>`；
`property-scoped`→`ua-<property>`、`property-index-scoped`→
`ua-<property>-<index>` 替换，index = 声明内 random+random-item 1 基序
canonical 重排前文本序；显式 `--ident`/`element-scoped`/`ua-<ident>` 保留；
`fixed calc(2 / 4)`→`calc(0.5)` 钉）+ `canonicalize_random_keys` 全值扫描
（引号状态机含转义/词边界/invalid 不改写/random-item 原样透传仅计
ordinal）。validate 面经 `parse_random_args` 委派自动获得 calc-sum-lite
接受面。探针（filter=random 20 页）**136P→175P 恰中 +39 预测零回归**：
random-serialize 29P/42F→**68P/3F**（A key 展开/替换 28 + B 单位归一 4 +
C 实参 lite 折叠 3 + E 显式 ua-ident 序归一 4；残 3F color 通道嵌套
D 桶挂账），余 19 页逐位 ±0。values 全量 **3,072 → 3,111 净 +39 零回归**；
五套件对 g19 ±0。单测 css-parser 3,152 全绿（random 组 11，新增 6：
42 形矩阵/不变形/字符串与 item 免疫/ordinal/fixed 门矩阵/fold 入口）/
style-system 2,525+5/engine 3,903 过（10 失败 g19 基线法证既有）/
guarded-clippy 零告警/fmt clean。
[证据](evidence/2026-11-12-dc2-g20-random-canonical.md)

**DC-2 g14 批首账（11-12 早，calc-mix 主刀 6d3e3af16 + layout 漏臂 c1a069a1c
+ minmax 回归修复 0f1c0639d 并行流收口）**：CSS Values 5 `calc-mix()` 全链
（parse 文法门/求值公式/指定值折叠/invalid 拒收，实现面与 provenance 事故
见证据）——css-values g14 语料首账 **2,787 → 2,882 净 +95**（calc-mix
serialize +51/computed +25/invalid +12 全清、minmax-length-serialize +8）；
回归 -1（minmax-length-percent 4 行）根因 eval Min/Max filter_map 丢 %
实参，并行流同 clone 碰头同期修复（eval 全实参参与 + 本流 fold % 门叠加
入库 0f1c0639d，规则 9 碰头信号记证据），单页复核 4 行翻回。g14b 复核轮收口
（0f1c0639d 单二进制）：values 2,787 → **2,887 净 +100 零回归**（calc-mix
三页 +88 与首账一致、minmax-length-serialize +9、minmax-length-percent-
serialize +3——minmax 修复红利，-1 收复外加 4 新塌缩行）；六套件对账
color/transitions/animations/transforms/easing 逐位 ±0 零漂移。**批终净 +100。**
g14c 写面收口（同日）：§5 归类 invalid 残 6F 勘根为 shim CSSOM 写面
`validate_decl_value` 白名单缺口（非 parse/fold 层）——text-indent（§10.5
型别门 + css-text-3 hanging/each-line `&&` 组合关键字臂）与 transform-origin-z
（registry 未收 → parser 型别门直裁 + % 排除）入白名单 10→12——invalid 页
12P→**18P 全清**，css-values 全量 256 案 2,887 → **2,893 净 +6 零回归**。
[证据](evidence/2026-11-12-dc2-g14-calc-mix-batch-step.md)

**DC-2 g15 批完成（11-12，calc-mix 嵌套和串刀）**：serialize 页「外层
calc 嵌套」桶指定值 10F 收口——`fold_mix_nest_sum` 嵌套臂（BinaryOp 和差
树叶项化：标量直读/裸 % 叶系数直读/Mix 叶塌缩项化，同单位常量合并、%
项前置其余保序、单项塌缩复用）+ 纯 % 值项字面权重加权和符号解析（保 %
维度，used 值期才需基准）+ `expr_contains_mix` 路由/`filled_literal_weights`
填充表与归一化臂共用——单页 66→76P，css-values 全量 256 案 2,893 →
**2,903 净 +10 零回归**（对 g14b 口径 2,887→2,903 净 +16：g14c 文法门
同轮单二进制，分页归因 serialize +10/invalid +6）。残余：sibling-index()
桶 37F（DOM 桥兄弟序求值）、computed/used 串面桶 43F（computed 通道非
存储透传实证——length_to_css Calc 空串域 + 单项解包 + 零项丢弃 + %
解析通道）。
[证据](evidence/2026-11-12-dc2-g15-calc-mix-nest-step.md)

**DC-2 g16 批完成（11-12，tree-counting 叶/符号零刀，ed76acefb）**：
CSS Values 5 §tree-counting 接入——`CalcExpr::TreeCounting` 新叶
（sibling-index()/sibling-count() 无参文法，parse 期 opaque，求值返 None
挂 DOM 桥 FIXME）+ eval_mix **符号零**（0 权重 × 不可求值项贡献 0，语料
"0% so calc-mix() can still be resolved" 注记；NaN·0 毒化保持）+ 型别/
权重门/串行/fail-closed 槽全臂扫（编译器驱动穷举核对）。单页探针（纯本刀
二进制）serialize 76→111P/computed 25→41P/invalid 18/18 稳，孤立 +51；
css-values 全量 256 案 **2,903 → 2,917 净 +14 页级零回归**（test 级 2 例
Pass→Fail 为 g15 意外通过修正——raw 透传期初始值假通过，现收敛进
computed 串面既有桶；组合二进制含他流 computed 串化刀，已提交
9dbe26c80，serialize 组合态 88P 对探针 111P 差 -23 归其面，处处 ≥
g15）。残余：sibling 非零权重行 + tree-counting 家族
23 页 DOM 值面（DOM 桥桶）、computed 串面（他流落账后重盘）。
提交态组合权威账（results-g17，bin 9dbe26c80 两刀齐，07:06 落盘）：
2,903 → **2,953 净 +50**，serialize 111/128、computed 41/53 与本刀探针
一致；增益 8 页 +78（两刀合力 + 9db 串化刀 minmax/border-radius 面），
回归 2 页 -28（min/max/hypot %+px used 值等值面——g15 双侧空串假通过
修正，min/max used 串化臂 fail-closed 残）归 9db 串面流挂账，净账仍
+50。六套件漂移对账已补录（evidence §6）：color/transitions/animations/
easing ±0、transforms +1（9db 刀 calc 支持外溢增益）、values 累计 +66
——六套件页级零回归。
[证据](evidence/2026-11-12-dc2-g16-tree-counting-step.md)

**DC-2 g13 批完成（11-12 凌晨，三提交 dcb81aad7/cc22f08e1/3351e0577）**：
前轮双 session（knife2 dev + e1 姊妹）rate-limit 双中断，本 session 按认领
笔记分别验证、分开提交，提交态 HEAD 单二进制重录权威账（results-g13/g13b，
systemd unit zw-g13-batch-0345/zw-g13b-values-0432）。① **serialize 刀2**
（css-parser，dcb81aad7）：§10.11 常量折叠扩 Length/Time 槽——width 与
transition/animation duration/delay 四属性入 fold_decl_specified 白名单；
非有限值 §10.6 canonical 发射 `calc(NaN|±infinity * 1<unit>)` 因子归一
（树首维度叶单位，物理单位 parse 期归一 px/ms 归一 s）；clamp NaN+实参
单位分歧结构外提逐 arg 规范化；两级求值仅非有限穿透 em/％ 基准；angle 位
±∞ 由拒折改 canonical 放行（§10.10）；连带修 eval_calc_with_context clamp
臂 NaN 丢毒（f64::min/max 链洗毒，§10.6 任一实参 NaN → NaN 先行短路）。
② **通用色通道 cq/sign 面**（engine，cc22f08e1）：part11-color 导出
`__zwContainerInlineSize` 基座探针，part11-computed-color 通用通道补
cqw/cqi/cqh/em/px 单位叶（cqh FIXME → NaN fail-closed）+ sign()/abs()
函数臂（§sign-related）+ color-mix 嵌套 calc 权重括号平衡扫描 + contrast-color
声明面 `color(calc 槽)` 规范形——css-color **10,970 → 10,972 净 +2 零
回归**（mix sign 形 +1、contrast-color calc-container +1）。
③ **round B=±∞ 策略表**（css-parser，3351e0577）：clamp 毒化门落地暴露
signed-zero -5 回归（IEEE 直算 0×∞ 产 NaN，基线由 clamp 洗毒掩盖）——按
round-function/signed-zero 语料钉面实现策略表（nearest/to-zero → ±0 签号
随 A；up 正 A → +∞；down 负 A → −∞；B 符号无关），在册 FIXME types.rs:1420
兑现；g13b 复核对 g13 净 +40 零回归。**批终账（五套件，零回归）：
values 2,646 → 2,787 净 +141（9 页：serialize-length +39/serialize-time +29/
divide-by-0 +21/round-function +16/signed-zero 全页翻绿 +15/minmax-time-
serialize +11/round-mod-rem-computed +4/serialize-angle +3/serialize-number
+3）、color +2、transitions/animations/transforms/easing 逐页零漂移，
合计净 +143。**单测 css-parser 3118 全绿（+4 案：clamp 毒化/serialize
canonical 全形态表/round ∞ 表 24 断言）；style-system 2516 全绿；clippy
零告警；make test 同例推迟（g12b 先例，在册欠账 10F 未触碰）。values
残余桶更新：serialize-resolution 29F（image-set parse 前置刀，deferred）、
serialize 五页残 ~76F、signs-abs invalid 52F 不动、calc-infinity-nan-computed
~31F（rotate matrix 化桶 14 + %-CB + unbalanced）。
[证据](evidence/2026-11-12-dc2-g13-serialize-fold-color-cq-round-step.md)

**DC-2 ∞/NaN 数学求值管线刀1批完成（11-12 凌晨，g12）**：css-values-4 §10.6/§10.12
兑现（calc-infinity-nan-computed 桶）——min/max 任一实参 NaN → 结果 NaN（IEEE
`f64::min/max` 忽略 NaN 丢毒）+ computed 期 censor（style-system resolve Calc
NaN→0；engine `time_list_to_css` duration 族负值/两族 NaN 钳 0，±∞ 保留走
format_num 饱和覆盖 ≥1e6 断言）+ 写面放行（`parse_time` math 臂撤 `is_finite`
门；`parse_animation_duration` 补数学形臂兑现 g11 挂账；`<number>` 槽 ±∞ 放行 +
`has_length` 叶扫描换 `is_number` 结果型别门救 `sign(<length>)`）+ 连带型别基建
（purity 门标量乘除 `<time>×/÷<number>` 合法 §10.1；`eval_calc_with_default_font`
parse 期 em/rem 16px 基准，%/vw/vh fail-closed）——css-values 语料
**2,572 → 2,641 净 +69 零回归**（test gains=69 losses=0，10 页增：
calc-infinity-nan-computed 9→29、round-mod-rem-computed +16、signs-abs-computed
+7、serialize angle/number +7 部分自解）；transitions 对 g11b +2、animations
+2 零回归；transforms/easing pre-flip A/B（results-g12pre 独立二进制对照）
零回归——easing 字节级零差，transforms -40 集中 rotate/scale/translate-
parsing-computed 三页 = 对照臂 registry 缺臂归因（非本批），g12c 干净复跑
（registry WIP 在树、单写者 transcript）定谳 **223 → 228 净 +5 test 级零
回归**（sign(1em-1px) 形 5 案：scale-computed 21→23/scale-valid 8→9/
sign-function 0→2），signs-abs-serialize 16P/F0 全清。**批终账五套件
values +69/transitions +2/animations +2/transforms +5/easing ±0，加 registry
values 侧补账 +5（见下段与证据 g12b values 轮段），合计净 +83 零回归。**

**DC-2 registry known_properties 收尾（g12b，0a0c910df 后增量对账
results-g12b）：rotate/translate/scale 入属性清单解封 CSS.supports 前置门
（computed-testcommon 整页前置翻红根因）+ inherit initial-value 臂 + engine
part02 supports 钉——values 复跑 2,641 → **2,646 净 +5 零回归**
（calc-infinity-nan-computed scale `CSS.supports` 组 29→34），transforms
228（对照 g12 186 = registry 维度 +42，批内 sign 形 +5 已上段入账）。
单测 +6 钉（css-parser infinity_nan.rs 4/style-system
1/engine 1）；三 crate clippy 零告警；make test 同例推迟（同树他流 WIP 污染
归属，g10/g11 先例）。残余：calc-infinity-nan-computed ~31F（rotate matrix 化
桶 14 + %-CB 形 + unbalanced）、serialize 五页 ~116F（刀2）、signs-abs
invalid 52F 不动。
[证据](evidence/2026-11-12-dc2-infinity-nan-math-pipeline.md)

**DC-2 <time> 族写面文法门批完成（11-12 凌晨，g11/g11b）**：T54 写面校验白名单
扩 4 属性（transition-delay/duration、animation-delay/duration，
`VALIDATED_PROPERTIES` 6→10，apply.rs；零 apply 逻辑改动——apply_advanced 四臂
既存，白名单接入即启用 min/max/round 数学形与畸形参拒收）+ 语料跑捕获回归
同批修正（css-animations-2 §propdef-animation-duration `auto` 关键字
csswg#6530 滚动驱动：animation-duration 逐 top-level 逗号项校验、auto 项放行，
`split_top_level_commas` 提升 pub(crate) 复用；shorthand `auto, 1s` 混合形
合法）——css-values 语料 **2,516 → 2,572 净 +56 逐页零回归**（minmax-time-invalid
0P/52F→52P/0F 单页全清、percentage-without-context +4 四属性家族 sign(50%) 形
拒收；g11b 修正后复跑对 g11 逐页 byte-identical）；transitions 对 v3 陈旧基线
净 +13、animations 净 +11，页级与 test 级均零回归（g11 首跑
animation-duration-computed 5→4 回归被逐页 diff 捕获、修正后复位——白名单批
必须配基线 diff 的实证）。style-system lib 2,515P/0F（+3 钉）；clippy 零告警。
挂账：parse_animation_duration 数学函数形（白名单使其写面可见，无语料页驱动）。
[证据](evidence/2026-11-12-dc2-time-family-write-face-whitelist.md)

**DC-2 time 维度+opacity 数学臂批完成（11-11 深夜，g10）**：CSS Values §6.3
<time> 维度接入 calc 型别表（`CalcExpr::Time` 秒底 + `allow_time` 门控：共享
calc 入口保持 time-closed、time 槽专用 `parse_time_calc` 入口 + 纯度判定拒混型，
五处既有构造点全补 false）+ `<opacity>` 数学函数臂（parse_layout 活版，
§10.12 NaN censor-to-zero 先于 ±∞ fail-closed 门——censor 后置则不可达，钉住；
±∞ 挂 FIXME(types.rs:1420) ∞ 桶）——css-values 语料 **2,464 → 2,516 净 +52
逐页零回归**（净增远超批前预估 +15：time 槽解锁是全家族效应——minmax-time
computed+serialize +30、round/mod/rem time 实参形 +12、calc-time +6、sign/abs/
hypot time 形 +4，6 页增益）；css-color 10,970P 零漂移；css-parser lib
3111P/0F。g9 残余主桶「数学函数 validity 放行 ~47F」就此清账（opacity/transition
页已出失败榜）。make test 权威账 **3,881P/10F**（重跑链：fmt 门死→
`cargo fmt -p zero-engine` 机械修复独立 style 提交→systemd PATH/artifact
在案瞬态两次→全量走通；在册 9F 同形 + r307_tree_order_identity_domains
新红——探针 db74aad18 复跑红钉死 ba 流 CSSOM 两提交窗口 0a435f0a1/
0ff1bc92c，g10 豁免，修复归 rendering 流碰头账）。残余主桶 = ∞
（FIXME types.rs:1420 族）+ %-mixing（CalcContext 族）
+ time invalid 形 + 独立大桶（ch/random/calc-mix/attr/viewport）。
[证据](evidence/2026-11-11-dc2-time-dim-opacity-math.md)

**DC-2 mod/rem 除零批完成（11-11 深夜）**：css-values-4 §10.5 mod()/rem() B=0
None（无效）臂撤除改 IEEE NaN（types.rs，round 族 g8 同道）+ opacity computed
NaN censor-to-zero（§10.12，r2320 先例；语料 inert 定谳——语料 computed 读走
shim 侧、宿主 face 今日不可达，保留为 validity 放行刀负载臂）——css-values
语料 **2,450 → 2,464 净 +14 逐页零回归**（computed +12 mod/rem B=0、serialize
+2 specified `calc(NaN)` 形）；css-color 10,970P 零漂移；css-parser lib
3107P/0F；make test 3879P/9F 逐名同形在册组零新增。provenance：22:04 初步跑
混合树（构建窗内编辑）经 g9b 干净复跑 byte-identical 归因清白；ba 流在途
part11 js hunks 收纳 stash@{0} 待认领（语料中性有 color 零漂移旁证）。残余
主桶 = 数学函数 validity 放行 ~47F（opacity 32 + transition-delay 15，下批
首选）+ serialize scale-matrix 5 / computed-NaN 6（shim censor 臂）。
[证据](evidence/2026-11-11-dc2-modrem-nan-opacity-censor.md)

**DC-2 round 除零语义批完成（11-11 深夜）**：css-values-4 §round-func round
族四臂撤 `bv==0→av` 特判走 IEEE NaN（types.rs，g6b 定谳修复——特判被
0061c7fff opacity 折叠白名单翻成 declared `calc(1)` 才暴露语义分裂；mod/rem
None 臂留下一批）——css-values 语料 **2,442 → 2,450 净 +8 逐页零回归**
（round-function +2、round-mod-rem-computed +4、serialize specified +2；
漂移页恰 round 族三页、253 页逐位一致，顺带核清 f415be99c/9d243bc67 对
values 语料零影响）。残余分桶：computed 两腿（opacity NaN censor 一行缺 +
经典 transform matrix 化在册桶）、mod/rem B=0 +16（§10.5 同段 NaN，下批
首选）、B=±∞ 有符号零桶（FIXME types.rs:1420）、单参 round ×2。
[证据](evidence/2026-11-11-dc2-round-divzero-nan.md)

**DC-2 组F 布局刀3 完成（11-11 晚）**：anchor() inset 求值 + 双介入门（提交
405b12a3c + be4c89a60 fmt 补）——物理边/center/纯百分比/bare 缺省边 = f 槽
同名边 + `<length>|<percentage>` fallback，CSS2 §10.3.7 双侧拉伸定尺寸/
单侧定位；`AnchorGeo` 双 flag（inline_flow/in_scroller）经 `gated_lookup`
统一消费（显式名与 position-anchor 默认锚同口径）：① 纯 inline 流锚门
（跨行分片几何不可靠，inline-block 原子盒不受门）② 滚动上下文失配门
（引擎不建模 scroll offset，同滚上下文同基允许；撤门条件 = 滚移感知锚
几何）。style-system `anchor_inset_*` 四承载 + g1f 单测 12 案全绿。
anchor 语料 **10,549 → 11,863 净 +1,314 逐页零回归**（writing-modes-001
单页 0P/1,296F → 1,296P/0F 全清 + 八小页增益）；三轮零漂移（g4/g5 双轮
+ 2152 终轮，语料目录他流 zz 探针 2 页不计账）；对未门控中间轮净 +1
（inline-002/scroll-js-expose 按门恢复、inline-001/003 各 −1P 挂账首片段
语义下刀）。
[证据](evidence/2026-11-11-dc2-anchor-inset-gate-face.md)

**DC-2 values 刀1 完成（11-11 晚）**：CSS Values 4 §10.5 型别表 + IEEE sign/除零语义
（提交 eef2600da，纯 css-parser）——`calc_expr_math_type` 型别表（sign 恒 number /
abs·round·mod·rem·hypot 随实参族 / number·length=length 维度算术）+ sign ±0 传符号位、
÷0→±∞（§10.6）+ 一元负号字面叶直取负（-0 保符号、长度叶保族）+ hypot 变参 1+
（§10.8 单参 ≡ abs）+ serialize scale 族折叠（scale/scalex/y/z/3d）。css-values 语料
**2,218 → 2,426 净 +208、12 页增益零回归**（signed-zero +80、signs-abs-computed +45、
hypot-pow-sqrt-computed +27、minmax-number-serialize +20、serialize 桶 +30 余、
calc-infinity-nan +2）；css-color 10,948P 零回归；engine 腿 3875P/7F 既有欠账组。
残余：style-system 槽臂 invalid 门 ~70F、opacity 槽 apply.rs 38 函数集腿（4d 面）、
used-value NaN/∞ 钳制、angle 叶 provenance。
[证据](evidence/2026-11-11-dc2-values-math-type-table-face.md)

**DC-2 g6b HEAD 核验账（11-11 晚）**：纯 HEAD 二进制（g6 build）双腿对账——
css-values 语料 fd2 基线 **2,426 → 2,442 净 +16**（0061c7fff opacity 折叠腿兑现：
exp-log/minmax-number/hypot-pow-sqrt/calc-infinity-nan 四页各 +4P）；round-mod-rem
1 回归，根因 = round 族四臂 `bv==0→av` 特判（types.rs:1890-1893，af02f52bf 旧码
「round(x, 0) = x」）违反 css-values-4 §round-func——WPT 配对 `calc(NaN)` 实证
round(1,0)≡NaN，mod/rem None 臂+shim NaN→0 面故未回归；折叠白名单把 declared
折成 `calc(1)` 才暴露语义分裂。修法（values 域）：去四臂特判走 IEEE NaN，
预计连带翻绿 round specified opacity/scale 2 腿。hypot-pow-sqrt 单页 16P/9F：
**UNCLAIMED「apply.rs 九函数微批」定谳关闭**——九函数已在库且求值正确
（scale(1)/scale(0.5) 实证），简报过时；9 红腿真因两面：① opacity 写面
validity 拒裸数学函数（3 腿，parse_opacity 值校验——折叠就绪但声明未存上）
② transform computed 未 matrix 化（6 腿；css-values 同因簇 **40 红腿/5 页**：
progress-serialize 14/minmax 10/hypot-pow-sqrt 6/round-mod-rem 6/exp-log 4；
css-transforms 面 UNCLAIMED，progress() 函数本体亦未实现）。color 桶 1
（cqw 入色值数学 ~10F）根因定谳两面：shim 基座 cqw 视口×1000 近似
（part11-color.js:370）+ host `LengthValue::Calc` 求值 CalcContext
container_width 未接（computed.rs:253 `..Default::default()`，container 参数
在手未喂）——语料页容器动态宽（1000→100px sign 翻转）两面都接不住。
详见 .acceptance/wptfyi-css/triage-color-longtail-63f.md。

**DC-2 color contrast-color computed 刀完成（11-11 晚）**：css-color-5
§contrast-color computed 臂（提交 36de2a6f6，part11 parse/WCAG 选白黑/
顶层零漂移门三件）——css-color 语料 **10,948 → 10,962 净 +14 零回归**，
真收割 color-computed-contrast-color-function.html 4F→0F 全清（+4）；
+10 为 zz-caprobe 探针页首入账（本地锚非真实缺口）。残余 3F 分面挂账：
valid 页 1F + calc-container 1F 写面 canonicalization、calc-container 1F
cqw 容器宽（桶 1 同根因随收）。color 长尾桶 5 收口，剩桶 1（cqw ~10F，
根因已定谳）为主收割面。
[证据](evidence/2026-11-11-dc2-contrast-color-computed-face.md)

**DC-2 round 除零批完成（11-11 深夜）**：css-values-4 §round-func round 族
四臂撤 `bv==0→av` 特判走 IEEE NaN（types.rs，g6b 定谳修复；0061c7fff opacity
折叠白名单翻出语义分裂的 1 回归所在腿随批修正）——css-values 语料
**2,442 → 2,450 净 +8 逐页零回归**（round-function +2、round-mod-rem-computed
+4、serialize specified +2；漂移页恰 round 族三页，253 页逐位一致顺带核清
f415be99c/9d243bc67 对 values 语料零影响）。css-parser lib 3107P/0F（旧断言
`round(x,0)=x` 退役改钉 NaN + 新增四策略 B=0 全 NaN 钉）；make test 3877P/9F
与 g7b 归因账逐项一致（7F 在册 + 2F f415be99c execCommand），本批零新增。
残余分桶：mod/rem B=0 §10.5 同段 NaN（+16 预期，g9 微批编辑点已探明）、
serialize computed 两腿（opacity NaN censor 一行缺 §10.12 + 经典 transform
matrix 化在册 UNCLAIMED 桶）、B=±∞ 有符号零桶（FIXME types.rs:1420）、单参
round ×2（nearest 平局朝 +∞，Rust .round() 半值远离零不等价）。
[证据](evidence/2026-11-11-dc2-round-divzero-nan.md)

**DC-2 color cqw 容器基座刀完成（11-11 晚）**：css-conditional-5
§container-lengths 计算面落地（提交 9d243bc67，part11-color 单文件）——
`__zwComputedColorValue` 入口经 gCS 宿主 query 通道探明最近 container-type
≠ normal 祖先内联尺寸置入 `_zwCCqwBase`，sign eval cq 单位按 N·W/100 求值；
无基座（declared 面/fail-closed）保 ×1000 兜底零漂移。css-color 语料
**10,962 → 10,970 净 +8 零回归**——桶 1 主收割：lab 页 4F 全清（页账
120P/0F）+ color-function/hsl/hwb 各 +1 + rgb +1。残余挂账：calc-container
2F（part11-computed-color 通用通道 prop=background-color 不经本入口，另刀）、
color-mix 100em 案（em 元素字号面）、rgb 残 1F（桶 4 转义域）。
[证据](evidence/2026-11-11-dc2-cqw-container-base-face.md)

**DC-2 组F 布局刀2 完成（11-11 午后）**：position-area 区域定位布局 pass
（提交 5af93e7e0）——css-parser anchor_area.rs 区域格矩阵单点下沉（canonical
串→3×3 格，axis_of 显式归类 + 双 shared 词对首词=block/次词=inline + self 族
元素 rtl 镜像）+ style-system 四属性承载（anchor-name/position-anchor/
position-area/position-try-fallbacks 小写原文）+ layout-engine anchor_pos.rs
两段式 pass（锚注册收集树序后者覆盖 + 自顶向下区域定位，margin box 对齐、
auto 尺寸格满填、ZW_ANCHOR_AREA=0 kill-switch；fallback 族不介入门——语料
实证介入净 -5）。anchor 语料 **10,485 → 10,549 净 +64 逐页零回归**
（position-area-in-position-try.html 0P/64F→64P/0F 单页全清；css-color
10,905P 零漂移；e2e 单测 6 案；fmt/clippy EXIT=0）。
[证据](evidence/2026-11-11-dc2-position-area-layout-face.md)

**DC-2 color 组E 批完成（11-11 晚）**：css-color 域外保形五修（part11：
序列化全面不钳 + hsl/hwb resolve-tail s/l/w/b floor 撤除 + oklab 投影数值
真逆 + `_zwCCMixSide` powerless hue 收窄真消色）+ part01 gCS 双臂 opacity
数学形抢答门（宿主 parse_opacity 拒 calc 族恒回 '1' 的读面规避修）——
css-color 簇 **10,905 → 10,948 净 +43 逐页零回归**（opacity-computed 20F、
relative-color-out-of-gamut 20F、color-mix-out-of-gamut 2F、
color-computed-relative-color 1F 清零，61 页精确持平）；anchor 语料
**11,863P 双轮逐页零漂移**（+3 为 HEAD 布局代码兑现，inline-001/003 −1×2
归 405b12a3c 提交时点自身 delta 布局流账）；engine 腿 3875P/7F（s387 在册
时序族，三方归因非本批）、wpt-runner bin 319P/0F；fix4/5 为并行流补刀合入
（工作树协作跨流同面），clone-blind 等场环坑修入 run-fd1.sh 等待环并沉淀
learning。
[证据](evidence/2026-11-11-dc2-color-out-of-gamut-opacity-face.md)

**DC-2 组F 布局刀1 完成（11-11 午后）**：offsetParent/offsetLeft/Top CSSOM
View spec 化（提交 cf5e8d0a2）——offsetParent 从 body proxy 近似升级 spec
算法（最近 positioned / CB 建立者 transform|perspective|filter|contain /
td-th-table 祖先单遍命中；fixed→null；html/body null R4003c 保持）+
offsetLeft/Top 相对 offsetParent padding edge 换算 + round 最近整数。
anchor 语料 **10,455 → 10,485 净 +30 逐页零回归**（anchor-center 四写作
模式组合页 0→3×4、last-successful 族 0→2×5 等十六页；css-color 10,905P
零漂移；solo 探针钉无顺序依赖；bridge 钉面 part60 + part01 mock 同步）。
[证据](evidence/2026-11-11-dc2-offset-cssom-spec-face.md)

**DC-2 组F 批完成（11-11 上午）**：css-anchor-position position-area 语法面
——撞车仲裁定案 grid G1 单实现双通道（Rust 基座 13535525b 并行流落库：文法/
序列化矩阵单点 + 单测语料 oracle；本流 shim 薄委托接线：part06 decl 臂
`__zw_position_area_normalize_decl` / part01c 计算臂 `__zw_position_area_computed`
含 css-wide 计算期语义 + part01 gCS 白名单与宿主回声覆盖两臂）。anchorbase
全语料 253 案 **7460P/5356F → 9893P/2923F 净 +2433F 逐页零回归**，
position-area-parsing 1800F 与 position-area-computed 633F **双页全清**。
[证据](evidence/2026-11-11-dc2-position-area-grammar-face.md)

**DC-2 color none 批完成（11-11 上午）**：组 E 残余 168F 归因四组（rgb none
门抢答 16F / relative hsl-hwb RT 20F / lch-oklch RT 6F / mix 2F）三面修复
——part01 gCS 双臂门 raw 影子优先（NeedsPlain 判定改吃归一前原文，与
part01c `_zwPlainDecl` 同判据）、hsl/hwb srcBare % 裸形跟随（序列化器
数组化：relative 输出恒裸、direct 随原文形）、lab 族 hue echo（parse 期
fromAngleConv 旗标 + mod 360，oracle 双锚 `73.3386`/`33.690066` 兼容；
computed 面恒 8 位回显 store 串）。css-color **10,842 → 10,887（净 +45）
逐页零回归**，values 持平，typed-om +4（13535525b 在库批兑现面）。
[证据](evidence/2026-11-11-dc2-color-none-serialization-face.md)

**DC-2 color var-origin 批完成（11-11 午前）**：RCS+var 入口放行——
`__zwComputedColor` var 分支对相对色形（10 函数 `from` 语法，或纯 var 引用
替换值为 RCS）先试本管线自解（var 臂级联近似 + chanexpr calc 通道 + 现代形
序列化），非 RCS 形保基座/宿主零漂移；死点为入口守卫短路 + 宿主 Rust
`parse_relative_color` 恒 legacy 形（calc 通道拒→invalid→黑），JS 管线能力
完备但从未被触达。color-computed-relative-color **13F 翻正**（形桶 7 + calc
桶 6，嵌套 `--mygray` 链与 sibling-index 组合案含），css-color **10,887 →
10,900 净 +13 逐页零回归**，values/typed-om 持平。探针 13/13 + 零漂移控制
先证，终验三语料零回归。
[证据](evidence/2026-11-11-dc2-color-var-origin-rcs-entry.md)

**DC-2 color 嵌套真逆批完成（11-11 午后）**：嵌套 relative-out oklab/oklch→xyz
真逆直转（OkM1/OkM2 Gauss-Jordan 真逆矩阵，发表逆 OkM1i 7.6e-4 松差曾致 rgb
投影往返噪声）+ decl 裸数学函数形放行——本批由同 session 双 spawn 竞态前轮
实例开发、矩阵修正后停摆，按孤儿爆发形态清杀采纳工作树认领件重建复验：
css-color **10,900 → 10,905 净 +5 逐页零回归**（oklab/oklch 嵌套 ×d50/d65
4 案精确 0.99 翻正 + pow 裸形 1 案，computed-relative-color 6F→1F，残余
hsl 嵌套归 out-of-gamut 桶）；values 连带 +5（clamp-color-computed），
typed-om 零漂移。
[证据](evidence/2026-11-11-dc2-color-nested-relative-true-inverse.md)

**DC-2 @position-try CSSOM 批完成（11-11 午后）**：CSSPositionTryRule/
CSSPositionTryDescriptors 全面物化——css-parser `PositionTry` 变体（非法 prelude
链式 None → skip_malformed 整条 extent 消耗，堵 body 泄漏为顶层样式规则回归位）
+ css_wire wire 条目（selectorText = `@position-try <name>` 分派面 +
serialize-an-identifier 重转义）+ part06 接口壳/38 键描述符白名单/_makePtRule
单实现双通道（wire cssRules 读面与 insertRule 写面共用，grid G1 先例形态）/
getRules 物化臂 + insertRule 校验臂。anchor 语料 **9,893 → 10,182 净 +289 逐页
零回归**（读面段 at-position-try-cssom 0P/265F → 264P/1F +265 含 caching-002
连带，写面段 parse 11F 与 allowed-declarations 13F 全翻 +24；css-color 10,905P
零漂移），单测 10 案（css-parser 6 + wire 4）。
[证据](evidence/2026-11-11-dc2-position-try-cssom-face.md)

**DC-2 组F anchor 族声明面文法批完成（11-11 午后）**：anchor 族七属性文法 +
anchor()/anchor-size() 函数拒面（提交 431d229e5 + 跟进刀）——anchor_props.rs
单实现双通道（grid G1/position_area 同型；try-fallbacks ident 夹心拒/none 整体
单独/position-area 段委托复用；anchor-size 隐式空参/单参 length 形/`||` 两侧
序/unitless zero fallback/属性域 size+inset+margin 全集，语料矩阵逐案钉面）+
part06 族臂与通用拒面臂 + part03 简写拆分/合成读/删除清双长手 + part01c 族
computed 臂 + CSS.supports decl_supported 臂 + 四宿主回调；**跟进刀两修**：
part01 gCS getPropertyValue/直取形两路 plain-first 宿主回声覆盖（首跑 computed
五页 0 翻根因——host 对 registry 外未设值/设值分别回 'initial'/'inherit' 字面量
与 ''）、`anchor-size(` 子串门双形并查（首跑 parse-invalid 仅 2 翻根因——不含
`anchor(` 子串）。anchor 语料 **10,182 → 10,455 净 +273 逐页零回归**（两段跑
+171/+102，18 页清零：声明面文法桶 ~142F 全数兑现 + visibility/fallbacks/
try/order computed 五页 + order 行为三页连带；css-color 10,905P 零漂移；单测
8 组；make test 3864P/7F 零新增）。
[证据](evidence/2026-11-11-dc2-anchor-family-decl-face.md)

**DC-2 values 批完成（11-11 凌晨）**：L4 数学函数进 transform/数值槽位 +
T54 inline style 写面值校验（三写臂：native CSD / shim writeDecl / shim
sel-handle 代理——corpus 实际写臂为 sel-handle 面）——values corpus
594 页 **Pass 1,540 → 1,770**（vs 基线 1,379 净增 +391），逐页不降 **0 回归**
（oracle 收窄后 v5f 口径；v5e 曾 -12 回归已修）。invalid 族 9 页翻正
（acos 族 63 全翻）；typed-om `.any.js` 变体通道同步落地（v5b 9,872 / +43）。

**DC-2 grid 批完成（11-11 凌晨）**：CSSOM 声明面校验 + canonical 序列化 +
supported property names has 面（native 17f30543c grid_cssom.rs + shim 面
接线：`_styleProxy`/`_zwMStyleProxy` 共享 `_zwStyleHas` 臂 + grid 归一臂
宿主回调，与 grid_cssom 单实现双通道）——锚页 grid-layout-properties
**13P → 94P（+81）**（`in` 门全解锁），探针页 20P/0F；残余 46F 分桶：
computed 值发射臂缺失（grid-line 族 computed 返 initial）、stylesheet
cascade 应用缺失（reset 不回落页面规则值）、span 形状序列化。
[证据](evidence/2026-11-11-dc2-grid-cssom-decl-face.md)

**DC-2 values serialize 批完成（11-11 凌晨）**：specified 值读面常量折叠
（trig/inverse-trig/minmax/clamp/round-mod-rem/hypot + canonical 序列化）三写臂
接线 + computed 值臂 NaN 规约（CSS Values 4 §10.12 censor-to-zero / §10.13
`calc(NaN)`）——values corpus 对重录基线（results-v5 v5d，oracle 收窄）
**1,548 → 2,213（净 +665），逐页不降 0 回归**（对上批账面 1,770 净 +443；
sin-cos-tan-serialize 40→270 全绿、acos 两页 0→62/60、minmax 族 +170、
round-mod-rem-computed +54）。**DC-2 color 上下文批完成**（提交 d1dd30e7a，三修合并）：currentcolor 三参
属性上下文（`__zwComputedColor(raw, el, prop)`，非 color 属性先试本元素，
css-color-4 §5.3）——currentColor 子测试 +26（relative-color +21、alpha +5）；
part03 raw 影子放宽（归一变形且原值含 none 即存原值）+ srcBare 分量语法跟随
证伪回退（declared 写侧剥 % 后 store 无语法）——hsl/hwb computed none 组
保形 8 案恢复（cc8 通过系旧 serial 强加 % 与影子漏存两错误抵消的 accidental
pass）；color-mix 插值空间别名折叠 `'in xyz' ≡ 'in xyz-d65'`——xyz none 组
13 案恢复（v6 记录轮为 pre-alias 测量态 −13）。v7 HEAD 复录终局核对。
[证据](evidence/2026-11-11-dc2-values-serialize-r2320-color-cc-ctx.md)
（values 侧 8b5fc9ebd：values corpus v5d 1,548 → 2,213 净 +665 逐页不降；
已知缺口挂账：atan2 混型实参、trig 实参时间单位各 1F。color 侧收口探针
法证与 v8 终账（对 cc8 净 +26 逐页 0 回归）见
[证据](evidence/2026-11-11-dc2-color-raw-shadow-mix-alias.md)。）

**DC-2 color 通道批完成（11-11 上午）**：light-dark() 双通道建模 + system
color dark 面板 + color-scheme 级联读取（提交 639bed48a）——shim part11
`_zwCCParseFn` light-dark 分支（支值暂存 AST 延迟 resolve，currentColor 支
走属性上下文）+ `_zwCCUsedSchemeDark` 沿父链合成（空格分词，light dark 双
token 保守 light 对齐 parse_color_scheme_dark）+ `_zwCCSystemDark` 12 键
dark 面板 + part01c color-scheme 级联分支（inline→作者臂→父链）+ Rust
dark 臂 menu 扩键——color corpus **10,782 → 10,801（净 +19），逐页 0 回归**
（system-color-compute +8、system-color-support +9、parsing/relative-color
+2 意外收；values 2,213P 零漂移）。通道分野定谳：support 页（动态元素）走
shim plain 臂、compute/consistency 页走宿主 cascade，两改动面互不越界
（consistency 页 ''=='' 空串钉面零变化）。
**DC-2 shadow/caret dark 贯穿批完成（11-11 上午，提交 6dd926911）**：
parse_color_with_scheme dark 贯穿 text-shadow/box-shadow/caret-color/
column-rule-color 四属性解析链（apply_advanced 四臂读
style.color_scheme_dark、supports 语法判定传 false、测试调用点适配）+
case 'system' 分派 dark 优先（LinkText/VisitedText 非空 light 静态值键
light 优先则 dark 分派永不生效）——color corpus **10,801 → 10,808（净
+7），两批合计 10,782 → 10,808 净 +26 逐页 0 回归**（compute shadow/caret
差分 4F、support LinkText/VisitedText/SelectedItem 3F 全数兑现；values
零漂移；干净树 cargo check EXIT=0——zwv10-check.log 101 为提交前旧态
伪信号）。残余挂账：light-dark image 上下文 31F、Typed-OM 相对色 2F、
currentcolor-in-light-dark 宿主黑 1F、fill/stroke gCS 读数 2F、consistency
dark FieldText 5F（UA 表单样式建模）。
[证据](evidence/2026-11-11-dc2-lightdark-scheme-system.md)

**DC-2 light-dark image 批完成（11-11 午后）**：css-color-5 §funcdef-light-dark
图像上下文贯穿四属性（提交 8c166641d）——background-image/mask-image 共用 +
list-style-image + border-image-source 三解析器 LightDark 变体（恰两参、支内
none 合法、色值支/arity 递归拒）+ cascade 期按 style.color_scheme_dark 选支
展开（computed 一次定案，inherit 继承选中支）+ 支内 none 展开 TransparentImage
（序列化 image(rgba(0, 0, 0, 0)) 区别顶层 none）+ paint 五臂补 exhaustive +
shim 写面 light-dark 前缀浅校验（invalid 组拒，白名单外 passthrough 零回归）。
color corpus **10,808 → 10,840（净 +32）逐页零回归**（共享树提前兑现，v11=70
记录轮 build 含本批在树 hunk：light-dark-image 1/32 → 30/32 本批 +29 全数兑现；
compute fill/stroke +2 与 currentcolor +1 归同树 70 的 fill/stroke hunk 非本批；
values 2,213P 零漂移；纯净 HEAD 终账归 DC-3）。挂账：content 图像 light-dark
1F、image-set/cross-fade 嵌套 1F、UA 表单 dark FieldText 5F。
[证据](evidence/2026-11-11-dc2-lightdark-image.md)

**DC-2 fill/stroke gCS + currentColor 宿主解析批完成（11-11 午后）**：fill/stroke
三层缺位补齐 + color 属性 currentColor 宿主后置解析（提交 179d04529）——apply
fill/stroke 两臂 parse_color_with_scheme（system color dark 传播，同
apply_advanced 四臂形态）+ inherit_property fill/stroke 继承臂（SVG2
Inherited: yes，R4280/R5187「继承由后续切片补」落地）+ inheritance.rs 继承完成后置
解析 `style.color == CurrentColor → parent.color`（css-color-3/5；shim plain 臂
已有同语义，宿主 cascade 面补齐——双通道一致）+ computed_style fill/stroke 序列化
臂（None→SVG2 初始 fill 黑/stroke none）——system-color-compute **25P/2F →
27P/0F 页面清零**、light-dark-currentcolor-in-color 1F→1P，本批面净 +3（v11
同轮 10,808→10,840 净 +32 含 e2 image 链 +29，逐页零回归按面分页归属；values
2,213P 零漂移）。残余挂账：consistency dark FieldText 5F（UA 表单样式建模）、
Typed-OM 相对色 2F、fill/stroke none/transparent 存储串区分（存储层变体）。
[证据](evidence/2026-11-11-dc2-fill-stroke-gcs-currentcolor-host.md)

**DC-2 grid-line 组合形批完成（11-11 午后）**：`<grid-line>` 组合形模型贯穿
（提交 c535b64b6 + fmt 收敛 9f025bc66）——GridLineValue 结构变体
（Line{line,name}/Span{count,name}）+ parse_grid_line 全文法重写
（css-grid-2 §7.1：`[<integer> && <custom-ident>?]` 任意序、`[ span &&
<integer>? && <custom-ident>? ]`、span2 无空格 token 边界纠正为 custom-ident）
+ grid_line_to_css canonical 序列化 + converter 落位 NamedSpan——勘根修正
G1 批猜测（46F 实为组合形 21F + template used 24F 两族，发射臂/cascade 臂
早已存在非缺口）；css-grid corpus v13 全量 635 页逐页对基线 **净 +26 零回归**
（锚页 grid-layout-properties 94P/46F → 115P/25F：3 组合形 × 7 属性全翻，
pair/area shorthand 走同一解析路径连带；grid-area-computed 8→13 含 unicode
ident）；锚页残余 25F 全为 grid-template-* used 值族挂账 G2b（taffy
detailed_layout_info 接线，架构面独立批）。
[证据](evidence/2026-11-11-dc2-grid-line-composite-model.md)

**DC-2 组 E 批完成（11-11 午后）**：Typed-OM 相对色骨架保形 + sibling-index()/count() 求值 + decl canon light-dark 保形 + var() 作者臂（part10 `_zwOmRelativeSkeletonText` 骨架检测 + part11 `__zwCssRelativeOrigin` 出口/`_zwCCSubSibling` 三刀位/`_zwCCDeclColorCanonic` light-dark 保形/part01c `__zwPlainAuthorDecl` 作者快照出口）——probe 家族 6 页 2439P/69F（对 v10 净 +2）；e2final 终验三语料：**css-color 10,842P（对 v10 净 +34 逐页零回归，本批面 +2**，relative-color-with-system-color 页 1P/2F→3P/0F 清零；同树在库批兑现 +32）**、values 2,213P 零漂移、typed-om 对 v5b -4**（G1 门 var() 误拒，非本批面，随放行刀复位）。事后归谳：computed 页 13 个 var 案根因在 calc 通道算术与 modern 形态序列化（var 查找既有通路已通），var 作者臂按守卫面保留零钉面交付挂账。
[证据](evidence/2026-11-11-dc2-typed-om-relative-skeleton.md)

**DC-2 grid var() 放行刀完成（11-11 午后）**：canonicalize_grid_decl 含 var() 值 parse 期放行（grid_cssom.rs 单实现双通道一处 + 单测）——typed-om **9,872P = v5b 基线复位零 diff**（4 案 'Can set grid-* to var() references' 翻回）；css-grid 对 v13 共 635 页逐案零漂移（case 列表 654 = 635 共页 + 19 数据目录漂移页 37P，subgrid fetch + 探针遗留，非批面；探针页已清出数据目录）。

**DC-3 预备完成（11-11 上午）**：周跑 reconciliation runbook + 验收核对清单 +
缺口三分类地图（A 语义 ~34k / B 布局 ~85k / C 零通过 API 门 ~950，对 90 子组
全 144,109 缺口）+ A 类四桶失败逐案归类（values 3,380F：abs/sign used-value
~250F、random/calc-mix ~325F、attr() ~103F；fonts 2,751F：custom-ident 语法
~608F、font 简写全语法 ~615F、@font-feature-values ~294F；selectors 2,116F：
`:has()` invalidation ~970F；anchor 6,213F：position-area 解析/序列化
~2,400F）——下一批直接取用。多实例仲裁：v11 共享树提前兑现 color
10,808→10,840（image +29 / fill-stroke +3 归属拆分）、v13 grid 净 +26；v14
周跑**延后**至批次平台期（距验收线远，周六 10:00Z cron 为免费 checkpoint，
DC-3 终跑须最终 HEAD）。合并树验证：fmt EXIT=0、clippy(v8) PASS；make test
抓获 quickjs 腿 E0433×2（callbacks.rs grid CSSOM 引用未加 v8 门，677aacba8
落地态缺陷，grid 链修复中）。
[证据](evidence/2026-11-11-dc3-prep-reconciliation-runbook.md)

[执行契约](../wptfyi-css.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | **完成** | [基线与三分类](evidence/2026-11-10-dc1-baseline-and-classification.md)；快照基线 29,958/196,461（[coverage-2026-11-09](../../compat/wptfyi/coverage-2026-11-09.md)） |
| DC-2 按簇收敛 | 二十五批完成 | [Typed OM API 入口面](evidence/2026-11-10-dc2-part10-typed-om-api.md)：typed-om 簇 25 → 9830；[css-color 色值管线](evidence/2026-11-10-dc2-part11-color-pipeline.md)：color 簇 1,293 → 7,229；[computed 数学管线](evidence/2026-11-10-dc2-part11-computed-color.md)：color 簇 7,229 → 9,318；[declared 声明面归一](evidence/2026-11-10-dc2-declared-normalization.md)：color 簇 9,318 → 10,530；[语法面收批](evidence/2026-11-10-dc2-color-layers-alpha-decl.md)：color 簇 10,530 → 10,756；[语料扩面 13 目录](evidence/2026-11-10-dc2-corpus-expansion.md)：新 corpus 8,562 子测试 / Pass 1,381；[values 数学槽位 + T54](evidence/2026-11-11-dc2-values-math-slots-t54.md)：values 簇 1,540 → 1,770；[grid 声明面 + has 面](evidence/2026-11-11-dc2-grid-cssom-decl-face.md)：锚页 13 → 94；[values serialize r2320 + color currentcolor ctx](evidence/2026-11-11-dc2-values-serialize-r2320-color-cc-ctx.md)：values v5d 1,548 → 2,213、color +26 子测试；[light-dark + system dark 面板](evidence/2026-11-11-dc2-lightdark-scheme-system.md)：color 10,782 → 10,808 逐页 0 回归（含 shadow/caret dark 贯穿批）；[light-dark 图像上下文](evidence/2026-11-11-dc2-lightdark-image.md)：color 10,808 → 10,840 逐页 0 回归（本批 +29，同树 fill/stroke +3 归 70）；[fill/stroke gCS + currentColor 宿主解析](evidence/2026-11-11-dc2-fill-stroke-gcs-currentcolor-host.md)：v11 10,808 → 10,840（本批面 +3，compute 页清零）；[grid-line 组合形模型](evidence/2026-11-11-dc2-grid-line-composite-model.md)：css-grid v13 净 +26 零回归、锚页 94 → 115（fmt 收敛 9f025bc66）；[Typed-OM 相对色骨架 + sibling 求值](evidence/2026-11-11-dc2-typed-om-relative-skeleton.md)：css-color e2final 净 +34 零回归（本批 +2 页清零）+ grid var() 放行刀 typed-om 9,872P 复位；[position-area 语法面](evidence/2026-11-11-dc2-position-area-grammar-face.md)：anchorbase 7460P → 9893P 净 +2433F 逐页零回归（parsing 1800F/computed 633F 双页全清，撞车仲裁 grid G1 单实现双通道）；[color none 序列化保形 + hue echo](evidence/2026-11-11-dc2-color-none-serialization-face.md)：css-color 10,842 → 10,887 净 +45 零回归（rgb none 门影子优先/hsl-hwb srcBare 裸形跟随/lab hue echo 双锚，typed-om +4 为在库批兑现）；[RCS+var 入口放行](evidence/2026-11-11-dc2-color-var-origin-rcs-entry.md)：css-color 10,887 → 10,900 净 +13 零回归（var-origin 相对色 13F，形桶 + calc 桶，values/typed-om 持平）；[嵌套 relative 真逆直转](evidence/2026-11-11-dc2-color-nested-relative-true-inverse.md)：css-color 10,900 → 10,905 净 +5 零回归（oklab/oklch 嵌套 4 案 + pow 裸函数形 1 案，values 连带 +5，双 spawn 孤儿采纳）；[@position-try CSSOM 面](evidence/2026-11-11-dc2-position-try-cssom-face.md)：anchor 语料 9,893 → 10,182 净 +289 零回归（CSSOM 读面 +265、insertRule 写面 +24，parse/allowed-declarations 全翻，残余 1F 布局依赖）；[anchor 族声明面文法](evidence/2026-11-11-dc2-anchor-family-decl-face.md)：anchor 语料 10,182 → 10,455 净 +273 零回归（18 页清零：声明面文法桶 ~142F 全数兑现 + computed 五页 + order 行为页连带）；[offset CSSOM spec 化刀1](evidence/2026-11-11-dc2-offset-cssom-spec-face.md)：anchor 语料 10,455 → 10,485 净 +30 零回归（offsetParent spec 算法 + offsetLeft/Top 相对 padding edge，十六页 improved）；[position-area 区域定位布局刀2](evidence/2026-11-11-dc2-position-area-layout-face.md)：anchor 语料 10,485 → 10,549 净 +64 零回归（区域格矩阵单点下沉 + 两段式布局 pass，in-position-try 64 案单页全清）；[color 域外保形五修 + opacity 数学形读面](evidence/2026-11-11-dc2-color-out-of-gamut-opacity-face.md)：css-color 10,905 → 10,948 净 +43 逐页零回归（opacity-computed/out-of-gamut/mix-out-of-gamut 三页清零，跨流补刀合入批）；[anchor() inset 求值 + 双介入门布局刀3](evidence/2026-11-11-dc2-anchor-inset-gate-face.md)：anchor 语料 10,549 → 11,863 净 +1,314 逐页零回归（writing-modes-001 单页 1,296F 全清，三轮零漂移，inline/滚上下文双门语料实证）；[serialize 刀2 + 通用色通道 cq/sign 面 + round B=±∞ 策略表（g13）](evidence/2026-11-12-dc2-g13-serialize-fold-color-cq-round-step.md)：values 2,646 → 2,787 净 +141、color 10,970 → 10,972 净 +2，五套件合计净 +143 逐页零回归（clamp NaN 毒化暴露的 round ∞ 缺口同批兑现，FIXME types.rs:1420 清账）；[attr() var 实参 deferred 重裁站 + font 槽写面豁免 + bolder/lighter JS 通道臂（g31续）](evidence/2026-11-14-attr-font-write-gate-deferred.md)：values 3,920 → 3,935（attr-all-types +9/argument-grammar +6 回位，对 trigfix 净 +2 反超）、fonts 2,502（lighter-bolder 22F→22P 全页清零）、color 10,972 恒一、四小套件 360 页零漂移；[system-color UA 关键词化 + 写面门豁免（g32）](evidence/2026-11-14-system-color-ua-keyword.md)：css-color 10,972 → 10,999 净 +27 逐页零回归（system-color-consistency 35P/27F→62P/0F 全页清零）、typed-om 9,872 归位（g27/g28 门豁免补刀）、cssom/values/四小零漂移 |
| DC-3 全量对账 | v14 周跑完成（run 43240558696），css 60,545/196,806（30.76%，对 11-11 净 +4,293） | [预备 runbook + 缺口地图](evidence/2026-11-11-dc3-prep-reconciliation-runbook.md) + [v14 对账](evidence/2026-11-13-dc3-v14-reconciliation.md)：三家 @ a17acfbc8a + corpus pins 核对 ✓；验收线 182,392 尚余 ~121.8k（结构判断维持）；分片核对——grid 精确一致/values/color 达阈值、**typed-om −43 = `.any.js`
通道周跑从未发过（fetch 排除机制定谳，fetch-b 目标已修待周跑显影，见
evidence §9）**；cssom −533 = serialize-values.html 页级崩已归因+已修复（window.length [Replaceable] descriptor 修正，探针 533P 精确回位）；四小回归 −5 = font-property-unit-cycles trig 型别门**已修复**（g26，5P 真实回位，见 [trig 型别门 evidence](evidence/2026-11-13-trig-typegate-font-unit-cycles.md)）；−4/−2/−1 已定位形态待修；zw_env_fails=0 |

碰撞记录：11-11 凌晨 union 树 engine 全量 7F（img/svg settle + websocket
族）——values/T54 批写面与该 7 测试零交集（part85/part45 无 style 用法），
归因待 color/grid 链在途 hunks 或 main 漂移。**定谳（11-11 凌晨，HEAD
worktree 判定）**：HEAD be6703d58 冷编译后同 7 测试全红（0.17s、同断言：
img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 /
fetch_filtered_response_type_matrix / iframe_service_worker_messages /
response_body_readable_stream_r2967 / t8k_websocket_error_nonterminal /
t8k_websocket_event_flow）——**main 既有欠账，与任何在途 hunk 无关**。
r2320/color ctx 批 make test 全量 3835P/7F，7F 即该既有组，本批新增测试
全绿，不随本批携带。

遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
**11-11 上午 anchor 探针双写撞车**：同 clone 两 unit 同默认 transcript 双写
→ 块体交错 merge 失败双零产出（取证
.acceptance/wptfyi-css/notes-anchor-collision-forensics.md）；后续探针一律
CSS_CORPUS_TRANSCRIPT 唯一路径。同日组F 撞车仲裁（本流 JS 解析器 vs 并行流
Rust 文法模块）经跨会话消息定案 grid G1 单实现双通道，JS 解析器删除改薄委托
——文件集正交流程（认领件提交 + 统一 rebase push）有效。
**11-11 晚间 anchor 零漂移对账三写竞态处置**：组E 收官对账触发同 clone 三方
同跑（d3 systemd 单元 + sibling inline run 同写 results-g5/transcript）。
处置：d3 单元停用清场（其 corpus 未启动、transcript 保持单写无损）；inline
run 遭会话超时终止（253 页完整 JSON 幸存但二进制 provenance 骑 fd1-gate2
重建线，留档 *.partial-collision-2040 作旁证 11,863P）；权威跑 = d3
zw-g5-anchor-2152（transcript 换名重起）。教训：语料起跑前先查
systemctl list-units + ps 按进程类清点，transcript 名带轮次戳。
**system-color-consistency 27F 四流批挂账**（triage
.acceptance/wptfyi-css/triage-system-color-consistency.md）：A ButtonBorder
符号身份 7F / B ButtonFace 值漂移 9F / C ButtonText-FieldText scheme 缺口
11F；S2 值对齐结构性不可行（同 keyword 双期望 239/255），唯一绿路 = S1
keyword 身份端到端。序列（组E 五修落地后开）：css-parser System(name) 符号
解析 → style-system used-time scheme-aware resolve（apply.rs:66-80 管道在）
→ engine UA 表单字面量 keyword 化 → shim 光表 keyword echo
（part11-computed-color.js:199 FIXME 到期：UA form modeling 落地破光表
''=='' 术）。预期全兑现 27F。**已闭账（g32，11-14）**：11-14 探针实证推翻
原 S1 假设（页为真值自洽比对非回声；S2「结构性不可行」结论同被推翻——值
对齐由 scheme-aware 双轮自洽达成，不依赖具体值），修法三面与终账见
evidence 2026-11-14-system-color-ua-keyword 与 triage 11-14 重写版。
**11-11 晚 color 组E 多流协作**：5a 流验证四修批期间，第三方会话（83/d3
询认未回）在工作树补刀 fix4/5（powerless hue 收窄 + hsl floor 撤除）并扩写
证据文件——5a 审 diff 后按仲裁认领合批（五修一提交，提交信息记协作来源）；
验证链互不重建：fd1-gate2 出 make test/css-values/css-color 账，5a 起短账腿
（engine/bin stash A/B 归因），anchor 零漂移门由补刀流 run-g5-anchor.sh 复用
fd1 二进制补跑——跨流同面时以「验证分账、提交单点」收敛。另：run-fd1.sh
等场环 ps 全局 grep 卡异 clone runner ~30 分钟（ZeroWeb-3 editing run），
已修为 /proc/*/cwd|exe 树归属过滤（见
docs/learnings/platform/2026-11/2026-11-11-clone-blind-wait-loop-ps-grep.md）。
**11-11 晚 g6 系命名撞碰与无 journal 构建**：21:04-21:09 他流 unit
zw-g6-color-2104 跑 make test exit=2（脚本 banner 亦复用「== g6 done ==」——
g6 系名与他流撞碰，本流后续核验链改 g7 系命名避让）；21:10:37
target/release/zero-wpt-runner 重建无任何 journal 痕迹（裸跑构建，违 systemd
守卫纪律），构建起点落在 21:09:57 contrast-color WIP shim 编辑前后 40-90s
编译窗内——该二进制是否含此 WIP 需行为法证，引用该二进制的他流账注意
shim 面归因。本流 g6b 账（20:56-20:58）早于该构建，纯 HEAD 无污染。
**11-11 晚 contrast WIP/g7 链归属定案（跨 session 消息，ba↔本流）**：
21:09:57 WIP 编辑者最可能为 ba 上轮（其压缩恢复误判自写线索成立；本流前轮
g7 脚本按「树上 WIP」法证口吻处理未认领编辑）——g7 核验脚本由本流前轮
21:10-21:19 写就（g6 撞碰避让改名），ba unit zw-g7-contrast-2118 执行；
提交单点 36de2a6f6+0f65b27df（ba）。zz-caprobe 探针页与 zw-g6-color-2104
unit 均非本流手笔。本 session 认领账：g7 结果逐页归因分析（+14 中 +4 真收割
+10 探针锚）+ mktest 账（见下条）。
**11-11 晚 make test 9F 归因（unit zw-g7b-mktest-2230，f415be99c+contrast
树）**：3877P/**9F**——7F 为在册欠账组；新增 2F = part06.rs
`test_execcommand_format_apply_r3254_m3` / `_toggle_state_r3254_m3_slice4`，
静态归因 **f415be99c**（六 inline format apply 切 `__zw_ce_format_apply`
直译引擎、part06.js ±30，未同步断言旧 SetInnerHtml 载体契约的 2 个 Rust
单测；part06.rs 上游最后改动 11ebb0757，历史账 7F 恒定）——editing 流域
落地态回归，待其流修复，本流 color hunks 零交集。
**11-13 v14 对账 cssom −533 跨流碰头（本流 × html-semantics 流）**：
html-semantics 流 b6793f4ad（s400 window.length）在 shim part01.js 以
configurable:false 定义 own accessor，本流 v14 周跑 cssom
serialize-values.html 顶层 `function length(){}` 走 GDI step 4 抛
SyntaxError 整页崩（533→0）。归因+语义定谳+修复由本流完成（spec 双源：
[Replaceable] 非 [LegacyUnforgeable]，Chromium window.idl/WPT unforgeable
钉测面佐证），对方流 s400 桥测 L4 pin 随修复同步修正
configurable:true（其余 pin 计数相位/bare-origin/error/移除面全保，桥测
2 passed 实证）。对方流无需另行动作；engine/shim 共享面后续改动注意
window.length descriptor 已回归 spec 口径。

**g36 anchor border 坐标系定谳修复 + s423 真态收口（11-14）**：三刀一账。
① s423（同 turn 读写陈旧）修复宽门 A/B 净 −1007 定谳为伪通过揭开（writing-
modes-001 1296P 基线全为陈旧读碰巧相等——drain-split 探针实证），门控收窄
纯 inline style 族（DomMutation::is_inline_style_family 八变体）保账本；
g35「CSSOM 声明面缺失」根因表述证伪（写面 _styleProxy 链早已通，真缺口 =
读陈旧）。② anchor_pos V 系统一：collect_anchors/apply_walk/apply_inset_walk
origin 累计 D=0（不含 border）vs rect 提取/渲染递归 D=border 错位——
borderless CB 两系重合（既有 14 g1f 全 borderless 故全绿），bordered CB
inset 通路多 +border、区域通路偏移多 +border 满填缩 2；修复 = 三 walk
origin 加 border、cb_x 去 border 项、inset 写入直用 padding 相对量、
apply_axis_align 删 to_parent；新钉测 g1f_border_cb_child_origin，g1f
15/15、layout-engine 2509、fmt/clippy 净。③ corpus A/B ab2→ab3
11,870/966/4T→11,977/876/4T 净 +107（真语料 +88 零真回归）：
position-area-basic +45/wm-dir +44/with-insets+align-justify+borders-002
各 +1；4 REG（last-successful-* keep flip-block）定谳伪通过揭开——四页
container 无 border 与 V 化无关，期望 200 恰为写前陈旧快照值、现算真值
250 = flip-block 重算值，「last-successful 几何保持」缺口挂账（桶 3F→4F）。
engine lib 12F（websocket/media/fetch/iframe 域）stash A/B 定谳 HEAD 同红
既有欠账非本轮，挂账待归属（rebase 后 12F→7F 与远端 94883fbbe 清偿账
精确对齐）。**组合态 ab4c**（远端 FORM named-first shim 面 + 本轮 V 化
Rust 二进制）265 cases 11,977P/876F/4T 与 ab3 逐页逐位一致 REGRESSED=0
——shim 改动与本轮 Rust 零交互扰动，g36 已 push（783b22267）。挂账：
writing-modes-001 伪通过桶/last-successful 4F/'none' 高 18/position-
area-value 文法缺口（span-top left/start center 形 +7）。[证据](evidence/2026-11-14-g36-border-coord-s423.md)
**g36 组合态对账（11-14 晚）**：rebase 后组合态二进制（远端 FORM
named-first shim part03/04/05 + 本轮 V 化 Rust，17:09 build2 provenance）
anchor corpus 复账 results-g36-anchor-comb2：**11,977P/876F/4T 与 ab3 逐值
一致零回归**——shim FORM 改动不触 anchor 面，组合态成立。跨流碰撞两笔：
ab4（17:09）遭并行流同名脚本竞态焚毁（transcript/JSON_DIR 被删，对方
editing-compat v1.108 提交信息认领）；ab4c（17:12 他会话发射）1 分钟产出
与 ab3 字节级同 JSON——corpus 固有 659 页头/394 空体形态下 265 真值页，
时钟上 1 分钟跑不完 265 案真账且无发射会话认领，按 salvage 疑似假账弃用，
不进账本；comb2 为组合态权威账。
**（ab4c 定性两说并存）**：并行轮 2841d6a2f 认领 ab4c 为真账（265 案逐位
一致 REGRESSED=0）；本流按 1 分钟时长无法覆盖 265 案 + 无发射会话认领判
salvage 假账弃用。两说并存记录在案——实质结论一致（组合态 11,977/876/4T
零回归，与 ab3 逐值一致），ab4c 定性分歧不影响 g36 主账与组合态结论；
后续轮次如需 ab4c 复用须先复跑定谳。
**g36 组合态对账（11-14，rebase 7ebe9a44b→783b22267 后）**：远端 shim
part03/04/05（FORM named-first/nameditem 面，4e3eea8f3+94883fbbe）与本轮
Rust 组合态二进制（build2 17:09 provenance）重跑 anchor 语料 comb2：
**11,977/876/4T 与 ab3 逐案持平**（JSON/transcript 字节级一致 = runner
确定性输出复现，非复制）——组合态零交互回归。首射 ab4 遭并行流同名脚本
竞态焚毁（330ba2b40 记载「salvage 字节读偏移」，transcript JSON 解析中断
结果目录空）；ab4c（并行流）与 comb2（本流）85 秒级独立双跑字节级互证。
计时校正：runner 真跑 ~0.3s/案（单页实验 1.8s 含启动），ab3 的 14 分钟为
ZeroWeb-3 电池+构建 CPU 竞争异常慢，勿再以它为基线推算。账本维持
783b22267，无新增提交面。

**g36 组合态权威账（11-14 晚，comb3-x7）**：rebase 组合态（远端
4e3eea8f3 shim part03/04/05 FORM named-first 施工 + 本流 s423 收窄/V 系
修复，二进制 provenance = 17:09 build2）anchor 语料全量重跑 **11,985/
868/4T，对 ab3 净 +8**（子测试 11,977/876）：IMP 12（anchor-size-parse-
valid +8 / anchor-parse-valid +4——anchor()/anchor-size() calc 嵌套与
降级形写面 parse 接受，**归因 4e3eea8f3 写面通道红利，它流域**）；REG 4
（anchor-invalid-fallback 嵌套无效形放行缺口，同源归因，挂账它流）。
进程教训入账：ab4 与 comb2 跑批被并行流同名脚本竞态焚毁/salvage（
330ba2b40 记载同源），ab4c 为 salvage 恢复副本（与 ab3 字节级同 md5，
不可作账）；comb3-x7 唯一名 x7 系发射，659 案 transcript 完整 + 5 案
真 diff + 97s（机器空闲 rayon 并行，非此前多流竞争态 14 分钟）为真账
判据。anchor 真态定格 11,985P（真页口径 11,988−zz 探针页）。

**g37 anchor()/anchor-size() calc 嵌入叶 + serialize canonical 三修（11-14 晚）**：
anchor-parse-valid 4F/2359T + anchor-size-parse-valid 8F/4305T 三类签名
（除法转乘/anchor 叶排序/fallback 补 px）全清——`CalcExpr::Anchor` 叶
（parse 期实参 canonical + 25 处 exhaustive match 补臂、求值 None
fail-closed）、Divide 臂除法转乘（product_term_from_factors 共用重构 +
殿首乘积项括号剥除）、canonicalize_anchor_functions fallback 补 px；
fold 白名单扩 top/max-width（12F 槽驱动，left 零漂移钉面保全）。
invalid-fallback 4 NEWFAIL 定谳「parse 成功 vs parse 失败」computed fail
策略分歧 → parse_length_or_math guard（anchor 叶式声明无效回落初始值，
css-anchor-1 §3.3 computed-value time invalid 语义，与此前 parse 失败
严格一致零布局回归）。基线仲裁：ab4c（选刀依据，签名与真态一致）/
comb3-x7（伪账，HEAD 反证 2355P/4F 复现，见上更正块）均非真态基线。
终账 g37-final：**265 页 11,993P/864F/4T，对 ab4c 净 +12 零 REGRESSED**
（parse-valid 两页 12 子测全清、invalid-fallback 回基线 6F）；单测
css-parser 3208 全过（新钉 15 形表 + fallback 7 断言）、style-system
2534、layout-engine 2509、engine lib 3951P/7F（ip M3-A 欠账名单一致）。
挂账：anchor-size used 值布局期求值刀 / anchor_size_domain 直值 vs calc
嵌套两级域收窄 / invalid-fallback 残 6F。[证据](evidence/2026-11-14-g37-anchor-serialize-canonical.md)

**（comb3-x7 伪账定谳更正，11-14 晚，g37 轮）**：上账 parse-valid 0F 与
invalid-fallback 10F 经 HEAD 反证证伪——stash 本刀全改动 → HEAD 态
release 重建 → anchor-parse-valid solo 复现 **2355P/4F**：comb3-x7 的
二进制（build2 17:09 provenance）非 HEAD 代码态（含未入远端的 shim 实验
码），「IMP 12 归因 4e3eea8f3 写面通道」撤销（4e3eea8f3 = FORM
named-first nameditem 面，与 anchor canonical 零交集）。canonical 12F
从未被远端代码拿到；comb3-x7 伪绿 12 由 g37 真清偿、伪红 4（invalid-
fallback）由 g37 parse_length_or_math guard 修正。HEAD 真态按 ab4c 签名
复现（parse-valid 4F/8F + invalid-fallback 6F 基线，全量未重跑）。[证据
2026-11-14-g37-anchor-serialize-canonical](evidence/2026-11-14-g37-anchor-serialize-canonical.md)

**（g36 复核轮：comb3-x7 执行真实性补证 + 归因伪账 firsthand 合账，11-14 深夜）**：
本轮按「验证组合态真账」任务独立复核，两条正交结论定谳。其一**执行真实**——
journal scope 记账 comb3-x7 = 111.5s CPU/97.5s wall（wrapper 17:45:01→
17:46:38 同一致），实测真语料案 release 单案 0.18s CPU
（anchor-animation-dynamic-default 273B 产出与 transcript 块一致），
659×~0.17s≈112s 精确闭合，transcript 659 案活写、JSON 纯由 transcript 合成、
fresh dir：真跑成立；comb2 块「~0.3s/案」同量级互证；「真跑应 ≥10 分钟」判据
与「ab3 14 分钟」旧说均废（journal 权威 ab3 = 1m39.7s wall/1m54.0s CPU），
comb3-x7 块「rayon 并行」表述更正为串行 release ~0.17s/案。其二**归因伪账
维持**（firsthand 计数器复核）——comb3-x7 parse-valid 0F/size-parse-valid 0F/
invalid-fallback 10F vs ab3 4F/8F/6F vs g37-final 0F/0F/6F：build2（17:09）
系共享树在途构建扫入 g37 前身实验码（无 parse_length_or_math guard 故
invalid-fallback 10F 恶化），IMP12/REG4 非组合态红利，上块撤销成立，
g37-final 11,993P/864F/4T 为 HEAD 终账；654/659 transcript 块字节级同 ab3
= runner 确定性复现非复制。教训入账：**共享树 cargo 构建会编译他流在途
半成品**（build2 即例证），跑批发射前须 git status 干净或 provenance 可证；
journal scope CPU/wall 记账为跑批真实性首选审计工具。ab4 焚毁、ab4c/comb2
salvage 副本不可作账维持原判。

**g39 空绝位 auto 高 strut 门（11-14 深夜）**：g36 挂账 'none'/空绝位
盒 h=18 根因定谳——`apply_abspos_shrink_to_fit_width` height_fix 臂对全部
塌 0 的 auto 高 abs 盒无条件写行高 strut（≈18），本意文本叶近似但未查
行内内容（taffy 树 dump 0×0 → 提取层 0×18 定位写入点；CSS2 §10.6.4
auto 高 = 内容高）。修复 = `has_inline_content` 门（strut 只对有行内
内容塌 0 盒，文本叶近似合法域保持）+ `style != style_before` 终写判
（塌 0 空盒不再触发空转重布局）；常驻钉测 `g1f_abs_empty_auto_height`
（探针转正：空 Inline §9.7 块化/空 Block 0×0 + 含文本行盒高 + 两对照）。
layout-engine lib 2510P/0F。组合态账（他流 g38 anchor-size 回填同树
在账，二进制含其接线态）：265 页 **12,061P/796F/4T**，对 comb3-x7
JSON +76 / 对 g37-final 真态 **+68**（skew 8 = 伪绿 12 − 伪红 4 互消）；
归因本刀 +3 零回归（position-area-basic 'none'/'span-all top' 空盒
路径 ×2 + multicol-001 inset bottom 高依赖式 ×1）、invalid-fallback
+4 = comb3-x7 伪红复正基线修复项、余 ~+65 归 g38（anchor-ident-function
P→F REG + anchor-name-003 页内混翻挂账 g38 流）。探针链先验：none
`0,0,0,0`（原 18）、span-all/top-left 两形态零扰动。
[证据](evidence/2026-11-14-g39-abs-empty-strut-gate.md)

**（g39 账目定谳更正 + g38 收口，11-14 深夜，g38 轮）**：g38
anchor-size() used 值刀收口时三 corpus 对账揭两处账基偏差并定谳 g39
伪通过揭穿。其一**账基线更正**——g37-final 在账 11,993P/864F/4T，
results-g37-final JSON 工件实存 **11,989P**/864F/4T（Δ+4P，原始
stdout 日志未存无法复源）；g39 组合态在账 12,061P/796F，留存日志
zw-g37-strutgate.log 实存 12,061P/**792**F（Δ4F）且该跑二进制
provenance 过期（18:27:55 起编先于 18:39 终版 gate 提交，sizing.rs
提交态分歧 solo bisect 实证）。其二**g39 揭穿定谳**——HEAD 提交态
（g38 stash）重录 corpus 11,129P/1,724F，对 g37-final 工件基逐案
分解：直接效应 +5（position-area-basic ×2、multicol-001、
dynamic-001、dynamic-004）、伪通过摘除 −865（anchor-position-
writing-modes-001 **864** + zz-g36probe-wm001-drain 探针页 1）、净
**−860**；writing-modes-001 1296P→432P/864F 机制 = g39
`style != style_before` 终写判关掉了空绝位盒的空转重布局，而该重
布局对 VRL 容器 in-flow 锚几何 load-bearing（1296P 系副作用撑出的
伪通过，g36 挂账桶定谳）；先在 VRL in-flow 布局缺口另立修复刀。
其三 **g38 净 +70**——六尺寸槽 + 四 inset 双形承载布局期回填 used
宽高（§4 三轴映射/隐式缺省/fallback/replaced 均匀缩放/清槽防回灌），
对 HEAD 基线 11,129P→**11,199P**/1,654F（+77/−7 全 Fail↔Pass：
anchor-size 四页 +52 恰合预估、anchor-name 家族 +18/−6、misc +7/−0、
ident-function −1）；−7 挂账两域：锚资格 §3.2 刀（含块链/tree-order
「positioned after」排除，name-002 ×2 + name-003 ×4，与
anchor-position-004 32F 同域）+ gCS resolved-value 刀（ident-function
−1，内联双形声明被接受后 width 槽 Auto gCS 'auto'≠'15px'）。
单测 style-system 4 + layout-engine 5 新钉全过（三 crate unit 全绿、
fmt/clippy 净）。教训：跑批账必须锚工件 JSON 逐案计数（stdout 汇总
与账本转抄可 ±4 漂移），二进制 provenance 先于数字采信。
[证据](evidence/2026-11-14-g38-anchor-size-used-value.md)
