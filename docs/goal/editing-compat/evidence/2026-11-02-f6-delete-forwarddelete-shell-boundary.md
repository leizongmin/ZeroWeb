# F6 delete/forwardDelete 撤壳边界（part01f deleteSelection 忠实移植 + 两笔 Chrome delta）

日期：2026-11-02（editing 流，控制面下一步 ①，基于 g16 基线对账）

## 0. 背景与范围

普查轮定谳 D1：multitest delete/forwarddelete 失败族主机制——flat 模型
（R3254-M3 切片 2）只做同文本节点 splice，跨容器选区 defer、无 strip-wrappers、
无块合并、无 override 重锚。multitest D1 ~100F + inserttext/ipara 共享面零散。

## 1. 实现（part01f 新分片 975 行 + part01d 两笔 + part06 调用点置换）

- **part01f 新分片**（挂 part01e 后）：vendored `deleteSelection`（:4550-5058）
  + `delete` action（:6193-6815）+ `forwarddelete` action（:6815-7052）忠实移植，
  含 getPosition（BpCompare 三值）、first/last equivalent point、canonical space
  sequence、isCollapsedBlockProp、getBlockNodeOf、inclusiveAncestors、
  recordValues/restoreValues（9 命令表）、recordCurrentStatesAndValues
  （activatedValues 布尔表 + 4 值命令走 `__zw_ce_value_query`）。入口
  `globalThis.__zw_ce_delete_apply(forward)`，part06 调用点置换 flat
  `_zwExecCmdApplyDelete`（已删）。 FIXME 臂：delete action 的 li/dt/dd
  first-child 分支（需 normalizeSublists）与 indentation-element 分支（需
  outdentNode）；restoreStatesAndValues 非 collapsed 恢复臂。
- **Chrome delta ①——strip-wrappers 幸存壳**（g16 语料逐案重推实证）：
  - 幸存壳 = 原起点容器行内链上**最外层** `<a>`（内→外走、后见者胜），无
    `<a>` 则最外层链包裹。证据：`foo<b>[bar</b>baz]`→`foo<b>a</b>`；
    `foo<u><a>[bar</a></u>baz]`→`foo<a>a</a>`（a 胜 u）；
    `foo<a><font color=blue>[bar</font></a>baz]`→`foo<a>a</a>`；
    `foo<sub><font size=2>[bar]</font></sub>baz`→`foo<sub>a</sub>baz`。
  - 幸存壳计长 0，其上方仅含它的包裹仍按空摘（`u>a` 链：a 留、u 摘）。
  - **包含**关系里的 `<a>` 整体摘除不留壳：`[foo<a>bar]</a>baz`→`abaz`。
  - 实现：deleteSelection 预扫 delKeepShell + strip 循环
    `parent_ !== delKeepShell` 才摘。
- **Chrome delta ②——preservesOverrides 包裹**（vendored myExecCommand :8495
  等价面）：part01d 新增 `_zwVaOverridesTakeRaw`/`_zwVaOverridesPutRaw`（state+
  value 双仓浅拷贝，不经 staleness reset）+ `_zwVaReanchorOverrides`（stored
  range 重锚当前活动选区）；`__zw_ce_delete_apply` 动作前抓快照、动作后重锚并
  整体回写。归因：本仓惰性 staleness 模型下，删除动作内部的 override 读
  （值查询/effective value 面）在选区位移后触发 reset 误清存储；g17 无修复批
  实证 bold qS-after-delete 期望 true 得 false（~40 断言族）、fontsize qV-after
  期望 "4" 得 "3"。语料方向定谳：Chrome **保留** typing override 跨 delete
  （非清除）。
- **Chrome delta ④——typing 包裹态快照回写**（g18 复验批归因）：Fix A 只复
  值命令族（backcolor/fontname/fontsize/forecolor/hilitecolor qV-after 全
  复），状态命令族（bold/italic/strikethrough/sub/sup/underline qS-after ~32
  断言）仍红——qS 读面是 DOM 包裹态 + `__zwTypingTags` typing 兜底
  （part06 `_zwQueryFormatState`），载体是**第三仓**：本仓删除动作内 caret
  落位走 Selection 公共面（collapse/extend/collapseToStart），`_zwSync` 对
  显式 selection 变更清 `__zwTypingTags`（part06 :10355，防跨子测泄漏语义）；
  vendored/Chrome delete 不清 typing style。修复：`__zw_ce_delete_apply` 对
  tags 同面快照回写（引用还原，防跨子测泄漏的 setup 显式 selection 清除语义
  不受影响）。
- **Chrome delta ③——幸存壳即占位**：blockOfParent 空块 br 占位臂加
  `parent_ !== delKeepShell` 门——`<blockquote><font color=blue>[foo]</font>
  </blockquote>` delete+inserttext 期望壳保留不补 br（g17 实证补 br 产出
  `<font><br></font>` 双占位）。
- Vendored 怪癖照搬：空 endBlock climb `parent_.removeChild(endBlock); endBlock
  = parent_;`（到达即 throw，语义保持）；forwarddelete combining marks 正则
  `/^[̀-֑ͯ-ֽׁׂ]$/`。

## 2. g17 无修复批归因（multitest 9,538P/554F → 9,535P/557F，P→F 50）

comm -13 f16 f17 全量聚类：

| 簇 | 数量 | 机制 | 处置 |
| --- | --- | --- | --- |
| override 误清（qS/qV-after-delete + compare 双生） | ~45 | 动作内 override 读触发 staleness reset 误清 | Fix A（快照回写） |
| blockquote 壳双占位 | 1 | br 占位臂不打幸存壳门 | Fix B |
| CE-false span caret 误入 | 4 | canonicalizeWhitespace 下钻臂入 contenteditable=false span（vendored 同病，flat 恰好躲过） | 挂账 TypingStyle 轮 |
| 全段 `<b><i>[def]</i></b>` 双壳 | 2 | 期望 b+i 双壳都留，与 sub+font 最外层单壳模型冲突 | 挂账 TypingStyle 轮 |

F→P 49（g16 失败族）：keep-shell 面 + 跨容器 deleteSelection + 块合并主簇。

## 3. 门禁（gate17/g18/g19 三批串行，systemd user unit + test-guard）

三批同面集（multitest 主信号 → inserttext/ipara/forecolor → 恒等电池 →
iloop/illist → engine lib），分别对应移植未修 / Fix A+B / Fix A+B+C：

- **g17（移植未修）**：multitest 9,538P/554F→9,535P/557F（P→F 50，§2 聚类）；
  inserttext 2,779P/230F 恒等。
- **g18（Fix A overrides 快照回写 + Fix B 幸存壳 br 门）**：multitest
  9,538P/527F——值命令族全复（backcolor/fontname/fontsize/forecolor/hilitecolor
  qV-after + compare 双生），blockquote 壳双占位复，table/wss 形复；状态命令
  族（6 命令 × qS-after/compare ~32 断言）仍红 → 归因 delta ④。批中断
  （面集被 g19 取代；分母 10,065 较全额 10,092 少 27 断言，g19 全额恢复）。
- **g19（+ Fix C typing 包裹态快照回写，终批）**：

| 面 | g16 基线 | g19 | 位移 |
| --- | --- | --- | --- |
| multitest | 9,538P/554F | **9,590P/502F** | F→P 59、P→F 7（全数挂账族） |
| inserttext | 2,779P/230F | 2,779P/230F | 恒等（fail 集逐行 comm 空） |
| ipara | 6,728P/467F | 6,728P/467F | 恒等 |
| forecolor | 2,279P/48F | 2,279P/48F | 恒等 |
| wss-ip / wss-ilb | 1,104P/0F / 552P/0F | 同 | 恒等 |
| run-ilb | 1,641P/50F | 1,641P/50F | 恒等 |
| join 三案 | 3,456/2,607/1,455P/0F | 同 | 恒等 |
| chd / ipihost | 420P/0F / 880P/0F | 同 | 恒等 |
| void 双语料 | 72+48P/0F | 同 | 恒等 |
| iloop / illist | 1,619P/142F / 1,910P/155F | 同 | 恒等（F1 面保持） |
| engine lib | 3,684P/0F | 3,684P/0F rc=0 | 恒等 |

multitest P→F 7 逐条（全数挂账，零意外回归）：CE-false span ×4（`<b><span
contenteditable=false>abc</span>X[]</b><i>def</i>` 双臂 + forwarddelete 双臂）、
全段 `<b><i>[def]</i></b>` 双壳 ×2、styleWithCSS-false delete+ipara qS(bold)
×1（caret 在 kept `<b>` 内的 ipara 拆分链，无 tags 参与，非 Fix C 面）。
multitest F→P 59：keep-shell 主簇 + 跨容器 deleteSelection + 块合并 +
overrides 双仓（值+状态）+ blockquote 壳占位 + table td caret 形。


## 4. 残余与挂账

- CE-false span ×4 + 全段 `<b><i>` 双壳 ×2 + styleWithCSS-ipara qS ×1
  （multitest P→F 7 全数）→ TypingStyle 轮挂账（与 F5 typing 尾 br 同轮；
  前两者 vendored 忠实 = Chrome 错位，flat 模型恰好撞中，修复需 typing 侧
  重挂语义非 delete 侧）。
- delete action FIXME 臂（li/dt/dd first-child + indentation element）→
  list FIXME 轮。
- 恢复臂（非 collapsed restoreStatesAndValues）当前调用域全 collapsed，
  暂无失败面。
