# R309 Evidence — pending-fused 子树查询：innerHTML 替换后同 turn 查询正确（ParentNode-querySelector 全族 100%）

**日期**: 2026-08-27
**切片**: M4/L2——R309(b) removed-elements 1F 归因修复（连带 L2 同 turn 查询语义首片）
**改动面**: `part04.js`（sel-proxy querySelectorAll 的 pending-fused 重建）+ `part24.rs`（+1 单测）

## 一、成果

| 套件 | 基线（main = R308 后） | R309 | Δ |
|---|---|---|---|
| ParentNode-querySelectorAll-removed-elements | 0P/1F | **1P/0F** | +1P/-1F |
| **ParentNode-querySelector 全族** | 2053P/1F | **2054P/0F（全族 100%）** | +1P/-1F |
| Element-matches / webkitMatchesSelector / cloneNode / createElementNS / createElement / insertBefore / closest | 全 0F | 同 | 持平 |
| Node-properties | 724P/2F（既存） | 同 | 持平 |
| MutationObserver | 117P/4F（既存） | 同（set-diff 恒等） | 持平 |
| vue e2e（integration） | 3P | 3P | 持平（首版回归已修，见下） |
| engine 单测 --lib | 2446 | **2447** | +1 |
| make test | 1F 环境项 | 同 | 持平 |
| fmt / clippy | — | 干净 | — |

## 二、根因与修复

**用例**（jsdom#2519 回归）：`container.innerHTML = ...` 替换后同 turn
`container.querySelectorAll('a.test')` 须返新子树元素、不返旧元素。

**根因**：主文档域 sel-proxy 的 `querySelectorAll` 走 `__zw_query_all_sub(sel, q)`——
host 在 **dom_html 快照**上查询，同步 turn 内的 innerHTML 替换经 host 异步 apply，
快照恒 stale（新子树 miss）。R304 已解 childNodes/firstChild 的同 turn 视图（overlay +
挂父槽），但**子树查询**不经 overlay。

**修复**（part04 sel-proxy querySelectorAll 分支）：
- **判据（限 innerHTML 替换域）**：本 sel 的 pending 桶 added 里存在「innerHTML 解析
  wrapper 打挂父槽」形态（R304 `_zwSelPendingParent`，无 handle 无 sel）→ 本容器有同
  turn 整体替换（基底已被 R304 置空）。
- **重建**：`_childNodeList(sel)`（含 overlay 的 live 子列表）DFS × 客户端 compound
  匹配器（`_parseCompoundOf`/`_matchCompoundOf`——tag/#id/.class/[attr] 组合）重建
  结果；不支持形态（组合器/伪类 `unsupported`）回落 host 快照语义零变化。

## 三、首版教训（vue e2e 回归，已修）

首版判据 = 桶非空即重建——`vue_reconciliation_lands` 立即回归（`lis:A,B,A,B`
双计）：Vue mount 的 append（handle/proxy 子）走 overlay 时，**基底快照 wrapper 与
pending wrapper 是不同对象**（identity 双源），seen 检查按 identity 恒 miss → 重复。
收紧判据到「innerHTML 替换形态」（挂父槽 wrapper）后 vue e2e 恢复 3P、WPT 全绿。

**教训**：overlay 的 identity 双源（基底 wrapper vs pending wrapper）使「桶非空」
不是「快照 stale」的充分判据——innerHTML 整体替换（基底置空）才是无歧义域；
普通 append 的同 turn 查询语义留待 L2 主线统一方案（基底+overlay 的融合查询去重）。

## 四、L2 关联

本切片 = **L2「查询读 live」在主文档域的第一片**（innerHTML 替换域）。R309(a) 原计划
的 getElementById live 读已有既有 pending-fused 基建（part06 getElementById 的
pending 消费，R125 in-document 门）——主文档域查询语义的剩余面：
1. 普通 append（handle/proxy 子）的同 turn 查询（identity 双源去重——需 L2 统一方案）；
2. removed 语义的 host 快照滞后（bucket.removed 过滤 host 结果——轻量候选）。
