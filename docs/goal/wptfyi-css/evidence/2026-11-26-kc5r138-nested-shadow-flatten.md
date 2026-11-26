# kc5r138 verdict——嵌套 zwsh 寻址＋`:has-slotted` 展平语义＋replace-all 保根（2026-11-26）

## 结论

- **电池净 +13，P2F=0**：b0（k137 bin）/b1（k138 bin）双腿 css-shadow 全量
  名册 90 页 361 键；F2P 13（query-selector 9＋changing-001 4＋
  host-part-003 1）、P2F 0、NEW/GONE 0。b0 275P/86F → b1 288P/73F。
- **换代硬证**：launch log 双 md5——b0
  `94582c536dc7d1c9e6623832e46d6b52`（k137 归档 r137-has-slotted-runner）、
  b1 `595998361ab8ed1e9c67b541b2e241cc`（r138-nested-shadow-runner）。
- **DOM slotting 字面语义定谳**（推演矛盾消解，另录 memory
  dom-slotting-spec-literal-trace）：find a slot 扫 shadow 的 **descendants**
  （非直接子代）；find flattened slottables＝「根为 shadow root 的 slot」
  **递归**展平＋空分配取 fallback。字面读法 8/8 自洽＝浏览器行为（上游
  chrome/firefox 全绿佐证）。此前 session 的「矛盾/skip 假说」源于错误
  记忆模型（parent 链递归），修正后逐字重抄算法文本推演全通。
- **fallback 计入定谳**：`:has-slotted` bare＝展平 slottables 非空
  （css-shadow-1 §3.2.5），递归**含 fallback**——has-slotted-flattened-001
  上游 chrome/firefox 绿＝slot_outer 分配空仅靠 fallback div 展平即须匹配；
  spec 第一句（链 DOM 算法）与第二句推断注记矛盾，以浏览器实证为准。

## 实现面（五处）

1. **嵌套 zwsh 寻址**（engine/shadow_address.rs）：`shadow_scoped_selector`
   host 段 fallback→`shadow_node_address`（自 node 上溯遇 ShadowRoot 取
   host 递归，逐层上升天然有界）；`try_shadow_scoped_fastpath` 改带
   depth（>32 → `Some(None)` 保守拒）递归自解；`shadow_content_entry_selector`
   删「嵌套 shadow 暂不支持」守卫。修嵌套深条目 selector 全省略＝shim
   EnsureContent 空转（query-selector 嵌套链页此前全红的寻址根因）。
2. **`:has-slotted` 展平语义**（dom/document/shadow.rs＋mod.rs）：新增
   `get_flattened_slottables`（spec find flattened slottables 四步：
   非 shadow 树 slot 空表；分配序列非空取分配；空取 fallback slottable
   子；「根为 shadow root 的 slot」递归展开；depth 预算 32）。求值臂
   k137 直连 `get_assigned_nodes` → k138 改展平表非空；单源收敛
   `is_has_slotted_element`。k137 直连对「分配了嵌套 slot 而链尾为空」
   恒非空＝空态误匹配（query-selector test1/3 形）。
3. **replace-all 保根**（engine/js_dom_bridge.rs）：`replace_all_sweep_children`
   帮手过滤 `NodeKind::ShadowRoot`，`innerHTML` 写入两处 sweep 改用——
   引擎把 ShadowRoot 挂宿主子列表的模型差异 vs spec childNodes 不含
   SR；此前 innerHTML 写宿主把嵌套声明式 SR 连根剥除
   （empty-state 页 slot 解析全断的根因）。
4. **级联匹配臂**（style-system/matcher/mod.rs）：`matches_pseudo_class`
   新增 `"has-slotted" => doc.is_has_slotted_element(element)`——此前落
   `_ => false` 使 `:has-slotted` 规则样式永不生效（gCS/reftest 全黑；
   probe 分诊矩阵 cInner/cOuter=black 实证）。DOM 选择器与 CSS 级联
   同源单源。
5. **matches 活体截胡**（shim part04.js matches trap）：裸 `:has-slotted`
   → `_zwHasSlottedLive(proxy)`（:focus/:modal 同款 JS 侧特判）——
   sel-form 文档元素 matches 落 `__zw_matches` 锚臂读 dom_html 快照
   （SlotAssign 变异批末才落地），同 turn appendChild 后恒 stale
   （probe P15 实证：sel-form slot_inner false 而 handle-form slot_new
   走 s83 活体臂 true 的不对称）。复合/函数形仍落 host（既有缺口），
   handle-form `slot:has-slotted` 复合面由 s83 `_hsOnly` 臂覆盖。

shim part05.js 两臂（`_matchPseudosOf` has-slotted 臂＋`_zwHasSlottedLive`）
k137 直连 `assignedNodes()` → k138 `assignedNodes({ flatten: true })`
（`__zwSlotAssignedNodes` WC-M3 flatten 递归既有，消费面升级）。

## 钉测与门禁账

- **dom --lib**：924P 0F（k138 四钉：auto 分配含 fallback 匹配、manual
  三态＋:not、嵌套 flatten 链直配非空链尾空不匹配/append 后展平含
  ancestor slot、非 slot 恒否）。
- **style-system matcher::tests::pseudo**：20P 0F，新钉
  `test_has_slotted_cascade_match`（分配非空命中 bare 规则＋空 slot 不命中）。
- **engine --lib**：4348P 3F——3F＝s330/r3046/p99s＝k137 恒红名单同键，
  零新红。
- **clippy**：`-p zero-dom -p zero-style-system -p zero-engine --all-targets
  -D warnings` RC=0。
- **探针页**（wpt-data 内临时页，电池发射前已删 trap-rm 核 census=0）3/3：
  initial（嵌套声明式 SR 结构可解＋flatten=0/matches=false/gCS black——
  展平语义判别断言，k137 在三点全翻转）；empty state（replaceChildren 后
  SR 存活＋空态五断言＋gCS black＝**s402 gCS 担忧对本通道解除**）；
  flattened chain（append 后 inner/outer 双 slot flatten=1＋matches 双真＋
  qSA 1/1＋gCS 双绿 rgb(0,255,0)）。

## 电池账

- 协议：flock `zw-k138-battery.lock` 独占；test-guard 逐腿包裹
  （--per-proc-mem 4 --total-mem 16 --time-limit 3000，CASE_WALL_S=60）；
  唯一 JSON_DIR（results-b{0,1}）/transcript；LEG-EXIT-b{0,1} rc=0＋
  BATTERY-DONE 标记；systemd-run --user --scope（MemoryMax=20G）发射，
  发射前 serialize-gate 三探（unit census 空/cargo 空/裸 bash ok）。
- 差分（键级 (page,test-name) Counter）：F2P 13——query-selector 9
  （element/text 分配＋replaceChildren 空态，direct/flattened 双形）；
  changing-001 4（reactive 失效→绿，级联臂＋活体截胡兑现）；
  host-part-003 1（`:host::part()` same-tree 否定断言——修③保根使
  跨树结构在 innerHTML 重写后存活，否定面恢复）。P2F 0；NEW/GONE 0。
- b0 roster 90 页 361 键 275P/86F/1 timeout；b1 288P/73F。

## 残 F 挂账（b1 名册内 has-slotted 族残红 6 键）

- **changing-002 2 键**：assign→empty 循环后期望 blue 得 blue 面已绿、
  assign 后期望 green 得 blue——reactive 失效传播时序缺口（changing-001
  4 键已翻，同族 deeper 时序挂 k139+ 评估）。
- **functional-changing-001..004.tentative 4 键**：k137 定谳踢出刀域
  （上游三浏览器全红＝speculative 自相矛盾，不实现反规范语义），FIXME
  注记维持。
- 既有挂账不变：`:is(:has-slotted)` over-match FIXME；exportparts；
  `::part` 盒布局 7 键（渲染流域）；marker 树位路径编码 FIXME(45-M3+)。
