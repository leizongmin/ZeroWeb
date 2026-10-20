# M3 s1：shim 语义组收口（FR-1/2/3 + FR-10 shim 侧）

**日期**: 2026-10-19
**切片**: RFC v1.1 §8.1 s1（§6.1+§6.2）；M3 点名（用户 2026-10-19）后首个切片
**结论**: **B 面 2 案实跑翻绿，通道 1/4 → 3/4（25.0% → 75.0% 中间口径）**；fullscreen 桥测试零回归

## 行为变化

| 项 | 变化 | 依据 |
|---|---|---|
| FR-1 激活门 | 无瞬态激活 → `requestPointerLock` reject **NotAllowedError**（DOMException）+ 派 pointerlockerror；**不消费激活** | WPT requires-gesture / without_gesture / unadjustedMovement_update |
| deny 路径 | host `__zw_pointer_lock_enabled` 返 '0'：TypeError → **NotAllowedError**（RFC §6.1 permission-denied 语义） | RFC v1.1 §6.1 |
| FR-2 事件面 | 新 `_firePointerLockEvent`：target=document、bubbles=false、cancelable=false、isTrusted=true、Event 真原型；旧 `_fireDocEvent`（本次修改后零调用方）删除 | WPT api_behavior 断言面 |
| FR-3 options | `requestPointerLock(options)` 字典校验（非对象 → TypeError）；`unadjustedMovement` 有效布尔入锁态；不实现 NotSupportedError | WPT unadjustedMovement 两案 |
| N2 no-op 细化 | 同元素**且**有效 unadjustedMovement 相同 → no-op；异选项重授派 change | WPT unadjustedMovement_update |
| FR-6 移除解锁 | `_plOnNodeRemoved`（el.remove() 挂钩，祖先链比对）：同步清 `_pl*` + 异步（setTimeout 1ms task）派 change | _fsOnNodeRemoved 先例 |
| FR-10 导航清理 | `__zw_reset_form_state` 增清 `_pl*` + `_fs*`（同族泄漏，用户确认随修）+ `_zwTransientActive/_zwActiveEver` | 生产触发点 js_worker.rs:1725 / tab_js_worker.rs:549 |
| §6.2 激活挂接 | `__zw_dispatch_event` R312 块增输入族白名单（pointerdown/mousedown/pointerup/keydown/touchend → 置瞬态激活），真实桌面输入可过激活门 | HTML spec transient activation 触发集 |

## 验证

- **通道实跑**（`make testharness-pointerlock`，pin 315976933，13 案 skip 维持）：
  - `pointerlock-requires-gesture.html` **Pass**（原 F）
  - `pointerlock_without_gesture.html` **Pass**（原 F）
  - `constructor.html` pointerlockerror 字典面子案 Pass（维持）；mouse 扩展面子案 Fail（movementX 缺省 `undefined vs 0`——**FR-4/s2 范围，M1 基线即红**，s1 未触碰该面）
  - 执行子集 **3/4 = 75.0%**（M1 基线 1/4）
- **桥测试**：`test_pointer_lock_api_r2939` 重写扩展（门/事件面/options/移除/导航清理/deny 全语义，1 案 8 组断言）绿；`test_fullscreen_*` 3 案零回归绿
- **fmt**: `cargo fmt -p zero-engine -- --check` 干净
- **make test**: 见本文件同日 commit 正文（切片验证门另一半）

## 实施期修正（RFC v1.1，§12 修订历史）

1. activation 门「消费」→「仅检查不消费」：`unadjustedMovement_update` 案一次激活连续两次请求均须过，spec request steps 无 consume 步
2. `_plOpts` 对象深相等 → `_plUnadjusted` 有效布尔相等：语料只变异单成员，最小实现

## 未决

- constructor.html mouse 扩展面红案 → s2（FR-4 位移流字段）
- C 面 13 案维持 skip → s3 锁管线（§6.4–6.7）
- TBD-1/2/3 维持 RFC §10 原状
