# M2 切片 33 — light dismiss 两遍模型：auto popover 关闭 + dialog 链停（2026-10-23）

## 范围

s32 挂账 E 第一刀：engine click 路径补 auto popover light dismiss（此前
R3071 defer 至今未回填）+ dialog closedby light dismiss 链停修正（一次
click 关全部 any dialog → 一次一链）。

## 落地（R5234，`__zw_dialog_light_dismiss_check` 两遍重写）

入口不变（`__zw_pointer_down_sequence` 在 R4300 light dismiss 前步调用），
内部改两遍：

1. **popover pass**：`Object.keys(_zwTopLayer)` 反走（插入序即显序，反走
   为自顶向下）。stale 键 / 属性已摘（`_zwReadPopover===null`）/ 断连
   跳过；`contains(target)` 即 break（包含点 → 级联停，其下全保持）；否则
   `_zwHidePopover`（s32 lenient 语义，正常走 beforetoggle/toggle 流）。
   spec 依据：popover light dismiss 对点击目标不在其内的 showing auto
   popover 逐层关闭（popover.html#light-dismiss-clear-close-watcher-stack）。
2. **dialog pass**：`querySelectorAll('dialog')` 反文档序（内层先）首个
   open+any dialog 一锤定音：backdropHit 且 target 为该 dialog 自身 →
   requestClose；contains(target) → break（内层 any 挡外层——点击在
   内层 dialog 内时外层不因链停误关）；否则 requestClose + break（一次
   pointerdown 至多关一个 dialog，spec light dismiss 一次一链）。
3. **shadow 守卫**：targetSel 非空但解析不到（shadow 内元素 sel 不在
   文档树 / stale）→ 保守跳过 light dismiss。complex 影域 testcase 全部
   clickOn 落 shadow 元素，此守卫保证不因误判引入回归（REG 0 前提）。

契约兼容：part60 `test_dialog_closedby_light_dismiss_r4300` R4（单
dialog light dismiss 三态：内保持 / 外关 / closerequest 不 light
dismiss）与重写兼容，1 passed。

## 集群验证

- dialog 集群 82 files **411 subtest 337P/73F/1T**（数据
  [2026-10-23-m2s33-light-dismiss-two-pass.json](2026-10-23-m2s33-light-dismiss-two-pass.json)），
  同 key 411 对 s32 账本逐位 diff：回归 **0**、增益 **16**（key 集零漂移）。
- **增益 16 全部集中在 closedby-complex normal 面的 dialogA/outside-all/
  popoverA 三类点击场景**（5+6+5）：popover light dismiss 从无到有 +
  dialog 链停一次一链，两遍模型按 spec 语义裁决。
- closedby-simple 1P/5F 维持（ESC 栈序，挂账 E 余项）；focus-after-close
  1T 等其余残余红为 s32 即有（REG 0 保证未动）。

## 探针破案（zz-probe-s33-ld，已删）

最小嵌套栈（dialogA > popoverA > dialogB > popoverB，加 dialogC 对照）
+ `__zwLdDbg` 决策日志，两案钉定：

- **light dismiss 逻辑按设计工作**：popover pass 反走裁决正确
  （popoverB 不含 target → 隐；popoverA 包含 → break），残余红不在
  两遍模型。
- **零盒根因 = 快照滞后**：`waitForRender`（双 rAF）在 shim OFF 模式
  （reftest 兼容默认路径）是同步 stub，不驱动 render；render 只发生在
  runner probe 拍间（mutation 门控刷新）。测试体内 setup 后立即 gBCR
  永远读到上一拍快照——首个 promise_test 全零盒、第二个测试读到首测
  点击触发的 render 代际（dialogB 有盒、dialogC 零盒铁证）。

### 帧驱动实验（挂账 F 实证）

`ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（R2713a 既有 kill-switch，probe 每
拍派发 `__zw_raf_tick`）复跑：

- 探针五元素全部有盒（快照新鲜）；
- closedby 两语料 46 子测全出（simple Timeout 也解体）**25P/21F**，
  对 OFF 同 key：**回归 0、增益 +8**（normal dialogB/popoverB 8 案
  全绿）。

未默认启用：rAF 时序语义变化需全集群 82 files 对账，独立切片。

## 挂账（本轮钉定）

- **F 帧驱动快照新鲜化**（runner，下一切片候选）：testharness-html-
  semantics 套件默认启用 `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1` + 全集群
  对账。已实证 +8 无回归。
- **G runner modal retarget 多模态规则**：点击落在下层 modal dialog 盒
  内仍被 retarget 成栈顶模态（探针实证 `clickOn(dialogB)` 落点
  (400,209) 在 dialogB 盒内 → `t=dialog.dialogC`）→ 链路错关。嵌套
  modal 的 light dismiss 正确性依赖此（retarget 应判落点命中的最深
  modal）。帧驱动 ON 后 dialogB/popoverB normal 8 案已绿说明多数面
  不依赖 retarget，残余在多模态叠加面。
- **H shadow 影域 light dismiss**：complex shadow slots 变体 16F——
  clickOn 落 shadow 内元素，shim 侧 targetSel 解析不到文档树元素，
  守卫保守跳过。需 target 解析穿 shadow（与 runner 命中面同源）。
- **E 余项维持**：ESC 栈序（simple 5F——「popover should close after
  first ESC」面，需统一 top-layer 序覆盖 popover 候选，涉 R4300 组栈
  改造）。
- A 强制同步布局冲刷的本流实体收敛进 F（waitForRender 通道）；
  B abspos 收缩（渲染流）、D getComputedStyle :modal 注入（低优）维持。

## 账本与门禁

- zero-engine lib **3429P/0F**；clippy `-D warnings` clean、fmt clean；
  零新增依赖。runner release 二进制重建（shim include_str! 进二进制）。
- 探针语料已删，集群账本为干净复跑（移除调试后重建 runner 再跑）。
