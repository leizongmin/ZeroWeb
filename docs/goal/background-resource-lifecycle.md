# 后台页面与重复导航资源生命周期

**版本**：v1.3 / 2026-10-22
**状态**：Active（M1 基线 + M2-S1 fetch 取消修复复测通过 + M2-S2 RSS 有界平台判定；文档存活计数与 DC-1/DC-2 修复面未开始）
**父目标**：[zero-web](zero-web.md) / **控制面**：[master](background-resource-lifecycle/master.md)

## 0. 执行摘要与 Mission

验证后台 timer、视口外动画、重复导航、停滞资源和页面关闭是否释放旧文档并减少无效工作。按同环境基线建立资源门禁，不先编造 CPU/RSS 达标数字。

依据：2026-10-20 用户要求将 [Ladybird 建议](../research/research-ladybird-progress-2026-10-20.md)转为后续 goal。

## 基线与 Support Envelope

静态基线：`apps/browser/src/process_backend.rs` 存在 `renderer_poll_order_prioritizes_active_tab_and_throttles_background_tabs`；renderer 运行入口为 `apps/renderer/src/runtime.rs`，页面共享入口在 `crates/page-runtime/src/lib.rs`。轮询节流不证明 timer/动画/合成器已停无效工作。

生产资源基线：M1 已建立（2026-10-22，12 腿，headless CDP + GUI X :20 双口径），见 [M1 基线](background-resource-lifecycle/evidence/2026-10-22-m1-baseline/ANALYSIS.md)。要点：嵌套零延迟 timer ~20 万 iter/s 无钳制；前后台 tab CPU 无差异（后台无策略）；CSS 动画不驱动持续呈现（视口内外一致，呈现链未激活）；重复导航 renderer RSS +224MB 且 stall fetch 连接随导航滞留；tab 开关 renderer 进程复用、内存不回落。M2-S1（2026-10-22）修复挂起 fetch 无取消路径——调度器 `cancel_by_url` + renderer 页面 fetch 登记 + browser `cancel_tab` 扩展，生产复测连接随导航关闭（61 次 create/cancel 零失手），见 [M2-S1](background-resource-lifecycle/evidence/2026-10-22-m2-s1-fetch-cancel/ANALYSIS.md)。M2-S2（2026-10-22）双腿 120 次导航判定 RSS 为爬坡+有界平台（非无界增长，M1 线性读数为窗口假象），文档存活计数待建，见 [M2-S2](background-resource-lifecycle/evidence/2026-10-22-m2-s2-rss-attribution/ANALYSIS.md)。观测资产：`scripts/background-resource-{observe.sh,probe.mjs,sampler.py,fixture-server.py}` + `tests/fixtures/background-resource/`。

范围：各进程 CPU、RSS 趋势、旧文档/回调/连接/队列与呈现次数；前后台切换、重复导航和关闭后恢复。

边界：复用 [page-state-compat](archive/page-state-compat.md)、[navigation-compat](navigation-compat.md)、现有 regression-guard。可见性 API 规范复用历史成果，本 goal负责长期生产资源；不绕过 iframe 或深结构门控。

碰撞：按 [run-rules §9–11](../rally/run-rules.md)协调 browser/renderer/page-runtime/engine/compositor；只写本目标控制面；不停止用户实例。

## Done Criteria / 验收场景（P0）

| ID | 条件与动作 | 期望与验证 |
|---|---|---|
| DC-1 / BG-01 | 连续零延迟嵌套 timer，切后台再恢复 | 嵌套最小延迟符合规范；后台策略明确且恢复正确。拟新增 timer 时序测试与生产 CPU 采样 |
| DC-2 / BG-02 | 动画移出视口、页面静止、隐藏标签 | 可跳过工作不持续重复呈现；音视频等合法工作不误停。生产帧计数和进程 CPU 对照 |
| DC-3 / BG-03 | 重复导航，新页有不结束子资源 | 已替换文档、回调与资源可释放；RSS 不随替换次数无界增长。拟新增本地延迟 HTTP fixture 与文档存活计数 |
| DC-4 / BG-04 | 关闭/重开页面并切回前台 | 本次 renderer/连接/队列退出或复用符合生命周期；前台输入与动画恢复。真实宿主操作与资源清理记录 |
| DC-5 | 固定场景持续运行并复测修复 | 时长、预热、样本、阈值和环境先记录；缺指标标 NOT_RUN/INCONCLUSIVE，不用单次峰值证明无泄漏 |

## 里程碑与实施交接

| 阶段 | 文件/职责与依赖 | 产物与验证 |
|---|---|---|
| M1 | 现有 regression-guard/进程采样 + 隔离浏览器 fixture | 四类场景基线；固定 run ID/profile/PID/视口/时间；先对比前后台 |
| M2 | page-runtime/renderer timer 与导航生命周期 | 首个增长来源调用链、最小回归；确认后修共享 owner |
| M3 | engine/compositor 帧调度 | 空闲/视口外工作门控与恢复行为；真实 GPU 计数 |
| M4 | 既有性能资源门禁 | 建立同环境阈值，长期配对复测、DC 审计与清理 |

实现来源：进程采样优先复用 regression-guard；文档/队列计数拟加在真实 owner 的诊断入口，默认不输出到页面。本地 fixture 放最近集成测试资源；守卫限制资源和墙钟。建议分批：基线资产、确认泄漏修复、调度修复。回滚本批调度或 owner 修改，保留观测与旧失败证据。

## 执行与 Document Control

`./scripts/rally-goal-background-resource-lifecycle.sh` 直接执行启动 Rally，加 `--dry-run` 仅预览；统一 CLI 与证据协议见 [启动说明](ladybird-followups/README.md)。

启动前不采样用户浏览器。入口稳定，master/evidence 更新，archive 只追加；启动后 `CONTINUE` / `BLOCK`，全部 DC 与适用门禁有证才 `DONE`。
