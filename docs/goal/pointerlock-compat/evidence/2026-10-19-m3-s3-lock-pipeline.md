# M3 s3：锁管线就绪（RFC §6.4–6.7，Commit 3 = 步骤 4+5+6）

**日期**: 2026-10-19
**切片**: RFC v1.3 §8.1 s3 前半（管线就绪，runner 可驱动；零产品行为变化）
**结论**: **webview 锁镜像 + host 桥 + IPC 双向就绪；e2e 桥测试一次全绿**
（C 面执行与桌面激活 = Commit 4，步骤 7+8）

## 行为变化

| 项 | 变化 | 依据 |
|---|---|---|
| webview 锁镜像 | 新模块 `crates/webview/src/webview/pointer_lock.rs`：`PointerLockState{target,last_pos,unadjusted,pending_report}`（Arc 共享形同 selector_handle_map）；`apply_locked_pointer_move`（未锁定 no-op；锁定 delta 镜像算 + 直接派锁目标 mousemove，不 hit test/不合成跨界序）+ `apply_locked_pointer_button`（Down/Up 改道 mousedown/mouseup；click 组合不在本入口——FIXME C 面实跑钉）+ `pointer_lock_target` 查询面 | RFC §6.4 |
| 导航清理 | 导航重置点四连（与 drag_input 同点）清镜像 + 挂 false 跃迁报告（锁不跨 document） | RFC §6.4 |
| host 桥 | `__zw_pointer_lock_state_changed`（payload JSON `{locked,selector,handle,unadjustedMovement}`，typeof 守卫同 deny 开关风格）：shim 四处通知（grant 含 option 重锁/deny/exit/移除解锁）；webview 侧静态注册（identity bridge 三武装点并列：execute_script_with_dom/run_page_scripts_impl/dispatch_dom_event_inner——浏览器与 runner 路径无分叉）；跃迁判定 host 侧（deny 冗余/已锁重授静默） | RFC §6.5 |
| IPC 双向 | `PointerLockInput{phase:Move/Down/Up,x,y}`（浏览器→渲染，ImeEvent 同款无回执）→ renderer 臂 `apply_locked_pointer_*`；`PointerLockChanged{locked}`（渲染→浏览器）→ process_backend 排队 + `take_pointer_lock_changes`（s4 消费；Commit 3 仅排队） | RFC §6.5/§8.2 步骤 6 |
| 渲染排空点 | `run_frame_transaction` 收尾排空镜像跃迁报告 → `send_regular(PointerLockChanged)`（帧事务内页面脚本 grant/exit 均捕获） | renderer 侧唯一突变源收口 |

## 实施期修正（RFC v1.3，§12）

1. **§6.5「事件派发改道」不实施**：shim change/error 保持 s1 `_firePointerLockEvent`
   同步派发（corpus 已验证 N1 面）；host 桥只做状态镜像（双馈管线本义）。改道经
   host→page 脚本回环会把同步派发变异步任务，破坏 corpus 已钉时序
2. **§6.5 payload `selector` 发纯 `sel`**：非 `_elKey` 复合键（`@handle`）——host
   侧 querySelector 解析面，与 drag hit selector 同约束
3. **§6.5 `PointerLockChanged{locked, tab_id}` → `{locked}`**：renderer 无 tab
   身份，通道关联同 FocusOwnerChanged 先例
4. **§8.2 步骤 5 注册点在 webview.rs**（非 js_dom_bridge.rs）：Arc 共享镜像需
   webview 侧状态，静态注册函数与 identity bridge 同形

## 验证

- **webview 模块单测 6 绿**：advance_move 边界（未锁 None/首 move 0/前后差/重授
  重置）、解锁报告一次性、payload 跃迁判定（grant/exit/重授/deny 冗余/三类坏载荷）
- **e2e 桥测试一次全绿**（`lock_mirror_e2e_via_host_bridge`，真实 WebView + shim）：
  grant → host 桥通知 → 镜像可查 + true 报告；锁定改道 mousemove 落锁目标
  （movement 0,0 → 3,2 镜像 delta + screenX=clientX）；exit → 镜像清 + false 报告；
  改道复归 no-op
- **协议 roundtrip**：`test_pointer_lock_messages_roundtrip`（两消息 serialize/
  deserialize 往返）绿
- **零回归**：`test_pointer_lock_api_r2939`（s1 语义组）4 绿、fullscreen 桥 3 绿、
  make test 全绿（0 FAILED）、clippy 四 crate lib+tests 0 告警、fmt 干净

## 未决

- C 面 12 案执行 + runner 锁定路由（§6.7）→ Commit 4（步骤 7+8）
- 桌面转发门/Esc/OS 光标（§6.6）→ s4/Commit 4
- TBD-1（shadow 资产）/TBD-3（Actions.scroll/侧键）维持 RFC §10
