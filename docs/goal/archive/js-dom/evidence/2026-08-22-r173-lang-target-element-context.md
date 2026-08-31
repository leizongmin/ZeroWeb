# R173 Evidence — :lang/:target element 查询的文档上下文（M4）

**日期**: 2026-08-22
**Commit**: `c7e2c2a78`
**切片**: M4 轻件——ParentNode 剩 33F 中的 `:lang`/`:target` 4F 收口（+Element-matches 3F 顺带归零）

## 一、根因

Element 上下文查询把**元素子树序列化**后 host re-parse——丢两个文档级上下文：
1. `<html lang="en">` 祖先属性 → 重解析后 `:lang` 继承链断（`#pseudo-lang-div1`
   自身无 lang，继承 html 的 en）。
2. fragment URL → `:target` 恒 miss（doc 级 R160 已透传 `_zwFragmentUrl`，
   element 查询传空串）。

## 二、修复（两件）

| 件 | 内容 |
|----|------|
| **owner 溯源槽** `_zwOwnerTree` | `ensureTree` 建树时**全树盖**（与 root-only 的 `_zwOwnerDetDoc` 解耦——后者语义绑 root-hit 特判）；`ownerDocument` accessor 增读此槽——iframe doc 树内任意元素溯源到源 doc（旧回落主 document） |
| **`_zwMQueryAll` 上下文注入** | `:lang(` 形态（限真树成员）序列化源拼 owner 的 html lang 包装（`_r159HtmlAttrs` 优先 / documentElement.lang 回落）；`:target` 形态传 owner 的 `_zwFragmentUrl` 作查询 URL |

**Detached 排除**：clone 产物无 `_zwOwnerTree` 槽（stamp 只在树建时）→ 守卫
天然排除——spec 上脱离文档无祖先继承（首版无守卫时 Detached 版
"not matching element with no inherited language" 反转 fail 实证）。

## 三、过程记录（两轮实验）

1. 全文档源方案（owner body.innerHTML + 子树键过滤）：+8 回归（Detached
   上下文的 owner 树不含自身——ns/:lang 直值全破）→ 回退。
2. html-lang 包装方案（本版）：主 document 场景沙箱验证 → iframe 场景差
   owner 溯源 → 补 `_zwOwnerTree` 槽 → Detached 反转 → 槽守卫收口。

## 四、验证

| 门 | 结果 |
|----|------|
| ParentNode-querySelector-All | 33→**29F**（:lang 2 + :target 2 收口） |
| Element-matches / webkitMatchesSelector | 3→**0F**（owner 槽连带——`[*|TiTlE]` 3F 顺带归零） |
| 全量 dom WPT polyfill | **9532P/333F/18T**（R172 9522P/343F——**净 +10P/-10F**） |
| 全量 dom WPT native | **9531P/333F/19T**，per-file 与 polyfill 零差异 |
| `make test` | 66 套件 **18093P/0F** |
| fmt / clippy | 干净 |

## 五、下一步（R174）

- ParentNode 剩 29F：ns 族 24（setup 树碎片化域）+ tree order 2 + Fragment
  NodeList 2 + 其它 1。
- ns 24F 的 setup 树碎片化统一（R173 owner 槽是其中间基建——setup 产物
  createElement 后 appendChild 的树归属可循槽）。
