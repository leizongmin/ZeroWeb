# WPT 全量通过率 · Test262、Encoding 与 Wasm

**状态** ✅ Completed（2026-11-11——DC-1~3 全部完成：三目录合计 1,393,984 ≥ 验收线 1,393,955，对 11-07 基线净增 76,029 ≥ 76,000；终态见 [终局对账](wptfyi-runtime/evidence/dc3-final-reconciliation-2026-11-11.md)）\
**立项** 2026-11-09，按用户要求拆分高贡献 WPT 缺口，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](../zero-web.md)\
**控制面** [master.md](wptfyi-runtime/master.md)\
**共同计分与运行规则** [五目标计划](../../compat/wptfyi-90-plan.md)

## 基线与目标

2026-11-07 已发布快照中，本目标目录已通过 1,317,955 / 1,398,399。`third_party/test262` 53,794 个上游生成路径均未覆盖，`encoding` 余 23,847 项，`wasm` 余 2,803 项。复用 [Encoding 归档](encoding-compat-v2.md) 和 [Wasm 归档](wasm-compat.md)。[WPT Test262 是独立测试类型](https://web-platform-tests.org/running-tests/command-line-arguments.html)，其 [serve 包装器](https://github.com/web-platform-tests/wpt/blob/8c04fb8d43/tools/serve/serve.py) 提供 harness、provider 和负例等元数据。

把 `third_party/test262、encoding、wasm` 的上游语料接入可复跑的 ZeroWeb 执行链，在同口径全量报告中净增 **76,000** 项通过，达到至少 **1,393,955** 项。保留同 revision 路径级证据，并随最新周跑另报上游漂移。

## 范围与协作

以 ZeroWeb 发布配置的 V8 运行时为主。Test262 只计能对应到固定 WPT revision 的真实生成 URL、语义和结果。Encoding 与 Wasm 只计各自一级目录，HTML 下的表单编码归 HTML 目标。JS 桥共享写面与 zero-web 流协调。 按 [run-rules](../../rally/run-rules.md) 处理共享面冲突。有活跃流在同一写面时，先推进独立语料、对账和失败归类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。选同 revision 的同步、async、module、负例及 realm/agent 样本，走上游包装并核对正反例和覆盖报告是否一一匹配。宿主不支持的变体单列，先证实批量运行和 CI 分片预算，再承诺该项收益。
- **DC-2 · 收敛**。按 WPT manifest 分批跑 Test262，处理 isolate/realm、module、异步及负例异常阶段。补 Encoding 的根层 label/流式与剩余 legacy 变体，补 Wasm 的 JS API 与 Web API；保护既有百万级编码分子和 Wasm core。 每批保留结果增减和常驻回归。
- **DC-3 · 全量对账**。同口径净增至少 76,000 项，最新周跑出有效产物，邻域无可归因回退。按共同规则完成守卫下的构建、测试与 clippy，并归档证据。

每轮只更新控制面及证据。最终单行使用 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`，按共同计划中的判据选择。
