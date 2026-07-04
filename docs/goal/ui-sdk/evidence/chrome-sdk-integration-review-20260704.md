# `zero-browser-chrome` 接入 UI-SDK 方案深度审查报告

> **摘要**
>
> **审查范围**：`browser-ui/chrome` 全 crate（21 文件 ~4153 行）接入通用 UI SDK 的方案——覆盖 SDK 设计符合性、最佳实践、与手绘 chrome 的一致性、实现缺陷与可靠性。
>
> **关键发现**：共发现 **16** 个问题（高 **4** / 中 **8** / 低 **4**）。最高优先级是 **SDK chrome 与手绘 chrome 的 chrome 高度差 12-28px**——SDK 把 toolbar 高度做到 36（vs 手绘 44），导致 `DesktopBrowserShell::layout()` 写死的 `top_h=104` 与 SDK widget 实际 paint 出的总高（~96）**不自洽**，破坏了 `ShellLayout` 这个公开契约。
>
> **验证状态**：已对 4 个高优先级问题做了路径推演与代码交叉验证。

## 审查上下文

| 字段 | 内容 |
|------|------|
| **审查对象** | `browser-ui/chrome/`（SDK 接入方案） |
| **审查维度** | SDK 设计符合性 / 最佳实践 / 与原始界面一致性 / 实现缺陷 / 可靠性 / API 契约 |
| **代码版本** | HEAD（2026-07-04，DC-14 进行中，`sdk-chrome` feature 默认开） |
| **SDK 参考** | `ui/runtime/src/host.rs` / `ui/core/src/widget.rs` / `ui/adapters/webview/` / `ui/adapters/render-foundation/` |
| **手绘参考** | `apps/browser/src/{layout.rs, colors.rs, app_render*.rs, tab_chrome.rs, app_platform.rs}` |

## 总体评估

整体方案的**架构方向是正确的**，且符合 spec §8.4.1A / DC-1 / DC-12 的核心约束：

- chrome crate 是浏览器与 SDK 唯一耦合点（spec 约束 3 / DC-1）✅
- 通过 `WidgetSpec` 声明树 + `WidgetHost::register` 工厂模式接入 SDK ✅
- 通过 `chrome_alias_token` 维护「业务色名 → semantic token」映射，core 保持浏览器无关 ✅
- `BrowserChromeModel` 单向数据流（spec FR-003），shell 只读消费 ✅
- 三 shell 共享模型 + 跨 shell 稳定 WidgetId（DC-12）✅
- i18n 浏览器文案集中在 chrome crate（spec FR-013）✅
- DC-14 替换式迁移通过 `host.rect_of(ID_VIEWPORT)` 暴露 SDK chrome 布局的视口 rect，方向正确 ✅

**主要问题集中在两个层面**：
1. **几何契约自洽性**：`ShellLayout::layout()` 返回的固定高度与 `register_chrome_factories` 注册的 widget 实际 paint 出的高度不自洽。
2. **行为缺失**：所有 widget 的 `event` 永远返回 `EventResult::Ignored`，使 chrome 成为纯绘制层，无法消费指针事件——与 spec §8.4.1A「不得绕过 ui-runtime 的事件管线」、§8.4.1B 手势 arena 仲裁存在张力。

---

## 问题清单

### 高优先级（Critical）

#### 1. [实现缺陷 + API 契约] `ShellLayout` 与 widget 实际高度不自洽

- **位置**：`browser-ui/chrome/src/shell.rs:226-238`（`DesktopBrowserShell::layout`）、`shell.rs:304-316`（`TabletBrowserShell::layout`）、对比 `render.rs:213/541/935/1155` 的 widget 几何常量
- **置信度**：0.95
- **验证状态**：✅ 已验证
- **描述**：`ShellLayout` 是 SDK chrome 对外暴露的公开契约（`AdaptiveChromeResult.layout`），调用方据此定位页面内容。但 `layout()` 的固定高度与 widget 实际 paint 的高度**对不上**：

  | Shell | `layout()` 写死 `top_h` | 实际 widget 高度 | 差值 |
  |-------|------|------|------|
  | Desktop | 104（`shell.rs:229`） | TAB_STRIP_HEIGHT(40) + ADDRESS_BAR_HEIGHT(32) + BOOKMARKS_BAR_HEIGHT(28) = **100**（无书签时 72） | **4-32px** |
  | Tablet | 76（`shell.rs:307`） | TAB_STRIP_HEIGHT(40) + ADDRESS_BAR_HEIGHT(32) = **72** | **4px** |
  | Phone | 48（`shell.rs:373`） | AddressBar widget 实际 layout 占 max_width × NAV_BAR_HEIGHT(44) | **4px** |

  Desktop 的 `ADDRESS_BAR_HEIGHT` 在 `render.rs:213` 写的是 `44.0`（注释「= ADDRESS_BAR_HEIGHT」），但 `AddressBarWidget::layout` 返回的 height 是 `ADDRESS_BAR_HEIGHT.clamp(...)` = **32**（render.rs:1155 的常量是 `ADDRESS_BAR_HEIGHT: f32 = 32.0`）——这里有两个**同名但不同值**的 `ADDRESS_BAR_HEIGHT` 常量，是 bug 之源。

- **触发条件**：任何使用 `AdaptiveBrowserChrome::build().layout` 的调用方都会拿到与 SDK 实际布局**不同**的 viewport rect。
- **影响**：
  - `ShellLayout` 公开契约不可信，调用方据此算出的页面/WebView 几何会偏移 4-32px。
  - `tab_top=36` 与 `tab_top=40` 的注释错误暴露 layout 函数作者与 widget 作者用的是不同的几何 mental model。
  - 与手绘 chrome（`TOOLBAR_HEIGHT=84`，`apps/browser/src/layout.rs:42`）差距更大——`top_h=104` 既不等于 SDK 实际（100），也不等于手绘（84/112）。
- **建议修复**：把 `layout()` 的硬编码改为「与 widget 几何常量同步」+ 消除 `ADDRESS_BAR_HEIGHT` 同名歧义（`render.rs:213` 的注释常量值 44 应改名 `TOOLBAR_ROW_HEIGHT`，与 `render.rs:1155` 的实际地址栏 pill 高 32 区分）。

---

#### 2. [实现缺陷] PhoneBrowserShell 的 `ID_ADDRESS_BAR` 节点 component 名混用

- **位置**：`browser-ui/chrome/src/shell.rs:332-340`
- **置信度**：0.9
- **验证状态**：✅ 已验证
- **描述**：Phone shell 的顶部地址栏节点声明把 `component` 设为 `"browser.AddressBar"`，但 desktop/tablet 的 `browser.AddressBar` 注册的是 `AddressBarWidget`（render.rs:1303），它会画自己的 pill border + 文本。后果：
  - Phone 顶部行被错位的 `AddressBarWidget` 替代为 32px 高的 pill；
  - 子节点 `browser.SecurityBadge` 因父 widget 占满 max_width × 32，画在父 pill 内部而非独立 chrome 行；
  - Phone 顶部行的实际高度被锁死为 32，而非 `layout()` 返回的 48（`shell.rs:373`）。

- **影响**：Phone shell 在生产接线（HarmonyOS/Android adapter）下顶部地址栏与 SecurityBadge 视觉布局完全错位；DC-15 移动端首帧交付会受此影响。
- **建议修复**：Phone 顶部行应使用**专用容器 component**（如 `browser.PhoneTopBar`），注册一个真正的容器 widget 或让 host 经 `props.layout="row"` 处理（不注册 widget）。

---

#### 3. [API 契约 + SDK 设计符合性] 所有 chrome widget 的 `event` 永远 `Ignored`

- **位置**：`browser-ui/chrome/src/render.rs`（9 个 widget 的 `event`）
- **置信度**：0.85
- **验证状态**：✅ 已验证
- **描述**：spec §8.4.1A 硬约束要求所有 chrome 组件不得绕过 `ui-runtime` 的事件、焦点、IME 和无障碍管线。当前所有 widget 的 `event` 都是 `EventResult::Ignored`，且没有 `focusable()` 覆写（默认 `false`）。这导致：
  - 点击地址栏无法聚焦；
  - 点击 tab 不切换、点击 nav 按钮不导航、点击书签不跳转、点击菜单不弹出；
  - Tab/Shift-Tab 焦点遍历跳过整个 chrome；
  - a11y 树里没有任何 FOCUSABLE 节点；
  - 与 §8.4.1B 手势 arena 仲裁契约张力（chrome 不消费任何事件，arena 永远把事件转发给 WebView）。

- **影响**：chrome 在 SDK 路径下是纯展示层，所有交互能力丢失；替换式迁移完成后（手绘 chrome 移除），浏览器将**无法点击任何 chrome 元素**。
- **建议修复**：每个可交互 widget 应覆写 `focusable(&self) -> bool { true }`，在 `event` 里识别 `UiEvent::PointerDown/Up` → 返回 `EventResult::Emit(action)`。

---

#### 4. [实现缺陷] `BrowserChromeModel::from_shell` 把 `page_load.fraction` 永远设为 `None`

- **位置**：`browser-ui/chrome/src/chrome_model.rs:105-108`
- **置信度**：0.9
- **验证状态**：✅ 已验证
- **描述**：

  ```rust
  model.page_load = PageLoadIndicator {
      loading: at.is_loading(),
      fraction: None,
  };
  ```

  `fraction` 硬编码为 `None`，意味着 `PageLoadIndicator::build_indicator()` 永远返回 `indeterminate()`。同时三个 shell 的 build 函数都**没有构造 PageLoadIndicator 节点**——加载进度在 SDK 路径下根本不显示。

- **影响**：
  - DC-14 像素 parity 永远无法达到 0% diff（页面加载时进度条缺失）；
  - 用户失去加载状态视觉反馈。

- **建议修复**：
  1. `from_shell` 从 `browser-shell` 的 active tab 提取加载进度（如 `at.load_progress()` 若有），填充 `fraction`；
  2. 在三个 shell 的 build 函数里构造 `browser.PageLoadIndicator` 节点；
  3. 注册 `browser.PageLoadIndicator` 工厂，paint 2px 进度条。

---

### 中优先级（Major）

#### 5. [实现缺陷 + 一致性] `BrowserChromeModel::from_shell` 的 Wayland 窗口控制宽度检测不正确

- **位置**：`browser-ui/chrome/src/chrome_model.rs:129`
- **置信度**：0.85
- **验证状态**：✅ 已验证（代码注释自承）
- **描述**：手绘 chrome 的判断是 `is_wayland() || cfg!(windows)`，SDK 层只用 `cfg!(target_os = "windows")`。Wayland 下 `window_controls_width = 0`，tab strip 几何错误。
- **建议修复**：把 `window_controls_width` 改为 `BrowserChromeModel` 的外部注入字段（由 apps/browser 根据 `uses_custom_window_controls()` 填入），或在 SDK 层做运行时检测（`std::env::var("WAYLAND_DISPLAY")`）。

---

#### 6. [API 契约] `register_chrome_factories` 全量注册 + `with_webview` 先全量再覆盖

- **位置**：`browser-ui/chrome/src/render.rs:1294-1370`
- **置信度**：0.7
- **验证状态**：未验证
- **描述**：`register_chrome_factories_with_webview` 先全量注册再覆盖 `PageViewportFrame`，产生两次工厂注册（虽然 HashMap 覆盖是 O(1)，但语义上不优雅）。所有闭包 `move` 捕获 `SemanticTokens` / `ChromeTabColors` 的 Copy 副本。
- **建议修复**：抽出 `_inner` 不含 PageViewportFrame 的版本；文档化 component namespace 占用情况。

---

#### 7. [实现缺陷 + 一致性] `BookmarksBarWidget` 文本截断用字符数近似

- **位置**：`browser-ui/chrome/src/render.rs:989-1001`
- **置信度**：0.7
- **验证状态**：未验证
- **描述**：`bm.chars().count() as f32 * 7.5` 是粗糙估算——CJK 字符、emoji、宽字符的 advance 远不止 7.5px。中文书签会严重重叠。手绘 chrome 用的是真实 glyph 度量。
- **建议修复**：用 SDK 的 font backend 做真实文本测量。

---

#### 8. [可靠性] chrome widget 缺少 `semantics` 实现

- **位置**：`browser-ui/chrome/src/render.rs`（所有 widget）
- **置信度**：0.8
- **验证状态**：未验证
- **描述**：SDK Widget trait 契约要求 semantics push 自描述 `SemanticsNode`。当前所有 chrome widget 都是空 `semantics`，chrome 在 a11y 树里完全缺失。违反 spec §8.4.1A「不得绕过 ui-runtime 的无障碍管线」与 DC-8 a11y 后端桥接契约。
- **建议修复**：每个 widget 至少 push 一个 `SemanticsNode` 带 `label`（来自 i18n catalog）+ `FOCUSABLE` flag + `role`。

---

#### 9. [实现缺陷] `PhoneBrowserShell::build` 的 NavigationButtons 用文本占位，且非 phone 形态

- **位置**：`browser-ui/chrome/src/shell.rs:346-354`
- **置信度**：0.8
- **验证状态**：未验证
- **描述**：Phone 底部 NavigationButtons widget 的宽度是 desktop toolbar 行内 nav 段宽（174px），不是 phone 底部导航栏的合理形态。`nav_status_label` 文本被 widget 静默丢弃。
- **建议修复**：Phone 底部导航应有专用 widget（`PhoneBottomNavWidget`），3 按钮均匀分布，宽度占满。

---

#### 10. [可靠性 + 一致性] `security_from_url` 派生逻辑过于简化

- **位置**：`browser-ui/chrome/src/chrome_model.rs:151-156`
- **置信度**：0.7
- **验证状态**：未验证
- **描述**：永远不返回 Mixed/Dangerous；`about:blank`/`chrome://`/`file://`/`data:` 都判为 Secure。手绘 chrome 的安全状态来自浏览器引擎（HTTPS 证书验证 + mixed content 检测 + safe-browsing）。
- **建议修复**：`browser-shell` 应扩展 `Tab::security_state()` 接口，由 net/security crate 填充；`from_shell` 直接读，不再启发式派生。

---

#### 11. [实现缺陷] `FindBar` shell 节点位置错位

- **位置**：`browser-ui/chrome/src/shell.rs:210-222`、`render.rs:1055-1085`
- **置信度**：0.7
- **验证状态**：未验证
- **描述**：FindBar 在 column 容器里是 viewport 的后续兄弟——但手绘 chrome 的 FindBar 是覆盖在 viewport 左下角的浮层。SDK 把它放在 column 末尾会挤压 viewport 高度。FindBarWidget 是静态文本条，无任何交互。
- **建议修复**：FindBar 应用 Stack 容器声明为 overlay；FindBarWidget 应是复合控件（TextInput + 计数 + 3 按钮）。

---

#### 12. [API 契约 + 可靠性] `render_chrome_via_sdk_with_webview_surface` 8 个参数

- **位置**：`browser-ui/chrome/src/sdk_render.rs:111-124`
- **置信度**：0.6
- **验证状态**：未验证
- **描述**：8 个参数（已标 `#[allow(clippy::too_many_arguments)]`）。`tokens` + `scheme` 冗余；`webview_surface` 的 `Option<(u64, RenderPrimitives, Option<ImageCache>)>` 三层嵌套。
- **建议修复**：引入 `ChromeRenderConfig` builder struct。

---

### 低优先级（Minor）

#### 13. [最佳实践] chrome crate README 声称「86 测」，但实际测试覆盖不均

- **位置**：`browser-ui/chrome/README.md:71`
- **置信度**：0.6
- **验证状态**：未验证
- **描述**：测试覆盖声明树 build 和 chrome_model 投影，但**没有测试 widget event 行为、ShellLayout 与 widget 实际高度的自洽性、from_shell 的 security_from_url 边界、phone shell 真实渲染快照**。
- **建议修复**：补充 (a) ShellLayout 与 host.rect_of 一致性断言；(b) phone shell snapshot 测试；(c) security_from_url 边界用例。

---

#### 14. [最佳实践] i18n 文案未与 widget paint 路径完整对接

- **位置**：`browser-ui/chrome/src/i18n/mod.rs` vs `render.rs`
- **置信度**：0.7
- **验证状态**：未验证
- **描述**：i18n 模块定义了完整的 message id 常量，但 widget paint 路径里几乎不用：FindBarWidget 用字面量 `"Find"`、AddressBarWidget 用字面量 `"Search or enter URL..."`、SecurityBadge tooltip message id 没被任何 widget 消费。违反 spec FR-013 的「production strict mode 禁止硬编码可见字符串」。
- **建议修复**：widget paint 时所有可见文案应经 `crate::i18n::resolve(id, params)` 解析。

---

#### 15. [实现缺陷] `DesktopBrowserShell::build` 的 toolbar 声明顺序无测试守护

- **位置**：`browser-ui/chrome/src/shell.rs:193-195`
- **置信度**：0.6
- **验证状态**：未验证
- **描述**：声明顺序（tab_strip → toolbar → bookmarks → viewport）已修正，但**没有测试守护**——未来重构可能再次 swap。
- **建议修复**：加测试断言 `root.children[0].id == ID_TAB_STRIP`、`root.children[1].id == ID_TOOLBAR`。

---

#### 16. [最佳实践] `ChromePanel::layout` fallback 默认值不统一

- **位置**：`browser-ui/chrome/src/render.rs:184` 等多处
- **置信度**：0.5
- **验证状态**：未验证
- **描述**：多个 widget 的 `paint` 用 `ctx.clip.map(...).unwrap_or_else(|| Size::new(XXX, YYY))`，fallback 值（400.0/1280.0/42 等）不统一。
- **建议修复**：统一 fallback 策略——要么 panic on None（debug）/log + skip（release），要么用 widget 自己的 layout 返回的 Size。

---

## 统计总览

| 维度 | 高 | 中 | 低 | 合计 |
|------|----|----|----|----|
| SDK 设计符合性 | 1（#3）| 1（#6）| 0 | 2 |
| 最佳实践 | 0 | 1（#12）| 3（#13/#14/#16）| 4 |
| 与原始界面一致性 | 1（#1 关联）| 4（#5/#7/#10/#11）| 1（#15）| 6 |
| 实现缺陷 | 3（#1/#2/#4）| 2（#8/#9）| 0 | 5 |
| 可靠性 | 0 | 2 | 0 | 2 |
| API 契约 | 1 | 1 | 0 | 2 |
| **合计（去重）** | **4** | **8** | **4** | **16** |

## 二次验证结果

| # | 问题 | 原始级别 | 验证结论 | 说明 |
|---|------|---------|---------|------|
| 1 | ShellLayout 与 widget 高度不自洽 | 高 | ✅ 确认 | widget 常量与 layout 函数硬编码值逐行核对，差值 4-32px 确凿；注释里 `tab strip(36)` 与实际 `TAB_STRIP_HEIGHT=40` 直接矛盾 |
| 2 | PhoneBrowserShell ID_ADDRESS_BAR 组件名混用 | 高 | ✅ 确认 | Phone shell 用 `browser.AddressBar` 作容器，但工厂注册的是 32px 高的 AddressBarWidget，逻辑冲突 |
| 3 | widget event 全 Ignored | 高 | ✅ 确认 | 9 个 widget 实现逐个核对，均无 focusable/event 覆写；与 spec §8.4.1A 硬约束直接冲突 |
| 4 | page_load.fraction 永远 None | 高 | ✅ 确认 | from_shell 硬编码 + shell build 不构造 PageLoadIndicator 节点，双重缺失 |

## 修复建议优先级

| 优先级 | 问题 | 建议动作 | 预估改动量 |
|--------|------|---------|-----------|
| **P0（替换式迁移完成前必须）** | #1 ShellLayout 不自洽 | 改 layout() 基于 host.rect_of 或对齐常量 + 消除 ADDRESS_BAR_HEIGHT 歧义 | 小 |
| **P0** | #3 widget event 全 Ignored | 为 9 个 widget 实现 focusable + event | 大 |
| **P0** | #4 PageLoadIndicator 缺失 | from_shell 填充 fraction + shell build 构造节点 + 注册工厂 | 中 |
| **P1（本迭代）** | #2 Phone shell 组件名混用 | 引入 PhoneTopBar 专用 component | 小 |
| **P1** | #5 Wayland 检测 | 把 window_controls_width 改为外部注入字段 | 小 |
| **P1** | #8 widget semantics 缺失 | 为 9 个 widget 实现 semantics | 中 |
| **P1** | #10 security_from_url 简化 | browser-shell 扩展 security_state 接口 | 中（跨 crate）|
| **P2（后续跟进）** | #7 BookmarksBar 文本测量 | 用 font backend 真实测量 | 小 |
| **P2** | #11 FindBar overlay | 改用 Stack 容器 | 小 |
| **P2** | #14 i18n 文案对接 | widget paint 路径接 catalog | 中 |
| **P2** | #6/#9/#12/#13/#15/#16 | 工程改进 | 各小 |

## 结论

chrome crate 接入 UI-SDK 的**架构方向是正确的**（声明树 + 工厂注册 + BrowserChromeModel 单向数据流 + chrome 别名 token 映射 + i18n 集中），符合 spec §8.4.1A / DC-1 / DC-12 的核心约束。但**实现深度不够**：

1. **几何契约层**（问题 #1/#2）有具体的契约不一致 bug，`ShellLayout` 公开契约不可信——这是替换式迁移的**前置阻塞**。
2. **交互能力层**（问题 #3）整体缺失，chrome 在 SDK 路径下是纯绘制层——这是替换式迁移**完成前必须闭合**的工作。
3. **视觉一致性层**（问题 #4/#5/#7/#9/#10/#11）多处与手绘 chrome 不一致——DC-14 像素 parity 难以达到 0%。
4. **a11y / i18n 合规层**（问题 #8/#14）落地不完整——spec FR-013 strict mode 与 §8.4.1A a11y 管线约束未满足。

**优先闭合 P0 三项**（#1/#3/#4）即可解锁替换式迁移；P1 是 spec 合规与移动端首帧的必要工作；P2 是工程改进与本地化完整度。
