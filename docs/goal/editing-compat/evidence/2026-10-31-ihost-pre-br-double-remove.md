# inline-host pre `{}<br>` 簇收口：setupEditingHost 标记删除同 turn 双入账（R5340 臂 × D1 印章 accessor 互斥缺失）

**日期**: 2026-10-31
**基线**: R5356 收口后 inline-host 720P/160F（本流 ea6eeb33e 碰头仲裁轮）
**结果**: **780P/100F**——pre/pre-line/pre-wrap × enter/shift-enter 全 60F（`{}<br>` 家族 10/ ws/方法）翻绿；剩余 100F 全部 shift×normal（补断落点簇，未触碰），零位移。

## 1. 分簇与嫌疑修正

R5356 轮把 160F 概括为「单一机制=补断 br 落宿主外」，碰头补记已存疑分簇：shift×normal 100F（补断落点）+ enter×pre* 30F（引擎线 br 消失，嫌疑 part04 R5058 文本 insertBefore 退化）。本轮证实：

- pre 簇实为 **60F**（enter 30 + shift-enter 30，语料账逐行核对），非 30F；
- 根因**不是** R5058 位置丢失（R5058 只丢锚不删 br，t19 实测 br 存活），而是 **setupEditingHost 标记删除的同 turn 双入账**——br 被第二笔错误寻址的 remove 删除；
- R5356 对「30 enter + 130 shift-enter 补断落点」的归因，其中 enter 侧 30F 实为本簇（双删），修正入档。

## 2. 根因链（apply trace 实证）

语料 `setupEditingHost("{}<br>")`（editor-test-utils.js:184）：`host.innerHTML = "{}<br>"` 后**同步**扫描 marker 文本节点 `{}` 并 `node.remove()`——同一 JS turn 内完成，journal 未 drain。

该文本节点经 D1 stamping（part03 `_zwSelStampParseTree`）带 `__zwSelPathWrite = true`。同步 `node.remove()`（part03:21692）同时触发两条 wire 发射：

1. **R5340 臂**（part04:7818，sel 父 + pending 解析子移除路径）：发射 `__zw_remove_child_at(sel, 0)`——按**发射时**索引 0 寻址，删 T"{}"，宿主剩 `[BR]`；
2. **D1 印章 accessor 移除面**（part03:21700，`alive()` 为真）：发射 `__zw_remove_child_at_path(sel, path.concat(0))`——path 末位**也是 0**，但该 path 语义是「被移除子自身」；引擎 apply 时按发射序执行，第一笔已删 0 号位，第二笔再删 0 号位 = **同父 `<br>`** → 宿主清空。

ZW_APPLY_TRACE 实录（临时插桩，已剥离）：

```
[apply] RemoveChildAt { parent_selector: "span", child_index: 0 }      ← R5340 臂（R129AT-pv 标记命中）
[apply] RemoveChildAtPath { anchor_selector: "span", path: [0] }       ← D1 印章 accessor（无前置标记，同 turn 第二笔）
```

引擎树 `[BR]` → CE 行断链（`__zw_ce_paragraph_split_at` → `_zwCeLineBreakAt`）在空宿主插入 `T("\n")` + 补断第二 breaker → wire `[T("\n\n")]`，序列化 `\n\n`；语料期望 `[T("\n"), BR]` / `\n<br>`。全簇 60F 同型。

### 为何手工复现页此前的 t18-c 不现形

t18-c 在 `innerHTML =` 与 `remove()` 之间插了 `await tick()`：journal 已 drain，`alive()`（`__zwSelPathWrite && __zwSelPendingTops[sel] === entry`）为假 → 印章 accessor 跳过 path 发射，只剩单笔。语料同步删除不 drain，两笔齐发。**「手工页删 br 存活 vs 语料 br 消失」悖论由此解开**——差异不在删除方式，在删除与 journal drain 的时序。

### 互斥先例

part03 removeChild D1 臂（19587）对可入 path 账的子早有身份印排除：`!(c.__zwHandle || c.__zwSelector || c.__zwSelPathWrite)`，注释「重复入队会双删」。R5340 臂（后加）缺同款排除，属同一互斥约束的第二处消费点。

## 3. 修复（part04.js，+9/-1）

R5340 臂 wire 发射加印章排除，桶 retract 与通知簿记不动（与印无关）：

```js
if (_pvIdx >= 0) {
  if (!child.__zwSelPathWrite && typeof __zw_remove_child_at === 'function') {
    try { __zw_remove_child_at(sel, String(_pvIdx)); } catch (_ePvH) {}
  }
}
```

D1 印章子的移除由印章 accessor 自带移除面持有（path 末位=被移除子自身），R5340 臂按位寻址的 wire 发射让位；语义与 D1 臂身份印排除一致。

## 4. 验证

- **最小复现页**（zw-probe-ihost-pre-min.html，未跟踪）：span host display:block;white-space:pre，setupEditingHost("{}<br>") + sendEnterKey。修复后 wire=`[T("\n"), BR]`、ctr=`\n<br>`、hostIh=`\n<br>`——与语料期望 `<span contenteditable="">\n<br></span>` 逐字一致；修复前 wire=`[T("\n\n")]`。担心的补断过发（多一 `\n`）未出现：br 存活后 `_zwCeLineBreakAt` 补断 `brk3.nextSibling` 非空，不触发。
- **探针页 t17/t18/t19**（zw-probe-ihost-lb.html，未跟踪）：t17 改造为 drain 后逐基元读线（`setTimeout(0)` tick），t18 setup 四分解案，t19 手工 CE 行断节奏对照——定位「线读 stale」与「t18-c 悖论」两处方法学坑。
- **全量语料**：inline-host **780P/100F**（修复前 720P/160F），F→P 60 / P→F 0；剩余 100F 逐行核对**全部** shift×normal（补断落点簇，R5356 已定性未触碰）。
- **位移基线**（修复二进制逐文件分跑）：见下节终账。

## 5. 门禁终账

修复二进制（part04.js 唯一 diff，+9/-1，插桩剥离后复验）逐文件分跑：

| 语料 | 修复前账 | 修复后账 | 判定 |
|---|---|---|---|
| inline-host | 720P/160F | **780P/100F** | F→P 60 / P→F 0，零位移 |
| join paragraphs | 3456P/0F | 3456P/0F | 恒等 |
| join lrlp | 2607P/0F | 2607P/0F | 恒等 |
| join lprl | 1455P/0F | 1455P/0F | 恒等 |
| cannot-have-div | 420P/0F | 420P/0F | 恒等 |
| white-space-style | 616P/488F | 616P/488F | 恒等 |
| insertlinebreak-ws | 0P/552F | 0P/552F | 恒等（首录账保持） |

方法学注：位移核对逐文件分跑（多文件合跑静默截断教训）；`insertparagraph-in-child-of-head.tentative.html` 不在 runner manifest（直跑零输出 EXIT=1），基线账 420P/0F 对应案为 `insertparagraph-in-editing-host-cannot-have-div.tentative.html`，此前账目标签「chd」易误读，已在本节更正。

engine lib **3614P/10F** 已知红恒等（10 红全为 fetch 契约簇 + s24 渲染流债，无编辑面红）；webview lib **825P/0F** 恒等。

## 6. 剩余

inline-host 100F = shift×normal 补断落点簇（`<br>` 落宿主 span 之后 container 子级，EXP 内部 `</span>` 前）。属 insertLineBreak 落位锚点语义，下一轮目标。探针两页保持未跟踪。
