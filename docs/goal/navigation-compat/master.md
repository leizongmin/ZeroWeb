# 导航与浏览上下文兼容 — 运行时控制面板（master.md）

**入口文档**: [../navigation-compat.md](../navigation-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-10-21（**M3-S5 parent→child postMessage + 子文档
onmessage 绑定落地**——全量 corpus 464P→**468P**（恰 4 案
Timeout→Pass：iframe_sandbox_block_modals 1..4），**其余 577 路径逐行
恒等零回归**；子窗 stub 预置 onmessage null 槽 + 子文档脚本条件
with(window) 包合（裸 `onmessage =` 绑定子窗——此前落父窗 globalThis
致 R2932 误注册、子窗 handler 永缺席）。S1~S4 详情见切片账。M2 终态
维持 437/476（**91.8%**）。M3 门控 2026-10-21 获批，DC-3 未豁免。
另：兄弟流 9799e6435 fmt 漂移阻塞 T58 前置门，本轮机械修复独立提交
d0f6d873a）

---

## 当前状态

**专项定位**：会话历史/导航事件（轻面，已收口 91.8%）+ iframe 浏览上下文（深面，
门控切片，先例 R1043/Phase A IFC；**2026-10-21 启动已获批**）。fission/进程模型
触碰即 BLOCK 上报用户。

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
| P2 | history pushState/replaceState/state/length/back/forward/go 语义 | ✅ S3 收口 + S4P 速率限制：the-history-interface 81.6%→**98.2%**（55/56——traverse 入队 + 空串 URL + 跨源 SecurityError + push/replaceState 10s/100 次速率窗；S4W 撤销 007「外部脚本基建」挂账——实为外链 404 abort 掩盖，7 子测试真实翻绿）；余 008 Timeout 挂账（S10 定性改判：document.write 引擎全缺——parser 域、envelope 外——+ `www.` 主机形态双缺口，非纯 runner 形态、非 M3） |
| P3 | popstate/hashchange 事件序 + location 写侧导航语义（带重入 guard） | ✅ 收口：location-interface 41.9%→**100%**（43/43——S1 exotic Proxy + S4P 动态脚本通道 + S10 location_port origin 端口化）；history-traversal 目录 44/45（余 order-in-bfcache-restore 1F = bfcache 挂账） |
| P3b | Navigation API（window.navigation 全域） | 🔶 S4~S4Y 二十七段落地 + S9 锚 href 动态写可见性（→**93.3%**，txt 原始 Pass 行 239/255）：read side/navigate/intercept/traverse/scroll-behavior/focus-reset/location exotic/precommitHandler/traverseTo/ongoing-abort/anchor-downloadRequest/host 激活锚线程/form submit navigate/activation 暴露/transition 派发时机/dispose 深簇/navState 入槽克隆/同文档 navState 承继/reentrant 孤儿 transition/提交期 abort/`:target` URL 同步/window.stop abort 面/session history 50 条上限/traverse 语义面/navigationType 随载入态（锚激活例外恒 push）/片段指示元素 decoded 匹配 + 无匹配滚顶 + 锚 record push 例外 + 跨源 canIntercept/pushState·replaceState hashChange=false/location.reload 接线 + 目的态承继/navigate() 同 URL auto→replace/runner readyState 宿过渡/navigateerror filename fire 时快照（S4P 回归追认收口）/锚 download presence own 门/锚激活 href 动态写 latest-wins 读全收/Tab 焦点导航（S11 runner 宿主 dispatch_tab_focus）+ intercept() scroll/focusReset 显式覆写门/autofocus load 期处理（S12 runner 初始 flush）+ 结算委托合格性门（S12 part02 首合格迭代）；「静态锚 href IDL 落 attr 回流 element IDL 域」**销项**（S9 定因勘误——attr 持久化从未缺失，纯快照读 stale；双案全绿）；autofocus 14 NotRun **销项**（S12，全量 NotRun 清零）；余 scroll-behavior reload 族 4F（渲染域） |
| P4 | iframe 浏览上下文最小面（contentWindow/frames/parent/top + 属性语义） | 🔶 **M3-S1+S2+S3+S4+S5 落地（2026-10-21）**：S1 srcdoc 处理模型 + sandbox 指令（ASCII 解析 + allow-scripts 门 + supports 固定表）+ 成熟度 servedDoc 双读面 + frames named access + 子文档 location 写侧 + load 信任面（16P→**25P**）；S2 doc.location↔win.location 同一对象（全 URL 解析面）+ 子窗 hash 写侧 + 合成树 innerText（25P→**27P**，srcdoc-anchor / srcdoc-attribute-reset 翻绿）；S3 联合 iframe session history 独立序列（惰性播种 + frame/顶层标记遍历路由 + 子写侧合流；顶层账本不膨胀）（27P→**28P**，srcdoc_change_hash 翻绿）；S4 replace-before-load 最小面（runner `{{GET}}` 查询模板替换 + 子脚本 history 形参 + 子发起载入前 replace + 父窗 back frame 局部回退；全量 460P→**464P**，簇外零变化）；S5 parent→child postMessage + 子文档 onmessage 绑定（子窗 stub onmessage null 槽 + 子文档脚本条件 with(window) 包合；域 28P→**32P**，block_modals 四案翻绿；webmessaging 跨套件回归臂零行为回归）；余 F 集中 meta-refresh 形态 / src-encoding / javascript: URL 族 / {{hosts}} 模板面 / replace-before-load 深缺口（search setter、子 form 提交 frame 定向、joint history.length、pageshow 簇）；后续切片：子发起嵌套导航控制（iframe_sandbox_navigate_*，S5 已解锁子窗 message 送达前置）、popup 管理 |
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
- **M2-S4P C 类余项第三批（2026-10-09/10）**：批一三根因——① joint session history 50 条
  上限（`_histTrimOldest`）收 dispose-for-full-session-history.tentative；②
  pushState/replaceState 10s/100 次速率窗收 optional ×2；③ 动态 classic 脚本执行 append
  通道（native `run_prepared_scripts` 四站点 + shim `_appendVariadic` sel-backed 面）收
  create-script-set-location。批二 traverse 语义面——navigate() classic 槽 null 化、
  剪除 traverse AbortError abort（入队可达性快照）、back({info})/forward({info}) 线程、
  host page_url 同步 setter（仅 hash-only）+ 初始 entry url 补章 + hash-setter 成功步骤
  ——收 navigate-history-state ×2 + navigate-intercept-history-state +
  forward-to-pruned-entry + navigate-navigation-back-same-document +
  navigation-back-same-document-preventDefault + navigatesuccess-same-document；连带
  intercept-handler-throws/-reject/-multiple-times-reject 3F。
  history-interface 91.8%→**98.0%**（48/49）；location-interface 95.3%→**97.7%**（42/43）；
  navigation-api 85.5%→**91.4%**（233/255）；全量 75.2%→**77.3%**（368/476）；18 翻全
  Fail/Timeout→Pass 零 Pass 回归。批二曾引 host 交接回归（AREA 激活 no-op）——`_zwSyncDocUrl`
  拆门收口：page_url 同步仅 navigate() API 路径开门，锚/href/assign host 交接路径保持
  doc 槽同步（make test 内当场捕获）。
  证据：[evidence/2026-10-09-m2-s4p-c-class-iii.md](evidence/2026-10-09-m2-s4p-c-class-iii.md)。
- **M2-S4Q 载入前导航类型面（2026-10-10）**：navigate 事件 navigationType 随载入态——
  location API hash 导航载入前 replace（与 S4M record 分派对齐）、锚激活例外恒 push
  （Following Hyperlink 传 push，`__zwNavSourceElement` 在场区分）。navigate-location 收口；
  navigation-api 91.4%→**91.8%**（234/255）；全量 77.3%→**77.5%**（369/476）；零回归。
  挂账：navigate-multiple-location/-pushState 2T（Chromium task 排队多导航模型——改造
  波及 52+ 在绿 ordering 测试，风险大于收益）。
  证据：[evidence/2026-10-10-m2-s4q-navtype.md](evidence/2026-10-10-m2-s4q-navtype.md)。
- **M2-S4R 片段指示元素匹配语义 + 锚 record push 例外 + 跨源 canIntercept 面（2026-10-10）**：
  三独立面——① `_scrollToAnchorForHash` decoded 匹配（percent-decode + UTF-8 lossy，
  `ignoreBOM: true` 防剥前导 BOM）四级查找 + 无指示元素滚回文档开头（traverse 路径
  `_noTopFallback` 门防覆写 entry 恢复位——首跑揪 scroll-restoration-navigation-samedoc
  回归后收口）；② hash-setter CCE record 分派与 navigate 事件同谓词（锚激活恒 push 适用
  record 侧）；③ href-setter 跨源 `canIntercept` 判定 + `intercept()` 对 canIntercept=false
  抛 SecurityError（原 InvalidStateError 误记）。8 翻 Fail→Pass 零回归（fragment-and-encoding
  ×6 + anchor-click + intercept-cross-origin）；navigation-api 91.8%→**92.5%**（236/255）；
  全量 77.5%→**79.2%**（377/476）。scroll-to-fragid 余项（几何/竖排 8 案）记账回流渲染域。
  证据：[evidence/2026-10-10-m2-s4r-fragment-indicated.md](evidence/2026-10-10-m2-s4r-fragment-indicated.md)。
- **M2-S4T pushState/replaceState hashChange 面（2026-10-10）**：pushState/replaceState 的
  navigate 事件 hashChange 恒 false（spec hashChange = fragment navigation 专属——原按
  hash-only URL 差计算误 true）。2 翻 Fail→Pass 零回归（navigate-history-pushState +
  navigate-history-replaceState）；navigation-api 92.5%→**93.3%**（238/255）；全量
  79.2%→**79.6%**（379/476）。首跑回归定稿挂账：intercepted traverse popstate 宏任务化
  （spec task）收 intercept-popstate-no-handler 即破 currententrychange-before-popstate-
  intercept 绿面（两案期望相反，Chromium task 管线排序可达、本沙箱同步结算模型互斥）
  ——维持微任务、no-handler 记账；form-submit-and-window-stop（helpers.js pin/master 双
  404 资产偏斜，归 same-url-replace 族）+ navigate-svg-anchor-fragment（svg:a dispatched
  click 激活缺失）入账。
  证据：[evidence/2026-10-10-m2-s4t-pushstate-face.md](evidence/2026-10-10-m2-s4t-pushstate-face.md)。
- **M2-S4U window.close 卸载事件面 + javascript: 锚执行面（2026-10-10）**：window.close()
  此前全缺（TypeError）——headless in-memory 近似派 beforeunload（cancelable）→ unload
  （不真关宿主）；javascript: 锚激活在 `_navAnchorNavigate` chokepoint 前置判定——间接
  eval 全局执行 + **_defer 微任务**（首版同步 eval 超时根因：真案 t.done() 闭包变量在
  async_test 构造期未赋值，真实浏览器于导航 task 执行——不在 click 栈内；FILTER 单案 +
  临时探针页定位）。2 翻 Fail/Timeout→Pass 零回归；全量 79.6%→**80.0%**（381/476，
  首次破 80）；navigation-api 域计数不变（两案均 html/browsers 域）。
  证据：[evidence/2026-10-10-m2-s4u-window-close-jsurl.md](evidence/2026-10-10-m2-s4u-window-close-jsurl.md)。
- **M2-S4V location.reload() 接线 Navigation API reload 面（2026-10-10）**：location.reload
  原 no-op 存根不派事件——接线 `navigation.reload()`（navigate 'reload' + intercept 生命周期，
  no-op reapply 近似维持）+ reload 目的态承继（destState ← 当前 entry navState）。收
  navigate-destination-getState-reload（Timeout→Pass）；navigation-api 93.3%→**93.7%**
  （239/255）；全量 80.0%→**80.3%**（382/476）；零回归。corpus 内其余 location.reload
  使用者均 iframe 门控/未导入（零波及，逐案核对）。
  证据：[evidence/2026-10-10-m2-s4v-location-reload-face.md](evidence/2026-10-10-m2-s4v-location-reload-face.md)。
- **M2-S4W strict 外链 fetch 失败收窄 + navigate() 同 URL replace 面（2026-10-10）**：两独立
  根因同轮——① `run_page_scripts_strict` 对外链脚本/模块 fetch 失败不再 abort（spec
  fetch-a-classic-script 失败派 script 元素 error 后页面继续；strict 门面收窄为「内联抛错」
  本意）——上游 helpers.js pin/master 双 404 掩盖的内联真断言暴露，push-same-url +
  form-submit-and-window-stop 直接翻绿、**007.html 全案 7 子测试翻绿**（撤销「外部脚本基建」
  与 S4T「资产偏斜」两族挂账；008 仍 Timeout）；② navigate() history 'auto' 且目标 URL
  等于当前 URL → replace（spec navigate-to-a-url auto 步；显式 'push' 不改写）收
  same-url-replace 双变体。12 Pass 新增 / 5 案 Fail→Pass 零回归；全量 80.3%→**82.8%**
  （394/476）；navigation-api 231→**234**/255；history-interface 48→**55**/56。
  记账：navigate-history-push-not-loaded 余 1F = runner 脚本阶段 readyState 宿未注入
  （t8m 三态过渡缺 testharness 路径镜像）——全语料 21 案跨 goal 域断言 readyState，
  须独立轮次跨域测量，不并入切片。计数口径勘误：S4V 标题 93.7% 与其 txt 不符
  （原始 231/255 = 90.6%，疑中途读数；+1 案判定确凿），本轮起按 txt 原始行计数。
  证据：[evidence/2026-10-10-m2-s4w-strict-narrow-family.md](evidence/2026-10-10-m2-s4w-strict-narrow-family.md)。
- **M2-S4X runner readyState 状态宿过渡（2026-10-10）**：t8m 镜像收口——`run_page_scripts_strict`
  前置 loading（页面脚本语义位置 = parser-inserted classic script）+ 生命周期 timer 任务内
  interactive→DCL→complete→load 原子过渡（含 readystatechange；此前 testharness 路径
  `__zwReadyState` 未注入、getter 恒缺省 "complete"）。跨域双臂测量兑现 S4W 记账：navigation
  全量恰 +1 零回归（push-not-loaded 收口）；net-api 全量红利 +1（sync-xhr-and-window-onload
  Timeout→Pass——同步 XHR 不再同步 fire onload 断言面）；csp-inheritance 逐行恒等；
  web-animations N/A（readyState 案不在本地导入面）。全量 82.8%→**83.0%**（395/476）；
  navigation-api 234→**235**/255。
  证据：[evidence/2026-10-10-m2-s4x-readystate-host.md](evidence/2026-10-10-m2-s4x-readystate-host.md)。
- **M2-S4Y navigateerror filename fire 时快照 + 锚 download presence own 门（2026-10-10）**：
  两独立根因——① `_navFireNavigateerror` filename 回退读派发时刻活 URL（内联脚本匿名
  eval → stack 帧 `<anonymous>` → 回退 `__zw_get_page_url()`）；S4P navigate() 路径
  page_url hash-only 提交同步使晚拒绝 handler 读到带片段 URL——修法 fire 入口快照
  `ev._zwFirePageUrl` 优先。**S4P 回归追认**：intercept-multiple-times-reject
  S4B~S4O 连续 Pass、S4P 起 Fail 六轮未察（S4P md 称收口但其入库 txt 原始 226/255 且
  Fail——中途读数第二例，per-txt 口径逮住）；② part04 锚 download presence 以
  `!== undefined` 判定，proxy 反射缺席 attr 返 `''` → 无 download 锚全判 presence——
  href 同款 own hasOwnProperty 门收口（attr 在场 ×8 复跑全绿；part03 无反射不触碰）。
  2 翻 Fail→Pass 各归各根因；全量 83.0%→**83.4%**（397/476）；navigation-api
  235→**237**/255（92.9%）。
  证据：[evidence/2026-10-10-m2-s4y-navigateerror-url-and-download-presence.md](evidence/2026-10-10-m2-s4y-navigateerror-url-and-download-presence.md)。
- **M2-S4Z traverse 锚滚 scrollRestoration 门（2026-10-10）**：plain-history traverse
  锚滚按**目标 entry** mode 门——manual 抑制片段锚滚（spec apply-the-history step scroll
  restoration mode），收 scroll-restoration-fragment-scrolling-samedoc；hash-setter
  新导航不受门（case 基面），与 S4D restore 侧 manual 门同谓词成对。1 翻 Fail→Pass；
  全量 83.4%→**83.6%**（398/476）；navigation-api 不变 237/255（html/browsers 域）。
  证据：[evidence/2026-10-10-m2-s4z-traverse-scrollrestoration-gate.md](evidence/2026-10-10-m2-s4z-traverse-scrollrestoration-gate.md)。
- **M2-S5 pagereveal reveal 事件面（2026-10-10）**：runner 路径 pagereveal 派发——
  页面脚本前包 rAF（首次调用先派，trusted + 门）+ 生命周期 timer 兜底（同门）。定位
  要点：runner 默认 rAF 同步 stub（reftest 兼容模型）使案内 rAF push 落页面脚本任务内，
  timer 期派发恒晚——spec reveal 先于该渲染机会 rAF 批 = 包 rAF 实现。挂账口径修订：
  pagereveal ×2 原 bfcache 族整目录挂账——new-document 形不依赖 bfcache 收口；
  bfcache-restore 形仍挂。1 翻 Fail→Pass；全量 83.6%→**83.8%**（399/476）；零回归
  （rAF 包裹全页面触达，语料树 grep 证惰性面）。
  证据：[evidence/2026-10-10-m2-s5-pagereveal-reveal-face.md](evidence/2026-10-10-m2-s5-pagereveal-reveal-face.md)。
- **M2-S6 派发 click 激活行为 + 锚默认动作 helper 化（2026-10-10）**：proxy
  dispatchEvent 派发路径此前**零默认动作**（S4T 挂账「svg:a 激活缺失」实为通用派发
  路径缺口，非 svg 专属）——A/AREA 锚派发 click 现跑激活（`_zwAnchorActivate` 共享
  helper，自 click() 分支原样抽出；svg:a 同分支命中）；untrusted 不签发 transient
  activation。收 navigate-svg-anchor-fragment（Timeout→Pass）；全量 83.8%→**84.0%**
  （400/476）；零回归（共享面双臂：navigation diff 恰一行 + 语料 grep 派发 click
  载体零 A/AREA）。挂账核验：bfcache 族 6 案 helper 资产双缺口（helper.sub.js/
  dispatcher.js 均不在本地语料 + 需 popup/真跨文档）、focus-reset 2T 需 send_keys
  TAB——两域挂账均属实维持。
  证据：[evidence/2026-10-10-m2-s6-dispatched-click-activation.md](evidence/2026-10-10-m2-s6-dispatched-click-activation.md)。
- **M2-S7 fragid 滚动/聚焦三面 + getElementById 特殊字符 id（2026-10-10）**：探针页
  逐值定位三叠加根因——①主文档 `getElementById` 经 `[id="…"]` 属性选择器实现且无
  捕获，id 含选择器语法外字符（%/[ 等）时**整体抛错**（raw/decoded 锚查找全灭）→
  选择器面 try/catch + `[id]` 枚举精确比对回落；②`documentElement.scrollTop` 读
  独立槽恒 0（spec scrollingElement 应镜像 window.scrollY）→ getter/setter HTML 门
  镜像窗口滚动；③`_scrollToAnchorForHash` 目标位按视口相对假设 +`_preTop` 双计
  （探针证 native gBCR 文档绝对，R3060 同约定）。第四面同 chokepoint：fragment
  滚动 focusing steps（可聚焦 → focus()；否则 viewport blur 回落 body）。六翻
  Fail→Pass 全 scroll-to-fragid 族；零回归（focus-reset/scroll-restoration 同
  chokepoint 恒绿）；全量 84.0%→**85.3%**（406/476）。家族余项重定性：竖排/书写
  模式滚动边 3F + reload scroll-anchoring 4F 维持渲染域。DC-4 reftest 账龄清零
  （S6 态 708 比较 0 不一致）。
  证据：[evidence/2026-10-10-m2-s7-fragid-scroll-focus-faces.md](evidence/2026-10-10-m2-s7-fragid-scroll-focus-faces.md)。
- **M2-S8 垂直书写模式块起始边水平滚动（2026-10-10）**：`_scrollToAnchorForHash`
  对 vertical-lr 根补 scrollX = border-box 左缘块起始对齐（探针证 vertical-lr abspos
  布局本就对、缺口纯 shim scrollX 面）；vertical-rl/inline-nearest/reload 族经探针
  三分维持渲染域（滚动区宽度/内容高 clamp/scroll anchoring 均布局滚动范围缺）。
  1 翻 Fail→Pass；全量 85.3%→**85.5%**（407/476）；零回归（diff 恰一行）。
  证据：[evidence/2026-10-10-m2-s8-vertical-blockstart-scrollx.md](evidence/2026-10-10-m2-s8-vertical-blockstart-scrollx.md)。
- **M2-S9 锚激活 href 动态写可见性 + 复核轮（2026-10-11）**：复核轮双项——① main 组成态
  （兄弟流 R5044-R5047/t8r-4 合入后）全量复跑与 S8 evidence 逐行恒等 + 残余 76 案逐案
  处置核对（C 类清零属实；reload-service-worker-fetch-event = SW fetch 管线 + iframe +
  跨文档三重门控补记入账）；② S4I 挂账定因勘误收口——探针证 `a.href = v` **确实落 attr**
  （R3069 → pending SetAttr mutation），缺口是读侧纯快照 stale（URL 分解分支 +
  `_zwAnchorActivate` attr 回落读不到同批写）：两点改 latest-wins（R3202 FORM 反射 /
  S4Y download presence 同款先例）收 navigate-anchor-cross-origin（Timeout→Pass，
  9 项断言全过）。「attr 持久化面回流 dom goal」销项。全量 407→**408**/476（85.7%）；
  零回归双臂（navigation per-subtest 恰一行 Timeout→Pass + html-syntax 全套件
  pre-change 基线构建逐行恒等 0 diff）。残余 75（35F + 26T + 14NR）。
  证据：[evidence/2026-10-11-m2-s9-anchor-href-lw.md](evidence/2026-10-11-m2-s9-anchor-href-lw.md)。
- **质量门禁（三十三片）**：`make test` 全绿 **20,404 P / 0 F**（两次全量各 1 例不同忙窗
  flake，solo 恒过 + 第 3 次全量全绿实证，归因记录见 S9 evidence）；纯 .js shim 变更无
  .rs（fmt/clippy 不适用）。**2026-10-11 复核**：组成态（R5048/R5049 合入后重建 release）
  全量复跑与 S9 evidence 逐行恒等；`make reftest` 708 比较 **0 不一致**（550 可信 +
  111 可疑 + 47 近似，与 S7 分布同）——S8/S9 两轮 shim 变更 + 兄弟流布局提交零渲染回归，
  DC-4 reftest 账龄清零。首跑 407 假信号 = A/B 基线臂陈旧 release 二进制（上轮 stash pop
  后未重建），learning 见
  [docs/learnings/bugs/2026-10/2026-10-11-ab-baseline-rebuild-stale-release-binary.md](../../learnings/bugs/2026-10/2026-10-11-ab-baseline-rebuild-stale-release-binary.md)。
  **R5050/R5051 组成态追认（2026-10-11）**：t8s1 合并 + style-compat R5050/R5051 入
  main 后全量复跑（release 零重编译 = 二进制即 HEAD `1430038a3` 组成），per-subtest
  语义 diff vs S9 evidence **逐行恒等**（408P/35F/26T/14NR，同组成连三跑全同）——
  残余处置抽核维持（sandbox allow_top_navigation 2F = popup+iframe+beforeunload 载体、
  history 008 = `www.` 跨主机脚本 URL，均属实挂账）。证据：
  [evidence/2026-10-11-composition-recheck-r5050-r5051.md](evidence/2026-10-11-composition-recheck-r5050-r5051.md)。
- **组成态复核：语料清空事故恢复 + scroll-restoration-order 收口（2026-10-20）**：
  开工跑测暴露 wpt-data 整目录重建（fetch-wpt-data `rm -rf` 重 clone 前例）后语料自愈
  断链——未认证 API 403 → resources/ 全缺、`.mjs` 白名单盲区（S4E/S4G 手工补拉未入
  脚本）、get-host-info.sub.js 缺失、快路径 support 文件洞，四项补拉全部入 goal 脚本
  （`dc126dc4e`），整目录重建可全量重建。组成态语义 diff 对 2026-10-11 账本恰 3 行
  （+新案 scroll-restoration-order；get-host-info 两案 Pass→Fail→补拉复绿）。新案定性 =
  10 月语料从未抓取的**新增可执行面真缺口非回归**（全部账本 grep 零出现）；修复 =
  hash-only traverse 保存滚动位恢复——popstate 派发后 hashchange 派发前（spec
  apply-the-history restore scroll position 步；`_scrollToAnchorForHash` 布尔化 +
  `_hist_dispatchPopState` defer 块带 skipScroll/锚未命中/_histRestoreScroll 自有
  has+manual 三重门恢复，S4Z/S4R 锚滚绿面零触碰）。全量 409P/35F/26T/14NR，对 10 月
  账本零漂移仅 +1 新 Pass；全量 85.7%→**85.9%**（409/476）。make exit 1 定性慢性面
  （sandbox 双案 `open()`→null 页面抛错，2026-08-13 `2b855a1d2` 起既有——10 月账本
  格式不记错误明细故当时不可见；判读看状态行集合，exit code 不作组成态门禁）。
  证据：[evidence/2026-10-20-composition-recheck.md](evidence/2026-10-20-composition-recheck.md)。
  同日二跑（HEAD 全新 release 重建，防陈旧二进制纪律）对 final 账本状态行集合逐行
  恒等零漂移，非 Pass 75 案逐案核对与挂账一致、无新可切片项；证据：
  [evidence/2026-10-20-composition-patrol.md](evidence/2026-10-20-composition-patrol.md)。
- **M2-S10 location_port origin 端口化 + history 008 定性改判（2026-10-20）**：
  对 6 案 runner 形态/任务模型挂账重评——location_port 案义自洽要求 URL 带显式端口
  （权威段 ":" 后子串 == location.port；无端口恒 Fail），上游 wptserve 恒带端口而
  runner 缺省 origin 无端口 = 案设环境缺失。修 = 跟随 net-api/webcrypto 页面形态
  谓词先例的单案 origin 端口化（`navigation_case_uses_ported_origin`，https 案锚
  `wpt.test:9443`；fetch handler 纯路径解析端口安全），引擎端口解析 + location.port
  仍为真断言。1 翻 Fail→Pass；全量 409→**410**/476（86.1%）；location-interface
  **43/43 收满**；per-subtest 语义 diff 恰一行零回归。008 挂账改判：document.write
  引擎全缺（shim/原生绑定零注册面、语料在账用户零）注入链不成立——parser 域
  （envelope 外）+ `www.` 主机形态双缺口维持挂账。门禁：runner 单测 224P/0F +
  clippy -D warnings 零 warning + fmt 干净。push rebase 撞兄弟流 R4004（shim
  label 激活共享面）——组合态复跑对 S10 账本逐行恒等零漂移追认。
  证据：[evidence/2026-10-20-m2-s10-location-port.md](evidence/2026-10-20-m2-s10-location-port.md)。
- **M2-S11 focus-reset Tab 焦点导航 + intercept() 显式覆写门（2026-10-20）**：
  S4E「Tab 焦点导航 2T」销项，两独立根因同轮——① runner 宿主此前丢弃
  `MoveFocus` 的 `PageEffect::Focus` 效果（真宿主由 renderer 消费），页面可见
  焦点不变 → focus-reset helpers waitForFocus 门铃永挂；修 = TAB 默认动作走
  `dispatch_tab_focus`（页面原生 focusing steps：标准可聚焦面候选序 + 正
  tabindex 升序在前 + 零/缺省文档序 + 起点 `document.activeElement` 身份/handle
  双判 → `next.focus()` 同键体系迁移；keydown 抑制首键 `__zw_host_focus`
  自聚焦前缀——迁移起点须为页面当前焦点）。② shim `intercept()` 旧版无条件
  覆写 `_zwScrollMode`/`_zwFocusReset`；spec「If options["scroll"] exists」
  has 门收口（undefined/null 同门跳过）——multiple-intercept「manual +
  (not provided)」双序案面先前 manual 保留。全量 410→**423**/476（**88.9%**，
  423P/34F/24T/14NR）；per-subtest diff：+13 Pass（basic 7 + multiple-intercept
  6）、−2 case Timeout 行、Fail/NotRun 逐行恒等零回归；navigation-api
  239→**252**（分母 255 口径）。autofocus 14 NotRun 定性 = promise_setup 首断言
  「load 期 autofocus 已聚焦」失败（引擎 load 期 autofocus 处理全缺，S12 候选）。
  跨流记账（不越界）：render-foundation `gpu_full_scene_filter_*` 两案灰度断言
  陈旧——R5194（`8e7ba040f`，10-19）GPU 矩阵改 709 派生权重后两案仍按 Rec601
  断言（54 = 0.2126×255），rendering 域待修，adapter 探测失败环境整段跳过故
  此前未暴露。fmt 干净 + clippy 零 warning + `make test` 76 suites
  22,186P/2F/199I（唯二 F = 上记跨流 GPU 断言，gate 复合命令吞退出码）。
  证据：[evidence/2026-10-20-m2-s11-focus-reset-tab.md](evidence/2026-10-20-m2-s11-focus-reset-tab.md)。
- **M2-S12 autofocus load 期处理 + 结算委托合格性门（2026-10-20）**：
  S11「autofocus load 期 14 NotRun」销项，两独立根因同轮——① runner testharness
  路径 `run_page_scripts_strict()` 前注入一次性初始聚焦（spec flush autofocus
  candidates「ready for post-load tasks」相位：文档序首个合格 candidate 获焦；
  runner 同步 rAF stub 模型下该相位必须落在全部页面脚本之前才可观察；合格性 =
  connected + 非 disabled + 非 hidden input；仅 runner 路径零产品面，
  `FIXME(产品面)` 已标）；② part02 `_navMaybeResetFocus` autofocus 委托由
  `querySelector` 首匹配改 `querySelectorAll` 首合格迭代（disabled/hidden
  input 跳过续查、全灭落 body——delayed 变体「disabled before finished」双案；
  attr 移除面 IDL setter 落 attr 移除免改、committed→finished 间插入 finish 时
  点拾取、finished 后插入无 flush 维持 body）。全量 423→**437**/476（**91.8%**，
  437P/34F/24T/**0NR**——全量 NotRun 清零）；per-subtest diff 恰 14 行
  （autofocus 双变体 NotRun→Pass）、Fail/Timeout 逐行恒等零回归；navigation-api
  252→**266**/280（95.0%）。忙窗批（另两 clone 并行收尾）无 Timeout 抖动。
  fmt 干净 + clippy 零 warning + `make test` 76 suites 22,200P/2F/199I
  （唯二 F = S11 同记跨流 GPU 断言；首跑 1F 忙窗 compositor SIGSEGV solo 恒过
  后全量重跑完成）。reftest/product-smoke 未跑：变更面零布局/绘制触达
  （S11 同理由，708 比较 0 不一致账本仍新）。
  证据：[evidence/2026-10-20-m2-s12-autofocus-load.md](evidence/2026-10-20-m2-s12-autofocus-load.md)。
- **M3-S1 iframe 浏览上下文最小面（2026-10-21）**：M3 门控获批后首切片，零
  runner 产品面变更——①srcdoc 处理模型（part01 合成 text/html wire 进
  `_zwFinishIframeEntry` 全链：srcdoc 优先于 src、地址 about:srcdoc、
  set/change/remove 三触发 + IDL setter 扩面、子发起导航 viaNavigation 绕
  属性处理；fetch 无通道时 src 也记账 entry.url）；②成熟度 servedDoc 双读面
  （cache 立即换新 entry 满足 R388 contentWindow 即时替换语义，contentDocument
  读 `servedDoc || doc`，成熟翻转与元素 load 派发同批延后 microtask——第一版
  cache 整体延迟换破 R388 且 settle 守卫 `cache === entry` 永假致
  network-error 子导航双案挂起 + lazy 双案回归，拆分后全解）；③iframe 子文档
  doc.location（doc-anchored 单例读 `_zwURL` 槽，detached 工厂 null 面不改，
  createHTMLDocument 非 browsing context 语义维持）；④load 信任面（两派发点
  `__zwMakeTrustedEvent('load')`——`_zwDispatchGuard` 对无印记事件翻
  isTrusted=false；reload 给被替换 entry 打 `_zwFrameElementLoadQueued` 封
  双派）；⑤sandbox 指令（token ASCII-only 小写解析防 İ/ſ 形近字误命中 +
  无 allow-scripts 门 `_zwRunIframeScripts` 直接返回 + supports 十九 allow-*
  固定表与属性值无关）；⑥跨源 contentDocument null（无 allow-same-origin =
  opaque origin 同判 + data: URL 同门，getSVGDocument 同语义）；⑦frames
  named access（R177 frames proxy 按 name 内容属性解析到 contentWindow）；
  ⑧解锁域 skip 规则（testharness.rs：stash 协议 / 子发起嵌套导航 / popup
  管理 / 三层嵌套 bc / worker-serviceWorker 面记账 skip，23 案自基线 Fail 池
  转记账；资产补拉 /common/blank.html + not-embeddable.html）。域内
  **25P/35F/27T、恰 4 案翻绿零回归**（document-getters-return-null-for-cross-origin、
  iframe-load-event、iframe-loading-lazy-multiple-queued-navigations、
  iframe-network-error.sub 4/4 子测试）；全量 corpus 对本工作树逐路径零回归
  （navigation-api 5 案 Pass→Fail 经隔离探针定性为 S12 后既有回归、归
  d45247864 邻域非本切片）；part62 十案 10/10 + 引擎库 3334/3334（含 R388
  既有语义）；fmt/clippy 零 warning；make test 76 suites **22,231P/0F/199I**；
  reftest 未跑（零布局/绘制触达）。
  证据：[evidence/2026-10-21-m3-s1-iframe-minimal-face.md](evidence/2026-10-21-m3-s1-iframe-minimal-face.md)。
- **M3-S2 子文档 location 合流 + 合成树 innerText（2026-10-21）**：srcdoc 深语义
  簇三根因同轮——①doc.location↔win.location 同一对象（spec dom-document-location：
  `_zwDocLoc` 扩全 URL 解析读面 + doc.location getter 惰性切 win.location；
  srcdoc_process_attributes / iframe_remove_src 只读 contentDocument.location
  .protocol 不触 contentWindow——S1 href-only 单例返 undefined 即簇根因；成熟度
  门前 toString 面维持）；②子窗 location.hash 写侧（同文档片段导航：'#' 可省 /
  空值剥 / 同值 no-op，URL 槽 + 子会话条目 push + 异步 trusted HashChangeEvent
  到 win.onhashchange；父窗 `_setLocationHash` 通道不动——子窗无 Navigation API
  面；FIXME(popstate) 记账）；③合成树 body.innerText（R3260 近似 ≈ textContent，
  detached 工厂 body 视图；srcdoc-anchor 族经 contentDocument.body.innerText
  断言——textContent 通而 innerText undefined 即簇根因）。域内 27P/32F/28T、
  恰 2 案翻绿（srcdoc-anchor、srcdoc-attribute-reset）+ srcdoc_change_hash
  Fail→Timeout 推进位移（innerText 门过后深入 step 3/4，卡联合 iframe session
  history 缺口——无 Pass 丢失，联合史最小面列下一切片）；全量 corpus 对 S1
  逐路径 diff 其余 425 路径恒等。part63 三案 3/3；引擎库 3353/3353；fmt/clippy
  零 warning；make test 76 suites **22,250P/0F/199I**；reftest 未跑（零布局/
  绘制触达）。
  证据：[evidence/2026-10-21-m3-s2-child-location-innertext.md](evidence/2026-10-21-m3-s2-child-location-innertext.md)。
- **M3-S3 联合 iframe session history 最小面（2026-10-21）**：S2 推进位移案
  srcdoc_change_hash 收口——`_hist_jointSeq` 独立序列（null 惰性播种 = 缺席
  零漂移；标记 `{top: idx}`/`{frameKey, url}` 两形；顶层 `_hist_entries`/
  Navigation records **不膨胀**——spec joint session history 为遍历面概念非
  顶层账本）。播种登记钩子 `__zwPushJointHistoryEntry`（1:1 镜像顶层列表 +
  前向截断 + 当前位随移；两处顶层 push 站点镜像 `{top}` + 50 上限镜像出列 +
  跨文档重置清序）；遍历路由入口分流 `_hist_applyTopTraversal`（原体零改动）/
  `_hist_applyJointTraversal`（frame 标记走 `__zwApplyIframeHashEntry` URL 槽
  + 子窗异步 trusted hashchange；顶层标记复用既有顶层面保 navigate/popstate/
  滚动恢复）；入队可达性按联合序界；子写侧（part05 hash setter）与父遍历合流
  同一落地钩子。FIXME 记账：history.length 仍顶层长度 / 父窗 popstate 不派 /
  顶层标记取消路径不回滚 / 父遍历不动子局部列表。域内 28P/32F/27T、恰 1 案
  翻绿；全量 460P/69F/51T、428 路径恰 1 变化其余恒等（M2 收口面零回归兑现）。
  part63 新增 m3s3 案 1/1；引擎库 3354/3354；fmt/clippy 零 warning；reftest
  未跑（零布局/绘制触达）。
  证据：[evidence/2026-10-21-m3-s3-joint-session-history.md](evidence/2026-10-21-m3-s3-joint-session-history.md)。
- **M3-S4 replace-before-load 最小面（2026-10-21）**：38 案簇根因分解三缺口
  ——runner fetch 通道 `.html` 无 wptserve `{{GET[name]}}` 查询模板替换
  （code-injector 正文原样注入 → SyntaxError，多数 Timeout 共同根因）/
  子脚本裸 `history` 误解析父窗 / 子局部历史无「置换前载入」语义 + 父窗
  back 无 frame 回退。落地：`wpt_apply_get_query_substitutions`（percent-decode
  + 残留占位清空）；子脚本 Function `history` 形参（→ part05 子窗 stub）；
  `__zw_reload_iframe` 记账 `loadedOnce` 承继 + **子发起**（`__zwChildNav`
  标记——iframeReload 与 location.href setter 两通道）载入前**原地替换**
  不 push；`_hist_applyTopTraversal` 越界分支 frame 局部回退
  `_zwJointFrameBackTarget`（局部历史 >1 取 `_zwNavSeq` 最大者落
  `history[len-2]`，`{history:true}` 重载；atEnd AbortError 面优先维持）。
  **R388 收窄**：首版条件无子发起标记 → r388 契约面打红（父层 src 属性更替
  载入派前仍须 push），全量 lib 复验收窄。FIXME 记账：joint-frame-forward
  （无前向槽）/ joint-frame-back（多 frame 最近活动近似）/ 子窗
  location.search 无 setter / 子 form.submit() 落父窗账本不驱动 frame /
  joint history.length（S3 既立）。全量 460P→**464P**（4 案 Timeout→Pass），
  簇内 3 案 Timeout→Fail 推进位移，**簇外 0 路径变化**；域内
  replace-before-load 4P/18F/17T。part63 新增 m3s4 两案 2/2；引擎库
  3357/3357；fmt/clippy 零 warning；reftest 未跑（零布局/绘制触达）。
  证据：[evidence/2026-10-21-m3-s4-replace-before-load.md](evidence/2026-10-21-m3-s4-replace-before-load.md)。
- **M3-S5 parent→child postMessage + 子文档 onmessage 绑定（2026-10-21）**：
  上轮 429 断点 WIP 收口。block_modals 四案簇根因：子文档裸 `onmessage =`
  落**父窗 globalThis**（new Function 内非严格裸赋值穿透宿主）——R2932
  setter 误注册父窗监听，子窗 `win.onmessage` 永缺席，父→子 postMessage
  投递（stub `win.dispatchEvent` → `win['on'+type]` 直调既有面）无人接。
  修 = ①子窗 stub 预置 `onmessage: null` 槽（spec getter-null 语义 +
  with 捕获前提：属性在场才落 with 对象）；②子文档脚本条件
  `with(window)` 包合（正则 `(^|[^\w.$])onmessage\s*=[^=]/`——排除成员
  赋值与 `==`/`=>`；未命中子文档逐字节不变）。part63 新增
  m3s5_parent_to_child_postmessage_roundtrip（回程序列 undefined/false/
  null/undefined + 绑定面防回环断言）。域内 28P/32F/27T→**32P/32F/23T**
  （恰 4 案 Timeout→Pass）；全量 464P→**468P**（581 案行）对 S4 逐路径
  diff 恰 4 行变化、其余 577 恒等。引擎库 3358/3358；webmessaging 跨套件
  回归臂 132P/3F（差项全为账本既有挂账 user-activation + `/common/gc.js`
  wpt-data 重建资产洞——见下方待办）；fmt 干净 + clippy workspace 零
  warning；`make test` 76 suites **22,264P/0F/199I**；reftest 未跑
  （零布局/绘制触达）。
  证据：[evidence/2026-10-21-m3-s5-postmessage-child-onmessage.md](evidence/2026-10-21-m3-s5-postmessage-child-onmessage.md)。

## M4 收口评估（2026-10-09）

**判定**：DC-1 ✅ / DC-2 ✅（主簇） / DC-4 ✅；**DC-3 ⏳ 用户门控 pending（未获豁免）→ goal
不判 DONE，维持 Active 推进态**。残余 136 Fail/Timeout 全量盘点定性：A 域回流（DOM 焦点
~9 / 渲染 ~20 / element IDL 2）、B runner 形态（replace-before-load 38 + bfcache 族 ~10 +
跨文档链 ~6）、C 可切片 ~25（form submit 5、state 分槽 2、activation 暴露 4、dispose 深簇 6
等）。P5 bfcache/fission 定稿：bfcache 无实现面挂账至 M3 后续立项；fission charter 排除维持。
证据：[evidence/2026-10-09-m4-closeout-assessment.md](evidence/2026-10-09-m4-closeout-assessment.md)。

## 下一步计划

1. **C 类余项定稿（S7 后）**：可切片项已清——navigate-multiple-location/-pushState 2T
   （task 排队模型）经评估挂账（风险/收益不成立，见 S4Q evidence）；same-url-replace 双案
   与 007（S4W strict 收窄 + 同 URL replace 面翻绿）；navigate-history-push-not-loaded
   （S4X readyState 宿过渡）；intercept-multiple-times-reject + 同源跨文档锚（S4Y）；
   scroll-restoration-fragment-scrolling-samedoc（S4Z）；order-in-new-document-navigation
   （S5 pagereveal）；navigate-svg-anchor-fragment（S6 派发 click 激活）；
   **scroll-to-fragid 族 6 案（S7——getElementById 特殊字符 id 抛错 +
   documentElement.scrollTop 镜像 + 目标位双计 + fragment 聚焦步）** +
   **scroll-position-vertical-lr（S8 块起始边 scrollX 对齐）**；剩余余项：vertical-rl/
   inline-nearest/reload 族经探针三分维持渲染域、bfcache 族（helper 资产双缺口）、
   focus-reset 2T（send_keys TAB）逐案核验挂账属实；其余全为 A 域回流与 B 类形态
   缺口（下两条）。S10 复核再收 runner 形态挂账 location_port 一案、008 定性改判
   （见切片账）。
2. **replace-before-load 38F 重定性 → M3 依赖**（2026-10-09 勘察）：全簇为 iframe 载体
   （setupSentinelIframe/insertIframe + 子文档 load 前自导航），断言 iframe 自有 session
   history 的 replace 语义——单文档 runner 形态不可达，随 M3 frame tree 一并解锁。
4. **M2 ③ 跨文档导航语义**（navigating-across-documents 2/42）：主体依赖跨文档导航链
   + testdriver 用户手势，单文档 runner 形态不可达——runner 形态升级前仅记账。
5. **M2 ④-D 残余挂账域回流点**：~~Tab 键顺序焦点导航~~（S11 销项）；
   ~~autofocus load 期处理~~（S12 销项——runner 初始 flush + 结算委托合格性门，
   全量 NotRun 清零；产品宿主解析路径 autofocus 初始处理仍未接线，
   `FIXME(产品面)` 挂真实 focus 域）；
   scroll anchoring + rect 快照刷新（渲染域，回流 rendering-compat）。
6. **M3**：frame tree 最小面——**已批准启动（2026-10-21 用户批复「批准 M3 最小面」，
   未豁免 DC-3）**；**S1~S5 已落地（2026-10-21，见切片账；S5 parent→child
   postMessage + 子文档 onmessage 绑定——全量 464P→468P，577 路径恒等零回归）**。
   下一切片候选（按域内 F 池密度）：子发起嵌套导航控制（iframe_sandbox_navigate_*
   ——skip 记账池 4T，S5 已解锁子窗 message 送达前置；需 sandbox 祖先导航守卫 +
   父窗 popstate 监听面）、popup 窗口管理（iframe_sandbox_popups_* + 用户手势）；
   replace-before-load 余缺口按 FIXME 记账（子窗 search setter、子 form 提交
   frame 定向、joint history.length、pageshow 簇、user-activation 断言面）。
   ~~服务端 stash 协议端点~~（2026-10-21 勘误：corpus 无消费案，销项）。
   ~~parent→child postMessage~~（S5 销项）。
   fission/进程模型触碰即 BLOCK 上报用户。
7. **跨套件资产洞记账（2026-10-21 S5 轮发现）**：wpt-data 2026-10-20 整目录
   重建后 `/common/gc.js` 未补拉——消费方 webmessaging（garbage-collected +
   Channel_postMessage_Blob 两案 load 失败，对 archive/webmessaging-compat
   收口账本 133/135 的差项归此）、css/css-nesting、navigation-api
   （navigate-from-initial-about-blank-gc，本 goal 现处 skip 面零影响）。
   共享面不硬解，归 wpt-data 资产账（若后续轮次主导相关套件时随其 goal
   脚本补拉）。

**待用户决策清单**：
- [x] M3 iframe 深结构切片启动授权（未获批期间 DC-3 保持 pending，不阻塞 M2 收口）
  **2026-10-04 已征询待批复**（goal 待决策巡检 msg `om_x100b63125d66c4a8b2070829b8dd058`；
  建议 = 暂不批准——M1/M2 尚未启动，待 M2 收口时随推进一并裁决）。批复前维持立项态挂起。
  **2026-10-06 48h 跟进提醒已发（一次性，到期）**（msg `om_x100b63647f7910a0b49d70def6b88ff`）：
  仍零回复——不回复即默认维持挂起，此后不再重复催办。
  **2026-10-20 裁决时点已到**：M2 已收口（437/476，**91.8%**，可切片项清零 +
  S12 全量 NotRun 清零、对 S11 账本零回归），10-04 建议的「待 M2 收口时随推进
  一并裁决」条件成立，已随飞书进度通知一并提请裁决
  （msg `om_x100b625b165f94acdc025cae55f0621`；批准 M3 最小面 / 明确
  豁免 DC-3 判 DONE / 维持挂起三选一）；
  未批复前维持挂起，不再催办。
  **2026-10-21 已批复：批准 M3 最小面**（用户当面批复；未豁免 DC-3，
  DC-3 随 M3 实施结果判定）——M3 解除挂起转入执行，本清单清零。
