# DC-2 切片 10 刀㉖收账——TT Report-Only 监测面＋CSP 违规 task 化派发＋动态 script 取归 script-src 门（TT F 127→202〔净 Pass 1955→2044＝+89〕＝k26 锚簇 9F+3F+1F＋45 页 Timeout 行全清／helper 解禁页 88 键新显形）

**日期** 2026-11-26　**判定** PASS（刀域目标簇全清＝solo/电池双证；GONE 58／NEW 92〔88 真实显形＋4 探针噪声 gitignored〕；WC/DP/SAN/DOM 四腿 SET-IDENTICAL〔comm 逐键 0/0〕）

## 提交与判读绑定

| 提交 | 内容 | 判读二进制 | 电池 |
|---|---|---|---|
| s10k27 刀㉖ 单 commit `678e17076`（谱系 abef432ca＝提交锚→rebase 携入远端五 docs-only 后终态；rebase 前后树 diff 空）（8 文件：testharness.rs sidecar RO 注入／part04 innerHTML 回放 RO 臂／part06 违规派发字段透传＋WS connect-src／part12 `__zwRawCbs` 寄存／part14 RO 管道主体／part01 fetch wire arg 12／part03 s110 臂车道旗标／webview.rs `__zw_fetch` 门 script 分流） | RO monitoring + 违规 task 化 + script 车道 CSP 分流 | `342a0ecf019f831c57bc7a844d34b4d6` | v12 |

v12 判读时 HEAD＝2620be612（发射器 EXPECT_HEAD 断言即此值；commit 于电池后，源字节与判读二进制一致——fmt 仅空白，JS 五文件电池前后无变更）。

发射器 `.acceptance/dps/s10k27-battery-v12.sh`（EXPECT_MD5+EXPECT_HEAD 双断言、flock 单例、五腿 test-guard 包裹、setsid detached）；19:33:44–19:57:28Z（24min，WC 5m27s/DP 12s/SAN 17s/TT 9m40s/DOM 9m07s；TT 腿墙钟缩短＝页 Timeout 行消解的直接效应）。

## 根因链（三段）

1. **RO monitoring 管道**（轮 A/part14＋testharness.rs；轮 B/part12＋part04 互补）：sidecar `.headers` 的 `Content-Security-Policy-Report-Only` 解析注入 meta（CSP3 §cspro-header——RO 指令不支持页面 meta 形态，运行器载荷通道注入）；`_zwTtParseCsp` 状态扩 `reportOnly/roPolicy/enforcePolicy`＋`_zwRoActive` memo（dirty/flush 同 enforcement 形）；`_zwTtRoSink` 监测臂＝TT 实例直通／default policy raw 回调（`__zwRawCbs` 寄存——part12 包装方法 undefined→TypeError/null→TT('') 先于管道臂，RO 面 null/undefined 语义需 raw 用户回调）／无 default→原串入 sink＋`_zwTtViolation`（disposition 'report'）。enforced 臂 null→TypeError 收口（返 '' 曾错置空串）。
2. **违规 task 化派发**：CSP spec §report-violation 入 task 队列（setTimeout(0) 载体）——语料 Count/listener 注册在 promise_test 微任务体（script job 后），同步派发先于注册即全数丢失；part06 WS connect-src 违规同 FIFO。WS CSP 检查前移至宿主桥存在性判定之前（trusted-types 套件页无 ws_fixture 桥——后置时该面永不可达）。
3. **动态 script 取归错门（残面根因，probe3f blockedURI 铁证）**：part03 s110/s96 经典臂外链取回走通用 `fetch()` wrapper→`__zw_fetch` 统一桥→webview.rs `check_connect`——connect-src 'none' 页（trusted-types-report-only.html headers 双政策形）动态 script.src 后设取回被错门阻止→假阳性 connect-src 违规（blockedURI＝`https://wpt.test/trusted-types/support/namespaces.js#abc`＝test1 赋值串）→csp-violations helper「首个 connect-src 即 resolve」契约在自身 sink 调用引发的违规上提前 resolve（violations=[]）→count 0。CSP3 fetch 指令映射元素发起 script 应归 script-src 族。修复：part03 s110 臂 `__zwScriptFetchLane` 旗标（s383 slot 同构）→part01 impl 头一次性捕获（closure 供 trickle/delay 延迟臂）→wire arg 12→webview.rs 门按 'script' 分流 `check_script`（script-src 族判定，类别替换非跳过；nonce 缺席近似挂 FIXME，https://www.w3.org/TR/CSP3/#directive-script-src-elem ）。

隔离证据链：probe2 形隔离四形全绿（detached/in-doc × fragment 形）排除 sink 路由面；probe3 复刻（policy "two"＋双政策 headers＋helper 流程）复现；probe3d 双站诊断（script eval 期收集器）定位时序；probe3f blockedURI 定谳；修复后 probe3f diag 实证假阳性违规消解、序列＝TT 违规先于 WS connect-src。

## v12 判读（锚＝k26 v11 终态 TT F 127（P 1955）/ WC F 1116 / DP F 222 / SAN F 50 / DOM F 364；F+T 键集 TT 192 / WC 1136 / DP 223 / SAN 51 / DOM 376）

| 腿 | Fail | Pass | GONE/NEW | Timeout 键集 |
|---|---|---|---|---|
| TT | 127→202 | **1955→2044（+89）** | **GONE 58＝刀域 Fail 13（dpro 9＋dp 3＋tt-RO 1）＋Timeout 行 45（刀域 4 页页行＋helper 解禁 41 页页行）/ NEW 92＝helper 解禁页子测显形 88＋探针 4〔gitignored〕** | 65→20（GONE 45 页行全消解，NEW 0） |
| WC | 1116→1116 | 17852 | **0/0** | 恒（20=20） |
| DP | 222→222 | 1313 | **0/0** | 恒（1=1） |
| SAN | 50→50 | 706 | **0/0** | 恒（1=1） |
| DOM | 364→364 | 57304 | **0/0** | 恒（12=12） |

TT GONE Fail 13 全部为刀域目标；NEW 88 真实键逐页归因：reporting-clipping-of-sample 12＋tentative 8／Element-setAttribute 9／svg-script-set-href 8／Window-function-constructor 4／HTMLScriptElement 4／eval-reporting 7／其余 1-2 键 ×22 页——全部为 v11 整页 Timeout（helper 挂死零行）页在 helper 解禁后的子测级未实现面显形（residual 池挂账，非本刀回归；comparing v11 该等页零行＝不可见）。

**净 +89＝累计 83,910**（距 ≥99,947 差 16,037）。

## Solo 判读（链 A3，bin 342a0ecf019f831c57bc7a844d34b4d6，19:32:11Z）

七语料页＋探针全绿零 Timeout：trusted-types-report-only 5P/0F（残面清）／default-policy-report-only 19P/0F/0T（Count 解挂）／empty-default-policy-report-only 4P/0F/0T／require-trusted-types-for-report-only 4P/0F/0T／no-require-trusted-types-for-report-only 9P/0F（零回归）／default-policy 19P/0F/0T／empty-default-policy 4P/0F/0T／probe 2P／probe2 1P／probe3f 1P/1F（Fail＝故意诊断 dump 行）。

## 门禁

fmt `cargo fmt --all -- --check` 绿；`make guarded-clippy` workspace 0 warning（轮 B 新增 part175_s10k27_ro_sink_pins.rs 5 处 E0308 execute(format!→&format!) 机械修复后 PASS）；make test workspace 腿＝预存 r5309 两钉（ch 单位解析，layout/style-system 域，k26 已单文件隔离实证非本刀域）。

## 双执行体协调

轮 A（管道主体＋sidecar＋WS＋script 车道分流）×轮 B（part04 回放臂＋`__zwRawCbs`＋task 队列化＋part175 钉测）——hunk 域不相交，COORD 账本 `.acceptance/dps/s10k27-coord.md` 全程接力（混合二进制防线＝共享 make 锁 `/tmp/s10k27-make.lock`；链 A1→A2→A3 三代判读）。
