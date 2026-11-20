# d58 · 刀 D 落地——R5450 AdjacentText wire 打记＋D3 copy 守卫消费：join 面 wire 批双插修复

**2026-11-19 · d58 轮**。上轮 d57 定谳 join 面复制形 P2F=18（`abcdefdef`）＝「bi=1 后
harness 回调读 `getTargetRanges()` 引擎往返撕开 pending-journal materialize 时序」。
本轮 TEMP DIAG57 插桩拍到 wire 批，定谳修正为 **wire 批双插**（非时序竞态）——shim
侧同位同文记了两条引擎 op；刀 D 最小修复（wire 臂打记＋copy 守卫消费）落地，bs13
复验双插消失。

## 1. 定谳：wire 批双插（TEMP DIAG57 实证）

- 插桩：`user_actions.rs` sandbox 路径（executor=None，`script_contenteditable_delete`
  经 `script_gen.rs:1678` 生成 `__zw_ce_delete('sel')` 走私有队列）recorded 批 dump
  ＋`apply_dom_script_result` apply 批 dump（`tracing::warn`，runner 须
  `ZW_SHAPED_ADVANCE_TRACE=1` 才 init subscriber）；part01 临时 `__zwDiagJoin`
  marks（已回退）。两处插桩交付前已删净（工作树只余刀 D 两文件）。
- bs13（`zz-diag-bs13.html`：`initializeTest("<div>abc<p>def<br>ghi</p></div>")`、
  caret `(def,0)`、`sendBackspaceKey`、`assert_false(true, dump)` 恒败 dump 页）拍得
  delete 脚本 wire 批 verbatim（2026-11-19 12:43）：

```
DIAG57 sbx-recorded n=0 ops=[]                          ← probe 脚本（空批）
DIAG57 sbx-recorded n=7 ops=[SetInnerHtml { selector: "html:nth-child(1) > body:nth-child(2) > div:nth-child(1)", html: "<div>abc<p>def<br>ghi</p></div>" }, CreateTextNode { handle: "__n0", text: "abc" }, CreateTextNode { handle: "__n1", text: "def" }, RemoveChildAt { parent_selector: "p", child_index: 0 }, Remove { selector: "p > br:nth-child(1)" }, InsertAdjacentText { selector: "p", position: "beforebegin", text: "def" }, InsertAdjacentHtml { selector: "p", position: "beforebegin", html: "def" }]
DIAG57 apply n=7 baselen=248386 has_setup=true
Fail input-events/zz-diag-bs13.html :: zzdiagBS13
  assert_false: … join="<div>abcdefdef<p>ghi</p></div>" bi=1 …
```

- **op6＝R5450 跨父移动 AdjacentText wire**（part04 insertBefore 臂，
  `__zw_insert_adjacent_text(refNode.__zwSelector, 'beforebegin', …)`）；
  **op7＝join D3 序列化 copy**（part01 copy 臂 `__zw_insert_adjacent_html`）。
  同位（`p` `beforebegin`）同文（`"def"`）两条 op → 引擎终态 `abcdefdef`。
- base 单插机制：base 树文本壳带 `__zwHandle`/`__zwSelector`（D3 守卫首条命中跳
  copy）；knife 树文本壳 plain（无 handle/sel，仅 idx-stamp）漏守卫 → copy 也跑。

## 2. 病灶链三层（全实锤）

1. **C-view world-sync plain 重建**：C-view 后 query-family `el`（path-form
   canonical）与 page-family 选区锚不同体 → `__zw_ce_caret_range` D3 world-sync
   回灌 `el.innerHTML = _htmlSync`（part01:10379）且 `__zwCeWorldSyncing=true` 压
   D1 stamping → 子树重建为 plain（无 handle/sel，仅 idx-stamp；family dump
   `shell t=3 h=- sel=- idx=0 v=def`）。
2. **tombstone → R5450 wire**：join plain 树 `takeLineFrom` 的 removeChild 置
   tombstone `__zwUnregFromSel`（part04:9379）→ 本地 insertBefore 命中 R5450 跨父
   移动臂（part04:10091 入口：tombstone≠undefined 且 ≠sel、`refNode.__zwSelector`
   存在、`!_zwLocalChildNodes`）→ AdjacentText wire（op6）。
3. **D3 copy 漏守卫**：copy 守卫 `_shP.__zwHandle||_shP.__zwSelector` 对 plain 节点
   双假 → InsertAdjacentHtml copy（op7）→ 双插。

## 3. 刀设计（最小三处编辑）

- **part04 两个 R5450 wire 臂打记**：insertBefore 臂（:10096 附近）与 appendChild
  变体臂（:10674 附近）wire 成功后 `newNode.__zwAdjTextWired = true`。
- **part01 D3 copy 守卫消费**：copy 臂守卫首位
  `if (_shP.__zwAdjTextWired === true) { _shP.__zwAdjTextWired = false; continue; }`
  （置于 handle/sel 守卫前，消费即清）。
- **判据选择**：`_zwSelPendingParent` slot-delta 不可靠（R97 臂 `noWire: true` 也
  置位）→ 改 wire 臂显式打记。fresh 文本（无 tombstone）不走 R5450 → copy 照旧
  （left-paragraph-and-right-line 363 案域保留）。无热路径入口成本（两次属性读写
  仅在 wire/copy 已发生的路径上）。

## 4. 复验（bs13 单页）

- 刀 D 干净二进制（/tmp/d58-knife-runner，release，构建 36.55s）：
  `join="<div>abcdef<p>ghi</p></div>"`（双插消失，bi=1 保持）——dump 页恒败属设计，
  账看 dump 内容。
- 修前对照：同页 d58-diag-runner `join="<div>abcdefdef<p>ghi</p></div>"`。

## 5. A/B（全套件电池，base＝d57 终版产物）

- **input-events**（/tmp/d58-suite-knife.json vs /tmp/d57v3-suite-knife.json，共键
  6667）：**F2P=10/P2F=0**——d57 的 20 键 P2F 中 **9 键翻 PP**（backspace corpus
  `abc<p>[]def<br>ghi</p>` 主形、non-collapsed ?Backspace/?Delete 同形、zz-iso-bs-13、
  zz-probe-b3p/b3q/b3r、zz-probe-d46b?Backspace/?Delete）＋1 键旧红 bonus 翻面
  （forwarddelete corpus `Delete at <div>abc[]<p>def<br>ghi</p>`，d56r base 起持续红，
  刀 D 的 R5450/copy 对称面顺手清偿）；**零键回归**。
- 对 d56r 真 base（/tmp/d56r-suite-base.json）直接对账（6667 vs 6664 键）：
  **F2P=817/P2F=11＝净 +806**（d57 同口径 816/20＝+796；本轮净增 +10＝9 join 键
  ＋1 bonus FD 键，双双直算验证）。
- **残余 11 键三簇**（d57/d58 失败消息逐字节相同＝刀 D 零抑制零形态变化）：
  ① **TypingA 路径双插 ×4**（d46b?TypingA ×2、non-collapsed?TypingA ×2；
  `<div>abcdefdef<p>adef</p></div>` 形）——insertText/typing 路径的**另一 copy 位**
  同病，刀 D 只守 delete-impl D3 copy，TypingA 路径刀待下轮；
  ② **bold 壳/bi=0 no-join ×5**（d56j T1/T2/T4、d56jt1 T1、zz-iso-bs-06；def 丢失
  `<div><b>abc</b></div>` 或 p 原样 no-join 形）——d56b ③ 派发鬼/无 join 族，
  非 copy 双插域；
  ③ **zz-diag-d56 T1/T2**（bi=0 事件计数族，d56b 在册）。
- **dom**（共键对账）：**F2P=0/P2F=0**（knife 57763 vs base 57864 键数差＝风暴页
  早停截断噪声，共键零翻面）。
- **uievents**：2400 共键 **F2P=0/P2F=0**。
- **selce**（contenteditable 过滤）：430 共键 **F2P=0/P2F=0**。

## 6. 哨兵（enginelib）

**4221P/0F**（exit=0，25.64s；d57 为 4214P——增量 7 测试＝并行流入树，非本刀面）。

