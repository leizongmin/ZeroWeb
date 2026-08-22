# R174 Evidence — ns setup 树碎片化统一：iframe 工厂反射 + no-ns 标记协议（M4）

**日期**: 2026-08-22
**切片**: M4 轻量——ParentNode ns 族 24F 完整收口（29F→5F，全量净 +22P/-22F）
**改动面**: part05（iframe 工厂 IDL 反射）+ part03/part06（序列化标记）+ zero_dom（标记还原 API）+ engine bridge（接线）

## 一、根因链（探针实证）

WPT `ParentNode-querySelector-All` 的 `setupSpecialElements` 在 iframe doc 上
`createElement` / `createElementNS` + `el.id = x` + `appendChild` 建 ns 测试元素。
三个串联缺陷使 doc 查询看不见这些产物（「树碎片化」的具体机制）：

1. **iframe 工厂元素无 IDL 反射**——`_zwIframeCreateElement` 产物是 plain
   object，`el.id = "any-namespace"` 写数据属性不进 `attributes` 数组 →
   `_zwMSerialize`（只读 attributes）序列化丢 id → `#any-namespace` 族全 miss。
2. **iframe 工厂元素无 outerHTML**——`_queryTreeByCompound` walk 时
   `String(c.outerHTML || '')` 为空触发 abort 守卫 → compound 门整体打回
   JSON 往返。
3. **序列化丢 ns**——`createElementNS("", "div")` 产物（EmptyNs 匹配域）经
   `_zwMSerialize` → host re-parse 后 ns 恒 HTMLNS → `|div`/`|*`
   （NsKind::EmptyNs）恒 miss。

## 二、修复（三层）

| 层 | 内容 |
|----|------|
| **part05 iframe 工厂** | ① `id`/`className` IDL 反射 accessor（get 读 attributes / set 写 attributes——空串删属性）；② `removeAttribute`（反射的空串分支消费）；③ `outerHTML` getter（复用 `_zwMSerialize`）；④ `appendChild` 对齐 `_zwMEl`（adopt 摘原父 + 入树清移除标记） |
| **no-ns 标记协议** | `_zwMSerialize`/`_zwMOuterHtml` 对 `!namespaceURI`（WebIDL 下 null/"" 归一）元素输出内部属性 `data-zw-empty-ns=""`；host `parse_html_element_json_full` 查询前经 `Document::apply_empty_ns_markers` 还原（ns 置空 + 剔标记属性，对快照零污染） |
| **ns 值语义纠正（回滚）** | 首版把 `namespaceURI = _nsStr \|\| null` 改为保空串——全量 A/B 抓到 createElementNS/createDocument 族 +97F 回归（WPT "empty string namespace" 断言 `=== null`，WebIDL DOMString? 归一）；回滚为 null 语义，标记协议判据改 `!namespaceURI` 覆盖匹配域 |

## 三、过程记录

1. 探针（zz-r174-ns-probe，wpt-data gitignored 临时域）逐层实证：id 反射破
   → append 后 doc qsa 0 命中 → `|div` 0 命中 → 修复后逐层转绿。
2. **A/B 门抓回归**：ns 值语义首版全量 408F（+75 vs R173 333F），diff 归因
   97F 集中在 ns 值断言族 → 纠正回滚 → 311F（净 -22）。
3. `#any-namespace *|div`（8F）仅靠修复 ①② 即收口；`|div` 族（16F）需
   标记协议全链路。

## 四、验证

| 门 | 结果 |
|----|------|
| ParentNode-querySelector-All | 29→**5F**（1942→1971P）；ns 族 24F **全收口**；剩 tree order 2 + Fragment body 2 + Document: new NodeList 1 |
| 全量 dom WPT polyfill | **9554P/311F/18T**（R173 9532P/333F——**净 +22P/-22F**，97 首跑回归全回滚、`>` 侧零新增） |
| 全量 dom WPT native | **9553P/311F/19T**，per-file 与 polyfill 零差异（唯一分歧 insertBefore-iframe-crash 为已知 flaky 超时） |
| zero-dom 单测 | `zz_r174_empty_ns_marker_roundtrip`（标记还原 + 未标记元素零影响护栏） |
| `make test` | 66 套件 **18107P/0F**（首跑 SW 已知 flake 1F，二次全量绿——R167 起观察项，归 service-workers 流域） |
| fmt / clippy | 干净 |

## 五、下一步（R175）

- ParentNode 剩 5F 小簇：tree order 2（QSA 结果序 vs 文档序）/ Fragment
  body 2 / Document: new NodeList 1。
- Document-createElement-namespace 40F 既存簇（R173 基线即有，iframe
  XML/XHTML 文档域）。
- M2/M6 面：S6 高层 API 去字符串 / native dom_bindings 补齐。
