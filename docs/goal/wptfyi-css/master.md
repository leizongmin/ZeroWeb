# CSS 子测试控制面

**2026-11-11 · Active**。目标净增 153,000 项（绝对线 182,392）。11-10 发布
快照 52,488 / 196,597（26.70%，run 42801058132），对验收线尚余 ~129,904；
已落库未入快照增量逐批见 DC-2 各批账与里程碑表（typed-om/color/values/
grid/anchor 各批，至组F 布局刀3 anchor 语料 10,549→11,863；本流组E color
face css-color 10,905→10,970，至 cqw 容器基座刀；values face
css-values 2,426→2,646，至 ∞/NaN 数学求值管线刀1+registry 前置门收尾）。
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
**卡点**：无阻塞。
**下一步**：组 F 布局桶（刀1 offset CSSOM + 刀2 position-area + 刀3 anchor()
inset 求值已落，anchor 语料 11,863P）：anchor 语料残余 953F 分桶——
position-area wm/dir + align/justify ~114F（非 htb 逻辑轴映射）、anchor()
复杂形/动态 ~98F（anchor-name-003/anchor-position-004/anchor-inside-outside/
anchor-name-mutation）、anchor-size() ~50F、position-area-basic 47F、fallback
布局 ~43F（溢出检查 + last-successful 状态机 + @position-try 应用引擎）、
inline 首片段 10F + 滚移感知 ~16F（刀3 双门撤条件）、apply.rs 数学函数残余
transform matrix 6 腿（并入 matrix 化桶）；values 残余桶（g12 后）：∞ 桶
（calc-infinity-nan-computed 残 ~31F：rotate 14 → matrix 化桶 + %-CB 形 +
unbalanced 形；serialize 五页残 ~116F——length 41/resolution 29/time 29 主体
未动 + angle/number 残，刀2 = fold_math_specified 扩 Infinity/NaN 规范形
`calc(NaN * 1px)`/`calc(infinity * 1s)` 因子归一 + signs-abs computed 残 ~94F/
invalid 52F——`abs()` 空参文法、font-weight 非白名单属性写面/serialize 4F，
FIXME types.rs:1420 round step ±∞ 族）、
%-mixing CalcContext 族（calc-mix-serialize 113F + calc-mix-computed 53F 主体）、
独立桶 line-break-ch
194F / random() 141F / attr-all-types 103F / viewport-units 84F / calc-size 62F /
progress-serialize 55F / position-computed 55F；其余散页；anchor-ident-function 求值面 12F
（`ident()` 串接需 calc/sign 求值管线，parse 接受面已通）；DC-3 全量对账
（等批次平台期 dispatch 周跑，HEAD 已含 color/values/grid/anchor 各批增量）；
color 挂账桶（var-origin/pow 裸函数形/嵌套 oklab 真逆/hsl 嵌套 1F/
out-of-gamut 20F/opacity calc 20F/mix-out-of-gamut 2F 已清——组E 批；
余系统色 27F/长尾 ~39F）；light-dark/system-color 建模残余；
grid G2b（taffy detailed_layout_info，架构面）。

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
| DC-2 按簇收敛 | 二十三批完成 | [Typed OM API 入口面](evidence/2026-11-10-dc2-part10-typed-om-api.md)：typed-om 簇 25 → 9830；[css-color 色值管线](evidence/2026-11-10-dc2-part11-color-pipeline.md)：color 簇 1,293 → 7,229；[computed 数学管线](evidence/2026-11-10-dc2-part11-computed-color.md)：color 簇 7,229 → 9,318；[declared 声明面归一](evidence/2026-11-10-dc2-declared-normalization.md)：color 簇 9,318 → 10,530；[语法面收批](evidence/2026-11-10-dc2-color-layers-alpha-decl.md)：color 簇 10,530 → 10,756；[语料扩面 13 目录](evidence/2026-11-10-dc2-corpus-expansion.md)：新 corpus 8,562 子测试 / Pass 1,381；[values 数学槽位 + T54](evidence/2026-11-11-dc2-values-math-slots-t54.md)：values 簇 1,540 → 1,770；[grid 声明面 + has 面](evidence/2026-11-11-dc2-grid-cssom-decl-face.md)：锚页 13 → 94；[values serialize r2320 + color currentcolor ctx](evidence/2026-11-11-dc2-values-serialize-r2320-color-cc-ctx.md)：values v5d 1,548 → 2,213、color +26 子测试；[light-dark + system dark 面板](evidence/2026-11-11-dc2-lightdark-scheme-system.md)：color 10,782 → 10,808 逐页 0 回归（含 shadow/caret dark 贯穿批）；[light-dark 图像上下文](evidence/2026-11-11-dc2-lightdark-image.md)：color 10,808 → 10,840 逐页 0 回归（本批 +29，同树 fill/stroke +3 归 70）；[fill/stroke gCS + currentColor 宿主解析](evidence/2026-11-11-dc2-fill-stroke-gcs-currentcolor-host.md)：v11 10,808 → 10,840（本批面 +3，compute 页清零）；[grid-line 组合形模型](evidence/2026-11-11-dc2-grid-line-composite-model.md)：css-grid v13 净 +26 零回归、锚页 94 → 115（fmt 收敛 9f025bc66）；[Typed-OM 相对色骨架 + sibling 求值](evidence/2026-11-11-dc2-typed-om-relative-skeleton.md)：css-color e2final 净 +34 零回归（本批 +2 页清零）+ grid var() 放行刀 typed-om 9,872P 复位；[position-area 语法面](evidence/2026-11-11-dc2-position-area-grammar-face.md)：anchorbase 7460P → 9893P 净 +2433F 逐页零回归（parsing 1800F/computed 633F 双页全清，撞车仲裁 grid G1 单实现双通道）；[color none 序列化保形 + hue echo](evidence/2026-11-11-dc2-color-none-serialization-face.md)：css-color 10,842 → 10,887 净 +45 零回归（rgb none 门影子优先/hsl-hwb srcBare 裸形跟随/lab hue echo 双锚，typed-om +4 为在库批兑现）；[RCS+var 入口放行](evidence/2026-11-11-dc2-color-var-origin-rcs-entry.md)：css-color 10,887 → 10,900 净 +13 零回归（var-origin 相对色 13F，形桶 + calc 桶，values/typed-om 持平）；[嵌套 relative 真逆直转](evidence/2026-11-11-dc2-color-nested-relative-true-inverse.md)：css-color 10,900 → 10,905 净 +5 零回归（oklab/oklch 嵌套 4 案 + pow 裸函数形 1 案，values 连带 +5，双 spawn 孤儿采纳）；[@position-try CSSOM 面](evidence/2026-11-11-dc2-position-try-cssom-face.md)：anchor 语料 9,893 → 10,182 净 +289 零回归（CSSOM 读面 +265、insertRule 写面 +24，parse/allowed-declarations 全翻，残余 1F 布局依赖）；[anchor 族声明面文法](evidence/2026-11-11-dc2-anchor-family-decl-face.md)：anchor 语料 10,182 → 10,455 净 +273 零回归（18 页清零：声明面文法桶 ~142F 全数兑现 + computed 五页 + order 行为页连带）；[offset CSSOM spec 化刀1](evidence/2026-11-11-dc2-offset-cssom-spec-face.md)：anchor 语料 10,455 → 10,485 净 +30 零回归（offsetParent spec 算法 + offsetLeft/Top 相对 padding edge，十六页 improved）；[position-area 区域定位布局刀2](evidence/2026-11-11-dc2-position-area-layout-face.md)：anchor 语料 10,485 → 10,549 净 +64 零回归（区域格矩阵单点下沉 + 两段式布局 pass，in-position-try 64 案单页全清）；[color 域外保形五修 + opacity 数学形读面](evidence/2026-11-11-dc2-color-out-of-gamut-opacity-face.md)：css-color 10,905 → 10,948 净 +43 逐页零回归（opacity-computed/out-of-gamut/mix-out-of-gamut 三页清零，跨流补刀合入批）；[anchor() inset 求值 + 双介入门布局刀3](evidence/2026-11-11-dc2-anchor-inset-gate-face.md)：anchor 语料 10,549 → 11,863 净 +1,314 逐页零回归（writing-modes-001 单页 1,296F 全清，三轮零漂移，inline/滚上下文双门语料实证） |
| DC-3 全量对账 | 预备完成，终跑待批次平台期 | [预备 runbook + 缺口地图](evidence/2026-11-11-dc3-prep-reconciliation-runbook.md)：验收线 182,392，11-10 快照后尚余 ~129.9k；周跑 dispatch 管线与分片截断核对清单已备，v14+ 起报号 |

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
''=='' 术）。预期全兑现 27F。
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
