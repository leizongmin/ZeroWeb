# F3 inserthtml caret 后移刀（2026-11-08）

F2 link-absorption 刀收口时定谳的 stay=2（`createlink>inserthtml(ab<b>c</b>d)>inserttext(a)` ×2 页）与 census 挂账 12 键同源：`_zwExecCmdApplyInsertHtml` caret 未移至插入内容之后。本轮落地 F3 小刀并跑全窗门禁。

## 背景与处方（ACTIVE-BATCH 07:22 定谳）

旧臂（part06.js :2938）：`nr.setStart(host, min(kids2.length, kids.indexOf(ec) >= 0 ? kids.indexOf(ec)+1 : kids2.length))`——`kids` 是 :2901 在 `host.innerHTML` 替换**前**捕获的 childNodes 快照，`ec`（endContainer）已被 innerHTML 替换销毁；`indexOf` 在旧快照上对旧对象仍可命中，但索引对新 `kids2` 无意义——caret 恒落插入内容起点而非末尾（editing spec execCommand insertHTML：caret 置插入内容之后）。

处方：frag 实体解码纯文本长度 `fragTextLen`（与 :2909 `textToHtmlOffset` 同计数法则：`&`…`;` ≤10 字符记 1）→ 文本坐标 `wantCaret = wantStart + fragTextLen`（:2921 `wantStart = textOffset + so` 为 splice 点文本坐标）→ 在替换后的新 `kids2` 上按直子文本长度累计映射：文本子直达 `(node, off)`；元素子边界（`wantCaret === acc+kLen`，含空元素 kLen=0）落 `(host, ci+1)`；位落元素子内 → 单层递归 best-effort（深层末尾兜 `(kc, gk.length)`）；未放置兜 `(host, kids2.length)`。纯文本分支（`!/[<]/.test(frag)`）不动。

## 红线（07:26 口径，batch-f3-inserthtml-caret-gate.sh）

- probe-f2 **14/0**、probe **224/17**（漂移即 ABORT 手审）；
- ip vs knife11-v5fix 零漂移（rows 7195 恒一 P2F 0）；
- inserthtml.html + exec-command 三腿（with-text-editor/recursive/never-throw）vs f3base：**P2F 0**（翻绿自由）；
- mt（vs v3 换基基线 f2ga-mt-post.log）：judge rows 恒一 P2F 0 post-only 0 lost 0；assertion 级 F2P **[10,14]**（预期 12 = inserthtml 链跨窗折叠）；
- calibrate（entry 级，f2ga-calibrate.py + census-mt-residual-v5fix2-keys.txt universe）：翻键 **[20,28]**（预期 24 = 12 链×双页）**全数带 inserthtml 链签名**（族外翻转 0）、**inserthtml 残 stay = 0**（F2 stay=2 白名单收编随此核）、stay 总数 ~120（144−24 ±漂移）；
- iol/iul vs knife11-v5fix2 零漂移；enginelib 6-red 名册恒一（[roster](enginelib-red-roster-2026-11-07.txt)）；battery 七腿 F=0。

## f3base 基线（10:13:27 采集，head=03900b511 pre-F3 二进制绑定）

| 腿 | 行数 | F |
|---|---|---|
| inserthtml.html | 1565 | 119 |
| exec-command-with-text-editor.tentative | 1806 | 474 |
| recursive-exec-command-calls.tentative | 1 | 1 |
| exec-command-never-throw-exceptions.tentative | 6 | 1 |

采集链 `.acceptance/dps/f3base-collect.log`；mt 侧基线沿用 v3 换基重验批产物 `f2ga-mt-post.log`（rows 10092、P 9969/F 85、judge 口径自证 vs knife11-v5fix2 F2P 38/P2F 0/lost 0 VERDICT OK）。

## 刀体（part06.js 三笔）

1. :2899 孤儿 `ec` 摘除（`eo` 保留供 :2925 选区删除跨度）；
2. :2930 函数头注释同步 F3 根因与映射法则；
3. :2942-2990 else 臂 `wantStart+fragTextLen` → 新 kids2 重映射（见处方）。

长度计算用 `nodeValue` 递归求和（`kTextLen` 局部递归，零 `textContent` 调用——OPTIMIZATION：textContent getter 全子树串拼接的中间串分配在 4G 守卫线上是 GC 债）。全量拼接（js_dom_bridge.rs include 序 21 文件）`node --check` SYNTAX OK。part06 md5 = `57631582a4d701c2d2be816fd7c4f6f4`。

## 批史

- **首发 ABORT（2509060，10:14:58）**：probe 腿 P=0 F=0——test-guard 单进程内存超限杀（RSS 4224220KB > 4194304KB 顶，超线 28MB/0.67%；f3-probe.log 尾 kill 记录）。v3 同腿同限通过 ⇒ 剃刀边方差：load 24（ZeroWeb-2 reftest 并行腿）下 V8 GC 饥饿推高 RSS 峰 + 刀体 textContent 全子树物化微量贡献。
- **减配重发（2740063，10:25:13，fresh 全腿）**：刀体 kLen/gLen 换 nodeValue 递归（md5 见上），门槛阈值不动（改守卫配置换绿灯=禁令）。head=03900b511、shim md5 特征断言过、probe-f2 **14/0** 过。若 probe 再内存杀 → solo probe 隔离刀体差量 vs 负载归因再深查（账本既定 triage）。

## solo 隔离与归因链（10:43-11:08）

probe 腿五连杀（test-guard per-proc 4G 线，全为 zz-probe 语料）：

| 腿 | 树/二进制内容 | kill RSS | 时刻 |
|---|---|---|---|
| batch 首发预检 | 03900b511+F3 刀 | 4224220 | 10:14 |
| batch 减配重发 | 03900b511+F3 刀(nodeValue) | 4423144 | 10:25 |
| armA solo | 03900b511 pre-F3（刀不在树） | 4464804 | 10:47:56 |
| canonical bisect | 2d0e74743（锁继承二进制，provenance 存疑） | 4500104 | 10:54:31 |
| 2866317 | viv+append 双 OFF 符号态（孤儿 35s 编译，混合二进制风险） | 4358600 | 11:00:10 |
| ON 对照 2871731 | main 内容 clean build（Compiling 链完整 38.62s） | 4642604 | 11:03 |

**刀体无罪链**：armA（刀不在树的 pre-F3 树态）同腿同杀 ⇒ F3 刀体（textContent 版与 nodeValue 减配版皆然）非 RSS 债来源；债随 2d0e74743（layout R5459/5460 viv 臂族，渲染流域）引入方向递增。

**测量完整性勘误（账本 11:04 注）**：canonical bisect 的 cargo 段含 build-lock 等待且零 Compiling 行——二进制系锁继承非树自证；原定「noR5460」腿 2871731 发射时树已翻 main（二进制指纹 viv+append 符号在），按原目的失效、转作 ON 对照。2866317 系死脚本孤儿编译产物，混合二进制风险同级，两腿均不立 S2。

**零构建决定性腿 Leg X**：viv 臂带运行时 kill-switch `ZW_VIV_SIZING=0`（vertical_block_flow.rs:145，R4428 族门；viv 臂族全系 2d0e74743 引入，-150 行 vertical_block_flow.rs + engine.rs 11 行）。账本 11:04 注钉 canonical 名 f3-solo-vivoff.log；核验方轮 11:06:11 接力执行（/proc 双进程 environ 核实 ZW_VIV_SIZING=0 在，env 直达 renderer）。判读矩阵：ON 对照杀 + Leg X 过 ⇒ viv 臂族坐实，按 §10 移交渲染流；两者皆杀 ⇒ viv 臂排除，疑点移 append 臂（146e8252c）或系统性。

## S2 终局（11:18-11:22，内容归因全排除）

Leg X 与 appendoff 臂补全判读矩阵（上表五腿 + 三腿）：

| 腿 | 内容态 | kill RSS | 时刻 |
|---|---|---|---|
| Leg X = f3-solo-vivoff | 同二进制 `ZW_VIV_SIZING=0` env 翻转（clean provenance） | 4567328 | 11:09 |
| f3-solo-appendoff | viv ON+append OFF（clean rebuild 30.2s） | 4353440 | 11:20 |

2×2 内容四格全杀（vivON+appON / vivOFF+appOFF / vivOFF+appON / vivON+appOFF），F3 刀 on/off 皆杀，静窗（load 1.26-1.4）与满载（load 24）皆杀。按上文 Leg X 判读矩阵前支「两者皆杀 ⇒ viv 臂排除」已成立，appendoff 再杀 ⇒ append 臂（146e8252c）同排除——**唯余系统性漂移（S2）**。

排除项：VM 4 天未重启（`uptime -s` = 11-04）；rustc 1.99.1 / Cargo.lock 未动；语料仅 `zz-probe-f2.html` 今日 07:35 生成（早于 v3 首跑 08:36，两侧同集）；早间 v3 同腿同 guard 同语料三连过（`f2ga-probe.log` 224/17=241 行完账 @08:38）。**漂移 = 10:14 后单进程峰值 RSS +160~450MB；机制（V8/glibc 时序类）未定位，挂账开放式，不阻塞 goal。**

**goal 侧定性**：probe 语料 = 76 页诊断压力页族（knife11-v2 26KB 等，241 行），单进程跑全程峰值 ~4G 贴 guard 线——mt 腿 10092 行轻案一进程可通过（`f2ga-mt-post.log`），probe 每案约 40× mt 案重；早间压线过、午后漂移必杀。仓库既有先例「单进程 200+ 条真实 apply 触发堆分配债须分片」即此。

**修复（不动守卫阈值）**：gate probe 腿改 per-page 分片——76 页各 `FILTER=<basename>.html`（contains 子串，全名含扩展名互斥）独立进程，行数跨片求和仍断言 224/17（case 本就逐案 `run_testharness_html_inner` 隔离，行集恒一）；单页超线则片名显形=免费归因。ip/mt 腿若漂移越线由 gate 逐腿 ABORT 显形后同法处置。

## 终局勘误（11:27，腿位同一性）——上节「唯余漂移」与分片处方撤销

腿位对账（全现档）：晨间 224/17 过账腿 = `FILTER=zz-probe-knife11-v2`（v2 子集 241 行，TIME_LIMIT=900，`f2ga-probe.log`）；下午全部 solo/批腿 = `runner testharness-editing-run zz-probe`（**全语料 ~609+ 行**，TIME_LIMIT=600）。全语料腿 13/13 份日志 Pass=0、无一完账（含更早轮 f3-iso-armA/armA2/armC/armC-bisect、f3-bisect-2d0e74743、f3A-head-probe）——该腿在 4G 守卫线下结构性不可完账，与内容、时段无关。「早间压线过、午后漂移必杀」系**不同腿比较**，撤销；+160~450MB「漂移」撤销；Leg Z（v3 等价内容全语料腿）4213344KB 杀属预期，佐证腿位伪影。推论：R5460（2d0e74743）、append（146e8252c）、viv 臂、F3 刀均**无有效归因证据**（所有判别腿皆跑永不过线全语料），渲染流无需移交。

分片处方（上节）一并撤销：224/17 仅属 knife11-v2 单页，全家 76 页行数无基线（v2 241 + slice 368 已 609 > 224）。**gate probe 腿勘误回单页 v2 腿（FILTER=zz-probe-knife11-v2 TIME_LIMIT=900）**，与晨间门同腿同限同基线（`knife11-v3-probe-post.log` 直接可比）。每案 ~35MB 级驻留 × 重页族仍为真实观察（同族先例「200+ 条真实 apply 分片」），留作独立小题（ZW_TESTHARNESS_MEM_LOG=1 量化），不构成本批阻塞。

### 分级定谳存档注（11:3x→11:4x，第 2 点已被下节对账补正推翻）

本节初版第 2 点（r1 批腿=门腿本尊、门腿剃刀边为真）**错误，撤销**，以 [对账补正](#对账补正114x对上节第-2-点的勘误批腿全语料非单页门腿) 与账本「终版裁定二次修正」节为准。保留两条核验事实：

1. **日志覆盖事实**：f3-probe.log 现内容=r2 过账（224/17，mtime 11:29）；「13 份全语料」的准确口径=12 份 solo 现档 + r1 批腿 transcript 铁证（r1 杀原始尾已被 r2 覆盖，RSS 数字存 §批史与 §solo 隔离表）。
2. **transcript 考古**：原版批脚本 Write @2026-11-07T23:43:41Z（=07:43:41+08，ece7628e session）probe 腿原文 `make testharness-editing-run-one FILTER=zz-probe TIME_LIMIT=600`——期望 224/17 与 judge 基线（knife11-v3-probe-post.log）均系单页产物错误嫁接；f3-probe-diff.txt（10:33）=10:25 批 ABORT 前 judge-diff 残物（IP2-* 行=全语料首部 ip 族 case）。**腿定义漂移自脚本创建即存在**，非后续手误。可复用经验已沉淀 [learning](../../../learnings/bugs/2026-11/2026-11-08-rss-razor-edge-same-content-time-ab.md)（归因批须断言现行门禁腿定义）。

## 门禁判读（r2，probe 腿=单页 knife11-v2）——ABORT 定谳与修刀

**r2 过程（11:28 发射，head=2e4b34fa6+刀 57631582a4）**：前段全绿——probe-f2 14/0、probe（knife11-v2 单页）224/17、ip vs knife11-v5fix 零漂移（rows 7195 P2F 0）、inserthtml.html vs f3base 零漂移（1565 行恒等 119F）、exec 三腿零漂移；mt 腿完账 20184 行（P 20016/F 168），judge rows 10092 恒一 P 10007/F 85 逐值恒等 baseline、P2F 0 post-only 0 lost 0，**assertion 级 F2P=0 越界 [10,14] → 12:16 ABORT**（iol/iul/enginelib/battery 未跑）。

**ABORT 定性：真阳性。** 刀在全部真实语料零行为变化（mt 双层恒一 + inserthtml.html 零翻绿；ip/exec 本就不经 insertHTML）。红线 [10,14]/[20,28] 是处方对刀效的核心预期（12 census 键 caret 后移），未兑现 ⇒ 刀未达成 documented 目的。judge 系 status 级 (page,assertion) 元组，对 Fail→Fail 内容漂移不可见——got 从 expect 邻形 `…d<b>a</b>` 变 `…dbar<b>a</b>` 而 judge 零翻转报告；红线 F2P 作内容敏感 tripwire 的必要性本轮实证。

**根因（三层互证，核验方轮与实施轮独立收敛）**：fragTextLen 循环只实现实体规则（`&…;`≤10 记 1），漏搬 textToHtmlOffset（part06.js :2842-2859）的标签分支（`<`→跳到 `>` 计 0）。碎片 `ab<b>c</b>d` 解码文本 4，旧循环计 11 → wantCaret=3+11=14 越过全树文本 10 → kids2 重映射走完 placed=false → 兜底 (host, kids2.length)=host 全内容末。got 形坐实：createlink 链 expect `fooab<b>c</b>d<a href>…a</a>bar`、got `fooab<b>c</b>dbar<a href>…a</a>`（'a' 落 host 末=兜底位）。代码→算术→行为互证。

**修复（12:17-12:23 落树）**：fragTextLen 循环补 `<`→`indexOf('>')` 跳段（功能差量 md5 f03e048cd；注释补记后终版 md5 `771c2225069bec79d6034365cdba2514`）。独立复核双份：21 文件拼接 `node --check` SYNTAX OK（6,955,909 字节）+ 计数法则 8 案单测（ab\<b\>c\</b\>d→4、a&amp;b→3、孤立 `<`→1、`<>`→0、plain→5）全过。

**修后 solo 预验证（12:22，zz-probe-f3.html，gitignored fixture）**：S1 标记 frag 主案/S3 元素尾 frag/S5 嵌套标签/S6 选区删除 **4 过**=wantCaret 重映射落 spec 位；zz-probe-f3sh P6 dump post-fix：inserthtml 后 sel=T"dar"@1（spec 位）✓。S2 实体 frag/S4 纯文本 frag 2 败=纯文本臂 detached-node caret 既有缺口（pre-F3 既有行为、F3 处方外；census 键全为标记 frag 不受影响，FIXME 挂账）。手审预警：若 r3 mt F2P 仍不足，下一嫌疑=inserttext 落既有文本节点时 pending-link wrap 不物化（mt 为权威）。

**r2 批窗二进制分裂披露**：11:36:55 并行轮 `pull --rebase --autostash` 将 fe9682b73（R5461 layout 修复）带入批窗树——11:28 二进制管 probe-f2/probe/ip，11:43 重编译二进制（31353f20d+刀）管 inserthtml/exec/mt。账本裁定继续：已过腿=无回归断言任意二进制有效，余腿同二进制自洽，probe-f2/probe/ip 零漂移+enginelib 名册+battery F=0 三重 tripwire 覆盖 R5461（渲染域与 editing 预期零交叠，非零即 ABORT 显形）。r3 教训采纳：pull 在发射前完成，整批单一树。

**r3 锚**：head=319c47d63（pull --rebase 后；incoming=s343 script module 调度域 part04/05+runner transform——与 editing 零交集，transform 对无 module 页 verbatim 透传，probe-f2/probe/ip tripwire 覆盖）、part06 md5=`771c2225…`（gate 脚本硬绑定断言）、红线不变。

### 对账补正（11:4x，对上节第 2 点的勘误——批腿=全语料，非单页门腿）

上节第 2 点「r1 两次批 probe 腿即门腿本尊（224/17 钉死单页）」不成立，两条独立证据：

1. **transcript 铁证**：本核验轮 11:25 对账 grep（`grep -oE 'runner testharness-editing-run zz-probe( |$)'`）命中当时 f3-probe.log（=r1 10:25 批 probe 腿输出，尚未被 r2 覆盖）——`-o` 匹配串止于 `zz-probe` 且后随空格/行尾；若该行是 `zz-probe-knife11-v2` 则 `-knife11-v2` 后缀使整式无匹配。即 **r1 批 probe 腿调用行=全语料（无 FILTER）**，Pass=0（完账前被杀）。
2. **同源 learning 自证**：[learning](../../../learnings/bugs/2026-11/2026-11-08-rss-razor-edge-same-content-time-ab.md) 根因第 1 条原文「门禁腿定义**从全家**改为单页（勘误 11:22）」——勘误前门禁 probe 腿即全家，与铁证互洽；learning 另载「门禁单页腿全天未被杀过」，与上节「门腿 10:14 杀 +0.67% / 10:25 杀 +5.4%」两点直接矛盾（该两点若为单页门腿则 learning 该句为假）。

**修订后时间线**：08:38 v2 单页腿静窗过（224/17）→ 10:14 / 10:25 批 probe 腿=全语料（原 08:43 脚本作者性错腿），结构杀 4.22G/4.42G（224/17 期望永不触达，杀先行）→ 午后 solo 全语料 12+1 份全杀（12 份现档可查 + r1 腿 1 份 transcript 在案、现档已被 r2 覆盖）→ 11:29 r2 单页腿静窗过（224/17）。**「门腿剃刀边+负载相关」叙事撤销**（其四时点系列混腿：两点为全语料杀，单页腿全天两时点皆静过），静窗纪律作为操作规范保留但不再是门腿存活的必要条件。终局结论不变（内容皆无罪、F3 刀中性、单页 v2 腿为正确门腿），证据链以本节为准。

## 门禁判读（r3）——PASS，F3 刀收口

**r3 过程（12:41:00 发射，head=cf856b9c2 单一树整批，part06 md5 `771c2225…` 硬绑定断言 + 修后特征串双门；r3 教训兑现：pull 先于发射，整批无二进制分裂窗）**：13:43:19 ALL-DONE 全绿。

| 腿 | 结果 | 红线 |
|---|---|---|
| 基线自证（v3 换基基线 vs knife11-v5fix2） | rows 10092 P 9969→10007 F 123→85，F2P 38 P2F 0 | VERDICT OK（rebase 漂移已入基线） |
| probe-f2 | 14/0 | ✓ |
| probe（knife11-v2 单页） | 224/17 | ✓（与晨间门同腿同限） |
| ip 单文件 vs knife11-v5fix | rows 7195 恒一，F2P 0 P2F 0 | 零漂移 ✓ |
| inserthtml.html vs f3base | rows 1565 恒一，119F 恒等，F2P 0 | P2F 0 ✓（corpus compare innerHTML 不含 caret 敏感案，翻绿不必要） |
| exec-command 三腿 vs f3base | F2P 0 P2F 0 | 零漂移 ✓ |
| **mt vs v3 换基基线** | rows 10092 恒一，P 10007→**10019**，F 85→**73**，**F2P 12** P2F 0 post-only 0 lost 0 | **F2P=12=预期值**（∈[10,14]）VERDICT OK |
| mt calibrate（entry 级） | rows 20184 恒一，**F2P 24** P2F 0 postF 0 lost 0；flips 2 in-family + 22 out-family | **24=预期值**（∈[20,28]） |
| iol / iul vs v5fix2 | 1761 行 2F / 2065 行 4F 恒一 | 零漂移 ✓ |
| enginelib | 3815P/6F | 名册恒一（FIX-GATE ROSTER OK） |
| ambient 七腿电池 | 420/880/1104/552/3456/2607/1455 行，全 F=0 T=0 | ✓ |

**翻键签名独立复核（gate 脚本瑕疵披露后的替代判定）**：gate calibrate 段两处瑕疵——①flip/stay 清单文件落 repo root（cwd 相对重定向）而脚本 grep `.acceptance/dps/` 路径 → 恒 "No such file"；②`:96` 空变量整数比较报 `[: : integer expected`，"mt 校准核验 OK" 系空转分支。判定不依赖该 OK：产物文件移正后独立复核——[f3-mt-flip.txt](../../../.acceptance/dps/f3-mt-flip.txt) 24 行 **24/24 含 inserthtml 签名、0 行不含**（22 out-family 全为 `[fmt]>inserthtml(ab<b>c</b>d)>inserttext(a)` 跨命令链、2 in-family 为 inserthtml 头链）；[f3-mt-stay.txt](../../../.acceptance/dps/f3-mt-stay.txt) 76 行 **0 行含 inserthtml** → **inserthtml 残 stay=0**（F2 轮 stay=2 白名单随此收编）。

**判定**：r3 全绿，F3 inserthtml caret 刀收口——12 census 键 caret 后移兑现，mt 残 85→73F（−12）。红线四元组（F2P/P2F/post-only/lost）与 rows 恒等全程成立；enginelib 名册恒一；battery 七腿零回归。cf856b9c2 纯 JS 域（shim+docs），fmt 不适用。

**遗留挂账（F3 域外 FIXME）**：zz-probe-f3 S2 实体 frag / S4 纯文本 frag 2 败=纯文本臂 detached-node caret 既有缺口（pre-F3 既有行为，census 键全为标记 frag 不受影响）；手审预警未触发（mt F2P=12 足额，pending-link wrap 物化路径无需追查）。
