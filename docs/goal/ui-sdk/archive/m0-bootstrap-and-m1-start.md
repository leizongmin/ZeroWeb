# Archive — M0 文档/架构边界确认 + M1 启动（2026-06-30）

## M0 交付（本 goal contract 即产出）

- 入口文档 `docs/goal/ui-sdk.md`（v1.0）+ 上游 spec `docs/specs/ui-sdk-spec-rfc.md`（v1.6.1）已存在。
- 本轮完成文档治理 bootstrap：创建 `docs/goal/ui-sdk/master.md`（运行时控制面）、`archive/`、`evidence/`。
- 首轮强制 checklist 全部执行：复核仓库事实、确认 done criteria、创建控制面、确认测试基线、选定 M1。

## 仓库事实复核（与 spec §6.5A、goal「Current Proven Baseline」比对）

均不存在：`ui/`、`foundation/`、`browser-ui/`、`docs/goal/ui-sdk/{master.md,archive,evidence}`、三棵树/retained 运行时/Theme/i18n/DSL。
可复用：render-foundation（含 `src/font`）、host-runtime（winit）、webview、browser-shell；workspace 已声明 fontdue/swash/rustybuzz/unicode-bidi/taffy(w本地 patch)/winit。
耦合热点：`apps/browser/src` 约 19.7k 行自绘 UI，多文件 >2000 行（app_input 2910 / app_render 2816 / main 2346 / headless 1761）——迁移时需拆分。

## M0 关键决策（详见 master.md 依赖决策日志）

- TBD-8：text foundation 独立为 `foundation/text`（不污染纯文本层，M2 桥接 render-foundation font 实现）。
- TBD-2：ui/render 不直接依赖 render-foundation，M1 自立 Scene/RenderNode 抽象，M2 trait 桥接。
- TBD-9：复用 workspace 已声明 fontdue/swash/rustybuzz/unicode-bidi，零新增依赖。
- TBD-7：i18n M1 手写 minimal plural/RTL，不引入 ICU4X/Fluent。

## 测试基线（首轮）

`make test` 本机 RED（script-sandbox debug-test V8/advapi32 链接环境失败，非本目标引入；release 绿、CI 绿）。UI SDK 验证改用 scoped test-guard。详见 `evidence/test-20260630-223559.txt`。

## M1 启动（本轮起）

广度优先：把 spec §FR-002 全部 crate 立起来（接口 + 最小单测）。本轮分 4 波推进（core/text → render/i18n/runtime/widgets/patterns → 13 能力域+dsl → adapters+browser-ui/chrome）。每波 scoped test-guard 验证 + 提交。
