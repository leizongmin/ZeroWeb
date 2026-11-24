# M2 s738 收账：generation change #14 窗（ef17dc229 +14,368B C=9 全链）＝dialog-band episode-3 单轮回位（stable.canon e34668ff exact return）＝刀⑪ 桥 NEW=1 吸收（b739=69355510/1749 swap）＝wob r55 hold round 12＝solo 第 130 连＝四点直证第 55 演（纯窗）

## 判定

s738 窗＝**世代交替 #14 窗＋dialog-band episode-3 单轮回位＋刀⑪ 桥邻接吸收**三面叠加，全部 watch 命中：

- **Runner 三点**：pre=c6c9b002/130,431,496 持 → mid=post=**ef17dc229/130,445,864**（新建 +14,368 bytes，leg1 C=9 Fresh=0 全链枚举：css-parser / product-version / net / style-system / layout-engine / engine / page-runtime / webview / wpt-runner＝metadata 6（product-version/net/engine/page-runtime/webview/wpt-runner）＋刀源 3（css-parser←math-depth、style-system+layout-engine←math-depth 下游））；leg2 C=0（mid=post 同二进制零重编）；bridge runner md5 ef17dc229=mid（零重编）。
- **a39e91b0/+6,176B 主预期作废**：双证——(1) peer kb243 run1 md5≠a39e91b0 判代兑现（88fea2a93 commit message 在案，peer 短链 engine 面 css-parser/style-system 不在列）；(2) 我方 +14,368 vs a39e91b0 +6,176 差恰 **+8,192B=8KiB 整**（同为 C=9、源码 docs-only 之差）＝链构成面第三证材料：漂移载荷不止系于 crate 计数，系于**链内成员构成**（我方链含 net+page-runtime 元数据域 vs peer 刀域链），s736 +2 形留勘并入此三证留勘。
- **Join 回位**：stable=**8517** disagree=0 l1only=0 l2only=0，stable.canon=**e34668ff exact return 字节回归持位**（构造验证：e34668ff 由 s634.stable.canon＋现行 30 翻转唯一决定，actual==constructed）；FLIPS=30 恒持（21 dialog shadow＋9 script＝30/30 对称，math-depth/刀⑪ join 面零扰）；**canon/norm 链重挂 round 1**（canon 双腿 0f54fef9＝s737 leg2 锚、norm 双腿 8918dfb3＝s737 leg2 norm，双腿一致）。
- **双轴 md5 语义钉死**（本轮判读澄清，纠我方中途误诊）：057764d3＝**s634.stable.keys 基线键表 md5**（8517 键名集；wob 行 22 处历史条目「stable 8517 md5 057764d3 held byte-exact/round N」均为键集引用＝宅式正确）；e34668ff＝**刀⑨ 世代起 canon 内容真值**（s733 证据在案：stable.canon（e34668ff））。s738 exact return 双轴并验：键表 057764d3＋内容 e34668ff 皆字节回归。s737 计划块与 wob r55 前缀「held 8517 md5 057764d3」非陈值＝键集引用宅式，不改。
- **Dialog**：r172 **2P/2P 双腿**＝episode-3 单轮回位（s712 episode-2 先例二演兑现，dialog-band return-to-held watch 命中）。
- **Wob**：wob 键双腿 hits=1 双 Pass＝**hold round 12 realized**（watch 命中；episode-6 判定门未触发＝band 升级未触发）。
- **Bridge**：**1749P/1F rcb=101**，NEW=1＝`js_dom_bridge::tests::test_s10k11_config_modifiers_knife11_pins`（刀⑪ sanitizer 桥邻接吸收＝+1P 形，刀⑩ ±2 形族内单键变量），GONE=0；FAIL=1＝s330 **十八连红**（`test_dialog_focus_restore_declarative_shadow_s330` 同根慢性）；insert_adjacent_html_e2e 不复发 round 4；**b739=69355510/1749 baseline swap**（刀⑪ pin 入带）。
- **Solo**：eaec4575 双腿 round 130 连（raw 248→247 剥 1 行 Finished＝0 Compiling（runner 已由 run-legs 预建）＋1 Finished；4F 哨兵常态面；rc=2 恒常非新号）。
- **Bands 全持**：select r177 279 键 8F IDENT se.577、password r177 225 键 0F、mpreload r200 15/15P、waapi r259 1F/1P（picker P/checkmark F）、fd r218 121/121P、onblur r170 2F/8P IDENT ob.s559、band r171 absent（out-of-band 常态）、dual-emit x2 Pass 双腿。
- **armstart 四点同值直证**（第 55 演）：armstart 02:36:31/88fea2a93 → legs.head 88fea2a93 → judge 02:52:41/88fea2a93，纯窗 reflog 零条目（臂后 02:36:56 no-op pull 零 reflog delta 在案）；候窗 16m10s 外载（load 4.5→8.5）。
- **Pull 置臂前**：吸收 88fea2a93（peer R6430 部署时点追记，纯 docs ×2 行）＝a39e91b0 主预判随 peer 判代兑现作废之情报源。

## 窗与布防

- 臂起：2026-11-25 02:36:31 / 88fea2a93（armstart.s738 两行式），三链 ps=3 零残留＋35s 零误 rearm；probe RC=0 ×2（557a3b10f 树 32.87s＋pull 后 88fea2a93 树 10.52s＝runner-only metadata 重检）。
- 部署：deploy.py 首跑 phase4（sed s737→s738 行口径族 13/7/20/10/4/7/0/0/0＋analyze2=7＋bridge=10 全数转换＋bridge round-94 头重写＋b737→b738）过；phase5 崩于 wob 前缀锚（**教训**：标签 rN 前缀叙述 rN−1——锚应为 `r53 both legs Pass` 非 `r54 both legs Pass`）；repair2.py 修锚重跑全绿（括号 60/60 depth 0、链 53/53/53、真值 21、bash -n 11/11）。s737 前缀括号净额实测 0（3 开 3 闭）＝标签 `(` 由尾闭链吸收（run=1+2×条目数），新前缀去 `(episode-6 gate on wob key only)` 括号保净 0；条目插入自平衡（条目净 +2 开/run +2 闭）。
- 候窗：两轮 poll 9.5m+6.5m，all.done 02:52:41；外载 load 峰 8.54（mykernel-2 域）。

## 收账细目

- 漂移判别三证并立（留勘升级）：C=6→字节恒（s737＋peer kb240 双证）/ C=8→+80B（s736 单证，+2 attribution 留勘）/ **C=9 同数异构→+8,192B**（本轮：我 ef17dc229 +14,368 vs peer a39e91b0 +6,176，链成员构成差＝net/page-runtime 元数据域 vs 刀域）。下一步可作：比对 peer kb242 run2 ×9 刀域链名细目（R6420 证据）与本轮 C=9 全链枚举的交集差集，锚定 8KiB 载荷归属 crate。
- 刀⑪ 桥吸收形：NEW=1 单键（`test_s10k11_config_modifiers_knife11_pins`），较刀⑩ ±2 形（NEW=2 含 insert_adjacent 回位）少一变量＝干净单键入带；b739 基线 1749 键含刀⑪ pin，下轮 GONE watch＝无。
- s330 慢性红十八连（FAIL=1 rcb=101 恒形）；insert_adjacent_html_e2e 自 s735 回位后四轮不复发。
- wob 演化链 +r55（hold round 12）；stable 状态链 8517 尾接 8516（s737 episode）→ 本轮 8517 回位后链 54 项（s685..s738，尾 [8517,8516,8517]）；episode 计：band episode-3 已闭（单轮），wob episodes 4 恒。
- solo 4F 哨兵键：select-home-end-pagedown-pageup.optional（Home/End/PageUp/PageDown 四行为 assert_true Fail＝round 130 常态）。

## s739 基线与预期

- **b739=69355510/1749** baseline swap（NEW=1 刀⑪ pin 入带，GONE=0）；wob1.keys=894e36fc 恒持。
- runner 预期：s739 窗无新刀前提下 ef17dc229 持位（C=0 或纯 metadata 重编形）；世代 #15 仅在 peer 新码刀到货时触发（kb243 收账 commit 为 docs 预期）。
- join 预期：stable 8517 md5 e34668ff 持位、FLIPS=30 恒、canon/norm 双腿一致 0f54fef9/8918dfb3 持位（链已重挂，无悬挂尾）。
- bridge 预期：1749P/1F rcb=101 持位（FAIL=1 s330 十九连红候选；NEW/GONE=0）；刀域无新刀则无吸收面。
- wob 预期：hold round 13（r56 watch）；dialog r173 2P 持位 watch（episode-3 已闭无余波）。
- solo 预期：eaec4575 round 131。

## s739 布防清单

1. 开工 pull --rebase（rule 13）；候窗；收账全套（三点/五数/solo/桥/band/wob/四点/reflog 纯窗）。
2. wob 行 r56 前缀重写锚 `r55 both legs Pass`→END；r55 条目插入 regex `\(arm 02:36:31/88fea2a93 to window 02:52:41/88fea2a93 direct(\)+): episode-3 resolved` len==21 count==1（前缀替换先行）；**条目文本先逐括号计数再进平衡门**（s737 课）；链长 r2..r55=54、s685..s738=54、states 尾 [8517,8516,8517]→append 8517=54 项尾三 [8517,8516,8517]；真值 22（b739=69355510/1749 baseline swap 入表，b738 carried 保留）。
3. 部署：cp -a /tmp/s738 /tmp/s739＋rearm 改名＋b739.keys（cp s738.bridge.keys 双验 69355510/1749）＋keep-27 sweep（drop b737.keys＋probe.s737×2；probe.s738×2 留存）；blanket sed s738→s739＋bridge round-95 头重写（无裸 b738 token；legal s738 ×2 头内）＋b738→b739 全替换；phase5 标签 dialog r173/select r178/password r178/mpreload r201/waapi r260/fd r219/onblur r171/band r172/wob r56。
4. probe：`timeout 420 cargo check -p zero-wpt-runner --release` RC=0 门。
5. 臂起三链全绝对路径＋ps=3＋35s 零误 rearm＋armstart.s739 两行式。

## 证据文件清单

- /tmp/s738/{armstart.s738, legs.head, runner.{pre,mid,post}.{md5,size}, analyze.out, analyze2.out, bridge.runner.md5, s738.bridge.{keys,result,new.keys,gone.keys,fail.keys}, s738-bridge.rc, s738.opt-solo{1,2}.{log,rc}, l582-{1,2}.canon, s738-l{1,2}.norm, probe.s738.{log,rc}}
- 本轮桥基线：s738.bridge.keys＝69355510/1749（b739 候选，cp 入 /tmp/s739 于部署时双验）
