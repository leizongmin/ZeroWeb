# knife11 v2 刀——typed-fate 处方 a/b/c 三相施工与探针弧线（census 轮续，2026-11-07）

knife11 sep 双面制落地（72da4f974）后续刀：mt 残 143F 中 typed-fate 25 键
（styled 22 + qS-after 2 + createlink rich 3-cmd 1）按 landing-division §5
处方三笔施工。探针 `zz-probe-knife11-v2.html` 21 案三相（faithful 全量 /
faithful 单案 instrument / 直调 per-step dump），批锁串行
（batch-knife11-v2-probe.sh，逐相 build-anchored md5 + marker 双检）。

## 相 (a)：port typing delta ④/⑤ 同构修（15:01，md5 c70be032）

`__zw_ce_ipara_apply`（part01f）补齐 `__zw_ce_delete_apply` 同构三件：
动作前 tt/toff 快照 + 非 collapsed tcap 捕获 + finally 回写/捕获补位
（动作内 Selection collapse 经 _zwSync 清 typing 态，typed 段裸奔——
landing-division §5.2 根因面）。

- 探针：**514/516/519/20 四案直翻绿**（预测族全中），其余 17 案恒一
  （531/533/535 哨兵绿恒一，P2F=0）。tt trace：515 post-ip
  `tt=["b","i"]`（捕获补位生效）但 caret 仍 DIV@0——typed 段落壳前
  div 级（`<b><i>def</i></b><b></b>ghi` 双壳形），入壳缺陷留 (b)。

## 相 (b)：caret 入壳下行（15:15，md5 66c6630a）

新 helper `__zw_ce_caret_in_leading_shell`（part01f，壳表复用
`_zwDelFmtTags`）：newBlock 首 子 inline 壳链下行（`<b></b>` → (b,0)；
`<b><i>f</i></b>` → (i,0)），port 主拆分支尾 collapse 调用。
Chrome 语料注锚：styled text 为新行首可见内容时 caret 应在 inline
元素内（multitest 515/529/547 族注释面）。

- 探针：**9/9 目标案全翻绿**（515/517/518 空壳填充 +
  529/539/545/547/553/555 幸存壳合流），(a) 四案与哨兵恒一。

## 相 (c)：fallback deleteSelection 前导 + wrap 臂容器标签复制（15:19/15:21，md5 d74e396e）

1. **前导**（part06 insertparagraph 派发臂）：`wants()` false 且
   range 非 collapsed 时复用 `__zw_ce_delete_apply(false)` 先删后拆
   （port :1542 前导同构；typing ④/⑤ 与 override 重锚同内建），
   传 null 让 split_at 经 caret_range live 重读（rng2 删后失效）。
2. **容器标签**（port 非 SLC wrap 臂 + 空 nodeList 臂）：Chrome
   collapsed→separator / range→container-copy 规则（landing-division
   §5(c) 考证面，multitest 495-499 语料双源录音）——delete 前导拆分
   的 wrap 块标签复制编辑宿主标签（`_zwIpWrapTag`），collapsed 保持
   separator 缺省。wrap p → 主拆分支同型复制 → 双块 div 与 exp[0]
   逐字节合。

- 探针：**495/497/498 翻绿**（19/21）；496/499 残红 = fallback
  delete 链越删缺陷（见 DC-4 节）。

## 探针弧线总账

| 相 | md5 | 翻绿 | 恒一 | 残红 |
|---|---|---|---|---|
| 基线（13:28 census） | ad212b5f 态 | 3（531/533/535） | — | 18 |
| (a) typing ④/⑤ | c70be032 | +4（514/516/519/520） | 17 | 14 |
| (b) caret 入壳 | 66c6630a | +9（515/517/518/529/539/545/547/553/555） | 17 | 5 |
| (c) 前导+wrap 标签 | d74e396e | +3（495/497/498） | 17 | 2（496/499） |

全弧线 P2F=0（绿案零回翻，哨兵三案逐相恒一）。

## DC-4 残账：496/499 fallback delete 越删（2 案挂账待刀）

`abc[<b>def]</b>ghi` / `abc{<b>def</b>}ghi`（range 起点/终点双落
宿主层元素边界）：基线（无前导）got `ghi<p>def<b>def</b>ghi</p>`
已缺 abc（fallback split 既有面）；v2 加前导后 got `ghi<p>ghi</p>`
→ 终态 `<p><b>def</b>ghi</p>`——abc 越删 + 新块边界取尾错位。
缺陷落 `_zwDelDeleteSelection` 元素边界 range 处理或删除后 caret
重锚面，非本刀引入（基线同缺 abc）。两案 judge 态 F→F 恒一，
不触碰红线；挂账独立刀（与 delete 域 element-boundary 族同查）。

## 门禁首轮（batch-knife11-v2-verdict.sh + battery2，15:23 起飞 head=c700bfb2a）

批前断言全过：build-anchored md5=`d74e396efa32e4086b051cea0266a09f`（=c2
探针锚）；knife11 marker paren old=0/new=3 + noparen old=1/new=1 恒一；
v2 特征串 tcap=7/dive=4/wraptag=4/delead=1。

**mt 全分母终判（knife11-v2-mt-post.log，raw 19,936P/248F = 20,184 行全分
母）**：judge vs knife11-mt-post.log——**F2P=18 / P2F=0 / post-only=0 /
lost=0 / rows 10,092 恒一**，红线三项全零。raw F 284→248 = 18 键 ×2 窗位
行恰合，键级行账内部自洽。

F2P 18 键逐键归因（零 UNEXPECTED）：

- **styled typed-fate innerHTML 16 键**：b+i 壳族（`de]f`/`g]hi`/`def]`
  变体 8）、空 s 壳族（`<s></s><b><i>` 形 4）、img 首可见族（1）、裸 b 族
  （`abc<b>[def…</b>ghi` 形 3）——knife (a) 捕获补位 + (b) caret 入壳 +
  (c) 容器标签三面语料映射。16 键全走 PORT 面（caret 容器=b/i 元素在宿主
  内），(c) wrap-tag 容器复制产 div 形合录音。
- **qS-after 2 键**：`queryCommandState("bold") after`（`<b>[abc]</b>` +
  `abc<b>[def]</b>`）——typing delta ④ 快照回写面（insertParagraph 后
  typing style 存续）。

预期 25 键未全翻差额 7 键记账：styled 6 键 = 496/499 DC-4 形 mt 变体 +
更深壳变体（探针残红面，F→F 恒一不触红线）；createlink rich 1 键 = v2
处方未覆盖域（维持挂账簇）。残 F 248 行 → 残键池续刀。

**单文件腿**：iol VERDICT OK（rows/P/F 逐位恒等）、iul 恒一（2065 rows
F2P=0/P2F=0）；**ip 腿 VERDICT REVIEW：F2P=0 / P2F=2**（rows 7195 恒一，
P 6987→6985）。

**enginelib 名册**：3794P/6F，6 红逐名精确恒一 V2 ROSTER OK。

**ambient 七腿电池**：inline-host 880/0、chd 420/0、wss 1104/0、ilb 552/0、
join 3456/0 + 2607/0 + 1455/0 全零位移。

## ip 腿 P2F×2 定谳与 wrap-tag unset 面守卫

P2F 键：`defaultparagraphseparator p` + insertparagraph 的 td 跨格族
（`<table><tr><td>[foo<td>bar]…` 与 `<td>[foo]` 两案）。归因链：caret
容器=td 元素（宿主内）→ PORT → (c) wrap-tag 对非 collapsed 无条件容器
复制（宿主标签 div）翻置显式 sep 'p'——而 ip 语料 td 族录音**遵守显式
separator**（期望 `td><p><br></p><p>{}<br></p></td>` 形，ip.js :23/:30/
:68/:73）；div 变体两案因复制标签恰同为 div 未现形。16 styled 翻绿键
全部无显式 separator（multitest bootstrap 仅 styleWithCSS）——wrap-tag
规则（collapsed→separator / range→container-copy）本就 derive 自 unset
面语料，越界到显式 setter 面违 knife11 双面制「显式 setter 一票胜出」。

修复：`_zwIpWrapTag` 仅在 `__zwDefaultParagraphSeparator` 未设置态生效
（显式 'p'/'div' 时保持 null → wrap 臂用 sep 原标签）。探针补相 A2（td
四案 div/p 双变体）直验，复判门禁（同批链新锚）验证 18 键保持 + P2F 归零。

## 复判门禁

（探针 post-d + verdict/battery 新锚复跑——待出数回填）

## 判读结论

（待复判出数）
