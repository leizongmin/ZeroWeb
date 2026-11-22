# KC-5 电池 verdict-red：P2F=21 REDLINE fail-stop——四件套主体在位（411/432 真过），残 21 键全 WAAPI 通道三簇

时间：2026-11-22 18:2x 判读。电池：`knife-kc5-cis-interp-real`（unit
zw-knife-kc5-cis-interp-real，17:55:25 发射 → 18:24:12 EXIT rc=0）。face：
9 文件 689 行 patch b622ab393faf734d6b5a51fd579a7936（KC-4 六件 41693c8d
复贴＋animation.rs CisAxis 文法 strict/lenient 双臂＋apply 写臂＋
transition_registry 白名单 89→92＋cascade.rs 三槽确定性合并 KC-5-B）。

## 判定

**REDLINE 触发（css-sizing P2F=21＞0）＝ fail-stop，face 不落库，tally
不动，KC-6 接力前置（树净）未满足不发射。** face 剥回 HEAD（checkout
9 文件），patch 留 .acceptance 作 r2 复贴源。

## 电池完整性

- 八腿全 rc=0（contain/sizing/values/transitions × b0/b1，17:57:46 起四腿
  顺次，尾腿 transitions b1 18:17:15→18:24:12）；四 b0 JSON＋四 b1 JSON
  齐落盘，无 LEG-EMPTY。
- serialize 门自斥过；S1/S3 ride 零扰动（md5 双核，RIDE=es_module.rs 单件
  已知 WIP）；FACE-RESIDENT 反向 check 过；PATCH md5 门过。
- 首行 `/proc/248410/cmdline` 探针残痕＝良性（前一序列窗 PID 复用探测）。

## RECONCILE 账面

- css-contain：F2P=0 P2F=0。
- css-sizing：F2P=16 P2F=21 —— **REDLINE**。
- css-values：F2P=0 P2F=0。
- css-transitions：F2P=0 P2F=0；WATCH event-dispatch b0F=17→b1F=16；
  ASYM ×2（animate-with-color-mix b0=5 b1=4、event-dispatch b0=17 b1=16）
  ——两侧断言数不等，按完备性不对称协议**挂起**（无 F2P/P2F 净额，不入
  账不判增益；疑 b1 单键截断/缺失，r2 电池复核）。

## interp 页（contain-intrinsic-size-interpolation.html，432 键）判读

预注册验收＝b1 真过零位移（PREP-kc5 判据；b0 432 双空串空洞过不可作回
满基）。实测 **b1=411 真过＋21 Fail**（P2F=21 位移非零）→ REDLINE。

411 真过侧证据：expected 侧 gCS 产真值（neutral 组 at(0.3) 期望 41px 45px
等），Pass 键 assert 通过＝双侧真值匹配非空洞；transitions 四通道全过、
CSSA 全过、WAAPI px 对组全过（含逐轴钳 0 at(1.5)=5px 0px、负外推
at(-0.3)=59px 75px）＝四件套主体在位。

21 残键**全部 Web Animations 通道**，got 恒＝静态 underlying「50px 60px」
（applyUnderlying 内联值）＝动画未生效回落静态，三簇：

- **簇 α（×6）端点轴数展开**：`from [0px 0px] to [10px]`——单值简写须
  逐轴复制（10px→10px 10px）后 lerp；期望形含折叠串（expected 侧 "0px "＝
  gCS 等轴折叠）。CSSA/transitions 通道同形键全过＝插值臂能处理展开，
  WAAPI 通道在采样前即未采纳（canonicalize/登记层拒）。
- **簇 β（×13）none/auto 关键字端点**：`from [none 15px] to [20px 15px]`
  ×6＋`from [none 15px] to [auto 20px 15px]` ×7——期望＝0.5 离散翻转形
  （＜0.5 from、≥0.5 to）。CSSA/transitions 通道同形键全过（离散翻转在
  位），WAAPI 通道全 at() got 静态＝未采纳（≥0.5 应得 to 值亦未达）。
- **簇 γ（×1）neutral 端点 progress-1.0 服务**：`from neutral to [20px 10px]`
  仅 at(1) 败（同组 at(-0.3)/(0.3)/(0.6)/(1.5) 真插值全过＝动画已注册已
  采样），got=静态 underlying——单 keyframe（仅 offset:1）动画在
  progress 恰 1.0（steps(1,start) 滤波输出 1.0）端点服务回落静态值，非
  注册问题，是端点服务边界 bug。

r2 迭代修位（假设，待读码定谳）：簇 α/β 指向 js_anim 通道采纳门
（js_anim_registry.rs:414 canonicalize ser0/ser1 比较＋strict 登记门对
关键字/展开形的拒绝路径）——CSSA 通道能采同形端点，WAAPI 采纳门与
CSSA 采样通道的端点预处理不对齐；簇 γ 指向单 keyframe WAAPI 效果
progress-1.0 分支（effect 尾相/finished 判定回落静态）。

## F2P=16（KC-4 挂账面回收方向，落库时随 face 入账）

- computed 页 13F→0F（含 auto 1px/none/inherit 全谱）＝gCS 聚合臂
  ＋decl fold 兑现。
- valid 页 3F→0F（5px 5px / auto 1px auto 1px / none none）＝CSSOM
  set 通路兑现；b0 基线 3F 确认（预注册 tripwire 基）。
- 两页 b1 0F 有 cascade.rs KC-5-B 三槽确定性合并 hunk 在 face 内支撑
  （face.patch :305-:409 实证），但 CLAIM-kc6 det 三连（同 binary 三连
  失败键集恒等）未跑——r2 PRE-LAUNCH 补跑后 computed 稳定性才算硬证。

## 账目处置

- tally 不动；F2P=16/P2F=21 全部挂 KC-5-r2 迭代后随落库结算。
- b0 432 空洞过基线继续有效（pre-registered）；r2 验收不变＝零位移＋
  got 非空。
- ASYM ×2 挂起不计。

## 正交发现（fame2b 前置，本 verdict 存证）

height-composition 六页 b0/b1 逐键同值（235F/101P）＝KC-5 face 对
css-sizing composition 页零扰动；b0 44F 形证明 gCS width 无 face 态产
真值＝KC-4/KC-5 聚合臂揭罩风险不成立。fame2b 序定谳：断言通道
double-gCS（interpolation-testcommon.js:449-459 expected 侧同过 gCS），
term 序两侧抵消，chrome 158 六页 60/60 全 PASS 实证——face 无需碰
`calc_computed_to_css` 序，文件集收敛 animation.rs 单文件；详见
.acceptance/wptfyi-css/PREP-fame2b-lengthpct-add-composition.md 附记 B。
fame2b 前置（KC-5 落库树净）被本 REDLINE 阻，继续备料态。
