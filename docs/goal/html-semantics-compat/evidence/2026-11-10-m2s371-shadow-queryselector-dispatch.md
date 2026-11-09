# M2 切片 371（s371）——runner 宿主链 shadow 选择器/坐标双根因修复（closedby-complex 16F→9F）

日期：2026-11-10（承 s370 断点续勘，本轮收口）
域：html/semantics/interactive-elements/the-dialog-element（dialog-popover-closedby-complex shadow-slots 段）
性质：runner 宿主链双根因修复——**16F→9F（7 案翻绿，s173 LOST 基线首次收窄）**

## 承接与排除面（s370 断点：runner 执行循环生命周期）

s370 定谳最小红态 = 3 层宿主链 + runner 执行，engine/webview 全入口绿，断点收敛 runner 侧。本轮仪器化三面逐一排除：

1. **gBCR 回调链**（`callbacks.rs __zw_getBoundingClientRect`，ZW_GBCR_DEBUG env 门控 eprintln 仪器化，勘根后完整移除净零）：LIVE_QUERY_DOC live doc 优先臂命中 ×861 无失效；with_query_view_doc 回落臂正常；rect snapshot map 按 node_id 查表正常。
2. **rect snapshot 键集**：map 含 shadow 内容 id；键集随 dialog/popover 开合相位动态收缩 = 正常语义（display:none 元素出快照）。
3. **探测环结构性刷新链**：shim outerHTML 序列化保真（echo 模板存活 srCount=2 嵌套完整，`documentElement` 无根不触发 s180 剔除门）；mutation 注入后 `refresh_if_html_changed` 返 false = mutation 应用路径已同步 cached_html（非丢刷新）；追加注释强制分叉走真刷新臂后 rect 保持非零——「忠实序列化→重渲染→物化→rect 快照」全链保真。

排除后收敛 runner 侧 clickOn→testdriver 选择器链（td_debug `ZW_TD_DEBUG=1` `[ptr-resolve]` 日志 + gitignore 区探针页定位）。

## 双根因定谳

**根因 A（选择器分派缺失）**：shim `document.querySelector` 无 zwsh 前缀分派 → R158 守卫（`_zwQueryGuard`）→ engine `selector_is_valid` 不识引擎命名空间串抛 SyntaxError → testdriver stub `selectorFor` 信任门 `document.querySelector(zwSel31) === element` 失败 → `__zw_selector_for_handle` 反查（shadow 内容 post-s370 返 None）→ 启发式 plain 选择器（`.popoverA`）与 normal 段孪生元素碰撞 → clickOn 解析到 normal 段元素 → 对 shadow popoverA 而言是「外部点击」→ light-dismiss 误关。

**根因 B（坐标串撕碎）**：`resolve_pointer_target` 输出 `sel|px|py` 以 `|` 分隔，而 zwsh compound `zwsh<len>:<host>|<local>`（engine shadow_address 铸造）内部含 `|` → best 命中 shadow 元素时（shim qSA `body, body *` 遍历含 shadow 内容）sel 撕碎、坐标 parse None → s176 elementFromPoint 回落（仅 zwsh 前缀采信）整段跳过 → 回落 origin(body) 派发 → click 落 body → light-dismiss 误关。

两根因共同出口 = light-dismiss 收到外部点击，16F 主矛盾「popoverA should be open」全由此驱动。

## 施工

1. **part01.js**：从 `_zwLdResolveTarget` 抽出共享扫描器 `_zwShadowContentByCompoundSel`（声明式 open 根注册表扫描 + 子树 `__zwSelector` 同串比对）；`_zwLdResolveTarget` zwsh 前缀同源复用（焦点面，行为不变）。
2. **part06.js**：`document.querySelector` zwsh 前缀分派置于 R158 守卫之前（守卫的 CSS 语法校验对命名空间串抛 SyntaxError）。**s370 挂账②「querySelector(zwsh) SyntaxError 缺口」销账**。
3. **testharness.rs `resolve_pointer_target`**：坐标字段自尾反析 + hit 段重 join——`|` 在 light 结构串不出现，尾两段恒坐标（zwsh compound 形态时 hit 段含内部 `|`，join 还原）。
4. **钉测**：
   - webview `test_shadow_content_gbcr_webview_s370` 收紧：SyntaxError 二态钉 → 恒命中钉 + 信任门身份钉（`querySelector(zwsh)` 返回 findElements 枚举的同身份 proxy——`selectorFor` 语义前置）。
   - webview 新钉 `test_runner_refresh_shadow_rect_s371`：探测环结构性刷新 shadow rect 保真常驻（s370 勘根排除矩阵转钉：序列化保真断言 + mutation 后 cached_html 同步 + 强制分叉真刷新臂 rect 非零）。

## 验证

- **solo 域跑（最终树，release runner 03:39:51 晚于全部源码 03:39:06）**：`make testharness-html-semantics FILTER=dialog-popover-closedby-complex` → **31P/9F**（16F→9F，7 案翻绿；normal 段 20 case 全绿不动）。日志 evidence/logs/s371-solo-closedby-complex.log（.log gitignore 本地留存，计数 31/9/0 机械核过）。
- **残余 9F 分布**（下一切片勘根域）：
  - over-dismiss 4 案（应保持 open 却被关）：popoverB m/modalB、popoverB modalA/m、dialogB modalA/m、popoverB modalA/modalB——断言 `popoverB/popoverA should be open expected true but got false`。
  - under-dismiss 5 案（应关内层 popover 未关）：dialogB m/m、dialogA m/m、dialogA m/modalB、dialogA modalA/m、dialogA modalA/modalB——断言 `popoverB/popoverA should be closed expected false but got true`。
- **webview lib 全量（最终树两轮）**：**847P/2F**——2 红 = `image_decoder::proxy_mode_decodes_png_correctly`（spawn 环境红，s168c/s256/s336 在案）+ `csp_gate::csp_connect_gate_blocks_fetch_sh1_m2s5`（s348 环境红），逐名同构零新增红。首轮第三红 `service_worker_runtime::navigator_update_activates_replacement_without_a_controlled_client`：与改动面无机制交集（无 shadow/选择器路径）、单跑 `--test-threads=1` 绿、次轮全量绿——s370 先例同构定谳并行负载 flake 非回归。
- **engine lib 全量**：**3826P/7F**——7 红与 s370 基线逐名同构（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_sw_messages_dispatch / response_body_readable_stream_r2967 / websocket_error_nonterminal / websocket_event_flow），零新增红。**计数口径注记**：总数 3833（`running 3833 tests`）vs s370 账本 3839 差 6——本刀 engine 无 .rs 改动（JS shim 不参与测试注册），HEAD 提交面实测 3833；s370 记数疑含当时工作树未提交的临时测试代码，差异不影响逐名同构结论。
- **钉测（最终树定向）**：webview `s37` 过滤 4 测全绿（prepare_document_state_clears_persistent_handle_nodes_s37 + s370 两收紧版 + s371 新钉）。
- **fmt**：`cargo fmt -p zero-webview -p zero-wpt-runner -- --check` PASS。
- **clippy**：`-p zero-engine -p zero-webview -p zero-wpt-runner --all-targets -- -D warnings` PASS（首轮 s371 收紧断言 bool_assert_comparison 2 处修后 PASS；systemd-run 24G scope 包裹）。

## 挂账（移交下一切片）

1. **残余 9F modal light-dismiss 分类面**勘根+修复：over-dismiss 4（modal 上下文点击应保持外层 open 被误关）+ under-dismiss 5（点击 dialog 应关内层 popover 未关）。勘根键面 = R5263 down/up 两腿 + top-layer 反走 containment/身份面 + light/shadow 双通道触发分类。
2. 既有在册维持：基建面 T=3（s102/s124）、probe8/9/10 转正决策、customizable-select 126F / dialog 41F 既有未建面（s366 sweep 口径）。
