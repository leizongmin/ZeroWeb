# M1-M3 里程碑归档（2026-09-07）

> 归档区域：只追加不修改。M1/M2/M3 的详细过程与证据（只读快照）——
> 运行时状态与后续决策见 `master.md`（控制面板）与 `evidence/`（验证证据）。

## M1 — WPT 键盘交互基线建立 + 分发层骨架（2026-09-07）

### 切片 1 — 用例导入 + 基线（commit aad135b52 前置）

- 9 用例导入（fetch-keyboard-subset.sh + `testharness-keyboard` 子命令 +
  Makefile 入口 `make testharness-keyboard`）：uievents/keyboard 5 +
  implicit-submission + css-scroll-snap/input 三案（兄弟 goal 共享基建）
- 基线：**1P/4F** → ENTER 映射修复（uE007 → Submit 动作）→ 6P/12F，
  Timeout/Unhandled rejection 清零，全案可执行
- evidence：`evidence/2026-09-07-m1-keyboard-baseline.md`

### 切片 2 — 失败聚类修复：testdriver Actions 键盘链（commit aad135b52）

- keydown/keyup 事件派发层：send_keys 路径非打印键 → cancelable KeyboardEvent
  事件对（'ok'/'prevented' 判定）→ keydown 未取消接默认动作 + keypress 派发
- preventDefault 抑制编辑事件与 value 变更（keydown-input-events 2P/0F）
- KeyboardEvent.composed 缺省 true（UI Events spec）；send_keys WebDriver 键
  扩展（滚动/导航/修饰键映射）

### 切片 3 — 分发表扩展：修饰键位透传（commit 1c279532e）

- DomEventDetail（script_gen.rs）新增 shift_key/ctrl_key/alt_key/meta_key 四位
  → detail JSON 透传 → `__zw_dispatch_event` KeyboardEvent init dict
- runner send_keys 修饰键（uE008/uE009/uE00A/uE03D）→ 对应位 true 的
  keydown/keyup 事件对
- modifier-keys.html 4F 全灭（event.shiftKey === (key === 'Shift') 断言族）
- keyboard 套件 6P/12F → **10P/4F**
- 单测 `test_modifier_key_dispatch_r3254_k2_slice3` 五组断言
- **残余 4F 精确根因（探针定位）**：implicit-submission 3F = js-dom 融合视图
  insertAdjacentHTML 双计 + childNodes stale（跨域协调点）；paged 1F = 真滚动
  管线缺失（keyboard-page-scrolling 共享面）

## M2 — 表单键与激活（2026-09-07）

### 切片 1 — 空格激活（commit f6eaed4d5）

- 空格对 button-ish 目标（BUTTON / input type=button|submit|reset）递归
  Activate（click 合成全管线复用——状态构建/click 派发/disabled 守卫/effects）
- Enter→Submit 臂已由 M1 uE007 路由覆盖；两键 keydown/keyup 时序差异
  （Space=keyup 触发、Enter=keydown 触发）在 runner 单发通道下语义合并，
  Actions 键盘链可细分（defer 记录）

### 切片 2 — Esc dialog cancel/close（commit bcd59d7ed）

- shim `__zw_esc_dialog_cancel`：模态 dialog 优先（_zwDialogModal 印记），其次
  文档序首个 open dialog；派 cancelable 'cancel'（preventDefault 阻断关闭）→
  未取消 _zwDialogClose（reason=cancel + 'close' 事件）
- top-level 注册（shim 装载即生效）；宿主 keydown Esc 默认动作阶段调用
- 单测 `test_esc_dialog_cancel_r3254_k3` 三组断言
- select 展开态收起——headless 无展开态 UI（goal Support Envelope 明示 JS 面
  验收），defer 记录

## M3 — select 导航 + radio/checkbox + 事件序收尾（2026-09-07）

### 切片 1 — select 方向键导航（commit 051e594df）

- shim `__zw_select_key_action`：焦点 SELECT 上 ArrowDown/Up/Home/End → 选项
  移动（跳 disabled、clamp 不回绕、value 编程选中）+ input + change 事件序
- 单测 `test_select_key_navigation_r3254_k5`

### 切片 2 — radio 方向键组内移动（commit 627d3ff20）

- 同 name 组（文档序）方向键移动：ArrowDown/Right=下一 enabled、ArrowUp/Left=
  上一 enabled（Chromium 不回绕——clamp）；checked 属性迁移 + input + change
- 单测 `test_radio_arrow_navigation_r3254_k5`
- 空格切换 checkbox = K4 Activate 管线 checkbox 臂（既有）

### 事件序收尾

- keydown-input-events 2P/0F（keydown→beforeinput→input 序 + preventDefault 抑制）
- keypress 派发（composed 面 3P/0F）
- 修饰键位透传（M1 切片 3）+ getModifierState 联动
- type-ahead 多字符缓冲 defer（headless 验收面限制）

## 最终验证状态（2026-09-07）

- WPT keyboard 面：**10P/4F**（9 用例全可执行；残余 4F 均跨域根因——见
  evidence 切片 3 段）
- engine 2631 全绿；clippy -D warnings 零警告
- 分发表现有臂：Tab / printable（IME 合成跳过 R3254-L5）/ Backspace / Enter
  （textarea 换行 vs Submit）/ Escape（dialog）/ SELECT+radio 方向键 / 修饰键
  事件对
