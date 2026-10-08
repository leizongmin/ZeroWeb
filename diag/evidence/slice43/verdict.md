# slice43 裁决——live 集合成员维护两缺口收口

分支 `slice43-live-collection-maintenance`，base = origin/main tip `81b25c70d`（fetch 后实测前移：a6c913e43 → 81b25c70d 含 a777e6908 R4997 flex AR 修复——sizing.rs/float_positioning.rs **零新增测试**，基线电池数不受影响；按卡「产品变更轮直接基于新 tip」执行）。commit 结构：`6fa7a564d`（钉，自然红立项）→ `8a43654d7`（修复，最小共享路径）→ evidence/docs。

基线（slice42 集成树 a6c913e43，PR #105 合并态）：电池 20,197P/0F/198I/70 腿、reftest 704/704、WPT named-access 43P/1F/2T。

## ① slice32 集合面 parsed 子树移除成员维护缺口 → 真实缺陷，最小共享路径修复

**自然红实证（xS-3 补证收口）**：常驻钉 `named_access_collection_parsed_subtree_remove_s43`（part43.rs :40）在修前码（钉提交 `6fa7a564d` 树 `9ec6a77cf8ac`）实测红——`red/natural-red-s43.log`（0P/2F，EXIT=101）：parsed 容器（`<div id='w43'>` 含双 `name='p43'` img）整树移除后 held NA 集合 stale 2（spec 0）。tag 集合 held/fresh 两面同族红由修复态绿 run 反向覆盖（`red/fixed-green-s43.log` 修复后全绿）。

**判定**：spec 集合视图限当下 document tree（https://dom.spec.whatwg.org/#concept-collection），子树移除成员随树离场；HTMLCollection live 语义要求 captured 引用反映移除。修前 `_zwHCCollectSubtree` 对 sel 父仅回落 pending 桶 added（R51c），快照解析后代（原始 HTML / 已应用 innerHTML 态）不入 remFlat → pendingRemoved 剔除面、live 集合 drop 面、fresh 构建面三处同源 stale（slice40 补偿扫仅覆盖 `_zwNADynEls` 动态账本面，parsed 面在册）。

**修复**（part05.js `_zwHCCollectSubtree`，提交态 L10232-10240）：sel 父分支在 R51c 桶回落之后补 parsed 后代展开——经 `_childNodeList(sel)`（R55 基底缓存 + overlay，元素 proxy `_wrapSelector` identity 稳定）枚举当下子面递归收集。共享路径生效面：`_zwPendingRemoved` 全局表（fresh 查询构建剔除）、per-parent 桶（childNodes overlay）、live 集合 drop 循环、NA 动态注销——单点展开四处对齐。与桶 added 重叠条目由下游 Set 判重吸收（对冲/removed 记账幂等）。

**判定面如实收窄（一次性探针，TEMP-PROBE 未入提交）**：修复后 `getElementById('a43')`（被移除 parsed 后代）即时报 null——ID 查询面由同一 remFlat 展开顺带修复；**`querySelectorAll('img')` 仍 stale 1、`querySelector('#w43 img')` 仍可解析**——静态选择器查询面走独立枚举路径（part04 querySelectorAll），不消费 pendingRemoved，属同族新证残余（见残余申报②）。

**变异 RED**：M-A 单腿撤 parsed 展开块（vs `8a43654d7` 纯 delta，hunk 头 L10232）→ pin1 FAILED/pin2 ok（`red/mutation-red-s43-a.log`，EXIT=101）；逐字节还原 sha256 自证 `14143a06…`（=提交 blob）。

## ② slice42 缺陷轮 I-5：attr 维护面连接性门 → 真实缺陷，最小修复（有实际可观测后果）

**自然红实证**：常驻钉 `named_access_attr_join_detached_gate_s43`（part43.rs :115）修前实测红——detached `createElement('img')` setAttribute('name') 命中既有 NA 集合名即混入（`window.q43.length` 3 vs 2，`red/natural-red-s43.log`）。**可观测后果评估**：`window.<name>` 文档级集合成员数含 detached 元素（length/索引/namedItem 可读），且可触发 morph 形态翻转——非沉底不可见；spec named objects 限 document tree（https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object，WPT basics "not reachable" 面）。三态判定：**真实缺陷 → 修复**。

**修复**（part05.js `_zwNAAttrChanged` join 臂，提交态 L10903）：成员资格判定补 `_zwDocContains36(el)` 门，对齐 R54/R333 in-doc 口径（childList 并入门、动态注册门、构建期门同款）。账内 detached 陈旧成员经同门失格剔除（自愈）；in-doc 插入经 childList 面重新并入，无永久丢失。对照臂实证 in-doc attr join 不误伤（`__r_indoc_join`=3 绿）。

**变异 RED**：M-B 单腿撤门谓词（`&& _zwDocContains36(el)`，hunk L10900）→ pin2 FAILED/pin1 ok（`red/mutation-red-s43-b.log`，EXIT=101）；还原自证同上。

## 变异单腿充分性

两变异位点均在 `crates/engine/src/js_dom_shim/part05.js`（zero-engine 域谓词）。engine 不在 `QUICKJS_TEST_CRATES`（Makefile :190）→ quickjs 测试腿不执行 engine 测试，单腿 RED 结构性充分（slice41/42 I 级硬标准）。

## 锚算术（逐级闭合）

| 轮次 | 树 | 电池 P/F/I/腿 | 增量归因 |
|---|---|---|---|
| slice42 集成基线 | a6c913e43 | 20,197/0/198/70 | PR #105 合并态（卡载基线） |
| 本轮 base | 81b25c70d | 20,197（预期同值） | a777e6908 R4997 零新增 `#[test]`（git diff 实测），81b25c70d 纯 docs |
| slice43 终态 | 8a43654d7 | **20,199/0/198/70** | +2 = slice43 双钉（part43，engine lib v8 腿）；engine 不在 QUICKJS_TEST_CRATES → quickjs 腿零变化、腿数 70 恒 |

20,197 + 2 = 20,199 精确闭合。reftest 704/704 零漂移；WPT 43P/1F/2T 同位（Fail/Timeout 身份 = cross-origin-named-access.sub / named-objects / window-named-properties，与 slice42 基线逐一吻合，零 P 回退）。

## 门禁（终锚 `8a43654d7`，TREE_SHA `c2d1cc741059…`）

| 门禁 | 实测 | log |
|---|---|---|
| fmt | FMT_EXIT=0 | gates/fmt-s43.log |
| clippy v8（workspace -D warnings） | EXIT=0 | gates/clippy-v8-s43.log |
| clippy quickjs（QUICKJS_CLIPPY_CRATES -D warnings） | EXIT=0 | gates/clippy-quickjs-s43.log |
| make test（test-guard 包裹） | 20,199P/0F/198I/70 腿，MAKETEST_EXIT=0（首跑即绿，零 flake 处置） | gates/maketest-s43.log |
| make reftest | 704/704（Layout 502 + Text 202），REFTEST_EXIT=0 | gates/reftest-s43.log |
| WPT named-access targeted | 43P/1F/2T 同位，NAMED_ACCESS_EXIT=1（既有非 Pass 身份恒定） | gates/wpt-named-access-s43.log |

## 残余申报（本轮新证，未修）

1. slice27 静态单命中面（安装期元素全局）不 live、stale 至换代——申报保持（slice42 :69 口径）。
2. **静态选择器查询面 parsed 子树移除 stale**（本轮探针新证）：`querySelectorAll('img')` / `querySelector('#w43 img')` 对已移除 parsed 后代仍可解析至换代回收——独立枚举路径（part04 querySelectorAll 系），不消费 pendingRemoved；与①同族（parsed 面换代即愈），归后续轮次。ID 查询面（getElementById）已随本轮 remFlat 展开顺带修复（探针实证 null 即时）。
3. morph 跟随面 >1 命中不升格集合——slice42 :62 口径保持。
