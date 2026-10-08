# slice44 红档索引（red evidence index）

分支 `slice44-live-collection-residuals`；基线 `8c5752049`（origin/main tip，= slice43 合并树，main 未前进）→ 钉提交 `cfec8f59e` → 修复提交 `7c44d04de`。

## 证据链总览

| 阶段 | 树 | 日志 | 结果 |
|---|---|---|---|
| 自然红（钉先行） | `cfec8f59e` | `natural-red-s44.log` | 1P/2F，EXIT=101（pin3 绿=语义已正确的覆盖钉） |
| 修复后绿 | `7c44d04de` | `fixed-green-s44.log` | 3P/0F，EXIT=0 |
| 变异 RED M-A | `7c44d04de` + `mutation-only-delta-s44-a.diff` | `mutation-red-s44-a.log` | 2P/1F，EXIT=101，红在 pin1 `__r_qsa_doc` |
| 变异 RED M-B | `7c44d04de` + `mutation-only-delta-s44-b.diff` | `mutation-red-s44-b.log` | 2P/1F，EXIT=101，红在 pin1 `__r_qs_deep` |
| 变异 RED M-C | `7c44d04de` + `mutation-only-delta-s44-c.diff` | `mutation-red-s44-c.log` | 2P/1F，EXIT=101，红在 pin2 `__r_head_named` |
| 变异 RED M-D | `7c44d04de` + `mutation-only-delta-s44-d.diff` | `mutation-red-s44-d.log` | 2P/1F，EXIT=101，红在 pin3 `__r_moved_id` |

四个变异各自独立 run（`cargo test -p zero-engine --lib -- s44`，test-guard 包裹，失败列表唯一归因到单钉）；每次 run 后 `git checkout -- <单文件>` 还原，`sha256sum` 与 `git cat-file blob HEAD:<file>` 逐字节一致（自证通过：part06.js `6b09906f…730ff`、part05.js 两轮均一致）。

## 钉位实测行号（树 `7c44d04de`）

- `crates/engine/src/js_dom_bridge_tests/part44.rs:47` — `static_selector_query_parsed_subtree_remove_s44`（任务①）
- `crates/engine/src/js_dom_bridge_tests/part44.rs:116` — `named_access_doc_tree_root_head_mount_s44`（任务②）
- `crates/engine/src/js_dom_bridge_tests/part44.rs:160` — `parsed_subtree_move_lifecycle_s44`（任务③）
- `crates/engine/src/js_dom_shim/part05.js:10219` — `_zwPendingRemovedSels()`（任务①修复，document 面共享漏斗）
- `crates/engine/src/js_dom_shim/part05.js:11002` — `_zwDocContains36` 根部统一（任务②修复）
- `crates/engine/src/js_dom_shim/part06.js:2128` — document.querySelector 剔除+重查（任务①修复）
- `crates/engine/src/js_dom_shim/part06.js:2533` — document.querySelectorAll 剔除过滤（任务①修复）

## 修前探针（TEMP-PROBE，未入提交；树 `cfec8f59e` 钉提交，过程证据归档于此）

任务① document 面移除后查询：

```
__r_qsa_doc   = 3        （stale：已移除 parsed 后代仍计入，规格应为 1）
__r_qs_deep   = "false"  （querySelector('#w44 img') 仍可解析已移除后代）
__r_qs_first  = "c44"    （querySelector('img') 首命中为已移除 c44）
__r_qsa_html  = 3        （元素面跨容器 stale——边界保持，见 verdict ③申报）
__r_qsa_body  = 1        （同桶 R310 既有覆盖，绿）
__r_qsa_body_cls = 1     （同桶类选择器，绿）
```

任务② 修前 `window.hd44`（head 内 id 命名元素）为 `undefined`（body 根谓词漏判），即 natural-red pin2 的 `__r_head_named="false"`。

## 变异 RED 判定位

- **M-A**（撤 document QSA 过滤，part06.js:2534-2548 块）：pin1 红在 `__r_qsa_doc`（left "3" vs "1"，回到修前 stale 值，与探针 `__r_qsa_doc=3` 互证）。
- **M-B**（撤 document QS 剔除+重查，part06.js:2127-2156 块恢复 `if (hit) return` 直返）：pin1 红在 `__r_qs_deep`（"false" vs "true"）。
- **M-C**（`_zwDocContains36` 根部回退 `body || documentElement`，part05.js:11002）：pin2 红在 `__r_head_named`（"false" vs "true"）。
- **M-D**（撤 part05.js:10413-10417 R51c added 方向摘除块）：pin3 红在 `__r_moved_id`（"false" vs "true"）。**判定位与立项预估（`__r_moved_fresh`）不同**，实测机制更有归因价值：撤摘除点后 m1/m2 同残 pendingRemoved 与 pendingAdded——fresh tag 面经 pendingAdded 双源并集仍复见 2（先过）；ID 面快照命中被 stale pendingRemoved 剔除，且 `#id` pending 回退不触发（host 快照仍命中）。摘除点是 move 生命周期闭环的承重点，判别力成立。

## 单腿充分性（engine 域谓词/查询面变异判别）

`QUICKJS_TEST_CRATES`（Makefile:190）不含 zero-engine → 三钉及其变异 RED 只存在于 v8 腿测试集；quickjs 腿测试面不受影响（腿数不变）。故 engine 域单腿变异 RED 结构上充分（Makefile:190 口径，slice41/42/43 同判例）。

## 钉本面（HEAD 绿）与对照臂

- pin1 主判定 = document 面 QSA/QS 不含已移除 parsed 后代；对照臂：id 面 null（s43 remFlat 展开面不回退）、held NA 清空（s43 pin1 面不回退）、同桶元素面 R310 覆盖（=1）、跨容器元素面 stale=3（边界保持申报，verdict ③）。
- pin2 主判定 = head 子树命名元素入册；对照臂：detached 不入册（s43 pin2 口径）、body 挂载零回退。
- pin3 = move 生命周期闭环（移除向 0/0/null + 回插向 2/2/true/2）；判别力由 M-D 承担。
