# s10k52 · 刀㊺b 收编：TT 预导航检查（javascript: location 导航 sink 门）

日期：2026-11-27。任务源：控制面 下一步 ②「part02 navigate 5F 写者域（45b WIP
`32985cd4` 在树）待其轮自结收编」——写者域未提交 80 行 diff 本轮验证后落库。

## 结论（定谳）

刀㊺b 实现 TT spec §4.2.1.1（`require-trusted-types-for`）对 `window.location` 赋
javascript: URL 的 **Pre-Navigation check**（此前 HEAD 无此门＝「Location href」
sink 违规发射点缺失，s10k51 轮 grep 扑空之谜即此）。套内
`navigate-to-javascript-url` 族 enforce/RO 五页（001/002/003/004/008）**F→P 确定性
翻面**：TT 电池 2249P/18F/19T → **2254P/13F/19T，其余键零漂移**。

## 变更（crates/engine/src/js_dom_shim/part02.js，+80 行）

- `_zwJsNavUrlParses45(converted)`：spec §4.2.1.1 steps 5-6 的 WHATWG URL 再解析
  （non-special scheme `//` authority 终结符 `/?#` 截断、userinfo 剥至末 `@`、
  `[` IPv6 字面量须闭合、`]` 后仅 `:`+数字 port；port 非数字＝解析失败 → 004
  make-invalid 臂靶形）。
- `_navLocationJavascript` 门：enforced/RO 页把 body 过
  `__zwTtSinkValue('TrustedScript', body, 'Location href')`；
  - TypeError 带 `This document requires` 标记＝pipe 已派发违规（routed，不重复派发）；
  - 其余异常→补派 `__zwTtViolation`，enforce 臂阻断（`return true`，不执行不外抛）、
    RO 臂 `_ljRun = _ljBody` 继续；
  - 转换成功但再解析失败→违规＋阻断；
  - deferred eval 改用 `_ljRun`（转换值）。
- 非 TT 页门不进（`__zwTtEnforced() || __zwTtRoActive()` 恒 false）＝legacy 字节同路径。

## 语料家族普查（勘误）

`navigate-to-javascript-url` 实为 **001-005 + 006.sub + 007-crash + 008(+.headers) +
csp-headers** 共 10 文件：

- 001-004：enforce 臂（inline meta CSP；002＝default policy 改写后执行、003＝policy
  抛异常、004＝policy 返 invalid URL——三门靶形全翻 P）。
- **007-crash.html**：default policy 摘框（`frameElement.remove()`）回归页，testless＝
  零行输出形。勘误：s10k51 轮「007 不存在」系 filter 字样巧合——文件名带 `-crash`
  后缀，套内三臂与本臂 `007` 记录全为 `script-enforcement-007.html`（异页），
  `007-crash` 本身零记录＝与本臂一致（无崩溃，无输出）。
- 008：**Report-Only 经 `.html.headers` sidecar**（`CSP-Report-Only:
  require-trusted-types-for 'script'` ＋ enforce `connect-src 'none'`）＝RO 臂靶形翻 P。
- 005/006.sub/csp-headers：**T 恒**（结构性 Timeout 族，s10k51 三臂归因在案，本刀不动，
  保持挂账 TT 席）。

## 验证

- **solo 单页**（bin `3f37ab178503`＝HEAD 98188dfbd＋本刀）：001-004/008 各 P=1 F=0；
  005/006 T=1；007-crash 零行（testless 形）。
- **套内电池臂 ver-tt5**（08:39:41→08:48:15Z，rc=2＝套件完成形，run-bin md5 断言
  首尾同值，空闲机）：**2254P/13F/19T**。
- **vs 双 clean 基线**（ver-tt3/ver-tt4＝2249P/18F/19T，s10k51 续在案）：
  - Fail 集逐键 diff＝**恰删 nav 001/002/003/004/008 五键**，无他变；
  - Timeout 集恒等（19=19，零 diff）；
  - Pass 页级多重集（uniq -c 对账）＝**恰 +5 行（五 nav 页各 1 子测），其余页计数零变**
    ＝无补偿形。
- 漂移判定：套内 F 18→13 恰 nav 五键离场，zz 探针 6＋散布 7 面保持＝**真实池 12→7F**
  （散布 7＝S3 2＋S4 5）。

## 非 TT 页惰性论证

- 门入口条件＝`__zwTtEnforced() || __zwTtRoActive()`：无 TT CSP 页两旗恒 false，
  走 legacy 路径（仅多一次旗检查与 `_ljRun` 同值初始化）。
- dom 语料全量扫描：唯一含 `javascript:` 页＝`dom/events/Event-dispatch-click.html`
  （anchor `href` 属性赋值，非 location 导航路径，本刀不触）＝DC-2 dom 面结构性无涉。
- spot 三页（`object-javascript-url`／`embed-javascript-url`／
  `execution-timing/079.html`）**未接线任何 suite 语料清单**（`imported-testharness.txt`
  等零命中，`testharness-html` 仅 media/forms/focus 子集）＝套内不可跑，接线属扩范围
  不做；惰性由结构论证＋套内非 enforcement 页（no-require 202P／no-require-trusted-types-for
  18P）计数零变覆盖。
- 近似挂账（不扩）：anchor 导航路径（part01 :9509+ 独立 inline body）不在本门域＝
  accepted 近似形态（corpus 无钉，不扩）。

## 工件

- /tmp/s10k52/：ver-tt5.sh/.log（电池臂）、spot-nontt.sh/.log（未接线确认）、
  f3/f5/t3/t5/p3/p5.txt（逐键 diff 对账）、part02-now.patch（发射前 WIP 备份，
  md5 `32985cd4` 与 .acceptance/dps/s10k45b-knife45b-part02.patch 恒等）。
- 账本：.acceptance/dps/s10k52-knife45b-coord.md。
- 等待窗纪律：兄弟席（ZeroWeb-2 css 流）reftest-upstream 腿链在飞（816% CPU）期间
  不发射；持续 3min 空闲确认后发射；臂脚本内嵌兄弟进程预检（防链间隙假启动）。
