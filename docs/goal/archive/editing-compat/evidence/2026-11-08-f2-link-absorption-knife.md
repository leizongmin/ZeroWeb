# F2 link-absorption 刀施工档（2026-11-08）

承 [预研档](2026-11-08-f2-link-absorption-prep.md)（统一 delta 法则）与
[普查档](2026-11-08-knife11-v5fix2-mt-residual-census.md)（F2 family 宇宙）。
本轮刀体施工 + 刀 fixture + 门禁批。基线 = knife11-v5fix2（旧基终判）。

## 口径校正（预研 26 → census 78）

预研档「F2a 26 键」为 main 窗片近似枚举；census 基线残键实测 F2 family
宇宙 = **78 条（(page, assertion) entry 级口径）**：

- F2a createlink 族 58：createlink 先行 23 链 × 2 页 = 46 + 非先行
  （forecolor×4 / underline / underline² > createlink）6 链 × 2 页 = 12；
- F2b 既有-link delete 族 20：`delete>inserttext(a)` 且 setup 含 `<a `，
  10 main + 10 ab。

**judge 与 census 域口径差（勿互减）**：judge-dps.py key = ` :: ` 后
assertion（main/ab 跨窗去重 10092，ab 窗后写胜出，基线 F 123）；
census/calibrate key = (page, assertion) entry 级（20184 raw 行，基线
F 244 = 123 main + 121 ab）。F2P 两域各表：judge 38（assertion 级）、
calibrate 76（entry 级），76 = 38 × 2 窗全翻转互洽。

mt 是唯一权威判读面；门禁期望不预写死，由实测输出经
`f2ga-calibrate.py` 校准（期望是判读参数，不是引擎真值）。

## 刀体（四 shim 文件，+229/-16）

统一 delta 法则的八个集成点：

**通道 A（tags 物化，part01）**

1. `_zwCeTypingInsert(tags, text, f2)`：armed 时滤 u/s（≡link 参照装饰），
   carry font（color/face/size，仅 ≠#0000ff 参照的 delta 落 attribute）内层，
   `<a href>` 最外裹；嵌套次序 a > carry-font > tags > text。
2. `_zwCeF2LinkContext(sc, el)`：上下文单点——href 槽仅 armed 有效（物理
   link 内键入不重裹 `<a>`）；carry 消费须 link 上下文（armed 或 sc 祖先
   物理 `<a>`）。
3. 变异核四臂（D4 / ttags1 / ttags2 / ttags3）条件扩展 `|| f2Wrap` 并透传
   f2 第三参；裸 splice 臂禁入（f2Wrap 时不得走无包裹路径）。

**通道 B（值重放，part01d `__zw_ce_value_restore_overrides` 门内）**

4. armed 且 cmd=forecolor：归一 ≡#0000ff 参照 → continue（不落 font）；
   非参照（brown/black）force 重放绕等值门（black 因 UA 填充
   `'rgb(0, 0, 0)'` 等值门恒过，须绕行；census black got `fooabar` 无 font
   为实证）。

**arm/保全/清面（part06）**

5. collapsed createlink 臂：置 `__zwTypingLink`（href 槽，原 `__zwTypingCmdArmed`
   判别旗标语义不动）。
6. presLink/presCarry 快照+回写：cmdPresOv 14 结构命令（formatblock/indent/
   hr/img/iol/iul/insertparagraph/insertlinebreak/justify×4/inserthtml）
   保全 armed 与 carry 跨命令不清。
7. `_zwSync` 清面：`__zwTypingLink`/`__zwTypingLinkCarry` 一并清（选区
   离开 armed 语境即失效）。

**F2b 既有-link delete 族（part01f）**

8. `__zw_ce_capture_link_carry(ar)`：delete_apply 前捕获 delta——armed 读
   `__zwTypingLink`；物理链形 sc 上行 linkEl 收 color/face/size
   （`_zwVaParseSimpleColor` 归一，≡#0000ff 零 delta 不捕）；armed pending
   形读 `_zwValEffectiveValue(sc, 'forecolor')`；物理链 collapsed 形保守
   不捕。`__zw_ce_delete_apply` 快照回写（carry 仅真捕获时覆盖 snapCarry）。
9. 外壳剥离补真：strip while 条件补「幸存壳置空计数」分支（`_zwJusNodeLength`
   无该特例，wrapper 含空 a 时 length=1 提前退出 → 剥不净）；摘壳改换位
   （insertBefore keepShell 升位后摘 wrapper）+ **R262 活动区吸附回填**
   （换位 insertBefore 触发 spec concept-node-pre-remove 边界调整
   part03.js:23400-23465，caret 吸附宿主位漂出壳——活动 range 折回
   delKeepShell 内 setStart/setEnd）。

## 刀 fixture 与探针三轮

`tests/wpt-runner/wpt-data/editing/run/zz-probe-f2.html` 14 案
（D1-D7 delete 族外壳/carry 形 + P1-P7 pending 形；P7 = styleWithCSS
true harness parity；P6 = 裸 forecolor 诊断钉 pin 当前态）：

- run1（刀前基线）3P/9F；
- run2（首版 unwrap）11P/3F——D2/D3 got `foo<a…></a>abaz` 形：换位摘壳后
  caret 漂出幸存壳（R262 吸附），→ 集成点 9 回填修复；
- run3（终版）**14P/0F 全绿**（/tmp/f2-probe-run3.log）。

探针环境注记：P6 裸 forecolor+inserttext 重放在探针单页环境不 materialize
font（got `fooabar`）；mt harness 同族重放可 materialize（census :63 got 含
font）。环境态差异根因未查（FIXME 在案），无语料行、mt 为权威，P6 pin
当前行为作翻动哨。

## 门禁批

七腿（probe-f2 14/0 → probe 224/17 → ip 零漂移 → mt 校准判读 → 残键交叉
核对 → enginelib 6-red 名册 → iol/iul 恒一 → battery 七腿 F=0）。

- run-1（05:46:58 发射）：probe-f2 14/0 ✓、probe 224/17 ✓、ip 零漂移
  `VERDICT OK`（7195 恒一）；mt 腿 06:06:30 被**外部 SIGTERM 击杀**
  （`make: Terminated` rc=143——跨 clone cmdline 同形误杀嫌疑，并行轮
  竞态先例见 ACTIVE-BATCH 账本），孤儿 runner 持批锁 fd 继续跑（语料尾
  缓冲），log 后由 v2 补跑覆盖。
- run-2（v2 门批 06:40 接力，mt 补跑 TIME_LIMIT 7200 后判读）：
  - **mt judge（assertion 级，vs knife11-v5fix2 基线）**：rows 10092→10092、
    P 9969→**10007**、F 123→**85**、**F2P 38、P2F 0**、post-only 0、
    lost 0、`VERDICT OK`——零回归；
  - **mt calibrate（entry 级）**：universe 78、**F2P 76**（in-family 76、
    **族外翻转 0**）、P2F/postF/lost 全 0、**stay-red 2**；mt raw
    P 20016 / Fail 168 = 244−76 双通道互洽；
  - **预测命中**：outdent/forwarddelete（不在 cmdPresOv 门）实际全翻转
    （no-op/删除路径不清 armed），白名单预测淘汰；
  - **stay=2 = `createlink>inserthtml(ab<b>c</b>d)>inserttext(a)` ×2 页**：
    基线 got `fooaba<b>c</b>dbar`（typed a 裸落）→ 本刀 post got
    `fooab<a href="http://www.google.com/">a</a><b>c</b>dbar`——wrap 已
    发生，typed a 落在 inserthtml 内容**起点**（caret 错位处），期望在
    内容**末尾**——根因 = inserthtml 后 caret 未移至插入内容之后
    （`_zwExecCmdApplyInsertHtml` part06.js:2938 `kids.indexOf(ec)` 陈旧
    快照索引，**F3 inserthtml 簇缺陷**，census 挂账 12 键同源），非 F2
    刀缺口——白名单收编挂账随 F3 刀处理；
  - 06:20 并行核验轮 hunk（NONWL 正则 census 剥参渲染适配）采纳，归属
    见 ACTIVE-BATCH 账本。
- 余腿（07:28:09 `ALL-DONE`）：enginelib **3814P/6F**（6-red 名册恒一
  `FIX-GATE ROSTER OK`）；iol 1761 恒一（F2P 0 `VERDICT OK`）/ iul 2065
  恒一（F2P 0 `VERDICT OK`）；battery 七腿全 OK（880/420/1104/552/3456/
  2607/1455，F=0 T=0）。
- run-2 判读终数：**F2P 76 entry 级（38 assertion 级）/ P2F 0 / 族外 0 /
  stay 2（F3 挂账）/ 全腿零漂移**。

## 已知边界

- 预研档 main 26 口径与 census 78 双页口径差 = 枚举粒度（main 近似 vs
  (page, assertion) 精确），census 为准，语义模型无变化。
- enginelib 6-red 名册（knife10c 档）恒一红线；SMIL/net 外债归因见
  [roster 档](enginelib-red-roster-2026-11-07-knife10c.txt)。

## v3 换基重验批（2026-11-08 08:36→09:38 ALL-DONE，组合态回执）

刀体推送 5470e071a（rebase 并入远端 11 提交）后，组合态门禁按换基门先例全新跑
（batch-f2-link-absorption-gate-v3.sh，head=d34a3404d 单干净二进制；此前两发 ABORT 均
系 multitest-ab 页级 5min 预算超时尾截断伪象——lost 88 全落 ab 尾段、judge 键域仍全绿，
第三发复现消失判 flaky 收口）：

- probe-f2 **14/0**、probe **224/17** 恒一；
- ip **7195 行恒一** P 6989/F 206 零漂移 `VERDICT OK`；
- mt judge **rows 10092 恒一、F2P 38（assertion 级）、P2F 0、post-only 0、lost 0、
  Timeout 0**（P 9969→10007、F 123→85，raw 20016P/168F 共 20184 行完整非截断）；
- calibrate **entry 级 F2P 76、universe 78、族外 0、stay 2**（= createlink>inserthtml
  ×2 白名单）、残键 168=244−76 互洽；
- enginelib **3815P/6F**（P +1 系基座新增测试，6-red 名册恒一 `FIX-GATE ROSTER OK`）；
- iol 1759P/2F、iul 2061P/4F 行恒一零漂移 `VERDICT OK`；battery 七腿全 OK F=0 T=0。

**VERDICT：组合态（刀体 + M2-S340/S341 + shadow M3 入站栈）全绿回执，F2 刀收口闭环**；
F3 inserthtml caret 后移刀修复窗开（处方 07:22 定谳、红线 07:26 口径）。批日志
`.acceptance/dps/batch-f2-link-absorption-gate-v3.log`。
