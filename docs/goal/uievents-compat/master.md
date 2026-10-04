# UI/指针事件兼容 — 运行时控制面板（master.md）

**入口文档**: [../uievents-compat.md](../uievents-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-04（M3 尾簇 6a+6b+6c 落地）

## 当前状态

**M3 + 尾簇 1/2/4/5/6 落地（2026-10-04）**：基线 230P → M2 片 1 343P → M3 375P →
尾簇 1 451P → 尾簇 2 457P → 尾簇 4 492P → 尾簇 5 502P → 尾簇 6a 1107P →
**1115P（+885 累计）**
（corpus：1115P/743F/97TO，`TIME_LIMIT=3600`——per-case 30s 超时 ×~100 TO 案
注定 corpus 超 20min 默认 test-guard 时限；尾簇 6a 后总册 1195→1955——fractional
untrusted 脚本中断解除、~760 subtest 全量入册；6a/6b+6c 双轮零回归）。
证据：[evidence/2026-10-03-m2-slice1.json](evidence/2026-10-03-m2-slice1.json)（M2 片 1）、
[evidence/2026-10-03-m2m3-after.json](evidence/2026-10-03-m2m3-after.json)（M3 后）、
[evidence/2026-10-03-m3-tail.json](evidence/2026-10-03-m3-tail.json)（尾簇 1 后）、
[evidence/2026-10-04-m3-tail4.json](evidence/2026-10-04-m3-tail4.json)（尾簇 4 后）、
[evidence/2026-10-04-m3-tail5.json](evidence/2026-10-04-m3-tail5.json)（尾簇 5 后）、
[evidence/2026-10-04-m3-tail6a.json](evidence/2026-10-04-m3-tail6a.json)（尾簇 6a 后）、
[evidence/2026-10-04-m3-tail6bc.json](evidence/2026-10-04-m3-tail6bc.json)（尾簇 6b+6c 后）。
门禁：clippy -D warnings、fmt 全绿（见尾簇 4 提交说明）；尾簇 6 系纯 JS 变更
（node --check + 聚焦族跑）。

**M3 尾簇 4（2026-10-04，650cdfa2e）——mutation 族收口**。三件套：

1. **runner 执行时点重解析 + shim html 结构性刷新**：指针命令出队时以 fresh
   gBCR 做元素 origin 中心+offset 命中（`resolve_pointer_target`，计划只编
   origin+offset、'@' 前缀 = viewport 绝对）；shim outerHTML 变化 → `render_html`
   全量重建 + handle 重绑 + rect 快照刷新（`WebView::refresh_if_html_changed`，
   R100 persistent_handle_nodes 经 `rebind_handle_node` 回填）。刷新经**宿主
   mutation 代际计数**（`js_dom_bridge::mutation_version`）门控——无 mutation 批
   零成本跳过（探测环 ~1ms/拍，无门控的逐拍百 KB 序列化曾跑穿 test-guard
   time-limit）。
2. **detach→re-insert 跨批语义（shim 根因）**：页内 `removeChild` 后跨 turn
   `appendChild` 曾致宿主 `InsertAdjacentSelElement` child 失配 lenient skip
   （R361 批内 stash 跨批即弃，元素永久丢失——探针实证）。新增
   `DetachedNodeStash`（selector→序列化片段，FIFO 封顶 32，导航边界清）：
   `Remove`/`RemoveChildAt` 记账、insert 失配时重解析片段插回（片段跨宿主全量
   重建有效，NodeId 跨重建失效故用片段）。配 `_zwIsConnected` 同步移除标记
   优先（脚本内 remove 后 host apply 前连接性即为否）。
3. **pointer/mouse 双层跨界拆分**：`_zwPtrState.mouseOverSel` 独立 compat mouse
   hover 位——touch 抬起悬停拆除仅 pointer 层（mouse 边界只随真实位置跨界，
   WPT ?touch mouse 子测试面）；pointerdown/up 派发中目标被页内 listener 移除 →
   compat mouse 事件**重定向**新落点（命中测试/宿主父链回退；hoverable 双层
   cross、touch 仅 mouse 层——`_zwRetargetSel` 加 skipCross 参数）。

结果：after_target_removed 主文件 3→**12/12 全绿**；37 subtest 净改善
（after_target_removed +12、capturing_boundary_event_handler_at_ua_shadowdom +9、
after_target_appended +9、pointerup_after_pointerdown_target_removed +3 等）。
表面回归 6 全部归因排除：appended_interleaved ×3 = vacuous pass 丧失（该族
expected 含 click@ 记账而页内 logEvent 只收 "mouse" 前缀——**pin 版上游即
Fail**，见 [evidence/2026-10-04-upstream-tentative.md](evidence/2026-10-04-upstream-tentative.md)）；
setpointercapture_to_same_element_twice ?touch ×2 = 已存于 committed HEAD
（m3-tail.json 基线早于尾簇 1 touch 语义）；predicted_events ?touch ×1 =
case 超时调度伪影（双树隔离跑逐字节同型）。

**M3 尾簇 5（2026-10-04，本轮）——insert-under-cursor 重入 + touch 隐式捕获
up 前清除**。两件：

1. **hover 重入旗标**：hover 元素被 remove（`_zwMarkRemoved` 钩子）或同父
   move 重挂（appendChild 已连接 sel 子，R334 分支记旗标——move 不抽
   Remove mutation 不走 mark）→ 下一指针事件派发前对回连的原 hover 元素补派
   over/enter 双层重入面（`_zwReentryCheck`，move/down/up 三站；跨界他元素即
   失效）——WPT after_target_appended moved variant「(child-moved) →
   pointerover@child → pointerup@child」断言面。
2. **touch 隐式捕获 up 前清除**：`_zw_implicit_` 前缀 pending 于 up 序列入口
   删除（此前 Process-Pending 于 pointerup 派发时换防并把 up 重定向 down
   目标；Chromium 行为 up@新命中目标——WPT after_target_appended ?touch
   「pointerdown@parent,(child-attached) → pointerup@child」断言面）。显式
   setPointerCapture 不受影响（capture 族回归扫零变化）。

结果：after_target_appended 9→**19P**（?mouse 全绿；?touch 残余 5 案 = Chromium
touch 接触失效语义尾簇：D2 隐含迁移 enter 序上游 expected 为 child→parent
内层先序〔逆于同文件 ?mouse 的 parent→child〕+ 重入后 up 的 teardown 抑制
+ enter@parent 抑制）。

**M3 尾簇 6a（2026-10-04，本轮）——untrusted 事件构造语义（分数坐标 + tilt/
angle 互换）**。两件，均纯构造层（派发路径共用 ctor 但面不变）：

1. **坐标 floor + page/offset 派生**（`_zwMouseCoordInit`）：MouseEvent/WheelEvent/
   DragEvent 全型 + PointerEvent 的 click/auxclick/contextmenu 三型坐标构造值
   floor（UI Events long 语义；PE fractional 例外集——其余 pointer 型保 double）；
   pageX/pageY/offsetX/offsetY dict 未给时自 clientX/clientY 派生（旧版落 0——
   fractional untrusted 168F 根因）。
2. **tiltX/tiltY ↔ azimuthAngle/altitudeAngle 归一**（`_zwPointerTiltInit` +
   `_zwTiltToAzAlt`）：spec 换算 + ±90° 角点简并（tan 爆炸 → altitude 0/双 90
   azimuth 0）+ 常规角 1-ulp snap（tan(π/4)≈0.9999… 的 atan 链漂移）。

结果：**corpus 502P→1107P（+605，零回归）**——corpus 总 subtests 1195→1955
（fractional untrusted 此前脚本中断未及注册的 ~760 subtest 全量入册）：
fractional untrusted 104→680P（+576）、tilt 1→24P（全绿）、constructor 双文件 +6。
证据：[evidence/2026-10-04-m3-tail6a.json](evidence/2026-10-04-m3-tail6a.json)。
门禁：shim 拼接 node --check 全绿（纯 JS 变更，无 Rust 面）。

**M3 尾簇 6b+6c（2026-10-04，本轮）——mousedown→focus 默认动作链 + touch 接触
失效语义收口**。三件：

1. **焦点迁移序修齐**（part04 proxy focus/blur 陷阱）：失焦相位先于获焦相位且
   blur 先于 focusout（WPT focus-events expected「blur@a → focusout@a →
   focus@b → focusin@b」；旧序 blur 迟到获焦相位后——focus.html 族全灭根因）+
   relatedTarget 双向（blur/focusout@旧 携新焦点、focus/focusin@新 携旧焦点）+
   失焦相位 activeElement 落空（spec focusing steps）。slice22 的
   `__zw_host_focus/__zw_host_blur` 首次接线（`_zwFocusSel`，down 序列 mousedown
   未取消时可聚焦目标迁移——此前 `script_host_focus` 无 caller 死代码）。
2. **up 内变异 post-up 结算**（尾簇 6c）：mutTick 代际（remove/appendChild 钩子
   推进）+ wire 落点记录（upMutSel）——同元素重插 → 重入面（F4/F5）；异元素插入
   → 跨界序**延迟结算**（pendingCross 于下一指针命令入口 flush——up 内插入元素
   本 turn 宿主树不可派发，zwprobe 实证空目标事件）；移除型变异不触发结算两分支
   （拆除照旧——after_target_removed pointerup-remover 断言面）。
3. **child-first enter 序**：跨界后拆除武装一次（`touchChildFirstEnter`，pointer
   层 enter 链目标先序消费即清——F1-D2 断言序；无跨界裸拆除保持外先内——F3/F4
   D1 断言序）。

结果：**corpus 1107P→1115P（+8，零回归）**：after_target_appended **24/24 全绿**
（?touch 3→8P——teardown 抑制 + 延迟跨界 + child-first 序）、focus-events +3
（focus / focus-contained / focus-automated same-DocumentOwner；残余 = iframe 跨
文档焦点（different-DocumentOwner）与 keydown→focus activation 两案）。
证据：[evidence/2026-10-04-m3-tail6bc.json](evidence/2026-10-04-m3-tail6bc.json)
（含 2 案本地 zwprobe 探针已剔除入账）。门禁：node --check + 双族聚焦跑全绿。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | uievents + pointerevents corpus 导入 + 基线 | ✅ 2026-10-03 |
| P2 | 鼠标事件序/坐标/click 组合语义修齐 | 🔄 核心已落地；残余 = mousemove-between（视口命中精度）、wheel 三案（scroll 源重放——wheel-basic/deadlock 可解，scrolling 需真滚动）、interface keyboard-click、uievents/mouse 尾簇（mouseover-at-removing 58F 与 6c 同族待复测、layerX/chorded buttons/image-map/modifier-no-movement） |
| P3 | Pointer 生命周期 + capture 三方法 + enter/leave 边界序 | ✅ 核心 + mutation 族 + 重入面 + touch 接触失效收口（尾簇 4/5/6c——after_target_appended 24/24 全绿）；残余 = pointercancel/touch-action 交互面、iframe 跨文档焦点；from_slot 案阻塞于 declarative shadow DOM（shadowrootmode 未实现——web-components 域前置，挂账）；interleaved 族行为面已对齐（残余 = 上游 expected 记账 bug，pin 版即 Fail，不再追） |
| P4 | touch-events / pointerlock / IME 组合挂账定稿 | ⏳ M4 |

## 已完成切片

- **M1（2026-10-03）**：corpus 导入 + runner 通道 + 分类基线 1024 subtests 230P +
  suites CSV 回填（[evidence/2026-10-03-m1-baseline.md](evidence/2026-10-03-m1-baseline.md)）。
- **M2 片 1（2026-10-03，5d5669118）**：Actions stub 重写为上游多源 API 面 +
  PointerEvent 分支 + 边界事件序 + click 组合序（dblclick/公共祖先/auxclick/
  contextmenu）+ viewport 命中测试。+113。
- **M3（2026-10-03，19d5bb1c4）**：Pointer Capture 语义化 + 捕获重定向 + active
  pointer 状态机 + Actions 逐步重放 + Event.prototype 面。+32。
- **M3 尾簇（2026-10-03，7561ba05a）**：element-origin 偏移语义（中心+offset 命中）
  + coalesced/predicted stub。+76。
- **M3 尾簇 2（2026-10-04，f3564b5c2）**：目标移除重定向（命中测试重定向 +
  dangling 边界面 + touch 悬停拆除 + 祖先链快照回退）。+6（after_target_removed
  pointerdown-remover 3 variant 转绿；uievents/mouse 17→19P、order-of-events
  6→7P）。
- **M3 尾簇 4（2026-10-04，650cdfa2e）**：mutation 族收口（执行时点重解析 + 跨批
  detach 片段 stash + 双层跨界 + compat 重定向）。457P→492P。
- **M3 尾簇 5（2026-10-04，d6433f0e8）**：insert-under-cursor 重入 + touch 隐式
  捕获 up 前清除。492P→502P。
- **M3 尾簇 6a（2026-10-04，68edcce1b→baaabf835）**：untrusted 构造语义（坐标
  floor + page/offset 派生 + tilt/angle 归一）。502P→1107P（总册 1195→1955）。
- **M3 尾簇 6b+6c（2026-10-04，本轮）**：mousedown→focus 默认动作链（迁移序 +
  relatedTarget + slice22 钩子接线）+ touch 接触失效收口（up 内变异 post-up
  结算 + 延迟跨界 + child-first 序）。1107P→1115P。

## 下一步计划

1. **M2/M3 尾簇**（按 Throughput 排序）：a) uievents/mouse 尾簇——mouseover-at-
   removing-mousedown-target 58F **根因已定位**（zwprobe 实证：remove→re-append
   跨 turn 循环中，shim 侧 reappend 记账完成〔childNodes/parentNode 对〕但宿主侧
   三路全失——`__zw_contains`/query 快照/getElementById 均无子；InsertAdjacentSel
   Element 的 stash 消费链（Remove 记账 → child 失配 → NodeId stash → 片段 stash
   重解析插回，js_dom_bridge.rs:1378 区）在该时序下未生效；迭代 0 断言面本身已过
   （mousedown@child→mouseover@parent→mouseup@parent），58F 全为 finally 再挂失败
   的级联 unhandled rejection——**DetachedNodeStash/shim registry 身份保真**专项，
   勿用 setTimeout 延迟 wire（runner 环境定时器不转，且破坏 F1/F2 in-turn attach
   ——已试已回退））、boundary_events_after_reappending 11F（重插语义同族）、
   modifier-no-movement 16F、image-map 8F、layerX/layerY；b) wheel 源 scroll 重放
   （wheel-basic/deadlock——Actions scroll 步记账不重放改重放 + `__zw_wheel` shim
   钩子）；c) focus 残余两案（iframe 跨文档焦点、keydown→focus activation——键盘域
   邻接，挂账候选）。
2. **M4 收口**：touch-action 解析/计算值核对（parsing 三案 0P 待查）、
   touch-events / pointerlock / IME / touch-action 交互面挂账定稿、DC-4 全绿门禁
   （make test + clippy + fmt + reftest 零回归 + product-smoke）。

**待用户决策清单**：（空——pointerlock 挂账重入 = 用户点名；declarative shadow
DOM 前置如需立项请点名）
