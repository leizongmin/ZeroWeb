# 键盘页面滚动 — WPT 驱动的滚动交互正确性目标

**版本**: v1.0
**日期**: 2026-08-17
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/html-compat.md`（HTML 行为兼容——本专项为其滚动交互面延伸）

> **说明**
> 本文档是 ZeroWeb「键盘页面滚动」专项目标执行契约。目标是把页面级键盘滚动
> （PageUp/PageDown/Space/Home/End/方向键 + Ctrl/Shift 修饰变体 + 焦点滚动 scroll into
> view）的默认动作与事件语义对齐到 Chromium 水平，以 WPT 真实用例通过率为验证标准。
> 本文定义 Mission、边界、Done Criteria、执行协议和文档治理规则，供后续 `rally run`
> 会话作为稳定输入。日常进展、evidence、active milestone 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-08-17 用户决策）**：与 keyboard-default-actions 同批拆出（键盘/编辑
> 方向三拆之三）。理由：① 滚动是「页面能翻吗」的第一手感——长文阅读/搜索定位/键盘用户
> 无障碍的基础，当前键盘滚动面零散（鼠标滚轮有 page_scroll.rs，键盘默认动作不系统）；
> ② 工作面清晰且**有新鲜底座**——p1a-element-scroll-design（2026-08-12）+ scroll 钳位
> 修复（b12f9b67 clamp scroll to rendered content）刚落，滚动管线是活跃基础设施；
> ③ 与 rendering-compat 的 scroll-snap（已实现）衔接但不重叠（本目标管键盘触发，不管
> snap 布局）；④ 零撞 js-dom（host 输入层工作面）。
>
> **▶ 基线事实（2026-08-17 实测）**：
> - **滚动管线**：`apps/browser/src/page_scroll.rs`（鼠标滚轮路径）+ 根滚动范围
>   `min(layout, painted)` 钳位（b12f9b67，2026-08-16 CI 修复轮固化）+
>   `local_composite_cpu_gpu_matrix_for_form_interactions` e2e（滚动后合成帧断言）。
> - **元素滚动**：p1a-element-scroll-design-2026-08-12（element.scrollTop/scrollLeft +
>   overflow scroll 容器——js-dom 流 P1a 产物）。
> - **scroll-snap**：解析 + 渲染指示已实现（css-parser/style-system/engine——master.md
>   Tier 1 表 ✅）。
> - **键盘滚动默认动作**：PageUp/PageDown/Space/Home/End/方向键的页面滚动分发——无系统性
>   实现（键盘输入经 app_input 派发 keydown 到页面，默认动作层不系统）。
> - **WPT 面**：无键盘滚动专项导入。

---

## Mission

以 WPT 真实用例通过率为验证标准（`css/css-scroll-snap` 键盘交互面 + `html/interaction`
滚动激活面 + custom reftest），把键盘页面滚动的默认动作、修饰键变体、焦点滚动与 scroll
事件语义对齐到 Chromium 水平。分阶段里程碑校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **基线建立** | 键盘滚动可观察面（scrollY/scroll 事件/scrollIntoView）用例导入 + 基线 |
| 中期 | **全键位通** | PageUp/PageDown/Space/Home/End/方向键（含 Ctrl/Shift 变体）的默认滚动量正确 |
| 长期 | **90%+（可校准）** | scrollIntoView（含 behavior/inline/block 选项）、容器级滚动、scroll-snap 键盘交互、嵌套滚动传播 |

**关键约束**：验证优先用上游 WPT 真实用例；键盘滚动量的精确像素期望在上游用例中较稀缺，
**不足处用本地 reftest/单测补**（等价本地断言，同 CLAUDE.md 测试资产化规则「无法导入上游
用例时至少补等价本地测试」）——本地用例须在 evidence 中标明，不冒充上游用例。

覆盖范围：

1. **滚动键默认动作** — PageUp/PageDown（视口高减一行）、Space/Shift+Space（视口高）、
   Home/End（文档顶/底）、方向键（行高步进）、Ctrl+Home/End；`preventDefault()` 阻断
2. **滚动目标判定** — 焦点在可滚动容器内时滚容器、否则滚最近可滚动祖先/根；嵌套滚动
   传播（内层到底后传外层）
3. **scrollIntoView** — `scrollIntoView()`/`scrollIntoViewIfNeeded()`（含
   `{behavior, block, inline}` 选项——behavior: smooth 的 headless 简化为 instant 须记录）；
   聚焦元素自动 scroll into view（与 keyboard-default-actions 的焦点滚动衔接）
4. **scroll 事件语义** — 滚动后 scroll 事件派发（cancelable=false、target 正确）；
   scrollEnd 简化处理记录
5. **scroll-snap 键盘交互** — snap 容器内滚动键的吸附行为（与已实现 snap 指示器衔接）
6. **JS 可观察面** — `window.scrollY/scrollX`、`window.scroll()/scrollTo()/scrollBy()`、
   `element.scrollTop/scrollLeft` 赋值（p1a 已有底座）与键盘滚动的一致性

执行方式：**交替推进** — 每轮同时扩展用例覆盖和修复发现的缺口。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| 键盘滚动默认动作 | 滚动键分发层 + 各键滚动量 | host 输入层（browser app_input → engine 滚动管线） |
| 滚动目标判定 | 焦点元素 → 可滚动容器 → 根的冒泡链 | 与 p1a element scroll 底座衔接 |
| scrollIntoView | 全选项（smooth 简化须记录） | 与键盘默认动作的焦点滚动共享实现 |
| scroll 事件 | 派发语义 + JS 可观察面一致性 | 走既有事件管线 |
| snap 键盘交互 | snap 容器内滚动键吸附 | snap 布局已有，接键盘触发面 |
| WPT 基础设施 | 用例导入 + 本地 reftest 补足 | 复用 tests/wpt-runner + `make import-wpt`；本地用例标明不冒充 |
| 单元测试 | 每项修复带单测 | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **指针滚动**（鼠标滚轮/触摸板/触摸滑动）— 已有路径（page_scroll.rs），维护归零-web 流
- **表单控件键盘默认动作** — 兄弟目标 `keyboard-default-actions.md`
- **文本编辑键** — 兄弟目标 `editing-contenteditable.md`
- **滚动条 UI**（可见滚动条拖拽/样式）— 渲染域（rendering-compat）
- **overscroll 行为/滚动链细节 spec 全量**（overscroll-behavior 指示器已有）— 深化归
  rendering-compat 流域
- **smooth 滚动动画帧插值** — headless 简化为 instant（记录限制即可，不做动画系统）

### 依赖约束

- **与 keyboard-default-actions 的边界**：滚动键（PageUp/Space/Home/End/方向键）在**非编辑
  宿主**的默认动作归本目标；编辑宿主内的键归 editing-contenteditable。分发顺序：编辑宿主
  优先消费，未消费走本目标的滚动默认动作。
- **与 js-dom 流碰撞管理**：element.scrollTop/scrollLeft 段（p1a 产物）若该流仍在迭代，
  先 `git log --since="14 days ago" -- crates/engine/` 核对；活跃则先做零碰撞面
  （browser 输入层滚动分发、WPT 导入、reftest 资产）。

---

## 当前能力/缺口基线

**详见** [keyboard-page-scrolling/master.md](keyboard-page-scrolling/master.md)（运行时
控制面板，唯一真实状态来源）。

**关键摘要**（2026-08-17 实测）：

- ✅ **滚动管线底座**：page_scroll.rs（滚轮路径）+ 根滚动 `min(layout, painted)` 钳位
  （b12f9b67）+ 滚动后合成帧 e2e 断言
- ✅ **元素滚动**：p1a element scrollTop/scrollLeft + overflow 容器（2026-08-12 设计已落）
- ✅ **scroll-snap 布局/指示器**：已实现（Tier 1 表 ✅）
- ⚠️ **缺口 1 — 键盘滚动默认动作缺失**：滚动键无系统分发（键位/滚动量/修饰键变体）
- ⚠️ **缺口 2 — 滚动目标判定缺失**：焦点→容器→根的滚动链未实现
- ⚠️ **缺口 3 — scrollIntoView 语义未核实**：选项面/自动聚焦滚动待摸底
- ⚠️ **缺口 4 — scroll 事件与键盘滚动联动未核实**
- ⚠️ **缺口 5 — 用例覆盖为零**：无键盘滚动专项用例（上游 + 本地）

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: 用例导入与基线

- [ ] 导入可用的上游 WPT 真实用例（css-scroll-snap 交互面 / html interaction 滚动面）
- [ ] 上游不足处补本地 reftest/单测（evidence 中标明「本地」不冒充上游）
- [ ] 建立分类通过率报告（文本 + JSON），记录基线
- [ ] driving 用例经 `make import-wpt` 常驻断言集并记入 `imported-tests.txt`
- [ ] 报告持久化到 `docs/goal/keyboard-page-scrolling/evidence/`，历史可追溯

### DC-2: 滚动键默认动作全通

- [ ] PageUp/PageDown/Space/Home/End/方向键（含 Ctrl/Shift 变体）滚动量与 Chromium 一致
- [ ] preventDefault 阻断滚动
- [ ] 焦点→可滚动容器→根的滚动目标判定 + 嵌套传播

### DC-3: scrollIntoView 与事件语义

- [ ] scrollIntoView（block/inline 选项；smooth 简化为 instant 有记录）+
      scrollIntoViewIfNeeded
- [ ] 键盘滚动后 scroll 事件派发正确；window.scrollX/scrollY 与实际一致

### DC-4: snap 键盘交互

- [ ] snap 容器内滚动键触发吸附（与既有 snap 管线集成验证）

### DC-5: 测试与质量不可退让

- [ ] `cargo test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + 用例资产化（上游或标明本地）

---

## 活跃里程碑

### M1 — 基线建立 + 现状摸底

**目标**：导入用例记录基线；摸清 scrollIntoView/scroll 事件/键盘分发现状。

**切片建议**：
1. 上游用例导入（可执行面）+ 本地 reftest 骨架（滚动量断言）
2. 键盘滚动分发层骨架（键位→滚动量映射，根滚动先行）
3. 失败聚类 → 轻量修复队列

### M2 — 全键位 + 滚动目标判定

**目标**：全滚动键（含修饰变体）+ 焦点→容器→根链 + 嵌套传播。

### M3 — scrollIntoView + 事件 + snap 交互收尾

**目标**：scrollIntoView 选项面、scroll 事件语义、snap 键盘交互；通过率达标（阈值按
M1 基线校准）。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~5 全部满足；验证基于上游真实 WPT 用例 + 标明本地的等价用例（无冒充）；
`cargo build` + `cargo test` + `cargo clippy` 全通过；master.md 内部自洽，archive 已建立。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**当前滚动管线（page_scroll.rs / element scroll / 钳位）与键盘分发的接缝
2. **自主导入/创建**用例，扩大覆盖范围
3. **自主运行**用例，分析失败原因（分发缺失？滚动量？目标判定？事件？）
4. **自主修复**，不等待用户逐步指令；每修 net≥0 即 land
5. **自主添加测试**，新修复必须有对应单元测试 + 用例资产化
6. **自主验证**：`cargo test` + clippy + 通过率确认修复有效
7. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：用例驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：碰 js-dom 共享面（engine element scroll 段）前先 `git log` 核对；有活跃
   编辑则转零碰撞面（browser 输入层、WPT 导入、reftest 资产）。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。
2. **用例失败分析**：每个失败 case 必须分析根因（键位映射？滚动量？目标判定？snap？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/keyboard-page-scrolling/master.md`：当前真实状态的唯一
  控制面板。治理规则：持续演进、不允许无限增长、各章节必须自洽。
- **归档区域** `docs/goal/keyboard-page-scrolling/archive/`：存储已完成里程碑的详细过程与
  历史证据，只追加不修改。
- **证据区域** `docs/goal/keyboard-page-scrolling/evidence/`：存储通过率报告、失败分析等
  验证证据，持续追加。
