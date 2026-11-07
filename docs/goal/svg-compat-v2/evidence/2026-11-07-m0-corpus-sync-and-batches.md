# M0 — 语料续拉对账 + SMIL 分类账批次划分 + DC-4 数值门（2026-11-07）

DC-1 三件套落地轮：①语料 sync 对齐 pin（missing=0）+ 差集清单；②SMIL
分类账对照上游现势重跑基线并落批次划分；③DC-4 数值门按决策规则实测落定。

## 1. 语料 sync 对账（missing=0）

- 工具：`scripts/goals/44-svg-compat-v2.sh`（新建，41 号 sync 模式移植——
  git trees recursive 列 pin 树全量 blob → 缺失即拉 / blob sha 不符定向重拉，
  双源回退 jsdelivr→raw，旧文件替换成功前不删 #31）。
- pin 未动：`315976933870b34d6ea30e3f6643403edae678ba`（一期收口以来
  lib.sh 零改动，上游滚动差集 = 0）。
- 对账结果：上游 svg/ 扩展过滤面 **2,163 blob**，本地 sync 前 **1,444**
  （残缺窗口起点未考，目录时间戳 2026-10-27 04:31–04:47；一期入账 2,144
  + 后续扩展名面 = 2,163 口径），**新拉 719、hash 不符重拉 0、missing=0**。
  残存文件全部与 pin 一致（0 mismatch）——sync 后本地语料与一期收口时
  **字节级一致**。
- 新拉 719 构成：`.svg` 401 / `.html` 307 / `.js` 7 / `.headers` 2 /
  `.xml` 1 / `.css` 1；按目录 text 160、styling 126、struct 95、path 91、
  types 90、shapes 60、pservers 49、render 22、scripted 11、painting 9、
  其余 5。清单：本目录 `m0-missing-pre.txt`（719 行）、
  `m0-hash-mismatch-pre.txt`（0 行）。
- 新拉 `.html` 307 按通道 skip 规则甄别：**入通道 145 案**
  （path/interfaces 14 + path/parsing 15 等——一期 S4c pathData 面语料、
  styling/struct/pservers 散面）、无 harness 渲染断言页 149、secure
  context 11、iframe 2。通道案集与一期收口恒等（486 案，见 §2）。
- helper 补拉：`common/rendering-utils.js`（559 B，pin 同 rev）——一期
  「渲染域 helper 不补」决策在本 goal 解除：`waitForAtLeastOneFrame` 是
  SMIL 采样断言案等帧前置，缺失即 4 子 ReferenceError 假失败（补拉后
  2P/2F，见 §2 脚注）。已写入 44 号脚本保持可复现。

## 2. M0 基线（通道现势，2026-11-07）

`testharness-svg --json`（HEAD `83cef074a`，test-guard 4G/8G/2700s 包裹）：

- **486 案 1,704 子：1,186P / 478F / 40T = 69.6%**——分母与一期收口
  （探针剔除口径 1,704）恒等，分子 +9（一期收口 1,177P：11 Fail→Pass −
  2 Pass→Fail，翻转归因见 §4）。
- 分目录头部：types 641P/41F（94.0%）、path 169P/8F（95.5%）、
  animations 52P/224F/32T（16.9%）、interact 105P/30F、geometry
  32P/52F、text 5P/53F、其余小面合计 81P/69F/8T。
- coverage 对账口径（coverage-2026-11-02：svg topdir 分母 15,342 子、
  zw 486 文件 1,181P = 7.70%）与本基线差 +5P——恰为 helper 补拉解锁的
  empty-segments 2P + svgnumber/geometry 等主引擎演化面（§4）；DC-4 门
  以 coverage 口径 1,181P 为基点。

## 3. SMIL 分类账对照上游现势 + 批次划分

一期 M4 挂账定稿口径（199F + 29T）对照现势重分桶（错误消息 + 文件名
机制归簇；32T 案按文件名入桶，29 案挂 SMIL 时钟、3 案渲染交互不入门）：

| 批次 | 机制（Mission 对位） | 现势 Fail | 现势 Timeout | T 注册子折算 |
|---|---|---:|---:|---:|
| **B1 时钟值文法** | full-clock-value 解析（越界分秒合法性） | 7 | 0 | 0 |
| **B2 interval 模型 / begin-end 时序** | begin/end 列表解析、active interval 创建、repeatCount/repeatDur 链、beginElement 真语义 | 0（+时钟操作共享 6） | 8 | 8 |
| **B3 时钟原点 / seek** | setCurrentTime/pause 真语义、seek 事件、freeze rewind | 1（+时钟操作共享 6） | 8 | 8 |
| **B4 同步基 syncbase** | id.begin/end 依赖解析、cyclic 环 | 3 | 3 | 4 |
| **B5 事件基 + 动画事件时基** | beginEvent/endEvent/repeatEvent 派发、eventbase begin、custom events | 6 | 7 | 7 |
| **B6 采样引擎** | calcMode/from-by/values/additive/accumulate/keyTimes/keySplines 值插值与应用 | 124 | 3 | 6 |
| **B7 path 'd' 插值** | 路径插值依赖 B6 采样 | 33 | 0 | 0 |
| **小计（SMIL 时钟可修池）** | | **180 + 6 共享 = 186** | **29** | **33** |
| 挂账#2 getCTM 载体 | `getCTM is not a function` 18 + animateMotion 位置/bbox 断言载体 13 | 31 | 0 | 不入门 |
| 渲染/style 协同 | color 插值 getComputedStyle rgb 空 | 13 | 0 | 不入门 |
| 渲染交互 | currentcolor-visited/conditional-processing/slider-switch | 0 | 3 | 不入门 |
| animations 面 P（已收） | 接口存在性 + 脚手架释放 + helper 解锁 | 52P | | 已入账 |

- **SMIL 时钟可修池 = 186F + 33T 注册子 ≈ 213 子**（一期口径 199F + 29T
  案的现势重算：helper 解锁 −4 假失败、消息桶按机制展开、Timeout 按
  testharness 注册数折算 29 案 → 33 子）。
- 与一期分类账差异说明：一期 155「时序/值断言」消息桶按机制展开为
  B2–B6；getCTM 载体 18F 与一期 22F 同域（rotate-angle 7 案脚手架释放
  后从 T 转 F 计入口径差）；color 插值一期 7F 现势 13F（rgb 空串消息
  口径一致，主引擎演化 ±1）。
- 批次推进序（依赖链）：B1 → B2 → B3 → B4 → B5 → B6 → B7。B1–B5 是
  时钟模型本体（interval/事件/seek），B6/B7 采样消费时钟。M1 = 通道
  定型 + B1 + B2 首批；M2 = B3–B6；M3 = B7 + 滚动新面 + 挂账 #2/#3 归口。
- 实现落点预注（M1 细化）：shim 侧文档级 SMIL 时间线（part03.js
  `_zwSvgSvgCtlState` per-element {t, paused} stub 的合流点已预留
  FIXME）；WPT SMIL 断言案主消费面是
  `pauseAnimations()+setCurrentTime(T)+assert` 同步 seek 模式——时钟
  模型落 seek 驱动采样即可覆盖 B1–B6 断言面，真时间泵不进 compositor
  动画泵（Mission 划界）。

## 4. 一期收口 → M0 翻转归因（11 F→P / 2 P→F）

方法：一期收口 runner（worktree `fb065a33c` 构建）与 HEAD runner 对
**同一语料**（sync 后，helper 在册）各跑全通道，逐子测试转移矩阵
（common 1,704 恒等，两侧Timeout 集恒等 40）。

**11 Fail→Pass（主引擎演化红利，方向良性）**：

| 面 | 子测试 | 归因域 |
|---|---|---|
| svg/geometry-apis | rendered rect / visibility:hidden / g opacity=0 / g visibility=hidden 各 1（box 正确回报） | engine gBCR pending-style 反映代（90572cc78 2026-11-04、s201 ebdfe3939 2026-11-01 一代） |
| svg/styling | attr-style-type-dynamic ×2（style type 动态失效语义） | engine style 元素语义 |
| svg/types | SVGLength-px detached ems/q 换算 ×2 | engine 单位换算 |
| svg/scripted | script-invalid-script-type 1（合法 type 后执行） | engine script 语义 |
| svg/animations | svgnumber-animation-1/2（枚举动画断言） | 主引擎演化（F→P 方向良性，逐因未溯） |

**2 Pass→Fail（确定性回归，具名挂账）**：

- `svg/geometry/svg-geometry-apis-in-non-rendered-elements.html`
  「rect in `<g>` with CSS display:none has no box」getBoundingClientRect +
  getClientRects 两子——**隔离单案复跑同样 Fail（确定性，非次序偶发）**；
  机制：display:none 子树 gBCR 从零盒变为返回 10px 盒（layout 侧现对
  display:none 子树计算盒）。spec：display:none 无盒 → 空 DOMRect。
  归因域：engine gBCR 桥/布局（90572cc78 代的语义副作用，嫌疑未逐 commit
  bisect）；面归属：svg/geometry = 一期挂账 #2/#4（getBBox/几何上下文
  rendering-compat 协同批）——按划界本 goal 不双改，记入碰头归口清单，
  DC-4 复测前须由协同批清偿或具名转挂。

**次序敏感观察（非回归）**：`setPathData-during-animation` 1 子与
`use-load-error-events` 2 子在首轮全通道跑 Fail、干净复跑（两代 runner
一致）全 Pass——全通道次序/时序敏感（跨案状态或 rAF settle 抖动），
非代码回归；DC-4 复测若再现按观察记账不按回归。

## 5. DC-4 数值门落定

决策规则（立项即定）：数值门 = 地址池 × 一期同类簇通过率。

- **地址池**：SMIL 时钟可修池 ≈ **213 子**（§3 小计：186F + 29T 案按
  testharness 注册数折算 33 子）+ 滚动新面导入 0（pin 未动，语料
  restore 与一期恒等，无新面增量）。
- **一期同类簇通过率 = 70%**：双锚点实测一致——①一期 corpus 收口率
  70.3%（同通道同语料同类 DOM 面簇）；②goal-10 timing-animation-compat
  收口率 70.5%（同为文档级动画时间线/采样语义簇，同 runner 底座）。
- **数值门 = (1,181 + 213 × 0.70) / 15,342 = 1,330 / 15,342 = 8.67%，
  落定 ≥ 8.6%**（下取整保守；增益 ≥ +149 子）。
- 不入门簇（具名挂账，不入分子）：getCTM 载体 31F（rendering-compat
  协同）、color 插值 13F（style-system 协同）、渲染交互 3T、SVGLength
  上下文单位与 a.ping（一期挂账 #3/#6 原面）、display:none gBCR 回归
  对 2F（§4，协同批清偿后另计红利）。
- M3 滚动新面导入若扩池，按同一规则上调并留痕（门是下限）。
