# WPT 全量子测试通过率 95% 冲刺

2026-11-09。06–11 承接 [01–05 的五个目标](wptfyi-90-plan.md)，沿用 [11 月 7 日快照](wptfyi/coverage-2026-11-07.md) 和 [共同验收规则](wptfyi-90-plan.md#共同验收)。这是 ZeroWeb 以同一 revision 的三家 stable summary 并集为分母的子测试通过率。

| 编号与目标 | 现有通过数 | 账面缺口 | 净新增预算 | 全量增量 |
|---|---:|---:|---:|---:|
| [06 DOM 与 Shadow DOM](../goal/wptfyi-dom.md) | 64,947 | 41,603 | 35,000 | +1.54 个百分点 |
| [07 Fetch 与网络安全策略](../goal/wptfyi-network.md) | 5,562 | 39,036 | 33,000 | +1.45 个百分点 |
| [08 URL、Streams 与存储](../goal/wptfyi-data.md) | 10,120 | 29,270 | 25,000 | +1.10 个百分点 |
| [09 SVG、MathML 与动画](../goal/wptfyi-graphics.md) | 3,974 | 26,276 | 22,000 | +0.97 个百分点 |
| [10 WebDriver 与真实输入](../goal/wptfyi-automation.md) | 2,129 | 12,613 | 10,000 | +0.44 个百分点 |
| [11 媒体与实时通信](../goal/wptfyi-media.md) | 1,402 | 14,643 | 10,000 | +0.44 个百分点 |
| **合计** | **88,134** | **163,441** | **135,000** | **+5.95 个百分点** |

当前基线为 1,551,067 / 2,268,984。按固定分母，95% 需要至少 2,155,535 项通过。01–05 的预算新增 471,000 项后还差 133,468 项，06–11 如再新增 135,000 项且其余范围无净回退，结果为 **2,157,067 / 2,268,984（95.07%）**，余量为 1,532 项。各 goal 首轮用实跑校准可回收量，最后以同一集成版本的全量报告确认 95%。新上游 revision 的分母变化单独报告。

06–11 各自按入口文档列出的一级目录计分，与 01–05 互斥。共享修复产生的连带收益归入测试路径所在的 goal。WebNN 的 13,836 项缺口留作候补，本批预算只计这六个目标。

## 无人值守运行

脚本默认使用 `claude-glm` 和 `--yes`，可加 `--dry-run` 预览。在同一 clone 里按编号串行运行。不同 clone 可以并行启动，优先让 07 和 10 验证多源 HTTPS 与 WebDriver 会话。每个 Linux clone 各自进入 [受限开发 shell](../rally/oom-guard.md#l2linux-cgroup-兜底在限内存-scope-内开发仅-linux)，测试和构建继续使用有内存上限与墙钟超时的守卫。各 clone 独立使用 `target/`、语料、日志和服务端口。同机重型构建和全量跑按可用内存错开。

```bash
bash scripts/rally-goal-06-wptfyi-dom.sh
bash scripts/rally-goal-07-wptfyi-network.sh
bash scripts/rally-goal-08-wptfyi-data.sh
bash scripts/rally-goal-09-wptfyi-graphics.sh
bash scripts/rally-goal-10-wptfyi-automation.sh
bash scripts/rally-goal-11-wptfyi-media.sh
```

07 负责多源 HTTP(S)、证书和动态响应夹具，10 负责 WPT 自动化会话与真实输入。其他目标可以先用已有夹具跑基线，再接入这两条执行链。按 [Rally 运行规则](../rally/run-rules.md) 管理共享文件。改动同一 runner、引擎入口、CI 分片或语料 pin 时，先在各自控制面记录归属，由一条流完成该处修改，其余流继续独立语料和失败分类。提交、推送、合并和对外通知按启动时已有授权执行。

每轮保存 ZeroWeb commit、WPT revision、原始分片结果和按路径的净增量。分片截断、环境受限和未运行分别记录。环境或依赖故障先按日志修复或重试，仍卡住就转到独立切片。达到本 goal 净增预算、验收门禁和归档要求后才输出 `DONE`。继续推进用 `CONTINUE: <下一步>`。全部剩余路线受同一个外部条件阻挡时，控制面记录证据和解锁条件，再输出 `BLOCK: <原因>`。
