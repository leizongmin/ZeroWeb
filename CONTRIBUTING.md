# Contributing To ZeroBrowser

感谢你愿意为 ZeroBrowser 贡献代码、文档、测试或设计讨论。

项目欢迎人工编写和 AI 辅助编写的贡献，但标准一致：改动必须可解释、可验证、可审查。

## Before You Start

建议先阅读这些文档：

- [README.md]($HOME/work/ZeroBrowser/README.md)
- [docs/architecture.md]($HOME/work/ZeroBrowser/docs/architecture.md)
- [docs/specs/zero-browser-spec-rfc.md]($HOME/work/ZeroBrowser/docs/specs/zero-browser-spec-rfc.md)
- 你计划修改的 crate 下对应 `README.md`

如果是跨 crate 的大改动，先开 issue 或 draft PR 说明范围、目标和取舍，再开始写代码。

## Development Setup

### Required Tooling

- Rust `1.85+`
- `cargo fmt`
- `cargo clippy`

### Linux System Dependencies

CI 在 Linux 上使用以下依赖；本地开发也建议保持一致：

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
  libgl1-mesa-dev
```

### Common Commands

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
./scripts/check-coverage.sh
./scripts/run-benchmarks.sh
```

## Contribution Workflow

1. 选择一个明确的问题或里程碑切片，不要同时推进过多方向。
2. 在改动前说明你的目标、边界和假设。
3. 保持修改最小化，避免顺手重构无关代码。
4. 为行为变化补测试；为 API 变化补文档。
5. 在提交前运行必要检查。

## Engineering Expectations

- 保持改动聚焦。每一行修改都应该能追溯到当前任务。
- 优先简单方案，不做推测性抽象。
- 公共 API 必须带 `///` 文档注释。
- 使用 `tracing`，不要新增 `println!` 作为正式日志方案。
- 如果修改热路径或性能敏感逻辑，补基准测试或说明为什么不需要。
- 如果修改跨 crate 协议、数据结构或行为契约，同步更新相关文档。

## Required Checks Before Opening A PR

默认要求：

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

在以下场景补充运行：

- 渲染、布局、解析器、存储等核心路径改动：`./scripts/run-benchmarks.sh`
- 大范围测试改动或覆盖率工作：`./scripts/check-coverage.sh`
- 只改文档：无需跑完整测试，但请确保文档内容与仓库现状一致

## AI-Assisted Contributions

AI 辅助贡献是这个项目的核心协作方式之一，但需要额外约束：

- 你需要对提交内容负责，不能提交自己也无法解释的生成代码。
- 保持 diff 可审查，避免一次性引入大块未经拆分的生成结果。
- 在 PR 描述中简要说明 AI 参与方式即可，不需要贴完整对话。
- 生成代码必须补足测试、错误处理和文档，不能把这些留给 reviewer 收尾。
- 如果 AI 基于外部代码或文档生成内容，确保来源和许可证边界清晰。

## Dependency And License Policy

ZeroBrowser 会优先接受许可证边界清晰的依赖，例如：

- `MIT`
- `Apache-2.0`
- `BSD`
- `ISC`
- `Zlib`

新增第三方依赖前，请先确认：

- 是否真的有必要引入
- 是否会进入主线核心路径
- 许可证是否与项目策略兼容
- 是否存在更简单的自实现方案

默认不要把 `GPL`、`AGPL`、`LGPL`、`MPL` 依赖引入核心浏览器路径。若确有必要，请先讨论，不要直接提交。

## Pull Request Checklist

- 改动范围清晰，标题和描述准确
- 测试和文档已同步更新
- 没有混入无关格式化或顺手重构
- 许可证和来源边界清晰
- reviewer 可以独立理解和验证你的改动

## Good Contribution Areas

如果你想参与但还没有具体题目，优先考虑这些方向：

- WPT 和集成测试补全
- `browser-shell` 产品层实现
- `script-sandbox` 的可用形态设计与实现
- 渲染、布局、样式系统中的兼容性缺口
- 文档、基准和开发者体验改进
