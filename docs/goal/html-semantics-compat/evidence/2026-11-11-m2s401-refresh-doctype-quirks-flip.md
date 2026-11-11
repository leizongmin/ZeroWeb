# M2 切片 401 — 探测环 doctype 丢失 quirks 翻转修复（driving 1F→P）

日期：2026-11-11（承 s400 下一步：本流域散单 focus scroll-into-view 1F）
commit：见 git log s401 提交

## 勘根（八步探针链定谳）

driving 案 `radio-focused-and-scrolled-into-view.html`（3 radio + `#offscreen-radio`
`margin-top:200vh`，`firstRadio.focus()` + 两轮 Actions ArrowDown，断言
`offscreenRadio === activeElement` 且 `scrollingElement.scrollTop > 500`）solo 红形：
`scrollTop=0 expected a number greater than 500`——focus 面已过，scroll 断言红。

1. 探针①（直调 `ro.focus()` 同步读）：scroll 完美工作——`foc:seT=650,sy=650`
   （1237+13−600 精确最近对齐）——R5257 focus scroll-into-view 臂健康。
2. 探针②（键盘路径复刻）：at-focus `sy=0`；final `sy=650 seT=0`——scroll 发生
   （focus 事件后、spec 序正确）但 `scrollingElement.scrollTop` 发散。
3. final 读：`cm=BackCompat seDE=false seB=true seTag=BODY`——**页面落 Quirks**；
   scrollingElement→body，body.scrollTop 读 `_scrollOffsets` 恒 0。
4. 早读（首个 test()）：`cm=CSS1Compat seDE=true doctype=html`——文档起点标准态，
   翻转发生在中途。
5. 相位标 t0/t0b/t0c/t1/t2/t3：`t0=CSS1Compat,t1=CSS1Compat,t2=BackCompat`——
   首个 Actions 轮内翻转。
6. 受控对照：纯 `setTimeout` await 不翻转；`setAttribute('data-zz','1')` + await
   **翻转**——任意 DOM mutation + 探测环即翻转。
7. `dt=html kids=2` 而 `cm=BackCompat`——shim 侧 document 保留 doctype；DOM_SOURCE
   （管线活文档）已 quirks——真相反转在宿主重建源。

**机制**：runner 探测环（testharness.rs `take_probe`，mutation_version 门控）执行
`document.documentElement.outerHTML` dump——按规范（DOM §4.4 outerHTML）元素序列化
以元素为根，**不含 doctype**——喂 `webview.refresh_if_html_changed` → `cached_html`
被无声明串覆盖 → `render()` 全量 re-parse → 无 doctype 即 Quirks（HTML §13.2.6.4.1
the-initial-insertion-mode）→ DOM_SOURCE quirks → `compatMode=BackCompat` +
`scrollingElement` 分派 body → `scrollTop` 读 0。**波及面 = 所有 testharness 页首个
mutation 批之后的探测环刷新**——全树 quirks 污染（多数页面断言不依赖 quirks 面故
未显形；driving 案的 scroll 断言恰是 quirks 敏感面）。

## 施工（一 webview 文件 + Cargo.toml + 常驻钉）

- `crates/webview/src/webview.rs` `refresh_if_html_changed`：入差分与替换前调
  `restore_live_doctype` 归一——入参序列化缺 doctype 前缀且活文档有 doctype 子节点
  → 前缀补回（`doc.outer_html(doctype_child)` 序列化）；活文档无 doctype（真
  quirks 页）原样透传。补回后的串同入 `runner_probe_html` 差分基线与 `cached_html`
  （前后迭代比对形态稳定）；首个修复刷新后重建源恒带 doctype，活文档 doctype 节点
  不衰减——自持续。共享路径单点修（一个 choke point 优于 runner 两 dump 点各补）。
- `crates/webview/Cargo.toml`：加 `zero-dom` workspace 依赖（`zero_dom::NodeKind`
  inline 路径匹配 doctype 节点，pipeline/mod.rs 同款）；Cargo.lock 同步。
- `crates/webview/src/tests/basic.rs` 新增常驻钉 `test_runner_refresh_preserves_
  doctype_s401`：带声明页探测环刷新后 `compatMode` 恒 CSS1Compat + 真 quirks 页
  透传不凭空补声明（s371 强制分叉同款相位）。

## 过程陷阱

- **U+E015 PUA 字符不可见**：探针 `keyDown("")` 字面含 U+E015（EE 80 95），显示为
  空串；Edit 只改到注释、python heredoc 替换找不到「原串」AssertionError——
  `grep | cat -A` 核字节确认文件本就含真实 PUA 字节，无需修。
- **并发 clone 编译撞负载**：the-input-element solo 里
  focus-dynamic-type-change-on-blur 基线 80P→文件级 T——ZeroWeb-2 流
  `cargo test --workspace` + clippy 编译（`/proc` cwd 核归属）同时开跑撞负载——
  solo ×2 复跑全 80P 定谳负载 flake（s387 截断族判据）。
- **pre-compaction 探针残留**：solo 显形 `zz-s401-focus-scroll-probe.html` 文件级 T
  ——前段 429 中断会话 12:32 的探针 0（summary 只记了后两个探针名）——
  probe hygiene 清理后重跑干净；顺带销案 s400 挂账「radio 探针残留案归渲染流处置」。
- **join 键字段反转**：keyed 文件形态 `key\x1fstate`，join/hash 用 field 2 = state
  → join 出 8.2k×8.2k 叉积假挂起——join 键恒 field 1，或直接对 pairs
  （`state\x1fkey`）做 awk hash-join 用 $2。

## 验证（串行管线，/tmp/s401/ 全量日志）

- 探针复跑：marks 恒 `CSS1Compat`（修复前 t0c 起翻转 BackCompat）；final
  `seTag=HTML sy=650 seT=650`（修复前 `seTag=BODY seT=0`）。
- driving solo：`radio-focused-and-scrolled-into-view` **1F→P**（sweep1/sweep2 双跑
  + solo 共三跑全绿）。
- the-input-element 域 solo（1848 行）：checkable-active 2F / date-readonly 1F /
  maxlength-number 1T / fdtcb 80P 全部与 s400 基线同态零位移（fdtcb 掠过的 1T 为
  并发负载 flake，×2 复跑 80P 定谳）。
- 全树 sweep 双跑 join（基线 = s400 cur2-s400.pairs unique 8473，LC_ALL=C + grep -a
  + awk hash-join）：
  - **Fail→Pass ×1 = driving 恰中**（dialog-overlay 单列见下）；
  - stable 双跑位同：sweep1 = sweep2 除 2 键全一致；
  - **Pass→Fail ×0**；home-end 双键 T↔F 双跑不稳（s396/s400 在册时序 flake 家族、
    真态 F 单列不变）；
  - dialog-overlay `dialog computed overlay initially 'none'` 基线 F → 本轮双跑全绿
    （s400 在档渲染流负载 flake 键；归属渲染流复核，非本流修复面）；
  - new ×2：driving 真子测 Pass（修复）+ srcset `\r` 命名子测行（s400 sweep 截断族
    漏记，solo ×1 + 双跑恒 P 补记，非行为位移）；
  - gone ×1：driving 文件 `probe diagnostics` 探针行（前段会话已还原文件 + 本轮删
    探针 0，键出集合——出账非位移）。
  - sweep raw 口径：cur unique 8474 = 8225P/216F/33T（基线 8222P/218F/33T）。
- 单元：`cargo test -p zero-webview -j 2`（test-guard）**849P/2F**（= s400 848P +
  本切片新钉 1P；2F 在册逐名同形：image_decoder proxy_mode + csp_gate
  connect_fetch）。engine/script-sandbox 本轮零改动未重跑（s400 口径不变）。
- 产品 smoke：`make product-smoke` PASS；`make product-smoke-legacy` PASS。
- fmt/clippy：`cargo fmt --check -p zero-webview` exit 0；`cargo clippy
  -p zero-webview --all-targets`（test-guard）Finished clean。

## 在册红口径

- 全域 sweep1 raw：unique 8474 = 8225P/216F/33T（基线 unique 8473 = 8222P/218F/33T；
  s400 headline「217F」为 raw 218F 剔探针行单列后的记法）。
- **本流域口径 218F+31T → 216F+31T**（s400 口径构成 = stable 216 + dialog 1 + 探针行
  1）：−1 driving（本流修复双跑全绿）−1 radio 探针残留行出账（s400 挂账
  「归渲染流处置」销案）−1 dialog-overlay（渲染流 flake 键本轮双跑全绿——按事实出
  红账，归属渲染流复核，若复翻红由渲染流入账）；T 口径 31 = raw 33 − home-end 双键
  单列（真态 F 不改判，s400 同规）。

## 残差与边界（钉死入账）

- 脚本运行中移除 doctype 节点（`document.doctype.remove()`）不翻转 compatMode 是
  spec 行为，但活文档 doctype 子节点移除后探测环重建源会再丢声明——compatMode 为
  parse 派生（非存储字段）的架构面，html/semantics 语料无此案，FIXME 单列待勘根。
- `restore_live_doctype` 只补 `<!DOCTYPE` 前缀，不复原 public/system id 之外的历史
  声明形态——活文档 doctype 子节点序列化是权威源，语料面无 delta。
- 探测环刷新本身的全量 re-parse 成本未动（s168c 语义维持）；本轮只修声明保真。

## 下一步

- 在册红 216F+31T（本流域口径）。下一候选：本流域散单（link-rel shadow DOM CSSOM，
  referent 待先核 s168b 是否已覆盖）或 object-handler 嵌套文档脚本通道（R206 object
  移植）；簇 D（customizable select 特性族）仍待用户决策。
- 挂账池：`__zw_resolve_handle_selector` forward map 空返低优先勘根；渲染流 5 面→
  rendering-compat；form-submission-target 6T→navigation M3；sizes-auto lazy 半边待
  勘根；A2+D 簇立项征询待用户决策。
