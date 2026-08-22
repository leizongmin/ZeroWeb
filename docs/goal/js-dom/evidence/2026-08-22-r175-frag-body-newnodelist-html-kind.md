# R175 Evidence — Fragment body 过滤 + new NodeList + iframe kind 判定（M4）

**日期**: 2026-08-22
**切片**: M4 轻量——ParentNode 剩 5F 中 3F 收口（5→2F，全量净 +3P/-4F）
**改动面**: part03（fragment QSA 子串直发）+ part04（iframe kind 判定）+ part05（'html' kind contentType 分支）+ engine 单测

## 一、Fragment body 2F

**根因**：fragment QSA 的序列化源包了字面 `<body>...</body>`——host
`filter_synthetic` 的 `contains("<body")` 判源含 body 标签 → 合成容器命中被
保留 → `body` 选择器误返 1（WPT Fragment "Type selector, matching body
element" expect 0——fragment 无 body）。

**修复**：子串直发（不包装）——html5ever 解析裸片段仍补合成容器，但源串
无 `<body` 标签，`filter_synthetic` 正确剔除。host 侧行为由单测
`zz_r175_frag_body_filter` 守住（合成 body 过滤 + div 正常命中双断言）。

## 二、Document: new NodeList 1F

**根因链**（DBG 探针逐层实证）：append 后新查询计数不增（102 vs 103）→
walk 命中 102 但新 div 不在 → 新 div `nodeName` 是**小写** 'div' ≠ want
'DIV' → part05 `_zwIframeCreateElement` 的 isHtml 分支没走（iframe doc
contentType 是 'application/xhtml+xml'）→ **R156 的 kind 判定把 `.html`
文件也判 'xhtml'**。

**spec 核实**：真浏览器按扩展/Content-Type——`.html` → HTML 文档
（contentType 'text/html'、createElement tagName 大写）；`.xhtml` → XHTML
（XML 语义大小写保持——WPT Document-createElement "XHTML document" 断言
tagName 原样）。R156 当时为修 body-empty 把 .html 并入 html 变体但选错了
kind 值。

**修复**：kind 三分——`.xhtml` → 'xhtml'、`.html` → 'html'、其它 → 'xml'；
part05 增 'html' 分支（contentType 'text/html'，createElement 大写路径自
然激活）。`kind !== 'xml'` 的 html-attrs 保真分支对 'html' 同样适用。

## 三、验证

| 门 | 结果 |
|----|------|
| ParentNode-querySelector-All | 5→**2F**（1972→1974P）——剩 tree order 2F（identity 深结构域，R171 已评估） |
| 全量 dom WPT polyfill | **9557P/307F/19T**（R174 9554P/311F——净 +3P/-4F；event-global flaky F↔T 互换 net 0；零回归） |
| 全量 dom WPT native | **9557P/307F/19T**，per-file 与 polyfill 零差异 |
| engine 单测 | `zz_r175_frag_body_filter`（host 过滤语义回归） |
| `make test` | 66 套件 **18118P/0F**（首跑 SW 已知 flake 1F，二次全量绿——R167 起观察项，归 service-workers 流域） |
| fmt / clippy | 干净 |

## 四、下一步（R176）

- tree order 2F = identity 深结构（traverse 真节点 vs QSA 产物归一——R171
  评估结论：element 上下文本树化 0 改善，须 L2 产物归一路径统一，记 RFC 域）。
- Document-createElement-namespace 40F 既存簇（iframe XML/XHTML 文档域）。
- M2/M6 面：S6 高层 API 去字符串 / native dom_bindings 补齐。
