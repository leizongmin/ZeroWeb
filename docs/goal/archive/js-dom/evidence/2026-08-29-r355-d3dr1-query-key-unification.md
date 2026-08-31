# R355 — d3d-r1 产物归一路径统一：`_zwQueryKey` 归一缓存键统一 helper

**日期**: 2026-08-29
**切片**: RFC v0.3 §6.2 路线 A 首片（d3d 重启前置之一）
**性质**: 纯重构行为等价（d3d-r1 验证门 = 全量双路径逐计数一致）

## 1. 现状（重构前）

四处键构造各自为政：

| 站点 | 形态 | empty-ns 剥离 | dup-seq |
|---|---|---|---|
| `_zwWrapCached`（doc 级 wrapper 缓存） | 双形态（`.tag`/`.tagName`，R170） | ❌ | ❌ |
| `_zwMWrapCached`（element 级 wrapper 缓存） | 单形态（`.tag`/`.outer`） | ✅（R307） | ✅（R188） |
| `_zwMFindRealNode` walk 键 | 真节点字段（`nodeName`/`outerHTML`） | ✅（R307） | —（数组分装） |
| Element QSA `_r188Seen` 重复检测 | 单形态 JSON | ❌ | —（计数器） |

键空间分裂使跨面归一依赖各处约定——R171 的 `:enabled` +2F（querySelector 与 QSA[0] 的
归一缓存键命中不同步）即此机制的实证，也是 d3d-r1 作为 d3d 重启前置的原因。

## 2. 改动

新增 `_zwQueryKey(info)`（part03，`_zwMFindRealNode` 前）为唯一键构造入口：

- **双形态入参**：`tag` 段读 `.tag`，空则回落 `.tagName` 小写（R170 语义）；outer 段读
  `.outer`，null 回落 `.outerHTML`。
- **empty-ns 剥离**（R307）：`data-zw-empty-ns=""` 标记段统一剥离——host JSON `.outer`
  从不含标记（剥离为无操作），真节点 `outerHTML` 含标记时键与 walk 键对齐（消一处
  键空间分裂，方向同 R307）。
- **dup-seq 后缀**（R188）：`info._zwDupSeq` 存在时追加 `\x1f#N`（仅 element 级 QSA
  设置；doc 级路径从不设置，行为不变）。

四个消费点全部改走 helper：`_zwWrapCached`、`_zwMWrapCached`、`_zwMFindRealNode` walk 键
（真节点字段以 `{tagName, id, outer}` 形态传入，outer 含 `_zwOuterFallback` 回落）、
Element QSA `_r188Seen`。

## 3. 行为等价性论证

- tag 段：host JSON `.tag` 恒小写（lowercase no-op）；真节点 `.tagName` 两版均小写；
  空串回落语义一致。
- outer 段：host `.outer` 不含标记 → 剥离 no-op；`_zwWrapCached` 的真节点入参
  （outerHTML 含标记）**此前键含标记、现被剥离**——与 walk 键对齐（该形态此前
  `_zwMFindRealNode(root, key)` 恒 miss 的键空间分裂点，属 R307 同向修复而非回归）。
- dup-seq：仅 element QSA 设置；doc 级不设置 → 分支不触发。
- `_zwMWrapCached` 的真节点入参路径今天不存在（`_zwMQueryAll` 产物恒 JSON），
  helper 的双形态读取仅为 d3d-r3 本树化的前置铺路。

## 4. 验证（d3d-r1 验证门）

- 全量 dom sweep（polyfill 路径，3300s guard）：**55480P/19F/16T vs R351 基线 55484P/
  21F/14T——真实 Fail 集合 17=17 已知集合恒等（零新增零丢失）**，Pass -4 为已知
  Timeout 轮转族（ParentNode-querySelector-All-content / query-target-in-load-event 等）。
- 文件级门：ParentNode-querySelector-All 1976P、Element-matches 675P、
  appendData 384P（三绿保持）、MO-attributes 42P——全部逐计数一致。
- make test 仅 XOpenDisplayFailed 环境项（预存）；fmt 无 diff；v8 全 workspace clippy
  `-D warnings` 零警告；quickjs 矩阵 clippy 零警告 + quickjs engine lib 1471 passed。

## 5. 下一步

d3d-r2（iframe 树源统一）——路线 A 第二前置；完成后 d3d-r3 本树化重启具备两前置。
