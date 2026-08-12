# Spec + RFC：HTML 行为兼容开发线

**版本**：v1.1
**日期**：2026-08-12
**作者**：AI Assistant
**状态**：待确认
**依据**：[`research-html-compat-parallel-track-2026-08-12.md`](../research/research-html-compat-parallel-track-2026-08-12.md)
**ZeroUI 补充调研**：[`research-zeroui-lessons-for-html-compat-2026-08-12.md`](../research/research-zeroui-lessons-for-html-compat-2026-08-12.md)

---

## 0. 执行摘要

- **一句话目标**：建立独立的 `html-compat` 开发线，按 M0-M4 逐步闭合基础 HTML 元素的状态、焦点、事件、默认动作和自动化兼容性。
- **本期范围**：目标表单页完整验收、紧邻规范差异修复、默认动作共享化、元素族扩展、WebDriver/WPT testharness/testdriver 接入。
- **明确排除**：CSS 解析、层叠、布局精度、控件外观、字体和 GPU/CPU 像素一致性修复。
- **核心约束**：
  1. 每个功能点必须有仓内自有基础测试；WPT 仅作外部规范 Oracle，不能替代自有测试。
  2. 自有测试至少覆盖正常路径和一个失败/取消/边界路径。
  3. HTML 默认动作不得依赖页面 JavaScript 是否启用。
  4. 多进程 renderer、单进程 TabWorker 和 WebView 必须消费同一行为核心。
  5. 生产代码中的规范行为必须附对应 WHATWG/W3C 规范链接。
  6. press/release、focus、retained state 和 automation 必须引用稳定 page node identity，不得把 selector 当 identity。
- **推荐方案**：先用 M0 固化产品场景，再在 M1 修正语义；M2 将默认动作收敛到 `zero-page-runtime`；M3 按元素族扩展；M4 接入真实 WebDriver 与 WPT 交互子集。
- **首个落地步骤**：新增目标表单页完整语义场景及配套 renderer/browser 自有测试，不先修改生产逻辑。

## 1. 背景与目标

### 1.1 背景

ZeroWeb 已具备 HTML 解析、DOM、页面脚本、retained 表单状态、多进程 renderer 和基础 WebDriver。当前目标表单页的部分路径已通过 renderer 与多进程产品测试，但测试没有在同一场景中覆盖退格、Tab、checkbox、radio、reset、submit、label 激活和 JavaScript 禁用模式。

现有行为逻辑分散在：

- `crates/dom/src/focus.rs`：可聚焦元素与 Tab 顺序。
- `crates/page-runtime/src/form_control.rs`：焦点、value、selection、composition 和 dirty 状态。
- `crates/engine/src/js_dom_shim/`：JS 可观察的 DOM/事件近似实现。
- `apps/renderer/src/main.rs` 与 `page_scripts.rs`：输入路由和默认动作。
- `apps/browser/src/tab_worker.rs`：单进程路径的镜像实现。
- `apps/webdriver`：进程内 WebView 自动化，尚未驱动真实 renderer 输入链。

这使新增元素行为容易在多条宿主路径重复实现，也容易出现单元测试通过但产品路径分叉的问题。

### 1.2 目标

业务目标：

- 建立一条可长期并行推进、以 HTML 行为兼容性为唯一主责的开发线。
- 让基础 HTML 元素从“能解析、能画出”提升到“用户操作和脚本观察符合规范”。

用户目标：

- 用户能在 ZeroBrowser 中完成目标表单页声明的全部交互。
- 集成方通过 WebView 或 WebDriver 获得与 ZeroBrowser 一致的 HTML 行为。
- 开发者能用仓内快速测试定位行为回归，而不必先运行完整 WPT。

### 1.3 用户流程

1. **加载与定位**：页面完成加载，交互元素可定位、可命中。
2. **输入与编辑**：用户点击文本控件，执行输入、退格、选区替换和 IME。
3. **焦点移动**：用户点击或按 Tab/Shift+Tab，焦点和 blur/change/focus 顺序正确。
4. **控件激活**：用户操作 checkbox、radio、label 和普通按钮，状态与事件一致。
5. **表单动作**：用户 reset 或 submit，默认动作可被规范允许的事件取消。
6. **自动验收**：仓内自有测试先验证基本行为，WPT 再验证外部规范一致性。

主要异常分支：

- `keydown`、`beforeinput`、`click`、`reset` 或 `submit` 被 `preventDefault()`。
- JavaScript 被禁用。
- 元素 disabled、hidden、inert、无 form owner 或已从 DOM 移除。
- 多进程子进程未构建、启动失败或测试并行争用。

### 1.4 范围边界

在范围内：

- HTML 表单控件状态与 dirty/default 状态。
- 文本输入、删除、selection、IME、beforeinput/input/change。
- focusability、Tab/Shift+Tab、label/control 关联。
- checkbox/radio/button/reset/submit 默认动作。
- form owner、implicit submission 和导航请求。
- renderer、TabWorker、WebView 三路径一致性。
- WebDriver Element Click、Send Keys、Get Active Element、live page script query。
- WPT testharness/testdriver 的 forms/focus/input-events 子集。
- 仓内自有单元、组件、集成、多进程和 WebDriver 测试。

不在范围内：

- `css-parser`、`style-system` 的 CSS 功能开发。
- `layout-engine` 的几何与排版修复。
- `render-foundation` 的控件外观或像素一致性修复。
- 文件选择器、日期选择器等平台原生 UI。
- 完整 WebDriver 协议覆盖。
- 一次性宣称所有 HTML 标签完全兼容。

跨线规则：

- 若失败根因属于 CSS、布局或绘制，本线只提交最小复现和失败断言，并移交对应开发线。
- 若 HTML 行为需要新增 hit-test、IPC 或 frame publish 契约，本线可以修改共享契约，但必须使用独立提交并补三路径测试。

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|---|---|---|
| 业务需求 | 是 | 用户要求建立并行 HTML 兼容线 |
| 用户需求 | 是 | 目标表单页必须完整工作 |
| 功能需求 | 是 | §3 FR-001 至 FR-012 |
| 非功能需求 | 是 | §4 NFR-001 至 NFR-008 |
| 接口需求 | 是 | §5 IF-001 至 IF-005 |
| 过渡需求 | 是 | M2 将散点逻辑迁入 `page-runtime` |

### 2.1 需求优先级

| 优先级 | 含义 |
|---|---|
| 必须 | 对应里程碑不可缺失；缺失即里程碑未完成 |
| 应该 | 默认实施；只有明确证据表明不适用时可记录豁免 |
| 可以 | 不阻塞当前里程碑，可在后续元素族扩展 |

### 2.2 自有测试与 WPT 的关系

| 测试层 | 目的 | 是否可被 WPT 替代 |
|---|---|---|
| crate 单元测试 | 状态转换、边界和取消语义 | 否 |
| engine/renderer 组件测试 | JS 可观察事件与 DOM 状态 | 否 |
| integration/WebView 测试 | 跨 crate 和嵌入路径 | 否 |
| browser 多进程测试 | 真实输入、IPC、命中和帧发布 | 否 |
| WebDriver HTTP 测试 | 自动化协议和真实页面状态 | 否 |
| WPT testharness/testdriver | 外部规范 Oracle 与上游回归 | 不适用，属于补充层 |

## 3. 功能需求

### FR-001：目标表单页完整语义验收

- **描述**：系统必须在真实多进程页面链路中完成目标表单页的加载、输入、退格、Tab、IME、checkbox、radio、普通按钮、reset 和 submit 流程，并逐步暴露可断言状态。
- **优先级**：必须
- **里程碑**：M0

验收场景：

```text
场景: 完整表单交互
  假设 ZeroBrowser 通过真实 renderer 加载 form-interaction-test.html
  当测试依次输入 abc、退格、Tab 到 textarea、提交 IME、切换 checkbox/radio、点击 button/reset/submit
  那么每一步的 value、checkedness、activeElement、output 文本和导航状态均符合页面约定
  验证: apps/browser/src/tests.rs::form_fixture_complete_multiprocess_semantics

场景: 控件未就绪
  假设 renderer 尚未发布包含目标控件的 hit-test 快照
  当测试等待达到规定超时
  那么测试必须报告 step index、缺失 selector、当前阶段、URL/navigation epoch 和最后 snapshot sequence，不得静默跳过后续步骤
  验证: apps/browser/src/tests.rs::form_fixture_reports_missing_control_stage

场景: Scenario 中间断言失败
  假设 HtmlScenario 第 4 步期望错误的 value
  当执行完整场景
  那么 helper 返回第 4 步的 description、selector、expected/actual 和页面同步状态
  验证: apps/browser/src/tests/html_scenario.rs::scenario_failure_reports_exact_step_and_state
```

### FR-002：默认动作与 JavaScript 开关解耦

- **描述**：系统必须在 JavaScript 禁用时继续执行 HTML 用户代理默认动作；仅页面脚本执行和脚本 listener 被禁用。
- **优先级**：必须
- **里程碑**：M1

验收场景：

```text
场景: JavaScript 禁用时基础控件可操作
  假设页面 JavaScript 被禁用且页面包含 input、textarea、checkbox、radio 和 reset
  当用户输入、删除、切换控件并重置表单
  那么 value、checkedness、radio 互斥和默认状态恢复仍生效，页面脚本计数保持不变
  验证: tests/integration/src/html_compat.rs::default_actions_work_without_javascript

场景: JavaScript 禁用时不执行 listener
  假设控件注册了修改 output 的 input/change/reset listener，随后禁用 JavaScript
  当用户执行对应默认动作
  那么控件状态改变但 output 不被 listener 修改
  验证: apps/renderer/src/page_scripts.rs::javascript_disabled_skips_listeners_not_default_actions
```

### FR-003：文本编辑事件序列

- **描述**：文本插入、删除、替换和 IME 必须按规定顺序派发 `keydown`、`beforeinput`、状态变更、`input`；`beforeinput` 的取消必须阻止状态变更，`input` 必须不可取消。
- **优先级**：必须
- **里程碑**：M1

验收场景：

```text
场景: 普通文本插入
  假设文本控件已聚焦并记录事件日志
  当用户输入字符 A
  那么日志包含 keydown → beforeinput(insertText) → input(insertText)，且 input.cancelable=false
  验证: crates/engine/src/js_dom_bridge_tests/part15.rs::text_insert_dispatches_beforeinput_then_input

场景: beforeinput 取消插入
  假设 beforeinput listener 对 insertText 调用 preventDefault
  当用户输入字符 A
  那么 value、selection 和 dirty 状态不变，input 事件不派发
  验证: apps/renderer/src/page_scripts.rs::prevented_beforeinput_does_not_mutate_value

场景: 删除到文本起点
  假设 selectionStart=0 且 selectionEnd=0
  当用户按 Backspace
  那么 value 不变，不派发 input，selection 保持 0
  验证: crates/page-runtime/src/form_control.rs::delete_backward_at_start_is_noop

场景: IME 取消
  假设存在未提交 composition
  当平台发送 Disabled 或空 Commit
  那么 composition 临时状态清除，未提交文本不进入 value
  验证: crates/page-runtime/src/form_control.rs::cancelled_composition_does_not_commit
```

### FR-004：焦点、激活与表单动作语义

- **描述**：系统必须统一实现 focusability、顺序焦点导航、label 激活、checkbox/radio 激活、reset 和 submit 取消语义。
- **优先级**：必须
- **里程碑**：M1

验收场景：

```text
场景: Tab 顺序和焦点事件
  假设页面包含正 tabindex、自然顺序控件、disabled、hidden 和 inert 控件
  当用户连续按 Tab 和 Shift+Tab
  那么仅可顺序聚焦元素进入焦点链，blur/focusout/focus/focusin 的目标与顺序正确
  验证: crates/dom/src/focus.rs::sequential_focus_skips_non_focusable_controls

场景: keydown 取消 Tab
  假设当前控件的 keydown listener 对 Tab 调用 preventDefault
  当用户按 Tab
  那么 activeElement 不变
  验证: apps/renderer/src/main.rs::prevented_tab_keeps_focus_owner

场景: label 激活关联控件
  假设 label 包含 checkbox 或通过 for 属性关联 input
  当用户点击 label 的非控件区域
  那么关联控件获得一次激活，checkbox 仅翻转一次
  验证: tests/integration/src/html_compat.rs::label_click_activates_associated_control_once

场景: 已选 radio 重复激活
  假设 radio 已 checked
  当用户再次点击该 radio
  那么 checkedness 不变且不重复派发 change
  验证: crates/page-runtime/src/form_control.rs::checked_radio_reactivation_is_noop

场景: reset 或 submit 被取消
  假设 reset/submit listener 调用 preventDefault
  当用户激活对应按钮
  那么 reset 不恢复状态，submit 不产生导航请求
  验证: apps/renderer/src/page_scripts.rs::prevented_reset_and_submit_skip_default_actions

场景: press 与 release 之间发生轻微移动或重排
  假设 primary press 已命中可激活控件并记录稳定 pressed target
  当指针发生阈值内移动或 DOM 重排后 release
  那么 release/cancel 回投给原 pressed target，click 是否生成仍服从拖动阈值和事件取消结果
  验证: tests/integration/src/html_compat.rs::release_uses_stable_pressed_target_across_reflow
```

### FR-005：共享 HTML 默认动作核心

- **描述**：renderer、TabWorker 和 WebView 必须通过 `zero-page-runtime` 的同一动作决策与状态转换实现 HTML 默认动作；宿主只执行脚本派发、导航和帧发布副作用。
- **优先级**：必须
- **里程碑**：M2

验收场景：

```text
场景: 三路径结果一致
  假设相同 HTML、初始控件状态和输入动作序列
  当分别通过 renderer、TabWorker 和 WebView 执行
  那么最终 value、checkedness、focus owner、事件日志和导航 intent 一致
  验证: tests/integration/src/html_compat.rs::default_action_conformance_across_hosts

场景: 取消结果一致
  假设相同事件在三条路径均被 preventDefault
  当执行对应动作
  那么三条路径均不改变默认状态且不产生导航 intent
  验证: tests/integration/src/html_compat.rs::prevented_action_conformance_across_hosts
```

### FR-006：文本控件元素族

- **描述**：系统必须闭合 `input` 文本类与 `textarea` 的 live value、default value、selection、change-on-blur、IME 和约束属性基础语义。
- **优先级**：必须
- **里程碑**：M3

验收场景：

```text
场景: live value 与默认值分离
  假设 input/textarea 具有初始值
  当用户编辑 value 后调用 form.reset()
  那么 live value 恢复默认值，defaultValue 未被用户编辑污染
  验证: crates/engine/src/js_dom_bridge_tests/part15.rs::text_control_reset_restores_unpolluted_default

场景: 不适用的 selection API
  假设 input type 不支持文本选区
  当脚本读取 selectionStart 或调用不适用的 selection 方法
  那么返回值或异常符合该 input state 的规范要求，不修改 value
  验证: crates/engine/src/js_dom_bridge_tests/part15.rs::non_text_input_rejects_text_selection_operations

场景: caret、selection 与点击定位使用同一文本边界
  假设文本包含比例字体、CJK 和代理对
  当用户点击字符边界并移动 selection/caret
  那么 hit-test、caret paint 和 IME rect 消费同一边界缓存，不使用固定字符宽度重新估算
  验证: tests/integration/src/html_compat.rs::text_control_hit_caret_and_ime_share_boundaries
```

### FR-007：选择控件与表单元素族

- **描述**：系统必须闭合 checkbox、radio、select、option、button、label、fieldset、form 和 output 的状态关联、disabled 传播、reset 与 submission 基础语义。
- **优先级**：必须
- **里程碑**：M3

验收场景：

```text
场景: 表单状态与成功控件集合
  假设表单包含 text、checkbox、radio、select、textarea 和 submitter
  当提交事件未被取消
  那么 entry list 仅包含 successful controls，顺序和值符合文档序
  验证: crates/engine/src/js_dom_bridge_tests/part13.rs::form_entry_list_covers_basic_control_family

场景: disabled fieldset
  假设控件位于 disabled fieldset 内且不属于首个 legend 例外
  当用户聚焦、激活或提交表单
  那么该控件不可交互且不进入提交数据
  验证: tests/integration/src/html_compat.rs::disabled_fieldset_blocks_interaction_and_submission
```

### FR-008：导航与交互元素族

- **描述**：M3c 交付时，系统必须闭合 `a`、`area`、`details/summary`、`dialog` 和 popover 的基础激活、取消和焦点语义，不扩展其视觉样式。
- **优先级**：应该
- **里程碑**：M3

验收场景：

```text
场景: 可导航链接激活
  假设 a 元素具有可导航 href
  当 click 未被取消
  那么产生一次规范化导航 intent；hash 链接执行同文档 fragment 行为
  验证: tests/integration/src/html_compat.rs::anchor_activation_produces_single_navigation_intent

场景: click 取消导航
  假设 click listener 调用 preventDefault
  当用户激活链接
  那么 URL、history 和 navigation epoch 均不变化
  验证: tests/integration/src/html_compat.rs::prevented_anchor_click_does_not_navigate

场景: disabled 或无效交互目标
  假设 dialog/popover/command 的目标缺失、断开或状态不允许操作
  当用户激活触发元素
  那么系统执行规范要求的 no-op 或异常，不产生残留 top-layer 状态
  验证: crates/engine/src/js_dom_bridge_tests/part15.rs::invalid_interactive_target_has_no_residual_state
```

### FR-009：媒体与资源元素基础事件

- **描述**：系统可以逐步闭合 `img`、`audio`、`video`、`source` 和 `track` 的加载状态与 load/error 基础事件；本里程碑不要求媒体解码播放。
- **优先级**：可以
- **里程碑**：M3

验收场景：

```text
场景: 图片加载完成
  假设 img 资源成功解码
  当资源状态提交到页面
  那么 naturalWidth/naturalHeight 可观察且 load 仅派发一次
  验证: tests/integration/src/html_compat.rs::image_load_state_and_event_are_coherent

场景: 资源加载失败
  假设 img/source URL 无法获取或解码
  当加载失败完成
  那么派发一次 error，不派发 load，页面加载循环不挂起
  验证: tests/integration/src/html_compat.rs::resource_failure_dispatches_error_without_hang
```

### FR-010：真实页面 WebDriver 自动化

- **描述**：WebDriver 必须通过真实页面运行时执行 Element Click、Send Keys、Get Active Element 和 Execute Script，不得只在独立虚拟 DOM 中返回成功。
- **优先级**：必须
- **里程碑**：M4

验收场景：

```text
场景: WebDriver 驱动目标表单
  假设 WebDriver session 已加载目标表单页
  当客户端定位 #name、发送字符和 Tab、点击 checkbox 与 button，并查询 active element 和页面状态
  那么 WebDriver 返回值与真实 renderer/WebView 页面状态一致
  验证: apps/webdriver/tests/http_session.rs::webdriver_drives_live_form_controls

场景: 无效或过期元素引用
  假设元素不存在或引用对应节点已被 DOM 替换
  当客户端执行 click 或 send keys
  那么返回 no such element 或 stale element reference，不操作其他匹配元素
  验证: apps/webdriver/tests/http_session.rs::webdriver_rejects_missing_and_stale_element_references
```

### FR-011：WPT testharness/testdriver 子集

- **描述**：WPT runner 必须能执行已选 forms/focus/input-events testharness 用例，并将 `test_driver.click`、`send_keys` 和基础 actions 路由到 ZeroWeb 自动化接口。
- **优先级**：必须
- **里程碑**：M4

验收场景：

```text
场景: 运行受支持 WPT 子集
  假设用例仅依赖已声明支持的 testharness/testdriver API
  当 runner 执行 forms/focus/input-events 清单
  那么收集每个 subtest 的 PASS/FAIL/TIMEOUT，并以非零退出码报告失败
  验证: tests/wpt-runner/src/testharness.rs::runs_supported_html_interaction_subtests

场景: 遇到未支持 testdriver 命令
  假设用例调用尚未实现的自动化 API
  当 runner 执行用例
  那么该 subtest 明确标记 UNSUPPORTED 或 FAIL，不得伪报 PASS 或无限等待
  验证: tests/wpt-runner/src/testharness.rs::unsupported_testdriver_command_is_explicit
```

### FR-012：仓内自有基础测试资产化

- **描述**：每个新增或修复的 HTML 行为点必须拥有不依赖上游 WPT 文件的仓内自有基础测试，并登记到兼容矩阵。
- **优先级**：必须
- **里程碑**：M0-M4 全程

验收场景：

```text
场景: 功能点具备本地测试矩阵
  假设一个 HTML 行为切片准备合入
  当检查该切片的测试清单
  那么至少存在一个正常路径和一个失败/取消/边界路径自有测试；用户可见动作另有一个跨层测试
  验证: docs/goal/html-compat/test-matrix.md 对应条目 + make test

场景: 仅新增 WPT 用例
  假设变更只导入或启用 WPT，而没有仓内自有基础测试
  当执行里程碑验收
  那么该功能点不得标记完成，必须补齐自有测试或记录经用户批准的豁免
  验证: docs/goal/html-compat/test-matrix.md 的 local_unit/local_integration 字段非空
```

## 4. 非功能需求

### NFR-001：自有测试完整性

- **描述**：每个行为点至少包含 2 个仓内自有基础测试：1 个正常路径，1 个失败、取消或边界路径。用户可见交互还必须包含 1 个跨层测试。
- **测量标准**：`docs/goal/html-compat/test-matrix.md` 每行的 `local_unit` 和 `local_integration` 非空；WPT 列不能代替这两列。
- **优先级**：必须

### NFR-002：宿主一致性

- **描述**：相同初始状态和动作序列在 renderer、TabWorker 和 WebView 上必须产生相同 HTML 可观察结果。
- **测量标准**：FR-005 两个 conformance 测试通过；不允许使用宿主专属预期值豁免。
- **优先级**：必须

### NFR-003：测试确定性

- **描述**：HTML 兼容测试不得依赖外网、人工点击、未构建的陈旧兄弟二进制或测试执行顺序。
- **测量标准**：使用本地 fixture/HTTP server；多进程测试串行；测试入口先构建所需 bin；不新增 `#[ignore]`。
- **优先级**：必须

### NFR-004：性能与帧发布

- **描述**：HTML 行为修复不得破坏现有表单输入性能预算；一次用户动作不得无理由产生重复完整解析或多帧发布。
- **测量标准**：`make bench-gate` 满足现有 `form_input` 基线；value-only 场景保持既有 parse/layout/publish 预算。
- **优先级**：必须

### NFR-005：资源安全

- **描述**：测试、构建和 WPT 命令必须经 `test-guard` 或项目 Makefile 入口执行，避免 OOM 或死循环影响宿主。
- **测量标准**：实施记录中的所有测试命令均为 `make test`、`make reftest` 或显式 `target/test-guard` 包裹的 scoped 命令。
- **优先级**：必须

### NFR-006：代码质量

- **描述**：公共 API 必须有 Rust 文档；规范行为必须附规范链接；不得新增 clippy warning。
- **测量标准**：`cargo fmt --all -- --check`、workspace clippy `-D warnings` 和 `make test` 通过。
- **优先级**：必须

### NFR-007：范围隔离

- **描述**：HTML 行为切片不得用 CSS、布局或绘制特判掩盖行为缺陷。
- **测量标准**：默认禁止修改 §6.6 的渲染路径；例外必须有跨线移交记录和独立用户确认。
- **优先级**：必须

### NFR-008：可追溯性

- **描述**：每个行为点必须能从规范条款追溯到 FR、实现模块、自有测试和可选 WPT。
- **测量标准**：`test-matrix.md` 包含 `spec_link`、`fr`、`implementation`、`local_unit`、`local_integration`、`wpt` 字段。
- **优先级**：必须

## 5. 接口需求

### IF-001：共享用户动作请求

- **类型**：内部 Rust API
- **权威位置**：`crates/page-runtime`
- **规格**：

```rust
pub struct PageNodeRef {
    pub navigation_epoch: u64,
    pub document_generation: u64,
    pub node: PageNodeHandle,
}

pub struct PageNodeHandle { /* opaque; representation selected by M1 spike */ }

pub enum HtmlUserAction {
    InsertText { text: String },
    DeleteBackward,
    MoveFocus { forward: bool },
    Activate,
    Reset,
    Submit,
}

pub struct HtmlActionRequest {
    pub target: PageNodeRef,
    pub action: HtmlUserAction,
    pub shift: bool,
}

pub struct PressedTarget {
    pub node: PageNodeRef,
    pub button: PointerButton,
    pub press_position: (f32, f32),
}
```

- **错误处理**：目标不存在、generation 失配、不可交互或动作不适用时返回显式 no-op/stale reason；不得用 selector 重新匹配第一个同标签元素。
- **默认动作**：无。该接口描述待决策动作，不自行绕过事件派发。
- **交叉引用**：FR-002、FR-004、FR-005。

### IF-002：事件派发与动作结果

- **类型**：内部 Rust API
- **权威位置**：`crates/page-runtime`
- **规格**：

```rust
pub struct EventDispatchResult {
    pub default_allowed: bool,
    pub html_changed: bool,
}

pub struct HtmlActionEffects {
    pub html_changed: bool,
    pub focus_change: Option<Option<PageNodeRef>>,
    pub navigation: Option<FormNavigationIntent>,
    pub frame_invalidation: FrameInvalidation,
}
```

- **错误处理**：脚本异常通过现有页面错误边界报告；UA 默认动作是否执行只取决于规范事件是否取消，不取决于 listener 是否存在。
- **默认动作**：`default_allowed=false` 时不得应用对应默认动作；非 cancelable 事件调用 `preventDefault()` 不改变结果。
- **交叉引用**：FR-003、FR-004、FR-005。

### IF-003：WebDriver HTML 交互端点

- **类型**：HTTP/W3C WebDriver 子集
- **权威位置**：`apps/webdriver`
- **规格**：

| 方法与路径 | 行为 |
|---|---|
| `POST /session/{id}/element/{ref}/click` | 在 live page 上执行元素点击 |
| `POST /session/{id}/element/{ref}/value` | 向 live element 发送文本及 WebDriver 特殊键 |
| `GET /session/{id}/element/active` | 返回 live page 的 active element |
| `POST /session/{id}/execute/sync` | 在 live page context 执行同步脚本并返回可序列化值 |

- **错误处理**：缺失元素返回 `no such element`；节点身份失效返回 `stale element reference`；不支持的键或参数返回 `invalid argument`。
- **默认动作**：请求不得静默回落到与页面无关的虚拟 DOM。
- **交叉引用**：FR-010。

### IF-004：WPT testharness 结果协议

- **类型**：runner 内部协议
- **权威位置**：`tests/wpt-runner`
- **规格**：

```rust
pub enum HarnessStatus {
    Pass,
    Fail,
    Timeout,
    Unsupported,
}

pub struct HarnessSubtestResult {
    pub name: String,
    pub status: HarnessStatus,
    pub message: Option<String>,
}
```

- **错误处理**：页面崩溃、超时、未支持命令必须进入确定状态并计入非成功结果。
- **默认动作**：没有结果回传的用例在超时后记为 `Timeout`，不得推断为通过。
- **交叉引用**：FR-011。

### IF-005：HTML 兼容测试矩阵

- **类型**：项目控制面
- **权威位置**：`docs/goal/html-compat/test-matrix.md`
- **规格**：

| 字段 | 含义 |
|---|---|
| `feature_id` | 稳定行为点标识 |
| `spec_link` | WHATWG/W3C 规范锚点 |
| `fr` | 本文 FR 编号 |
| `implementation` | 主实现模块 |
| `local_unit` | 正常与边界自有单测 |
| `local_integration` | 跨层或产品自有测试 |
| `wpt` | 上游用例，可为空 |
| `status` | planned/partial/pass/blocked |

- **错误处理**：缺少自有测试字段时不能标记 `pass`。
- **默认动作**：WPT 为空不阻止基础行为完成；自有测试为空必须阻止完成。
- **交叉引用**：FR-012、NFR-001、NFR-008。

## 6. 约束、决策与假设

### 6.1 必须约束

- M0、M1、M2、M3a-M3c、M4 必须按顺序推进；M3d 在 M2 后可独立排期。
- 每个生产行为改动必须与其仓内自有测试在同一提交中合入。
- 每个规范行为实现必须包含对应规范链接注释。
- 多进程测试必须先构建 `zero-renderer`；涉及 compositor 时同时构建 `zero-compositor`。
- 多进程 GUI 测试必须串行并使用现有互斥机制。
- 所有测试和构建必须遵守 `docs/rally/run-rules.md` 的 `test-guard` 约束。
- WPT 导入必须记录上游路径和当前支持状态。

### 6.2 禁止约束

- 不得把 WPT 通过当作缺少仓内自有测试的豁免。
- 不得为通过测试硬编码目标 fixture 的 selector、文案或节点位置到生产行为核心。
- 不得让 JavaScript 开关关闭 UA 默认动作。
- 不得在 renderer、TabWorker 和 WebView 复制三套规范状态机。
- 不得通过 CSS/layout/render 特判修复 HTML 行为语义。
- 不得新增 `#[ignore]` 隐藏 HTML 兼容测试失败。
- 不得把 unsupported、timeout 或未返回结果计为 WPT PASS。

### 6.3 已定决策

- 复用并扩展 `zero-page-runtime`，不新增 HTML 行为 crate。
- 仓内自有测试是第一门禁，WPT 是第二门禁。
- WebDriver 元素引用改为 session 内 opaque ID，并保留节点身份校验；不继续直接使用 selector 作为最终元素引用。
- WebDriver M4 通过 `zero-protocol` 驱动真实 `zero-renderer`，不以独立 WebView 虚拟 DOM 冒充产品 renderer。
- WPT testdriver 与 WebDriver 共享自动化动作语义；协议适配可以不同，行为核心不能重复。
- M3 按元素族交付，不按标签字母顺序推进。
- M1 先建立最小 `PageNodeRef` contract，并在页面交互状态中记录稳定 `pressed_target`；release/cancel 不重新以当前 hover target 替换。
- M2 把 focus owner、retained form state 和 automation element ref 全部迁到同一 `PageNodeRef` identity contract。
- M0 的 `HtmlScenario`/`PageQuery` 是测试专用 helper；不为其新增生产依赖或通用 UI crate。
- ZeroUI 仅作为机制参考，不添加 ZeroUI crate 依赖，不复制其 UTF-8 byte-offset TextEditCore 或 Widget reducer 状态机。

### 6.4 技术约束

- Rust edition、MSRV、feature gate 和 V8/QuickJS 规则沿用工作区现状。
- `zero-page-runtime` 已依赖 `zero-engine`，共享动作层可以复用 `DomMutation`、`FrameInvalidation` 和现有表单 helper。
- `zero-protocol` 消息枚举只能尾部追加，避免跨版本序号错位。
- `apps/webdriver` 当前是单线程最小 HTTP 服务；M4 不要求并发 session 性能优化。
- QuickJS 仍为扩展/测试矩阵，不要求与 V8 页面引擎完全等价，但公共接口必须可编译。

### 6.5 假设

| ID | 假设 | 状态 | 验证方式 |
|---|---|---|---|
| A-1 | 目标表单页 M0 完整流程不会暴露布局根因 | 待验证 | M0 首个完整场景 |
| A-2 | `page-runtime` 可承载共享动作而不形成 engine 反向循环依赖 | 已验证 | 当前依赖方向为 page-runtime → engine |
| A-3 | WebDriver 通过新增 IPC 可查询 live page state | 待验证 | M4 IPC spike + protocol roundtrip test |
| A-4 | 选定 WPT 子集不依赖未实现的复杂 WebDriver endpoints | 待验证 | M4 导入前 manifest/API 扫描 |
| A-5 | engine 可提供跨增量 DOM 更新稳定、跨 replacement 失效的 node handle | 待验证 | M1 PageNodeRef spike + identity tests |

待验证假设不得写成阶段完成事实；对应里程碑必须先用测试或 spike 关闭。

### 6.5A 实现来源说明

| 能力/行为 | 来源类型 | 具体来源 | 备注 |
|---|---|---|---|
| HTML 解析与查询 | 复用现有模块 | `zero-dom`、html5ever | 不新增 parser |
| 表单 retained 状态 | 复用现有模块 | `zero-page-runtime::form_control` | M2 扩展 |
| DOM mutation 与表单 helper | 复用现有模块 | `zero-engine::js_dom_bridge` | 逐步减少散点调用 |
| 脚本事件派发 | 复用现有模块 | `JsExecutor`、renderer/TabWorker JS worker | 由 adapter 注入 |
| 多进程自动化 | 扩展现有模块 | `zero-protocol::RendererHandle`、`zero-renderer` | M4 新增 IPC |
| WebDriver HTTP | 扩展现有模块 | `apps/webdriver` | 不引入 WebDriver 第三方 server |
| testharness/testdriver | 仓内自实现适配 | `tests/wpt-runner` | 复用上游 JS 资源和现有 runner |
| 测试资源保护 | 复用现有工具 | `scripts/test-guard.rs`、Makefile | 禁止裸跑长测试 |
| press target capture | 借鉴机制、仓内实现 | `PageInteractionState` + `PageNodeRef` | 参考 ZeroUI capture，不复制 Widget 代码 |
| 自有交互 Scenario | 借鉴机制、测试专用实现 | `apps/browser/src/tests/html_scenario.rs` | typed steps + step diagnostics |
| automation owner loop | 借鉴机制、扩展既有 IPC | `RendererHandle` + request/reply map | 不引入 ZeroUI WebSocket |

本计划不新增第三方 Rust 依赖。若实施中确需新增依赖，必须先更新本节并重新确认。

### 6.6 代码变更边界

允许修改：

- `crates/dom/**`
- `crates/page-runtime/**`
- `crates/engine/src/js_dom_bridge*`
- `crates/engine/src/js_dom_shim/**`
- `crates/protocol/**`
- `crates/webview/**`
- `apps/renderer/**`
- `apps/browser/src/tab_worker.rs`
- `apps/browser/src/tab_manager.rs`
- `apps/browser/src/tab_scripts.rs`
- `apps/browser/src/tests.rs`
- `apps/browser/src/tests/**`
- `apps/browser/src/app.rs` 中测试 helper 或输入接线
- `apps/webdriver/**`
- `tests/integration/**`
- `tests/wpt-runner/**`
- `examples/forms/**`
- `docs/goal/html-compat/**`
- 本文档与对应 learning 文档

禁止修改：

- `crates/css-parser/**`：CSS 不在范围。
- `crates/style-system/**`：层叠与计算值不在范围。
- `crates/layout-engine/**`：布局修复必须移交。
- `crates/render-foundation/**`：像素和 GPU 不在范围。
- `apps/compositor/**`：合成器不在范围。
- `Cargo.lock`：本计划不新增依赖。

修改禁止路径时必须停止当前切片，提交最小复现，并说明跨线依赖。

### 6.7 执行技能提示

| 范围 | Skill | 模式 | 原因 |
|---|---|---|---|
| 真实 GUI/WebDriver 最终验收 | `lei-product-acceptance` | preferred | 验证产品输入链而非仅单测 |
| 复杂运行时分叉且静态分析不足 | `TRAE-debugger` | preferred | 收集 renderer/browser 运行证据 |
| 普通 Rust 实施与测试 | `lei-code-guidelines` | required | 保持精准修改与测试先行 |

## 7. 优先级、里程碑与实施交接

### 7.1 里程碑

| 里程碑 | 需求 | 完成定义 | 回滚切点 |
|---|---|---|---|
| M0 | FR-001、FR-012 | 目标页完整自有测试矩阵建立，现有行为有逐步断言 | 仅测试/文档提交 |
| M1 | FR-002、FR-003、FR-004 | JS-disabled、输入事件、焦点/激活/表单语义通过自有测试 | 每个规范算法独立提交 |
| M2 | FR-005 | 三宿主共享 `page-runtime` 行为核心且 conformance 全绿 | adapter 切换提交 |
| M3 | FR-006 至 FR-008；FR-009 可选 | 文本、选择/表单、导航/交互元素族按子切片交付 | 每个元素族独立提交 |
| M4 | FR-010、FR-011 | WebDriver live renderer 桥和选定 WPT 子集可稳定运行 | IPC、WebDriver、WPT 分三批 |
| 全程 | FR-012 | 每个功能点都有自有正常+边界测试和跨层测试 | 不允许事后补测 |

M3 分片：

- **M3a**：文本控件，FR-006。
- **M3b**：选择控件与表单，FR-007。
- **M3c**：导航与交互元素，FR-008。
- **M3d**：媒体与资源基础事件，FR-009，可延后且不阻塞 M4。

### 7.2 各里程碑自有测试清单

| 里程碑 | crate/组件自有测试 | 跨层自有测试 | WPT |
|---|---|---|---|
| M0 | renderer 完整 fixture 序列；WebView 全控件 hit-test；HtmlScenario helper | browser 真实多进程完整表单；逐步失败诊断 | 无要求 |
| M1 | page-runtime 状态机；engine 事件字段/顺序；renderer 取消语义 | integration JS-disabled；browser Tab/label/submit | 可选导入，不作完成前提 |
| M2 | action plan/commit/rollback 单测；adapter 单测 | 三宿主 conformance；多进程回归 | 可选 |
| M3a-M3d | 每个行为点正常+边界单测 | 每个元素族至少 1 个 integration/browser 场景 | 对应上游用例 |
| M4 | protocol 序列化；WebDriver 路由/错误；harness parser/result | WebDriver 驱动目标表单；testdriver click/send_keys | 必须运行选定子集 |

### 7.3 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险/注意事项 |
|---|---|---|---|
| `docs/goal/html-compat/` | 新增 | master、test matrix、WPT 账本 | 不复制 Spec 全文 |
| `apps/browser/src/tests.rs` | 修改 | M0/M1 产品场景 | 多进程锁、先构建 bin |
| `apps/browser/src/tests/html_scenario.rs` | 新增 | typed M0 步骤与逐步诊断 | helper 自身须有失败路径测试 |
| `apps/renderer/src/page_scripts.rs` | 修改 | 组件行为与事件测试 | 后续逻辑迁往 runtime |
| `crates/page-runtime/src/form_control.rs` | 扩展 | 状态机与动作核心 | 保持文件 <2000 行，必要时拆模块 |
| `crates/page-runtime/src/html_actions.rs` | 新增 | action plan/commit/rollback | M2 主实现 |
| `crates/dom/src/focus.rs` | 修改 | 规范化 focusability | 不引入样式依赖 |
| `crates/engine/src/js_dom_shim/` | 修改 | 事件对象和 IDL 可观察语义 | V8/QuickJS 编译矩阵 |
| `apps/renderer/src/main.rs` | 修改 | 接入共享 coordinator | 避免继续添加规范分支 |
| `apps/browser/src/tab_worker.rs` | 修改 | 单进程 adapter | 必须与 renderer parity |
| `crates/webview/` | 修改 | WebView adapter 与事件测试 | 不复制状态机 |
| `crates/protocol/` | 修改 | M4 automation IPC | 消息只尾部追加 |
| `apps/webdriver/` | 修改 | live renderer session 与端点 | localhost、安全边界 |
| `tests/integration/src/html_compat.rs` | 新增 | 三路径和元素族自有测试 | 加入 `lib.rs` |
| `tests/wpt-runner/` | 修改 | testharness/testdriver 子集 | unsupported 必须显式 |

### 7.4 职责映射

| 模块 | 单一职责 | 依赖 | 验证 |
|---|---|---|---|
| `zero-dom` | 纯 DOM 关系与 focusability | HTML parser | crate 单测 |
| `zero-page-runtime` | 默认动作规划、状态转换、effects | engine 类型 | crate + conformance |
| `zero-engine` | DOM/IDL/JS 事件可观察语义 | DOM、script runtime | bridge tests |
| renderer/TabWorker/WebView adapter | 宿主副作用与接线 | page-runtime | integration |
| `zero-protocol` | automation IPC 契约 | serde/bincode | roundtrip tests |
| `zero-webdriver` | W3C HTTP 到 automation 的映射 | protocol/renderer | HTTP tests |
| WPT runner | harness 执行与结果收集 | WebView/automation | runner tests |

### 7.5 推荐修改顺序

1. 创建 `docs/goal/html-compat/master.md` 和 `test-matrix.md`，登记 M0 行为点。
2. 完成 M0 自有测试，不修改生产语义；记录每个失败步骤。
3. M1 先做 `PageNodeRef` identity spike，再接 `pressed_target` capture。
4. 按 M1 P0 顺序修 JS-disabled、beforeinput/input、键盘入口；每项先加正常+边界测试。
5. 完成 checkbox/radio/label/reset/submit/focus 子切片。
6. 在 `page-runtime` 新增动作 plan/commit/rollback，并逐个切换 renderer、TabWorker、WebView。
7. 增加三宿主 conformance，删除已被共享核心替代的宿主重复逻辑。
8. 按 M3a-M3c 扩展元素族，每个行为点登记自有测试和 WPT。
9. M4 先扩 automation IPC，再改 WebDriver，最后接 WPT harness/testdriver。
10. M3d 在 M2 后按资源独立排期，不阻塞 M4。
11. 运行全量质量门禁并更新 test matrix 状态。

### 7.6 首批提交建议

| 批次 | 范围 | 预期结果 | 验证 |
|---|---|---|---|
| Commit 1 | goal docs + test matrix | 并行线控制面就绪 | Markdown/path 检查 |
| Commit 2 | M0 renderer/WebView 自有测试 | 组件基线可定位失败 | scoped tests |
| Commit 3 | M0 browser 多进程测试 | 目标页完整产品流程有逐步断言 | guarded browser test |
| Commit 4 | M1 PageNodeRef + pressed capture | paired events 目标稳定 | identity + integration |
| Commit 5 | M1 JS-disabled | UA 动作不依赖脚本 | unit + integration |
| Commit 6 | M1 input events | beforeinput/input 语义闭合 | engine + renderer |
| Commit 7 | M1 focus/activation/forms | 目标表单紧邻差异闭合 | crate + browser |
| Commit 8 | M2 shared action core | 三宿主一致 | conformance |
| Commit 9+ | M3 每元素族一批 | 兼容矩阵单向增长 | local + WPT |
| M4-A | protocol automation | IPC 可往返 | protocol tests |
| M4-B | WebDriver live renderer | HTTP 自动化真实生效 | webdriver tests |
| M4-C | WPT harness/testdriver | 选定子集可运行 | runner + WPT |

### 7.7 阶段验证命令

Scoped 测试可使用 `target/test-guard` 包裹；里程碑完成必须执行：

```bash
cargo fmt --all -- --check
make test
./target/test-guard --per-proc-mem 10 --total-mem 28 -- \
  cargo clippy --workspace --all-targets -- -D warnings
```

涉及性能关键路径时额外执行：

```bash
make bench-gate
```

M4 额外执行新建的 HTML testharness Makefile 入口；该入口本身必须调用 `target/test-guard`，不得裸跑。

## 8. 技术设计（RFC）

### 8.1 现状分析

当前行为路径：

```text
Browser input
  -> TabManager / protocol
  -> renderer dispatch_dom_at
  -> JS event
  -> renderer-specific default-action branch
  -> page_scripts helper / JS shim mutation
  -> WebView render + frame publish

TabWorker input
  -> mirrored branch
  -> tab_scripts helper / JS shim mutation

WebView API
  -> separate dispatch_event path
```

主要问题：

- 默认动作与 JS listener 派发耦合，JavaScript 禁用时动作被短路。
- checkbox/radio、输入、focus、reset/submit 分散在宿主代码。
- `DispatchDomEvent` 与 `KeyboardEvent` 两个 renderer 入口取消语义不同。
- WebDriver 当前进程内 WebView 与产品多进程 renderer 不同源。
- WPT runner 能解析 testharness 类型，但没有完整结果和 testdriver 动作闭环。
- 现有测试覆盖分散，缺少逐功能点的自有测试账本。

### 8.2 目标架构

```text
Platform / WebDriver / testdriver
              |
              v
       HtmlActionRequest
              |
              v
  +---------------------------+
  | zero-page-runtime         |
  | target resolution contract|
  | action plan               |
  | prepare / commit / rollback|
  | shared form/focus state   |
  +---------------------------+
        |        |        |
        v        v        v
   renderer  TabWorker  WebView
        |        |        |
        +--- host adapters--+
                 |
       JS dispatch / DOM apply
       navigation / frame publish
```

上图为本 RFC 的目标状态。规范决策集中在 `zero-page-runtime`；宿主 adapter 只提供：

- 目标查询和 live DOM 访问。
- JS 事件派发及 `default_allowed` 结果。
- mutation 应用。
- navigation intent 执行。
- frame invalidation/publish。

### 8.3 设计选项

| 维度 | A：继续宿主内补分支 | B：全部放 JS shim | C：page-runtime 共享动作核心 |
|---|---|---|---|
| 初始改动 | 低 | 中 | 中 |
| 三路径一致性 | 差 | 一般 | 好 |
| JS-disabled | 差 | 不可满足 | 好 |
| 可单元测试性 | 一般 | 差 | 好 |
| 长期维护 | 差 | 一般 | 好 |
| 决定 | 拒绝 | 拒绝 | 采用 |

采用方案 C。

理由：

1. retained 表单状态已位于 `page-runtime`，扩展职责自然。
2. UA 默认动作不能依赖 JavaScript runtime。
3. 纯 action plan/transaction 可直接覆盖正常、取消和回滚测试。
4. renderer、TabWorker、WebView 只需实现窄 adapter。

### 8.4 动作事务模型

#### 8.4.1 生命周期

```text
resolve target
  -> build action plan
  -> optional prepare/pre-activation
  -> dispatch cancelable event
     -> canceled: rollback prepared state -> return
     -> allowed: commit default state
  -> dispatch non-cancelable follow-up events
  -> emit focus/navigation/invalidation effects
  -> host applies effects once
```

动作计划建议结构：

```rust
pub struct HtmlActionPlan {
    pub target: PageNodeRef,
    pub prepare: Vec<DomMutation>,
    pub cancelable_event: Option<PlannedEvent>,
    pub rollback: Vec<DomMutation>,
    pub commit: Vec<DomMutation>,
    pub followup_events: Vec<PlannedEvent>,
    pub focus_effect: Option<Option<PageNodeRef>>,
    pub navigation: Option<FormNavigationIntent>,
}
```

该结构是内部设计，不对外承诺稳定 API。实现可拆分为更小类型，但必须保留 prepare/rollback/commit 三阶段能力。

#### 8.4.2 Press target capture

primary press 命中可交互节点时，coordinator 保存 `PressedTarget`。后续 release/cancel：

1. 先按 `PageNodeRef` 解析原 pressed target。
2. generation 有效时把 paired event 回投给原目标。
3. 节点被替换、移除或页面导航时取消 activation transaction。
4. release 坐标仅用于 click 拖动阈值和事件坐标，不用于替换目标 identity。

该机制只固定 paired event target，不强制生成 click。

#### 8.4.3 文本编辑

普通文本输入：

1. `keydown` 被取消时终止。
2. 构造 `beforeinput(inputType=insertText, data=...)`。
3. `beforeinput` 被取消时不修改 value/selection。
4. 更新 retained value/selection/dirty。
5. 派发不可取消的 `input`。
6. 汇总 listener mutation，最多发布一帧。

Backspace 使用 `deleteContentBackward`。删除必须按 UTF-16 selection contract 定位，并至少保证代理对不被拆开；更完整 grapheme cluster 删除可作为 M3a 后续行为点登记。

IME 使用 composition 状态：

- preedit 只更新临时 composition，不写入 live value。
- commit 通过 `beforeinput(insertCompositionText)` 或规范对应 inputType 进入正式 value。
- cancel 清除临时状态且不派发错误的 committed input。

#### 8.4.4 checkbox/radio 激活

checkbox/radio 需要 pre-activation 事务：

1. 保存旧 checkedness 和 radio group 旧选中项。
2. 在 click listener 可观察前应用规范要求的临时 checkedness。
3. 派发 cancelable click。
4. click 被取消时 rollback。
5. 未取消时 commit，并按规范派发 input/change。

重复激活已选 radio 的 plan 必须是 no-op，不派发错误 change。

#### 8.4.5 label 激活

label 激活流程：

1. 解析显式 `for` 或首个可标记后代 control。
2. 若 click 原始目标本身是 label 内的交互后代，避免重复转发。
3. 向关联 control 发起一次 activation request。
4. 保留事件来源信息，防止 label/control 递归激活。

#### 8.4.6 focus 与顺序导航

`FocusManager` 从标签白名单升级为基于元素状态的 focusable-area 判定：

- `a/area` 仅在具有可导航链接时自然可聚焦。
- `input[type=hidden]` 不可聚焦。
- disabled、inert、hidden 子树不可顺序聚焦。
- disabled fieldset 遵守首个 legend 例外。
- `tabindex < 0` 可程序聚焦但不进入顺序导航。
- 正 tabindex 升序，同值保持文档序；随后是 0/自然顺序。

样式造成的不可见性不在本线判断；本线只处理 HTML 属性和树状态。

#### 8.4.7 reset 与 submit

reset：

1. 派发 cancelable reset。
2. 未取消时恢复全部 resettable control 的默认状态。
3. 同步 retained state 和 JS 可观察 property。
4. 不派发伪造的 input/change。

submit：

1. 确定 form owner 和 submitter。
2. 执行当前已支持的约束验证子集。
3. 派发 cancelable submit。
4. 未取消时构造 entry list 和 `FormNavigationIntent`。
5. renderer/宿主执行 GET/POST 导航。

### 8.5 宿主 adapter

#### 8.5.1 renderer

- 将 `handle_dispatch_dom_event` 与 `handle_keyboard_event` 合并到一个 coordinator 入口。
- 继续持有真实 JS worker、WebView、navigation 和 publish 权限。
- 每个 action transaction 最多触发一次最终 publish。

#### 8.5.2 TabWorker

- 删除与 renderer 重复的默认动作判定。
- 复用相同 coordinator；单进程 worker 只实现 script/mutation/navigation 消费。
- 现有 `ExecuteScriptForTest` 继续用于自有测试回读，不成为生产接口。

#### 8.5.3 WebView

- `dispatch_event` 保持公开 API 兼容。
- 新增内部 user action 入口供 WebDriver/testdriver adapter 使用。
- 不要求嵌入方构造 platform-specific winit event。

### 8.6 WebDriver live renderer 设计

#### 8.6.1 Session 架构

```text
WebDriver HTTP client
  -> Driver session actor
  -> RendererHandle
  -> AutomationRequest IPC
  -> zero-renderer live page
  -> AutomationResponse IPC
  -> W3C response mapping
```

每个 session 持有：

- renderer child handle。
- navigation epoch。
- opaque element reference registry。
- 有上限的 pending automation request map。
- 本地 fetch 代理和超时状态。

所有 mutation/query 按 request id 进入单一 renderer owner；HTTP 连接线程不得直接持有 live page 内部可变对象。

#### 8.6.2 IPC

在 `IpcMessageKind` 尾部追加通用 `AutomationRequest` 和 `AutomationResponse`，内部 operation 包含：

- FindElement。
- ElementClick。
- SendKeys。
- GetActiveElement。
- ExecuteScript。

协议测试必须覆盖：

- 每个 operation roundtrip。
- 长文本和 Unicode。
- 缺失可选字段。
- 未知/不支持 operation 的显式错误。
- 跨 navigation epoch 的 stale reference。
- pending 上限、peer close、request timeout 和 shutdown。

#### 8.6.3 元素引用

HTTP 层继续使用 W3C 固定 element key，但 value 改为 session-local opaque ID。renderer registry 绑定：

- navigation epoch。
- live document generation。
- node identity/handle。

导航后全部引用失效。DOM 替换导致原节点不存在时返回 stale，不允许 selector 重新匹配到新节点。

### 8.7 WPT testharness/testdriver 设计

#### 8.7.1 Harness

runner 为目标页面注入最小 reporter，收集：

- harness completion。
- subtest name/status/message。
- uncaught error。
- timeout。

runner 使用固定墙钟超时，并在页面/renderer 崩溃时产生确定失败结果。

#### 8.7.2 testdriver

首批支持：

- `test_driver.click(element)`。
- `test_driver.send_keys(element, keys)`。
- 基础 key actions 中的 Tab、Shift+Tab、Backspace、Enter 和文本。

未支持命令返回明确 rejection，由 harness 记录 Unsupported/Fail。不得提供返回成功但不执行动作的 stub。

#### 8.7.3 用例选择

首批目录：

- `html/semantics/forms/` 中与 M1/M3 已完成功能直接对应的 testharness。
- `html/interaction/focus/` 或 manifest 中等价焦点用例。
- `input-events/` 中普通 input/textarea、beforeinput/inputType 用例。
- `uievents/` 中键盘、focus、composition 的已支持子集。

导入前必须扫描每个用例依赖的 testdriver API；超出支持面时先标 blocked，不修改预期绕过。

### 8.8 测试策略

#### 8.8.1 自有测试最低标准

每个 `feature_id` 必须满足：

| 行为类型 | 必须自有测试 |
|---|---|
| 纯状态转换 | 正常单测 + no-op/取消/边界单测 |
| JS 可观察行为 | event fields/order 正常测试 + preventDefault/异常测试 |
| 用户可见交互 | 至少一个 integration、browser 或 WebDriver 场景 |
| IPC/API | roundtrip 正常测试 + malformed/stale/unsupported 测试 |
| WPT harness | PASS 收集测试 + FAIL/TIMEOUT/UNSUPPORTED 测试 |

WPT 不计入上述最低数量。

#### 8.8.2 HtmlScenario 与 PageQuery

M0 在 browser 测试模块提供 typed steps：

- `Click(selector)`、`TypeText(text)`、`PressKey(key)`、`ImePreedit/Commit`。
- `AssertValue`、`AssertChecked`、`AssertFocused`、`AssertText`、`AssertUrl`。
- `WaitForSnapshot` 只作为同步原语，不作为行为成功断言。

失败必须返回 step index、step description、selector/node ref、expected/actual、URL/navigation epoch 和 snapshot sequence。helper 自身必须测试正常序列与第 N 步失败诊断。

#### 8.8.3 自有 fixture

- `examples/forms/form-interaction-test.html`：产品级完整流程。
- `tests/integration/src/html_compat.rs`：跨宿主和元素族最小 HTML。
- crate 测试使用最小 inline HTML，不依赖大页面 fixture。
- WebDriver 测试使用本地 TCP HTTP server，不访问外网。

fixture 必须包含稳定 ID；无 ID 元素身份行为应由单独测试覆盖，不依赖脆弱坐标。

#### 8.8.4 测试分层

```text
Level 1: zero-dom / page-runtime pure unit tests
Level 2: engine JS bridge + renderer page_scripts tests
Level 3: zero-webview + zero-integration-tests
Level 4: zero-browser real multiprocess tests
Level 5: zero-webdriver HTTP tests
Level 6: selected upstream WPT
```

修复必须从最低能复现根因的层开始测试，再补一层用户可见回归。禁止只加 Level 4/5 大测试而没有根因层测试。

#### 8.8.5 关键自有回归簇

| 簇 | 正常路径 | 边界/失败路径 |
|---|---|---|
| 文本输入 | insert、selection replace、IME commit | beforeinput cancel、start deletion、IME cancel |
| 焦点 | click、Tab、Shift+Tab | disabled/hidden/inert、keydown cancel |
| checkbox/radio | toggle、group switch | click cancel rollback、checked radio repeat |
| pointer pairing | press/release 同目标 | 移动、重排、移除、导航 |
| label | nested/for activation | interactive descendant、防递归 |
| reset/submit | default restore、GET/POST intent | preventDefault、无 form owner |
| 三宿主 | 相同动作相同结果 | 相同取消相同 no-op |
| WebDriver | find/click/send keys/query | missing/stale/malformed |
| harness | pass/fail result | timeout/unsupported/crash |

#### 8.8.6 确定性短序列压力测试

M3 为 focus/IME/reset/input 增加 20-100 轮短序列测试，至少断言：

- 最终 value/checkedness/focus 与单轮语义一致。
- composition、pressed target 和 pending request 均被清理。
- retained state 条目和 revision 不发生无界增长。
- 页面 timer 相关测试可显式推进 test clock；墙钟只负责总超时保护。

### 8.9 安全考虑

- WebDriver 继续只绑定 loopback 地址；本 RFC 不开放远程监听。
- HTTP 请求体保留 1 MiB 上限，并为 send keys/execute script 增加显式长度校验。
- Execute Script 只在 renderer 页面上下文运行，不暴露文件系统、进程或宿主凭证。
- Automation IPC 使用现有序列化边界，不接受未枚举的宿主操作。
- element reference 必须绑定 session 和 navigation epoch，防止跨 session/跨页面误用。
- 表单提交继续经过现有网络、安全和导航策略，不由 action core 直接发网络请求。
- 测试 fixture 不包含密钥、真实账号或外网依赖。

### 8.10 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| M1 修改事件顺序导致既有测试变化 | 中 | 每个事件算法单独提交，记录规范依据 |
| M2 迁移期间双实现分叉 | 高 | adapter 切换前后跑同一 conformance；切换后删除旧分支 |
| press/release 间节点被替换 | 中 | PageNodeRef generation 失配时 cancel，不重命中新节点 |
| focusability 依赖 CSS 可见性 | 中 | 本期只处理 HTML 属性/树状态，CSS 部分移交 |
| WebDriver renderer host 需处理 fetch | 中 | 复用 RendererHandle 与本地 fetch 代理，先做本地 fixture |
| opaque element identity 受文档重建影响 | 高 | document generation + node handle；优先防误命中 |
| WPT 依赖超出已支持 API | 高 | 预扫描依赖，unsupported 显式记录 |
| 多进程测试资源争用 | 高 | 全局锁、单线程、test-guard、先构建 bin |

### 8.11 实施与回滚

实施原则：

- 每个 Commit 对应一个规范算法或测试基础设施切片。
- 每个切片先写自有失败测试，再改生产代码。
- 仅在 M4-C 导入与已实现行为对应的 WPT。
- 里程碑完成后更新 test matrix，不在实现前预标 pass。

回滚：

- M0 为测试/文档，可独立回滚。
- M1 每个行为算法独立提交，可逐项 revert。
- M2 在删除旧分支前保留 adapter parity 提交；若回归可回退 adapter 切换。
- M3 每个元素族独立回滚，不影响其他族。
- M4 的 protocol、WebDriver、WPT 分批；protocol 只尾部追加，回滚消费者不破坏旧消息。
- 不使用长期双实现 feature flag；临时迁移开关必须在 M2 完成前删除。

## 9. Spec Lint 报告

### 9.1 结构完整性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 执行摘要存在性 | Pass | §0 包含目标、范围、排除项、约束、方案和首步 |
| 场景存在性 | Pass | FR-001 至 FR-012 均包含至少 2 个验收场景 |
| 异常路径覆盖 | Pass | 每个 FR 至少包含 1 个取消、失败、no-op、stale、timeout 或 unsupported 场景，数量不低于正常场景 |
| 测试绑定 | Pass | §3 每个场景均绑定测试函数、矩阵或命令 |
| UI 对齐 | Skip | 本文不设计新视觉 UI；目标页可见状态已在 FR-001 定义 |
| TBD 清零 | Pass | §10 无阻塞级 TBD；A-1/A-3/A-4/A-5 均绑定里程碑验证 |
| 约束覆盖 | Pass | §6.1 由 FR-012、NFR-003/NFR-005/NFR-006 和 §7.7 门禁覆盖 |
| 实施交接完备 | Pass | §7.3-§7.6 包含文件、职责、顺序和提交批次 |
| 首步可执行性 | Pass | §0 与 §7.5 均明确先建控制面和 M0 自有测试 |

### 9.2 语言精确性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 模糊动词 | Pass | FR 使用“派发、阻止、恢复、返回、执行、登记”等可断言行为 |
| 无量化描述 | Pass | NFR-001 给出最少测试数；性能沿用现有可测量基线 |
| 非确定性措辞 | Pass | “应该/可以”仅用于优先级分类；各里程碑交付条件使用“必须” |

### 9.3 一致性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 范围冲突 | Pass | §1.4 与 §6.6 一致排除 CSS/layout/render |
| 约束冲突 | Pass | 自有测试第一门禁与 WPT 第二门禁在 §0、FR-012、NFR-001 一致 |
| 方案漂移 | Pass | §8 只使用 §6.5A 已列现有模块和仓内实现 |
| CLI 语义一致 | Skip | 本文不新增 CLI；仅新增 Makefile 测试入口 |
| 默认动作闭合 | Pass | IF-002 和 §8.4 明确 allowed/canceled/rollback/commit |
| 章节引用正确 | Pass | FR/NFR/IF 与 §7/§8 的交叉引用均指向对应主题 |
| 外部事实保守化 | Pass | WPT API、live renderer IPC 和 node handle 来源列为 A-3/A-4/A-5 待验证 |
| 未验证细节泄漏 | Pass | PageNodeHandle 保持 opaque；M1/M4 先安排 spike/扫描 |
| 场景预期泄漏 | Pass | 验收场景断言行为和错误类型，不硬编码未验证上游资产名 |
| 实现来源闭合 | Pass | §6.5A 列出每项能力的现有模块或仓内承载位置 |
| 来源-测试联动 | Pass | §7.4 将实现模块与测试层对应 |
| 脆弱选择逻辑覆盖 | Pass | FR-010 stale ref、FR-011 unsupported 和 §8.6/§8.7 明确测试 |
| 类型分层清晰 | Pass | FR 定义行为，IF 定义接口，§6 定义决策/假设，§8 定义实现 |
| 优先级完备 | Pass | FR-001 至 FR-012、NFR-001 至 NFR-008 均标优先级 |
| 代码边界完备 | Pass | §6.6 明确允许和禁止路径 |
| 清单数量一致 | Pass | 12 FR、8 NFR、5 IF 与 §2 计数一致 |
| 依赖清单一致 | Pass | §6.5A 明确无新增第三方依赖，禁止 `Cargo.lock` 变更 |
| 重复失控 | Pass | 测试最低标准以 §8.8 为实现主定义，其他章节仅摘要或交叉引用 |

**汇总**：28 Pass / 0 Warning / 0 Fail / 2 Skip
**门禁判定**：Fail = 0，允许进入用户确认。

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|---|---|---|---|---|
| TBD-1 | M0 是否暴露布局根因 | 重要 | 完整产品场景尚未执行 | M0 测试先行；若属布局则移交 |
| TBD-2 | page/automation element identity 的最小稳定句柄 | 重要 | live document generation 下的节点保活边界 | M1-A 先做 engine/renderer spike |
| TBD-3 | 首批 WPT 精确清单 | 重要 | 每个上游用例的 testdriver API 依赖 | M4-C 导入前自动扫描并登记 |

以上 TBD 均不阻塞 Spec 确认；它们是对应里程碑的首个验证任务。

## 11. 修订历史

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v1.0 | 2026-08-12 | 将 M0-M4 转为实施规格；新增仓内自有测试硬门禁、共享动作 RFC、WebDriver/WPT 设计 |
| v1.1 | 2026-08-12 | 吸收 ZeroUI 源码调研：PageNodeRef、pressed target capture、HtmlScenario/PageQuery、bounded automation owner 和短序列压力测试 |
