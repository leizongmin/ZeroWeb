# 2026-11-06 dps 轮——enable 相邻同名列表壳归并（dps compare 族 12 行收口）

qv13 轮残差最大可行动族：iol/iul dps 元组 compare ×12（三形 × iol/iul ×
div/p）+ iol qV-before ×1。本轮归因、修复、门禁与 qVb 面定谳。

## ① 根因归因（双壳 got 的机制链）

失败形状 `<ol><li>foo</ol> <p>[bar]` exec 后 got
`<ol><li>foo</li></ol> <ol><li>bar</li></ol>`（双壳，夹缝空白保留）vs 期望
`<ol><li>foo</li> <li>bar</li></ol>`（单壳，空白移入壳内 li 间）。

机制链（静态推 + got 逐位吻合双证）：

1. p 换型 li 后 `_zwJusWrap`（part01c.js:996-1001）按 invisible 吸收把夹缝
   空白文本节点收进 members；
2. 吸收后的空白成员触发 flat-model innerHTML 整段重写臂（editing host 直子
   文本成员，part01c.js:1008-1014）；
3. 重写臂成员扫描 `members.indexOf(kids[k])`（part01c.js:877）对空白文本节点
   失配——js_dom_shim 文本节点包装逐访问新建（[多族包装 identity
   learning](../../../learnings/bugs/2026-10/2026-10-14-js-dom-wrapper-family-identity.md)），
   吸收时持有的包装与 `host.childNodes` 重访包装非同一对象；
4. firstIdx 只锚 li → prevSib/nextSib 落在空白文本节点（nodeType≠1）→
   reusePrev/reuseNext 归并臂（part01c.js:912-927）不可达 → 恒新壳；
5. 语料三形 got 树与该路径输出逐位吻合（iol/iul 同构）。

语料期望树即 Blink `insert_list_command.cc` mergeWithPreviousList/
MergeWithNextList 等价行为（语料 pin 快照即契约口径）：夹缝纯空白文本节点
移入存活壳归并点后再移 children。

## ② 实现（part01e.js，+55/-1）

enable 主循环 wrap 出口新增归并段：creator 恒新壳臂打 `freshShell` 标，
wrap 返回值（flat 重写后为重定位孪生壳）落位后——

- 后向归并：previousSibling 链上纯空白文本节点（`/^[ \t\n\f\r]*$/`，同
  part01e.js:584 既有形态）按序移入前邻同名可编辑列表壳（`utag` +
  editable 双守卫），再移 children，空壳移除；
- 前向吸收：同一规则作用于存活壳与后邻；
- 全部移动走 `_zwJusMovePreservingRanges`（端点在被移节点内随动）；
- 缩进合并臂返回既有壳内层，不打标不归并（嵌套口径归 normalize 管辖）。

共享 part01c wrap 路径零触碰（justify 等共用面零风险）；q12 出口收钳与
qv13 壳标出口不受扰（端点在移入 li 内 = in-list content，收钳不触发，
壳标落存活壳）。

静态审读附注（不在本轮修）：part01c `_zwJusWrapViaHtml` reusePrev/reuseNext
归并臂（part01c.js:912-927）存在潜在双写（前缀含 prevSib 序列化又重嵌
pHtml 切片），但与机制链第 3-4 步同因——文本包装逐访问新建——臂在本 shim
中实际不可达；语料无空格邻接形走元素扫描路径不经过该臂（iol F2P 恰 6 无
额外修复为证）。列表域归并由本出口承接后，该臂维持死路径，不追。

## ③ 门禁

- 执行体证据：`grep -a -c 'dps compare 族归并出口'` = 1（旧码 qv13 marker 3
  在位），构建行 34.13s 真编译；批内 0.18-0.30s no-op 复用同二进制。
- **探针 zz-probe-dps 11/11 首跑全绿**（editing/run，语料期望树逐字断言）：
  目标 8 形（三形 × iol/iul，div/p 独立性 ×2）全合并中靶；控制 3 形（无邻
  接列表恒新壳 / 跨 tag 邻接不合并 / 实体元素隔断不合并）全不合并。
- **探针 zz-probe-qv13 26/26**（editing/other 入口
  `testharness-editing-other-one`）：qv13 壳标三分读面零扰动。
- **iol 全腿：1741P/20F → 1747P/14F**（judge-dps.py：F2P 6 / P2F 0 /
  postF 0 / lost 0，rows 1761→1761 恒等，VERDICT OK）。
- **iul 全腿：2041P/24F → 2047P/18F**（F2P 6 / P2F 0 / postF 0 / lost 0，
  rows 2065→2065 恒等，VERDICT OK）。
- 残余名册逐位吻合零外溢（详 §⑤）。

## ④ qVb 行定谳：窗界状态重置伪影（除名不追）

iol `<p>[foo]<blockquote><p>bar</blockquote><p>baz` qV dps-before（?1001-last）
唯一失败。泄漏模拟对账（语料序全量 56+84 案 × log 实测 qVb 行）：

- 语料 qV-before 面编码**录音会话顺序泄漏语义**：before 值 = 前案 dps set
  泄漏值（case 543 before="p" → case 548 before="div" 逐对吻合），首案
  before="div" = 录音浏览器初态；
- 我方引擎同构泄漏（`__zwDefaultParagraphSeparator` 跨案持久 + 未设态回退
  'div'），模型 A（回退 div）对账 **iol 55/56 + iul 82/82 吻合**；模型 B
  （回退改 'p'）反劣（iol −1/iul +1 净恶化）——回退 'div' 为全量最优拟合，
  不可改；
- 唯一失配 = 该案恰为 **?1001-last 切片窗界后首个 dps 案**：切片页重置
  state（回退 div）而 expected 沿用全文件会话泄漏链值（前案 set p）。
  与 eb1② divbr 跨窗 straddle 同族（窗界切片 vs 录音全文件会话的状态语义
  差），除名不追。

## ⑤ multitest 门禁批 + enginelib（batch-dps-mt，systemd 脱离）

batch-dps-mt（systemd 脱离，12:36:44 起，head b116a3b53，release 31.01s
构建 + marker `fwdInterim` ×4 执行体证据；flock 单实例 + RUNSTATE 申报）。

- **mt 腿**：`make zero-wpt-runner-release` + `testharness-editing-run-one
  FILTER=multitest TIME_LIMIT=5400`，19284P/832F（rc=2 既有失败带）。
- **judge-dps.py 三段对账 vs qv13 mtpost 基线**：全文件键 **10,092→10,092
  逐位恒等**——P 9676→9676 / F 416→416，shared **F2P 0 / P2F 0**，post-only
  Pass 0 / Fail 0，base-only lost 0。VERDICT OK（红线守：P2F=0 ∧ newF=0）。
  dps 刀对 multitest 全分母零行为差（dps compare 族面在 iol/iul 语料，
  multitest 残差 416F 逐名恒基线）。
- enginelib 腿首跑（judge OK 条件接力 `make guarded-test`，dev-guard scope
  13:09:52 起）：编译期 ~35s 外层 make 遭外部 SIGTERM（rc=143，杀时点
  ≤13:10:27）。杀源排除：journal 无 systemd stop/OOM 事件、磁盘 36% /
  内存余量充足、target/ 完好、test-guard 编译阶段无时限——守卫自杀排除，
  外部来源未定谳（渲染流 r7 构建席 13:10:54 落点晚于杀时点，不能归因）。
  scope 孤儿续跑不受 parent 终止影响：js_dom_bridge 测试二进制 ~13:12
  规范收官 **3,766P/5F**（五红逐名 p56_adopted_snapshot /
  p99s_custom_property / r3276 / r2810 / r2809，attempt-1 日志在案），
  scope 本体 13:12:14 MemoryMax=24G 收杀（peak=cap，后续 crate 编译触顶）
  致 make test 序列未走完。日志存 enginelib.attempt1-terminated.log。
- **enginelib 补跑（并行席代跑 dps-enginelib-retry，13:15:17 起，head
  e62929a58 = 本刀码 + learning 文档，码面同 b116a3b53；本席按 VERDICT
  文件核验采纳）**：同入口 `make guarded-test` 暖缓存重跑，canonical 行
  `test result: FAILED. 3766 passed; 5 failed`（13.67s）= **3,766P/5F
  五红恒一**，失败数与首跑孤儿收官恒一同、名单与 qv13 基线恒一（s249
  CSSOM 重序列化既有回落红，非本刀面）。代跑席并报 workspace 其余单元
  5,397P/0F——原始日志 enginelib-retry.log 17.7GB 失控输出事故已删，
  该分项未独立复核不作门禁依据（tmpfs 曾被写满致并发席输出丢失，13:19
  恢复 3%；事故存证同 ⑮ 盘满口径）。判读账 enginelib-retry-VERDICT.txt
  （原始日志已删，canonical 行 + start 头在案）。

## ⑥ 残债更新

- **iol 残差 14F**：?1-1000 br 边界 compare ×3；?1001-last ×11 = rtl ×3 +
  nested qState-after ×3（③ 族）+ blockquote ×2 + styled-ol+span compare ×1
  + noneditable-span ×1 + qVb 窗界伪影 ×1（§④ 除名）。
- **iul 残差 18F**：?1-1000 ×5（nested qState-after ×2 ③ 族 + br compare
  ×3）；?1001-2000 ×4（blockquote ×2 + nested qI/qS-after ×2 ③ 族，dps
  元组混入系该案 exec 前缀）；?2001-last ×9（divbr qSa/qVa straddle ×2 +
  rtl ×3 + div 边界 ×3 + noneditable-span ×1）。
- dps compare 族 ×12 全收口；下一步族排序：br compare ×6 → rtl ×6 →
  div 边界 ×3 → blockquote ×4 → noneditable ×2 → styled ×1；③ 族 nested
  ×5 与窗界伪影族维持不追。
