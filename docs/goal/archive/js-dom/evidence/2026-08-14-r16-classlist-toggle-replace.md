# R16 — classList toggle no-op + write runUpdate 规范化 + replace 顺序（M4 / DC-3）

**日期**: 2026-08-14
**轮次**: R16
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**commit**: 见 `git log`（feat(js-dom): classList toggle no-op + write runUpdate + replace order）

## 背景

R13 classlist 去重后剩 60 失败。聚类：toggle 25（force 无变化时错误规范化重写）、replace 20（顺序 + 同名 mutation）、remove 10、indexed/assign 5。

## 改动（part03 `_classListProxy`）

### 1. write() 加 runUpdate 比较（spec DOMTokenList update 算法）

write 比较「新 token 集合序列化（单空格分隔）」与「原 attribute 原始值」，相同则不 setAttribute（避免无谓 mutation；MutationObserver 检查依赖此）。add/remove/replace 总经此——即使 token 集合不变，原值含尾空格/重复时仍规范化重写（WPT checkAdd("a b c ",["a","a"],"a b c")）。

### 2. toggle force 分支 no-op（spec toggle(token, force)）

force 与现状一致（on 且已在 / off 且不在）→ **no-op，不触发 update**（直接 return，不 write，保持 attribute 原样）。WPT checkToggle("a a a  b","a",true)→保持原样 "a a a  b"（非规范化 "a b"）。仅状态冲突时修改 + write。

### 3. replace 顺序 + 同名 runUpdate（spec dom-domtokenlist-replace）

- oldT===newT 且 oldT 存在 → 返 true + runUpdate（规范化 attribute，WPT checkReplace("a","a") with "a a a  b" 期望 mutation）。
- replace 在 oldT 位置替换为 newT，移除 newT 后续重复（有序去重，保留 index i）。WPT checkReplace("c b a","c","a")→"a b"（a 占 c 的 index 0，原 a 去重）。

## 基线结果（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R15 | R16 | Δ |
|------|----|----|---|
| polyfill | 52.11% | **53.00%** | +0.89pp |
| native | 51.84% | **52.73%** | +0.89pp |

双路径对等差 0.27pp。**classlist 用例**：1360P/60F → **1400P/20F**（+40 pass）。完整 JSON 快照入 evidence。

## 验证

engine v8 2086 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 剩余（classlist 剩 20F，下轮或独立）

- replace(" ","")（5×多 node）：newT="" 的 assert_throws_dom 异常名细节。
- classList assignment unchanged（5×多）：classList setter（赋值 no-op）。
- 个别 toggle/replace 边缘。

## 下一步

- createEvent 剩 15F + event target null / createElementNS 大小写。
- classlist 剩 20F 边缘。
- iframe.contentDocument（深结构 html-compat 域，待评估）。
