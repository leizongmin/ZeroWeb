# M2 切片 395（s395）——动态 append rect 供给链勘根施工（2F→P：handle-only geometry 身份派生 + definite 本地兜底）

日期：2026-11-11（承 s394 下一步：动态 append rect 供给链勘根——live doc 发布/mutation 同步链）
域：html/semantics/embedded-content/the-img-element（driving）+ geometry trap 共享面
性质：**勘根+施工轮——shim 单侧（part04/part05），雷区（s284/naturalWidth）边界先钉后动刀**

## 勘根（五探针定谳链，zw394-probe-img v2-v5 用后即删）

**driving 面**：`not-broken-while-load-task-scheduled.html` 2F（img width/height=50 + 未加载 src，append 后同 turn 断言 clientWidth=50）。s394 已定谳「动态 append rect 供给链缺席」；本切片钉死链上每个环节：

- **架构事实**（webview.rs:2857 注在案）：runner 的 shim 把结构变更拆 **shim 树（同步）+ 宿主 wire（异步 drain）**两层——JS appendChild 只推 `DomMutation` 队列（callbacks.rs:3490），flush 在 execute_script 尾部（webview.rs:3030）/探测轮首（2865）等 drain 点；apply 尾刷新 rect snapshot（6190）。
- **P6 探针（决定性）**：动态 div 带 id，`__zw_getBoundingClientRect('#p5dyn')` 同 turn 返空（P2，mutation 未 flush），**1.5s 后返 `8,58,70,20` 正确几何**——flush 后 host 对动态节点全可见且快照正确。同刻 shim 侧 `clientWidth=0`：fresh querySelector 仍返原 proxy（handle 去重，sameProxy=true），身份恒 handle-only（`__n0`）。
- **缺口单点**：in-process rect 回调（callbacks.rs:733）**无 handle→selector 解析臂**（多进程 rect_bridge.rs 版有，:157 注；in-process 版刻意收窄——s284 勘根），裸 handle 非合法 selector 恒 miss；`__zw_resolve_handle_selector('__n0')` 亦空（forward map 无此 handle）。selector→NodeId→snapshot 键一致性靠 fresh-parse 确定性契约（rect_bridge.rs:143）。
- **雷区实证**：img IDL width/height rendered 臂（part04:945，s283）与 geometry trap 同源消费 `_layoutRect`——兜底若进 `_layoutRect` 直接踩 s284 attr 期望面（naturalWidth 族 30 行回归史）。img 机具消费 rect 走 host 回调直连（resolver + `__zw_getBoundingClientRect`），不经 trap——trap 级兜底不触机具面。

## 施工（shim 单侧三 helper + 五 trap 消费面）

- **part05 三 helper**（`_layoutRect` 邻位）：
  - `_zwRectIdent(sel, handle)`：handle-only 且元素带 id → `'#'+id`（flush 后 host find_by_selector 可解析，P6 实证）；无 id 维持 handle（现状恒 0 不变）。
  - `_zwLocalDefiniteRect(sel, handle)`：仅 handle-only；双轴均 definite（inline style px / 维度属性无单位数——attr 即 presentational hint 面，probe v2 A 臂 computed 50px 实证）→ 本地 0 基盒（R34xx 文本几何兜底同型；CSSOM View geometry 读 update-the-rendering 强制布局语义的 flush 前近似）；缺轴/百分比/auto → null。
  - `_zwLayoutRectTrap(sel, handle)`：身份派生 + host 查询 + 本地兜底组合。**`_layoutRect` 本体保持裸语义零变化**（img IDL 臂等既有消费者零风险）。
- **part04 五消费面**：gBCR（13451）、getClientRects（13473）走 ident+兜底内联；offsetWidth/Height、offsetTop/Left、scrollWidth/Height、offsetParent（`_zwLayoutRectTrap` 六处换接）。img IDL rendered 臂 945 **不动**。

## 验证

- solo driving：**2F→P**（`run_test("src")`/`run_test("srcset")` 全翻转，8 断言双相全绿）——solo-s395-driving.log 退出码 0。
- **金丝雀 join**（s284 雷区仲裁）：`FILTER=naturalWidth` solo 258P/0F，对 s394 基线 263 键 join——5 差集全为 solo 路径过滤 scope 伪位移（svg-img-density-vs-zoom/natural-size-orientation 文件路径不含 naturalWidth 子串未入 solo），匹配域 258/258 精确一致零真位移。
- 全树 sweep join（基线 = s394 sweep unique 8463）：实测 **8466 行 = 8205P/224F/37T，unique 8463**；**gone 2F = new 2P 恰为 driving 两键，零 P→F、零 Timeout 位移、零意外键**。红键 226F→224F。
- 单元测试：test-guard 包裹 `cargo test -p zero-engine -j 2` **3834P/7F**——7F 与 s391/s393/s394 在册红集完全同形（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_service_worker_messages_dispatch / response_body_readable_stream_r2967 / t8k_websocket ×2），零新增红。
- 产品 smoke：`make product-smoke` PASS（struct-check 0 issues）；`make product-smoke-legacy` PASS（struct-check failures: 0）。
- shim 拼装 25 part 全序 node --check SYNTAX-OK；探针 hygiene：zw394-probe-img.html v5 后删除（gitignore 本地数据，git ls-files 先核无跟踪态）。
- fmt/clippy：本流零 Rust 改动，不适用。

## 残差与边界（钉死入账）

- 无 id 的 handle-only 元素 flush 前仍 0（definite 双轴不全时）——`__zw_resolve_handle_selector` forward map 对新 append handle 的空返（P3/P4 实证）未勘根，属下一层（如需再开）；现缺口仅剩「无 id + 无 definite 尺寸」动态元素 geometry，corpus 驱动面已清。
- offsetParent 臂语义随兜底微移：handle-only definite 元素 offsetParent null→body proxy（真实浏览器语义——已 append 可见元素非 null）；全树 join 零位移实证无 corpus 面。

## 下一步

- 挂账池：`__zw_resolve_handle_selector` forward map 空返勘根（低优先——corpus 驱动面已清）；渲染流 5 面（basic 亚像素 / error alt-box / svg-zoom / lineheight / sizes-auto 几何半边）→ rendering-compat、form-submission-target 6T → navigation M3、sizes-auto lazy 半边待勘根；A2+D 簇立项征询待用户决策。
- 在册红 224F+37T 下一候选：回到 forms/树散单分诊或按域 solo 分诊推进（承 s392 分诊方法论）。
