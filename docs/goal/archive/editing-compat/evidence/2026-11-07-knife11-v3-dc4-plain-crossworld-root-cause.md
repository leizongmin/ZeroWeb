# knife11 v3 DC-4 root-cause 定谳：plain 节点跨域移动语义矩阵与 ① 臂双缺陷面（2026-11-07 19:xx）

head d687c2b62，shim 锚 3b7a64a5（探针期无刀体改动）。探针语料
`tests/wpt-runner/wpt-data/editing/run/zz-probe-knife11-v2.html`（gitignored
runtime-read），批脚本 `batch-knife11-v3-prim.sh`（flock，build+probe+
PRIM 行提取）。三轮 PRIM（P/Q/相 P3 修正轮）+ 真 宿主 A/B（Q6/Q7）。

## 探针三轮演进

1. **P 轮（createElement host，engine 世界）**：P1/P2 engine 父+engine 子
   ok；P3/P4/P5/P7 engine 父×plain 子一律 TypeError `parameter 1 is not
   of type 'Node'`——engine 父桥拒绝 plain 子参数（在挂态）。
2. **Q1/Q2（innerHTML 解析 host，semi-plain）**：plain 父
   insertBefore(engine 子, plain ref/null) 均 ok 且物化——与旧推导
   「:10702 静默 no-op」矛盾，引出世界深度变量。
3. **Q3-Q7 修正轮（primPlainHost 改构 `abc<b>def</b>ghi` 摘 b →
   [abc-text, ghi-text] 同构真实 post-delete 态；Q6/Q7 直上 testDiv
   全 plain 世界）**——定谳数据（batch-knife11-v3-prim.log 19:41）。

## 宿主桥跨域语义矩阵（实证）

| 面 | 操作 | 结果 |
|---|---|---|
| engine 父 × 在挂 plain 子 | appendChild/removeChild | TypeError 拒绝 |
| engine 父 × detached plain 子 | appendChild | ok（Q7 s1 hb+ABC；内容入块） |
| semi-plain 父 × engine 子, plain ref | insertBefore | **物化**（Q1 `<p>X</p>abcghi`） |
| fully-plain 父 × engine 块, plain ref | insertBefore | **静默半挂**（Q6 s1：wrapper 父指针已设 `hb@DIV`，view/childNodes 未动） |
| plain 父 × engine 子, null ref | appendChild | ok 物化（Q2/Q7 s3） |
| engine 块 × 在挂 plain 子 | appendChild | **序列化拷贝 + wrapper 改挂**（Q6 s3：sep=ghi 且 host view 残留原文本 `kids=[#text!(p=P)]`） |
| engine 块 × detached plain 子 | appendChild | 干净移动（Q4：rm=ok app=ok host 清 sep 全） |

世界深度变量实证：同一 insertBefore(engine 块, plain ref) 在 semi-plain
宿主物化（Q1）、fully-plain 宿主（testDiv，markup 链 contenteditable）
半挂（Q6 s1）——桥面按宿主解析深度分流。

## ① 臂双缺陷面 → CASE496 `ghi<p>ghi</p>` 逐字复现

Q6（现行 ① 臂 boundaryText 分支逐字重放，testDiv）：

- s1 `host.insertBefore(headBlock1, boundaryText)`（:10702）→
  fully-plain 宿主**半挂**：view `abcghi` 未动、headBlock1 不入
  childNodes → head 块消失（缺陷面一）。
- s2 `host.removeChild(boundaryText)` → ok（plain 父自有方法，view 正确）。
- s3 `sep1.appendChild(mv1t)`（:10709 raw，在挂 plain 尾兄弟）→
  **拷贝**：sep=ghi 且 view 残留裸 ghi（缺陷面二）。
- s4 `host.insertBefore(sep1, headBlock1.nextSibling || null)`（:10717）：
  headBlock1 不在 childNodes → nextSibling null → sep append 尾部 →
  **终态 `ghi<p>ghi</p>` 与 CASE496 观测逐字节同**。

route/ops 通道空 ≠ ① 臂未走：raw 宿主桥原语不经过 typed-fate 插桩，
空通道恰是 raw 面特征（arc-verify 笔记 `wants=false → FALLBACK` 引句
属 v2 修复前二进制，现锚不适用）。

## 修复处方（已施工，新锚 83d0a29c）

part01.js `_zwCeParagraphMutate` ① 臂（boundaryText 分支 + else 元素
边界分支同治）：

1. **移动纪律**：收集（读 previousSibling/nextSibling 链）→ 统一
   `parentNode.removeChild` detach（try/catch 同 void 臂
   :10652-10660 先例）→ `appendChild` 保序（head 逆序收集正序回放，
   不走 engine 父 insertBefore——Q4 证干净移动）。
2. **挂载延后**：两块建齐后 `host.appendChild(headBlock1);
   host.appendChild(sep1)`（Q7 s3 证 fully-plain 宿主 append 物化且
   记账一致；拆分域前序兄弟保留、拆分域全消费 ⇒ append 位置语义
   等价，且消除两处 engine 节点 ref 查找）。else 分支
   `insertBefore(sep1, refAfter1||null)` 同治。
3. caret 面 `nr1.setStart(sep1, 0)` 不动（sep1 记账经 append 一致）。

影响面：① 臂仅 `block === host && hostTag === 'div'` 宿主直下拆分；
② 臂及以后不动（本刀最小域）。engine 世界宿主同纪律等价（engine 父
removeChild 自有子 + append 均合法路径）。

## 门禁（batch-knife11-v3-fix-gate.sh）

probe CASE496/499 翻 P 快速失败面（Fail 21→17）→ ip 单腿（judge vs
knife11-v2fix-ip-post，预期 F2P=2 / 208→206 / P2F=0）→ mt 全分母（judge
vs knife11-v2fix-mt-post，预期 F2P=2 / 125→123 / P2F=0 / post-only 0 /
lost 0，F2P 键须全落 496/499 族）→ enginelib 6-red 名册恒一 → iol/iul
恒一 → ambient 七腿 F=0。marker 全检：v2 四标记 + paren 0/3/1/1 +
v3 old（`insertBefore(mv1h`/`insertBefore(sep1`) = 0、new
（`headParts1`/`tailParts1`）≥3。判读回执见批后补记。

## 相 P3 补记：首轮门禁 ABORT → 缺陷面二（sepTag 产线 'p' 越权）定谳

首轮门禁（锚 83d0a29c，19:48 起飞）probe 腿 ABORT：P=220/F=21 与修复前
恒一。但 CASE/STAGE 数据面揭示**跨域修复已生效**：STAGE-D ip 步
`html=<p>abc</p><p>ghi</p> sel=P@0`——旧病形 `ghi<p>ghi</p>` 消失（半挂
head 块与拷贝残段两面均愈），结构面修复确证。

残红改判：`got=<p>abc</p><p><b>def</b>ghi</p>` vs
`exp=["<div>abc</div><div><b>def</b>ghi</div>","abc<div><b>def</b>ghi</div>"]`
——**块标签面**：① 臂 sepTag（:10611 双面制产线缺省 'p'）对 unset 面
产出 p，recording 期望宿主 copy div。溯源：双面制 'p' 依据 = rich 组
foo[]bar ×12 + B 簇 12 的 ambient p 形录音——但该两族 caret 在 styled
行内壳内，block = 行内壳（非 host），走 ②b sep2b 面**不经 ① 臂**；
① 臂守卫 `hostTag === 'div'`（block===host 宿主直下）在 mt 语料中仅
496/499 族命中。故 ① 臂 unset 面 host-copy（= v2 容器面 `_zwIpWrapTag`
同规则 transplant；spec execCommand 缺省 single-line container 'div'）
对 rich/B 簇零影响，②b/part01e/part01f 三面 'p' 缺省不动。

二笔施工（build 锚 **22fbfeb8**，批 19:53:03 现算；施工后手算 31ca39d0
系沙盒 fs 对刚落盘文件视图滞后，以 build 时锚为准，内容逐字对账无外来
hunk）：`:10611` 后加 unset 臂 `sepTag = hostTag`
（`typeof __zwDefaultParagraphSeparator !== 'string'` 门，显式 setter
一票胜出不变）+ 双面制刀注括注修正（① 臂移出产线 'p' 清单）。重跑
同批脚本（marker 全检：v2 四标记 + paren 0/3/1/1 + v3 old 0/0 new 4/8 +
wraptag=5 含 DC-4 刀注提及；probe 快速失败面不变）。

## 相 P4 补记：二/三轮判读回执与三笔施工（explicit 面守卫复合门）

锚台账：3b7a64a5 = v2 已提交基线（knife11-v2fix-* 判读基準，19:17 rebase
gate 恒一面全绿：ip 6987/208 F2P0 P2F0、mt 10092 行 F 143→125 F2P18、
iol/iul F2/F4 恒一）；83d0a29c = 首轮（跨域修复单笔）；22fbfeb8 =
二轮；f216b3bd = 三轮。

**二轮（22fbfeb8，typeof 单门）**：
- probe ✓ 224/17，CASE496/499 翻 P（快速失败面通过）。
- ip 恒一面**全绿**：rows 7195→7195，P 6987→6989，F 208→206，
  F2P=2 P2F=0 post-only 0/0 lost 0（VERDICT OK）——F2P=2 主证据成立，
  两案 = `foo[]<br>` 双变体（div 变体经 unset 回退、p 变体经 explicit
  面直取）。
- mt REVIEW：rows 10092→9989，F2P=2 P2F=23 **lost=103**。lost =
  multitest-ab Timeout 页截断（并发编译毒化，solo 重跑归因面）；
  P2F=23 = rich 组全表（11 style 命令 × ±inserttext + createlink，
  `foo[]bar` 形）——typeof 单门对挂起 override 不敏感 → host copy
  越权产出 div，recording 期望产线 'p'（p 形录音 = multitest.js:100
  era）。定谳：① 臂 unset 回退须再叠 override 门。
- enginelib ROSTER DRIFT：`test_insert_paragraph_split_r3254_m3_slice5`
  断言旧 p 契约（① execCommand 面 + ③ `__zw_ce_insert_paragraph` 面
  两处 `tag == "p"`）→ part06.rs 双断言改 `tag == "div"`（v3 host-copy
  契约），单测绿。
- iol/iul 恒一 OK；battery 七腿 F=0 OK（21:08:57 ALL-DONE）。

**三轮（f216b3bd，override 单门替换 typeof 门）**：
- probe ✓ 224/17。
- ip REVIEW：F2P=1 P2F=4，post-only 0/0 lost 0。4 案 P2F 全 explicit
  `["defaultparagraphseparator","p"]`（`[]foo`/`foo[]`/`foo[]bar`/
  `foo[bar]baz`）——二轮→三轮判别式换门把 explicit 面也走了
  empty-override 回退 hostTag，违背本臂刀注自述"显式 setter 一票胜出
  不变"（代码与刀注背离）。F2P=1 = `[dps=div] foo[]<br>`；翻失案 =
  `[dps=p] foo[]<br>`（expect `<p>foo</p><p><br></p>`，三轮 got
  `<div>foo</div><div><br></div>`——二轮 explicit p 面直取即绿）。
  二轮 F2P=2 之谜解：第二案即 p 变体，非状态相关 flake。

**三笔施工（四轮，双门合一）**：① 臂回退条件改
`typeof globalThis.__zwDefaultParagraphSeparator !== 'string' &&
!hasPendingOvr`。语义矩阵：explicit 面（typeof 为 string）一票胜出
不走回退（4 案 P2F 复绿 + p 变体 F2P 复翻）；unset + 挂起 override →
产线 'p' 维持（mt rich 组 23 案复绿维持）；unset + 无 override →
host copy（496/499 族维持）。预测：probe 224/17 恒一；ip rows 7195，
P 6989 F 206，F2P=2 P2F=0（= 二轮全绿面）；mt 与三轮恒一（语料零
explicit dps 命中 ① 臂 → typeof 恒真 → 门退化同三轮）；enginelib/
iol/iul/battery 行为面恒一。四轮批独立 `v4fix` 产物前缀（三轮共享
文件名截断事故不再犯），marker 增 dualgate 计数。回执见下批后补记。

## 相 P5 补记：三轮余腿回执 + v4 探针 ABORT 定谳 + 五笔施工（v5 判别信号）

**三轮（f216b3bd）余腿**：enginelib ROSTER OK（6-red 名册恒一）；
iol 1759 P + 2 F / iul 2061 P + 4 F 恒一 VERDICT OK；ambient 七腿 6 腿
OK，仅 wss（insertparagraph-with-white-space-style）1072 P + 32 F
DRIFT——32 案全带 `(defaultParagraphSeparator: p)` 语料尾注（wss 语料
setup 显式 set dps=p），三轮 override 单门对 explicit 面不设防 → hostTag
越权（与 ip 4 案 explicit-p P2F 同根）；mt lost=93（multitest-ab 页
Timeout 截断，9989/10092 行——并发编译毒化期归因，待 solo 复证）。
22:20:40 ALL-DONE。

**四轮（v4，三笔施工后）探针 ABORT 定谳**：锚 9f408268（三笔后新锚），
marker 全检过（typeof=1/armed=2/link=4），probe **220/21 = 修前态**。
失败集 = 三轮 17 诊断行（13 PRIM + 2 DUMP + 2 STAGE-D，恒红基线）+ 4 案
翻红：IP2-CASE CASE496/499 + multitest 族 `[ip, inserttext×3]
"abc[<b>def]</b>ghi"` / `abc{<b>def</b>}ghi`（即 mt F2P=2 同形键）。
STAGE-D 数据帧（探针相 D 自带 dump）：CASE496/499 命令表 =
`[insertparagraph, inserttext d/e/f]` **零 format 命令**，delete 步后
`tt=["b"]`——arming 源 = 处方 c delete 前导删 styled `def`（F6/Chrome
typing style 随删 survives 语义），非显式命令。同帧 recording 双要求：
① 臂面 host copy（`<div>abc</div>...`）**且** inserttext 走 `<b>` 包裹
（`<div><b>def</b>ghi</div>`）——`__zwTypingTags` 双源在案（显式命令 +
delete 前导），不能作 ① 面判别读；v4 读壳即此 4 案族全翻红定谳（mt 腿
F2P=2 键同病，未及跑）。

**五笔施工（v5）**：废弃 `__zwTypingLinkPending`（零消费臂），改专用
显式命令旗标 `__zwTypingCmdArmed`——置位点仅两处（part06 collapsed
format toggle 臂 + collapsed createlink 臂），_zwSync 同生命周期清
（removeAllRanges/addRange 公共面 :11110/:11130 均过 _zwSync，multitest
每案 setup 防跨案泄漏），part01 ① 臂第三门改读旗标
（`=== true`）。delete 前导只 arm 壳不 arm 旗标 → CASE496 族/mt F2P 键
面 host copy 保持 + inserttext 包裹消费不变；rich 组 23 案旗标门 →
产线 'p'（v2 基线同面）。探针相 C/D 每案 removeAllRanges+addRange 实证
自 arming（本 case ip 语料内 delete 前导）非跨案泄漏。v5 产物前缀 `v5fix`
（v4 abort 证据独立保留 knife11-v4-abort-*）。

## 相 P6 补记：v5 批回执（batch-knife11-v5-fix-gate.log，22:58:56 起飞）

- 构建锚：shim snapshot md5=951358a4（build 时现算，五笔终形）；marker
  全检过：paren 0/3/1/1 + v3 old 0/0 new 4/8 + v5 typeof=1 cmdArmed=5
  linkPending=0。probe 腿 224/17 即修复行为面在二进制内生效的核验
  （learning #28 口径：修前态探针恒为 220/21，翻 P 面只在新刀体出现）。
- **probe ✓**：224P/17F，CASE496/499 翻 P（快速失败面通过）。
- **ip ✓ VERDICT OK**：rows 7195→7195，P 6987→6989，F 208→206，
  F2P=2 P2F=0 post-only 0 lost 0——F2P 双案 = `foo[]<br>` div 变体
  （unset 回退）+ p 变体（explicit 面直取）。
- **mt ABORT（lost=16）**：rows 10092→10076，Pass 19,647/Fail 205，
  F2P=2 P2F=0 post-only 0——共享键 VERDICT OK，但 base-only 16 行触发
  lost≠0 门禁 ABORT。归因：`multitest-ab.html` 页 Timeout（log :9878）
  截断带，16 行（15P+1F，F 为已知红）全部落 ab 页 delete/forwarddelete
  × inserttext 白空格 pre 族邻域，纯截断伪影非行为漂移。与三轮 lost=93
  同页同因（ab 页墙钟预算贴沿，并发编译毒化放大；本批 solo 仍截 16 行
  实证预算边距过紧）。enginelib/iol/iul/battery 余腿因 ABORT 未跑。
- 产物：knife11-v5-probe-post.log / knife11-v5fix-{ip,mt}-post.log。

## 相 P7 补记：v5fix2 续批（2026-11-08，batch-knife11-v5-cont.log）

solo 复证批（01:38:56 起飞，02:29:11 ALL-DONE，head d687c2b62；shim
md5 part01=852f0996…/part06=5538d0eb… 与 v5 构建锚 951358a4 同树，
v5fix2 前缀不覆写 v5 ABORT 证据）全绿，六门全过：

- **mt 全分母 VERDICT OK**：rows 10092→10092，P 9967→9969，F 125→123，
  **F2P=2 P2F=0 post-only 0 lost 0**——红线 496/499 族 `[ip,
  inserttext d/e/f]` × `abc[<b>def]</b>ghi` / `abc{<b>def</b>}ghi`
  双变体逐案兑现（multitest.html?9001-last + multitest-ab 变体重复
  发射 4 raw 行折 2 judge 键，与 P4「mt F2P=2 键同病」同形键预测吻合，
  零计划外翻键；`foo[]<br>` div/p 双变体系 **ip 单腿** F2P=2 键，见
  P6，非本腿）；raw 19940P/244F **Timeout=0**（v5 批 lost=16 的
  multitest-ab 页 Timeout 截断带 solo 复跑未再现，纯并发编译毒化伪影
  定谳；16 行 lost 行域与 v5 批逐案同带）。[键归属勘误
  VERIFY-NOTE-knife11-v5fix2-tailchain-verify.md]。
- **enginelib 名册恒一**：3798P/6F，六红逐名精确（ROSTER OK）。
- **iol 恒一 VERDICT OK**：rows 1761→1761，1759P/2F；**iul 恒一
  VERDICT OK**：rows 2065→2065，2061P/4F。
- **ambient 七腿电池 F=0 全过**：880/420/**1104（wss 复位，三轮
  override 单门时 32F DRIFT 面）**/552/3456/2607/1455，零 Timeout。

v5 五笔终形（跨域移动纪律 + sepTag 三门 + `__zwTypingCmdArmed` 判别
信号 + part06.rs 名册双断言 div 翻转）至此全门禁收口：v5fix2 判读基线
（本批产物 knife11-v5fix2-*）移交换基门（batch-knife11-v5-rebase-gate.sh，
期望零漂移）。

