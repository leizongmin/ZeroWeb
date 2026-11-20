# d42：sel 方言归一刀 A/B 判决（暂停）+ 重映射序守卫落树

日期：2026-11-16。前置：d41 B3R 破案（`__zwSelector` 短形 `div` ↔ 路径形
`html:nth-child(1) > body:nth-child(2) > div:nth-child(1)` 双方言 → cache 双键
双 proxy → harness 身份检查误判换代 → remove 不中 → listener 遗留 → got=2 双派）。
本轮 = 归一刀实现 + A/B 判决 + 守卫固化。探针/运行产物均在 /tmp（gitignored）。

## 归一刀实现面（已回滚，仅存判决记录）

- 桥侧：`js_dom_bridge.rs` + `js_dom_bridge/callbacks.rs`——`__zw_query_match`
  hit 的 sel 命名归一（query 视图树节点 sel 字段生成点统一方言，d41 步骤①：
  短形 = live 视图 `unique_selector_for_node_ctx` 同视图唯一即短形、路径形 =
  快照 re-parse 视图或宿主静态方言）。
- shim 侧：`part03.js` proxy 键统一（`_makeProxy` 消费归一 sel）。
- 实验二进制链：`zw-runner-b3t`（trace 插桩）→ `b3u`（活锚让位门变体）→
  `b3v`（反转守卫 + 归一刀）→ `b3w`（BASE+守卫，刀 checkout 后）→
  `zw-runner-d42`（b3x 重构建 = 最终落树态）。

## A/B 判决（input-events 全语料，zz-probe 排除口径）

| 二进制 | 构成 | 全语料（真实文件） | 判决 |
|---|---|---|---|
| b3r-base | BASE | 461P/2867F/3T | 基线 |
| b3r-wip2 | 归一刀 | 434P/2830F/3T | 净 **-27** |
| b3u | 活锚让位门（①在锚∈el 子树时跳过） | 369P/2907F/3T | marker 语料管线崩（caret 案重映射即关键工作，489P 含探针） |
| b3v | 归一刀 + 反转守卫 | 434P/2830F/3T | 守卫只修探针副本，真实语料 = WIP2（净 -27 持位） |
| b3w / d42 | BASE + 守卫 | **461P/2867F/3T** | 与 BASE 逐字节恒等 |

逐文件 delta（base → knife，dP 合计 -27）：

- joining-dl-elements ×2 变体：36P→24P（**-12×2**，主 P2F 面）
- joining-dl-element-and-another ×2：20P→18P（-2×2）
- deleting-in-list-items ×4：12/13P→10/11P（-2×4）
- backspace 主文件：38P→39P（+1）
- non-collapsed-selection ×2：8P→11P（**+3×2**，身份修复真实解锁面）

P2F 分解（joining-dl 12P）：8 案 = targetRanges 期望编码了 BASE 的
identity-churn null 端点伪影（knife 稳定端点不再匹配——这些期望值是上游
Chrome 真值、BASE 靠伪影巧合命中，knife 未完全复刻上游端点报告语义）；
4 案 = bi=0 join 形（归一后重映射塌缩，见下）。**结论：方向正确（+3 面
实 WIN），需要 generation-aware 重设计后分层重放**——身份稳定必须与端点
报告语义成对修，单刀归一不可落。

## 重映射序失真根因（b3s TRACE 实证，守卫落树依据）

`zz-probe-b3s.html`（joining-dl 语料副本 + caret_range/delete 入口注入）+
WIP2 shim `__zwCrTraceLog` 轨迹（trace 条目 [151]）：backward select r0 =
`(text,10)-(text,0)` 非塌缩 → `_zwCeReanchorRange` ① down-walk 两端点在归一
方言后收敛同节点 → 输出 `(X,10)/(X,0)` → `nr.setEnd` 的 R203 镜像
（`_zwRangeBpAfter(start,end)` 即 start 在 end 后 → start 重写为 end）→
select 塌缩 `(0,0)` → dl-join select 案 bi=0（BASE 下该案 ①不触发
identity-walk 失败走 keep-gate 保 r0，恒无此回归面）。

**守卫**（part01.js `_zwCeReanchorRange`，本轮唯一落树变更）：非塌缩 r0 的
重映射若 `start` 落在 `end` 之后（`_zwRangeBpAfter` 树序比较）= 端点身份
失真 → 弃本轮重映射返 null，交活锚让位门保 r0。塌缩 caret 重映射
（marker 语料管线主消费面）不受影响。BASE+守卫全语料逐字节恒等 = 纯加固，
仅在身份损伤时承载载荷（归一刀重落树时的既有回归面即被兜住）。

## 哨兵链（BASE+守卫 = d42 二进制）

- input-events 全语料：`/tmp/d42-ie.json` vs BASE 461P/2867F/3T **恒等** ✅
- html-semantics 全语料：`/tmp/d42-hs.json` **P 恒等 5378** ✅
- selection 全语料：见 master.md 本轮记档（429 中断后本 session 补跑对拍）

## 下轮方向

1. generation-aware 归一刀重设计：端点报告语义与身份稳定成对修——
   joining-dl 8 案 artifact 期望逐条对齐上游端点报告（Blink target range
   生成点），4 案塌缩面由守卫兜底后重放 +3 WIN 面。
2. B4-S1 cross-block select-delete join（非塌缩 range 跨块删除主簇 ≈820F 的
   text-anchored 子面）——与身份刀解耦，先走 innerHTML 语义面。
