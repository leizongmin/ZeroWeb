# s683 收账证据——clean-soak 候窗（8m16s）＝**摆动键第三发兑现演（迁移形信号）**＝`checkable-active-onblur-with-click … following label` l1 Pass／**l2 Fail**＝disagree=1＝stable **8516**（8517−1）＝**GONE=1**（FLIPS=0 NEW=0）＝分布 **8304P**/206F/5T/1U（Pass −1＝键自 stable 除名形）＝**canon 链断链演**（80d67629 ≠ f1aef7b7＝s634→s682 持 75 连后首断）＋**norm 整腿复现链断链演**（l1=c735d371 held／l2=88a0d3fa＝s597→s682 持 83 连后首断＝双腿不再逐字节同值）＋keys 排序指纹 af147e78 ≠621e00fe（键域漂移恰 1 键＝GONE 键自证）＝l582 双 canon 31683796/28a5ab80 分叉＝runner 候选七十六持位第 15 演不受扰（pre=mid=post **5f0b023d**/129,895,784＝leg1 Compiling ×6/leg2 ×0）＋b683 第 39 轮全绿 fa1ff46b rcb=0 第三十二演（新世代锚定后第 16 轮全绿＝桥面零扰）＋solo 发射形同值第 76 连（248/249 行＝329e35ed 双 IDENT＝单跑形持位）＋'No such' 归零第 78 演＋四点同值零漂移第 6 演＋**键带哨位化升级条件触发（ wob1 带布防输入）**

日期：2026-11-23（**自臂自收形续＋臂前落账续演**：上 turn s682 收账 commit 34dfed59e（17:0x 推达）→部署 s683→臂起 17:02:45→armstart HEAD=34dfed59e＝**收账 commit≡布防 HEAD 零漂移**（臂前落账续演兑现）→本 turn 开工 rule 13 同步＝pull 无新（窗前合法＝开工时 window.s683.judge/legs.done/all.done 均未存在＝早于窗开 17:11:01）→候窗 **8m16s**（17:02:45→17:11:01＝clean-soaked 三连检通过）→窗开 17:11:01（judge HEAD=**34dfed59e**）→legs 顺跑→all.done 17:16:57→本 turn 收账）。
环境：本机多 clone 并行；`/tmp/s683`；legs.head=**34dfed59e**（＝窗 judge HEAD＝armstart HEAD＝开工捕获 HEAD）；窗＝34dfed59e→34dfed59e＝**本流窗 0 commit＝平凡窗**；对端 3 commit（d505f68cb 17:14:22 批二十九收账＋da82f2226 17:14:47 R5949 rendering-compat＋7e113546e 17:15:42 批三十布防＝均 **窗内落 origin/main**）＝均 wptfyi-html/rendering-compat 域 docs-only＝本流域零触碰＝零碰头（窗后 fetch 恰 **3 commit**＝已定位待下次 pull 吞入）；**四点同值＝零漂移第 6 演**（armstart=judge=开工捕获=legs.head 全四点同值 34dfed59e＝断连计数不重置照记）。

## 第三发定谳（本轮核心事实＝诚实 trail）

- **GONE 键＝摆动键本身**：`html/semantics/forms/the-input-element/checkable-active-onblur-with-click.html :: Checkboxes should clear :active when the user tabs away from them while pressing the following label with a pointing device`。
- **双腿状态**：l1 **Pass**（s683-l1.norm 有键＝Pass＝l582-1.canon md5 `31683796` ≡held 形逐字节）／l2 **Fail**（s683-l2.norm 有键＝Fail＝l582-2.canon md5 `28a5ab80`）→disagree=1→stable join 除名→GONE=1。
- **canon diff 恰 1 行**（diff l582-1 vs l582-2＝3992 行 Pass→Fail 唯一差异）＝本轮全部 join 变化皆自该键＝零伴生漂移。
- **历史**：二发瞬态（s666 首发形/s672 二发形记录在案）→s674 回位→s675–s682 连续八演持位（无第三发）→**s683 第三发兑现**＝既有升级条件「第三发瞬态再现→键带哨位化升级」**触发**＝s684 布防 wob1 键带哨位（镜像 band8 形＝逐轮双腿状态记形）。
- **归因（按面核对后定谳）**：runner pre=mid=post ≡5f0b023d/129,895,784＝候选七十六世代零增量＝**非 DP 面码刀**；桥面 1738P/0F fa1ff46b＝js_dom_bridge 零扰；solo failset 329e35ed ≡held＝solo 谱零漂移；norm l1 逐字节 held＝l1 腿复现基线形→**单键单腿瞬态形（l2 Fail）＝测试面 timing 敏感交互键第三发＝非编译失败冒充＝非码面迁移**（真迁移判据留观：连续双演双腿 Fail 才升级迁移形全归因）。

## 布防核验（单臂洁净形＝s679 修正持位）

- 部署（上 turn）＝臂起前三链 ps 零残留核验＋**三链全绝对路径单臂**→臂起后 ps **恰三进程**（3660748 rearm-s683.sh/3660749 wait-and-run.sh/3660758 post.sh，cmdline 全绝对路径）＝s679 修正续证（s680 后第 4 演单臂核验形）。
- 22 文件洁净态＝b683.keys=cp s682.bridge.keys 实际件（md5 **fa1ff46b**/1738 双验＝瑕疵①照规）＋标签 r117/r122/r122/r145/r204/r163/r115＋band r116 全清单复核 8 项 IDENT（＝analyze2.out 轮次标签逐项一致）＋碰撞链 r116→r117 先行〔dialog〕/r115→r116 次之〔band〕/r114→r115 末〔onblur〕＝零碰撞＝瑕疵②照规＋bridge-extract 头注释第 39 轮语境重写＝残留 grep 零命中＝瑕疵③照规＋bash -n 全过＋执行位 11 脚本全在。
- waiter.s683.log 空＝8m16s 静候（clean-soaked 形）；rearm.s683.log 不存在＝零真重臂；post/rearm terminal.log 双在册＝**两臂齐全第 82 演零空窗**（s602–s683）；收账 commit 34dfed59e 先于臂起 17:02:45＝收账先于布防；窗开至 all.done 本仓零 commit/pull/push＝**窗内 pull 违例零复发第 36 演连**（对端 3 commit 窗内落 origin/main＝对端推树非本流 pull＝零碰头如实记形）。

## 窗与 legs

- 候窗 **8m16s**（17:02:45→17:11:01）＝clean-soaked 形；window.s683.judge＝**clean-soaked procs=0 load<3 across 3x45s checks HEAD=34dfed59e**＝浸泡窗史 s598–s683 续。
- leg1 17:11:01→17:13:46（2m45s）rc=2，**Compiling ×6 Fresh=0＝重编位等形**（候选七十六锚定后持位第 15 演）；leg2 17:13:46→17:15:45（1m59s）rc=2，**Compiling ×0＝收敛形**；单跑核验＝`Using 8 test job` ×1。
- **runner 候选七十六持位形兑现（零扰）**：pre=mid=post=**5f0b023d**/129,895,784 三点同值＝bridge.runner.md5 ≡5f0b023d。
- 桥测（run-legs 内，test-guard recipe）：test **26.18s**，rcb=**0**（**全绿持位第三十二演**＝1738P/0F＝摆动键第三发零扰桥面）。
- solo：solo1 17:16:26→solo2 →17:16:57 rc=2 ×2；s683.opt-solo1/2.log＝**248/249 行**（solo2 多 1 非 Fail 行＝raw 记形微变）/4 Fail/Using ×1 ×2＝单跑形持位；failset（`^Fail` 原始序）md5 **`329e35ed` 双 solo IDENT ≡s611–s683＝发射形同值第 76 连**（×4 Fail raw 直取＝摆动键不在 solo 谱＝零漂移）。窗起至全毕 **5m56s**（17:11:01→17:16:57）。

## join 判定（analyze.out）＝**链断演**（三发瞬态除名形）

- `stable=**8516** disagree=**1** l1only=0 l2only=0`；**NEW=0 GONE=1 FLIPS=0**；分布 **8304P**/206F/5T/1U（Pass −1＝键自 stable 除名形；Fail/Timeout/Unsupported ±0）＝**迁移形信号如实记形**（持位形主预判未兑现＝断连照记）。
- **canon 链断链演**：s683.stable.canon md5 `80d67629…` ≠ s634.stable.canon `f1aef7b7…`＝**s634→s682 链持 75 连后首断**（断因＝摆动键除名单行＝canon 行数 8518）。
- **norm 整腿复现链断链演**：l1=**c735d371**（≡held 逐字节＝l1 腿复现基线形）/l2=**88a0d3fa**＝双腿不再同值＝**s597→s682 链持 83 连后首断**（断因同上＝l2 单键 Fail 行）。
- keys 排序指纹 **af147e78** ≠621e00fe＝键域漂移恰 1 键（＝GONE 键自证＝零伴生）。

## b683 判定（bridge.out）＝第 39 轮＝全绿形主预判兑现＝零挂账基持位

- keys=**1738**（b683=1738）fail=**0** NEW=0 GONE=0；s683.bridge.keys md5 **`fa1ff46b`**（≡b683 基＝cp 实际件核验〔瑕疵①〕）；test result **ok. 1738 passed; 0 failed**＝rcb=0 第三十二演＝**全绿基第 39 轮持位**＝候选七十六 5f0b023d 新世代锚定后第 16 轮全绿观察兑现＝桥面零扰自证。
- 桥红待办挂账 **×0 持位**＝桥面零挂账基。

## 哨位（analyze2.out，SENTINELS_DONE 单行＝单链无竞态）

dialog band r117＝2P 双腿（flaky 键持位第 72 轮）；select-event r122＝279/279 keys 8F **F-set IDENT se.577**＝持位；password r122＝225/225 keys 0F＝持位；**mpreload r145＝15/15P 双腿＝稳位第 44 轮兑现**；waapi r204＝picker P/checkmark F 双腿；focus-dynamic r163＝121 all P；**onblur r115＝双腿 2F/8P＋ob Fset.l1/l2 双 IDENT ≡ob.s559＝双面第 82 轮兑现**；族带 r116＝band8 hits=8×2 全 Fail（**聚合形持位第 100 连**）；双发伪影 ×2 恒（dup ×4）；爆栈 so1/so2/sob=0/0/0；**'No such' 归零审计 leg1/leg2/bridge/solo1/solo2＝0/0/0/0/0 第 78 演兑现**（预装修复形第 48 轮续）。轮次标签全对齐（r117/r122/r122/r145/r204/r163/r115＋band r116＝与 s683 布防计划算术一致）。**摆动键第三发＝键带哨位化升级条件触发**（本轮尚无 wob 哨位＝s684 布防 wob1 带）。

## 窗底账（s683 窗＝34dfed59e→legs.head 34dfed59e＝本流 0 commit＝平凡窗）

- 本流窗内 0 commit/pull/push＝**窗内 pull 违例零复发第 36 演连**；窗内零新码刀＝无补录形；窗底账：armstart=judge HEAD=开工捕获 HEAD=legs.head **全四点同值 34dfed59e**＝**四点同值零漂移第 6 演**（断连计数不重置照记）。
- 对端 3 commit（批二十九收账/R5949/批三十布防＝wptfyi-html/rendering-compat 域 docs-only）窗内落 origin/main＝本 clone 工作树零触碰＝零碰头；s684 窗预期输入＝对端 3 commit 在 origin（零新真码刀在途）。

## 基线决策（s684）

- join：**对账基线继续 held＝s634.stable 8517**（f1aef7b7/057764d3）＝**不因单键瞬态 disagree 重锚**（s674 回位先例＝瞬态除名后 stable 复归 8517 形）＝（s684 主预判＝**双形观察**：〔a〕瞬态回位形＝摆动键双腿复 Pass→stable 复归 8517 disagree=0 GONE=0＝canon 链复挂第 76 连＝norm 双腿复归 c735d371＝keys 复归 621e00fe＝s674 先例重演；〔b〕再发持位形＝该键再 Fail→GONE=1/disagree=1 持位＝wob1 带逐演记形＝**连续双演双腿 Fail 才升级真迁移形**→按面归因〔测试面 timing 键先核＝候选七十六 DP-①② 触点核对＝编译失败不冒充行为失败〕；无论何形 wob1 哨位逐演记形＝再现不可预报照记）。
- 全桥：s684 桥基线＝**b684＝cp s683.bridge.keys 实际件**（keys **1738** fail=**0**＝**fa1ff46b 第 40 轮全绿基持位观察**＝**必须拷实际提取件 s683.bridge.keys 非参考件〔瑕疵①教训照规〕**；主预判＝零红基线持位＝1738P/0F 持位预期＝rcb=0 第三十三演；副形＝谱内新 flip 挂账形按面归因〔候选七十六 DP 面＝js_dom_bridge 触点核对〕；**桥指针链 b683→b684 逐处核改＝s636 漏改教训照记**）。
- runner：**候选七十六 5f0b023d/129,895,784 持位预期＝新世代锚定后持位第 16 演＝pre=mid=post 三点同值＋leg1 Compiling ×6 重编位等形/leg2 ×0**。
- solo：**发射形同值第 77 连观察＝单跑 248 行/4 Fail/329e35ed 持位预期**（s680–s683 单跑形四演确认；s683 solo2 249 行＝非 Fail 行记形微变＝观察 raw 形；若回双跑形＝布防回归按瑕疵链追查）。
- **键带哨位化升级（本轮新增布防面）**：wob1.keys＝摆动键单键带（1 键）＝analyze2.sh 新增 wob 哨位段（镜像 band8 形＝双腿 grep 状态＋hits 计数）＝逐演记形＝升级轮标签 wob r1。

## 下一步（s684 部署清单）

- `cp -a /tmp/s683 /tmp/s684`＋b684.keys=**cp s683.bridge.keys 实际件**（fa1ff46b/1738 双验）＋**wob1.keys=cp s683.wob1.keys**（单键带 1 行核验＝**键带哨位化升级实施**）＋全量运行产物清扫（含 k1/k2/l1u/l2u/s1/s2/disagree/flip/gone/ob.curF1/ob.curF2/ob.f1k/ob.f2k＋legs.head＋旧 b683.keys＋l582 双 canon＋s683 运行 log/时间戳产物＝瑕疵③）＝**洁净清单 23 文件**（11 脚本＋12 基线件＝新增 wob1.keys）＋rearm-s683.sh→rearm-s684.sh 改名＋sed b683→b684/s683→s684＋analyze2.sh 新增 wob 哨位段（`=== wobble-key band wob1 r1 (THIRD EPISODE at s683 l2 Fail; escalation; per-leg status per round; s674 resolution precedent)`＋双腿 grep -a -F -f wob1.keys＋status 打印＋hits 计数＝镜像 band8）＋bridge-extract 头注释第 40 轮语境重写（s669..s683→s669..s684＋fifteenth→sixteenth hold round＋**WOBBLE-KEY third episode FIRED at s683 l2 Fail→wob1 band escalation active**＝替换 no-third-episode 旧语境＝残留 grep 零命中）＋标签 +1（**r118/r123/r123/r146/r205/r164/r116＋band r117＋wob r1**＝sed 防碰撞链 r117→r118 先行〔dialog〕/r116→r117 次之〔band〕/r115→r116 末〔onblur〕）＋部署后标签全清单复核＋bash -n/chmod/inventory/flags clean＋**臂起前三链 ps 零残留核验**＋**三链全绝对路径单臂**＋臂起后 ps 恰三进程核验。
- 摆动键 wob1 哨位联动记账：join disagree/GONE 逐演核该键双腿状态（回位形/再发形判形）＝wob hits 逐演核。
- 聚合形第 101 连观察；onblur 双面第 83 轮；mpreload 稳位第 45 轮；'No such' 第 79 演；两臂齐全第 83 演。
- solo 单跑形态持位核验（248 行/4 Fail/329e35ed＝布防修正持续生效判据）。
- 布防落账臂前落账续演（同笔形四点核验随对端落树时点如实记形＝断连计数不重置）。
