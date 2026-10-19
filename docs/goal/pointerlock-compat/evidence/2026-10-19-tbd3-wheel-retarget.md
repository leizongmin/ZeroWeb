# TBD-3 收敛——锁定态 wheel retarget + 滚动链默认动作（2026-10-19，M3 收口后切片）

M3 Commit 5 收口后遗留移交面推进：wheel 案 ×2 翻绿。实施期复核发现 TBD-3
记账归因过时——`Actions.scroll` 并非未实现（uievents 尾簇 40d9122ec 已落
`Actions.prototype.scroll` → `wheel_scroll` 命令 → `__zw_wheel_scroll`），真
缺口是**锁定态 wheel 语义**：s3/Commit 4 的锁定改道只覆盖 pointer/mouse 位移
与按键，未覆盖 wheel 源。本切片补齐后通道 **19/22（86.4%）→ 21/22（95.5%），
文件 15/18 → 16/18**；唯一红案 = constructor 跨域 residual（dom/events 域，
不占本 goal 分母）。

## 1. 根因链

红案实跑（`make testharness-pointerlock FILTER=wheel`，Commit 5 代码态）：
两案 `errors.length ≥ 1`，首条均 `Wheel event should be dispatched`
（wheelCount=0）——锁目标 #target 上的 wheel 监听未触发。读用例
（pointerlock_wheel_event_outside_target.html）：点击 #toggle 获锁（光标留在
toggle 上，界外），随后 `Actions().scroll(0, 0, 0, 100, {origin: toggle})`——
runner `wheel_scroll` 命中 #toggle 派发，#toggle 无监听 → 计数 0；且滚动默认
动作不消费 → `target.scrollTop` 恒 0。

按 Pointer Lock spec（https://w3c.github.io/pointerlock/#user-interface），
锁定中 wheel 事件 **target 为锁元素**（与光标位置无关），且「the lock target
is the target from which scrolling should chain」——runner 常规命中管线天然
不满足，语义缺失在 shim 权威面而非 runner 合成面。

## 2. 修复面（2 文件）

1. **part06.js `__zw_wheel_scroll`**（RFC v1.6）：
   - 锁定分支：`_plKey` 置位时 retarget `_plTarget` **原对象**直派（实例域/
     trap 域同形，learning #32 identity 保持；回落 `_makeProxy(_plSel,_plHandle)`
     兜底）——不 hit test 光标下元素。未锁定路径逐字节不变（uievents wheel 族
     零回归面）。
   - 滚动链默认动作：锁定且未 `preventDefault` → `_zwWheelScrollChain`：
     自锁目标沿 `parentNode` 上行找首个 delta 轴 overflow scrollable
     （'scroll'/'auto'；computed `overflow-y/x` cascade 优先，宿主 miss 回落
     inline style，shorthand 单值展开双轴——CSS Overflow 3），经 R3047 setter
     滚动（同步派 'scroll' 事件）。不钳制滚动范围：shim 布局 rect 无溢出展开量
     （scrollHeight≈clientHeight 近似），与 setter 负值 clamp 同口径。
2. **webview/pointer_lock.rs**：e2e 钉 `locked_wheel_retarget_scroll_chain_e2e`
   （四断言面：锁定 retarget 落锁目标 / 滚动链滚首个 scrollable 且非目标不滚 /
   preventDefault 取消滚动 / 解锁回落常规命中且不消费滚动）。

runner（testharness.rs `wheel_scroll` 命令）与 `script_wheel_scroll` **零改动**
——语义权威在 shim，未来桌面 wheel 派发底座接入（挂账面）自动获得 retarget。

## 3. 实跑验证

- `make testharness-pointerlock`：**21/22 子测试 Pass（95.5%），文件 16/18**。
  wheel 两案翻绿（含 `target.scrollTop=100` 滚动断言与 preventDefault 抑制
  断言）；红案余 constructor（Event.NONE 跨域 residual，dom/events 域，
  uievents 50 挂账同源）。
- uievents wheel 族回归（`make testharness-uievents FILTER=wheel`）：
  wheel-basic / wheel-scrolling / wheel-multiple-sources-deadlock **3/3 Pass**
  ——未锁定路径零回归。
- webview 单测（test-guard 包裹）：pointer_lock 模块 **8/8**（含新 e2e 钉）。
- `make test`：全腿绿（含 clippy `-D warnings` 腿）。
- `make reftest`：712/712 = 100.0%，零回归。

## 4. 残留挂账（更新后移交面）

- constructor.html 1 红——`Event.NONE` 等 Event 接口 phase 常量，dom/events 域
  （uievents 50 residual），不占本 goal 分母。
- 桌面锁定 wheel 转发——renderer 无 DOM wheel 派发底座（v1.4 挂账，与本切片
  同域不同面；接入时语义面已就绪）。
- TBD-4 OS 光标隐藏/显示——平台窗口面远期池。
- 桌面端 GUI 手验（真窗口锁光标观感）——WSL2 环境受限，挂账桌面域。
- 实例域锁的 `__zw_pointer_lock_state_changed` 镜像 payload `selector:null`
  ——host 侧转发按 selector 命中收不到锁定输入流；WPT 面不触达（shim 侧
  retarget 不经 host 镜像，本切片不受此限制影响）。
