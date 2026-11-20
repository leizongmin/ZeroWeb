# DC-4 挂账伪影族普查收口＋exec2/exec3 试探键真差异定谳（2026-11-18）

普查轮（R22 后首刀前普查）。背景：R22 ip I4 刀收口后控制面 v1.141 下一步＝
「转挂账伪影族普查轮或 fb 5043/3、fs 2419/3、event 176/4 残红小刀按 DC-4
簇序择小」。本轮普查先对账三面残红构成，**小刀选项为空**（残红全数已定谳
协议伪影/Chrome 同红，见下三分表），转普查收口＋试探键分类。

## 一、fb/fs/event 残红构成对账（普查判读，无真红）

| 面 | 锚计数 | 残红构成 | 定谳档 |
|---|---|---|---|
| fb（formatblock.html） | 5043/3/0 | dl qI ×2（不可满足互斥对）＋dPS qV ×1（录音复位） | [fb-event-census](2026-11-17-fb-event-census.md)：R3/R4/R7 协议伪影除名挂账维持；R13 compare 真差异已由 [R13 刀](2026-11-17-r13-cefalse-wrap-knife.md) 翻绿 |
| fs（fontsize.html） | 2419/3/0 | tbody data-start ×1＋ba[r</font>b]az ×2 | [fs-census](2026-11-17-fs-census.md)：B/C 骑窗伪影四证（窗界分布＋初始值签名＋单窗探针绿＋骑界索引算术）除名转正 |
| event（event.html） | 176/4/0 | Changing-selection＋fontSize ""/quasit＋formatBlock "" | [fb-event-census](2026-11-17-fb-event-census.md)：Chrome-160 master 同红四键；补派即翻 Chrome 反向 delta 禁做 |

三面自 r21a fresh 锚（R21 刀批 18398dd1 era）以来 failset 恒一（r21b 八腿
SET-IDENTICAL 判读＋R22 批域不相交论证）。本轮 led1 普查批（head
`4932e5d95`＝R22 后组合树，docs-only 漂移）fresh 复核结果见 §四。

## 二、ip 10F 伪影机制档案（三分表引证）

ip 现残 10 键（ip1 批 7185/10/0＋组合窗 spot check failset 逐行恒一）＝
[15F 分解](2026-11-15-c4-pad-br-exemption-knife.md)（2026-11-15 18:40
机制定谳＋20:07 单页复放实证）中窗伪影 10 键，真红 5 键（C5 2＋I5 1＋
I4 2）已由 C5/I5/R22 三刀收口：

- **qV dps before/after ×8**：窗 harness 结构机制——before/after query 为
  裸调用、execCommand 只活在 exec-return subsetTest 子测；窗界切口落单案
  子测区间内＝exec 在窗 N 页跑、query 在窗 N+1 页跑（全新 document）→
  after 读 unset／页首 carry 链重置 before 读 fresh 态。zz-b4ul-pfx 单页
  复放 5014P/1F（唯一残红＝真红 I5）逐项命中机制预测。
- **C4②③ compare ×2**（blockquote `<p>foo[]<p>bar` dPS div／
  `<ul><li><div>foo[]</ul>` dPS p）：同机制（exec 被窗界跳过→compare 对
  纯 setup 形断言＝no-op 签名）；part121 钉测证 ②③ 引擎语义正确（隔离
  即绿）。债账除名。

## 三、exec2/exec3 试探键真差异定谳（本轮新增）

R5450 普查记「exec2/exec3 各 1F（tentative），无簇」。本轮 Chrome 对账
（wpt.fyi chrome-160.0.8168.0 master run，revision 71b58bb1，summary_v2
逐子测 127321 案）：

| 面 | Chrome master | 我方（led1 fresh） | 定谳 |
|---|---|---|---|
| recursive-exec-command-calls.tentative | **1/1 OK** | 1F | **真差异** |
| exec-command-never-throw-exceptions.tentative | **6/6 OK** | 1F | **真差异** |

- exec2（1 子测）：input 事件监听内递归 `execCommand("insertText")` 须
  返 false 且零效果（w3c/editing#200 tentative 方向；Chrome 绿）——我方
  允许递归重入。刀形＝execCommand 入口重入门（in-flight 深度 >0 时返
  false 零事件零变更）；刀前须全语料嵌套暴露审计（mt faithful 链/ie
  事件计数面）。
- exec3（6 子测）：unknown-command 六查询面（execCommand/qCE/qCI/qCS/
  qCSupported/qCV）返 false/"" 不抛。我方 `_zwQueryCommandState` 表外命
  令已返 {false,false}（part06 :3254），五读面 fallback false/"" 面
  （part06 :7694/:7709/:7711 尾）——具体失败子测以 led1-exec3-failset
  取数定谳后定刀位。
- 分类：**真差异刀候选**（Chrome 绿我方红，非 tentative 同红）；规模各
  1 键无簇，列下一刀序（exec3 刀面最小先行）。

## 四、led1 普查批 fresh 复核（head 4932e5d95）

批链 led1（收养轮判读：writer 轮 429 亡故后批 setsid 存活自跑完，日志
`.acceptance/dps/led1-*.log`）2026-11-18T22:41:06 起跑 23:41:59 ALL-DONE，
bin 5f4dfe50ae68ae15adcb83b415e24016，七腿：

| 腿 | 结果 | 判定 |
|---|---|---|
| L1 fb | 5043/3/0 rc=2 wall=1020s | SET-IDENTICAL 对 r21a 锚 ✓ |
| L2 fs | 2419/3/0 rc=2 wall=77s | SET-IDENTICAL 对 r21a 锚 ✓ |
| L3 event | 176/4/0 rc=2 wall=3s | SET-IDENTICAL 对 r21a 锚 ✓ |
| L4 indent | 1329/0/0 rc=0 wall=19s | 清零域维持 ✓ |
| L5 img | 962/0/0 rc=0 wall=12s | 清零域维持 ✓ |
| L6 exec2 | 0/1/0（取数腿，计数门不设） | Fail 集 1 键＝Recursive `Document.execCommand()` shouldn't be supported（`assert_equals expected "0" got "0, 1, ..., 9"`＝递归重入逐层插字实证） |
| L7 exec3 | 5/1/0（取数腿） | Fail 集 1 键＝Testing execCommand with unknown command（`assert_true expected true got false`）；五读面（qCE/qCI/qCS/qCSupported/qCV）全 Pass |

exec3 失败子测取数订正 §三预判：六查询面中五读面已绿，唯一红是
`execCommand` 本体对 unknown command——assert 期望「不抛且返 true 形」
（具体断言形态见 [exec2/exec3 双门刀](2026-11-19-exec2-exec3-double-gate-knife.md)），
刀位收敛到 execCommand 分派尾 unknown 臂。

## 五、残面全图（post-R22 三分表）

| 面 | 计数 | 归类 |
|---|---|---|
| fb 3 | dl qI ×2＋dPS qV ×1 | artifact（协议伪影挂账） |
| fs 3 | tbody ×1＋font-tail ×2 | artifact（骑窗伪影挂账） |
| event 4 | Chrome 同红四键 | artifact（同红挂账，补派被禁） |
| ip 10 | qV 8＋C4②③ | artifact（窗 harness 机制挂账，单页复放实证） |
| mt 2 | X37 双键 | artifact（窗界除名，multiplier2 判读面） |
| exec2 1 | 递归禁止 | **真差异（刀候选）** |
| exec3 1 | unknown-command | **真差异（刀候选）** |
| exec1 474 | text-control tentative 面 | handoff（user 域决策门，user 留置） |
| ie 残 | input-events 对端流面 | handoff（对端流域，d54 重锚 1942/1386/3 恒一） |

DC-4 验收账本状态：自认领真红面仅剩 exec2/exec3 刀候选（各 1 键）；
伪影挂账面 failset 全链恒一（R20 终态→r21a→r21b→ip1→led1 连续）。

> 后记（2026-11-19）：双刀已落地（exec 1383ac371）＋门禁批 e23 判读与
> 跨流 d55 误删 ⑩ 臂回归的捕获修复，见
> [exec2/exec3 双门刀](2026-11-19-exec2-exec3-double-gate-knife.md)。
