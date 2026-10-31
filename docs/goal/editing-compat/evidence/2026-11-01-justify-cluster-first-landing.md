# justify 簇首簇落地——justifycenter 全簇 + 查询面三笔（2026-11-01）

## 0. 结论速览

| 门禁 | 本轮 | 上一账 |
|---|---|---|
| justifycenter 语料（run 面 8 variant 全分母） | **4,728P/1,359F/0T** | 簇序挂账 justify 族 12,108F（全四命令族合计旧账） |
| bold run 面 | **1,602P/1,446F**（+464P） | 1,138P/1,910F |
| wss / ilb / chd | 1104P/0F、552P/0F、420P/0F | 逐位恒等 |
| join 三案 | 3456+2607+1455 = **7,518P/0F** | 逐位恒等 |
| inline-host（宽 filter） | 888 案 880P/8F（既有 8F：shadow-root delete 4F + placeholder 4F） | 逐位恒等 |
| engine lib | 3,635P/1F（s24 已知红唯一） | 逐位恒等 |
| webview lib | 832P/0F | 逐位恒等 |
| zz-jcprobe 探针 | 8/8 Pass | （新建探针） |

bold +464P 对账：P 1138→1602 与 F 1910→1446 恰对称（Y−X=464 恒等式两侧一致），无新增失败位；增量全部来自本轮查询面三笔（styleWithCSS 滞后态、dps 值面、justify 查询锚）命中 bold 语料的同名查询断言（录制方 Chrome 与本实现现同模型）。

## 1. 交付内容（part01c.js 新文件 1,286 行 + part06.js +62/−3 + js_dom_bridge.rs +8）

### 1.1 blockExtend 两档（Chrome delta，非参考实现直译）

参考实现（vendored implementation.js 4193-4284）的尾爬升无条件爬到最近块边界祖先；在语料页形状（宿主为 body 第 2 子）对 `foo[]bar` 产出 `1<BODY>@2..1<DIV>@1` 空含域（参考实现同页直调实证），而录制期望（真实 Chrome）把当前行收入含域。故分两档：

- **默认档**（apply ① 清理 pass）：尾爬升仅当对侧端界已逃出当前容器子树时继续（`startNode !== endNode && !_zwJusIsAncestorOf(startNode, endNode)`），再宿主夹取（严格祖先 → `(host,0)/(host,len)`）——祖先容器 center/带 align div 须入含域以供清理，caret 行不得误收祖先。
- **chromeTight 档**（apply ② wrap/query pass）：不做尾爬升；端界落在闭合容器（td/th/li，`_zwJusClosestClosure`）或宿主的祖先上时下移到该容器内容边界（`_zwJusClampToClosure`）——wrap 面只含当前行/当前 cell 内容。

### 1.2 `_zwJusBpCompare` branch 2 equality 修复（contained=0 根因）

边界 offset == 路径子 index 时，该子内部的节点严格在边界之后；旧码 `oB < i2` 丢等号导致 IsContained 恒 false → node list 空 → wrap 全 no-op。修为 `oB <= i2`。此修复先于 wrap 路径独立生效（p caret/td/table 形状随之全通）。

### 1.3 flat 模型边界 wrap（`_zwJusWrapViaHtml` splice 路径）

宿主直子文本 model-backed，innerHTML getter 走 pending 模型——裸 DOM 搬运（insertBefore/removeChild/moveChild）改了 live 树但读数不变（探针 w3/w5 实证：np-parented 后 `tc=` 串仍旧）。sanctioned 变更路径 = `host.innerHTML =` setter（`_zwExecCmdApplyFormat` part06 先例：setter 即建 pending 节点）。splice 路径：收集 live kids → 成员区间 prefix+openTag+memberHtml+closeTag+suffix 整段重写；复用兄弟包臂（prev/next sibling criteria 命中 → 并入其内容 + `<br>` 分隔，录制形 `foo<br>[bar]`）；选区端点映射（成员文本内累积偏移 → 重建后按 open tag 串匹配定位新包 → `_zwJusLocateTextOffset` DFS 文本副本定位；复用 prev 包时偏移加包内前置文本长）。

### 1.4 查询面三笔（part06）

- **execCommand("styleWithCSS")** 状态存储 + `queryCommandState` 滞后快照：Chrome 的 `queryCommandState("styleWithCSS")` 滞后一拍——快照在每个 execCommand 入口从当前模式同步（justifycenter 语料 250/250 行实测该模型：state_before 显 stale 值、state_after 显 justifycenter 执行时的模式）。`queryCommandValue("styleWithCSS")` 恒 `''`（录制 250/250），不暴露模式。
- **queryCommandValue("defaultparagraphseparator")** → 当前分隔符（缺省 `'div'`；语料 before/after = 上一行/本行 applied 值，持久 global 与录制一致）。
- **justify 查询面 caret 锚 fallback**：contained 叶为空（端点在容器边界、wrap 后 DOM 全在非叶块内）→ 锚到选区起点所在块（元素容器位取前一可见子，无则后一子）。

### 1.5 对齐值读数两笔（`_zwJusGetAlignmentValue`）

- start/end resolved 先询祖先显式提示链（style text-align / center 标签 / align 属性）——wrap 后 td 内文本继承外层包 div 的 center（`{<table>}` 选区行录制口径）。
- align 属性只认 HTML § rendering 对齐映射元素集——quasit 等未知元素的 align 不产生 CSS text-align（`<quasit align=right><p>` 录制 left）。

### 1.6 样式序列化

wrap 新建包 `text-align:center` 无空格形（录制口径；runner compare 不归一化该空格）。

## 2. 验证明细

- 探针 `zz-jcprobe`（临时页，commit 前删）8 形状全绿：basic foo[]bar、p caret、center no-op、div align=left、td cell、whole table、p caret query face、query face after wrap（value/state/indeterm 三面）。
- bisect driver `zz-jcslice`（临时页，commit 前删）：P→F **0**（对 /tmp/jc-baseline.log 逐行对账）、F→P **398**。
- 真页 `justifycenter.html` 8 variant 全分母：4,728P/1,359F/0T（`/tmp/jc-real.log`）。
- 方法论注记：语料 case 级 timeout（`ZW_CORPUS_CASE_TIMEOUT_SECS`，缺省 30s）小于单 variant 页墙钟时 runner 截断（`Timeout ... unreported`）→ 全页对账必须 case timeout ≥ 页墙钟（本轮 30s 截断致两次全页 log 行数漂移，逐行对账只取双方已报告行的交集，P→F 结论不受影响）。
- 参考实现 ≠ 录制期望闭环：vendored implementation.js 直调（探针注入）对 corpus 页形状同样不 wrap——录制期望来自真实 Chrome，本流按 Chrome delta 移植（§1.1 两档即此决策的落地形）。

## 3. 残余与挂账

- **merge FIXME 族（dom-wrong ~57 行 ×4 variant）**：跨 cell 选区行（data-start=1 data-end=2 只 wrap 首 cell 的不对称）、`<div style=text-align:center>foo</div>[bar]<div style=text-align:center>baz</div>` 三节点合并行、tbody/tr-attr selection 行（setup IndexSizeError 既有族 193F×4 variant 不变）、`{<table>…}` 整表选择行的部分变体。记 FIXME 于 part01c wrap 节注释。
- **quasit align（4F×2 命令）**：reset.css `quasit { text-align: inherit }` 后真实 computed style 链属 style-system 面（渲染流），本流读数已按映射集规避 align 属性误读，残余为引擎 computed style 对未知元素 text-align:inherit 的求值债，挂账不跨界。
- **justifyright/full/left**：与 justifycenter 共用 `_zwJus*` 实现（分派臂 part06 4391-4395），预期同族收益，本轮未单独全页对账（簇序次轮补账）。
- 临时探针页两件删除；重建配方：probe = 显式建 host（`while (removeChild)` + createElement/appendChild，setter 延迟建节点会拿旧节点）+ 7 形状 + query 三面；slice = justifycenter 拷贝 + `?N-M` slice + R-done 标记。

## 4. 门禁命令口径

- runner 电池：`ZW_CORPUS_CASE_TIMEOUT_SECS=60 ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit <按页> -- ./target/release/zero-wpt-runner testharness-editing-{run,other} <filter>`；justifycenter 全页 `ZW_CORPUS_CASE_TIMEOUT_SECS=150`、guard 1500s。
- engine lib：`cargo test -p zero-engine --lib`；webview lib：`cargo test -p zero-webview --lib`（均 test-guard 包裹）。
- bold 失败集与 10-30 基线不再逐位 identical（本轮查询面三笔有意改变该面，+464P 零新增失败，见 §0 对账）。
