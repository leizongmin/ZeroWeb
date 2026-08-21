# R152b — remove-unscopable 全族 + Node-parentElement + Node-lookupNamespaceURI

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**commit**: `7c1de07c3`（rebase 前 `01bfdd125`；远端已有并行流 R152 `242b9b7d1` click 激活切片，本切片记 R152b）
**驱动用例**: `dom/nodes/remove-unscopable.html`（6 subtest）+ `dom/nodes/Node-parentElement.html`（12 subtest）+ `dom/nodes/Node-lookupNamespaceURI.html`（75 subtest）

## 根因与修复（三件）

### ① remove-unscopable 3P/3F → 6P/0F（三重根因）

- **根因 A（R134 误判修正）**：`setAttribute('on*', v)` 的 R134 失效+重编译路径**回读
  host 快照**（`__zw_get_attr`），而 `__zw_set_attr` 是异步 mutation 批处理（不立即落
  快照）→「重」编译出**旧 handler 体** → 六变体 dispatch 永远只跑首个体。R134 当年
  误判为「缓存未失效」。修复：`_ensureInlineHandler` 增 `codeOverride` 参数，R134
  调用点传刚写入的值 `v`（part03.js）。
- **根因 B**：元素 proxy 的 has 白名单（part05，R129）缺 `prepend`/`append` →
  `with(this)` HasBinding 跳过元素层 → 外层 `with(document)` 命中 `document.prepend`
  （ParentNode API，无豁免表）→ bare 名解析到 function。修复：白名单补两名。
- **根因 C**：document 自身无 `@@unscopables` 表——Document 同样实现 ParentNode
  mixin，真实浏览器两层都有表。修复：part06 document 挂
  `{prepend, append, replaceChildren: true}` getter。

诊断方法学：逐层最小 PoC（node 对照组 + 沙箱 `new Function('with(this){…}')` 最小
proxy 对照 `minsix`）把「引擎语义」与「shim 状态」分离，定位到 has 白名单与 document
层捕获。

### ② Node-parentElement 9P/3F → 12P/0F

- **fragment 子的 parentElement**：`_parentNodeFor` 的 handle 分支对 fragment 父
  （`_zwNodeParent[handle].parentHandle` 命中 fragment）返回 `_wrapHandle(fragment)`
  ——spec `dom-node-parentelement` 只返**元素**父。修复：`elementOnly` 模式判
  `_fragmentHandles[parentHandle]` 返 null。
- **`document.appendChild is not a function`**：shim document 从未有 appendChild
  （R117 的 append/prepend/replaceChildren 内部调它也是吞异常 no-op）。修复：补
  `document.appendChild`（spec pre-insert 校验：Text/Document→HRE、Comment/PI/
  Doctype→本地记账插入 `_zwNodeParent` 反链、Element→单元素约束+记账）。

### ③ Node-lookupNamespaceURI 0P/1F（页面级 error）→ 75P/0F

`document.appendChild` 补齐后用例从页面级 error 展开为 75 条断言级（暴露
`lookupNamespaceURI`/`isDefaultNamespace` 全缺）。实现 spec
`dom-node-lookupnamespaceuri` 完整算法（part03 `_zwLookupNamespaceURI` + 各节点形态）：

- **逐站 xmlns 扫描**：沿祖先链（sel 域 `__zw_parent` + handle 域 `_zwNodeParent`
  反链）每元素站扫 `_zwAttrInstances` 的 `xmlns`/`xmlns:<prefix>` 声明，最近声明胜出；
  `xmlns=""` 空串 = 显式默认无 ns → null。
- **元素自身 prefix→ns 映射**：`createElementNS('fooNS','prefix:elem')` 查 `'prefix'`
  命中自身 ns；**无 prefix 元素**（`'childElem'`）的自身 ns 即 default 声明；有 prefix
  元素的 ns 非 default（fooElem 查 null 期望 null）。
- **xml/xmlns 预绑定按分支**：仅**元素**起点（spec 元素分支硬规则）与 **Document** 站
  生效；detached fragment/doctype 查 'xml'/'xmlns' → null（WPT 对照断言）。
- **Document 查找**：default（无 prefix）恒返 HTML ns（document 自身 namespace，**不读**
  documentElement 的 xmlns 声明——WPT "Document should have xhtml namespace"）；有
  prefix 经 documentElement 声明 + 预绑定。
- **Attr**：经 `ownerElement` 委托；disconnected（ownerElement null）恒 null。
- **doctype**：恒 null / default 空。
- **detached doc（new Document()）**：经惰性 documentElement（无元素子 → null 恒 null，
  无预绑定）。

## A/B 验证

| 项 | polyfill | native |
|----|----------|--------|
| remove-unscopable | 6P/0F | 6P/0F |
| Node-parentElement | 12P/0F | 12P/0F |
| Node-lookupNamespaceURI | 75P/0F | 75P/0F |
| 全量 dom 套件 | **6253P/301F/18T**（baseline 6172P/308F/18T → 净 +81P/-7F，fail 集合 diff：消除 remove-unscopable 3 + parentElement 3 + lookupNamespaceURI 1，**零新增 fail**） | 同量级 |
| 单测 | r152b 两件全绿（unscopable 六族 / lookupNamespaceURI 20 断言族） | — |
| `make test` | 66 套件全绿（stale_etag_revalidation 偶发 flaky 一轮，复跑两轮全绿，网络域非本切片工作面） | — |
| fmt / clippy | 零警告 | — |

## 技术要点沉淀

- **Rust 多行字符串 + `//` 注释陷阱**：Rust 字符串字面量里**裸换行合法且保留换行**，
  测试里 `// 注释` 行末尾若误加 `\`（续行），拼接后换行被删 → `//` 注释吞掉同一物理行
  的全部后续代码 → `Unexpected end of input`。既有测试的注释行都以**无 `\` 的裸换行**
  结尾。本轮 lookupNamespaceURI 单测首版踩此坑，已修。
- **异步 mutation 快照滞后是系统性坑**：任何「setAttribute 后立即重读 host 属性」的
  shim 路径都会读到旧值（R134 根因 A 与未解问题 #4 同源）。

## 未收（记入 R153 候选）

- **Element-closest 4F**（`:scope`/`:has(> :scope)`/`select > :scope`/`:invalid`）：
  closest 的 spec 语义中 `:scope` = 调用元素自身，当前实现经文档级 querySelectorAll
  全匹配集（`:scope` 匹配 html）——需 per-callsite scope 注入（选择器引擎深结构，
  `closest_matching_selector` 须改为以 elem 为 root 的作用域匹配）；`:invalid` 需表单
  校验状态联动。非轻量切片。
- Attr-prefix 2F / MO-document 3F / realm·adopt 族（R152 候选 (b) 剩余）。
