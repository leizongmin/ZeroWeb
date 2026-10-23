# Ladybird 建议：ZeroWeb 后续目标与启动入口

**日期**：2026-10-20。7 个 goal 与 Shell 入口已准备；本轮只交付规划资产，未启动产品开发、基准、安装或发布。各目标处于 Active 启动门控，运行基线全部 **NOT_RUN**（2026-10-22 更新：P2 合成动画与 Worker Canvas 已完成收口并归档；2026-10-23 更新：P0 后台资源生命周期已完成收口并归档，P1-release 安装更新与崩溃反馈经用户决策终止归档——非 DONE，恢复=授权 P1/P4，见各自控制面；**2026-10-23 用户决策：renderer-trust-boundary / dom-operation-bench / media-runtime-isolation 三目标 M1 解锁**，无人值守可启动，RFC 门与深结构门控不随 M1 解锁，见各自 master）。

依据：[Ladybird 近期进展调研](../../research/research-ladybird-progress-2026-10-20.md)。这些目标是参考方向，不表示 ZeroWeb 已存在同样缺陷。源码与复用资产索引见 [catalog.json](catalog.json)；目标契约决定范围，master 是唯一执行状态控制面。

## 目标、优先级与入口

| 优先级 | 目标契约 | 控制面 | Shell 入口 |
|---|---|---|---|
| P0 | [Renderer 信任边界](../renderer-trust-boundary.md) | [master](../renderer-trust-boundary/master.md) | [rally-goal-renderer-trust-boundary.sh](../../../scripts/rally-goal-renderer-trust-boundary.sh) |
| P0 | [增量渲染完整计算对照](../incremental-rendering-oracle.md) | [master](../incremental-rendering-oracle/master.md) | [rally-goal-incremental-rendering-oracle.sh](../../../scripts/rally-goal-incremental-rendering-oracle.sh) |
| P0 | [后台资源生命周期](../archive/background-resource-lifecycle.md)——✅ 已完成（2026-10-23，模式 A 归档） | [master](../archive/background-resource-lifecycle/master.md) | 无（启动脚本已随归档删除，不重启） |
| P1 | [DOM 操作基准](../dom-operation-bench.md) | [master](../dom-operation-bench/master.md) | [rally-goal-dom-operation-bench.sh](../../../scripts/rally-goal-dom-operation-bench.sh) |
| P1-release | [安装更新与崩溃反馈](../archive/browser-delivery-loop.md)——⏸️ 终止归档（2026-10-23，用户决策；非 DONE，恢复=授权 P1/P4 并从归档重启） | [master](../archive/browser-delivery-loop/master.md) | 无（启动脚本已随归档删除） |
| P2 | [合成动画与 Worker Canvas](../compositor-animation-continuity/archive/compositor-animation-continuity-goal-v1-2026-10-22.md)——✅ 已完成（2026-10-22，模式 B 归档；控制面原地保留） | [master](../compositor-animation-continuity/master.md) | 无（启动脚本已随归档删除，不重启） |
| P2 | [平台媒体解码与隔离](../media-runtime-isolation.md) | [master](../media-runtime-isolation/master.md) | [rally-goal-media-runtime-isolation.sh](../../../scripts/rally-goal-media-runtime-isolation.sh) |

建议先推进三个 P0；DOM 基准可为渲染优化提供配对数据。交付闭环可在核心稳定后启动。动画、媒体先建立生产证据，再投入架构切片。优先级不代表同时启动，共享源码面需先协调。

## 后续执行

脚本使用 `#!/bin/sh`，不依赖 Python。复用现有 `rally run`，默认 agent 为 `claude-glm`，其余参数原样透传；要求环境已有 Rally 与对应 agent。

```sh
# 预览所选目标，Rally 不执行开发轮。
./scripts/rally-goal-renderer-trust-boundary.sh --dry-run

# 后续决定投入时，直接执行即可启动该目标。
./scripts/rally-goal-renderer-trust-boundary.sh
./scripts/rally-goal-incremental-rendering-oracle.sh
./scripts/rally-goal-dom-operation-bench.sh
./scripts/rally-goal-media-runtime-isolation.sh
```

以上逐行是独立入口。脚本定位自身所在仓库，因此从其他工作目录调用也使用该仓库与目标。`--dry-run` 由 Rally 处理；Shell 不另实现计划器、状态管理器或守卫。运行结束码原样返回。

用户后续明确要求启动或亲自执行不带 `--dry-run` 的脚本时，解锁该目标 M1 与契约允许的开发切片；不自动解锁其他目标、深结构门控或待确认 RFC。不继承兄弟流的提交、推送、通知、外部上传与发布授权。媒体架构、平台/签名/服务等待决项记录在各自 master。

## 运行守卫与证据

启动前读取 [目标维护规则](../README.md)、[Rally 规则](../../rally/run-rules.md)、[质量门禁](../../engineering/quality-gates.md)与 [OOM 守卫](../../rally/oom-guard.md)。核对当前源码/feature、已有工作区差异与并行流占用；按 §9–11 分配共享面，不覆盖其他流的状态或测试资产。运行中遵循目标 Final Output Protocol；阶段完成后继续下一切片，全部 DC 有证才 `DONE`。

每次执行在该目标 `evidence/<run-id>/` 独占创建记录，重复 ID 不覆盖。至少保存以下信息：

| 信息 | 记录要求 |
|---|---|
| 版本与环境 | HEAD、相关 diff、二进制/产物哈希、feature、平台、浏览器版本、GPU/字体等条件 |
| 隔离与动作 | 本次 profile/目录、PID/子进程、fixture 来源、视口、命令、超时与内存阈值 |
| 结果 | 场景/DC、期望与实际、原始报告/截图/事件/资源样本、PASS/FAIL/NOT_RUN/INCONCLUSIVE |
| 清理 | 只清理本次创建的进程/目录/服务，记录残留与恢复；保留失败和回滚证据 |

不要记录 token、Cookie、页面隐私正文或个人路径；日志使用项目相对路径。生产 GPU 帧不能由 CPU 截图代替；历史 fixture 不能冒充新生产基线。无数据保持 `NOT_RUN`，样本污染标 `INCONCLUSIVE`。

无人值守构建、测试、clippy 必须同时限制进程树内存和墙钟时间；使用现有守卫并覆盖编译阶段。脚本启动 Rally 不等于子命令自动受限。病态输入/故障复现遵守 [fuzz-repro-guard](../../../scripts/fuzz-repro-guard.sh) 与适用验收流程。按实际改动选定向检查；文档和独立脚本不触发 Rust 全量测试，后续产品修复必须补常驻回归并满足产品质量门禁。

## 规划范围审计

| 维度 | 已明确 | 留待基线/RFC |
|---|---|---|
| 目标 | 7 个独立 Mission 与可观察 DC | 确認缺陷与首切片 |
| 用户/行为 | 页面、宿主、后台、worker、媒体和安装者场景 | 所选平台/格式/支持子集 |
| 数据 | 身份、缓存、资源、样本、版本与报告 | 采样阈值、容差与报告保留 |
| 接口 | 现有 protocol、webview、自动化与打包入口 | 必要协议扩展与消费者同步 |
| 架构 | 复用共享路径，按模块职责分批 | 媒体隔离与合成独立推进 RFC |
| 质量 | 合法/非法、回滚、故障、恢复与生产证据 | 配对实测与平台门禁 |
| 交付 | 契约/master/evidence/archive 和 Shell 入口 | 提交、渠道、签名、服务、公开发布授权 |

规划资产检查见 [planning-checks.md](planning-checks.md)。该检查仅验证文档与启动脚本，不能用于勾选产品 DC。入口只在目标变化时改写，进展写 master，证据与里程碑分别追加到 evidence/archive。
