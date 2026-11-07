# svg 二期 — SMIL 时钟模型专项 + 语料滚动扩面

**版本**: v1.1
**日期**: 2026-11-04（立项，coverage-2026-11-02 对账驱动，用户「按建议立项」）
**状态**: ✅ Completed——**M0–M4 全部完成（2026-11-07 收口，DC-1~4 全
✅；归档与 Rally 循环处置待用户判定）**：B3/B4/B5/B6a/
B6b/B6c/B7 七批 + M3 滚动收尾 + M4 coverage 复测全落档。收口批（M3 余项
+ M4）：语料 pin 前滚 goal 域 override（`315976933` → `28c152b74`，44 号
脚本域内重设，共享 pin 未动；子树 +329/94/2，missing=0，通道 486→554
文件 +9,682 子）+ 新面修复三件（getSimpleDuration fround——Blink SMILTime
float32 往返、animationsPaused per-element 暂停集——Blink 每容器独立
paused 消费面、repeatEvent seek 合并派发 repK 水位去重）svg 通道
+4F→P 零 P→F；DC-4 复测双口径收口：**冻结分母 1,355/15,342 = 8.83%
≥ 8.6% 达线**（交集 −5P 纯上游 getBBox-06/07 改写语料漂移，代码零回归），
滚动新面 +5,072P 另账，全量分母刷新待 wptfyi 周快照（独立管线不阻塞）；
挂账 #2/#3 归口落档（均归 rendering-compat 协同）、#6 缩减记账
（noreferrer iframe 化出通道）；web-animations constructor 域非确定
抖动注记。台账见
[evidence](svg-compat-v2/evidence/2026-11-07-m3-roll-m4-closeout.md)。
七批明细：B6a calcMode 四模式 + keyTimes/keySplines 文法
校验（comma-wsp 分隔）+ additive/accumulate 三明治 + 单位上下文
（em/ex/%/calc() 项级求值）+ restart=always 中途截断 + freeze 迭代边界
浮点钳制 + by-only replace underlying 起点，39F→P 零 P→F，
[evidence](svg-compat-v2/evidence/2026-11-07-m2-b6a-sampling.md)；B6b
`_sampleAnimStr` token 采样（角度域/数值列表域/不可插值离散 + from-by/
by-only 合成 + invalid-value 负向 + additive=sum 三明治）+ animVal 反射
消费接线（enum/bool/integer/string/angle/list/rect）+ get-trap target
分支 + `_chainOf` 跨族 id 回落，41F→P 零 P→F，
[evidence](svg-compat-v2/evidence/2026-11-07-m2-b6b-type-domain.md)；
B6c animateTransform 域（type 组件向量插值 + by-only 零向量起点 +
animVal 表项三明治——gradientTransform/patternTransform 清 base vs
transform 属性恒复合定谳）+ href 显式定靶面（三消费钩子 `_scanDoc`
按 `#id` 对齐，读时解析——change-target 改靶即时生效）+ 连带补缺
（result/kernelMatrix 反射、SVGTransform.angle 原始角度态、by-only
角度域合成）+ part09 2000 行准则切分 part09b，10F→P 零 P→F，全通道
1,329P = 77.7%、web-animations 959 子恒等，
[evidence](svg-compat-v2/evidence/2026-11-07-m2-b6c-transform-domain.md)；
B3-B5 见 archive 与 §里程碑；B7 path d 插值（EDM 整数 ms 相位精度模型
+ to-animation 线性 lerp 定谳 + SVGPathSeg 最小读消费面 + has trap
接口原型链兜底）31F→P 零 P→F、MmAaZ 1 文件残留定谳记账、web-animations
恒等，[evidence](svg-compat-v2/evidence/2026-11-07-m3-b7-path-d.md)。**
**卡点**: additive-type-by 1F 案级（B6c 采样断言面十一域全过后余卡
expectFillColor——getComputedStyle().fill = #4 门，跨域记账）；engine
渲染耦合回归集（svg/geometry = 挂账 #2/#4 协同域，按划界
不双改——DC-4 前须协同批清偿或具名转挂）：display:none gBCR 回归对 2F
（M0 翻转归因）+ geometry 再下滑 2 子（M0 快照后 main 演化，part09 在场
A/B 逐子 0 flip 归因非本 goal，嫌疑 4a5fd3051 声明式物化）。
B3/B4 后显形挂账面（案从 Timeout 转完成后真实断言显形，非 Pass 丢失）：
first-interval ×2 + escaped-dots ×2 + correct-events ×1 + slider-switch
×1（B5 新显形——事件链全通后 getComputedStyle display 不读 SMIL 表现值
；四案同挂账 #4）+ href-mpath ×2（挂账 #2 getCTM/路径引用邻接）+
animate-currentcolor-in-visited-link（渲染交互）+
syncbase-remove-add ×1（引擎查询层：doc-qSA 对动态 append
同步不可见 + wrapper 跨族 identity #32——B4 探针实证 17 子 P→F 已回退，
统一归一后 `_scanDoc` 重接活遍历）——并入 DC-4 前协同清偿/具名转挂清单
（event-listeners ×1 已于 B5 清账——onbeginEvent/onendEvent/onrepeatEvent
expando 门 + 保留名跳过）
**执行模式**: WPT 驱动（上游 svg corpus 为验收标尺）+ 专项语义收敛 + 语料扩面
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——SVG 面）
**一期**: `docs/goal/archive/svg-compat.md`（80 号，2026-10-22 收口，corpus
70.3%、七项挂账定稿）

## coverage 对账（coverage-2026-11-02 基线，全量头条 67.54%）

svg topdir 全量分母 **15,342 子测试 / 1,465 文件**，zw 覆盖 486 文件
1,181P = 7.70%，未覆盖 **14,161**。一期收口时 corpus 70.3% 入账、基线
8.8%——上游语料滚动 + 一期排除域（SMIL 时钟模型、渲染面）是缺口主体。

一期七项挂账定稿清单（master.md + evidence/2026-10-21-m4-smil-ledger-
and-closeout.md，本 goal 承接底稿）：

| # | 挂账项 | 体量 | 归口 |
|---|---|---:|---|
| 1 | SMIL 时钟模型（动画时序/同步/事件时基） | 199 子 Fail + 29 案 Timeout | **独立立项面——本 goal 即承接载体** |
| 2 | getCTM/getScreenCTM 族 | 22 子 | rendering-compat 协同排期 |
| 3 | SVGLength 上下文单位（cap/ch/ic/lh/rlh + viewport/percentage） | S2a FIXME 批 | 与 getBBox 几何上下文同批评估 |
| 4 | 文本几何 / image 内在尺寸 / CSS cascade 命中 / CSS d property | 既有挂账 | style-system/rendering 协同 |
| 5 | `.py` 动态端点（script-load-error-events 2F） | 2F | runner/infra 域 |
| 6 | a.ping / a.rel-\* | 小 | 导航域 |

关键事实：SMIL 分类账（199F + 29T 案级量化）是一期 M4 交付物——本 goal
M0 直接以分类账为底稿落批次，不需从零归因。

## Mission

1. **SMIL 时钟模型专项**（M1/M2）：一期分类账逐簇推进——时钟原点/
   begin-end 时序/同步基（syncbase、event-value、repeat-value）/动画事件
   时序语义；落 `zero-media`/engine 动画泵边界（SMIL 是 SVG 文档级时间
   线，不进 compositor 动画泵）。
2. **语料滚动扩面**（M0/M3）：FORCE 续拉对齐 pin + 一期收口后上游差集
   甄别；渲染面（reftest）不入账照旧，testharness 新面按簇导入。
3. **挂账 #2/#3 归口重估**（M3）：getCTM 族与 rendering-compat 碰头归口；
   SVGLength 上下文单位批评估结论落档。
4. **复测收口**（M4）：svg topdir 落数值门；一期已收口面零回归。

## Done Criteria

- **DC-1（M0 对账）**：语料续拉对齐 pin（差集清单 + missing=0）+ SMIL
  分类账对照上游现势的批次划分落 `docs/goal/svg-compat-v2/evidence/`；
  DC-4 数值门按决策规则落定并回填本文件。
- **DC-2（SMIL 通道）**：SMIL 面入常驻通道（runner 子命令或 svg 通道
  扩面，M0 定）+ skip 规则 + Makefile target；分类账基线复核落档。
  **✅（2026-11-07 M1 定型）**：svg 通道承载（`animations/` 一期即无目录
  排除，skip 规则 `svg_case_skipped` + Makefile `testharness-svg` 均在，
  不设独立子命令），见 M1 evidence §4。
- **DC-3（语义收敛）**：SMIL 批次簇逐批收敛或具名挂账（每批 evidence：
  FIXED/REGRESSED 记账）；挂账 #2/#3 归口结论落档。
- **DC-4（coverage 复测）**：svg topdir 从 7.70% 升至数值门；一期已收口
  corpus 账逐 subtest 零回归；头条不回落。
  **✅（2026-11-07 M4 收口）**：冻结分母 8.83% ≥ 8.6% 达线、svg 通道
  零 P→F；分母刷新待 wptfyi 周快照（§M4 台账）。

**DC-4 数值门决策规则（立项即定）**：SMIL 分类账地址池（199F + 29T 中
定性可修簇）+ 滚动新面导入即得，两项合计为地址池；数值门 = 地址池 ×
一期同类簇通过率（M0 实测后填入）；渲染级耦合簇不进门，具名挂账。

**DC-4 数值门落定（2026-11-07，M0 实测）**：地址池 ≈ **213 子**
（SMIL 时钟可修池 186F + 29T 案按注册数折算 33 子 + 滚动新面导入 0）；
一期同类簇通过率 = **70%**（一期 corpus 收口 70.3% 与 goal-10
timing-animation-compat 收口 70.5% 双锚点）；**数值门 = svg topdir
≥ 8.6%**（(1,181 + 213×0.70)/15,342 = 8.67%，下取整保守；增益 ≥ +149
子）。测算底稿见 M0 evidence §3/§5。

## 划界

- **渲染面 reftest 不入账**：svg reftest 归 rendering-compat（一期口径
  不变）。
- **动画泵边界**：SMIL 时钟模型落文档级时间线语义，与
  rendering-compat/compositor-animation 的 CSS/WAAPI 动画泵不混写；
  共享时钟原点改动须双方控制面留痕。
- **挂账 #2/#4 渲染耦合项**：getCTM/getBBox/文本几何归 rendering-compat
  协同，本 goal 只做归口结论不双改。
- **`.py` 端点（#5）与 a.ping（#6）**：分别归 runner/infra 与导航域，
  不进本 goal DC。

## 里程碑

- **M0**：语料续拉 + 分类账批次划分 + 数值门（DC-1）。**已完成**
  （2026-11-07：语料 sync missing=0 新拉 719/hash 不符 0、批次 B1–B7
  落档、数值门 ≥8.6% 落定回填；台账见
  [evidence](svg-compat-v2/evidence/2026-11-07-m0-corpus-sync-and-batches.md)）。
- **M1**：SMIL 通道落地 + 时序基簇第一批（DC-2 起步）——B1 时钟值文法
  （7F）+ B2 interval 模型首批。**已完成**（2026-11-07：part09.js 落地 +
  DC-2 通道定型 + B1 收敛 7F→0 + B2 定谳面 P；全通道 1,209P/468F/31T
  零回归、web-animations 506P/959 复验零回归；台账见
  [evidence](svg-compat-v2/evidence/2026-11-07-m1-smil-channel-b1-b2.md)）。
- **M2**：SMIL 同步基/事件时基簇收敛（DC-3 主体）——B3 seek / B4
  syncbase / B5 eventbase / B6 采样引擎。**已完成（B3 + B4 + B5 + B6a +
  B6b + B6c 六批全收口）**
  （2026-11-07：B3 seeking-events-1..8 全 P + F→P ×6 采样连带收益；B4
  时间序离散事件模拟落地 4T→P——cyclic-syncbase-2/events、
  invalid-attribute-type、syncbase-after-removal；B5 事件基动态实例源 +
  处理器名错位映射 8F→P + 4T→P；B6a calcMode/keyTimes/keySplines/
  additive-accumulate 三明治/单位上下文/restart 截断/冻结钳制 39F→P；
  B6b token 采样器角度域/列表域/离散域 + animVal 反射接线八处 + get-trap
  target 分支 + `_chainOf` 跨族 id 回落 41F→P；B6c animateTransform type
  域组件向量插值 + animVal 表项三明治（gradientTransform/patternTransform
  清 base vs transform 属性恒复合定谳）+ href 显式定靶面（读时解析改靶
  即时生效）+ result/kernelMatrix 反射 + SVGTransform.angle 态 10F→P；
  六批均零 P→F、web-animations 恒等；台账见
  [B3](svg-compat-v2/evidence/2026-11-07-m2-b3-seek-events.md) /
  [B4](svg-compat-v2/evidence/2026-11-07-m2-b4-syncbase.md) /
  [B5](svg-compat-v2/evidence/2026-11-07-m2-b5-eventbase.md) /
  [B6a](svg-compat-v2/evidence/2026-11-07-m2-b6a-sampling.md) /
  [B6b](svg-compat-v2/evidence/2026-11-07-m2-b6b-type-domain.md) /
  [B6c](svg-compat-v2/evidence/2026-11-07-m2-b6c-transform-domain.md)）。
- **M3**：滚动新面导入 + 挂账 #2/#3 归口（DC-3 收尾）——B7 path d 插值。
  **已完成**（2026-11-07：B7 path d 四形态采样 + EDM 精度模型 + to-anim
  lerp 定谳 + SVGPathSeg 消费面 + has trap 兜底，全通道 1,360P/343F/7T
  零 P→F，台账见
  [evidence](svg-compat-v2/evidence/2026-11-07-m3-b7-path-d.md)；滚动
  收尾 pin 前滚 goal 域 override missing=0 + 通道 486→554 文件簇归口 +
  新面修复三件 +4F→P 零 P→F + #2/#3 归口（归 rendering-compat 协同）+
  #6 缩减记账，台账见
  [evidence](svg-compat-v2/evidence/2026-11-07-m3-roll-m4-closeout.md)）。
- **M4**：coverage 复测收口（DC-4）。**已完成**（2026-11-07：冻结分母
  1,355/15,342 = 8.83% ≥ 8.6% 达线——交集 −5P 纯上游 getBBox-06/07
  改写语料漂移，代码零 P→F；滚动新面 +5,072P 另账；全量分母刷新待
  wptfyi 周快照，独立管线不阻塞；台账见
  [evidence](svg-compat-v2/evidence/2026-11-07-m3-roll-m4-closeout.md)）。

## 依赖约束（run-rules §9）

- svg 通道（80 号已收口面）每批次逐 subtest 零回归复验；SMIL 时序改动
  额外复验 web-animations/timing 通道（timing-animation-compat 已收口
  台账）不受文档级时间线改动波及。
- 与 rendering-compat（活跃 rally）交界：getCTM/几何上下文/动画泵共享
  面先碰头归口再动，双方控制面留痕。
- 上游滚动走 pin 对齐 + blob-hash 对账（goal-35/80 先例）。
- SMIL 长动画案（Timeout 面族）跑 runner 时按 test-guard 内存与墙钟
  口径包裹，防无人值守超预算。
