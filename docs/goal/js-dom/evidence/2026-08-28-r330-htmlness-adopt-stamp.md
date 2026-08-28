# R330 Evidence — getElementsByTagName 的文档 HTML-ness 语义（HTMLNess 全文件转绿；三层修复：htmlCtx 捕获 + docEl/synthetic adopt 印记 + qualifiedName 精确匹配）

**日期**: 2026-08-28
**切片**: M4——R330(a) 备档集巡检：Element-getElementsByTagName-change-document-HTMLNess 1F
**改动面**: `part05.js`（`_zwFilterByTagNameNS` htmlCtx 参数 + `_zwCtxIsHtmlDoc` helper + `_nsHandles` qualifiedName 精确匹配 + `_r177syntheticHtml` adopt 传播）+ `part03.js`（detached docEl appendChild adopt 传播）+ `part04.js`（getElementsByTagName 调用点透传）

## 一、根因（三层叠加）

WPT `Element-getElementsByTagName-change-document-HTMLNess`（single_test，5 断言序）：
`createElementNS` 混合 ns 四元素 → HTML 文档查 `getElementsByTagName('A')` 期望
`[n1(xhtml a), n4(no-ns A)]` → 移入 XML iframe 文档后**旧 live list 仍 HTML 语义**
（`[n1, n4]`）→ **新查询**按 XML 语义精确比较（期望 `[n2(xhtml A), n4]`）。

1. **HTML-ness 按元素 ns 判定而非查询上下文文档**：`_zwFilterByTagNameNS` 旧版把
   `namespaceURI null/undefined` 一并视为 HTML ns（isHtml），使 no-ns 元素误走大小写
   折叠（n3[no-ns 'a'] 被 'A' 命中——断言 1 即败）。
2. **移入子文档无 adopt 印记**：`frames[0].document.documentElement.appendChild(parent)`
   走 `_r177syntheticHtml`（R177 合成 docEl，空 markup 文档）appendChild——旧无
   R191 adopt 子树传播，`parent.ownerDocument` 回落主文档（探针 odCt=text/html 实证），
   新查询的上下文 HTML-ness 判定无从谈起。detached docEl（part03 7811）同缺。
3. **qualifiedName 被创建期烘焙**：`_nsHandles` 的 `htmlUpper` 按 ns 在 createElementNS
   时烙定（HTML ns → 大写 tagName），移入 XML 文档后 wrapper tagName 仍 'A'——
   spec 的 tagName getter 按当前 node document 动态大写，XML 文档应返原值 'a'，
   精确比较 'A' 不应命中 n1（断言 3 败）。

## 二、修复（每层一行语义）

1. **`_zwFilterByTagNameNS(els, input, nsArg, htmlCtx)`**：HTML-ness 折叠 = 查询上下文
   文档 HTML-ness（`htmlCtx`）**与**元素 HTML ns 的合取；NS 变体不变（localName 恒
   精确）。
2. **`_zwCtxIsHtmlDoc(sel, handle)`**：查询时从 context object 的 ownerDocument 捕获
   （adopt 印记优先，R191 同源）`contentType === 'text/html'`，缺省 true（主文档）。
   live matches 闭包同捕获（live 判定与构建期语义一致——spec 旧 list 保持 HTML 折叠）。
3. **qualifiedName 原值精确匹配**：`_nsHandles[handle].qualifiedName` 优先于烘焙 tagName
   （仅精确比较分支；HTML 折叠分支不受影响——`htmlUpper` 烘焙值折叠后与原值等价）。
4. **adopt 传播两处补齐**：`_r177syntheticHtml.appendChild`（part05，iframe 空文档域）
   + detached docEl.appendChild（part03，`_makeDetachedDocument` 域）——与 R191/R112
   同构（handle 落 `__zwAdoptDocByHandle`、plain defineProperty ownerDocument getter）。

## 三、A/B

| 项 | R329 基线 | R330 | Δ |
|---|---|---|---|
| Element-getElementsByTagName-change-document-HTMLNess | 0P/1F | **1P/0F** | +1 |
| getElementsByTagNameNS 30P / Document-getElementsByTagName 32P / Element-getElementsByTagName 36P / case.html 285P / createElementNS 596P / cloneNode 144P / closest 29P / QSA 1976P / matches 675P / classList 族 | 全绿 | 全绿 | 零回归 |
| Element-getElementsByClassName 2P/1F | R319 备档（live×activation 深域） | 同 | 既存维持 |
| **全量 dom sweep**（TIME_LIMIT=2400） | 54150P/53F/23T | **54151P/52F/23T** | **Fail set 恰 -1（HTMLNess）零新增** |
| engine --lib（v8/quickjs） | 2468/1466 | 2468/1466 | 零回归 |
| clippy（engine/wpt-runner 双矩阵） | 干净 | 干净 | — |
| fmt | — | 无 diff | — |

## 四、方法论

- **指纹探针定分派域**（R327 硬性流程第三次应用）：`String(docEl.appendChild)` 特征
  串实证 iframe 空 XML 文档的 documentElement 走 `_r177syntheticHtml` 域（非
  `_makeDetachedDocument` docEl 字面量——两者 appendChild 均无 adopt 传播，逐域补齐）。
- **探针分步归因**：断言 1（no-ns 折叠）→ 断言 2（旧 list 语义）→ 断言 3（新查询语义）
  逐层推进，每层修复后重跑定位下一层——单测文件 single_test 顺序断言天然分层。

## 五、残余（本切片不追）

- spec 的 tagName getter 动态大写（按当前 node document）未实现——wrapper 烘焙值 +
  qualifiedName 原值优先的精确匹配组合覆盖了本用例与既有 NS 簇；完整动态化归 L2。
- Element-getElementsByClassName live×activation 交互（R319 定性备档）维持。
