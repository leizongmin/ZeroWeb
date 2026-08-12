# 参与 ZeroWeb 开发

感谢你愿意为 ZeroWeb 贡献代码、文档、测试或设计讨论。

这里欢迎人工编写，也欢迎 AI 辅助编写。标准只有一个：改动要说得清、测得到、看得懂。

这个仓库现在还是实验项目，主要拿来学习、研究和做工程探索。不要默认把它当成能直接进生产的浏览器。如果你的改动就是冲着商用或生产场景去的，请在 issue、proposal 或 PR 里把风险、验证边界和没覆盖到的地方写清楚。

## 开始前

先看这几份文档：

- [README.md](README.md)
- [docs/architecture.md](docs/architecture.md)
- [docs/specs/zero-web-spec-rfc.md](docs/specs/zero-web-spec-rfc.md)
- [docs/governance/contribution-responsibility.md](docs/governance/contribution-responsibility.md)
- 你计划修改的 crate 下对应 `README.md`

如果变更跨两个及以上责任域，或涉及安全边界、稳定 API、IPC、依赖、CI、发布、治理或质量基线，别直接开写。先开 proposal，把范围、责任、验证和回滚方案说清楚。

仓库里已经准备了几种标准入口：

- bug、feature、proposal、question 请优先使用 `.github/ISSUE_TEMPLATE/`
- 提交 PR 时请按 `.github/pull_request_template.md` 填写验证、文档和许可证影响
- 安全问题不要开公开 issue，按 [SECURITY.md](SECURITY.md) 处理

## 责任接纳与风险等级

补丁作者负责解释和验证改动，但代码能否进入主线，由项目维护者决定。每项合入变更必须有一名人类责任维护者（Responsible Maintainer，RM）明确接管；没有 RM 的 PR 不会合入。

提交 PR 时先建议风险等级，最终分类由 RM 确认：

- **C0 开放**：非规范文档的拼写、格式或注释修正。
- **C1 受管核心**：普通功能、渲染语义、产品行为、测试框架或规范文档。
- **C2 关键边界**：安全/网络/存储/协议/沙箱、稳定 API、依赖、CI、发布、治理、Oracle/baseline/阈值等质量事实。
- **C3 受限**：未披露漏洞、密钥、签名、发布凭据或仓库权限，只能走私密安全通道。

当前项目处于 G0 单维护者阶段。C2 使用一名 RM 加完整证据包，不会把同一人的多次操作描述为双人 review。完整角色、责任域、门禁和 owner 回收规则见 [贡献责任边界](docs/governance/contribution-responsibility.md)。

## 开发环境

### 必备工具

- Rust `1.85+`
- `cargo fmt`
- `cargo clippy`

### Linux 依赖

CI 在 Linux 上装的是这几项，本地最好也保持一致：

```bash
sudo apt-get update
sudo apt-get install -y \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libfontconfig1-dev \
  libwayland-dev \
  libx11-dev \
  libxrandr-dev \
  libxi-dev \
  libgl1-mesa-dev \
  mesa-vulkan-drivers   # wgpu Vulkan 后端（GPU 渲染 / GPU 测试）必需；缺省时回退 GL/llvmpipe 软件渲染
```

### 常用命令

```bash
cargo build --workspace
make test
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
./scripts/check-coverage.sh
./scripts/run-benchmarks.sh
```

## 提交流程

1. 选择一个明确的问题或里程碑切片，不要同时推进过多方向。
2. 建议风险等级并列出受影响责任域；C2 或跨域变更先关联已接受的 proposal。
3. 在改动前说明你的目标、边界和假设。
4. 保持修改最小化，避免顺手重构无关代码。
5. 为行为变化补测试；为 API 变化补文档。
6. 在提交前运行必要检查，并等待 RM 对最终版本作 adoption。

## 写代码时的基本要求

- 保持改动聚焦。每一行修改都应该能追溯到当前任务。
- 优先简单方案，不做推测性抽象。
- 公共 API 必须带 `///` 文档注释。
- 使用 `tracing`，不要新增 `println!` 作为正式日志方案。
- 如果修改热路径或性能敏感逻辑，补基准测试或说明为什么不需要。
- 如果修改跨 crate 协议、数据结构或行为契约，同步更新相关文档。

## 提 PR 前至少跑这些

默认要求：

```bash
cargo fmt --all -- --check
make test
cargo clippy --workspace --all-targets -- -D warnings
```

下面这些情况，建议再多跑一步：

- 渲染、布局、解析器、存储等核心路径改动：`./scripts/run-benchmarks.sh`
- 大范围测试改动或覆盖率工作：`./scripts/check-coverage.sh`
- 只改文档：无需跑完整测试，但请确保文档内容与仓库现状一致

## AI 辅助贡献

AI 辅助是这个项目的重要工作方式，但也有几条额外要求：

- 你需要对提交内容负责，不能提交自己也无法解释的生成代码。
- 保持 diff 可审查，避免一次性引入大块未经拆分的生成结果。
- 在 PR 描述中简要说明 AI 参与方式即可，不需要贴完整对话。
- 说明你人工复核了哪些关键不变量、失败路径和外部来源。
- 生成代码必须补足测试、错误处理和文档，不能把这些留给 reviewer 收尾。
- 如果 AI 基于外部代码或文档生成内容，确保来源和许可证边界清晰。
- AI 和 bot 不能成为 reviewer、owner、RM 或批准票。

## 依赖和许可证

ZeroWeb 会优先接受许可证边界清楚的依赖，比如：

- `MIT`
- `Apache-2.0`
- `BSD`
- `ISC`
- `Zlib`

想加第三方依赖，先想清楚这几件事：

- 是否真的有必要引入
- 是否会进入主线核心路径
- 许可证是否与项目策略兼容
- 是否存在更简单的自实现方案

依赖增删、升级或 `Cargo.lock` 解析结果变化至少属于 C2，必须先有 proposal、RM 和回滚方案。

默认不要把 `GPL`、`AGPL`、`LGPL`、`MPL` 依赖拉进核心浏览器路径。真有必要，先讨论，不要直接提。

## PR 自查

- 改动范围清晰，标题和描述准确
- 风险等级、责任域、proposal 和 RM 状态已填写
- 测试和文档已同步更新
- 没有混入无关格式化或顺手重构
- 许可证和来源边界清晰
- reviewer 可以独立理解和验证你的改动

## 如果你想找点事做

还没挑好题目的话，可以先看这几类：

- WPT 和集成测试补全
- `browser-shell` 产品层实现
- `script-sandbox` 的可用形态设计与实现
- 渲染、布局、样式系统中的兼容性缺口
- 文档、基准和开发者体验改进
