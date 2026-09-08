# M1-M3 里程碑归档（2026-09-07）

> 归档区域：只追加不修改。M1/M2/M3 的详细过程与证据（只读快照）——
> 运行时状态与后续决策见 `master.md`（控制面板）与 `evidence/`（验证证据）。

## M1 — 基线建立 + 分发层骨架（2026-09-07）

### 切片 1 — 上游可执行用例导入

- defer 解除：testdriver Actions 键盘链落地（keyboard-default-actions M1 切片 2
  共享基建）+ send_keys 滚动键事件对（uE00E-uE015 → keydown+keyup 对，WebDriver
  key 名映射）
- css-scroll-snap/input 三案（keyboard.html / paged.html / scroll-padding-paged.html）
  导入 keyboard 套件，全部可执行
- evidence：`evidence/2026-09-07-m1-keyboard-scroll-baseline.md`

### 切片 2 — 键盘滚动分发层骨架

- 底座 R3254-M9 既有（keydown 回执驱动）+ `scroll_delta_for_key` 映射单测固化：
  Space/PageUp/PageDown/Arrow/None 键位语义断言（含 ±Shift 修饰变体）

### 切片 3 — 失败聚类（2026-09-07 与 keyboard-default-actions 同轮收口）

- paged 1P/1F/1Timeout + keyboard 1 Timeout + scroll-padding 1 Timeout
- **Timeout 根因定位**：waitForScrollEndFallback 依赖 scroll 事件 + rAF 帧循环
  静默窗——runner 无真帧循环（`__ZW_RAF_FRAME_DRIVEN` OFF 路径 rAF 同步 stub），
  scroll 未发生时 promise 永挂 → testharness completion 不触发
- paged 1F = snap 容器 Space 键滚动目标判定（P3 跨域项同一根因）

## M2 — 全键位 + 滚动目标判定（2026-09-07）

### 切片 1 — Ctrl+Home/End 修饰变体（commit fad120776）

- dispatch_ctrl_scroll_key 回执链 + e2e 断言；browser 413 全绿
- 全键位面：Space/±Shift/PageUp/PageDown/Arrows/Home/End/Ctrl+Home/End
  （回执链 preventDefault 阻断）
- **跨域 defer**：焦点→容器→根滚动目标判定依赖 renderer S3 layout 几何
  （R3298 S2 注记协调点）；文档级/根级滚动已闭合

## M3 — scrollIntoView + 事件 + snap 交互收尾（2026-09-07）

- R3060 scrollIntoView 选项面摸底：block start/end/center 公式断言 +
  smooth→instant 简化记录；scrollIntoViewIfNeeded R3075
- scroll 事件语义断言资产两件：窗口滚动七断言（事件 cancelable=false + 轴独立
  + round-trip）+ scrollIntoView 五断言（标明本地）
- snap 键盘交互断言需 runner 真渲染 viewport（跨域协调项，M1 记录延续）

## 最终验证状态（2026-09-07）

- 本地资产：分发映射单测 + 窗口滚动七断言 + scrollIntoView 五断言（均标明本地）
- 上游面：snap/input 三案可执行（断言 F/Timeout 均有跨域根因记录）
- 全 workspace 测试基线全绿（上游 18,968 基线 @ 2026-09-07 并行流确认）
- **跨域协调项汇总**：① 焦点→容器链（renderer S3 layout 几何，R3298 协调点）；
  ② snap 断言真渲染 viewport（渲染流域）；③ runner 真帧循环 scrollend 语义
  （runner 基建域）

---

## 收口注记（2026-09-07 追加）

- M1/M2/M3 本流可闭环面全部落地（分发映射 + Ctrl 变体回执链 + runner 侧滚动默认动作
  `__zw_scroll_key_default` + 帧驱动 rAF opt-in + scrollIntoView 选项面断言 + snap 键盘
  交互 keyboard.html 8/8 可完成）。
- 残余（焦点→容器链、snap 位置精确断言、scrollIntoView 元素级滚动）均为 renderer S3
  布局几何同一跨域协调点（R3298 注记）——不阻塞本 goal 收口判定。
- keyboard 套件共享面终态：18P/7F/2T（兄弟 goal keyboard-default-actions 的
  implicit-submission 残余切片已在本轮收口归档）。
