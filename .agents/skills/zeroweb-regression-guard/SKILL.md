---
name: "zeroweb-regression-guard"
description: "低频审计 ZeroWeb 的 renderer/compositor 内存、断连、性能和 Chrome 生产路径一致性；发现劣化后先飞书告警，再最小修复、验证并推送 main。用于夜间 Rally 巡检或用户明确要求防劣化验收时。"
---

# ZeroWeb 防劣化巡检

本 Skill 是低频、证据驱动的产品回归门禁，不属于每次提交前的固定流程。默认由 Rally 每天 06:00（调度机本地时区）执行；用户明确要求巡检时也可手动使用。

## 必读资料

开始前完整阅读：

1. [references/guard-policy.md](references/guard-policy.md)；
2. 仓库 `docs/rally/run-rules.md`；
3. 涉及真实页面、点击或像素证据时，完整阅读并遵守 `zeroweb-browser-chrome-parity` Skill 及其 evidence contract；
4. 需要提交修复时，完整执行 `lei-pre-commit-guard` Skill。

## 执行流程

1. **同步并确定范围**
   - 确认当前分支为 `main`、工作区无未提交变更；否则按飞书规则报告 `BLOCKED`，不得覆盖、暂存或提交既有改动。
   - 执行 `git pull --rebase origin main`，禁止强推。
   - 读取 `.rally/zeroweb-regression-guard.last-success` 和 `.rally/zeroweb-regression-guard.last-deferred`。若当前 `HEAD` 已成功巡检，或已作为同一未关闭专项问题的记录提交，直接输出 `DONE`，不启动构建、浏览器或额外 Agent 轮次。
   - 首次运行或 SHA 已变化时，记录 `last-success..HEAD` 的提交和受影响模块；不得只凭 diff 推断“没有产品回归”而跳过核心门禁。

2. **收集证据**
   - 证据写入已忽略的 `.rally/zeroweb-regression-guard/<UTC timestamp>/`，不得提交截图、日志、临时报告或本机基线。
   - 按 [guard-policy.md](references/guard-policy.md) 执行核心门禁。CPU、页面耗时和整体峰值 RSS 必须复用 `make bench-gate` 及仓库已提交的 `docs/perf/baselines/`，不得另建重复基线。测试、构建、reftest 和基准必须走 `docs/rally/run-rules.md` 规定的 test-guard 入口。
   - 环境缺失、共享机器繁忙或 GUI/GPU 不可用属于 `INCONCLUSIVE/BLOCKED`，不得伪装为 PASS，也不得据此修改产品代码。

3. **判定**
   - 所有核心门禁通过：原子写入当前 `HEAD` 到 `.rally/zeroweb-regression-guard.last-success`，输出 `DONE`。正常通过不发送飞书，减少噪声。
   - 任一门禁出现可复现劣化：保存失败命令、指标、阈值、基线、日志和证据路径；先发送飞书告警，再开始修复。
   - 首次告警发送失败不阻断排查，但必须记录通知失败；不得因此把失败判为通过。

4. **分级处置**
   - 先定位引入提交和根因，优先修共享路径，不压制 watchdog、错误日志或门禁表现。
   - 只有根因明确、改动局部且能在本次 job 内完成完整验证时才自动修复。只修改与回归直接相关的代码；不得放宽阈值、覆盖较差基线、关闭检查或改测量配置来换取通过。
   - 涉及跨进程协议/生命周期重构、多模块架构调整、基线或政策放宽、无法完成质量门禁，或两次修复尝试仍失败时，停止自动改代码，转入 [guard-policy.md](references/guard-policy.md) 的“专项修复记录”流程，等待人工确认。
   - 自动修复时为根因补最贴近的常驻回归测试。渲染兼容性修复按仓库规则补 WPT/reftest；深入排查得到的经验写入 `docs/learnings/`。

5. **验证、提交和通知**
   - 重跑失败门禁，并执行 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`make test`；渲染/布局变更额外执行 `make product-smoke`，性能关键路径额外执行 `make bench-gate`。
   - 暂存且仅暂存本次修复，执行 `lei-pre-commit-guard`；只有裁决为 PASS 才能提交。
   - 提交前再次 `git pull --rebase origin main`，解决可安全归因的冲突后重验受影响门禁；提交并普通推送到 `main`，禁止 `--force`。
   - 修复推送成功后更新本地成功 SHA，并发送飞书结果（根因、修改、验证、commit SHA）。转专项时只提交问题记录，写入本地 deferred SHA，并发送飞书说明“等待人工确认与专项修复”；不得声称已修复。

## 飞书通知

通知必须复用 `docs/rally/run-rules.md` 的 `lark-cli` 应用机器人命令，不在仓库中保存用户 ID、webhook、token 或其他凭据。

告警至少包含：`[ZeroWeb 防劣化]`、当前 SHA、失败门禁、实测值与阈值、证据目录、是否开始自动修复。

结果至少包含：最终状态（已修复/仍阻塞）、根因、改动摘要、验证结果；已推送时附 commit SHA。
