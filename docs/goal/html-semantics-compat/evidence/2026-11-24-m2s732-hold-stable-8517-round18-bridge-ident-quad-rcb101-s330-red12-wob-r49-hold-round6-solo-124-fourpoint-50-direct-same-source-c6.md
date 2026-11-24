# s732 收账（2026-11-24）——join 持位 round 18＋桥面稳态四连形（baseline-identical）＋wob r49 hold round 6＋四点第 50 演直证形＋同源重编面（leg1 C=6 零字节差）

## 判定

- **主预判逐项命中**：join 持位形（stable 8517 **round 18**）＋全桥稳态延续形（baseline-identical **四连形** s729/s730/s731/s732）双主预判全中；副形（世代变更 #11）未兑现＝**零世代变更**。
- **join 持位形 round 18**：stable **8517** md5 **057764d3** ≡held；**五零**（NEW=0 GONE=0 disagree=0 l1only=0 l2only=0）；分布 **8284**P/227F/5T/1U 恒；**FLIPS=21**（a1dee2f4）＝**十二连恒持**＝刀④ face 未动＝docs-only 轮 join-face 零扰动；**canon/norm 双腿同值持位 round 11**：canon **39bf4f52**/39bf4f52＋norm **7392c958**/7392c958（8519 行×2 each）；stable.canon **f67a698a** ≡held。
- **runner 零世代变更＋同源重编面照记**：pre=mid=post=**365a4c10**/130,360,608 三点恒＝世代锚 365a4c10 anchored at s731 **持位（同源二轮）**；**leg1 Compiling=6 Fresh=0**／leg2 C=0＝**同源重编面第五演**（s719 C=8/s725/s727/s728 C=6 先例族；docs-only 轮零字节差 fingerprint 失效形）；legs.head=**cb2ff95bb**。
- **全桥稳态延续形命中（baseline-identical 四连形）**：rcb=**101**＝`1746 passed; 1 failed`＝**NEW=0 GONE=0** 对 b732＝bridge keys md5 **c3725e92 ≡b732** 逐字节同＝bridge.runner.md5=**365a4c10**（docs 轮对 js_dom_bridge 谱零扰动）；**FAIL=1 test_dialog_focus_restore_declarative_shadow_s330＝s330 十二连红**（对端域挂账持续）。
- **四点同值第 50 演＝四点直证形（主形兑现）**：armstart **22:48:02/cb2ff95bb**＝judge **22:53:30/cb2ff95bb**＝legs.head **cb2ff95bb**＝收账 HEAD cb2ff95bb 四点同值；reflog 窗区间 [22:48:02, 22:53:30] **零条目**＝**纯窗形十三连兑现＋窗内 pull 零复发第 22 演连兑现**。
- **wob r49 双腿 hits=1 双 Pass＝post-episode-4 hold round 6 兑现**（watch 条款逐字兑现；**episode-6 判定门未触发**＝band 升级未触发＝band8 不动）。
- **哨位全持位**：dialog r166 双腿 **2P/2P＝hold round 19**；select-event r171 279 keys 8F 双腿 **F-set IDENT se.577**；password r171 225 keys 0F；**mpreload r194 15/15P＝稳位第 93 轮**；waapi r253 1P/1F 双腿；**fd r212 121/121P**；**ob r164 2F/8P＋Fset 双 IDENT**（双面第 130 轮）；**band r165 absent＝聚合形第 149 连**；双发伪影 ×2（n=2 ×4）恒。
- **solo（双腿 248 净形＝发射形同值第 124 连兑现）**：raw **248×2** strip3 **247×2** 锚 **eaec4575** 双腿同算；rc 双 2（Error 1 已知形常态）；so1/so2=0。
- **杂项**：'No such' 0×全 log＝第 127 演兑现；SENTINELS_DONE 单行（analyze2.out）＝两臂齐全第 131 演兑现；**日期串 26.11.25=0×全 log＝跨 11-25 判据四连未触发**（窗全程 22:53–23:00 均在 11-24）；三链窗毕自退（rearm/post terminal.log 零字节双）。

## 窗与布防

- 臂 22:48:02（三链全绝对路径单臂直发；ps 恰三 PID 3998359/60/61；35s 零误重臂；armstart.s732 部署手落件两行形）→候窗 **5m28s**（浸泡 3×45s 过闸，judge **22:53:30** clean-soaked procs=0 load<3，HEAD=cb2ff95bb）→leg1（C=6 重编波）→leg2→legs.done **22:59:26**→桥（rcb=101）→solos→all.done **23:00:14**＝judge 起至全毕 **6m44s**。
- **部署**（本 session 完成）：cp 前 ls 目标→rearm 改名先于清扫（课①）→b732.keys=cp s731.bridge.keys（c3725e92/1746 双验）→keep-list 清扫（27 文件＝24 核心＋.head＋probe.s731×2；b730.keys 退役）→blanket sed s731→s732→bNNN 五处→round-88 桥头→**deploy.py phase5 watch-rep 锚遭 blanket sed 预转换拦截一次（锚误用前 sed 形；resume.py 改后 sed 形锚续跑过＝课：phase5 各 rep 锚一律写后 sed 形）**→标签 9/9→wob r49 头重写＋r48 条目插入（闭括 9→11 预期）→**课⑤平衡门 opens=closes=44/depth=0 过**（勘误后平衡语义首次平演）→链 47/47/47＋真值 15＋watch 升轮→bash -n 11/11＋exec→残检：桥头 s731×4 全史叙合法位（carry＋anchored＋first-run×2＝round-88 头自带双叙非残留）＋其余脚本零残留＋analyze2 s731 仅 wob 行演化链。
- **臂前探针**：RC=0（**29.07s**）＝判据续演；独立执行。

## s733 基线与预期

- join：**持位形＝held s634.stable 8517**（057764d3）＝s733 主预判＝持位形 round 19＋五零＋FLIPS=21 十三连＋canon/norm 双腿同值 round 12＋stable.canon f67a698a；副形照记（episode-6 判定门＝band 升级条件；对端下一波落树扰动形按面归因）。
- 全桥：s733 桥基线＝**b733＝cp s732.bridge.keys 实际件**（keys **1746** fail=0＝**c3725e92 承带恒**）；主预判＝**1746P/1F rcb=101 桥面稳态五连形**（NEW=0/GONE=0 对 b733；FAIL=1 s330 十三连红挂账）；副形＝对端修复刀④→rcb=0 按面归因＝**桥指针链 b732→b733 五处定点核改（课⑧续）**。
- runner：s733 世代锚＝**365a4c10/130,360,608 anchored at s731（同源二轮）**；主预判＝同源重编面照记（leg1 C>0 零字节差 s719/s725/s727/s728/s732 五先例）或 Fresh 树静面；副形＝对端下一波码刀→**世代变更 #11**＝leg1 重编波预期；**跨 11-25 判据重点续**（s733 窗若跨午夜→26.11.25 环境面如实记不冒充行为回归）。
- solo：raw 248 双腿净形＋strip3 锚配方（247→eaec4575）＝**第 125 连**预期。
- 四点第 51 演：主形＝四点直证（部署→臂→判窗间零 pull）；副形照记（臂后窗前落树形 s717/s731／三点形 s719/s729/s730）。
- 哨位承带：wob r50＝**post-episode-4 hold round 7 watch**；dialog r167＝hold round 20；fd r213 121/121P；ob r165 2F/8P＋Fset IDENT；band r166 absent；select se.577；mpreload 稳位第 94 轮。

## s733 布防清单

- cp -a /tmp/s732 /tmp/s733（cp 前先 ls）＋rearm-s732.sh→rearm-s733.sh 改名先于清扫（课①）＋**b733.keys=cp s732.bridge.keys**（c3725e92/1746 双验）＋wob1.keys 承带核验（894e36fc）＋keep-list 清扫（join 五中间件＋s1/s2＋ob 四中间件勿漏＋**b731.keys 退役**：keep b732+b733＝13 基线件＋11 脚本＝24 核心；probe.s732×2 留存、probe.s731×2 清＝先 ls 后 rm 两步清）＋sed s732→s733（逐文件 pre-count 断言转换恒；analyze2 仅 watch 形 1 处预期）＋**b732.keys 五处定点**（header/comm×2/echo 标签/文件名＝课⑧ bNNN 无 s 前缀）＋round-89 桥头整行重写＋**sed 后 wob 史叙 bNNN 逐处核对（课②真值表十六处**：b715 baseline 89f069bf/1739、b719 5079d627/1740、b720 4d5a5125/1741、b721 9600dffa/1742、b722 3e1f0ddc/1742、b723 646cd3f5/1743、b724 4f890f9b/1744、b725 2c43de56/1745、b726=2c43de56/1745、b727=2c43de56/1745、b728=2c43de56/1745、b729=2c43de56/1745、b730=2c43de56/1745、b731 baseline c3725e92/1746、b732=c3725e92/1746 carried、b733=c3725e92/1746 carried；stale 'b732 baseline c3725e92/1746 and' prefix 形零残留）＋rearm v3.5 argv 形状恒持＋**wob 哨位段头整行重写**（r49→r50＝**r49 post-episode-4 hold round 6 realized 语境入注**（同源二轮零世代变更＋leg1 C=6 零字节差 face）＋r49 演化条目插入（闭括 9→11）＋尾叙维持 episode-4 resolved＋链 r2…r49＋s685…s732＋states 尾插 **8517**＝**课④三者计数对齐 48×3＋课⑤全行括号平衡常设门（勘误在册＝平衡非固定对数勿写死绝对数）**）＋标签 +1（**r167/r172/r172/r195/r254/r213/r165＋band r166＋wob r50**＝防碰撞链 dialog→band→onblur）＋**phase5 各 rep 锚一律后 sed 形**（本轮 deploy.py 拦截教训入课）＋bash -n/chmod/inventory/flags clean。
- 臂前探针独立执行（timeout 420 cargo check -p zero-wpt-runner --release 落 probe.s733.log/rc）＝RC=0 方可臂。
- 三链 ps 零残留→绝对路径单臂直发→ps 恰三→35s 零误重臂→armstart.s733 部署手落件两行形→**s733 收账注记 commit 置窗后落账**（纯窗形十四连＋pull 零复发第 23 演连 watch）。
- **臂后窗前间隙纪律照记**：部署→臂→判窗间对端 push 落地→pull 只许臂前完成；臂后窗前 pull 合法但按 s717/s731 形如实记；窗内 pull 零容忍。
- 聚合形第 150 连观察；onblur 双面第 131 轮；mpreload 稳位第 94 轮；'No such' 第 128 演；两臂齐全第 132 演；四点第 51 演观察；runner 世代核验（365a4c10 恒或世代变更 #11＝`cat` 内容口径＋analyze2.out 权威行为准）；solo 锚配方 strip3 判据；**刀④ 21 键面 watch 续**（FLIPS=21 恒持或对端修复回 Pass 均按面如实记＝s330 红待办跨轮挂账直至对端处理）；**b733 承带面 watch**（c3725e92/1746 恒预期）。

## 证据文件清单

- /tmp/s732：window.s732.judge（22:53:30/cb2ff95bb）、armstart.s732（22:48:02/cb2ff95bb）、legs.head（cb2ff95bb）、runner.pre/mid/post md5+size（365a4c10/130,360,608 ×3 三点恒）、s732.stable.keys（8517/057764d3）、s732.flip.keys（21/a1dee2f4）、l582-1/2.canon（39bf4f52×2/8519）、s732-l1/2.norm（7392c958×2）、analyze.out（分布 8284P/227F/5T/1U＋五零＋FLIPS=21）、s732.stable.canon（f67a698a）、analyze2.out（SENTINELS_DONE＋wob r49 hits=1×2＋dialog r166 2P/2P＋fd r212 121/121＋ob r164 2F/8P IDENT＋select 279/8F IDENT se.577＋band r165 absent＋dup n=2×4＋runner 三点恒＋leg1 C=6 Fresh=0）、s732.bridge.keys（1746/c3725e92≡b732）、s732.bridge.new/gone.keys（0/0 行）、s732.bridge.fail.keys（s330）、s732-bridge.rc（101）、s732.opt-solo1/2.log（raw 248/strip3 247/anchor eaec4575）、s732-leg1/2.log（C=6/C=0）、reflog（窗区间 22:48:02–22:53:30 零条目）、all.done 23:00:14。
