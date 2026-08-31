# R150 — 进程内 gBCR + MouseEvent offsetX/offsetY 派发期计算 + Event timeStamp 5µs 量化

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**驱动用例**: `dom/events/mouse-event-retarget.html`（1 subtest）+ `dom/events/Event-timestamp-safe-resolution.html`（1 subtest）+ `dom/events/Event-timestamp-high-resolution.https.html`（1 subtest）

## 根因与修复（三件）

### ① 进程内 `__zw_getBoundingClientRect`（callbacks.rs 新回调）

进程内 webview（testharness-dom runner 路径）此前无 RectBridge（多进程 renderer
js_worker 专有）→ `el.getBoundingClientRect()` 恒零 rect → MouseEvent offsetX
派发期计算（本切片②）无从取 target 几何。

修复：

- `register_dom_callbacks` 签名加 `rect_snapshot_opt: Option<&LayoutRectSnapshot>`；
  进程内 webview 传 `Some(&self.layout_rect_snapshot)`（render / apply 后 /
  native repaint 后 `refresh_layout_rect_snapshot` 刷新——无缓存布局时清空，
  快照与布局一致性优先）。
- 回调内 selector→NodeId：LIVE_QUERY_DOC live doc 优先（进程内常驻发布），
  miss 回落查询快照 re-parse（与 `__zw_query_match` 同源逻辑）。rect 查
  snapshot 返 `"x,y,w,h"`；miss → 空串（shim 回落零 rect）。
- **多进程零回归**：renderer js_worker 传 `None`（空快照恒空串）；其自身
  `RectBridge::register` 在其后注册同名回调，V8 `execute` 期按注册顺序
  `global.set` = **last-wins**，RectBridge handler 生效——多进程路径行为不变。
- 21 个测试文件机械更新调用签名（+`None` 参数）；附带
  `pipeline.layout_rect_for_selector` + `hit_test::fill_rect_from_layout_box_pub`
  公开入口（当前无调用方，R151 评估去留）。

### ② MouseEvent offsetX/offsetY 派发期计算（part03 `_dispatchWithBubble`）

spec CSSOM View §dom-mouseevent-offsetx：offset = client 坐标 − target padding
边缘（实现近似为 gBCR 左/上）。真实浏览器 MouseEvent initDict **无 offsetX
字段**——恒为派发期计算；shim 旧实现恒 0。WPT mouse-event-retarget：clientX 50
派发到 body margin 8px 下的 target，offsetX 期望 42。

修复：`_dispatchWithBubble` 设 target 后，若 event 有 clientX/Y 数值、未显式
init offset（`_zwOffsetInit` 印章，part05 `_defineEventSubclass` 两工厂形态
盖章）、未计算过（`_zwOffsetComputed` 防重算/冒泡重复计算）→ 读 target
gBCR 计算 offsetX/offsetY。target 无 gBCR（detached/文档节点）保持现值。

### ③ Event timeStamp 5µs 量化（定时侧信道缓解）

WPT Event-timestamp-safe-resolution：千样本相邻构造事件差值的 GCD 须 ≥ 5µs
（0.005ms）——真实浏览器对 Event timeStamp 施加 coarse 粒度防 timing attack；
高分辨率原值（µs 级）使 GCD 跌到 1。

修复：shim `_makeEvent`（JS `Math.ceil(t * 200) / 200`）与 native
`event.rs perf_now_ms`（Rust 同款 ceil）**双路径同语义**。ceil 而非 round：
任意正 elapsed 量化后恒 ≥ 0.005ms 且 > 0（R22 断言 `timeStamp > 0` 不回归）。
high-resolution 用例（FocusEvent vs performance.now 单调性）不受影响。

附带 **GamepadEvent 构造器**（part05 `_defineEventSubclass`，spec Gamepad
§gamepadevent：`gamepad` 属性默认 null / init 透传）——解锁
Event-timestamp-high-resolution.https（旧 `new GamepadEvent` ReferenceError）。

## A/B 验证

| 项 | 结果 |
|----|------|
| mouse-event-retarget | **1P 双路径**（polyfill + ZW_NATIVE_DOM=1） |
| Event-timestamp-safe-resolution / high-resolution(.https) | **3P 双路径** |
| events 全量 | **444P / 12F / 9T 双路径完全一致**（fail+timeout 集合 diff 为空；vs R149 441P/15F/9T = +3P/−3F，消失 3 件即上列驱动用例） |
| 单测 | r150 三件：offset 计算值（50−8=42/30−8=22）+ 显式 init 保持构造值；timeStamp 200 样本单调 + 差值 ×200 恒整数 + 首值 > 0；GamepadEvent instanceof 链 + gamepad 默认 null/init 透传 |
| `make test` | 66 套件全绿（exit 0） |
| fmt / clippy | 双矩阵（v8 全 workspace + quickjs QUICKJS_CLIPPY_CRATES）零警告 |

## 轮间恢复记录（429 打断）

上一轮 session 在 10 次 API 重试（429 rate-limit）后中断——R150 实现已完成但
未验证未提交（工作树 32 文件 dirty）。本轮恢复流程：`git stash push -u` →
`git pull --rebase`（12 commit：并行流 layout intrinsic 族 + service-worker +
part05 MessagePort 改动）→ `git stash pop` 零冲突 → 核对 diff 归属（21 测试
文件纯机械签名更新）→ 验证 → 补单测 → land（commit `04bde0bbd`）。
