# R306 Evidence — doc 作用域裸 `*` 的结构元素归位（tree-order Document 上下文全解）

**日期**: 2026-08-27
**切片**: M4/L2——R306(a) R305 诊断的低成本路径落地
**改动面**: `part03.js`（`queryBody` 裸 `*` 走 JSON 全树 + R296 结构桥）+ `part24.rs`（+1 回归单测）

## 一、修复

R305 诊断的 Document 上下文 idx0（`trav=HTML/res=META`）：iframe contentDocument
的 `querySelectorAll("*")` 走 `_queryTreeByCompound` 的 **body 内容树**路径
（`_tree` 无 html/head 层）→ 结构元素恒缺席。

**R305 的「host `*` 返 1」假设被本轮探针推翻**：`__zw_parse_html_query
(detHtml,'*','1')` 实返 6 元素以 html 起（上一轮实验失败是 `*` 被解析为
`tag='*'` 使 `!comp165.tag` 判据 miss 的 shim 侧 bug，非 host 缺口）。

修：`queryBody` 增 `isBareUniversal` 判据（tag 为 `'*'` 或 null 且无
id/class/attr 约束）→ 跳过树路径走 JSON 往返（detHtml 全树文档序），经
`_zwWrapCached` 包装时 **R296 结构桥**把 html/body/head 归一到 doc 视图对象
（`documentElement`/`body`/`head`——与 traverse 的 firstChild/nextSibling 读
identity 一致）。

探针（engine 单测断言）：`dd=7|ddFirst=HTML|bridge=true/true|json0=html`——
结果以 html 起、html/body 与视图对象 identity 全等。

## 二、验证

| 套件 | 基线 | R306 | Δ |
|---|---|---|---|
| ParentNode-querySelector-All | 1971P/4F | **1973P/3F** | +2P/-1F（**Document tree-order 全解**；余 Detached/Fragment/In-doc = cloneNode identity 深结构） |
| ParentNode-querySelector 全族 | 2050P/5F | **2051P/4F** | +1P/-1F |
| Element-matches | 675P/0F | 同 | 持平 |
| traversal | 1603P/1F | 同 | 持平 |
| 主 document `*`（sel 域 host 路径） | main=4 | main=4 | 持平（判据只影响 detached-doc 工厂域） |
| engine 单测 | 2442 | **2443**（r306：ddFirst=HTML + bridge identity + json0=html 三断言） | +1 |
| make test | — | 1F = XOpenDisplayFailed 环境项 | 持平 |
| fmt / clippy | — | 干净 | — |

## 三、剩余（R307）

tree-order 剩 3F（Detached/Fragment/In-doc）= **cloneNode 树与查询包装的
identity 归一**（R291 深结构域——traverse 读 `element.cloneNode(true)` 工厂
克隆树 vs 查询走 JSON wrapper/handle registry 两套对象）。须先评估 R171 桥
扩展的 blast radius（902F 依赖历史）。

## 四、教训

上轮 R305 的「host `*` 返 1」归因**错误**——那是实验自身的 shim 判据 bug
（`*` → `tag='*'` 使 `!tag` 恒 false）。**探针要打到被测层的正下方**（本轮
`__zw_parse_html_query` 直测一步证伪），诊断结论在修复尝试前应先过这层验证。
