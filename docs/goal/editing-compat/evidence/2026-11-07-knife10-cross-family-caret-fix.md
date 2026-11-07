# 第十刀——inserttext 跨家族 caret 三修（2026-11-07，part01 caret_range/part06 尾簇 33）

第九刀收口（7cb7af38b）后续刀：mt 残 236 Fail 键普查归因 → A 簇（[值/样式命令，结构命令，
inserttext] 链）faithful 环境 caret 病三层定位。mt 终判 **VERDICT OK：F2P=70 / P2F=0 /
post-only 0 / lost 0 / rows 10092 键恒一**（F 236→166），零回归。

## 普查归因（faithful slice 探针三级裁决）

A 簇代表案 18 `[["bold",""],["insertorderedlist",""],["inserttext","a"]] "foo[]bar"`：直调
探针恒过，multitest 忠实环境（runConformanceTest + setup innerHTML 换代）恒红
`<ol><li>foobar</li><b>a</b></ol>`（typed 壳挂列表尾）。KNIFE10-DBG 三级 trace（iol 后内部
raw caret / 重锚两级 landed / 变异核 sc 身份）定谳三层：

1. **路由层**：execCommand inserttext 经尾簇 33 早臂以 `activeElement` 定宿主——setup
   innerHTML 换代 + 结构命令重锚后 activeElement 家族视图与选区家族视图分叉，变异核在
   activeElement 视图副本上回灌重建（`__zw_ce_caret_range` 世界同步），typed 段落副本树
   compare 侧不可见（faithful 案产零变异树）。其余全部编辑命令自选区爬升定宿主，唯
   inserttext 走 activeElement。
2. **重锚层**：跨家族让位后 ①登记表 miss（家族身份），②sel-path 重解析误开火——
   `_zwCeCaretAnchorStale` 对仍存活于宿主内、位置正确的活锚经 query wrapper 家族视图下行
   解析出身份翻异（跨族 churn 伪阳性），按登记路径重映射到包裹壳边界 (ol,1)。
3. **回灌层**：世界同步回灌以锚侧视图重建 el 子树时打死活锚（document.contains 真但
   el.contains 假），回植缺失使②接手 detached 锚落壳边界。

## 修刀过程（修一无效 → 修二定位路由层 → 修三收口）

- **修一**（活锚让位 guard `el.contains(startContainer)` 跳过②）：无效——world-sync 回灌
  先于 guard 运行，guard 看到的已是回灌打死后的 detached 锚。
- **修二**（回灌前捕获锚 path/offset、回灌后同位回植）：臂翻 textsplice（`#text@3`
  scInEl=true）但 compare 仍零变异 → el 身份 trace（elDet/elAe/aeTag/aeDet）定谳路由层
  根因（变异落 shim 家族副本），回植本身正确但宿主错了。
- **修三**（尾簇 33 选区让位门）：inserttext 宿主路由与 generic 分支（选区爬升）同径，
  分叉消失，case16（ilb 链）+ case18（iol 链）双翻绿。三修共存必要：让位门消除路由分叉，
  活锚让位门消除②误开火（章程面 detached 旧对象补偿语义保留），回捕获/回植保回灌场景
  caret 不丢（`_zwSyncCaretLost` 置位时仍走②补偿）。

## 刀体（三笔，+87/-1，KNIFE10-DBG 插桩收口前全剥离）

1. **尾簇 33 选区让位门**（part06 :4574 条件函数）：选区已有 range 且起点锚爬升至
   editing host 时让位 generic 分支（与其余编辑命令同径）；章程面（focus 未建 Selection
   range / 锚不在宿主内）行为不变。
2. **活锚让位门**（part01 `__zw_ce_caret_range`）：锚存活于本宿主子树内（el.contains）时
   跳过② sel-path 重解析、保留原 range。②章程（D3 注自述）是 setup/resync 换代后
   detached 旧对象残留补偿，章程案锚已移出宿主 childNodes 不受影响。
3. **回灌 caret 捕获回植**（part01 回灌块）：回灌前按 (锚→_phSync) 索引链捕获塌缩 caret
   逻辑位，回灌后自 el 同位下行回植并本轮跳过②；捕获失败（链断/锚离文档）不回植，章程
   行为不变。

## 验证链（probe + mt 终判 + enginelib）

- **faithful slice 探针**（zz-probe-knife10-slice，二进制 marker `_rrKeep×3/_syncCaret×9/
  _zwSyncCaretLost×3/_sel57×2`，dbg-absent 双检 0）：case 16（bold+ilb+inserttext）与
  case 18（bold+iol+inserttext）compare innerHTML 双翻绿；直调环境恒绿不受扰。
  pre-knife（knife10-slice2.log）A 簇链 compare 全红；终刀体（knife10-slice16.log）
  **366 Pass / 2 Fail**——case16 :369、case18 :408、formatblock :134 / indent :212 / ip /
  iul / jc 等链全 Pass，残 2 = case-14 inserthtml 链旧红（非本刀域）+ SLICE18 蓄意 dump
  行（got=exp 恒等）。
- **mt 终判**（batch-knife10-mt.sh，10:07-10:39，judge vs knife9-mt-post.log）：
  **rows 10092→10092 / P 9856→9926 / F 236→166 / F2P=70 / P2F=0 / post-only 0 / lost 0 →
  VERDICT OK**（raw 19,825P/326F；该批判读域与提交字节断链——rebase autostash pop 带入 shadow-dom part06 域改动 + 注释日期 sed 两层字节差，09cf4972→b5323c70，判读不绑定，终判以 knife10b 复锚为准，见下条）。
- **mt 终判复锚（knife10b mtrerun，11:03-11:37，head=49abc3e63）**：按红线处方 (b) 以现树字节态复跑（batch-knife10b-mtrerun.sh，flock 串行，build-anchored md5=b5323c701d8be72fb806fccfa21a70b2 断言过，dbg-absent 双检 0，marker _rrKeep×3/_syncCaret×9/_zwSyncCaretLost×3/_sel57×2 在位）：红线 judge vs knife9 基线 **rows 10092→10092 / P 9856→9926 / F 236→166 / F2P=70 / P2F=0 / post-only 0 / lost 0 → VERDICT OK**（与首跑元组逐位一致，raw 19,854P/330F）；漂移中性 judge vs knife10 首跑 **rows 10092→10092 / P 9926→9926 / F 166→166 → VERDICT OK**（行级恒等，字节漂移语义零差）。raw +33 行经 line-multiset 对账全落 multitest-ab.html delete/forwarddelete 表格链族重复发射（首跑 raw lost 33 同族），judge 键域恒一、末次发射态逐键恒等——变体/窗界运行产物域，非语义漂移。本刀 VERDICT 自此绑定提交字节态 b5323c70。enginelib 名册复核批（batch-knife10b-roster.log，11:39，head=49abc3e63 + 现树字节）：3795P/4F 红集与新档四红逐名恒等 **ROSTER OK**。
- **F2P 键级对账**（raw (file,assertion) last-wins 复跑定谳）：F2P=147 = 预测集命中 77
  （multitest-ab.html 族；a_keys 145 余 68 未翻——ip 链 36 / ilb 链 10 / inserthtml 链 12
  各有追加病害（ipara fallback `__zw_ce_paragraph_split_at` 裸奔 / ilb 变异核 /
  inserthtml apply），非本轮 caret 病域）+ 主语料 multitest.html?N-M 窗口族 70
  （[值/样式, 结构, inserttext] A 簇同形键；census a_keys 文件域仅覆盖 ab 变体，窗口族
  未入预测集——普查清单归档域缺陷，非计划外翻键）。judge 协议口径（键域 10092 恒一）
  F2P=70。raw 口径 lost 33（ab 变体族 delete 表格链未跑键）/ postF 1（?9001-last 窗界
  forwarddelete 链）为变体/窗界运行产物，judge 口径 lost=0 / postF=0 为红线判据。
- **enginelib**：零新增红；九红名册 9→4——五 CSSOM 序列化红（p56_adopted/p99s/add_rule_
  remove_rule/rule_style_r2810/stylesheets_write）确定性转绿（复跑 16 passed 0 failed，
  roster 注记的 net-api 流对齐已落 main），fetch×1/iframe-sw×1/ws×2 外债红恒一。
  全量 make test 复核（batch-knife10-maketest.log，10:47:49，rc=2）：3795 passed / 4 failed，
  四红逐名等同名册外债四红（fetch_filtered_response_type_matrix /
  iframe_service_worker_messages_dispatch / t8k_websocket_error_nonterminal /
  t8k_websocket_event_flow_open_message_close）→ 刀体零新增红定谳。**pristine A/B 实证**：
  stash 刀体后 eb933a55d 守卫内复跑 js_dom_bridge 全组 1364P/4F 逐名同集（pop 回植 md5
  双对上），四红与本刀 diff 无关定谳。按名册协议另立新档
  （enginelib-red-roster-2026-11-07-knife10.txt），原 2026-11-07 档不覆盖。

## 残账

mt 残 166 F 键 = inserttext compare 119（ip 链 36 + ilb 链 10 + ihtml 链 12 及其余 typing
壳面）+ insertText（F6 styled 壳重建）29 + insertparagraph compare 12 + 零散 6。A 簇未翻
68 另按选区形态三分（knife10-reconcile.py 可复算）：rich-sel 33（styled-edge/range/img
非素选区，27 个 inserttext 多发链在内——②让位门只保素文本锚）+ ip/ihtml 素选区 22（两
命令各有自管 apply 臂，generic 路径刀体够不到，需臂内同构修）+ createlink 13（`<a>` 包裹
干涉独立机制）；7 键孪生不对称（ab 单页翻绿、mt 分窗未翻）为单页/分窗上下文敏感 #37 同
族，judge last-wins 权威口径 P2F=0 无红线影响。普查簇
分类：B 簇 separator div→p 12（spec-impl div 默认与 Chrome 对齐需论证）；C 簇
delete+inserttext styled 壳重建——stash@{0} `typing-style-round-1102` 草稿已评估：patch
文本过期（part01f.js:428 context 漂移）apply 失败、语义部分被 `_zwDelKeepShellOf` 重构与
784e52047 吸收，留 stash 不销毁；D 簇 inline 壳内续写。按残键降序（ip 36 → ihtml 12 →
ilb/createlink 链 10）续刀。
