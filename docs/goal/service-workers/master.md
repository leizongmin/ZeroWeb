# Service Worker 真实化 — 运行时控制面板（master.md）

**入口文档**: [../service-workers.md](../service-workers.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-17（立项——M0 选型 RFC 待启动）

---

## 当前状态

**专项定位**：存储方向三拆之三（唯一带启动门控的）。把 `navigator.serviceWorker` 从注册表
状态机近似（R3318）深化为真实 SW 执行环境 + fetch 拦截。**M0 选型 RFC 须用户批准后才动
源码**；M0 期间可自主推进 WPT 可执行面分析与 RFC 起草。

**与兄弟 goal 的边界**：
- [storage-indexeddb](../archive/storage-indexeddb.md)（已归档）/ storage-cache-api —
  IDB 与 Cache API 自身语义归其管；本目标只消费
  `indexedDB`/`caches` 接口做 SW 模式集成验收
- js-dom — fetch 拦截段**等其 fetch 改造（L2/S6）land 后再开**；生命周期段碰 part02.js
  R3318 段前先 `git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ 注册 API 面：R3318（part02.js:2369）——register/getRegistration/getRegistrations/
  ready/unregister + scope 派生 + oncontrollerchange + installing/waiting/active 经
  setTimeout(0) 逐态推进
- ✅ Rust 状态机：`crates/storage/src/service_worker.rs`（818 行）——
  ServiceWorkerRegistry register/unregister/state/scope 匹配 + 单测
- ⚠️ register 的 scriptURL **不被下载执行**——SW 事件处理器无从注册
- ⚠️ fetch 拦截为零；install/activate 为 setTimeout 模拟非真事件
- ⚠️ WPT `service-workers` 未导入，无基线
- ⚠️ SW 执行环境架构未选型（M0 门控项）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| S1 | SW 执行环境架构未选型（深结构，须 RFC + 用户批准） | 🔄 M0（当前活跃） |
| S2 | scriptURL 不下载执行 | ⬜ M1 |
| S3 | fetch 拦截为零 | ⬜ M2（等 js-dom fetch 改造） |
| S4 | 事件为 setTimeout 模拟 | ⬜ M1 |
| S5 | WPT 覆盖为零 | ⬜ M0 期间可先做可执行面分析 |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | SW 执行环境选型 RFC（独立 V8 context / 独立线程 / 复用 Worker 基建） | ⬜ RFC 起草中——批准后解锁 M1+ |

## 下一步计划

1. **M0 切片 1**：WPT `service-workers` 可执行面分析（哪些用例当前环境能跑——零源码改动）
2. **M0 切片 2**：候选架构调研（工程量/风险/事件循环集成面对比）
3. **M0 切片 3**：RFC 起草 → 提交用户审批（**批准前不动源码**）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 选型 RFC（门控） | 🔄 待启动（当前活跃） |
| M1 — 脚本真实执行 + 生命周期真事件 | ⬜ 门控：RFC 批准 |
| M2 — fetch 拦截 + Cache 集成 | ⬜ 门控：js-dom fetch 改造 land |
| M3 — 控制语义 + 消息 + 收尾 | ⬜ |

## 验证基线

- 测试基线：storage crate 既有单测全绿（立项时点）；clippy 零警告
- WPT service-workers 面：无基线（未导入）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
