# WebGL 兼容 — 运行时控制面板（master.md）

**入口文档**: [../webgl-compat.md](../webgl-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-22（用户四项裁决——门控解除，全自主至 DONE）

## 当前状态

**M1 已完成（2026-10-22）**。pin rev 实测 `webgl/` 为
10 文件平铺目录（7 个 testharness 错误语义案约 15 subtests + 1 手工 crash 页 +
1 idlharness；dehydrated/渲染断言均 0——立项「体量大/dehydrated」假设修正）。
无 GL 语义层现状不变，GPU 栈依托 render-foundation，不另起 wgpu 实例。
M1 盘点与首批收敛子集建议书见
[evidence/2026-10-22-m1-import-inventory.md](evidence/2026-10-22-m1-import-inventory.md)。

**Rally 循环已停（2026-10-22 用户指示 park）**。当日 09:05–09:07 无人值守
单 goal run 在 M1 完成后依原契约输出 `CONTINUE: 等待用户门控`，rally 协议视
CONTINUE 为继续下一轮，~15s/轮自旋 8 轮零状态空转。契约已修：门控等待态改
输出 `BLOCK: 等待用户门控`（rally 立即停），见 [../webgl-compat.md](../webgl-compat.md)
约束 #1 与输出协议。

**2026-10-22 用户四项裁决——门控解除，授权自主推进至 DONE**：
① M2 启动，批次 **B 全量一次收**（7 案 15 subtests 一个切片）；
② clear/draw 标尺 **①+② 都要**——本地 reftest 补标尺（常驻 `local-reftests`，
   rendering-compat 通道先例）+ corpus 覆盖面照收；
③ idlharness 资产**补拉**（webgl1.idl / webgl2.idl / idlharness.js /
   WebIDLParser.js，按 fetch 脚本既有模式自 WPT pin rev 拉）；
④ M2/M3/M4 **全授权自主至 DONE**：DC-3 切片边界 Agent 按盘点与基线自划，
   每切片落账 + 常驻测试 + 飞书通知；DC-4 全绿门禁不放松；M4 挂账文档产出后
   交用户签收；`BLOCK` 仅用于真实阻塞（GPU 进程边界触线、DC-4 不可修复回归、
   资源门禁触发、无自主工作可做）。run 重启随本记录获授权
   （`bash scripts/rally-goal-99-webgl-compat.sh`）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | webgl/ corpus 导入 + 用例分布盘点（dehydrated/渲染断言占比） | ✅ 2026-10-22 |
| P2 | 首批收敛子集建议书提交用户 | ✅ 2026-10-22（裁决：批次 B） |
| P3 | WebGL 1.0 最小上下文（创建/clear/draw） | ▶ M2 已授权（批次 B；标尺 ①+②） |
| P4 | 缓冲/纹理/shader 程序管线 | ▶ M3 已授权（切片边界 Agent 自划） |
| P5 | WebGL2/WebGPU 挂账定稿 | ▶ M4 已授权（挂账文档交用户签收） |

## 已完成切片

- **M1 导入盘点**（2026-10-22）：`99-webgl-compat.sh` 补 `GOAL_PULL_ANY_JS=1`
  后拉齐语料（wpt-data/webgl/ 平铺 10 文件）；盘点 + 建议书 +
  suites CSV note 回填（planned 行维持，计数留空——无 runner 通道无基线）。
  evidence：[2026-10-22-m1-import-inventory.md](evidence/2026-10-22-m1-import-inventory.md)。

## 下一步计划

1. **M2 全量切片（自主）**：idlharness 资产补拉 + runner 通道
   （testharness_webgl + Makefile target + idlharness 载入）→ 上下文对象面 +
   WebIDL 校验 + buffer/texture 错误语义 + getError 状态机（GPU 依托
   render-foundation，不另起实例）→ clear/draw 本地 reftest 标尺
   （常驻 `local-reftests`）→ 基线首跑 + suites CSV 回填 + 飞书通知。
2. **M3 逐簇（自主）**：切片边界 Agent 按盘点与基线自划，逐簇落账。
3. **M4 收口（自主）**：WebGL2/WebGPU 挂账定稿文档 → 交用户签收。

**已裁决清单（2026-10-22 用户当面敲定）**：
- [x] M2 启动授权 + 批次 → 启动；批次 B 全量一次收
  （历史征询：2026-10-04 msg `om_x100b63125d66c4a8b2070829b8dd058`、
  2026-10-06 msg `om_x100b63647f7910a0b49d70def6b88ff` 均零回复；
  2026-10-22 建议书通知 msg `om_x100b62a4c43f40a0ddb7e528a0ae1f2`
  后当日用户当面裁决）；
- [x] clear/draw 验收标尺 → ①+② 都要（本地 reftest 补标尺 + corpus 覆盖面照收）；
- [x] idlharness 资产 → 补拉；
- [x] 自主授权范围 → 至 DONE 全自主（DC-3 边界 Agent 自划，M4 挂账交签收）。
