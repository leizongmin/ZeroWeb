# 桌面浏览器应用产品化 — browser-shell 真窗口验收（M11）

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active
**执行模式**: 演示流驱动（每功能一条可脚本重放的验收流）；遇深结构（GPU surface 平台
差异、engine 文本搜索 API 缺位）→ 记「待用户决策」→ 跳过 → 继续其他功能面
**父目标**: `docs/goal/zero-web.md`（M11 浏览器应用 + DC-2「浏览器日常可用」主路径）

> **说明**
> 本文档是 ZeroWeb「桌面浏览器应用产品化」专项目标执行契约。browser-shell 已有
> 标签页/书签/历史/下载/设置数据模型，但产品形态与真实窗口验收缺位——本目标把
> ZeroBrowser 从「数据模型 + headless 冒烟」推进到「真窗口日常可用」。
>
> **▶ 拆分动机（2026-09-12 用户决策）**：① 父目标三大交付物之一（ZeroBrowser 桌面
> 应用）的主路径，DC-2 整段未验收；② M11 交付物清单明确（地址栏/下载/查找/缩放/
> 菜单/设置），范围天然可契约化；③ 与 android-browser（Kotlin chrome）产品面独立，
> 可安全并行。
>
> **▶ 基线事实（2026-09-12 实测）**：
> - **browser-shell 现状**：标签页/书签/历史/下载/设置/上下文菜单等**数据模型**落地
>   （architecture.md 口径）；真窗口/GPU/display 产品验收未做（ROADMAP M11 🚧）。
> - **host-runtime**：winit 窗口/事件循环/IME/transport 面存在；桌面主链路（窗口 →
>   surface → 渲染 → 输入 → 导航）的端到端产品验收缺失。
> - **合成器**：compositor-process RFC 已落地（Linux 默认 GPU dma-buf 链路，frame_flow
>   17/17）——真窗口 GPU 合成路径有底座，父 DC-4.4「GPU 加速合成」的显示环境验收
>   联动本目标 M1。
> - **viewport 桥遗产**：event-loop-spec ④ shim viewport 真值桥（innerWidth ← config）
>   已 default——缩放功能（Ctrl±）的页面侧联动有基建。
> - **碰撞面**：`apps/browser` 与 android-browser（活跃并行流）、devtools goal（M3
>   GUI 模式 CDP server 接线）共享。

---

## Mission

以**逐功能可脚本重放的演示流为验收标尺**，把 ZeroBrowser 桌面应用推进到三平台
「日常可用」：真窗口主链路稳定、导航/标签流畅、内容工具（查找/缩放/下载）可用、
数据面（收藏/历史/设置）完整。

**关键约束**：
1. **演示流可重放**：每个功能验收 = 脚本化流（driver 或 CDP 原语）+ 记录留档，
   不接受纯人工口说验收。
2. **生产渲染零回归**：功能开发不得触动渲染管线语义；reftest/testharness 门禁照常。
3. **三平台声明**：Linux 为第一验收平台（本仓 CI/开发环境），macOS/Windows 以
   编译 + 冒烟验收（CI 矩阵覆盖），深体验验收如实标注平台范围。

覆盖范围：

1. **真窗口主链路** — winit 窗口 → surface → 渲染管线 → 输入 → 导航端到端；
   GPU 合成路径显示验收（联动父 DC-4.4）
2. **导航与标签** — 多标签（创建/关闭/切换/拖拽排序）、地址栏（URL 输入/自动补全/
   加载进度）、导航控制（前进/后退/刷新/主页）
3. **内容工具** — 下载管理器（进度/打开所在文件夹）、页面查找（Ctrl+F 搜索高亮，
   需 engine 文本搜索 API）、缩放（Ctrl±/重置）、右键上下文菜单
4. **数据面** — 收藏夹（文件夹/收藏栏）、历史（记录/搜索/清除）、基础设置页
   （默认搜索引擎/主页/隐私）

### 排除（明确不在范围内）

- 自动更新机制（M14，后续评估）
- 完整 DevTools 面板 UI —— devtools goal（复用 frontend 路线）
- 移动端 UI —— android-browser goal
- 渲染兼容性本身 —— rendering-compat 流（本流只消费渲染结果）

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| apps/browser | 桌面入口主链路、功能接线 | 与 android-browser 并行流碰前 git log |
| browser-shell | UI 组件从数据模型到交互面 | 标签/地址栏/收藏/历史/设置 |
| host-runtime | 真窗口 surface/输入/IME 集成验收 | winit 平台差异记账 |
| engine | 页面查找文本搜索 API、缩放 viewport 联动 | 最小 API 面，不动渲染语义 |
| 验收资产 | 每功能演示流脚本 + smoke 资产化 | evidence/ 持续追加 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 android-browser（活跃并行流）**：`apps/browser` 共享——碰前
  `git log --since="14 days ago" -- apps/browser/ crates/browser-shell/` 核对。
- **与 devtools goal**：其 M3（GUI 模式 CDP server 接线）与本目标同改 `apps/browser`
  ——碰头协调排序，避免同窗并发改主进程接线。
- **与 rendering-compat**：crate 零重叠（本流不触渲染子 crate；发现要碰即暂停记档）。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: 真窗口主链路

- [ ] 三平台（macOS/Linux/Windows）可编译启动（CI 矩阵佐证）；Linux 真窗口端到端
      演示流绿（启动 → 加载 URL → 渲染 → 输入交互）
- [ ] GPU 合成路径显示验收记账（联动父 DC-4.4 证据归档）

### DC-2: 导航与标签

- [ ] 多标签演示流（创建/关闭/切换/拖拽排序）全绿
- [ ] 地址栏演示流（URL 输入导航/自动补全/加载进度）+ 导航控制演示流全绿

### DC-3: 内容工具

- [ ] 下载管理器演示流（触发下载/进度/打开所在文件夹）全绿
- [ ] 页面查找演示流（Ctrl+F 搜索/高亮/计数）全绿（engine 文本搜索 API 就位）
- [ ] 缩放演示流（Ctrl±/重置，页面 viewport 联动）全绿
- [ ] 右键上下文菜单演示流（复制/粘贴/检查元素入口）全绿

### DC-4: 数据面

- [ ] 收藏夹演示流（添加/删除/文件夹管理/收藏栏）全绿
- [ ] 历史演示流（记录/搜索/清除）全绿
- [ ] 基础设置页演示流（默认搜索引擎/主页/隐私）全绿

### DC-5: 测试与质量不可退让

- [ ] `make test` 全绿零失败 + clippy `-D warnings` + fmt 干净
- [ ] 全部演示流脚本化入 evidence/ 可重放；product smoke 资产随功能同步扩充

---

## 活跃里程碑

### M1 — 真窗口主链路验收

**目标**：Linux 真窗口端到端演示流（启动/加载/渲染/输入）+ GPU 合成显示验收 +
macOS/Windows CI 编译冒烟。

### M2 — 导航与标签

**目标**：标签族 + 地址栏 + 导航控制演示流。

### M3 — 内容工具

**目标**：下载/查找（engine 文本搜索 API）/缩放/右键菜单演示流。

### M4 — 数据面

**目标**：收藏/历史/设置演示流。

### M5 — 收口

**目标**：DC 逐项判定 + smoke 资产定稿 + 挂账（平台差异项）。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方「DONE 允许条件」 |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~5 全部满足；演示流基于真实窗口（Linux 端到端）且可脚本重放；
`cargo build` + `make test` + `cargo clippy` 全过；master.md 内部自洽。

---

## Execution Protocol

### 自主执行原则

1. **每功能一演示流**：功能与验收流同切片 land，不留「先功能后补验收」
2. **Linux 先行**：真窗口深验收 Linux 优先，macOS/Windows 差异记账
3. **渲染语义不碰**：UI/接线层问题不外溢到渲染子 crate

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮
2. **平台差异**（winit/GPU/surface）：Linux 口径先行，差异项记「待用户决策」或挂账
3. **深结构拦路**（engine 文本搜索 API 缺位等）：评估最小 API 面自主做；超范围记决策

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。
  仅在目标本身实质变化时修改；每轮执行不重写。
- **运行时控制平面** `docs/goal/desktop-browser/master.md`：当前真实状态唯一控制面板。
- **归档区域** `docs/goal/desktop-browser/archive/`：只追加不修改。
- **证据区域** `docs/goal/desktop-browser/evidence/`：演示流记录与 smoke 资产，持续追加。
