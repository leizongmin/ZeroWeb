# d60 · bold 壳 no-join 簇部分诊断（零码变，插桩已回退）

**2026-11-19 · d60 诊断轮（未完成，DC-3 coverage dispatch 优先中断）**。d59 挂账
「bold 壳 bi=0 no-join ×5（d56j T1/T2/T4、d56jt1 T1、bs-06）」按 d58 DIAG57 同款
配方（user_actions.rs `tracing::warn!("DIAG57 sbx-recorded …")` wire 批 dump＋
part01 `__zwDiagJoin` impl/walk 臂标记）建隔离页
`input-events/zz-diag-bs15.html`（zz-probe-d56j 同款 zzCase＋恒败 dump），四形定谳：

## 1. 形级定谳（zzdiagBS15 T1/T2/T4/T7）

| 形 | init HTML（caret `[]`） | 终态 | bi | 定谳 |
|---|---|---|---|---|
| T1 | `<div><b>abc</b><p><b>[]def</b></p></div>` | `<div><b>abc</b></div>`（def 丢失） | 1 | **死锚 wire op 新机制**（§2） |
| T2 | `<div>abc<p><b>[]def</b></p></div>` | 原样（无 join） | 0 | **n=0 静默 bail**（§3，③ 页上下文族） |
| T4 | `<div>abc<p>[]def</p></div>` | 原样（无 join） | 0 | **n=0 静默 bail**（§3） |
| T7 | `<div><p>abc</p><p>[]def</p></div>` | `<div><p>abcdef</p></div>` | 1 | 对照全绿（join 形＋事件均正确） |

诊断产物：`/tmp/d60-bs15-diag.json`、`/tmp/d60-bs15-diag2.json`（trace 前缀混
stdout，DIAG57 行 grep 提取；`/tmp/d60-diag-runner` 为插桩二进制，插桩回退后
二进制作废）。

## 2. T1 死锚 wire op（新机制，非派发鬼）

T1 wire 批（n=6，bs15-diag2 拍得）：

```
[SetInnerHtml { div, "<div><b>abc</b><p><b>def</b></p></div>" },
 CreateTextNode { __n0, "abc" }, CreateTextNode { __n1, "def" },
 Remove { "p > b:nth-child(1)" },                                   ← 摘 b 壳（join 提取 def 为新文本）
 Remove { "div > div > p:nth-child(2)" },                           ← 摘 p
 InsertAdjacentSelElement { div > div, beforeend,
   child_selector: "…p:nth-child(2) > b:nth-child(1)" }]            ← 死锚：引用 op5 已摘的 p 内 b
```

op6 adjacent-move 臂以 selector 寻址**同批早前 op 已移除的节点**——引擎 apply 时
selector 解析落空 → 整 op no-op → def（`__n1` 已建但无落位 op）丢失。
bi=1＝事件派发正常，**非 d56b ③ 派发鬼族，是 wire 保真缺口**：join 的
adjacent-move 记账在「壳先摘、后补插」序下产出自引用死锚。修向（下轮）：摘壳序
wire op 应以 handle/父级锚代替子 selector 锚，或 adjacent-move 臂检测锚节点已在
本批 remove 集时改挂父级 afterend/beforeend 语义。注意 d56j 页上下文同形键报
bi=0（本页 bi=1）——**页上下文改变症状面**，两页账目不可直并。

## 3. T2/T4 n=0 静默 bail（③ 页上下文族镜像，零码变持位）

T2/T4 impl 入口与 walk 臂标记均命中（`impl range=ok collapsed=true sc=#text
so=0` → `walk sc=#text so=0`）但 wire 批 n=0＝walk 臂内静默 bail；DOM 原样、
bi=0。T4 与 corpus backspace 主形同构（corpus `abc<p>[]def<br>ghi</p>` 形 d58
已翻 PP），差异仅在页上下文（本探针页 `gSelection.collapse` 直设 vs corpus
harness 块内建选区）——d56b ③ wrapper 家族域在册形，判负后无入口级现成臂，
持位不动。T7 对照（同页同入口）全绿说明**同页内 plain 块界 join 不受该 bail
影响**，bail 条件与 b/p 壳形相关（T2/T4 共同因子＝首个块界后邻带元素壳）。

## 4. 下轮入口

1. 若 DC-3 快照差线：T1 死锚修（§2 修向，预计惠及 bold 壳形 corpus 键）；
2. 形 2 ghost 毒化 ×2（真实 corpus 键，non-collapsed?TypingA br 形）仍挂
   d56b ③；
3. ④ B3 text-control execCommand 期望矩阵（~474F）为最大剩余簇。
