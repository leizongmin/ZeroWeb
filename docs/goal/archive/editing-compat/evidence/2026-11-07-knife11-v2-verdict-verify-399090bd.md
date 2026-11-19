# knife11 v2 终判核验与 ip 腿 P2F×2 归因（核验方 399090bd，2026-11-07 16:20）

对 `batch-knife11-v2-verdict.log`（15:23 起飞 head=c700bfb2a，16:12:01 ALL-DONE）的独立核验。mt 红线全过；ip 单文件腿 P2F=2 为 rx-c 表格宿主面回归（归因与修向见 §3）；残账 7 键已具名（§2）。

## 1. 四腿核验

| 腿 | 结果 | 核验 |
|---|---|---|
| mt 全分母 | rows 10092 恒一，F2P 18 / P2F 0 / post-only 0 / lost 0，F 143→125 | **VERDICT OK** ✓（raw 19936P/248F = +36/−36 = 18 键 × 2 页）✓ |
| insertparagraph 单文件 | rows 7195 恒一，**P 6987→6985，F 208→210（P2F=2）** | **VERDICT REVIEW**——非窗口伪影（两键逐行双 log 核实），真回归，见 §3 |
| iol / iul 单文件 | rows 1761/2065 恒一，P/F 逐键恒等 | VERDICT OK ✓ |
| enginelib | rc=101，3794P / 6F | 6-red 名册恒一（SMIL 外债档）✓ |

## 2. F2P=18 键级账（vs 预测 25）

- **16 compare 键**全落 styled 壳 typed-fate 族：`<b><i>def</i></b>` 主形（含 div 包裹 / `<s></s>` void 前缀 / img 变体 / `{}` 与 `[]` 边界形）+ 纯 `<b>` 壳形 3 键——处方 (a)(b) 面全数兑现。
- **2 qS-after 键**翻绿：`styleWithCSS false + ip + inserttext` 的 `queryCommandState("bold") after`（`<div><b>[abc]</b></div>` 与 `<div>abc<b>[def]</b></div>` 双形）——typing 存续回写（④）兑现。
- **残 7 compare 键具名**：A 簇 2（`abc[<b>def]</b>ghi` / `abc{<b>def</b>}ghi` 边界形）；B 簇 2（`styleWithCSS false + delete + ip + inserttext`，delete 前导链）；C 簇 2（`styleWithCSS false + ip + inserttext` 无 delete，div 块主语）；D 1（`createlink + ip + inserttext a` foo[]bar，knife11 残账原键）。B/C 簇形（显式 styleWithCSS false + div 块主语）与 knife11 styled 组 DC-4 语料跨会话不一致签名同形，先做泄漏/语料对账再定读面；A 簇与 D 为引擎可修残面。

## 3. ip 腿 P2F×2 归因（本会话独立定谳）

两键均为 **table 宿主 + 显式 `defaultparagraphseparator "p"` + 非 collapsed 跨格选区**（`<table><tr><td>[foo<td>bar]<tr>…` 与 `<table><tr><td>[foo]</table>`，compare innerHTML）。

**排除链**：两键链中无 inserttext 步 ⇒ delta ④/⑤（typing 快照/捕获，仅影响动作后 typed 态）对其 innerHTML 无作用面 ⇒ 唯一改变非 collapsed 拆分输出的是 **处方 c**：wrap 臂两处 `createElement(sep)` → `createElement(_zwIpWrapTag || sep)`，而 `_zwIpWrapTag = _zwJusTag(_zwListEditingHostOf(startContainer))`（part01f 新 +3 行）——list 向宿主查找在 **td 宿主**上解析出非 `p` 标签（或 null→'div' 兜底），语料期望（Chrome 录音）为 **separator 面 `<p>`**。基线（knife11 post）两键 P ⇔ 旧代码恒用 sep=p ✓。

**修向（最小刀）**：`_zwIpWrapTag` 解析加表宿主守卫——`_zwListEditingHostOf` 命中 td/table 时置 null（回落 `sep` separator 面），或 `_zwJusTag` 对 td/table 返回 null；只动 rx-c 解析面，④/⑤/rx-b 不涉。修复后 ip 腿两键应复位 P，mt 分母（multitest 无显式 separator 表案）零影响预期。

## 4. 结论

**mt 红线 VERDICT OK 成立**；v2 刀（④/⑤/rx-b）兑现面 18 键 + qS 双键无异议。**ip 腿 P2F×2 = rx-c 表格宿主面，非窗口伪影、非 typing 臂**——建议 claim 轮按 §3 修向后补最小守卫并复跑 ip 腿（mt 无需重跑，修向不触 mt 分母面）；REVIEW 闭账后 v2 可落地。
