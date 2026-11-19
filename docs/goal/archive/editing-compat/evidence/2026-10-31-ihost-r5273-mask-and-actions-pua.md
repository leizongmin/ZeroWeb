# inline-host 断行主簇独立归因撞车仲裁——采纳 R5356，本流同面机制撤改零残留 + 独立佐证与增量诊断

日期：2026-10-31。语料：`editing/other/insertparagraph-in-inline-editing-host.tentative.html`（880 测）。

## 结论

本流自同基线 170P/710F **独立归因出与 R5356 相同的两根因并独立实现修复，终点恒等 720P/160F**；rebase 撞车按「消费域收敛、不回退对方修复面」采纳先落地对方版本（3b25bb2c7 引擎 + 78601a924 runner，账见
[evidence/2026-10-31-r5356-stale-seed-and-actions-modifier.md](2026-10-31-r5356-stale-seed-and-actions-modifier.md)），本流同面机制撤改零残留。本文保留独立归因链与对方账面未覆盖的增量诊断。

## 独立归因链（与 R5356 同判、路径不同）

- **根因一（R5273 登记遮罩）**：本流探针 t12（`zw-probe-ihost-lb.html`，不入库）链路解剖证明 rust 读链（`__zw_get_inner_html`）各时点全对、仅 getter 融合序列化侧陈旧——排除快照/基座缓存/R55 重登记三个假设后落到 `_zwTextElsBySel` 注册节点遮罩吞 apply 后兄弟。R5356 归因链（R48 原地清空注册节点 data、引擎快照 `[br, T:abcd]` 正确）与此同判互补：两侧共同钉死「引擎树为真相、融合序列化吐过时种子」。
- **根因二（Actions 臂 PUA 插入）**：本流自 editor-test-utils.js `sendKey` Actions 路径 + runner `keydown|keyup` 臂无修饰键护栏定位，护栏修为 `return None`（纯抑制）。R5356 修为键名映射 + 修饰键步早退，为本流护栏的**严格超集**（连事件键名 PUA 原文问题一并解决）。

## 撤改面清单（零残留）

| 本流文件 | 本流机制 | 处置 |
| --- | --- | --- |
| part05.js `_zwOverlayPendingChildNodes` | 跨代遮罩分叉：换代后仅基座首位文本子内容相等才替换、否则退役（live 视图也适用） | 撤改，取 R5356 序列化 walk 单门 |
| part06.js `_zwRegisterTextEl` | 登记项 `gen` 代际戳 | 撤改（R5356 同构已有） |
| testharness.rs keydown 臂 | 修饰键 PUA 插入抑制 `return None` | 撤改（R5356 键名映射+早退覆盖） |

**语义分叉留档**：live 视图换代后本流臂曾按内容相等退役遮罩读基座；R5356 判 live 种子恒保留为真值（R5273 原语义，仅序列化 walk 放行）。两版对语料终点无差（720P 恒等），取对方；后续若 live 消费面出过时种子反例，本流相等护栏是已验证候选臂。

## 独立佐证与增量诊断（对方账面未覆盖）

1. **A 簇（normal×shift-enter 100F）逐测真实性**：语料夹具清理在 `finally`（157/200/244 行等），断言失败也清场 → 宿主外 `<br>` **非页内级联**，是每测自身产出；探针 t16 新页 shift 路径正确 → 缺陷与页内状态/落位分叉相关，与 R5356「可见性补断 br 落宿主之后」定性相容（落位锚点语义）。
2. **enter×pre* 30F 机制存疑**：本流 t13/t15 探针示 pre `{}<br>` 操作后**引擎线** `[T("\n\n")]`——既有 `<br>` 引擎侧消失（非「落宿主外」，是「不在线内」）；头号嫌疑 part04.js R5058（~8935 行）`insertBefore(文本节点, 元素引用)` 退化为 `__zw_insert_adjacent_text(sel,'afterbegin'|'beforeend',text)` 丢锚点 + 即时线/pending journal 时序。「160F 单一机制」概括对这 30F 存疑，落位锚点归因时须分簇验证。
3. **新增分母首录**：`insertlinebreak-with-white-space-style.tentative.html` **0P/552F**（insertLineBreak 语义族，块级 div 宿主；合并树复跑数见主账）。
4. **runner 工具坑**：`testharness-editing-other` 多文件调用**只跑首个文件**、其余静默零输出退出码 0——本流 join 三案合跑 3456P 对账面 7,518 差 4,062 才暴露；多案位移核对须逐文件分跑（3456+2607+1455=7,518 恒等复核）。

## 下轮入口

并入 R5356 下一步：① 160F 落位锚点归因（**分簇**：shift×normal 100F 补断落点 + enter×pre* 30F R5058 线索）→ ② ilb 552F（同 insertLineBreak 语义族）→ ③ white-space-style 488F → ④ part04 BUILD void 70F → ⑤ renderer R383a 对账、簇 A glue、execCommand 簇序。
