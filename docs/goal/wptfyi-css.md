# WPT 全量通过率 · CSS 子测试

**状态** Active\
**立项** 2026-11-09，按用户要求拆分高贡献 WPT 缺口，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](zero-web.md)\
**控制面** [master.md](wptfyi-css/master.md)\
**共同计分与运行规则** [五目标计划](../compat/wptfyi-90-plan.md)

## 基线与目标

2026-11-07 已发布快照中，本目标目录已通过 29,392 / 196,331。Grid、Typed OM、Color、Values 合计缺 49,238 项。现有 [CSS 分片](../../tests/wpt-runner/ci-shards.yml) 和 [CSS runner](../../tests/wpt-runner/src/testharness_css_corpus.rs) 已接多类子语料，可用真实覆盖清单定位剩余文件、变体和环境缺口。

把 `css` 的上游语料接入可复跑的 ZeroWeb 执行链，在同口径全量报告中净增 **153,000** 项通过，达到至少 **182,392** 项。保留同 revision 路径级证据，并随最新周跑另报上游漂移。

## 范围与协作

以 [rendering-compat](rendering-compat.md) 和 [CSS testharness 归档](archive/css-testharness-compat.md) 为已有实现和门禁。只对 `css/` 下可计分子测试记账。涉及渲染的修复用 reftest 钉住可观察行为。渲染流已有修复由其继续，遇到共享样式和布局入口先记下归属。 按 [run-rules](../rally/run-rules.md) 处理共享面冲突。有活跃流在同一写面时，先推进独立语料、对账和失败归类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。按 Grid、Typed OM、Color、Values 重跑分片，区分缺语料、执行截断、断言失败。优先查 Typed OM 348 个文件只记 25 项通过的原因。
- **DC-2 · 收敛**。补共享 CSSOM 读写、值解析与计算通路，再做 Grid、Sizing、Anchor Position 的布局与几何断言。逐步扩到 Fonts、Masking、Backgrounds、Shapes 和剩余高权重目录。 每批保留结果增减和常驻回归。
- **DC-3 · 全量对账**。同口径净增至少 153,000 项，最新周跑出有效产物，邻域无可归因回退。按共同规则完成守卫下的构建、测试与 clippy，并归档证据。

每轮只更新控制面及证据。最终单行使用 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`，按共同计划中的判据选择。
