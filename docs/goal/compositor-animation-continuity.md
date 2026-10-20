# 主线程阻塞时的合成动画与 Worker Canvas

**版本**：v1.0 / 2026-10-20
**状态**：Active（启动门控：本轮只立项和准备脚本；等待后续明确启动）
**父目标**：[zero-web](zero-web.md) / **控制面**：[master](compositor-animation-continuity/master.md)

## 0. 执行摘要与 Mission

在有界 JS 主线程负载下验证可合成 opacity/transform 动画与 worker Canvas 持续呈现，并保持脚本查询、事件与命中测试一致。先实测支持子集，再设计脱离主线程的最小切片。

依据：2026-10-20 用户要求按 [Ladybird 建议](../research/research-ladybird-progress-2026-10-20.md)建立后续 goal。

## 基线与 Support Envelope

静态基线：`crates/engine/src/animation.rs` 有动画运行时，`apps/compositor/src/main.rs` 是合成器入口；协议与 Canvas 底座分别在 protocol/canvas。不能据这些模块存在推断动画或 worker canvas 已独立呈现；真实窗口基线 **NOT_RUN**。

范围：合成 opacity/transform、动画查询/结束/暂停/取消、命中测试；worker OffscreenCanvas 2D 的转移/呈现。background-color/filter 扩展按实测支持与单独切片评估。

边界：复用 [timing-animation 历史成果](archive/timing-animation-compat/master.md)与 [workers-compat](workers-compat.md)，不重复事件循环或 worker 生命周期。WebGL/WebGL2 依赖 [webgl-compat](webgl-compat.md)既有门控，此 goal不能替代其授权；缺能力记录依赖，不宣称全 Canvas 已实现。

碰撞：按 [run-rules §9–11](../rally/run-rules.md)协调 engine/protocol/compositor/workers 与 rendering-compat；协议字段改动同步双方和 Android 消费者。

## Done Criteria / 验收场景（P2）

| ID | 条件与动作 | 期望与验证 |
|---|---|---|
| DC-1 / COMP-01 | 页面运行有界长 JS，opacity/transform 持续动画 | 生产合成帧持续更新；用生产 GPU 帧时间和状态采集，不能用 CPU 截图证明 |
| DC-2 / COMP-02 | 脚本读取 computedStyle/currentTime/playState，动画结束 | 查询与合成状态一致，结束事件不重发/漏发；拟新增事件与帧联合 fixture |
| DC-3 / COMP-03 | 动画过程中点击、暂停、取消、切后台再恢复 | 命中当前几何，暂停/取消立即正确，恢复不跳旧世代；真实输入和事件断言 |
| DC-4 / COMP-04 | Canvas 转 worker，主线程阻塞，worker 退出/上下文丢失 | 支持的 2D 子集继续呈现；退出后释放资源，错误路径明确；拟新增 worker transfer fixture |
| DC-5 | 关闭/导航后验收资源与回归 | 无旧动画/Canvas 句柄或队列残留；支持范围、未执行 WebGL 与适用门禁如实记录 |

## 里程碑与实施交接

| 阶段 | 文件/职责与依赖 | 产物与验证 |
|---|---|---|
| M1 | engine 动画 → protocol → compositor → 真实窗口 | 阻塞负载 fixture 与支持矩阵；先 opacity/transform |
| M2 | 动画描述、时间采样、结束事件共享契约 | 最小合成独立推进切片；按需采样查询与真实点击回归 |
| M3 | canvas/worker 转移与呈现管线 | 2D 支持子集，生命周期与错误回退；WebGL 保持依赖 |
| M4 | 现有生产 parity 与资源门禁 | GPU/语义/资源证据、默认路径性能与 DC 审计 |

实现来源：复用现有 compositor、动画时间模型和 Canvas；拟新增动画描述在 protocol，双方消费共用契约；worker 2D 呈现复用 render-foundation。建议分批动画、查询/输入一致、worker 2D；新优化可分批回退 renderer 现有路径，保留证据。

## 执行与 Document Control

`./scripts/rally-goal-compositor-animation-continuity.sh` 直接执行启动 Rally，加 `--dry-run` 仅预览；统一 [启动协议](ladybird-followups/README.md)。内核开发与 GPU 验收本轮 **NOT_RUN**。

入口稳定，master/evidence 更新，archive 只追加；启动后默认 `CONTINUE`；依赖无法推进且无其他范围内任务时 `BLOCK`；完整 DC 与适用门禁有证才 `DONE`。
