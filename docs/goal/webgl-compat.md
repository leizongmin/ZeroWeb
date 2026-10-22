# WebGL 兼容 — webgl/（远期深度门控）

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active（**自主推进中**——2026-10-22 用户全量授权至 DONE，裁决记录见 master.md）
**执行模式**: WPT 驱动（上游 webgl/ conformance corpus 为验收标尺）；
GPU 管线深依赖——**除 M1 资产切片外全部用户门控**（先例：R1043/Phase A IFC）
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——GPU 图形面）

> **说明**
> 本文档是 ZeroWeb「WebGL 兼容」专项目标执行契约。WebGL 是地图/图表/游戏/可视化的
> 底座，工程量在四批 goal 中最大（GL 状态机 + shader 编译管线 + 上下文管理）。
> 本 goal 立项即声明：**M1（corpus 导入基线）可自主推进，M2 起每一切片须用户点名**，
> 避免无人值守循环在深结构上空转（8 轮净负回退教训）。
>
> **▶ 门控解除（2026-10-22 用户裁决）**：M2/M3/M4 全授权自主至 DONE，
> 四项裁决内容见 [master.md](webgl-compat/master.md) 已裁决清单。
>
> **▶ 拆分动机（2026-09-12 用户决策，第二批立项）**：① 真实可用性图谱完整性
> 要求该面有明确归属与挂账位置；② render-foundation 已有 wgpu+WGSL 管线底座
> （M7 全 13 种图元），GL 语义层有可依托的宿主；③ 立项即冻结边界——防止渲染流
> 或其他流误触 GL 域。
>
> **▶ 基线事实（2026-09-12 实测）**：
> - **无 GL 语义层**：render-foundation 仅 GPU 图元管线（wgpu+WGSL），无
>     WebGLRenderingContext/CGL2 状态机、shader 编译、纹理/缓冲对象模型
> - canvas getContext 面存在（2d 已用）——'webgl' 上下文类型缺
> - **WPT corpora**：`webgl/`（Khronos conformance 移植，体量大；dehydrated
>     形态 + 渲染断言——runner 形态支持启动时评估）
>     **▶ 2026-10-22 M1 实测修正**：pin rev 处 `webgl/` 为 10 文件平铺目录
>     （7 个 testharness 错误语义案约 15 subtests + 1 手工 crash 页 +
>     1 idlharness），dehydrated 与渲染断言均 **0 案**；Khronos 全量套件
>     不在 WPT 仓。见 [M1 盘点](webgl-compat/evidence/2026-10-22-m1-import-inventory.md)

---

## Mission

以 **WPT webgl/ conformance 为验收标准**，分切片落地 WebGL 1.0 最小上下文
（上下文创建/clear/draw 基础状态机）→ 缓冲/纹理对象 → shader 程序管线，
最终使基础 WebGL 场景可渲染。

**关键约束**：
1. **自主推进至 DONE（2026-10-22 用户全量授权）**：M2/M3/M4 全授权，切片边界
   Agent 按盘点与基线自划，每切片落账 + 常驻测试 + 飞书通知；`BLOCK` 仅用于
   真实阻塞（GPU 进程边界触线、DC-4 不可修复回归、资源门禁触发、无自主工作），
   禁止用 `CONTINUE` 表达等待——会逐轮自旋（2026-10-22 实证 8 轮零状态空转）
2. **GPU 栈依托 render-foundation**：不另起 wgpu 实例；CPU 回退路径评估记账
3. **conformance 体量分级**：首批收敛已裁决（2026-10-22：批次 B 全量一次收，
   7 案 15 subtests）

覆盖范围（远期视图，全部门控）：上下文创建/类型 / clear/draw 状态机 /
buffer/texture/framebuffer 对象 / shader 编译链接 / uniform/attribute / GLSL 翻译
（WGSL 后端评估）。

### 排除（明确不在范围内）

- **WebGL2 / WebGPU** —— 远期挂账（WebGL1 收口后评估）
- **canvas 2d 渲染面** —— canvas-2d goal 已归档收口
- **GPU 进程架构** —— compositor/renderer 多进程面归 protocol 域

---

## Support Envelope

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | getContext('webgl') 上下文对象面 | 实现切片（M2） |
| render-foundation | GL 状态机→wgpu 映射层 | **依托既有 GPU 栈，不另起实例** |
| WPT 资产 | webgl/ 子集导入与盘点（M1 唯一自主切片） | fetch 脚本 + 账本 |

**依赖约束（run-rules §9）**：与 rendering-compat —— paint/GPU 图元管线只依托
不改动；与 svg-compat —— 无共享面；与 compositor/renderer 进程架构 —— 不触
zero-protocol 契约；触 GPU 进程边界即 BLOCK 上报。

---

## Done Criteria

- [x] **DC-1**（2026-10-22）：webgl/ corpus 导入 + 用例分布盘点（dehydrated/渲染断言占比）+
      首批收敛子集建议书提交用户 + suites CSV 回填
      （[M1 盘点](webgl-compat/evidence/2026-10-22-m1-import-inventory.md)）
- [ ] **DC-2**（2026-10-22 授权）：WebGL 1.0 最小上下文（创建/clear/draw）WPT 子集收敛
- [ ] **DC-3**（2026-10-22 授权）：缓冲/纹理/shader 程序管线逐簇收敛（切片边界
      Agent 自划，逐簇落账）
- [ ] **DC-4**：`make test` 全绿 + clippy `-D warnings` + fmt + reftest 零回归
      + GPU/CPU 双路径差异记账

## 活跃里程碑

**M1** 导入盘点（唯一自主切片；corpus fetch 用编号脚本 `tests/wpt-runner/scripts/goals/99-webgl-compat.sh`（索引 README；幂等续拉，GitHub API 限流时部分目录跳过属预期）——**2026-10-22 完成**，见 [M1 盘点](webgl-compat/evidence/2026-10-22-m1-import-inventory.md)）→ **M2-M3**（自主切片：M2 批次 B 全量一次收 + clear/draw 标尺 ①+②；M3 边界
Agent 自划——均 2026-10-22 裁决）→
**M4** 收口（WebGL2/WebGPU 挂账定稿）。

## Final Output Protocol

`DONE`（DC-1~4 全满足或用户豁免 DC-2/3 并定稿挂账）/ `CONTINUE: <下一步>`（仅当
本轮有实质推进）/ `BLOCK: <原因>`（真实阻塞即输出——GPU 进程边界触线、DC-4
不可修复回归、资源门禁触发、无自主工作可做；rally 立即停。无工作时禁止
`CONTINUE`，防逐轮自旋——2026-10-22 八轮空转教训）。

## Document Control / Archive Policy

入口文档实质变化才改；控制平面 `docs/goal/webgl-compat/master.md`；
archive/ 只追加；evidence/ 持续追加。
