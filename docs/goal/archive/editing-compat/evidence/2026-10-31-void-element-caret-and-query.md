# void 簇收口：querySelector 解析子树下钻 + void caret 两臂

日期：2026-10-31（R5398 轮）
范围：`crates/engine/src/js_dom_shim/part04.js`（R322 解析分支子树下钻）、`crates/engine/src/js_dom_shim/part01.js`（insertParagraph void 拆分臂 + insertText void 插入臂）
语料：`tests/wpt-runner/wpt-data/editing/other/insert-paragraph-in-void-element.tentative.html`（2P/70F→**72P/0F**）、`insert-text-in-void-element.tentative.html`（0P/48F→**48P/0F**）

## 前置归因（接 58142f42f 探查）

上轮探针（zz-probe-void.html）三事实：`editor.querySelector('br'/'img')` 恒 null、`querySelector('div')` 命中且 `firstChild.nodeName === 'BR'`、`__zw_query_all('div')` 命中。本轮扩展探针（分层判据 L1-L10）定谳：

- **L3 `__zw_query_all('br')` = 空、L4 `__zw_query_all('div')` = 单条 "div"**——host 快照（dom_html 解析 + pending 结构 mutation）此刻只有空 editor div：innerHTML 替换经 host **异步 apply**（R309 既有语义），同 turn 查询面 = pending 视图，**host 快照层无 void 读盲**。
- **L5 `editor.querySelectorAll('br')` 命中**——querySelectorAll 的 R309 pending-fused 臂从 `_childNodeList` 全量重建并逐节点 compound 匹配。
- **L1 `editor.querySelector('br')` = null**——querySelector 的 R322 回落（host miss → pending 桶）**只 compound 匹配桶 added 顶层代理**：`<div><br></div>` 的顶层是 div（命中），其子代 br/img 永不达。handle 分支（createElement 链，R322 后半）既有 `_zwHCCollectSubtree` 下钻，解析分支（非 handle）缺失——**70F 的 caret 不落位假失败根因**（18 rect 炸同源）。

## 实现

### 1. R54xx：R322 解析分支子树下钻（part04.js）

非 handle（innerHTML 解析）顶层代理不命中时 `_zwHCCollectSubtree` 下钻、首个命中后代原样返回 plain 代理（与 handle 分支对齐；不物化——`_wcRebuildAsHandle` 产生 detached 副本脱离 pending 树，消费面 = caret + 编辑命令的 plain 树操作，R309 重建产物同款语义）。顶层命中仍优先（树序）。探针 L1/L5/L6/L10 全绿（L2 document 级、L3 host 快照按设计不接，见「范围外」）。

### 2. ⓪ void 拆分臂：insertParagraph（part01 `_zwCeParagraphMutate`）

触发：collapsed、`!lineBreakOnly`、sc 为元素且 `_zwCeIsVoidElementTag(sc)`（br/img/hr/input/embed/wbr）。位于 ① 宿主级拆分之前。

- 容器判定：自 void 上溯至首个 `_zwCeIsBlockTagName` 祖先（div/h1/li；inline 壳 span 穿透）；容器 = host 不接（语料面皆包块容器，落 ③ 既有行断）。
- 拆分：void 及后兄弟移入新块；新块标签 li→li、标题容器按语料分支——可见 void（hr/img/input）或 void 后有内容 → 同标签（`</h1><h1>`），否则 div（语料注释自认 Blink hr/embed/wbr 不一致，分支判定以 visibleTag + 后继内容复刻）；div 容器恒 div。
- 容器空 → 占位 `<br>`（`__zwPlaceholderBr` 打标，与 ① 同款）。
- **跨域摘挂**：void 及后兄弟是解析 plain 节点，宿主桥 appendChild 对 plain 子做序列化拷贝不摘原父（探针 zz-probe-void2 M 臂实证：sepKids=1 且 contKids=1 双现 → `<div><embed></div><div><embed></div>`）——先 `cont0.removeChild`（plain 工厂方法）再挂 sep0。

### 3. ⓪ void 插入臂：insertText（part01 `__zw_ce_insert` 元素分支头）

触发：sc 为 void 元素且 so===0。文本不进 void（void 序列化丢弃子内容——旧径 appendChild 进 br 形同未插，48F 全为 no-op）。按 Blink 形态可见性回溯（语料期望即 spec）：

| 回溯命中 | br/embed/wbr（不可见） | img/input（可见 inline） | hr（块级） |
|---|---|---|---|
| 非空文本 | 接尾追加 | 接尾追加 | 接尾追加 |
| 带文本元素 | 其末文本节点追加 | 同左 | void 前插入 |
| ws-only 文本 | 跳过 | void 前插入 | 消费 ws + void 前插入 |
| 空 inline 元素 | 跳过（记 skipV） | 跳过（记 skipV） | void 前插入 |
| 耗尽 | void 前 | skipV 存在→skipV 前（embed/wbr/img；**input 例外→void 前**），否则 void 前 | void 前 |

skipV 例外（T4 语料）：embed/wbr/img 耗尽回落**首个跳过元素前**（`abc<span></span><tag>`），br/input 回落 void 前（`<span></span>abc<br>`）；hr 空元素/ws 提前 break 不经耗尽路径。

## 验证

### 语料

- insert-paragraph-in-void：2P/70F → **72P/0F**
- insert-text-in-void：0P/48F → **48P/0F**

### 门禁矩阵（全部零位移）

| 面 | 结果 | 基线 |
|---|---|---|
| ilb（insertlinebreak-with-white-space-style） | 552P/0F | 同 |
| wss（insertparagraph-with-white-space-style） | 1104P/0F | 同 |
| chd（insertparagraph-in-editing-host-cannot-have-div） | 420P/0F | 同 |
| inline-host（insertparagraph-in-inline-editing-host） | 880P/0F | 同 |
| join 三案 | 2607+1455+3456 = 7,518P/0F | 7,518P/0F |
| bold run（editing/run/bold.html） | 1,138P/1,910F | 同 |
| engine lib（DISPLAY=:20） | 3,617P/10F | 同 |
| webview lib（DISPLAY=:20） | 830P/0F | 同 |

engine lib 红集 10 名 = fetch/Request 族 8 + iframe SW + s24 flip，与现行账红集族完全一致（本轮 `grep "^test .* FAILED" | sort | md5sum` = `316301039d620912f5c6dd10a8d062bf`，与历史 md5 `ba096efcd2f859d1ac81edc8cbd99480` 串法不同不作直比，按名单逐名核对）。

### stash 对照归因（非基线门禁面的 4 处）

工作树 stash 后基线二进制复跑，与改动后逐位同态，均为既有非绿面、非本轮回归：

- undo-insertparagraph-after-moving：0P/4F（两态同）
- cloning-attributes-at-splitting：32P/20F（两态同）
- insertparagraph-in-non-splittable / in-child-of-head / in-child-of-html：0P/0F（两态同；非 testharness 断言面）

## 范围外（已知不接）

- `document.querySelector` 纯 tag/compound pending 回落（R145/R3902b）不下钻解析子树——void 语料只用 `editor.querySelector`，document 级加宽属全局行为变更，未立项不动。
- host 快照查询（`__zw_query_all` 族）对同 turn pending 内容不可见——R309 既有设计（apply 异步），非缺陷。
- 探针 `zz-probe-void.html`（分层判据 L1-L10）/ `zz-probe-void2.html`（跨域摘挂 M/R 臂）未跟踪保留。
