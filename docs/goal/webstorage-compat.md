# Web Storage 兼容 — localStorage / sessionStorage

**版本**: v1.0
**日期**: 2026-10-08
**状态**: Active（已立项，快赢切片——无用户门控项）
**执行模式**: WPT 驱动（上游 webstorage corpus 为验收标尺）+ 语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——存储面）

> **说明**
> 本文档是 ZeroWeb「Web Storage 兼容」专项目标执行契约。localStorage/sessionStorage
> 是真实网页的基础设施（偏好/主题/会话态）；zero-storage 早已实现存储本体，
> 但 JS 面语义从未对上游语料系统度量。
>
> **▶ 拆分动机（2026-10-08，GB-20261007 批复后快赢三连之一，用户「按照建议来」）**：
> ① 存储本体（zero-storage）与 web-components/storage 系 goal 均已归档收口，唯
> `webstorage/` corpus 从未立 goal；② 纯 JS API 面，无深结构、无渲染面交集，
> 预期短节奏收口。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：
> - **存储本体**：zero-storage crate 实现 localStorage/sessionStorage 本体
>     （crates/storage；storage-cache-api/storage-indexeddb/storage-opfs 三 goal
>     已归档，同 crate 面既有测试惯例可复用）
> - **JS 桥面**：window.localStorage/sessionStorage shim 存在性 M1 盘点确认；
>     getter/setter/clear/key/length/quota 异常语义未度量
> - **WPT corpus**：`webstorage/`（含 event/ 子目录——storage 事件为双 document
>     场景，runner 静态语料可执行性 M1 甄别）

---

## Mission

以 **WPT webstorage corpus 为验收标准**，收敛 localStorage/sessionStorage 的
JS 面语义（存取/枚举/配额/异常），使真实站点的偏好与会话态存储行为正确。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **轻量修复优先**：语义修齐走 shim/桥面，不动 zero-storage 存储本体核心
   （本体已有归档 goal 的测试资产守卫）
3. **事件面先甄别后推进**：storage 事件依赖双 browsing context，可执行性
   M1 定，若需 frame 树则与 navigation-compat M3 同门槛（用户门控）

覆盖范围：
1. **localStorage/sessionStorage 存取语义** — getItem/setItem/removeItem/clear/
   key/length、键序、quota 异常（QuotaExceededError）语义
2. **origin 隔离** — 同源判定与分区
3. **storage 事件**（甄别切片）— 事件触发条件/事件属性（key/oldValue/newValue/
   url/storageArea）

### 排除（明确不在范围内）

- **IndexedDB / Cache API / OPFS / StorageManager** — storage 三件套已归档
  goal 域（archive/storage-*）
- **cookie** — cookies-compat goal（本批同立项）
- **多 tab 实时同步** — 单进程静态 runner 无法真实模拟双 top-level traversable，
  事件面以 runner 可执行子集为口径

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | window.localStorage/sessionStorage 桥面语义 | part 系 JS 语义 |
| zero-storage | 存取/quota 语义对齐（本体核心不动） | 归档 goal 测试资产守卫 |
| WPT 资产 | webstorage corpus 导入 | fetch 脚本 + 账本回填 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 cookies-compat（本批同立）**：工作面零交集（storage vs cookie jar），
  排期可并行
- **与 rendering-compat**：零渲染面交集
- **与 navigation-compat M3**：storage 事件若触 frame 树 = 同一深结构，BLOCK
  并挂该 goal 门控，不单方面开

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 导入与基线

- [ ] webstorage corpus window 可执行子集导入（可执行性甄别记录落 evidence/）
- [ ] 分类通过率基线落 evidence/，并回填 docs/compat/trends/wpt-suites.csv planned 行

### DC-2: 存取与配额语义收敛

- [ ] getItem/setItem/removeItem/clear/key/length 上游语义对齐（异常类型/消息
      按 corpus 口径）
- [ ] origin 隔离与键序语义修正收口出数字（分级通过率不再下行两个连续轮次）

### DC-3: storage 事件（甄别切片）

- [ ] 可执行性甄别结论落 evidence/；可执行子集语义收敛；不可执行面记录挂账
      理由（frame 树依赖 → navigation-compat M3 同门槛）

### DC-4: 测试与质量不可退让

- [ ] 每个语义修复附带回归测试（单测或导入 WPT 常驻断言集，imported-tests.txt
      账本纪律）
- [ ] make test 全绿、make reftest 零回归、product-smoke welcome 逐字节恒值

---

## 活跃里程碑

### M1 — WPT 导入与基线

fetch 语料 + runner 通道（testharness 子命令 + per-dir skip + Makefile target）
+ 分类通过率基线 + csv 回填。

### M2 — 存取/配额/origin 语义收敛

DC-2 清单逐项修齐，每项独立提交附 A/B。

### M3 — storage 事件甄别切片 + 收口

DC-3 + 最终账本（分级通过率）+ 收口归档。
