# F1' shell carry 刀收口后残 30F 普查（2026-11-09，⑤ 开工前置分簇定账）

④ 刀（6cc1416c7，[收口档](2026-11-09-f1p-shell-carry-knife.md)）后的 mt 残键重普查。
零源码改动轮。目的：DC-4 降序定 ⑤ 刀目标（F7 misc / F4 ilb 分治的逐键底账）。

## 账目与口径

- 源 log：`.acceptance/dps/f1shell-mt-post.log`（④ 收口批 mt 腿 05:2x 产物，judge
  F 52→30 权威口径，rows 10092 恒一）。raw Fail 58 实例 = main 30 / ab 28；
  shared-red 28 双页同红，main 单红 2 = X37 对。
- 工具：复用 `census-mt-residual-v5fix2.py`；逐键名单
  `.acceptance/dps/f1shell-residual-30.txt`（含 expected/got detail）。
- 跨页 divergence：main 红 ab 绿 2 = X37 同对（subscript>formatblock qS/qV，
  签名 A 指纹未变，维持除名）；**零 ab 单红新增键**。

## 簇表（main 残 30 降序）

| 簇 | 键数 | 链 × markup | got 特征 |
|---|---:|---|---|
| **F4 ilb corruption** | **9** | `insertlinebreak>inserttext(d)>inserttext(e)` 5 + `…>inserttext(f)` 3 + `…>inserttext(c)>inserttext(d)>inserttext(e)` 1 | img 变体幻影 `<b><i><img src="/img…` 4 + `<s></s>` 空壳 2 + 纯文本重复（`abc<br>dec…` / `<br>de<div>…`）3 |
| **F7 delete-merge-div** | **6** | `delete>inserttext(g)>inserttext(h)` 全在 ?9001-last：flex span 2（裸/`<b>` 壳）+ grid span 2（裸/`<b>` 壳）+ div 跨块 `<b>` 壳 2 | 合并后 typed 段重复/错位（`<div>abc<br>defabc…` 形）——flex/grid 容器与 div 跨块合并同基本面 |
| **F7 ip-placeholder** | **4** | `swcss>delete>insertparagraph>inserttext` 2 + `swcss>insertparagraph>inserttext` 2 × `<div><b>[abc]</b></div>` / `<div>abc<b>[def]</b></div>` 双形 | 块内置换/尾随空块形 |
| F6b handoff（backlog） | 4 | `delete>inserttext(a)` × `<font size=2><sub>` / `<sub><font size=3>` 双形双位 | 幽灵 `<sub>a</sub>` 外溢尾壳（`…</font><sub>a</sub>baz`）——N1/N2/N4/N5 引擎产出形 vs 语料期望形分歧维持 |
| F6a color-case | 2 | `delete>inserttext(a)` × 蓝色 font 壳 | exp `#0000ff` got `#0000FF` 大小写归一缺失 |
| F1' fontsize(4) 残 | 1 | `swcss>fontsize(4)>insertText(a)` × `<font size=7>{}<br></font>` | 他机制（非 shell carry 域），与 F7/F4 同窗评估 |
| X37 伪影（除名） | 2 | `subscript>formatblock>inserttext` qS/qV | 维持除名（2026-11-08 定谳） |

合计 **30** ✓。

## 对 ④ 前普查的漂移对账

| 族 | ④ 前 | 现 | 归因 |
|---|---:|---:|---|
| F1' typed 壳继承 | 23 | **1** | shell carry 刀收 22（蓝红 16 + fontSize 中置 6）；残 1 = fontsize(4) 单命令，机制预测内他机制 |
| F7 misc | 12 | 12 | 未动；构成与 [④ 前普查档](2026-11-09-f6b-post-census-52f-regroup.md) 一致（delete-merge-div 6 + ip-placeholder 4 + host-div-wrap 1 + trailing-empty-p 1） |
| F4 ilb | 9 | 9 | 未动（img 4 + `<s></s>` 2 + 纯文本 3 恒） |
| F6b handoff | 4 | 4 | 未动 |
| F6a | 2 | 2 | 未动 |
| X37 | 2 | 2 | 维持除名 |

零家族漂移，④ 刀翻转严格收敛于 F1' 域——与 calibrate FLIPNON=0 互证。

## ⑤ 刀建议（DC-4 降序）

- **F4 ilb 9**：got 形两分——① 文本重复/错位（`abc<br>dec` = 首字符 dup +
  尾块丢失，7 键含 img/s 变体）② 幻影片段（`<b><i><img`、`<s></s>`，2 键）。
  链形统一为 ilb 后多步 typing，与 F1'-B（swcss typing 拆壳）不同域（此处无
  swcss 前缀），病灶在 insertlinebreak 应用路径的壳/块边界。
- **F7 delete-merge-div 6**：链形统一 `delete>inserttext(g)>inserttext(h)` 双步
  typing；markup 全带 flex/grid 容器或 div 跨块 + inline 壳。与 ④ 刀同域的
  跨块合并 caret 位（块级伴随物=flex/grid span、div 边界）——先对照 ④ 刀
  `_zwShellCarryAtBoundary` 块级止步表（flex/grid 容器 span 是否在块表）评估
  是 carry 门覆盖缺口还是独立合并形缺陷。
- ip-placeholder 4 + 零散 2 键（host-div-wrap/trailing-empty-p）：微机制分治，
  不与本刀强并。
- F6b handoff 4 / F6a 2 / F1' fontsize(4) 1：维持既有 backlog / 待小刀归并。
