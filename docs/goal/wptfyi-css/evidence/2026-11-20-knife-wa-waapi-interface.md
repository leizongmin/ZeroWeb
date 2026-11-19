# 刀 WA：池④ WAAPI interface 簇 face——CSSAnimation 对象语义（2026-11-20）

## 池与 face

池＝residue survey 批三候选④ WAAPI interface 簇（animations+transitions 双语料，
03:5x 修复后 JSON 精盘 ~245F shim 对象语义面）。face＝四 shim 文件：

- `part03.js` +541：`_zwCssAnimationsForElement` 槽位注册表（(name, 同名自尾序 r)
  认领 + 新表项序输出，dynamic-changes 五案律——trim 幸存第二槽/interleave
  [A2,nb,A1]/旧 startTime 相等保持等）+ `_zwCssAnimFramesFor` 关键帧构形
  （css-animations-2 §keyframes：offset 分层 (easing 等价类, composite) 分组合并、
  隐式端点 replace 帧底层值、部分完整端点层内补底）+ 探针 gCS canonical 化 +
  var() 借 computed custom properties 解析。
- `part04.js` +10：Element.getAnimations CSSAnimation 浮出挂接（JS 动画/
  CSSTransition 之后）。
- `part06.js` +17：Document.getAnimations CSSAnimation 浮出挂接。
- `part05.js` +4（本刀 hunk）：AnimationEvent.animation 接口行
  （css-animations-1 §AnimationEvent init dict 第 4 属性）——接口面先行，
  事件派发属池④b。

## 电池代际与 canonical 定位

发射窗四连作废后收敛：knife-waapi b0(03:24)/b1(03:49) 预账 gained=94 属 face
03:32 修复前代（occ-自首序+prepend 旧设计，错案对质后弃）不入账；b2/b3 两轮
strip 电池死于 results-knife-wa 并发删除（墓地路径宣告）。canonical＝**r3 电池**
（zw-wa-r3，knife-wa-battery.sh，04:26:06 发射）：

- b0 `5cc27980`（face 剥除，53s）/ b1 `764b081d`（face 入树，47s）M2 异 md5 过；
  face 标记 `_zwCssAnimationsForElement` b1×7 / b0×0。
- S3-MISMATCH 仲裁：ride census part10-typed-om.js 428004bf→f8152a67 窗内漂移
  （peer typed-om 流 WIP），二进制切片法证 b0/b1 内嵌 part10 区段 md5 恒等
  ＝两臂同吃同代，**A/B 对称清洁，账可收**（peer gen-2/gen-3 不入本刀账）。
- RIDE 15 文件（收账核零扰动）。
- r3 04:44:50 被外部 stop（journal "Stopping"，RuntimeMaxUSec=infinity）：
  b0 六腿全 + b1 css-animations 全，b1 其余五腿缺失。**补全**＝冻结 b1-runner
  （md5 门 `764b081d`）补跑 5 条缺失腿（兄弟 session 04:49:33 先手执行，本 fork
  同案链让位零双写）——同一二进制对的完整 12 腿账，无重建无代际漂移。
  AnimationEvent 行双侧 ride parity（其归因按 A 03:56 声明 + part05 在树实证
  注记，无独立 A/B 数字）。

## 主靶对账（css-animations b1−b0）

`TOTAL: gained=73 lost=0 regressed=1 pages_gain=17`：

| 页 | ΔP |
|---|---|
| AnimationEffect-getComputedTiming.tentative | +19 (2→21) |
| AnimationEffect-updateTiming.tentative | +1 (0→1) |
| CSSAnimation-animationName.tentative | +1 |
| CSSAnimation-id.tentative | +1 |
| CSSAnimation-pausing.tentative | +3 |
| CSSAnimation-playState.tentative | +3 |
| CSSAnimation-ready.tentative | +1 |
| CSSAnimation-startTime.tentative | +1 |
| Document-getAnimations.tentative | +2 (1→3) |
| Element-getAnimations-dynamic-changes.tentative | +4 (1→5) |
| Element-getAnimations.tentative | +14 (2→16) REGRESSED=1 |
| KeyframeEffect-getKeyframes.tentative | +16 |
| KeyframeEffect-setKeyframes.tentative | +2 |
| KeyframeEffect-target.tentative | +1 |
| animationevent-interface.html | +1 (18→19) |
| pending-style-changes-001.html | +2 |
| sample-on-last-keyframe.html | +1 |

真回归 1 条（挂残面不阻塞收账）：`getAnimations for CSS Animations that have
finished`（Element-getAnimations P 2→16 页内 P→F）——face 槽位注册表未做
finished 动画 relevance 过滤（web-animations-1 §finished 动画不再 relevant，
getAnimations 不得返回）。

## 哨兵与 ride parity（补全后全绿）

补全腿（冻结 b1-runner，04:49:33-05:02，与 sv2-battery3 腿并发＝timing 窗噪
由零差异结果排除）落定后，全量对账 `TOTAL: gained=73 lost=0 regressed=1
pages_gain=17` 与 css-animations 单语料对账逐字节一致——五哨兵全数零位移：

- transitions（getAnimations 共享 trap 面）：gained=0 lost=0 regressed=0。
- fonts（ride parity）：零差异。
- css-properties-values-api / css-typed-om / css-shapes（零差异哨兵）：全零。

补全后的完整账目录＝`results-knife-wa-b-killed-0447/`（r4 fork 05:01 park 时
保全，shapes 腿在 park 瞬间在跑、写完已移 inode 后字节计数与 b0 shapes 恒等）。
监视器一度误报 "wipe" 实为 park mv；launch 方（r4，zw-wa-r4 05:01:50 全量重跑）
声明以 r4 为其收账口径，本证据以 killed-0447（同一二进制对完整 12 腿）为
canonical，r4 GATE-DONE 后逐页对齐作独立确认。

## 收账与落库

- face 落库 `e7a1334b3`（part03/04/06 全量 + part05 AnimationEvent hunk，
  pathspec 精确 4 文件 +571/−1；DOMMatrix translateSelf/scaleSelf 4 行他流
  ride 未卷入）。AnimationEvent 行双侧 ride parity，无独立 A/B 数字，按
  03:56 完账声明 + part05 在树实证入账。
- guard 扫描 PASS（4 文件 571 新增行，零安全发现零 advisory）。
- 残面 131F 见下节，r4 独立确认窗与 sv2-r2b 纯账重跑由 r4 fork 排程。

## 残面（b1 态实测 131F，后续刀候选）

- AnimationEffect-getComputedTiming 残 22F + updateTiming 残 4F（全集/迭代缺口）。
- 事件派发面（池④b，接口面已先行）：animationevent-interface 残 23F +
  animationevent-types/pseudoelement/marker 5F——AnimationEvent 构造/派发通道。
- CSSAnimation 生命周期残：canceling 9F + effect 8F + getCurrentTime 2F +
  finished 1F + compositeOrder 2F（finished 过滤同根因）。
- getAnimations 族残：Document 15F + Element 5F（含上述 finished 回归）。
- KeyframeEffect 残：getKeyframes 16F（composite/fill 变体构形）+ target 2F。
- CSSTransition 生命周期 6 页 29F 本刀零位移（对象面另一刀域）。
