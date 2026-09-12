# 计时与动画兼容 — hr-time / performance-timeline / user-timing / WAAPI

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active（已立项，未启动——启动顺序由用户点名）
**执行模式**: WPT 驱动（上游计时/动画面 corpus 为验收标尺）+ 语义修齐；
定位为**轻量快赢切片**（不触布局/渲染计算，只做 JS API 面）
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——计时与动画面）

> **说明**
> 本文档是 ZeroWeb「计时与动画兼容」专项目标执行契约。performance.now 与 WAAPI
> 是性能度量与程序化动画的通用底座；本目标为四 goal 同批立项中的轻量面，
> 预期最快出数字。
>
> **▶ 拆分动机（2026-09-12 用户决策，四 goal 同批立项之一）**：① performance
> 时间轴是埋点/性能库（含 rally 自身度量语义对标）的通用依赖；② WAAPI 是
> CSS 动画的程序化入口，shim 已有部分面（Animation 43 处）；③ 纯 JS API 面、
> 不触布局渲染计算，与渲染流零碰撞，适合作为四 goal 中的首个热身切片。
>
> **▶ 基线事实（2026-09-12 实测）**：
> - **计时面**：shim performance.now 15 处——精度语义（timeOrigin/单调性/
>     时间域）未度量；WPT hr-time/ corpus 存在
> - **WAAPI 面**：Animation 43 处 / getAnimations 6 处——部分存在未测；
>     WPT web-animations/ corpus 存在
> - **rAF 帧驱动**：engine 有 rAF（dom_bridge/callbacks.rs）；keyboard-page-scrolling
>     轮实证 headless 帧循环为 opt-in（`__ZW_RAF_FRAME_DRIVEN`）——帧时间戳精度是
>     已知约束，WPT 动画时序用例受此影响需如实标注
> - **WPT corpora**：`hr-time/` `performance-timeline/` `user-timing/`
>     `web-animations/`（testharness）

---

## Mission

以 **WPT 计时/动画面真实用例为验收标准**，落地单调高精度时间轴
（performance.now/timeOrigin）、performance timeline（mark/measure/observer 评估）
与 WAAPI（Animation/KeyframeEffect/getAnimations）语义，使性能度量库与程序化
动画可用。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **不触渲染计算**：CSS animation/transition 的渲染效果面归 rendering-compat；
   本 goal 只做 JS API 语义（时间轴/播放状态/效果解析）
3. **headless 帧精度如实标注**：帧驱动 opt-in 的已知约束记账，不用宽松容差粉饰

覆盖范围：

1. **hr-time** — performance.now/now 精度与单调性、timeOrigin、时间域
2. **performance-timeline / user-timing** — performance.mark/measure/getEntries*
   + PerformanceObserver 评估
3. **WAAPI** — Animation/KeyframeEffect/getAnimations/playState/finish/cancel
   + promise 语义

### 排除（明确不在范围内）

- **CSS animation/transition 渲染效果** —— rendering-compat 域（样式/布局/绘制）
- **resource-timing / navigation-timing** —— 依赖导航与资源管线上层面，挂账
  （重入条件 = navigation-compat M2 落地后评估）
- **帧调度宿主面** —— host-runtime 的 vsync/帧循环能力面，只消费不改造

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | performance 时间轴 + WAAPI 语义面 | part 系 JS 语义 |
| engine | Animation 对象桥（播放状态机，纯 JS 面） | 不改 painter/layout 动画计算 |
| WPT 资产 | hr-time/performance-timeline/user-timing/web-animations 子集导入 | fetch 脚本 + 账本 |
| 测试 | 单测 + 集成 + 上游 corpus 通道 | 照既有先例 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 rendering-compat**：不碰 crates/style-system、layout-engine、render-foundation
  的动画计算路径；CSS 动画渲染效果缺口记账回流。
- **与 event-loop-spec（已归档）**：rAF/微任务先例消费；观察器语义如其 IO/RO 账本
  已覆盖处不重复立账。
- **与 keyboard-page-scrolling（已归档）**：`__ZW_RAF_FRAME_DRIVEN` 帧驱动约束为
  消费事实，改动其门控须跨流核对。
- **与其他三 goal（同批立项）**：无共享面（计时/动画不触网络/导航/worker envelope）。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 导入与基线

- [ ] 四 corpus window 可执行子集导入（fetch 脚本 + 通道 + imported 账本）
- [ ] 分类通过率基线落 evidence/，并回填 docs/compat/trends/wpt-suites.csv planned 行

### DC-2: 计时面收敛

- [ ] hr-time 精度/单调性/timeOrigin 语义修齐
- [ ] user-timing mark/measure + getEntries* 语义；PerformanceObserver 评估结论记账

### DC-3: WAAPI 收敛

- [ ] Animation/KeyframeEffect/getAnimations/playState + finish/cancel promise 语义
- [ ] headless 帧精度约束如实标注（不放宽容差粉饰）

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿零失败 + clippy `-D warnings` + fmt 干净
- [ ] 每语义切片带单测/集成测试；`make reftest` 零回归

---

## 活跃里程碑

### M1 — WPT 导入与基线

**目标**：四 corpus fetch 脚本 + 导入 + 基线（纯资产零源码改动）。

### M2 — 计时面

**目标**：hr-time + user-timing/timeline 语义修齐。

### M3 — WAAPI

**目标**：Animation 状态机 + KeyframeEffect + getAnimations 修齐。

### M4 — 收口

**目标**：DC 逐项判定 + resource-timing/navigation-timing 重入条件挂账定稿。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方「DONE 允许条件」 |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；验证基于上游真实 WPT 用例（帧精度约束如实标注）；
`cargo build` + `make test` + `cargo clippy` 全过；master.md 自洽，evidence 持久化；
resource/navigation-timing 重入条件挂账定稿。

---

## Execution Protocol

### 自主执行原则

1. **基线先行**：M1 纯资产切片零源码改动
2. **逐簇收敛**：WPT 失败聚类 → 逐簇修齐 → 账本更新 → suites CSV 回填
3. **跨域记账**：CSS 动画渲染效果缺口记账回流 rendering-compat，不越界硬改

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮
2. **帧精度容差**：WPT 动画时序用例按上游 fuzzy 注解处理，无注解不自行放宽
3. **WAAPI 与 CSS 动画桥接缺口**：记账回流，不单方面改样式系统

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。
  仅在目标本身实质变化时修改；每轮执行不重写。
- **运行时控制平面** `docs/goal/timing-animation-compat/master.md`：当前真实状态
  唯一控制面板。
- **归档区域** `docs/goal/timing-animation-compat/archive/`：只追加不修改。
- **证据区域** `docs/goal/timing-animation-compat/evidence/`：WPT 基线/修齐账本，
  持续追加。
