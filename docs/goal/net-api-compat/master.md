# 网络 API 兼容 — 运行时控制面板（master.md）

**入口文档**: [../net-api-compat.md](../net-api-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-12（立项）

---

## 当前状态

**专项定位**：网络 API（fetch/XHR/URL/mimesniff/streams/EventSource）一致性收敛，
WPT 六 corpus 为验收标尺。WebSocket 二期挂账（需宿主 socket 面 + 帧协议）。

**与兄弟 goal 的边界**：
- security-hardening — CSP 对 fetch 的策略执行归其；本 goal 提供语义钩子位
- service-workers（已归档）— 其 fetch 通道已收口；拦截扩展另行记账
- zero-web P1a — URL/URLSearchParams 既有实现为本 goal 修齐对象（改动走本 goal 账本）
- rendering-compat 及渲染流 — 无共享 crate 面

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 六 corpus fetch 脚本 + 导入 + 基线 | ⏳ M1 纯资产 |
| P2 | fetch/Request/Response/Headers/Body 语义收敛 | ⏳ M2（预期最大簇） |
| P3 | XHR 状态机 + EventSource 解析/重连 | ⏳ M3 |
| P4 | URL 边缘语义 + mimesniff 对齐 | ⏳ M3 |
| P5 | streams 底座一致性（fetch body 依赖） | ⏳ M4 |
| P6 | WebSocket 二期切片（宿主 socket + 升级握手/帧协议） | 🚫 挂账，用户点名重入 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：六 corpus fetch 脚本（照 observers/fs 先例）+ 导入 + 通过率基线
   （零源码改动纯资产）+ suites CSV planned 行转数据行
2. **M2-M4**：按入口文档里程碑逐面收敛

**待用户决策清单**：（空——启动顺序由用户点名）
