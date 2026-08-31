# R262 Evidence — removeChild live-range 边界迁移（mutations-removeChild 100%，+18P）

**日期**: 2026-08-26
**切片**: M4——R262(a) removeChild 的 pre-remove 边界迁移
**改动面**: part03（`_zwAdjustRangesForRemove` helper + `_zwMEl`/doc 级/docEl/detached
body 四 removeChild 站点接线）+ part04（proxy 域 handle 子/sel 子两分支接线）+
part23.rs（r257 场景三期望更新 + r262 回归单测）
**commit**: 见本轮 commit

## 一、spec 语义（WPT modifyForRemove 逐字引用）

WPT Range-mutations.js 的 `modifyForRemove`（testRemoveChild 的期望生成器）
引用 spec `concept-node-pre-remove` 末段两规则：

1. **边界在被移除子树内**（node 是 removed node 或其子孙）→
   `(old parent, old index)`（移除前的父与索引）。
2. **边界在 old parent 且 offset > old index** → `offset − 1`
   （offset ≤ index 不动）。

**调用时机契约**：oldParent/index 读移除前形态——须在实际摘除**前**调用
（与 R261 splitText 的「原始 offset」教训同源：判定与迁移都用移除前快照）。

## 二、实现

### `_zwAdjustRangesForRemove(removed)`（part03，挂 globalThis 跨 part 路由）

- 身份匹配三键（与 R260 `sameNode260` 同源策略）：identity / `__zwHandle`
  字符串（proxy 域单次 get trap 产物 identity 不稳）/ `__zwSelector` 字符串。
- 子孙判定：沿 `parentNode` 上行 128 跳（每跳做三键 sameNode）。
- 父匹配：identity 优先，handle/sel 键兜底（proxy 域「removed 的父」与
  「range 边界容器」可能来自不同 trap 产物）。
- 边界重写同时清 `_mode`（selectNode 形态的现算 getter 会覆盖写入值）。

### 六个接线站点（全部先于树状态变化）

| 域 | 站点 | 测试形态 |
|---|---|---|
| part04 proxy | handle 子分支（`_unrecordHandleChild` 前） | **主战场**：`paras[0].parentNode.removeChild(paras[0])`（testDiv 是 handle 容器） |
| part04 proxy | sel 子分支（`__zw_remove` 前） | sel-based 父形态 |
| part03 `_zwMEl` | mutTree `node.removeChild`（splice 前） | detached 容器域 |
| part03 doc 级 | detached doc `removeChild`（`foreignDoc.removeChild(documentElement)`——WPT 10,x） |
| part03 docEl | factory docEl removeChild |
| part03 body | detached body removeChild（`_tree.removeChild` 前，proxy/registry 域兜底） |

## 三、R257 单测场景三的语义翻转（非回归，单测期望更新）

19,9（`[detachedPara1,0,…,1]` + detachedDiv 父 newParent）：

- **旧**：清子循环 `dd.removeChild(dp1b)` 不迁移边界 → range 仍 (dp1b,0) →
  R257 ancestor 检查从 sc=dp1b 上行，dp1b.parentNode 已被清子断为 null →
  不命中 → 「成功 wrap」（`ok3`）。
- **新（spec 正确）**：removeChild 按 spec 把边界 (dp1b,0) 迁到 (dd,0) →
  ancestor 检查从 sc=dd 上行第一跳即命中 newParent=dd（inclusive）→ HRE。
- **真浏览器引证**：chromium 中 live range 同样被清子循环迁移到 (dd,0)，
  sim（mySurroundContents）与 actual（surroundContents）**双侧同步抛 HRE**，
  WPT 19,9 作为 assert_throws 匹配通过。本引擎翻转后两侧同步——
  **surround 套件 1840P/0F 保持 100% 是契约成立的直接证据**。
- r257 单测场景三期望从 `ok3|wrap=Y` 更新为 `t3:HierarchyRequestError`，
  注释记录翻转机制。

## 四、验证（vs R261 基线）

| 项 | R261 | R262 | Δ |
|---|---|---|---|
| Range-mutations-removeChild | 2P/18F | **20P/0F** | **+18（100%）** |
| Range-mutations-splitText | 116P/0F | 116P/0F | 持平（100%） |
| Range-surroundContents | 1840P/0F | 1840P/0F | 持平（100%，含 19,9 语义翻转） |
| Range-insertNode | 1841P/0F | 1841P/0F | 持平（100%） |
| Range-deleteContents | 80P/49F | 80P/49F | 持平（stash A/B 逐条一致） |
| Range-mutations-appendChild | 42P/28F | 42P/28F | 持平（stash A/B 逐条一致，预存簇） |
| Range-mutations-replaceChild | 30P/30F | 30P/30F | 持平（stash A/B 逐条一致，预存簇） |
| engine 单测 | 2401 | **2402** | +1（r262 回归单测）+ r257 期望更新，全绿 |
| fmt / clippy（workspace） | 干净 | 干净 | — |

**预存簇核实**（stash A/B，clean-HEAD 重建二进制）：appendChild 28F /
replaceChild 30F / data 族 5 超时（R261(a) 已归因累积型慢）——全部与
clean-HEAD 逐条一致，0 回归。

## 五、R263 靶点

- **Range-mutations-appendChild 28F / replaceChild 30F 重聚类**：两簇是
  mutations 域最后两个非超时失败面。WPT testInsertBefore/testReplaceChild
  的期望 = modifyForRemove（已落地）+ modifyForInsert（「boundary 在 new
  parent 且 offset > new index → +1」——插入侧调整尚未实现）——appendChild
  的 `testDiv.appendChild(testDiv.lastChild)` 形态预期先经 modifyForRemove。
  候选：`__zwAdjustRangesForInsert` 对偶实现 + appendChild/insertBefore 站点
  接线（移动语义 = remove + insert 两段）。
- extractContents 残余 32F / cloneContents 29F 重聚类。
- replaceData 累积型超时（预存，低 ROI）。
