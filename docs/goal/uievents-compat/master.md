# UI/指针事件兼容 — 运行时控制面板（master.md）

**入口文档**: [../uievents-compat.md](../uievents-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-04（M3 尾簇 4+5 落地）

## 当前状态

**M3 + 尾簇 1/2/4/5 落地（2026-10-04）**：基线 230P → M2 片 1 343P → M3 375P →
尾簇 1 451P → 尾簇 2 457P → 尾簇 4 492P → **502P（+272 累计）**
（corpus：502P/596F/97TO，`TIME_LIMIT=3600`——per-case 30s 超时 ×~100 TO 案
注定 corpus 超 20min 默认 test-guard 时限；尾簇 4→5 零回归、+10 subtest）。
证据：[evidence/2026-10-03-m2-slice1.json](evidence/2026-10-03-m2-slice1.json)（M2 片 1）、
[evidence/2026-10-03-m2m3-after.json](evidence/2026-10-03-m2m3-after.json)（M3 后）、
[evidence/2026-10-03-m3-tail.json](evidence/2026-10-03-m3-tail.json)（尾簇 1 后）、
[evidence/2026-10-04-m3-tail4.json](evidence/2026-10-04-m3-tail4.json)（尾簇 4 后）、
[evidence/2026-10-04-m3-tail5.json](evidence/2026-10-04-m3-tail5.json)（尾簇 5 后）。
门禁：clippy -D warnings、fmt 全绿；make test 见尾簇 4 提交说明（19,614P/0F）。

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

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | uievents + pointerevents corpus 导入 + 基线 | ✅ 2026-10-03 |
| P2 | 鼠标事件序/坐标/click 组合语义修齐 | 🔄 核心已落地；残余 = mousemove-between（视口命中精度）、wheel 三案、focus-events 四案、interface keyboard-click、uievents/mouse 尾簇（layerX/chorded buttons/image-map） |
| P3 | Pointer 生命周期 + capture 三方法 + enter/leave 边界序 | 🔄 核心 + mutation 族 + 重入面已落地（尾簇 4/5）；残余 = after_target_appended ?touch 5 案（touch 接触失效 enter 序/teardown 抑制）、pointercancel/touch-action 交互面；from_slot 案阻塞于 declarative shadow DOM（shadowrootmode 未实现——web-components 域前置，挂账）；interleaved 族行为面已对齐（残余 = 上游 expected 记账 bug，pin 版即 Fail，不再追） |
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
- **M3 尾簇 5（2026-10-04，本轮）**：insert-under-cursor 重入 + touch 隐式捕获
  up 前清除。492P→502P。

## 下一步计划

1. **M2/M3 尾簇**（按 Throughput 排序）：a) after_target_appended ?touch 残余 5 案
   （?mouse 已全绿）——Chromium touch 接触失效语义收尾：D2 隐含迁移 enter 序
   （上游 expected 为 child→parent 内层先序，逆于 ?mouse 同文件 parent→child）、
   重入后 up 的 teardown 抑制、enter@parent 抑制；b) wheel 源 scroll 命令（wheel
   三案）；c) mousedown→focus 默认动作链（focus-events 四案）；d) uievents/mouse
   尾簇（layerX/layerY、chorded buttons 位、image-map 命中）。
2. **M4 收口**：touch-action 解析/计算值核对（parsing 三案 0P 待查）、
   touch-events / pointerlock / IME / touch-action 交互面挂账定稿、DC-4 全绿门禁
   （make test + clippy + fmt + reftest 零回归 + product-smoke）。

**待用户决策清单**：（空——pointerlock 挂账重入 = 用户点名；declarative shadow
DOM 前置如需立项请点名）
