# R172 Evidence — ParentNode 剩 33F 聚类 + ns 词法层修复（M4）

**日期**: 2026-08-22
**Commit**: `d538dee36`
**切片**: M4 轻件收口——ParentNode-querySelector-All 剩 33F 聚类定位 + zero_dom ns 前缀词法修复

## 一、33F 聚类（当前形态）

| 簇 | 计数 | 性质 |
|----|------|------|
| Namespace selector（`|*`/`|div`/`*|div` × 4 上下文 × 2 方法） | 24 | 两层：词法误拒（本轮修）+ setup 树碎片化（深结构） |
| `:lang` / `:target`（In-document） | 4 | 伪类上下文域 |
| tree order（Fragment/In-document） | 2 | identity 域 |
| `Fragment: new NodeList` / Fragment body | 2 | 动态 NodeList 域 |
| Element-matches 侧 `[*|TiTlE]` | （另 3F） | 树碎片化 |

## 二、ns 词法层修复（本轮 land）

**根因**：`selector_lexically_valid` 的 `|` 分支只认**整串段首**（idx==0）的
显式空 ns 前缀——`#no-namespace |div`（后代段）被误判为未声明 ns 形态拒
（querySelector 抛 SyntaxError）。spec selectors-4：显式空 ns 前缀在**任何
段首**合法；仅**有名前缀**（`ns|div`——`|` 前是非空白 ident 内容）与 `||`
非法。

**修复**：`|` 前是空白/组合器符号（段首）→ 放行；`idx==0` 放行（原直接拒）；
`ns|div` 回溯停在非空白处仍拒。回归测试 `zz_r172_ns_forms_validity`
（8 合法形态 + 3 非法形态）。

**匹配层核实**：组合链 ns 段（`#any-namespace *|div`）在 dom crate 匹配
正常（`zz_r172_ns_chain_match_probe` 单测实证后转回归知识）。

## 三、ns 24F 的完整收口路径（树碎片化域，记档）

WPT probe 实证：`doc.getElementById('no-namespace')` 返 **null**——ns 用例的
锚元素是 `setupSpecialElements` **运行时 createElement+appendChild** 产物，
落在 per-element mutable tree（R159 已实证与 doc 查询树互不合并）。
**词法修复后 fail 形态从「SyntaxError」进到「合法但 0 命中」**——剩余收口
= 树碎片化统一（L2 深结构域，master.md R159 起多轮记录）。

## 四、验证

| 门 | 结果 |
|----|------|
| zero-dom | 852P/0F（含新回归测试） |
| 全量 dom WPT polyfill | **9522P/343F/18T**（= R171 逐计数一致——词法修复中性，fail 形态迁移） |
| `make test` | 66 套件 **18090P/0F** |
| fmt / clippy | 干净 |

## 五、下一步（R173）

- ns 24F 完整收口 = 树碎片化统一（setup 产物入 doc 查询树）——L2 深结构，
  需评估 setup 面（createElement+appendChild 的 host 同步）。
- 或 `:lang`/`:target` In-document 4F（伪类上下文）轻件评估。
