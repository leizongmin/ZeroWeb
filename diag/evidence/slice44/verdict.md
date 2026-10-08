# slice44 verdict — live 集合残余三件（静态查询面 parsed 移除 stale / NA 谓词根部统一 / move 生命周期钉）

分支 `slice44-live-collection-residuals`；base `8c5752049`（origin/main tip，fetch 实测 = slice43 合并树，main 未前进）→ `cfec8f59e`（三钉常驻，自然红立项）→ `7c44d04de`（修复）。红档：[red/README.md](red/README.md)。

## 三态判定总览

| 任务 | 判定 | 处置 |
|---|---|---|
| ① 静态选择器查询面 parsed 子树移除 stale（slice43 残余②） | **document 面 = 真实缺陷**；元素面跨容器 = **边界保持** | document 面最小修复（共享漏斗 `_zwPendingRemovedSels`）+ 常驻钉 `static_selector_query_parsed_subtree_remove_s44`；元素面钉内边界断言 + 残余申报④ |
| ② `_zwDocContains36` 根部统一评估（slice43 缺陷轮 I-2 转池） | **真实缺陷**（head 子树 / documentElement 自身命名元素漏判） | 根部统一至 documentElement（widening-only 论证）+ 常驻钉 `named_access_doc_tree_root_head_mount_s44` + 双对照臂零回退 |
| ③ move 生命周期（slice43 残余④ xS-1 钉候选） | **现状正确**（slice43 勘误语义维持） | 覆盖钉 `parsed_subtree_move_lifecycle_s44`（HEAD 绿）+ 判别力由 M-D 变异 RED 承担 |

## 任务①：静态选择器查询面 parsed 子树移除 stale → 真实缺陷（document 面），修复

**自然红实证**（树 `cfec8f59e`，`red/natural-red-s44.log`，1P/2F EXIT=101）：常驻钉修前实测——`document.querySelectorAll('img')` 移除后 stale 3（应 1）、`document.querySelector('#w44 img')` 仍解析已移除后代、`document.querySelector('img')` 首命中为已移除 c44。slice43 卡面探针（一次性，未入提交）为过程证据，本轮以常驻钉固化自然红。修前探针全值归档 red/README.md（`__r_qsa_doc=3 / __r_qs_deep="false" / __r_qs_first="c44"`）。

**可观测后果**：同步 turn 内移除 parsed 子树后，document 面选择器查询返回已离树节点（length / item / 身份可比对），与 spec「查询面限当下 document tree」相悖（https://dom.spec.whatwg.org/#concept-node-list-alive）——非沉底不可见。

**修复面评估（共享漏斗判定）**：slice43 修复面 = ID 查询面（remFlat 展开使 pendingRemoved 含 parsed 后代，ID 面即时 null）。本轮实证 document QSA/QS 走**独立枚举路径**（`__zw_query_all`/`__zw_query_match` 快照 sel 列表），但消费同一张全局 `_zwPendingRemoved` 表——**无需第二个移除记录入口**，补一个共享 sel 视图漏斗 `part05.js:10219 _zwPendingRemovedSels()` 即可双面（QS :2128 剔除+重查、QSA :2533 过滤）合流。与 slice43 remFlat 展开同表消费，无新账本。

**元素面边界保持（同任务内的三态分叉）**：part04 querySelectorAll 系（元素面）**有意不消费**全局 pendingRemoved——R310 同桶口径（bucket(queried container).removed）是内部写 API 的 stale 窗口契约：`.rows` getter = `_wrapSelector(sel).querySelectorAll('tr')`，`deleteRow` 靠快照 stale 行定位删除目标（`test_html_table_delete_row_and_section_r3243` 实证，本轮修复首轮试错中全局过滤直接打红该测试，机理见提交 7c44d04de 说明）。同桶面（R310）已有正确覆盖（钉内 `__r_qsa_body=1` 对照臂）；跨容器桶外 stale 维持「换代即愈」，申报为残余④。修复首轮曾试改元素面，实测打红后**全量回退 part04 改动**（零残留，git diff 实证；打红实证在册者为 `test_html_table_delete_row_and_section_r3243`，试改轮 RED log 未归档、打红全名单不可追溯——双首轮缺陷评审 I-2 口径收窄），边界判定由 R310 契约读码 + 钉内边界断言 + 回退后全量门禁绿支撑。

**变异 RED**：M-A 撤 QSA 过滤 → 红在 `__r_qsa_doc`（3 vs 1，回到探针 stale 值）；M-B 撤 QS 剔除+重查 → 红在 `__r_qs_deep`。分 site 隔离 run + 逐字节还原自证（red/README.md）。

## 任务②：`_zwDocContains36` 根部统一（I-2）→ 真实缺陷，根部统一修复

**自然红实证**：head 内挂载 `id='hd44'` 命名元素，修前 `window.hd44 === undefined`（body 根谓词 `body.contains(el)` 对 head 子树恒 false）。spec named objects 限 document tree（https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object）——head 属 document tree，漏判为真实缺陷。

**修复**（part05.js:11002）：根部 `document.body || documentElement` → `document.documentElement`。**widening-only 论证**：body 存在时 `body.contains(el) ⇒ documentElement.contains(el)`（body ∈ documentElement 子树），六个调用点/四组面（attr join :10922 / collectMatches :11031 / register 重核 :11131+:11177 / 注销与补偿扫 :11149+:11206——双首轮缺陷评审 I-4 计数口径勘误，原「四消费面」为四组归并）对原 true 判定零变化，新增命中仅 head 子树与 documentElement 自身（`_zwNodeContains` 含自身，:10996 注释在册）；body 缺失（detach 态）时 documentElement 兜底，语义不窄化。detached / shadow / 异文档 contains 恒 false，既有排除面保持。

**零回退实证**：钉内双对照臂（detached `dt44` 不入册 = s43 pin2 口径保持；body 挂载 `b44` 单命中 = 既有面零回退）+ 变异 RED M-C 撤根部回退 → 红在 `__r_head_named`；全量门禁 `make test` 20,203P/0F（既有 named-access 钉网 `named_access_shadow_tree_excluded_s40` / `attr_join_detached_gate_s43` 全绿）。

## 任务③：move 生命周期（xS-1）→ 现状正确，覆盖钉 + 判别变异

**判定**：sel 父带 parsed 后代「移除→回插」，后代经 addFlat 重并集合（R333 门）并从 pendingRemoved 摘除（R51c added 方向摘除点 :10413）——集合长度回复 2 / fresh 查询复见 / ID 面复见 / NA 全局重现，钉在 HEAD 全绿（`parsed_subtree_move_lifecycle_s44`），无缺陷可修。

**判别力**（覆盖钉的变异 RED 义务）：M-D 撤 added 方向摘除点 → 红在 `__r_moved_id`（false vs true）。**判定位与立项预估（`__r_moved_fresh`）不同**，实测归因更有价值：撤摘除点后 m1/m2 同残 pendingRemoved+pendingAdded，fresh tag 面经 pendingAdded 双源并集仍复见（先过），ID 面快照命中被 stale pendingRemoved 剔除且 `#id` pending 回退不触发（host 快照仍命中；该「回退不触发」精确成立于 getElementById 形态——pendingEarly/回退逐条目 `_zwPRSet` 守卫排除，`querySelector('#id')` 字面量回退无该守卫、同态会命中——双首轮缺陷评审 I-5 形态分叉注记，防后续复用归因于 '#id' 形态）——摘除点是 move 闭环的承重路径，非冗余防御。

## 顺带修复申报（测试机制现代化，非行为变更；提交 7c44d04de 内明示）

修复首轮试错暴露两处既有测试依赖「刚移除节点仍可查询」的旧机制（document QS 剔除后该面收窄，属机制面收敛的连带）：

1. `part03.rs` r47（mutation records）：`r1.remove()` 后重查询取身份比较 → 改为移除前捕获 `var r1 = ...`，断言 `removedNodes[0] === r1`。测试意图（records 身份）不变。
2. `part20.rs` iframe fetch：`document.body.innerHTML = ...` 替换后查询 iframe → 改为替换前捕获引用。测试意图（contentWindow.fetch 按 iframe URL 解析）不变。

两处均不弱化断言；同提交内声明，评审可按 hunk 逐行核对。

## 变异单腿充分性

三钉与全部变异位点均在 zero-engine 域（js_dom_shim / js_dom_bridge_tests）。engine 不在 `QUICKJS_TEST_CRATES`（Makefile:190）→ quickjs 测试腿不执行 engine 测试，单腿 RED 结构性充分（slice41/42/43 同判例）；quickjs clippy 腿（QUICKJS_CLIPPY_CRATES 含 engine）独立过闸。

## 锚算术（逐级闭合）

| 轮次 | 树 | 电池 P/F/I/腿 | 增量归因 |
|---|---|---|---|
| slice43 合并基线 | 8c5752049 | 20,200/0/198/70 | 主证据逐行求和重算（gates/maketest-mergetree-s43.log：70 测试二进制，P=20,200 F=0 I=198），非照抄卡面；fetch 实测 main 未前进 |
| slice44 终态 | 7c44d04de | **20,203/0/198/70**（实测，gates/maketest-s44.log 求和） | +3 = slice44 三钉（part44.rs，engine lib v8 腿）；engine 不在 QUICKJS_TEST_CRATES → quickjs 腿零变化、腿数 70 恒 |

20,200 + 3 = 20,203 精确闭合。

## 门禁（终锚 `7c44d04de`）

| 门禁 | 实测 | log |
|---|---|---|
| fmt | FMT_EXIT=0 | gates/fmt-s44.log |
| clippy v8（workspace -D warnings） | CLIPPY_V8_EXIT=0 | gates/clippy-v8-s44.log |
| clippy quickjs（QUICKJS_CLIPPY_CRATES -D warnings） | CLIPPY_QJS_EXIT=0 | gates/clippy-quickjs-s44.log |
| make test（test-guard 包裹） | 20,203P/0F/198I/70 腿，MAKETEST_EXIT=0（首跑即绿，零 flake 处置） | gates/maketest-s44.log |
| make reftest | 704/704（真通过-可信 546 + 可疑 111 + 近似 47，与 slice43 基线逐项同值零漂移），REFTEST_EXIT=0 | gates/reftest-s44.log |
| WPT named-access targeted | 43P/1F/2T 同位，NAMED_ACCESS_EXIT=1（既有非 Pass 身份恒定：cross-origin-named-access.sub / named-objects / window-named-properties，与 slice43 基线逐一吻合） | gates/wpt-named-access-s44.log |
| release build | RELEASE_BUILD_EXIT=0 | gates/release-build-s44.log |

## 残余申报（保持 + 本轮新证）

1. slice43 ①（slice27 静态单命中面不 live、stale 至换代）——保持。
2. slice43 ③（morph 跟随面 >1 命中不升格集合）——保持。
3. slice43 ⑤（pin2 第三态无断言备案）——保持。
4. **元素面跨容器 parsed 移除 stale（边界保持，本轮明示）**：元素面（part04 元素查询面系——querySelectorAll 系 / closest / 集合 matches 谓词同伞，双首轮测效评审 xI-3 口径扩展；documentElement/非同桶容器）对已移除 parsed 后代维持 stale 至换代（钉内 `__r_qsa_html=3` 边界断言）——R310 同桶口径是内部写 API stale 窗口契约（.rows→deleteRow，r3243 实证），全局过滤破坏契约且本轮实测打红 3 测试；document 面已修复为本钉主判定，元素面 stale 窗口换代即愈，归后续轮次（如需，可评估按「读面/写面」拆分消费口径）。
5. **document 面查询对 handle-form 移除条目 stale（双首轮缺陷评审 I-1 新证，本轮不修）**：`_zwPendingRemovedSels`（part05.js:10219-10228）仅收含 `__zwSelector` 的 sel-form 条目；createElement 产物 handle-form proxy（get trap `prop === '__zwSelector'` 恒 null，part03.js:15511）经 remFlat 记账后不入 Set——前批已 apply 的动态创建节点于本批同 turn `remove()` 后，document QS/QSA 快照 stale 命中不被剔除，与本轮修复面同款 spec 偏离（https://dom.spec.whatwg.org/#concept-node-list-alive）。即修复面内部 sel-form（已修）/handle-form（未修）不对称；对照：getElementById 面按 proxy identity 比对（`_zwPRSet().has`）两态均覆盖、无此缺口。口径同④（apply 前窗口、换代即愈），归后续轮次（可评估漏斗补 `__zw_handle_for_selector` 反查后 identity 核对，与 ID 面同法）。
