# M4 R16 切片 — classList toggle no-op + write runUpdate + replace 顺序

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**前置**: R15（implementation.createDocumentType）
**commit**: 见 `git log`（feat(js-dom): classList toggle no-op + write runUpdate + replace order）

## 背景

R13 classlist 去重后剩 60 失败。聚类：toggle 25（force 无变化错误规范化）、replace 20（顺序+同名 mutation）、remove 10、assign 5。

## 改动（part03 `_classListProxy`）

1. **write runUpdate 比较**：比较新集合序列化 vs 原 attribute 原始值，相同不 setAttribute（MutationObserver 依赖）。add/remove/replace 总经此——原值含尾空格/重复时规范化重写。
2. **toggle force no-op**：force 与现状一致 → 直接 return 不 write（保持 attribute 原样）。仅状态冲突时修改 + write。
3. **replace 顺序 + 同名**：oldT===newT 存在 → runUpdate；replace 在 oldT 位置换 newT + 移除后续重复（有序去重）。

## 基线（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R15 | R16 | Δ |
|------|----|----|---|
| polyfill | 52.11% | 53.00% | +0.89pp |
| native | 51.84% | 52.73% | +0.89pp |

classlist 用例 1360P/60F → 1400P/20F（+40）。双路径对等差 0.27pp。

## 验证

engine v8 2086 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 剩余（classlist 剩 20F）

replace(" ","") 异常名 + classList assignment setter + 个别边缘。

## 下一步

createEvent 剩 15F + event target null / createElementNS 大小写 / classlist 剩 20F / iframe.contentDocument（深结构）。
