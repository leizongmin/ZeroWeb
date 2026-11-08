# F6b sub/sup×fontsize carry 刀收口（2026-11-09，③ F6b 门禁批）

前置分析：[2026-11-08-f6b-script-fontsize-carry-analysis.md](2026-11-08-f6b-script-fontsize-carry-analysis.md)（活体普查 14 键三分族 + oracle 12 案定谳 + 语料统一规则 + 引擎三臂函数级落点）。

## 刀体（三臂四 shim + 公共机具）

| 臂 | 通道 | 落点 |
|---|---|---|
| C（指定值） | fontsize 命令重放面 | part01d：`_zwVaKwBucketF6b` 桶等价门 + forceValue 两道等值门壳级缩放口径（0.83em 后比较），非 script 保旧口径零行为差 |
| A（删除携带） | capture→typing 内层 | part01f：`__zw_ce_capture_script_carry` 三形分账（壳内 font 自身档 / walk_px 直走桶 / armed-only +inner 旗标）；part01：`_zwCeTypingInsert` f6b 内层臂裹 text（tags 循环前），`_zwCeF6bArmedBucket` 仅 consume `carry.inner`（A 通道独占） |
| N（嵌套树幸存） | post_insert 外层 | part01：`__zw_ce_f6b_post_insert` 外层幸存 font[size] 剥离 + 异桶重放/幸存内层 attr 提出/等桶溶解 |
| 公共 | — | part01：`__zw_f6b_bucket_of` 最近档距离式（sizeMap {1:10,…,7:48} 同源 part01d :180 静态表）+ `__zw_f6b_walk_px`；part06：`_zwSync` 清 `__zwTypingScriptCarry`（选区变更即清，防跨动作陈旧 carry） |
| 收集器（连带） | 跨壳边界 delete | part01c：`_zwJusCollectContainedNodes` 含界判定（spec #contained）+ stop=null 卫兵——N4/N5/N6/W2 跨壳边界 delete 空转根因（learning [delete-selection-cross-shell-noop](../../../../docs/learnings/bugs/2026-11/2026-11-09-delete-selection-cross-shell-noop.md)） |

fixture：`zz-probe-f6b.html` 17 案（A4+C5+N6+绿见证 W1/W2——C 族含 qV-after 断言面 r2 补全；wpt-data gitignored 需 add -f，19 个 zz-probe 先例均 tracked）。

## 首发与修复链（两连废后重发收口）

- **v1 发射（2026-11-08 23:5x）**：三臂落盘，四文件字节绑定 md5 = part01 4d29835a / part01d 71b8bcbc / part01f a729e4c2 / part06 8a5c039c（此为首发锚，后经修复链作废）。
- **首发后 diag 修复链**（D1-D7 诊断 + E1-E4 修复 + TRACE 对拍实证，probe-f6b solo 16/16 rc=0）：bucket 中点扫描整档差一档改最近距离式；capture 三形分账；ArmedBucket 独占 inner；f6b 内层臂改裹 text；forceValue 壳级缩放口径；part01c 收集器两道缺陷（含界+stop 卫兵）。重锚：part01 17fda0ae / part01d 411d5e9e / part01f fb6530be / part06 8a5c039c。
- **重发①（00:36）静默死亡**：非 setsid/环境问题——批脚本门行字面量创建期转录损坏（四锚 if 行 56 字符 = 32 位 md5 + 24 位垃圾尾，恒 ABORT），输出走 exec 重定向 gate.log 故外层 run.log 0 字节。python 机械截断四处 56→32 hex 修复，bash -n OK，刀体 md5 复核与锚一致。
- **重发②（00:42:31，pid 4052563）**：过四锚门入基线核，mt 腿前全腿绿（probe 五腿、ip/inserthtml/exec×3 零漂移）。
- **重发②批 ABORT（01:24，mt judge 红线拦截）**：F2P=14 恰机制预期，但 P2F=2 违反零回归红线（v1 脚本 sed 门 P2F 0 不匹配→空值越界报「F2P 越界」，消息误导，实为 P2F≠0；归档 [batch-f6b-script-fontsize-gate-r1-abort.log](../../../.acceptance/dps/batch-f6b-script-fontsize-gate-r1-abort.log)）。P2F 2 键 = C1/C2（fontsize 2↔subscript 互序）`queryCommandValue("fontsize") after` expected "2" got "3"。
- **根因（r1 P2F 定谳）**：等桶溶解移除 font 壳后树为裸 `foo<sub>a</sub>bar`，qV 回落缺省通道；shim host gCS 与 plain 走查双通道盲读 UA `sub,sup{font-size:0.83em}`（style-system UA 表 shim 侧不可见，learning #3 同源盲区）→ 16px 过 legacy 中点映射得 '3'；语料/Chrome computed 语义 16×0.83=13.28px→'2'。probe 16/16 与 mt 红 2 键并存的机制：probe 内 C1/C2 qV-after 走 part01d 溶解臂 override 重绑通道（override 存活），mt 流程 override 失效后走文档态回落暴露盲读。
- **r2 修刀（2026-11-09 01:30-01:40，双机制同靶冗余守备）**：① part01c `_zwValEffectiveValue` fontsize 缺省回落位（v==='16px'）script 缩放——sub/sup 祖先 ∧ 无显式 font[size] 载体 → 13.28px（载体在场的 16px 是 attr 派生真值不缩放，C3/C4/A/N 族零扰动；r1 全量矩阵实证影响面恰 C1/C2 四断言行）；② part01d 溶解臂 qV 载体重绑（`f6bDissolvedKw` 收集→caret 塌到位后 `_zwVaSetValueOverride`，override 通道修复）；③ probe fixture 补 qV 断言面（C3/C4 qV-after "3" 载体不缩放护栏 + C5 静态无 override 案 "2" 纯文档态读=①的直接闸门），16→17 案；④ 门批脚本重锚（part01d 9fbab2af 含重绑新字节）。

## 判读绑定（learning #28 字节锚定）

- shim 源锚 mt 后复读一致（批脚本内建门；r2 锚五文件含 part01c——r1 漏锚修正）。
- 执行二进制 `target/release/zero-wpt-runner` 构建 01:34:13（r2 双修复后重建），早于批内全部腿；五 shim mtime（part01 00:12:55 / part01c 01:31:50 / part01d 01:30:30 / part01f 00:34:57 / part06 2026-11-08 23:49:06）均早于二进制，cargo freshness 保证字节绑定；r2 特征串在体：part01c `hasCarrier`×3、part01d `f6bDissolvedKw`×4。
- **part01c 行为绑定（其锚缺位补强）**：solo 预验 [f6b-r2-probe-solo.log](../../../.acceptance/dps/f6b-r2-probe-solo.log) 17/0 rc=0——C5 静态案过 = 缺省回落位缩放在体（未修时 '3' 必红）；C3/C4 qV-after "3" 过 = 载体存在性分派门在体（仅缩放无载体门的首版修复必红）。
- 刀体特征串（r1 期）：`__zw_f6b_walk_px`×9、`_zwCeF6bArmedBucket`×4、`__zw_ce_f6b_post_insert`×5、`_zwVaNodeInScriptF6b`×2、F1'-B `_zwCeSwcssAbsorbSpan`×2、F2 `_zwCeF2LinkContext`×3。

## 门禁结果（批次链：v1 r1 拦截 → r2 修刀 → v2 绑定断裂 → v1.1 全红线过+校准白名单缺口 → v1.2 白名单收编全链收口）

- **v2 批绑定断裂 ABORT（01:4x，受控）**：发射后判明 probe 腿跑 16 案版 fixture 而 fixture 于批中 01:40:11 被并发写者翻为 17 案版（r2 补 C5 案），且脚本四锚无 part01c——绑定断裂即受控 ABORT（pgid 组杀+孤儿 runner 点杀+LOCK-FREE 验证），原档 [batch-f6b-script-fontsize-gate-v2.log](../../../.acceptance/dps/batch-f6b-script-fontsize-gate-v2.log)。
- **v1.1 批（01:55:31，pid 4102078，五锚+fixture 锚+二进制特征门）**：锚门过 → probe-f6b **17/0**（C5 静态 qV 闸门案过=r2 修复①在体）→ 护栏 f1b 7/0 / f2 14/0 / f3 7/0 / probe 224/17 → ip 7195 / inserthtml 1565 / exec×3 1806/1/6 恒一零漂移 → **mt 腿 rc=2 Pass=20082 Fail=102（17 预期红所致，rc 非门）→ judge rows 10092 恒一，P 10026→10040，F 66→52，F2P=14，P2F=0，post-only 0，lost 0，VERDICT OK（DC-4 红线全过）** → mt 后六点 md5 复读过 → 校准 entry 级翻键 28（窗口 [14,28] 内）但白名单按 14 census 键构建仅命中 20，**白名单外 8 → ABORT**（归因见下节）。
- **v1.2 批（02:46:39，pid 4133652，白名单 18 键=14 census+4 B-table）**：锚门过 → probe-f6b 17/0 → 护栏 f1b 7/0 / f2 14/0 / f3 7/0 / probe 224/17 judge OK → ip 7195 / inserthtml 1565 / exec 1806/1/6 恒一零漂移 → **mt rows 10092 恒一，P 10026→10040，F 66→52，F2P=14，P2F=0，post-only 0，lost 0，VERDICT OK** → mt 后六点 md5 复读一致 → calibrate entry 级翻键 28 全落白名单 28/外 0（分键 A=2/2/2/2、C=2/2/2/2、N=0/0/2/0/0/2、B=2/2/2/2）→ iol 1761 F2→2 / iul 2065 F4→4 恒一 VERDICT OK → **enginelib 名册门 ABORT：3816P/7F vs 6-red 既有集，新增 `test_response_body_readable_stream_r2967`**。日志 [batch-f6b-script-fontsize-gate-v1.2.log](../../../.acceptance/dps/batch-f6b-script-fontsize-gate-v1.2.log)。
- **enginelib ABORT 归因（三重证据，刀外外部债）**：①03:34:44 五 shim 被批外整文件 checkout（批中字节战首次抹刀），enginelib 腿 `--compile-first` 以漂移字节重编，该腿判读无效（mt 及以前腿绑发射锚字节仍有效）；②净树 bracket 两轮独立收敛——stash 五 shim 后 HEAD 复跑恒 3816P/7F（r2967 在，含清代理 env、cargo clean 重建），刀无罪；③s347 自验证记录「3817P/6F」与其树字节矛盾（疑构建锁等待继承二进制），bracket 收窄至 08f55becb(s347) 或更早。**r2967 定谳外部债**（fetch response.body ReadableStream 域，与 editing 刀域距远；嫌疑 252fbb3fd/30584b1e7 净域提交），名册档升级 7-red 带日期注记。
- **v1.2b 尾批（并行核验方，03:36 发射）**：绑定核验通过（二进制 mtime 03:40 特征串 f6bDissolvedKw×4 / inScript&&!hasCarrier×1=锚版）→ enginelib 名册门 **OK(7-red)**（3815P/8F，第 8 红 `paint::tests::r5296_single_line_inflated_lh_span_bg_box_level_content_anchored` 批负载 flake——solo 复采样 2/2 绿 0.01s + quiet 全量 enginelib 7F 双复采样（03:35 rerun1 14.21s / post-rebase 13.53s）paint 红零复现，非真红）→ battery 七腿：前五腿全绿（cannot-have-div 420 / inline-host 880 / with-white-space 1104 / linebreak-white-space 552 / join-paragraphs 3456 rows，F=0），**末两腿（join-…left-line-and-right-paragraph / …left-paragraph-and-right-line）rc=2 P=0**——ghost-br 轮 stash pop（~03:45-46）在 js_dom_bridge.rs:1478 留冲突标记致重编译 Error 101，编译失败非行为红（语料在 editing/other/ 完好），UU 清除后 solo 补跑双绿——leg6 `…left-line-and-right-paragraph` **2607P/0F/0T rc=0**（commit ① 树重建二进制）、leg7 `…left-paragraph-and-right-line` **1455P/0F/0T rc=0**（rebase 后新树重建二进制；首次 rerun 档 306B=fetch-wpt-editing 依赖链 Terminated 基建波及无测试执行，不作数），battery 七腿终态全绿。日志 [batch-f6b-script-fontsize-gate-v1.2b.log](../../../.acceptance/dps/batch-f6b-script-fontsize-gate-v1.2b.log)。
- **v1.2c 残余批（写位轮单链收口，04:02:08 发射 head 206678d4b）**：锚门（part01 `7bab520f`/part06 `d13021a7` 为 rebase 后新锚运行时记录；part01c/c/d/f+fixture 恒一门+刀体特征串 6 项 5/4/5/3/4/2 全对）→ **enginelib 全名册 3816P/7F ROSTER OK(7-red)**（r2967 复核仍红=外部债口径维持）→ battery 末两腿补跑 **leg6 2607P/0F/0T + leg7 1455P/0F/0T**（与核验方独立复跑及 2026-10-31 门禁基线三方逐位恒等，battery 七腿全绿收口）。首发射 03:57:09 enginelib 腿 rc=127 ABORT=`target/test-guard` 并行 make 重编译 exec 竞态（同名先例见 memory test-guard 自 exec 竞态），二发干净 ALL-DONE 04:07:30。日志 [batch-f6b-script-fontsize-gate-v1.2c.log](../../../.acceptance/dps/batch-f6b-script-fontsize-gate-v1.2c.log)。
- **门盲点（硬化建议，非本刀阻塞）**：名册门 R 提取仅 grep `js_dom_bridge` 前缀——跨模块（paint 等）真实回归将静默过门（v1.2b 8F 即例）。后继刀脚本建议双门：`test result` 行 failed 总数 == EXPECT 数 + 全量 failures 块对比。

## 字节战与 git 固化（commit ①）

- 批中两次抹刀（03:34:44、03:5x 窗，均未认领）+ 两轮独立恢复（并行写者 stash pop / 核验方 `git checkout 58ce304876 -- <五 shim>`，md5 六点≡发射锚）。锚字节载体三重备份：树上工作区、03:40 二进制 .rodata 内嵌（include_str! 编译期内嵌）、stash 提交 `58ce304876`（03:34:10 f6b-roster-attribution-temp，五 shim diff 156/39/81/58/3=刀体全量 337 行，保留未 drop）。
- **收口 commit ① `206678d4b（rebase 前为 0200851f2，replay 干净零冲突）`**（fix(engine-shim)，六文件 536+/5-，guard 隔离扫描 PASS 零发现）：committed blob md5 六点 ≡ 发射锚（part01 17fda0ae / part01c fb8aa0d8 / part01d 9fbab2af / part01f fb6530be / part06 8a5c039c / fixture 3cdf7835）——刀体 git 固化，字节保全竞赛终结；此后五 shim 的任何 checkout/stash 动作无内容风险（HEAD 即锚）。判读绑定不因提交改变（v1.2 前段腿验证的就是这组字节）。
- **尾链④ rebase 合并核验双源 + 发货树终验**：rebase 由并行轮代执行（账本回执），replay 于 origin/main 972a4b16d（远端 delta s349 trickle/盖章 + s352-354 写流 + part03，与刀区不相交）。合并核验双源一致：代执行轮字节对账（六特征串计数≡锚 walk_px 9/ArmedBucket 4/post_insert 5/f6bDissolvedKw 4、part01c/d/f 零 delta、fixture 恒一、part06 刀位清线平移 :11679）+ 本执行体复核（刀区特征 5/4/5、零冲突标记、①′ diff 536+/5- 纯刀）。**新树终验**：battery leg7 1455P/0F/0T + enginelib 3816P/7F 13.53s 名册逐项≡7-red 档（test-guard 包裹）——push 内容（发货树）集成验证闭环。远端 testharness.rs +206 行（757f4db84 聚合容错）可能改 rows 口径，后继批判读须重对账 mt 基线（记档）。

## 机制预测 vs 实测（v1.1 mt 定谳）

预测（oracle 12 案 + census 14 键）：mt 翻键恰 14 F6b 键（A4+C4+N6 全翻）×双页 ≤28，P2F=0。实测键级 F2P=14 恰中但**成分修正**：

1. **N1/N2/N4/N5 未翻（oracle 超预测）**：probe 17/0 过而语料残红——引擎产出形（重映射/提出形）与语料期望形分歧，行级基线/post 恒一（Fail=2/Pass=30），非回归（P2F=0），残红并入 F 残 52，backlog 独立刀。实际翻绿 N 族仅 N3/N6（等桶溶解散形）。
2. **B-table 4 键域外翻绿（附带修复）**：`delete/forwarddelete>inserttext(B)` 跨 `<b>` 壳 td/th 表格键 ×双页，基线 Fail=8→post Fail=0——part01c 收集器含界判定+stop 卫兵修复（N4/N5/N6/W2 跨壳 delete 空转根因，learning 见上）的共享面正向效应，跨 `<b>`/td/th 边界 delete 不再空转。

键级 14 = A4+C4+N2+B4（与预测数值巧合，成分不同——白名单门按 14 census 键构建漏 B-table 连带面，v1.1 门按设计拦截，v1.2 白名单扩 18 键收编）。

### DC-4 残键三分表（census 14 键逐键）

- **fixed（10）**：A1 subscript+delete / A2 subscript+forwarddelete / A3 superscript+delete / A4 superscript+forwarddelete / C1 subscript+fontsize(2) / C2 fontsize(2)+subscript / C3 subscript+fontsize(3) / C4 fontsize(3)+subscript / N3 outer font3 等桶溶解 / N6 tail outer font3 等桶溶解。
- **handoff（4）**：N1 outer font2 remap / N2 inner font3 hoist / N4 tail outer font2 remap / N5 tail inner font3 hoist——引擎产出形（probe 17/0 佐证）与语料期望形分歧，残红非回归（judge 行恒一）；接收域 = F6b 残键语料形对齐独立刀（backlog），重入条件 = 专项普查该四键语料期望序列化。
- **域外附带 fixed（4，非 census）**：B-table `delete/forwarddelete>inserttext(B)` × td/th——收集器共享面翻绿，随本刀入账。

## learning（四条落 docs/learnings/，2026-11-09）

1. [数值分桶中点扫描在整档值上系统性差一档](../../../../docs/learnings/bugs/2026-11/2026-11-09-numeric-bucket-midpoint-scan-off-by-one.md)（bugs）——bucket_of 最近档距离式。
2. [样式 carry 刀的捕获三形分账与壳级缩放口径](../../../../docs/learnings/patterns/2026-11/2026-11-09-style-carry-capture-consume-routing.md)（patterns）——捕获/消费旗标路由 + UA 盲读缩放校正放最窄层级。
3. [deleteSelection 跨壳边界选区静默空转](../../../../docs/learnings/bugs/2026-11/2026-11-09-delete-selection-cross-shell-noop.md)（bugs）——收集器含界判定 + stop 哨兵不作循环终止条件。
4. [wpt testharness 断言消息只在 Fail 输出](../../../../docs/learnings/platform/2026-11/2026-11-09-wpt-testharness-fail-only-dump-aggregation.md)（platform）——状态转储聚合单断言通道。
