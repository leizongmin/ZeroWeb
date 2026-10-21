# WebGL 兼容 — 运行时控制面板（master.md）

**入口文档**: [../webgl-compat.md](../webgl-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-22（M1 建议书飞书通知登记）

## 当前状态

**M1 已完成（2026-10-22），goal 转等待用户门控态**。pin rev 实测 `webgl/` 为
10 文件平铺目录（7 个 testharness 错误语义案约 15 subtests + 1 手工 crash 页 +
1 idlharness；dehydrated/渲染断言均 0——立项「体量大/dehydrated」假设修正）。
无 GL 语义层现状不变，GPU 栈依托 render-foundation，不另起 wgpu 实例。
M1 盘点与首批收敛子集建议书见
[evidence/2026-10-22-m1-import-inventory.md](evidence/2026-10-22-m1-import-inventory.md)。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | webgl/ corpus 导入 + 用例分布盘点（dehydrated/渲染断言占比） | ✅ 2026-10-22 |
| P2 | 首批收敛子集建议书提交用户 | ✅ 2026-10-22（批次选项 A/B 待裁） |
| P3 | WebGL 1.0 最小上下文（创建/clear/draw）——**门控** | 🔒 M2（批次 + clear/draw 标尺两条路待用户裁决） |
| P4 | 缓冲/纹理/shader 程序管线——**门控**（切片边界用户定） | 🔒 M3 |
| P5 | WebGL2/WebGPU 挂账定稿 | ⏳ M4 |

## 已完成切片

- **M1 导入盘点**（2026-10-22）：`99-webgl-compat.sh` 补 `GOAL_PULL_ANY_JS=1`
  后拉齐语料（wpt-data/webgl/ 平铺 10 文件）；盘点 + 建议书 +
  suites CSV note 回填（planned 行维持，计数留空——无 runner 通道无基线）。
  evidence：[2026-10-22-m1-import-inventory.md](evidence/2026-10-22-m1-import-inventory.md)。

## 下一步计划

1. **等待用户门控**：M2 批次选择（选项 A 逐批 / 选项 B 全量一次收）+
   clear/draw 验收标尺两条路（本地 reftest 补标尺 / 接受 corpus 覆盖面）——
   均在 M1 建议书内。M2 落地时附带 runner 通道（testharness_webgl +
   Makefile target + idlharness skip）与基线首跑。

**待用户决策清单**：
- [ ] M2 最小上下文切片启动授权 + 批次选项（A 逐批 / B 全量）
  - 前置 M1 已产出（2026-10-22），建议书随本文件提交；
  - 早期征询（2026-10-04 msg `om_x100b63125d66c4a8b2070829b8dd058`）与 48h
    跟进（2026-10-06 msg `om_x100b63647f7910a0b49d70def6b88ff`）均零回复，
    当时建议维持挂起；M1 建议书已就绪，待用户随建议书一并裁决；
  - M1 建议书送达通知（2026-10-22 msg
    `om_x100b62a4c43f40a0ddb7e528a0ae1f2`）：随 M1 落账发出三项裁决汇总
    （启动授权 + 批次 / clear/draw 标尺 / idlharness 资产补拉）；
  - 附带裁决项：clear/draw 标尺（建议接受 corpus 覆盖面即验收面，clear/draw
    挂账 M3）。
