# WebGL 兼容 — 运行时控制面板（master.md）

**入口文档**: [../webgl-compat.md](../webgl-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-09-12（立项）

## 当前状态

**远期门控 goal**：M1（corpus 导入盘点）是唯一自主切片；M2 起每切片须用户点名，
无人值守循环遇边界输出 `CONTINUE: 等待用户门控` 转其他面。无 GL 语义层现状，
GPU 栈依托 render-foundation（wgpu+WGSL，M7 全 13 图元），不另起 wgpu 实例。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | webgl/ corpus 导入 + 用例分布盘点（dehydrated/渲染断言占比） | ⏳ M1 纯资产 |
| P2 | 首批收敛子集建议书提交用户 | ⏳ M1 产出 |
| P3 | WebGL 1.0 最小上下文（创建/clear/draw）——**门控** | 🔒 M2 |
| P4 | 缓冲/纹理/shader 程序管线——**门控**（切片边界用户定） | 🔒 M3 |
| P5 | WebGL2/WebGPU 挂账定稿 | ⏳ M4 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：webgl/ corpus fetch + 盘点（goals/99 编号脚本可跑 fetch 步）+ 建议书 +
   suites CSV 回填；完成后转等待门控态

**待用户决策清单**：
- [ ] M2 最小上下文切片启动授权（M1 建议书提交后）
