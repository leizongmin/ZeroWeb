# E2 切片 12 — selection/textcontrols selectionchange 全语义面（2026-09-08）

**用例来源**：上游 WPT `web-platform-tests/wpt` @ `315976933870b34d6ea30e3f6643403edae678ba`
`selection/textcontrols/` 首批 3 案，经 `fetch-selection-subset.sh`（`TEXTCONTROLS_CASES`）
拉取到 `wpt-data/selection/textcontrols/`（gitignored）。runner `SELECTION_TEST_SUBDIRS`
追加 `selection/textcontrols` 目录。

**导入清单**：
- `selectionchange.html` — 60 subtests（4 collector × 15 语义：selectionStart/End
  setter、setSelectionRange、select、setRangeText 的派发/不派发/去重面）
- `selectionchange-bubble.html` — 4 subtests（text control selectionchange bubbles=true）
- `onselectionchange-content-attribute.html` — 2 subtests（内容属性 handler 编译触发）

**排除项**（有据）：`focus.html`（pointer 拖选 + focus 取消扩展——依赖布局命中测试，
runner 无布局 rect，同 anchor-removal Actions 拖选归因 renderer S3）；`click-input-after-
iframe-focus.html` / `initial-selection-*`（focus 交互/渲染面）；`selectionchange-on-
shadow-dom.html`（shadow DOM 面，js-dom 域）；`selectionchange.html`(textcontrols 之
外的 caret/bidi/contenteditable 子目录) 归后续切片。

## 本轮结果

```
selection 全套件：2993P / 6F（基线 2928P/5F，净 +65P——新增 66 subtest 全 Pass）
textcontrols 三案 25 连跑 0 Fail/Timeout
keyboard 套件 18P/7F/2T 零回归
```

**切片 13 勘误（2026-09-08）**：上文「三连跑稳定」**仅对 textcontrols 三案成立**；
当时记录隐含的「onselectionchange-on-document 全量跑稳定通过」与复验不符——全量
套件 3 连跑确定性 6F（fail 集恒定含该案 IndexSizeError）。详见下方残余记录段与
切片 13 修正。

## 实现语义（shim）

spec 依据：selection-api「Firing selectionchange event」（element target bubbles=true /
document target bubbles=false）+「Scheduling selectionchange event」（has-scheduled
按 target 去重）+ HTML「To set the selection range」（extent/direction 实际变更才
queue select/派发任务）。

1. **part06 排程器**：pending 条目改为 `{node, bubble}`（同任务多 target 各持自己的
   bubbles flag）；去重按 `entry.node === target`（spec has-scheduled 语义）。
2. **part03 setSelectionRange**：同值早退——`(start,end,direction)` 全等不排程。
3. **part03 setRangeText**：末步「Set the selection range with selection start and
   selection end」→ `(start,end)` 变更才排程（preserve 不变形态 0 事件）。
4. **part04 select()**：spec「Set the selection range with 0 and infinity」→ 已全选时
   不排程（select() twice = 1 事件）。
5. **part04 selectionStart/End/Direction setter**：前值/后值比对，变更才排程。
6. 全部 text control 调用点 `bubbles=true`；document 调用点（`_zwSync`）不变。

## 修复的 runner 基建缺陷（本轮重大发现）

**timer stub 单 turn 存活缺陷**：`V8Sandbox::execute` 每次执行都把 `register_callback`
注册的原生回调重新 `global.set` 到全局对象（v8_runtime.rs execute 内 for-callbacks
循环）——runner `timer_stub` 对 `__zw_setTimeout` 的 JS 层赋值只存活一个脚本 turn；
下一个 `execute_script`（probe 泵）起，shim `setTimeout` 静默回落 host 真线程路径
（`drain_next_async_callback_if_pending` 每次 execute 只 resolve 一个、到达序随机）。
后果：页面加载 turn 之后排的所有 timer 派发顺序随机——本次 WPT textcontrols 断言族
~30%+ flake 的根因，也是 master.md 既有记录「onselectionchange-on-document timer 时序
flake」的真实根因。

**修复**：stub 换独立名 `__zw_test_setTimeout`（不在 host 注册表 → 永不被 rebind）；
shim `setTimeout`/`setInterval` 经 `_zwHostSetTimeout()` 优先探测该名，生产路径（无
stub）回落 `__zw_setTimeout` 原语义零回归。全部 timer 从此走 stub 确定性队列（push 序
FIFO + due 分割保序）。

**验证**：最小复现探针（4 collector × selectionStart 语义）修复前 ~27-50% 失败率 →
修复后 textcontrols 三案 25 连跑 0 失败；make test 19,005 全绿（含 runner 既有
send_keys/timer 相关测试）零回归。

## 残余记录

- `onselectionchange-on-document.html` 第 3 subtest（'task to fire selectionchange
  event gets queued each time'）稳定报 `IndexSizeError: The given offset is out of
  bounds` unhandled rejection（`setPosition(container, 2)` 于 innerHTML pending 未
  apply 时读 fusion childNodes 越界）。基线（stash 验证）同型 flake 亦存在——既有
  问题，非本轮引入，待 innerHTML fusion 视图与 selection 端点校验协调切片。
  **切片 13 勘误（2026-09-08）**：本段初版记录「FILTER 单案跑下稳定失败、全量套件
  跑下通过（三连跑稳定）」**与实测不符**——复验（后续轮次）全量套件 3 连跑均确定性
  失败（fail 集恒定 6F 含该案）；「FILTER 独有 / 全量通过」的定性系当时验证不充分
  所致的错误结论，已在下节勘误说明与 master.md #21 同步修正。

### 同日追加：flake 精确归因（最小上下文复现）

复现形态：subtest 1 + subtest 2（各做一次 `container.innerHTML = ...` + setPosition）
前置后再跑 subtest 3 原样——第 4 次 `setPosition(container, 2)`（spin 后）抛
IndexSizeError，**抛点实测 `container.childNodes.length === 0`**。

机制定位（初版）：同 sel 连续多次 innerHTML 赋值（s2 一次 + s3 一次）后，spin await
边界处 `#container` 的融合 childNodes 视图塌缩为 0（R304 挂槽的解析 wrapper 从
overlay 桶消失 + `_zwChildBaseCache['#container']` 被 setter 置空 `[]` 后未再失效）
——第二次 innerHTML 的 queue-side invalidate 与 `_zwFragmentAdded` 预注册的桶条目
存在时序耦合。单案最小形态（单 subtest、单次 innerHTML）700+ 连跑不复现；初版记录
称「全量套件三连跑稳定通过」（**该句已勘误**——见上）。

定性与切分：js-dom 共享面（fusion 视图 R51c/R304/R380 族）的既有深缺陷，非本轮
selectionchange 变更引入；修复需专门切片（stacked same-sel innerHTML 的桶生命周期
重整），记入 js-dom/编辑协调点。

### 切片 13（同日后续轮次）：修复落地 + 归因修正

**归因修正**：初版机制定位的「base 缓存 `[]` 未失效」路径经 shim 侧逐点插桩
（baseSet/cacheHit/overlay 进出 + 桶 added/removed 计数）复验**不成立**——塌缩点
（C subtest spin 后首读）实测 base cache **len=2（正确 rebuild）**，桶 added=4
removed=4；塌缩发生在 overlay 合并层：**removed 补偿残留**。精确链路：

1. innerHTML setter（sel 路径）入队 SetInnerHtml + 桶记账 added（解析 wrapper）/
   removed（旧子 proxy——经 `_proxyCache` 按 sel 稳定 identity）。
2. apply+bump（pa2b）只清 parse 补偿 **added**（K3 切片 C），**removed[] 条目
   残留**。
3. 换代后新基底 rebuild：host 快照真实子经 `_wrapSelector` → `_makeProxy` 命中
   `_proxyCache` **复用同一 proxy 对象** → overlay 的 removed 剔除按 identity
   命中 → 快照真实子整批剔空（trace 实证 bucket a=4 r=4 时 fresh base 2→0）。
4. `setPosition(container, 2)` 读 `_nodeLength` = childNodes.length = 0 → offset
   2 越界抛 IndexSizeError。初版归因的「base 置空未失效」「桶条目消失」两说均
   系插桩不足下的误判，此处以 trace 实证修正。

**修复**（js-dom 共享面 pa2b 语义扩展，与本 goal 协调落地）：`__zw_apply_generation_bump`
补 **removed 补偿同批作废**——桶 removed[] + 全局 `_zwPendingRemoved` 在 apply
代际边界清空。语义依据：removed 条目是「快照已含节点、host apply 未落」窗口的
视图修正补偿，apply 后快照已真删除，其另一半消费面（live 集合/query stale 剔除）
同理只服务 apply 前窗口；与 K3 切片 C 的 parse 补偿 added 清理同族。handle-only
removed 条目本为死数据（R51c 压实语义），一并清除。

**验证**：

```
onselectionchange-on-document.html：4 subtest 全 Pass（修复前 3 连跑确定性失败）
selection 全套件：2993P/6F → 2994P/5F（净 +1；5F 全为既有跨域归因），3 连跑 fail 集恒定
单测 r3254_e2_slice13_apply_generation_invalidates_removed_compensation：
  无修复复现失败（removed 残留 → fresh base 塌缩 0）、有修复通过——根因锚定
```
- textcontrols `selectionchange.html` 断言依赖事件在**单 spin** 内到达；stub 队列
  FIFO 下已稳定（25 连跑）。

## 门禁

- `make test`：19,005 passed / 0 failed（guard 包裹）
- `cargo clippy --workspace --all-targets -- -D warnings`：零警告
- `cargo fmt --all -- --check`：无 diff
- 单测：`test_selectionchange_textcontrol_full_semantics_r3254_e2_slice12`（六组断言）
