# 第十一刀认领——insertparagraph 链 36（2026-11-07 11:55，本会话）

按控制面 v1.67 下一步（残 166F 按残键降序续刀）认领第十一刀：**ip 链 36**（纯
`[["insertparagraph",""],["inserttext","d"(,"e"(,"f"(,"g")))]]` 链；18 形 × ab/9001-last
两文件 = 36 judge 键，全部 compare innerHTML 面，输入全为跨 inline 壳 range 选区形，
无折叠 caret 形）。并行轮请勿同刀重复施工——可取 ilb 10 / inserthtml 12 /
createlink 前缀链 10 或核验方角色；本刀门禁批走 flock batch.lock 串行。

## 普查定谳（knife10b-mt-post.log 残集，judge 键域 last-wins）

- 全局 Fail 330 键中 insertparagraph 相关 96：纯 ip+inserttext 链 **36**（ab 18 +
  9001-last 18，同形镜像）+ 值/样式命令打头链 60（30 ab + 30 窗口族，typing 壳面另账）。
- 18 形清单（multitest.js 输入串）：`abc<b>[def</b>]ghi`、`abc[<b>def]</b>ghi`、
  `abc<b>[def]</b>ghi`、`abc<b>{def}</b>ghi`、`abc{<b>def</b>}ghi` 及 b+i 嵌套
  （`<div>abc<b><i>[def</i></b>]ghi</div>` 等 10 形）+ `<s></s>` 空壳前缀 2 形 +
  img 形 1 形。
- 病理双分层（got 证据，knife10b-mt-post.log）：
  - **宿主边界形（`abc[<b>def]</b>ghi`、`abc{<b>def</b>}ghi` 族）→ legacy 回落臂
    `__zw_ce_paragraph_split_at` 无 deleteSelection 前导**——选中 "def" 不删即拆
    （arm ① boundaryText/元素边界路径直移后兄弟入新块），typed "def" 落 caret 裸位
    → 双/三重发射（got `<div>abc</div><div>def<b>def</b>ghi</div><b>def</b>ghi`）。
    即控制面记名的「fallback 裸奔追加病害」。port 分派门
    `__zw_ce_ipara_wants`（part01f :1441）对宿主本体 caret 按 `_zwJusIsEditable`
    宿主约定返 false，选中内容删除责任落在 legacy 臂入口，缺失。
  - **壳内起步形（`abc<b>[def</b>]ghi` 族）→ port 删除前导与拆分正确**
    （got `<div>abc</div><div><b></b>ghi</div>` 恰 Chrome 空壳形），但后续
    inserttext 三发全灭（expected `<b>def</b>ghi`，空壳未回填）——knife10 残账
    rich-sel 族（②让位门只保素文本锚）的 ip 空壳锚形。
- 处方方向：11-A = 分派点 decline 臂补 spec deleteSelection 前导（仿 port
  :1540 `if (!_zwIpCol) _zwDelDeleteSelection()`，删后重读 live caret 传 null
  range）；11-B = probe 定谳后落刀（疑锚回灌/②重解析对空壳元素锚形）。
