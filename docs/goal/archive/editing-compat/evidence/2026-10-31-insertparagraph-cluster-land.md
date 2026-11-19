# insertParagraph 簇落地：live-DOM 降解模型（2026-10-31）

前置：v1.13 状态块（D1/D2/D3 + 第 4 应用路径换代通知）；本簇原计划见 v1.12/v1.13 簇序（「insertparagraph 簇」）。

## 0. 一句话

insertParagraph 从 flat outerHTML-rewrite 模型（host 分离、turn 内不可见、无降解规则）换代为 **live-DOM in-place 模型**：局部 shim 树写（`createTextNode`/`insertBefore`/`nodeValue=`）产生 DomMutation 且 turn 内同步可见（尾簇 34 已证明的管线），降解规则按语料矩阵三分支；cannot-have-div **0P/420F → 420P/0F 全扫**，white-space-style **36P/1068F → 616P/488F**，inline-editing-host **0P/880F（被既有融合序列化 attr 层 gap 封顶，HEAD 同证，语义层探针证明零可见语义失败）**。

## 1. 落地清单

| 项 | 位置 | 内容 |
| --- | --- | --- |
| 三分支降解核心 | part01.js `_zwCeIsBlockTagName`/`_zwCeDisplayOf`/`_zwCeWsDeclared`/`_zwCeWsSignificant`/`_zwCeLineBreakAt`/`_zwCeParagraphMutate` | ① host div + display block-ish + host 级 caret → separator 元素分家（defaultParagraphSeparator，默认 div）：尾部文本 + 后续兄弟迁入，空侧 `<br>` 填充；caret 在内容头 → 空分隔占位插前部，尾部保持裸文本（`[]abc` → `<sep><br></sep>abc`）。② blockish 后代（显式 display:block，或无 display 声明 + div/p/h1-h6/li 标签）边界直在块内 → 同标签分家并复制 `style` 属性（`sep2.setAttribute('style', stB)`），双侧掏空补 `<br>`。③ 其余 → 行断：white-space significant（声明链上溯，**声明 normal 即停**）→ `'\n'` 文本，否则 `<br>`；断点后无兄弟才补第二个 break；display:inline 子代无自有 ws 声明 → forceLf（Blink quirk，恒 `\n`） |
| 事件契约 wrapper | part01.js `__zw_ce_paragraph_split_at(host, range, fireEvents)` | execCommand 臂自行派发 beforeinput(insertParagraph, cancelable) → 以 `fireEvents=false` 调核心；键盘 wrapper `fireEvents=true`。事件链：beforeinput → textInput(`'\n'`) → mutate → input |
| 入口 wrapper | part01.js `__zw_ce_insert_paragraph(sel)` | 元素或 selector 双形态；webview Submit 臂经 `script_contenteditable_insert_paragraph` 调用 |
| insertText `\n` 等价路由 | part01.js `__zw_ce_insert` | textInput 门后 `ins === '\n'` → `paragraph_split_at(el, range, false)`；核心返 false 回落字面拼接。语料等价性：insertText("\n") ≡ insertParagraph。`__zw_ce_insert` 同时支持元素实参（动态 host 无 `__zwSelector`，querySelector 回环空转的修复） |
| execCommand 接线 | part06.js insertparagraph 臂 | 换调 `paragraph_split_at(host2, rng2, false)`；删除旧 flat 函数 `_zwExecCmdApplyParagraphSplit`。新增 `defaultparagraphseparator` 案（仅 div/p 小写，否则 false） |
| bridge 生成器 | script_gen.rs `script_contenteditable_insert_paragraph(selector)` | 新增 pub fn |
| webview Submit 臂 | webview/user_actions.rs | CE 分支改调新脚本；FIXME：无 modifier payload（Shift+Enter 未区分，insertLineBreak 语义待接线） |
| runner Actions 键盘 | wpt-runner/testharness.rs keydown 臂 | `key == "\u{E007}"` 且 CE probe 命中 → `HtmlUserAction::Submit` + keypress 后续；同 FIXME |
| 既有 bug 顺手修 | part01.js `__zw_ce_enter` | 折叠 caret 时 `v.slice(0, so) + v.slice(eo)` 双写尾部（`ab<br>b` 类）→ `v.slice(0, so)`；与分支③同根因，共享路径修（准则 6） |

## 2. 语料账（同驱动同参数 apples-to-apples）

驱动：`testharness-editing-other <file>`，`ZW_SCRIPT_TIMEOUT_MS=300000 ZW_CORPUS_CASE_TIMEOUT_SECS=60`，test-guard 4G/8G/3600s。基线 log `/tmp/zw-ip-corpus.log`（23:19，实现前二进制）；终账 `/tmp/zw-ip-final.log`（00:27 驱动，终版二进制）；cannot-have-div 终账 `/tmp/zw-ip-cnd2.log`。分母逐位一致。

| 语料 | 基线 | 终账 | 备注 |
| --- | --- | --- | --- |
| insertparagraph-in-editing-host-cannot-have-div | 0P/420F | **420P/0F** | 全扫（span/p host 嵌套子代规则矩阵全命中） |
| insertparagraph-with-white-space-style | 36P/1068F | **616P/488F** | +580P；488F 残余主属 italic/bold wrapper 组（temporary-inline 保留，见 §5） |
| insertparagraph-in-inline-editing-host | 0P/880F | 0P/880F | **被既有 infra gap 封顶**（§4）；失败行唯一 delta = stale style 属性，语义结构全对 |
| insert-paragraph-in-void-element | 2P/70F | 2P/70F | DEFERRED（Blink 特有 void 处理，§5） |
| insertparagraph-in-child-of-head / -in-child-of-html / -in-non-splittable-element | 0 结果行 | 0 结果行 | runner 静默跳过（r5342 全量基线同为 0 行，非本轮回归） |

run 面 insertparagraph.html 回归核对（基线 `/tmp/zw-4case-retest-insertparagraph.out` 7,195r 3,774P/3,421F vs 终版 `/tmp/zw-ip-final.log` RUNFACE 段 6,557r 3,492P/3,065F）：**逐测试 ID 对照，交集 6,557 = 终版全集（前缀截断，零新增），F→P 翻转 61、P→F 翻转 2**，净 +59。-638 行为重型变体在 300s 预算窗内完成数减少的纯截断（与同日五轮 -174/-266 同型）。2 处 P→F 同一变体形状：`<ul contenteditable><li>{}<br></ul>` 空 li Enter——Blink 预期 no-op（保持 `<ul><li><br></li></ul>`），现实现 host 分家在 UL 直下插入杂散 `<br>`；同变体相邻 `queryCommandIndeterm` 子测试异常判同族级联（状态污染），归入空 li no-op 残余一条。61 处 F→P 抽样：defaultparagraphseparator div/p 组、table 单元格分家组——新降解模型正向翻转。

## 3. 探针验收（真实管线）

单变体临时页（跑完即删）直调 `__zw_ce_paragraph_split_at` / `__zw_ce_insert_paragraph` / `__zw_ce_insert`，期望矩阵逐项命中：

- `a[]b`（host div）→ `a<div>b</div>`；`<p>a[]b</p>` → `<p>a</p><p>b</p>`；`a<br>b` 型行断；空 host → `<br><br>`；`abc[]` → `abc<div><br></div>`；`[]abc` → `<div><br></div>abc`
- pre host + display:block 子代 → 同标签分家带 style 复制
- insertText `'\n'` ≡ insertParagraph（`a<br>b`）；defaultParagraphSeparator=p → `a<p>b</p>`
- 键盘双路实证（send_keys 与 Actions keydown `\u{E007}`）：空 span → `<br><br>`；`ab[]cd` → `ab<br>cd`
- 白盒：`cargo test -p zero-engine --lib insert_paragraph` 1 passed（断言从 SetOuterHtml 换代为新局部树形态：CreateElement{div} + AppendChild + SetChildText）

## 4. inline-host 0P/880F 归因：融合序列化 attr 层 gap（既有，非本轮引入）

**现象**：runner 动作后 `editingHost.removeAttribute("style")` 生效于属性读路径（`hasAttribute` → false），但 `container.innerHTML` 序列化仍带旧属性。

**A/B 钉死**：git stash 双跑同探针，HEAD 二进制同症（`styleGone:true` + `container` 序列化含 `style="display:inline;white-space:normal"`）；子列表变更（插入的 `<br>`）正常可见。

**机制定位**（time-box 源读，挂账修复面）：

- outerHTML sel 分支融合序列化的**门**（part04.js ~1823）：`_zwPendingByParent` 桶非空（stamp 匹配当前代际）**或** `_zwPendingSubtreeDirty(sel)`——纯 attr 变更不开门；
- 门关 → 回落 `__zw_get_outer_html`（Rust 基树快照序列化）——**pending attr 变更 apply 前不进基树** → stale；
- 属性**读**路径 `__zw_has_attr_lw`/`__zw_get_attr_lw` 是 latest-wins（pending 感知）→ 正确返 false。读写两层对 pending attr 的可见性不同步，即 D1 族（快照/融合视图层）在 attr 维度的残余；
- handle 分支序列化（part04.js ~1860）走 `__zw_attr_names_handle`（仅 mutations，无快照基底），与 sel 侧 pending 感知层亦不同源。

**语义零失败的证据**：880 条 Fail 行逐条 delta 仅 stale `style` 属性——内部结构（`<br><br>`、`ab<br>cd`、`abcd<br><br>`、尾缀 `abc` 文本，corpus 自建 `createTextNode("abc")`）与 expected 全部一致（/tmp/zw-ip-final.log 1-1762 行抽验）。attr 层修复后该面预期大幅翻转，列**下轮首位候选**。

## 5. 残余与挂账

1. **融合序列化 attr 层 gap**（§4）——inline-host 880F 封顶项，D1 族 attr 维度；修面：outerHTML 门纳入 attr 变更（`_zwPendingSubtreeDirty` 记 attr 脏）或序列化前同步 pending attr。
2. **white-space-style 488F 残余**：italic/bold wrapper 组（temporary-inline 保留语义——分家时 `<i>`/`<b>` 壳随迁）， documented FIXME。
3. **void-element 70F**：Blink 特有 void caret 处理（void 前插 br + 容器分家、br void 吞噬、img 迁移），DEFERRED。
4. **空 li no-op**：`<ul contenteditable><li>{}<br></ul>` 空 li Enter 应 no-op，现 host 分家插杂散 `<br>`（run 面 2 处 P→F 唯一根因）。
5. **Shift+Enter 未区分**：键盘两路均无 modifier payload，FIXME 两处（user_actions.rs / testharness.rs）。
6. uievents textInput 语料（basic.html 等）页面脚本 SyntaxError（3:38）：HEAD 同症，非本轮引入，归 uievents 面旧账。

## 6. 质量门

| 门 | 结果 |
| --- | --- |
| engine lib | 3619P/9F——9F 为已知红集（零新增，v1.13 同账） |
| webview lib | 822P/1F——1F 为 pre-existing `test_document_scrolling_element_r3259` |
| fmt/clippy | fmt clean；clippy 复核在提交前完成（结果见提交信息） |
| bench-gate | 首跑（与 RUNFACE 并发，00:48）5 项 wall-clock FAIL（medium/morning first_paint 1.7-1.8×、medium layout 2×）——归因并发 CPU 争用（本方法失误，已记录）；改动零热路径触碰（新增 JS 函数声明为冷路径，无 style/layout/paint 代码变更，+377 行声明不足以解释 2× layout），且规则 scope 为热路径变更门。干净复跑挂账：机器上另有 v1.13 跨域电池 systemd 滚动任务（load 7-10），wall-clock 测量持续噪声；与 attr 层修复轮同窗补跑 |

## 7. 变更文件

`crates/engine/src/js_dom_shim/part01.js`（+355 行主体）、`crates/engine/src/js_dom_shim/part06.js`（execCommand 接线 + defaultparagraphseparator）、`crates/engine/src/js_dom_bridge/script_gen.rs`、`crates/webview/src/webview/user_actions.rs`、`tests/wpt-runner/src/testharness.rs`、`crates/engine/src/js_dom_bridge_tests/part06.rs`（断言换代）。合计 +377/−153。
