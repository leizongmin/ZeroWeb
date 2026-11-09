# WPT 全量通过率 · WebCrypto

**状态** Active\
**立项** 2026-11-09，按用户要求拆分高贡献 WPT 缺口，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](zero-web.md)\
**控制面** [master.md](wptfyi-webcrypto/master.md)\
**共同计分与运行规则** [五目标计划](../compat/wptfyi-90-plan.md)

## 基线与目标

2026-11-07 已发布快照中，本目标目录已通过 26,779 / 70,872。`derive_bits_keys` 缺 24,640 项、`modern-algos` 缺 14,348 项。[v3 归档](archive/webcrypto-compat-v3.md) 已记录 67,845 项本地通过，尚需按 aligned stable 路径核对可入账部分。[当前周跑 crypto 分片](../../tests/wpt-runner/ci-shards.yml) 调用 `fetch-wpt-webcrypto`，其 [Makefile 入口](../../Makefile) 拉取一期语料。

把 `WebCryptoAPI` 的上游语料接入可复跑的 ZeroWeb 执行链，在同口径全量报告中净增 **31,000** 项通过，达到至少 **57,779** 项。保留同 revision 路径级证据，并随最新周跑另报上游漂移。

## 范围与协作

仅计 `WebCryptoAPI/`。复用 [38 号语料脚本](../../tests/wpt-runner/scripts/goals/38-webcrypto-compat-v3.sh) 与既有实现。保持 secure context、密钥用途、extractable、异常与算法参数语义；超出已有支持范围的算法先对照上游和归档证据，避免重复实现。 按 [run-rules](../rally/run-rules.md) 处理共享面冲突。有活跃流在同一写面时，先推进独立语料、对账和失败归类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。用干净环境重跑一期和 v3 语料，先做按路径与 scope 对照，定位快照少记的 derive、现代算法和 worker 结果。验证新旧目录布局映射后接入周跑分片。
- **DC-2 · 收敛**。对补账后仍有差异的密钥派生、现代算法和 worker 路径分簇，按最高可回收量逐批处理。验证同路径只有一份计分结果，并保留一期兼容回归。 每批保留结果增减和常驻回归。
- **DC-3 · 全量对账**。同口径净增至少 31,000 项，最新周跑出有效产物，邻域无可归因回退。按共同规则完成守卫下的构建、测试与 clippy，并归档证据。

每轮只更新控制面及证据。最终单行使用 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`，按共同计划中的判据选择。
