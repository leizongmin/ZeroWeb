# KC-5-r3＋fame2b 电池 verdict-red：P2F=335 REDLINE fail-stop——F2P=689 池在案不落库（composition 六页 175＋sizing interp 251＋values calc-size 族 247）

时间：2026-11-22 21:4x 判读收账。电池：`knife-kc5r3-fame2b`（unit
zw-knife-kc5r3-fame2b，attempt1 rc=2＝PROBE-PAGE-IN-CORPUS 哨自愈
（zz-kc5r3-probe 残 css-sizing animation 子目录），attempt2 21:15:31
发射 → 21:43:21 EXIT rc=0；八腿全 rc=0 无 LEG-EMPTY）。

face：10 文件 patch `knife-kc5r3-fame2b-face.patch` md5
`745a6c4da8e8cf8e533349f715baa8dd`＝r2 face（75bae55e）复贴＋r3 四刀：
① apply_single_property 六长手委托 `style-system apply_property_value`
（animation.rs；parse_px 塌 0px 与 min/max 严格回落二臂）＝r2 calc-mix
P2F 主修；② part03.js ④ 门 sizing 六长手豁免（`_zwIsSizingLp`——px↔%
跨型对 replace 注册解杀）；③ γ 端点＝probe-r3c A/B 两臂全过判非独立
缺陷转电池复判；④ 遮蔽清理（interpolate text-indent 臂收窄＋K1 臂
width/height 移除）。b0/b1 `d2d1d594…`/`23b4b026…`（M2 异 md5 过；
b0-base 5c99ca42c＋staged ride 五件双臂等基＋es_module unstaged ride）。

## 判定

**REDLINE 触发（TOTAL P2F=335＞0）＝ fail-stop，face 不落库，tally
不动。** face 已剥（21:44，`git apply -R`；9 文件回 HEAD 逐件
diff-quiet 核证、part03.js 回 index＝peer DC-2 staged 态零扰动、ride
面原态），patch 留 .acceptance 作 r4 复贴源。F2P=689 全池挂 r4，随
落库结算（r2 池 153 → r3 实测 689，增益 +536）。

## RECONCILE 官方账（键级）

- css-contain：F2P=0 P2F=0。
- css-sizing：F2P=442 P2F=134——height-composition P2F 8＋interp 五页
  126（height 78＋width 27＋max/min 4/8/4/4）＋cis interp γ at(1) 1
  （r2 同键持续＝probe 定谳「非独立缺陷」在 corpus 面不成立，r4 需
  另修）；composition 六页 F2P=175（height 19/max-h 33/max-w 28/
  min-h 36/min-w 36/width 23，r2=134）；interp 面 F2P=251（min-h 81/
  min-w 76/max-w 38/max-h 28/width 28）；KC-4 复现 computed 13＋valid 3。
- css-values：F2P=247 P2F=201——calc-size-height-interp P2F 53/F2P 49、
  calc-size-width-interp P2F 44/F2P 73、interpolate-size 六 interp 页
  P2F 104/F2P 91＋composition 页 F2P 34＋which-value 1。
- css-transitions：F2P=0 P2F=0；ASYM ×2（animate-with-color-mix
  b0=3/b1=4＝b1 多收 1 案良性向、event-dispatch b0=12/b1=18＝收集数
  波动，均 completeness-asymmetry 协议挂起不计，r2 的 b0=30/b1=18
  dropout 形本轮反转为 b0 收窄＝纯收集波动带）。

## P2F=335 归因（r3 主发现＝过量修复形，与 r2 失服务形相反）

r2 的 calc-mix 中程塌 0px 已被 ① 修复实证（height-composition b0F=26
→b1F=15，19 增/8 残）；r3 的红不是「动画值错了」而是「动画通道开始
服务真值、expected 侧静态 gCS 还在 legacy 折叠」：

1. **interp 族 ~327 键**（sizing 126＋values 201）：apply 委托后动画
   通道真服务 auto/max-content/fit-content/calc-size() 关键字轴，WPT
   expected 侧经同页静态 gCS 旧折叠（height auto→0px、fit-content→
   max-content＋「max - content」带空格序列化）＝静态路径 used-value
   折叠 legacy 即根因；b0 双侧同折叠巧合对称过、r3 打破对称。
2. **height-composition % 基 8 键**：underlying 10% 静态侧折 0px vs
   LP compose 侧折 10px（at(0) expected 100px got 110px）；at(1) got
   20%＝端点 add 未加 underlying（r2 同形延续）。
3. **min/max 负外推钳 16 键**：④ 遮蔽清理移除 K1 臂后钳 0 语义未
   继承（from 0px→100px at(-0.5) got 10px/100px 怪形，expected 0px）。

## tripwire 与完整性

- css-contain 全零＋css-transitions 零位移（F2P/P2F 双零）＋
  no-interpolation 页零 WATCH＝域外零损，红全部落在 face 自身目标域
  （sizing/values interp＋composition）＝过量修复不外溢。
- 八腿 rc=0 齐 JSON；S1/S3 ride 零扰动；探针页哨自愈一次（attempt1
  rc=2）；漂移门过；per-page 双侧断言数对称（432/432）非截断伪影。
- det-triple 本轮不跑（无落库面；KC-6 沿用 r2 evidence
  2026-11-22-kc5r2-fame2b-verdict-red 附记三连 IDENTICAL 件）。

## 处置与 r4 方向

- face 剥除不落库；patch md5 745a6c4d 留 .acceptance；tally 不动；
  F2P=689/P2F=335 全部挂 KC-5-r4。
- r4 主修面＝**静态 gCS used-value 关键字折叠对齐**（style-system
  域，height/min/max auto、fit-content、calc-size() 轴的 used 折叠
  与序列化空格形；预期同时兑付 interp 族 ~327 键两通道对称）＋
  % 基同源（composition 8）＋端点 add 补 underlying 加算＋钳 0 语义
  移入委托臂（16）＋γ at(1) 端点服务（1，corpus 面重开）。
- 协调记录：本电池窗 resume-swarm 17 fork 在树（71d5eda5 作者＋
  538424 仲裁/监视＋2fa86ef0/d5853e37 谱系多 fork），仲裁与让位见
  NOTES-arb-2057-poison-clean-r3-yield.md；收账由 d5853e37 fork
  21:43 认领执行（剥除＋本 evidence＋master.md）。
