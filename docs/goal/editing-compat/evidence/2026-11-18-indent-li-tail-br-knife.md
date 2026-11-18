# indent F 族 li 尾 br 尾位持行刀（R21 收口档）

日期：2026-11-18 ｜ 刀体：`d51bc6d55`（part01c.js wrap 尾清理域局部豁免 v2 前邻界判别＋part142 钉五形）｜ 门禁批：r21b 八腿全绿 17:02 @bin `18398dd15d2d0e9336447b9b299f3812`，推后组合 spot check indent 1329/0＋lib 4188/0 @`d51bc6d55`

## 根因与刀面

indent 包 ul 后 `_zwJusWrap` 尾清理 `_zwJusRemoveExtraneousLineBreaksAtTheEndOf`
下钻 newParent 摘 li1 尾 br（语料 :802 `<ul><li>a<br>{<br>}</li><li>b</li></ul>`
got 摘成 `a<br>`）。机制：共享 extraneous 判定的 precedes 免布局近似在块尾 br
形过判——前走出块爬升后 BlockStartPoint 看到的前邻可见块正是 br 自在块，撞块
边界报 true；vendored :840 用 height 测量（隐 br 后 li 三行塌两行）判非冗余。

刀面：wrap 尾段新增候选定位（镜像 AtTheEndOf 下钻＋隐形回走）＋
`_zwJusBrHoldsTailLine` 尾位判别，持行 br 跳过 AtTheEndOf 摘除；Before 清理
方向不同不受豁免。豁免只落 wrap 尾段（域局部）：全局判定豁免实证翻红 8 枚 ip
拆壳钉（insertparagraph F-right/i6 族，语料 :1163 caret 拆分形靠现判定摘除）——
Chrome 语义按命令域分叉（wrap 保留 vs ip 摘除），共享 extraneous 判定不动。

## v1→v2：门禁批 L1 ABORT 实证的前邻界修订

v1 判别「自身无后继＋上溯含块边界各层祖先均无后继」把
`<p>foo<br>{}</p><p>extra` 前邻可见文本形也豁免——该形尾 br 仅止 foo 行，
摘除行数不变＝extraneous 须摘（Chrome 录形 `<blockquote><p>foo</p></blockquote>`
无尾 br）。r21 v1 批（pid 2171196 @9214474ae）L1 indent 1328/1 该键翻红 ABORT。

v2 改前邻界回走判别：前邻可见节点是 br、或回走到含块/editing host 边界
（br 为该块首行唯一持高者）才豁免；前邻普通可见内联内容（文本/img/带内容的
内联壳）不豁免走摘除原径。回走遇不可见节点下钻其末子代续走，守卫 256。
part142 增 pD 前邻可见文本摘 br 控制键（钉五形：li 三选区收敛双 br 全留形＋
无尾 br 基线＋pD 摘 br 形）。

## 门禁批（r21b ALL-GATE-PASS）

判读锚＝r21a fresh 锚七腿（r21-anc.sh @98eceb3a7 pre-knife）：
indent 1328/1（残恰 F 族键）＋fb 5043/3＋fs 2419/3＋img 962/0＋event 176/4＋
ie 1942/1386/3（重锚态）＋mt 10090/2（anc2 补腿实测）。

- r21a L6 ABORT 归因＝窗内 d54 刀 b50dc5f38 claimed face 恰合
  （joining-dl-elements 142＋dil 200＋jdla 48＝390 翻绿零新增红），R19 d53
  先例归因后重锚；收养轮补核 is-ancestor：d54 ∈ 98eceb3a7 锚基，自洽。
- r21b（pid 2198733 @25a1405fb，八腿 bin 18398dd1 全程恒一）：
  indent **1329/0/0** 翻绿恰 F 族键整行集合＋零新增红；fb 5043/3、fs 2419/3、
  img 962/0、event 176/4 全 SET-IDENTICAL（L3 前 css 流 knife-fv1 租户
  ensure_solo 串行等待）；ie 1942/1386/3 SET-IDENTICAL 对重锚零翻绿零新增红
  ＝本刀零 ie 面实证；lib **4179 passed / 0 failed** no-skip pin-in 11/11；
  mt **10090/2** multiplier2 压轴，Fail 集恰 X37 双键零暴露。
- 脚本尾注：r21b-fbr.sh:215 `echo ... $F ...` 引用未定义变量（v1 遗留笔误，
  局部变量实名 FM），set -u 于八腿断言全过后、终态 banner 前中止——ALL-DONE
  行缺失非门失败，逐腿 OK 行与腿日志为证；后继批脚本尾行禁再引用 $F。
- 429 亡故轮收养链：writer 轮 13:28 429 亡故批 setsid 存活→后继轮收养判读
  （anc2 L7 mt 补腿 14:30 兑现 10090/2 零漂）→发射前 pull --rebase 误序
  （早于门禁批改写刀体 hash 9214474ae→3ea1f4aaf）→update-ref 复原＋上游窗
  staged 内容逐字节快照留档（r21-prelaunch-upstream-index-backup.patch）→
  按账本「批先行、rebase 后置」恢复协议。全程见
  `.acceptance/dps/r21-ledger.md`（未跟踪，收口后随档失效）。

## 推送与组合窗 spot check

刀体 push rebase 并入 fv1/A3r/A3b 窗（origin c93818c37→daddfdfed：
7effb5d25 @font-feature-values 全链＋daddfdfed animation.rs fmt）后定锚
`d51bc6d55`。推后组合 spot check @d51bc6d55：lib 4188 passed / 0 failed
（＋9＝上游 fv1/animation 钉，零失败）＋indent 1329/0/0 rc=0——F 族键组合窗
维持翻绿零新增红。

indent 语料 1328/1→**1329/0 域清零**。
