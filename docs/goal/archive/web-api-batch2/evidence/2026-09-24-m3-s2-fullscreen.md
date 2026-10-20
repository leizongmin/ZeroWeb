# M3 切片 2 — 节点移除全屏联动 + M3 域内收口（fullscreen 86.0%→88.0%）

**日期**: 2026-09-24
**套件**: `make testharness-fullscreen`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-24-m3-s2-fullscreen.json](2026-09-24-m3-s2-fullscreen.json)
**前值**: [2026-09-24-m3-s1-fullscreen.md](2026-09-24-m3-s1-fullscreen.md)（129/150 = 86.0%）

## 结果

| 指标 | M3-s1 | M3-s2 | Δ |
|---|---|---|---|
| subtests Pass | 129/150 = 86.0% | **132/150 = 88.0%** | +2.0pp（+3 纯增益，pass set 零丢失） |
| 全绿用例 | 34/55 | **37/55** | +3 |

新增全绿：model/remove-single、model/remove-parent、model/remove-first。
remove-child（无涉移除不影响全屏）保持绿。

## 落地面

**节点移除 → 全屏退出联动**（engine part06 `_fsOnNodeRemoved` + part04 remove() 钩子）：
spec top-layer 移除语义——移除子树包含当前全屏元素（自身直接命中，或沿 parentNode
上行身份比对命中祖先形态；parent 链读移除前视图，host mutation 异步 apply 不影响）→
fullscreenElement **同步**置 null（三案「immediately after removal」断言）+ 异步 step 派
fullscreenchange 且 **target=document**（移除子树按定义 detached → `forceDoc` 强制
document 派发 + 显式设 `ev.target = document`——`_dispatchToListeners` 第四参是
currentTarget 不写 target，三案第三事件 `target === document` 断言根因；handle 元素
gBCR 探针对已移除节点残留 stale rect → isConnected 误判，forceDoc 绕开）。

单测 `test_fullscreen_removal_exit_wab2m3s2`（自身移除/祖先移除/无涉移除三形态）。

## M3 域内收口判定（DC-3 对照）

- **requestFullscreen/exitFullscreen（含 options）/fullscreenElement/fullscreenEnabled +
  fullscreenchange/error 事件**：88.0%（132/150，全绿案 37/55）；三轮逐簇修齐
  （M3-s1 异步状态机 + 激活面 + PermissionStatus；M3-s2 removal 联动）。余簇全部甄别
  且归属明确（见下），engine 域内无未定位缺口。
- **全屏状态 viewport 语义联动验证（消费 ④ viewport 桥）**：corpus 可观察面三案全绿——
  document-exit-fullscreen-timing（resize 先于 fullscreenchange 的事件序 +
  fullscreenElement 事件前已变）、element-request-fullscreen-timing（change 时序 +
  rAF 次序）、element-request-fullscreen-screen-size「element fullscreen」（screen 尺寸
  不变断言）。**真窗口 OS 级全屏**（host-runtime 平台面、真 viewport resize 联动）与
  平台剪贴板后端同款挂账 M4 定稿。

## 余簇记账（M3 终态归属）

- **栈模型缺口 ×3**：remove-last（移除栈顶需回退前一栈元素再异步全退——需完整
  fullscreen element stack 模型）、remove-first-sibling / remove-last-sibling（断言
  `matches(':fullscreen')` → rendering-compat 流域）。移动出全屏 move-fullscreen-element
  同因（matches 断言）归 rendering-compat。
- **跨域记账（不变）**：svg-rect/svg-svg（SVGElement instanceof，js-dom）；shadowroot-
  fullscreen-element（shadowRoot.fullscreenElement per-root 状态面，Timeout）；move-to-
  inactive-document（iframe 文档管道，Timeout）；cross-origin.sub / navigate-iframe.sub
  （iframe 管道，Timeout）；frameset-crash（crashtest 挂起甄别保持）；element-request-
  fullscreen 基本案（window.event 派发后持久性，runner infra）；display-contents（UA
  display:block 覆盖 + `::backdrop`，rendering-compat）；fullscreen-reordering
  （`/common/top-layer.js` fetch，runner infra）；screen-size tab-capture（getDisplayMedia
  缺）；nested-shadow-dom（shadowRoot.getElementById，js-dom）；nested（testdriver
  selector 面）；allowfullscreen iframe（跨域）。

## 附带影响

- clipboard-apis 未复跑（本轮零 clipboard 域改动；M3-s1 复跑 61/73 基线有效）。
- 质量门禁：cargo fmt 干净 + clippy `-D warnings` 全过 + make test 见提交说明。
