# 第十一刀普查——ip 链 36 + separator 双面制定谳（2026-11-07，knife10b 终判后开工；12:30 刀体落树修订）

## 残键具名（knife11-reconcile.txt，judge 权威域）

`knife10-reconcile.py knife9-mt-post.log knife10b-mt-post.log`：rows 20172→20184 /
F2P 147 / P2F 0 / postF 1；A-RESIDUAL 68，其中 ip 链 **36** =
multitest-ab.html 族（a_keys 域）：

- **12 rich**（compare innerHTML）：case 20/53/86/119/152/185/218/251/284/317/350/383 =
  `[值/样式命令, insertparagraph, inserttext a] "foo[]bar"`，期望 `<p>foo</p><p><b|span|font|a>a[]</…>bar</p>`。
- **22 styled**（compare innerHTML）：case 485-488（styleWithCSS false 组）+ 495-499 +
  514-520 + 529/531/533/535（img 组）+ 539/545/547/553/555，期望 `<div>` 分隔
  （如 495 exp `["<div>abc</div><div><b>def</b>ghi</div>", "abc<div><b>def</b>ghi</div>"]`）。
- **2 qS-after**：`[styleWithCSS false, ip, it a|d]` on `<div><b>[abc]</b></div>` /
  `<div>abc<b>[def]</b></div>`，queryCommandState("bold") after。

另 mt 残 insertparagraph compare **B 簇 12**（非 a_keys 域）= `[值/样式命令, ip]` 双命令链
（无尾 inserttext），期望 `<p>` 分隔——与 rich 12 同 separator 红源。

## 归因定谳：separator 双面制（产线 p / 值面 div）

### 产线块面（rich 12 + B 簇 12 红源）

- `__zwDefaultParagraphSeparator` 无显式初始化（唯 part06:4893 setter 赋值，
  未设置态=undefined），产线四处读取恒 `=== 'p' ? 'p' : 'div'`（part01:10553 ① sepTag、
  part01:10700 ②b sep2b、part01e:513 list retag、part01f:1531 ipara_apply）——未设置时产 div。
- multitest.js 全 628 案**零** defaultParagraphSeparator 命令（grep -c=0）；run/multitest.html
  与 multitest-ab.html bootstrap 仅 styleWithCSS（两页逐字同，grep 零 separator 命中）。
  rich 组 ambient ip 期望 `<p>` 分块 = 录音事实 → 产线面缺省翻 `=== 'div') ? 'div' : 'p'`
  （undefined→p，显式 setter 一票胜出不变）。
- ① 臂实证为 rich 生产面：`foo[]bar` 裸文本 caret（block===host、boundaryParent===host、
  isText）落 :10616 守卫，:10618 `sep1 = createElement(sepTag)`；⓪ void 臂 :10587
  `newTag0 = 'div'` **硬编码**（语料 void 期望显式 div）翻置安全。
- F2 簇注引的 ipara 语料 `foo[bar]baz` 期望 `<div>foo</div><div>baz</div>` = **显式 div 链**
  （data/insertparagraph.js:3-7 `[["defaultparagraphseparator","div"],["insertparagraph",""]]`）
  ——显式 setter 路径翻置免疫，非 ambient 冲突。

### 值面（qV 读面必须保 div——假阴性教训）

- **此前「qV 零暴露」是坏 grep 假阴性**：`queryCommandValue("defaultparagraphseparator")`
  调用串只活在 tests.js 运行时（:5595/:5627 browser 列），data 文件里只有
  `[false,false,"div",false,false,"p"]` 形**元组**，grep 调用串零命中 ≠ 零断言。
- 语料实证（data/insertparagraph.js:3-30、formatblock.js:4-7 同构）：各文件**首案**
  separator qV before 值断言 `"div"`（= 未设置态真值），后续案 before=上一案 after 的
  **顺序泄漏链**（:7 div/div → :12 div/p → :17 p/div → :22 div/p 交替，与链内显式设置
  逐一吻合）。值面翻 p 将使每文件首案 P2F——**part06:5015 保持
  `=== 'p' ? 'p' : 'div'` 不翻**（undefined→div）。
- spec 分层（诚实记账）：HTML spec 的 defaultParagraphSeparator IDL 已移除
  （interaction §6.8.4 现仅指 [EXECCOMMAND]——旧锚
  `#dom-document-defaultparagraphseparator` 已死，webReader 实证）；现行 execCommand
  spec（w3c.github.io/editing/docs/execCommand/）2016-08 起 default single-line
  container name **"must initially be div"**（附浏览器共识更新注）——**值面 div 与现行
  spec 一致**；rich 组 p 形属演进前录音（styled 组 div 期望同源，翻后仍红按 DC-4
  记账，今日 Chrome Enter 亦 div 形）。
- 两副面孔为语料双源强制：multitest rich 组（产线 p）与 per-command 首案 qV before=div
  （值面 div）同仓并存，单一默认不可双过。

### styled 22+2（第二层病害，翻后仍红）

styled 链 caret 在宿主 inline 链内（`abc<b>[def</b>]ghi`、div 包裹 `<b><i>` 等），
part06:4752 ip 臂 `__zw_ce_ipara_wants()` 嵌套单行容器判定不接 → 回落
`__zw_ce_paragraph_split_at(host2, rng2, false)`（part01:10821 → `_zwCeParagraphMutate`
part01:10524）。病害名（knife10 普查）：fallback 裸奔**追加**——疑似 split 位置/重锚
落点错致新块追加在宿主尾而非 caret 处拆分；期望 div 分隔翻后 got 变 p 形语义正确，
作刀体佐证不入 Fail 预测集反转账，按 DC-4 三分表记 Chrome delta/语料不一致。

## 刀体（已落树，2026-11-07 12:28）

逐点翻转（非助手形状——并发轮 399090bd 两形皆接受，取 diff 最小逐点形）：

1. part01.js ① sepTag：翻 paren 形 + 主刀注（双面制 + spec 分层 + DC-4）。
2. part01.js ②b sep2b：翻 paren 形 + 短注引 ①。
3. part01e.js:513 list retag：翻 no-paren 形 + 短注。
4. part01f.js:1531 ipara_apply：翻 paren 形 + 短注。
5. part06.js:5010-5016 qV 值面：**代码不翻**，注释改写为双面制注（首案 div 断言 +
   顺序泄漏链 + execCommand spec initially div + 产线/值面分歧语料双源）。
6. part01.js ⓪ 臂注释 :10558-10559 修正（「defaultParagraphSeparator 缺省」失真 →
   「void 面语料固定 div，与 ① 臂产线缺省翻 p 不同面」）。

## 验证面

- **探针批** `.acceptance/dps/batch-knife11-probe.sh`（12:29 发射）：
  build-anchored **四文件** md5（part01/part01e/part01f/part06；yielder 原三文件清单
  按其注「按贵轮实际改动文件清单改」扩）；双面制四 marker——paren 旧形期望 0 /
  paren 新形期望 3（①/②b/ipara）、no-paren 旧形期望 1（part06 qV 值面保留）/
  no-paren 新形期望 1（part01e）；slice 语料
  `zz-probe-knife11-slice.html`（yielder 资产：0..18 忠实窗 + case 19/20 直调
  GOT=EXP + HOSTOFFSET 观察面）判 case 19/20 双翻绿。
- **mt 终判**：build-anchored 新字节，judge vs knife10c-mt-post.log（12:27 新基线，
  P 9926/F 166）；红线 P2F=0 / lost=0 / post-only=0 / rows 恒一；F2P 预测集 =
  **rich 12 + B 簇 12 = 24 compare 键**（styled 22+2 留红不预测；qS-after 2 键依赖
  caret 病害修，本刀不预测）；list retag ambient 超额 F2P（insertorderedlist/
  insertunorderedlist/formatblock）逐键记账。ipara 单文件腿全分母（464 显式 + 48
  ambient 双收）+ join/wss/ilb 腿零位移 → enginelib 6-red 名册（另册：
  2026-11-07-knife10c-roster-drift-6red.md）。

## 并发轮事件（399090bd 交叉印证）

并发会话同窗独立普查出同一 separator 根因后即时让位（认领 bfef95693 11:59 在先）：
checkout 回退其未提交三处翻转（12:06:40 mtime 迹），树复原 pristine，转核验方；
其探针资产（slice 语料 + 批脚本）留盘复用。其独立复核四项（multitest 零分隔符命令/
ab 变体同向/insertparagraph 显式面零回归/styled 翻后仍红）与本 census 逐项吻合；
其「HTML spec 缺省 p」表述经 webReader 实证修正为「IDL 已移除，execCommand spec
2016-08 起 initially div」——双面制注按修正后史实落。核验交接单（yield note
12:10）：四处读取全替（含 part01e:513 与 qV 读面处置说明）、spec 链接在注、styled
组 DC-4 三分表、mt judge 红线 + F2P 键级账——本文件即处置说明，qV 读面处置 =
保 div 不翻（首案断言 + spec initially div）。
