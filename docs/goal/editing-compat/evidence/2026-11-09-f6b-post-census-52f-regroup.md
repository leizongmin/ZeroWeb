# F6b 收口后残 52F 重普查（2026-11-09，④ 开工前分簇定账）

F6b 刀收口（206678d4b）后的 mt 残键重分簇。目的：按 DC-4 降序规则定 ④ 刀目标；
对 2026-11-08 [普查档](2026-11-08-knife11-v5fix2-mt-residual-census.md) 做家族漂移对账。
零源码改动轮。

## 账目与口径

- 源 log：`.acceptance/dps/f6b-mt-post.log`（v1.2 批 mt 腿 03:29 产物，judge
  F 66→52 权威口径；v1.2c 新树终验 enginelib/battery 不改 mt 域）。raw Fail 102
  实例 = main 52 / ab 50；去重后**真实键 52**（50 键双页同红 + X37 两键 main 单红）。
- 工具：复用 `census-mt-residual-v5fix2.py`；逐键名单
  `.acceptance/dps/f6b-census-keys.txt`（含 expected/got detail）。
- 跨页 divergence：main 红 ab 绿 2 = X37 同对（subscript>formatblock qS/qV，
  签名 A 指纹未变，维持除名）；无新增 ab 单红键。

## 簇表（main 残 52 降序）

| 簇 | 键数 | 构成 | got 特征 |
|---|---:|---|---|
| **F1' typed 壳继承残** | **23** | swcss-false delete/forwarddelete+insertText 16（蓝红双 span 段族，单一 got 形）+ fontSize 中置 6（fontSize(5)×foreColor 组合 2 + 纯 fontSize(5) 4）+ fontsize(4) 单命令 1 | delete 合并跨块后打字落**裸文本**（`<span 蓝>foo</span>A<span 红>bar</span>`），期望并入左侧 typing 态壳（`fooA`）——与 F1'-B 收口 learning 的 D4 拆壳臂丢 armed tags→裸字根因同域（[learning](../../../learnings/bugs/2026-11/2026-11-09-delete-selection-cross-shell-noop.md)）；fontSize 中置族 got 为旧 font size 壳重裹/丢层 |
| F7 misc boundary | 12 | delete-merge-div 6（flex/grid 4 + div 跨块 2）+ ip-placeholder 4（`<div><b>[abc]</b></div>` delete/insertparagraph 组合，got `<div><br></div>` 置换形）+ host-div-wrap 1 + trailing-empty-p 1 | 各自独立微机制（普查档口径不变，键数漂移见下节） |
| F4 ilb corruption | 9 | `insertlinebreak>inserttext×2/3` 9（img 变体 4 + `<s></s>` 空壳变体 2 + 纯文本 3） | 文本重复/幻影 `<b><i><img>` 片段，机制同普查档 |
| F6b handoff（backlog） | 4 | delete+inserttext × `<font size=2><sub>` / `<sub><font size=3>` 双形 | N1/N2/N4/N5 引擎产出形与语料期望形分歧，F6b evidence DC-4 表已定 backlog |
| F6a color-case | 2 | delete 族 forecolor 尾键 | exp `#0000ff` got `#0000FF`，大小写归一缺失（普查档口径不变） |
| X37 伪影（除名） | 2 | subscript>formatblock>inserttext qS/qV | 签名 A 跨窗伪影，2026-11-08 已定谳，不入引擎债 |

合计 **52** ✓（真债候选 50）。

## 对 2026-11-08 普查档的漂移对账

| 族 | 普查档 | 现 | 变化归因 |
|---|---:|---:|---|
| F2 link-absorption | 36 | **0** | F2 刀收口（createlink 全族 + delete 族 link 组成 + createlink>ilb 1 键随之闭） |
| F1' styled 壳重建 | 29 | **23** | F1'-A（override 清除 3）+ F1'-B（typed-wrap 4）收口 −7；got 形演化：陈旧 monospace 壳重裹 → 裸文本直落（中间刀改变了失败形态，机制域未闭） |
| F6b sub/sup+fontsize | 14 | 4 | F6b 刀 fixed 10，余 4 键 handoff backlog |
| F3 inserthtml | 12 | 0 | F3 刀收口（cf856b9c2） |
| F4 ilb | 10 | **9** | createlink>ilb 1 键随 F2 闭 |
| F7 misc | 18 | **12** | table-cell br 壳 4 键已不在残集；ip-placeholder 5→4、delete-merge-div 7→6——具体收敛面系 F1'-B/F6b 等中间刀跨族闭合，未逐键归因，以本普查现值为准 |
| F6a color-case | 2 | 2 | 未动 |
| X37 伪影 | 2 | 2 | 维持除名 |

## ④ 刀建议（DC-4 降序更新）

普查档建议的 F4/F7 分治已过时：降序首位现为 **F1' 残 23**，其中 16 键蓝红双 span
族 got 形完全一致（8 种 markup 变体同一裸字 got）＝单一机制最佳首刀候选；fontSize
中置 7 键次之，可与 16 键族同刀评估（同属 typing 态壳继承/重建域）。

开工前置：F1'-B 根因 learning 在档（D4 拆壳臂 armed tags 丢面 + 前向同 tag 壳合并
缺失两笔），16 键族与其同根概率高，首步对照该 learning 的修复面差异定新臂范围。
