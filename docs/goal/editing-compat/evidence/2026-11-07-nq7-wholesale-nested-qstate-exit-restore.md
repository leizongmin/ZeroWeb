# 2026-11-07 wholesale 同型嵌套跨入形出口恢复刀（nq7）——读面挂账族收口 + 伪影除名勘误

日期：2026-11-07。基线 = nesh 两刀后账面（iol 4F / iul 6F，`batch-nesh2-*-post.log`）。目标族 = 控制面「下一步①」读面挂账族（原挂账 ×5 + ×2）。**终判 = 勘误后真债恰 4 行全修，两伪影除名；iol 4→2F / iul 6→4F。**

## 挂账族勘误（开工前窗界核对）

控制面 ① 面原记「同型嵌套 qState ×5 + 跨列表读面 ×2」。逐行窗界分布核对（sr2 轮签名 A 先例同法）定谳：

- **真债 ×4**：iol `:448`/`:478` + iul `:233`/`:263`——`<ol|ul><li>[foo<ol|ul><li>bar]</ol|ul><li>baz</ol|ul>` 同型嵌套跨入形，qState-after 录音 false、引擎读 true。
- **伪影除名 ×2**：iol `:393`（`<ul><li>[bar]` nested 前奏形，qState-after）+ iul `:428`（`b[ar</ul><li>b]az` 跨列表形，qIndeterm/qState-after）——断言块跨窗界，exec 在前窗跑过、后窗对未执行 setup 树直读恒假红（签名 A 复发，[sr2 定谳](2026-11-06-sr2-exit-selrestore-straddle-artifact.md) 同签名）。**探针 in-window 复核**（本轮 `zz-probe-nq7`，exec 真跑）：iol-393 树逐字合语料期望 `<ul><li>foo</li><ol><li>bar</li></ol><ul><li>baz</li></ul><li>quz</li></ul>` + qSA=true；iul-428 树逐字合 `<ul><li>foo</li><li>bar</li></ul><div>baz</div>` + qI-iul=true/qS-iul=false + 跨界选区保留——引擎无辜，除名不追。勘误回写控制面（真债面从 ×5+×2 修正为 ×4）。
- qVb ×1 / divbr ×2 窗界伪影维持不追（既有账）。

## 机制（探针 + 参考算法双证）

真债形 enable wholesale（嵌套被换壳整壳 retag）后，引擎选区塌缩停在嵌套 li 内（探针实证 sel=`#text"bar"<LI<UL<LI<OL@3`）；P10 qState 端点直读面按端点爬祖先得 true。语料录音 qI=false/qS=false＝`'none'` 签名＝**选区停列表外**（P10 定谳「after 录值含命令史态＝选区恢复耦合」同族）。参考算法核验：marker 忠实选区（foo@0..bar@3）下 `getSelectionListState`（implementation.js:3949 every-ol 子句）本应返 "ol"/true——录音 false 证明 Chrome 原生录音的选区在 exec 后被扰到列表外，与 qv13 轮「语料 qV 面两录音源混合」同一 provenance 结论。**刀口 = 出口选区恢复**（复现录音扰动位），非读面改动。

## 刀（part01e.js 两笔）

- **入口快照**：`nqOut` = enable 模式 ∧ 起点 li **严格包含**终点 li（同壳内选区跨嵌套边界）。census 防扩面：真债恰 4 行命中；dps 族同形源全部 qSB=true 走 disable 面被 enable 门天然排除；塌缩选区形（:393/:418 族 endLi==startLi）与端点反向形（:453/:428，起点 li 不含终点 li）均不命中。
- **出口恢复**：`nqOut` 触发时把两端点收拢到最外层换壳列表树位的宿主层锚点（`retaggedFinal`/`shellSuccessor` 解析后爬出 li/ol/ul 链到非列表族 parent——wholesale 双壳 retag 的落位在嵌套壳，须爬到宿主层才得 'none' 签名；排他跳过 q13 常规收钳，其列表内判定对塌缩嵌套位恒真恒跳过；锚点解析失败放行常规道维持既有行为）。

## 探针（zz-probe-nq7.html 十案，wpt-data 本地 gitignored）

判读按 [全断言面纪律](../../../learnings/bugs/2026-11/2026-11-06-probe-assert-full-row-family.md)（树 + 选区 + qreads 三方对齐）：

| 案 | 判据 | 结果 |
|---|---|---|
| iol-448 / iol-478 / iul-233 / iul-263 | 树逐字合语料期望（`:448` `<ol><li>foo<ol><li>bar</li></ol></li></ol><ul><li>baz</li></ul>` 等）+ qreads 双 F + sel 塌宿主层（'none' 签名） | 4/4 ✓ |
| iol-393 / iul-428（伪影 in-window 复核） | exec 真跑树逐字合期望 + qreads 合录音 | 2/2 ✓ 引擎无辜 |
| G-iol453（反向跨入守卫，语料 :453 现绿） | 刀不触发，树/选区/qSA=true 恒等现状 | ✓ |
| G-38（sr1 出口收钳承重行） | qSA=true 恒等 | ✓ |
| G-343-before / G-343-exec（before 悖论面） | 恒等现状 | ✓ |

首两轮探针 SETEXC 为探针自身 baz 端点路径笔误（外层 ul 恰 2 个 li，[0,2,0]→[0,1,0]），非引擎面。

## 门禁（batch-nq7-legs，systemd user 单元 + flock + MemoryMax=24G + TimeoutStartSec=90min）

| 腿 | 结果 | judge（vs bq 基线） | VERDICT |
|---|---|---|---|
| iol | 1,759P/**2F** | F2P=4（nesh :688/:768 + nq7 :448/:478）/ P2F=0 / lost 0 | OK |
| iul | 2,061P/**4F** | F2P=6（nesh divb 3 + NE :860 + nq7 :233/:263）/ P2F=0 / postF 0 / lost 0 | OK |
| enginelib | 3,782P/**9F** | 八红名册 + 1 漂移（见下节，bracket 归因外部债） | DRIFT→归因后 OK |
| mt | 19,331P/**837F** | 行级键级恒等（rows 10092→10092，F2P/P2F/postF/lost 全 0，与 nesh2 批逐位一致） | OK |

基线 = `batch-bq-*-post.log`（bq 刀后账面；nesh2 刀 F2P 已含基线对比时序）。二进制 marker：`nqOut=3`。批脚本 `.acceptance/dps/batch-nq7-legs.sh`。

## enginelib 第九红 bracket 归因：net-api M3 auth 改契约，s180 红钉测试未同步（外部债移交）

名册沿革：五红（⑫ fix12b 轮起，p56_adopted/p99s/r3276/r2810/r2809）→ +ws×2（divb 轮归因 ba03fd7ff）→ +fetch×1（divb tip 复核归因 f021e11fb）= 八红名册。本批 enginelib 九红，漂移项 = `test_iframe_service_worker_messages_dispatch_to_iframe_container`（part23.rs:1079），**六代批 enginelib 腿（batch/divb/nesh/nesh2/bq/rtl）恒绿后首红**。

归因链（bracket 双证）：

- **语义变更**：4615680c0（2026-11-06 23:23，`fix(xhr): M3 auth 五簇引擎语义收口`）把 URL userinfo 凭证从 s180「send 层首发注入 Basic Authorization」改为 **M3 延迟注入**（part03.js:772-804 凭证捕获 + userinfo 剥离；Basic 唯一赋值点 :1193 在 401 挑战重试臂内）。该测试的 mock `__zw_fetch` 直返 200 无挑战 → 重试臂恒不触发 → 断言 `contains "authorization\x1eBasic YWE6YmI="` 恒假红——**新语义下确定性失败，非 flake**。
- **测试未同步**：part23.rs 该断言自 s180（2e260d532，10-31「红钉断言改钉根修语义」）后未动，仍钉 s180 首发注入契约。
- **时序 bracket**：nesh2 批 enginelib 绿（23:13，head=99969d8cb，早于 4615680c0 的 23:23:10）→ nq7 批 enginelib 红（00:31，head=e5a219f1f，含 4615680c0）——两代间该测试面唯一变化即 4615680c0。本刀（part01e.js 列表 toggle 局部 var，无全局写入）与 XHR auth 面无机制接触；3,782P 通过 + 恰一名册外单红也排除解析级破坏。
- **决定性 A/B 终证**：stash 本刀后 clean HEAD（e5a219f1f）单测复跑同位 FAILED（part23.rs:1079，0 passed/1 failed，3,790 filtered out，log `nq7-ab-cleanhead.log`）——红为 main 树态（4615680c0 引入），本刀完全无涉。

处置循 divb 轮 ws×2/fetch×1 先例：**非本刀回归，测试对齐属 net-api/xhr 面，挂账移交**（net-api 证据档 `2026-11-06-m3-xhr-auth-clusters.md` 无 enginelib 桥测记录，同 divb 轮「零回归审计面为 WPT harness、rust 桥测试漏审」盲区模式）；名册更新九红（本地批脚本维护）。

## tip 复核（rebase 组合基面恒一）

push 前 rebase 并入远端五提交（xhr part03 getAllResponseHeaders legacy-uppercased 排序 4f27056e2 + embed s316 paint 门 81d215227 + wpt-runner cors fixture 46a6e20f3 + s317 print-reftest 0a507685a + 渲染账目），共享 shim bundle 变化，按 nesh2 tip 复核先例对组合基面重验双腿：iol 1,759P/2F（F2P=4/P2F=0）+ iul 2,061P/4F（F2P=6/P2F=0）与门禁批逐字一致，**TIP-VERDICT OK**（log `nq7-tip-recheck.log`，tip head=9aafcb04e，二进制 marker nqOut=3）。
