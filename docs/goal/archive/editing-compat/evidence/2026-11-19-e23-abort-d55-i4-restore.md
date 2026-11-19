# e23 批 L8 ABORT → d55 跨域回归归因与 I4 ⑩ 臂恢复（2026-11-19）

## 一、事件

e23 门禁批（head `1383ac371`，exec2/exec3 双门刀）L1-L7 全绿后，L8 ip 腿 ABORT：

- 实测 `P=7183 F=12 T=0`（期望 7185/10），pass→fail 翻转恰 2 键；
- failset diff：e23 侧比 `ip1-got-failset.txt` 多 2 键，均为 `editing/run/insertparagraph.html?6001-7000` 窗 `compare innerHTML` 断言面：
  - `[[dps,div],[insertparagraph,]]` `<dl contenteditable=false><dd contenteditable>ab[]cd</dd></dl>`
  - `[[dps,div],[insertparagraph,]]` `<ol contenteditable=false><li contenteditable>ab[]cd</li></ol>`
- 两键形制 = ip I4 族 CE-island host 本体 dd/li 拆分（R22 刀 8368fc4db / b86912191 的目标键）。

## 二、归因链（三证）

1. **锚对账**：ip1 批（`d35da58db`，I4 刀收口批，bin `0e7cc766`）同键 `compare innerHTML` 为 Pass；现 head `1383ac371` 翻 Fail。
2. **静态 diff**：`d35da58db..1383ac371` 链上 js_dom_shim 改动提交逐个核对，`a763e74b1`（d55 刀，input-events 域批面）在 part01.js：
   - 删 `arm1IslandLid`（⑩ CE-island host 本体 li/dd 直子文本中点 caret 窄门）整段，臂门收窄回 `hostTag === 'div'`；
   - `sepTag = (hostTag === 'div') ? 'div' : 'p'` 直通化为 `sepTag = hostTag`。
   - 当前树 `arm1IslandLid`/`⑩ CE-island` 全仓零命中，无等价替换。
3. **钉测复现**：`test_ip_island_li_dd_pins`（part143，I4 语义常驻钉）在 `1383ac371` 树 FAILED（0 passed 1 failed）。

定谳：d55 刀误删 I4 ⑩ 臂致 2 键 P2F；其批面（input-events 语料）不含 editing/run insertparagraph，P2F 漏网。exec2/exec3 双门刀（1383ac371）不碰该域，无罪。

## 三、修复（1cbcb7420）

按 `a763e74b1^`（c329bf8ff）原语义完整恢复两处（part01.js，26+/2-）：

1. sepTag 门恢复 `sepTag = (hostTag === 'div') ? 'div' : 'p'`（li/dd host 回退缺省 'p' 防 li 嵌 li 非法形）；
2. `arm1IslandLid` 窄门定义与臂门条件 `(hostTag === 'div' || arm1IslandLid)` 恢复。

两处附恢复注（d55 其余 BS/FD 域改动不动）。

## 四、验证

- 恢复后钉测：`test_ip_island_li_dd_pins` ok ＋ `test_exec_gate_pins` ok（双绿，1 passed 各）。
- e24 门禁批（head `1cbcb7420`，bin `40014d7d0bb16b2fa906bd74f45df478`）：

| 腿 | 门 | 结果 |
|----|----|------|
| L1 exec2 | 1/0 strict | rc=0 **1/0** wall=1s ✓ |
| L2 exec3 | 6/0 strict | rc=0 **6/0** wall=1s ✓ |
| L3 ip | 7185/10 strict＋failset==ip1 恒一 | rc=2 **7185/10/0** wall=953s，failset==ip1 伪影集 10 键逐行恒一 ✓（CE-island 两键 F2P 复绿） |
| L4 ilb | 1691/0 strict | rc=0 **1691/0** wall=33s ✓ |
| L5 lib | 0F＋passed≥4206＋双钉测在册 | rc=0 **4214P/0F** wall=17s，part143/144 双钉测在册 ✓ |
| L6 mt | 10090/2 strict＋failset==r21a-mt 恒一 | rc=2 **10090/2/0** wall=1633s，X37 SET-IDENTICAL ✓ |

**ALL-DONE 2026-11-19T05:49:23+0800，六腿全绿**（writer 轮 429 亡故于发射前，批 setsid 存活自跑完；收养轮按日志判读回填本表）。判读附加对账：e24-ip-failset 与重射 e23-ip-failset 字节恒一（10 键）；批 head `1cbcb7420` 至判读树漂移仅 docs（rendering-compat/wptfyi-css 三笔 .md），零代码漂移，bin 锚仍绑现树。

e23 批 L3-L7（fb 5043/3、fs 2419/3、event 176/4、indent 1329/0、img 962/0，SET-IDENTICAL 对 r21a）已于 `1383ac371` 树全绿实证；修复 diff 仅 insertParagraph ⑩ 臂域（五页命令不消费该域），不重跑。

## 五、教训

1. **跨域刀批面盲区**：刀的批面必须覆盖其修改函数的全部消费语料域。d55 改 part01.js insertParagraph 行断臂域，批面只有 input-events 语料——同一文件不同命令域的回归无法被单域批面捕获。part143 常驻钉测（R22 收口时落库）是本次捕获的第二道闸（批 L8 先捕，钉测复现确证）。
2. **发射后禁改批脚本**同样适用于「复用已终态批脚本重射」：同 session 并发轮改写 e23-fbr.sh 重射两次（第二实例 ABORT HEAD 漂移；第三实例与 e24 同 head 重复），按后射者让位击杀（pid 3338301，死时在 ensure_solo 无在飞腿），COORD-NOTE 双日志留痕后以 e24 为收敛批。
