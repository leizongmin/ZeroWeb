# dc2-s10k18 刀⑱ 判定：inline on* 属性 handler 消费 + DETAILS 点击翻转 toggle 单发（2026-11-25）

刀⑱＝双提交合账：k18 码刀（rebase 前 4e2b2e2b2／后 `4ae84d869`，R519 + part163 ⑨段钉）＋ k18b 修复刀
（rebase 前 3eb3d9ec6／后 `9c3b8baf1`，R519b + part163 ⑨-f段钉）。v1 电池 GONE 55/66 后残 11 DETAILS →
诊断链定位双发机制 → k18b 去重 → v2 电池五腿零回归收账。二进制：k18 v1 =
`34613d148ebaa3797f2240409937acda`、k18b v2 = `7a694e615ac430534eacd671245ef277`。

## 刀面（根因链）

**k18 R519**（part03 `_zwMElDispatchEvent` s186 块）：MEl plain object 无属性↔属性
反射，s186 旧只消费 JS 属性赋值面（`node['on'+t]` expando 死槽），模板源
`onsubmit/onreset/ontoggle="…"` **属性形**从不执行 → WPT
`dom/events/Event-dispatch-single-activation-behavior.html` 残 66F（k17b 后）100%
按 child 形聚类（FORM<INPUT> 33 / FORM<BUTTON> 22 / DETAILS 11，与父形无关）：
form submit/reset 与 details toggle 派发在 MEl 本地派发路径、inline 属性 handler
缺元素 → acts 缺元素全案 FAIL。修复＝s186 补 `getAttribute('on'+t)` 属性回落编译：
R152 词法域（`with(document){with(this)}`）+ s90 求值换基（`_zwCallHandlerDocBase`）
+ CSP `script-src-attr` 检查点（`__zwCspAttrAllowed`）+ `__zwAttrClear` 窗口；
click/input/change 三特判型不重入（R152 onclick 与 R155 oninput/onchange 拥有面，
避免双执行）。

**v1 电池**：DOM 430→375＝GONE 55（FORM 两族全消）／P2F 0；残 11 全 DETAILS。

**残 11 诊断链**：(a) testharness format_value 的 `[...]`＝同对象重复出现（seen 集
判定）→ acts=[details, details]；(b) solo 页 121/11（页内状态复现）；(c) 插桩诊断
页（已删，备份 `.acceptance/dps/s10k18-PROBE-diag-backup.html`）实证
`tg=2 open=true dump=[DETAILS:open:CHILDROOT|DETAILS:open:CHILDROOT]`＝**一次
open 翻转 + 两次 ontoggle 执行**，两推入均为 child details 本体。

**k18b R519b 机制**：`_zwMElClick` DETAILS 激活块（R152 面）翻 open 属性经
`_zwMDetailsAttrTail4100` → `_zwToggleQueueNotify4009` 排 setTimeout(0) toggle
任务（R4009 spec「queue a details toggle event task」），同块又同步
`dispatchEvent(toggle)`；corpus 的 `await` 双 step_timeout 事件轮转把定时任务转出
→ 第二发。修复＝点击翻转以 `__zwClickToggleSkip4100` **逐节点**标记跳过 R4009
排程，同步派发保留（无事件轮转的 plain 沙箱钉测面立即可见，R152 拥有面）。逐节点
而非全局窗口：互斥 'others' prong 在 setAttribute 尾钩内嵌套关组内其他成员，全局
旗标会连坐吞掉它们各自照常的 toggle 排程；页面脚本直写 `setAttribute('open')`
不受影响。spec：activation behavior 每次状态恰变迁一次、toggle 单发
（interactive-elements#binding-details）。

## 验证链

- probe 21/21（k18，已删，备份 `.acceptance/dps/s10k18-probe-tmp-backup.rs`）：
  s186 属性回落编译复刻（produced=[] → 21/21）。
- probe_tmp_k18b（已删，备份 `.acceptance/dps/s10k18b-probe-tmp-backup.rs`）：
  DETAILS-child × parent 三形 acts 逐项身份转储（诊断输入）。
- 诊断页（已删，备份 `.acceptance/dps/s10k18-PROBE-diag-backup.html`）：修前
  `tg=2` 双推入 → 修后 132 Pass tg=1。
- solo 页 `single-activation`：修前 121 Pass/11 Fail → 修后 **132/132**。
- 回归面：html-semantics `the-details-element` 41P/2F（两案
  auto-expand-fragment 滚动置 0、closed-details-layout 坐标 0＝scroll/layout
  预存失败非本刀域）；`details-toggle-source` 4P（R4400 invoker source 面）；
  name-exclusivity + 键盘激活绿。
- 钉测 part163 ⑨段常驻：⑨-a submit／⑨-b target-only（祖先站不重入）／⑨-c
  reset／⑨-d toggle 激活+open 翻转／⑨-e 三特判型恰序单执行 `oc,in,chg`／⑨-f
  （k18b）双击恰两发+identity+open 复位。引用纪律＝语料同构 getContainer 插入
  （clone root 直挂 handle div 走宿主重建世界 identity 不可用）。s10k17 pins
  guard 全绿（1 passed，4279 filtered）。
- fmt 零 diff＋clippy engine lib+tests 0 warning＋门禁 PASS（两提交）。

## v2 电池终判读（test-guard 包裹，发射器 `.acceptance/dps/s10k18b-launch-v2.sh`，main log EXIT 五腿齐）

| 腿 | 锚（k17b v2） | k18b v2 | 判读 |
|----|----|----|----|
| WC | F 1118 / P 18047 | F 1118 / P 18260 | **GONE=0 NEW=0 双向零差**；Pass +213 全落 `shadow-dom/declarative/gethtml.html` 单页（3979→4192，F 0→0 恒等）＝巨页截断落位 census 增益非修复，ledger 不计 |
| DP | F 222 | md5 `f70f8f97…` 恒等 | SET-IDENTICAL |
| SAN | F 50 | md5 `6e53a750…` 恒等 | SET-IDENTICAL |
| TT | F 362 | md5 `95c054e1…` 恒等（379,705 B 逐字节） | SET-IDENTICAL |
| DOM | F 430 / P 57517 | **F 364** | **GONE=66 NEW=0**，66/66 全限 Event-dispatch-single-activation（R519 55 + R519b 11）；Range-mutations 双页 ±154/±116（F 0→0 恒等，三跑 P 933/839/779 与 952/825/836 全不同）＝已知截断抖动对非回归 |

- 日志：`/tmp/s10k18b-{wc,domparsing,sanitizer,tt,dom}-v2.log`、main
  `/tmp/s10k18b-battery-v2-main.log`（03:22:55Z→03:54:15Z）；v1 对照
  `/tmp/s10k18-*-v1.log`、`/tmp/s10k18-battery-v1-main.log`；pre 锚
  `/tmp/s10k17b-*-v2.log`。

## 收账

- 本刀净增＝**dom +66**（GONE 66／P2F 0），累计 **83,517 → 83,583**；
  距 ≥99,947 差 **16,364**。
- 残面：Event-dispatch-single-activation **132F→0F 全案收官**（k16 132 wholesale
  → k17 121 → k17b 66 → k18 11 → k18b 0）。
- 挂账池不变：san 残 50F；R516 GECN walker template-drill 风险；MEl classList
  indexed 读缺口；MEl clone re-parent RangeError；协议刀 InsertAdjacentHtmlOnHandle。
