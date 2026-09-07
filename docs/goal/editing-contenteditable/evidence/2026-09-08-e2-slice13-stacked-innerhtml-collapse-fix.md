# E2 切片 13 — stacked same-sel innerHTML 融合视图塌缩修复（2026-09-08）

**驱动用例**：上游 WPT `selection/onselectionchange-on-document.html` 第 3 subtest
（'task to fire selectionchange event gets queued each time selection is mutated'）
@ `315976933870b34d6ea30e3f6643403edae678ba`——切片 12 导入面内的确定性失败案
（非新导入）。

## 勘误（先于修复）

切片 12 的 master.md #21 与 evidence「残余记录」当时声称该 subtest「FILTER 单案跑
失败、全量套件跑下通过（三连跑稳定）」。本轮开工前复验（`make testharness-selection`
全量 3 连跑 + FILTER 单案）**推翻该表述**：

```
全量套件 3 连跑：每轮均 6F，fail 集恒定，恒含该案
  IndexSizeError: The given offset is out of bounds（unhandled rejection）
FILTER 单案：稳定失败（与切片 12 记录一致）
```

即「FILTER 独有 / 全量通过」系当时验证不充分所致的错误结论。DONE 门要求 master.md
内部自洽，故先修正 master.md #21（追加勘误注记）与本 evidence 残余记录段，再开
修复切片。

## §9 碰撞核对

`git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`：js-dom 流最后
活动为 R387（2026-08-31，parse-backlog MO replay）；近 14 天该路径全部为
editing/keyboard 流提交。fusion 视图面（R51c/R304/R380 族）无并行流活跃，可直接
进入。

## 根因（shim 侧逐点插桩实证）

对 shim `_childNodeList` / `_zwOverlayPendingChildNodes` / R304 基底置空点 /
`_zwChildBaseInvalidateAll` 逐点埋 trace（baseSet/cacheHit/overlay 进出 + 桶
added/removed 计数 + 代际号），最小复现案逐 subtest 快照：

```
塌缩点（C subtest spin 后首读）现场：
  overlayIn inLen=2            ← base cache rebuild 正确（2 子，推翻「置空 [] 未失效」初版归因）
  overlayBucket a=4 r=4        ← 桶残留 4 added + 4 removed（两次 innerHTML 各计 2+2）
  overlayMerged len=0          ← 合并后 0
  cacheHitOut len=0            ← childNodes 塌缩为 0
  → setPosition(container, 2)：_nodeLength=0，offset 2 越界抛 IndexSizeError
```

精确链路（修正切片 12 初版归因的「base 置空未失效」「桶条目消失」两处误判）：

1. innerHTML setter（sel 路径）入队 SetInnerHtml + 桶记账：added=解析 wrapper、
   removed=旧子 proxy。子 proxy 经 `_wrapSelector` → `_makeProxy` → `_proxyCache`
   **按 sel 稳定复用同一对象**（R100 identity 语义）。
2. host apply → `__zw_apply_generation_bump`（pa2b）：只清 parse 补偿 **added**
   （K3 切片 C），**removed[] 条目残留**（桶 + 全局表）。
3. 换代后（bump → `_zwChildBaseInvalidateAll`）新基底 rebuild：host 快照真实子
   经同一 `_proxyCache` 重建 → 与残留 removed 条目 **identity 相同**。
4. overlay 的 removed 剔除按 identity 匹配 → 快照真实子被整批剔空 → childNodes=0。

## 修复

`__zw_apply_generation_bump`（crates/engine/src/js_dom_shim/part05.js）补
**removed 补偿同批作废**：

- 桶 `_zwPendingByParent` 各条目 `removed[]`/`removedSet` 清空 + 全局
  `_zwPendingRemoved` 清空（Set 惰性重建置 null）。

语义依据：removed 条目是「快照已含节点、host apply 未落」窗口的视图修正补偿
（`_zwOverlayPendingChildNodes` 剔除 / live 集合剔除 / query stale 命中剔除），
apply 后快照已真删除，全部消费面只服务 apply 前窗口；对换代新基底是纯毒源。
与 K3 切片 C 的 parse 补偿 added 清理同族——apply 代际边界统一作废。handle-only
removed 条目本为死数据（R51c 压实语义），一并清除。added 侧 identity 记账
（re-append 移动语义、live 集合并入）按 pa2b 原设计保留，不动。

## 验证

```
onselectionchange-on-document.html：4 subtest 全 Pass（修复前全量/FILTER 均确定性失败）
selection 全套件：2993P/6F → 2994P/5F（净 +1），3 连跑 fail 集恒定：
  anchor-removal 2F（renderer S3 布局命中，切片 11 归因）
  script-and-style-elements 1F（渲染投影，M3 切片 4 归因）
  Document-open 1F（legacy 形态，记录不追）
  editing/event 1F（legacy 形态，M2 切片 1 归因）
单测 r3254_e2_slice13_apply_generation_invalidates_removed_compensation：
  四组断言（同 sel 两次 innerHTML 跨 apply 代际后 fresh 基底长度 + 内容）；
  无修复复现失败、有修复通过——根因锚定测试
make test：全绿（guard 包裹）/ clippy 零警告
```

## 门禁

- `make test`：全绿（guard 包裹）
- `cargo fmt --all -- --check`：无 diff
- `cargo clippy --workspace --all-targets -- -D warnings`：零警告
