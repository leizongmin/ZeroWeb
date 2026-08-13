# JS/DOM 原生化 — 双引擎（V8 + QuickJS）原生绑定生产路径收口目标

**版本**: v1.2（v1.1→v1.2：接手 canvas path-objects JS 侧 API 语义，2026-08-13 用户决策）
**日期**: 2026-08-13
**状态**: Active
**执行模式**: 长期无人值守持续执行（`rally run docs/goal/js-dom.md`）
**父目标**: `docs/goal/zero-web.md`（ZeroWeb 总体目标，本目标对应其 Done Criteria §3「JS/DOM」+ P1 主线）
**关联 RFC**: `docs/specs/p1b-v8-native-bindings-rfc.md`（P1b 原生绑定，方案 C 混合 DOM-Node；V8 S0–S5 已 land；TBD-5「原生绑定仅 V8」需更新为双引擎）

> **说明**
> 本文档是 ZeroWeb「JS/DOM 原生化」专项目标执行契约。目标是把页面 JS↔DOM 的桥接从 polyfill 字符串桥（`js_dom_shim` + `__zw_*` 字符串编解码）彻底迁移到**原生绑定**（`dom_bindings/`），使 **V8 与 QuickJS 两个页面引擎的 native 路径都成为唯一生产路径**（双 feature 默认开启并移除 kill-switch），让 ZeroWeb 能真实跑通现代 SPA（React/Vue/Svelte 之一）与 Web Components（customElements/lit 之一），并通过上游 WPT `dom`/`jsdom` 真实用例建立通过率基线。本文定义 Mission、边界、Done Criteria、执行协议和文档治理规则，供后续每个 `rally run` session 作为稳定输入。**不要在每轮执行里重写本入口文档**；日常进展、evidence、active milestone 更新写入 `master.md`。
>
> **▶ v1.1 双引擎扩展（2026-08-13 用户决策）**：v1.0 误以 QuickJS 为「仅扩展沙箱非页面引擎」（沿用 RFC TBD-5 措辞）。代码核实推翻此假设——QuickJS 是 feature-gated 的**页面引擎之一**（`webview.rs:1215 #[cfg(feature="quickjs")]` 与 V8 对称构造页面沙箱；`apps/browser/src/tab_js_worker.rs:276` 同款；Makefile CI 矩阵 `make test`/clippy 都跑 `--no-default-features --features quickjs`）。其页面 DOM 路径**仅走 polyfill 字符串桥**（`QuickJSSandbox::register_callback` 注册 `__zw_*`，与 V8 共用同一 `js_dom_shim`），而原生绑定 escape-hatch `Sandbox::install_native_bindings` 默认返 `false`、QuickJS **不实现**——故若不覆盖 QuickJS，其页面引擎路径将**永远停在 polyfill 字符串桥**（正是本目标要消除的状态）。用户裁决：**方案 A 全 feature 等价**——V8 与 QuickJS 的 native 路径都达到 production-ready（QuickJS 需从零实现 rquickjs 原生绑定，镜像 V8 S0–S5 切片）。新增 DC-7 + M6。RFC TBD-5 由「仅 V8」更新为「双引擎」。
>
> **▶ 拆分动机（2026-08-13 用户决策）**：父目标 P1「DOM/JS Bridge 原生化」已推进到 P1a 实质完成 + P1b S0–S5 native bindings 已 land，但 native 路径**默认仍关闭**（`ZW_NATIVE_DOM` 默认关，生产仍走 polyfill 字符串桥），SPA/Web Components 不可用是父目标 Done Criteria §1.3/§3 剩余的主要阻塞。用户要求把这条主线拆成独立专项收敛到 production-ready：**P1a follow-up + P1b 全量原生化（L2/S6/S7 + default-on + 删 kill-switch）**。理由：① JS/DOM 桥是父目标唯一一条「能力未达成而非环境受限」的阻塞（P3 GPU/Display 受限于物理环境），是产品可用性破局点；② 边界清晰——`dom_bindings/` + `js_dom_shim` 的 DOM/事件/桥接段，与 CSS 渲染（rendering-compat）、canvas（canvas-2d）、表单事件（html-compat）三条并行流工作面正交，碰撞可经 run-rules §9 管理；③ 有独立验收面（真实 SPA/WC 端到端 + WPT `dom`/`jsdom` 上游用例）。
>
> **▶ 基线事实（2026-08-13 实测，详见 [js-dom/master.md](js-dom/master.md)）**：
> - **P1a 已实质完成**：HTML spec event loop（rAF 帧驱动 / requestIdleCallback / setTimeout 真延迟 / microtask checkpoint）、fetch 真实化（method/headers/body/AbortSignal 走 net crate）、MutationObserver/IntersectionObserver/ResizeObserver（尽力而为单发）。剩余 = 非阻塞 follow-up（IO/RO 持续 tick、纯 host 侧 MO 触发）。
> - **P1b native bindings S0–S5 已 land**：Node/Element/Document/EventTarget/Event 原生 + customElements/Web Components 五件套 + lifecycle 四件套 + Live Document 共享（L1，`Rc<RefCell<Document>>`，native 写触发重渲染）。native 读比 polyfill 快 ~15.6x（RFC §4 bench：215ns vs 3.36µs）。`dom_bindings/` 共 19 文件（node/element/document/html_element/event_target/event/custom_elements/gc/css_style_declaration/dom_token_list/dataset/namednodemap/factories 等）。
> - **生产路径仍走 polyfill 桥**：`ZW_NATIVE_DOM` 默认关，`js_dom_shim/part01-06.js`（~810KB）+ `js_dom_bridge.rs`（~122KB）+ `dom_bridge.rs` 为权威路径。这是本目标要消除的核心状态。
> - **WPT dom 分类无真实上游用例**：`test_cases_js_dom.rs` 全部为内建用例（`render_completes`/`js_executes_ok` smoke），0 上游导入，无通过率基线。
> - **QuickJS 页面引擎 native = 真空（v1.1 核实）**：`crates/script-sandbox/src/quickjs_runtime.rs` 经 `register_callback` 注册 `__zw_*` 走 polyfill 桥（与 V8 共用 `js_dom_shim`），无任何原生 DOM 绑定；`Sandbox::install_native_bindings` 默认 `false`，QuickJS 未实现 escape-hatch。CI 已强制跑 `--features quickjs` 矩阵（Makefile QUICKJS_CLIPPY_CRATES/QUICKJS_TEST_CRATES），但 dom_bindings 相关 quickjs 测试点 14 个 vs v8 73 个——native DOM 这块 quickjs 是真空。本目标 M6 从零补齐。
>
> **▶ v1.2 接手 canvas path-objects（2026-08-13 用户决策）**：canvas-2d goal 的 `html/canvas/element/path-objects` 剩余工作（JS 侧 API 语义面）合并入本 goal 统一执行。canvas 流已完成 roundRect 基础（角对半径/比例缩放/非有限守卫/16 段椭圆弧，commit `d0874c28`），并已从 `CANVAS_TEST_SUBDIRS` 移除 path-objects 目录（canvas 流不再跑）。**接手待办**：roundRect 批量 panic（NaN 排序，canvas 流观察到 wpt-runner 崩溃级，接手第一优先级——需重新导入用例后复现定位）、roundRect DOMPoint 断言精度（~26 用例）、arc/arcTo/quadratic/bezier/isPointIn* 形状精度、roundrect 语义校验（异常/边界）。**核实修正**：canvas-2d master.md 交接记录说「用例已导入 205 文件」，但实测 `tests/wpt-runner/wpt-data/html/canvas/element/path-objects/` 目录**为空**（canvas 流移除 SUBDIRS 后用例未留在仓库）——故 js-dom 流接手时**须先重新导入** path-objects 用例，不能假设用例已在。新增 DC-8 + M8。运行入口：`zero-wpt-runner testharness-canvas path-objects`（用例导入后按需重新加入 `CANVAS_TEST_SUBDIRS`）。

---

## Mission

把页面 JS↔DOM 桥接从 polyfill 字符串桥彻底迁移到**原生绑定**，使 **V8 与 QuickJS 两个页面引擎的 native 路径都成为唯一生产路径**（双 feature 默认开启并移除 kill-switch），让 ZeroWeb 能真实跑通现代 SPA（React/Vue/Svelte 之一）与 Web Components（customElements/lit 之一），并通过上游 WPT `dom`/`jsdom` 真实用例建立通过率基线。

**Mission 不变式的三层含义**（每轮执行都必须向这三层收敛）：

1. **能力层**：ZeroWeb 能端到端加载并交互式运行真实 SPA 框架页面与 Web Components 页面（不是只渲染静态页），这是父目标 Done Criteria §1.3「执行页面 JS + 基础 DOM 操作」对 SPA 的实质达成。
2. **架构层**：原生 DOM node 对象（V8 = `NodeId` internal slot；QuickJS = rquickjs 原生对象持有 `NodeId`）是 JS 持有 DOM 引用的**单一权威形态**；polyfill 字符串桥（`__zw_*` 回调 + `DomMutation` + Proxy selector/handle）在**两个引擎**的生产路径都被移除或萎缩为死代码。
3. **验证层**：上游 WPT `dom`/`jsdom` 真实用例有导入、有通过率基线、有按子分类追踪；**V8 与 QuickJS 双 feature 均验证**；不是用内建 inline smoke 充数。

**双引擎对等原则**：QuickJS 作为 feature-gated 页面引擎（`--features quickjs`），其 native 路径必须与 V8 **行为等价**（同一套 driving 测试 + polyfill vs native A/B 对照门在两 feature 下都跑），不允许「V8 优先 QuickJS 凑合」。default-on 与 kill-switch 移除对**两个 feature 同时生效**。

**分阶段里程碑校准**（数字在首次导入/首跑后按实测校准）：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **V8 polyfill-live 合一 + 高层 API 去字符串** | V8 侧 L2（polyfill 桥改读 live Document）+ S6（Fetch/Observer/FontFaceSet 等 shim 改调 native node）→ native(A)=polyfill(B)=renderer(C) 三方合一 |
| 第二阶段 | **SPA/WC 端到端跑通 + WPT dom 基线** | 真实 SPA 框架页 + Web Components 页端到端验收通过（V8 先行）；上游 WPT dom 用例导入 + 通过率基线建立 |
| 第三阶段 | **QuickJS native 移植（双引擎对等）** | QuickJS 从零实现 rquickjs 原生绑定（镜像 V8 S0–S5），双 feature A/B 行为等价 |
| 第四阶段 | **双引擎 default-on + 收尾** | `ZW_NATIVE_DOM` 双 feature 默认开启、移除 kill-switch、S7 删 polyfill 桥死代码、shim 萎缩；**双引擎** native 为唯一生产路径 |

**执行方式**：**轻量修复优先、永不停、深结构护栏**（借鉴 rendering-compat / canvas-2d 裁决）——每轮推进一个可独立 land 的切片，遇需用户决策项（如 default-on 这个改 Mission 级单向门、或深结构跨面改）记入「待用户决策」清单并跳过，继续下一个轻量修复。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| P1b V8 native bindings 收尾 | V8 侧 L2 polyfill-live 合一；S6 高层 API（Fetch/Observer/FontFaceSet/事件循环）改调 native node、去 `__zw_*` 字符串 ser/deser；S7 删 polyfill 桥死代码、shim 萎缩 | RFC §3.6/§3.7 + §4 切片定义 |
| **QuickJS native 移植（v1.1 新增）** | QuickJS 从零实现 rquickjs 原生 DOM 绑定（镜像 V8 S0–S5：Node/Element/Document/EventTarget/customElements/Live Document），实现 `Sandbox::install_native_bindings` escape-hatch；双 feature polyfill vs native A/B 行为等价 | QuickJS 当前 native = 真空；TBD-5 由「仅 V8」更新为双引擎 |
| 双引擎 default-on 生产路径 | `ZW_NATIVE_DOM` **双 feature** 默认开启为生产路径；default-on 后**移除 kill-switch**（双引擎 native 为唯一生产路径，polyfill 桥死代码删除）；default-on 前每切片仍 kill-switch + 全量回归守稳 | 改 Mission 级单向门（rule 11），default-on 动作本身记「待用户决策」清单，goal 把它定为收敛目标而非自动触发 |
| P1a follow-up | IntersectionObserver/ResizeObserver 持续 tick（Slice 2b，依赖 host render-loop tick）；纯 host 侧 DOM 变更触发 MutationObserver；event loop 每-task microtask checkpoint（spec 严格化） | 非阻塞 follow-up，随原生化主线附带推进 |
| customElements/Web Components 完整化 | attributeChangedCallback 全化 + observedAttributes 完整 parity；lit/stencil 等真实 CE 库集成测试；Web Components 端到端验收（V8 先行，QuickJS 随 M6 对齐） | RFC §3.5.1 S5 剩余后续项 |
| SPA 框架端到端验收 | 真实加载并交互运行 React / Vue / Svelte 之一（至少其一）的代表性页面，验证 reconciliation、hydration、事件、状态更新 | 父目标 Done Criteria §1.3 对 SPA 的实质达成 |
| WPT dom 上游基线 | 从上游 WPT 仓库导入 `dom/` + `jsdom/`（若范围合适）范围内真实用例，建立按子分类的通过率报告（文本 + JSON），记录基线 | 当前 dom 分类 0 上游导入，本目标建立基线并持续扩展 |
| **Canvas path-objects API 语义（v1.2 接手，见 DC-8/M8）** | 接手 canvas-2d 流移交的 `html/canvas/element/path-objects` JS 侧 API 语义工作：roundRect（panic 修复 + DOMPoint 断言精度）、arc/arcTo/quadratic/bezier/isPointIn* 形状精度、roundrect 语义校验（异常/边界） | WPT 用例需 js-dom 流重新导入（canvas 流已从 `CANVAS_TEST_SUBDIRS` 移除 path-objects，目录实际为空）；运行入口 `zero-wpt-runner testharness-canvas path-objects` |
| 单元测试与覆盖率 | dom_bindings 每项迁移/修复带单测；polyfill vs native A/B 对照门（每切片行为等价，**双 feature 均跑**）；覆盖率持续提升、不退化（dom_bindings 为新模块，补齐独立 coverage 口径） | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **CSS 渲染兼容性**：属 rendering-compat 目标域（字体栈/布局/绘制管线差异），本目标不碰
- **Canvas 2D 像素/光栅/合成正确性**：属 canvas-2d 目标域（offscreen/createImageBitmap options/compositing/pixel-manipulation 等）；**本目标只接手 path-objects 的 JS 侧 API 语义**（roundRect/arc/arcTo/bezier/isPointIn* 行为），其余 canvas 子目录（`CANVAS_TEST_SUBDIRS` 当前 7 个）仍归 canvas-2d 流
- **表单元素默认动作/交互语义**：属 html-compat 目标域（focus/activation/checkedness/submit）；本目标只经 DOM 事件与原生 node 提供基础，不旁路修补表单语义
- **V8/QuickJS 引擎替换**：rusty_v8 与 rquickjs 仍是两个页面引擎后端，不替换引擎本身；QuickJS 的**扩展脚本沙箱**用法（非页面路径）与本目标无关
- **网络栈/存储/WASM 行为本身**：net/storage/wasm-sandbox crate 行为不变，仅 JS 侧桥（fetch/Storage/WebAssembly JS API）因去字符串 ser/deser 间接受益
- **GPU/Display 验证**：父目标 Done Criteria §4 剩余项，受限于物理环境，非本目标
- **新 crate 或大规模新依赖引入**：最小化新依赖，仅在必要时引入 MIT/Apache-2.0/BSD 许可证兼容 crate

### 工作面切分（与并行流边界，run-rules §8/§9/§10）

本目标**只改 DOM/事件/桥接段**：

| 工作面 | 归属 | 本目标是否改 |
|--------|------|--------------|
| `crates/engine/src/dom_bindings/` | **本目标** | 是（核心） |
| `crates/engine/src/js_dom_bridge.rs` + `dom_bridge.rs` | **本目标** | 是（polyfill 桥萎缩/合一） |
| `js_dom_shim/part01.js`（event loop / rAF / Observer / 定时器）+ `part02.js`（AbortSignal / Fetch shim）+ `part03.js`（customElements registry / lifecycle） | **本目标** | 是（DOM/事件/桥接段） |
| `js_dom_shim/part04.js` / `part05.js` 的 **path-objects API 段**（roundRect/arc/arcTo/bezier/isPointIn* 的 JS 桥） | **本目标**（v1.2 接手，见 DC-8） | 是 |
| `js_dom_shim/part04.js` / `part05.js` 的 **其余 canvas 段**（offscreen/createImageBitmap/compositing/pixel 等） | **canvas-2d 流** | 否（碰撞先 `git log` 核对，转零碰撞面） |
| `crates/canvas/src/path.rs`（round_rect/arc/arc_to 等 Rust 路径几何实现） | **本目标**（path-objects API 语义对应实现） | 是（仅 path-objects 相关） |
| `js_dom_shim` 的表单事件 / 默认动作段 | **html-compat 流** | 否（同上） |
| `crates/webview`（native_dom 接线 / default-on 开关） | **本目标** | 是（生产接线） |
| `apps/browser`（TabWorker / js_worker） | **本目标**（与零-web 流共享） | 是，但遵守零-web 流工作面（engine/dom/script-sandbox/net/webview + zero-web/*） |

**碰撞管理**：S6/S7 会改写 `js_dom_shim`，开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/ crates/engine/src/js_dom_bridge.rs` 核对；若 canvas-2d 或 html-compat 流最近编辑过 part04/05，先做零碰撞面（dom_bindings 本体、part01/02/03、WPT 导入、Rust 侧），碰头段等其告段落（rule 9 碰头信号）。共享面（engine、Cargo.lock、imported-tests.txt、wpt-data）冲突 = 碰头信号，暂停一边记入 master.md，不硬解（rule 9）。

### 依赖约束

- **rusty_v8 150.x（V8 后端）**：`ObjectTemplate` internal-field + `FunctionTemplate` 继承链 + `Weak` handle GC（TBD-1/TBD-2 已 S0 验证，RFC §6）
- **rquickjs 0.7（QuickJS 后端，v1.1 新增）**：提供 `Ctx`/`Object`/`IntoJs`/`from_js`/`Class`/`Accessor`/`Getter`/`Setter` 等原生绑定能力（`features = ["bindgen"]`）；M6 需验证 rquickjs 原生对象持有 `NodeId` + 生命周期/GC 管线（镜像 V8 TBD-1/TBD-2 的 rquickjs 等价物），标 TBD 待 M6 首切片验证
- **许可证**：仅接受 MIT / Apache-2.0 / BSD
- **无新 MPL 依赖**（父目标硬约束）

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。任何一项未满足，必须输出 `CONTINUE: <下一步>`。

### DC-1: 原生 DOM 为双引擎唯一生产路径（架构闭环）

- [ ] `ZW_NATIVE_DOM` **双 feature 默认开启**：V8 与 QuickJS 两个 feature 的 `WebViewConfig.native_dom` 均默认 `true`，生产 `run_page_scripts` 路径在两个 feature 下都默认安装并使用原生绑定
- [ ] kill-switch 已移除：`ZW_NATIVE_DOM` env 与 `native_dom=false` 回退路径作为死代码删除（default-on 后双引擎 native 为唯一生产路径，无一键回退——用户已决策）
- [ ] V8 L2 polyfill-live 合一完成：polyfill 桥（`__zw_*` 回调）从 re-parse `dom_html` String 改读共享 live Document（`Rc<RefCell<Document>>`），native(A)=polyfill(B)=renderer(C) 三方合一，无独立快照
- [ ] V8 S6 高层 API 去字符串完成：Fetch / Observer / FontFaceSet / 事件循环等高层 API 改调 native node 方法，DOM 操作热路径不经 `__zw_*` String ser/deser
- [ ] S7 收尾完成：polyfill 桥死代码（无调用方的 `__zw_*` 回调 + `DomMutation` 变体 + Proxy selector/handle）删除，`js_dom_shim` 体量显著萎缩（相对基线 810KB）

### DC-2: 真实 SPA / Web Components 端到端跑通

- [ ] 至少一个现代 SPA 框架（React / Vue / Svelte 之一）代表性页面可真实加载、渲染、交互（事件触发、状态更新、reconciliation、hydration），非仅静态渲染（**至少 V8 feature 验证通过**；QuickJS feature 在 DC-7 达成后跑同一验收页对齐）
- [ ] 至少一套 Web Components（customElements + lit/stencil 之一或原生 customElements）代表性页面可真实运行：自定义元素定义/实例化、connectedCallback/disconnectedCallback/attributeChangedCallback lifecycle、Shadow DOM 基础
- [ ] SPA/WC 验收页面作为常驻 e2e 测试资产化（进入 `tests/integration` 或 `apps/browser` 测试，`make test` 内运行），有可复现的断言

### DC-3: WPT dom 上游基线

- [ ] 从上游 WPT 仓库（`https://github.com/web-platform-tests/wpt`）导入 `dom/`（及范围合适的 `jsdom/`）真实用例，进入 `tests/wpt-runner`，**不允许用内建 inline 用例替代或充数**
- [ ] 建立按子目录/子分类的通过率报告（文本 + JSON），记录基线（首跑数字即基线，后续持续提升）
- [ ] 每项迁移/修复的 driving WPT 用例经 `make import-wpt TEST=<上游用例> REF=<参照> NOTE="Rxxxx 修复"` 资产化并记入 `imported-tests.txt` 账本（CLAUDE.md 测试资产化规则）
- [ ] 通过率报告持久化到 `docs/goal/js-dom/evidence/`，历史可追溯

### DC-4: 测试与质量不可退让

- [ ] `cargo test` 全绿（含 dom_bindings / engine / webview / integration 全链路），零失败；无遗留红灯（flaky / 历史遗留失败当作当前任务修到稳定）
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] dom_bindings crate 覆盖率**持续提升、不退化**（新代码必带测试；既有 polyfill 行为在迁移期由 A/B 对照门守住）
- [ ] 每项迁移/修复有对应单元测试 + driving WPT 用例资产化 + polyfill vs native A/B 行为对照

### DC-5: 性能不退化

- [ ] perf-gate（`make bench-gate`）在 default-on 后全 NEW/PASS 或退化在预算内（`make test` + scoped reftest 不覆盖 JS 桥热路径，故 default-on 切片**必须**额外跑 perf-gate + `make product-smoke`，net≥0 才 land）
- [ ] JS→DOM 桥调用基准（RFC §4 S0 bench：native 单次读取 ~ns 级）持续记录趋势，default-on 后不退化

### DC-6: 文档治理就位

- [ ] `master.md` 内部自洽（active milestone / done criteria / coverage / Latest Evidence 互不矛盾）
- [ ] `archive/` 已建立，完成的里程碑/切片记录到 archive
- [ ] RFC `p1b-v8-native-bindings-rfc.md` 的剩余切片状态与 master.md 实际推进一致（TBD 项关闭/更新同步）；**TBD-5 由「原生绑定仅 V8」更新为「双引擎 V8+QuickJS」**

### DC-7: QuickJS 原生绑定等价（v1.1 新增）

> QuickJS 当前 native = 真空（`Sandbox::install_native_bindings` 默认 `false`、QuickJS 不实现）。本 DC 是双引擎对等原则的硬收敛项。

- [ ] QuickJS（rquickjs）实现原生 DOM 绑定，镜像 V8 S0–S5：Node/Element/Document/HTMLElement/EventTarget/Event/customElements（五件套 + lifecycle 四件套）+ Live Document 共享（rquickjs 原生对象持有 `NodeId`，经 `QuickJSSandbox::install_native_bindings` escape-hatch 安装到页面 Context）
- [ ] QuickJS 原生绑定的 GC/生命周期安全验证（镜像 V8 TBD-1/TBD-2：rquickjs 原生对象与 Rust `NodeId` 的引用关系 + stale 校验 + 节点移除后不悬垂/泄漏）
- [ ] **双 feature polyfill vs native A/B 行为等价**：同一套 driving 测试（既有 dom_bindings 单测 + driving WPT 用例）在 `--features v8` 与 `--features quickjs` 两个矩阵下行为一致；CI 已强制跑 quickjs 矩阵（Makefile `QUICKJS_TEST_CRATES`/`QUICKJS_CLIPPY_CRATES`），M6 补齐 dom_bindings 相关 quickjs 测试点（当前 14 vs v8 73 的差距）
- [ ] QuickJS 页面引擎路径在 default-on 后走 native（不再经 `__zw_*` polyfill 桥），与 V8 生产路径对等

### DC-8: Canvas path-objects JS 侧 API 语义（v1.2 接手）

> canvas-2d 流移交的 `html/canvas/element/path-objects` JS 侧 API 语义工作。本 DC 只覆盖 **path-objects 一个子目录**的 JS 侧行为，不涉 canvas 其余像素/光栅/合成正确性（仍归 canvas-2d 流）。

- [ ] **path-objects 用例重新导入**：从上游 WPT 导入 `html/canvas/element/path-objects` 用例到 `tests/wpt-runner/wpt-data/`，重新加入 `CANVAS_TEST_SUBDIRS`，建立通过率基线（canvas 流已移除，目录实测为空）
- [ ] **roundRect 批量 panic 修复**（接手第一优先级）：canvas 流观察到 wpt-runner 崩溃级 panic（NaN 排序 / scale 归一化后复现，疑似负 w/h 或 NaN radii 组合）——导入用例后复现、定位根因、修复到稳定可重复（CLAUDE.md「不允许留给下一轮」）
- [ ] **roundRect DOMPoint 断言精度**（~26 用例）：shim `"p<x>,<y>"` 编码 + host 配对解析已通，但渲染偏离（fill 扫描线与椭圆弧交点配对 / 16 段精度）——对齐到上游期望
- [ ] **arc/arcTo/quadratic/bezier/isPointIn* JS 侧 API 语义对齐**（形状精度 + API 行为），driving 用例经 `make import-wpt` 资产化并记入 `imported-tests.txt`
- [ ] **roundrect 语义校验**（badinput/negative/toomany 抛异常、winding/zero 边界）与上游 spec 一致
- [ ] path-objects 通过率报告持久化到 `docs/goal/js-dom/evidence/`，历史可追溯

---

## Current Proven Baseline

截至 2026-08-13（基线信息来自 `docs/goal/zero-web/master.md` R3404 轮 + 代码实测，详见 [js-dom/master.md](js-dom/master.md)）：

- **P1a — 实质完成**：HTML spec event loop（rAF 帧驱动 + requestIdleCallback + setTimeout/setInterval 真延迟 + microtask checkpoint 经 `_defer`）、fetch 真实化（`FetchRequest`/`FetchResponse` byte-wire + AbortSignal，走 net crate）、MutationObserver/IntersectionObserver/ResizeObserver（尽力而为单发）。e2e 基线：`tab_js_worker` fetch 端到端（post/method/headers/body/abort）、定时器、MutationObserver 五连测。
- **P1b native bindings — S0–S5 已 land，默认关**：`crates/engine/src/dom_bindings/`（19 文件）= Node/Element/Document/HTMLElement/EventTarget/Event/custom_elements/gc/css_style_declaration/dom_token_list/dataset/namednodemap/factories。Live Document 共享（L1，`Rc<RefCell<Document>>`，native 写经 `sync_render_after_native_dom` 触发重渲染）。customElements 五件套（define/get/getName/whenDefined/upgrade）+ lifecycle 四件套（connected/disconnected/attributeChanged/upgrade）。native 读 ~15.6x 快于 polyfill（bench：215ns vs 3.36µs）。kill-switch `ZW_NATIVE_DOM` 默认关。
- **生产路径仍走 polyfill 桥**：`js_dom_shim/part01-06.js`（~810KB）+ `js_dom_bridge.rs`（~122KB，30+ `__zw_*` 回调）+ `dom_bridge.rs`（~69KB）+ `dom_bindings/computed_style.rs`（~158KB）为权威路径。
- **测试基线**：workspace ~13,000+ 测试全绿，行覆盖率 95.46%（函数 96.94%、区域 94.88%），clippy 零警告。dom_bindings 尚无独立 coverage 口径（纳入本目标首个 milestone）。
- **WPT dom 分类**：23 分类之一，但 `test_cases_js_dom.rs` 全部为内建 smoke 用例（`render_completes`/`js_executes_ok`），**0 上游真实导入**，无通过率基线。
- **RFC**：`docs/specs/p1b-v8-native-bindings-rfc.md` v0.2，S0–S5 + L1 已 land；S6/S7 + L2/L3 + default-on 为本目标剩余；**TBD-5「原生绑定仅 V8」需更新为双引擎（v1.1 决策）**。
- **QuickJS 页面引擎 native = 真空（v1.1 核实）**：`crates/script-sandbox/src/quickjs_runtime.rs` 经 `register_callback` 注册 `__zw_*` 走 polyfill 桥（与 V8 共用 `js_dom_shim`），无任何原生 DOM 绑定；`Sandbox::install_native_bindings` 默认 `false`、QuickJS 未实现 escape-hatch。QuickJS 是 feature-gated 页面引擎（`webview.rs:1215` / `tab_js_worker.rs:276` 与 V8 对称），CI 已强制 `--features quickjs` 矩阵（Makefile `QUICKJS_CLIPPY_CRATES`/`QUICKJS_TEST_CRATES`），但 dom_bindings 相关 quickjs 测试点 14 vs v8 73——native DOM 这块 quickjs 是真空。本目标 M6 从零补齐。

**核心缺口（本目标要消除）**：
1. V8 native 路径默认关 → 生产仍走 polyfill 字符串桥（SPA/WC 不可用根因）
2. V8 polyfill 桥仍 re-parse String 快照，三方 Document（A native / B polyfill / C renderer）未合一（L2 未做）
3. V8 高层 API（Fetch/Observer）仍经 `__zw_*` String ser/deser（S6 未做）
4. polyfill 桥死代码未清理、shim 未萎缩（S7 未做）
5. WPT dom 上游用例 0 导入，无通过率基线
6. 无真实 SPA/WC 端到端验收资产
7. **QuickJS 页面引擎 native 完全缺失**（v1.1 新增）：QuickJS 无原生 DOM 绑定，页面 DOM 路径仅走 polyfill 桥，无法 default-on（M6 补齐）

---

## Single Active Milestone

> **本节定义「当前唯一活跃里程碑」。执行 agent 每轮进入时，先读 `master.md` 确认当前 active milestone 的实际进度，再决定本轮推进哪个切片。不要同时开多个 active milestone。**

### 当前活跃里程碑：**M0 — 基线建立 + polyfill-live 合一起刀（L2/S6 入口）**

**目标**：建立本目标独有的验证与度量基线（dom_bindings coverage 口径、**双 feature 可参数化的** polyfill vs native A/B 对照门、上游 WPT dom 导入空集确认、QuickJS native 真空核实），并完成 V8 侧 L2（polyfill 桥改读 live Document）的首个可独立 land 切片，使 native(A) 与 renderer(C) 在 live Document 上对齐，为后续 V8 S6/S7（M2/M5）、QuickJS native 移植（M6）、双引擎 default-on（M7）铺路。

**首轮必须完成的入口动作**（Must-Complete-First-Round，详见「首轮进入检查清单」）：
1. 探索 `dom_bindings/` + `js_dom_bridge.rs` + `js_dom_shim/` 当前事实，确认 RFC §3.7 L1/L2 切片定义与代码现状一致
2. 创建/更新 `master.md`（当前状态评估 + 首个切片计划 + 测试基线 + 缺口清单 + 待用户决策清单）
3. 确认 `docs/goal/js-dom/archive/` 与 `evidence/` 目录存在
4. 补齐 dom_bindings 独立 coverage 口径（scripts/check-coverage.sh 能单独报告 dom_bindings 行覆盖率）
5. 建一个 polyfill vs native A/B 对照门测试骨架（验证同一 DOM 操作两条路径行为等价）
6. 选定首个可独立 land 切片并直接动手推进（不许停在「文档框架已建好」）

**首个切片候选**（kill-switch 仍开 → 可安全 land，零生产回归）：
- **L2-first**：polyfill 桥 `__zw_*` 回调中读 DOM 的路径，从 re-parse `dom_html` String 改读 `pipeline.cached_doc` 共享句柄（RFC §3.7 L2，高风险，需全量 WPT + 既有 dom_bridge 测试 + reftest 守）——**首个切片建议先做 L2 的一个最小子集**（如只读类 getter：`getElementById`/`querySelector` 读 live Document，写仍走旧路径），验证三方合一管线，再逐步扩大。

**完成判据**（M0 结束、转 M1 的条件）：
- master.md 就位、archive/evidence 目录就位、coverage 口径就位、A/B 对照门骨架就位、L2 首切片 land + 全量回归零退化（`make test` + `make product-smoke`）

---

## Ordered Next Milestones

> M0 完成后按序推进。每个 milestone 切成可独立 land 的切片（kill-switch + A/B 对照门 + 全量回归）。深结构护栏：default-on（M7）是改 Mission 级单向门，记「待用户决策」清单，goal 把它定为收敛目标。**双引擎对等**：M1–M5 先以 V8 为权威路径推进；M6 完成 QuickJS native 移植后，M7 的 default-on 对双 feature 同时生效。**M8 独立并行**：canvas path-objects（v1.2 接手）工作面与原生绑定主线基本不重叠，可作为轻量填充在任意轮次穿插推进，不强制排在 M7 之后。

### M1 — polyfill-live 合一（L2 完整）

**目标**：polyfill 桥全部 `__zw_*` 回调改读 live Document（`Rc<RefCell<Document>>`），native(A)=polyfill(B)=renderer(C) 三方合一，无独立快照。

**切片建议**：
1. L2 只读 getter 族（getElementById/querySelector/getAttribute 等读 live Document）
2. L2 写入/子树 mutation 族（setAttribute/appendChild/removeChild 经 live Document）
3. L3 清理：移除 `cached_html` String 路径 + re-parse 死代码 + A/B 快照分支

**验收**：全量 WPT + 既有 dom_bridge 测试 + reftest 全绿；A/B 对照门证明 native 与 polyfill 行为等价。

### M2 — 高层 API 去字符串（S6）

**目标**：shim 的 Fetch/Observer/FontFaceSet/事件循环等高层 API 改调 native node 方法，DOM 操作热路径不经 `__zw_*` String ser/deser。

**切片建议**：按 API 族逐个迁移（Fetch → MutationObserver → IntersectionObserver → ResizeObserver → FontFaceSet → 事件 target/mutation target 用 native node），每族 A/B 对照 + WPT。

**验收**：高层 API 行为与迁移前等价（既有 R2945–R2953 测试 + WPT）；shim 中 `__zw_*` 调用点显著减少。

### M3 — 真实 SPA / Web Components 端到端验收

**目标**：建立并跑通真实 SPA 框架页 + Web Components 页的端到端验收资产。

**切片建议**：
1. Web Components 端到端（customElements + lifecycle + Shadow DOM 基础，lit 或原生 CE）
2. SPA 框架端到端（React / Vue / Svelte 之一代表性页：hydration + 事件 + reconciliation）
3. customElements 收尾：attributeChangedCallback 全化 + observedAttributes 完整 parity（RFC §3.5.1 S5 剩余）

**验收**：DC-2 全项满足；验收资产进入 `tests/integration` / `apps/browser` 测试，`make test` 内运行。

### M4 — WPT dom 上游基线建立与扩展

**目标**：从上游 WPT 导入 `dom/`（及范围合适的 `jsdom/`）真实用例，建立按子分类的通过率基线并持续扩展。

**切片建议**：
1. 上游用例导入 + 分类通过率报告（零源码改动，纯资产）+ 失败聚类分析
2. 按聚类驱动修复（API 语义？JS 接线？Rust 层 bug？），每修 net≥0 即 land，driving 用例经 `make import-wpt` 资产化

**验收**：DC-3 全项满足；通过率报告持久化 `evidence/`。

> **注**：M4 可与 M1–M3 早期并行（导入本身零源码改动），但修复须以 M1/M2 的 live Document + native 路径为权威（避免在 polyfill 桥上做长期修复再被废弃）。

### M5 — V8 default-on + 收尾（L3 + S7）

**目标**：V8 侧 `ZW_NATIVE_DOM` 默认开启、移除 kill-switch（V8 路径）、S7 删 polyfill 桥死代码、shim 萎缩，**V8 native 为唯一生产路径**。

**切片建议**：
1. V8 default-on：`WebViewConfig.native_dom`（v8 feature）默认 `true`，全量回归 + product-smoke + perf-gate net≥0 守稳（**改 Mission 级单向门，记「待用户决策」清单，等用户点名后 land**）
2. S7：删无调用方的 `__zw_*` 回调 + `DomMutation` 变体 + Proxy selector/handle（V8 不再经 polyfill 桥的部分），shim 体量压缩
3. L3 清理：移除 `cached_html` String 路径 + re-parse 死代码 + A/B 快照分支

**验收**：DC-1（V8 部分）+ DC-5（V8）满足；kill-switch 移除后 V8 native 为唯一生产路径，全量回归 + product-smoke + perf-gate 全绿。

> **注**：M5 只收敛 V8 路径；QuickJS 的 default-on + kill-switch 移除在 M7 完成（M6 先补 QuickJS native）。polyfill 桥的彻底删除（含 QuickJS 还在用的部分）在 M7 之后。

### M6 — QuickJS 原生绑定移植（双引擎对等，v1.1 新增）

**目标**：QuickJS 从零实现 rquickjs 原生 DOM 绑定，镜像 V8 S0–S5 切片，使 QuickJS 页面引擎路径具备与 V8 行为等价的 native 能力。

**切片建议**（镜像 V8 切片命名，加 `q` 后缀）：
1. **S0q 骨架 + PoC**：`crates/script-sandbox/src/quickjs_dom_bindings.rs`（新）+ rquickjs 原生对象持有 `NodeId`（`Ctx`/`Object`/`IntoJs`/`from_js`）+ 一个 PoC 原生 `Element.nodeType`/`tagName` getter + GC/生命周期 PoC（镜像 V8 TBD-1/TBD-2 的 rquickjs 等价物）+ bench 对照 + `Sandbox::install_native_bindings` 在 QuickJS 实现 escape-hatch
2. **S1q 只读属性族**：tagName/nodeName/nodeType/attributes 原生绑定
3. **S2q 写入 + 子树**：setAttribute/removeAttribute/appendChild/insertBefore/removeChild/childNodes 原生 + 经 live Document
4. **S3q 查询**：querySelector/querySelectorAll/getElementById 原生（消费 `zero_dom` 选择器引擎）
5. **S4q EventTarget**：addEventListener/removeEventListener/dispatchEvent 原生
6. **S5q customElements/Web Components**：原生 HTMLElement class（rquickjs `Class`）+ customElements 五件套 + lifecycle 四件套（镜像 V8 S5）

**验收**：DC-7 满足；双 feature polyfill vs native A/B 行为等价（driving 测试 + WPT 在 `--features v8` 与 `--features quickjs` 两个矩阵下一致）；QuickJS 矩阵 dom_bindings 测试点补齐（缩小 14 vs 73 的差距）。

### M7 — 双引擎 default-on + 全量收尾

**目标**：QuickJS 侧 default-on + kill-switch 移除，**双引擎** native 均为唯一生产路径；polyfill 桥彻底删除。

**切片建议**：
1. QuickJS default-on：`WebViewConfig.native_dom`（quickjs feature）默认 `true`，quickjs 矩阵全量回归 + perf-gate net≥0 守稳（**改 Mission 级单向门，记「待用户决策」清单，等用户点名后 land**）
2. 移除 kill-switch 残余（`ZW_NATIVE_DOM` env 全删）+ `native_dom=false` 回退死代码全删
3. polyfill 桥彻底删除：QuickJS 不再经 `__zw_*` 的部分（`register_callback` 注册的 DOM 回调）删除，shim 最终萎缩

**验收**：DC-1 全项（双引擎）满足；双 feature default-on、kill-switch 移除、polyfill 桥死代码全删，双引擎 native 为唯一生产路径，全量回归 + product-smoke + perf-gate（双 feature）全绿。

### M8 — Canvas path-objects JS 侧 API 语义（v1.2 接手，可与其他 milestone 并行）

**目标**：接手 canvas-2d 流移交的 `html/canvas/element/path-objects` JS 侧 API 语义工作，建立通过率基线并修复 panic + 精度缺口。

> **并行性**：M8 工作面（`crates/canvas/src/path.rs` + `js_dom_shim` path-objects API 段 + WPT 导入）与 M1–M7 的原生绑定主线**基本不重叠**（path-objects 是 Canvas 路径几何 + JS 桥语义，非 DOM node 原生绑定），可作为轻量填充在任意轮次穿插推进，不强制排在 M7 之后。

**切片建议**：
1. **用例重新导入 + roundRect panic 复现定位**（接手第一优先级）：导入 path-objects 用例 → 重新加入 `CANVAS_TEST_SUBDIRS` → 跑 `zero-wpt-runner testharness-canvas path-objects` 复现 panic → 定位 NaN 排序/scale 归一化根因 → 修复到稳定
2. **roundRect DOMPoint 断言精度**（~26 用例）：fill 扫描线与椭圆弧交点配对 + 16 段精度对齐
3. **arc/arcTo/quadratic/bezier/isPointIn* 形状精度**（~16+ 用例）+ roundrect 语义校验（异常/边界）
4. driving 用例经 `make import-wpt` 资产化 + 通过率报告持久化 `evidence/`

**验收**：DC-8 满足；path-objects panic 修复（稳定可重复）、API 语义对齐、通过率基线建立；`make test` + `make reftest`（canvas 段）零回归。

---

## Testing & Quality Gates

### 测试层次

| 层次 | 覆盖范围 | 工具/入口 | 要求 |
|------|----------|-----------|------|
| **单元测试** | dom_bindings 每个迁移/修复 | `cargo test -p zero-engine` 等 | **强制**：每个 API 迁移/修复必带单测 + polyfill vs native A/B 对照；不写无测试的迁移 |
| **集成测试** | 跨 crate JS→DOM→重渲染链路 | `tests/integration`（含 `e2e_rendering.rs`） | DOM mutation → style/layout/paint 更新顺序可测；SPA/WC 端到端资产化 |
| **WPT dom 上游** | 规范合规性 | `make reftest` + `make import-wpt` | M4 建立基线，持续扩展；每修复必带 driving 用例资产化 |
| **tab_js_worker e2e** | 真实页面脚本执行（fetch/定时器/Observer） | `apps/browser` 测试 | 既有基线（fetch 端到端 / 定时器 / MO 五连测）持续全绿 |
| **产品 smoke** | 产品级回归（welcome.html vs chromium oracle） | `make product-smoke` | **default-on / 渲染热路径切片必跑**（曾致 R428 回归藏 14 轮） |
| **性能门禁** | JS→DOM 桥热路径、首屏、RSS | `make bench-gate` | **default-on / S6 切片必跑**；首跑全 NEW/PASS → `make bench-capture JUSTIFICATION="..."` → 再跑真比较；禁止临时改测量配置或跳门禁（config_hash 会暴露） |

### 质量门禁（每轮执行必须满足才能前进）

1. **编译门禁**：`cargo build --workspace` 成功，`cargo clippy --workspace --all-targets -- -D warnings` 零警告（CI 用 `-D warnings`，本地同等严格；v8 feature 因环境缺库时至少在可编译 feature 下跑 clippy 并注明覆盖范围）
2. **测试门禁**：`make test`（release + scripts/test-guard.rs 包裹）全通过，零红灯——**禁止裸跑 `cargo test`**（内存型 bug 会触发 OOM 连累整个 session；见 docs/rally/oom-guard.md）
3. **覆盖门禁**：新代码必带测试；dom_bindings 覆盖率持续提升、不退化（**不缩范围伪造达标**）
4. **回归门禁**：渲染/JS 桥热路径变更额外跑 `make product-smoke` + `make bench-gate`，net≥0 才 land
5. **fmt 门禁**：`cargo fmt --all -- --check` 无 diff（有 diff 先 `cargo fmt --all`）
6. **文档门禁**：公共 API 必须有 `///` doc comment；实现 web 规范行为处加 spec 链接注释（CLAUDE.md 规范驱动注释）

### 覆盖率策略

- **目标**：dom_bindings crate 行覆盖率**持续提升、不退化**（不写硬阈值，给执行 agent 自主裁量；既有 polyfill 行为在迁移期由 A/B 对照门守住）
- **统计口径**：全 crate 行覆盖，**不缩范围**；经 `scripts/check-coverage.sh` 一键报告
- **报告**：每轮记录 dom_bindings 覆盖率数据到 `master.md` + `evidence/`
- **缺口即工作**：若当前缺少 dom_bindings 独立 coverage 口径、缺少统一统计脚本、缺少报告链路，或某些子模块尚无法纳入 coverage——**这不是 BLOCK 理由**，而是要继续推进的 active milestone（把「补齐 coverage 测量能力」视为当前工作内容）

### 无人值守运行安全

- 测试/reftest 一律走 `make test` / `make reftest`（release + scripts/test-guard.rs 包裹），**禁止裸跑** `cargo test` / `cargo run --bin zero-wpt-runner -- reftest`（内存型 bug 如无限循环 realloc 会触发系统 OOM，连累 rally 父进程 / tmux session）
- 性能门禁走 `make bench-gate`（测量 + 门禁比较，退出码 0/1/2，全经 test-guard 包裹）

### 阶段性提交与并行同步

- 有阶段性进展及时 commit + push；push 前 `git pull --rebase`（并行双流常态，自主 rebase、禁强推）
- commit 小而频繁；commit 信息用英文，文档/注释用中文
- 提交前必跑 `lei-pre-commit-guard`（PASS 才提交）；代码变更必跑 `cargo fmt` + `cargo clippy`

---

## Latest Evidence

> **本节是入口文档的静态快照**。真实、最新的状态永远以 [js-dom/master.md](js-dom/master.md) 为准——本节只记基线锚点，不每轮重写。

**当前状态快照**（2026-08-13，详见 master.md）：

| 项 | 状态 |
|----|------|
| P1a（event loop / fetch / Observer） | ✅ 实质完成（非阻塞 follow-up 见 master.md） |
| P1b **V8** native bindings S0–S5 | ✅ 已 land，默认关（`ZW_NATIVE_DOM`） |
| L1 Live Document 共享（V8） | ✅ 已 land（`Rc<RefCell<Document>>`） |
| L2 polyfill-live 合一（V8） | ❌ 未做（本目标 M1） |
| S6 高层 API 去字符串（V8） | ❌ 未做（本目标 M2） |
| **QuickJS 原生 DOM 绑定** | ❌ **完全缺失**（v1.1 新增，本目标 M6） |
| S7 死代码清理 + shim 萎缩 | ❌ 未做（本目标 M5/M7） |
| **双引擎** default-on + 删 kill-switch | ❌ 未做（V8 在 M5，QuickJS 在 M7，改 Mission 级单向门） |
| 真实 SPA/WC 端到端验收 | ❌ 无资产（本目标 M3，V8 先行） |
| WPT dom 上游基线 | ❌ 0 上游导入（本目标 M4） |
| 编译/clippy/test | ✅ workspace ~13,000+ 测试全绿，行覆盖 95.46%，clippy 零警告（含 `--features quickjs` CI 矩阵） |
| dom_bindings 独立 coverage 口径 | ❌ 待补齐（本目标 M0） |

**关键证据锚点**（RFC commit / 测试位置，执行 agent 可按此核对）：
- P1b RFC：`docs/specs/p1b-v8-native-bindings-rfc.md`（S0 PoC R3095 / S1 R3096 / S2 R3097 / S3 R3099-R3100 / S4 R3109–R3126 / S5 R3262–R3269 / L1 R3106–R3108）
- polyfill vs native bench：RFC §4 S0 gate（native 215ns vs polyfill 3.36µs，~15.6x）
- 生产接线 kill-switch：`crates/engine/src/dom_bindings/mod.rs`（`ZW_NATIVE_DOM` env，`install_dom_bindings_if_enabled`）+ `crates/webview`（`WebViewConfig.native_dom`，默认 `false`）
- 既有 e2e 基线：`apps/browser/src/tab_js_worker.rs`（fetch 端到端 / 定时器 / MutationObserver 五连测）
- WPT runner：`tests/wpt-runner/src/runner/test_cases/test_cases_js_dom.rs`（内建 smoke，0 上游）
- **QuickJS 页面引擎路径**（v1.1 核实锚点）：`crates/script-sandbox/src/quickjs_runtime.rs`（`register_callback` 注册 `__zw_*` 走 polyfill 桥，无原生绑定）；`crates/webview/src/webview.rs:1215` + `apps/browser/src/tab_js_worker.rs:276`（`#[cfg(feature="quickjs")]` 页面沙箱构造，与 V8 对称）；`crates/script-sandbox/src/lib.rs:167`（`Sandbox::install_native_bindings` 默认 `false`，QuickJS 未实现）；Makefile `QUICKJS_CLIPPY_CRATES`/`QUICKJS_TEST_CRATES`（CI 强制 quickjs 矩阵）

---

## Document Control / Archive Policy

### 两层文档控制面（强制）

本目标采用**两层文档控制面**。所有 session 都必须以下列固定路径为准，不要替换成其他入口形态：

#### 入口文档（稳定，不频繁修改）

- **路径**：`docs/goal/js-dom.md`（本文件）
- **职责**：定义长期 Mission、Done Criteria、执行协议、文档治理规则
- **修改条件**：仅在目标本身发生实质性变化时修改（如调整范围边界、修改完成标准、Mission 变化）。**除非 contract 本身发生实质变化，不要在每轮执行里重写它。**
- **禁止行为**：每轮执行不重写本文件；日常进展、evidence、active milestone 更新写入 `master.md`

#### 运行时控制平面 `docs/goal/js-dom/master.md`（持续演进）

- **职责**：当前真实状态的唯一控制面板，保存仍然有效的目标边界、done criteria、active milestone、测试基线、验证证据、下一步计划、未解决问题列表、待用户决策清单
- **治理规则**：
  - master.md 是**持续演进的增量控制面，不是一次性交付物**。创建第一版 master.md 只表示治理框架建立，**不**表示核心目标已被覆盖，**不**表示任何核心能力已完成。
  - **不允许无限增长**——过时内容必须重写、压缩或迁移到 archive
  - 各 section 之间必须**自洽**（active milestone / done criteria / coverage / Latest Evidence 不能互相矛盾）；出现矛盾（如「还有未完成 milestone，但 evidence 声称 all done criteria met」）必须先修正文档和状态判断再继续

#### 归档区域 `docs/goal/js-dom/archive/`（历史记录，只追加）

- **职责**：保存已完成 milestone/切片的详细过程、关键决策、验证结果、commit hash 和历史证据
- **性质**：archive 是历史记录区，**不是当前状态的来源**；只追加，不修改已归档内容

#### 证据区域 `docs/goal/js-dom/evidence/`（持续追加）

- **职责**：存储 WPT dom 通过率报告、失败聚类分析、coverage 报告、A/B 对照门结果、SPA/WC 端到端验收记录等结构化验证证据
- **性质**：持续追加；每个证据有日期 + 测试命令 + 结果摘要

### 首轮进入检查清单（Must-Complete-First-Round）

执行 agent 在首次进入时**必须**完成以下操作——这些不是可选的，也不是可以推迟的工作：

- [ ] **探索当前仓库事实**：`dom_bindings/`（V8 原生绑定）+ `js_dom_bridge.rs` + `js_dom_shim/` 当前代码状态、RFC §3.7 L1/L2 切片定义与代码现状是否一致、`ZW_NATIVE_DOM` kill-switch 现状（V8）、既有 e2e 测试基线；**v1.1 新增：核实 QuickJS 页面引擎路径**（`quickjs_runtime.rs` 的 `register_callback` + `Sandbox::install_native_bindings` 默认 `false`）确认 native 真空状态、CI quickjs 矩阵覆盖范围（`QUICKJS_*_CRATES`）
- [ ] **定义/确认 Done Criteria**：与本文件 DC-1~8 一致；若发现代码现状与本文件基线事实不符，先在 master.md 记录勘误
- [ ] **创建 `docs/goal/js-dom/master.md`**：包含完整的当前状态评估 + 首个 active milestone（M0）切片计划 + 测试基线 + 缺口清单 + 待用户决策清单（含 default-on M5/M7）
- [ ] **确认 `docs/goal/js-dom/archive/` 与 `evidence/` 目录存在**（不存在就立即创建——这不是可选项，也不是以后再补的工作）
- [ ] **确认测试基线**：`make test`（含 `--features quickjs` 矩阵）当前全绿状态、dom_bindings 无独立 coverage 口径（M0 要补齐）、WPT dom 分类 0 上游导入（M4 要补齐）
- [ ] **选定第一个 active milestone（M0）的首个切片并直接动手推进**

**关键要求**：完成第一版 master.md + archive/evidence bootstrap 后，执行 agent 在**同一轮内必须继续启动第一个真实 milestone（M0 首切片）**，直接推进核心目标能力本身——**不允许**把「文档框架已建立」当成 milestone 完成或收工依据。

### 文档治理原则

1. master.md 各 section 必须自洽——active milestone / done criteria / coverage / Latest Evidence 互不矛盾
2. 发现矛盾时，执行 agent 必须先纠正文档和状态判断，再继续推进
3. master.md 不允许无限增长——过时内容必须重写、压缩或归档
4. archive 是只追加的——不修改已归档内容
5. 所有验证证据必须以结构化形式持久化（测试命令、coverage 报告路径、A/B 对照结果、通过率报告、验收矩阵）

---

## Final Output Contract

> **本节是终局输出协议。执行 agent 在每轮结束前必须按此判定输出 `DONE` / `CONTINUE` / `BLOCK`。**

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| DC-1~8 **全部**满足，目标能力达到 production-ready 水平 | `DONE` | 见下方「DONE 允许条件」 |
| 进展仍可推进，还有未完成的工作 | `CONTINUE: <下一步>` | **这是默认输出** |
| 遇到真正的外部阻塞（依赖不可用、平台根本性不支持、安全漏洞无法绕过） | `BLOCK: <原因>` | 罕见使用 |
| verify 发现未满足条件但进展仍可推进 | `CONTINUE: <下一步>` | **返回执行，不是 DONE**，不是解释性段落 |

### DONE 允许条件

**同时满足以下所有条件时才允许输出 `DONE`**：

1. ✅ DC-1~8 全部满足
2. ✅ 目标能力本身已达到生产可用质量（**双引擎** native 为唯一生产路径 + SPA/WC 端到端跑通 + WPT dom 基线建立），**不只是文档完整**
3. ✅ 有真实代码、测试和验收证据直接对应目标能力（非仅计划）
4. ✅ `make test`（含 `--features quickjs` 矩阵）+ `cargo clippy --workspace --all-targets -- -D warnings` + `make product-smoke` + `make bench-gate` 全通过
5. ✅ master.md 内部自洽，archive + evidence 已建立，完成的 milestone/切片已归档
6. ✅ RFC `p1b-v8-native-bindings-rfc.md` 剩余切片状态与 master.md 一致；TBD-5 已更新为双引擎

### 禁止输出 DONE 的情况

即使以下情况中部分条件看起来「还行」，也**不允许**输出 `DONE`：

- ❌ master.md 缺失、必填 section 缺失、archive 为空且无有效里程碑
- ❌ 无测试证据，或测试存在红色（失败）项
- ❌ 无实际代码/测试进度（仅有文档和计划）
- ❌ coverage 无法证明（无测量脚本、无报告管线、无量化数据）——**这不是 BLOCK，而是要继续推进的 active milestone**
- ❌ master.md 各 section 矛盾（如「里程碑未完成但 evidence 声称全部满足」）
- ❌ 所有 master.md section 都填了、archive/evidence 建了、计划列了，但**没有真实代码、测试和验收证据直接对应目标能力**
- ❌ 测试全绿、coverage 达标、文档完整，但**目标能力本身未达到生产可用质量**（如 native 仍默认关、SPA/WC 仍跑不通、WPT dom 仍 0 上游）
- ❌ 仅通过内建 inline 用例，未导入上游真实 WPT dom 用例
- ❌ default-on 未做（DC-1 硬条件）
- ❌ **QuickJS native 缺失或未对齐**（DC-7 硬条件，v1.1）：只 V8 达成而 QuickJS 仍走 polyfill 字符串桥 = 双引擎不对等，不允许 `DONE`
- ❌ **只在一个 feature 下验证**：driving 测试/A/B 对照/WPT 只跑了 `--features v8` 没跑 `--features quickjs`（或反之）

### BLOCK 策略（默认禁用 BLOCK）

用户要求禁用 BLOCK，遵循以下规则：

- 「未完成、证据不足、coverage 暂时无法验证、文档状态不一致、深结构跨面改、default-on 等待用户点名」**都是继续推进的信号**，不是 BLOCK 的理由
- 即使遇到困难，如果仍有可能推进，输出 `CONTINUE: <下一步>`
- **只有在真正无法继续时才输出 `BLOCK`**：外部依赖不可用且无替代方案、平台根本性不支持、安全漏洞无法绕过

### verify 发现缺口时的处理

- 默认输出 `CONTINUE: <下一步>` 并**返回执行**，不是 `DONE`，不是大段解释
- 如果仍有可能推进，就不结束——`DONE` 只在 DC-1~8 全满足时才允许

---

## Execution Protocol

### 自主执行原则

执行 agent 必须：

1. **自主探索**当前 JS/DOM 桥状态（V8 与 QuickJS 双路径），识别 native vs polyfill 缺口（每轮开工前先 `git pull --rebase` 拉最新 main）
2. **自主分解**milestone 为可独立 land 的切片（kill-switch + A/B 对照门 + 全量回归）
3. **自主实现**迁移/修复代码，不等待用户逐步指令；每片 net≥0 即 land，commit 小而频繁
4. **自主添加测试**：每项迁移/修复必带单测 + polyfill vs native A/B 对照（**双 feature 均跑**）+ driving WPT 用例资产化
5. **自主验证**：`make test`（含 `--features quickjs` 矩阵）+ clippy +（渲染/JS 热路径）`make product-smoke` + `make bench-gate` 确认修复有效、零回归
6. **自主归档**：完成的 milestone/切片记录到 archive；evidence 持久化到 evidence/
7. **持续推动**，直到 DC-1~8 全部满足——在未达 done criteria 前持续推进，不等待用户逐步下达下一条指令

### 轻量修复优先（借鉴 rendering-compat / canvas-2d 裁决）

1. **主线 = 轻量修复**：kill-switch 仍开、根因清楚、改动面小、A/B 无新失败的切片
2. **永不停**：遇需用户拍板事项（default-on 改 Mission 级单向门、深结构跨面改）记「待用户决策」清单并跳过，继续下一个轻量修复
3. **深结构护栏**：default-on（M5 V8 / M7 QuickJS）、QuickJS native 移植（M6 全量）、polyfill 桥全量重写（L2 完整）等改 Mission / 跨面深改不自主 land，记待决策清单等用户点名；其余切片自主推进
4. **碰撞管理**：碰 canvas-2d / html-compat 共享面（`js_dom_shim` part04/05、表单事件段）前先 `git log` 核对；有活跃编辑则转零碰撞面

### 遇到问题时的处理原则

1. **已知失败测试**：**不允许留给下一轮**。遇到 flaky test、遗留失败、环境脚本问题、测试基础设施缺陷，当作当前任务的一部分修复到稳定可重复
2. **覆盖缺口**：dom_bindings 缺独立 coverage 口径、缺统一统计脚本、缺报告链路——视为当前工作内容，而非 BLOCK
3. **迁移期行为不一致**：polyfill vs native A/B 对照发现差异，必须先修对照门、定位根因（API 语义？JS 接线？Rust 层 bug？），再 land 切片
4. **技术决策**：在 master.md 记录关键决策及理由；TBD 项关闭/更新同步回 RFC
5. **范围变更**：发现目标需调整，在 master.md 记录并说明理由，**不**修改本入口文件（除非 Mission 本身变化，且需用户批准）
6. **重大进展或卡点**：及时经飞书 CLI 通知本人（run-rules §7），消息说明具体进展/卡点；通知仅为告知，不阻塞后续工作

### 提交安全（run-rules §提交前门禁）

- `git commit` 前必调 `lei-pre-commit-guard`（PASS 才提交；BLOCK 输出报告等待修复后重扫）
- 代码变更必跑 `cargo fmt` + `cargo clippy`；文档/`.github`/`.md` 豁免项目代码检查，但仍须 `git diff --check` + lei-pre-commit-guard
- 单文件 ≤ 2000 行（CLAUDE.md §5；超 2000 行按职责拆分模块——`js_dom_shim` 的萎缩本身会缓解 `computed_style.rs` 158KB 等超大文件）

---

## 后续执行建议（写入 goal，不立即实施）

以下为执行阶段参考，本轮**不实施**：

1. **M0 首切片选 L2-read-only**：polyfill 桥只读 getter（getElementById/querySelector/getAttribute）改读 live Document，写仍走旧路径——最小风险验证三方合一管线，kill-switch 仍开零生产影响。
2. **A/B 对照门前置**：M0 就建 polyfill vs native 行为等价测试骨架，每个迁移切片复用——这是整个迁移期「行为不退化」的安全网。**v1.1：A/B 对照门设计为双 feature 可参数化**（同一套断言跑 `--features v8` 与 `--features quickjs`），为 M6 QuickJS 对齐提前铺路。
3. **WPT dom 导入可早期并行**：M4 导入本身零源码改动，可与 M1/M2 并行启动（先建通过率基线，暴露真实缺口）。
4. **V8 先行，QuickJS 镜像**：M1–M5 先把 V8 路径收敛到 native production-ready；M6 再镜像 V8 切片到 QuickJS（S0q–S5q）。这样避免双引擎并行开发的状态爆炸——V8 切片模式稳定后，QuickJS 是"翻译"而非"设计"。
5. **M6 首切片选 S0q PoC**：先验证 rquickjs 原生对象持有 `NodeId` + GC/生命周期管线（镜像 V8 TBD-1/TBD-2），确认可行再铺 S1q–S5q。若 rquickjs API 边角阻塞，标 TBD 不 BLOCK，转其他切片。
6. **default-on 分两步（V8→QuickJS）**：M5 先 V8 default-on，M7 再 QuickJS default-on，最后删 kill-switch。每步都是改 Mission 级单向门，各自记「待用户决策」清单等用户点名；default-on land 前必跑全量 `make test`（双 feature）+ `make product-smoke` + `make bench-gate`，net≥0 才 land。
7. **默认禁用 BLOCK**：执行 agent 默认输出 `CONTINUE: <下一步>`；未完成/证据不足/coverage 暂不可验/文档不一致/等用户点名 default-on 都是继续信号。
