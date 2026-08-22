# R162 Evidence — nth 公式内空白 + matches 的 :target fragment（M4 nodes）

**日期**: 2026-08-22
**Commit**: `0ed5d7ceb`（rebase 后；原 `d01e0cbba`）
**切片**: M4 — Element-matches 残簇收尾（669P→672P/3F；全量 9510P→9515P/9516P 双路径等价）

## 两件修复

### 1. `parse_nth` 公式内空白全剥（2F）

spec CSS microgrammar 允许 an+b 记号间任意 ASCII whitespace——WPT
`li:nth-child(2n \t\r\n+ \t\r\n4)` 形态。旧版 `trim()` 只去两端，`+` 与数之间
的内部空白致 i32 parse fail → None → 整选择器判非法。修：过滤全部
ASCII 空白后再 parse（odd/even/an+b/纯 b 四形态不受影响）。

### 2. matches 根上下文查询透传 fragment URL（1F）

`_zwParseEl.matches(':target')` 的 `_zwRootHtml` 查询不传 URL → host 重 parse
无 fragment → 恒 miss。修：经 `_zwOwnerDoc._zwFragmentUrl` 透传（与 doc 级
queryBody R160 路径同源）。

## 结果

| 子集 | 结果 |
|---|---|
| Element-matches | **672P/3F**（剩 3F 全部 = `[*|TiTlE]` 第 3 命中——树碎片化域） |
| ParentNode-querySelector-All | 1937P/37F（不变——本轮两项不在其 fail 集） |
| 全量 dom WPT polyfill | **9515P/347F/19T** |
| 全量 dom WPT native | **9516P/347F/18T**（等价，±1 边缘 Timeout） |

七轮累计（R156 起 6290P）：**+3225P**。

## 残余全部收敛到统一 identity 域（L2）

Element-matches 剩 3F + ParentNode 剩 37F（tree order identity 断言 / Fragment
dynamic NodeList / ns 簇 / `[*|TiTlE]`）共同根因 = **查询面与遍历面的元素
identity 不统一**（querySelectorAll 返回缓存 wrapper、traverse 走 mutTree
节点、setup 走 createElement proxy——三个对象域）。正解 = L2 统一 live
Document（M1），本轮及后续轻件已榨干。

## 验证

- `cargo test -p zero-dom`：849 全绿；`cargo test -p zero-engine`：2307 全绿
- `make test` 全绿；fmt/clippy 干净
