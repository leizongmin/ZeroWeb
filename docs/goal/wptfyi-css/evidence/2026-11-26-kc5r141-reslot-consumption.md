# kc5r141 · css-shadow「slotted 失效消费面」刀收账证据

日期：2026-11-26 · 流：wptfyi-css Rally 跨轮次换岗轮 · 刀：k141 gCS 代际缓存 slotted 失效消费面

## 根因与实现

- 根因（k140 收账挂账的「级联/失效消费面」残 F）：gCS 代际缓存 warm 路径在 slot 分派变化后的消费缺口，两个具体臂——
  1. suffix 含 handle attr/style 变异（k140 岛形刀后页面经岛 handle 的属性写，如 `removeAttribute("name")` 触发 re-slotting）：旧 `is_tree_mutation` 排除面含 handle 四类（「handle 元素不在快照 doc，重放不解析」契约），suffix 走 `apply_inline_style_overrides`（无 handles 表可解析）→ 变异静默丢弃 → cached doc 分派表过期。
  2. 选择器域 attr suffix（如宿主 light 子 `slot` 属性写改分派目标）：`apply_inline_style_overrides` 经 R34xx 泛化通道落 attr，但 slot 分配表不重解析 → 分派过期。
- 实现两面（`js_dom_bridge/computed_style_cache.rs`）：
  1. `is_tree_mutation` 收编 `SetAttrOnHandle`/`RemoveAttrOnHandle`/`SetStyleOnHandle`/`RemoveStyleOnHandle` 四类 → 含它们的 suffix 走权威全量重放（`compute_document_styles_with_inline_overrides_css` → `apply_dom_mutations_full` 批末自带 `resolve_slots_all_hosts`，handles 链批内自洽重建）。
  2. 选择器域 suffix 臂 `apply_inline_style_overrides` 后补 `s.3.resolve_slots_all_hosts()`（与权威臂批末同口径；无 shadow 页 `shadow_roots` 空 O(1) 快返零开销；spec：https://dom.spec.whatwg.org/#assigning-slots-and-slots ）。
  3. `try_sync_with_drain_record` 注释勘误：drain 批 replay 经 `apply_dom_mutations` → `apply_dom_mutations_full`，批末已含 `resolve_slots_all_hosts`——本路径无分配表过期缺口（勘误前期注释）。
- 定性修正：k140 挂账将 slotted-invalidation 定性为「`.outer ::slotted(#slotted)` 祖先类级联匹配面」——k141 实证其残红同为消费面（className 写走 `SetAttrOnHandle`，收编后权威重放全量重算，匹配引擎本就命中），非匹配面缺口。

## 钉测（computed_style_cache.rs `kc5r141_slotted_reslot_tests`，2 全绿，红先证）

- pin ① `k141_suffix_advance_onhandle_attr_routes_authoritative_replay`：同代 advance suffix 含 `RemoveAttrOnHandle`（slot1 name 移除）→ 权威全量重放 → span（无 slot attr）回落首 unnamed slot 绿；修复前 suffix 重放丢弃变异恒红（实证 0P/2F）。
- pin ② `k141_suffix_advance_selector_slot_attr_reslots`：选择器域 `SetAttr(slot=b→a)` suffix → 落 attr 后补 `resolve_slots_all_hosts` → span 分派 b→a 红→绿；修复前分配表过期恒红。
- 红先证协议：两行为 hunk 临时回退（保留钉测）跑 0 passed/2 failed（各 pin 末断言 stale `rgb(255, 0, 0)` vs 期望绿），复原后 2 passed/0 failed。

## solo 预验（残 F 三页，ONLY_SUBSTR 定向，bin md5 2bdf41d4）

| 页 | b1 结果 | 定性 |
|---|---|---|
| shadow-reassign-dynamic-003 | 2P/0F | **翻绿（F2P 兑现）**——k140 diag 定谳的消费面缺口收口 |
| slotted-invalidation | 1P/0F | **翻绿（F2P 兑现）**——定性修正实证（消费面非匹配面） |
| scope-pseudo-in-shadow | 1P/1F | 持红＝`:scope` in shadow root 匹配面（真匹配面缺口，归 k142 候选） |

## 电池双腿（协议：flock /tmp/zw-k141-battery.lock + test-guard 逐腿 + 唯一 JSON_DIR/transcript）

- b0＝k140 归档 bin（.acceptance/wptfyi-css/k140/bins/r140-island-bind-runner，md5 `cdcb897fed9e8430305506fc5e4eab6d`）；b1＝本刀 bin（.acceptance/wptfyi-css/k141/bins/r141-reslot-runner，md5 `2bdf41d4ebdc0a978d28281c7a592d4d`）。
- **2 腿 RC=0（LEG-EXIT-b0 rc=0 13:35:31／LEG-EXIT-b1 rc=0 13:36:06）**；90 页全名册：b0 294P/67F/1T（k140 b1 同 bin 确定性复现逐位相等）→ b1 **296P/65F/1T，净 +2**。
- 键级差分（`/tmp/zw-k141/diff-k141.py`，(page, test-name) 键）：**P2F=0（红线成立）NEW=0 GONE=0，F2P=2**——两键全数消费面：
  1. `shadow-reassign-dynamic-003.html`「Computed color after re-slotting.」（removeAttribute→re-slotting 消费）
  2. `slotted-invalidation.html`「CSS Test: Style invalidation for ::slotted()」（className 写经岛 handle 消费）
- 1 Timeout `part/invalidation-part-pseudo.html` 两腿同位恒持（::part 盒布局域，非本刀域）。

## 门禁

- engine --lib 全量（test-guard 包裹）：**4355P / 3F**——`test_dialog_focus_restore_declarative_shadow_s330`／`test_expando_enumeration_r3046`／`test_p99s_pure_path_inline_custom_property`，与 k140 收账红名单（s330/r3046/p99s）逐名恒对账，零新红；总 fn 4356→4358（本刀 +2 pins）。
- clippy：zero-engine `--all-targets -D warnings` RC=0。

## 残 F 挂账（k142+ 候选）

- `:scope` in shadow root 选择器匹配面（scope-pseudo-in-shadow，1P/1F 持红——真匹配面缺口）。
- 原有：exportparts 跨树转发、::part 盒布局 7 键＋invalidation-part-pseudo Timeout 恒持（渲染流域）。
