# R300 Evidence — Node-insertBefore 全文件收口（40P/0F 100%，顺序断言族 + doc-parent 簇 + step-6 死代码复活）

**日期**: 2026-08-27
**切片**: M4——R300(a) Node-insertBefore 剩余 8F 全解（原计划只做顺序断言族 ×3，探针连环发现另两层根因，整文件收口）
**改动面**: `part03.js`（泛型 step-3 sel 域限定 + winning fn ownerDocument 门移除 + doctype 唯一性 + `_r296hre/nfe` 定义补齐 + detached-doc step-3）+ `part04.js`（trap NotFound 前移 + trap step-6 补齐）+ `part24.rs`（+1 单测）

## 一、根因三层（sandbox 探针逐层暴露）

### 层 1：顺序断言族 ×3——step-3 NotFound 晚于类型 HRE

WPT `pre-insertion-validation-notfound` 三断言（步骤 3 先于 4/5/6）：非法 node +
detached child 双违时须报 NotFound(8) 非 HRE(3)。探针六形态全 HRE(3)：
- **泛型路径**（`Node.prototype.insertBefore.call(parent,...)`）：R296 后步骤序 =
  1-2 → 4-6（`_r117GenVal`），step-3 完全跳过（lenient 回退时删除）；
- **trap 路径**（parent 是 proxy → R219 委托）：step-3 存在但在 step-4 之后。

### 层 2：winning fn 的 step-6 死代码（`_r296hre` 未定义）

R296 在 detached-doc 的 winning `insertBefore`（part03:8302）写了完整 step-6 检查，
但 `_r296hre` **从未定义**——`ReferenceError` 被外层 `catch (_e296dv)` 静默吞掉
（catch 只 rethrow `name === 'HierarchyRequestError'`）→ 整段检查 no-op。探针
实证 `doc.insertBefore(el, null)`（doc 已有 html）no-throw。**R296 轮的
「step-6 已 land」认知有误**——实际从未生效。

### 层 3：ownerDocument 门 + doctype 唯一性缺口

补定义后 doc-parent 簇仍有 3F：winning fn 的元素冲突检查被
`newNode.ownerDocument !== doc` 门限定（同 doc 元素跳过——但 WPT 的 el 恰是
doc.createElement 产物）；doctype 唯一性只在尾部追加分支（`!refNode`）拦截，
`insertBefore(doctype, comment)`（既有 doctype 在后）漏过。

## 二、修复五处

1. **泛型 step-3**（part03:1842 域）：`_r117GenValParentAncestor` 后、`_r117GenVal`
   前插 NotFound——**sel 域限定**（`this.__zwSelector` undefined 才查——detached
   容器；R296 回归域的 blank 页 loading 路径是 sel-based，整体排除）+ parentNode
   identity 判据；
2. **trap NotFound 前移**（part04:3848）：既有 R296 检查从 step-4 后移到前
   （纯重排，无新抛出条件）；
3. **trap step-6 补齐**（part04）：trap 版此前完全没有 doc-parent 检查（探针
   实证 createHTMLDocument 产物经 wrapper proxy 走 trap 而非 winning fn）——按
   插入位方向实现 element/doctype 冲突 + fragment 形状 + text 禁入；
4. **winning fn 修复**：`_r296hre/_r296nfe` 函数内定义（死代码复活）+
   ownerDocument 门移除（同节点豁免已由 `_r296wSelf` 承担）+ doctype 唯一性
   补全（任何位）；
5. **detached-doc step-3**（part03:8070）：childNodes identity miss → NotFound
   （WPT detached 文档不经 loading 路径，视图完整）。

## 三、验证

| 套件 | 基线 | R300 | Δ |
|---|---|---|---|
| **Node-insertBefore** | 32P/8F | **40P/0F（100%）** | +8P/-8F |
| Range-insertNode（R296 回归哨兵） | 1841P/0F | 1841P/0F | 持平 ✓ |
| Range-surroundContents（同） | 1840P/0F | 1840P/0F | 持平 ✓ |
| NodeIterator-removal（同） | 29P/0F | 29P/0F | 持平 ✓ |
| Range-delete/extract/cloneContents | 129/192/191P 0F | 同 | 持平 ✓ |
| Node-replaceChild / appendChild / removeChild / CharacterData | 58/11/28/157P | 同 | 持平 ✓ |
| Range-mutations / mutations / insert-adjacent / Node-cloneNode | 见 A/B log（detach 跑） | — | — |
| engine 单测 | 2437 | **2438**（r300 单测：六形态 step-3 序 + doc-parent step-6 四断言 + 合法插入对照） | +1 |
| make test | — | 1F = XOpenDisplayFailed 环境项 | 持平 |
| fmt / clippy | — | 干净 | — |

**sel 域限定的实证**：Range/surround/insertNode 全绿证明 loading lenient 域未被
波及（这些套件内部大量经 sel-based 容器 insertBefore 挂 pending ref）。

## 四、教训

1. **catch-and-filter 吞 ReferenceError**：`catch (e) { if (e.name === 'X')
   throw e; }` 模式会静默吞掉函数体内**未定义变量**的 ReferenceError——step-6
   检查「写了但从未生效」藏了两轮（R296 land 时探针未覆盖同 doc 元素形态）。
   异常工厂须与 throw 同点定义（本轮函数内 var 定义）。
2. **探针要打到「实现真正所在」**：R296 以为 step-6 落在 detached-doc 的
   insertBefore，但 createHTMLDocument 产物实际经 wrapper proxy 走 **trap**
   （`String(doc.insertBefore).slice(0,60)` 一行 introspect 即暴露）——同语义
   多实现点（trap/泛型/winning fn/factory）须逐点验证而非按注释推断。
3. **门条件过宽的豁免**：`ownerDocument !== doc` 想豁免「同节点移动」但实际
   豁免了「同文档所有元素」——豁免条件应精确到 identity（`=== newNode`）而非
   文档归属。
