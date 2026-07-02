# Zero UI SDK 抽取与浏览器迁移 — 运行时控制面板

> **入口文档**：`docs/goal/ui-sdk.md`（goal contract，稳定）。本文件是持续演进的增量控制面，**不是**一次性交付物。创建第一版只表示治理框架建立，**不表示**核心目标已被覆盖。
> **上游权威**：`docs/specs/ui-sdk-spec-rfc.md`（v1.6.1）。实现具体接口/模块/组件时必须查阅 spec 对应章节。
> **姊妹目标**：`docs/goal/rendering-compat.md`（只读引用，不得改写；其 `make test`/`make reftest`/`make product-smoke` 主线不得因本目标退化）。
> **工作分支**：`ui-sdk`。

---

## Active Milestone

**M1 — UI SDK 核心骨架（广度优先，spec §M1）**。依赖：无（与浏览器并存）。浏览器运行时不触碰。

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
| DC-17 | Coverage 量化 | ⬜ 待基线 | M1 crate 建立后跑 `scripts/check-coverage.sh` 取 ui/* 基线 |
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
- `ui/*` + `foundation/text` coverage：**待建立**。M1 crate 落地后跑 `scripts/check-coverage.sh`（cargo-llvm-cov）取基线，曲线写入本节。目标 ≥ 85%。

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
- `evidence/dep-isolation-*.txt` — Wave 4 落盘（DC-1 机械验证）。
- `evidence/capability-matrix-*.md` — Wave 4 落盘（M1 已验证能力矩阵 + 缺口）。
- `evidence/coverage-*.txt` — M1 crate 落地后补。

## Next Steps

1. **Wave 1**：`ui/core`（geometry/event/widget/element/action/binding/theme/focus/semantics/invalidation/layout）+ `foundation/text`（IF-008 接口），真实类型 + 单测，加入 workspace，scoped test-guard 全绿。
2. **Wave 2**：`ui/render` / `ui/i18n` / `ui/runtime` / `ui/widgets` / `ui/patterns`。
3. **Wave 3**：13 能力域 + `ui/dsl` schema skeleton。
4. **Wave 4**：`ui/adapters/{winit,webview}` + `browser-ui/chrome` + DC-1 dep-isolation evidence。
5. 每波：`cargo build --workspace` + scoped test + clippy + fmt + 提交推送。
6. M1 crate 全落地后：跑 `scripts/check-coverage.sh` 取 ui/* 基线；写 M1 收口 archive。
