# 导航与浏览上下文兼容 — 运行时控制面板（master.md）

**入口文档**: [../navigation-compat.md](../navigation-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-10-09（M2-S1~S4O 十八片——location-interface 95.3% / traversal 93.3% /
history-interface 91.8% / navigation-api 85.5%；全量 20.9%→75.2%（358/476））

---

## 当前状态

**专项定位**：会话历史/导航事件（轻面，独立推进）+ iframe 浏览上下文（深面，
**用户门控切片**，先例 R1043/Phase A IFC）。轻面可不等门控独立收口出数字。

**与兄弟 goal 的边界**：
- rendering-compat — viewport/滚动/渲染面不碰（style-system/layout-engine/render-foundation
  不在 envelope）；iframe 渲染面缺口记账回流
- event-loop-spec（已归档）— IO/RO 与事件循环遗产为消费基础
- zero-web P1a — location 读侧已落；写侧导航语义归本 goal
- zero-protocol / 多进程 — fission 排除；M3 触进程模型即 BLOCK 上报用户

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 四 corpus 导入 + 基线 | ✅ 落地（fetch 三域 + 嵌套 resources 2026-10-07 恢复后入库；基线 20.9%，S1+S2 后全量 29.2%） |
| P2 | history pushState/replaceState/state/length/back/forward/go 语义 | ✅ S3 收口：the-history-interface 81.6%→**91.8%**（traverse 入队 + 空串 URL + 跨源 SecurityError）；余 xhr helper infra ×1 + 速率限制 optional ×2 挂账 |
| P3 | popstate/hashchange 事件序 + location 写侧导航语义（带重入 guard） | 🔶 S1+S2+S4F 落地：location-interface 41.9%→**95.3%**（exotic 内部方法面 Proxy 承载收口）、traversal 62.2%→**93.3%**；余 2F = runner 无端口 URL（形态缺口挂账）+ create-script-set-location（归 ③ 跨文档簇） |
| P3b | Navigation API（window.navigation 全域） | 🔶 S4~S4O 十八段落地（→**85.5%**，218/255）：read side/navigate/intercept/traverse/scroll-behavior/focus-reset/location exotic/precommitHandler/traverseTo/ongoing-abort/anchor-downloadRequest/host 激活锚线程/form submit navigate/activation 暴露/transition 派发时机/dispose 深簇/navState 入槽克隆/同文档 navState 承继/reentrant 孤儿 transition/提交期 abort/`:target` URL 同步/window.stop abort 面全收；余静态锚 href IDL 落 attr 面（2 案——回流 element IDL 域）、Tab 焦点导航 2T + autofocus load 期 7 NotRun（DOM 焦点域）、scroll-behavior reload 族 4F（渲染域） |
| P4 | iframe 浏览上下文最小面（contentWindow/frames/parent/top + 属性语义） | ⏳ M3 **用户门控** |
| P5 | bfcache / fission 挂账定稿 | ✅ M4 定稿（bfcache：无实现面，依赖 frame tree→Document 快照管线，挂账至 M3 后续立项；fission：charter 排除维持。见 [M4 评估](evidence/2026-10-09-m4-closeout-assessment.md)） |

## 已完成切片

- **M1 runner 通道 + 首轮基线（2026-10-07）**：`testharness-navigation` 子命令 +
  `navigation_case_skipped` 筛减规则 + Makefile target；基线 309 案 387 子测试
  81 Pass = 20.9%。证据：[evidence/2026-10-07-m1-baseline.md](evidence/2026-10-07-m1-baseline.md)。
- **M2-S1 Location 接口语义（2026-10-08）**：Location [LegacyUnforgeable] 面 +
  `window.Location` 接口对象 + port + 写侧 SYNTAX_ERR + Document.location dedup +
  runner 绝对路径 helper 通道（stringifiers.js）。location 41.9%→**86.1%**；余 5F =
  exotic 面 ×3（FIXME 已标）+ runner 无端口 URL ×1 + Navigation API ×1。
  证据：[evidence/2026-10-08-m2-s1-location-interface.md](evidence/2026-10-08-m2-s1-location-interface.md)。
- **M2-S2 traverse 事件序（2026-10-08）**：fragment navigation 同步 popstate +
  isTrusted（`__zwTrusted` 口）+ PopStateEvent hasUAVisualTransition/非 new TypeError +
  scrollRestoration per-entry + runner 顶层 `let` accessor 转发导出（R201 同款，
  before-load-hash 族根因）。traversal 62.2%→**93.3%**、event-order 12/13。
  证据：[evidence/2026-10-08-m2-s2-traverse-events.md](evidence/2026-10-08-m2-s2-traverse-events.md)。
- **M2-S3 session history 收尾（2026-10-08）**：back/forward/go 按 spec traverse 算法
  入队 FIFO 结算（位置执行时计算）+ 空串 url 保留 fragment（whatwg#9343）+ 跨源
  pushState/replaceState SecurityError。history-interface 81.6%→**91.8%**。
  证据：[evidence/2026-10-08-m2-s3-history-queue.md](evidence/2026-10-08-m2-s3-history-queue.md)。
- **M2-S4 Navigation API read side + currententrychange（2026-10-08）**：window.navigation
  （currentEntry/entries/updateCurrentEntry/back/forward + on\*）+ NavigationHistoryEntry
  （key/id/url/getState）+ NavigationCurrentEntryChangeEvent 构造器 + entry 生命周期
  hook（push/replace/保 key/traverse 还原）。navigation-api 0.4%→**8.6%**。
  证据：[evidence/2026-10-08-m2-s4-navigation-read-side.md](evidence/2026-10-08-m2-s4-navigation-read-side.md)。
- **M2-S4B navigate 事件 + navigate()/reload() + intercept（2026-10-08）**：NavigateEvent/
  NavigationDestination 构造面 + navigate 同步派发（五 hook）+ preventDefault 中止 +
  intercept 顺序链 + navigateerror ErrorEvent + anchor sourceElement 线程。
  navigation-api 8.6%→**24.4%**。
  证据：[evidence/2026-10-08-m2-s4b-navigate-event.md](evidence/2026-10-08-m2-s4b-navigate-event.md)。
- **M2-S4C traverse 面 + destination 动态面 + preemption（2026-10-08）**：back/forward/go
  派发 navigate 'traverse'（destination 绑目标 record、preventDefault 取消、intercept 照常
  应用）+ NavigationDestination 动态 index/getState + detach record 复活 + preemption +
  navigation.transition + addEventListener once + send_keys fixture 2s 超时修复（连轮闪红
  根因）。
  证据：[evidence/2026-10-08-m2-s4c-traverse-face.md](evidence/2026-10-08-m2-s4c-traverse-face.md)。
- **M2-S4D scroll-behavior 面（2026-10-08）**：滚动保存（离 entry 前）/ restore 规格
  （frag 形锚滚|文档顶 + saved 形 traverse 保存位）/ scroll 代次 skip 判定 / intercept({scroll})
  after-transition|manual / e.scroll()（dispatch 期 InvalidStateError——顺手修掉 manual-immediate-scroll
  假绿：listener assert 异常被 dispatch 吞）/ committed 改链任务头结算（intercept 链只挡
  finished——scroll-behavior 族 14 Timeout 同根）/ finished 拒绝原因原样透传。
  navigation-api 27.8%→**36.1%**（scroll-behavior 1/26→22/26）；全量 45.0%→**49.4%**。
  余 4F = reload 族挂账渲染域（scroll anchoring + rect 快照刷新）。
  证据：[evidence/2026-10-08-m2-s4d-scroll-behavior.md](evidence/2026-10-08-m2-s4d-scroll-behavior.md)。
- **M2-S4E focus-reset 面（2026-10-08）**：handler **同步起跑**（spec commit 抑制段——
  intercept_resolve 同步可见）+ focusReset 枚举（缺省 after-transition/'manual'，非法值
  TypeError）+ 焦点变更追踪（focus()/blur() 记变 → 结算跳过重置）+ 结算焦点重置（blur 落旧
  焦点可重入导航 / focus 落 body，先于 success/error）+ 移除聚焦元素 unfocus（removeChild/
  el.remove 两路）+ **webview 模块静态导入实拉**（R3093 空存根 → 递归拉真实源——模块化
  corpus 命名导入全数 compile error 的根因）+ focus-reset/resources 资产。
  navigation-api 36.1%→**40.1%**（focus-reset 0→9P）；全量 49.4%→**50.6%**。
  余挂账：Tab 键焦点导航（2T）+ autofocus load 期处理（7 NotRun）= DOM 焦点域。
  证据：[evidence/2026-10-08-m2-s4e-focus-reset.md](evidence/2026-10-08-m2-s4e-focus-reset.md)。
- **M2-S4F Location exotic 内部方法面（2026-10-08）**：Location 对象 Proxy 包裹——
  [[PreventExtensions]] 恒 false / [[SetPrototypeOf]] 不可变（__proto__ 经原型链 setter 回落
  trap）/ 新 own 属性拒 / 实例 prototype 链 Location.prototype；immutable-prototype helper
  资产启用。location-interface 86.1%→**95.3%**（消 M2-S1 遗留 FIXME）；全量 50.6%→**52.9%**。
  余 2F = runner 无端口 URL（形态挂账）+ create-script-set-location（③ 跨文档簇）。
  证据：[evidence/2026-10-08-m2-s4f-location-exotic.md](evidence/2026-10-08-m2-s4f-location-exotic.md)。
- **M2-S4G precommitHandler + traverseTo + ongoing-abort 事件序（2026-10-09）**：
  precommitHandler 全生命周期（同步调起/redirect/addHandler/提交延迟化——navigate/reload/
  pushState/replaceState/traverse 五路 commitFn 闭包）+ navigate({state}) navState 分槽 +
  traverseTo(key) + NavigationHistoryEntry/Transition 接口对象（instanceof 面）+ 进行中导航
  抢占（spec inform-abort while 循环、非 intercept 同参与、abort 序 = signal→finished→
  navigateerror→transition）+ window.stop() + 非 intercept success steps + transition 生命
  周期对齐（派发时仍暴露）+ traverse 入 task 队列 + dispose 事件 + ordering Recorder 资产。
  navigation-api 40.1%→**69.7%**（precommit-handler 0→38P、ordering 1→28P）；全量
  52.9%→**67.4%**；per-subtest 精确 diff 零回归。
  证据：[evidence/2026-10-09-m2-s4g-precommit-traverseto.md](evidence/2026-10-09-m2-s4g-precommit-traverseto.md)。
- **M2-S4H anchor downloadRequest + 锚点击通用导航面（2026-10-09）**：anchor download 属性
  → downloadRequest（expando 优先/attr 回落）+ 通用锚导航 helper（sameDocument 按本源/
  canIntercept=同源可重写/同 URL=replace/download 未拦截吞导航）+ userInitiated ← 瞬态激活
  （读后清）+ 锚激活 sourceElement 物化（part04 代理侧 + part03 plain-node 双路径）。
  navigation-api 69.7%→**72.6%**（anchor-download 族 6T/F→10P）；全量 67.4%→**69.2%**；
  per-subtest 精确 diff 零回归。余 host 激活路径锚线程 = S4I。
  证据：[evidence/2026-10-09-m2-s4h-anchor-download.md](evidence/2026-10-09-m2-s4h-anchor-download.md)。
- **M2-S4I host 激活路径锚线程（2026-10-09）**：`__zwNavAnchorNavigate` 全局暴露（状态回传
  'canceled'/'intercepted'/'download'/'host'——host 导航决策外移）+ user_actions.rs SetFragment
  前置线程（sourceElement/downloadRequest）+ Navigate effect 锚预导航（状态 ≠ host 跳过 host
  导航）+ 锚 href expando-first 读（hasOwnProperty 门）。
  navigation-api 72.6%→**73.5%**（userInitiated/download-userInitiated 两案收口）；全量
  69.2%→**69.7%**；per-subtest 精确 diff 零回归。余静态锚 href IDL 落 attr 面回流 element
  IDL 域。
  证据：[evidence/2026-10-09-m2-s4i-host-activation.md](evidence/2026-10-09-m2-s4i-host-activation.md)。
- **M2-S4J form submit navigate 事件面（2026-10-09）**：`_zwFormSubmitNavigate` navigate
  事件先行（sourceElement=submitter||form、formData=POST FormData/GET null、navigationType=
  瞬态激活→push/同URL→replace、preventDefault→cancel 微任务 navigateerror、intercept→同文档
  提交链）+ POST 入面（未拦截零投递——R-baidu5 契约维持，回归钉当场捕获后修复）+ GET 空 entry
  list 追加 '?' + navigateerror 微任务派发（同步派发漏听）+ runner 合成 click 补调
  `__zwNavFormRequestSubmit`。navigation-api 73.5%→**76.1%**（form 族 5T→6P）；全量
  69.7%→**71.0%**；per-subtest 精确 diff 零回归。
  证据：[evidence/2026-10-09-m2-s4j-form-submit.md](evidence/2026-10-09-m2-s4j-form-submit.md)。
- **M2-S4K navigation.activation 暴露 + runner variant-meta 展开（2026-10-09）**：
  NavigationActivation 接口对象 + getter（entry 首次访问定格/同文档导航不变/replaceState
  孤儿化 index -1）+ runner `run_any_js_corpus_subdirs_with_helpers_variants`（expand 门仅
  navigation corpus；`case_variants` 既有提取器复用；每变体独立运行 case URL 追加 query）。
  navigation-api 76.1%→**74.5%**（分母 230→255：state `?method=` ×4 Pass、ordering
  `?currententrychange` 变体面首次运行 + activation 2 案收口）；全量 71.0%→**69.1%**
  （329/476，分母 +34）。per-subtest 精确 diff 零回归（既有 case 状态零变化）。
  证据：[evidence/2026-10-09-m2-s4k-activation-variants.md](evidence/2026-10-09-m2-s4k-activation-variants.md)。
- **M2-S4L transition 派发时机对齐（2026-10-09）**：transition 创建由 `_navRunIntercept`
  （commit 后）前移至 `_navFireNavigate` dispatch 后（spec inner fire step 29——CCE 派发时
  transition 须已暴露）；`_navRunIntercept` 复用同一对象/结算钩子。ordering
  `?currententrychange` 变体 Fail 簇 11 案全收。navigation-api 74.5%→**78.4%**；全量
  69.1%→**71.2%**；per-subtest 精确 diff 零回归。余 re-entrant 嵌套 fire 时序 2T。
  证据：[evidence/2026-10-09-m2-s4l-transition-lifecycle.md](evidence/2026-10-09-m2-s4l-transition-lifecycle.md)。
- **M2-S4M dispose 深簇收口 + 载入后 hash-setter push 语义 + ongoing 槽生命周期（2026-10-09）**：
  window 'load' 置载入标记 → hash-setter 片段导航 Navigation API 侧载入前 replace/载入后 push
  分派（dispose-same-document「entries=start+3」与 location-api「replace+index 不变」由载入态
  区分）+ ongoing 槽 commit 清除（dispose 处理器内后续导航不被过期槽 abort）+ 非 intercept
  同文档导航同步立即结算 + finished 标记 handled（spec Mark as handled）+ ordering
  `?currententrychange` CCE 变体 11 案（transition 前移 dispatch 后）+ 顺修 traverse 提交
  双调用（r3065 门禁捕获）。navigation-api 74.5%→**79.2%**（dispose 深簇 3F 收口 + CCE
  变体 11 案）；全量 69.1%→**71.6%**；per-subtest 精确 diff 零回归。
  证据：[evidence/2026-10-09-m2-s4m-dispose-cluster.md](evidence/2026-10-09-m2-s4m-dispose-cluster.md)。
- **M2-S4N navigate({state}) 入槽结构化克隆（2026-10-09）**：navState 入 entry 槽时
  StructuredSerializeForStorage 克隆（spec 克隆时机在导航时）——页面脚本后续对原对象的变更
  不再渗入已存状态（旧引用直存别名污染）。state away-and-back-navigation-api 1F 收口；
  navigation-api 79.2%→**79.6%**；全量 71.6%→**71.8%**；per-subtest 精确 diff 零回归。
  余挂账：dispose-navigate-during unhandled 定位 1F、location-api away-and-back 1T、
  href-intercept-reentrant 双变体 2T。
  证据：[evidence/2026-10-09-m2-s4n-c-class.md](evidence/2026-10-09-m2-s4n-c-class.md)。
- **质量门禁（十七片）**：`make test` 全绿 20,324 P / 0 F（S4N 轮；锚 20,319 → 20,324 含
  兄弟流新增）；`make reftest` 704/704 零失败（2026-10-08 复验）；clippy -D warnings 零
  warning；fmt 零 diff；每轮全量语料零回归（per-subtest 精确 diff）。全量批墙钟
  TIME_LIMIT=2700（批内不与 make test 并发）。
- **M2-S4O C 类余项第二批（2026-10-09）**：四根因同轮——① 同文档导航 navState 承继
  （spec apply the push or replace history step；`_navPushCurrent` carryState + fire 期
  `destState` 下发，四 push 路接线）收 away-and-back-location-api 双变体 +
  navigate-destination-getState-fragment-via-href；② transition 创建去重 + 抢占守卫
  （删 S4L dispatch 前恒死副本，dispatch 后副本加 errored/settled 门）收
  intercept-reentrant 四变体；③ 提交期 abort 不 reject committed（`_zwCommitting` 印记）
  收 dispose-same-document-navigate-during + intercept-and-navigate（意外收口）；
  ④ 同文档导航 `:target` URL 同步（原生绑定 `__zw_native_set_document_url` + shim
  `_zwFragmentUrl` 双槽，三 chokepoint 接线）收 navigate-same-document 双案 +
  target-pseudo-after-reinsertion（连带收口）；⑤ window.stop abort 三件套（`onabort`
  处理器触发 / 跨文档 navigate() committed 微任务结算 + `_zwCommitting` 块末清零 /
  AbortError 附 stack）收 signal-abort-window-stop 三案 + signal-abort-preventDefault
  （连带收口）。
  navigation-api 79.6%→**85.5%**（218/255）；全量 71.8%→**75.2%**（358/476）；per-subtest
  精确 diff 零回归（16 翻全 Fail/Timeout→Pass）。
  证据：[evidence/2026-10-09-m2-s4o-c-class-ii.md](evidence/2026-10-09-m2-s4o-c-class-ii.md)。
- **质量门禁（十八片）**：`make test` 全绿 **20,336 P / 0 F**；clippy -D warnings 零
  warning；fmt 零 diff。

## M4 收口评估（2026-10-09）

**判定**：DC-1 ✅ / DC-2 ✅（主簇） / DC-4 ✅；**DC-3 ⏳ 用户门控 pending（未获豁免）→ goal
不判 DONE，维持 Active 推进态**。残余 136 Fail/Timeout 全量盘点定性：A 域回流（DOM 焦点
~9 / 渲染 ~20 / element IDL 2）、B runner 形态（replace-before-load 38 + bfcache 族 ~10 +
跨文档链 ~6）、C 可切片 ~25（form submit 5、state 分槽 2、activation 暴露 4、dispose 深簇 6
等）。P5 bfcache/fission 定稿：bfcache 无实现面挂账至 M3 后续立项；fission charter 排除维持。
证据：[evidence/2026-10-09-m4-closeout-assessment.md](evidence/2026-10-09-m4-closeout-assessment.md)。

## 下一步计划

1. **C 类可切片余项（S4O 后重盘）**：dispose-for-full-session-history 1T、
   create-script-set-location 1F（跨文档 load 序——B/C 边界）、004/007/008 history
   helper infra 与速率限制 optional 4 案。
2. **replace-before-load 38F 重定性 → M3 依赖**（2026-10-09 勘察）：全簇为 iframe 载体
   （setupSentinelIframe/insertIframe + 子文档 load 前自导航），断言 iframe 自有 session
   history 的 replace 语义——单文档 runner 形态不可达，随 M3 frame tree 一并解锁。
4. **M2 ③ 跨文档导航语义**（navigating-across-documents 2/42）：主体依赖跨文档导航链
   + testdriver 用户手势，单文档 runner 形态不可达——runner 形态升级前仅记账。
5. **M2 ④-D 残余挂账域回流点**：Tab 键顺序焦点导航 + autofocus load 期处理（DOM 焦点域）；
   scroll anchoring + rect 快照刷新（渲染域，回流 rendering-compat）。
6. **M3**：frame tree 最小面——**启动前须用户点名批准**（2026-10-04 已征询待批复，
   维持挂起，见下）。

**待用户决策清单**：
- [ ] M3 iframe 深结构切片启动授权（未获批期间 DC-3 保持 pending，不阻塞 M2 收口）
  **2026-10-04 已征询待批复**（goal 待决策巡检 msg `om_x100b63125d66c4a8b2070829b8dd058`；
  建议 = 暂不批准——M1/M2 尚未启动，待 M2 收口时随推进一并裁决）。批复前维持立项态挂起。
  **2026-10-06 48h 跟进提醒已发（一次性，到期）**（msg `om_x100b63647f7910a0b49d70def6b88ff`）：
  仍零回复——不回复即默认维持挂起，此后不再重复催办。
