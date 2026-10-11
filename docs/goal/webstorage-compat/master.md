# Web Storage 兼容 — 运行时控制面板（master.md）

**入口文档**: [../webstorage-compat.md](../webstorage-compat.md)
**创建日期**: 2026-10-08（goal 立项） | **最后更新**: 2026-10-11（M2 三切片全部落地）

## 当前状态

**M1 完成 + M2 完成（2026-10-11）**：通道 = `make testharness-webstorage`
（`WEBSTORAGE_CASES` 固定清单 25 案 `.window.js` 可执行子集 + test-guard 包裹）。
轨迹：基线 1187/1251 = 94.9% → P1 1221（+34P，named property 语义）→ P2 1239
（+18P，缺参 TypeError + key() ToUint32 取模）→ P3 **1243/1251 = 99.4%**（+4P，
配额强制 5MiB/storage 独立 + QuotaExceededError 接口 + native DOMException 补
QuotaExceededError→22 共享路径根因）。DC-2 存取/配额语义收口，仅余 StorageEvent
构造/init 8 subtests（M3 面）。
可执行性甄别（DC-3 前置）：runner 可执行子集 = StorageEvent 构造/init 2 案；
event_*.html 派发族 19 案 + window.open/iframe 5 案挂账 frame 树依赖 →
navigation-compat M3 同门槛，不单方面开。
evidence：[m1-baseline](evidence/2026-10-11-m1-baseline.md) /
[m2-p1](evidence/2026-10-11-m2-p1-named-property.md) /
[m2-p2](evidence/2026-10-11-m2-p2-arity-key-index.md) /
[m2-p3](evidence/2026-10-11-m2-p3-quota.md)。

**卡点**：无（无用户门控切片；修复面全在 engine shim/bindings 层，不触
zero-storage 本体与深结构）。

**下一步（M3，DC-3 收口）**：
1. **StorageEvent 构造/init 语义收敛**（8 subtests：event_constructor 3/6 +
   event_initstorageevent 0/5）——StorageEvent 接口（constructor + initStorageEvent
   成员），DC-3 可执行子集全绿。
2. M2/M3 收口门禁：make test 全绿 + make reftest 零回归 + product-smoke welcome
   恒值（DC-4），随后最终账本 + 归档。

M3（DC-3 收口）：StorageEvent 构造/init 语义收敛（3/11 → 全绿）+ 分级通过率
最终账本 + 归档。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | corpus 通道 + 可执行子集甄别 + 基线 | ✅ M1（2026-10-11，94.9%） |
| P2 | named property 写入/定义语义（Proxy trap 面） | ✅ M2-P1（+34P → 97.6%） |
| P3 | 方法缺参 TypeError + key() index 转换 | ✅ M2-P2（+18P → 99.0%） |
| P4 | 配额强制（QuotaExceededError + 独立性） | ✅ M2-P3（+4P → 99.4%） |
| P5 | StorageEvent 构造/init（可执行子集） | ⬜ M3（3/11） |
| P6 | storage 事件派发面（frame 树依赖） | ⬜ 挂账 navigation-compat M3 同门槛 |

## 记账

- 数字维护：全量复跑 `make testharness-webstorage` 后向 `docs/compat/trends/
  wpt-suites.csv` 追加行（基线行 2026-10-11）。
- 配额用例（`while(true) setItem`）依赖 CASE_TIMEOUT 单案超时，配额落地前恒
  Fail 记账，不视作 runner 故障。
