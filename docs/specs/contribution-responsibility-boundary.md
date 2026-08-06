# Spec/RFC：ZeroWeb 贡献责任边界

**版本**：v0.1
**日期**：2026-08-07
**作者**：AI Assistant
**状态**：已确认

---

## 0. 执行摘要

- **一句话目标**：ZeroWeb 继续接受外部代码贡献，但每项合入变更必须由一名人类责任维护者明确接管，并按风险等级接受对应评审和验证。
- **本期范围**：定义角色、责任生命周期、变更风险等级、模块责任域、评审门禁、AI 辅助规则、单维护者过渡规则及后续落地文件。
- **明确排除**：本期不关闭公众 PR，不授予任何新成员合并权限，不建立委员会，不引入治理机器人，不修改 GitHub 仓库设置。
- **核心约束**：
  1. 补丁作者不自动成为代码所有者，评审者也不自动承担维护责任。
  2. AI 不能成为作者、评审者、责任维护者或批准主体。
  3. 高风险边界不能因当前只有一名维护者而伪造“双人批准”。
  4. `CODEOWNERS` 只负责评审路由和平台门禁，本文后续落地的治理政策才是责任定义的权威来源。
  5. 测试、基准和 Oracle 基线属于产品事实的一部分，不能按“只是测试文件”降低风险等级。
- **推荐方案**：采用“开放贡献 + 人类承责 + 四级风险 + 两档治理成熟度”的分级模型。
- **首个落地步骤**：确认本文后，先新增 `docs/governance/contribution-responsibility.md` 和 `.github/CODEOWNERS`，将当前仓库所有责任域暂时归口到 `@leizongmin`。

### 0.1 核心判断

Ladybird 在 2026-06 收窄的是代码引入权，而不是所有外部参与：bug report、reduction、网站测试、标准讨论、设计讨论、安全报告和技术反馈仍然开放。其明确理由是浏览器处理不可信输入，项目真正需要控制的是“代码进入浏览器后由谁负责”，而不是“谁生成了补丁”。

ZeroWeb 当前不适合照搬“仅维护者写代码”：

- 当前 90 天提交历史显示实际维护者只有一人，关闭公众 PR 只会损失有效输入，不能减少现有评审负担。
- 仓库已经公开承诺欢迎人工和 AI 辅助贡献，直接关闭代码渠道会与现行贡献政策冲突。
- ZeroWeb 已有 Owner 审批后才运行全量 CI 的门禁，缺少的是责任归属和风险分级，不是更强的入口封锁。

因此，本设计保留公开 PR，同时把“是否采用”和“采用后由谁负责”收回到维护者侧。

## 1. 背景与目标

### 1.1 背景

ZeroWeb 是一个 AI-first、实验性、跨平台浏览器项目。浏览器内核同时包含：

- 面向任意网页输入的解析、样式、布局、脚本和渲染路径；
- 同源策略、CORS、CSP、网络、存储、IPC、脚本沙箱和 WASM 沙箱等信任边界；
- 面向外部嵌入者的稳定 `ZeroWebView` API；
- Chromium Oracle、WPT、reftest、产品 smoke 和性能基准等质量控制面；
- 多平台构建、依赖下载、打包和发布供应链。

现有 `CONTRIBUTING.md` 已要求贡献可解释、可测试、可审查，PR 模板也要求披露 AI 使用、风险和验证。但仓库尚未定义：

- 谁可以代表项目接受某项变更；
- 谁承担合入后的回归处置、文档同步和后续维护；
- 哪些路径需要领域负责人或第二人批准；
- 当前只有一名维护者时，关键变更如何避免形式化自我批准；
- 责任人离开或按 §8.7 判定为 unavailable 时，责任如何回收。

### 1.2 目标

- **项目目标**：在开放协作下保持核心代码、安全边界、公开 API 和质量指标的演进可控。
- **维护者目标**：在合入前明确“谁接管”，避免 reviewer 被动承担作者未完成的维护工作。
- **贡献者目标**：在开始实现前知道变更等级、必要证据、评审人和可能的等待成本。
- **用户目标**：任何主线变更都能追溯到一名真实的人类责任维护者。

### 1.3 范围边界

**在范围内**：

- GitHub issue、proposal、PR、review 和 merge 的责任模型；
- 仓库路径和语义变更的风险分类；
- 公开贡献、AI 辅助贡献和维护者自提交变更；
- 安全、发布、依赖、CI、基线和公开 API 的额外门禁；
- 责任维护者的任命、接管、退出和回收。

**不在范围内**：

- 法律实体、基金会、董事会或财务治理；
- 行为准则的替代或扩写；
- 安全漏洞响应流程本身，继续由 `SECURITY.md` 定义；
- 具体 GitHub 权限名单和密钥管理细节；
- 贡献积分、报酬、雇佣或晋升制度。

## 2. 需求类型与设计依据

### 2.1 需求类型概览

| 类型 | 是否适用 | 来源 |
|------|---------|------|
| 业务需求 | 是 | `docs/architecture.md` 的“开源协作下保持核心代码与演进节奏可控” |
| 用户需求 | 是 | 贡献者、维护者、安全报告者和下游嵌入者 |
| 功能需求 | 是 | §3 |
| 非功能需求 | 是 | §4 |
| 接口需求 | 是 | §5 |
| 过渡需求 | 是 | 单维护者阶段不能直接执行双人批准 |

### 2.2 证据矩阵

| 关键结论 | 证据 1 | 证据 2 | 一致性 | 置信度 | 处理 |
|----------|--------|--------|--------|--------|------|
| 责任边界应围绕合入后的责任，而不是补丁来源 | Ladybird `CONTRIBUTING.md` 只允许维护者引入代码 | Ladybird 2026-06 治理公告及现有研究 §2.4/§5.3 | 一致 | 高 | 采用“责任维护者” |
| 外部参与不应等同于代码合入权 | Ladybird `GettingStartedContributing.md` | Ladybird `ISSUES.md` 的 reduction 流程 | 一致 | 高 | 保留非代码参与通道 |
| AI 输出不能替代人类理解和责任 | Ladybird `Documentation/CodePolicy.md` | ZeroWeb `CONTRIBUTING.md` 与 PR 模板 | 一致 | 高 | AI 不具备治理身份 |
| 高风险专域需要定向 owner，而非全仓平均分配 | Ladybird `.github/CODEOWNERS` 只覆盖 Crypto/TLS/JS Intl/Wasm/WebDriver 等专域 | ZeroWeb `docs/architecture.md` 明确信任边界和稳定 API | 一致 | 高 | 按责任域分配 owner |
| ZeroWeb 当前无法执行强制双人批准 | `git shortlog --since='90 days ago'` 只有一名人类作者 | CI 仅接受 `author_association == 'OWNER'` 的批准触发 | 一致 | 高 | 引入 G0/G1 两档成熟度 |
| 测试和基线本身会改变质量事实 | ZeroWeb Chromium Oracle 是诚实度量 | Ladybird 将 WPT 导入和测试政策视为主线质量资产 | 一致 | 高 | 基线变更升为关键风险 |

### 2.3 来源范围

**Ladybird 一手源码快照**：https://github.com/LadybirdBrowser/ladybird，提交 `a17243421d5706584b10ec26e014f910d57f92a2`，提交时间 2026-08-06。

**Ladybird 文件**：

- `CONTRIBUTING.md`
- `.github/CODEOWNERS`
- `Documentation/CodePolicy.md`
- `Documentation/GettingStartedContributing.md`
- `ISSUES.md`
- `SECURITY.md`

**ZeroWeb 文件**：

- `CONTRIBUTING.md`
- `SECURITY.md`
- `.github/pull_request_template.md`
- `.github/ISSUE_TEMPLATE/proposal.yml`
- `.github/workflows/ci.yml`
- `.github/workflows/security.yml`
- `Cargo.toml`
- `docs/architecture.md`
- `docs/research/research-ladybird-browser-2026-08-06.md`

> **来源说明**：上述事实均来自本地源码和仓库文档。本文的角色模型、四级风险模型、G0/G1 成熟度和模块映射均为作者综合设计，不是 Ladybird 原有制度。

## 3. 功能需求

### FR-001：责任域必须有明确归口

- **描述**：仓库中每个可合入路径必须映射到一个责任域、风险下限、主责任维护者和后备升级路径。
- **优先级**：必须

**验收场景**：

```text
场景: 查询普通源码路径
  假设 PR 修改 crates/layout-engine/src/lib.rs
  当贡献者查阅责任矩阵和 CODEOWNERS
  那么必须得到“渲染语义域、C1、当前 owner @leizongmin”
  验证: 对照 §8.4.3，并在 GitHub draft PR 中检查自动请求的 reviewer

场景: 新路径没有显式条目
  假设 PR 新增未被专用规则覆盖的顶层路径
  当执行责任路由
  那么必须回退到仓库默认 owner，不能成为无人负责路径
  验证: CODEOWNERS 首行存在 `* @leizongmin`
```

### FR-002：每项合入变更必须有人类责任维护者

- **描述**：PR 合入前必须记录一名具备合并权限的人类责任维护者（Responsible Maintainer，RM）。RM 接受变更进入主线，并承担回归分诊、修复或回滚协调、相关文档和测试闭环。
- **优先级**：必须

**验收场景**：

```text
场景: 外部贡献被项目采用
  假设外部贡献者提交一个满足质量门禁的 PR
  当维护者决定采用该变更
  那么 PR 必须记录 RM，且 RM 的 adoption 发生在最终提交版本
  验证: PR 模板存在 Responsible Maintainer 字段，合并记录包含最终版本的 adoption

场景: 没有人愿意接管
  假设 PR 技术上可行，但没有维护者愿意承担后续责任
  当 PR 进入合入判断
  那么项目必须关闭或暂缓该 PR，不能以“贡献者自行维护”替代 RM
  验证: 贡献政策明确“无 RM 不合入”
```

### FR-003：变更必须按最高风险语义执行门禁

- **描述**：每项变更必须按 §8.4.2 的 C0-C3 分类执行评审；同时命中多个等级时取最高等级，不能通过拆文件、改测试或改文档规避高风险门禁。
- **优先级**：必须

**验收场景**：

```text
场景: 单一低风险文档修正
  假设 PR 只修复非规范文档中的拼写
  当 RM 将其分类为 C0
  那么只需要一名 RM 批准和文档一致性检查
  验证: PR 记录 Risk Class: C0，changed files 不含规范或控制面文件

场景: 代码与基线混合变更
  假设 PR 同时修改布局代码和 Chromium Oracle 基线
  当执行风险分类
  那么整项 PR 必须按 C2 处理，或拆成可独立审查的 PR
  验证: PR 记录 Risk Class: C2，且满足 C2 门禁

场景: 治理规则自我降级
  假设 PR 修改责任模型并声称自身属于 C0
  当执行风险分类
  那么该 PR 必须按 C2 处理，不能由被修改后的宽松规则自我批准
  验证: PR 记录 Risk Class: C2，并按变更前规则完成 adoption
```

### FR-004：跨责任域变更必须先建立设计和责任链

- **描述**：跨两个及以上责任域、改变稳定 API、IPC 契约、信任边界或架构职责的变更，必须在实现前获得 proposal 接受，并明确主 RM 和受影响领域 owner。
- **优先级**：必须

**验收场景**：

```text
场景: 跨域架构变更
  假设 proposal 修改 protocol、renderer 和 webview
  当项目接受该 proposal
  那么必须指定一个主 RM，并记录三个责任域的评审要求和验证计划
  验证: proposal 的 Affected Domains、Risk Class、Responsible Maintainer 字段完整

场景: 未经设计直接提交
  假设 PR 改变 IPC 消息契约但没有已接受 proposal
  当 reviewer 识别到跨域契约变化
  那么 PR 必须转为 draft 或被阻止合入，先补齐 proposal
  验证: PR 评论或状态记录指向对应 proposal
```

### FR-005：AI 辅助不产生治理身份

- **描述**：AI 可以辅助分析、编码和测试，但不能计为作者批准、review、owner 或 RM。提交者必须理解最终 diff，RM 必须独立判断是否采用。
- **优先级**：必须

**验收场景**：

```text
场景: AI 辅助贡献
  假设提交者使用 AI 生成部分实现
  当提交 PR
  那么必须披露 AI 的作用、人工验证范围和来源/许可证边界
  验证: PR 模板的 AI Assistance 与 Human Verification 字段完整

场景: 提交者不能解释生成代码
  假设 reviewer 要求说明关键不变量或失败模式，提交者无法回答
  当 RM 评估是否接管
  那么 PR 必须暂缓或关闭，不能把理解成本转移给 reviewer
  验证: 贡献政策明确“不能解释则不合入”
```

### FR-006：责任必须能够转移和回收

- **描述**：领域 owner 辞任、失去维护权限或按 §8.7 判定为 unavailable 时，项目维护者必须将责任转交给合格继任者；没有继任者时由默认 owner 临时接管，并可冻结该领域的新功能变更。
- **优先级**：必须

**验收场景**：

```text
场景: 正常责任移交
  假设领域 owner 主动辞任且已有继任者
  当项目维护者批准移交
  那么治理文档与 CODEOWNERS 必须在同一 PR 更新
  验证: 单个 PR 同时修改两处映射且通过 owner review

场景: 领域无人接管
  假设领域 owner 离开且没有合格继任者
  当新的 C1/C2 功能 PR 到达
  那么默认 owner 只能选择临时接管、降级为修复范围或冻结，不能默认放行
  验证: PR 中记录明确处置，不存在空 owner
```

### FR-007：非代码贡献和安全报告必须保留独立通道

- **描述**：项目必须继续接受 bug report、最小复现、WPT/reftest reduction、网站测试、标准/设计讨论、文档反馈和安全报告；安全细节必须遵循 `SECURITY.md`，不能因贡献政策公开化而暴露。
- **优先级**：必须

**验收场景**：

```text
场景: 外部参与者只提供 reduction
  假设参与者没有提交代码
  当其提供可复现的最小 HTML/CSS/JS 用例
  那么项目必须允许该材料进入 issue 并被后续实现引用
  验证: CONTRIBUTING 保留非代码参与清单和 issue 链接

场景: 报告包含未公开漏洞细节
  假设参与者准备公开 issue
  当内容命中安全报告范围
  那么模板必须将其引导到 SECURITY.md 的私密通道
  验证: issue template 和 SECURITY.md 链接保持有效
```

## 4. 非功能需求

### NFR-001：可追溯性

- **描述**：每个合入 PR 必须能从平台记录中确定作者、RM、风险等级、批准人、验证结果和关联 proposal。
- **测量标准**：抽查最近 20 个合入 PR，字段完整率必须为 100%。
- **优先级**：必须

### NFR-002：责任覆盖

- **描述**：仓库内所有受版本控制的路径必须命中默认 owner；C1/C2 专域必须有比默认规则更具体的映射。
- **测量标准**：默认 CODEOWNERS 规则覆盖率 100%，§8.4.3 中 C1/C2 路径均存在显式条目。
- **优先级**：必须

### NFR-003：诚实门禁

- **描述**：G0 单维护者阶段不得把同一人的多次操作表述为双人批准；G1 生效前不得宣称已具备双人复核。
- **测量标准**：治理文档明确当前 maturity，分支保护与文档表述一致。
- **优先级**：必须

### NFR-004：低维护成本

- **描述**：M1 只复用 GitHub 原生 CODEOWNERS、PR 模板、issue 模板和分支保护，不新增常驻服务、bot 或第三方 SaaS。
- **测量标准**：M1 不新增运行时依赖和外部服务账号。
- **优先级**：必须

### NFR-005：规则一致性

- **描述**：治理政策、CODEOWNERS、CONTRIBUTING、PR 模板和 SECURITY 不得对同一风险等级给出冲突要求。
- **测量标准**：每次治理规则变更必须在同一 PR 完成受影响文件的同步更新。
- **优先级**：必须

## 5. 接口需求

### IF-001：PR 责任声明

- **类型**：GitHub PR 模板
- **规格**：
  - `Risk Class`：作者建议填写 C0/C1/C2/C3，最终由 RM 判定。
  - `Affected Domains`：列出 §8.4.3 中的责任域。
  - `Responsible Maintainer`：由维护者填写 GitHub handle；作者不能替维护者认领。
  - `Proposal / Security Process`：C2 填公开 proposal；C3 只在私密通道记录 advisory，公开模板填 `redacted/security process`，不得泄漏编号或细节。
  - `AI Assistance`：说明用途、人工复核范围、外部来源和许可证检查。
  - `Validation`：按责任域列出已执行和未执行的门禁。
  - `Adoption`：RM 在最终提交版本上确认“我接受该变更进入主线，并承担合入后的处置责任”；G0 维护者自提交时使用带最终 commit SHA 的记录，不能伪装成 GitHub approval。
- **错误处理**：必填项缺失时保持 draft 或阻止合入；不能由 bot 自动补全 RM。
- **默认动作**：未分类按变更命中的最高风险处理；未指定 RM 时不合入。

### IF-002：Proposal 责任声明

- **类型**：GitHub issue template
- **规格**：
  - 新增 `Affected Domains`、`Proposed Risk Class`、`Responsibility Plan`。
  - proposal 被接受不等于实现被接受；具体 PR 仍需 RM 最终 adoption。
  - C2 proposal 必须描述威胁模型、兼容性、回滚切点和验证证据。
- **错误处理**：跨域或 C2 变更未填写责任计划时，不进入实现阶段。
- **默认动作**：没有维护者 sponsor 的 proposal 可讨论，但不得标记为 accepted。

### IF-003：CODEOWNERS 路由

- **类型**：GitHub 系统集成
- **规格**：
  - 第一条必须是全仓 fallback owner。
  - 后续按 §8.4.3 从宽到窄排列，关键路径使用显式 owner。
  - CODEOWNERS 表示“必须请求谁评审”，不表示该 owner 独占提交权，也不表示 owner 已经接受 RM 身份。
- **错误处理**：治理矩阵和 CODEOWNERS 不一致时，以治理矩阵为责任语义，但必须阻止该治理变更合入并同步修正 CODEOWNERS。
- **默认动作**：当前全部 owner 暂时为 `@leizongmin`。

### IF-004：分支保护

- **类型**：GitHub 仓库设置
- **规格**：
  - M2 完成后，`main` 禁止直接 push，维护者紧急回滚除外。
  - 必须通过现有 required status checks。
  - 必须解决 review conversation。
  - G0 外部 PR 必须由 CODEOWNER review；维护者自提交 PR 使用最终 SHA adoption 记录和 C2 证据包。
  - G1 必须要求 CODEOWNERS review。
  - G1 激活后，C2 变更要求两名不同的人类维护者批准，其中至少一名为对应领域 owner。
- **错误处理**：若平台无法按 C2 动态设置批准人数，G1 前继续使用 G0；G1 后通过拆分保护分支或仓内 check 实现，不能仅依赖人工记忆。
- **默认动作**：G0 只要求一名 owner/RM，不宣称双人批准。

### IF-005：安全贡献通道

- **类型**：GitHub Security Advisory / `SECURITY.md`
- **规格**：C3 变更的漏洞细节、利用样例、临时分支和参与人名单只进入私密通道；公开 PR 只能在披露安全后出现。
- **错误处理**：发现公开 PR 泄漏未修复漏洞时，先限制公开信息并转入安全流程，不能继续普通 review。
- **默认动作**：是否公开由 `SECURITY.md` 的披露流程决定。

## 6. 约束、决策与假设

### 6.1 必须约束（Must）

- 每个合入 PR 必须有一名人类 RM。
- RM 必须具备仓库合并权限，并对最终提交版本留下 adoption 记录。
- 每个路径必须有 fallback owner。
- 风险等级必须取路径等级和语义等级中的较高者。
- C2/C3 变更必须存在设计或安全上下文及明确回滚方案。
- 治理规则变更本身按 C2 处理。

### 6.2 禁止约束（Must Not）

- 不得把 AI、bot、组织账号或无合并权限的贡献者列为 RM。
- 不得把 CODEOWNERS 自动请求 reviewer 等同于 owner 已批准或已接管。
- 不得通过只改测试期望、Oracle 图片、baseline、豁免列表或文档指标来规避实现风险等级。
- 不得让 PR 作者自行声明“长期由我维护”来替代项目侧 RM。
- 不得在公开治理文件中记录漏洞 embargo 细节、密钥、签名身份或私密联系方式。
- 不得因贡献者身份、是否使用 AI 或提交量自动降低门禁。

### 6.3 已定决策

- 采用分级开放模型，不采用 Ladybird 当前的“仅维护者引入代码”模型。
- 责任归属以人类 RM 为核心，领域 owner 负责专域判断，项目维护者负责最终升级。
- 采用 C0-C3 四级风险模型。
- 采用 G0/G1 两档治理成熟度，不设置虚假的单人“双重批准”。
- M1 不引入治理 bot。

### 6.4 技术约束

- GitHub CODEOWNERS 只能按路径匹配，无法识别“只改注释”“改变基线语义”等内容差异，最终风险等级必须由人判断。
- 当前仓库只有一名实际人类维护者，G1 双人批准暂不可用。
- 当前 CI 由 Owner 的 approved review 触发；落地时不能破坏该成本控制机制。
- 测试和构建必须继续遵守 `make test` / `make reftest` 的 test-guard 约束。

### 6.5 假设

| 假设 | 状态 | 依据 |
|------|------|------|
| GitHub 仓库 owner 为 `@leizongmin` | 已验证 | `origin` 为 `github.com/leizongmin/ZeroWeb.git` |
| 当前治理处于 G0 | 已验证 | 90 天提交历史只有一名人类作者，CI 只接受 OWNER 批准 |
| 项目继续接受公众 PR | 已验证 | 现有 `CONTRIBUTING.md` 和 PR 模板 |
| GitHub 分支保护的当前实际配置 | 待验证但不阻塞设计 | 仓内文件无法读取托管平台设置；M1 实施时核对 |

### 6.5A 实现来源

| 能力 | 来源类型 | 具体来源 | 边界 |
|------|----------|----------|------|
| 路径 owner 路由 | GitHub 原生能力 | `.github/CODEOWNERS` | 只做路由和批准门禁 |
| 责任与风险声明 | 仓内文档 | PR / proposal template | 人工填写，M1 不自动判定 |
| 质量门禁 | 复用现有 CI | `.github/workflows/*.yml`、Makefile | 不新增测试框架 |
| 安全报告 | GitHub 原生能力 | Security Advisory + `SECURITY.md` | 不在公开 PR 存漏洞细节 |
| 责任权威定义 | 仓内文档 | `docs/governance/contribution-responsibility.md` | 实施后成为单一权威来源 |

### 6.6 代码变更边界

**本文确认后的 M1 允许修改**：

- `docs/governance/contribution-responsibility.md`
- `.github/CODEOWNERS`
- `CONTRIBUTING.md`
- `.github/pull_request_template.md`
- `.github/ISSUE_TEMPLATE/proposal.yml`
- `SECURITY.md`
- `README.md`

**M1 禁止修改**：

- `crates/**`、`apps/**`、`tests/**`、`tools/**`：治理文档落地不需要业务代码变化。
- `.github/workflows/**`：先用现有 Owner 审批机制，不在首批变更引入自动化。
- `AGENTS.md`：这是 Agent 行为规则，不是项目贡献治理的权威来源。

## 7. 优先级与实施交接

### 7.1 里程碑

| 里程碑 | 进入条件 | 范围 | 完成条件 |
|--------|----------|------|----------|
| M1：责任可见 | 本文获确认 | 治理政策、CODEOWNERS、贡献/PR/proposal/安全文档 | 所有路径有 owner，PR 能记录 RM 和风险等级 |
| M2：G0 门禁对齐 | M1 完成 | 核对并调整 GitHub `main` 分支保护 | 平台设置与 G0 文档一致 |
| M3：G1 双人批准 | 至少两名已按 §8.7 任命且当前可用的人类维护者 | C2 双人批准和失效回退机制 | 代表性 C2 PR 无法由一人独立合入 |

### 7.2 文件/模块清单

| 路径 | 动作 | 目的 | 风险 |
|------|------|------|------|
| `docs/governance/contribution-responsibility.md` | 新增 | 发布规范性政策和责任矩阵 | 必须与本文设计一致 |
| `.github/CODEOWNERS` | 新增 | 路由 owner review | 当前单 owner，不能误称职责已分散 |
| `CONTRIBUTING.md` | 修改 | 告知贡献者分级流程 | 保留开放贡献和非代码通道 |
| `.github/pull_request_template.md` | 修改 | 收集风险、责任和 adoption 证据 | 字段不能重复现有验证清单 |
| `.github/ISSUE_TEMPLATE/proposal.yml` | 修改 | 架构变更前置责任计划 | 不让 proposal acceptance 替代 PR adoption |
| `SECURITY.md` | 修改 | 说明 C3 与私密通道关系 | 不扩写未验证 SLA |
| `README.md` | 修改 | 增加治理文档入口 | 只做导航 |

### 7.3 推荐修改顺序

1. 新增规范性治理政策，固定术语、风险等级和责任矩阵。
2. 新增 CODEOWNERS，将全部当前责任显式归口到 `@leizongmin`。
3. 更新 CONTRIBUTING、PR 和 proposal 模板，把责任声明接入现有流程。
4. 更新 SECURITY 和 README 的导航与边界说明。
5. 在 GitHub 上创建代表性 draft PR，验证 owner 路由和 G0 分支保护。

### 7.4 首批提交建议

| 批次 | 范围 | 预期结果 | 验证 |
|------|------|----------|------|
| Batch 1 | 治理政策 + CODEOWNERS | 建立责任权威来源与全路径 fallback | `rg` 检查责任域，GitHub draft PR 检查 reviewer |
| Batch 2 | CONTRIBUTING + PR/proposal template | 贡献流程能记录风险与 RM | 使用 C0/C1/C2 三个虚拟变更走表单 |
| Batch 3 | SECURITY + README + 平台设置 | 收口安全边界和文档入口 | 链接检查 + 分支保护人工核对 |

## 8. 技术设计（RFC）

### 8.1 现状分析

现有治理具备三个基础：

1. `CONTRIBUTING.md` 已定义最小修改、测试、文档、依赖许可证和 AI 披露要求。
2. PR/proposal 模板已收集影响范围、验证和风险。
3. CI、安全扫描、reftest 和 benchmark 已在 Owner 批准后运行。

当前缺口是“质量要求有了，但责任主体没有建模”：

- 没有 CODEOWNERS，平台无法按专域请求 reviewer。
- PR 没有 RM 字段，批准不等于明确接管。
- 高风险路径与普通文档使用同一套贡献说明。
- 测试基线、依赖、稳定 API、IPC 和信任边界没有额外批准语义。
- 单维护者现状与浏览器项目理想的双人复核之间没有过渡制度。

### 8.2 目标状态

```text
外部输入 / 维护者任务
          |
          v
  issue / proposal / advisory
          |
          v
   风险分类 + 责任域路由
          |
          v
   人类 RM 明确 sponsor
          |
          v
 实现 -> 专域 review -> 自动门禁
          |
          v
 RM 对最终版本作 adoption 决定
          |
          v
 merge -> RM 负责回归分诊/修复/回滚协调
```

原则上，贡献者证明“改动可行”，领域 owner 判断“专域上正确”，RM 决定“项目愿意长期接纳”。三者可以是不同的人；在 G0 中只能由同一名维护者兼任时，必须如实记录。

### 8.3 角色模型

| 角色 | 可以做什么 | 不能自动获得什么 |
|------|------------|------------------|
| Reporter | 提交 bug、安全报告、reduction、网站测试和技术反馈 | 代码合入权 |
| Contributor | 提交 proposal、代码、测试和文档 | owner、RM 或合并权限 |
| Reviewer | 对指定版本给出技术 review | 合入后责任 |
| Domain Owner | 对责任域的不变量、风险和验证作权威判断 | 独占修改权；自动成为 RM |
| Responsible Maintainer（RM） | sponsor 变更、作最终 adoption、协调合入后处置 | 免除专域 review 或质量门禁 |
| Project Maintainer | 任命 owner/RM、解决跨域冲突、回收无人责任域 | 绕过 C3 安全流程 |
| Security/Release Owner | 处理安全 embargo、供应链和发布边界 | 普通业务域的独占决策权 |
| AI / Bot | 辅助生成、检查、提醒和执行确定性任务 | 作者、Reviewer、Owner、RM 或批准票 |

**RM 的责任起点**：在 PR 上明确 sponsor 时开始。

**RM 的责任终点**：变更被拒绝/关闭、被完整回滚，或责任通过治理记录转交给另一名维护者。代码长期存在不要求原 RM 永久独占维护，但在未转交前必须负责首轮回归分诊和处置协调。

### 8.4 风险与责任域

#### 8.4.1 分类规则

风险等级由两个维度共同决定：

```text
最终风险 = max(路径风险下限, 语义风险)
```

- **路径风险下限**：代码所在责任域的最低等级。
- **语义风险**：该变更实际改变的信任、兼容性、供应链或治理行为。
- 如果 reviewer 无法确定等级，按较高一级处理，直到 RM 给出可审计的降级理由。

#### 8.4.2 四级风险

| 等级 | 定义 | 典型变更 | 合入门禁 |
|------|------|----------|----------|
| C0 开放 | 不改变规范行为、产品承诺、指标或执行路径 | 非规范文档的拼写、格式或注释修正 | 1 名 RM；目标检查 |
| C1 受管核心 | 改变普通功能、渲染语义、产品行为或规范文档 | DOM/CSS/布局/绘制实现、产品功能、测试框架、Spec | RM + domain owner；相关测试；跨域时需要 proposal |
| C2 关键边界 | 改变信任边界、稳定 API、IPC、供应链、发布、治理或质量事实 | security/net/storage/protocol/sandbox、webview API、依赖、CI、Oracle baseline | accepted proposal；G0 证据包或 G1 双人批准；完整相关门禁；回滚方案 |
| C3 受限 | 未披露漏洞、密钥、签名、发布凭据或仓库权限控制 | embargo 修复、凭据轮换、分支保护管理员操作 | 私密通道；授权维护者；披露后才公开 |

以下语义变更无论路径都至少是 C2：

- 新增、删除或升级第三方依赖，修改 `Cargo.lock` 的解析结果；
- 修改安全默认值、跨源/权限判断、沙箱限制、IPC 消息和序列化兼容性；
- 修改 `ZeroWebView` 公开 API 的兼容性承诺；
- 修改 required checks、发布产物、下载源、签名或打包流程；
- 修改 Oracle 图片、通过率分母、阈值、豁免/跳过列表、基准基线或对外宣称的质量数字；
- 修改本文责任模型、CODEOWNERS 语义或分支保护规则。

#### 8.4.3 ZeroWeb 责任域矩阵

| 责任域 | 路径/语义 | 风险下限 | 专域不变量 |
|--------|-----------|----------|------------|
| Governance & Supply Chain | `.github/CODEOWNERS`、`.github/workflows/**`、`.github/dependabot.yml`、根 `Cargo.toml`/`Cargo.lock`/`deny.toml`/`Makefile`、`LICENSE`、`AGENTS.md`、`CONTRIBUTING.md`、`SECURITY.md`、发布/下载/打包脚本 | C2 | 门禁不可被贡献者自行绕过；来源和许可证可追溯 |
| Web Semantics | `crates/dom`、`css-parser`、`style-system`、`layout-engine`、`engine`、`canvas` | C1 | 规范行为有测试；不跨越既有 crate 职责 |
| Patched Dependency | `crates/taffy-local`、`[patch.crates-io]` | C2 | 本地补丁有上游来源、差异说明和兼容验证 |
| Rendering & Platform | `crates/render-foundation`、`host-runtime` | C1；unsafe/FFI/GPU 隔离语义为 C2 | 平台差异显式；CPU/GPU 和资源生命周期可验证 |
| Runtime Trust Boundary | `crates/net`、`security`、`storage`、`protocol`、`script-sandbox`、`wasm-sandbox`、`page-runtime`、`apps/renderer` | C2 | 不可信输入、权限、隔离、超时和资源限制不被削弱 |
| Stable Embedding API | `crates/webview` | C2 | 外部 API、错误语义、feature 和宿主边界保持兼容 |
| Product | `crates/browser-shell`、`apps/browser`、`apps/webview-demo`、产品资产 | C1 | 产品层优先通过 webview；不绕过安全/运行时边界 |
| Quality Control | `tests/integration`、`tests/wpt-runner`、benchmark、test-guard、测试脚本 | C1；基线/阈值/豁免/指标为 C2 | 失败不能通过改期望被隐藏；诚实度量口径稳定 |
| Normative Docs | `docs/specs`、`docs/architecture.md`、`docs/goal/**`、`ROADMAP.md`、`CHANGELOG.md`、`CODE_OF_CONDUCT.md`、贡献模板、治理政策 | C1；治理、安全路由和指标控制面为 C2 | 文档与实现一致；规格变化有实施或明确状态 |
| Informational Docs & Tools | `docs/research`、一般 README、`tools/icon-gen` | C0；可执行逻辑或产品资产变化为 C1 | 不冒充规范；生成产物可复现 |

初始 owner 全部为 `@leizongmin`。矩阵仍然有价值，因为它先固定责任域和风险语义；未来新增 owner 时只替换对应域，不需要重写制度。

### 8.5 PR 生命周期

```text
Unclassified
    |
    v
Classified ----> Rejected / Needs Proposal
    |
    v
Sponsored by RM
    |
    v
Implementation Ready
    |
    v
Domain Review + Automated Gates
    |
    v
RM Final Adoption
    |
    v
Merged ----> Regression ----> Fix / Revert / Responsibility Transfer
```

1. **分类**：作者建议风险等级；RM 作最终判断。
2. **Sponsor**：维护者愿意评估并可能接管，不代表承诺合入。
3. **实现完成**：作者提供代码、测试、文档、来源和未覆盖风险。
4. **专域 review**：owner 审查领域不变量；跨域变更逐域检查。
5. **自动门禁**：执行现有 fmt、clippy、test、安全、reftest 或 benchmark。
6. **最终 adoption**：RM 只对最终 commit SHA 作接纳决定；外部 PR 使用 approval，G0 维护者自提交使用带 SHA 的 adoption 记录；后续 push 使旧记录失效。
7. **合入后处置**：回归首先由 RM 分诊；高严重度问题优先回滚，再决定重做。

### 8.6 G0/G1 治理成熟度

#### G0：单维护者期（当前）

- `@leizongmin` 可同时作为 project maintainer、domain owner 和 RM。
- 外部 PR 由 `@leizongmin` 提交 GitHub approval；维护者自提交 PR 无法自我 approval，必须在最终 commit SHA 上留下 adoption 记录。
- C2 不能宣称“双人批准”，必须附一份证据包：
  - 设计/proposal 或安全上下文；
  - 最终 diff 的独立二次审阅记录；
  - 相关全量测试、安全检查、兼容性或性能结果；
  - 明确的回滚触发条件和回滚步骤；
  - AI 参与时，人工复核关键不变量和来源边界。
- 二次审阅必须针对完成后的最终 diff，不能用实现过程中的自检代替。
- 紧急安全修复或回滚可以先处置，但必须在后续公开记录中补齐不敏感的决策依据。

#### G1：多维护者期

G1 只在以下条件全部满足后，通过一个 C2 治理 PR 显式激活：

1. 至少两名人类拥有合并权限并接受维护者责任；
2. C2 责任域至少有一名主 owner 和一名 fallback；
3. 分支保护或仓内 check 能阻止单人独立合入 C2；
4. 离职、休假和紧急回滚的升级路径已验证。

G1 下：

- C0/C1 可由一名 RM 完成 adoption；C1 仍需相应 domain owner review，二者可为同一人。
- C2 必须有两名不同的人类维护者批准，其中至少一名是 domain owner，另一名承担 RM 或 Security/Release Owner。
- PR 作者不能同时提供两张批准票。
- C3 参与人遵循最小知情原则，人数由安全事件决定，不公开固定名单。

### 8.7 Owner 任命与责任回收

**任命条件**：

- 已通过代码、review、设计或事故处置证明对该领域不变量的理解；
- 能解释该领域的验证方式、常见回归和回滚路径；
- 明确接受响应和移交责任；
- 由 project maintainer 通过 C2 治理 PR 任命。

提交数量、活跃天数或 AI 产出规模都不能自动换取 owner 身份。

**失效与回收**：

- owner 可主动辞任，不需要证明理由；
- 权限被撤销时，owner 身份同时失效；
- owner 自行声明不可用、失去权限，或对两次间隔至少 7 个自然日的责任请求连续 30 个自然日无响应时，project maintainer 可将其标记为 unavailable 并启用 fallback；该规则用于责任路由，不构成对 issue/PR 响应时间的 SLA；
- 没有 fallback 时，默认 owner 临时接管，只接受安全修复、回归修复和必要维护；新功能可被冻结；
- 治理矩阵和 CODEOWNERS 必须在同一变更中更新。

### 8.8 安全考虑

- RM 机制不能代替最小权限、沙箱、输入校验或自动安全扫描。
- CODEOWNERS 是路由机制，不是安全边界；仓库管理员仍需保护分支和凭据。
- C3 信息不得出现在公开 proposal、PR 模板、commit message 或 CODEOWNERS。
- 外部贡献者可以参与私密漏洞处置，但只获得完成任务所需的最小信息。
- 依赖升级不能仅凭 Dependabot 通过；必须由 RM 接管并检查许可证、公告、feature 和平台影响。
- 质量基线变更按 C2 处理，防止“改测试让它通过”掩盖真实回归。

### 8.9 方案对比

| 方案 | 贡献入口 | 责任控制 | 当前可行性 | 长期风险 | 决定 |
|------|----------|----------|------------|----------|------|
| A. 维持现状 | 开放 PR | 依赖 reviewer 默契 | 高 | 责任漂移、关键路径同门禁 | 拒绝 |
| B. 分级开放 + RM | 开放 PR 和非代码参与 | 人类接管 + 风险分级 | 高 | 需要维护政策一致性 | **选定** |
| C. Ladybird 式维护者闭环 | 外部不提交代码 | 仅维护者引入 | 低 | 单维护者项目失去输入且产能收缩 | 暂不采用 |
| D. 全自动风险治理 | 开放 PR | bot 分类和动态门禁 | 中 | 误分类、维护成本、过度设计 | M1 拒绝 |

选择 B 的原因：

1. 它直接解决“合入后由谁负责”，而不是用身份替代质量判断。
2. 它与 ZeroWeb 当前开放贡献和 AI-first 定位一致。
3. 它允许当前 G0 如实运行，也为未来 G1 留出确定升级路径。
4. 它复用现有 GitHub 和 CI 能力，首期改动范围小。

### 8.10 测试策略

**静态检查**：

- 检查治理政策、CONTRIBUTING、PR/proposal 模板使用同一组术语。
- 检查 CODEOWNERS 存在全仓 fallback 和 §8.4.3 的显式路径。
- 检查 README、SECURITY 和贡献文档链接有效。

**场景检查**：

- C0：非规范文档拼写 PR 能正确路由并由 RM 接纳。
- C1：布局实现 PR 要求 Web Semantics owner 和相关测试。
- C2：IPC + renderer PR 要求 proposal、证据包和回滚方案。
- C2：只修改 Oracle baseline 的 PR 不能被分类为“测试变更 C0”。
- C3：安全报告不会被公开 issue 模板收集。
- 无 RM：即使 CI 全绿也不能合入。

**平台检查**：

- 使用 draft PR 验证 CODEOWNERS 自动请求。
- 核对 `main` 分支保护、required checks、conversation resolution 和 stale review 行为。
- G1 激活前后各验证一次 C2 的批准人数。

### 8.11 回滚计划

- M1 全部是文档和路由变更，可以通过单个 revert 恢复现状。
- 如果 CODEOWNERS 导致 PR 无法推进，先恢复全仓 fallback，不删除治理政策；修正规则后再细分。
- 如果 PR 模板字段造成明显重复，保留 `Risk Class`、`Responsible Maintainer` 和 `Adoption` 三个核心字段，删除非必要字段。
- 如果 G1 动态门禁实现不可靠，立即退回 G0 并在政策中如实标记，不能静默绕过。
- 回滚治理门禁本身按 C2 处理；紧急恢复仓库可用性时允许先回滚后补记录。

## 9. Spec Lint 报告

### 9.1 结构完整性

| 规则 | 裁决 | 依据 |
|------|------|------|
| 执行摘要存在性 | ✅ Pass | §0 包含目标、范围、约束、方案和首步 |
| 场景存在性 | ✅ Pass | FR-001 至 FR-007 均有验收场景 |
| 异常路径覆盖 | ✅ Pass | 每个 FR 至少包含一个异常场景，数量不少于正常场景 |
| 测试绑定 | ✅ Pass | 每个场景均包含 `验证` 行 |
| UI 对齐 | ⏭️ Skip | 本设计不包含 UI |
| TBD 清零 | ✅ Pass | §10 没有阻塞级 TBD |
| 约束覆盖 | ✅ Pass | §6.1 分别由 FR-001 至 FR-004、IF-001 和治理自我降级场景覆盖 |
| 实施交接完备 | ✅ Pass | §7 包含文件、职责、顺序、批次和验证 |
| 首步可执行性 | ✅ Pass | §0 和 §7.3 首步均为新增规范性治理政策 |

### 9.2 语言精确性

| 规则 | 裁决 | 依据 |
|------|------|------|
| 模糊动词 | ✅ Pass | FR/NFR 使用“记录、阻止、回退、转交”等可观察行为 |
| 无量化描述 | ✅ Pass | NFR 使用 100%、20 个 PR；unavailable 使用 7/30 自然日 |
| 非确定性措辞 | ✅ Pass | 规范性要求使用“必须/不得”；可选处置明确列出决策主体 |

### 9.3 一致性

| 规则 | 裁决 | 依据 |
|------|------|------|
| 范围冲突 | ✅ Pass | §1.3 治理范围与法律/财务/报酬排除项无交集 |
| 约束冲突 | ✅ Pass | G0 记录 adoption、G1 强制不同人批准，未伪造双人 review |
| 方案漂移 | ✅ Pass | §7 M1 只修改 §6.6 允许路径，不引入 bot 或业务代码 |
| CLI 语义一致 | ⏭️ Skip | 本设计不包含 CLI |
| 默认动作闭合 | ✅ Pass | IF-001 至 IF-005 均定义默认动作或阶段条件 |
| 章节引用正确 | ✅ Pass | §3、§5、§6、§7 对 §8.4/§8.7 的引用均指向对应定义 |
| 外部事实保守化 | ✅ Pass | GitHub 分支保护现状在 §6.5 标为待验证，未写成已生效事实 |
| 未验证细节泄漏 | ✅ Pass | M2 才核对和调整分支保护，M1 不依赖其当前状态 |
| 场景预期泄漏 | ✅ Pass | 验收场景不把未验证的平台设置写成现状断言 |
| 实现来源闭合 | ✅ Pass | §6.5A 指明 GitHub 原生能力、现有 CI 和仓内文档 |
| 来源-测试联动 | ✅ Pass | §8.10 对 CODEOWNERS、模板、安全通道和平台设置分别给出验证 |
| 脆弱选择逻辑覆盖 | ⏭️ Skip | 本设计不选择外部资产或响应结构 |
| 类型分层清晰 | ✅ Pass | Requirement 在 §3-§5，Decision/Assumption 在 §6，设计在 §8 |
| 优先级完备 | ✅ Pass | FR-001 至 FR-007、NFR-001 至 NFR-005 均标注优先级 |
| 代码边界完备 | ✅ Pass | §6.6 同时定义允许和禁止修改路径 |
| 清单数量一致 | ✅ Pass | 四级风险、两档成熟度、七个 FR 和五个 NFR 与实际列举一致 |
| 依赖清单一致 | ✅ Pass | M1 明确不新增运行时依赖或外部服务 |
| 重复失控 | ✅ Pass | §3 定义行为，§5 定义接口，§8 定义实现和取舍 |

**汇总**：27 Pass / 0 Warning / 0 Fail / 3 Skip

**门禁判定**：允许用户确认；GitHub 实际分支保护配置留到 M2 验证，不阻塞本设计。

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|----|------|--------|----------|--------|
| TBD-1 | 当前 `main` 分支保护配置 | 重要 | 仓内无法读取 GitHub 托管设置 | M2 开始时核对 required checks、review 和 bypass |
| TBD-2 | G1 维护者名单 | 可选 | 当前没有第二名已任命维护者 | 满足 §8.7 后通过 C2 治理 PR 任命 |
| TBD-3 | G1 的 C2 动态门禁实现 | 重要 | GitHub 原生规则能否按风险标签动态要求两人尚未验证 | G1 激活前做最小平台实验，再决定原生规则或仓内 check |

## 11. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1 | 2026-08-07 | 初始设计：开放贡献、人类 RM、C0-C3、G0/G1 和十个责任域 |
