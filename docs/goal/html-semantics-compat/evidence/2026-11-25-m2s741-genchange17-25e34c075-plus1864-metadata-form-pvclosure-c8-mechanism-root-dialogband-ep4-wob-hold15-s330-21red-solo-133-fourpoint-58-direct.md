# M2 s741 收账：generation change #17 窗（25e34c075 +1,864B＝metadata 形：零源码刀＝rerun-if-changed .git/HEAD 机制根因定位＝product-version 下游闭包 C=8）＝对端 kb248 逐字节双录＝dialog-band episode-4（stable 8516 disagree=1 与 s737 逐项同签名）＝wob hold round 15 realized（episode-6 域分离门未触发）＝s330 二十一连红＝solo 133＝四点同值第 58 演直证

## 判定

s741 窗＝**世代交替 #17 metadata 形＋dialog-band episode-4＋wob hold round 15 realized＋b741 基线首演持位**四面叠加：

- **世代 #17 metadata 形（本轮新形＝机制根因定位）**：臂时 solo1 04:34:04→04:35:23 raw 256＝**C=8 重编＝{product-version, style-system, net, layout-engine, engine, page-runtime, webview, wpt-runner}＝product-version 下游闭包八员**（刀域 css-parser 零触＝零源码刀）→ runner 由 held **361a7840/130,466,600 换值 25e34c075/130,468,464（+1,864 bytes，mtime 04:35:06）**。**机制根因（源码实据）**：build-support/product_version.rs `emit_cargo_env()` 打 `cargo:rerun-if-changed=../../.git/HEAD`＋`cargo:rerun-if-changed=../../.git/logs/HEAD` ⇒ HEAD 每移动（含 docs-only 提交 95467b3cb）→ build.rs 重跑 → `ZERO_BUILD_COMMIT` rustc-env 换值（617125aaa→95467b3cb）→ 闭包级联重编；**二进制内无 commit 哈希字符串（strings 0 hits）**＝+1,864B 为重链确定性再布局＝非语义载荷。**回溯自洽**：#14「metadata-6＋knife-source-3」分解、刀轮 C=9（九员＝8 闭包＋css-parser 刀域员）全解；**C=8＝首个无刀纯 metadata 枚举＝机制隔离面**。
- **对端跨流双录（逐字节）**：kb248（R6480＝3d30033d1）独立记录「pre 361a7840→run×3 恒 **25e34c07 +1,864B**」与本流臂时捕获逐字节同值＝**metadata 形跨流可复现直证**（peer byte-exact cross-record 第二演，kb246 先例形）；kb249（R6490＝384371231）预录刀⑬面 **e8fa2e33/+1,656B ×9 dom 新增**＝s742 臂时重编预测值（跨流逐字节验证 watch）。
- **四点同值第 58 演直证（直接形）**：armstart=judge=legs.head=**95467b3cb** 三点同值＋**reflog 窗内零条目**（末条 04:28:37＝部署 pull finish，早于臂 04:34:00；候窗期 fetch 观察未 pull）＝第 58 演四点直证兑现。
- **runner 四点二进制同值（窗内 quiet）**：pre=mid=post=bridge.runner.md5=**25e34c075/130,468,464**（leg1 C=0 Fresh=0 fin=1；leg2 同）＝世代 #17 在臂时 solo 面完成交替、窗内持位 quiet＝s740 先例副形二再演（重编由臂时 solo 承担、leg 面 C=0）。
- **dialog-band episode-4（本轮异常主形）**：`html/semantics/interactive-elements/the-dialog-element/dialog-overlay.html :: dialog computed overlay initially 'none'` **leg1 Fail/leg2 Pass**＝same-binary leg-unstable（canon/norm 双文件各恰此一行 l1/l2 异）；join＝**stable=8516 face-move −1 disagree=1 gone=1 new=0 l1only=0 l2only=0**＝与 s737 episode-3 逐项同签名；**canon/norm l1 侧移（l582-1.canon 4b7e34fb5／s741-l1.norm 7e08f575c）、l2 侧逐字节 held（0f54fef9／8918dfb3）**＝链悬挂 held 形；dialog r175＝l1 1F/1P、l2 2P episode 面。先例：s712 二演（单轮 return s713）、s737 三演（单轮 return s738）→ **s742 单轮 return-to-held watch**。
- **wob r58 双腿 Pass（hits=1 双 Pass）＝post-episode-4 hold round 15 realized**；**episode-6 wob-key-only 域分离门未触发**（失败键 dialog 域非 wob 域）＝band 升级不执行。
- **join（episode 面持位部分）**：**FLIPS=30 恒持**；tally **236F/8274P/5T/1U**（P 8275→8274＝dialog 键 episode 面 −1 自洽）；s741.stable.canon=03bc1320e（8516）。
- **全桥（b741 基线首演持位）**：**1750P/1F rcb=101**（30.01s，2520 filtered）＝**NEW=0 GONE=0 vs b741**；FAIL=1＝s330 **二十一连红**（`test_dialog_focus_restore_declarative_shadow_s330` 同根慢性）；insert_adjacent 不复发 round 7；**b741 carried cmp 0（18c5f489/1750 双验）**；桥 test-profile **C=5＝{product-version, style-system, net, layout-engine, engine}**（＝闭包成员在桥测试依赖树内子集＝s740 C=6 减 css-parser＝机制自洽）。
- **solo：eaec4575 四发射锚同值 round 133 连**：臂时 solo1 raw 256/C=8/fin=1（strip3 247 行）＋臂时 solo2 raw 248/C=0/fin=1＋post 双腿 raw 248/C=0/fin=1（05:07:01→05:07:17→05:07:34＝16s/17s 快跑形）＝**第三种 C 构成同锚**（257C9→248C0→256C8 全归 247 strip 行逐字节恒）；rc=2/2×2 恒常非新号；4F 恒常哨兵 ×4 日志（select-home-end-pagedown-pageup.optional PageUp/PageDown 面）；lock-wait 0。
- **哨位全持**：select r180 279 键 8F IDENT se.577、password r180 225 键 0F、mpreload r203 15/15P、waapi r262 1F/1P（checkmark F/picker P）、fd r221 121/121P、onblur r173 2F/8P IDENT ob.s559、band r174 absent（out-of-band 常态）、dual-emit x2、dup(2) ×2 恒形；probe RC=0（13.82s，95467b3cb 树）；rearm 零事件（waiter 首臂即窗）；'No such' 0。
- **观察计数兑现**：聚合形第 159 连、onblur 双面第 139 轮、mpreload 稳位第 103 轮、'No such' 第 136 演（0）、两臂齐全第 140 演、四点同值第 58 演直证。

## 窗与布防

- 臂起：2026-11-25 04:34:00 / 95467b3cb（armstart.s741 两行式），三链 ps 恰三（pid 1580273/74/75 同秒连发）＋35s 零误重臂；probe RC=0。
- 部署核验：bash -n 11/11；bridge-extract 头 legal s740 ×2（cp s740.bridge.keys＋s740 window）＋s739 ×1＋s738 ×1＋裸 b740=0＋锚唯一；b741=18c5f489/1750 与 s740.bridge.keys 逐字节 cmp 同＝carried 双验；wob1=894e36fc 承带。
- **候窗外载形**：judge 05:00:51＝候窗 26m51s（外载 qemu mykernel-2 域 load 4-7＝CPU id 84% D-state 阻塞型负载；judge 后回弹 4.89＝s740 外载窗中峰 11.49 同域先例形；waiter 门槛 v3.4 固定不动，deadline 9000s 余量足）。
- 候窗：judge 05:00:51（clean-soaked 3×45s procs=0 load<3，HEAD=95467b3cb）；leg1 05:00:52→05:02:57（rc=2）；leg2 →05:05:05（rc=2）；bridge →05:06:24（rcb=101）；legs.done 05:06:24＝窗内 5m33s（零重编快速窗形）。
- **链形照记（s741 先例固化）**：三链＝waiter＋rearm＋solos 臂时直发；post.sh 由收账会话窗后手发（analyze→analyze2→bridge-extract→solos→all.done 05:07:42）；**臂时 solo 快照三件（armsolo1.snapshot.log/.rc＋armsolo2.snapshot.log）在 post 链 solos 步前落定**＝快照保护课固化（post 链 solos 覆写 s741.opt-solo* 前先 cp）。
- 轮开 pull（rule 13）：部署会话臂前已至 95467b3cb；本收账轮 fetch 观察＝**留置区间 95467b3cb..9965da495＝3 笔实列（kb250 课执行）**：7684e6cac（**刀⑬ 码刀**＝sanitizer DC-2 切片 10 foreign ns 端到端保真，触 dom/node.rs＋engine js_dom_bridge＋shim part03.js＋新 pinned test part159_s10k13_foreignns_pins.rs）＋c81d7ae43（learnings docs）＋9965da495（wptfyi-dom docs）；候窗期到货只 fetch 观察未 pull，收账 pre-push pull 吸收（HEAD=384371231）。
- **测量口径课（本会话实证）**：solo 锚配方＝strip 全日志 `Compiling|Finished` 子串行后 md5（「strip3 247 行」）；`^` 锚 grep 口径读出假 C=0/fin=0（s740 已知件同口径亦误＝先校准已知件再测新件）；join 归因 comm 必须 `LC_ALL=C`（zh_CN 会话裸跑＝假 GONE 键列表，[巡查 join 管线 LC_ALL=C] 课再演）。

## 收账细目

- 世代 #17 与刀代语义分野：#14/#15/#16＝源码刀载荷（字节差随源内容差量伸缩）；#17＝**零源码刀 metadata 形**（字节差＝重链确定性再布局；commit 哈希不嵌入二进制 strings 面）。判代锚＝二进制 md5 换值照记，语义面按形归因（非回归、非新刀）。
- rerun-if-changed 机制解释历史全表：凡「HEAD 移动后的首个 make」承担 8 员级联重编（s741 臂时 solo 承担＝leg C=0；s736 metadata-drift +80B＝同机制 leg 面 C=8 先例回溯自洽）；部署头若 post-pull 立即臂，臂时 solo/leg1 承担；probe=cargo check 不暖 build 指纹照旧。
- 对端 kb248/kb249 判代链（361a7840→25e34c07→e8fa2e33）与本流锚链同值同差＝跨流代际账双录；s742 本流刀⑬重编若同值 e8fa2e33＝跨流双录第二例（预测已由对端预录）。
- dialog-band 家族账：episode-1 s699 前？无——wob 域 episode 账独立（episode-2 s699/episode-3 s720/episode-4 s725 均 wob 键域）；dialog 域：episode-2 s712、episode-3 s737、**episode-4 s741**（本演）——均为 same-binary leg-unstable 单键面、单轮归位先例 2/2。
- FLIPS=30（knife-9 face）在 metadata 换代下恒持＝重链再布局对 join 谱零扰（语义面零变化自证）。
- s330 慢性红二十一连（FAIL=1 rcb=101 恒形）；insert_adjacent 自 s735 回位后七轮不复发。
- wob 演化链 +r58 待 s742 部署时入表（世代 #17 metadata 形＋dialog-band episode-4＋hold round 15 realized 条目）。

## s742 基线与预期

- join：s634.stable 8517 持位基线不动；主预判＝**dialog-band episode-4 单轮 return-to-held＝stable 8517 round 4＋stable.canon e34668ff exact return round 4＋canon/norm 双腿 0f54fef9/8918dfb3 re-hung round 4＋五零＋FLIPS=30 恒持**；副形＝episode-4 不归位双演 watch（s712/s737 先例均单轮归位；双演不归位＝band 升级判定入场）。
- 全桥：s742 桥基线＝**b742＝cp s741.bridge.keys 实际件（1750 keys＝18c5f489 carried）**；主预判＝**刀⑬ 入树→NEW=1 `js_dom_bridge::tests::test_s10k13_foreignns_pins`（s10kN 命名链第 13 演＝副形二再演）＋b743 swap 候选＋s330 二十二连红挂账＋insert_adjacent 不复发 round 8 watch**；桥 test-profile C 预期回 6（dom 进级联＝刀⑬ 直触 dom/engine/shim）。
- runner：s742 世代锚＝**25e34c075/130,468,464 anchored at s741**；主预判＝**世代 #18＝刀⑬ 源刀面：臂时 solo（post-pull 树）×9 预测（8 metadata 闭包＋dom 直触）＝peer kb249 预录 e8fa2e33/+1,656B（跨流逐字节验证 watch：本流重编同值＝跨流双录第二例）**；副形＝纯 metadata 级联照记（s741 机制＝HEAD 移动即触发）。
- solo：**锚 eaec4575 round 134 预期**（刀⑬ 不触 html-semantics 谱输出行＝strip3 247 行恒预期；C=9 重编面 raw 行数变化 watch＝256→新值）。
- wob：**s742 post-episode-4 hold round 16 watch（r59 双腿 Pass 预期）＋dialog r176 2P return-to-held watch**（episode-4 归位判据；wob 键自身再翻 Fail＝episode-6 判定）。
- 观察计数布防：聚合形第 160 连、onblur 双面第 140 轮、mpreload 稳位第 104 轮、'No such' 第 137 演、两臂齐全第 141 演、四点同值第 59 演观察。

## s742 部署清单

1. 开工 rule 13：pull 置臂前（吸收本收账提交及后续到货＝全量枚举到货面；**归因区间实列逐笔核码面＝kb250 课**）；候窗期到货只 fetch 观察不 pull（留置纪律续演）。
2. probe：独立执行自仓根 `timeout 420 cargo check -p zero-wpt-runner --release > /tmp/s742/probe.s742.log 2>&1; echo $? > /tmp/s742/probe.s742.rc`＝RC=0 方可臂。
3. `cp -a /tmp/s741 /tmp/s742`（cp 前先 ls 目标）＋rearm-s741.sh→rearm-s742.sh 改名先于清扫＋b742.keys=**cp s741.bridge.keys 实际件**（1750 keys 双验＋md5 计算）＋wob1.keys 承带核验（894e36fc）＋keep-27 清扫（11 脚本＋13 基线＝b742/b741/wob1/band8/fd121/ob.s559/s477-dialogov/s477-mpreload/s477-onblur/s477-waapi/s634.stable.canon/s634.stable.keys/se.577＋legs.head＋probe.s741.log/rc；**drop b740.keys＋probe.s740×2＋armsolo 快照×3＋轮内工件**；comm 两步 mkdir 先行）。
4. blanket sed s741→s742（行口径族＋analyze2/bridge 动态 pre-count＋skip-if-converted 守卫）＋bridge-extract 头注释**第 98 轮语境整行重写**（legal s741 ×2 头内＝cp s741.bridge.keys＋s741 window 措辞；s740 ×1 深层叙述；锚 `# s742 full-bridge reconciliation:` 唯一）＋b741→b742 全 occurrence（body ≥3 处；残差 b741==0／s740 头内 legal 形按重写落定）＋bash -n 11/11。
5. analyze2 phase5：标签 9 处轮转（dialog r176/select r181/password r181/mpreload r204/waapi r263/fd r222/onblur r174/band r175/wob r59，各 count==1 断言）＋wob 段头整行重写（锚 `r57 both legs Pass`→END＝`, TRANSIENT RESOLUTION recorded at s684,`；s741 face 语境入注＝括号净 0 核验）＋r58 条目插入（锚 r57 尾 `(arm 03:54:32/617125aaa to window 04:07:02/617125aaa direct)` 先逐括号计数＝前缀替换先行后计；**条目文本先逐括号计数再进平衡门（s737 课）**＝generation #17 metadata 形＋dialog-band episode-4＋hold round 15 realized at s741 face）＋watch bump（POST-SED 锚 `; s741 post-episode-4 hold round 15 watch via wob1 r58:`→round 16/r59 形）＋链三 append（r 链尾 /r57→/r57/r58＝57 项 r2..r58；s 链尾 /s740→/s740/s741＝57 项 s685..s741；states 尾锚 `8517/8517/8517 join states` 唯一→`8517/8517/8517/8516 join states`＝57 项尾四 [8517,8517,8517,8516]）＋真值族逐项核（世代 #17 25e34c075 +1,864B metadata 形/C=8 闭包/机制根因 rerun-if-changed/kb248 双录/dialog-band episode-4 stable 8516/hob hold round 15/FLIPS=30/s330 二十一连/fd r221/ob r173/solo 133/四点直证 04:34:00→05:00:51 direct）＋平衡门＋残差族核验（analyze2 s741 wob 行唯一）。
6. 臂起前三链 ps 零残留核验＋三链全绝对路径单臂直发（terminal.log 约定）＋链形照记（s741 先例＝solos 臂时直发＋post.sh 收账手发；**臂时 solo 快照三件在 post 链 solos 步前落定＝覆写保护课固化**）＋臂起后 ps 恰三核验＋35s 零误重臂核验＋armstart.s742 必落两行式 `date +%H:%M:%S`＋`git rev-parse --short=9 HEAD`。
7. 哨位：wob1 r59 逐演核＝post-episode-4 hold round 16 watch；dialog r176 return-to-held 逐演记形；select/password/mpreload/waapi/fd/onblur/band 七哨位照记；观察计数 s742 形照记。

## 证据文件清单

- /tmp/s741/{armstart.s741, legs.head, window.s741.judge, runner.{pre,mid,post}.{md5,size}, analyze.out, analyze2.out, bridge.out, bridge.runner.md5, s741.bridge.{keys,result,new.keys,gone.keys,fail.keys}, s741-bridge.{log,rc}, s741-leg{1,2}.{log,rc}, s741.opt-solo{1,2}.{log,rc}, armsolo{1,2}.snapshot.{log,rc}, l582-{1,2}.canon, s741-l{1,2}.norm, probe.s741.{log,rc}, all.done, post.s741.terminal.log, waiter.s741.log, solos.terminal.log, rearm.s741.terminal.log}
- 本轮桥基线：s741.bridge.keys＝1750 keys（18c5f489＝b741 cmp 0 carried；b742 swap 候选＝cp 入 /tmp/s742 于部署时双验＋md5 计算）
- 臂时 solo 重编快照：armsolo1.snapshot.log（raw 256 C=8 fin=1）/armsolo1.snapshot.rc/armsolo2.snapshot.log（raw 248 C=0 fin=1）
