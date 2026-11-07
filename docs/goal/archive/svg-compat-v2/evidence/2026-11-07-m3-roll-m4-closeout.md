# M3 收口（滚动新面导入 + 挂账归口）+ M4 coverage 复测收口（2026-11-07）

M3 余项两件一次收口：语料 pin 前滚（goal 域 override）+ 69 新通道文件
导入与簇归口；三个新面修复（getSimpleDuration fround、animationsPaused
per-element 暂停集、repeatEvent seek 合并派发）全绿。M4 DC-4 复测按
双口径收口：冻结分母 1,355/15,342 = 8.83% ≥ 8.6% 达线，代码零 P→F
实证；滚动新面增量另账 +5,072P（本地实测），wptfyi 周快照未到新 pin
——全量分母复测待快照刷新（独立管线，不阻塞本 goal）。

## 1. 滚动导入台账

- **pin 前滚**：`315976933870b34d6ea30e3f6643403edae678ba`（lib.sh 共享，
  一期起从未 bump）→ `28c152b74f8679534e558745fe83312c8dcca8c8`（upstream
  master @ 2026-11-07）。**44 号脚本 goal 域 override**（source lib.sh 后
  重设 `WPT_REV` + `RAW_ROOT`——RAW_ROOT 在 lib.sh source 时嵌入 rev，
  须同步重推）；共享 pin 未动：22 脚本共用、一次前滚即全 goal 域语料
  事件，越出本 goal 范围，其余 goal 后续各自按先例滚动。
- **子树 diff**（git trees recursive 清单对照）：added 329 / removed 2 /
  changed 94。语料 sync 走 44 号脚本（pin 树清单逐文件对账 + blob sha
  复算）：新拉 329 / 重拉 0 / **missing 0**（上游 2,490 blob 全量对账
  absent 0 mismatch 0）。同步须代理（`~/use-proxy-local.sh`）+ 认证
  （`gh auth token` 组装 GH_API_AUTH）。
- **通道扩面**：`svg_case_skipped` 内容规则自动发现，486 → 554 文件
  （新增 69 − 出通道 1：`a.rel-noreferrer-policy.html` 上游 iframe 化
  触发 iframe 排除规则，见 §4）。

### 1.1 新面簇表（69 文件 / 9,682 子 = 5,072P + 4,609F + 1T）

| 簇 | 文件 | 子测试 | P | F | 归口 |
|---|---:|---:|---:|---:|---|
| interact/hittest 矩阵 | 6 | 9,015 | 4,794 | 4,220 | 单文件 `pointer-events-fill-stroke-hittest.html` 8,911 子（4,698P/4,212F/1T）——pointer-events 命中矩阵，渲染域（paint-order 命中），归 rendering-compat |
| types（list-identity/getPathSegmentAtLength/URLUtils 邻接） | 15 | 354 | 219 | 135 | SVGList mutator identity + DOM 反射面，部分属后续批次 |
| linking（URLUtils/a-hyperlinkutils） | 4 | 87 | 13 | 74 | a.ping/导航域邻接（挂账 #6 同域） |
| styling（attributeStyleMap/css-var-dom） | 4 | 68 | 0 | 68 | CSSOM 属性面——attributeStyleMap 未实现，style-system 域 |
| text（getStartPositionOfChar/getRotationOfChar） | 10 | 47 | 5 | 42 | 文本几何，挂账 #4 同门 |
| struct（mixin 反射） | 11 | 43 | 19 | 24 | SVGElement mixin 面部分可修 |
| path（getPathSegmentAtLength） | 7 | 27 | 9 | 18 | B7 SVGPathSeg 消费面邻接 |
| animations（animateMotion 族/SMIL 新案） | 9 | 19 | 5 | 14 | animateMotion ×8 案 1F/案——getCTM 邻接（挂账 #2）；新 SMIL 案三件本批修复（§2） |
| coordinate-systems（DOMMatrixReadOnly） | 2 | 14 | 2 | 12 | getScreenCTM→DOMMatrix 邻接（挂账 #2） |
| painting | 1 | 8 | 6 | 2 | paint-order 面小簇 |

新面 13/69 文件全绿；导入时 kept 面（486 交集）零 flip——roll 基线
6,423P 对本批终态 6,427P，代码面零 P→F（§3.2）。

## 2. 新面修复三件（同批落地，全通道验证 §3）

### 2.1 getSimpleDuration float32 往返（smil-clock-value-large-hours）

Blink SMILTime 内部存 float——`getSimpleDuration` 秒值经 float32 往返
（1193047h = 4294969200s → `Math.fround` = 4294969344；常规值 fround
恒等零外溢）。part09b `getSimpleDuration` 返回值加 fround。2/2 子 P。

### 2.2 animationsPaused + per-element 暂停集（pause-and-seek-before-begin）

- `animationsPaused` 原缺列。首版挂文档级 `st.paused` 兜底——同文档双
  svg 消费面（pauseThenSeek 先 pause、seekThenPause 后 pause）下
  pauseThenSeek unpause 后 seekThenPause 仍须报 paused=true，文档级
  标志反照（Blink 每容器独立 paused）。
- 终版：`st.pausedEls` per-element 暂停集（`_elKey` 键位）——
  `pauseAnimations` 加集位 + 文档钟冻结；`unpauseAnimations` 清集位、
  **集非空早退**（另一 svg 仍持暂停则钟保持冻结）；`animationsPaused`
  只读本元素集位（不回读文档级）。`st.paused` 写点仅 pause/unpause
  两处且与集合同维护（不变式：集空 ⟺ paused=false），pump/seek/
  setCurrentTime 各读点语义不变。
- seeking-events-1..8（pause+seek 驱动面）零回归实证——单 svg 场景集
  语义与原文档级等价。

### 2.3 repeatEvent seek 合并派发（seek-repeat-event-condition）

- 自然播放 repeat 块加 `repK` 水位去重（part09 state 初始化 `repK: 0`，
  两处 begin 落位清零）——往返 seek 不重放。
- seek 合并派发块（差分兜底后）：unpaused 前向 seek 跨 ≥1 迭代边界合并
  派一次（kEnd = 最大 k≥1 且 b+k·sd<e 且 ≤t，边界恰落 e 不算 repeat）；
  paused 时间线不派（seeking-events-5/6 定谳——EventWatcher 对未期待
  事件 assert fail）。块前捕获 `seekW0 = s.lastT` 作前向守卫。

## 3. 验证（test-guard 4G/8G 包裹，`--json` stdout 落档本目录）

### 3.1 svg/animations 定向通道

- **231P/111F/0T**（前次 230P/112F）：`+1F→P` =
  pause-and-seek-before-begin::seekThenPause；pauseThenSeek、
  seek-repeat-event-condition、seeking-events-1..8、
  smil-clock-value-large-hours 全 P。
- 期间返修：animationsPaused 文档级兜底版 pauseThenSeek P→F
  （assert_false "animations are unpaused" 中招）→ per-element 纯集位
  语义复验双子 P。

### 3.2 svg 全通道（`2026-11-07-m4-coverage-channel.json`）

- **554 文件 11,395 子：6,427P / 4,961F / 7T**。对 roll 基线
  （6,423P/4,965F/7T）：**+4F→P 零 P→F**——逐子 flip 矩阵核，
  恰为 §2 三修复全集（seekThenPause / pauseThenSeek /
  seek-repeat-event-condition / smil-clock-large-hours）。
- Timeout 7 案恒等；web-animations 依赖通道见 §3.3。

### 3.3 web-animations 依赖通道（`2026-11-07-m4-coverage-web-animations.json`）

- 507P/446F/9T（B7 基线 508P/445F/9T）：唯一 flip
  `DocumentTimeline/constructor::An origin time of zero is used when
  none is supplied` P→F——**非确定抖动定谳**：定向 3 连跑 origin-time
  3/3 P，第 3 跑同文件异子测试（zero-origin-time-identical）翻转；同
  文件异子测试连跑波动，时钟精度抖动域，与 SMIL 改动无语义交集（B6c
  updating-the-finished-state 注册抖动同族备注扩展）。

### 3.4 单元回归

拼接语法检查通过（21 文件 `new Function` 全量解析）；part09b 1,405 行
< 2000 行准则。

## 4. 挂账归口（DC-3 收尾）

- **#2 getCTM 族**：现势 40 文件 / 70F（getCTM/getScreenCTM 入口缺失
  邻接——animateMotion ×30 案各 1F、coordinate-systems DOMMatrixReadOnly
  ×12F、textpath getStartPositionOfChar 同族）＋ hittest 矩阵 4,212F
  单文件渲染域另计。归口结论：**归 rendering-compat 协同排期**（几何
  上下文/呈现命中面，本 goal 不双改，与立项划界一致）。
- **#3 SVGLength 上下文单位**：现势 23 文件 / 61F（SVGLength
  cap/ch/ex/ic/rlh + convertToSpecifiedUnits-font-change + svglengthlist-
  percentage + attributeStyleMap/css-var 邻接）。归口结论：**与 getBBox
  几何上下文同批评估维持**（渲染上下文单位，S2a FIXME 批，rendering-
  compat 协同），本 goal 不开批。
- **#6 缩减记账**：`a.rel-noreferrer-policy.html` 上游 iframe 化出通道
  （语料在、skip 规则排除，项消）——#6 余 a.ping（现势
  a.ping-functionality 1P/2F）＋ a.rel-noopener-policy（1 子空转），
  均导航域，体量缩小；新面 `a.referrer-invalid-value.html` 1 子 P
  （referrer 枚举校验已过）。
- **语料漂移记账（非回归）**：滚动改写 `SVGGraphicsElement.getBBox-06/
  07`——B7 时 7 个 Pass 子测试在上游新内容中消失、新内容 2 子转 P，
  交集净 −5P；roll→now 零 P→F 实证为纯语料内容变化（getBBox display:
  none 域属挂账 #4，上游测试形态演进）。

## 5. M4 DC-4 复测收口（双口径）

- **冻结分母可比口径**（coverage-2026-11-02 分母 15,342 子不变，一期
  通道 486 文件交集剔除语料漂移）：交集分子 **1,355**——精确分解
  1,360（B7）− 7（getBBox-06/07 上游改写消失子）+ 2（getBBox-06 新子
  P）= 1,355，逐子明细见 §4 语料漂移记账；§2 三修复的 +4F→P 全部落
  在新面文件（三案均 roll 新增，不在交集），**8.83% ≥ 8.6% 达线**
  （B7 8.86% −0.03pt 全部来自上游 getBBox-06/07 改写，代码零回归）。
- **滚动新面增量另账**：+69 文件 +5,072P（本地实测，冻结分母不含新
  分子——分子分母须同源，wptfyi 快照刷新后由其管线自然落账）。
- **全量分母复测注记**：15,342 为 wptfyi coverage-2026-11-02 快照
  （上游 Chrome 运行注册口径）；本地 runner 注册口径不可比（不支持
  API 提前退出致子测试少注册），不复算本地分母。wptfyi 周快照未到
  新 pin 28c152b74——**分母刷新待下次周快照**（独立管线，wpt.fyi 接
  入方向已上线周快照+网站展示）。
- **头条不回落**：全量头条 67.54%（coverage-2026-11-05 快照 68.1%）
  为 wptfyi 管线独立产出，本 goal 语料滚动不触其分母口径；收口后由
  周快照自然复核。

## 6. 结论

- M3 **收口**：滚动导入（pin override + missing=0 + 簇归口）+ 挂账
  #2/#3 归口结论落档 + #6 缩减记账 + B7 完成。
- M4 **收口**：DC-4 冻结口径 8.83% ≥ 8.6% 达线、一期已收口面零回归
  （svg 通道 roll→now 零 P→F；WA 抖动域注记）、头条管线不受本 goal
  影响。全量分母刷新待 wptfyi 周快照（外部节奏，不阻塞 goal 关闭）。
