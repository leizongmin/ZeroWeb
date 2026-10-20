# P9 修复——ARIA 反射跨文档 scope + getElementById 同 turn shadow 移树（2026-10-20）

前序：[P8 xmldecl 收编](2026-10-20-p8-xmldecl-iframe-charset.md)

## 结果

`html/dom/aria-element-reflection.html` 残差 **2F → 0F**（M4 片 e 起挂账的
async-apply 架构缝两案全部翻绿）：

- **'Cross-document references and moves.'**——跨文档引用 getter 曾返引用元素，
  修复后 invalid scope 判 null；移入主文档后恢复可见（上游 implementation-defined，
  Chrome 取可见）。
- **'Reparenting referenced element cannot cause retargeting of reference.'**——
  同 turn 影树移树 + id 改名后 `document.getElementById('apple')` 曾返快照 stale
  的 apple，修复后命中改名 banana。

全通道回归见 master.md 当日账（预期 61345P/63F：+2P 零新增 Fail）。

## 归因（探针实测，非代码推测）

`_zwAriaTreeChain` 链行走探针（临时 probe 页，已删）：

1. **主/分离文档树身份同串**：detached（`implementation.createHTMLDocument`）树
   的 html 根 `parentNode` 指向 document 对象（非 null），主文档根同理——行走终点
   都是 document 对象，`String(document)='[object Object]'` 使两文档同落
   `'sel:[object Object]'` 桶 → 跨文档引用误判 valid。part01 既有 `'html'` 根分支
   是死代码（根元素 parentNode 非 null，分支不可达）。
2. **WeakMap.size 不存在**：`'doc' + (_zwAriaDocIds.size + 1)` 产出 `'docNaN'`
  （`size + 1 = NaN`）——死代码分支内的同一 idiom，激活后立现。
3. **plain 子 parentNode 不重指**：`createHTMLDocument` 元素 append 入 sel 父
   （handle=null）后，R368 盖章分配 host handle，但 R84 parentNode 重指分支 gated
   父 `handle` 非空整体跳过——工厂对象 parentNode 仍指 detached 源树，链行走沿
   stale detached 链上行判 invalid（'afterMove visible' 断言实证）。
4. **gEBI 快照命中短路**：apple 移入 shadowFridge 影树同 turn，host 快照仍列
   原位；`_r125Overridden`（R125 id 覆盖表）只认 id **写**变更，apple 未改名 →
   命中先验返回 stale apple，banana 的新 id 覆盖项（流程②）不可达。

## 修复（三处，R5206）

| 位置 | 内容 |
| --- | --- |
| part01 `_zwAriaTreeChain` | 行走终点 **document 对象（nodeType 9）按文档身份落桶**：主文档（`=== document` 或 `defaultView === globalThis`——handle 族终点的 document 包装对象与 globalThis.document 非同一 JS 对象，多族并存 R32；defaultView 是 browsing context 锚，分离文档 spec 无 → null）落 `'doc'`，分离文档 WeakMap 序号（`++_zwAriaDocIdSeq` 独立计数器替代 `size + 1`）。`'html'` 分支同 idiom 修正保留（非容器根元素 parentNode 为 null 的形态仍可达） |
| part04 `appendChild` | sel 父 + plain 子（R368 盖章后 `__zwHandle` 有、`__zwSelector` 无、父 handle=null）**parentNode/parentElement 重指**调用方父 proxy（getter defineProperty，形状对齐 R84 先例；identity 保持调用方路径原对象 R32）——spec `concept-node-pre-insert` 步骤 6「set parent」 |
| part06 `getElementById` `_r125AncestorRemoved` | 父链上行先消费 **`_zwAriaSyncParent` 同步父记录**（写点 part05 `_recordHandleChild`/part04 R5009，sel 父接手时清除）：记录命中 shadow 根或分离 fragment = 整棵子树不在文档树 → getElementById 不可见（spec 只搜文档树）；随后既有 `_zwNodeParent`/`__zw_parent` 快照上行保持。gate 让位后流程② id 覆盖表命中改名 banana |

spec：https://html.spec.whatwg.org/multipage/common-dom.html#ref-for-concept-node-append
（ARIA element reflection scope——T_E ∈ C_A 树链判定）、
https://dom.spec.whatwg.org/#concept-node-pre-insert 、

## 常驻测试

- `test_aria_cross_doc_scope_and_get_element_by_id_shadow_move_r5206`
  （part21.rs）四断言：crossDoc null / afterMove visible / parent host /
  gEBI banana——两上游案的最小镜像面。
- WPT aria-element-reflection.html 全绿（通道常驻）。

## 门禁

- `cargo test -p zero-engine --lib` **3294 passed 0 failed**（shim 共享路径全量
  桥接测试含新案）。
- `cargo fmt -p zero-engine -- --check` 干净；`cargo clippy -p zero-engine
  --all-targets` 无 warning；shim 拼接 `node --check` 通过。
- stash 前后对照 `element-render-blocking-018` 同败——SyntaxError 面为
  render-blocking 既有域失败，与本轮改动无关。

## 余账

- handle 父 + plain 子的 parentNode 重指同缝（R368 盖章使 R84 gated
  `!child.__zwHandle` 跳过）——无现有断言面，未触及；后续案浮出再修。
- `insertBefore` 的 plain 子 sel 父路径未审（两上游案均为 appendChild 形态）。
- bench：dom_bindings_bench 覆盖 native/polyfill 桥路径，不含 shim 内
  getElementById 走查；本修新增成本 = 每祖先跳一次对象属性查（有同步记录时
  替代 `__zw_parent` host 回调，更廉价），无门禁覆盖路径（P5–P8 先例同记账）。

## R5206 续：视图回放 latest-wins + R342 离一修复（同日）

P9 通道验证后追查 ambiguous-ampersand 2F 时发现共享路径两个真实缺陷
（`js_dom_bridge/callbacks.rs` `with_query_view_doc` 视图回放筛）：

1. **R342 id 去重离一 panic**：`&frag[start + 4..]` 应为 `start + 3`
   （`id=` 是 3 字节）。双重后果：(a) 分片在 `id=` 处截断时 panic（byte
   index out of bounds——char-by-char document.write 探针实证，runner 整体
   abort）；(b) 带引号 id（`id="foo"`）rest 首字节恰越过开引号，
   `strip_prefix('"')` 恒 miss——**去重自落地上报（uievents-compat 尾簇 13）
   起从未生效过**。修复 `start + 3`，带引号 id 去重激活（无引号形态保守
   不判重，语义不变）。
2. **同 turn 重置覆盖（窗口内 latest-wins）**：视图回放只应用 IAH 子集
   （R3029——SetInnerHtml 不应用），而 R5001 document.write 流每步
   `body.innerHTML = 基线` + `insertAdjacentHTML(累计缓冲)` 各入队一对
   SIH+IAH：全量重放把 N 个截断分片子树堆上基座（161 写 → 视图 160 divs，
   probe 实证 view_divs 全量重放态）。修复：窗口内同 selector 的 IAH 被更晚
   SetInnerHtml 覆盖则不重放（只保留最后一次重置之后的插入）；增量链前提
   加严——链段 `[0..prev_count)` 含 SIH 即退全量重建（链基座缺重置语义）。
   修复后视图收敛 base + 最终满缓冲（probe：21 divs → 3 divs，qsaWritten
   19 → 1），与活文档一致。

**ambiguous-ampersand 2F 维持挂账**：host 视图层已收敛（探针 view_divs=2
正确），残差在 **shim 侧读路径**——R5001 每写步的 innerHTML+IAH 双簿记各挂
一批 `_zwSelPendingParent` 解析补偿节点，整页单程序执行无 host apply
（探针 `_zwApplyGeneration()=0`，drain 只在页尾），apply 代际原语
`__zw_apply_generation_bump` 收敛尝试实证无效（gEBT 恒 4、children 层反爆
498——多层独立积累器：pending 桶/children 融合/tag 视图读数分裂
gEBT4/Kids6/Children8/ihl1675）。全案归 async-apply 架构缝 document.write
子集（与 P9 已修的 aria 同 turn 移树子集同缝异面）。

### 门禁（续）

- 常驻测试 `view_doc_replay_tests` 三案（callbacks.rs）：截断分片不 panic /
  带引号 id 去重激活 / latest-wins 重置覆盖。
- `cargo test -p zero-engine --lib` **3297 passed 0 failed**；fmt/clippy
  干净；shim 拼接（part01+01b+02..07）`node --check` 通过。
- 全通道复测 **61345P/63F/13T/1U = 61422 与 P9 后基线逐位一致，零回归**
  （两个行为变更：去重激活 + latest-wins，均无案翻红）；fail 集构成不变
  （58 render-blocking + 4 parsing + 1 dom）。
