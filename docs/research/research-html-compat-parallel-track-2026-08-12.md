# HTML 行为兼容线：从表单场景验收到规范驱动的并行开发

> 日期：2026-08-12
> 模式：源码深潜 + 官方规范交叉验证
> 范围：基础 HTML 元素的解析、DOM/IDL、交互状态、事件与默认动作；不处理 CSS 样式、布局精度和控件外观

## 来源分级总表

| 分级 | 本文使用方式 | 代表来源 |
|---|---|---|
| 一手事实 | 当前仓库源码、测试、提交记录及本次定向测试 | [1]-[12] |
| 官方规范 | WHATWG HTML、W3C UI Events/Input Events、WPT 文档 | [13]-[20] |
| 补充源码调研 | ZeroUI 事件、identity、测试与自动化机制 | [21] |
| 💡 推理 | 基于源码现状和并行开发约束给出的赛道设计 | 各章显式标注 |
| 作者综合 | 验收矩阵、模块边界、里程碑和优先级模型 | 各表格显式标注 |

## 30 秒速览

- **建议另起 `html-compat` 开发线，但不是另起长期 Git 分支**：使用第三个独立 clone，继续在 `main` 上小步提交、推送前 `pull --rebase`。
- 这条线的对象不是“标签能画出来”，而是 **HTML processing model**：解析、元素状态、IDL 反射、焦点、激活、事件默认动作、表单 reset/submit。
- `form-interaction-test.html` 当前并非不可用。本次实测中，3 个 renderer 场景测试和 1 个真实多进程 GPU/四档 DPI 测试全部通过。
- 但现有产品测试没有执行页面写明的完整步骤：缺少退格、Tab、checkbox、radio、reset、submit 的同一端到端场景和逐步结果断言。
- 已确认的规范差异包括：JS 禁用时 HTML 默认动作被短路、`input` 被错误标为可取消、缺少 `beforeinput`、两条键盘入口的 `preventDefault` 行为不一致。
- 第一阶段不应大改架构。先把目标页完整步骤锁成一个稳定场景，再修暴露出的行为差异；行为稳定后才把默认动作下沉到 `page-runtime`。
- 后续以“元素族 + 用户场景”推进，而不是按标签字母表推进：文本编辑 → 选择控件 → 按钮/表单 → 标签关联/焦点 → 其他交互元素。

完整工作循环：

`规范条款/WPT 用例 → 最小产品场景 → 失败归因 → 共享行为层修复 → 分层回归 → 多进程产品验收 → 更新兼容矩阵`

## 执行摘要

### 核心裁决

| 问题 | 裁决 | 置信度 |
|---|---|---|
| 是否值得另起并行线 | 值得。现有渲染线优化像素一致性，zero-web 线覆盖更广的 DOM/JS；HTML 行为需要独立、持续的验收节奏 | 高 |
| 是否从目标表单页开始 | 是。它同时覆盖文本编辑、IME、焦点、选择控件、按钮、reset/submit 和脚本可观察结果 | 高 |
| 当前页面是否“完全正常” | 尚不能这样宣称。已覆盖路径通过，但完整用户步骤未被同一端到端测试证明，且存在已确认的规范差异 | 高 |
| 是否需要新 crate | M0/M1 不需要。优先复用 `zero-page-runtime`；行为稳定后再扩展其职责 | 高 |
| 是否把样式问题纳入 | 不纳入。布局/绘制只作为“能命中、能上屏”的契约；根因属于 CSS/布局时移交渲染线 | 高 |
| 是否先接全量 WPT testharness | 不先做。当前 runner 能解析 testharness 清单，但产品交互自动化尚未形成 WPT testdriver 执行闭环 | 中高 |

### 推荐下一步

1. 建立 `docs/goal/html-compat.md` 与 `docs/goal/html-compat/master.md`，记录兼容矩阵、当前切片和跨线移交项。
2. 新增一个完整执行目标页步骤的多进程场景测试，先不改生产逻辑。
3. 用该场景暴露真实失败，再按优先级修复 JS-disabled 默认动作、事件语义、label 激活和输入路由分叉。
4. M1 稳定后，把表单默认动作从 renderer/JS shim 的散点逻辑收敛到 `zero-page-runtime`。
5. 再接 WPT testharness/testdriver 子集，避免自写测试长期成为唯一 Oracle。

> **📌 来源说明（执行摘要）**
>
> - **一手事实** [1]-[12]：目标页、现有实现、现有测试和测试基础设施。
> - **官方规范** [13]-[20]：HTML 控件状态、焦点、激活、输入事件与自动化测试口径。
> - **💡 推理**：并行线组织方式和里程碑顺序是基于现有所有权冲突与测试缺口的工程判断。
> - **作者综合**：核心裁决表和工作循环为本文综合产物。

## 1. 任务规划

### 1.1 5W1H

| 维度 | 当前理解 | 待处理 |
|---|---|---|
| What | 建立专门处理基础 HTML 元素行为兼容性的并行开发线 | 先以表单页形成可重复模板 |
| Why | 当前实现和测试分散在 DOM、engine shim、renderer、browser；缺少单一兼容矩阵和完成定义 | 防止“局部单测绿”被误认为“页面可用” |
| Where | `dom`、`page-runtime`、HTML 专属 engine bridge、renderer 输入路由、browser/WebDriver 验收 | CSS/style/layout/render-foundation 明确排除 |
| When | 以 2026-08-12 当前 `main` 为基线 | 每个切片短周期合入，不维护长期分叉 |
| Who | HTML 兼容线主责；渲染线和 zero-web 线通过共享文件门禁协作 | 共享面变更需先 rebase、后小提交 |
| How | 规范 + WPT + 产品 fixture 三重证据，按用户场景纵向闭环 | 先验收、后修复、再抽共享机制 |

### 1.2 术语映射

| 用户表述 | 精确术语 | 本文采用的范围 |
|---|---|---|
| 基础 HTML 标签渲染 | HTML elements / processing model | 元素语义和交互，不含视觉样式 |
| 兼容性 | Web compatibility / conformance | DOM/IDL、状态、事件、默认动作、导航 |
| 表单正常工作 | Form control infrastructure | value/checkedness、focus、activation、reset、submit |
| 点击和输入 | User interaction / UI Events / Input Events | pointer、keyboard、IME、beforeinput/input/change |
| 自动验证 | WPT testharness/testdriver / WebDriver | 产品场景先行，逐步接上游测试 |

### 1.3 调研子任务

1. 逐项拆解目标页的可观察行为。
2. 追踪 browser → renderer → page-runtime → JS shim → DOM mutation → frame publish 链路。
3. 对照现有单测、集成测试和产品测试，识别“实现存在但未验收”的空档。
4. 对照 HTML/UI Events/Input Events 规范，识别明确差异。
5. 设计不触碰样式域的模块所有权和并行集成方式。
6. 给出可直接转成 Spec/RFC 的里程碑与完成标准。

> **📌 来源说明（第 1 章）**
>
> - **一手事实** [1]-[12]：模块分布、测试形态和当前并行规则。
> - **官方规范** [13]-[20]：术语映射。
> - **⚠️ 假设**：团队会继续采用“独立 clone + 同一 main”的既有并行工作方式。
> - **作者综合**：5W1H 和子任务表。

## 2. 当前实现：能力已经形成，但分布过散

### 2.1 目标页实际覆盖的行为面

目标页不是单纯的表单控件截图。它包含以下可观察契约 [1]：

| 行为族 | 页面元素/脚本 | 成功信号 |
|---|---|---|
| 文本编辑 | `input#name` | `input` listener 读取 live `value` 并更新 `output` |
| 多行编辑 | `textarea#note` | 普通输入、换行和 IME 能进入独立控件 |
| 选择状态 | checkbox/radio | checkedness、互斥组和 `change` |
| 普通激活 | `button[type=button]` | `click` listener 更新 `output` |
| 表单重置 | `button[type=reset]` | `reset` 事件 + 控件恢复默认状态 |
| 表单提交 | `button[type=submit]` | `submit` 事件可取消，取消后不导航 |
| 焦点导航 | Tab | 按文档顺序切换焦点，旧控件按需派 `change` |
| 可访问反馈 | `output[aria-live=polite]` | 脚本更新文本，供视觉与辅助技术观察 |

这也是该页面适合作为首个里程碑的原因：它能把 HTML 元素状态、事件系统、默认动作和真实产品输入链路放在一个场景中。

### 2.2 当前运行链路

```text
Browser platform input
  -> TabManager::DispatchDomEvent
  -> protocol DispatchDomEventParams
  -> renderer dispatch_dom_at
  -> JS listener dispatch
  -> default action gate (default_allowed)
  -> page_scripts / FormControlStateStore
  -> WebView live DOM mutation + render
  -> frame publish
  -> Browser snapshot / hit-test
```

上图是**作者综合**，依据 [3]-[6]。

当前实现已有几个正确的基础：

- `FormControlStateStore` 独立保存 value、UTF-16 选区、composition、dirty 状态和焦点会话 [5]。
- browser 主路径通过异步 dispatch 回执决定默认动作，Tab、Backspace、Enter 和可打印字符已进入 renderer [4]。
- `input`/`textarea` 的 live value 与内容属性已分离，目标页测试明确断言输入后 `value` 属性仍为空 [3]。
- checkbox、radio、reset、submit 均有专门 helper，而不是全部退化成通用属性写入 [3]。
- WebView 层已验证 `#name` 和三个无重复选择器的按钮可通过坐标命中 [8]。

### 2.3 本次定向验证

在同步到 `origin/main` 后，先重建独立 renderer 二进制，再执行以下受 `test-guard` 保护的定向测试：

```bash
./target/test-guard --per-proc-mem 10 --total-mem 28 --time-limit 900 -- cargo build -p zero-renderer
./target/test-guard --per-proc-mem 10 --total-mem 28 --time-limit 300 -- cargo test -p zero-renderer form_interaction_fixture -- --nocapture --test-threads=1
./target/test-guard --per-proc-mem 10 --total-mem 28 --time-limit 180 -- cargo test -p zero-browser form_fixture_physical_clicks_reach_controls_at_windows_scale_factors -- --nocapture --test-threads=1
```

结果：

| 测试 | 结果 | 覆盖 |
|---|---|---|
| renderer `form_interaction_fixture_*` | 3 passed / 0 failed | 文本输入结果、普通按钮、reset、submit |
| browser `form_fixture_physical_clicks_*` | 1 passed / 0 failed | 真实 renderer、GPU 帧、1.0/1.25/1.5/2.0 DPI、name/note/button 命中、IME |

此前文档记录的 4 个表单快照超时，已被仓库经验文档归因为陈旧独立二进制、静默单进程回退和并行子进程竞争 [12]。本次按文档要求先重建 renderer、单线程执行后通过，因此不能把旧超时直接解释为页面行为失败。

> **📌 来源说明（第 2 章）**
>
> - **一手事实** [1]-[8][11][12]：页面行为、实现链路、现有测试和本次定向测试。
> - **官方规范** [13]-[18]：value/checkedness、焦点、事件和默认动作的术语口径。
> - **💡 推理**：目标页是高价值首个切片，因为它用一个页面串联多个 HTML processing model。
> - **作者综合**：运行链路图和行为族表。

## 3. “页面完全正常”的证据缺口

### 3.1 当前逐项状态

| 页面能力 | 实现证据 | 产品级证据 | 当前判定 |
|---|---|---|---|
| 页面加载与控件命中 | WebView hit-test [8] | 四档 DPI 多进程测试 [2] | 已验证 |
| `#name` 输入 + listener 读 live value | renderer fixture [3] | 单独 typing 快照测试 [2] | 基本验证 |
| Backspace | shim 支持选区和代理对删除 [6] | 目标页未覆盖 | 已实现，待端到端 |
| Tab 到下一控件 | renderer 默认动作 + FocusManager [4][7] | 目标页未覆盖 | 已实现，待端到端 |
| textarea + IME | retained composition [4][5] | 目标页一次 preedit/commit [2] | 已验证主要路径 |
| checkbox + `change` 输出 | toggle helper [3] | 仅通用 checkbox 快照，不断言目标页输出 [2] | 待场景验收 |
| radio 互斥 | radio helper [3] | 无目标页产品断言 | 待场景验收 |
| 普通按钮输出 | renderer fixture [3] | 产品测试只验证点中 `#click` | 待结果验收 |
| reset 事件和默认恢复 | renderer fixture [3] | 无产品级完整状态断言 | 待场景验收 |
| submit + `preventDefault` | renderer fixture [3] | 无产品级“未导航”断言 | 待场景验收 |
| 点击 label 激活关联控件 | 未找到 activation 实现 | 无测试 | 高概率缺口 |
| JS 禁用时基础控件仍工作 | 默认动作 helper 被 JS gate 短路 [4] | 无测试 | 已确认缺口 |

因此，当前最准确的表述是：

> 页面已有可用基线，已覆盖路径在本次定向测试中通过；但“按页面说明走完整流程且所有控件语义正确”尚无证据。

### 3.2 已确认的规范差异

#### 差异 A：HTML 默认动作错误依赖 JavaScript 开关

renderer 的文本输入、删除、submit、reset、checkbox 和 radio helper 都在 `javascript_enabled == false` 时直接返回 [4]；browser 侧在 JS 禁用时也停止向 renderer 派发页面事件，只保留外部 pending action [4]。

HTML 控件的 value、checkedness、激活、reset 和 submit 是用户代理行为，不是页面 JavaScript 提供的行为 [13]-[15][20]。关闭脚本后，listener 可以不执行，但输入、勾选和 reset 等基础动作仍应成立。

**影响**：这是本开发线的 P0 语义问题，且与“不处理样式”的边界完全一致。

#### 差异 B：输入事件序列不完整

`__zw_text_input` 和 `__zw_text_delete` 直接修改 value 后派发 `input`，并把 `input` 构造成 `cancelable: true` [6]。Input Events 定义用户编辑应提供 `beforeinput` 和 `input`，其中普通 `beforeinput` 可取消，而 `input` 用于观察已经发生的编辑 [17][18]。

**影响**：目标页本身不监听 `beforeinput`，所以当前演示可工作；编辑器、校验库和阻止输入的页面会出现兼容差异。

#### 差异 C：键盘默认动作有两条不一致入口

`handle_dispatch_dom_event` 只在 `default_allowed` 时执行 Tab/输入/Backspace/Enter [4]；`handle_keyboard_event` 丢弃 dispatch 结果，源码明确标注“不尊重 keydown preventDefault” [4]。

**影响**：同一页面行为取决于宿主走 `DispatchDomEvent` 还是 `KeyboardEvent`，会造成 browser、WebDriver 或未来嵌入宿主之间分叉。

#### 差异 D：焦点判定是标签白名单近似

`FocusManager` 当前把 `a/button/input/select/textarea/summary/details` 作为默认可聚焦标签，只直接排除元素自身 `disabled` [7]。这无法完整表达 `a` 必须有可导航链接、`input[type=hidden]`、inert/hidden 子树、disabled fieldset 继承等规则 [16]。

**影响**：目标页自然顺序大体可用，但扩展到一般网页时会出现错误 Tab stop。

#### 差异 E：控件激活行为仍是“click 后补默认动作”

renderer 先派 `click`，在其未取消后再调用 checkbox/radio helper [3][4]。HTML 对 checkbox/radio 定义了更细的激活与 checkedness 语义 [14]；当前 helper 还会对已选中的 radio 重复派 `change` [3]。

**影响**：目标页 checkbox 的 `change` listener 可得到新状态，但 `click` listener 观察 checkedness、取消激活回滚、已选 radio 重点等边界可能不一致。

### 3.3 首个完整验收脚本

目标页的完成定义应至少执行以下顺序，并在每步等待 renderer 回执或新帧：

| 步骤 | 操作 | 必须断言 |
|---|---|---|
| 1 | 加载页面 | 全部 8 个控件可定位；初始 `output` 为“等待交互” |
| 2 | 点击 `#name`，输入 `abc` | `value == "abc"`；output 为“输入事件：abc” |
| 3 | Backspace | `value == "ab"`；output 为“输入事件：ab” |
| 4 | Tab | `document.activeElement == #note`；`#name` 发生正确 blur/change |
| 5 | textarea 输入两行或 IME | live value、选区和 composition 生命周期正确 |
| 6 | 点击 checkbox | checkedness 翻转；output 显示“已选中” |
| 7 | 点击 `pro` radio | basic=false、pro=true；重复点击 pro 不产生错误状态变化 |
| 8 | 点击普通按钮 | output 显示 click 已触发 |
| 9 | 修改控件后点击 reset | reset listener 触发；文本/checkbox/radio 恢复默认状态 |
| 10 | 点击 submit | submit listener 触发；`defaultPrevented=true`；URL/导航 epoch 不变 |
| 11 | 点击 label 文本 | 关联控件获得焦点或被激活 |
| 12 | JS 禁用变体 | listener 不运行，但输入、选择、reset 等 UA 默认动作仍有效 |

该表是**作者综合**。步骤 1-10直接来自页面 [1]；步骤 11-12用于补齐基础 HTML 语义，不扩大到 CSS。

> **📌 来源说明（第 3 章）**
>
> - **一手事实** [1]-[8]：实现与测试覆盖。
> - **官方规范** [13]-[18]：表单状态、激活、焦点和输入事件。
> - **⚠️ 假设**：label 激活被标为“高概率缺口”，依据全仓搜索未发现实现；应由失败测试最终确认。
> - **💡 推理**：目标页现有测试通过不足以证明完整兼容，因为用户步骤与断言集合不一致。
> - **作者综合**：逐项状态和首个完整验收脚本。

## 4. 并行开发线设计

### 4.1 组织形态

推荐名称：`html-compat`。

推荐运行方式：

```text
clone A: zero-web 主线
clone B: rendering-compat 主线
clone C: html-compat 主线
共同目标: origin/main
```

每个 clone 都在本地 `main` 工作，阶段性小提交；推送前 `git pull --rebase`，禁止强推。这样沿用项目现有并行纪律，又避免长期 feature branch 与快速变化的 renderer/engine 主线大幅漂移。

不建议把这条线合并进 `rendering-compat`：

- rendering-compat 的主要 Oracle 是像素/reftest。
- HTML 行为兼容的主要 Oracle 是状态、事件序列、默认动作和导航结果。
- 二者共享 hit-test 和 frame publish，但失败归因、测试工具和完成定义不同。

不建议继续完全混在 zero-web 主线：

- zero-web 当前还承担 DOM/JS bridge、event loop、fetch、storage 等更广范围。
- 表单页这类纵向场景需要持续追踪多个元素族，容易被更广目标稀释。

### 4.2 范围边界

#### 主责范围

| 模块/目录 | html-compat 职责 |
|---|---|
| `crates/dom` | HTML 解析结果、元素关系、focusability、label/form owner 等纯 DOM 规则 |
| `crates/page-runtime` | 页面交互状态、表单控件状态、与 JS 无关的默认动作核心 |
| `crates/engine/src/js_dom_*` | HTML IDL、事件对象、脚本可观察状态；仅 HTML 行为切片 |
| `apps/renderer` | 将平台事件连接到共享默认动作，不在此层重复定义规范 |
| `apps/webdriver` | click/send keys/active element 等产品自动化入口 |
| `examples/html`、`examples/forms` | 最小产品场景 |
| `docs/goal/html-compat*` | 兼容矩阵、当前轮次、跨线移交 |

#### 明确排除

| 不在范围 | 处理方式 |
|---|---|
| CSS 属性解析、层叠、计算值 | 移交 zero-web 或 rendering-compat |
| block/inline/flex/grid/table 几何 | 移交 rendering-compat |
| 字体、字形、控件视觉外观 | 移交 rendering-compat |
| GPU/CPU 像素一致性 | 仅作为产品验收依赖，不在本线修 |
| 浏览器 chrome/UI | 仅修页面输入路由契约，不改界面设计 |

#### 跨线契约

html-compat 可以要求：

- 每个交互元素能得到稳定节点身份或选择器。
- hit-test 返回正确目标及几何。
- live DOM 状态变更能触发局部或完整新帧。
- browser 能把 platform input 无损传到 renderer。

如果上述契约失败，html-compat 先提供最小复现和断言，再将根因归属模块移交对应开发线；不直接在 CSS/layout/render-foundation 中做旁路修补。

### 4.3 共享文件冲突控制

最高冲突区域是 `crates/engine`、`apps/renderer`、`apps/browser` 和 `crates/protocol`。建议采用以下规则：

| 规则 | 目的 |
|---|---|
| M0 只加测试和目标文档 | 先建立边界，不与生产改动抢文件 |
| 默认动作优先落 `page-runtime` | 降低 renderer 与 JS shim 双写 |
| `protocol` 变更单独提交 | 便于另外两条线 rebase 和审查 |
| browser 只保留平台输入与回执消费 | HTML 规范判断留在页面侧 |
| 每个切片只覆盖一个规范算法 | 避免一次提交横跨多个流域 |
| 共享文件变更前先拉取远端 | 把冲突暴露在编码前 |

> ### 💡 推理分析：为什么最终应收敛到 `page-runtime`
>
> **观察**：当前 retained 表单状态已经位于 `zero-page-runtime` [5]，但默认动作仍分散在 renderer、TabWorker 和 JS shim [3][4]。
>
> **推理**：继续在三个宿主各补一种标签行为，会让多进程、单进程和嵌入式 WebView 逐渐不一致。默认动作本身不属于 renderer IPC，也不属于 JavaScript。
>
> **结论**：M0/M1 先用现有路径锁行为；M2 再把与宿主无关的状态转换和动作决策收敛到 `page-runtime`。这比立即新建 crate 更符合现有架构。

> **📌 来源说明（第 4 章）**
>
> - **一手事实** [3]-[7][11][12]：当前状态与默认动作分布、项目并行经验。
> - **官方规范** [13]-[18]：默认动作属于用户代理的 HTML/UI processing model。
> - **💡 推理**：第三 clone、职责边界和 `page-runtime` 收敛路径。
> - **作者综合**：模块所有权与跨线契约表。

## 5. 里程碑与优先级

### M0：锁定目标页完整验收

目标：不改生产行为，先让第 3.3 节步骤 1-10 成为一个稳定的真实多进程场景。

交付：

- 扩展现有 `form_fixture_physical_clicks_reach_controls_at_windows_scale_factors`，或新增职责更单一的完整场景测试。
- 通过 renderer/WebDriver 查询 live value、checkedness、activeElement、output 文本和导航 epoch。
- 保留四档 DPI 命中测试，但完整语义流程只需在一个 DPI 跑，降低时长和波动。
- 新增 JS-disabled 最小场景，先证明当前失败。

完成门禁：

- 目标页步骤 1-10 全绿。
- 测试先重建独立 renderer，使用 `test-guard`，多进程测试串行。
- 失败信息包含“第几步、目标 selector、预期状态、实际状态”，不只比较 snapshot sequence。

### M1：闭合目标页和紧邻规范差异

优先顺序：

| 优先级 | 切片 | 原因 |
|---|---|---|
| P0 | JS-disabled 默认动作与 listener dispatch 解耦 | 基础 HTML 不应依赖 JS 开关 |
| P0 | 统一两条键盘入口的 `default_allowed` | 消除宿主差异 |
| P0 | `beforeinput` → mutation → `input` 序列和 cancelability | 文本编辑核心契约 |
| P1 | checkbox/radio activation、取消和重复点击语义 | 选择控件高频 |
| P1 | label 点击转发和 focus/activation | 页面现有结构直接使用 label |
| P1 | reset 后 retained state 同步 | 防 DOM 状态与绘制状态分叉 |
| P2 | 更完整的 sequential focus rules | 从目标页扩到通用网页 |

### M2：默认动作共享化

目标：让多进程 renderer、单进程 TabWorker 和 WebView 嵌入路径消费同一套动作决策。

建议最小接口形态：

```rust
pub enum HtmlDefaultAction {
    InsertText { target: String, text: String },
    DeleteBackward { target: String },
    MoveFocus { forward: bool },
    ToggleCheckbox { target: String },
    SelectRadio { target: String },
    ResetForm { form: String },
    SubmitForm { form: String, submitter: Option<String> },
}
```

这只是**作者综合的接口草图**，不是要求立即按该枚举实现。关键约束是：

- 事件派发先返回 `default_allowed`。
- 状态转换不依赖 JS runtime 是否启用。
- JS 只负责观察或取消事件，不持有 UA 默认状态的唯一真相。
- 宿主只执行导航、平台 IME 和帧发布等外部副作用。

### M3：按元素族扩展

| 顺序 | 元素族 | 首批场景 |
|---|---|---|
| 1 | 文本控件 | input/textarea、selection、beforeinput、change-on-blur、IME |
| 2 | 选择控件 | checkbox/radio/select/option、label、disabled |
| 3 | 表单动作 | button、reset、submit、implicit submission、form owner |
| 4 | 导航交互 | a、area、hash、target、download 基础语义 |
| 5 | 折叠/弹层 | details/summary、dialog、popover |
| 6 | 媒体基础 | img/audio/video 的加载状态与事件，不含像素质量 |

每个元素族至少包含：

- 一个纯状态/算法单测。
- 一个 JS 可观察 IDL/事件测试。
- 一个 renderer 或 WebView 集成测试。
- 一个真实 browser/WebDriver 场景。
- 能导入时，附带上游 WPT testharness 用例。

### M4：接入 WPT 交互子集

当前 WPT runner 能解析 `testharness`、`reftest` 等 manifest 类型 [9]，但仓内主力能力仍是 reftest；WebDriver 的 Element Click 也明确处于“存在性验证”，onclick 注入仍标为后续能力 [10]。WPT 官方建议用 `testdriver.js` 的 click/send_keys/actions 自动化真实用户输入 [19]。

因此建议按以下顺序接入：

1. WebDriver 补齐 Element Click、Send Keys、active element 和脚本查询的真实 renderer 桥。
2. 建立最小 `testharness.js` 结果回传。
3. 接 `testdriver.click` 和 `testdriver.send_keys`。
4. 首批只导入与 M1/M3 当前元素族直接相关的测试。
5. 将通过的上游用例常驻账本，避免只跑一次。

不建议在 M0 前先做完整 testdriver：那会把产品缺口、协议缺口和 harness 缺口混成一个大项目，延迟目标页验收。

> **📌 来源说明（第 5 章）**
>
> - **一手事实** [2][3][5][9]-[12]：现有测试、runtime、WPT manifest 和 WebDriver 成熟度。
> - **官方规范** [13]-[19]：各元素族的行为域和 WPT 自动化方式。
> - **⚠️ 假设**：M0 完整场景可能暴露额外产品失败，M1 排序应据真实失败微调。
> - **💡 推理**：先产品场景、再共享化、最后扩 WPT，可降低同时修改 harness 与内核的归因成本。
> - **作者综合**：里程碑、优先级表和接口草图。

## 6. 证据验证 Gate

| 关键结论 | 来源 1 | 来源 2 | 一致性 | 置信度 | 处理 |
|---|---|---|---|---|---|
| 目标页已有可用基线 | renderer fixture [3] | 本次多进程产品实测 + browser 测试 [2] | 一致 | 高 | 直接采用 |
| 当前不能宣称完整兼容 | 页面步骤 [1] | 产品测试覆盖 [2][3] | 一致：步骤多于断言 | 高 | 直接采用 |
| HTML 行为应独立于样式赛道 | HTML processing model [13]-[18] | 当前像素/reftest 与行为测试分离 [2][9][11] | 一致 | 高 | 直接采用 |
| JS-disabled 默认动作是实际缺口 | renderer/browser gate [4] | HTML 控件 UA 行为 [13]-[15] | 冲突，说明实现差异 | 高 | M1 P0 |
| 输入事件语义不完整 | shim `input` 实现 [6] | UI/Input Events [17][18] | 冲突，说明实现差异 | 高 | M1 P0 |
| 键盘入口存在行为分叉 | 两个 renderer handler [4] | UI Events cancelable default action [17] | 冲突，说明实现差异 | 高 | M1 P0 |
| label 激活可能缺失 | 全仓源码搜索无 activation 路径 | HTML label 语义 [20] | 只有负面源码证据 | 中高 | 先加失败测试 |
| 应复用 `page-runtime` 而非新 crate | retained 状态已在该 crate [5] | 三宿主重复逻辑 [3][4] | 一致 | 高 | M2 |
| WPT testdriver 不应早于 M0 | runner 主要解析/reftest 能力 [9] | WebDriver click 仍为 M1 存在性验证 [10] | 一致 | 中高 | M4 |

Gate 结论：核心推荐均有两个独立源码/规范证据；label 激活是唯一主要假设，已明确要求以 driving test 验证后再修改生产代码。

> **📌 来源说明（第 6 章）**
>
> - **一手事实** [1]-[12]：全部仓内证据和本次测试。
> - **官方规范** [13]-[20]：规范对照与测试方法。
> - **⚠️ 假设**：label activation 缺口。
> - **作者综合**：证据矩阵和置信度评级。

## 7. 风险与防护

| 风险 | 概率 | 影响 | 防护 |
|---|---|---|---|
| 与 zero-web 线同时改 engine/renderer | 高 | rebase 冲突、行为重复 | M0 测试先行；共享文件小提交；默认动作尽快下沉 |
| 把布局命中问题误修成 HTML 特判 | 中 | 架构污染、像素回归 | 先最小复现并移交 rendering-compat |
| 单进程测试通过但产品多进程失败 | 高 | 假绿 | 每个元素族保留一个真实多进程场景 |
| 陈旧 renderer/compositor 造成假红 | 已发生 | 误判回归 | 测试入口先构建独立 bin [12] |
| GUI 测试并行争抢资源 | 已发生 | 超时、波动 | 互斥串行 + test-guard [12] |
| 自写近似语义偏离规范 | 中 | 长期兼容债 | 每个动作附规范链接，优先导入 WPT |
| 为“所有标签”一次扩太大 | 高 | 长期无可交付结果 | 按元素族和产品场景切片 |
| 行为修复触发性能退化 | 中 | 输入卡顿 | 复用既有 form-input perf gate [11] |

### 7.1 完成指标

短期指标：

- 目标页完整场景 12 步全部通过。
- JS on/off 两种模式的 UA 默认动作符合各自预期。
- renderer、单进程 worker 和 WebView 对同一动作产生一致状态结果。
- 目标页相关测试在 Linux/Windows CI 不依赖陈旧外部二进制。

中期指标：

- 每个元素族有状态、JS、renderer、browser 四层测试。
- 通过的 HTML testharness/testdriver 用例持续单向增长。
- `apps/renderer` 不再为每个新标签复制默认动作状态机。
- HTML 行为修复不修改 `css-parser/style-system/layout-engine/render-foundation`，除非有明确跨线移交记录。

### 7.2 停止条件

单个切片出现以下情况时暂停该切片并移交，而不是扩大修改面：

- 根因是 inline formatting、控件尺寸或绘制顺序。
- 需要同时改 protocol、browser、renderer 和 engine 且无法给出最小行为契约。
- 没有独立规范或浏览器 Oracle 能判断正确行为。
- 产品场景与上游 WPT 对同一行为存在无法解释的冲突。

> **📌 来源说明（第 7 章）**
>
> - **一手事实** [2][3][9]-[12]：测试分层、既有性能计划和多进程踩坑。
> - **官方规范** [13]-[20]：规范锚点与 WPT 自动化。
> - **💡 推理**：风险概率、停止条件和完成指标。
> - **作者综合**：风险矩阵。

## 8. 最终建议

这条并行线应立即成立，但首个动作不是大规模实现“所有基础标签”，而是把当前表单页升级为可信的 HTML 行为验收基准。

建议首批提交顺序：

1. `docs(html-compat): establish behavior compatibility track`
2. `test(html): cover complete form interaction fixture`
3. `fix(html): decouple default actions from javascript enablement`
4. `fix(input): implement beforeinput and correct input event semantics`
5. `fix(html): unify keyboard default-action routing`
6. `fix(forms): align checkbox radio and label activation`
7. `refactor(page-runtime): share html default actions across hosts`
8. `test(wpt): run initial forms and focus testdriver subset`

其中第 2 个提交是后续全部修改的前置门禁。只有它先失败并准确指出步骤，生产修复才有清晰目标。

> **📌 来源说明（第 8 章）**
>
> - **一手事实** [1]-[12]：当前基线、缺口和工程约束。
> - **官方规范** [13]-[20]：行为正确性和自动化方向。
> - **💡 推理**：提交顺序是按风险和归因成本排序的实施建议。

## 9. ZeroUI 源码调研回补

对 `../ZeroUI` 的源码深潜确认，既有方向中“分级失效、retained 表单状态、统一焦点/IME”已在 ZeroWeb 落地，不应重复建设 [21]。

新增高价值决策：

- M0 增加测试专用 `HtmlScenario`/`PageQuery`，失败携带 step、expected/actual、URL/epoch 和 snapshot sequence。
- M1 先建立 opaque `PageNodeRef` contract，再用 `pressed_target` 固定 press/release paired event identity。
- M2 将 focus owner、retained form state 和 action target 迁到同一 node identity，并强制完整事件闭环。
- M3 增加 focus/IME/reset/input 的确定性短序列压力测试。
- M4 automation 使用 bounded request/reply、单 renderer owner、显式 timeout/shutdown。

明确不复用：

- ZeroUI `TextInputState` 的 UTF-8 byte offset 与 DOM UTF-16 selection contract 冲突。
- ZeroUI TextEditCore 没有 beforeinput/cancel/activation rollback。
- ZeroUI Radio 的组状态由应用 reducer 管理，不等价于 HTML checkedness/reset。
- ZeroUI Widget/semantics tree 不能替代 live DOM 和 WebDriver node identity。

详细证据见 [`ZeroUI 对 HTML 兼容线的可迁移机制`](research-zeroui-lessons-for-html-compat-2026-08-12.md)。

> **来源说明（第 9 章）**
>
> - **一手事实** [21]：ZeroUI 多模块源码、回归测试、learning 和 guarded scoped tests。
> - **💡 推理**：只回补尚未迁入且符合 Web processing model 的机制。

## 参考资料

| 编号 | 来源 | 类型 | 用途 |
|---|---|---|---|
| [1] | [`examples/forms/form-interaction-test.html`](../../examples/forms/form-interaction-test.html) | 一手事实 | 目标页面与用户步骤 |
| [2] | [`apps/browser/src/tests.rs`](../../apps/browser/src/tests.rs) | 一手事实 | 多进程、GPU、DPI 和快照测试 |
| [3] | [`apps/renderer/src/page_scripts.rs`](../../apps/renderer/src/page_scripts.rs) | 一手事实 | 输入、选择、reset、submit helper 与 fixture 测试 |
| [4] | [`apps/renderer/src/main.rs`](../../apps/renderer/src/main.rs) | 一手事实 | 输入路由、焦点、默认动作和导航 |
| [5] | [`crates/page-runtime/src/form_control.rs`](../../crates/page-runtime/src/form_control.rs) | 一手事实 | retained 表单状态 |
| [6] | [`crates/engine/src/js_dom_shim/part01.js`](../../crates/engine/src/js_dom_shim/part01.js) | 一手事实 | 文本插入、删除和 input 事件 |
| [7] | [`crates/dom/src/focus.rs`](../../crates/dom/src/focus.rs) | 一手事实 | Tab 顺序和 focusability |
| [8] | [`crates/webview/src/tests/basic.rs`](../../crates/webview/src/tests/basic.rs) | 一手事实 | 目标页 hit-test |
| [9] | [`tests/wpt-runner/src/manifest.rs`](../../tests/wpt-runner/src/manifest.rs) | 一手事实 | WPT 类型解析能力 |
| [10] | [`apps/webdriver/tests/http_session.rs`](../../apps/webdriver/tests/http_session.rs) | 一手事实 | WebDriver 当前交互成熟度 |
| [11] | [`docs/specs/zeroui-gui-smoothness-migration-spec-rfc.md`](../specs/zeroui-gui-smoothness-migration-spec-rfc.md) | 前期调研/二手来源 | 表单 retained 状态与性能门禁 |
| [12] | [`docs/learnings/platform/multiprocess-binaries-and-parallel-gui-tests.md`](../learnings/platform/multiprocess-binaries-and-parallel-gui-tests.md) | 一手事实 | 多进程测试假红根因与规避 |
| [13] | [WHATWG HTML: Form control infrastructure](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html) | 官方规范 | value、form owner、submit、reset |
| [14] | [WHATWG HTML: The input element](https://html.spec.whatwg.org/multipage/input.html) | 官方规范 | input 类型、checkedness、activation |
| [15] | [WHATWG HTML: Form elements](https://html.spec.whatwg.org/multipage/form-elements.html) | 官方规范 | button、textarea、output、fieldset |
| [16] | [WHATWG HTML: User interaction](https://html.spec.whatwg.org/multipage/interaction.html) | 官方规范 | focus 与 sequential navigation |
| [17] | [W3C UI Events](https://w3c.github.io/uievents/) | 官方规范 | keyboard、focus、composition、input |
| [18] | [W3C Input Events Level 2](https://w3c.github.io/input-events/) | 官方规范 | beforeinput/input 和 inputType |
| [19] | [WPT testdriver.js Automation](https://web-platform-tests.org/writing-tests/testdriver.html) | 官方文档 | click/send_keys/actions 自动化 |
| [20] | [WHATWG HTML: Forms and the label element](https://html.spec.whatwg.org/multipage/forms.html#the-label-element) | 官方规范 | label/control 关联与表单基础语义 |
| [21] | [`ZeroUI 对 HTML 兼容线的可迁移机制`](research-zeroui-lessons-for-html-compat-2026-08-12.md) | 补充源码调研 | identity、capture、Scenario、automation owner |

## 质量审查

- [x] 范围保持在 HTML 行为，不扩到 CSS 样式。
- [x] 核心结论有源码与规范双重证据。
- [x] 当前通过项与未覆盖项分开陈述。
- [x] 旧超时记录与本次定向实测差异已解释。
- [x] 假设项明确标注，没有把负面搜索写成确定事实。
- [x] 推荐方案可直接输入后续 Spec/RFC。
- [x] 报告未要求立即引入新 crate 或大范围重构。
