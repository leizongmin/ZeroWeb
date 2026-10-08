# 导航与浏览上下文兼容 — 运行时控制面板（master.md）

**入口文档**: [../navigation-compat.md](../navigation-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-10-09（M2-S1~S4G 十切片——location-interface 95.3% / traversal 93.3% /
history-interface 91.8% / navigation-api 69.7%；全量 20.9%→67.4%）

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
| P3b | Navigation API（window.navigation 全域） | 🔶 S4~S4G 十段落地（→**69.7%**）：read side/navigate/intercept/traverse/scroll-behavior/focus-reset/location exotic/precommitHandler/traverseTo/ongoing-abort 事件序全收；余 anchor-download ×2T（downloadRequest 面）、Tab 焦点导航 2T + autofocus load 期 7 NotRun（DOM 焦点域）、scroll-behavior reload 族 4F（渲染域）、userInitiated/跨文档（runner 形态） |
| P4 | iframe 浏览上下文最小面（contentWindow/frames/parent/top + 属性语义） | ⏳ M3 **用户门控** |
| P5 | bfcache / fission 挂账定稿 | ⏳ M4 |

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
- **质量门禁（十切片）**：`make test` 全绿 20,225 P / 0 F（S4G 轮；锚 20,209 → 20,225 含兄弟
  流新增）；`make reftest` 704/704 零失败（2026-10-08 S4D~S4F 三轮 shim 变更后统一复验）；
  clippy -D warnings 零 warning；fmt 零 diff；每轮全量语料零回归（per-subtest 精确 diff）。
  全量批墙钟 TIME_LIMIT=2700（批内不与 make test 并发）。

## 下一步计划

1. **M2 残余小簇**：anchor-download（2T——anchor download 属性 → downloadRequest/canIntercept
   面，锚点击导航管线小改）；scroll-to-fragid 变体（编码/几何，部分回流渲染/焦点域）。
2. **replace-before-load 38F 重定性 → M3 依赖**（2026-10-09 勘察）：全簇为 iframe 载体
   （setupSentinelIframe/insertIframe + 子文档 load 前自导航），断言 iframe 自有 session
   history 的 replace 语义——单文档 runner 形态不可达，随 M3 frame tree 一并解锁。
3. **M4 收口评估**：DC 逐项判定（DC-1 ✓ / DC-2 轻面主簇已收 / DC-4 门禁连续全绿；DC-3
   用户门控维持 pending）；bfcache/fission 挂账定稿。
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
