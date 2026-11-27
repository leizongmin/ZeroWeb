# s10k68 刀56：dom/traversal 语料 support 三件补齐（assert_node 整簇假败）＋iframe 移除后 browsing context 丢弃门——TreeWalker 钉面 28F 清零＋contentWindow null spec 面

日期：2026-11-28（chain68 链窗 21:22:51Z→21:41:04Z，pid 2936414）。刀体 commit `35ac9d5f8`（发射判读号 `0380a258a`，push 前 rebase 重放号变〔html 流 docs `1ec4a449f` 并入〕，part04/fetch 脚本双 blob 恒等复核＝判读域不变；两文件：`tests/wpt-runner/scripts/fetch-dom-subset.sh` +7 常驻化＋`crates/engine/src/js_dom_shim/part04.js` +32 丢弃门；发射 bin `8c5a3388962f`＝提交域字节，链后 md5 恒等复核）。刀号：**刀56**（十进制续编，承刀55）。

## 刀位与根因（控制面挂账：traversal 29 下刀候选）

chain67 域 traversal 29F 分布：acceptNode-filter 10＋skip 6＋reject 6＋skip-most 2＋previousSibling/previousNode 钉面 2＋basic/currentNode/cross-realm-null-browsing-context 各 1（9 页）。

**形态解壳＝语料缺口非引擎语义**：8 页 28F solo 全 `assert_node is not defined`——fixture 引 `support/assert-node.js`，而 `dom/traversal/support/` 目录不在库。`fetch_dir_html` 只列子目录顶层 `.html/.js` 不递归 `support/`（R198 NodeList 同族先例）；引用为 HTML attr 与 JS 赋值双形态（`iframe.src = "support/…"` 形前次 attr grep 漏捕，本轮 JS 侧引用普查补齐）。引擎侧 TreeWalker/NodeIterator 实现（part06 R2803 eager walker＋R51/R84/R85/R88/R3257 filter 语义）**本就正确**——TreeWalker.html 大 oracle 页 761 子测全 P 为证；28F 全是缺 helper 的假败。

**support 三件**（pin `31597693` 实取）：`assert-node.js`（9 用例共享断言 helper）、`empty-document.html`（cross-realm iframe fixture）、`TreeWalker-acceptNode-filter-cross-realm-null-browsing-context-subframe.html`（subframe 全局定义包 `parent.document.createNodeIterator` 的 dummy-filter 代理函数＝`contentWindow.createNodeIterator` 解析面）。fetch 脚本常驻化同提交。

## contentWindow 丢弃门（part04，真 spec 面）

补齐语料后 null-browsing-context 页失败形态前移：`assert_equals(iframe.contentWindow, null)` expected null but got object——iframe.remove() 后（browsing context discard）主文档 identity 链 contentWindow getter 仍返 entry.win。修复＝get trap 加 connectedness 门（`state==='done'` 且断链返 null；**no-src fallback 臂〔刀③ k3 registries detached 文档 about:blank 兜底〕与 loading/error 态零触碰**；sel 形读元件 proxy `isConnected` 全语义面——R4200 同步移除标记＋R90 反链 append 窗真值，首版门手卷 contains 误杀同步窗首读， solo 一次即回退改读全语义面；handle 形仅显式移除记账＋宿主反查双确认）。spec https://html.spec.whatwg.org/multipage/iframe-embed-object.html#dom-iframe-contentwindow

**A/B 定谳**（wc 页不在四腿链须自守）：pre `74db1f231b60`（＝链67 同字节，stash 门后重建实证）vs post `8c5a3388962f` 三 registries 红页（define-upgrade-order 7F／append 10F／initialize 1F）F 集多重集零差＝**红面既有非 gate 所致**（独立挂账：setHTMLUnsafe docElement 面／registry undefined，非本刀域）。绿页守护：effective-global 66P/0F、registry-define-get-etc 7P/0F、cross-realm 5P/0F、traversal-skip 6P/0F 恒。

## 判读（chain68，21:22:51Z→21:41:04Z，四腿）

单页先行：9 钉面页 post-fetch 0F（traversal-skip 6P／reject 6P／skip-most 2P／acceptNode-filter 12P／basic 6P／currentNode 4P／previousSibling/previousNode 1P＋cross-realm 5P 恒）。

| 臂 | 锚（chain67） | 实跑 | 归因 |
|---|---|---|---|
| san 全臂 | 773P/5F/0T | **773P/5F/0T** | F/T 集逐字节恒等＝刀零 san 影响 |
| dom 臂 | 57542P/261F/12T | **57642P/233F/12T** | **F 页集净减 8 页 −28F（全 traversal assert_node 族）零新增 F**；T 集恒等；ΔP+100＝28 翻面（含单侧新键 16）＋72 Range-mutations 截断噪声带（replaceData +48＋dataChange +24，已知族） |
| tt 臂 | 2290P/14F/18T | **2290P/14F/18T** | F/T 集逐字节恒等（真实池 0F 维持） |
| ref 臂 | 725/725 | **725/725**（Layout 521/521＋Text 204/204） | 恒等 |

腿 rc 留痕：san 2／dom 2／tt 2／ref 0（rc 非门信号）；发射断言 `8c5a3388962f`+head `0380a258a` 过＋链后 md5 恒等。语料页即常驻回归。

## 残相挂账

- **null-browsing-context 页 1F 恒留**：`assert_true(errorWasThrown)`——测试期望 subframe 代理创建的 NodeIterator 在 iframe.remove() 后 `nextNode()` 抛 InvalidStateError＝**chromium 专有 incumbent-realm 绑定**（Blink 把 iterator 绑到创建调用者 realm 的文档并随其 shutdown detach；现代 whatwg DOM spec detach 已移除＝iterator 绑 root 的 node document〔parent，恒 active〕无 throw）。复现需调用者 realm 追踪机制，1F 不值；`contentWindow null` 断言已随门翻真。wpt.fyi 上游判定查询未成（search API 307→HTML 面板），Firefox 侧未能核。
- registries 三红页（define-upgrade-order 0P/7F、append 6P/10F、initialize 26P/1F）＝既有红面（F 集 A/B 绑定在档），候选独立刀位：`doc.documentElement.setHTMLUnsafe is not a function`（setHTMLUnsafe docElement 面 registry 文档域缺口）等。
- dom 残池刀后构成（233F）：ranges/tentative 189＋nodes 24＋events 11＋historical 3＋window-extends-event-target 2＋html 2＋lists 1＋traversal 1（chromium incumbent-realm 挂账）。

census：+28 → **84,520，距 DC-3 校准线 ≥94,093＝9,573**（原线 ≥99,947＝15,427）。
