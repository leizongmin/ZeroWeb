# evt-reject 刀＋fbc3 门禁批收口（2026-11-17）

## 根因链（普查档 [fb-event-census](2026-11-17-fb-event-census.md)）

event.html 180 子测 173P/7F（fbc1 普查腿，af733e29f 基线 bin f4cf0af85）对 wpt.fyi
Chrome-160.0.8166.0 master（run 739652a308 report.json 逐子测提取）对账：7F 中 3 键真
差异＝**值拒绝路径零 input 事件**（fontSize "15px"／formatBlock "quasit"／insertImage
""——Chrome 派 1 input、我方 0），4 键 Chrome 同红（fontSize ""／fontSize quasit／
formatBlock "" 同为拒绝零事件＋Changing selection from handler target 面）。根因定位＝
part06 :6553 value_validate＋:6564 fb/img 值守卫 `return false` 前置于事件派发（evtgt
探针实证拒绝路径 count=0）。4 同红形补派即反向 delta，禁做。

## 刀体（49d8e758f，发射时名 2667a841e）

part06 两臂＋钉测一文件（crates 域 +165/−6）：

1. `_zwExecCmdRejectFire` 按形判定：fontSize CSS 长度形（"15px" 类；legacy 1-7/±N
   拒绝面维持零事件）／formatBlock 标签名归一非空剥 <>（"quasit" 类）／insertImage
   空串；边界门＝选区端点出可编辑域（enabled 面）维持零事件。
2. `_zwExecCmdFireRejectInput` 补派 1 input：data/inputType 与 apply 形同源＋
   isTrusted 标记；`ret=false`＋树不变保持（录形 ret [false]＋树不变断言在册：
   formatblock.js :872/:1117/:1455 quasit、insertimage.js :20 ""；fontSize "15px"
   全语料零 ret/树面）。
3. 钉测 `part133_evt_reject_pins.rs` 9 形（3 补派＋3 同红零事件＋enabled 面控制键＋
   bold/formatBlock 合法值 2 控制键），单 fn 入 js_dom_bridge_tests include 尾。

刀前自验：probe 9 形矩阵绿＋event.html 176P/4F＋test-guard 单测绿＋clippy -p
zero-engine --lib --tests -D warnings 过。

## fbc3 门禁批（ALL-DONE 14:01:32，head=2667a841e bin 6a2ad920）

批发射链：attempt1 ABORT（test-guard `--compile-first` 不支持 cargo build）→
attempt2 ABORT（ie 腿误用 editing-run-one FILTER= 改写 testharness-input-events
目标，1s P=0/F=0）→ attempt3 13:04 发射（tail.sh md5 1dea3acd 冻结＋exec 前
md5 断言＋systemd unit zw-fbc3）。

| 腿 | 期望 | 实测 | 判定 |
|---|---|---|---|
| fb 主门 | 5042/4/0＋Fail 集==fbc1 | 5042/4/0 wall 739s | 恒一＝刀零扰动 |
| img | 962/0/0 | 962/0/0 wall 12s | i7i1 收官面恒一 |
| fs | 2415/7/0＋Fail 集==fbc2 fresh 锚 | 2415/7/0 wall 59s | 恒一 |
| event | 176/4＋残红恰 Chrome 同红 4 键 | 176/4/0 wall 3s | 真差异 3 键翻绿维持 |
| ie | 1461/1867/3＋Fail/T 集双恒等 fbc1 | 1461/1867/3 wall 163s | SET-IDENTICAL |
| enginelib | 4136/0＋pin-in≥2 | 4136 passed/0 failed 36.24s | pin 2/2（evt_reject＋img_breakout） |
| mt（multiplier2） | 10090/2/0＋残红恰 X37 | 10090/2/0 wall 2069s | X37 双键恒一零暴露 |

writer session 13:17 429 限额中断，批经 systemd unit 独立完走；后继轮（本轮）按
账本协议代跑尾链。

## 后继轮独立复核（本轮，从原始日志）

- event Fail 集 7（fbc1）→4（fbc3）逐键核对：翻绿恰 fontSize "15px"／formatBlock
  "quasit"／insertImage "" 三真差异键，残红恰同红 4 键。
- fb Fail 集逐字节 == fbc1（4 键：dl qI ×2＋dPS qV ×1＋R13 cE=false compare ×1）。
- ie Fail 集 diff 恒等＋T 行集 diff 恒等（1867/3 对 fbc1-inputevents-pre）。
- lib `test result: ok. 4136 passed; 0 failed`；mt 残红恰 X37 双键（subscript qS＋
  formatblock qV，mt ?2001-3000）。

## 组合态 spot check（evtr-comb-spot，14:21–14:25 ALL-DONE 4m07s）

fbc3 判读域 pre-inbound；pull --rebase 并入他流三刀（d51 part01 +132/−2、d52
part01 +18、刀H part10-typed-om +191/−24，均 12:18–12:44 落 origin）。刀脸零漂移
证明＝part06.js blob md5 双头恒等（dce56fc18e）。组合态 head=49d8e758f
bin dda72c25 四廉价腿：

| 腿 | 期望 | 实测 | 判定 |
|---|---|---|---|
| ie | F2P=15 恰 d51+d52 声称面 | 1476/1852/3 wall 144s | 翻绿恰 15 键逐条对账 d51/d52 证据表（BS ul 形 ×3＋FD def[] 形 ×4＋non-collapsed ×8），零新增红零 T 漂移 |
| event | 176/4 | 176/4 wall 3s | 刀主脸组合态维持 |
| fs | 2415/7＋Fail 集==fbc2 锚 | 2415/7 wall 57s | 恒一 |
| img | 962/0/0 | 962/0/0 wall 12s | 恒一 |

意义：d51/d52 两刀在组合态（含本刀＋img 出壳刀）下 F2P=15 全数兑现且 P2F=0——他流
专项 A/B 账在我流全量腿交叉自证。fb/mt 重腿不在 spot 覆盖，由下一刀 R13 cE=false
wrap 刀门禁批在组合态 fresh 锚定承载（ie 新锚 1476/1852/3 自本轮起入册）。

## 挂账

- event.html 残 4 键＝Chrome 同红挂账（除名，无刀）； Changing-selection 归 target
  面同档。
- 9 窗界伪影溶解机制 bisect recipe（census 档）择期定谳，不阻塞刀序。
- R13 cE=false wrap 刀（fb ×1 真差异＋indent ×2 同族带候选，刀前普查归域）＝刀序 2。
- exec1 474 域决策挂账维持；`_zwFbqInsertAtRangeStart` :973 raw splitText fb 面候选
  注记维持。
- zz-probe-evtgt.html 探针页已删（刀门禁批前 writer 清理，find 全仓零命中实证）。
