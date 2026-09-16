---
name: zeroweb-regression-guard
description: 低频审计 ZeroWeb 的内存、断连、性能和 Chrome 生产一致性。用于 Rally 巡检或手动防劣化验收；修复、飞书通知和推送仅按既有任务授权执行。
---

# ZeroWeb 防劣化巡检

本 Skill 是低频、证据驱动的产品回归门禁，不属于每次提交前的固定流程。默认由 Rally 每天 06:00（调度机本地时区）执行；用户明确要求巡检时也可手动使用。

## 运行模式与授权

- 手动验收：执行用户指定版本的检查与本地报告；不因触发本 skill 自动切 main、拉取更新、修复、通知或推送。脏树可在授权范围内用隔离副本核验，并记录 patch 摘要；无法固定版本时报告 BLOCKED。
- 已授权巡检：从用户指令或有效调度配置提取检查范围、修复、通知、commit/push 的权限及时间上限，复用既有授权，不重复询问。下述 main 流程仅适用于已授权 main 交付的专用干净工作区。
- 部分授权：继续已授权工作；未授权的修复或外部动作写入本地报告，不把缺少可选权限作为检查阻塞。阅读 run-rules 只复用工具与资源入口，不自动获得其中其他 job 的外部操作权限。
- 讨论或编辑 skill 不启动巡检，也不创建调度任务。

## 必读资料

开始前完整阅读：

1. [references/guard-policy.md](references/guard-policy.md)；
2. 仓库 `docs/rally/run-rules.md` 的资源包裹和工具入口；
3. 涉及真实页面、点击或像素证据时，完整阅读并遵守 `zeroweb-browser-chrome-parity` Skill 及其 evidence contract；
4. 需要提交修复时，完整执行 `lei-pre-commit-guard` Skill。

## 执行流程

1. **同步并确定范围**
   - 先确定运行模式；main 巡检确认专用工作区在 `main` 且干净，否则报告 `BLOCKED`，不得覆盖、暂存或提交既有改动。
   - main 巡检执行 `git pull --rebase origin main`；手动验收固定用户指定版本。禁止强推。
   - main 巡检读取 `.rally/zeroweb-regression-guard.last-success` 和 `.rally/zeroweb-regression-guard.last-deferred`。成功记录须满足 guard-policy 的身份与完整门禁契约；同一未关闭专项记录仅可报告 `DEFERRED`，不能作为 PASS。匹配时直接结束，不启动构建或浏览器。手动明确要求重验则不以缓存跳过。
   - 首次运行或 SHA 已变化时，从成功记录提取 `git_sha`，记录该 SHA 到 HEAD 的提交和受影响模块；没有有效旧记录则标首次完整巡检。不得只凭 diff 推断“没有产品回归”而跳过核心门禁。

2. **收集证据**
   - 证据写入已忽略的 `.rally/zeroweb-regression-guard/<UTC timestamp>/`，不得提交截图、日志、临时报告或本机基线。
   - 按 [guard-policy.md](references/guard-policy.md) 执行核心门禁。CPU、页面耗时和整体峰值 RSS 必须复用 `make bench-gate` 及仓库已提交的 `docs/perf/baselines/`，不得另建重复基线。测试、构建、reftest 和基准必须走 `docs/rally/run-rules.md` 规定的 test-guard 入口。
   - 环境缺失、共享机器繁忙或 GUI/GPU 不可用属于 `INCONCLUSIVE/BLOCKED`，不得伪装为 PASS，也不得据此修改产品代码。

3. **判定**
   - 所有核心门禁通过：按 guard-policy 保存完整报告；main 巡检原子更新成功记录，输出 `DONE`。正常通过不发送飞书。
   - 任一门禁出现可复现劣化：保存失败命令、指标、阈值、基线、日志和证据路径；有通知授权时先告警，有修复授权时再进入修复，否则交付本地失败报告与建议。
   - 首次告警发送失败不阻断排查，但必须记录通知失败；不得因此把失败判为通过。

4. **分级处置**
   - 先定位引入提交和根因，优先修共享路径，不压制 watchdog、错误日志或门禁表现。
   - 只有修复已授权、根因明确、改动局部且能在剩余预算内完成完整验证时才自动修复。只修改与回归直接相关的代码；不得放宽阈值、覆盖较差基线、关闭检查或改测量配置来换取通过。
   - 涉及跨进程协议/生命周期重构、多模块架构调整、基线或政策放宽、无法完成质量门禁，或两次修复尝试仍失败时，停止自动改代码，转入 [guard-policy.md](references/guard-policy.md) 的“专项修复记录”流程，等待人工确认。
   - 自动修复时为根因补最贴近的常驻回归测试。渲染兼容性修复按仓库规则补 WPT/reftest；深入排查得到的经验写入 `docs/learnings/`。

5. **验证、提交和通知**
   - 重跑失败门禁，并执行 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`make test`；渲染/布局变更额外执行 `make product-smoke`，性能关键路径额外执行 `make bench-gate`。
   - 暂存且仅暂存本次修复，执行 `lei-pre-commit-guard`；只有裁决为 PASS 才能提交。
   - 有提交授权时先本地提交，确认工作区干净后再 `git pull --rebase origin main`，不对暂存的未提交修改执行 rebase，不使用 autostash。冲突只处理可安全归因部分；解决时再次检查将进入历史的内容，不能绕过安全门禁。
   - 在最终 HEAD 上重跑全部核心门禁；rebase 改变源码时同时重验受影响的工程门禁。最终 HEAD、源码树、二进制和证据必须绑定；只重跑原失败项不足以更新成功记录。
   - 有推送授权且最终验证通过时普通推送，禁止 `--force`。non-fast-forward 则重新同步并验证新 HEAD；预算不足保存本地结果，不标记新版本成功。
   - 按通知授权发送根因、修改、验证与真实交付状态。转专项按 guard-policy 保存记录；未经验证的修复不能夹带进文档提交，也不得声称已修复。

## 飞书通知

仅有明确通知授权时复用 `docs/rally/run-rules.md` 的 `lark-cli` 应用机器人命令；否则将同样内容保存在本地报告。不在仓库中保存用户 ID、webhook、token 或其他凭据。

告警至少包含：`[ZeroWeb 防劣化]`、当前 SHA、失败门禁、实测值与阈值、证据目录、是否开始自动修复。

结果至少包含：最终状态（已修复/仍阻塞）、根因、改动摘要、验证结果；已推送时附 commit SHA。
