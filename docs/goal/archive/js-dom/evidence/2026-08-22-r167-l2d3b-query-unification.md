# R167 Evidence — L2-d3b 查询产物归一（identity 桥消费）落地（M1）

**日期**: 2026-08-22
**Commit**: `5258ab632`（rebase 后；含并行流 CacheStorage 两提交）
**切片**: M1 L2-d3b——三查询面产物经桥归一 + 四件配套修复（matches 语义 / 链派发 / ownerDocument / listener 槽）

## 一、核心落地

三查询面的 wrapper 工厂加「桥归一前置」——构造 `_zwParseEl` 前，先经共用
`_zwMFindRealNode`（root 子树 tag+id+outer 键索引，R163 fragment nodeIdx 的
共用化提取，挂 root `_zwNodeIdx` 槽缓存）找同键真实 mutTree 节点，命中即返
其桥对象：

| 面 | 归一处 |
|----|--------|
| doc 级（queryOne/queryAll→`_zwWrapCached`） | `ensureTree()` 后查 `_tree` 索引 |
| Element.prototype（`_zwMWrapCached`） | 查 root（this）子树索引 |
| fragment QSA | R163 nodeIdx 换 `_zwMFindRealNode` + miss 补登桥 |

**语义不变式**：JSON 往返仍是查询语义权威；归一只改产物 identity。

## 二、全量 A/B 驱动的四件配套修复（每件都是归一暴露的真缺口）

1. **Element.prototype.matches 重写**（Element-matches 305F 大回归的根因×2）：
   - 查询源从「自身 outerHTML 单元素序列化」改「沿 parentNode 找最近可序列化
     根，对整根查询」（组合器 `#universal>*` 对单元素必 miss——wrapper 形态
     靠 `_zwRootHtml` 凑巧成立，归一产物无此机制）。
   - 自身判定从「arr[0] 的 id+tag 弱键」改「结果集按 id+tag+outer 强键找自身」
     （多命中时 arr[0] 是文档序在前的其它节点）。
2. **`_zwMEl.dispatchEvent` 链派发**（spec concept-event-dispatch 三阶段）：
   链沿 parentNode 上行；**B 域 proxy 入链不续上行**（proxy 世界有独立派发
     管线 `_dispatchWithBubble`+R114 shadow 语义，交叉 = lit e2e 双触发
     count:2 实证）；proxy 站 fire 其 `_listenerStore` + tag-registry 视图
     兜底 + 宿主 doc 接续；树根经 `_zwOwnerDetDoc` 印章接回 detached doc 的
     body/docEl/doc 站。capture 循环从链顶（doc）起（旧 length-2 起跳过 doc
     致顺序反转）。
3. **listener 槽化**：`_mEvListeners` 从闭包变量改挂节点 own 槽 `_zwMEvLs`
   （链派发对**中间站**的 C 节点 fire 时读槽——闭包跨节点不可见）。
4. **ownerDocument accessor**：`_zwMEl` 工厂出口 defineProperty（getter：
   own-write > `_zwOwnerDetDoc` 印章 > 主 document 回落；setter：显式赋值转
   own 数据属性——detached createElement 的 `e.ownerDocument = doc` 旧是
   plain 赋值，getter-only 会静默失败致 R126 syn-el od:false→true 回归）。
   `ensureTree` 建树时全树 stamp `_zwOwnerDetDoc`。

## 三、过程负结果与二分记录（对 d3b2 有直接输入价值）

- **链派发首版**：lit e2e `count:2` 双触发（`disp:1|inc:2` 探针实证单次
  dispatch 双 handler）→ 二分定位 = 链进 proxy 站 → 修 = proxy 止链后改
  「入链不续上行 + listenerStore 委托」。
- **matches tagOf 修复不足**：`arr[0].tag||tagName` 只修弱键一半——组合器
  查询源问题独立存在（`#universal>*` 簇），需整根上下文。
- **capture 顺序**：14 站到齐但顺序反（Document 在末）= capture 循环起点
  off-by-one。
- **残留 2F×2 变体**：`In window.document.cloneNode(true)` / `In new
  Document()`（12 vs 14 站，缺 1 站）——`new Document()` 的 docEl 是 plain
  未经 `_zwWireLocalEvents` 接线，`documentElement` 惰性 getter 返回内部
  docEl 与链上 clone html proxy 身份错位；tag registry 兜底对该形态不生效。
  **记 d3b2 首项**。

## 四、验证（全量）

| 门 | 结果 |
|----|------|
| 全量 dom WPT polyfill | **9518P/347F/18T**（基线 9516P/347F/18T——净 +2P、F 持平） |
| — ParentNode-querySelector-All | 1936→**1942P**（+6P）、37→**33F**（-4F） |
| — Event-dispatch-bubbles | 各 5P→3P（cloneNode/new Document 两变体 4 subtest 转 Fail——d3b2） |
| — Element-matches / webkitMatchesSelector | 回基线 3F（中途 305F 已全修） |
| 全量 dom WPT native（ZW_NATIVE_DOM=1） | **9517P/347F/19T**，per-file fail 与 polyfill **零差异**（±1P/1T 边缘漂移） |
| `make test` | 66 套件 **18056P/0F**（SW 域 3 个不同 flake 单跑绿，webview 零改动，归并行负载） |
| fmt / clippy（v8 workspace + quickjs 矩阵） | 干净 |
| shim 语法 | acorn 全拼接解析 OK（每步改动后验证） |

## 五、下一步（R168）

- **d3b2**：bubbles 残留两变体（new Document/cloneNode docEl 身份错位——
  `_zwWireLocalEvents` 接线补全或 docEl getter 身份映射）。
- **d3c**：doc 上下文 compound gate（queryBody 形态门扩 `_queryTreeByCompound`
  全形态——R165 实证 doc 上下文无回归）。
- **d3d**：element/fragment 本树化（R165 902F 回归面在此被桥消解）。
