# M4 R32 — Event.srcElement（target 的 legacy IE 别名）

**日期**: 2026-08-14
**轮次**: R32
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复）
**前置**: R31（dom_bindings coverage 提升 + DOMException name 缺省修复）
**状态**: ✅ 已 land（双路径对等，零回归）

---

## 背景

`Event.srcElement` 是 spec `dom-event-srcelement` 定义的 `Event.target` 的 legacy IE 别名。IDL：

```
[LegacyLenientThis] readonly attribute (EventTarget|null) srcElement;
```

getter 返回 `Event.target`（dispatch 前 target=null 故 srcElement=null；dispatch 期 target 已设故 srcElement=target）。这是 R21 失败聚类里「Event 对象缺属性」类（30 个用例之一）的具体一项，WPT 用例 `dom/events/event-src-element-nullable.html` 双路径此前 0-pass。

接手状态：上一轮 R32 在 429 rate-limit 打断时已写完实现 + native 单测 + fetch-dom-subset 幂等快路径，但未验证未 land。本轮（rollover）核对实现、补全验证、归档 land。

## 实现

### polyfill 路径（part03.js `_makeEvent`）

- `_makeEvent` 初始属性加 `srcElement: null` 占位（与其他公开镜像属性同款，仅作 for-in 可见性占位）
- 用 `Object.defineProperty(ev, 'srcElement', { enumerable:false, configurable:true, get: function(){ return this.target; } })` 定义 getter 读 `this.target`
- **为何用 getter 而非 data 属性**：dispatch 时 polyfill `_dispatchToListeners` 会更新 `ev.target`，getter 读 `this.target` 自动反映；若用 data 属性占位 null，dispatch 后读 srcElement 会得 null 而非 target（WPT `assert_not_equals(e.srcElement, null)` 会 fail）
- **setter 不定义**：spec srcElement 只读；赋值静默丢弃是 JS 默认行为（与 returnValue setter 需副作用的区别）

### native 路径（dom_bindings/event.rs + event_target.rs）

- `set_event_init`（event.rs）：`new Event` 时 `srcElement` 初始化为 null（与 target/currentTarget 同款 data 属性镜像，注释说明避免 prototype accessor 复杂度）
- `dispatch_event_impl`（event_target.rs）：dispatch 期 `srcElement` 与 `target` 同步设为派发目标（`this.into()`）
- native 用 data 属性而非 prototype getter——与 target 同生命周期，dispatch 期显式 set，避免 native prototype accessor 复杂度

### 辅助：fetch-dom-subset.sh 幂等快路径

`fetch_dir_html` 加幂等快路径：WPT_REV 固定 → 目录文件集稳定。若目录已含 .html 用例且未设 `FORCE=1`，跳过 GitHub Contents API 列目录步骤。

**动机**：此前每次 `make testharness-dom` 都调 GitHub Contents API 列目录，未认证调用触发 GitHub 60/h 速率限制 → 403 阻断 `make testharness-dom`。快路径避免无谓 API 调用；缺文件时 `fetch_raw` 自身 `-s` 跳过已存在；`FORCE=1` 强制重列拉最新文件集。

## 验证

| 门禁 | 命令 | 结果 |
|------|------|------|
| R32 native 单测 | `cargo test -p zero-engine --features v8 --lib native_event_src_element_r32` | ✅ 1 passed |
| engine v8 全量 | `cargo test -p zero-engine --features v8 --lib` | ✅ 2109 passed（基线 2107 +2，零回归） |
| engine quickjs 全量 | `cargo test -p zero-engine --no-default-features --features quickjs --lib` | ✅ 1411 passed（基线 1410 +1，零回归） |
| clippy v8 | `cargo clippy -p zero-engine --features v8 --all-targets -- -D warnings` | ✅ 零警告 |
| clippy quickjs | `cargo clippy -p zero-engine -p zero-wpt-runner --no-default-features --features quickjs --all-targets -- -D warnings` | ✅ 零警告 |
| fmt | `cargo fmt --all -- --check` | ✅ 无 diff |
| WPT polyfill | `make testharness-dom FILTER=event-src-element-nullable` | ✅ Pass |
| WPT native | `make testharness-dom-native FILTER=event-src-element-nullable` | ✅ Pass |

**WPT 用例结果**：`dom/events/event-src-element-nullable.html` 双路径 0P → **全 Pass**（srcElement 此前完全未实现）。

## 决策记录

- **getter vs data 属性（polyfill）**：选 getter 读 `this.target`，因 polyfill dispatch 更新 target 而 srcElement 须同步反映；data 属性占位会在 dispatch 后读 null fail。
- **native 用 data 属性而非 getter**：native dispatch 显式 set target，srcElement 同步 set 即可，与 target 同生命周期，避免 native prototype accessor 复杂度（与 polyfill 取舍不同，因 native dispatch 路径已显式写 target）。

## 净影响

- DC-3（WPT dom 基线）：dom/events 双路径各 +1 用例（srcElement 用例从 0→全 pass）
- DC-4（A/B 对照）：polyfill vs native 双路径行为等价（srcElement null 初始 + dispatch 期 === target）
- 辅助改进：fetch-dom-subset 幂等快路径消除 GitHub 60/h API 限流阻断（此前无人值守跑 testharness-dom 易被 403 阻断）
