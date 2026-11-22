# KC-5-r2＋fame2b 电池 verdict-red：P2F=2 REDLINE fail-stop——F2P=153 大面收益在案（interp 21F→1F＋composition 134/235）

时间：2026-11-22 20:2x 判读。电池：`knife-kc5r2-fame2b`（unit
zw-knife-kc5r2-fame2b，19:58:43 attempt3 发射 → 20:25:54 EXIT rc=0；
attempt1 19:53 rc=3＝CRATES-DIRTY 门拒 peer staged 新四件、attempt2
19:56 rc=7＝s1/s2 漂移门结构性伪漂移（part03.js 双刀驻留 s1 含 staged
态、s2 剔 face 后不可比）＝两形皆脚本门修正后重发，非 face 问题）。

face：10 文件 patch `knife-kc5r2-fame2b-face.patch` md5
`75bae55ea696daaeeee42523b6e4a0df`＝r1 face 9 文件原样复贴＋r2 两刀：
① part03.js ④ 门 contain-intrinsic 前缀豁免（KC-5-A shim 刀）；②
fame2b（part03.js sizing 六长手 `_zwIsLp` 入引擎合成域＋animation.rs
域门六属性＋端点预合成换道＋`parse_lp_terms`/`render_lp_terms`
compose LP 加法臂＋interpolate LP 插值臂）。b0/b1 二进制
`360a8fd2…`/`25dea9c9…`（M2 异 md5 过；b0 两次构建 byte-identical
确定性实证）。

## 判定

**REDLINE 触发（TOTAL P2F=2＞0）＝ fail-stop，face 不落库，tally
不动。** face 已剥（9 文件 checkout 回 HEAD；part03.js checkout 回
index＝peer DC-2 staged 态，peer 面零扰动），patch 留 .acceptance 作
r3 复贴源。r3 迭代直接在 patch＋残键三簇上续作。

## 电池完整性

- 八腿全 rc=0（20:00:35 起 ~2 分钟/腿，transitions 腿 ~6.5 分钟）；
  四 b0＋四 b1 JSON 齐落盘无 LEG-EMPTY。
- serialize 门过（peer 的 dom runner 在跑但不含本 gate 三探测任一）；
  RIDE=es_module.rs 单件 unstaged＋staged ride 四件（peer DC-2：
  bridge.rs＋shim part04/05/06）＝**双臂等基**（strip 只剥本流
  unstaged face，staged 双臂常驻）——b0 溯源＝HEAD＋DC-2 rides，
  面差仍严格隔离本 face；S1/S3 ride md5 零扰动；探针页哨过
  （四语料 zz-* 发射前移除 9 件）。
- 中窗漂移门 attempt3 形过（期望 ride 集＝s1 剔 face 派生，修复
  r1 形制在双刀驻留下的结构性伪漂移）。

## RECONCILE 账面（官方）

- css-contain：F2P=0 P2F=0。
- css-sizing：F2P=151 P2F=2 —— **REDLINE 源**。
- css-values：F2P=2 P2F=0。
- css-transitions：F2P=0 P2F=0；ASYM ×1（event-dispatch b0=30
  b1=18——采收集 dropout 恶化形，按完备性不对称协议挂起不计；
  r1 的 animate-with-color-mix ASYM 本轮消失＝transient 定谳）。

## 大面收益（r2 face 实测，r3 随落库兑现）

- **interp 页 21F→1F**：KC-5-A shim ④ 豁免修复 α（轴数展开）＋β
  （none/auto 关键字对）＋γ 大部——contain-intrinsic-size-
  interpolation.html 432 键 431 真过（r1=411），唯余 γ at(1)。
- **composition 六页 134/235**：height 26F→5F、max-height 41→14、
  max-width 36→10、width 44→21、min-width/min-height 44→26×2
  ＝块1（均匀 add 纯 px）＋块2（负 lerp 钳 0 后置）＋块4（混合 op）
  全翻绿＋块5 部分。
- computed 13F→0F＋valid 3F→0F 复现（KC-4 面）；values
  interpolate-size-width-composition 30F→28F 增益 2 键。
- 域外零回归：css-contain/css-transitions 全零，values P2F=0。

## P2F=2 归因（r3 三簇定界）

1. **γ at(1) 端点服务**（cis interp P2F）：`from neutral to
   [20px 10px] at (1)` got 静态 underlying「50px 60px」——单 keyframe
   （仅 offset:1）动画 progress 恰 1.0（steps(1,start) eased=1.0）
   时动画值未服务回落静态；同组 at(-0.3)/(0.3)/(0.6)/(1.5) 真插值
   全过＝注册/采样链已通，纯端点服务边界。r1 同键同形，非 r2 新增。
2. **calc-mix apply 路不对称**（height-composition P2F＋残 ~19 键）：
   at(0) 键 `underlying [10%] from add [100px] to add [20%]` 期望
   「100px」（expected 侧静态 gCS 把 calc 折纯 px）got「0px」＝
   预合成帧 `calc(100px+10%)` 经**动画应用路径**塌缩 0px（height/
   width 形）；**同键 WAAPI 通道 b1 真 Pass**（JSON 实证）＝CSSA 败
   WAAPI 过的通道不对称；height 残族 at(-0.3)/(0.5) 双通道同败。
   min/max 四属性同族键 got「10%」（underlying 原样）＝动画值未应用
   回落。静态声明路径 vs 动画应用路径对 calc-mix 的消费不对称＝r3
   主修面（引擎 tick→apply/gCS 链）。
3. **块5 auto 端点离散残**（min/max 页 ~14 键）：`add [100px] to
   add [auto]` ≥0.5 期望 auto got 合成 px＝离散切换未服务；expected
   侧 gCS(auto) 产出「auto」本引擎实证（r1 记录「0px」实为 **height
   形**勘误——height gCS(auto) 折「0px」与期望侧对称巧合致 height
   auto 块双通道全过；min/max auto gCS 产「auto」两侧对称）。

## 残留 102F 键级普查（六页 composition，b1，reconverge 补录）

calc-mix 块每页 ×10（CSSA/WA 各 5）＝60；auto 端点 min-width/min-height
×6 each＝12；min/max-content 块 min-width ×10＋min-height ×10＋
max-height ×4＝24；height calc-mix 残 ×5（含 P2F at(0)）＋width 页
杂 ×1＝102。全族一个根因＝动画值应用路径对非纯 px 值失能（纯 px
全过反证采纳/采样链主体在位）；min/max-content 族＝不可加 underlying
形（块6 tripwire 判据兑现＝域门挡下回落 replace，b0 同值保持未误伤）。
杂项增益在案：width-interpolation 单 %-对键 F2P ×1（LP 插值臂首兑现）。

## 处置

- tally 不动；F2P=153/P2F=2 全部挂 KC-5-r3，随落库结算。
- r2 判据（零位移＋got 非空）对 431 真过侧成立（interp 页 got 全
  真值非空洞）；b0 432 空洞过基线继续有效。
- det 三连（KC-6 副判据，computed 页 b1-runner 三连失败键集恒等）
  已跑（20:27，剥 face 后按二进制态）：三连全 0F 键集恒等＝
  **DET-TRIPLE IDENTICAL**，KC-5-B cascade 三槽确定性合并稳定性
  硬证入案（.acceptance/wptfyi-css/results-kc5r2-fame2b/det-triple/）；
  KC-6 主线仍系本 face 落库前置。

## 附记：b1-runner 存档探针定谳（20:4x，fame2c 探针 20 用例，剥 face 后追证）

探针页 `zz-fame2c-probe.html`（已移 .acceptance/probe-pages/）＋归档
b1-runner `25dea9c9…`（树态无关）实拉，三件硬证补强 r3 归因：

1. **静态 apply 全通（正 calc-mix）**：`calc(100px+10%)`/`calc(130px+4%)`/
   `calc(50px+20%)` setProperty→gCS 全折纯 px 真值（100px/130px/50px）＝
   簇 2「不对称」精确定位在**动画应用路径**（非 apply 门、非串化序）；
   负 px 形 `calc(-50px+40%)` 静态即拒（回落 inline 10%→gCS「0px」）＝
   外推键（at≥1.5）独立拒收面。
2. **端点命中 vs 中程采样的分叉**：add 通道预合成帧在 eased=0 端点
   verbatim 服务真值（100px 实证），eased∈{-0.3, 0.5} 中程插值串全塌
   0px＝动画路径对中程 calc-mix 串的消费失能（端点快径不过同一消费点
   ＝r3 修位在中程采样→应用链）。
3. **④ 门整族杀 REPLACE 跨型对（新发现，扩簇 3）**：replace
   `[100px→20%]` 双帧动画**全 at 死**（含 offset 显式、端点 verbatim
   形、CSSA 无合成形 C4 全 0px）＝④ 整串单位 multiset 门对 px↔% 跨型
   对整动画拒登记（非 cis 豁免域）。r2 账面 width-interpolation 仅 1
   %-对键 F2P 的真因＝其余 %-对键全被 ④ 杀在注册前——LP 插值臂兑现
   池远大于 1（六 interp 页 %-对 + 关键字对残 ~150 键量级），r3 ④
   豁免域扩展（跨型可插值对 admission）后走 LP 臂直兑。
4. CSSA add 通道 at(0) 死而 WAAPI 同帧活（C2 vs W3）＝cssa→engine
   composite/underlying 传播与 js_anim 通道不对齐（簇 2 min/max 页
   got「10%」回落即此通道死形），r3 连带核对 cssa_registry 组装面。
