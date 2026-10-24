# 增量样式布局与完整计算对照

**版本**：v1.0 / 2026-10-20
**状态**：Done（2026-10-24 五 DC 与 M1–M4 全部收口；执行状态见 [master](incremental-rendering-oracle/master.md)）
**父目标**：[zero-web](zero-web.md) / **控制面**：[master](incremental-rendering-oracle/master.md)

## 0. 执行摘要与 Mission

为现有增量样式、布局缓存和帧复用建立确定性完整计算参考路径，发现失效传播、缓存输入和后处理导致的分歧。先验证现有优化，不预设重写样式引擎。

拆分依据：2026-10-20 用户要求按 [Ladybird 建议](../research/research-ladybird-progress-2026-10-20.md)立项，开发后续进行。

## 基线与 Support Envelope

源码核查：`crates/style-system/src/lib.rs::compute_styles_incremental` 重算祖先链和变更子树，含 `:has()` 时回退全量；布局/绘制经 `crates/engine/src/pipeline/mod.rs`，预算会话见 `pipeline_budget.rs`。运行基线 **NOT_RUN**。

范围：DOM/CSSOM 变更、继承、伪元素、Shadow DOM、容器/字体/语言变化；完整计算与增量结果的样式、几何、溢出、基线、命中测试、最终像素；缓存内存保留。

边界：规范修复与 Chromium oracle 仍复用 [rendering-compat](rendering-compat.md)，本 goal 负责内部完整计算对照与回放，内部等价不代表规范正确。Taffy 后处理全部纳入最终输出；深结构重写仍遵守已有授权边界。

碰撞：按 [run-rules §9–11](../rally/run-rules.md)协调 style/layout/engine/字体与共享 corpus；只写本 goal 控制面。原始 WPT pin 不变，不覆盖现有语料。

## Done Criteria / 验收场景（P0）

| ID | 条件与动作 | 期望与验证 |
|---|---|---|
| DC-1 / INC-01 | 依次改 class/属性、移除 CSSOM 最后声明、移动子树 | 增量和完整计算的可观察样式一致；拟新增确定性序列回放与差异报告 |
| DC-2 / INC-02 | 修改父字体、语言、伪元素、Shadow/slot/`:has()` 依赖 | 继承、选择器和伪元素更新正确；全量回退有计数；拟新增子系统测试 |
| DC-3 / INC-03 | 变更宽度、视口、基线、字体到达、布局影响属性；纯 paint 变更 | 缓存按真实输入失效或保留，移动复用几何正确；拟新增命中/必须失效断言 |
| DC-4 / INC-04 | 最终后处理、半透明帧复用及多次导航/释放缓存 | 几何/溢出/命中/像素符合预期；缓存不保留旧文档。复用生产 parity 与拟新增生命周期观察 |
| DC-5 | 关闭检查模式运行生产路径 | 检查默认不增加生产全量工作或诊断输出；对应基准与常驻回归有据 |

浮点比较容差在 M1 按输出契约确定并记录，不能临时放宽掩盖分歧。

## 里程碑与实施交接

| 阶段 | 文件/职责与依赖 | 产物与验证 |
|---|---|---|
| M1 | style-system 全量/增量 → layout-engine → engine 最终输出 | 选最小变更序列，记录同输入双路径快照；无依赖路由重写 |
| M2 | 最近样式/布局测试与参考计算入口 | 回放与首差异定位；人工注入失效错误能被检测 |
| M3 | render-foundation / compositor 转换消费者 | 帧复用/像素对照与缓存释放，遵守 GPU 生产证据 |
| M4 | scripts/bench-report、现有 WPT/oracle | 检查模式成本、默认路径性能、规范回归与 DC 审计 |

实现来源：复用现有完整计算和缓存；序列记录与比较拟放 engine/style-system 最近测试模块，生产像素复用现有 parity 采集器。建议分批落样式检查、布局/帧检查、成本门禁；每批可关闭新检查模式或撤回本批优化，不删除旧参照与失败证据。

## 执行与 Document Control

统一 CLI/证据/守卫见 [启动说明](ladybird-followups/README.md)；`./scripts/rally-goal-incremental-rendering-oracle.sh` 直接执行启动 Rally，加 `--dry-run` 仅预览。

入口稳定，master/evidence 持续更新，archive 只追加。未启动不执行 M1；启动后用 `CONTINUE: <下一切片>` / `BLOCK: <真实阻塞>`；全部 DC 与适用门禁有据才 `DONE`。
