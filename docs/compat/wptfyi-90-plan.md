# WPT 子测试通过率接近 90% 的五个目标

2026-11-09。以最新已发布的 [2026-11-07 全量快照](wptfyi/coverage-2026-11-07.md) 为预算基线。ZeroWeb 已通过 1,551,067 / 2,268,984 个子测试（68.36%），执行过 16,548 / 126,638 个测试文件（13.07%）。本文的 89% 和 90% 均指子测试通过率。分母取 Chrome、Firefox、Safari 同 revision stable summary 的并集，沿用仓库对账器的分母修正规则。无子测试的 reftest 和 crashtest 不贡献该指标。

[11 月 8 日周跑](https://github.com/leizongmin/ZeroWeb/actions/runs/42499865933) 产生了 68.03% 的候选报告，聚合护栏发现 `custom-elements` 和 `shadow-dom` 的通过数掉到零，阻止了发布。候选报告的头部缺口排序基本相同。开工先恢复缺失产物，确认同 revision 的全量基线，再判断实际增量。

## 按全量百分比贡献排序

每行按一级目录归属，互不重复。缺口为快照分母减已通过数，包含未运行、未通过和执行环境缺口。上限是假定快照分母固定且该行全过的账面值。验收目标是本次为长期执行选定的预算。

| 顺序与目标 | 统计范围 | 尚未通过 | 上限 | 验收净新增 | 全量增量 |
|---|---|---:|---:|---:|---:|
| 1. [CSS 子测试](../goal/wptfyi-css.md) | `css` | 166,939 | +7.36 pp | 153,000 | +6.74 pp |
| 2. [Editing、Selection 与 Input Events](../goal/wptfyi-editing.md) | `editing`、`selection`、`input-events` | 138,411 | +6.10 pp | 130,000 | +5.73 pp |
| 3. [HTML、XHTML 与 DOM 接口](../goal/wptfyi-html.md) | `html` | 88,227 | +3.89 pp | 81,000 | +3.57 pp |
| 4. [Test262、Encoding 与 Wasm](../goal/wptfyi-runtime.md) | `third_party/test262`、`encoding`、`wasm` | 80,444 | +3.55 pp | 76,000 | +3.35 pp |
| 5. [WebCrypto](../goal/wptfyi-webcrypto.md) | `WebCryptoAPI` | 44,093 | +1.94 pp | 31,000 | +1.37 pp |
| 合计 | | 518,114 | +22.83 pp | 471,000 | +20.76 pp |

分母固定时，五项达到预算且其他目录无净回退，合计 2,022,067 / 2,268,984，约 **89.12%**。达到 89% 需要新增 **468,329** 项，达到 90% 需要 **491,019** 项，比预算再多 **20,019** 项。五项全部补齐的账面上限为 **91.19%**。预算要求回收五项缺口的约 91%，属于长期目标。CSS 和 HTML 各自要回收九成以上缺口，HTML 还依赖真实的 XML/XHTML 文档路径。Encoding 已占现有通过数约 73%，全量百分比与平台功能广度应分别阅读。

把五个范围内的通过数替换为同 revision 的 Chrome 实测通过数，其他目录维持 ZeroWeb 基线，约为 **89.22%**。逐文件取三家中最高的通过数作参考，约为 **90.01%**。后者是由多个浏览器组成的参考值，不能作为单一浏览器能达到的成绩。按这一参考值冲 90%，离门槛仅余 141 项，说明 89% 有可比的长期目标，90% 对测试覆盖、实现和回归余量都要求更高。[wpt.fyi 同 revision stable runs](https://wpt.fyi/api/runs?products=chrome,firefox,safari&labels=stable&max-count=1&aligned=true&sha=8c04fb8d43)

五项之外，最大的缺口是 SVG 13,996、WebNN 13,836、Fetch 11,059、WebDriver 10,439、Shadow DOM 10,239、DOM 9,445 和 Referrer Policy 8,814。它们仍留在分母中。每个目标的首轮应按上游路径和变体重估可回收量。上游三家均失败、依赖提案、缺少执行能力的部分分别列出，并用实跑结果更新达到 89% 和 90% 的差额。预算不足时保留差额及证据，继续推进可做的部分。

后续 06–11 的目录归属和启动脚本见 [95% 冲刺计划](wptfyi-95-plan.md)。

## 共同验收

1. 保存当前 ZeroWeb commit、runner 后端、语料 pin、三家 aligned run ID、按路径和变体的覆盖清单。重跑本目标与直接相邻套件，核对旧快照中已经通过的用例是否进入当前样本。每批把新增执行、失败转通过、已有通过回退、超时和未执行分别记账。对账时保持上游 revision 一致，另列上游漂移和新增分母。
2. 使用真实上游语料和 ZeroWeb 执行结果，修复随附常驻 WPT 或等价回归测试。上游 summary 用于对照，不复制结果。校验原始产物非空、命中预期的真实路径与变体，并检查分片是否截断。对账脚本可能把未知路径丢掉，也可能把同一源文件的多个名字投到同一变体。验收前确认每个计分路径只贡献一次，探针、环境跳过和重跑同一用例均不增加分子。
3. 将新增结果接入现有 [weekly 分片](../../.github/workflows/weekly.yml) 和 [coverage 聚合](wptfyi-gap-analysis.md#全量对账coverage)。保存干净语料的分片结果与同口径全量报告，复核其他目录无可归因回退，保留已有环比护栏。收口同时满足 [Goal 完成规则](../goal/README.md#4-done-判定) 和 [质量门禁](../engineering/quality-gates.md)。构建、测试与 clippy 使用有进程树内存上限和墙钟超时的守卫。
4. 从入口与 `master.md` 恢复进度。每轮写下有效增量、证据和下一批路径。达到本目标预算且所有门禁有据才输出 `DONE`。有可执行下一步输出 `CONTINUE: <下一步>`。全部剩余路线都依赖当前不可获得的外部条件时，先记录缺口与已尝试路线，再输出 `BLOCK: <原因>`。

## 无人值守启动

脚本使用仓库现有的 `claude-glm` 路由和 `--yes`，可加 `--dry-run` 预览。Linux 在 [受限开发 shell](../rally/oom-guard.md#l2linux-cgroup-兜底在限内存-scope-内开发仅-linux) 内运行。表格按预计贡献排序。启动时按脚本编号从 01 到 05 依次运行。01 先做 WebCrypto 的 CI 补账，02 验证 Test262 的计分通路。每个命令都是一个长任务，同一 clone 内等上一项结束再启动下一项。提交、推送和对外通知沿用执行时已有授权。已有 Rally 在改相同代码时，复用其已落地结果，并按 [Rally 运行规则](../rally/run-rules.md) 避开共享文件冲突。

```bash
bash scripts/rally-goal-01-wptfyi-webcrypto.sh
bash scripts/rally-goal-02-wptfyi-runtime.sh
bash scripts/rally-goal-03-wptfyi-css.sh
bash scripts/rally-goal-04-wptfyi-editing.sh
bash scripts/rally-goal-05-wptfyi-html.sh
```
