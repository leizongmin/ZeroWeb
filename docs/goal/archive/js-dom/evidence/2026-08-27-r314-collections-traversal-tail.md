# R314 Evidence — collections/traversal 域尾部归因（collections 0F；traversal 1F root 止步修复 + handle 兄弟链备档）

**日期**: 2026-08-27
**切片**: M4——R314(a) collections/traversal 域尾部归因
**改动面**: `part06.js`（TreeWalker previousNode 的 root 止步 spec 修复）+ `part24.rs`（+1 归因诊断单测）

## 一、成果

| 套件 | 基线 | R314 | Δ |
|---|---|---|---|
| dom/collections | 49P/0F | 同 | 持平（全域 0F——无尾部） |
| dom/traversal | 1603P/1F | 1603P/1F | 数字持平；失败分量改善（错误节点 `__n0`→`null`——root 止步生效，余 handle 兄弟链面） |
| NodeIterator | 795P/0F | 同 | 持平 |
| TreeWalker | 811P/1F | 同 | 持平（同 traversal 1F） |
| Node-cloneNode / ParentNode-querySelector / Element-matches | 基线 | 同 | 持平 |
| engine 单测 --lib | 2452 | **2453** | +1（r314 归因诊断） |
| make test | 1F 环境项 | 同 | 持平 |
| fmt / clippy | — | 干净 | — |

## 二、traversal 1F 归因（TreeWalker-walking-outside-a-tree，Acid3 6a）

探针逐步打点（主文档 createElement 产物 = **handle proxy 域**）实断两个独立缺口：

1. **regraft 后 handle 兄弟链断**（`p.previousSibling` 恒 null）：`doc.removeChild(body)`
   + `doc.appendChild(p)` 重挂后，handle registry 的兄弟融合视图看不到同父的 `head`
   （应返 `head` → 进子树尾 `title`）。**R291 域**（identity 双源——R309 教训的
   同构面），需 handle registry 重挂时的兄弟 relink 或融合视图注册表反查——**深结构备档**。
2. **previousNode 父上行越过 root**（本轮已修，part06）：R85 循环的 root 止步只挡
   `node === root` 但仍会对链外节点跑 check 并可能返回（探针 prevNode=DIV——root 外
   的 doc 自身被误返）。修：上行节点须在 root 子树内（祖先链含 root）才参与 check，
   否则 null。修复后探针 `prevNode=null`（正确边界行为；完全通过待缺口 1）。

## 三、域状态总览（R305–R314 dom 域尾部盘点）

- **collections**：0F（干净）
- **traversal**：1F（TreeWalker regraft——缺口 1 的 R291 域备档 + 缺口 2 已修）
- **events**：4F（R313 全部域界定：handlers-changed/event-global 深结构备档 +
  pseudo 不追 + 本地探针）
- **MutationObserver**：4F（parse-time 3F + R220 工厂可观察 1F，备档维持）
- **Node-properties / ParentNode 全族 / Element-matches**：0F

## 四、教训

`w.lastChild()` 这类**方法调用返回节点对象**的探针，`String(x && x.nodeName)` 在
x 为 null 时安全、x 为对象但取属性失败时不安全——Rust 侧断言字符串拼接与 JS 空值
语义要对齐（本轮编译期字符字面量引号错误 + null.nodeName 两次返工）。
