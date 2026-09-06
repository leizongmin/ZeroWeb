# M3 Service Worker InstallEvent Type

**日期**：2026-09-02
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：complete；promoted to core baseline

## 变更

- `service-workers/service-worker/install-event-type.https.html` 从
  `defer-after-core` 提升到 core runner。
- Service Worker bootstrap 的 `ExtendableEvent` 初始化显式暴露
  `bubbles === false`，与已有 `cancelable === false` 一起满足 worker 内
  `InstallEvent` 类型断言。
- 新增独立 install-event-type asset manifest：
  [2026-09-02-m3-install-event-type-assets.tsv](2026-09-02-m3-install-event-type-assets.tsv)。

## 验证

- `make testharness-service-workers-core FILTER=install-event-type --always-make`：
  1 Pass

## 结论

core Service Worker baseline 从 40 case / 166 subtest 提升到 41 case / 167 subtest。
该切片复用已有 worker-testharness 结果通道，仅补齐 install event 基础 WebIDL 形态。
