# ip C4 容器链垫/劈裂刀第一笔——垫底 br 持行豁免（2026-11-15）

刀体 commit **04369d39a**（parent 1421360e0，已推 origin；3 文件 +148：part01c.js +11 臂、
part121 钉测新建 133 行、js_dom_bridge_tests.rs include 注册 +4）。census 残 16F 的 C4 族
①dt 垫键翻绿，残 15F。

## 根因链（三源互证：探针 dPS 实录＋vendored 读码＋port 读码）

1. 语料 :510 空 dt 拆分案：`<dl><dt>{}<br><dd>baz</dl>` dPS div 期望在空 dt 留
   `<dt><br></dt>` 垫；port 实际垫被摘（`<dt></dt>`），选区/exp 全错位。
2. 摘除者＝splitParent 尾清理 `_zwJusRemoveExtraneousLineBreaksBefore` →
   `_zwJusIsExtraneousLineBreak` 对该 br 误判冗余。
3. 判定差异：port 用 `_zwJusPrecedesLineBreak` 免布局近似；br 独撑块行时该近似回走
   撞父块边界报 true（过判）。vendored `isExtraneousLineBreak` :840 用 height 测量
   （隐 br 后块塌高→持行→判非冗余）。
4. 同族先例：vendored :845 li 独子显式豁免（li 靠 marker 持行）。本刀推广到无 marker
   块（dt/dd/p/div…）：**块级直父＋唯一子＝持行**；inline 壳与多子面沿
   precedesLineBreak 原径不变（臂窄门，无行为面外溢）。
   规范锚：https://w3c.github.io/editing/execCommand.html#extraneous-line-breaks

## 钉测 part121（常驻，5/5 GREEN）

`js_dom_bridge_tests/part121_ip_c4_container_pad_pins.rs`：setupDiv121 忠实复刻语料
include/tests.js marker 剥离（含 `{}` 元素边界选区），五形 err/ok/out/sel 四元断言：

| 形 | 入 | dPS | exp out | exp sel |
|---|---|---|---|---|
| ① dt-pad | `<dl><dt>foo<dd>bar<dl><dt>{}<br><dd>baz</dl></dl>` | div | `<dl><dt>foo</dt><dd>bar</dd><dt><br></dt><dd><dl><dd>baz</dd></dl></dd></dl>` | DL@0 |
| ② bq-tail | `<blockquote><p>foo[]<p>bar</blockquote>` | div | `<blockquote><p>foo</p><p><br></p><p>bar</p></blockquote>` | P@0 |
| ③ ul-div-tail | `<ul><li><div>foo[]</ul>` | p | `<ul><li><div>foo</div></li><li><div><br></div></li></ul>` | DIV@0 |
| ⑦ 控制 | `<p>foo[]` | div | `<p>foo</p><p><br></p>` | P@0 |
| ⑧ 控制 | `<div><p>foo[]</div>` | div | `<div><p>foo</p><p><br></p></div>` | P@0 |

②③隔离即绿；语料内仍红（见下）——上下文敏感，非本臂回归。

## 语料判读（v4 修复树实录 /tmp/ip-leg-c4fix-v4.log，ephemeral）

- **7180P / 15F / 0T**（rows 7195；LEG-RC=2 属 make 层 rc，计数门判读非验证信号）。
- **C4① 翻绿**：census 16→15 键，dt 垫键零 F 残（修复在语料上下文生效，非仅隔离）。
- 15 Fail 行按批 judge 同链归类（桶计数与 judge 干跑逐位一致）：
  qV 8 ／ C5 2 ／ I4 2 ／ I5 1 ／ C4② 1 键（compare innerHTML no-op＋qV before 两行）／
  C4③ 1 键（compare innerHTML no-op＋qV after 两行）／未归类 0。
- C4②③ 语料 got 均纯 no-op：②got `<blockquote><p>foo</p><p>bar</p></blockquote>`
  （exp 中插 `<p><br></p>`）；③got `<ul><li><div>foo</div></li></ul>`（exp 劈新
  `<li><div><br></div></li>`）——命令吞没位被前驱毒化，**毒化前驱 bisect 待办**
  （语料窗 FILTER 定位，先按窗界泄漏伪影纪律对账再定读面 bug）。

## 门禁批 v1（c4p-v1）：发射 15:13 @04369d39a bin df3ea92d ksig 2/2

- 设计（判读基线档失传重构——.acceptance/dps v6 后被清理，v3/v6 原始 log 不可恢复，
  改自包含双腿设计）：L0 release 构建＋ksig 2 特征串（「持行豁免」、
  `_zwJusIsBlockNode(br.parentNode)`）＋bin md5 锚 → L0.5 enginelib failed==0 严格
  （**4033P/0F 实测过**，含 part121 钉，提前熔断位）→ L1 ip 双腿同 bin fresh×2＋
  judge → L2 ab 10092/0/0 → L3 mt 10090/2＋X37 双键 → L4 ilb 1658P/33F 严格。
- 防伪影：中性名 systemd 载体＋flock 前置 exec>log＋ensure_solo（comm+SID）＋
  load_wait（≤3.0）＋chk_head/chk_bin 腿间漂移熔断。
- **ip 双腿 fresh 全绿**：A 7180P/15F/0T（wall 1077s）、B 7180P/15F/0T（wall 1512s，
  外域租户拖慢）——同 bin A/B 自证确定性；judge 正确版对真实批腿复判 RC=0。
- **judge 门 ABORT @16:33＝脚本发射后被改写的字节错位**：脚本 mtime 15:15:08 晚于
  发射 15:13:15，bash 启动即把回滚前旧字节读入解析缓冲，judge 执行的是键-only 旧版
  （执行签名 qV=0/C4b=2/C4c=2/未归类=6 与旧版行为指纹精确吻合）；盘上终版（qV 臂
  全行 dict 版）从未被执行。熔断门 fail-closed 拦下的是未验证分类器，刀体无涉。
  **教训：冻结（chmod 400）必须先于发射完成；发射时刻晚于脚本 mtime 即版本错位，
  在飞脚本禁改写**（bash 偏移读纪律的发射序推广）。

## 判读四方同数（L1 门实质通过）

干跑三方（281180/281377 独立提取冻结版 heredoc 对 v4 副本合成双腿＋writer 原版
自证）＋真实批腿独立复判两席（281377 于 c4p-v2.log L1' 段＋兄弟席
c4p-v1-judge-manual.log `VERDICT: IP-GREEN`）——桶全部一致：
**qv8/c5-2/i4-2/i5-1/c4a-0（C4① 语料翻绿在批腿复确认）/c4b-1/c4c-1/unknown-0，
A/B 漂移 F2P/P2F/lost/newF 四零，rows 7195 恒一**。
行格式实证：runner Fail/Pass 行＝`Fail <page>?<slice> :: <key> <assertion-msg>`
（断言消息与键**同行**）——协调账本 VETO 节「消息印在下一行」前提不成立；dict 键=
全余行含消息，按消息归类恰好正确。

## 恢复＝surgical 补批（双席独立收敛→让位归一）

- 两席对 v1 ABORT 各自完成取证后收敛到**恒等方案**（复用 v1 双腿日志＋补跑
  ab/mt/ilb 三腿）：兄弟席 rt9e-tail.sh（systemd unit zw-rt9e，16:37:11 发射）与
  本席 c4p-gate-v2.sh（unit 名撞车→setsid 兜底，16:38:42 发射）。
- 同 clone 双 make 语料腿并发＝磁盘守卫连坐/test-guard 自 exec 灾难面 → 按先行者
  优先＋账本「后射者让位」序：本席叶先 -9 杀腿链（runner→guard→make），脚本以
  「ABORT ab 计数失配」干净自终（c4p-v2.log 留痕）。兄弟批独跑（ab→mt→ilb，
  终态 `^(ALL-DONE|ABORT)` @rt9e-tail.log）。
- 谱系更正（防误杀）：make 1585063/test-guard/rustc 树＝ZeroWeb-2 jsanim 外域租户
  （`/tmp/zw-maketest-jsanim2-163844.log`，16:38:44 起），非兄弟批；兄弟批在
  ensure_solo 等租户清场后串行跑腿。协调全程回执 .acceptance/dps/COORD-c4p.md
  （**ephemeral 路径，不随库**）。

## 终态回填（rt9f 收口链全录）

**恢复链**（前 writer/verifier 双席 17:06 后 429 脱离，续轮接管取证后按 16:53
契约执行）：

- **c4p-v1**（15:13 发射，unit zw-rt9d，pid 497979）：L0 构建 bin=df3ea92d
  ksig 2/2 ＋ L0.5 enginelib **4033P/0F**（含 part121）→ L1 ip 双腿 fresh
  **A/B 各 7180P/15F/0T**（wall 1077s/1512s，B 腿外域租户拖慢；同 bin 自证
  确定性）→ 脚本内嵌 judge 执行**发射前旧字节**（脚本 mtime 15:15:08 晚于发射
  15:13:15，bash 启动即读入回滚前解析缓冲——执行签名 qV=0/C4b=2/C4c=2/
  未归类=6 与旧版行为指纹吻合）ABORT @16:33。**执行版冻结版 judge 对 v1 真实
  腿复判三方同数**（c4p-v1-judge-manual.log `VERDICT: IP-GREEN` ＋ v2 L1' 复判
  ＋ writer 自证）：漂移四零＋桶 **qv8/c5-2/i4-2/i5-1/c4a-0/c4b-1/c4c-1/
  unknown-0**，rows 7195 恒一。教训：**冻结（chmod 400）必须先于发射**。
- **c4p-gate-v2**（16:38:42 发射，unit 名撞兄弟 rt9e → setsid 兜底，pid
  1584911）：L1' 复判过 → 发现与兄弟双 surgical 批并发 → 按先行者优先叶先
  -9 杀己腿链，`ABORT ab 计数失配` 干净自终（c4p-v2.log 留痕，L1' 复判
  RC=0 仍是有效回执）。
- **rt9e**（16:37:11 发射，unit zw-rt9e，pid 1681218）：L0 证据头检＋bin
  恒一过 → L2 ab **两连 ABORT**（首跑 16:50 T=1 整页 Timeout；重射 16:59
  `rc=0 P=9832 F=0` 截断 260 行，wall 306s≈300s 页预算）——归因 spin2
  并发租户伪影（ZeroWeb-2 jsanim make test 拖 load，M3-A comb-r1 前同款
  签名；ab 与 ip 刀域无交叠＋截断前零 Fail＝非刀域回归）。
- **rt9f**（17:16:00 发射，unit zw-rt9f，pid 1794554，续轮按账本契约重射；
  脚本 md5 4addb074… 400 冻结先于发射）：证据头检＋L0 bin 恒一过 → L2 ab
  **rc=0 P=10092 F=0 T=0 wall=277s OK**（租户清后干净跑）→ L3 mt → L4 ilb
  （在飞，终态行待回填）。

**终态链（rt9f ABORT → rt9g 三 strike → rt9h 乘数终审 ALL-DONE）**：

- **rt9f 终态＝ABORT mt Timeout**（18:05，tail.log 留痕）：L2 ab 干净绿
  `rc=0 P=10092 F=0 T=0 wall=277s`（执行域＝发射原版**无乘数**——乘数只存在于
  盘上 191bc206 变体，18:14 才被误执行；277<300 系预算内真过）；L3 mt wall=1941s
  T=1 整页（?9001-last）预算耗尽截断 **P=9667 F=2**——截断前 F=2 已实录恰 X37
  双键（subscript+formatblock）＝语义面零漂移信号。死因：ZeroWeb 外域 clone
  release 全量构建（rustc ×N ~95% CPU 17:40+ 起入，load 峰 14）拖页吞吐——
  rt9e ab 双 ABORT / s476「整页 Timeout solo 回位」同款环境相位伪影。
- **rt9g 重射**（兄弟席 18:07:35 发射，pid 2054820，md5 f99e6839…；writer 同刻
  撞车已按账本让位归一）：仅重跑 L3 mt＋L4 ilb（ab 引 rt9f 绿账、ip 引 v1 绿账，
  三段同 bin df3ea92d 拼链）。执行域字节经 fd 取证定谳：`/proc/<pid>/fd/255` →
  inode 288007 (deleted) md5 **f99e6839**＝发射时版本；盘上 a36b0216（18:10:58）
  ＝writer v4 草稿（load_wait 放宽 60min＋ensure_solo 补 cc1）**非执行域**——
  置换走 rename 新 inode，无 v1 偏移读污染面。执行版 L3 mt **无乘数**（裸 300s
  页预算）→ 18:11 账本预授权裁决规则：solo 干净窗再 T≥1＝第三 strike →
  rt9h＝mt ZW_TIMEOUT_MULTIPLIER=2 立即重射不开辩；「贴天花板页族」事实
  无论胜负均入册（同 bin 同页 ~10% 墙钟散点＋rt9e ab T=1@load1.61）。
- **rt9g 终态＝第三 strike ABORT mt Timeout**（18:46，rt9g-tail.log 留痕）：L3 mt
  rc=2 **P=10074 F=2 T=1** wall=1866s，T 页=?9001-last 同签名，2F 恰 X37 双键
  零漂移——**solo 干净窗（load 1.2-2.6 全程）纯争抢假说出局**；18:16-18:2x 与
  变体误重跑 ab 腿并存窗（18:19 load 3.30）在 T（18:44-18:46）前 ~25min 已结束，
  定谳不受影响。18:11 预授权规则生效：第三 strike＝乘数终审，不开辩。
- **18:14 证据灭失事故**：盘上乘数变体 rt9f-tail.sh（191bc206，17:40 Write 留档
  副本）于 18:14-18:15 被误重跑（疑 herd 哨按 rt9e→rt9f 先例重拉
  rt9f-launch.sh；.rt9f.lock 在 rt9f ABORT 后已释放，flock 不拦）——截断
  rt9f-tail.log（终态行灭失）与 rt9f-ab.log（ab 原始 log 只余 1797B make 序言
  零行，输出缓冲未落盘即死；重跑链现无 bash/fd 属主＝死透）。**ab 腿原始 log
  不可恢复**，机检回执链留证：rt9g-tail.log :2-4（18:07:36 rt9g 门对灭失前
  ab log 实测「证据头检 OK（v1 A/B 7195/15 ＋ rt9f ab 10092/0）」）＋四席
  COORD 收据＋上 rt9f 条目。rt9f-launch.sh 已隔离改名
  rt9f-launch.sh.quarantined-1814；盘上变体保留 forensics 勿执行。教训：ABORT
  后重拉同名发射器前必先核盘上 md5 与账本一致；同名产物脚本在发射窗换字节后
  须删/隔离。
- **乘数终审裁决**（预授权规则落地）：三 strike（rt9e ab @load 1.61／rt9f mt
  外域 load 窗／rt9g mt solo 干净窗）＋ab 同页双跑散点 277s（rt9f 无乘数过）vs
  306s（rt9e 杀点）vs 317s（rt9h ×2 全跑完）＝**刀后页族贴 300s 预算顶**（垫
  br DOM 增重 ~3% 吞吐降 × 尾切片堆分配债最重页；非纯争抢、非悬挂——臂纯谓词
  无环向量＋ip 双腿 T=0 直证）。处方＝ZW_TIMEOUT_MULTIPLIER=2（writer v3 原案；
  testharness.rs :66-73 mega-case 乘数先例；页预算＋案 deadline＋看门狗同乘；
  ilb 远离天花板不乘）。**后续同规模 corpus 批默认带乘数或分片**。
- **rt9h ALL-DONE 20:04:50**（18:53:41 发射，unit zw-rt9h，pid 2309915，writer
  席；执行版 md5 a7d50b0b 400 冻结先于发射；v1 三射死于「ab 实录」头检 grep
  已灭失日志，v2 改 fresh ab 腿重跑＝一手证据再生）：L1 证据头检（v1 ip 双腿
  7195/15＋IP-GREEN）＋L0 bin 恒一过 → L2 ab ×2 **10092/0/0 wall=317s OK** →
  L3 mt ×2 **10090/2/0 wall=2053s OK（X37 双键恒一，乘数2 末页清）** → L4 ilb
  缺省 **1658/33/0 wall=36s OK（零暴露核定）**。**gate 严格计数门全过：ip 双腿
  （v1）＋ab/mt/ilb fresh（rt9h）同 bin df3ea92d 全绿，C4 垫刀 corpus 义务
  销账。**

## C4②③ 毒化假说证伪——窗界跨页伪影机制（2026-11-15 18:40）

E 矩阵（zz-bisect-c4ul，带 flip=swcss=true）五实验**全绿**——语料红案单案/紧邻
前驱对/跨形前驱/反向感染全部复现不出，推翻页级状态与紧邻前驱残留两假说后转
读码定谳：

**harness 结构**（include/tests.js runConformanceTest :5556）：before/after query
是**裸调用**（:5588/:5620，无条件执行、副作用读 global）；execCommand 只活在
**exec-return subsetTest 子测**（:5610，窗过滤可整组跳过）；compare（:5667）与
noneditable（:5643）是子测，对 **exec 留下的 DOM** 断言（自身不重跑命令）。
窗界 1000 行切口落在单案子测区间内时＝**exec 在窗 N 页跑、compare/query 在
窗 N+1 页跑**——N+1 页全新 document：`__zwDefaultParagraphSeparator` 未置
（qV 读数恒 'div'，part06.js:6391 单写者 :6239 单读者定谳）、host 未执行
→ 比对＝纯 setup 形＝**no-op 签名**（expected 拆分后形 vs got 未拆分输入形）。

**逐案窗键实锤**（c4p-v1-ip-a.log）：ulP 的 exec-return 两行挂 `?4001-5000`
（Pass），compare＋qV-after 挂 `?5001-6000`（expected "p" got "div"＝unset
读数）；bq（:700 bq-tail）exec 两行挂 `?1001-2000`（Pass），compare＋
qV-before 挂 `?2001-3000`。同页实跑的失败案窗键前后一致。

**15F 分解＝真红 5＋窗伪影 10**：

- 真红 5：pre `<pre>foo&#10;[]</pre>`（?1-1000 同页实跑）＋ol/dl
  contenteditable=false 2 案＋div[abc] 2 案（?6001-7000 同页实跑）＝判读桶
  c5-2/i4-2/i5-1 精确对应。
- 窗伪影 10：bq 1＋ulP 1＋qV 8＝判读桶 c4b-1/c4c-1/qv8 精确对应（qV 8 的
  8 窗各 ~1 行频率吻合窗界机制；机制两个：跨页 exec 跳过→after 读 unset，
  与页首 carry 链重置→before 读 fresh 态）。
- **C4① dt-pad 翻绿不受影响**：v4 红→v1 绿系窗内实跑响应刀体（伪影不响应
  引擎修复——exec 被跳过与刀体无关），真修定谳不变。

推论：C4②③ 上下文敏感毒化不存在，part121 钉测已证 ②③ 引擎语义正确；
qV 8 挂账（认领前域决策）消解——非引擎 bug，窗 harness 伪影（确定性：固定
切口×固定行布局→每次跑同伪影→15F 稳定恒一与三轮实测吻合）。

**窗伪影实证收尾（pfx 单页复放，2026-11-15 20:07-20:11，a36b0216 席执行，
22:17 续轮核验）**：zz-b4ul-pfx.html（涉事案 slice(0,302) 单页无窗，
wpt-data gitignored 已落盘）实测 **5014P/1F/0T**（log
`.acceptance/dps/zz-b4ul-pfx-probe.log`，ephemeral 不入库；rc=2 系 make 层
rc，按计数门判读）——唯一残红恰 `<pre>foo&#10;[]</pre>` compare innerHTML
（真红 I5），bq-tail/ul-div-tail 的 compare·qV 行与 span-qV 行全数转绿：
18:38-18:40 机制预测逐项命中，**窗伪影机制定谳实证完成**。残 15F 最终分解＝
真红 5（C5 2＋I4 2＋I5 1）＋窗伪影 10（qV 8＋C4②③ 各 1，债账除名）。
机制与无窗复放验证配方按 [经验沉淀流程](../../../engineering/learnings.md)
入 docs/learnings/（2026-11-15 窗切 census 伪影篇，与 2026-11-07 行级审计
先例互链）。

## 下一步

1. ~~rt9g/rt9h 终态回填~~（已完成：五批链全录入稿）→ 本稿＋控制面
   同稿提交推送（pull --rebase 吸收上游后推；批锚 04369d39a 为 origin/main
   祖先，上游提交不触及 editing-compat 文档域，续轮复核）。
2. ~~窗伪影实证收尾~~（已完成：pfx 单页复放 5014P/1F/0T 预测逐项命中，
   见上节）＋learnings 入册＋控制面 census 重述（2026-11-15 续轮落地）。
3. census 重述后真残 5 案顺推：C5 根层裸 br（2 键）与 I5 pre 换行归一
   （1 键）均为 census 标注小刀，为下刀候选；I4 跨 CE=false 边界写面
   （2 键）域敏感缓。qV 8 挂账撤销（伪影除名），exec1 474 认领前域决策
   维持挂账。
