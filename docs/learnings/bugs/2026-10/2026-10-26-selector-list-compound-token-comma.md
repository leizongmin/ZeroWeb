---
date: 2026-10-26
modules: engine
---

# compound token 读取吞顶层逗号——s81 约束误剔致选择器列表查询恒空

## 问题描述

`document.querySelector(All)` 对含 `#` 的顶层逗号选择器列表恒空：
`["#post_list",".post-item"].some(s => document.querySelector(s))` 恒 false，
m1 冻结探针（50ms 轮询等待 `complete|true|…`）8/8 轮 45s 超时（cnblogs/baidu）。
单选择器恒真、带空格列表部分形态恒真——形态敏感：`#t,.c`→0、`#t, .c`→2、
`.c,#t`→1、`#t,#t`→0。host 侧（`__zw_query_all` → zero_dom
`query_selector_all`）原生支持列表且返回正确——缺陷全在 shim 查询面的
JS 后处理。

## 根因分析

两层叠加：

1. `_readCompoundToken`（compound token 读取，part05）的分隔集只有
   `.` `#` `[` `:` 与 CSS 空白——顶层 `,` 被当 ident 字符吞进 token：
   `_parseCompoundOf("#target,.cls")` 解析出 `ids=['target,']`、
   `div,#target` 解析出 `tag='DIV,'`。
2. 95c8f7503（s81）引入 `_zwSelIdConstraint`：为同 turn id 改名可见性，
   它提取「最终复合段 id 约束」对 host 命中做 latest-wins 复验（stale 剔除）。
   它把整条选择器串当**单 complex**（`_splitComplex` 按组合器切分、逗号是
   普通段字符），伪 id `target,` 不匹配任何元素 → host 全部命中被判
   violated 剔除 → 列表查询恒空。带空格列表 `.cls, #target` 也错：`_splitComplex`
   把 `.cls,` 当 `#target` 的祖先段，干净末段 `#target` 的约束误剔 `.cls` 段命中。
   95c8f7503 之前吞逗号无害——没有消费伪约束的读面，故 T4（48937037e）正常、
   回归窗口内 first-bad 可精确到该提交。

排障链：活体复现锁定「shim `document.querySelectorAll` 0 vs host
`__zw_query_all` 2」→ 直接调 `__zw_query_all("#target,.cls")` 证明 host 正确
→ 逐行读 `document.querySelectorAll.toString()` 找后处理臂 → 离线提取
part05 解析函数用 Node 执行，12 个形态逐一对上 live 矩阵定谳。

## 解决方案

共享路径四处，均按 spec（CSS Selectors 4 <selector-list>，逗号分隔 complex
selector，compound 内无顶层逗号）：

- `_readCompoundToken` 分隔集加 `,`（`\,` 转义仍走 escape 臂属 ident）——
  `_parseCompoundOf` 的全部消费臂（s81/R3902b/R322/R309/`_parseComplexOf`）
  统一落既有 unsupported 分支；
- `_zwSelIdConstraint` 入口对选择器列表 early-return null（复用既有
  `_splitSelectorListOf` 判顶层逗号）——补偿模型只对单 complex 成立，列表
  host 结果原样透出；单选择器改名补偿语义不变；
- `_splitSelectorListOf` 补 `(...)` 深度（返修 DF-1，对齐 host
  `split_top_level_selector_list`）——`:not(.a,.b)` 实参内逗号不是组边界，
  否则单 complex 被误判为列表、s81 门误 bail 丢同 turn 改名补偿；
- `_matchPseudosOf` 的 `:not` 臂列表感知（返修 DF-2，CSS Selectors 4
  <negation>）——实参按列表逐段求值「任一命中即否决」；unsupported 段保守
  否决；complex 实参（`div > p`）未实现标 FIXME 落 unsupported 否决。

评审发现的联动教训：DF-1 单独修（门内补括号）会把 `:not(…,…)` 形态从 host
兜底推进 shim 的 `:not` 臂——而该臂当时只会恒否决（DF-2），等于把「丢补偿」
换成「错结果」。同一解析面上的联动缺陷必须同批修，先核对修复后各形态
**实际走的臂**再下手。

## 避免方式

- 自研选择器解析器的 token 读取分隔集要对照 CSS ident 文法全集（不含 `,`），
  新增消费面（如约束提取/改写臂）时先用**列表形态选择器**过一遍解析断言。
- 「对 host 结果做剔除」的后处理臂，失效模式不是漏剔而是**误剔全灭**——
  新增此类臂必须带形态门（单 complex/单 compound/白名单），列表等不适形态
  显式 bail 到 host 行为。
- 逐形态矩阵（有无空格、正逆序、重复段、单/列表、括号实参内逗号）是查询面
  回归的最低覆盖；修前红（0≠2 断言）修后绿要同提交留证。
