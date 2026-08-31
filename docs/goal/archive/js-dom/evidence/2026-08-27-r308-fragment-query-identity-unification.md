# R308 Evidence — R291 桥首切片：iframe 工厂域查询 identity 归一（tree-order 全解 + Element-matches 139F 全解）

**日期**: 2026-08-27
**切片**: M4/L2——R308(a) R291 桥首切片 + R308(b) Element-matches 归因（连带全解）
**改动面**: `part03.js`（×4）+ `part05.js`（×2）+ `part24.rs`（+2 单测）

## 一、成果总览

| 套件 | 基线（main = R307 后） | R308 | Δ |
|---|---|---|---|
| ParentNode-querySelector-All.html | 1973P/3F | **1975P/0F（100%）** | **+2P/-3F（tree-order 四上下文全解）** |
| ParentNode-querySelector 全族 | 2051P/4F | **2053P/1F** | +2P/-3F（余 1F = removed-elements 既存） |
| Element-matches.html | 530P/139F（main 既存回归） | **675P/0F（100%）** | **+143P/-139F（Fragment matches 簇全解）** |
| Element-webkitMatchesSelector | 669P/0F | 同 | 持平 |
| Node-cloneNode / Document-createElementNS / Document-createElement | 145/596/794 P 全 0F | 同 | 持平 |
| Node-properties / Node-insertBefore / closest | 724P/2F / 40P/0F / 29P/0F | 同 | 持平 |
| engine 单测 --lib | 2444 | **2446** | +2（r308 iframe identity + r308 fragment matches 断言） |
| make test | 1F 环境项 | 同 | 持平 |
| fmt / clippy（-p zero-engine -D warnings） | — | 干净 | — |

## 二、六层修复（探针逐层定位，每层一个真实 WPT 断言形态）

R307 已把工厂域（createHTMLDocument）identity 修全等；本轮指向 WPT 本体的 iframe
contentDocument 域（`doc = frame.contentDocument`，src 加载 ParentNode-querySelector-All-content.html）：

1. **`_zwIframeCreateElement` 出口 identity 桥登记**（part05）：iframe 工厂 createElement
   产物是 plain 字面量（不经 `_zwMEl`），`_zwBridgeGet` 恒 miss → append 进工厂树后查询
   键命中但桥断 → 回落 wrapper（探针 `wrap:null:real=true:bridged=false`）。补
   `_zwBridgeSet(el, el)`（与 `_zwMEl` 出口同源）。
2. **iframe 工厂元素的祖先查询索引失效**（part05）：appendChild/removeChild/insertBefore
   后沿父链 `_zwNodeIdx = null` + `_zwQWrapMap.clear()`（与 R307 ④ `_zwMEl` 同款）。
3. **`_zwMWrapCached` 桥 miss 补登**（part03）：桥表被 doc 级操作（`_zwQWrapBump`/
   innerHTML 写点）整表清空后，先于清空创建的节点查询时 miss → wrapper。键命中即
   节点本体，miss 补登（fragment 路径 R163 的同款自愈）。
4. **`_zwMOuterHtml` 的 void 元素无闭合标签**（part03）：与 `_zwMSerialize` 的
   `_ZW_VOID_TAGS` 表对齐——旧恒输出 `</hr>` 使含 void 后代的元素 walk 键与 host
   JSON `.outer` 恒失配（WPT Detached idx1 `#universal` 含 `<hr>` 的探针 diff@172）。
5. **`_zwMOuterHtml` 文本 `>` 转义**（part03）：与 `_zwMEscapeText`/host 的 `&<>`
   三字符转义对齐（WPT Fragment `</div>>` vs `</div>&gt;` 探针 diff@9217）。
6. **fragment 查询的同键重复 identity 分离**（part03）：`id-li-duplicate` 族同构元素
   的出现序号后缀（R188 的 fragment 版）。
7. **`_zwMQueryAll` 对 DocumentFragment 根的序列化源直发**（part03）：nodeType 11 的
   `_zwMOuterHtml` 返 '' 使 matches 的 root-up-track 上行到 fragment 后候选恒 0——
   **Element-matches 139F（main 既存回归，R306 evidence 表记过期）的单一根因**。

## 三、探针方法（真实 WPT 文件注入 + fetch 覆盖规避）

- R302 教训复用：`make testharness-dom` 依赖 `fetch-wpt-dom` 会**重新拉取覆盖注入**——
  注入后**直接跑 release binary**（`./target/release/zero-wpt-runner testharness-dom
  <file>`，test-guard 包裹），跑完 `cp backup restore`。
- 增量定位：first-div 探针（qa vs trav 逐位首分歧 + 双侧 outerHTML JSON.stringify 转义
  diff@N）+ kind 探针（factory/handle/sel/wrap/plain 五域指纹）+ 桥 dbg 注插（临时，
  已还原）。
- **每层修复后重建 binary**（`cargo build --release -p zero-wpt-runner`）——本轮一次
  「修复无效」的假象就是 binary 未重建（shim 是 include! 进 binary 的）。

## 四、域状态更新（R291 域大幅收窄）

R305 定性的「R291 wrapper→视图归一深结构（R171 902F 依赖）」经 R307+R308 两轮七层
轻量修复后：**ParentNode-querySelector-All 与 Element-matches 两套件 100%**，iframe
contentDocument 域的查询/traverse/matches 三面 identity 已归一。R291 域剩余面收窄为
非查询消费面（Range/mutation 族的 wrapper identity——master.md 未解决问题 16 的
主文档 proxy 域 remove 同步语义缺口等，L2 主线域）。

## 五、教训

1. **假象优先查 binary 新鲜度**：shim 经 include! 编进 runner——改 shim 必须重建
   release binary 才在 WPT 生效（本轮 +4F「回归」一度误判为 void 修复的破坏）。
2. **「既存回归」可能是单一根因的浅层缺口**：Element-matches 139F 在 master.md 记为
   「待独立归因」，实为 fragment 根序列化源空串一行级缺口（R308 层 7）。
3. **bisect 中性化优于 stash 往返**：`if (false && ...)` 单点禁用比整树 stash 更快
   定位多层修复中的致变层。
