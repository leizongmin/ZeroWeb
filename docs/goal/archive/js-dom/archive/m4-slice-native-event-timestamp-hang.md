# M4 Slice R22 — native Event.timeStamp 死循环修复 + native dom/events 基线

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R22
**前置**: R21（导入 dom/events，polyfill 基线 31.61%，误报 "native events dispatchEvent hang"）

## 问题（更正 R21 误判）

R21 报告"native dom/events dispatchEvent hang（native_dom=1 下卡死，根因疑 dispatchEvent）"。R22 深入诊断**推翻此结论**：

1. **诊断插桩**（native dispatchEvent + polyfill __zw_parent 加 eprintln）：两者**0 调用**——用例 element 走 polyfill dispatch（R9：document 是 polyfill），native dispatch 未触发。
2. **直接 binary 跑**（绕开 cargo run 编译开销 + test-guard）：单用例 `Event-dispatch-click` native **10P/27subtests** 正常完成（10.1s = polyfill 同样耗时）。
3. **逐用例 timing 扫描**（60s cap）：10 个慢用例 ~10s（命中 CASE_TIMEOUT 正常超时），**仅 1 个真死循环**：`Event-timestamp-safe-resolution`（>60s）。

**真根因**：`Event-timestamp-safe-resolution.html` 的 `do { e2.timeStamp - e1.timeStamp } while (delta==0)` 收集非零时间戳样本。**native `Event.timeStamp` 恒为 0**（event.rs:188 硬编码 `v8::Integer::new(scope, 0)`，注释"沙箱无 perf timer，暂 0"）→ 连续 `new MouseEvent()` 时间戳相同 → 差恒 0 → do-while 死循环 → 拖垮 native dom/events 全量（>570s 超时）。

**polyfill 不 hang**：polyfill `_makeEvent` 的 timeStamp 用 `__zw_performance_now()`（真实单调计时器，callbacks.rs:42 `Instant::elapsed`）→ 连续创建有非零差 → do-while 正常退出（用例快速 assert 失败返回）。

## 修复

native `Event.timeStamp` 改用单调 perf time（spec DOMHighResTimeStamp），对齐 polyfill（`crates/engine/src/dom_bindings/event.rs`）：

- **`perf_time_origin()`**：模块级 `OnceLock<Instant>`，首次构造 Event 时懒初始化（线程安全 lazy init）。
- **`perf_now_ms()`**：`origin.elapsed().as_secs_f64() * 1000.0`（ms，子毫秒精度）。
- **`set_event_init` timeStamp**：`v8::Integer::new(scope, 0)` → `v8::Number::new(scope, perf_now_ms())`（f64——spec 要求子毫秒精度，5µs 分辨率断言；Integer 丢精度）。

**设计**：不要求与 polyfill perf_origin 完全一致——spec 仅要求单调 + 连续创建非零差（解锁死循环 + 合规）。`OnceLock<Instant>` lazy init 线程安全；`Instant::elapsed` 无锁纯读。

## 验证

- **单测** `native_event_timestamp_monotonic_nonzero_r22`（tests.rs）：① timeStamp>0 + Number.isFinite；② 连续 new MouseEvent 差值可收集非零（模拟 WPT do-while，限 1e4 迭代防爆——旧恒 0 时永不退出）。v8 pass。
- **fmt + clippy 双矩阵**：zero-engine v8 + quickjs 零警告。
- **hang 消除**：`Event-timestamp-safe-resolution` native **>60s hang → 0s 完成**（exit 1 = 用例失败但快速返回）。
- **native dom/events 全量**（完整 JSON 入 evidence）：**97P/213F/9timeout，319 subtest，81 cases，pass/(pass+fail)=31.29%，96s**（从 570s 超时 → 96s 完成）。

  **双路径对等**：

  | 路径 | Pass | Fail | Timeout | pass/(pass+fail) |
  |---|---|---|---|---|
  | Polyfill | 98 | 212 | 9 | 31.61% |
  | Native | 97 | 213 | 9 | 31.29% |
  | 差 | -1 | +1 | 0 | 0.32pp |

  双路径基本对等（差 0.32pp，个别用例 native/polyfill 边缘）。

## 决策记录

- **为何 R21 误报 hang**：R21 用 `cargo run --release` 经 test-guard 包裹，cargo 反复编译 + test-guard 时间限制叠加，且未隔离单用例，把"1 个用例死循环拖垮全量"误判为"dispatchEvent 系统性 hang"。R22 经诊断插桩 + 直接 binary + 逐用例 timing 三步精确定位到单用例 `Event-timestamp-safe-resolution`。
- **为何 timeStamp 用 Number 非 Integer**：spec DOMHighResTimeStamp 是子毫秒 f64（5µs 分辨率断言依赖）。Integer 丢精度会让 `Math.round((e2-e1)*1000)` 对微小差值返 0，可能仍在边缘死循环。Number 保 f64 精度。
- **`make testharness-dom-native` 仍慢但可跑完**：10 个慢用例 ~10s each（命中 CASE_TIMEOUT=10s 正常超时），全量 ~96s。这些用例本身做大量事件分发循环（非缺陷），CASE_TIMEOUT 是合理上限。R22 仅修死循环，未改 CASE_TIMEOUT。

## 残留（转 R23+）

- polyfill/native dom/events 各 ~213 fail / 54 个 0-pass 用例（Event 对象属性 timeStamp 高分辨率断言 / 三阶段分发语义 / EventListener handleEvent）——R23+ 聚类驱动修复。
- `Event-timestamp-safe-resolution` 本身仍 fail（断言 5µs 最小分辨率——需更高精度 perf timer，非死循环问题）。
- dom/nodes 双路径差 0.65pp（native namespaceURI 独立化，低优先级）。
