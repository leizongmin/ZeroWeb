# M2-S3 — hr-time 语义修齐（toJSON / EventTarget / 粗化 / timeOrigin）

**日期**: 2026-09-28
**通道**: `make testharness-hr-time`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-28-m2-s3-hr-time.json](2026-09-28-m2-s3-hr-time.json)

## 结果（M1 基线 → M2-S3）

| corpus | M1 基线 | M2-S3 后 |
|---|---|---|
| **hr-time** | 7P/6F/2T（15）| **10P/3F/2T（15）** |
| user-timing | 241P/108F/5T（354）| **561P/8F/1T（570）**（M2-S1/S2 叠加 + timing 桩解锁 measure.html ×36 + measure_exceptions_navigation_timing 4P）|
| performance-timeline | 19P/16F/9T（44）| **36P/4F/4T**（webtiming-resolution 随粗化网格转绿）|
| web-animations | 52P/901F/10T（963）| 52P/901F/10T（零回归——M3 面）|
| **合计** | 319/1376（23.2%）| **659/1592（41.4%）** |

## 修齐的语义（part01b.js performance 面）

| 缺口 | 修法 | 解锁 |
|---|---|---|
| `performance.toJSON` 缺失 | toJSON()（timeOrigin/timing/navigation）+ Navigation Timing L1 桩——21 属性 unavailable=0、`navigationStart` 为**现场读 timeOrigin 的 getter**（epoch ms，与 `new Date()` 同 timebase）、navigation {type:0, redirectCount:0}；用例断言全为 json↔live 自洽比较 | performance-tojson 全绿 + user-timing measure.html「no start」duration 面 + measure_exceptions_navigation_timing（timing 属性归位后 setup 通过，4P）|
| Performance 未继承 EventTarget | addEventListener/removeEventListener/dispatchEvent（监听表存取；spec 无 performance 事件源）| basic.any「Performance interface extends EventTarget」|
| now() 分辨率无下限 | `_perfNow` 粗化到 **0.1ms 网格**（floor 保单调；hr-time §privacy-security 非隔离 100μs 推荐分辨率）| webtiming-resolution（≥5μs 下限）转绿；timing-attack（100μs）保持绿；monotonic/正值不受影响 |
| timeOrigin = 0（相对原点）| **惰性 getter + 首访问缓存**（`Date.now() - now()`，epoch ms 锚定）——静态 init 曾被 shim-init→用例执行的无界间隔击穿 30ms 容差；首访问与用例断言同 tick | timeOrigin.html 首子测试（Window timeOrigin ≈ Date.now()）|

## 余非 Pass（全分类）

| 项 | 分类 |
|---|---|
| clamped-time-origin-isolated / cross-origin-isolated-timing-attack ×2F | `crossOriginIsolated` = false——COOP/COEP 隔离上下文 infra（安全/网络域跨域记账）|
| raf-coarsened-time 1F（`DocumentTimeline is not defined`）| WAAPI timeline 构造器——**M3** |
| timeOrigin.html / clamped-time-origin.html 各 1T（`new Worker(blob)` 子测试）| worker 执行面——workers-compat 流域（goal 排除）|
| user-timing ×8F / ×1T、performance-timeline ×4F / ×4T | 与 M2-S2 后分类一致：L1/L2 legacy 上游同失败 + resource/navigation-timing 排除域 |

## 下一步（M3）

WAAPI——Animation/KeyframeEffect/DocumentTimeline 构造器 + effect 属性桥（web-animations
52/963，`KeyframeEffect is not defined` ~370 / `effect` 缺失 ~256 / `Animation is not
defined` ~64 三主簇）→ 关键帧参数解析校验 → ready/finished promise（headless 帧精度
如实标注）。
