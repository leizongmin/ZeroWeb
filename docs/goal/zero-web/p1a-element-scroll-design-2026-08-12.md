# RFC: 元素滚动（overflow 子容器）+ 元素 'scroll' 事件派发 — 设计文档

**版本**：v0.1（设计草案，待实现）
**日期**：2026-08-12
**状态**：设计完成，待分片实现（跨 crate 多切片，涉渲染流域协调点）
**流域**：zero-web 主导（browser/renderer/engine），**layout-engine 暴露面为渲染流域协调点**（run-rules §9 碰头信号，实现时需与渲染流协调或点名）
**承接**：master.md「下一步优先级」P1a 长期 defer 项 + R3253（用户滚动文档级 'scroll' 派发）遗留的元素级缺口

---

## 0. 摘要

当前 ZeroWeb 仅支持**文档级**滚动（整文档内容 vs 视口，browser 侧 `TabScrollState` + `compute_page_scroll_layout` 处理视觉滚动）。**`overflow: auto|scroll` 子容器**的滚动完全不支持，且**用户滚动的 'scroll' 事件在主路径完全未派发**：

1. **用户滚动 'scroll' 事件主路径不可达（最高优先级真 gap）**——R3253 加了 renderer `handle_scroll_event`（收 IPC `ScrollEventParams` → 注入 `__zw_user_scroll` → 派 'scroll'），但**核实发现 browser 主路径从不向 renderer 发 ScrollEvent IPC**：`apply_page_scroll_delta`（app_input.rs:153）仅本地更 `TabScrollState` + 重绘，不发 IPC；multi-process（默认）与 single-process 均如此。renderer `handle_scroll_event` 仅为 `#[test]`（protocol/src/process.rs:795 harness）可达。→ **页面 JS 永远观察不到用户滚轮**（infinite scroll / lazy load / sticky nav / parallax 全断）。R3253 test（part01.rs:1253）直接在 sandbox 调 `__zw_user_scroll` 验证 hook 有效，但未验证 hook 主路径可达。
2. **视觉滚动缺失**——`overflow: scroll` 容器不产生可滚动表面，内容溢出直接裁剪或撑开文档（layout 仅为每个盒记 `overflow_x`/`overflow_y` clip 模式，`scroll_x`/`scroll_y` 硬编码 `0.0`，从未实际滚动）。
3. **元素 'scroll' 事件缺失**——用户在子容器上滚轮时，`scroll` listener 永不触发（依赖缺口 1 闭合 + 元素命中测试）。
4. **`element.scrollTop`/`scrollLeft` 程序化滚动**已部分实现（R3051 `_zwFireScroll` 派发程序化 'scroll'），但**仅更新 JS 态 `_scrollOffsets`，无视觉滚动效果**（视觉与 JS 态不一致）。

本 RFC 设计闭合这四层缺口，**优先级最高 = 缺口 1（用户滚动 'scroll' 主路径派发，纯 zero-web 自主面）**，分阶段切片实现。

---

## 1. 当前架构（直接代码核实，2026-08-12）

### 1.1 浏览器侧（zero-web 流）

- `apps/browser/src/app_input.rs:15` `BrowserApp::handle_scroll(delta, at_x, at_y)`：
  - **有光标位置** `at_x`/`at_y`（滚轮发生视口坐标）。
  - Ctrl+滚轮 → 缩放；否则调 `apply_page_scroll_delta(tab_id, delta_x, delta_y)`——**丢弃 `at_x`/`at_y`**，仅传 delta。
- `apps/browser/src/page_scroll.rs:228` `compute_page_scroll_layout(content_x,y,w,h, doc_w,h, scale)`：
  - **纯文档级**——视口 vs 文档内容尺寸算 `max_scroll_x/y`，无 per-element 溢出容器几何。
  - `primitives_content_height`（:208）从 `RenderPrimitives` 算文档总高（css-overflow-3 §scrollable）。
- `TabScrollState`（`app_types.rs`）：per-tab 文档滚动偏移 `{scroll_x, scroll_y}`，无 per-element 滚动状态。

### 1.2 IPC 协议（共享面）

- `crates/protocol/src/message.rs:452` `ScrollEventParams { delta_x, delta_y }`——**只有 delta，无光标位置/目标元素**。
- `ScrollEvent(ScrollEventParams)` browser→renderer（:206）。

### 1.3 渲染进程（zero-web 流）— ⚠️ 主路径不可达

- `apps/renderer/src/main.rs:1638` `handle_scroll_event(params: ScrollEventParams)`：
  - R3253：注入 `script_user_scroll(dx, dy)` → `__zw_user_scroll` 更新 `_winScroll` + 派文档级 'scroll'。
  - **⚠️ 主路径不可达**：renderer 仅在收到 `IpcMessageKind::ScrollEvent`（:1697）时调用，但 **browser 从不发此 IPC**（`apply_page_scroll_delta` 本地处理滚动，process.rs:795 的 send 仅在 `#[test]` harness）。multi-process（默认）+ single-process 均如此。
  - **结论**：R3253 的用户滚动 'scroll' 派发**生产路径失效**——页面 JS 永不观察用户滚轮。这是高优先级真 gap（见 §0 缺口 1）。
  - **修复方向**：browser `apply_page_scroll_delta`（或 `handle_scroll`）需向页面 JS 派发 'scroll'——multi-process 经发 ScrollEvent IPC（激活既有 renderer 路径），single-process 经 in-process tab_worker 直接注入 `__zw_user_scroll`。

### 1.4 布局引擎（渲染流域）

- `crates/layout-engine/src/engine.rs:1464-1471` `LayoutResult` per-element：
  - `overflow_x`/`overflow_y`：clip 模式（Visible/Hidden/Clip/Auto/Scroll，`convert_overflow_to_clip`）。
  - `scroll_x`/`scroll_y`：**硬编码 `0.0`**——布局不追踪滚动位置，视觉滚动从未应用。
  - `overflow_clip_margin_box`/`overflow_clip_margin_length`：裁剪边距盒。
- 布局产生裁剪（`overflow: hidden` 视觉裁剪生效），但 **`overflow: auto|scroll` 不产生可滚动表面**（无滚动条、无滚动交互、内容溢出处理 = 裁剪等同 hidden）。

### 1.5 engine shim（zero-web 流）

- `part01.js` `_zwFireScroll`（R3051）：程序化滚动（`scrollTop`/`scrollLeft` setter / `scrollTo`/`scrollBy`/`scrollIntoView`）更新 `_scrollOffsets[key]` + 派 'scroll' 事件。
- `__zw_user_scroll`（R3253）：用户滚动更新 `_winScroll`（文档级）+ 派 'scroll'。
- **缺口**：程序化 `element.scrollTop` 仅更 JS 态，视觉不滚动（`_scrollOffsets` 不反映到渲染）。

### 1.6 可复用基础设施

- **元素命中测试**：`crates/engine/src/element_from_point.rs` `ElementFromPointCache`（共享 `Arc`，`document.elementFromPoint` R2924 用）+ browser `tab_worker.rs:185` 注入。视口 (x,y) → 最深命中元素 NodeId + 选择器。
- **`RenderPrimitives` 几何**：每个图元含盒坐标，供命中测试 + 溢出容器识别。
- **`script_user_scroll`** / `script_*` 注入模式：renderer 已建立的 JS 注入路径。

---

## 2. 目标 / 非目标

### 目标

1. **G1（视觉）**：`overflow: auto|scroll` 容器产生可滚动表面——内容溢出时容器内滚动（不撑开文档），渲染正确裁剪 + 偏移。
2. **G2（交互）**：用户在子容器上滚轮 → 命中可滚动祖先容器 → 容器内滚动（视觉 + JS 可观察 `scrollTop` 变化 + 派 'scroll' 事件到容器）。
3. **G3（程序化一致）**：`element.scrollTop = N` / `scrollTo` 既更 JS 态**又**触发视觉重渲染（闭合 R3051 限制：程序化滚动视觉生效）。
4. **G4（JS 可观察）**：容器 `scroll` 事件 listener 触发；`scrollTop`/`scrollLeft`/`scrollWidth`/`scrollHeight` 反映真实滚动状态。

### 非目标（本轮 defer）

- 滚动条 UI（scrollbar painting）——browser `page_scroll.rs` 已有文档级滚动条几何（`scrollbar_geometry`/`push_scrollbar_fills`），元素级滚动条 UI 为后续。
- `scroll-snap` / `scroll-behavior: smooth`（CSSOM View 平滑滚动）——后续。
- 嵌套滚动容器冒泡（滚到边界 delta 传给祖先容器）——后续，先单容器。
- `IntersectionObserver` 的滚动容器根（root 为元素而非视口）——后续。

---

## 3. 设计

### 3.1 数据模型：per-element 滚动状态

**新增**：browser per-tab `ElementScrollState`（`HashMap<NodeId, (scroll_x, scroll_y)>`）+ renderer 镜像。

- 滚动状态由 **renderer 拥有**（它持 live layout + 命中测试），browser 仅转发输入。
- 或由 **browser 拥有**（与 `TabScrollState` 同层，renderer 接收已算好的偏移）——**待实现时定**（见 §5 决策点 DP-1）。

### 3.2 layout-engine 暴露面（渲染流域协调点）

layout-engine 需向 renderer/browser 暴露**可滚动容器几何**：

- 每个盒的：`overflow` clip 模式（已有 `overflow_x`/`overflow_y`）、**内容溢出尺寸**（`scroll_width`/`scroll_height` = 内容盒 vs padding 盒，当前未算）、**client 盒**（padding 盒，已有）。
- 当前 `LayoutResult.scroll_x`/`scroll_y` 硬编码 `0.0`——改为由 renderer 写回实际滚动偏移（layout 不算滚动，只提供可滚动性 + 几何）。

**这是渲染流域改动**——layout-engine 属渲染流（run-rules §9）。实现此切片前需与渲染流协调，或用户点名跨流域推进。

### 3.3 IPC 扩展（共享面）

**新增字段**（非破坏性，向后兼容）：

```rust
pub struct ScrollEventParams {
    pub delta_x: f32,
    pub delta_y: f32,
    // R3293（拟）：光标视口坐标，供 renderer 命中可滚动容器。
    pub cursor_x: f32,  // 默认 0.0（向后兼容文档级滚动）
    pub cursor_y: f32,
}
```

- browser `handle_scroll` 传 `at_x`/`at_y`（已有，当前丢弃）。
- renderer `handle_scroll_event` 用 cursor 命中可滚动祖先容器；无命中（光标在无可滚动容器区域）→ 文档级滚动（现状）。

### 3.4 renderer 元素滚动流程（G2 交互）

```
handle_scroll_event(delta, cursor_x, cursor_y):
  1. 命中测试：cursor → ElementFromPointCache → 最深命中元素 NodeId
  2. 沿祖先链找最近可滚动容器（overflow_x/y 为 Auto/Scroll 且 scrollWidth>clientWidth）
     - 无 → 文档级滚动（现状 R3253 路径）
     - 有 → 元素滚动：
       a. apply delta 到 ElementScrollState[container]（clamp 到 [0, max_scroll]）
       b. 视觉：标记 container 子树需重绘（按 scroll 偏移裁剪 + 平移子内容）
       c. JS：注入 __zw_element_scroll(container_selector, scroll_x, scroll_y)
          → 更新 _scrollOffsets[key] + 派 'scroll' 事件到 container
```

### 3.5 程序化滚动视觉生效（G3）

`element.scrollTop = N`（engine shim setter，R3051 仅更 JS 态）→ 经 DomMutation 回注 renderer → renderer 更新 `ElementScrollState[container]` + 触发视觉重绘。

- 复用既有 `apply_dom_mutations` 回注路径 + R3108（native 写触发重渲染）模式。

### 3.6 JS API 一致性（G4）

shim `_scrollOffsets[key]` 既由程序化 setter（R3051）也由 `__zw_element_scroll`（renderer 用户滚动）更新——单一真源。`scrollTop`/`scrollLeft` getter 读 `_scrollOffsets`（现状）。`scrollWidth`/`scrollHeight` 需 host 回调读 layout 内容溢出尺寸（新增 `__zw_get_scroll_size` 或复用 gBCR 路径）。

---

## 4. 切片计划（每片 kill-switch + make test 零回归）

**重排优先级**：缺口 1（用户滚动 'scroll' 主路径派发）为最高优先级纯 zero-web 自主面，提前为首刀。

| 切片 | 范围 | 流域 | 风险 | 验证 |
|------|------|------|------|------|
| **S0** | **闭合缺口 1**：browser 用户滚动 → 页面 JS 'scroll' 派发主路径。multi-process：`apply_page_scroll_delta` 发 `ScrollEvent` IPC（激活既有 renderer R3253 路径）；single-process：in-process tab_worker 注入 `__zw_user_scroll`。kill-switch `ZW_USER_SCROLL_EVENT`。 | zero-web（browser + renderer + engine） | 🟢 低（激活既有 hook，零新算法） | make test + browser 滚动集成测试 |
| **S1** | IPC `ScrollEventParams` +cursor_x/y（默认 0.0 向后兼容）+ browser `handle_scroll` 传光标 | 共享面（protocol）+ zero-web（browser） | 🟢 低（纯增字段） | make test + IPC 序列化测试 |
| **S2** | renderer `handle_scroll_event` 用 cursor 命中**文档级**滚动（验证光标坐标链路通，不改滚动行为） | zero-web（renderer） | 🟢 低 | make test |
| **S3** | layout-engine 暴露 per-element 可滚动几何（`scroll_width`/`scroll_height` + 可滚动性判定） | **渲染流域** | 🟡 中（layout 数据面） | make test + make reftest（溢出容器几何） |
| **S4** | renderer 命中可滚动祖先 + `ElementScrollState` + 元素滚动视觉（裁剪 + 平移） | zero-web（renderer）+ 渲染（视觉） | 🟡 中 | make reftest + product-smoke |
| **S5** | JS 桥：`__zw_element_scroll` 更新 `_scrollOffsets` + 派元素 'scroll' 事件 | zero-web（engine shim） | 🟢 低 | engine lib 测试 |
| **S6** | 程序化滚动视觉生效（`scrollTop` setter → DomMutation → renderer 重绘） | zero-web（engine→renderer） | 🟡 中 | make test + reftest |
| **S7** | `scrollWidth`/`scrollHeight` host 回调 + JS 一致性 | zero-web | 🟢 低 | engine lib 测试 |

**里程碑**：**S0 = M0（用户滚动 'scroll' 主路径，最高优先级，纯 zero-web）**；S1–S2 = M1（IPC 光标链路）；S3 = M2（几何暴露，渲染流域协调点）；S4–S5 = M3（元素滚动主体）；S6–S7 = M4（程序化一致 + JS 完整）。

---

## 5. 决策点（待定）

- **DP-1**：滚动状态 owner = renderer 还是 browser？
  - renderer 持 live layout + 命中测试 → owner=renderer 更自然（browser 仅转发输入）。
  - browser 持 `TabScrollState`（文档级）→ 一致性 owner=browser。
  - **倾向 renderer**（命中测试 + layout 在 renderer，避免 IPC 往返算几何）。
- **DP-2**：S2（layout 几何暴露）是渲染流域改动——自主推进还是等渲染流协调？
  - run-rules §9：layout-engine = 渲染流域，zero-web 流不硬解。
  - **倾向**：S0/S1/S4（纯 zero-web）先自主 land；S2/S3/S5 标渲染流域协调点，碰头信号记 master.md，等渲染流或用户点名。
- **DP-3**：视觉滚动实现——renderer 重绘时按 `ElementScrollState` 平移子内容 + 裁剪到容器 padding 盒。复用既有裁剪路径（overflow: hidden 已裁剪）+ 新增偏移。

---

## 6. 风险

| 风险 | 级别 | 缓解 |
|------|------|------|
| **跨流域协调**（layout-engine = 渲染流域） | 🟡 中 | S2 标协调点，zero-web 先 land S0/S1/S4，S2/S3/S5 碰头 |
| **视觉滚动正确性**（裁剪 + 平移 + 滚动条） | 🟡 中 | 每片 reftest + product-smoke 守，kill-switch `ZW_ELEMENT_SCROLL` |
| **性能**（per-element 滚动状态 + 命中测试） | 🟢 低 | 复用 ElementFromPointCache（已共享 Arc，命中测试开销已验证） |
| **程序化 vs 用户滚动状态一致** | 🟡 中 | 单一真源 `_scrollOffsets`（程序化 + 用户都写它） |
| **向后兼容**（IPC +cursor 字段） | 🟢 低 | 默认 0.0，文档级滚动路径不变 |

---

## 7. 验证计划

- **每切片**：`make test` 零回归 + 相关 crate clippy/fmt。
- **S2/S3/S5**（涉渲染）：`make reftest`（溢出容器 reftest）+ `make product-smoke`（welcome 无 overflow 容器故应中性）+ `make product-smoke-legacy`（legacy fixture 含 form-controls scroll 容器，struct-check 守）。
- **集成测试**：`overflow: scroll` 容器 + 内联 `<script>` 注册 'scroll' listener + 程序化 `scrollTop` + 用户滚动（wheel）全管线。
- **WPT**：css-overflow / cssom-view 滚动相关用例（`element.scrollTop`/`scrollLeft`/`scrollWidth`/`scrollHeight` round-trip）。

---

## 8. 当前结论

本 RFC 基于直接代码核实（非 agent 扫描），映射了当前滚动架构的完整数据流。**关键发现**：R3253 声称的「用户滚动 'scroll' 派发」**主路径不可达**（browser 不发 ScrollEvent IPC，renderer handle_scroll_event 仅为 #[test] 可达）——这是最高优先级真 gap，纯 zero-web 自主面。四层缺口（用户滚动派发 / 视觉滚动 / 元素 scroll / 程序化一致）均明确，切片计划首刀 S0 = 闭合用户滚动 'scroll' 主路径派发。

**下一步**：**S0（用户滚动 'scroll' 主路径派发，纯 zero-web，🟢 低风险，最高优先级）**可自主 land 作为首刀；S3 几何暴露为渲染流域协调点，碰头信号记 master.md。
