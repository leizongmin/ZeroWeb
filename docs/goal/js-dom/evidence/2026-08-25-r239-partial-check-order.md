# R239 Evidence — 部分包含检查序 + nextNode 遍历（INVALID_STATE 30F 全解）

**日期**: 2026-08-25
**切片**: M4——R239(a) INVALID_STATE 30 簇（20–22,x/29/31,x）
**改动面**: `part06.js`（surroundContents 入口两件）+ `part23.rs`（r239 单测）
**commit**: `3ddd77312`

## 一、根因

20,x `[t0,0, t1,0]`（跨 text 边界）+ Document/Doctype/Fragment 作
newParent：host 的 R209 nodeType 检查（InvalidNodeTypeError）先于
R210 部分包含检查执行——sim（mySurroundContents）是 partial 检查
在前，WPT 期望 INVALID_STATE_ERR（30F「must be thrown」簇）。

## 二、修复两件（过程坑两个）

1. **检查序对调**：R210 partial 检查移到 R209 nodeType 检查前。
   过程坑①：拼接丢闭合大括号——shim 编译错当场暴露（SyntaxError
   Unexpected token ','），立即修复。
2. **DFS → nextNode 序遍历**：过程坑②——首版保留 DFS（比 sim 的
   nextNode 遍历更完备），实测 24,x 反向翻转 12F（host DFS 扫到
   sim 遍历漏掉的 partially-contained 节点——sibling 链在 shim
   形态上断链使 sim 提前终止 → WPT 期望 INVALID_NODE_TYPE 而
   host 抛 INVALID_STATE）。改为 sim 同款遍历原语
   （hasChildNodes→firstChild / 爬 parentNode 取 nextSibling），
   盲区与 sim 对齐后 fixed=30/new=0。

## 三、验证链（vs R238 基线）

| 项 | R238 | R239 | Δ |
|---|---|---|---|
| Range-surroundContents | 1671P/169F | **1701P/139F** | **+30，0 新失败**（INVALID_STATE 簇全解） |
| Range-insertNode | 1841P/0F | 1841P/0F | 0（100% 保持） |
| Range-extract/delete/clone | 125/67/155 | 125/67/155 | 0 |
| events 失败集 | 7 | 7 | 一致 |
| **ranges 全量**（除 probe） | 40080 行 | 40080 行 | set-diff **30 Fail→Pass / 0 反向** |

- **native 同值**：ZW_NATIVE_DOM=1 surround 1671→1701P（+30 一致）。
- **engine 单测**：**2385 全绿**（新增 r239_partial_check_order_and_traversal）。
- fmt/clippy 干净。

## 四、R240 靶点（139F 重聚类）

| 簇 | 计数 | 行 | 备注 |
|---|---|---|---|
| cDP | 40 | 17,x 17 + 30,x 23 | 绑 host foreignDoc surround 全序（R235 负结果） |
| assert_unreached | 40 | 24,x 32 + 28,x 6 + 18/19,x 各 1 | 24,x 跨子区间 `[testDiv,2,paras[4],1]` |
| differing | 34 | 28,x 28 + 13/14,x 4 + 24,x 2 | 28,x `[foreignDoc.body,0,foreignTextNode,36]` |
| HRE | 14 | 25/26,x 各 6 + 18/19,x 各 1 | 25/26,x `[document,0,…]`/`[document,0,document,2]` Document 容器 |
| startOffset | 11 | 16,x | harness-iframe index 算术 |

- **首选**：28,x differing 28F（`[foreignDoc.body,0,foreignTextNode,36]`
  ——ec 是 body 直接 CD 子但 foreignDoc.body 无 cDP/方法面的交织，
  R236 祖先分支对 foreignDoc 形态的适配）。
- 次选：25/26,x HRE（Document 容器 `[document,0,…]` 的 surround
  全序——insertNode 的 doc 级位序校验）。
