# R245 Evidence — factory doc parentNode getter-only 移动语义修复（17,x 深诊断轮）

**日期**: 2026-08-25
**切片**: M4——R245(a) 17,x differing 12F 深诊断（引擎 move 语义 bug 落地，WPT 净 0）
**改动面**: `part03.js`（`_zwMEl.appendChild` 摘除守卫 + 父链强写 ×2 站点）+ `part23.rs`（r245 单测）
**commit**: 见 master.md 本轮记录

## 一、诊断链（R245-probe 五轮探针 + R245b sim 步日志）

17,x（`[foreignDoc.documentElement,0,…,1]`）元素 newParent 族：

1. **身份假设排除**：bodyP-docEl:true / k0P-self:true——wrapper identity 稳定，
   NOT_FOUND 非身份漂移。
2. **sim 内部日志**（monkey-patch `ensurePreInsertionValidity`/`myInsertNode`）：
   全部 `=ok`——NOT_FOUND 不来自 myInsertNode 返回串。
3. **micro-probe 步进隔离**：`fragAppend-OK / clear-OK / insB-OK /
   p1Append-THREW:NotFoundError`——抛点锁定 `p1.appendChild(frag-with-HEAD)`。
4. **运行时标记**：`preAppend(headP=HTML,...)`——**HEAD.parentNode 恒指 factory
   docEl**（应为 frag）。
5. **根因两层**：
   - factory `headEl`/`body` 的 `parentNode` 是 **getter-only accessor**
     （part03 R130 `{ get: () => docEl }`）——fragment appendChild 的
     `c.parentNode = this` 裸赋值被静默吞（sloppy no-op）；
   - `_zwMEl.appendChild` 的「从旧父摘除」`c.parentNode.removeChild(c)`
     **无 try/catch**——HEAD 已从 docEl.childNodes 摘出，factory docEl.removeChild
     按 identity 找不到 → NotFoundError 未包裹直接传播到 sim
     `mySurroundContents` catch → `NOT_FOUND_ERR`（WPT 17,4/11/13 期望异常）。

## 二、修复（两站点，spec `concept-node-pre-insert` 的 adopt 摘除幂等语义）

1. `_zwMEl.appendChild`（part03 ~5989）：摘除调用包 try/catch（旧父已无此子不视为
   错误）+ 入树父链 `defineProperty` 强写（getter-only 遮蔽，`_r223SetParent` 同款）。
2. factory fragment `appendChild`（part03 ~8021）：`c.parentNode = this` 改
   `defineProperty` 强写。

micro-probe 修复后：`preAppend(headP=#document-fragment)` +
`p1Append-OK(n:1)` ✅。

## 三、验证链（vs R244 基线）

| 项 | R244 | R245 | Δ |
|---|---|---|---|
| Range-surroundContents | 1806P/34F | 1806P/34F | **净 0**（17,4/11/13 失败形态迁移：NOT_FOUND assert → sim ret:ok + 树 walk TypeError——sim 成功后 assertNodesEqual 走到更深处，17,x 全解需后续 sim 树等价工作） |
| ranges 全量（除 probe） | 40080 行 | 40080 行 | set-diff **0 Fail→Pass / 0 Pass→Fail** |
| Range-insertNode | 1841P/0F | 1841P/0F | 100% 保持 |
| dom/nodes 失败集 | 57 | 57 | **逐条一致**（diff exit 0） |
| engine 单测 | 2391 | 2392 | 全绿（新增 r245 单测：fragment 父链强写 + 摘除守卫 + 17,4 形态端到端） |

- `make test` 1F 为 XOpenDisplayFailed 环境项（run-rules §10，历轮一致）。
- fmt/clippy（`-D warnings`）干净。

## 四、判定：land 依据

WPT 净 0 但引擎 move 语义 bug 真实（micro-probe 五轮链实证）：factory doc 内
「HEAD 移入 frag 再移入元素」的 spec pre-insert adopt 语义修复，为 17,x 下一步
（sim 树等价 / assertNodesEqual walk 的 `"[object Object]"` 节点形态）扫清
NOT_FOUND 噪声——下一轮 17,x 诊断从「sim 成功后树比较」起步而非异常链。

## 五、R246 靶点（34F 不变，形态已迁移）

| 簇 | 计数 | 行 | 备注 |
|---|---|---|---|
| 17,x | 12 | 4/11/13 TypeError 6 + 0/6/9 "[object Object]" 6 | sim ret:ok 后的树 walk 深分歧——assertNodesEqual 的 expected 树含无 nodeType 对象（probe dump expectedRoots 首差节点） |
| 16,x startOffset | 11 | body[4,5] | harness-iframe index 算术 |
| 残余 | 11 | 30/13/14/18/19/28 | — |

- **首选**：17,x sim 树等价——探针 dump `mySurroundContents` 成功后 expected
  树（frag→newParent 的 HEAD 子树形态 vs host clone 形态）。
- 次选：16,x startOffset 11F。
