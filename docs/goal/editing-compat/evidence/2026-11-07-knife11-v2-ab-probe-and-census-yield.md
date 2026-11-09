# knife11 v2 刀 (a)/(b) 探针 VERDICT 与 (c) 施工让位记录（census 轮，2026-11-07 15:22）

## 刀体归属与字节态

v2 刀三笔按 landing-division §5 处方施工，工作区字节态按作者分段：

- **(a) port typing delta ④/⑤ 同构修（census 轮，~15:00）**：part01f.js
  `__zw_ce_ipara_apply` 三处——① `snap` 后 `tt`/`ttoff` 快照（delete_apply
  同构）；② `_zwIpCol` 后非 collapsed 面 `tcap` 捕获（capture_typing_chain
  同参）；③ `finally` 内 override 回填后 typing 回写 + 捕获补位
  （显式态存活优先，态空落 tcap，俱空保持原值）。
- **(b) caret 入壳下行（census 轮，~15:08）**：part01f.js 新增
  `__zw_ce_caret_in_leading_shell(block)`（首子 inline 壳链 _zwDelFmtTags
  表深行，命中返 `[容器,0]`）+ port 主拆分支尾 `collapse(newContainer,0)`
  改壳链命中则 `collapse(壳,0)`。
- **(c-v1) fallback 前 deleteSelection 前导（census 轮，15:16）**：part06.js
  insertparagraph 派发臂 else 分支三分——wants→port；collapsed 或无
  delete_apply→原 `split_at(host2, rng2)`；非 collapsed→
  `__zw_ce_delete_apply(false)` 后 `split_at(host2, null, false)`
  （live caret 重读传 null，delete_apply 内建 typing ④/⑤ 与 override
  重锚复用）。
- **(c-容器面) 并行轮（15:21 起）**：part01f.js `_zwIpWrapTag` 块
  （非 collapsed 面 wrap 臂块标签复制编辑宿主标签，collapsed 保持
  separator 缺省）+ 批脚本日志名 sed post-c → post-c2 + batch
  15:21 起飞。census 轮不触该域，本笔记落定让位分工。

## (a) 探针 VERDICT（batch-knife11-v2-probe.log，15:01:26-15:03:09）

build-anchored 四文件 md5=`c70be032d92912a423db87deca8a1b2b`；knife11 双面制
marker 0/3 + 1/1 恒一；tcap refs=7。21 案 verdict：**514/516/519/520 四案
ok=true（landing-division §5(a) 预测集精确命中）**，其余 17 案与 13:28 基线
逐案恒一（531/533/535 哨兵绿保持，P2F=0 哨兵零违）。

## (b) 探针 VERDICT（knife11-v2-probe-post-b.log，15:15:56 ALL-DONE）

build-anchored md5=`66c6630a8645a9a1937211189913f33d`。**(a) 四案保持绿 +
515/517/518（空壳 `<b></b>` typed 入壳填充）+ 529/539/545/547/553/555
（幸存壳 `<b><i>f</i></b>` typed 入壳与幸存内容合流）九案全翻 ok=true**，
哨兵三案恒绿，其余恒一——处方 (b) 目标集 9/9 命中，P2F=0。

## (c-v1) 探针结果（knife11-v2-probe-post-c.log，15:18:15 ALL-DONE）

build-anchored md5=`30346ea0cefb34a22e971aed163adc8f`。IP2-CASE verdict
与 post-b 逐案恒一（495-499 仍红，其余 16 绿恒一，P2F=0）。dump 归因：

- **CASE495（route=PORT）**：got `<p>abc</p><p><b>def</b>ghi</p>`——内容
  模型已完美（port 自有 delete 前导 + (a) 捕获 tt=["b"] + (b) 空壳 dive
  caret=B@0 typed 入壳），残差仅容器标签 p vs exp div——按 collapsed→
  separator / range→container-copy 规则归 (c) 容器面。
- **CASE496/499（route=FALLBACK rng=null(live-reread)，(c-v1) 分支已生效）**：
  got `ghi<p>ghi</p>`——delete 前导生效后 live 重读 caret 与 ①arm
  boundaryText 分支组合产出 **重复块伪影**（pre-c 为 `ghi<p>def<b>def</b>ghi`
  重影，前导消了 def 重影、暴露 ghi 复制新形）——fallback 前导语义需
  caret 落位对账，并行轮施工域，census 不并改。
- **CASE497/498**：形状未动（待容器面批判读）。
- 语料 marker 语义核：setupDiv（tests.js :5333-5395）对 `{}`/`[]` 同构建
  REAL range（braces 仅元素边界可置位的等价形式，非 collapsed 折叠语义），
  495-499 全部为真删除面。

## (c-容器面) post-c2 探针 VERDICT（knife11-v2-probe-post-c2.log，15:22:55 ALL-DONE）

build-anchored md5=`d74e396efa32e4086b051cea0266a09f`（= 汇合态四文件 recipe，
含 census (a)(b)(c-v1) + 并行轮 wrap 标签复制）；marker 双检恒一、tcap refs=7。
**21 案 19 绿：495/497/498 容器面翻绿**（非 collapsed 拆分 wrap 臂复制宿主
标签，exp DIV 形命中），仅 496/499 残红（fallback live-reread 双块伪影，
容器面触 port wrap 臂不达 fallback 拆分路径）。531/533/535 哨兵绿、
(a) 四案与 (b) 九案全部保持。并行轮终判批（batch-knife11-v2-verdict.sh）
已按该锚值 SHIM-DRIFT 断言 + mt 全分母 + ip/iol/iul 三腿 + enginelib
名册 + battery2 七腿 ambient 链起飞（15:23 mt 在飞），496/499 按 DC-4
记账预期不翻——census 轮对此分工无异议，转纯核验方。

## census 轮门禁链核验（16:22，verdict 批 ALL-DONE 16:12:01）

- **mt 全分母 VERDICT OK**：rows 10092 恒一，F2P=18 / P2F=0 / post-only=0 /
  lost=0；F 143→125 精确对账。18 键按族：16 styled innerHTML（b/i 壳
  typing 链，img 变体 + `<s></s>` 前缀 + `{}` 边界变体）+ 2 qS-bold；
  496/499 fallback 族未翻（DC-4 预期），createlink rich 键未现（待账）。
- **iol/iul 腿 VERDICT OK**（F 2→2 / 4→4 零位移）；**ip 腿 VERDICT
  REVIEW：P2F=2**（regress 文件 `v2-insertparagraph-vs-knife11.txt`，
  F 208→210）。
- **enginelib 名册 V2 ROSTER OK**（3794 passed / 恰 6 红）。
- battery2 七腿 5/7 OK 在飞（16:22 腿 6）。

### ip 腿 P2F=2 静态归因（census 判读，供施工轮取证）

两键均 `defaultparagraphseparator:p` + 非 collapsed table 选区：
`<table><tr><td>[foo<td>bar]<tr><td>baz<td>quz</table>` 与
`<table><tr><td>[foo]</table>`。dps:**div** 同形键恒一（两日志 Pass 对齐）
→ 病灶是 **dps:p 翻转被压**，非 table 删除语义本身。路由面：td 内层
caret → wants=true → PORT 域（(c-v1) fallback 前导不涉），非 collapsed
→ `_zwIpCol=false` → 并行轮 wrap-tag 覆盖
`_zwIpWrapTag = _zwJusTag(_zwListEditingHostOf(...))` 把 knife11 sep
双面制的 block-face p 翻转覆盖成宿主标签 div。容器复制规则（collapsed→
separator / range→container-copy）按语料考证只属 **host 级拆分域**；
td 内层拆分仍受 separator 语义统治。修复方向（施工轮裁量）：wrap-tag
覆盖加路由门（仅 fallback/①arm 域或 caret container==编辑宿主时生效，
可复用 wants() 的 `_zwJusIsEditable` 严格内约定谓词）。
**本键族不属 DC-4**（DC-4 = fallback live-reread 双块伪影），勿混账。

### 本树收敛动态（16:22 观察）

twin 已删两个与远端重复的 untracked learnings（与 census 预检结论一致）、
修 oof-float learning frontmatter（补 `---` 开栏）并重生成 INDEX
（bugs 244→246 / patterns 31→32）、落 arc-verify 账本笔记（探针弧线独立
核验与 census 判读完全一致：post-a 7 / post-b 16 / post-c 16 / post-c2 19，
残红恰 {496,499}，全弧 P2F=0）。第三执行体确认为**另一 clone**
（ZeroWeb-2，site-opt 轮 post-merge make-test-remainder），与本树无涉。

## 勘误（16:55，census 轮对自身两处结论的修正）

施工轮 root-cause 笔记（verdict-verify-p2f-root-cause.md）证伪本笔记
前文两个结论，按「保留历史、勘误留痕」原则修正如下：

1. **「非 collapsed → container-copy」假说证伪**：本笔记多处引「语料
   考证容器复制规则」。实测根因：vendored spec insertParagraph
   （implementation.js :7574 非 SLC 臂）新块标签恒取 separator（Chrome =
   dps 设置，spec 缺省 div）——mt styled 16 键翻绿非 container-copy
   命中，而是 mt 语料 `defaultparagraphseparator` 零命中（grep 实证）
   dps=unset 时宿主 copy 恰合 spec 缺省 div 的巧合等值。
2. **ip 腿 P2F=2 修复方向修正**：本笔记建议「路由门（仅 fallback/
   host 域）」；终处方为 **dps-unset 门**（显式 separator 一票胜出回落
   sep 产线面，unset 面保 host-copy），dps 门同时关闭「非 td 显式
   dps + PORT」语义缺口。本会话归因的病灶定位（wrap-tag 越权压 dps:p
   翻转）与键面证据（仅 dps:p 翻红）仍成立，仅机制层修正。
3. 施工轮已按终处方改 part01f.js（16:34:42）并拉起 fix-gate 批
   （ip 腿→mt 全分母→enginelib 复跑；红线：mt F2P=18 恒一全零红线 +
   ip P2F=0 复位 + 名册恒一；新 md5 锚 build 时现算）。本会话继续
   核验方，fix-gate 出数后判读回执。

## 分工定谳（本笔记核心）

- census 轮持有 (a)/(b) 探针 VERDICT 与 (c-v1) 派发前导刀体；
- 并行轮持 (c) 容器面与 fallback 前导后续施工（post-c2 批 15:21 在飞）；
- census 轮让位转核验方：post-c2 出数后按元组口径判读（21 案逐案对账
  post-c 基线 + P2F 哨兵），mt 全分母门禁批待施工面稳定后由施工轮跑或
  双方协商归属，判读结论回填本笔记或后继笔记。
- 共享树收敛（origin behind 3：3b9b22919 learnings + 59c3cb32f
  wipe-recovery 补记 + a812e9852 SMIL B5——其中 a812e9852 触 part01.js
  +11/part06.js +15 与刀体文件相邻）**待批链静默后执行**，autostash 前
  核远端 hunk 与在飞脏区不相交。
