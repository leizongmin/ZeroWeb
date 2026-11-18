# d38 gtr 家族双根因刀：链形查询 overlay 匹配 + runner 变体展开

日期：2026-11-16。范围：shim 查询链形补刀（part04/part05.js）+ runner 变体展开（testharness.rs）+ 引擎侧 diag 单测 3 件。

## 根因 A：同 turn innerHTML 后链形查询落 stale 宿主视图

gtr 家族 743F 中 crash 类 321F（`.firstChild` null）的机制。探针
`zz-probe-b2diag.html` P12 跨 turn 对照（同二进制修复前）：

```
OBS12 sameTurnChain=false nextTurnChain=true nextTurnDocChain=1 nextTurnPlainDt=2
```

同 turn（innerHTML 赋值后立即）链查询 null；60ms 后（mutation 已 apply）同一查询
命中——宿主视图滞后实锤。R309 注释自证架构（part04.js）：「同步 turn 内 innerHTML
替换经 host 异步 apply，快照查询对刚写入的子树恒 miss」——简单形有 R309
（`_parseCompoundOf` compound overlay 匹配）/R161（doc 级 tag 归并）pending 归并臂
可看 overlay，链形（组合器）无归并臂 → 落 stale host 快照恒 miss。

P10 快照同佐证：`docSimple=2 docPlain=0 elPlain=0`——doc 级简单形命中（归并臂）、
doc 级链形与全部 subtree 查询 0 命中（host stale）；`bodyInner` 显示 overlay 内容
完整在（overlay 与 host 双源分叉）。

## 根因 B：`_readCompoundToken` 不含组合器字符 → 无空格链伪 tag

探针 P14 四象限（同 overlay 同二进制）：

```
OBS14 nsFirst=null spSecond=hit nsPlain=null spPlain=hit nsChild=null
```

`p+p>b`/`p+p`/`p>b`（无空格）恒 null，`p + p > b` 等（空格）恒 hit——与查询顺序
无关。raw 桥两形态恒等（`__zw_query_all_sub` 均 ''）排除 host 层。定位
`_readCompoundToken`（part05.js:8736）break 条件缺 `+`/`>`/`~`：无空格链整串被单
token 吞成伪 tag `'P+P>B'` → `_parseCompoundOf` 返回 supported compound →
R322/R309 compound 臂恒 miss → qS/qSA null。空格版 token 止于空白 → 第二裸 token
→ unsupported → 正确落既有链臂路径（我方新加）。

## 修复点

1. `part05.js` `_zwOverlayChainAll(kids, chain)`：overlay 子树四组合器链求值
   （`_parseComplexOf` 产物 compounds+combinators，与 DOM crate
   query.rs::Combinator R3286 对齐；DFS 建父/前元素兄弟上下文表 + 逐段锚定收窄，
   树序输出）。
2. `part04.js` qSA R309 臂 else 分支：compound unsupported 时 `_parseComplexOf`
   成功 → overlay 链匹配全量（innerHTML 替换域 `_r309ih` 门内；伪类真 unsupported
   维持回落 host）。
3. `part04.js` qS R322 compound 门 else 分支：同款链匹配首命中（plain 代理返回，
   R54xx 下钻同款不物化；域判定 `_r309ih` 同口径）。
4. `part05.js` `_readCompoundToken` break 补 `+`/`>`/`~`（CSS Syntax
   <ident-token> 不含三者；`\+` 转义经既有 escape 臂先行）。
5. `testharness.rs` `run_corpus_subdirs_ext`：R329 同款变体展开（`case_variants`
   逐 query 跑、case 名 `format!("{relative}{variant}")`；无 variant 文件零影响）。
6. 引擎侧 diag 单测 3 件入树（修复依赖的引擎行为正确性守护）：
   - `crates/dom/src/tests/tests_9_query_coverage.rs::diag_chain_sibling_match`
   - `crates/engine/src/js_dom_bridge_tests/part01.rs::diag_bridge_query_all_chain_sibling`
   - `crates/engine/src/js_dom_bridge_tests/part01.rs::diag_parse_html_json_chain_sibling`

## 数字

- input-events 全语料：289P/875F/3T（25 files、1164 子测）→ **473P/2877F**
  （32 case、3350 子测；变体展开口径变化）。P +184。
- crash 家族（reading 'firstChild' 等 null-prop）：**321F → 1F**（残余 1 在
  non-collapsed-selection）。
- Unhandled variant：**488F → 0**。
- 新账结构：innerHTML-mismatch 1480、beforeinput 事件计数断言 908、
  targetRanges-assert 409——真实编辑语义面（B2 剩余/B3 输入）。

## 哨兵对拍（新旧二进制，shim stash 前后各自 release build）

- html-semantics：base（stash 前）5378P/461F/950 case vs 修复版
  5378P/464F/950 case——**P 恒等**；3 差异全落
  `select-home-end-pagedown-pageup*.optional` 键盘发射带内 flaky（s491 形差带同族：
  1 子测 Fail→Timeout + 4 子测注册数形差）。
- 本地 wpt-data 部分拉取态基线记档：本机 html-semantics 跑数 5378P 与 s492 收口账
  8307P（8516 子测）差异系本地语料不全，**非代码回归**——对照实验用的是同一本地
  语料态，恒等结论有效。
- 桥测试 `cargo test -p zero-engine --lib js_dom_bridge`：**1657P/0F**（含新 diag 2）。
- dom 单测 diag：1P。
- 探针终态：P1-P7/P13/P14 全 Pass（P13/P14 为 assert_true(false) 诊断输出形态，
  行为数据 hit/hit 全 hit）；P8（doc 级链同 turn miss）、P9（matches 链形）为范围外
  已知项（corpus 消费面 = 元素级 qS/qSA，未消费）。
