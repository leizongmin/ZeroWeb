# Pointer Lock 兼容 — requestPointerLock / 指针捕获语义（门控 goal）

> **整体归档（模式 A，2026-10-20）**：按原目标约定范围收口；控制面与证据随入口归档，原有排除项、残差和重入条件保留。对应 Rally 启动脚本已删除。

**版本**: v2.0
**日期**: 2026-10-19
**状态**: **M3 完成 + constructor residual 收敛（2026-10-19，收口后切片 2）·
执行面 22/22 子测试 Pass（100%），文件 18/18 全绿·通道零红**——M2 RFC v1.7
（已确认，实施期修正见其 §12）。
RFC 见 [m2-pointer-capture-pipeline-spec-rfc.md](pointerlock-compat/m2-pointer-capture-pipeline-spec-rfc.md)；
M1 甄别报告见
[evidence/2026-10-18-m1-baseline.md](pointerlock-compat/evidence/2026-10-18-m1-baseline.md)
**执行模式**: WPT 驱动（上游 pointerlock corpus 为验收标尺）
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——交互面）

> **▶ 拆分动机（2026-10-08 html5test 缺口立项，用户「立项」）**：uievents goal
> （编号 50，已收口）挂账面之一；canvas 游戏/3D 场景的基础 API。corpus 体量小，
> M1 盘点成本极低，先立项占位。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：element.requestPointerLock/
> document.exitPointerLock/pointerlockchange/pointerlockerror 面存在性未盘点；
> 指针捕获本质依赖 host 光标管线（隐藏系统光标 + 相对位移流）——host-runtime
> 面，缺即门控。

## 当前状态（2026-10-19 constructor residual 收敛，通道零红）

- **constructor residual 收敛完成（2026-10-19，收口后切片 2）：最后一红
  `Event.NONE` 关闭，通道 22/22（100%）、文件 18/18 首次全绿**。实施期修正
  修复面定位——缺装面在 native `dom_bindings/event.rs`（页面 realm 的
  `globalThis.Event` 被 `register_ctor` 原生构造器覆盖，shim part05 R23 常量
  生产页面不生效），非 shim：`attach_event_constants` 补装 NONE=0/
  CAPTURING_PHASE=1/AT_TARGET=2/BUBBLING_PHASE=3（接口对象静态 +
  Event.prototype，WebIDL es-constants）+ dom_bindings 钉测试。附带收益：
  dom 通道 Event 面 A/B 实跑 **+21 subtest 翻绿、零新增红**（251→230 fail，
  Event-constants/bubbles/dispatch 序/EventListenerOptions-capture 族——均以
  eventPhase 常量断言）；uievents wheel 族 3/3 零回归；make test 全绿 +
  make reftest 712/712。**账本校正**：Commit 4 起文件口径少记 1（wheel×2 同
  文件按两文件计）——Commit 5 实为 16/18、TBD-3 实为 17/18，历史记录保留
  以证据 §4 为准。见
  [constructor 证据](pointerlock-compat/evidence/2026-10-19-constructor-residual-event-constants.md)。
  **下一步**：goal 通道零红，无未决实现项；挂账同下（桌面 wheel 转发随
  renderer wheel 派发底座另启）。
- **TBD-3 收敛完成（2026-10-19，M3 收口后切片）：锁定态 wheel retarget +
  滚动链默认动作，wheel 案 ×2 翻绿**。实施期复核发现 TBD-3 记账归因过时——
  `Actions.scroll` 自 uievents 尾簇 35 已实现（`wheel_scroll` 命令 →
  `__zw_wheel_scroll`），真缺口是锁定改道未覆盖 wheel 源。落点 shim
  `__zw_wheel_scroll` 锁定分支（`_plTarget` 原对象直派 + 未取消沿 overflow
  scrollable 链滚动）+ webview e2e 钉；runner/`script_wheel_scroll` 零改动
  （语义权威在 shim，桌面 wheel 底座接入即自动获得 retarget）。通道
  **21/22（95.5%），文件 16/18**；uievents wheel 族 3/3 零回归；make test
  全绿 + make reftest 712/712。RFC → v1.6。见
  [TBD-3 证据](pointerlock-compat/evidence/2026-10-19-tbd3-wheel-retarget.md)。
  **下一步**：goal 自身无未决实现项（残留挂账见下）；桌面 wheel 转发随
  renderer wheel 派发底座另启。
- **M3 收口完成（2026-10-19，Commit 5）：FR-7 双域 retarget + corpus 总账**。
  TBD-1 已决（shadow-dom.js 单文件补拉）；FR-7 实施期发现克隆树为 R307
  **实例域**（内部节点无 `__zwHandle`，attachShadow 走原型轻量路径）——初始
  handle 域 retarget 不可达，重写为域无关 composed 上行（`_plTarget` 原对象 +
  鸭面判界 + 对象身份视图表）+ `Element.prototype.requestPointerLock` 轻量回落 +
  `ShadowRoot.prototype.pointerLockElement` accessor；`pointerlock_shadow` 全案
  翻绿。通道终账 **19/22（86.4%），文件 15/18**；红案 3 记账（constructor
  Event.NONE 跨域 residual、wheel×2 TBD-3）。门禁 make test 全绿 + make reftest
  712/712 零回归。实施期修正 → RFC v1.5。见
  [Commit 5 证据](pointerlock-compat/evidence/2026-10-19-m3-commit5-s4-close.md)。
- **M3 Commit 4 完成（2026-10-19）：C 面执行 + 桌面激活**。C 面 18 案入
  执行面：**18/22 子测试 Pass，文件口径 14/18 全绿**——maintains-mousedown
  （buttons 键态）/all_mouse_events（click 组合 + 独占 retarget）/
  suppress_boundary（pointermove 同发 + 零 boundary）/mouse_buttons_back_forward
  （button 穿透）翻绿；红案 4 = constructor（Event.NONE 跨域残留，不占分母）/
  shadow（TBD-1）/wheel×2（TBD-3）。桌面激活（§6.6）：宿主仲裁镜像
  `pointer_lock_tabs`（回执维护 + 三连清）+ 转发门/Esc Exit 相位（dnd 邻位）
  + chrome 悬停跳过 + IPC 契约 Exit + renderer 臂。实施期修正两点
  （button 签名/Exit 相位）→ RFC v1.4。见
  [Commit 4 证据](pointerlock-compat/evidence/2026-10-19-m3-commit4-cface-desktop.md)。
- **M3 s3 管线就绪（2026-10-19，Commit 3）**：webview 锁镜像新模块
  `pointer_lock.rs`（`apply_locked_pointer_move`/`_button` 单入口 + 导航四连点
  清理）+ host 桥 `__zw_pointer_lock_state_changed`（shim 四处通知 + identity
  bridge 三武装点并列注册，浏览器/runner 无分叉）+ IPC 双向
  （PointerLockInput/PointerLockChanged + renderer 臂 + process_backend 排队）。
  e2e 桥测试一次全绿（grant→镜像→改道位移→exit→报告）。实施期修正四点
  （事件派发改道不实施/payload 纯 sel/省 tab_id/注册点在 webview）→ RFC v1.3。
  见 [s3 证据](pointerlock-compat/evidence/2026-10-19-m3-s3-lock-pipeline.md)。
- **M3 s2 完成（2026-10-19）**：位移流字段落地（FR-4，RFC §6.3 三层：序列化
  movementX/Y + runner 事件源算 delta + shim mouse 分支透传）。**实施期发现**
  shim `MouseEvent` 构造键表缺 movementX/Y（键表外 init 键静默丢弃——
  constructor.html M1 红案真根因），part05 补表项 → RFC v1.2；
  mouse 分支合成 screenX=clientX（§6.3 假设落定）。movementX_Y_basic 实跑
  翻绿（skip 窄化 13→12），执行子集 **4/5 = 80.0%**。见
  [s2 证据](pointerlock-compat/evidence/2026-10-19-m3-s2-movement-stream.md)。
- **M3 s1 完成（2026-10-19）**：shim 语义组落地（FR-1 激活门 NotAllowedError
  不消费 + FR-2 事件面新派发器 + FR-3 options/no-op 细化 + FR-6 移除解锁 +
  FR-10 导航清理含 `_fs*` 同族 + §6.2 真实输入激活白名单）。B 面 2 案翻绿
  （1/4 → 3/4 = 75.0% 中间口径）；fullscreen 桥测试零回归；make test 全绿。
  实施期修正两处（激活不消费/有效布尔比对）→ RFC v1.1。见
  [s1 证据](pointerlock-compat/evidence/2026-10-19-m3-s1-semantic-group.md)。
- **M1 完成**：corpus 导入 27/27 测试资产（pin 31597693）+ census + 可执行
  子集甄别 + runner 通道（`make testharness-pointerlock`）+ 基线落账 +
  csv planned 行回填（active，1/4 = 25.0%）。
- **关键盘点发现**：API 面**已存在**——R2939（2026-08-07，Fullscreen 同轮）
  在 js_dom_shim 落地 spec-alike 锁态（requestPointerLock Promise grant/deny、
  exitPointerLock void、pointerLockElement、change/error 事件 + on* handler、
  host `__zw_pointer_lock_enabled` deny 开关）。缺口五项（transient activation
  门 / deny TypeError≠NotAllowedError / 位移流 / unadjustedMovement / 全屏
  联动+移除解锁+retarget）已列入甄别报告 M2 RFC 范围输入。
- **M2 RFC（2026-10-19）**：用户点名批准起草，同日征询**确认通过**
  （v1.0；M3 实施期修正 → v1.1）。方案 = shim 语义权威 + webview 锁镜像
  双馈管线；FR-1~FR-10 覆盖 M1 五缺口 + 探索新发现 N1–N4（change/error
  事件面 bubbles/target/trusted 不符、重复请求 no-op 条件过粗、锁态/激活
  跨导航泄漏、锁态宿主不可见）；`_fs*` 同族跨导航泄漏随 FR-10 一并修
  （用户确认）；s1–s4 切片、五提交切分。Lint 26P/0F。
- **M3 点名（2026-10-19）**：用户点名启动，按 §8 交接顺序 s1→s4 实施，
  实施阶段持续执行不中断。
- **跨域 residual（✅ 2026-10-19 已收敛）**：`Event.NONE` 等 Event 接口
  phase 常量缺失（constructor.html 红，原归 dom/events 域 / uievents 50
  residual）——native `dom_bindings/event.rs` `attach_event_constants` 补装，
  通道翻绿；附带 dom 通道 Event 面 +21 subtest（见
  [constructor 证据](pointerlock-compat/evidence/2026-10-19-constructor-residual-event-constants.md)）。

## Mission

M1 完成 corpus census 与可执行子集甄别（API 面存在性/异常语义可静态执行的
部分）；捕获管线（相对位移流/unadjustedMovement/全屏联动语义）出 RFC 后按
用户点名推进。排除：mouse lock 之外的指针 API（编号 50 域）、游戏手柄（远期池）。

---

## Done Criteria

- **DC-1（自主）✅ 2026-10-18** 导入 + census + 可执行子集甄别报告 + csv
  planned 行回填——见 [M1 基线](pointerlock-compat/evidence/2026-10-18-m1-baseline.md)
- **DC-2（✅ 2026-10-19）** 捕获管线 RFC 已确认（方案 A，终版 v1.7）；M3
  实现收口完成（s1–s4 + TBD-3 收敛 + constructor residual 收敛）
- **DC-3（✅ 2026-10-19）** 回归测试账本纪律（通道 22/22 逐案落账、红案
  全部收敛——constructor residual 切片关闭最后一红）；make test 全绿、
  make reftest 712/712 零回归

## 里程碑

M1 盘点（自主，**✅ 完成**）→ M2 管线 RFC（**✅ v1.7 已确认 2026-10-19**）→
M3 实现与收口（**✅ 完成 2026-10-19，s1–s4 + Commit 5 收口 + TBD-3 收敛 +
constructor residual 收敛，通道 22/22 全绿**）。

## 依赖约束（run-rules §9）

host-runtime 光标/输入面与 keyboard 系遗产（已归档）同域；与 28-dnd-compat
的输入注入管线 RFC 合并征询可减重复评审。
