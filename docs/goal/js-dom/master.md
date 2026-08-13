# JS/DOM 原生化 — 主控面板（master.md）

**入口文档**: [../js-dom.md](../js-dom.md)（长期 Mission / Done Criteria / 执行协议 / 文档治理规则）
**关联 RFC**: [../../specs/p1b-v8-native-bindings-rfc.md](../../specs/p1b-v8-native-bindings-rfc.md)
**创建日期**: 2026-08-13（goal 拆分 bootstrap）
**本轮**: R0 核实 + master.md 重写（基线 commit `f7219b2c`，2026-08-13）

> **本文件由执行 agent 于 2026-08-13 按入口文档「首轮进入检查清单」逐项核实重写**，替换 bootstrap 占位符。
> 所有状态带证据（commit hash / 文件路径 / 行号 / 测试命令）。并行双流下 main 随时漂移（run-rules §10），每轮开工先 `git pull --rebase`。

---

## 当前状态（2026-08-13 核实，基线 commit `f7219b2c`）

| 项 | 状态 | 证据 |
|----|------|------|
| P1a（event loop / fetch / Observer） | ✅ 实质完成 | 入口文档基线块（fetch/定时器/MO e2e 在 `apps/browser/src/tab_js_worker.rs`）；非阻塞 follow-up 见下「缺口」 |
| P1b **V8** native bindings S0–S5 | ✅ 已 land，默认关 | `crates/engine/src/dom_bindings/` 19 文件；`mod.rs:55 native_dom_enabled()` 读 `ZW_NATIVE_DOM` env，`OnceLock` 缓存 |
| L1 Live Document 共享（V8） | ✅ 已 land | `webview.rs user_actions.rs:367` `config.native_dom` + `install_native_dom_bindings()`；`mod.rs` `parse_and_install`（`Rc<RefCell<Document>>`，注释「read-only 快照」未完全 live） |
| **V8 native API 面（核实后更完整）** | ✅ 超基线 | `mod.rs:558-624` 已注册：`__zw_native_query_selector/_all`、`create_element/text_node/comment/document_fragment`、`documentElement/body/head`、`get_element_by_id`——比入口文档基线描述更完整 |
| L2 polyfill-live 合一（V8） | ❌ 未做（M1） | `js_dom_bridge/callbacks.rs:119-137` `__zw_query_match/_all` 仍 `with_query_doc(snap, …)` 经 `parse_html(dom_html)` 重解析快照；`callbacks.rs:1454` 注释明示每次查询 re-parse |
| S6 高层 API 去字符串（V8） | ❌ 未做（M2） | Fetch/Observer/FontFaceSet 仍经 `__zw_*` String ser/deser（part02.js / part01.js） |
| **QuickJS 原生 DOM 绑定（DC-7）** | ❌ **完全真空（核实确认）** | `script-sandbox/src/lib.rs:167 install_native_bindings` 仅 `#[cfg(feature="v8")]` 存在，QuickJS trait 方法**不存在**；`quickjs_runtime.rs:357 register_callback` 走 `__zw_*` polyfill 桥（与 V8 共用 `js_dom_shim`） |
| S7 死代码清理 + shim 萎缩 | ❌ 未做（M5/M7） | `js_dom_shim/part01-06.js` 共 ~815KB（part01 111KB+part01b 28KB+part02 149KB+part03 148KB+part04 127KB+part05 150KB+part06 103KB） |
| **双引擎** default-on + 删 kill-switch | ❌ 未做（V8=M5, QuickJS=M7，改 Mission 级单向门） | `WebViewConfig.native_dom` 默认 `false`（`webview_builder.rs:79`） |
| 真实 SPA/WC 端到端验收 | ❌ 无资产（M3） | 无 React/Vue/Svelte/lit 端到端 fixture |
| WPT dom 上游基线 | ❌ 0 上游导入（M4） | `tests/wpt-runner/.../test_cases_js_dom.rs` 全内建 inline smoke（`dom_has_body` 等），无上游 `dom/` 用例 |
| **Canvas path-objects JS 侧 API（DC-8, v1.2 接手）** | ⚠️ 用例**完全缺失**（须重新导入） | `wpt-data/html/canvas/element/` 目录本地不存在（不止 path-objects，整个 canvas element 子树未 fetch）；`testharness.rs:26 CANVAS_TEST_SUBDIRS` 8 个目录无 path-objects；`testharness-canvas` 子命令已就绪（`main.rs:220`） |
| `make test` / clippy / coverage（含 quickjs 矩阵） | ✅ 基线全绿（入口文档） | workspace ~13,000+ 测试，行覆盖 95.46%，clippy 零警告；Makefile `QUICKJS_TEST_CRATES`/`QUICKJS_CLIPPY_CRATES` CI 强制 `--features quickjs` |
| dom_bindings 独立 coverage 口径 | ❌ 待补（M0 项 4） | `scripts/check-coverage.sh` 仅 workspace 全量，无单 crate/子模块口径；`cargo-llvm-cov` **本地未安装**（环境前提，见下） |

**核心缺口**（本目标要消除，按优先级）：
1. **WPT dom 上游用例 0 导入**（DC-3）——无通过率基线，无法度量 native vs polyfill 真实差距
2. **polyfill vs native A/B 对照门缺失**（DC-4）——dom_bindings 5457 行测试全 native-only，无两条路径行为等价断言（M0 must-complete 项 5）
3. **dom_bindings 独立 coverage 口径缺失**（DC-4）——`check-coverage.sh` 无子模块报告（M0 项 4）
4. V8 native 路径默认关 → 生产仍走 polyfill 字符串桥（DC-1，M5）
5. V8 polyfill 桥仍 re-parse String 快照，三方 Document 未合一（DC-1，M1 L2）
6. V8 高层 API 仍经 `__zw_*` String ser/deser（DC-1，M2 S6）
7. **QuickJS 页面引擎 native 完全真空**（DC-7，M6）
8. canvas path-objects 用例需重新导入 + roundRect panic/精度（DC-8，M8，但当前 canvas 流活跃碰撞）

---

## Active Milestone

**当前活跃里程碑**: **M0 — 基线建立 + polyfill-live 合一起刀**（见入口文档「Single Active Milestone」）

**M0 must-complete 入口动作进度**（入口文档「首轮进入检查清单」）：
- [x] 1. 探索 `dom_bindings/` + `js_dom_bridge.rs` + `js_dom_shim/` 事实，核实 RFC L1/L2 与现状一致 ✅（本文件即产出）
- [x] 2. 创建/重写 `master.md`（本文件）
- [x] 3. 确认 `archive/` + `evidence/` 目录存在（均空，待首轮证据追加）
- [ ] 4. 补齐 dom_bindings 独立 coverage 口径（**环境前提 `cargo-llvm-cov` 本地未装**，本轮记入「未解决问题」，待装后补）
- [ ] 5. 建 polyfill vs native A/B 对照门骨架（双 feature 可参数化）← **本轮首切片目标**
- [x] 6. 选定首切片并直接动手推进 ✅（A/B 对照门骨架，项 5 与项 6 合并完成）

**M0 首切片（本轮）**: **polyfill vs native A/B 对照门骨架（must-complete 项 5）**
- 理由：入口文档明列 must-complete；纯新增测试文件，零生产代码改动、零碰撞；为后续 M1(L2)/M6(QuickJS) 所有迁移切片提供「行为不退化」安全网（DC-4）；双 feature 可参数化设计为 M6 提前铺路。
- 设计：对同一 HTML + 同一可观测 DOM 操作（如 `getAttribute`/`textContent`/`tagName`/`querySelector` 命中），断言 native 路径（`install_dom_bindings` + `__zw_native_*`）与 polyfill 路径（`generate_js_dom_shim` + `register_dom_callbacks` + `__zw_*`）返回一致。聚焦**行为等价**（可观测结果），不强求 API 形态同构（native=真对象 vs polyfill=Proxy）。

**为何本轮不选 L2-read-only / canvas path-objects 作为首切片**（决策记录）：
- **L2 完整改 polyfill 桥是深结构跨面改**：`register_dom_callbacks` 签名收 `Arc<Mutex<String>>`（dom_html 快照），改读 live Document 需改签名为 `Rc<RefCell<Document>>`，触及 renderer/browser/reftest 三处调用点（callbacks.rs:18-21 注释明示）→ 触发入口文档「深结构护栏」，不宜作首切片。L2 的最小只读子集（仅 getElementById/querySelector getter）虽可行，但缺 A/B 对照门前无法证明行为等价，风险高于先建安全网。
- **canvas path-objects 当前是热碰撞面**：`git log` 核实 canvas 流最近非常活跃（`f7219b2c` 刚改 `js_dom_shim/part05.js` + `canvas.rs` + `testharness.rs`），part05.js 的 canvas 段与本目标 v1.2 接手段并发编辑 → 按入口文档护栏「碰 canvas 共享面前先 git log，有活跃编辑则转零碰撞面」，本轮转 A/B 对照门（DOM 面，零碰撞）。canvas path-objects 待 canvas 流告段落再接手（记「未解决问题」）。

**本轮/本切片进度**: ✅ **完成**——polyfill vs native A/B 对照门骨架已实现并验证（`crates/engine/src/dom_bindings/tests_ab_compare.rs`）。
- 9 条读操作用例（tagName/nodeType/getAttribute/hasAttribute/querySelector(All)/getElementById/descendant/反射 id）+ querySelectorAll 索引读 + 2 个 sanity → **native 与 polyfill 读路径全等价**（4 测试函数全绿）
- 双 feature 验证：v8 矩阵 zero-engine 2063 测试全绿（含新 4 个）；quickjs 矩阵 zero-engine 1405 全绿（A/B 模块 `#[cfg(feature="v8")]` 排除，不影响 quickjs）；双矩阵 clippy 零警告
- 关键结论：**实证 native 读路径与 polyfill 读路径行为等价**——为 M1 L2-read-only 切片（polyfill 桥改读 live Document）提供了可直接复用的 A/B 验收门

---

## 测试基线

| 基线 | 命令 | 当前值 |
|------|------|--------|
| zero-engine 测试（v8） | `cargo test -p zero-engine --features v8 --lib` | ✅ 2063 passed（含 A/B 门 4 个，本切片实测） |
| zero-engine 测试（quickjs） | `cargo test -p zero-engine --no-default-features --features quickjs --lib` | ✅ 1405 passed（A/B 门 cfg 排除，本切片实测） |
| zero-webview 测试（v8） | `cargo test -p zero-webview --features v8` | ✅ 17 passed（native_dom 接线回归） |
| clippy（v8 + quickjs 双矩阵） | `cargo clippy -p zero-engine ...` | ✅ 零警告（双矩阵） |
| workspace 全量 `make test` | `make test` | ⚠️ 本轮单次超时未跑完（workspace 全量编译+测试+quickjs 矩阵 >580s）；聚焦验证已覆盖变更面（纯测试新增，无生产代码改动） |
| 行覆盖率（全量） | `scripts/check-coverage.sh` | 95.46%（基线；**本地缺 `cargo-llvm-cov`，本轮无法实测**） |
| dom_bindings 覆盖率（独立） | 待补口径（M0 项 4） | ❌ 无口径 |
| product-smoke | `make product-smoke` | 本轮非渲染热路径变更，A/B 骨架（纯测试）豁免 |
| bench-gate | `make bench-gate` | 本轮非 JS 桥热路径代码变更，豁免 |

---

## Coverage 矩阵

| crate/模块 | 行覆盖率 | 趋势 | 备注 |
|------------|----------|------|------|
| dom_bindings（zero-engine 子模块） | ❌ 待补口径 | — | `mod.rs` + 14 子模块 + 5 测试文件（5457 行测试）；M0 项 4 待 `cargo-llvm-cov` 装后补 `check-coverage.sh` 子模块口径 |

**覆盖率口径规则**：不缩范围伪造达标；新代码必带测试；持续提升、不退化。

---

## Latest Evidence

| 日期 | 轮次 | 证据 | 结果 |
|------|------|------|------|
| 2026-08-13 | goal 拆分 bootstrap | 入口文档基线事实块 | 框架占位 |
| 2026-08-13 | R0 核实 | 代码实测（commit `f7219b2c`）：dom_bindings 19 文件 / QuickJS native 真空确认 / L2 仍 re-parse 确认 / canvas wpt-data 缺失确认 / cargo-llvm-cov 未装确认 | 本文件重写，勘误见下 |
| 2026-08-13 | R0 首切片 | A/B 对照门骨架 `tests_ab_compare.rs`（9 读用例 + 索引读 + 2 sanity）；v8 2063 + quickjs 1405 + webview 17 全绿，双矩阵 clippy 干净 | **native 读路径 ≡ polyfill 读路径**（行为等价实证），M0 项 5+6 完成 |

**本轮勘误**（vs 入口文档基线块）：
1. dom_bindings native API 面**比基线描述更完整**：除 S0–S5 基线外，`mod.rs:558-624` 已注册 querySelector 族 + createElement/Text/Comment/Fragment + documentElement/body/head 全套工厂（注释「R3098/R3131/R3136」）。入口文档「19 文件」清单未列全这些工厂——native 写能力实际比「读 ~15.6x」更广。
2. canvas path-objects 缺口**比基线描述更严重**：本地 `wpt-data/html/canvas/element/` 整个 canvas element 子树都不存在（不止 path-objects），`make fetch-wpt-data` 拉的 wpt-data repo（v1.10）目前只含 reftest 数据（css/fonts/images/quirks），**不含 html/ testharness 用例**。canvas testharness 用例须从上游 wpt 仓库单独导入（非 fetch-wpt-data）。

---

## 下一步计划

1. **本轮（已完成）**：polyfill vs native A/B 对照门骨架 ✅（M0 项 5+6）→ 验证 native≡polyfill 读路径等价 → land
2. **下轮候选（按优先级）**：
   - **(a) WPT dom 上游用例导入**（M4 早期切片，零源码改动）：从上游 `web-platform-tests/wpt` 导入 `dom/` 真实用例到 `wpt-runner`，建按子分类通过率基线（DC-3）——这是 DC 硬缺口中**唯一纯资产、零碰撞、可独立 land** 的，且立即暴露 native/polyfill 真实规范差距。优先选此。
   - **(b) dom_bindings 独立 coverage 口径**（M0 项 4）：需先装 `cargo-llvm-cov` + `llvm-tools-preview`，扩 `check-coverage.sh` 加 `-p zero-engine` 子模块报告。
   - **(c) M1 L2-read-only 最小子集**：A/B 门已就位，可安全切 `getElementById`/`querySelector` getter 改读 live Document（仍需评估 `register_dom_callbacks` 签名跨面改动）。
3. **后续主线**：M1 L2 → M2 S6 → M3 SPA/WC → M4 WPT dom → M5 V8 default-on（待用户决策）→ M6 QuickJS native → M7 双引擎 default-on + 收尾；M8 canvas path-objects 待 canvas 流告段落接手

---

## 待用户决策清单（深结构护栏）

| 事项 | 触发条件 | 状态 |
|------|----------|------|
| V8 `ZW_NATIVE_DOM` default-on（改 Mission 级单向门，M5） | M1–M4 完成、V8 native 路径生产就绪 | 待 M5 启动前征询 |
| QuickJS `ZW_NATIVE_DOM` default-on（改 Mission 级单向门，M7） | M6 QuickJS native 移植完成 | 待 M7 启动前征询 |

> 本轮 A/B 对照门骨架为纯测试新增，不触发上述门禁。

---

## 未解决问题

1. **`cargo-llvm-cov` 本地未安装** → dom_bindings 独立 coverage 口径（M0 项 4）本轮无法实测落地。方案：`cargo install cargo-llvm-cov` + `rustup component add llvm-tools-preview`，再扩 `check-coverage.sh` 加 `cargo llvm-cov -p zero-engine` 子模块报告。本轮记入，下轮处理（安装非阻塞）。
2. **canvas path-objects 是热碰撞面**：canvas 流（`f7219b2c` 等）正在活跃编辑 `part05.js` canvas 段 + `canvas.rs`。M8 path-objects 接手须等 canvas 流告段落，或确认 part04/05 path-objects 段无并发编辑后再动。碰撞信号点：`git log --since="14 days" -- crates/engine/src/js_dom_shim/part05.js crates/canvas/src/`。
3. **canvas wpt-data html 子树缺失**：canvas testharness 用例（含 path-objects）需从上游 `web-platform-tests/wpt` 仓库单独导入到 `wpt-data/html/canvas/element/`，`make fetch-wpt-data` 不提供。M8 接手第一动作。

---

## 归档记录

> 已完成的 milestone/切片记录到 `archive/`。当前无已完成项。

- （无）
