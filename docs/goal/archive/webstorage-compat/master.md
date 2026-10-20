# Web Storage 兼容 — 运行时控制面板（master.md）

**入口文档**: [../webstorage-compat.md](../webstorage-compat.md)
**创建日期**: 2026-10-08（goal 立项） | **最后更新**: 2026-10-11（M3 收口——goal Completed）

## 当前状态

**目标 Completed（2026-10-11，M3 收口判定）**：DC-1~4 全满足，可执行子集 25 案
**1251/1251 = 100%**（单日轨迹 94.9% → 97.6% → 99.0% → 99.4% → 100%，单调无下行）。
门禁：make test 分解逐腿全 0 failed（browser xvfb 腿三次静默终止经隔离复跑
LEG-EXIT=0 归因并行流争用，§10 归因纪律）+ make reftest 708/708 零回归 +
product-smoke welcome oracle diff 15.40% ≤ 20% + fmt/clippy 干净 +
zero-engine lib 2977P/0F。
判定文档：[evidence/2026-10-11-m3-close-verdict.md](evidence/2026-10-11-m3-close-verdict.md)。

里程碑台账：M1（10-11）通道 + 甄别 + 基线 94.9%；M2-P1（10-11）named property
语义 +34P；M2-P2（10-11）WebIDL arity/ToUint32 +18P；M2-P3（10-11）配额强制 +
QuotaExceededError 接口 + native DOMException 补 22 映射（共享路径根因）+4P；
M3（10-11）StorageEvent 接口保真 +8P → 100%。逐轮证据见 evidence/
（m1-baseline / m2-p1 / m2-p2 / m2-p3 / m3-close-verdict）。

**余量诚实口径**：100% 为可执行子集（25 案 `.window.js`）；上游全语料 56 案中
30 案排除面全分类挂账（见判定文档排除表），不宣称全语料 100%。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | corpus 通道 + 可执行子集甄别 + 基线 | ✅ M1（94.9%） |
| P2 | named property 写入/定义语义（Proxy trap 面） | ✅ M2-P1（→ 97.6%） |
| P3 | 方法缺参 TypeError + key() index 转换 | ✅ M2-P2（→ 99.0%） |
| P4 | 配额强制（QuotaExceededError + 独立性） | ✅ M2-P3（→ 99.4%） |
| P5 | StorageEvent 构造/init（可执行子集） | ✅ M3（1251/1251 = 100%） |
| P6 | storage 事件派发面（frame 树依赖） | ⬜ 挂账 navigation-compat M3 同门槛 |

## 后续重入条件（记账，非待办）

- navigation-compat M3（frame 树/多 browsing context 管道）落地 → storage 事件
  派发面 19 案 + window.open 双上下文 4 案 + 跨源 iframe 1 案 + 分区/document-domain
  4 案可随其管道一并基线（wpt-data 已拉取）。
- 数字维护：全量复跑 `make testharness-webstorage` 后向 `docs/compat/trends/
  wpt-suites.csv` 追加行（收口行 2026-10-11 = completed）。
- 配额值 5MiB/storage 为 UA 定义值近似（Chrome 口径 UTF-16 code units）；宿主
  侧 zero-storage 若引入真实配额策略，shim 计数器（`__zwBytes`）可切换为宿主桥。
