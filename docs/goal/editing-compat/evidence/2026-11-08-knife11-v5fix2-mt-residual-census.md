# v5fix2 mt 残 123F 普查分簇（2026-11-08，DC-4 节奏输入）

knife11 v5 落地（28b31d7d4）+ v5fix2 solo 复证六门全绿后的残键普查。目的：按
DC-4 收敛定稿（残键数降序）立项下一刀；产出簇表 + #37 嫌疑甄别 + 既有挂账对账。

## 账目与口径

- 源 log：`knife11-v5fix2-mt-post.log`（2026-11-08 02:13 cont 批产物，judge 绑定
  见 P7 回执）。raw Fail 244 条全部唯一键：**main 123**（multitest.html 十窗片，
  judge F=123 权威口径）+ **ab 变体页 121**。
- 跨页对账：shared-red **121**（main 红且 ab 红——ab 页无窗切、单发射执行，窗界
  伪影不可能复现 → 记真实引擎债候选）；divergent **2**（main 红 ab 绿 → #37 嫌疑，
  见下节）。工具：`census-mt-residual-v5fix2.py`（v2 修正版——v1 备稿把命令链
  正则打在 file 段，恒出 `?`，已重写）＋逐键全名单
  `census-mt-residual-v5fix2-keys.txt`（244 条含 expected/got detail）。

## #37 窗界嫌疑 → 定谳除名（2 键，签名 A 跨窗伪影，零源码修改）

`[["subscript",""],["formatblock","<div>"],["inserttext","a"]] "foo[]bar"` 的
`queryCommandState("subscript") after` 与 `queryCommandValue("formatblock") after`
双断言（?2001-3000 窗红，ab 页全绿）。**本轮窗界核对定谳为 ⑪ 轮签名 A 复发，
引擎无辜，两键除名**（真残基线 123→**121**）：

- **断言块跨 ?2000/2001 窗界**：该案 23 子测试分属两窗——exec 三连
  （subscript/formatblock/inserttext `return value`）+ `compare innerHTML` +
  `checks for modifications` + subscript qI/qS/qV before 读全在 **?1001-2000**
  且全绿（exec 在该窗真跑、树逐字合录音）；formatblock/inserttext 双向读 +
  subscript qV-after + 两红行落在 **?2001-3000**。
- **?2001-3000 窗无 exec**：红行 got = 未动 setup 树直读——qS("subscript") after
  期望 true（树应含 sub）got false；qV("formatblock") after 期望 "div"（exec 后
  块）got ""（裸 body）；而同窗 subscript qV-after 绿 = 期望 "" 与未动树巧合一致
  ——Pass/Fail 分割线恰为「期望值是否编码 exec 后状态」，签名 A 指纹逐条命中。
- **ab 页反证**：单发射全案顺序执行（含真 exec），23 子测试**全绿含两红行本体**
  ——引擎 in-window 行为的隔离验证证据（探针等价）；同案 ?1001-2000 exec 窗
  compare 亦绿。

处置：两键按[分窗切片伪影判据](../../../learnings/bugs/2026-11/2026-11-07-window-straddle-accounting-audit.md)记 artifact
除名，不入引擎债账；sr2 轮先例同构（2026-11-06，iol/iul 除名 3 行零源码修改）。

## 簇表（main 残键降序；chains 为命令链签名 × 键数）

| 簇 | 键数 | 构成 | got 特征签名 |
|---|---:|---|---|
| **F2 link-absorption** | **36** | createlink 前缀链 26（裸 `createlink>inserttext` 1 + 中置结构/样式命令 13 + forecolor 互序 8 + underline 互序 4）+ delete 族 link 组成 10 | 期望 typed 段并入 `<a>`（`foo<a>a</a>bar`）；got 三形：裸文本 `fooabar`（无 link）、`<font color>a</font>`（有样式无 link）、`<font|u>` 反包 `<a>`（`<font color="blue"><a>a</a></font>`） |
| **F1' typed styled-shell rebuild** | **29** | monospace 重建 24（swcss-false delete/forwarddelete+insertText 16 + fontSize(5) 中置 8）+ bold>italic 组成 3 + swcss 单命令 2 | **= 既有「insertText styled 壳重建 29（F6 域）」挂账，普查对账闭合**。got：typed 段带陈旧 `<font face="monospace" size="7" color="#ff0000">` 重裹（删除区旧壳），期望并入当前 typing 态壳（`<span style="color:rgb(0, 0, 255)">fooA</span>`） |
| F6b sub/sup+fontsize carry | 14 | sub/sup>delete/forwarddelete 4 + sub×fontsize 互序 4 + delete 族 sub/fontsize 嵌套 6 | 期望 `<sub>` 外/内层 font 尺寸 carry（`<sub><font size="3">a</font></sub>` ↔ `<font size="3"><sub>a</sub></font>`），got 丢层或错序 |
| F3 inserthtml shell-order | 12 | 12 值/样式命令 × `inserthtml>inserttext`（bold/italic/strike/sub/sup/u/backcolor/createlink/fontname/fontsize/forecolor/hilitecolor） | 期望 typed 壳接在 inserthtml 内容之后（`ab<b>c</b>d<b>a</b>`），got typed 壳插在整块之前（`ab<b>a</b><b>c</b>d`）——**= 既有「inserthtml 链 12」挂账闭合** |
| F4 ilb corruption | 10 | `insertlinebreak>inserttext×2/3` 9 + createlink>ilb 1 | 期望 `abc<br>def…`；got 文本重复/幻影 `<b><i><img src="/img/lion.svg">def</i></b>` 片段——**= 既有「ilb 链 10」挂账闭合** |
| F7 misc boundary | 18 | delete-merge-div 7（`{abc</div><div>def</div>}` 跨块 delete+双 typed）+ ip-placeholder 5（`<br>` 占位包裹形）+ table-cell br 壳 4（td/th `<b><br></b>` vs `<br>`）+ host-div-wrap 1（`<b>a</b>` vs `<div><b>a</b></div>`）+ trailing-empty-p 1 | 各自独立微机制 |
| F6a color-case | 2 | delete 族 forecolor 尾键 | exp `#0000ff` got `#0000FF`——typed 段 font color 属性值大小写归一缺失 |
| **X37 伪影（已定谳除名）** | **2** | subscript>formatblock>inserttext qS/qV | 签名 A 跨窗伪影（见上节），不记引擎债 |

合计 **123** ✓，其中真债候选 **121**（X37 两键 artifact 除名）。对账校验：既有
挂账四族中「styled 壳重建 29」「inserthtml 链 12」「ilb 链 10」三族与普查逐键
闭合；「createlink 前缀链 10」旧账为 knife10 时点计数，现族已扩至 36（中置结构
命令族 + forecolor/underline 互序族 + delete 族组成 10 并入同一 absorption 机制域）。

## 下一刀建议：F2 link-absorption 36（残键数降序第一簇）

- **机制**：Chrome typing 态携带 link 面——① collapsed 下 createlink 置 pending
  link 态，后续 typed 段并入 link（F2a 26）；② 既有 link 内容 delete 后 typed 段
  重入 link 且 forecolor/underline 组成在 link 内层（F2b 10）。本实现
  `__zw_ce_capture_typing_chain` 链上行遇 `<a>` 即弃捕（part01f.js :1147-1149
  `hasA`→`return []`，注释锚 :2163/:2168 双向实证）——该排除面是 F2 直接病灶，
  但盲撤会回退 :2163/:2168 两锚（当时实证语义），需区分「pending link 态」与
  「既有 link 捕获」两条路径分别处理。
- **开工前置**：① 尾链落地（换基门绿 + push）后才动源码；② 评估 stash@{0}
  `typing-style-round-1102`（收口协议 §运行约定——F1'/typing 域在途草稿，F2 与
  其同域相邻，评估后决定吸收或让位）。X37 两键窗界核对已在本轮完成并除名。

## stash@{0} 评估（2026-11-08 本轮完成，F2 开工前置清账）

`typing-style-round-1102`（delKeepChainShells 双壳保全草稿，2026-11-02）评估定谳
**陈旧 + 目标债已被后继处方治愈，保留待删**（按运行约定不盲 pop、不静默丢弃，
删除随 F2 刀落地提交一并执行留痕）：

- **不适树**：`git apply --check` 失败——strip-wrappers 段自 2026-11-02 后重构
  （delKeepShell 逻辑迁至 part01f.js :479 `origChainHasInline` 形 + sameStrip 环），
  草稿 hunks 上下文全部漂移。
- **目标债已闭**：草稿针对的 `<b><i>[def]</i></b>` 双壳保全族（delete/forwarddelete
  × 3 选择形 × 双页）在 v5fix2 基线 **12/12 全 Pass**——knife11 v2 typed-fate
  处方（caret 入壳 `__zw_ce_caret_in_leading_shell` + 非 collapsed fallback
  deleteSelection 前导）已覆盖该语义，草稿无残余价值面。
- 草稿注释中 `foo<u><a>[bar</a></u>baz]` / `foo<a><font>[bar</font></a>baz]` 两形
  与 F2b 的语义关联（最外层 a 留、上壳摘）已由现行 strip-wrappers 幸存壳臂承载
  （F6 轮实证），F2b 残 10 键是 absorption 面非 preservation 面，与草稿机制不同。

## 运行产物（gitignored，本地 .acceptance/dps/）

- `census-mt-residual-v5fix2.py`（v2 普查脚本）、`census-mt-residual-v5fix2.out`
  （簇表输出）、`census-mt-residual-v5fix2-keys.txt`（244 键逐键全名单 + detail）。
