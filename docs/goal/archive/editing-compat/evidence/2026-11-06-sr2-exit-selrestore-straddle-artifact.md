# 2026-11-06 sr2 出口选区恢复 ×3 定谳——切片伪影签名 A 复发，非引擎债（零源码修改）

日期：2026-11-06。基线 = bq 刀后 fail 面（iol 1755P/6F / iul 2055P/10F，`batch-bq-{iol,iul}-post.log`）。目标族 = 上一轮指定的「出口选区恢复 ×3」：iol `:393` qState-after（期望 true got false）+ iul `:428` div 变体 qIndeterm/qState-after（期望 true/false got false/true）。**终判 = 三行全数 ⑪ 切片伪影（签名 A straddle），引擎行为无缺陷；零源码修改，账面除名。**

## 诊断设计（SR2DIAG）

1. **隔离探针** `zz-probe-sr2.html`（wpt-data 本地，gitignored）：三目标形 × plain/styleWithCSS 双通道 + sr1 两守卫（:38/:43）——**七案全绿**（:393 树逐字合期望 + 选区留 OL 内 + qSA=true；:428 qIndA=true/qSA=false + 选区跨 `b[ar`(LI<UL)..`b]az`(DIV)）；E3 点 sIn=true eIn=true rf=null，出口收钳全管线零脱壳。隔离面无缺陷。
2. **variant 门控语料插桩**（临时改 gitignored 语料页，判读后复原）：包装 `runConformanceTest`，按输入串命中开 `__zwDbgListSel` trace（part01e 六时点 E0-E4 插桩，同 sr1 门控），dump 终态 sel/qSA/TRACE 为断言消息。iol/iul 全腿跑（`sr2-diag-iol.log`/`sr2-diag-iul.log`）。

## 决定性证据

- **iol :393**：?1-1000 页 exec+compare 全 PASS，dump 全 trace（E0-E4：normalize lift → retag ul→ol → 选区恒在 `#text"bar"<LI<OL` 内，qSAX=true）；**?1001-last 页 dump `TRACE=undefined`**＝`_zwListToggle` 从未入内，qSAX=false＝未执行原树读数（bar 在 ul li，最近列表 ul≠ol）。失败断言（qStateA 期望 true got false）恰在该页。
- **iul :428 div**：?1-1000 页 exec 真跑（全 trace mode=disable 管线，qIndX=true/qSAX=false 逐值合期望）；**?1001-2000 页 dump `TRACE=undefined`**，qIndX=false/qSAX=true＝未执行树读数（两端点最近列表均 ul → qInd=false/qSA=true），恰为失败断言 got 值。
- **对照组 iul :433 p 变体**：exec 断言序号落 ?1001-2000 窗内 → 同页 exec 真跑（全 trace）→ 全 query 断言 PASS。同形双变体红绿分裂唯由窗界位置解释。
- **三 variant dump 配对**（iul 6 dumps）：?1-1000 div/p 双 full、?1001-2000 div undefined / p full、?2001-last 双 undefined（该案无断言注册，无害）。

## 机制（subset-tests.js 切窗执行语义）

`subsetTest` 对窗外断言不创建 test 体；每案 execCommand 包在 exec 断言体里。case 断言块 = exec(×命令数) + noneditable + compare + before×3 + after×3（单命令 9 条）——**断言块跨窗界的 case 在后窗 variant 页跳过 exec 但 after-reads 无条件跑**，读未执行原树。iol :393 exec 序号 ≤1000、qStateA 序号 >1000（恰跨 #1000 界）；iul :428 div 同构跨 ?1000 界。期望值来自 Chrome 录音全量会话，切片 variant 下任何引擎都会在这些行读假值——**非 ZeroWeb 引擎可修面**。⑪ 轮 straddle 16 行同族（签名 A），本次为 list toggle 面复发实例。

## 勘误链清算

- sr1 时代「:393 切片跨窗伪影除名」——**结论正确**，当时证据（隔离探针全绿）不足以服人；
- rtl 后归因铺底的「勘误」（「exec 时点读 true 与 harness 终态读 false 并存恰证出口臂事后改位」）——**错误**，未核查失败 variant 页内 exec 是否运行；
- learning [2026-11-06-dbg-probe-read-point-mismatch-harness-assert](docs/learnings/bugs/2026-11/2026-11-06-dbg-probe-read-point-mismatch-harness-assert.md) 旧版「伪影定性错误」+「确定性复红＝非伪影」判据——**双双证伪**（窗界固定 ⇒ straddle 伪影天然逐代恒红），已重写为 variant 门控判别法。
- 归因铺底「:428/:433 dps=div/p 双变体」表述不精确：rtl 基线即仅 div 变体红（`batch-rtl-iul-post.log` 核），p 变体从未红过。

## sr1 出口收钳机制面收讫

sr1 同位重建孪生收钳（:38 形承重）本轮守卫 F/G 恒绿；:393/:428 的「出口臂事后改位」假说证伪——出口臂在真跑 variant 内全管线零脱壳（E3 sIn=true eIn=true）。出口选区恢复域无新增修复，机制域关闭。

## 零扰动证明

part01e 插桩仅 trace 记录无行为分支，判读后 `git checkout` 恢复 HEAD 态（cc62635e3 零 diff）；语料页 SR2DIAG 包装复原。diag 跑批 Pass 计数与 bq 基线逐位恒一（iol 1755P / iul 2055P），fail 差 +2/+6 恰等 SR2DUMP 诊断断言数（iol 2 dumps / iul 6 dumps）——门禁零位移。

## 残差重排（定谳后）

- **iol 6F** = 切片伪影 ×1（:393，本轮除名）+ P10 命令史态 ×2（:448/:478 形 qState-after，不追）+ qVb 窗界伪影 ×1（已除名）+ **真实引擎债 ×2**：styled compare ×1（`<div><ol style…>` × span[baz]）+ noneditable compare ×1（A/B/C ce=false spans）。
- **iul 10F** = 切片伪影 ×2（:428 div，本轮除名）+ P10 ×2（:233/:263 形，不追）+ **真实引擎债 ×7**：divbr straddle ×2（`<div>{}<br>` qState+qValue after，eb1 时代跨窗记档）+ div 边界 compare ×3（`<div>[foo</div>bar]<div>baz</div>` / `<div>[abc…}`<br>` / `<div>[abc…ghi]`）+ noneditable ×1。
- 下一族候选 = iul div 边界 compare ×3（同族归一臂域，br/dps 刀相邻域）或 styled/noneditable compare ×2（iol/iul 共有）。
