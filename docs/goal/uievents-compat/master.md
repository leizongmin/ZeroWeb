# UI/指针事件兼容 — 运行时控制面板（master.md）

**入口文档**: [../uievents-compat.md](../uievents-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-05（M3 尾簇 10 落地）

## 当前状态

**M3 + 尾簇 1/2/4/5/6/7/8/9/10 落地（2026-10-05）**：基线 230P → M2 片 1 343P → M3 375P →
尾簇 1 451P → 尾簇 2 457P → 尾簇 4 492P → 尾簇 5 502P → 尾簇 6a 1107P →
尾簇 6b+6c 1115P → 尾簇 7 1167P → 尾簇 8 1202P → 尾簇 9 1218P → **1570P（+1340 累计）**
（corpus：1570P/281F/97TO，`TIME_LIMIT=3600`；97TO 恒值。尾簇 10 净 +352P 零回归——
fractional untrusted 11 MouseEvent 型 × 32 subtest 全绿；逐 subtest 对账
improvements=356/regressions=0）。
证据：[evidence/2026-10-05-m3-tail10.json](evidence/2026-10-05-m3-tail10.json)（尾簇 10 后）、
[evidence/2026-10-05-m3-tail10.md](evidence/2026-10-05-m3-tail10.md)（根因链 + 排除路径）、
[evidence/2026-10-04-m3-tail9.json](evidence/2026-10-04-m3-tail9.json)（尾簇 9 后）、
[evidence/2026-10-04-m3-tail9.md](evidence/2026-10-04-m3-tail9.md)（根因链 + 排除路径）、
[evidence/2026-10-04-m3-tail8.json](evidence/2026-10-04-m3-tail8.json)（尾簇 8 后）、
[evidence/2026-10-04-m3-tail8.md](evidence/2026-10-04-m3-tail8.md)（根因链 + 排除路径）、
[evidence/2026-10-04-m3-tail7.json](evidence/2026-10-04-m3-tail7.json)（尾簇 7 后）、
[evidence/2026-10-04-m3-tail7.md](evidence/2026-10-04-m3-tail7.md)（根因链 + bisect 记录 +
排除路径）、
[evidence/2026-10-03-m2-slice1.json](evidence/2026-10-03-m2-slice1.json)（M2 片 1）、
[evidence/2026-10-03-m2m3-after.json](evidence/2026-10-03-m2m3-after.json)（M3 后）、
[evidence/2026-10-03-m3-tail.json](evidence/2026-10-03-m3-tail.json)（尾簇 1 后）、
[evidence/2026-10-04-m3-tail4.json](evidence/2026-10-04-m3-tail4.json)（尾簇 4 后）、
[evidence/2026-10-04-m3-tail5.json](evidence/2026-10-04-m3-tail5.json)（尾簇 5 后）、
[evidence/2026-10-04-m3-tail6a.json](evidence/2026-10-04-m3-tail6a.json)（尾簇 6a 后）、
[evidence/2026-10-04-m3-tail6bc.json](evidence/2026-10-04-m3-tail6bc.json)（尾簇 6b+6c 后）。
门禁：workspace 17,246P/0F、renderer lib 204P/0F、reftest 704/704、fmt + clippy
（quickjs 面）全绿；shim 拼接 node --check 全绿。

**M3 尾簇 10（2026-10-05，本轮）——untrusted 事件构造 native 路径坐标语义**。
native MouseEvent 模板（唯一生产构造路径——R384）坐标族 floor 化（`init_floor_int`，
浏览器语义非 WebIDL truncate）+ pageX/Y/offsetX/Y 派生（init 显式则 floor 采信、
缺省 = floor(client)；`is_number` 门防缺失键 NaN→0 压派生——首轮 4 范围全灭根因）。
对齐尾簇 6 的 shim 面 `_zwMouseCoordInit`。corpus **+352P 零回归**（细节与排除路径见
[evidence/2026-10-05-m3-tail10.md](evidence/2026-10-05-m3-tail10.md)）。

**M3 尾簇 9（2026-10-05，ce7ca7f37）——image-map 命中 + 跨目标 click 组合 + 跨批 handle
插入序列化落地**。五件 + 宿主活性修复（根因链与排除路径见
[evidence/2026-10-04-m3-tail9.md](evidence/2026-10-04-m3-tail9.md)）：

1. **image-map 几何命中**（runner 命中测试 JS）：`img[usemap]` → map → area 的
   rect/circle/poly 几何命中（文档序首个 inside 即 top-most）。+12。
2. **跨目标 click 组合以实派 target 为准**：`effDown = st.downSel || downSel ||
   upSel`——stub 编码的 downSel 是 origin 元素近似（image-map 面 origin=img 实派
   =area）。+2。
3. **变异代际扩面**：悬停元素属性变异 / IMG image-map 属性 / AREA 插入（三站）推进
   `mutTick`。
4. **settle 延迟结算**：命中目标未落 applied 快照时 defer 不消费代际。
5. **指针命令入口 settle**（`mut_hover_settle_if_dirty`）：同批命令连发场景的渲染
   机会近似。

附带宿主活性修复：createElement 产物跨 apply 代际（含 refresh_if_html_changed 重建
窗口）后 `InsertBefore{child_handle}` 硬错曾**卡死共享 mutation 队列**（webview 吞咽
Err 游标不推进，批尾全丢）——创建代际印章 + 跨批改走 child outerHTML 序列化落地
（InsertAdjacentHtml 物化，shim 唯一真相）；apply 硬错批丢弃推进游标（P19 钉死的
apply 侧 lenient 禁令不动，活性修在 webview 批丢弃层）；id 型 handle 节点移除改派
selector Remove（镜像绑定失效时 RemoveHandle no-op 泄漏）。

**M3 尾簇 8（2026-10-04，6045a7462）——mutation 驱动悬停重定向 + 修饰键全局态**。三件
（细节与排除路径见 [evidence/2026-10-04-m3-tail8.md](evidence/2026-10-04-m3-tail8.md)）：

1. **修饰键全局态**（`_zwModifiers`）：keydown/keyup 维护（事件位先算、后落态），
   合成 pointer/mouse 事件缺省携带当下态（detail 显式值优先）。+32 主修复面。
2. **瞬态悬停重结算**（`hoverTransient {sel, reattached, px, py}` + 双路 settle）：
   rAF 派发点同步结算（渲染机会边界近似——OFF 模式 rAF 同步执行、测试尾段同一脚本
   任务，探测环来不及）+ 探测环几何结算（poll → fresh 命中测试 → 端点/瞬态两段
   跨界）。px/py 快照守卫防 cleanup 重插（指针已移开）泄漏。
3. **R3254-K2 修饰键单测校准**：持久态语义下每臂补 keyup 复原（意图不变）。

**M3 尾簇 7（2026-10-04，1cbbeaaa8）——R334 重插同 turn 查询可见性 + variant 大小写
保真**。三件：

1. **by-selector 重插索引**（part05 `_zwPendReselBySel`）：R334 sel 子重插的结构 wire
   异步 drain，窗口内 gEBI 恒 null（mouseover-at-removing 30 迭代链 i1 起同步抛错 →
   整链 microtask 塌缩进单轮、永不自愈）。R51c by-id 索引登记不了它——重插子宿主侧已
   无节点，`nd.id` 读链落空返 ''。以 R334 已知 `__zwSelector` 为键直登，gEBI 查
   `'#'+id`（in-doc 门 = 槽位 parentSel 树中判定）；表随 apply 代际 bump 清空。
   mouseover-at-removing **1P/29F→30P×2 全绿**。
2. **探测轮首 flush**（`WebView::flush_pending_shared_mutations` + `take_probe` 入口）：
   上一轮 JS 排队的结构 wire 本轮 JS 前落活 DOM（跨轮查询/命中少断一代；队列空零成本）。
3. **variant 值大小写保真**（`case_variants`）：旧实现小写源码后提取 variant——
   `?Shift`/`?preventDefault=…` 被写成 `?shift`/`?preventdefault=…`，页面
   `URLSearchParams.get`（大小写敏感）miss。修复后 modifier 族 `keyDown(undefined)`
   16F 解除为真语义缺口 2P/4F×4；click_during_parent_capture / synthetic-button-state
   的 pd=/buttonType= 变体首次跑真实配置。

尾簇 7 排除路径（bisect 实证，勿重试）：`with_query_view_doc` 把 `InsertAdjacentSelElement`
纳入 live_ok 排除 + 视图烘焙——live_ok 扫 append-only 队列全史，任一 sel-insert 入队
（即使已 drain）即永久切视图路径，after_target_removed 4P→2P 真回归。

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

**DC-4 门禁（2026-10-05，尾簇 10 后）**：make test 全腿 **19,825P/0F**（锚恒）；fmt +
clippy -D warnings（zero-engine）全绿；**reftest 704/704 零回归**（不一致 0）；shim
拼接 node --check 全绿。历史（尾簇 9 后）：clippy engine/webview/wpt-runner 三 crate
全绿（真通过 546 + 近似 47）；尾簇 6 期：workspace 17,445P/0F + renderer lib
202P/0F，6b 旧序钉双校准。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | uievents + pointerevents corpus 导入 + 基线 | ✅ 2026-10-03 |
| P2 | 鼠标事件序/坐标/click 组合语义修齐 | 🔄 核心已落地；残余 = mousemove-between（视口命中精度）、wheel 三案（scroll 源重放——wheel-basic/deadlock 可解，scrolling 需真滚动）、interface keyboard-click、uievents/mouse 尾簇（mouseover-at-removing ✅ 尾簇 7 全绿；mutation 驱动悬停重定向 + 修饰键态 ✅ 尾簇 8 全绿；image-map 命中 + 跨目标 click 组合 ✅ 尾簇 9 全绿；同 turn gBCR 强制同步布局——reappending 11F + removing_last_over 4F 根因；image-map img-resized 双案 = 查询视图缓存双计——尾簇 10；layerX/chorded buttons） |
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
- **M3 尾簇 6b+6c（2026-10-04，001375034 校准）**：mousedown→focus 默认动作链（迁移序 +
  relatedTarget + slice22 钩子接线）+ touch 接触失效收口（up 内变异 post-up
  结算 + 延迟跨界 + child-first 序）。1107P→1115P。
- **M3 尾簇 7（2026-10-04，1cbbeaaa8）**：R334 重插同 turn 查询可见性（by-selector 索引 +
  探测轮首 flush）+ variant 值大小写保真。1115P→1167P（总册 1955→1948——pd=/
  buttonType= 变体按真实配置计册）。
- **M3 尾簇 8（2026-10-04，6045a7462）**：修饰键全局态（`_zwModifiers` + 合成事件缺省
  携带）+ mutation 驱动瞬态悬停重结算（rAF 派发点同步 + 探测环几何，px/py 守卫）。
  1167P→1202P。
- **M3 尾簇 9（2026-10-05，ce7ca7f37）**：image-map 几何命中（runner 命中测试 JS 扩
  area shape/coords 面）+ 跨目标 click 组合实派 target 优先 + mutation 代际扩面
  （悬停元素属性/IMG image-map 属性/AREA 插入）+ settle 延迟结算 + 指针命令入口
  settle + 跨批 handle 插入序列化落地（共享队列卡死修复）。1218P（前值 1202P）。
- **M3 尾簇 10（2026-10-05，本轮）**：native MouseEvent 模板坐标 floor + page/offset
  派生（`init_floor_int` + is_number 门）。1218P→1570P（+352）。

## 下一步计划

1. **M2/M3 尾簇**（按 Throughput 排序，根因链见
   [evidence/2026-10-04-m3-tail7.md](evidence/2026-10-04-m3-tail7.md)/
   [evidence/2026-10-04-m3-tail9.md](evidence/2026-10-04-m3-tail9.md)/
   [evidence/2026-10-05-m3-tail10.md](evidence/2026-10-05-m3-tail10.md) 残余节）：
   a) **click_is_a_pointerevent 10F**——click/auxclick/contextmenu 应为 PointerEvent
   实例（PE spec），当前 dispatch 走泛型 Event（与 R108 激活事务 checked 翻转契约
   耦合——part06 `__zw_dispatch_event` click 分支注记，需专项设计）；
   b) **同 turn gBCR 强制同步布局**——`mouse_boundary_events_after_reappending_last_
   over_target` 11F + removing_last_over_element 4F + pointer 孪生：createElement 后
   同步 `getBoundingClientRect` 读 stale 布局（真浏览器 gBCR flush layout；需把
   drain+relayout 接进 gBCR 宿主回调——pipeline 句柄进 callbacks 层的设计题）；
   c) **click_during_parent_capture 14F / pointercapture_in_frame 18F**（iframe 捕获面）；
   d) **查询视图缓存双计**（img-resized 双案 2F——`with_query_view_doc` snapshot+全史
   重放烘焙双计）；e) wheel 源 scroll 重放（wheel-basic/deadlock）；f) focus 残余两案
   （iframe 跨文档焦点、keydown→focus activation——键盘域邻接，挂账候选）。
2. **M4 收口**：touch-action 解析/计算值核对（parsing 三案 0P 待查）、
   touch-events / pointerlock / IME / touch-action 交互面挂账定稿、DC-4 全绿门禁
   （make test + clippy + fmt + reftest 零回归 + product-smoke）。

**待用户决策清单**：（空——pointerlock 挂账重入 = 用户点名；declarative shadow
DOM 前置如需立项请点名）
