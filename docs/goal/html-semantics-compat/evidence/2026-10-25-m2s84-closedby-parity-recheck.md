# M2-s84 分诊轮：dialog closedby 家族「Chrome-parity 同红」假设重估（零代码）

日期：2026-10-25 ｜ 轮次：s84（分诊，s53 先例）｜ 前序：[s83](2026-10-25-m2s83-handle-matches-complex.md)

## 1. 假设重估：同红已过时

A 族 closedby×slot 16F 的挂账依据是 crbug 417571176（2026-04）「Chrome-parity 同红」。本轮取 wpt.fyi 同 API 最新 chrome-158.0.8122.0-linux（revision ffe56d4420d2）summary_v2 实证：

- `dialog-popover-closedby-complex.html` **40/40**、`dialog-popover-closedby-simple.html` **6/6**、`dialog-closedby.html` 27/27、`bounds-clicking` 14/14、`corner-cases` 30/30、`show-stacked` 4/4、`start-open` 2/2——**closedby 家族 chrome 全绿**。s53 已证 wpt.fyi summary 与同 build 直探一致口径可用。
- 结论：同红假设过时（closedby 已 ship，popover light-dismiss interop 修齐），**23F 转可施工**（complex 16 + simple 2 + bounds-clicking 2 + corner-cases 1）。

## 2. 本地红图（probe 实测，FILTER=dialog-closedby 家族 + pin）

closedby 家族本地 7 文件：`dialog-closedby.html` 27P/0F、`show-stacked` 4P/0F、`start-open` 2P/0F（三项与 chrome 持平）；`corner-cases` 29P/1F、`bounds-clicking` 12P/2F、`popover-closedby-simple` 4P/2F、`popover-closedby-complex` 16F（chrome 40/40）。

## 3. 根因隔离（三步探针，临时已删）

失败面首腿全部是「点击 popover 内元素应保持全开」——light-dismiss 逻辑（R5234 两遍/R5263 两腿）被误触发，而非其自身缺陷。探针链：

1. **插桩 ld 调用序**（包 `__zw_dialog_light_dismiss_check`）：首测（any 变体）点击 popover 的 light-dismiss 落点 `px=0,py=0`、target 经 s30 backdrop retarget 成 `#dialog_outer` → popover pass 判外 → 误隐。同页后续变体（closerequest/none）同断言同坐标命中 `#popover_inner` 正常——差异只在**首建布局的 rect 采样时点**。
2. **gBCR 时序探针**：showModal+showPopover 后逐拍采样——sync / 2 rAF / 4 rAF 三拍 rect **恒零**，+100ms setTimeout 后才有值。`clickOn`（corpus popover-utils，不可改）在 2 rAF 采样 → 首点击 (0,0)。
3. **序列化同步探针**：同一时刻 `outerHTML` 长度 +23（`closedby="any"` + `open`）与 rect 转有效**同拍**出现——shim mutation 批的宿主 materialize（apply → render → rect 快照刷新）在 rAF 驱动节拍下整链滞后，~5+ 探测轮（100ms 定时器路径）才落。

## 4. 定位边界与下一步

黑盒探针到顶：rAF tick（testharness.rs:8537 `__zw_raf_tick`）与 take_probe flush/version-gate refresh（:9259 起，flush → timers → modal 注入 → `refresh_if_html_changed`）的链路上，该批 mutation 为何未被 2-4 轮内的 version 门/flush 捕获，需 Rust 侧定证（契约测试或 tracing 直插 take_probe/flush/refresh 三点）。候选面：mutation_version 门时序、`refresh_if_html_changed` 的 `runner_probe_html` 基线、或 timer 泵与 rAF 拍的批次归属。

修复方向预判：让 showModal/showPopover 后的首个 rAF 拍即完成 materialize（rect 快照同步可见）——`abspos-dialog-layout` 5F（dialog UA position:fixed 面，本轮顺带观测 dR y=8 即 in-flow，`:modal` UA 定位未上）与 closedby 家族共桥。

## 5. 范围与记账

- 零代码轮：不改 engine/webview/runner 生产面（s53 先例）。
- 23F 记「可施工——根因已隔离到 gBCR materialize 滞后」，下轮 Rust 侧定证 + 修复。
- corpus 不可改（popover-utils.js 的 2-rAF 采样是上游语料行为，Chrome 下同步 rect 语义使其天然正确——修复必须落在我方 rect 新鲜度）。
