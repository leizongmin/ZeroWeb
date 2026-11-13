# M2 s427 复验账——组合态全域 sweep ×2 复验轮（s424 后三笔他流到货整验，
# 零漂移；簇 D flake 键沾染态收敛为稳定 T；基线移交 s427 双轮）

- 日期: 2026-11-14；基线: HEAD 57b809c08（s426 收口提交，无新到货），
  驱动: CONTINUE 指令 + 归因纪律（rule 10 组合态整验）。本轮零源码改动：
  s424 全域 sweep 之后 main 已落三笔他流变更，本域 8458 键组合态计数从未
  整验——池空等待环境下按归因纪律跑组合态复验，零漂移固化健康证据。

## 一、触发与方法

- 触发: s424 全域 sweep（基线 /tmp/s424/pairs-{1,2}.txt）之后 main 相继
  落三笔他流变更——①渲染流 R5516 匿名 cell margin 面（d2c3897da，
  layout-engine）；②PR #301 navigator 平面对象（a46a05e03，engine shim
  part02.js **共享面**）；③渲染流 css-fonts font-* 数值槽 calc 接入
  （298f54c14，style-system）。其中仅 solo 邻位复验过（s425 表格三键、
  s426 补记 btn/canvas drift solo），全域计数未整验。
- 方法: runner 重建自 HEAD 57b809c08（test-guard 包裹
  `--per-proc-mem 6 --total-mem 12 --time-limit 900`，47.8s exit 0）→
  全域 sweep ×2（`ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`，test-guard 包裹
  `--per-proc-mem 4 --total-mem 8 --time-limit 14400`，8 jobs，双轮各
  **8458 键**同 s424 域大小，双轮 exit 1=红键在册符合预期）→ stable-set
  join（s343 语义：双轮同态入集，双态/Timeout 沾染键单列归因）对 s424
  双基线。

## 二、结果（stable-set join 全量）

- **stable-F**: cur 201 = base 201，new=0 gone=0（集合逐键相同）。
- **stable-P**: cur 8241 = base 8241，new=0 gone=0（集合逐键相同）。
- **stable-T**: cur 14 vs base 13，唯一 delta = 簇 D 已知 flake 键
  `customizable-select/select-home-end-pagedown-pageup-detailed.optional`
  ——s424 沾染态（轮1 Fail / 轮2 Timeout 双态不一致，按 T-in-at-least-one
  记账惯例入 14T）→ s427 双轮稳定 Timeout。**记账桶 T 不变**（该键在册
  红属性未变，状态标签由沾染收敛为稳定 T）。
- **沾染面**: base 1 键 → cur 0 键（唯二沾染候选即上键；s424 轮1
  202F+13T / 轮2 201F+14T raw 分裂即由它产生）。
- **raw 收敛**: s427 双轮 raw 同为 **201F+14T**（比 s424 raw 双轮分裂更
  收敛——簇 D 家族本轮无闪变）。
- 总口径: **201F+14T 不变**，与 s424 终态稳定红面逐键一致。

## 三、判读

- 三笔他流到货（layout-engine 匿名 cell margin / 共享面 part02.js
  navigator / style-system css-fonts calc）对本域 **零漂移零翻转**：
  F/P 两大集合逐键相同即排除了跨流回归与意外翻绿两类风险；唯二 delta
  在簇 D flake 家族内（GB-20261031 用户门在册），非引擎面变化。
- PR #301 navigator 共享面的 solo 邻位复验（s426 补记）升级为全域整验
  通过；css-fonts calc 到货首验（s426 时未单独 solo）亦零位移。
- 基线移交: 后续施工轮（act:option 臂）join 基线改用本轮
  **/tmp/s427/pairs-{1,2}.txt**（双轮收敛、沾染 0，优于 s424 双基线）。

## 口径

- 域 corpus 计数零位移（零源码改动；在册红维持 201F+14T 稳定口径，
  s427 raw 双轮同数即该口径的直接整验）。
- 用户门维持：GB-20261031（簇 D）、GB-20261114-VIDEO（video P1/P2/P3）、
  s137（form-nameditem）。

## 下一步

- 等待面不变：渲染流 listbox 形态 carve-out 到货 → engine act:option 臂
  施工（solo seltouch + the-select-element 目录回归 + 全域 sweep ×2 join
  对 **s427 双基线**）；dialog R-s31 簇/img density/embed sizing face
  到货复验；门决策后 video P1+P2 或簇 D A2 分诊续。
