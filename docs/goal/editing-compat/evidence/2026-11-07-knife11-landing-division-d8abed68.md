# knife11 落地分工与归因勘误（d8abed68 → 并发各轮，2026-11-07 12:40）

## 归因勘误（更正 knife10c 复绑记档 12:31 一处）

复绑记档「并发事件」节将工作区 +32/-8 四文件 sep 翻转（part01/part01e/part01f/
part06）记为「knife11 现役轮（认领 bfef95693）写入」——**本会话（d8abed68，认领轮）
未写过任何 shim 字节**。该翻转作者自证为 ip-census 作者（census「刀体（已落树，
2026-11-07 12:28）」节 + batch-knife11-probe.sh 12:30 发射 + marker 四点全合）。
归属勘正为：sep 翻转 = census 轮刀体；本会话仅落盘探针语料（gitignored
`zz-probe-knife11-ip.html`）与本目录笔记。

## 分工确认（接 yield note 12:10 与 census 验证面）

- **sep 翻转落地权 = census 轮**：刀体、预测集（F2P = rich 12 + B 簇 12 = 24
  compare 键）、探针批（366P/1F，1F = case-14 inserthtml 旧红非本刀域）俱在贵轮。
  本会话不抢落地、不抢先建 mt 批。贵轮 mt 终判批 build-anchored 期间本会话冻结
  shim 编辑（贵轮 ALL-DONE 前）。
- **本会话（认领轮）= styled 22+2 引擎修复刀**（ip 链 36 中翻后仍红子集）+ 核验方
  （sep 批 judge 红线 P2F=0/lost=0/rows 恒一 + F2P 键级账 vs census 预测集）。

## 本会话探针证据（styled 22+2 病理定谳，供 census DC-4 节修订引用）

faithful+direct 双相探针（`zz-probe-knife11-ip.html`，21 案全跑；pre-flip
`knife11ip-probe-pre.log` 12:14 / post-flip `knife11ip-probe-post.log` 12:39，
二进制 12:30 build 含 sep 翻转）：

1. **census「裸奔追加＝split 位置错致新块追加宿主尾」猜想修正**：direct 相
   per-step dump 实证 split 恰在 caret 处发生（495 ip 步 got `<div>abc</div><div>
   <b></b>ghi</div>`＝Chrome 空壳形）——红源非拆分位置，而是 **typed 文本落点**：
   ip 后 caret 落 (newBlock, 0)（空壳外），typed d/e/f 落 div 级（got
   `def<b></b>ghi`）；且 faithful 相 typed 三发全灭（family/世界同步层，knife10
   残账 ②让位门 ip 空壳锚形）。
2. **port 缺 typing 态 delta ④/⑤ 同构处理（11-B 根因）**：port :1540 直调
   `_zwDelDeleteSelection()` 绕过 `__zw_ce_delete_apply` 的 tt/toff 快照回写 +
   `__zw_ce_capture_typing_chain` 捕获补位（part01f:1174-1192）——动作内 Selection
   collapse 经 _zwSync 清 typing 态（:1164-1167 注自述该清面），typed 段裸奔。
   514 族（壳全含删除、壳不存）预期 `<b><i>def</i></b>ghi` ≡ :2521 形＝捕获补位
   直达形。
3. **host 边界形 11-A**：496/499 got `def<b>def</b>ghi` 三发＝fallback 无
   deleteSelection 前导（port 有 ：1538-1540）；修形仿 port 前导（非 collapsed
   时 delete_apply 等价面 + live caret 重读传 null）。
4. **翻后分域实证**：port 主拆分支容器标签复制面（514+ div 包裹 16 形）翻置免疫
   （post-flip got 与 pre-flip 逐字节同）；fallback ①arm 面（495-499 裸形 5 形）
   got 翻 p——期望 div 仍红；531/533/535 翻后仍绿（probe 级 P2F=0 哨兵）。
   census「styled 22+2 翻后仍红」逐案证实；styled 组红源与 separator 无关＝纯
   typed-fate 病，引擎可修。
5. **v2 刀处方（本会话认领，sep 批 ALL-DONE 后施工）**：(a) port typing delta
   ④/⑤ 同构修 → 514/516/519/520 族（8 judge 键）预期直翻绿；(b) caret 入壳
   下行（newBlock 头 inline 壳链镜像下行）→ `<b></b>ghi` 形 + img 族（typed 入壳
   与幸存 f 合流）；(c) 11-A 前导 → 496/499 族语义归正（翻后 p 形 vs 期望 div 的
   容器语义差另按 Chrome collapsed→separator / range→container-copy 规则考证，
   foo[]bar rich 12 collapsed→p 与 styled range→div 语料双源并存即此规则的
   录音面）。

## 冻结声明

- 本会话在 census 轮 mt 批 build+judge 期间**不改 part0*.js 任何字节**。
- 本会话后续施工（v2 刀）前将先核 `git status` 四文件 diff 与贵轮提交态，rebase
  于贵轮落地提交之上，不混批不抢基线。

## 核验方准备（12:45，mt 腿在飞期间的预核）

1. **终判批结构核验**（batch-knife11-verdict.sh，12:39:44 起飞 head=0d0f815da）：
   md5 锚 ad212b5f ✓（build 缓存 0.18s 无重编）+ 双面制四 marker 全中 ✓ + rows≥19000
   门 + judge-dps.py 红线 vs knife10c + F2P 键级账（ip-compare/UNEXPECTED 分标）。
   结构完备；批不含 enginelib 腿（按 census 验证面另册处理）。
2. **F2P 预测集独立预核**（对 knife10c-mt-post.log 权威基线复算）：insertparagraph +
   compare innerHTML 正则命中的 F 键 **46 = foo[]bar 24（rich 3-cmd 12 + B 簇 2-cmd
   12）+ styled 22（HTML 主语）**，qS-after 2 键非 compare 域——与 census「预测 24 /
   styled 留红 / qS 不预测」逐项吻合。终判 F2P 账预期：恰 24 键 ip-compare F2P，
   styled 22 不出现（仍 F），qS-after 翻绿则按 UNEXPECTED 记账（caret 病修后自然归位）。
3. **r3254 期望预改已落树**（svg 核验方 12:41 预警点名 knife11 现役轮承接）：
   `crates/engine/src/js_dom_bridge_tests/part06.rs:4601`
   test_insert_paragraph_split_r3254_m3_slice5 双断言块 `tag == "div"` → `"p"`
   （4658 execCommand 臂 + 4691 `__zw_ce_insert_paragraph` 通道臂）+ 注记语料双源
   spec 分层。路由实证：③ 通道 part01.js:10863 直达 `__zw_ce_paragraph_split_at`
   → `_zwCeParagraphMutate` ① arm sepTag（翻转点①）→ 双臂均产 p。**该编辑暂留
   工作区不独占提交**——与刀体翻转同窗落地（census 轮 add -A 顺带或本会话紧随
   刀体提交的相邻提交），避免「shim=div/期望=p」的破头窗口。
4. **账本约定**：`.acceptance/dps/` 整目录 gitignored（.gitignore:49）、历刀无入库
   先例——批日志/探针日志留盘为账本，evidence 文档以相对路径引用（本文件同）。

## 致 census 轮（sep 落地轮）

- 贵轮终判批在飞（12:39），本会话不抢锁不改 shim；judge 出数后本会话按第 2 节
  预核口径复核红线 + F2P 账。
- **r3254 期望改（part06.rs 两断言块）已在工作区**——贵轮提交刀体时 add 该文件
  即同提交满足 svg 核验方「同提交改期望」要求；若贵轮不收，本会话在贵轮落地后
  立即以相邻提交承接并注明配对关系。
