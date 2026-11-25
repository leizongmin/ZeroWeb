# DC-2 切片 10 刀㉘收账——TT 违规 blockedURI 字段＋'nonce-self' 同源外链＋动态内联 script-src-elem 门＋classic 执行体 eval 包装旁路（TT P 2070→2115＝+45＝GONE 37／NEW 0〔语料〕／Timeout 键集恒 20=20）

**日期** 2026-11-26　**判定** PASS（记录腿 k28c 锚 `7afb21275`，发射器 `/tmp/s10k28/launch-k28c.sh` HEAD 全哈希断言 21:25:42Z–21:35:11Z 9min；判读 vs k28pre 基线 `a09bdd83c` 组合树键集，judge `/tmp/s10k28/judge-k28c.sh`）

## 提交与判读绑定

| 提交 | 内容 | 验证 |
|---|---|---|
| 刀体 A `3e4222588`（7 文件：security csp.rs `has_nonce_self`＋context_csp_tests 新钉＋shim part14 blockedURI 字段＋part12 createPolicy block 两面 blockedURI＋part03 动态内联 CSP 门＋webview `__zwCspInlineScriptAllowed` 回调＋part175/176 钉测） | blockedURI 字段（29 键）＋children-change 簇（4 键） | 中间腿 k28：P 2113/GONE 43，唯 TNEW 2 |
| 刀体 B `7afb21275`（1 文件 11 行：part03 动态 classic 执行体 `(0, eval)` → `(0, __zwRealEval)` 回落原 eval） | 修 k28 TNEW 2 根因 | 记录腿 k28c：TNEW 0，008/009 classic 面 2P/页 |

## A 段根因链（ blockedURI＋nonce-self＋动态内联门）

1. **blockedURI 字段**（part14/part12）：spec TT §create-a-trusted-type-violation-record 违规记录 `blockedURI` 面——sink 执法/上报面＝`"trusted-types-sink"`、createPolicy 名单/重名 block＝`"trusted-types-policy"`；此前 shim 恒空串。语料 reporting-for-* 八页 12 键失败形 `expected "trusted-types-sink" but got ""`（刀㉗ 归因更正的缺 sink-拦截簇，实为此字段缺失）＋clipping/cspTests 残键同簇——29 键一并翻面。
2. **'nonce-self' 同源外链**（security csp.rs）：WPT 语料 nonce 约定 token `'nonce-self'` 下无 nonce 同源外链放行、跨源仍拦、无 nonce 内联仍拦（blockedURI "inline"）、真 nonce（'nonce-abc'）政策严格如旧——Chromium 口径经 wpt.fyi Edge master 跑批 report.json 实证（children-change 2/2、script-enforcement-008/009 5/5、nonce-enforce-blocked PASS）。`SourceListSummary.has_nonce_self` 源列表级旗标，`is_self_match` 放行。
3. **动态内联 script-src-elem 门**（part03＋webview）：动态插入内联 script prepare 时点查 CSP inline 面（nonce/hash——hash 取 TT 门变换后执行体，default policy 救援输出参与 inline hash 判定＝script-enforcement-008 判据语义）；阻止 → 置 run-once 记账＋violation 经 shim `setTimeout(0)` task FIFO 派发（与 TT sink/connect-src 同队列保序——csp-violations helper 以 connect-src 为收集边界）。宿主桥返打包 `"0\x1f<directive>\x1f<policy>\x1f<blockedURI>"`。children-change 双页（HTML/SVG）内联臂翻面＝4 键。

## B 段根因链（k28 TNEW 2 → k28c TNEW 0）

k28 记录腿（仅 A）008/009 页中途 Timeout（T 22，TNEW 2；5 基线键转挂起非翻面）。诊断链（`zz-s10k28-hashallow-probe` 探针定谳）：A 段内联门放行的脚本（hash/nonce 均 "1"）在执行体落点被二次拦截——`globalThis.eval` 被 M2-s6 per-script CSP 门禁包装（eval/Function Proxy），part03 动态 classic 臂 `(0, eval).call` 走包装 → `EvalError: call to eval() blocked by CSP`＋`script-src`/`eval` 违规两枚（classic＋DONE 脚本各一）；`script_messages_for` 的 DONE 脚本在 rAF×2 回调内 appendChild，异常未捕获 → helper promise 永悬 → 页 Timeout（classic 臂异常被 fn() try/catch 捕获、DONE 臂无捕获＝悬挂点）。

修法：HTML spec「run a classic script」非 eval 编译——CSP unsafe-eval 门只约束页面显式 `eval()`/`Function()`，script 元素执行体已过 inline 门（nonce/hash）。part03 执行体改 `(0, (typeof globalThis.__zwRealEval === 'function' ? globalThis.__zwRealEval : eval)).call(...)`（包装未装回落原 eval＝无 CSP 页/engine 钉测零路径变化；体内嵌套 `eval()` 运行时仍查 `globalThis.eval` 走包装，页面 eval 政策语义不变；part01 defer 面 s-早刀同型先例）。

## 判读（k28c vs k28pre 基线；基线＝`tt-full-combined.log` 179F/20T/P 2070）

| 项 | 值 | 构成 |
|---|---|---|
| GONE | 37 | 29 blockedURI（setAttribute 9／HTMLScriptElement 4／setTimeout 2／Document-write 2／DOMParser/parseHTMLUnsafe/insertAdjacentHTML/outerHTML/innerHTML 四面/srcdoc/setHTMLUnsafe 两面/createContextualFragment 各 1）＋4 children-change（HTML/SVG 双页）＋4＝008/009 页键 5→3（ReferenceError 双键翻 Pass） |
| NEW | 1 | zz-s10k28-hashallow-probe（gitignored 诊断探针，非语料）＝语料 NEW 0 |
| TNEW / TGONE | 0 / 0 | Timeout 键集恒 20=20（k28 的 TNEW 2＝008/009 挂起，B 修复清零） |
| P | 2070→2116 raw（探针 1P）＝**2115 语料 +45** | ΔP +45 > ΔF −37 差 8＝先前失败短路的子测现跑到 Pass（无新失败，NEW 语料 0） |
| F | 179→143 raw（探针 1F）＝**142 语料 −37** | RC=2＝runner 有 F 退出码语义非门信号 |

008/009 现态：各 2P（meta 断言＋**classic 面**＝本刀语义主目标）＋3F（importmap/module/two-importmaps——动态 importmap 型处理与内联 module 求值序（setTimeout 近似输给 rAF×2 DONE）独立缺口，后续刀）。

## 钉测与门禁

- part176 两钉 6 面断言 blockedURI 定值（'trusted-types-sink'/'trusted-types-policy'）；part175 ro 钉 `blocked !== ''` → `!== 'trusted-types-sink'`（WebSocket connect-src 面键不动）。
- 探针链（全 gitignored）：nonce-ext-probe（外链放行/内联拦）2/2；inline-gate-probe（阻断面：violation 字段四元组）2/2；hashallow-probe（放行面全序 `CREATE_SCRIPT|sink|CLASSIC|DONE` 零违规零页错）——B 修复判据。
- engine lib 全量 4323P/3F（3F＝对端预存红名单 dialog_focus_s330/expando_r3046/p99s_inline_custom_property 恒定）；A 提交过 fmt/clippy（-D warnings）绿；B 纯 JS（include_str!）无 Rust diff。

## 累计与残池

TT P 2070→2115＝**累计 83,936→83,981（距 ≥99,947 差 15,966）**。TT residual 142F＋Timeout 20 页：svg-script-set-href 7／block-string-assignment insertAdjacentHTML 7＋outerHTML 6／createHTMLDocument 5／function-constructor 4／script-enforcement-004 4／policy-without-return-value 4／eval-with-permissive-csp 4＋eval-csp-tt-no-default 4／eval-reporting-no-unsafe-eval 3／script-enforcement-006/007/008/009 各 3（008/009＝importmap/module 新残面）／block-text-node-insertion 3／DOMParser-parseFromString 3／其余散布（eval 2、SharedWorker 2、ServiceWorkerContainer 2 等）。判读工件 `/tmp/s10k28/`（k28c-* keys＋judge-k28c.sh＋三探针日志）。
