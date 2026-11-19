# ⑥ J-A/J-D justify 祖先壳改写刀 — 机制、矩阵与门禁

**日期**: 2026-11-05；**轮次**: editing 流 ⑥（J-A/J-D ancestor-shell rewrite cut）
**基线**: HEAD 4046c20a9（⑬ 终态）；justifycenter 6,078P/125F（jcbase 腿 1，同口径 41 fixture/104 compare 红行）
**语料权威**: tests/wpt-runner/wpt-data/editing/data/justify*.js（2017 era Chrome 录音冻结；非 modern Chromium 现行源）

## 1. 挂账与目标

justifycenter 125F 残余（⑩ 轮后口径）经 [census](2026-11-05-justifycenter-family-census.md) 归族：J-A（祖先载体壳改写缺失）、J-B（表格面）、J-C（边界↔br 共享原语）、J-E（fresh wrap ws 吞并）、β（邻接同值壳吸收）。本轮主刀 J-A/J-D + J-E + β 三面，J-B/J-C 不在本轮。

41 unique fixture / 104 compare 红行分类（/tmp/qv-fix4/jc-fixtures.txt，逐 fixture EXP/GOT 实录）：

| 族 | fixture | compare 行数 | 机制 |
| --- | --- | --- | --- |
| α 祖先壳改写 | 0,5,14,15,18,19,27,28,29,30,34 | 22 | 见 §2.1 |
| β 邻接同值壳吸收 | 6,7,20,21,32,33 | 10 | 见 §2.2 |
| J-E fresh wrap ws 吞并 | 1,2,31 | 12 | 见 §2.3 |
| J-C 边界↔br | 3,4,8,9,10,11,12,13,16,17,22,23,24,25,26,39,40 | 44 | 本轮不动（与 ③'-d 共享原语） |
| J-B 表格面 | 35,36,37,38 | 16 | 本轮不动 |

## 2. 机制模型（录音权威，非现行 Chromium）

### 2.1 α 臂：祖先对齐载体壳改写（value-asymmetric）

选区完全落在单个祖先对齐载体壳（`_zwJusIsAlignCarrierEl`：`<center>` / `[align]` 属性 / style 含 text-align 声明）内部时，`__zw_ce_justify_apply` 在 elementList 收集后、逐成员处理前插入壳改写臂：

- **target ≠ left（center/right/full）**：壳归一为 `style="text-align:T"` 形。
  - `<center>` → 改名 div + 设 inline text-align；
  - div 载体（align 属性任意值含 nonsense/left）→ 去 align 属性 + 改写/追加 style text-align（其余声明保留）；
  - 未知名（quasit）→ 去 align 属性 + 壳外包新 `<div style="text-align:T">`（outside-wrap，fixture 34）。
- **target == left**：剥对齐（去 align 属性 + style text-align 声明摘除），div/center 改名后 unwrap，后裔提升；justifyleft 语料 9 祖先形全部期望 `<p>[foo</p><p>bar}</p>`（壳消失）。
- 进入门：elementList 为空（`{}`-boundary 全选壳内容时 pass ① 不收集祖先壳，让位既有 pass ① 路径——所有 `{<div align=center>…}` 语料 fixture 壳内均有载体，不受影响）；range.startContainer 爬升止于 editing host；endContainer 在壳内；`_zwJusShellRewritable`（每子节点 `_zwJusIsContained` + 递归无 non-editable 后裔——保护 partial-selection 行如 justifycenter :618 `<div align=center><p>[foo]<p>bar` 期望仅内层 wrap 现行已绿）。

### 2.2 β 臂：邻接同值 div 载体壳吸收（target ≠ left only）

成员块为元素且紧邻（可选 ws-only 文本）同值 div 载体壳（`_zwJusGetAlignmentValue` === alignment）时，块移入壳末、ws 一并移入，再 `_zwJusCollapseAlignedShell` 合并（壳内相邻同值壳递归吸收，guard 64）。fixture 6：`<div align=center><p>foo</div> <p>[bar]` → `<div align=center><p>foo</p> <p>bar</p></div>`。center 载体壳不吸收（录音权威：center 邻接行期望保持 `<center>` 形）；target==left 全臂关闭（justifyleft 邻接行期望 no-op，pass ③ 复用路径已绿）。纯文本成员（fixture 39/40 裸文本）不入臂，走 J-C 路径。

### 2.3 J-E 臂：fresh wrap ws 吞并

pass ③ fresh wrap（`npFn` 新建 wrapper 分支）成功后，吞并 wrapper 紧邻的 ws-only 文本兄弟（前插 index 0 / 后接 append）。根因：`_zwJusIsCollapsedWhitespaceNode` 深度优先爬升会扎进前兄弟后代遇 text 才断，块后 ws 判为「可见」不入 unshift → fresh np 两 div 形（fixture 1 GOT `<center><p>foo</p></center> <div…>` vs EXP ws 入内）。复用路径（非 fresh）不动——β 吸收对该面产出恒等 DOM。

## 3. 门禁设计

- **对照**: jcbase-justifycenter.log（腿 1 先于 ⑥ 编辑启动，15:12 ⑐ binary，rows=6203/fails=125 有效）+ jcbase-{justifyleft,right,full}.log（⑥ 编辑回退后纯净 HEAD 重跑，legs 2-4）；multitest/enginelib 对 ⑬ 既有 fix15a-mt-part-*.log / fix15a-enginelib.log（fix15a 跑批 14:21-14:40 与并行落树交叠的混合二进制窗：enginelib 腿 14:21:30 先于 M3 51d8a6949 落树，实为 M3 前产物——r3033 差异归因见 §4；multitest 片级 fails 前 7 片逐值恒等支持可比性）。
- **处理腿**（run-jc-cut.sh）：四 justify 语料全量 → zz-probe-carrier 探针（⑬ apply 面守卫 32P/0F）→ `make guarded-test` enginelib（对 fix15a 3,731P/5F 五红名恒一）→ multitest 10 片（slice-one.sh + pristine trap，对 fix15a 合并基线行级元组）。
- **判决**: judge-jc.py（四语料行级元组，row key `corpus :: assertion`）+ judge-jc-mt.py（multitest 10 片合并）。红线：P2F/newF ≠ 0 即停刀回溯。

## 4. 门禁结果

**四 justify 语料（judge-jc.py 行级元组，处理腿 jccut-*.log 对基线 jcbase-*-sec.log）**：

| 语料 | 基线 | 处理后 | F2P | P2F | newF | lost |
| --- | --- | --- | --- | --- | --- | --- |
| justifycenter | 6,078P/125F | 6,126P/77F | 48 | 0 | 0 | 0 |
| justifyleft | 2,437P/66F | 2,470P/33F | 33 | 0 | 0 | 0 |
| justifyright | 4,720P/94F | 4,756P/58F | 36 | 0 | 0 | 0 |
| justifyfull | 4,675P/87F | 4,711P/51F | 36 | 0 | 0 | 0 |
| **合计** | 372F | 219F | **153** | **0** | **0** | **0** |

红线（P2F/newF ≠ 0 停刀）全绿。F2P 153 逐族命中 §2 机制模型：α 祖先载体壳改写（`<div align=X>` 任意值归一/剥离、`style=text-align:X` 载体、quasit outside-wrap）、J-D `<center>`→div 换代、β 邻接同值壳吸收（`<p>[foo]</p> <div style=text-align:X><p>bar</div>` 邻接行）、J-E fresh wrap 包缘 ws 吞入、部分 `<table align=X>` 载体改写（α 臂；J-B 块层级面另计）。justifyleft 33F→基线 9 祖先形剥壳臂正面全翻转（对齐剥离后 unwrap 提升形）。

**zz-probe-carrier**（⑬ apply 面守卫）：rc=0，32 Pass / 0 Fail。

**enginelib**（make guarded-test，132s 热构建）：3,730P/**6F**——⑬ 五红名（p56_adopted / p99s_custom_property / r3276 / r2810 / r2809，全 CSSOM 重序列化签名族）逐名恒一，另现 **r3033**（HTMLCollection for...in 枚举泄漏 length/item/namedItem）。**归因定谳（五步豁免链，非 ⑥ 回归）**：① fix15a-enginelib.log mtime 14:21:30 早于 51d8a6949（M3 documents 簇，14:34:37）等 4 笔引擎提交——§3「同 HEAD 产物」前提对 enginelib 腿不成立，3,731P/5F 基线系 M3 落树前产物；② 51d8a6949 将 HTMLCollection 原型面 length/item/namedItem enumerable false→true（part05 `_hp` 层，注释明引 WebIDL for-in 可见 + document.forms iteration 期望含 item/namedItem/length）；③ r3033 红签名恰为该三成员泄入 for...in（left `0,1,length,item,namedItem` vs right `0,1`），part13.rs:679 旧期望未随 M3 对齐；④ **worktree 探针（HEAD 4046c20a9 无 ⑥ diff，补链 rusty_v8 缓存后构建）r3033 FAIL 0P/1F**——正式豁免为 HEAD 既有第 6 已知红（51d8a6949 测试对齐欠账，s249 CSSOM 族同性质，挂账测试对齐独立轮）；⑤ 含 ⑥ diff solo 复跑同签名 FAIL，无 diff 交互。⑥ enginelib 门终判 = 5 已知红逐名恒一 PASS。

**multitest 10 片**（judge-jc-mt-v2.py 交集口径——首版判读器把「基线有/处理缺」行双计入 F2P+lost，已修）：

- 片 1-9 fails 对 fix15a 基线逐值恒一（30/24/34/24/50/49/59/47/14）。
- 尾片 ?9001-last 两次截断：竞争跑 1,077 行/1T、solo 复跑 956 行/1T——切点漂移系页内 300s watchdog 预算非确定切点（solo 反短于竞争跑，无 cargo 交叠可归因，机器并发负载漂移）。
- **交集权威账：base 9,676P/416F → post 9,661P/416F，交集 10,076 行 F2P 0 / P2F 0**；单侧 newF 1 = 截断页 Timeout 页行、lost 16 = 截断带 base-only 行（全 delete/forwarddelete+inserttext 族）。
- 联合覆盖分账：尾片 1,092 行中 1,076 行被两次处理跑联合覆盖、竞争交集带与 solo 交集带均零回归；未覆盖 16 行（尾部 1,078..1,092 带）**零 justify 断言**（不经 `__zw_ce_justify_apply` 路由）——⑥ 机制作用域内无未验证断言；尾带按 ⑧ 轮截断伪影口径记档。

## 5. 附注

- part01c.js 本轮 +214 行至 **2,275 行**，已超 2,000 行指引（此前 2,061 已超）；本轮为刀内完整性未拆分，拆分挂账下轮独立处理。
- modern Chromium 已漂移（applyBlockStyle 段级 MoveParagraphContentsToNewBlockIfNecessary + AddBlockStyle style-prepend）；语料录音为唯一权威，不追现行。
- 十片 multitest ?9001-last 截断带口径沿用 ⑧ 轮（尾带伪影单独归因）。
