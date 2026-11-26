# DC-2 切片 10 刀㊱ verdict——CE 记账融合双键门（fragment-parsing 潜 latent 面收口＝WC +1）

**2026-11-26 · 已落地 `0aa92f851`（推远端；记录腿 HEAD＝刀㉟ `fde571d6c`）；修改 1 文件（part03.js，+39/−6），插桩全剥（剥后零 `DIAG-K3x` 残留），clippy `zero-engine --all-targets` 0 警告（dev-guard scope，/tmp/s10k35/clippy-k35.log 为刀㉟ 面；刀㊱ 纯 js 改动复用同门）**

## 结论

- **WC 语料净 +1**：`custom-elements/connected-callbacks-html-fragment-parsing.html` 主文档键 P→F 回摆（k35 电池 NEW-1 键清零），solo **8P/0F RC=0**。
- **零回归（solo 三页）**：disconnected-callbacks **40P/0F RC=0**（刀㉟ +25 增益持有）、connected-callbacks **40P/0F RC=0**（刀㉞ 增益持有）。
- **k36 电池四腿判读**：见下方电池节（san/dp/dom 零回归、wc GONE 1/NEW 0）。

## 定性订正：k35 电池 NEW-1 键非刀㉟ 引入

- k35 电池（刀㉟ 树）wc 腿出现 `connected-callbacks-html-fragment-parsing.html` 主文档键 P→F。**HEAD 基线（33809a5c6，无刀㉟）直接复现同键 Fail**（solo 实证）＝该回归自刀㉞ 提交（剥插桩后终树）即潜 latent，k35 电池只是首个全量 wc 腿把它显形。刀㉟ 对 HEAD 基线真 delta＝GONE 25/NEW 0（已按此入账，累计 84,207）。
- 机制注：k34 电池（插桩树）该键 Pass、剥桩提交树 Fail——插桩计数器置于 `_ceApplyConn` 函数头不改变键语义，键换面在融合 memo（见根因）；插桩树/剥树行为差异的真实机理未二次定谳（不影响本刀修复与判读，挂档备考）。

## 根因（插桩定谳，diag 三演）

CE 连接记账（`_ceConn`）键在融合视图换面不稳：

1. **Phase A**（`documentElement.innerHTML` 装配）：解析产物为 plain `_zwMEl` 元素，`__zwCeAttachForAdded` → `_ceApplyConn` 以 plain 身份章 `'plain:'+__zwPlainCeId` 记账并派发 cc（P、C 双双 booked+dispatched）。
2. **Phase B**（`documentElement.appendChild(firstChild)` 移动）：融合 memo（`_r126FuseMemo`）给同对象盖章 sel/handle → 插入臂换 `_elKey(sel,handle)` 键查 was → **恒 miss（wasSel=false）** → 已派元素重派 cc(P)#2。
3. cc(P)#2 回调体内 `removeChild(C)+appendChild(C)` 再入舞步：remove 臂（刀㉟ 面）派 dc(C)+清账 → append 臂重派 cc(C)#2 → `calls expected 1 got 2`。

插桩直证：`[CONN X-DP-DIAG key=@__n0 plainId=p1 wasSel=false wasPlain=true phase=after-innerhtml]`——**融合 proxy 透读 `__zwPlainCeId`（plainId=p1），plain 键记账可见（wasPlain=true），单键门漏判**。cc(C)#2 与 dc(C) 全部发生在 cc(P)#2 栈内（phase 记录 in-ccP/in-ccP-done）＝抑制 cc(P)#2 即整链消失。

## 刀面（part03 两站，同型同门）

| 站 | 修 |
|---|---|
| ① `_ceApplyConn` walk 入口 was 门 | 键组并查（sel/handle 键 + plain 身份键）/记账写全组/断连清全组——融合换键不换账，重派被 was 门吸收（探针实证 `plainId` 经融合 proxy 可透读，双键语义可达） |
| ② `_ceUpgradeNode` 升级 walk connect 派发门（15219 面） | 同型同门：was 判定（`!== true` 语义保真——'prop' 放行不改）改键组并查、记账写全组。define 后升级 walk 对 plain 章已派元素换键 miss 重派同形预防 |

## 判读证据（k36 电池，byte-bound diff md5 `e1f2222f`）

- 四腿串行电池 18:49:06–19:04:34（UTC 10:49–11:04，driver-k36.log），零 ABORT（rc=2 全腿＝语料存在 Fail 常态退出码）。
- **san：BYTE-IDENTICAL**（cmp 0 vs k35）。
- **dp：F/T 多重集 IDENTICAL**（F 221/T 1 恒；log 字节差＝输出序噪声，行为零变化）。
- **wc：GONE 1/NEW 0、T 20==20**——GONE 恰 fragment-parsing 主文档键；P 17694→16496＝−1198 算术闭合＝+1 翻面 −1199（gethtml.html 3435→2236 截断带，巨页落位族，F/T 零变化）。
- **dom：F/T 多重集 IDENTICAL**（F 364/T 12 恒）；P −424＝Range-mutations-dataChange −217/replaceData −207（master.md 2026-11-23 碰撞记录同页族截断抖动）。
- **累计 84,208**（84,207＋1）＝仅 F→GONE 翻面计数、P 漂移带不计（k30/k33/k34 先例口径）。

## 插桩定谳法（防重走）

- 双键 was 探针（`wasSel`/`wasPlain` 分别读两键）优于单键探针：单键探针（k35 diag 首演）对 plain 元素读 `_elKey(null,null)=null` 恒 false，误导判读（曾据此误判 cc(C)#1 相位）。
- runner 二进制内嵌 shim js（include_str 链）：`make browser-build` **不**重建 `zero-wpt-runner`——直跑 `./target/release/zero-wpt-runner` 吃陈旧二进制＝假判读（本轮 diag-base/baseline 两演全被陈旧二进制污染，k36 diag 三演改 `cargo build --release --bin zero-wpt-runner` 后定谳）。判 shim 行为必显式重建 runner。
- k35「baseline 复现」结论曾依陈旧二进制得出——基线对照一律以电池 make 链产物为准（k34 电池 log 本身就是权威基线）。

## 残池移交

- fragment-parsing 页全清（8 键全绿）；connected/disconnected 双页全绿。
- 刀㉚ 承继不变：Node replaceChild mutation events 1F。
- TT residual 池与 master.md 卡点清单不变（刀㊱ 未触）。
