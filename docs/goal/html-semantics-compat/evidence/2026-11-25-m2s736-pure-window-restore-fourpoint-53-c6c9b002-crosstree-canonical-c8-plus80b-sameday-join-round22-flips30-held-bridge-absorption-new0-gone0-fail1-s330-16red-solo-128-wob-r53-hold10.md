# s736 收账（2026-11-25）＝纯窗复挂＋四点直证第 53 演＋runner c6c9b002 跨树同 face 互证＋C=8/+80B 同日漂移判别对＝join 持位 round 22＋全桥吸收形 NEW=0/GONE=0＝s330 十六连红＝solo 第 128 连＝wob r53 hold round 10

## 判定

- **窗型**：平凡持位窗全绿（s736 布防主预判逐字兑现）：join 持位 round 22＋桥 1748P/1F 对 b736 NEW=0/GONE=0（±2 形吸收完成＝回位形）＋solo 第 128 连＋哨位全持位＋wob r53 hold round 10。
- **runner**：非同源二轮恒（主预判候选一落空如实记）＝**元数据漂移形命中（候选二）**：pre=58d15621/130,431,416 stale→mid=post=**c6c9b002/130,431,496**＝同码 +80 字节漂移；**零换代**（无对端新码刀＝#14 未触发）。**跨树互证重磅**：对端 kb239（R6390＝6872e45f2）与 kb240（R6400＝c257bc911）独立记录 binary **130,431,496＝c6c9b002 ×4 恒入册**＝与我 mid=post **同 md5 跨树同 canonical face**（码面同态直证＝docs 零嵌入直证续证）。
- **判别对新观察（机制细分候选留勘）**：对端 kb240 run1＝同日 docs churn＋C=6（metadata 链 ×6）→**binary 恒（输出恒）**；本轮＝同日同码 docs churn＋**C=8**（metadata ×6＋style-system→layout-engine ×2 位差重编）→**+80B**。两对并读＝**漂移载荷与 +2 crate 重编共现、metadata 链单独重编字节恒**＝kb240「同日恒」判读定义在本流 C=8 形下需细分（对端自记「判读定义修正候选」同域）；+2 重编归因未定（本树无 style-system 码变＝probe check 干扰候选/cp -a mtime 候选留勘，不妄断）。
- **窗纪律**：reflog 窗区间 [01:00:09, 01:31:11] **零条目**（最新 00:57:59 臂前 pull）＝**纯窗形复挂**（s735 十五连破后首回）＋**四点直证形恢复**（arm=judge=legs.head=收账 HEAD=ac2a45d87＝第 53 演四点同值）＝pull 置于臂前纪律兑现；**fetch 观察留置纪律首演兑现**（候窗期 fetch 观察到 dec06d4f5 等 4 提交 docs-only 到货，故意不 pull 保四点直证形，收账后 pre-push pull 吸收＝s736 新课入册）。
- **候窗**：31m02s（01:00:09→01:31:11）＝近期最长（外部 mykernel-2 qemu/cargo 负载驱动峰值 load 12.21）；waiter deadline 9000s＋rearm ×3 吸收，judge clean-soaked procs=0 load<3 across 3×45s 判定窗开＝浸泡门设计承压验证。

## 窗与布防

- armstart.s736 **01:00:09/ac2a45d87**（部署手落件两行形）→judge **01:31:11**（clean-soaked，HEAD=ac2a45d87）＝候窗 **31m02s**→leg1（**C=8** 重编波）→leg2（C=0）→legs.done **01:37:02**＝judge 起至 legs 毕 **5m51s**→桥→solos→all.done **01:37:43**＝judge 起至全毕 **6m32s**；leg1.end 01:34:17/leg2.end 01:36:24；rc 2/2 双腿已知形。
- reflog [01:00:09, 01:31:11] 零条目＝窗内零 commit/pull/push＝纯窗复挂；legs 相零 pull 纪律恒持。
- 布防（上一 session 完成）：cp 前 ls→rearm 改名先于清扫→b736.keys=cp s735.bridge.keys（2cefd313/1748 双验）→wob1 承带核验（894e36fc）→清扫 27 文件（drop b735.keys＋probe.s735×2）→blanket sed s735→s736（族 13/7/20/10/4/7/0/0/0＋动态 analyze2/bridge）→round-92 桥头整行重写→phase5 标签 9/9→wob r53 头重写＋r52 条目插入（闭括 15→17）→括号平衡门 55/55 depth=0→链 51×3＋真值 19→bash -n 11/11→残检＝**deploy.py 首轮收敛零迭代**（上轮 waapi 锚课收敛兑现）；探针 RC=0。
- 证据：[s735 收账](docs/goal/html-semantics-compat/evidence/2026-11-25-m2s735-genchange13-58d15621-knife10-compound-armpost-windowpull-join-round21-flips30-held-bridge-plus2-insert-adjacent-restored-fail1-s330-solo-127-threepoint-armpost-form.md)。

## 收账细目

- **join（持位 round 22＝五零）**：stable **8517** md5 **057764d3** ≡held 逐字节＝round 22；五 keys 全零（GONE/disagree/NEW/l1only/l2only）；分布 **8275**P/**236**F/5T/1U 恒；**FLIPS=30 恒持**（465d10d8＝刀⑨ face）＝刀⑩ 零扰动续证 round 2；**canon/norm round 4**：0f54fef9/0f54fef9＋8918dfb3/8918dfb3（8519 行×2）；stable.canon **e34668ff** ≡held。
- **runner（漂移形＋跨树互证）**：三点 pre=**58d15621**/130,431,416 stale→mid=post=**c6c9b002**/130,431,496＝**+80B 同日同码漂移**（s735 leg1 树 c5818cb57 vs s736 leg1 树 ac2a45d87＝差 4bd0015fb＋ea1e558e9 均 docs＝码零变化）；**leg1 Compiling=8 Fresh=0 全链枚举**＝zero-product-version/zero-net/zero-page-runtime/zero-style-system/zero-layout-engine/zero-engine/zero-webview/zero-wpt-runner（较 C=6 基形 +2＝style-system/layout-engine 位差，归因留勘）；leg2 C=0 Fresh=0；bridge.runner.md5=c6c9b002。
- **全桥（吸收完成形＝回位）**：`1748 passed; **1 failed**`（27.09s；rcb=**101**）；bridge keys **2cefd313**/1748 ≡b736 逐字节＝**NEW=0 GONE=0**（±2 形吸收完成＝主预判逐字兑现）；**FAIL=1＝s330 十六连红**（chronic 挂账续）＋**insert_adjacent 销账不复发续证 round 2**；s736.bridge.gone.keys=0/bridge.new.keys=0 双落件。
- **solo（第 128 连）**：raw **248×2**→strip3（子串匹配）**247×2**→锚 **eaec4575** 双腿同算＝发射形同值第 128 连；rc 双 2 已知形。
- **wob r53 双腿 hits=1×2 双 Pass＝post-episode-4 hold round 10 兑现**（episode-6 判定门未触发＝band 升级未触发＝wob1.keys 894e36fc 承带恒）。
- **哨位（SENTINELS_DONE＝两臂齐全第 135 演兑现）**：dialog r170 **2P/2P＝hold round 23**；select r175 279 keys 8F **F-set IDENT se.577**；password r175 225 keys 0F；**mpreload r198 15/15P＝稳位第 98 轮**；**fd r216 121/121P**；**ob r168 2F/8P**；**band r169 absent＝聚合形第 154 连**；dup=0；'No such' 0×全 7 log＝**第 131 演**；日期串 26.11.25 run-log 面 **0**（唯一命中＝wob 标签自述文本「date-version flip 26.11.24->26.11.25」×2＝**审计口径污染识别：date-string 审计需排除标签行，s737 起按 run-log 面口径**）。
- **互证（对端 R6390/R6400＝跨树同 face＋判别对）**：对端 kb239「97a5abd3→c6c9b002 ×3 恒入册＝binary 130,431,496＝+16,272B＝Compiling ×8 混合形＝metadata ×6＋style-system→layout-engine 刀链 ×2＝css-parser 零编＝69bf56dc6 实改 style-system 单 crate 直证」＋kb240「c6c9b002 ×4 恒＝run1 C=6 metadata 链 mtime 必触＋输出恒＝docs churn 纯形定谳＝同日恒」＝与本流 mid=post 同 md5（跨树 canonical face 直证）＋C 构成判别对（详见判定节）；对端 kb238（4bd0015fb）已于臂前吸收＝97a5abd3（26.11.24 面）与 58d15621（26.11.25 面）分叉属日期面预期。
- **pull 到货全量枚举（留置吸收形）**：收账时 fetch 观察 origin 领先 4 提交＝**c257bc911 docs（R6400）＋9d4ba6da6 docs（kc5r92 布防＝math-depth 属性刀预注册）＋6872e45f2 docs（R6390）＋dec06d4f5 docs（kc5r91 勘误）＝四提交逐提交核类型全 docs 零码刀**＝窗后 pre-push pull 吸收（零码刀＝s737 世代 #14 主预期仍为持位/漂移域）；对端 kc5r92 math-depth 刀码未到＝s737+ watch 项挂账。
- 探针 probe.s736 RC=0（log 1935B）。

## s737 基线与预期

- join：对账基线不动＝held s634.stable 8517（057764d3）＝**持位形 round 23 主预判**（stable 8517＋五零＋FLIPS=30 刀⑨ face 恒持＋canon/norm round 5＋stable.canon e34668ff）；副形＝对端 kc5r92 math-depth 刀码落树→扰动形按面归因。
- 全桥：**b737=cp s736.bridge.keys**（**2cefd313/1748 承带**＝NEW=0/GONE=0 预期）；**FAIL=1＝s330 十七连红挂账 watch**（对端修复→FAIL=0 rcb=0 首演按面归因）；insert_adjacent 销账不复发 watch round 3。
- runner：s737 世代锚＝**c6c9b002/130,431,496 anchored at s736**；主预判＝**同源恒**（三点 c6c9b002/130,431,496＝对端 kb240 同日恒先例；C 构成 watch＝6 形或 8 形均如实枚举）或**漂移再动形**（+δ 微量＝C=8 再现候选）；副形＝对端 kc5r92 刀码落树→**世代变更 #14** 按面归因；跨日 watch＝s737 窗若跨 11-25→11-26＝日期换代 #15 形预期（VERSION=构建日期机制）。
- solo：raw 248 双腿净形＋strip3 247（子串匹配）＋锚 eaec4575＝**第 129 连预期**。
- 哨位承带：wob r54＝**post-episode-4 hold round 11 watch**；dialog r171 hold round 24；select r176/password r176/mpreload r199/waapi r258/fd r217/ob r169/band r170 absent；'No such' 第 132 演；两臂齐全第 136 演；聚合形第 155 连；onblur 双面第 135 轮；mpreload 稳位第 99 轮；**四点同值第 54 演＋纯窗二连 watch＋pull 置于臂前纪律续**。

## s737 布防清单

- 开工 rule 13：pull 置于臂前（吸收 4 docs 提交＋到货全量枚举课续演）；先核 /tmp/s736 三链自退零残留。
- 探针：独立执行 `timeout 420 cargo check -p zero-wpt-runner --release > /tmp/s737/probe.s737.log 2>&1; echo $? > /tmp/s737/probe.s737.rc`＝RC=0 方可臂。
- `cp -a /tmp/s736 /tmp/s737`（cp 前 ls）＋rearm-s736.sh→rearm-s737.sh 改名先于清扫＋b737.keys=cp s736.bridge.keys（2cefd313/1748 双验）＋wob1.keys 894e36fc 核验＋清扫 27（drop b736.keys＋probe.s736×2＝keep b736 前一代 b737 当前；**mkdir 落件目录先行**）。
- blanket sed s736→s737（行口径族 13/7/20/10/4/7/0/0/0＋analyze2/bridge 动态 pre-count）＋b736→b737 全 occurrence（**课⑧ bNNN 五处**：header×1＋comm×2＋echo 标签＋echo 文件名）＋sed 后 wob 史叙真值表**二十处**（十九处承带＋**b736=2cefd313/1748 baseline swap 形迁移**：prefix 重写后「b736 baseline 2cefd313/1748」消失＝真值表增 **b737=2cefd313/1748 carried**＋r53 条目内 b736 swap 形保留；stale 'b736 baseline 2cefd313/1748 and' prefix 形断言零残留）。
- bridge-extract 头注释**第 93 轮语境整行重写**（round-92→93＋s736 face 语境：pure-window restore/four-point 53/c6c9b002 crosstree canonical/C=8 +80B same-day drift/join round 22/bridge absorption NEW=0 GONE=0/FAIL=1 s330 sixteenth red hold watch/day-boundary note 26.11.25）。
- analyze2 phase5：标签 +1（**r171/r176/r176/r199/r258/r217/r169＋band r170＋wob r54**）＋**wob 段头整行重写 r53→r54**（r53 both legs Pass＝post-episode-4 hold round 10 realized at s736 语境入注：s736 face context＝generation-change-less window on runner c6c9b002 first build (metadata-drift form: same-day same-code +80 bytes vs stale 58d15621, leg1 C=8 Fresh=0 full-chain enumeration, peer kb239/kb240 c6c9b002 ×4 crosstree canonical cross-proof, C-composition discriminator C=6->bytes-constant vs C=8->+80B), join held stable 8517 md5 057764d3 round 22 (five-zero, FLIPS=30 knife-9 face held, canon/norm held round 4 0f54fef9/8918dfb3), bridge absorption complete 1748P/1F rcb=101 NEW=0 GONE=0 vs b736 = FAIL=1 s330 sixteenth red + insert_adjacent not-recurrence round 2, dialog r170 both legs 2P hold round 23, solo dual-legs 248 clean anchor eaec4575 round 128, armstart four-point same-value restored (arm 01:00:09/ac2a45d87 to window 01:31:11/ac2a45d87 direct, 31m02s external-load wait)＝r53 演化条目插入（锚 `(arm 00:30:26/7775ba25b to window 00:36:26/c5818cb57 three-point same-value`＋**17 闭括形→19**）＋尾叙 watch bump **round 10→11/r53→r54**（post-sed 锚＋锚前值对盘面核对）＋r 链 52→53＋s 链 52→53（s685…s736）＋states 尾插 8517＝**课④三者计数 52×3 对齐＋课⑤括号平衡常设门**）。
- 臂起三链 ps 零残留核验＋全绝对路径单臂直发＋ps 恰三＋35s 零误重臂＋armstart.s737 必落（四点第 54 演主预期）＋收账注记 commit 置窗后。
- 哨位逐演：wob r54 hold round 11 watch；dialog r171 hold round 24 逐演记形；fd r217 121/121P；ob r169 2F/8P；聚合形第 155 连；'No such' 第 132 演；date-string 审计**run-log 面口径**（排除 wob 标签行文本）。
- runner 核验：三点 c6c9b002/130,431,496 比对（恒或漂移或 #14 均按面如实枚举 C 构成）＋`cat` 内容口径＋analyze2.out 权威行为准；solo 锚配方续；FLIPS=30 刀⑨ face watch；FAIL=1 s330 十七连 watch；insert_adjacent 不复发 watch；b737 承带 watch。

## 证据文件清单

- /tmp/s736/armstart.s736＝01:00:09＋ac2a45d87；window.s736.judge＝01:31:11 clean-soaked HEAD=ac2a45d87；legs.head＝ac2a45d87；legs.done 01:37:02；all.done 01:37:43。
- runner.pre.md5/size＝58d15621…/130431416；runner.mid.md5/size＝c6c9b002…/130431496；runner.post 同 mid；s736-leg1.rc/leg2.rc=2/2；leg1 Compiling=8（八 crate 枚举在收账细目）Fresh=0；leg2 C=0。
- join：s634.stable.keys=s736.both.keys 8517 行 057764d3；s736.disagree/gone/new/leg1only/leg2only=0×5；s736.flip.keys 465d10d8；l582-1/2.canon 0f54fef9×2；s736-l1/l2.norm 8918dfb3×2；s736.stable.canon e34668ff。
- bridge：s736-bridge.log `1748 passed; 1 failed` 27.09s；s736-bridge.rc=101；s736.bridge.keys=b736.keys 2cefd313；bridge.new/gone.keys=0×2；bridge.fail.keys=s330 单行；bridge.runner.md5=c6c9b002。
- solo：opt-solo1/2 raw 248×2；strip3 247×2 eaec4575 双算；rc 2/2。
- 哨位：analyze2.out wob r53 hits=1×2；dialog r170 2P/2P；select r175 279/8F se.577 IDENT；password r175 225/0F；mpreload r198 15/15P；waapi r257 1F/1P；fd r216 121/121P；ob r168 2F/8P；band r169 absent；dup=0；SENTINELS_DONE=1；'No such'=0；26.11.25 run-log 面 0。
- probe.s736.rc=0。
