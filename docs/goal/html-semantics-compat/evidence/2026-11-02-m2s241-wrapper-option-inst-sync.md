# M2 s241 — wrapper option 读/写源分裂补面（实例层写侧同步；探针勘根翻案「读侧不经实例层」假设）

日期：2026-11-02。切片号 s241（承 s240）。根：s240 挂账「wrapper 读/写源分裂」
——handle-backed pending 形（template content/live innerHTML）set value/label
落属性层但 IDL getter 读旧值（o.value='x2' 后读 "v1"、getAttribute 读 "x2"、
sel.value 链读 "x2"）。

## 入场定刀位

s240 下一步候选：wrapper 读/写源分裂补面（s240 新账——写侧已通读侧 stale，
探针在案语料无钉面，先勘根读侧快照源定归因再施工）。

## 勘根（trace 探针两轮；仪器教训在账）

- **首轮 trace 探针被报告截断误读**：`_zwOptAttrNoNs` 决策路径插桩
  （`globalThis.__zwS241Trace`，临时仪器验后删），probe 以 `join('\n')` 落
  assert 消息——报告在首换行截断，只见首个 marker，误判「读侧不经
  `_zwOptAttrNoNs`、另寻 serving path」（_zwListOfOptions/_ocList 方向白查
  一轮：items 是 `_zwLocalChildNodes`/`_handleChildren` 原对象直推，无二次
  包装）。**分隔改 `' ~ '` 单行化复跑拿到全 trace，翻案。**
- **全 trace 定谳**（仪器教训：assert 消息含 `\n` 被报告截断——探针 dump
  一律单行化）：
  ```
  read o1.value → NoNs value elKey=@__n1 listLen=2 → HIT inst value=v1
  write o1.value=x2 →（实例层无变化）
  read o1.value → HIT inst value=v1（仍陈旧）
  sel.value → NoNs value elKey=__n1 listLen=null → 实例 miss → 活读 → "x2"
  ```
- **根因两层**：
  1. **读侧**：trap OPTION 读臂（part03 value ~27578 / label ~29269）经
     `_zwOptAttrNoNs` **实例优先**——handle-backed 解析树的
     `_zwAttrInstances` 有**解析期种子记录**（elKey=`@__n1`，2 条 =
     value+label，local/qname 全小写）。s240 账面「IDL getter 读 pending
     快照层」修正：快照源即 R122 实例层（JS 侧属性权威层）的种子记录，
     非独立 pending 层。
  2. **写侧**：R5009 value 族臂（part04 set trap，BUTTON/LI/PROGRESS/PARAM/
     OPTION 五 tag 共尾）与 R3069 reflected 臂（OPTION label 在列）直写宿主
     （`__zw_set_attr` 系）**不经 setAttribute** → 实例层不刷新（setAttribute
     臂 part04 ~3797 有 qname 命中原位更新/否则 upsert 的同步；R5006 已有
     移除侧同步先例 `_zwAttrInstanceRemoveKey`——直写臂缺 set 侧对应物）。
- **sel.value 链读新鲜是键派生不一致的意外**：`_zwOptionValue` 传裸 handle
  （`__n1` 无 `@`，trap 用 `_elKey`=`@handle`）→ 实例查 miss → 活读回落——
  非设计保证，挂账统一候选（须先核实例层全写径同步再动，本切不动）。
- 规范面：html.spec.whatwg.org 本环境不可达（同 s238/s240 先例），语义锚点
  用既有语料钉面互证（reflection.js「reflected only on setting」+ R5009/
  R3069 写侧既有 verbatim 形），spec 语义 = reflected IDL setter 与
  setAttribute 同一 content attribute 写步骤 → 实例层同步口径以 setAttribute
  臂为锚。

## 桥力学

`_zwAttrInstSyncNoNs(key, name, value)`（part05，`_zwAttrInstUpsert` 邻位）：
与 setAttribute 臂同口径——qname 命中原位更新 `.value`，否则
`_zwAttrInstUpsert(key, name, null, null, name, value)`（`_zwAttrInstList` 自
建列表，与 setAttribute 首写行为一致）。读侧不动（`_zwOptAttrNoNs` 实例优先
+ 活读回落结构保留——种子记录刷新后 HIT 即新鲜，miss 面活读照旧）。

## 施工（一处 helper 两挂点，零 rust）

- part05：`_zwAttrInstSyncNoNs` helper + s241 注记（勘根定谳摘要）。
- part04：R5009 value 族尾宿主写后挂接（五 tag 共尾单点）。
- part05：R3069 reflected 尾宿主写后挂接（label 全族单点）。

## 验证

- node --check 全 shim concat（js_dom_bridge.rs include_str! 序 18 片）干净；
  `make zero-wpt-runner-release` 重建 28.5s。
- 探针 before/after（zz-s241-probe.html，验后删）：before `o1.value.after=
  "v1"`（陈旧）/ after 全面新鲜——value 写读回环 "x2"、label "L2"、
  **attributes 枚举面 `value=x2,label=L2` 同步刷新**（R122 attrObj 值实例
  优先，同步后跟随）、sel.value="x2" 不变。
- solo the-option-element **108P/0F/0T**（R3907/s239/s240 钉面零回归）。
- 全树 A/B（evidence/logs/s241-ab.log vs s240-ab.log）：见下节。

## 全树 A/B（evidence/logs/s241-ab.log vs s240-ab.log）

- (key,state) 对 sort -u join：8293/8293 对，LOST 0 / GAINED 0 / 状态翻转
  0（join 输出唯一一行「Pass  Pass」形为键内含制表符的 awk 切分视觉假象，
  双侧同 Pass 非翻转）——**完美 A/B**。行级 7708P/507F/81T 双侧逐数同。
- 靶面无语料行（wrapper option IDL 读面语料无钉面，s240 同形）——靶面验证
  由探针 before/after 承担（含 R122 attributes 枚举面）。改动臂 = 实例层
  set 侧同步（既有记录刷新 / 无记录新建），只影响「实例优先读 × 实例层有
  记录」交叉面——语料无该交叉面行，零位移与机理一致。

## 残余挂账（本切不动）

- **同类直写臂未同步清单**：meter 六属性臂（R4002）、progress.max 臂、
  FRAMESET cols/rows 臂等散点直写——读侧不经实例优先（meter/progress 读走
  专用数值分支活读）、语料无钉面，挂同类同步候选（扩面前先勘各自读面）。
- **`_zwOptionValue` 键派生不一致**（裸 handle vs `_elKey` 的 `@handle`）：
  现状靠活读回落保持新鲜，统一须先核实例层全写径同步（本切已补 value/label
  直写臂，meter 等散点臂未补），挂账。
- s240 存量：DOMParser 工厂 option accessor 面（s239 家族延伸）；plain
  detached select.value undefined（s186 FIXME）；option.text null→'' vs
  DOMString "null" 字面差异（语料无钉面两态连续）。

## 下一步候选

DOMParser 工厂 option accessor 面（s239 家族延伸，独立勘根轮）；modal-
dialog-selection（pointer 拖拽面存量）；customizable-select 177F（挂起待
用户拍板）；img SVG 密度族（naturalWidth 22F + current-pixel-density 13F，
需独立勘根轮）。
