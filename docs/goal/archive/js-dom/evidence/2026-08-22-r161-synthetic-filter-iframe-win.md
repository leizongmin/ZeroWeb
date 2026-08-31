# R161 Evidence — 合成容器过滤 + iframe win 构造器 + QSA pending 回落（M4 nodes）

**日期**: 2026-08-22
**Commit**: `874c50604`
**切片**: M4 — ParentNode-querySelector-All 剩余分散簇（1917P→1937P；全量 9490P/373F→9510P/353F 双路径一致）

## 三件修复

### 1. 元素子树查询的合成容器过滤（~12F）

html5ever 解析任意片段都补 **html/body 合成容器**——元素子树查询
（`_zwParseEl`/`_zwMEl`/fragment 的 outerHTML 序列化 → `__zw_parse_html_query`
重 parse）中 `querySelectorAll("html")` 命中合成祖先（WPT element 上下文
expect 0——spec 子树查询不含根外元素）。

- `parse_html_element_json_full`（新全参变体）：`filter_synthetic` 剔除源串
  无 `<html`/`<body` 开标签时的对应容器命中
- `__zw_parse_html_query` arg[5] 透传；**仅三个元素/fragment 子树调用点 opt-in**
- doc 级 detHtml（R159 的 `<html ...><body ...>` 真实包装）与 DOMParser 的
  合成 body 导出不受影响（默认 false——engine 2307 全绿验证）

### 2. iframe window 构造器（~4F）

`windowFor(root).NodeList` 的 instanceof 判定——`_zwMakeIframeWin` 缺
NodeList/HTMLCollection 构造器（旧 "Right-hand side of 'instanceof' is not
an object"）。从父 global 转发（与 EventTarget 同款 R140 模式）。

### 3. querySelectorAll tag pending 回落（~4F）

R145 的 querySelector 单点回落镜像到 QSA：host 快照 miss 的纯 tag 查询扫
`_zwPendingAdded`（同 turn append 的元素——WPT `querySelectorAll(null)` 对
setup 添加的 `<null>` 元素 expect 1）；identity 去重（handle/id 键）。

## A/B 双路径

全量 dom WPT **9510P/353F/18T**（polyfill 与 native 逐计数一致）。
vs R160（9490P/373F）：**+20P/-20F**。六轮累计（R156 起 6290P）：**+3220P**。

## 未收（R162 候选）

- ns 选择器族 + `[*|TiTlE]` 第 3 命中（树碎片化域——L2 正解）
- tree order（QSA 结果与文档序差异——host 返回序 vs DOM 序）
- "Fragment: new NodeList"（动态 NodeList 语义——长度随 DOM 变化）
- Element-matches 剩 ~14F（`:lang`/`:nth-child` 上下文）

## 验证

- `cargo test -p zero-dom`：849 全绿；`cargo test -p zero-engine`：2307 全绿
  （含 DOMParser 合成 body 依赖三件——filter 默认关验证）
- `make test` 全绿；fmt/clippy 干净
