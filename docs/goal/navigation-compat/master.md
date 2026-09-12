# 导航与浏览上下文兼容 — 运行时控制面板（master.md）

**入口文档**: [../navigation-compat.md](../navigation-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-12（立项）

---

## 当前状态

**专项定位**：会话历史/导航事件（轻面，独立推进）+ iframe 浏览上下文（深面，
**用户门控切片**，先例 R1043/Phase A IFC）。轻面可不等门控独立收口出数字。

**与兄弟 goal 的边界**：
- rendering-compat — viewport/滚动/渲染面不碰（style-system/layout-engine/render-foundation
  不在 envelope）；iframe 渲染面缺口记账回流
- event-loop-spec（已归档）— IO/RO 与事件循环遗产为消费基础
- zero-web P1a — location 读侧已落；写侧导航语义归本 goal
- zero-protocol / 多进程 — fission 排除；M3 触进程模型即 BLOCK 上报用户

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 四 corpus（html/browsers + history + navigation-api + iframe）导入 + 基线 | ⏳ M1 纯资产 |
| P2 | history pushState/replaceState/state/length/back/forward/go 语义 | ⏳ M2 |
| P3 | popstate/hashchange 事件序 + location 写侧导航语义（带重入 guard） | ⏳ M2 |
| P4 | iframe 浏览上下文最小面（contentWindow/frames/parent/top + 属性语义） | ⏳ M3 **用户门控** |
| P5 | bfcache / fission 挂账定稿 | ⏳ M4 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：四 corpus fetch 脚本 + 导入 + 基线（纯资产）+ suites CSV 回填
2. **M2**：history/导航事件逐簇修齐（不等 M3 门控）
3. **M3**：frame tree 最小面——**启动前须用户点名批准**

**待用户决策清单**：
- [ ] M3 iframe 深结构切片启动授权（未获批期间 DC-3 保持 pending，不阻塞 M2 收口）
