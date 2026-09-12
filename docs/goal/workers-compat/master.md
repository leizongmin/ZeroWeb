# Worker 并发兼容 — 运行时控制面板（master.md）

**入口文档**: [../workers-compat.md](../workers-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-12（立项）

---

## 当前状态

**专项定位**：dedicated worker 从构造器桩（dom_bridge.rs:1839）升级为真实运行时
（独立隔离实例 + 事件循环 + 全局作用域）+ postMessage 往返语义。消息底座
（MessagePort/structuredClone）P1a 已落，只差隔离执行实例。

**与兄弟 goal 的边界**：
- zero-page-runtime — worker 实例归属/消息通道**先对契约再动协议**（§9）
- script-sandbox / wasm-sandbox — 隔离组织范式复用，其对外宿主桥接边界不碰
- event-loop-spec（已归档）— 微任务机制先例复用，遗留问题记账回流
- rendering-compat 及渲染流 — 无共享 crate 面

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | workers/dedicated-workers corpus 导入 + 基线 | ⏳ M1 纯资产 |
| P2 | 与 zero-page-runtime 的 worker 契约协商（实例归属/消息通道） | ⏳ M2 前置 |
| P3 | 独立隔离实例 + 独立事件循环 + 全局作用域最小面 | ⏳ M2 |
| P4 | classic 脚本执行 + importScripts；module worker 评估 | ⏳ M2 |
| P5 | postMessage/structuredClone 往返 + transferable（ArrayBuffer transfer） | ⏳ M3 |
| P6 | SharedWorker / nested worker 挂账定稿 | ⏳ M4 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：两 corpus fetch 脚本 + 导入 + 基线（纯资产）+ suites CSV 回填
2. **M2 前置**：zero-page-runtime 契约协商（不单方面改协议）
3. **M2-M4**：按入口文档里程碑推进；v8/quickjs 双矩阵差异如实记账

**待用户决策清单**：（空——启动顺序由用户点名）
