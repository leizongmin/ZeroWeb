# M3 片 b——serializing 邻面全绿（2026-10-03）

## 结果

**serializing-html-fragments 域 137/137 全绿** + **serializing-xml-fragments 域
112/112 全绿**（本轮起点 0/112）。全通道 46316/61303 = **75.55%**（片 a 收口
75.36%），对 M1 基线 per-case 逐案比对 **0 回归**。DC-4 实测：make test 68 suites
全绿 + clippy `-D warnings` 干净 + fmt + reftest **704/704** 零不一致。

## 修齐面 → 根因 → 修法

| 用例面 | 根因 | 修法 |
|---|------|------|
| template.html（2 失败） | ① handle 路径 content 视图 `_r145SelKids` 缺册时返回临时数组——`content.appendChild` 的 push 落在弃置数组；② `__zw_get_inner_html` 读链 live-aware 直读 live doc，同 turn InsertAdjacentHtml 烘焙的 template（含 contents）缺席——查询面 view doc 有、读链 live 无 | ① `_r145SelKids` 就地建册（读写共享活数组）；② `__zw_get_inner_html` 切 `with_query_view_doc`（视图文档）——宿主 serializer 本就有 template contents 分支（innerHTML/outerHTML 双向），读链切视图后两断言直接绿；无 pending structural mutations 时同一 live fast path（零空闲成本），Remove/SetInnerHtml 仍排除在烘焙外（R3029 语义不变） |
| processing-instructions.html（3 失败） | 三处序列化循环（innerHTML getter handle 分支 / R380 sel 融合分支 / `_zwMSerialize`）均缺 nodeType 7 分支 → PI 子序列化输出空 | 补 PI 分支：`<?` + target + ' ' + data + `?>`（spec fragment serializing——空 data 不特判，分隔空格恒在） |
| serializing-xml-fragments/outerHTML.html（111 失败） | XML 文档元素（createDocument 产 `createElementNS` HTML ns 元素）走 HTML 序列化——无 xmlns 声明、void 无自闭合斜杠 | 新增 `_zwXMLSerialize`（DOM-Parsing §3.2.1 语料面：inherited≠自身 ns → xmlns 声明；HTML ns void → ` />`；非 HTML ns 空 → `/>`；名字大小写敏感）+ 接线三点：`_zwMSerialize` 头部 XML 域分支（ownerDocument.contentType 非 html 族）、outerHTML getter handle 分支同判定、`_zwParsedDoc` 轻量元素 outerHTML（DOMParser XML 文档域）+ attributes 视图 |
| outerHTML.html 'Node for canvas'（1 失败） | standalone canvas 对象（`_zwMakeCanvas`，R57）无 outerHTML/innerHTML 属性 → 读 undefined | 补二 getter 委托 handle 代理（`<canvas></canvas>`） |

## 架构注记

`__zw_get_inner_html` 切视图文档是「解析插入子树同 turn 读一致性」架构片的首块
落地（R57 view doc 读链对齐）——ambiguous-ampersand 残差的另一半
（`__zw_child_nodes` live-aware）本轮**未动**（其失败形态还叠加 char-by-char 流
语义，留独立片评估）。挂账清单相应收窄。

## 门禁

- make test（TIME_LIMIT=1800）：68 suites 全绿（0 failed / 0 panic）。
- make guarded-clippy（-D warnings）：干净；cargo fmt --check 干净。
- make reftest：**704/704**（Layout 502 + Text 202）零不一致。
- 全通道 46316/61303 = 75.55%；M1 基线逐案比对 0 回归。
