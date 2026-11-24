# M2 s739 收账：generation change #15 窗（5d337a1a +7,744B C=9 同九员链＝kc5r93 css-parser 刀兑现主预判）＝arm-post window-pre pull 面（353b835cd 入嵌，s735 先例二演）＝join round 2 全持（e34668ff/0f54fef9/8918dfb3）＝桥持位 NEW=0（s330 十九连红）＝solo 131＝wob hold round 13

## 判定

s739 窗＝**世代交替 #15 窗＋pull 面四点断裂＋join/canon 链 round 2 持位**三面叠加，主预判全中：

- **Runner 三点**：pre=ef17dc229/130,445,864 持 → mid=post=**5d337a1a/130,453,608**（新建 **+7,744 bytes**，leg1 C=9 Fresh=0 全链枚举：product-version / css-parser / net / style-system / layout-engine / engine / page-runtime / webview / wpt-runner＝**与 #14 同九员链**（metadata 6＋css-parser 刀源＋style-system/layout-engine 下游）；leg2 C=0 Fresh=0（mid=post 同二进制零重编）；bridge runner md5 5d337a1a=mid（零重编）。**主预判（世代变更 #15）兑现**：kc5r93 css-parser ident()/vtn 刀（d0ac6a824）臂前入树＋臂时重编必至，逐演命中。
- **漂移判别第四证**：C=9 同员集两异值——#14 双刀（math-depth＋sanitizer-11）→ +14,368B vs 本轮单刀（kc5r93）→ +7,744B。同链成员、不同载荷 ⇒ 漂移载荷随**源内容差量**伸缩，成员集不充分（并入既有链：C=6→字节恒 / C=8→+80B / C=9 同数异构→+8,192B 构成差）。
- **Pull 面**（s735 先例二演兑现）：armstart 03:13:06/aa00eaea4 → 轮开 pull 03:13:29 吸收 353b835cd（peer R6440 kb244 docs-only）→ legs.head=353b835cd=judge＝**三点同值直证**；四点断裂＝pull 面；runner 入嵌 353b835cd；reflog 窗内唯一条目＝该 pull（臂后 23s）。
- **Join 全持 round 2**：stable=**8517** disagree=0 l1only=0 l2only=0；stable.canon=**e34668ff**（exact return 持位 round 2）；canon 双腿 **0f54fef9**、norm 双腿 **8918dfb3**（双腿同值持位 round 2，链重挂后无悬挂尾）；FLIPS=30（NEW=0 GONE=0）＝**kc5r93 join 面零扰**（peer kb242 先例兑现）；tally 236F/8275P/5T/1U 恒形。
- **Bridge 持位**：**1749P/1F rcb=101**，NEW=0 GONE=0 vs b739（主预判中＝刀域无新吸收面）；FAIL=1＝s330 **十九连红**（`test_dialog_focus_restore_declarative_shadow_s330` 同根慢性）；insert_adjacent_html_e2e 不复发 round 5；b740=69355510/1749 carried。
- **Solo**：eaec4575 双腿＝**round 131 连**（247 行＝raw 248 剥 1 Finished＝0 Compiling；4F 哨兵＝select-home-end-pagedown-pageup.optional ×4（Home/End/PageUp/PageDown）常态面；rc=2/2 恒常非新号）。
- **Wob**：wob 键（checkable-active-onblur-with-click）双腿 hits=1 双 Pass＝**hold round 13 realized**（watch 命中；episode-6 判定门未触发＝band 升级未触发）。
- **Bands 全持**：dialog r173 **2P/2P 双腿**（episode-3 已闭无余波 hold watch 命中）、select r178 279 键 8F IDENT se.577、password r178 225 键 0F、mpreload r201 15/15P、waapi r260 1F/1P（checkmark F/picker P）、fd r219 121/121P、onblur r171 2F/8P IDENT ob.s559、dual-emit x2 Pass 双腿、band r172 absent（out-of-band 常态）。
- probe RC=0；all.done 03:36:04（窗 22m58s，候窗期外载 load 峰 14.29 mykernel-2 域未扰收账）。

## 窗与布防

- 臂起：2026-11-25 03:13:06 / aa00eaea4（armstart.s739 两行式），三链 ps=3＋35s 零误 rearm；probe RC=0（aa00eaea4 树）。
- 部署：deploy.py 首跑全绿（s738 两轮课全数前置应用：锚 r54 非 r55、括号台账先计、POST-SED watch 锚、states 尾锚唯一性）。
- 轮开 pull（rule 13）：03:13:29 吸收 353b835cd＝**臂后窗前**落树（23s 缝隙）→ pull 面成形，s735 先例按面兑现。
- 候窗：两轮 poll，all.done 03:36:04；外载 load 14.29（mykernel-2 域）。

## 收账细目

- 世代 #15 与 #14 的链员集逐项相同（九员），唯触发源不同（#14＝math-depth＋sanitizer-11 双刀、#15＝kc5r93 单刀＋docs 入嵌）；字节差 14,368−7,744=6,624B 记为双刀对单刀载荷差留勘（后续双刀再现时可对账）。
- pull 面判读要点：armstart（aa00eaea4）≠ legs.head（353b835cd）≠ 四点同值断裂，但 legs.head=judge 三点同值＋runner 入嵌窗 HEAD＝按 s735 先例面记，非异常。
- s330 慢性红十九连（FAIL=1 rcb=101 恒形）；insert_adjacent_html_e2e 自 s735 回位后五轮不复发。
- wob 演化链 +r56（世代 #15 窗条目）；链 55 项（r2..r56）待 s740 部署时入表。
- join tally 形（236F/8275P/5T/1U）与 s738 恒同＝域内零漂移。

## s740 基线与预期

- b740=**69355510/1749 carried**（NEW=0 GONE=0 恒持）；wob1.keys=894e36fc 恒持。
- runner 预期：s740 窗无新刀前提下 **5d337a1a 持位**（C=0 或纯 metadata 重编形）；世代 #16 仅在 peer 新码刀到货时触发。
- join 预期：stable 8517＋stable.canon e34668ff 持位 round 3；canon/norm 双腿 0f54fef9/8918dfb3 round 3；FLIPS=30 恒。
- bridge 预期：1749P/1F rcb=101 持位（FAIL=1＝s330 二十连红候选；NEW/GONE=0）。
- wob 预期：hold round 14（r57 watch）；dialog r174 2P 持位 watch。
- solo 预期：eaec4575 round 132。

## s740 部署清单

1. 开工 pull --rebase（rule 13）；候窗；收账全套（三点/五数/solo/桥/band/wob/四点/reflog）。
2. wob 行 r57：标签 r56→r57＋九标签轮转（dialog r174/select r179/password r179/mpreload r202/waapi r261/fd r220/onblur r172/band r173/wob r57）；前缀锚 `r55 both legs Pass`→END（旧前缀括号台账 **(4,4)** 先断言）；新前缀净 0；r56 条目插入 regex `\(arm 02:36:31/88fea2a93 to window 02:52:41/88fea2a93 direct(\)+): episode-3 resolved` **len==23 count==1**（前缀替换先行）；条目括号台账净 +2 先计；watch bump **POST-SED** 锚 `; s740 post-episode-4 hold round 13 watch via wob1 r56:` → round 14/r57；链长 r2..r56=55、s685..s739=55、states 尾 [8517,8516,8517] append 8517（锚 `8517 join states` 唯一）；真值 23（b740=69355510/1749 carried 入表；负断言 `b739=69355510/1749 carried and` 不在）。
3. 部署：cp -a /tmp/s739 /tmp/s740＋rearm-s739.sh→rearm-s740.sh 改名＋b740.keys（cp s739.bridge.keys 双验 69355510/1749）＋keep-27 sweep（drop b738.keys＋probe.s738×2；probe.s739×2 留存）；blanket sed s739→s740＋bridge round-96 头重写（legal s739 ×2 头内、s738 ×1、s737=0、b739=0）＋b739→b740 全替换＋bash -n 11/11＋残留核对。
4. probe：`timeout 420 cargo check -p zero-wpt-runner --release` RC=0 门。
5. 臂起三链全绝对路径＋ps=3＋35s 零误 rearm＋armstart.s740 两行式。

## 证据文件清单

- /tmp/s739/{armstart.s739, legs.head, runner.{pre,mid,post}.{md5,size}, analyze.out, analyze2.out, bridge.runner.md5, s739.bridge.{keys,result,new.keys,gone.keys,fail.keys}, s739-bridge.rc, s739.opt-solo{1,2}.{log,rc}, l582-{1,2}.canon, s739-l{1,2}.norm, probe.s739.{log,rc}}
- 本轮桥基线：s739.bridge.keys＝69355510/1749（b740 carried 候选，cp 入 /tmp/s740 于部署时双验）
