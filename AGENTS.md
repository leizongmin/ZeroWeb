# AGENTS.md

## 项目与入口

ZeroWeb 是 MIT 许可的 Rust 跨平台浏览器项目，交付嵌入式 `ZeroWebView` 库和 `ZeroBrowser` 应用。桌面支持 macOS/Linux/Windows，Android 使用 Kotlin/Compose 宿主与 Rust JNI 桥。Rust edition 2024，MSRV 1.85；workspace 有 33 个 member（22 库、8 应用、2 测试工具、1 开发工具）。

- 目录、职责和页面管线见 [架构总览](docs/architecture.md)；依赖与 member 以 [Cargo.toml](Cargo.toml) 为准。`crates/taffy-local` 通过 `[patch.crates-io]` 接入，不在显式 member 清单。
- 开发环境：[Linux/macOS](docs/development/linux-macos.md)、[Windows](docs/development/windows.md)；Android 进度见 [任务控制面](docs/goal/archive/android-browser/master.md)。
- 启动浏览器用 `make browser`，仅构建用 `make browser-build`；workspace 构建用 `make build`；完整测试用 `make test`；WPT reftest 用 `make reftest`。
- 运行构建、测试、clippy 或准备提交前，必须读取 [命令与质量门禁](docs/engineering/quality-gates.md)，按实际影响执行检查，不把命令表当每次必跑清单。

## 编码准则

准则整个会话持续生效，不因压缩或任务切换遗忘。谨慎程度按任务复杂度判断。

1. 实现前说明假设与取舍；不同理解要说明，不能静默选择或掩盖困惑。无法确定的关键问题要提问；有更简单方案要指出。
2. 依次优先复用已有实现、标准库/平台能力、已装依赖和最小实现。只做需求内功能，不为一次使用引入抽象、配置或不可能场景的错误处理；代码能明显缩短就简化。
3. 信任边界输入校验、防数据丢失的错误处理、安全措施及无障碍属性不可简化。
4. 精准修改并匹配原风格，不顺手重构、清理或改相邻注释。只删除本次修改造成的孤立 import/变量/函数；既有无关死代码只提及，不删除。每行变化应能追溯到需求。
5. 多步骤先列简要计划和各步检查。成功标准来自需求：校验覆盖非法输入，bug 测试复现目标行为，重构保证前后测试通过；循环验证直到达成。
6. 修 bug 先追踪调用方、定位根因，优先修共享路径；仅压制症状时继续排查。
7. 单个源码文件不超过 2000 行，特别是 `.rs`；超限按职责、高内聚低耦合拆分。
8. 路径使用环境变量或相对路径，日志引用项目相对路径；不硬编码本机目录、用户名、主机名。既有泄漏修复见 commit `1bb7c6ca6`（SL-008/SL-010）。

快速原型/spike 可放宽简洁与修改范围；用户确认不需要测试的紧急热修复可放宽验证。用户明确说“跳过准则”“不用讲究”或同义表述时暂停，恢复编码任务后自动生效；安全、输入校验和防数据丢失始终不能跳过，其他放宽结束后恢复。

代码风格：

- 使用 rustfmt；公共 API 必须有 `///` 文档注释；日志使用 `tracing`，不用 `println!`。
- 实现 HTML/CSS/DOM/JS 规范行为处必须添加对应规范链接；未实现步骤标 `// FIXME:`；优化路径标 `// OPTIMIZATION:` 并说明理由。
- 编写、审查、重构本仓代码及排查兼容性/性能时，读取 [zeroweb-guidelines](.agents/skills/zeroweb-guidelines/SKILL.md)，并按子系统查 [learning 索引](docs/learnings/INDEX.md)。
- 本项目知识与验证 Skill 使用 `zeroweb-knowledge`、`zeroweb-verify`，初始化和维护均覆盖通用 Skill 的固定名称与路径。编码前独立检查 `.agents/skills/zeroweb-knowledge/SKILL.md` 与 `.agents/skills/zeroweb-verify/SKILL.md`；存在则按任务读取知识索引或功能判据，并核对当前源码。初始化不自动采用提交前维护，也不代表功能已通过。

## 架构边界

- `zero-webview` 是稳定嵌入边界；浏览器像外部宿主一样优先通过该边界接入页面能力，不随意绕过底层。
- `zero-protocol` + `apps/renderer` 定义 IPC 边界；导航、输入、存储、网络代理变化先核对契约，同步主进程与 renderer 两端。
- `zero-engine` 串联 DOM/CSS/样式/布局/绘制，`render-foundation` 输出 GPU/CPU 图元，职责不混写。
- `zero-media` 解码与播放解耦；HTMLMediaElement 语义在 engine/webview，不混入媒体底座。
- Android JNI、角色生命周期和输入转发变化同步核对 Kotlin 声明、Rust 桥和 IPC。
- JS/WASM 集成保持 `script-sandbox`、`wasm-sandbox` 的 feature gate、宿主桥接与错误边界清晰。
- 行为回归优先落到最贴近的 `tests/integration` 或 `tests/wpt-runner`；兼容性修复必须附常驻 WPT/reftest 或等价本地测试，与修复同一提交，细则见质量门禁。

## 安全与运行

- 不提交 `.env`、`credentials.json` 等敏感文件，不硬编码密钥、密码或 token；认证/授权变化须谨慎核验。
- 禁止 `push --force`、`reset --hard`、`clean -f` 等破坏性 Git 操作。
- `git commit` 前必须调用 `lei-pre-commit-guard` 扫描暂存区并获得 PASS；BLOCK 时修复重扫或等待用户确认，用户明确要求跳过可豁免。扫描、行为审查和仓库审批互不替代，也不自动授予提交、发布或合并权限。
- 无人值守测试、构建和 clippy 必须同时限制进程树内存与墙钟时间；使用既有守卫，禁止裸跑。阈值、编译阶段与 Linux 兜底见 [质量门禁](docs/engineering/quality-gates.md) 及 [OOM 防护](docs/rally/oom-guard.md)。
- 病态输入、fuzz 复现与最小化无论是否有人观察，都必须走 [fuzz-repro-guard.sh](scripts/fuzz-repro-guard.sh)，先预编译再限制内存与超时。

## 网站长任务

启动/继续 `zeroweb-site-optimizer` 时必须读取 [恢复入口](.agents/skills/zeroweb-site-optimizer/references/recovery.md)，从原运行目录生成恢复导航。压缩、交接、新会话及发布/审查/合并阶段切换后重新读取当前阶段原规则；摘要中的“已读/已完成”不能替代原账本和证据。只讨论或修改 Skill 时不启动或恢复网站任务。运行中自进化提案和无法自行修复的提交门禁 BLOCK 按该 Skill 的 [启动授权](.agents/skills/zeroweb-site-optimizer/references/intake.md) 记入待决清单、继续其他任务，停止或交付时集中提出；BLOCK 项仍不得提交。

网站任务 PR 创建后及返修/交付前，必须读取并执行 [双审查流程](.agents/skills/zeroweb-site-optimizer/references/independent-review.md) 和 [GitHub 交付规则](.agents/skills/zeroweb-site-optimizer/references/github-delivery.md)。执行独立双首轮、统一返修和非作者复核，证据及正文绑定当前版本；就绪、pr_only 完成或合并前核验远端与适用截图。具体角色、输入隔离、回执和例外以原流程为准；本地检查器不替代真实审查、CI 或 GitHub 审批，不扩大为所有本地提交前提。

## 写作与规则维护

文档、commit、PR 和飞书进度通知先写具体变化或卡点，再写验证和未解决项。用常用词、短句和事实，删除套话、重复背景和无证据形容词；必要术语首次出现时解释。定稿参考 `lei-humanizer` 检查拗口和冗长，不套文学化写法。

提交标题说明改动和结果，例如 `fix(layout): 修复返回列表后内容空白`；正文只补必要原因、验证和限制。简洁不能省略准确数字、证据链接、失败检查、安全说明及未解决问题。

更新 `docs/goal/*/master.md` 时，开头写当前状态、卡点和下一步，标日期或版本；新进展只记变化、结果和证据。同一事实不在多处重复；长日志、测量和排查放已有 `evidence/` 或 `archive/` 并链接。保留有效决策、历史和失败证据，不为缩短改写事实。

- 发现可复用技术经验时，必须读取 [经验沉淀流程](docs/engineering/learnings.md)，保存到 `docs/learnings/`；不手改生成索引。
- 发现可复用 Agent 行为、工具用法或长期偏好时，必须读取 [自进化边界](docs/engineering/self-evolution.md) 并使用 `lei-self-evolution`；未授权提案先静默草案，输出纯文本审批请求后停下，不能先改目标规则。
- 创建、合并或精简 `AGENTS.md` 时，必须读取 [规则维护流程](docs/engineering/agents-maintenance.md)，语义去重、更新原参考并同步维护入口，不改变有效约定。
