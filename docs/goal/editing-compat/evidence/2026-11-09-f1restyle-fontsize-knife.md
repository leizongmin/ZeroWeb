# F1' fontsize armed 壳原位 restyle 刀（2026-11-09，⑧' lane9 残键收口）

## 立项

F6a 收口后 mt 键级残 3F 唯一本域键（X37×2 除名维持）：

- multitest :9780/:19877 双页：`[["stylewithcss","false"],["fontsize","4"],["insertText","a"]]`
  on `<font size=7>{}<br></font>`，期望 assert_in_array
  `["<font size=\"4\">a<br></font>", "<font size=\"4\">a</font>"]`，
  实得 `<font size="4">a</font><font size="7"><br></font>`（两壳分裂）。
- Chrome 录音语义（tests.js :2840 TypingStyle 先例）：fontsize collapsed armed
  后打字，壳复用原位改档 size 7→4，typed 段入壳首，占位 br 留壳内。

## 三版定位（v1/v2 未命中 → 诊断探针定谳 → v3 落地）

**v1**（臂门 `/^[1-7]$/` 直测 override）：探针 2091P/1F got 形不变。根因：
override 记录侧存 **keyword 形**——`__zw_ce_value_apply` fontsize 臂
`_zwVaNormalizeFontSize('4')` → `'large'` 进 `_zwVaSetSelectionValue`，塌缩
无 contained formattable 臂 `_zwVaSetValueOverride('fontsize','large')` 直存
（part01d :2083/:1701）。快照导出原始形，`^[1-7]$` 门恒拒。

**v2**（补读侧同款 `_zwVaLegacyValueOf` 归一，'large'→'4'）：探针仍
2091P/1F got 形不变——格式门非唯一断点。

**诊断探针 zz-probe-f1dbg**（临时件三前提直读，已随批撤除）：
- D1 `__zw_ce_value_override_snapshot()` = `{"fontsize":"large"}`（快照
  keyword 形实证）+ qV 读侧 '4'（legacy 换算闭环）；
- **D2 caret_range → `(font@0)`**——caret 居 armed 壳**内**元素位 0，非
  「caret 提升出壳到 host 级」假设；got 的 host 级分裂形是值 override 重放
  「异桶壳外重放」（F6bN C/N 族先例语义，size 7≠4 异桶）产物；
- 由此定谳 v1/v2 的 `kids[idx]` 臂取到 `kids[0]=br` 判 font 恒假 → 臂永不
  命中。几何假设必须实证，机制注里的「提升出壳」是对 got 的误读。

**v3**（签名改 `(sc, so, ovSnap, ins)`——sc 本身即 armed 壳门）：门 =
so=0 ∧ sc.tagName font ∧ size attr 纯数字档 ≠ override 归一档 ∧ 壳子恰空
或单 br → 原位 `setAttribute('size', ov)` + typed 段裸插壳首。重放等值门
读 restyled 壳指定形（font[size=4] → kwMap 'large' ≡ override 'large'）
零再裹，异桶壳外重放径自然不触；壳内 br 无 `__zwPlaceholderBr` 标记不被
占位消费扫描摘除（:10298 只认标记）＝期望首形 br 留壳内。

## 刀体

`crates/engine/src/js_dom_shim/part01.js` 两处（commit 41ab66909，+62）：

1. `_zwCeF1RestyleShell`（`_zwCeNormalizeHexColorShells` 之后）：上述门链
   + restyle + 插入，返回新文本节点。
2. `__zw_ce_inserttext_mutate` 元素位 car3-miss else 分支调用
   （`!swAbs` 门——swcss(true) CSS 模式语义不入本臂），命中即 node=rst3
   走共享 caretOff/重放尾，未命中原裸插零回退面。

相对 +n/-n 壳档不入（`^[0-9]+$` 纯数字门防 `parseInt("+1")=1` 折档误判，
part01d specifiedValue 相对臂同坑先例）；含文壳不入（内容门恰空/单 br，
Chrome 走 split 语义，语料无此形）。

## 批前预验（release 二进制直验，2026-11-09 21:3x-21:4x）

- zz-probe-f1dbg 3P/0F（v3 三前提直验：终态 `<font size="4">a<br></font>`）。
- zz-probe-f6bwin **2092P/0F**——刀后 probe 首次全绿（刀前 2091P/1F）。
- zz-probe-f6b 17/0 + zz-probe-knife11-v2 224/17 judge 零漂移 VERDICT OK。

## 门禁批（batch-f1restyle-gate-v1，锚 41ab66909）

三射小账：v1 发射两连 ABORT 均脚本缺陷（P01 预写 v2 旧 md5 未随 v3 重算
同步——锚检查零腿运行拦截，无判读域污染），第三射 21:48:19 开跑
**ALL-DONE 22:58:27 全绿一次过**。

判读域（vs F6a 终态基线）：

- probe：f6bwin **2092P/0F**（F1' 翻绿）+ f6b 17/0 + f7m 149/0 + f7 9/0 +
  k11 224/17 judge OK（fontsize 家族回归面零漂移）。
- 语料腿零漂移：ilb / ip / ihtml / exec×3 / iol 1759 / iul 2061 判读
  F2P 0 P2F 0 post-only 0 lost 0 全 VERDICT OK。
- mt：20180P/4F → **20182P/2F**（assert 级）；残红独立核验恰 X37 双行
  （subscript+formatblock qS/qV，?2001-3000 窗）；judge 键空间 F2P 1
  （F1' 双页 2 行去重 1 键=机制预期）P2F 0 postF 0 lost 0 VERDICT OK。
- calibrate：行级 flip 恰 2 全落 fontsize 白名单（白名单外 0）。
- enginelib：failed total=7 + 名册逐名恒一（6 先例 + r2967 外部债）双门。
- battery 七腿 420/880/1104/552/3456/2607/1455 全 F=0 T=0。
- binary sig 六串 + 刀串 `_zwCeF1RestyleShell` 计数 4=源码。

**判定：DC-4 红线全过，F1' 键收口——mt assert 级 4→2F、键级 3→1F（残
=X37×2 除名维持），F1' 簇全程 23→1→0 清账。**

## 教训

1. **几何假设必须实证**：连续两版未命中的根因是机制面误读（把 got 的
   重放产物形态当成了 caret 位置证据）。诊断探针（快照 JSON/caret_range
   几何/终态三前提直读）一轮定谳，先于第三版改刀——疑似机制假设先造
   最小直验件，不靠 got 反推。
2. **override 记录/读侧异形**：fontsize 值 override 记录侧存 keyword
   （normalize 后进流程），读侧 qV 才做 legacy 换算——直接消费快照原值
   的路径必须过 `_zwVaLegacyValueOf` 归一，不能假设存储形。
3. **untracked 诊断件在受管语料目录会被 fetch-wpt-editing 同步清掉**
   （zz-probe-f1dbg 随批消失）——临时诊断件不留库是纪律也是自保护；
   常驻探针（f6bwin 等）必须入库才在语料同步下存活。
4. **批脚本 md5 锚必须在 commit 后当场从现树重取**（F6a 三射教训同族
   复发：草稿期预写值未随终版重算）。
