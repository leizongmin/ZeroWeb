# knife11 v2 verdict 独立核验与 ip 单腿 P2F 根因定谳（并行轮，2026-11-07 16:2x）

verdict 批（15:23-16:12，batch-knife11-v2-verdict.log，head c700bfb2a，
shim md5 d74e396efa32）出数后本会话按核验方职责独立复核全部红线与分账。

## mt 全分母核验（VERDICT 数面全过）

- rows 20184（≥19000 门 ✓）；judge 去重口径 base=post=shared=10092。
- **F2P=18 P2F=0 post-only=0 lost=0**——四红线全绿。
- unique 账：base Fail 143 → post 125，Δ=18=F2P ✓；raw 账 284→248，
  Δ=36=18×2 窗化 ✓（语料两窗跑，judge 字典去重为权威口径）。
- F2P 键面：styled ip-compare 16（裸三形 495/497/498 族 + 双壳
  515-518 族 + 部分壳 de]f 族 + img 变体 + 5 键插入 g 变体）+
  qS-after bold 2——**零 UNEXPECTED**；496/499 族如预测不翻
  （DC-4 记账）；createlink rich 1 键不在 v2 域照旧红。
- enginelib：3794P/6F，6-red 名册逐名精确匹配，ROSTER OK。

## ip 单腿 P2F=2（verdict 自身已标 VERDICT REVIEW）——根因定谳

keys（dps=p 显式 + 非 collapsed + td 内 bare-text caret，PORT 路由）：

1. `["defaultparagraphseparator","p"],["insertparagraph"]`
   `<table><tr><td>[foo<td>bar]<tr>...` compare innerHTML
2. 同前缀 `<table><tr><td>[foo]</table>` compare innerHTML

实测 diff：expected `<td><p><br></p><p><br></p></td>`（录音面 separator p）
got `<td><div><br></div><div><br></div></td>`（宿主标签 copy div）。

**根因 = v2 容器面 `_zwIpWrapTag` 宿主标签 copy 规则对显式 dps 的越权**：
vendored spec insertParagraph（implementation.js :7561 `var tag =
defaultSingleLineContainerName`）新块标签恒取 **separator**（Chrome =
dps 设置，缺省 div）。multitest styled 16 键翻绿非因"container-copy 规则"
——而是 mt 语料零显式 separator 命令（628 案零命中）dps=unset，宿主
标签 copy 恰巧等于 spec 缺省 div；td 两案 dps=p 显式 → 录音 p，
host-copy 越权产出 div。**"非 collapsed → container-copy"假说被证伪**，
正解 = separator 值（显式 setter 一票胜出，unset 面按 knife11 双面制
既有产线缺省 p——rich/B 簇已绿账不动）。

## 修复处方（最小三行门，part01f :1579-1583）

`_zwIpWrapTag` 赋值门加 dps-unset 条件——显式 separator（setter 仅存
'div'/'p' 字符串，part06 :4903-4905）回落 `sep` 既有产线面：

```js
var _zwIpWrapTag = null;
if (!_zwIpCol
    && typeof globalThis.__zwDefaultParagraphSeparator !== 'string') {
  var _zwIpHost = _zwListEditingHostOf(ar0.startContainer);
  _zwIpWrapTag = _zwIpHost ? _zwJusTag(_zwIpHost) : 'div';
}
```

影响面审计：mt（dps 恒 unset）零变化（18 F2P 保持）；td 两案回 P
（= pre-v2 基线行为字节还原）；collapsed rich/B 簇（`!_zwIpCol` 门
不触发）零变化；显式 div 值面 `sep='div'` 同构。预期复跑：ip 单腿
F 210→208、P2F=0；mt F2P=18 恒一、P2F=0。

## 施工序（battery2 静默后）

**（16:25 补记）施工轮（399090bd）同期独立落 verdict 核验笔记**
（2026-11-07-knife11-v2-verdict-verify-399090bd.md §3）——同一 P2F×2
独立定谳（td 宿主 + 显式 dps=p + 非 collapsed PORT 路由，无 inserttext
步排除 ④/⑤ 面），修向处方不同：其 td/table 宿主守卫 vs 本笔记
dps-unset 门。两处方对现语料等效（td 两案均回 sep 面，mt 16 styled
均保 host-copy），本会话取 dps-unset 门为终处方：vendored spec
implementation.js :7561 新块标签权威恒为 separator（Chrome = dps 设置，
spec 缺省 div），无 container-copy 亦无 td 例外——dps 门同时关闭
「非 td 显式 dps + PORT」的语义缺口（现语料无此形，处方不应留洞）。
§4「claim 轮修向 + 复跑 ip 腿」按让位分工由本会话承接（容器面字节
经让位记录归本会话施工域）。

1. battery2 全绿确认（5/7 OK 至 16:25，F=0 不变量）。
2. pull --rebase（behind 9：SMIL B6a/R5449/s330/cssom + 站点 PR 合入；
   远端 shim hunk 与脏区不相交已核——part01f/part01e 远端零 hunk，
   part01.js 干净文件落远端 hunk，part06.js 脏区 :4753 vs 远端
   :1830-1955/:6570+/:16492+ 全不相交）。
3. 修复落地 → 新 md5 锚 → mt 全分母 + ip 单腿 + enginelib 名册复跑
   （批脚本 batch-knife11-v2-fix-gate.sh；施工轮「mt 无需重跑」判断
   成立但不取——bytes 变化后落地账以新锚全证为准，31 分钟无人值守
   成本可忽略）。
4. 落地提交：代码（part01f+part06）+ v2 证据集（typed-fate/ab-yield/
   双 verify 笔记）+ goal doc v1.69（含 knife11 落地提交正文方向勘误，
   per landing-verify-399090bd）；他人 learnings/wipe-recovery 不入
   本提交。guard PASS 后提交，pull --rebase 后推送，lark 通知。
5. DC-4（496/499 fallback 病形 + B/C 簇泄漏对账）与 fallback 容器标签
   线程化（blockTag 传参，同 dps-unset 门约束）按
   arc-verify-and-dc4-trace 笔记处方独立刀独立锚，落地后下一轮。

## 核验回执（第二核验方，16:4x）

三项关键事实独立复核全过：

1. P2F 键逐字直提（python 双日志键级 diff）= 本笔记 §定谳两键逐字节
   吻合（td×2 + dps=p 显式 + compare innerHTML，无 inserttext 步）。
2. mt 语料 `defaultparagraphseparator` 命中 **0**（grep 实证）——
   「16 styled 键翻绿系 dps=unset 恰合 spec 缺省」前提成立。
3. vendored `include/implementation.js:7574` 非 SLC 臂
   `var tag = defaultSingleLineContainerName`（:8326 setter 同变量）——
   新块标签恒 separator 的 spec 依据确认。

dps-unset 门处方审阅：接受。unset 臂保留 host-copy 系字节保守选择，
与 spec 缺省 div 在 mt 全 div 宿主语料严格等价；显式面回落 sep 闭合
非 td 缺口更优。无异议，施工按 §4 让位分工由本会话推进，第三执行体
不触 part0*.js 字节；fix-gate 出数后交叉核验 ip 腿 P2F 归零与 mt 恒一。

## 施工序完成回执（后继轮代跑，2026-11-07 19:0x）

谱系两轮（施工轮 d081bc66 系 + 后继轮 d0476931 系）17:18 前 rate-limit
相继终止，批链脱离会话存活至 17:45；fix-gate 出数后无判读方，按让位
分工与 yield 笔记预告条款由 19:04 后继轮代跑完成 §4 尾链：

1. ✅ battery2 全绿——supp 批七腿 F=0（17:43，ambient 恒一）。
2. ✅ pull --rebase——19:14 执行（behind 13，hunk 不相交预判成立：
   part06 刀体 hunk 头偏移 4753→4776、内容逐字节恒等；part01f 远端
   零 hunk），17c4a4bde → c5a28c114。
3. ✅ 修复落地→新锚→复跑——fix-gate 批（16:45:49→17:30:52，锚
   57db60a1）：ip P 6987/F **208**（rows 7195 恒一）F2P=0/**P2F=0**
   复位如本笔记 §处方预期（210→208 逐字命中）；mt F2P=**18** 恒一/
   P2F=0/post-only 0/lost 0（rows 10092 恒一，F 143→125）；enginelib
   3794P/6F 名册恒一。判读回执见 yield 笔记 §判读回执。
4. ✅ 落地提交——代码 17c4a4bde（16:55，guard PASS，字节=批锚树态，
   rebase 后 c5a28c114）+ v2 证据集 + goal doc v1.69（含 72da4f974
   正文方向勘误）由后继轮提交推送；他人 learnings/wipe-recovery 未入
   本提交（wipe-recovery 本地副本与远端逐字节相同，checkout 让位）。
5. ⏳ DC-4（496/499 plain pending 跨域语义，①臂 raw appendChild vs
   delete 面宿主桥差异 bisect 入口）+ fallback 容器标签线程化
   （blockTag 传参）——下一刀下一轮，独立刀独立锚。
