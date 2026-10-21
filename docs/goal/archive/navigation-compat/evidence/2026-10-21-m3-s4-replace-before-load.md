# M3-S4 replace-before-load 最小面（2026-10-21）

M3 第四切片：html/browsers/browsing-the-web/navigating-across-documents/
replace-before-load 38 案簇（iframe 载入前自导航**置换**当前会话条目 +
父窗 back 哨兵回跳）。四处变更（runner 1 + shim 3），零既有测试改写。

## 根因分解

对簇内案面逐案追踪（code-injector.html + resources/helpers.js），失败归因
三个可修缺口：

1. **fetch 通道 `.html` 不做 wptserve `{{GET[name]}}` 查询模板替换**——
   code-injector 正文即 `{{GET[code]}}`，子文档脚本文本原样注入 →
   SyntaxError，自导航从未发生（簇内多数 Timeout 的共同根因）。
2. **子脚本裸标识符 `history` 误解析父窗 history**——注入代码
   `history.pushState(...)` 落父窗账本、子窗 URL 不变。
3. **子局部会话历史无「置换前载入」语义 + 父窗 back 无 frame 回退**——
   自导航 push 而非 replace；哨兵案 `checkSentinelIframe` 的 ONE
   history.back() 落空（父窗账本无更早条目即静默 no-op）。

## 实现

- **(a) runner `{{GET}}` 替换**（testharness.rs）：fetch 通道静态 `.html`
  响应体按该请求查询串替换 `{{GET[name]}}`（percent-decode；查询未提供的
  残留占位替换空串——wptserve 语义）。helper
  `wpt_apply_get_query_substitutions`，紧邻 `wpt_query_value`。
- **(c) 子脚本 `history` 形参**（part01 child script Function formals 尾部）：
  `history` → `win.history`（part05 子窗 stub，pushState/go/back 面）。
- **(G2) 载入前 replace**（part01 `__zw_reload_iframe` history 记账）：
  `entry.loadedOnce` 随 entry 链承继（两处 load 派发 defer 翻转 true）；
  子发起（`__zwChildNav` 标记，见下）且前一文档 load 未派 → **原地替换**
  `history[len-1]`（不 push）；其余维持 push。
  **R388 收窄教训**：首版条件只看 `!previous.loadedOnce`，全量 lib 把
  r388_iframe_history_navigation_preserves_session_history 打红（父层 src
  属性更替契约面——载入派前更替仍 push、`history.go(-1)` 需两段条目）。
  修正：子发起标记仅加在子导航两个通道（part05 `iframeReload` +
  location.href setter `{ viaNavigation }`）；父层属性路径不带标记维持 push。
- **(G3-lite) 父窗 back 的 frame 局部回退**（part02
  `_hist_applyTopTraversal` 越界分支）：顶层账本无可达条目时先查
  `_zwJointFrameBackTarget()`——`_iframeDocCache` 中局部历史 >1 条的 frame
  取 `_zwNavSeq` 最大（最近活动），delta −1 落其 `history[len-2]` 以
  `{history:true}` 重载（frame 局部 pop + 元素 load 派发）；ctrl 在场结算
  镜像同目标分支（resolve + 延迟 finishedSettle）。atEnd（执行时目标被剪除）
  的 AbortError 面优先维持（M2-S4P3 断言面）。spec 依据：
  [joint session history](https://html.spec.whatwg.org/multipage/nav-history-apis.html#joint-session-history)
  遍历最小近似 + [#navigate](https://html.spec.whatwg.org/multipage/nav-history-apis.html#navigate)
  置换前载入。

## FIXME 记录（最小面边界，簇内剩余 Fail/Timeout 的归因）

- `FIXME(joint-frame-forward)`：entry.history pop 模型无前向槽——父窗
  forward 的 frame 回退不支持（仅 delta −1）。
- `FIXME(joint-frame-back)`：多 frame 在场按最近活动近似，不做联合序并序。
- `FIXME(history.length)`（S3 既立）：joint/子条目不进父窗 history.length
  报数——history-pushstate{,-during-load} 断言「length 2」仍红（真实浏览器
  为联合长度）。
- 子窗 `location.search` 无 setter（location-setter.html search/hash 断言
  面——search setter 未实现，href 面已 Pass）。
- 子文档 form.submit() 落父窗 `_pushHistNav` 不驱动 frame 重导航
  （form-submit.html）。
- `*-user-click/-user-mouseup` 簇：testdriver 合成用户激活后断言面
  （location-setter-user-click 3 子测 Fail——激活时序+search setter 复合）。
- `*-during-pageshow` 簇 Timeout：pageshow 面未达。

## 验证

- 集成测试：`js_dom_bridge_tests/part63.rs` 新增
  m3s4_child_during_load_self_nav_single_load（载入中自导航恰 1 次 load +
  子局部历史折叠 1 条 + 父窗长度不膨胀）与
  m3s4_parent_back_falls_back_to_frame_local_history（两段属性载入 → 父
  back 落子局部前一条目 + load 派发 + pop 收缩）2/2 过；全量
  `cargo test -p zero-engine --lib` **3357/3357**（含 R388 契约面回归
  驱动的收窄复验）。
- 域内：`make testharness-navigation FILTER=replace-before-load`
  对 (a)-only 中间账本案级迁移：**4 案 Timeout→Pass**（location-assign /
  location-assign-during-load / location-setter href+search 分裂为 Pass+Fail /
  …），history-pushstate-during-load Timeout→Fail（断言面显形=推进），
  零既有 Pass 回退。明细见原始输出。
- 全量 corpus：`make testharness-navigation` **464P/69F/48T**（581 案行）对
  S3 全量（460P/69F/51T）逐路径 diff：**全部变化路径均落在
  replace-before-load 簇内**——4 案 Timeout→Pass（location-assign /
  location-assign-during-load / location-setter / location-setter-during-load），
  form-submit-cross-frame、history-pushstate-during-load、iframe-src 三案
  Timeout→Fail（断言面显形=推进，归因见 FIXME 节）；**簇外 0 路径变化**
  （含 navigation-api 5 案既有 Fail 面与 M2/S3 全部断言面）。
- fmt 干净；clippy -p zero-engine -p zero-wpt-runner --all-targets 零
  warning；`make test`（test-guard，DISPLAY=:20）**76 suites
  22,258P/0F/199I exit=0**（shim/runner 变更触发全量门）。
- reftest 未跑：变更面（会话历史记账 + 脚本文本替换）零布局/绘制触达
  （S1/S2/S3 同理由）。

原始输出：[2026-10-21-m3-s4-replace-before-load-family.txt](2026-10-21-m3-s4-replace-before-load-family.txt)
（域内）、[2026-10-21-m3-s4-replace-before-load-full.txt](2026-10-21-m3-s4-replace-before-load-full.txt)
（全量）。
