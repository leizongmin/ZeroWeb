# 合成动画连续性控制面

**日期**：2026-10-21
**状态**：Active；M1 ✅ 完成——headless 基线 + 真实窗口帧序列均实测 FROZEN（真实窗口内容归因经像素三重证实，首轮 INCONCLUSIVE 已由 run-20261021b 闭合）。
**当前卡点**：生产 renderer 时钟泵（`tick_animation_clock`）无调用方，CSS 动画停在静态 t=0（R3882）；窗口实测确认加载收敛后合成器零提交（renderer 不推进 t、compositor 不重采样，缺口在两端）。
**下一步**：M2 首切片：动画描述入 protocol 的切片 RFC（合成器 Get 时采样，参照 `GetCompositorFrame` scroll 重光栅先例）+ 最小实现与 DC-1 窗口验收。
**入口**：[目标契约](../compositor-animation-continuity.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 动画是否独立推进及实际支持子集 | COMP-01 基线探针已跑：GPU 截图 3 帧全同 + 查询恒定（见 M1 证据） | BASELINE_FROZEN（2026-10-21） |
| 时间/状态/命中几何共享契约 | 暂停、取消、结束和后台恢复联合验证 | NOT_STARTED |
| Worker OffscreenCanvas 2D 呈现与释放 | 转移、worker 退出和 context lost fixture | NOT_STARTED |
| WebGL 依赖与扩展属性 | 遵守 webgl-compat 门控，独立记录未支持项 | 待核查 |
| headless 阻塞面备忘：同步 evaluate 占用 mux 主循环（1100ms 截图等待实测） | M2 切片设计时核对 #24/#15 学习 | 已记录（M1 证据 README §结论 4） |
| 窗口模式无 automation 通道：负载时序无法在窗口观察中触发（首跑已记） | DC-1 完整验收走 M2 切片（compositor 采样工具+headless 负载通道证据组合） | 已记录（window-frames README §局限） |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 真实窗口阻塞基线与支持矩阵 | ✅ 完成（headless 基线 + 真实窗口帧序列 FROZEN，内容归因像素三重证实；窗口多帧采样工具 frame_dump 已入库） | [2026-10-21-m1-comp01-baseline](evidence/2026-10-21-m1-comp01-baseline/README.md) / [2026-10-21-m1-window-frames](evidence/2026-10-21-m1-window-frames/README.md) |
| M2 | 合成动画、状态与输入一致切片 | IN_PROGRESS（首切片：动画描述入 protocol + compositor Get 时采样 RFC/最小实现） | 无 |
| M3 | Worker 2D 呈现与生命周期 | NOT_STARTED | 无 |
| M4 | GPU/语义/资源与 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | COMP-01-transform-opacity-under-js-load | NOT_RUN（验收口径；M1 双基线实测 FROZEN：headless + 真实窗口） | [M1 基线](evidence/2026-10-21-m1-comp01-baseline/README.md) / [窗口帧序列](evidence/2026-10-21-m1-window-frames/README.md) |
| DC-2 | COMP-02-script-state-and-end-event | NOT_RUN | 无 |
| DC-3 | COMP-03-hit-test-pause-cancel | NOT_RUN | 无 |
| DC-4 | COMP-04-worker-canvas-transfer | NOT_RUN | 无 |
| DC-5 | 关闭/导航释放与支持审计 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：engine/protocol/compositor/canvas/workers、Android 协议消费者。2026-10-21 本轮改动仅 fixture（tests/fixtures/compositor-animation/）、探针脚本（scripts/compositor-animation-comp01-probe.{sh,mjs}）与本 goal 文档，未触 engine/protocol/renderer/compositor 源码，无共享面冲突；开工前 `git pull --rebase` 已同步 main。
2026-10-21 续轮：新增 `apps/compositor/src/frame_dump.rs`（env 门控帧落盘，默认关闭，含单测）及 lib.rs 接线；M1 真实窗口证据落账（evidence/2026-10-21-m1-window-frames/）。触碰 compositor 源码但仅新增诊断旁路，不改合成语义；M2 RFC 动笔前待核对 protocol 消费侧（Android/JNI）共享面。

复用资产：timing-animation（runner 泵/查询面）、workers-compat（worker 生命周期）；不重复实现。授权记录：2026-10-20 规划轮不授权提交/推送；2026-10-21 用户启动指令授权按仓库规则提交推送。生产 GPU 证据口径：headless GPU 光栅为生产渲染管线+生产 GPU 光栅路径，非窗口合成器 vsync 帧序列（局限见证据 README）。
