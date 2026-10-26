# M2-s82：handle 形元素 closest 读面（R5282）

日期：2026-10-25 ｜ 切片：s82 ｜ 提交：R5282 ｜ 前序：[s81](2026-10-25-m2s81-query-id-rename-visibility.md)

## 1. 问题

`document.createElement` 产物是 handle 形代理：part06 主路径走 `__zw_create_element` host 调用 + `_wrapHandle(handle)`，`__zwSelector` 恒 null，身份永驻 handle 形（并非同 turn 窗口问题）。closest 入口的 `if (!sel) return null` 门使其**永久盲**——已 append 进文档、排水后依然 null。matches 已有 R134/t6 锚回臂（part04），closest 完全无 handle 分支。

## 2. 分诊

探针矩阵（临时 probe，验证后删除）：`matches('span')`=true（t6 臂）、`matches('#anc span')`=false（complex stale 缺口）、`closest('span'/'div'/'body')`=null ×3、文档级 getElementById/qS 正常（R322/R368 面）。

语料需求核查：117 文件 F 清单 × createElement（39 处）× closest/matches 共现 = 0——**没有语料 F 面直接要求本切片**。立项依据：DOM spec closest 定义（祖先轴匹配）+ 真实站点 slice（jQuery/Swiper 对创建元素的 `closest` 定位）。

范围决策：s82 只接 handle 形 closest；matches complex stale 面（t6 锚回 host 快照 miss）记遗留，不在本片建——需新增 Rust 视图文档 matches 回调或 query_all 回退，非平凡。

## 3. 修复

双臂（part05 `_zwHandleClosest`，part04 closest 入口 `!sel && handle` 分支）：

1. **R100 锚回臂**：`_r100SelOfHandle(handle)` 拿 sel 锚（connected 且曾 sel 向解析过）→ host `__zw_closest(anchor, q)` 全语义（complex/伪类活）。同 turn append 的快照 stale 窗口锚未命中，落 ②。
2. **JS 单 compound 祖先走**：`_zwClosestSingleCompounds` 只接单 compound 无伪类组（`_matchCompoundOf` 不求值 pseudos，混入会 over-match，保守排除）；父链双形态解析（`_zwClosestParentOfPx`）：sel 形走 `__zw_parent` 视图文档，handle 形走 `_zwNodeParent` 反链；nodeType≠1 即停（fragment 容器 11 → null，不越 shadow 边界——spec closest 祖先轴不含 shadow host）；48 跳封顶（R125 口径）。

## 4. 验证

- 契约测试 `handle_element_closest_s82`（part90，15 断言）：(a) sel 父 append 五面 + (b) s81 改名面组合（lw 现值命中/旧 id stale）+ (c) handle 父链双形态混合爬链 + (d) detached（self 命中/上行 null）+ (e) complex/伪类保守 null。
- 探针 4 面 Pass（exit 0）后删除。
- engine lib：3521P + 新增 1T → 3522P / 0F（test-guard 守卫跑）。
- 域（FILTER=embedded-content）：976P/252F/30T 持平，REG 0。
- pin（FILTER=interactive-elements）：439P/44F/0T bit-exact。
- fmt clean；clippy 仅 part90.rs:47 既有 concat! 警告（s78 引入，不在本 diff）。

## 5. 遗留

- matches complex stale 面：handle 形元素 `matches('#anc span')` 同 turn false（锚回 host 快照）——语料 0 需求，账本挂账。
- `_matchCompoundOf` 忽略 pseudos 的 over-match 防线依赖调用方预过滤（`_zwClosestSingleCompounds` 已过滤；后续调用方须沿用）。
