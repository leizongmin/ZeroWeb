# 贡献责任边界

本文件是 ZeroWeb 贡献责任、风险等级和 owner 路由的权威政策。设计依据见 [贡献责任边界 Spec/RFC](../specs/contribution-responsibility-boundary.md)。

## 当前治理状态

- **成熟度**：G0（单维护者）
- **项目维护者**：`@leizongmin`
- **默认 owner**：`@leizongmin`
- **贡献模式**：开放 issue、proposal、PR 和非代码贡献

G0 不具备双人批准条件。关键变更使用一名人类责任维护者加完整证据包，不能把同一人的多次操作描述为双人 review。

## 核心原则

1. 补丁作者负责说明和验证改动，但不自动获得代码所有权。
2. 每项合入变更必须由一名人类责任维护者明确接管。
3. 领域 owner 负责专域正确性，责任维护者决定项目是否长期接纳。
4. AI 和 bot 可以辅助工作，但不能成为作者批准、reviewer、owner、责任维护者或批准票。
5. `CODEOWNERS` 只负责评审路由和平台门禁，不代表 owner 已经接管变更。
6. 测试期望、Oracle、baseline、阈值和豁免列表属于产品事实，不能按普通测试文件降级。

## 角色

### Contributor

Contributor 可以提交 issue、proposal、代码、测试、文档和 reduction。Contributor 必须解释最终改动、提供验证证据，并披露 AI 参与和外部来源。

### Domain Owner

Domain Owner 判断责任域内的不变量、风险和验证是否满足要求。Owner 不独占修改权，也不因一次 review 自动承担合入后的责任。

### Responsible Maintainer

Responsible Maintainer（RM）必须具备仓库合并权限，并承担：

- sponsor 变更并确定最终风险等级；
- 确认所需 owner 和验证门禁；
- 对最终 commit SHA 作 adoption 决定；
- 合入后负责首轮回归分诊；
- 协调修复、回滚或责任转交。

没有 RM 的变更不能合入。外部贡献者声明“后续由我维护”不能替代 RM。

## 风险等级

最终风险取路径风险下限和语义风险中的较高者。无法确定时按较高一级处理，直到 RM 给出可审计的降级理由。

| 等级 | 定义 | 典型变更 | 最低门禁 |
|------|------|----------|----------|
| C0 开放 | 不改变规范行为、产品承诺、指标或执行路径 | 非规范文档的拼写、格式或注释修正 | 1 名 RM；目标检查 |
| C1 受管核心 | 改变普通功能、渲染语义、产品行为或规范文档 | DOM/CSS/布局/绘制、产品功能、测试框架、Spec | RM + domain owner；相关测试；跨域时先 proposal |
| C2 关键边界 | 改变信任边界、稳定 API、IPC、供应链、发布、治理或质量事实 | 安全/网络/存储/协议/沙箱、webview API、依赖、CI、Oracle baseline | accepted proposal；G0 证据包；完整相关门禁；回滚方案 |
| C3 受限 | 涉及未披露漏洞、密钥、签名、发布凭据或仓库权限 | embargo 修复、凭据轮换、管理员操作 | 私密通道；授权维护者；披露后才公开 |

以下变更无论路径都至少是 C2：

- 新增、删除或升级第三方依赖，或改变 `Cargo.lock` 解析结果；
- 修改安全默认值、跨源/权限判断、沙箱限制、IPC 消息或序列化兼容性；
- 修改 `ZeroWebView` 公开 API 的兼容性承诺；
- 修改 required checks、发布产物、下载源、签名或打包流程；
- 修改 Oracle 图片、通过率分母、阈值、豁免/跳过列表、基准基线或公开质量数字；
- 修改本政策、`CODEOWNERS` 语义或分支保护规则。

## 责任域

| 责任域 | 路径/语义 | 风险下限 | 当前 owner |
|--------|-----------|----------|------------|
| Governance & Supply Chain | `.github/CODEOWNERS`、workflows、Dependabot、根构建/依赖/许可证/治理文件、发布/下载/打包脚本 | C2 | `@leizongmin` |
| Web Semantics | `crates/dom`、`css-parser`、`style-system`、`layout-engine`、`engine`、`canvas` | C1 | `@leizongmin` |
| Patched Dependency | `crates/taffy-local`、`[patch.crates-io]` | C2 | `@leizongmin` |
| Rendering & Platform | `crates/render-foundation`、`host-runtime` | C1；unsafe/FFI/GPU 隔离语义为 C2 | `@leizongmin` |
| Runtime Trust Boundary | `crates/net`、`security`、`storage`、`protocol`、`script-sandbox`、`wasm-sandbox`、`page-runtime`、`apps/renderer` | C2 | `@leizongmin` |
| Stable Embedding API | `crates/webview` | C2 | `@leizongmin` |
| Product | `crates/browser-shell`、`apps/browser`、`apps/webview-demo`、产品资产 | C1 | `@leizongmin` |
| Quality Control | integration、WPT runner、benchmark、test-guard、测试脚本 | C1；基线/阈值/豁免/指标为 C2 | `@leizongmin` |
| Normative Docs | Spec、architecture、goal、roadmap、changelog、贡献模板和治理政策 | C1；治理、安全路由和指标控制面为 C2 | `@leizongmin` |
| Informational Docs & Tools | research、一般 README、`tools/icon-gen` | C0；可执行逻辑或产品资产变化为 C1 | `@leizongmin` |

所有未显式列出的路径回退到默认 owner。更精确的路径映射见 [`.github/CODEOWNERS`](../../.github/CODEOWNERS)。

## 合入流程

1. 作者在 PR 中建议风险等级并列出责任域。
2. RM 确认分类。跨两个及以上责任域或命中 C2 时，先关联已接受的 proposal 或私密安全上下文。
3. 作者补齐实现、测试、文档、来源和未覆盖风险。
4. Domain Owner 审查领域不变量，自动门禁验证代码。
5. RM 只对最终 commit SHA 作 adoption；后续 push 使旧记录失效。
6. 合入后若出现高严重度回归，优先回滚，再决定修复或重做。

### G0 自提交

GitHub 不允许 PR 作者批准自己的 PR。维护者自提交时：

- 在最终 commit SHA 上留下明确的 adoption 记录；
- C2 附完整证据包；
- 不把该记录表述为 GitHub approval 或双人 review。

### C2 证据包

- 已接受的 proposal 或安全上下文；
- 针对最终 diff 的独立二次审阅记录；
- 相关全量测试、安全检查、兼容性或性能结果；
- 回滚触发条件和具体步骤；
- 使用 AI 时，对关键不变量和来源/许可证边界的人工复核。

紧急安全修复或回滚可以先处置，但必须补齐不敏感的决策记录。

## AI 辅助贡献

- 提交者必须理解并能解释最终 diff。
- PR 必须说明 AI 的作用和人工验证范围，不需要附完整对话。
- AI 生成内容必须满足与人工内容相同的测试、错误处理、文档和许可证要求。
- 提交者无法解释关键不变量或失败模式时，RM 必须暂缓或关闭 PR。

## Owner 任命与回收

Owner 必须证明理解责任域的不变量、验证方式、常见回归和回滚路径，并由项目维护者通过 C2 治理 PR 任命。提交数量、活跃天数或 AI 产出规模不能自动换取 owner 身份。

出现以下任一情况时，可启用 fallback owner：

- owner 主动声明不可用；
- owner 失去仓库权限；
- owner 对两次间隔至少 7 个自然日的责任请求连续 30 个自然日无响应。

没有 fallback 时，默认 owner 临时接管。该责任域只接受安全修复、回归修复和必要维护，新功能可以冻结。治理矩阵和 `CODEOWNERS` 必须在同一 PR 更新。

## 非代码贡献与安全报告

项目继续接受 bug report、最小复现、WPT/reftest reduction、网站测试、标准/设计讨论、文档反馈和安全报告。

C3 信息只能进入 [安全报告流程](../../SECURITY.md)。公开 issue、proposal、PR、commit message 和 `CODEOWNERS` 不得包含未披露漏洞细节、私密 advisory 编号、密钥或凭据。

## G1 激活条件

只有以下条件全部满足，并通过 C2 治理 PR 后，项目才能从 G0 切换到 G1：

1. 至少两名人类拥有合并权限并接受维护者责任；
2. C2 责任域至少有一名主 owner 和一名 fallback；
3. 平台门禁能阻止单人独立合入 C2；
4. 离职、休假和紧急回滚路径已经验证。

G1 下，C2 必须由两名不同的人类维护者批准，其中至少一名是对应 Domain Owner。C0/C1 仍可由一名 RM 完成 adoption。
