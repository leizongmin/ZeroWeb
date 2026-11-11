# M2 切片 394（s394）——textarea wrap=hard wrapping transformation（2F→P）+ img 未加载盒 2F 归属定谳（挂账：动态 append rect 供给链）

日期：2026-11-11（承 s393 下一步：①textarea wrap=hard 折行变换；②img unloaded attr sizing 归属探针）
域：html/semantics/forms/the-textarea-element + embedded-content/the-img-element
性质：**双刀轮——①施工（part02/part04 entry 构造面）；②归属探针（定谳后挂账，s394 内不施工）**

## 刀① textarea wrap=hard wrapping transformation（2F→P）

**driving 面**：`the-textarea-element/wrapping-transformation.window.js`（s392 分诊：FormData API 值未按 cols 折行，期望 `ABCDEFGHIJ\nKLMNOPQRST\nUVWXYZ` 得原文；LF 归一臂过）。

**spec**（HTML §4.10.11 The textarea element，wrap=hard 状态，#attr-textarea-wrap-hard）：wrap=hard 时 textarea 的值在 **entry 构造面**（constructing the entry list）应用 **wrapping transformation**——按 cols 无条件折行（字符计数、非 layout 面）：LF 切行，每行超 cols 段逐段截断补 LF；变换幂等（输出行均 ≤ cols，重变换零变化——round-trip 判据面）。**不进 API value**（`.value` 读面不折行，仅 entry 面——driving test 的 value/FormData 双断言面与此一致）。cols 取 IDL face（R5009 片 d limited 解析 fallback 20）。soft/默认态原值返回。

**旧实现偏差**：entry 构造两个 walk（FormData 构造器 + form 提交序列化）均直推 `f.value`，无 textarea wrap 臂。

**修复**（三处）：
- part02.js 新增共享 helper `_zwTaHardWrap(val, el)`（`_zwFdEntry` 邻位——跨 part 作用域同 `_zwFdEntry` 先例）：wrap 属性 ASCII 大小写无关判 hard；cols 读 `el.cols`（number 守卫 + min(_c, 2147483647)）；LF 切行 + while 超宽截断 + join('\n')。
- part02.js FormData 构造器泛化推入臂：`tag === 'textarea'` 时经 helper（tag 小写约定同 4233 行既有面）。
- part04.js form 提交 TEXTAREA 专用臂：经 helper（urlencoded/multipart/text-plain 三序列化共用 `_pairs`，单点覆盖）。

**验证**：`FILTER=wrapping` solo **8P/0F**（原 2F 全翻转：「Wrapping happens with LF newlines」+「hard-wrapping should honor the col count unconditionally」；5 round-trips + LF 归一臂全绿）。日志 /tmp/s394/solo-wrap.log。

## 刀② img unloaded attr sizing 2F：归属定谳（探针三轮，探针文件用后即删）

**driving 面**：`the-img-element/update-the-image-data/not-broken-while-load-task-scheduled.html` 2F（`run_test("src")` / `run_test("srcset")`——img.width/height=50 + 未加载 src，append 后同 turn 断言 clientWidth/clientHeight=50 + computed 50px）。s392 分诊假设：engine attr→style 面 vs layout 未加载盒面。

**探针矩阵**（zw394-probe-img.html v2/v3/v4，dump 内嵌子测试名随结果行透传——Pass 行不带 assert 消息通道）：

| 臂 | 位形 | computed | client/gBCR |
|---|---|---|---|
| A/A2 | 动态 img attr 面（new Image IDL setter / setAttribute）未加载 | **50px×50px** | **0×0 / 0,0,0,0** |
| B | 动态 img CSS style 50px 未加载 | **50px×50px** | **0×0** |
| C | 动态 img attr 无 src | 50px×50px | 0×0 |
| D | 动态 **div** CSS 70px（2s 后，frame-driven 多帧） | **70px×20px** | **0×0** |
| E | **静态** div CSS 60px 参照 | 60px×20px | **60×20** ✓ |
| S1-S4 | 静态 img attr / 静态 div（same-turn + 2s） | 50px / 60px | **50×50 / 60×20** ✓ |
| I1-I4 | 动态 img/div **带 id（sel 身份）** same-turn + 2s | 50px / 70px | **0×0 / 0,0,0,0** |

**定谳（三面剥离）**：
1. **attr→style presentational hint 面：健康**。动态 img attr 面 computed width/height=50px（style-system collect_presentational_hints img 臂 crates/style-system/src/lib.rs:3264 与加载态无关，实证吻合）；CSS 面同。
2. **静态元素 geometry 面：健康**。静态 img/div same-turn 即有 rect（layout 先于脚本执行），2s 稳定。
3. **动态 append 元素 rect 供给链：缺席（根因）**。动态 append 产物无论身份形态（handle 或 id/sel）、无论时点（same-turn 或 2s 多帧后）、无论标签（div 对照排除 replaced 语义），gBCR/clientWidth 恒 0——**JS mutation 产物不进 rect 查询文档**。computed style 面同时段同元素正常（70px）——style 查询文档与 rect 查询文档分裂。

**与既有在案证据互证**：`_formControls` t8f 注释（bilibili 勘根）——host 视图 `with_query_view_doc` 不应用 handle 链 mutation；in-process `__zw_getBoundingClientRect`（js_dom_bridge/callbacks.rs:733）经 `find_by_selector`（live doc 优先回落 query view doc）+ 渲染后 rect snapshot，动态产物双源皆 miss。s284 `__zw_resolve_handle_selector` 刻意收窄 handle 解析面（全局点亮曾使 clone img 真实 rect 泄入 attr 期望面，naturalWidth 族 30 行 REGRESSED，2026-11-04 勘根）——handle 单点非全因（I 臂 sel 身份仍 0），修复须同时补「动态子树进 rect 查询文档」与「读数时点 update-the-rendering 语义」（rect_bridge.rs:239 CSSOM View 注释在案），并与 s284 收窄面、naturalWidth attr 期望面纠缠。

**裁定**：根因属**共享路径基建缺口**（动态 append rect 供给），非 img/attr/未加载盒特异；2 键维持 2F，**s394 内不施工**，挂账池入册（施工独立切片，需先勘 live doc 发布与 mutation 同步链）。失败签名证据：/tmp/s394/solo-driving.log（2F，退出码 2）。

## 全树验收（sweep join）

- 基线：s393 验收 sweep（/tmp/s392/n-sweep.log）unique **8463** (key,state)＝当前 HEAD 组合态。
- 实测（/tmp/s394/sweep.log，8 job 全语料）：8466 行 = **8203P/226F/37T；unique 8463**。
- join 归因（LC_ALL=C，(key,state) 对 sort -u 后 comm）：
  - **2F→P**：wrapping-transformation 两键（「Wrapping happens with LF newlines」+「hard-wrapping should honor the col count unconditionally」）——恰为刀①，零多修零漏修。
  - 零 P→F、零 Timeout 位移、零意外键；刀②两键维持 F（226F 口径 -2 = 228→226）。

## 验证

- solo 刀①：solo-wrap.log **8P/0F**；solo 刀②证据：solo-driving.log 2F 签名（退出码 2）。
- 全树 sweep join：见上节——**2F→P 精确归因 + 零意外位移**。
- 单元测试：test-guard 包裹 `cargo test -p zero-engine -j 2`（Rust 零改动，shim JS 变更面）**3834P/7F**——7F 与 s391/s393 组合态在册红完全同形（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_service_worker_messages_dispatch / response_body_readable_stream_r2967 / t8k_websocket ×2），零新增红（exit 101 即在册红集所致，非新失败）。
- 产品 smoke：`make product-smoke` **PASS**；`make product-smoke-legacy` **PASS**（stage5a/5b 退出码双 0）。
- 探针 hygiene：zw394-probe-img.html 三版迭代后用后即删（先核 `git ls-files` 无跟踪态——wpt-data 为 gitignore 本地数据）；管线 stage2 在 sweep 前删除，sweep 语料=收尾树态。
- 验收管线：串行五阶段（driving solo → 删探针 → 全树 sweep → 单元 -j 2 → smoke 双），防 total-mem 争用（s393 教训①延续），/tmp/s394/stage*.done 逐段旗标。
- fmt/clippy：本流零 Rust 改动，不适用（s391/s393 先例：仅本流触面执行）。

## 下一步

- **s395**：动态 append rect 供给链勘根（本轮刀②挂账项的直接延续）——live doc 发布时机与 mutation 同步链（LIVE_QUERY_DOC 何时发布/append 后是否增量同步）、rect snapshot 对动态子树的刷新路径；先勘根定最小施工面，与 s284 收窄面、naturalWidth attr 期望面（30 行回归史）的边界一并在勘根产物里钉死再动刀。
- 挂账池更新：+动态 append rect 供给链（本刀②，影响面含 not-broken-while-load-task-scheduled 2F——同类「动态元素几何」断言散在几何/可见性判据面，修复收益可能外溢）；维持：渲染流 5 面（basic 亚像素 / error alt-box / svg-zoom / lineheight / sizes-auto 几何半边）→ rendering-compat、form-submission-target 6T → navigation M3、sizes-auto lazy 半边待勘根；A2+D 簇立项征询待用户决策。
