# M3 s2：位移流字段收口（FR-4，RFC §6.3）

**日期**: 2026-10-19
**切片**: RFC v1.2 §8.1 s2（§6.3）
**结论**: **movementX_Y_basic 实跑翻绿，执行子集 3/4 → 4/5（75.0% → 80.0%）**

## 行为变化

| 项 | 变化 | 依据 |
|---|---|---|
| FR-4 序列化层 | `DomEventDetail` 增 `movement_x`/`movement_y: i64`（缺省 0）+ 序列化 `movementX`/`movementY` 键；`script_pointer_move` 签名扩至 5 参（第 4 参 chordBtn 传 undefined 保槽位，位移落 5/6） | RFC §6.3 层 1 |
| FR-4 事件源层 | runner `pointer_move` op 在 `resolve_pointer_target` 后、更新 `last_pointer` 前算 delta（前后位置差，i64 截断——spec movement 为 long；首步无前位 → 0）；down/up op 同坐标更新不扰动 | RFC §6.3「计算职责在事件源」 |
| FR-4 shim 钩子层 | `__zw_pointer_move` 签名扩至 6 参；mousemove 派发 detail 透传 movementX/Y（part06 唯一 mousemove 派发点，流与 move op 严格 1:1——delta 伸缩成立） | RFC §6.3 层 2 |
| **实施期发现**：MouseEvent 构造键表 | part05 `_defineEventSubclass('MouseEvent', ...)` init 键表补 `movementX`/`movementY`（缺省 0）——**键表外的 init 键被静默丢弃**，此即 constructor.html M1 红案「movementX 缺省 undefined vs 0」的真根因；PointerEvent/WheelEvent 经原型链继承 → RFC v1.2 | 构造缺省 = spec `dom-mouseevent-movementx`（长整 0） |
| screenX 合成面 | mouse 分支显式 `screenX=clientX`/`screenY=clientY`（无宿主窗口几何时视口即 screen 基准——movementX_Y_basic 逐步断言依赖）；`new MouseEvent` 构造路径 screenX 缺省 0 不受影响（分支隔离，dom/events 断言面核对过） | RFC §6.3 假设落定（v1.2） |
| skip 窄化 | `pointerlock_case_skipped` 对 `movementX_Y_basic.html` 名字例外放行（movement-only 案，不取真锁——Actions 步走合成指针管线即满足） | 13 skip → 12 |

## 验证

- **通道实跑**（`make testharness-pointerlock`，pin 315976933）：
  - `movementX_Y_basic.html` **Pass**（原 skip）——逐步断言 `movementX == screenX 前后差` + 总和断言全过
  - 执行子集 **4/5 = 80.0%**（s1 3/4 = 75.0%）；skip 13 → 12
  - `constructor.html` mouse 扩展面红案**维持**，失败点 = line 21 `assert_equals(ev.eventPhase, Event.NONE)`（`Event.NONE` undefined——dom/events 域残留，M1 即挂账，不占本 goal 分母）；movement 断言（line 27–46：缺省 0 / 初始化跟随 10）已被构造键表修复支撑，因断言序不可达，由 movementX_Y_basic 实跑等价证明
- **桥测试**：新增 `test_pointer_move_movement_stream_s2`（mousemove 位移流透传 + 合成 screenX/Y=clientX/Y 两断言面）绿；`test_pointer_lock_api_r2939` 零回归绿
- **fmt**：`cargo fmt --check` 干净（script_gen.rs 一处 format! 换行）
- **clippy**：`zero-engine`/`zero-webview`/`zero-wpt-runner` lib+tests 零告警
- **make test**: 见本文件同日 Commit 2 正文（切片验证门另一半）

## 未决

- constructor.html line 21 eventPhase 红案维持 dom/events 残留（不占本 goal 分母）
- C 面 12 案维持 skip → s3 锁管线（§6.4–6.7）
- TBD-1/3 维持 RFC §10 原状
