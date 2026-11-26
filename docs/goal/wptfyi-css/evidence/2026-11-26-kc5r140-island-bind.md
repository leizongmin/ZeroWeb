# kc5r140 · css-shadow「island-bind」刀收账证据

日期：2026-11-26 · 流：wptfyi-css Rally 跨轮次换岗轮 · 刀：k140 handle 岛形统一 bind journal

## 根因与实现

- 根因（k139 收账挂账的「handle 岛形」残 F 硬错）：shim t8r innerHTML/setHTMLUnsafe 物化臂经 `_wcRebuildAsHandle` 铸的 wrapper handle 是 detached 复制岛——引擎 `SetInnerHtmlOnHandle` 物化的真实 attached 节点与岛 handle 无绑定，页面经岛 handle 的属性写（slot 移除 name 触发 re-slotting）与 by-handle 读（querySelector 产物 gCS）都解析到 detached 节点：写不穿透引擎树、读 styles miss。
- 实现两面：
  1. shim `part04.js` t8r innerHTML 物化臂 + `part05.js` setHTMLUnsafe 物化臂：`_handleChildren[handle] = …` 落表后按平行树位（解析代理 `_ihAdded[i]` ↔ 物化 wrapper `_ihMat[i]` 同循环 push 1:1）递归发 r80 同款 bind journal `__zw_bind_handle_by_path(handle, path.join(','), childHandle)`——锚 handle + child-index 全节点序路径；物化失败回退代理（无 `__zwHandle`）与纯文本子自然跳过；`typeof` 门 + try/catch 包裹。
  2. engine `js_dom_bridge.rs` `BindHandleByPath` 臂：path 下行从 `walk_child_path` 换本地下行**滤 ShadowRoot 位次**（k139 SlotAssign `@path:` 臂同款——引擎模型 SR 挂宿主 children 而 shim `__zw_child_nodes` 无 SR，原生序在含 SR 容器整体错位）；k138 `replace_all_sweep_children` 同款过滤先例；`walk_child_path` 是 t7 语义面不动；`handles.contains_key` 才覆盖重绑（未注册 handle 不新增）；批末 persistent_nodes 回填保持跨批。

## 钉测（crates/engine/src/js_dom_bridge/shadow_forest.rs，4 全绿）

- `k140_island_bind_sr_container_write_through`：SR 容器 bind 跨批写穿 + SlotAssign 岛 handle 通道（manual root）。
- `k140_island_bind_filters_shadowroot_position`：宿主容器 path[0] 滤 SR——修复前形态（`walk_child_path` 旧形）红已验证（SR 误绑实锤），修复后绿。
- `k140_island_bind_deep_path_element_container`：元素容器深路径 [0,1] 零漂移守卫。
- `k140_island_bind_reslot_after_name_removal`：mutation 层复刻 shadow-reassign-dynamic-003——bind + RemoveAttrOnHandle + 批末 resolve_slots_all_hosts → `get_assigned_nodes` 命中 span；钉引擎 mutation 层链完整。

## 运行时诊断定谳（K140DIAG 临时日志，已撤）

diag 页（ShadowRoot innerHTML 双 slot + removeAttribute re-slot）：bind 为 SR 容器发射（`bind anchor=__n0 path=[0] handle=__n1 -> attached`）＋ removeAttribute 落 attached（`remove_attr handle=__n1 name=name attached=true`）——引擎侧链（bind 重绑→属性写穿透→批末 resolve_slots→get_assigned_nodes 重解析）全通。残 F 定性为**级联/失效消费面**（slot 属性变化→分派变化→slotted 内容 computed style 未刷新），非岛形刀域，归 k141 挂账。

## solo 预验（四目标页，ONLY_SUBSTR 定向）

| 页 | b1 结果 | 定性 |
|---|---|---|
| shadow-host-removal-invalidation | 1P/0F | **翻绿（F2P +1 兑现）** |
| scope-pseudo-in-shadow | 1P/1F | 读面已修（gCS 到真树，红取代空）；残红＝`:scope` in shadow root 匹配面 |
| shadow-reassign-dynamic-003 | 0P/1F | 引擎链全通（diag 定谳）；残红＝re-slot 后 slotted computed style 失效消费面 |
| slotted-invalidation | 0P/1F | className 写已落（journal 通道验证）；残红＝`.outer ::slotted(#slotted)` 祖先类级联匹配面 |

## 电池双腿（协议：flock /tmp/zw-k140-battery.lock + test-guard 逐腿 + 唯一 JSON_DIR/transcript）

- b0＝k139 归档 bin（.acceptance/wptfyi-css/k139/bins/r139-slot-pathref-runner，md5 `d4f6a87eab0374d66161bdf982cb2c74`）；b1＝本刀 bin（.acceptance/wptfyi-css/k140/bins/r140-island-bind-runner，md5 `cdcb897fed9e8430305506fc5e4eab6d`）。
- **2 腿 RC=0（LEG-EXIT-b0 rc=0 12:59:45／LEG-EXIT-b1 rc=0 13:00:22）**；90 页全名册：b0 290P/71F/1T → b1 **294P/67F/1T，净 +4**。
- 键级差分（`/tmp/zw-k140/diff-k140.py`，(page, test-name) Counter）：**P2F=0（红线成立）NEW=0 GONE=0，F2P=4**——四键全数岛形面（by-handle 读/写经岛 handle 直达引擎）：
  1. `shadow-host-removal-invalidation.html`「:host rules are properly invalidated when stylesheets are removed」（solo 预验同键）
  2. `slotted-nested.html`「Style invalidation works correctly for nested slots」
  3. `keyframes-002.html`「@keyframes from the document should apply in the shadow tree」
  4. `part/both-part-and-exportparts.html`「::part() rules match elements having both @part and @exportparts」
- 两腿同构 1 Timeout 非本刀域同位恒持（k139 收账同位）。solo bin（be15199e）与电池 b1 bin（cdcb897f）差异仅为诊断期 tracing 日志行（引擎侧 tracing::warn K140DIAG，已撤），功能态同一（电池 F2P=4 含 solo 四页域兑现为证）。

## 门禁

- engine --lib 全量（test-guard 包裹，rc=101 形红＝3 恒红名单）：**4353P / 3F**——`test_dialog_focus_restore_declarative_shadow_s330`／`test_expando_enumeration_r3046`／`test_p99s_pure_path_inline_custom_property`，与 k139 收账红名单（s330/r3046/p99s）逐名恒对账，零新红；本刀 +4 pins（HEAD shadow_forest 10→14 test fn）全数绿。
- clippy：zero-engine `--all-targets` RC=0。

## 残 F 挂账（k141+ 候选）

- slot re-assignment → slotted 内容 computed style 失效消费面（shadow-reassign-dynamic-003）。
- `:scope` in shadow root 选择器匹配面（scope-pseudo-in-shadow）。
- `::slotted()` 祖先类级联匹配/失效面（slotted-invalidation）。
- 原有：exportparts 跨树转发、::part 盒布局 7 键（渲染流域）。
