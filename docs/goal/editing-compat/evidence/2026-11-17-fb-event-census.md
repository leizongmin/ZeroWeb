# fb 标准页＋event.html 普查与 Chrome-160 对账（2026-11-17）

普查轮双腿 + 上游三方对账，锚定 af733e29f，bin f4cf0af85c1cb41ab57f328aa895e669（fb/event/evtgt 探针三腿同二进制，event 腿零 rebuild 实证）。

## 腿 1：fb 标准页（formatblock.html 六窗全量）

**5042P/4F/0T，rows 5046 恒一**（窗覆盖 1000×5+46 实证零缺窗）。vs tq v7 锚（13535525b，5033/13）：

- **9 窗界伪影全溶解**（del/quasit/xmp/listing/h5 单块形 ×5＋h1 合并形 ×4）。h5 六读面行（qI/qS/qV×dps/fb）Pass 行实证同键翻绿；formatblock.js 语料 13535525b..af733e29f 零提交（0 commit 实证）＝非语料漂移。溶解机制＝runner/engine 管线漂移（13535525b..af733e29f 间 11 runner 提交＋img 刀/d50/css 刀候选），精确刀点未定谳；recipes：worktree @13535525b 旧 runner＋?1-1000 窗（h5 单块形）复跑对照 bisect。窗界伪影本就除名挂账（fb13-r34-verdict），溶解不改刀序，仅记账。
- 残 4F 恰 tq v7 修复页终态形：
  - **R13** compare（cE=false span 三连形 wrap 能力缺口）＝fb 面唯一真差异，刀序保留；
  - R3/R4 dl qI ×2（不可满足互斥对）＋R7 录音复位＝协议伪影，除名挂账维持。

## 腿 2：event.html（selection 通道，180 子测）

**173P/7F**。ef429d0e9 记账「173P 满命中」为该轮 leg 计数（180 分母形未提及）；本轮 7F 逐键：

| 键 | 我方 | Chrome-160 master | 定谳 |
|---|---|---|---|
| Changing selection from handler | F | F | **Chrome 同红**，除名挂账 |
| fontSize "" | F | F | **Chrome 同红**，除名挂账 |
| fontSize "quasit" | F | F | **Chrome 同红**，除名挂账 |
| formatBlock "" | F | F | **Chrome 同红**，除名挂账 |
| fontSize "15px" | F | P | **真差异**，刀 |
| formatBlock "quasit" | F | P | **真差异**，刀 |
| insertImage "" | F | P | **真差异**，刀 |

Chrome 侧实证：wpt.fyi master chrome-160.0.8166.0 run 739652a308 report.json 逐子测提取（173/180；同红 7 键恰含我方 4 同红＋3 我方 Pass 键）。Firefox 162.0a1 summary 163/180（失败面更多，不作对齐基准）。

- **我方超额 Pass 3 键**（Partially-selected editable content / Selection spans two editing hosts / createLink ""）＝Chrome 红 我绿，无动作。
- **evtgt 探针**（zz-probe-evtgt.html，TEMP 刀后删）：bold 案 input 事件 {tgt=EL:DIV, phase=2(AT_TARGET), bubbles, trusted} 实证现行派发形＝host 深派发冒泡外层（event.html `div`＝外层 #test 容器，`div.firstChild`＝内层 contenteditable div——此前「e.target 应为文本节点」读法为误，测试契约与现行派发形吻合）；fontSize "15px" 案 ret=false count=0 实证拒绝路径零事件＝3 真差异键根因（part06 :6553 value_validate＋:6564 fb/img 值守卫 `return false` 前置于事件派发）。

## 刀序（本轮普查产出）

1. **EVT-REJECT 刀**（3 真差异键）：拒绝路径按形补派 input（trusted 印记＋input 时刻 host 现解析，与正常臂同构造；beforeinput 不补——无测试证据要求且 input-events 基线在册防回归）：
   - fontSize：legacy 1-7/±N 拒绝面外，CSS 长度形（"15px" 类）触发补派；""/"quasit" 保持零事件（Chrome 同红形）。
   - formatBlock：标签名归一非空（"quasit" 类）触发补派；"" 保持零事件。
   - insertImage：空值拒绝触发补派。
   - 三键 ret 与树形不改动（录形 ret [false]＋树不变断言在册：formatblock.js :872/:1117/:1455 quasit、insertimage.js :20 ""，fontSize "15px" 全语料零 ret/树面）。fontSize ""/quasit＋formatBlock "" 补派即翻 Chrome 反向 delta，禁做。
2. **R13 cE=false wrap 刀**（fb ×1 真差异＋indent ×2 同族带候选，刀前普查归域）。
3. Changing-selection＝Chrome 同红挂账，无刀。

## 语料面风险对账（刀前定谳）

- editing/run＋editing/other 语料断言 ret/compare/qV 面，无事件计数断言——补派不触。
- input-events 语料（gap-batch1 通道）beforeinput/input 计数面在册：刀前基线 fbc1-inputevents-pre.log，刀后同门复核。
- multitest 语料：fontSize "15px" 零行（data 目录 grep 实证）；X37 双键除名维持。

## 挂账

- 9 窗界伪影溶解机制 bisect recipe 见上，择期定谳（不阻塞刀序）。
- exec1 474 域决策挂账维持；`_zwFbqInsertAtRangeStart` raw splitText fb 面候选注记维持。
- zz-probe-evtgt.html 探针页：EVT-REJECT 刀门禁批后删除。
