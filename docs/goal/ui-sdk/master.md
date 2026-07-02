# Zero UI SDK 抽取与浏览器迁移 — 运行时控制面板

> **入口文档**：`docs/goal/ui-sdk.md`（goal contract，稳定）。本文件是持续演进的增量控制面，**不是**一次性交付物。创建第一版只表示治理框架建立，**不表示**核心目标已被覆盖。
> **上游权威**：`docs/specs/ui-sdk-spec-rfc.md`（v1.6.1）。实现具体接口/模块/组件时必须查阅 spec 对应章节。
> **姊妹目标**：`docs/goal/rendering-compat.md`（只读引用，不得改写；其 `make test`/`make reftest`/`make product-smoke` 主线不得因本目标退化）。
> **工作分支**：`ui-sdk`。

---

## Active Milestone

**M1 — UI SDK 核心骨架（广度优先，spec §M1）**。依赖：无（与浏览器并存）。浏览器运行时不触碰。

**🟢 M1 Wave 1-4 crate skeleton 已全部落地（2026-06-30）**：23 个新 crate（22 通用 + 1 共享 + 2 耦合点中
adapter-webview/chrome 计入）存在、可编译、有无窗口单测（合计 135 passed / 0 failed）；
`cargo build --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all` 全净；
DC-1 依赖隔离机械验证通过（见 evidence/dep-isolation-20260630-234530.txt）。

M1 剩余收口项：①coverage 基线（`scripts/check-coverage.sh`，DC-17）；②`ui/examples` 在 M3。
M1 目标：把 spec §FR-002 列出的 crate 全部立起来（接口边界 + 无窗口单测），让架构在编译期与测试期成立；`ui/core` 三棵树 + 单向数据流 + 局部失效有最小可用单测；text foundation 接口接入 `ui/render`（skeleton 级）。浏览器 chrome 迁移、完整 DSL 表达式语言、移动端运行时分别在 M2/M3/M4。

## Done Criteria 进度

| DC | 标题 | 状态 | 证据 / 备注 |
|----|------|------|------------|
| DC-1 | 目录与依赖隔离 | 🟡 进行中（M1） | ui/ + foundation/text + browser-ui/chrome crate skeleton 落地中；依赖隔离脚本待 Wave 4 出 evidence |
| DC-2 | 三棵树 + 单向数据流 + retained | 🟡 skeleton（M1） | ui/core Widget/Element/RenderNode/EventResult/Invalidation 类型落地中；单测随 Wave 1 |
| DC-3 | WebViewWidget 自定义组件 | ⬜ M2 | adapter/webview skeleton 在 Wave 4；真实接入 M2 |
| DC-4 | 滚动语义与滚动条边界 | ⬜ M2 | ScrollBar skeleton 在 Wave 2；迁移自 page_scroll.rs 在 M2 |
| DC-5 | 主题系统 | 🟡 skeleton（M1） | ui/core::theme 类型 + ui/runtime::theme_provider 在 Wave 1/2 |
| DC-6 | YAML DSL + 完整表达式语言 | ⬜ M3 | ui/dsl schema skeleton 在 Wave 3；完整表达式语言 M3 |
| DC-7 | 首批组件（通用+组合+浏览器） | ⬜ M2 | widgets/patterns skeleton 在 Wave 2；browser-ui/chrome skeleton 在 Wave 4；实现在 M2 |
| DC-8 | 无障碍/焦点/IME | 🟡 skeleton（M1） | ui/core::focus/semantics + ui/runtime::ime/accessibility |
| DC-9 | 局部失效刷新 | 🟡 skeleton（M1） | ui/core::invalidation needs_layout/paint 区分单测在 Wave 1 |
| DC-10 | 国际化资源与 message id | 🟡 skeleton（M1） | ui/i18n IF-007 接口在 Wave 2 |
| DC-11 | 共享文本/字体基础层 | 🟡 skeleton（M1） | foundation/text IF-008 接口在 Wave 1；真实共享接入 M2 |
| DC-12 | 响应式/自适应 + 移动 chrome | ⬜ M2/M4 | WindowMetrics/ViewportClass 在 ui/core::layout（Wave 1）；shell 在 M2/M4 |
| DC-13 | 完整应用级 UI 能力（13 域） | 🟡 skeleton（M1） | 13 域接口 + 最小单测在 Wave 3；浏览器接入在 M2-M4 |
| DC-14 | 浏览器迁移完成 + 零退化 | ⬜ M2-M4 | 浏览器运行时本轮不触碰 |
| DC-15 | 移动端运行时 | ⬜ M4 | — |
| DC-16 | 测试与质量不可退让 | 🟡 守门中 | 新 crate scoped test 全绿；详见 Testing & Quality Gates |
| DC-17 | Coverage 量化 | 🟡 基线已立（M1） | 聚合 line 89.89% / function 89.98% / region 90.65%（≥85%）；per-crate 7 项 <85% 待补（见 Coverage 基线节曲线） |
| DC-18 | 证据持久化与文档自洽 | 🟡 守门中 | evidence/ 持续落盘；本表与 Latest Evidence 自洽 |

图例：⬜ 未开始 / 🟡 进行中或 skeleton / ✅ 完成。

## Current Proven Baseline（首轮复核，2026-06-30）

复核结论与入口文档「Current Proven Baseline」一致，**均不存在**项确认：

- `ui/` 顶层目录 — **不存在**。
- `foundation/` — **不存在**（text foundation 待新建）。
- `browser-ui/` — **不存在**。
- `docs/goal/ui-sdk/{master.md,archive/,evidence/}` — **不存在**（本轮创建）。
- Widget/Element/Render 三棵树、retained 运行时、Theme/i18n/DSL/表达式语言 — **均不存在**。

已有可复用基础（SDK 将扩展而非重写）：

- `apps/browser/src`：自绘 UI 耦合在 `app.rs`/`app_render.rs`/`app_input.rs`/`page_scroll.rs`/`colors.rs`/`app_platform.rs`/`layout.rs` 等（合计约 19.7k 行；多个文件 >2000 行：`app_input.rs` 2910、`app_render.rs` 2816、`main.rs` 2346、`headless.rs` 1761 — 迁移时需注意拆分）。
- `crates/render-foundation`：渲染后端（GPU/CPU 图元 + `src/font/{cache,loader,shaper}` 字体栈 + image_cache）；依赖 wgpu/tiny-skia/resvg/png。
- `crates/host-runtime`：winit 0.30 窗口/事件/IME/surface。
- `crates/webview`：嵌入式 WebView 稳定边界。
- `crates/browser-shell`：浏览器业务状态（标签/书签/历史/设置/下载，UI-agnostic）。
- workspace 已声明依赖：`fontdue 0.9`/`swash 0.2`/`rustybuzz 0.20`/`unicode-bidi 0.3`/`taffy 0.7`(本地 patch)/`winit 0.30`/`serde`/`serde_json`/`thiserror`/`tracing`/`slotmap`/`hashbrown`/`compact_str`/`parking_lot`。

## Testing & Quality Gates（本机实测基线）

**⚠️ 本机 `make test` 基线状态：RED（环境性，非本目标引入）**

- 本轮首轮跑 `make test`（= `test-guard -- cargo test --workspace --exclude zero-render-foundation`）在 **`zero-script-sandbox` (lib test) 链接阶段失败**：MSVC `link.exe` 报 `LNK2019: 无法解析的外部符号 __imp_EventRegister / RegOpenKeyExW / ...`（`advapi32` 系符号），源自 `rusty_v8` 的 debug test 二进制。
- **关键定性**：`cargo build --release -p zero-script-sandbox` **成功**（0.86s）；仅 debug **test** 二进制链接失败。即 V8 库与环境本身正常，是 rusty_v8 debug-test 在本 MSVC BuildTools 上未自动链接 `advapi32` 的既有环境问题。
- **与本目标无关**：`zero-script-sandbox` 属禁止修改区（脚本 sandbox），且为环境/工具链问题，非代码/逻辑失败。CI（Linux，真后端）按 `ci.yml` 跑全量 `cargo test --workspace` 正常。
- **UI SDK 验证路径**（本轮及后续）：对新增 `ui/*` + `foundation/text` + `browser-ui/chrome` 使用 **scoped test-guard**：`./target/test-guard -- cargo test -p zero-ui-<crate>`（OOM 防护 + 不触碰 script-sandbox）；并用 `cargo build --workspace` 确认不破坏其它 crate 编译。最终门禁仍以 `make test` 为准（待环境恢复或 CI 验证）。
- 跟踪项：本机 script-sandbox debug-test 链接环境问题记入「未解决缺口」，不阻塞本目标推进（属环境，非 UI SDK 代码）。

**其它门禁**：单 `.rs` ≤ 2000 行；`cargo fmt` 无变更；clippy `-D warnings`；新增能力必须带测试；依赖仅 MIT/Apache-2.0/BSD 且最小化。

## Coverage 基线

- 全仓 floor（来自 `rendering-compat` 基线）：line 95.46% / function 96.94% / region 94.88%。本目标不得显著下降。
- **`ui/*` + `foundation/text` + `browser-ui/chrome` DC-17 基线（2026-07-01 建立）**：
  - 聚合：**line 89.89% / function 89.98% / region 90.65%**（3315 行，已超 85% 目标）。
  - 命令（统一口径）：`cargo llvm-cov -p zero-text-foundation -p zero-ui-* ... -p zero-browser-chrome --summary-only --ignore-filename-regex '[\\/](crates|apps)[\\/]' -- --test-threads=1`（`--ignore-filename-regex` 必须用 `[\\/](crates|apps)[\\/]` 匹配 profdata 绝对路径中的目录分量，`^` 锚定不生效）。
  - 证据：`evidence/coverage-20260701-021806.txt`。
- **per-crate line 曲线**（向 85% 推进的焦点标 ▲）：

  | crate | line | 备注 |
  |-------|------|------|
  | ui/assets · ui/collections · ui/forms · ui/platform | 100% | — |
  | ui/animation 98.4% · ui/patterns 98.2% · ui/devtools 97.3% | — | — |
  | browser-ui/chrome 96.3% · ui/navigation 95.4% · ui/design-system 94.9% · ui/i18n 94.1% | — | — |
  | ui/adapters 93.5% · ui/core 92.2% | — | — |
  | ui/render 88.0% · ui/runtime 87.1% | — | — |
  | ui/testing 85.1% | — | 刚过线 |
  | ▲ ui/dsl 84.6% · ▲ ui/widgets 84.6% · ▲ ui/gestures 84.4% | <85% | widgets 主要被 button.rs 61% 拖低 |
  | ▲ foundation/text 82.8% | <85% | shaping/text_blob/text_measure 三个 M1 stub 0%（M2 真实现后补测） |
  | ▲ ui/restoration 79.3% · ▲ ui/overlay 77.5% · ▲ ui/commands 75.0% | <85% | 优先补测目标 |

- **下一步 coverage 推进**：先补 ui/commands / ui/overlay / ui/restoration / ui/widgets::button 单测（纯逻辑、无窗口，本轮或下一轮可做），把曲线整体推到 ≥85% per-crate。foundation/text stub 等待 M2 真实现。

## 依赖决策日志

| 决策 | 结论 | 理由 | Spec 锚点 |
|------|------|------|-----------|
| TBD-8 text foundation 落点 | **新建独立 `foundation/text`（`zero-text-foundation`）**，不复用 `crates/render-foundation/src/font` 作为最终落点 | render-foundation 耦合 wgpu/png/resvg/tiny-skia 图形后端；纯文本/字体基础层应独立，避免被图形后端污染，且 UI 与 WebView 都能依赖而不拖入 GPU 栈。M2 再把 render-foundation 现有 fontdue/swash/rustybuzz 实现迁移/桥接到 foundation/text | spec §6.5A / IF-008 / TBD-8 |
| TBD-2 ui-render 与 render-foundation 依赖方向 | **M1 `ui/render` 定义自己的 Scene/RenderNode/PaintCtx 抽象，不直接依赖 render-foundation**；通过 trait 在 M2 桥接 render-foundation 后端 | 通用 UI 层不应被 wgpu 后端耦合；M1 只需 scene 抽象 + 单测 | spec §8.4.1 / TBD-2 |
| TBD-9 text shaping/font 依赖 | **复用 workspace 已声明** fontdue/swash/rustybuzz/unicode-bidi，零新增依赖 | 已在 workspace.dependencies；M1 skeleton 用 stub 实现，M2 接真实栈 | spec §6.4 / TBD-9 |
| TBD-7 i18n 依赖 | **M1 手写 minimal plural/RTL**，不引入 ICU4X/Fluent | 接口先行；依赖评估留 M3 | spec §6.5A / TBD-7 |
| TBD-6 表达式 parser | **M3 评估**，M1 不涉及 | DSL 表达式在 M3 | TBD-6 |
| TBD-1 serde_yaml | **M3 评估**，M1 dsl 只立 schema skeleton | — | TBD-1 |

依赖自治条款（已与用户确认）：YAML 解析、表达式 parser、i18n、text shaping/font、布局均由执行器自主决策，硬约束 = 仅 MIT/Apache-2.0/BSD、最小化、优先复用 workspace 已有 crate、论证写入本表 + archive。

## Latest Evidence

- `evidence/test-20260630-223559.txt` — 首轮 `make test` 实测：RED（script-sandbox debug-test V8 链接环境失败；release 构建 + CI 绿；定性见上）。
- `evidence/dep-isolation-20260630-234530.txt` — **DC-1 机械验证 PASS**：22 通用 crate 零浏览器依赖；adapter-webview→zero-webview；chrome→ui/*+browser-shell+adapter-webview。
- `evidence/capability-matrix-20260630-234530.md` — M1 能力矩阵 + DC skeleton 证据锚点 + 未解决缺口。
- `evidence/coverage-20260701-021806.txt` — **DC-17 基线（2026-07-01）**：聚合 line 89.89% / function 89.98% / region 90.65%（ui/*+foundation/text+chrome，排除依赖污染）；per-crate 曲线见 §Coverage 基线。

**M1 门禁实测（2026-07-01 复核）**：
- `cargo build --workspace` — Finished（0 错误）。
- `cargo clippy --workspace --all-targets -- -D warnings` — Finished（0 警告；clippy 不链接，故绕过 script-sandbox V8 链接问题）。
- `cargo fmt --all --check` — 净（0 diff）。
- 新 crate 单元 + 集成测试（scoped test-guard）— render 6+3 / runtime 7+3 / testing 3+1 / core 31 / widgets 18 / i18n 12 / foundation/text 12 / patterns 7 / … 全绿。
- DC-17 coverage 基线已立（见上）。

## Next Steps

1. **DC-17 per-crate 抬升**：补 ui/commands（75%）/ ui/overlay（77.5%）/ ui/restoration（79.3%）/ ui/widgets::button（61%）单测，把曲线整体推到 per-crate ≥85%（纯逻辑、无窗口、本轮/下一轮可做）。foundation/text 三个 stub（shaping/text_blob/text_measure）等 M2 真实现后补测。
2. **M1 收口 archive**：把 M1 skeleton 过程/决策/证据归档到 `archive/m1-skeleton-complete.md`，DC-1/DC-2/DC-9/DC-11 skeleton 等标记收口。
3. **进入 M2**（按 §Ordered Next Milestones）：browser-ui/chrome 其余 §8.4.1A 组件实现；
   text foundation 接入 `ui/render` 与 `zero-webview`（真实 fontdue/swash/rustybuzz 桥接，DC-11）；
   `ui/render` Scene → render-foundation 后端 trait（TBD-2）；`apps/browser` 逐组件灰度迁移（shim/feature-flag）。
4. `ui/examples`（counter/form/browser-shell-demo）随 M3 落地（DC-14）。
5. 跟踪项：本机 `make test` 受 script-sandbox debug-test V8 链接阻塞（环境性）；零-security cors.rs 曾瞬时工作树损坏（已恢复 HEAD 状态，git 干净）。
