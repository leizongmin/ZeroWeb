# M2-S196：`:active` 伪类通道——按压态槽 + 查询面注入 + 指针/键盘钩子

**日期**：2026-11-01
**入场**：s195 在册首席候选 checkable-active 族 8F（checkable-active-onblur 2F +
checkable-active-onblur-with-click 6F）——s195 ledger 定谳根因「`:active` 伪类通道
整体缺位」。
**结果**：定向 8F——**稳定翻绿 3**（with-click：radio 本体 + checkbox/radio
后随 label，收口后 6 轮全 Pass）；**flake 域翻绿 3**（onblur 键盘 ×2 对照窗
双 Pass + checkbox 本体指针全树跑 Pass——runner 单命令链竞速负载敏感）；
**恒 F 2**（父 label ×2 = 含控件 label 零矩形 layout 缺口，挂账不扩刀）。

## 勘根

缺口四根：

1. **无按压态通道**：`matches_pseudo_class` 对 `PseudoClass::Active` 落
   `_ => false`；`Document` 无 active 元素态；无 pointerdown/up / 键盘 Space
   钩子。上游语义（HTML §interaction `#selector-active`）：元素被用户按住
   （指针按下未松、Space/Enter 键盘激活进行中）即匹配 `:active`。
2. **查询面 Document 皆临时构建**：engine shim 的查询走查询期文档——
   `with_query_view_doc` 三臂（live `LIVE_QUERY_DOC` / O(1) 视图缓存命中 /
   rebuild+链增量）与字符串路径逐次现 parse（`element_matches_test_selector` /
   `closest_matching_selector`）。对某一次构建的 Document 直写字段不能跨查询
   存活——按压态必须**槽 + 注入**：宿主持字符串槽，查询构建点统一解析。
   渲染样式面同型先例 = `inject_focus_state`（:focus 同架构，CSS 渲染面
   per-frame 注入，本刀不扩，挂账）。
3. **宿主回调早绑定 shim 作用域**：页面 JS 包装
   `globalThis.__zw_set_doc_active` 只截获页面调用，shim 内部直呼原始回调
   ——页侧捕获/伪造写不进可信链（probe3/4 早期 cap=[] 结论部分作废的原因）。
   写通道只挂 shim 可信 UA 面（指针 pre-dispatch、键盘 trusted 派发链、
   焦点 commit/blur 臂）。
4. **PUA 字面量运输坑**：Space/Tab 在 runner 键通道用 PUA（U+E00D / U+E004）
   传输——字面量在日志与 Edit 工具中不可见（Edit 落盘会剥成空串），必须
   python 字节级写入 + repr/hexdump 验证。s196 中途 Space 键面一度静默回归
   即此坑（`key !== ''` 早退），已修并钉规程。

## 施工（9 文件）

**zero-dom（4 文件）**：

- `document/mod.rs`：`active_element: Option<NodeId>` 槽 +
  `set_active_element`；`element_matches_selector` 补 `Active` 臂（查询期
  `is_active_element` 再评——两阶段选择器，`query_selector` 路径）。
- `query.rs`：`matches_full` 对 `:active` 返 deferred true（静态面不误配，
  匹配集交给文档态再评）。
- `document/focus.rs`：focusing/blur steps 联动清 active（焦点迁移清钩）。
- `document/snapshot.rs`：pristine 快照携带 active_element（视图链不断态）。

**engine（5 文件）**：

- `js_dom_bridge.rs`：`DOC_ACTIVE_SELECTOR: Mutex<String>` 字符串槽（""=无）
  + `sync_active_state(doc)`——槽 selector 经 `find_by_selector` 解析回
  NodeId 后 `set_active_element`；解析失败 = 目标已离开文档视图，按无按压态
  处理（幂等无条件覆盖，重复注入无害）。
- `js_dom_bridge/callbacks.rs`：`__zw_set_doc_active` 回调改纯槽写（删旧
  桩的 dom_html 误锁）；`with_query_view_doc` 三臂注入——live 臂
  `try_borrow_mut` 同步后释放再交查询闭包（借用冲突回落视图路径，与原
  `try_borrow().ok()` miss 同型）；O(1) 缓存命中臂先注入再执行（缓存条目
  跨查询复用而按压态可无 DOM 突变翻转）；rebuild 臂落缓存与执行查询前注入。
- `js_dom_bridge/selector_match.rs`：字符串路径 `element_matches_test_selector`
  （`__zw_matches`）与 `closest_matching_selector`（`__zw_closest`）现 parse
  后注入。DOMParser 面（`__zw_parse_html_query` 等）**有意不注入**——惰性
  文档无按压态语义。
- `js_dom_shim/part06.js`：指针面 `__zw_pointer_down_sequence` PRE-dispatch
  标记（retarget 后命中元素选择器解析：`__zwSelector` → `_proxyCache` 反查
  → id 兜底（非 qName 字符转义）→ 未路由时 `tgt === el` 用原 sel 兜底——
  proxy 不必然带 `__zwSelector`）；pointerup 清；`__zwKeyActiveGate` 键面
  ——**keyup 无条件终结前置**（任何可信键 up 结束按压序，Tab onblur 语料
  钉面），Space keydown POST-dispatch 标记（prevented 不标），keyup 清。
- `js_dom_shim/part04.js`：焦点 commit 臂（`_activeElKey` 落定后）与 blur
  双臂清按压态槽（焦点迁移清钩第二通道，与 focus.rs 宿主侧清钩互补）。

## 验证

- shim 全序拼接 `node --check` OK（part04 为语法片段，须拼全部 6 part 检查）。
- **stash 隔离基线 vs WIP 对照（同机同 runner，s195 同法）**：
  - forms 全树：基线 **4443P/258F/11T**（复现 s195 数字逐位一致——语料
    零漂移归因干净）→ WIP **4448P/253F/11T**（ΔP=+5/ΔF=−5 精确闭合）。
    逐测试键 diff（(key,state) 对 sort -u 去重）**恰 5 键 F→P、零 P2F、
    零键增减**：checkable-active-onblur 键盘 2 键 + checkable-active-
    onblur-with-click 3 键（radio 本体 + checkbox/radio 后随 label）。
  - 基线其余 3 键未翻绿均定谳既有缺口精确挂账：checkbox 本体指针键
    （pointerDown→assert 单命令链 probe-tick 竞速 flake——全树跑同键 Pass
    实证）；父 label 2 键（含控件 label getBoundingClientRect 全零矩形 →
    Actions 原点数学误路由 BODY——layout 面缺口，与 `:active` 通道无关，
    3 轮复跑 3/3 恒 F）。
  - interactive-elements：基线与 WIP 均 **439P/44F bit-exact**（零键位移；
    inert-svg-hittest 第 2 子测预期 F 保持，挂账）。
  - lib：zero-engine **3642P/1F**（在册 s166 失败
    `test_ua_display_li_list_item_canvas_inline_s24_flip` 同键）+ zero-dom
    **880P/0F**，stash 基线逐位一致。
  - 全域参考跑（html-semantics 全域，非对账口径）7414P/613F/91T——s196
    靶族仅父 label 2F 在败列，button/onblur/指针面全 Pass。
- 探针语料（probe 页，验后删）逐通道钉面：宿主写通道（A/B/nth-of-type）、
  指针直写 + matches 面、pointerup 清、Tab 清、按钮键盘面全 Pass；日志存
  `evidence/logs/s196-*.log`。
- fmt 过（`__zw_set_doc_active` 注册处 rustfmt 折叠一处，语义等价）+
  guarded-clippy workspace `-D warnings` 过。
- **fmt/clippy 后定向复跑（REVERIFY 段，诚实记账）**：with-click 3 面
  稳定（following-label 双面 6/6 轮全 Pass）；**onblur 键盘 2 面回落 flake
  域**——对照窗（全树跑 + forms WIP 跑）双 Pass，收口后 6 连跑 F（并行流
  bench/workspace 测试持续负载期，load 5-7）；`cargo fmt` 双向对照
  （fmt 后 F×5 → 手工还原 pre-fmt 字节复跑仍 F）证明与格式化无关，负载
  敏感的 runner 单命令链竞速为在册根因。父 label 2 面全程 3/3→9/9 恒 F
  （layout 缺口钉死）。

## 残留/挂账

- **父 label 零矩形**（2F 硬 F）：含表单控件的 label 布局矩形
  (0,0,0,0) → Actions origin 数学误路由 BODY——layout 面缺口，非本刀。
- **runner 单命令链 probe-tick 竞速**（flake 域含 onblur 键盘 2 面 +
  checkbox 本体指针面，负载敏感 0-100% 摆动）：单命令链 keyDown→assert /
  pointerDown→assert 面 probe 轮询 tick 时序竞速；多命令链稳定。runner 面
  独立勘根（负载与 tick 对齐双变量待分型）。
- **CSS 渲染面 `:active`**：渲染样式 per-frame 注入未做（`inject_focus_state`
  同型挂账）——当前只在查询/匹配面生效。
- **`:not(:active)`**：两阶段选择器否定面同限（deferred true 否定语义）。
- **inert-svg-hittest 第 2 子测**：SVG hit-test retarget + matches 面独立勘根。
- **at-active wrapper activeElement 不稳**：s195 在册 seam（gate 正确拒非
  input 靶）。
- **Space 激活行为**：checkedness 翻转（activation behavior）未实现——本刀
  只做按压态标记，不做激活。

## 下一步候选

disabled-003 crash/hang 2F 独立勘根 → customizable select 177F 挂起待拍板
（GB-20261031 征询在途）。

## 日志

`s196-forms-stash-baseline.log` / `s196-forms-wip.log` /
`s196-interactive-stash-baseline.log` / `s196-interactive-wip.log` /
`s196-lib-stash-baseline.log` / `s196-lib-wip.log` / `s196-clippy.log` /
`s196-forms-full-tree.log`（全域参考）+ 探针族 `s196-probe*.log` /
`s196-checkable-active-*.log` / `s196-onblur-round3.log` /
`s196-button-active.log` / `s196-inert-svg.log`（同目录）。
