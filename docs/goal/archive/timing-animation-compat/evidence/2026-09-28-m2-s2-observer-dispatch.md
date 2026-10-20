# M2-S2 — PerformanceObserver 异步派发与 observe 语义修齐

**日期**: 2026-09-28
**通道**: `make testharness-performance-timeline` / `make testharness-user-timing`
**原始数据**: 2026-09-28-m2-s2-*.json（本轮 gates 落盘）

## 结果（M1 基线 → M2-S2）

| corpus | M1 基线 | M2-S1 后 | M2-S2 后 |
|---|---|---|---|
| performance-timeline | 19P/16F/9T（44）| 20P/15F/9T | **35P/5F/4T（44）** |
| user-timing（observer 面）| Timeout ×2（buffered-flag / supported-usertiming-types）| Timeout ×2 | **全绿（522P/6F/2T）** |

## 修齐的语义（part01b.js PerformanceObserver）

| 缺口 | 修法 | 解锁 |
|---|---|---|
| callback 只传 `(entries)` | spec 签名 `cb(entries, observer, options)` + `this = observer`（探针定位：回调内 `observer.disconnect()` 取 undefined → 全族 Timeout）| buffered-flag / supported-usertiming-types / po-callback-mutate 等 11 案 Timeout 簇 |
| `PerformanceObserverEntryList` 无接口 | global 接口对象（Illegal constructor + prototype 方法 + toStringTag），entryList 以之为基础创建 | po-observe instanceof/this 断言 |
| `observe({type, buffered: true})` 不回放 | observe 时回放 buffer 中**精确匹配 type** 的既有 entries（大小写敏感）| buffered-flag-after-timeout / multiple-buffered-flag-observers / po-observe-type 等 |
| `observe({})` 静默 | 无 type/entryTypes → TypeError；type+entryTypes 同传 → TypeError；entryTypes 传 string → TypeError；空序列不抛 | po-observe-type / po-observe |
| 形态切换不校验 | type 形态 ↔ entryTypes 形态切换 → InvalidModificationError DOMException | po-observe-type ×2 |
| observe 过滤语义一刀切 | **type 形态叠加**（different type values stacks）/ **entryTypes 形态替换**（replace observer）——队列内已排队 entries 不清 | po-observe-type async（stacks）+ po-callback-mutate（链式改过滤）|
| 派发批不排序 | flush 时按 startTime 升序 stable 排序（spec：observer queue sorted）| po-entries-sort |
| mark 后同步 disconnect 仍回调 | disconnect 取消 pending flush（`_pending` 闸门）| po-disconnect「disconnected after a mark」|
| 无第三实参 options | `{droppedEntriesCount: 0}`（本引擎无 resource buffer 丢弃恒 0）| droppedentriescount 首案路径 |

## 余非 Pass（全分类）

| 项 | 分类 |
|---|---|
| case-sensitivity ×2F（resource entries 断言）| resource-timing 排除域（M4 挂账）|
| droppedentriescount ×2F + 1T（`setResourceTimingBufferSize`/dropped 计数）| resource-timing 排除域 |
| po-observe.html 1T / po-resource 1T（等 `resource` entry——img 加载入 resource buffer）| resource-timing 排除域 |
| navigation-id-initial-load 1T（navigationId）| navigation-timing 排除域（M4 挂账）|
| webtiming-resolution 1F（连续 now() 差 3.7μs < 5μs 分辨率下限）| **M2-S3**：now() 粗化语义（timing-attack 100μs 网格化与 5μs 下限同簇）|

## 下一步（M2-S3）

hr-time——`performance.toJSON` + Performance EventTarget 继承 + **now() 粗化网格**
（100μs 非隔离网格使 webtiming-resolution 与 timing-attack 同簇闭合）+ timeOrigin
epoch 锚定 + clamped-time-origin Timeout 拆解。
