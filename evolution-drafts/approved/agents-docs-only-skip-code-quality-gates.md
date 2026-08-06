# Evolution Proposal: Skip code quality gates for documentation and GitHub metadata changes

- Created-At: 2026-08-07 00:28
- Target-File: AGENTS.md
- Trigger-Type: explicit-instruction

## Why This Matters
- 仅修改 `docs/**`、`.github/**` 或 Markdown 文件时，Rust 格式、clippy、构建和测试不会验证这些内容，却会下载依赖并消耗大量时间。
- 明确豁免边界可避免把文档和 GitHub 元数据提交误升级为项目代码检查，同时保留文件类型对应的验证和提交安全门禁。

## Evidence
- Source: 用户纠正
- Observed Problem: 文档治理变更提交前启动了由 `test-guard` 包裹的全 workspace clippy；用户指出只更新文档、`.github/**` 和 Markdown 文件时不应执行项目代码相关检查和测试。
- Correct Pattern: 当且仅当全部待提交文件都匹配 `docs/**`、`.github/**` 或 `**/*.md` 时，跳过 Rust 格式、clippy、构建、测试、reftest、基准和覆盖率；继续执行相关 Markdown 链接、YAML、CODEOWNERS、`git diff --check` 和 `lei-pre-commit-guard`。
- Suggested Action: 在 `AGENTS.md` 的“提交前质量门禁”中增加文档与 GitHub 元数据例外，并明确任一不匹配允许范围的文件都会取消豁免。
- Verification or Resolution: 用户已明确批准，规则已写入 `AGENTS.md`。

## Conflict Points
- 现有规则写明“执行 `git commit` 前，必须先在本地跑通 `cargo fmt` 和 `cargo clippy`，禁止跳过”，未区分文档和 GitHub 元数据提交。新规则将为仅含 `docs/**`、`.github/**` 或 `**/*.md` 的提交增加窄范围例外。

## Plan
1. 将“提交前质量门禁”开头替换为：仅当全部待提交文件都匹配 `docs/**`、`.github/**` 或 `**/*.md` 时，可跳过 `cargo fmt`、`cargo clippy`、构建、测试、reftest、基准和覆盖率。
2. 豁免项目代码检查时，仍须执行 `git diff --check`、`lei-pre-commit-guard`，并按变更类型执行相关 Markdown 链接、YAML、CODEOWNERS 或其他配置语法检查。
3. 明确只要待提交内容包含任一不匹配上述允许范围的文件，就不适用豁免，必须继续执行原有 `cargo fmt --all -- --check` 和 `cargo clippy --workspace --all-targets -- -D warnings` 门禁。
4. 保留现有默认 feature 无法编译时的降级规则及原因说明。
