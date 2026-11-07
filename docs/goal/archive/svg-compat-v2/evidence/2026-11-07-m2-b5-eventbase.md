# M2/B5 — 事件基与处理器名错位映射（2026-11-07）

B5 一件事：SMIL eventbase 动态实例源（begin="click"/"id.event"）+
repeat(k) 静态传播 + repeatEvent 自然播放重放 + restart 门 + 条件处理门
+ SVG 动画事件处理器名错位映射（onbegin↔beginEvent）。全通道 +
web-animations 双通道零回归复验落档。

## 1. 实现落点

- **`part09.js`（+278/−53 净含 B4 尾注更新）**：
  - **`_zwSmilNotifyEvent(target, type)` 派发通知契约**：全文档扫描
    begin 形态 event 规格，id 名 → getElementById（非 SVG 元素允许——
    eventbase-non-svg-element 定谳）、裸事件名 → 父元素；base/target 同
    节点判定走 `_elKey` 键比较（#32 identity 墙兜底）；命中即 `dynBegins
    .push(t + off)` + 链缓存作废 + `_wake`。时间线状态经 `_timelineFor`
    现建（pre-load 通知实例不丢——inline 脚本 load 前派发
    eventbase-non-svg-element 定谳）；首 tick 后无 SMIL 文档快路径返
    （tickEverRan/smilSeen 标记，热路径只付 WeakMap 读）。
  - **repeat(k) 静态传播**：`_docChains` 依赖槽 timeKind='repeat'，传播
    值 = b + k·sd + off（k·sd ≥ e−b 越界不传播——repeatn 实例语义；
    k=0 无实例——repeatn-remove-add 实证）。
  - **repeatEvent 自然播放窗口重放**：`_scanDocEvents` 对活跃区间补派
    k₀ = ceil((max(w₀, b) − b)/sd) 起（fired > 1000 break 防无限）。
  - **restart 门入模拟**：rec 记 `startedN`；pick 循环 never（已启动即
    跳）与 whenNotActive（活跃区间存在即跳）过滤——restart-never-and-
    begin-click 定谳面。
  - **条件处理门 `_condOK(el)`**：祖先链 systemLanguage（RFC 4647 前缀
    匹配）/requiredExtensions（非空判否）——conditional-processing-02
    定谳面；判否元素无实例、无边界、不入事件源。
- **`part01.js`**：共享映射表 `_ZW_SMIL_TYPE_TO_BASE`（beginEvent→begin
  等）/`_ZW_SMIL_BASE_TO_TYPE`（begin→beginEvent 等）。
- **`part03.js`**：`_ensureInlineHandler` 内容属性派生（endEvent 事件 →
  onend 属性）；三处派发调用点 + s328 采纳臂对裸 begin/end/repeat 保留
  名跳过 on\* 查找（custom-events 负向面——保留名非注册事件类型）。
- **`part04.js`**：
  - setter/getter 注册键映射（onbegin → beginEvent 键——IDL 动画
    handler 错位注册键）。
  - **onbeginEvent/onendEvent/onrepeatEvent expando 门**：三别名补形名
    非任何 IDL handler 名，真浏览器落 expando 派发不触达；泛型 on\* 若
    注册则与同键 onend 互相驱逐（后写者从 `_listenerStore` 移除先写者
    ——probe4 实证），门须置于 BASE_TO_TYPE 映射之前（onend 映射成
    endEvent 后会误入门）。
  - dispatchEvent/click() 包装派发尾 `_zwSmilNotifyEvent`（元素派发路径
    不经 part05 EventTarget——eventbase 四案通知面）。
- **`part05.js`**：EventTarget 派发尾同通知（xhr 等非元素目标——
  eventbase-non-svg 定谳分流）；R114 泛型 on\* 查找按保留名跳过 +
  type→base 映射（读 part01 表）。
- **`part06.js`**：宿主派发通道 `__zw_dispatch_event` 尾同通知（UA 合成
  事件——testdriver.click/runner 输入链不经页面 dispatchEvent 与
  part05 两面，restart-never-and-begin-click 定谳面；SMIL #Eventvalue
  semantics 不区分 trusted/合成）。

## 2. 验证（test-guard 4G/8G/2700s 包裹，`--json` 落档本目录）

### 2.1 svg 全通道（`2026-11-07-m2-b5-svg-channel.json`）

- **486 案 1,710 子：1,239P / 464F / 7T = 72.5%**（B4 1,227P / 471F /
  12T：**+12P / −7F / −5T**）。
- **零 P→F**（逐子 flip 矩阵核）。
- **F→P ×8（FIXED）**：custom-events ×6（三对保留名/错位名对称语义——
  负向 'end'/'repeat'/'begin' 不触发任何 handler，正向 'endEvent'/
  'repeatEvent'/'beginEvent' 触发 on\* 属性 + IDL + listener 三面）+
  event-listeners ×1（onendEvent expando 不触发——B4 显形挂账清账）+
  conditional-processing-02 ×1（systemLanguage 判否祖先链）。
- **T→P ×4（FIXED）**：beginevents-1 + eventbase-non-svg-element +
  eventbase-after-removal（scripted/）+ restart-never-and-begin-click
  （testdriver.click 经宿主通道）。
- **T→F ×1（显形，非 Pass 丢失）**：slider-switch——事件链全通后终断言
  `getComputedStyle(a).display` 得 "inline"，**挂账 #4 style-system
  协同域门**（同 correct-events 族；fix(es).display 表现值不入 computed
  style）。
- animations 目录 93P→105P、5T→1T；其余目录逐子恒等。

### 2.2 web-animations 依赖通道（划界约束）

`testharness-web-animations --json`（同 guard 口径，干净轮捕获本目录
`2026-11-07-m2-b5-web-animations.json`）：**506P/444F/9T = 959 子，对
B4 逐子恒等零 flip**——处理器名映射/expando 门/通知钩子未波及 WAAPI/
CSS 动画泵。

## 3. B5 批次记账（DC-3 FIXED/REGRESSED）

- **FIXED +12P**（§2.1 八案）。
- **REGRESSED 0**。
- **T→F 显形 ×1**：slider-switch（§2.1，挂账 #4 域）。
- M0 账 6F/7T 目标簇全数兑现（6F = custom-events 6；7T 中 4 转 passe、
  onhover-syncbases/repeatn-remove-add/correct-events 三案仍 F/T——
  各自挂账门见 §4，非本批范围）。

## 4. 挂账门（本批不改，跨域记账）

- **#4 getComputedStyle SMIL 表现值门**：onhover-syncbases、
  repeatn-remove-add、correct-events、escaped-dots、slider-switch（本批
  新显形）——computed style 不读 SMIL 动画表现值，归 style-system 协同
  批。
- **引擎查询层 doc-qSA 动态 append 同步不可见**：syncbase-remove-add
  （B4 §4 在案，wrapper 归一待查询层批次）。
- **attributeTypes**：JS harness 错（B6 域边界，M0 已定性）。

## 5. 下一步（M2 续）

B6 采样引擎（calcMode discrete/spline、additive/accumulate、keyTimes/
keySplines——M0 账 124F/3T 最大簇，B3/B4/B5 连带收敛后现势全通道
464F 中 animations 域占大头）→ M3 B7 path d 插值（33F）+ 滚动新面导入
+ 挂账 #2/#3 归口 → M4 coverage 复测收口（DC-4：svg topdir ≥ 8.6%）。
