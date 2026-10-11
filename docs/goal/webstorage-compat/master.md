# Web Storage 兼容 — 运行时控制面板（master.md）

**入口文档**: [../webstorage-compat.md](../webstorage-compat.md)
**创建日期**: 2026-10-08（goal 立项） | **最后更新**: 2026-10-11（M1 基线落档）

## 当前状态

**M1 完成（2026-10-11）**：DC-1 两项达成。通道 = `make testharness-webstorage`
（`WEBSTORAGE_CASES` 固定清单 25 案 `.window.js` 可执行子集 + test-guard 包裹）；
基线 **1187/1251 = 94.9%**（17 案全绿，storage_setitem 1106 子测试全绿）。
可执行性甄别（DC-3 前置）：runner 可执行子集 = StorageEvent 构造/init 2 案；
event_*.html 派发族 19 案 + window.open/iframe 5 案挂账 frame 树依赖 →
navigation-compat M3 同门槛，不单方面开。
基线 evidence：[evidence/2026-10-11-m1-baseline.md](evidence/2026-10-11-m1-baseline.md)。

**卡点**：无（无用户门控切片；M2 修复面全在 engine shim JS 层，不触 zero-storage
本体与深结构）。

**下一步（M2，DC-2）**：按失败聚类切片修齐，每项独立提交附 A/B 数字：
1. **P1 named property 写入/定义语义**（defineProperty 0/12 + set 8/20 +
   symbol-props 6/14 + storage_builtins 0/2，共 28 subtests）——shim Proxy
   defineProperty 拦截 + `Object.prototype` 成员遮蔽序 + builtins 不可覆盖。
   修在 `crates/engine/src/js_dom_shim/part02.js` Storage 块（R4875 域）。
2. **P2 方法缺参 TypeError**（missing_arguments 0/10）——WebIDL arity 校验。
3. **P2 `key()` 大 index 转换**（storage_key 14/22）——unsigned long modulo 2^32。
4. **P3 配额强制**（4 案 quota）——QuotaExceededError + local/session 独立
   （5MiB/origin 近似，UA 定义值）。

M3（DC-3 收口）：StorageEvent 构造/init 语义收敛（3/11 → 全绿）+ 分级通过率
最终账本 + 归档。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | corpus 通道 + 可执行子集甄别 + 基线 | ✅ M1（2026-10-11，94.9%） |
| P2 | named property 写入/定义语义（Proxy trap 面） | ⬜ M2（28 subtests） |
| P3 | 方法缺参 TypeError + key() index 转换 | ⬜ M2（18 subtests） |
| P4 | 配额强制（QuotaExceededError + 独立性） | ⬜ M2（4 subtests） |
| P5 | StorageEvent 构造/init（可执行子集） | ⬜ M3（3/11） |
| P6 | storage 事件派发面（frame 树依赖） | ⬜ 挂账 navigation-compat M3 同门槛 |

## 记账

- 数字维护：全量复跑 `make testharness-webstorage` 后向 `docs/compat/trends/
  wpt-suites.csv` 追加行（基线行 2026-10-11）。
- 配额用例（`while(true) setItem`）依赖 CASE_TIMEOUT 单案超时，配额落地前恒
  Fail 记账，不视作 runner 故障。
