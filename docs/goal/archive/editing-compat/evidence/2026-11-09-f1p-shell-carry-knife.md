# F1' typed 壳继承 shell carry 刀（2026-11-09）

残 23F（[普查档](2026-11-09-f6b-post-census-52f-regroup.md) ④ 首位）：16 键蓝红双 span 族单一裸字 got + fontSize 中置 6 + fontsize(4) 单命令 1。

## 根因链

spec 忠实移植的 deleteSelection 把 caret collapse 到两块之间元素位（backward 方向交换语义），删除边界一侧的 inline 壳层信息丢失——后续 insertText 元素位 caret 裸插产裸文本 got（`<span 蓝>foo</span>A<span 红>bar</span>`），语料期望 typed 段并入左边界既有壳尾（`fooA` extend-in-place）。Chrome/Blink 以 TypingStyle 自删除边界推导并存续到后续打字；execCommand spec 无此概念，引擎侧扩展。同域根因先例 = F1'-B（D4 拆壳臂丢 armed tags → 裸字，[evidence](2026-11-08-f1p-b-typing-wrap-knife.md)）。

fontSize 中置 6 键 got 形 trace：值 override 重放祖先已满足跳过面——consume 后重放须产单壳（probe C17-C22 闸）。

## 刀体（commit 6cc1416c7，五 shim + bridge + fixture）

- **part01h.js 新建（186 行）**：`__zw_ce_capture_shell_carry` 四门捕获（① collapsed 块边界位——占位 br/块级伴随物放行 fd 带块尾 br 变体、词中 inline 内容即非边界 `_zwShellCarryAtBoundary` 逐层走查；② 左边界文本存在 `_zwShellCarryPrevText`；③ 壳链非空 `_zwShellCarryChainOf`；④ `<a>` 链 / off 表壳弃捕）+ `__zw_ce_shell_carry_consume`（caret 左边界链与 carry 链**逐节点同一** → typed 段 createTextNode appendChild 入 innermost 壳尾，consume 即清单发；carry 壳已脱离文档 → 弃账即清防误配）。存**节点引用链**非 tag 串：adjacent same-shell 不合并（`<b>a</b><b>b</b>` 字节不等 `<b>ab</b>`）。
- **part01f.js（+10）**：`__zw_ce_delete_apply` 捕获接线 + 替换式回写 `__zwTypingShellCarry`（非 collapsed 走 delta ⑤ tcap 互不相干；1985 行 <2000 ✓）。
- **part01.js（+28）**：inserttext 元素位 caret 两臂 consume 钩子（arm A target at===0 / arm B 裸插面；at>0 splice 自延不消费防双插）。
- **part06.js（+3）**：`_zwSync` 清除清单挂 `__zwTypingShellCarry`（script carry 同型，选区变更即清）。
- **js_dom_bridge.rs（+9）**：part01h 注册（挂 part01g 后同闭包复用 `_zwJus*`）。
- fixture `zz-probe-f1shell.html`（add -f 入库）：22 目标案（蓝红 16 语料原形逐字节 + fontSize 6 含 override 重放祖先满足单壳面）+ 5 护栏（G1 链空 / G2 词中 / G3 a 链 / G5 选区变更清账 / G7 fd 词中）。
- md5 锚（批硬绑定九点）：part01=`6108210b` part01c=`fb8aa0d8` part01d=`9fbab2af` part01f=`f5e149d2` part01h=`9592ebd3` part06=`452c4242` bridge=`f7d65f32` f1shell fixture=`9f9cfbe4` f6b fixture=`3cdf7835`。

probe 前置两轮：首轮 22/27（5 红 = 捕获门①把块尾 br 当词中内容，fd 带块尾 br 变体拒捕；G7 系断言笔误）；门语义改写（占位 br/块级伴随物放行——跨块合并消费面非词中内容）后二轮 **27/27 全绿**。

## 机制预测

- mt assertion 级 F2P = 恰 22（蓝红 16 + fontSize 中置 6）；fontsize(4) 单命令 1 键他机制留红；entry 级翻键窗口 [22,44]（双页折叠）。
- 残 52→30F。翻键 ⊆ 22 键签名白名单（`["styleWithCSS","false"],["delete"/"forwarddelete",...],["insertText",...]` × 蓝红双 span 8 markup 变体 + fontSize 5 组合 6 形）。

## 门禁批判读（batch-f1shell-gate-v1.sh，04:50:54 发射 head=6cc1416c7 → 05:46:35 ALL-DONE，56min）

锚复读 OK（md5 九点 + 特征串 12 项含 F6b 回归计数 walk_px=5/post_insert=5）；二进制身份（learning #28）：runner mtime 04:51=批首腿构建，`__zw_ce_capture_shell_carry`/`__zw_ce_shell_carry_consume`/`__zwTypingShellCarry`/`__zw_f6b_walk_px` 特征串 grep 全在；批中 mt 后刀体 md5 复读门过。

| 腿 | 结果 | 判读 |
|---|---|---|
| probe-f1shell | 27/0 rc=0 | 刀 fixture 全绿 |
| probe-f1b / f2 / f3 / f6b | 7/0 / 14/0 / 7/0 / 17/0 | 回归护栏四腿恒一 |
| probe-k11 | 224/17，rows 229 恒一 judge OK | 大探针零漂移 |
| ip | rows 7195 恒一，P 6989=6989 F 206=206 OK | 零漂移 |
| ihtml | rows 1565 恒一，P 1446=1446 F 119=119 OK | 零漂移 |
| exec×3 | rows 1806/1/6 恒一 OK×3 | 零漂移 |
| **mt judge** | **rows 10092 恒一，P 10040→10062 / F 52→30，F2P=22 P2F=0 post-only 0 lost 0 OK** | **机制预期恰值命中** |
| mt calibrate | entry 翻键 44=22 键×双页（窗口上沿），白名单 44/44（B=2×8 FB=2×8 S=2×6），**白名单外 0**；stay-red 78 族（F4 ilb 9 + F6a + createlink 链族） | 域外零翻转 |
| iol / iul | rows 1761/2065 恒一，F 2/4 恒一 OK | 零漂移 |
| enginelib | 3816P/7F（13.19s，rc=101 计数惯例），ROSTER OK(7-red 含 r2967 外部债) | 名册恒一 |
| battery 七腿 | 420/880/1104/552/3456/2607/1455 全 F=0 T=0 | ambient 零扰动 |

## 残账与下一步

mt 残 30F：F1' 残 1（fontsize(4) 单命令，他机制）+ F7 misc 12 + F4 ilb 9 + F6b handoff 4（N1/N2/N4/N5 backlog）+ F6a 2 + X37 2（除名）。DC-4 降序下一刀目标 = ⑤ **F7 misc 12 / F4 ilb 9 分治**。
