# 归档：M0 首切片 — polyfill vs native A/B 对照门骨架

**日期**: 2026-08-13
**轮次**: R0（goal 拆分后首轮核实 + 首切片）
**Milestone**: M0（基线建立 + polyfill-live 合一起刀）
**切片**: M0 must-complete 项 5（A/B 对照门骨架）+ 项 6（首切片动手）
**Commit**: `c7cde09e`
**基线**: `f7219b2c`（rebase 后含渲染流 part05.js 改动，无冲突）

## 切片目标

建立 JS↔DOM 桥原生化迁移期（M1 L2 / M2 S6 / M6 QuickJS native）的「行为不退化」安全网：
对同一组可观测 DOM 读操作，断言 native 路径与 polyfill 路径返回一致。

## 实现产物

**新文件**: `crates/engine/src/dom_bindings/tests_ab_compare.rs`（191 行）
- `run_native(html, expr)`：复用 `tests::run_script`（install_dom_bindings），额外桥 `document` 到 global
- `run_polyfill(html, expr)`：复刻 `js_dom_bridge_tests` 模式（V8Sandbox + shim + register_dom_callbacks）
- `READ_CASES`：9 条读操作用例表（tagName/nodeType/getAttribute/hasAttribute/querySelector(All)/getElementById/descendant/反射 id）
- 4 个测试函数：主对照门 + querySelectorAll 索引读 + 2 个 sanity
- 双 feature 可参数化设计（为 DC-7 QuickJS 对齐铺路），整模块 `#[cfg(feature="v8")]` 门控

**改动**: `crates/engine/src/dom_bindings/mod.rs`（+5 行，注册新测试模块）

## 关键决策

1. **首切片选 A/B 门而非 L2-read-only**：L2 完整改 polyfill 桥需改 `register_dom_callbacks` 签名（`Arc<Mutex<String>>` → `Rc<RefCell<Document>>`），触及 renderer/browser/reftest 三处调用点 → 深结构护栏。L2 最小只读子集缺 A/B 门前无法证明行为等价，风险高于先建安全网。
2. **本轮不碰 canvas path-objects（M8）**：`git log` 核实 canvas 流正在活跃编辑 `part05.js` canvas 段（`f7219b2c` 等），按入口文档护栏「碰 canvas 共享面前先 git log，有活跃编辑则转零碰撞面」，转 DOM 面（零碰撞）。
3. **A/B 对照哲学**：聚焦可观测行为等价（同 HTML + 同读操作 → 同返回串），不强求 API 形态同构（native 真对象 vs polyfill Proxy）。native 侧桥 `document` 到 global 使两路径共用 `document.querySelector(...)` 脚本形式。

## 验证证据

| 矩阵 | 命令 | 结果 |
|------|------|------|
| zero-engine v8 lib | `cargo test -p zero-engine --features v8 --lib` | ✅ 2063 passed（含新 4 个 A/B 门） |
| zero-engine quickjs lib | `cargo test -p zero-engine --no-default-features --features quickjs --lib` | ✅ 1405 passed（模块 cfg 排除） |
| zero-webview v8 | `cargo test -p zero-webview --features v8` | ✅ 17 passed（native_dom 接线回归） |
| clippy v8 | `cargo clippy -p zero-engine --features v8 --all-targets -- -D warnings` | ✅ 零警告 |
| clippy quickjs | `cargo clippy -p zero-engine --no-default-features --features quickjs --all-targets -- -D warnings` | ✅ 零警告 |
| fmt | `cargo fmt --all -- --check` | ✅ 无 diff |

**核心结论**: **native 读路径 ≡ polyfill 读路径**（行为等价实证）。M1 L2-read-only 切片可直接复用本 A/B 门作验收。

## 未跑

- `make test` 全量（workspace + quickjs 矩阵单次 >580s 超时）。聚焦验证已覆盖变更面（纯测试新增 + mod.rs 注册两行，无生产代码改动）。

## 勘误（本轮发现，已写入 master.md）

1. dom_bindings native API 面**比入口文档基线描述更完整**：`mod.rs:558-624` 已注册 querySelector 族 + createElement/Text/Comment/Fragment + documentElement/body/head 全套工厂。
2. canvas path-objects 缺口**比基线描述更严重**：本地 `wpt-data/html/canvas/element/` 整个 canvas element 子树不存在，`make fetch-wpt-data`（v1.10）只含 reftest 数据，canvas testharness 用例须从上游 wpt 仓库单独导入。
