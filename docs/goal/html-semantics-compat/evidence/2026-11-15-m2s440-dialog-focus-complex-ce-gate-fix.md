# M2 s440 — dialog focus-after-close「Complex dialog usage」五窗红相归因定谳 + CE 域门修复（1 键 F→P 零位移）

日期：2026-11-15（凌晨，跨 11-14 深夜窗）。前情：s437-s439 三切片 dialog
phase-flip 挂账——`focus-after-close.html :: Focus should be moved to the
previously focused element (Complex dialog usage)` 自 11-14 14:29 绿窗后
五窗连续红（s437 双跑 → s438 双跑 → s439 双跑 → 本轮 s440 双跑 + solo×4），
s439 承诺「风暴平息后下窗仍红则重开归因面」。本轮机器重启后低负载窗
（loadavg 1.8）solo 仍红 → 负载假设证伪，重开归因面，**定谳代码回归 +
修复**。

## 归因链（bisect 三重）

**判据**：solo `FILTER=interactive-elements/the-dialog-element/focus-after-close`
红 iff Complex 键 Fail（测试文件在册：light DOM 单 `<input>`，promise_test
Complex 变体内 `querySelector("input")` → `input.focus()`）。

| commit | 结果 | 备注 |
|---|---|---|
| 2de501560（HEAD/main） | 红 | 双跑 join 双腿 F + solo |
| 6f77f0cd1（s439 收口树，代码恒等） | 红 | 双跑 join 复现 |
| **266b45602（M3-A insertParagraph 拆分刀）** | **红 ×3** | 同签名 `TypeError: Cannot read properties of null (reading 'focus')` |
| 36de32a4d（父） | **绿** | 锚绿 |
| 266b45602 + AB-KILL（禁用两处位置章重算块） | **绿** | hunk 级因果钉死 |

**first bad = 266b45602**（2026-11-14 13:25，editing-compat M3-A 根层裸文
本 insertParagraph 拆分）。14:29 绿窗 = build 早于该刀到货（13:57 push 后
即时落地 + 渲染流 R5531 补验 14:16 记录在案）；15:21 起（25fdd6e45 solo×3）
全红——时窗吻合，非负载相位。

## 失败机制（v4 探针实证）

复现页（语料内临时插桩，验后已删）四版迭代定谳：

1. 失败形态非断言失败，是 **promise_test unhandled rejection**
   `TypeError: Cannot read properties of null (reading 'focus')`；
2. v2 诊断：null 时刻 = **complex(false) 变体开头**（complex(true) 全程过）；
3. v3/v4 交叉验证：null 时刻 `body.children = [INPUT, DIALOG, SCRIPT]`
   （children 枚举通道 input 在档）但 `document.querySelector('input')` 与
   `body.querySelector('input')` 双双 null，而
   `getElementsByTagName('input').length = 2`——**host 侧元素集合孤儿双现**
   （残段 + 重挂副本），查询通道对双现节点歧义静默 null；children 枚举走
   JS 本地视图（即时更新）故仍见 3 子。

**根因**：266b45602 给 part04 摘挂臂（R125 removeChild / appendChild
move-in `_mv`）新增「位置章 `… > tag:nth-child(k)` 发射前重算 + 回写」，
动机是修 ip 面「同批前序摘挂致 apply 时章值漂移 → Remove miss → 双现」
（insertParagraph 场景实证）。但重算臂是**所有页面 DOM 摘挂的通用路径**：
普通页面（非 CE）交叠批次下（td 命令与摘挂 wire 穿插 drain、promise_test
await 间隙执行同步 test()——本文件 Complex 是唯一 promise_test，其 await
窗口恰被 shadow-root/body_if_fails 的 remove+insertBefore 序列插入）重算
模拟的「apply 时树态」与 host 实际 drain 交互失准 → Remove 错位/miss →
孤儿双现 → querySelector 通道 null → Complex(false) `input.focus()` 崩。
时序敏感面（交叠窗口 = 泵节奏函数）解释 s437 时代「同树跨时窗翻转」的
phase-flip 表象——**实为 13:57 到货后恒红**（14:29 绿窗是 build 未含毒刀
的最后窗口），负载只是当时误归因方向。

## 修复（CE 域门，两处 +4 条件）

重算只在摘挂对象位于 contenteditable 宿主子树内时启用——
`child.nodeType === 1 && globalThis.__zw_is_ce_host(child)`（复用 part01
既有 helper，沿 parentNode 链 inherit 解析）：

- part04 R125 臂：外层条件加三门（`_r125Wire` 发射章保持 = `_r125Sel`
  缓章当 CE 外）；`_zwMarkRemoved` 线不动。
- part04 `_mv` appendChild move-in 臂：同款三门（`_mvWire` 同理）。
- 文本 child 构造性无行为差（原块内 tag 门已使章值不替换，wire = 原章）。

**等值性论证**：CE 宿主内摘挂 = 266b45602 后行为逐位恒等（门恒开）→
editing ip A/B 族 37 键修复保持；CE 外 = 266b45602 之父（36de32a4d =
s433 基线带）行为 → 全域绿面回位。

## 验证

| 检查 | 结果 |
|---|---|
| solo focus-after-close | **7/7 全 Pass**（修复前 Complex Fail ×5 窗） |
| solo dialog 目录 | 16F（修复前基带 17F，净 −1 恰靶零新增） |
| 全域双跑 join | **8065P/254F/7T/1U vs 修复前同口径 8064P/255F/7T/1U——恰 1 键 F→P 零其他位移** |
| 双跑 vs 基带逐键差 | 唯一差异 = 目标键 Fail→Pass |
| `make test`（workspace） | **10798P/1F**；唯一 F = image_batch_e2e zero-browser bin 被 target-disk-guard 清理（s116 在册环境形态，panic 位 image_batch_e2e.rs:233 与 s437 记录同款，非行为失败） |
| `cargo fmt --all -- --check` | 绿 |
| `make guarded-clippy` | exit 0 |
| shim 拼装（part01-part06 主序 139355 行）node --check | SYNTAX_OK |
| editing ip 面 | 构造性等值（CE 内门恒开）+ part114 探针钉测在 make test 全量内绿 |

工件：/tmp/s440/{sweep-1,sweep-2}.log（HEAD 修复前双腿）、
sweep-s439base{,2}.log（s439 收口树双腿，代码恒等复现同态）、
fix-sweep-{1,2}.log（修复后双腿）、extract.sh/join.sh、*.ks/*.sw、
make-test.log、clippy.log。

## 全域记账两件（本轮发现，挂账下轮）

1. **键面记账口径失真**：文档 s433-s439 记录的 stable 带（8307-8309P/
   199-203F/2-3T）与今晨同口径（extract.sh + join.sh）双腿可复现面
   （**8064P/255F/7T/1U**，s439 收口树与 HEAD 恒等复现四腿逐键一致）
   不吻合（差 P−243/F+55/T+4/U+1）。join 数学（stable ≤ 单腿 8323）排除
   8510 形态来自同口径 join——历史记录口径或 storm 窗 runner 形态待考。
   **本轮起基带切换为可复现口径**：8065P/254F/7T/1U（含本刀）。
2. **relevant-mutations.html 46F 大块红**：stable-F 大头（s126 时代曾
   全 Pass）+ zzz-s433-probe-N7-localName.html 探针残留语料（stable-T
   在册）等 red-face 重盘点，下轮红池分诊续。

## 结论

s440 收口：dialog 五窗红相 = 266b45602 摘挂章重算毒化普通 DOM 摘挂路径
（孤儿双现 → 查询歧义 null），CE 域门修复 1 键 F→P 全域零位移收口；
modulepreload strict-origin 双腿 Pass 回位 ✓（s439 沾染洗清）。
挂账：键面口径切换（8065/254/7/1U 新基带）+ relevant-mutations 46F 红池
续分诊 + 探针残留清理。
