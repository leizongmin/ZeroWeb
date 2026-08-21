# R145 — pointer-event-document-move 1F→0F（template contents 查询不可见 + handle identity 派发桥）

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**驱动用例**: `dom/events/pointer-event-document-move.html`（1 subtest）

## 用例链路与根因（四层）

用例：`document.querySelector('template').content.cloneNode(true)` → clone 内 `p`
注册 pointerup listener → `document.body.append(clone)` → `test_driver.click
(document.querySelector('p'))` → 断言 listener 触发（「Moving a node to new document
should move the registered event listeners together」）。

| # | 断点 | 根因 | 修复 |
|---|------|------|------|
| ① | `querySelector('p')` 命中 **template 内解析产物 p**（sel-proxy）而非 append 的真 p | ZW 解析器把 template contents 内联为 template 子（`get_template_contents` 返目标自身）；真实浏览器 contents 在 inert DocumentFragment **非文档树后代** | dom crate 查询遍历 R145 template opacity：5 个遍历函数（find_first_matching/_chain/_chains + collect_matching/_chains）加 `skip_templates` 开关——template 子树不进入，template 自身可命中；**显式含 template 段**的组合链走 direct-address 例外（不跳过——shim `template.content` 子代理的结构路径 `body > template > p` 依赖可解析） |
| ② | `test_driver.click` 报 "no stable selector" | stub `selectorFor` 在 **enqueue 时**解析（promise body 同 turn 求值，mutation 未 apply，handle→selector 正置表空） | **延迟解析**：enqueue 存元素引用（`queuedElements`），宿主出队时经 `__zw_td_selector(id)` 现场解析（跨 turn，apply 已 merge）+ 新 `__zw_selector_for_handle` 正置反查回调（webview `handle_selector_forward` 镜像表，merge/清空与倒置表同步维护） |
| ③ | click 的 pre_events（pointerdown/pointerup）派发到 sel-key，listener 注册在 handle proxy 的 `_listenerStore['@__n1']` → miss | sel-key 派发与 handle-key listener store 的 identity 断链（R100 桥的 dispatch 侧缺口） | part06 `__zw_dispatch_event` sel→handle identity 桥：`__zw_handle_for_selector(sel)` 命中 → 以 handle 形态派发（`_elKey(handle)` 锚定 listener store）；未命中 → 原 sel 路径零回归 |
| ④ | 同 turn（await 前）`querySelector('p')` 返 null（host 快照未含 append 的 p） | 查询不反映同 turn pending mutation（R51c 仅 `#id` 形式回落） | part06 `querySelector` 补**纯 tag 形式** pending 回落（扫 `_zwPendingAdded` 按 tagName 匹配）；part04 sel-based template `content` 视图（WPT 用例的 `template.content.cloneNode(true)`，旧返 undefined） |

## lit 回归与修复（关键教训）

part04 初版对 `_tplContent` **无条件**加 own `cloneNode`——handle 形态
（lit-html：`createElement('template') + innerHTML=`）原依赖 Node.prototype
**泛型** cloneNode（registry 子全 mutation 语义，R128 fragment 分支），own 版本
遮蔽泛型 → `make test` 抓到 `e2e_lit_library` 4 件失败（rr-kids:1，首渲染不落地）。
修：`cloneNode` **仅 sel 形态**（handle 形态保持 R95 原样）。教训：**shim 的
own-property 修复必须区分节点形态**（sel/handle 两形态的委托链不同，own 遮蔽
对一形态是修复对另一形态是破坏）。

## A/B 结果（polyfill / native 双路径）

| 套件 | 结果 |
|---|---|
| pointer-event-document-move | **1P/0F 双路径 100%** |
| dom/events 全量 | polyfill 419P/24F / native 419P/24F（fail 集一致，仅 Event-dispatch-redispatch polyfill 多 1 subtest）；vs R144 416P/33F：fail 集仅 pointer-event-document-move 消失 + 净 3P（DocumentFragment-getElementById / svg-template-querySelector 为既存 Fail，git stash 隔离核实基线同 Fail 非回归） |
| dom/nodes 全量 | 5474P/230F（vs R141 记载 232F 零回归——template opacity 未引入 nodes 回归） |
| `make test` | 66 套件全绿（含 lit e2e 回归修复后） |
| fmt / clippy | 零 diff / 零警告（v8 + quickjs 双矩阵） |
| product-smoke | 23.61% 与基线逐字节一致（git stash 对照；既存 drift 非本轮引入，struct-check PASS） |

## 单元测试

- dom crate `tests_9_query_coverage.rs`：`test_query_skips_template_contents_r145`
  （纯 tag 查询命中 template 外 p + all 长度 + template 自身可命中）、
  `test_query_explicit_template_addressing_r145`（`body > template > p` 直达 +
  列表形态）、`test_query_skips_nested_template_contents_r145`（嵌套子树整体不可见）
- webview `integration.rs`：`test_selector_for_handle_callback_r145`（正置反查
  round-trip）、`test_template_clone_identity_chain_r145`（同 turn identity +
  sel-key 派发经 handle 桥触发 listener）
- page-runtime `html_actions.rs`：R145 扩展 pre_events 断言
  [pointerdown, mousedown, pointerup, mouseup]（Pointer Events 序）
