# WPT 全量通过率 · Editing、Selection 与 Input Events

**状态** Active\
**立项** 2026-11-09，按用户要求拆分高贡献 WPT 缺口，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](zero-web.md)\
**控制面** [master.md](wptfyi-editing/master.md)\
**共同计分与运行规则** [五目标计划](../compat/wptfyi-90-plan.md)

## 基线与目标

2026-11-07 已发布快照中，本目标目录已通过 16,184 / 154,595。`editing/run` 占 96,745 项缺口，`selection/` 一级目录还缺 31,681 项。[11 月 8 日 catch-all 分片](https://github.com/leizongmin/ZeroWeb/actions/runs/42499865933/job/128809887622) 的 editing-run JSON 有 22 个 `zz-probe*` 本地路径，上游变体仍为 0 命中。本地已有 [editing runner](../../tests/wpt-runner/src/testharness_editing_corpus.rs) 与 [按用例运行的脚本](../../tests/wpt-runner/scripts/run-editing-testharness-corpus.sh)。

把 `editing、selection、input-events` 的上游语料接入可复跑的 ZeroWeb 执行链，在同口径全量报告中净增 **130,000** 项通过，达到至少 **146,184** 项。保留同 revision 路径级证据，并随最新周跑另报上游漂移。

## 范围与协作

复用 [editing-compat](editing-compat.md) 的命令语义成果和 [Selection 归档](archive/selection-compat-v2.md)。本目标归集三目录上游语料及周跑结果。编辑命令或 selection 桥的共享写面与现有流协调，避免在两个控制面同时开同一修复。HTML 下的测试归 HTML 目标。 按 [run-rules](../rally/run-rules.md) 处理共享面冲突。有活跃流在同一写面时，先推进独立语料、对账和失败归类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。从干净语料目录比较 33 号 fetch、现有按案执行和周跑 catch-all 的路径清单，确认为何分片只有探针。按命令、variant 分片跑真实 `editing/run`，验收至少一个上游文件的完整变体进入聚合后再扩面。
- **DC-2 · 收敛**。补 Selection/Range 的窗口、iframe、shadow 和输入集成；按真实失败量修 execCommand/queryCommand、contenteditable、剪贴与 Input Events 的目标选择和事件顺序。补 `plaintext-only` 和空白处理的高权重簇。 每批保留结果增减和常驻回归。
- **DC-3 · 全量对账**。同口径净增至少 130,000 项，最新周跑出有效产物，邻域无可归因回退。按共同规则完成守卫下的构建、测试与 clippy，并归档证据。

每轮只更新控制面及证据。最终单行使用 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`，按共同计划中的判据选择。
