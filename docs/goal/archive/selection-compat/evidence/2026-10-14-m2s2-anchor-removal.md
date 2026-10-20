# M2 切片 2 — anchor-removal ×2：拖选选区构建 + live-range 边界吸附（2026-10-14）

## 变更

### `crates/engine/src/js_dom_shim/part06.js`（Selection 段）

1. **拖选默认动作**（WPT anchor-removal 的 `select_range` 走 testdriver
   Actions pointer 序列，runner 无文本选区构建能力）：
   - `__zw_pointer_down_sequence` mousedown 默认动作钩（`_zwFocusSel` 后）：
     左键 + mouse 时 collapse 到 caret。**body/html 门**：命中键为 body/html
     = runner 零布局回落（origin 元素 gBCR 全零 → Actions 点退化为 (0,0)），
     坐标路径对垃圾点无意义 → 跳过坐标路径走 sel 键回落。
   - `__zw_pointer_move` 拖选钩（range-input apply 后）：`buttons & 1` 且
     downSel 非 range input 时 extend 到 caret。
   - caret 生产三路：
     - `_zwCaretFromPointDoc(x,y)`：文档域坐标→文本偏移命中（元素级 gBCR +
       行盒序遍历，depth ≤ 4）。
     - `_zwCaretFromSelText(sel,end)`：sel 键回落——body/html 直走
       `document.body`/`documentElement` childNodes（家族 identity 需求，
       querySelector 返回 proxy 族）；其余走 querySelector；depth ≤ 4 扫描，
       end=0 取文档序首个可选文本@0，end=1 取末个文本@length。
     - `_zwSelNAHome(node)`：NA 域同置重投影——取最近 id 祖先（含自身），
       经 `globalThis[id]` 取 named-prop 族孪生，按子索引路径重投影回等位
       节点；nodeType + 文本 nodeValue 双重校验 + **连通性门**
       （`_zwDocContains36`，历史代残留 globalThis[id] 不接）；任一步失配
       返回原对象。坐标/sel 键两路产出的 offsetNode 均经此重投影。
2. **`__zwSelectionFollowRemoval` 匹配器 NA 族桥**（match-only，不换吸附
   目标对象）：`same()`/`sameParent()` 在直键（identity/`__zwHandle`/
   `__zwSelector`）皆不命中后，双侧经 `_zwSelNAHome` 重投影再比 identity。
3. **吸附目标保持 `oldParent262` 原对象**（规则① `n = oldParent262`）：与
   part03 live range 吸附同源，identity 面天然一致；不做 NA 重投影（见下
   「bridge 回归」节——`globalThis[id]` 即便连通也可能是异族孪生）。

### `crates/engine/src/js_dom_shim/part04.js`（R3254-E2 切片 11 处，跨段
共享根因）

**换代刷新**：innerHTML setter 解析 walk 的 `_r3kReg` 注册值打
`__zwR3kNamedId` 标记；同名现值带同标记 = 本机制上一代注册（上一轮
innerHTML 的解析 wrapper），树已重建则其 `.remove()` 等方法落在死节点上
→ 覆盖刷新。脚本自有值（无标记）仍不遮蔽。这是测试 2
（「with asserts」）的直接根因：**第二次 `body.innerHTML = kHTML` 后
`parentParagraph.remove()` 不触达活树**（探针实证 remove 后
`docContains=true`、无删除事件），同步触发 `__zwSelectionFollowRemoval`
无从发生。

## 根因链（探针记账）

1. **DOWN 落点退化为垃圾元素**：runner `resolve_pointer_target` 在 origin
   元素零 rect 时传 sel='body'+(0,0)；querySelAll 新 proxy 在 Actions 时
   有真实刷新布局（探针 cp1 gBCR L8 T24.5 W157.6 H18.6），坐标路径把
   (0,0) 最近元素语义解析到错误文本元素 → body/html 门。
2. **跨族 identity 必败**：同树节点在 shim 内有多族稳定 JS 对象——树行走
   /getElementById/querySelector proxy 族、named-prop 全局族（R3K 解析
   wrapper = 活树真实节点对象）、`_wrapNodeEntry` 每访问新建的纯文本包装
   族。探针实证 `document.getElementById('parentParagraph') !==
   parentParagraph`（bare global），两侧各自 `firstChild` identity 恒定但
   跨族恒异。WPT 断言用 bare global（named-prop/解析族），caret 产出树族
   对象 → `_zwSelNAHome` 桥。
3. **R3K 世代滞留**：guard `globalThis[nid] === undefined` 使第二代
   innerHTML 不刷新同名 global，死 wrapper 遮蔽活节点 → 换代刷新。
4. **桥测试回归**（`test_selection_live_range_removal_snap_m2s2` 场景②
   `__b3`，js_dom_bridge_tests/part06.rs:1496）：吸附目标若取
   `_zwSelNAHome(oldParent262)`，`globalThis['r']` 与测试持有的
   `querySelector('#r')` proxy 属异族（即便经换代刷新保持连通）→
   `focusNode === r2` 败。连通性门只解历史残留面，异族孪生 identity 面
   由吸附目标回退 `oldParent262` 原对象解决——WPT 面的
   `removed.parentNode` 本就是解析族对象，与 bare global 同物，无需重投影
   （回退后 WPT selection 复跑 2968/2 不变，实证该重投影非承重）。

## 修复结果

- `make testharness-selection`（test-guard 包裹）：**2966 → 2968 Pass，
  4 → 2 Fail**。`anchor-removal.html` ×2 转绿（本切片目标）；余 2 失败
  维持定性（Document-open 跨域挂账 / editing/event 排除域），见 master.md
  缺口清单。
- `cargo test -p zero-engine --lib`（test-guard 包裹）：**3044/0**（切片 1
  基线 3043 + 切片 1 新增桥测试；含回归修复后复跑）。
- 临时探针 `tests/wpt-runner/wpt-data/selection/zz-probe-snap*.html` 已删
  （gitignored，未入库）。

## 门禁

- shim 拼接 `node --check` 通过（part01…part07 全量 concat）。
- `make testharness-uievents`（TIME_LIMIT=2400，按 fileapi-compat
  2026-10-11 行记账）：**仍无完成结果**——runner 跑满 2400s guard 墙后被
  终止，0 条用例输出（该 runner 结果至结束才刷出），双侧依旧无基线。墙钟
  撞击为套件级既有状况（切片 B 之前已存在，见 csv fileapi 行），非本切片
  回归。暴露面以 selection 套件覆盖佐证：testdriver Actions 拖选端到端
  驱动同一 pointer 路径（本切片钩子全量在环）且 2968/2 绿；engine lib
  3044/0。uievents 专项基线（更大 TIME_LIMIT 或 FILTER 分片）另行记账。
- 本切片纯 .js shim 变更，无 .rs 变更；受影响构建（engine lib）与行为
  （selection/uievents testharness）已验证。
