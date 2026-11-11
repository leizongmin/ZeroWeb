# M2 切片 409 — script-created link stylesheet 不阻塞 script（s163 延迟仿真归一 + 阻塞计数门）

日期：2026-11-12（承 s408 下一步：links 族散单续勘）
commit：见 git log s409 提交

## referent 与勘根

driving 双键（s408 收口时在册 F，document-metadata 仅余 2F）：

- `interactions-of-styling-and-scripting/script-created-link-stylesheet-does-not-block-script.html :: Script-created link stylesheet is not script-blocking`
- `interactions-of-styling-and-scripting/dynamic-render-blocking-link-stylesheet-does-not-block-script.html :: Script-created render-blocking link stylesheet is not script-blocking`

两案同构断言面：script 内 `createElement('link') + rel=stylesheet + href='stylesheet.py?delay=1'` 插入 head，后继 script `assert_false(styleExists('h1 { color: purple; }'), 'stylesheet should still be pending')`。style 双胞胎（`script-created-style-element` / `dynamic-render-blocking-style-element`）已 Pass——@import 内层 URL 解析时已绝对化、延迟臂命中，反证缺陷在 link 发题路径。

双层勘根（探针相位定位，探针勘根毕即删）：

1. **发题侧相对 URL miss 延迟分支**：script-created link 插入臂 `_zwSsVisitCss(_ssHref, …)` 发题传**原始 href**（相对形 `stylesheet.py?delay=1`，part03 MaybeStart 语义，`_ssAbs` 只作 memo/mixed 门）；s163 `_zwFetchSimDelay` 的 interactions 前缀分支（`indexOf('interactions-of-styling-and-scripting/stylesheet.py')`）按绝对形态设计 → 相对形 miss → delay 仿真返回 0 → `setTimeout(_issueFetch, 0)` 立即发题，runner 端点零 sleep 即时交付，取回在脚本间微任务排空点完成 → `styleExists` true → `assert_false` 恒红。
2. **阻塞计数门缺 parser-inserted 限定**：s349 `_ssDoVisit` 的 `_zwSsBlockPending` 计数对**全部臂**生效（media 匹配即计数）。script-created link 修复①后延迟发题 1000ms，但动态插入臂若仍计数，s349 等待泵（`__zwScriptBlockingSheetsPending` 探测 + `__zw_fire_due_timers` 泵）会把延迟取回**等完再放行后继脚本**——延迟仿真被泵消解，断言照样红。spec the-link-element：script-created stylesheet 不入 script-blocking 集（两案标题即断言面）。两改动缺一不可，共同构成 F→P。

## 施工

- **part01 `_zwFetchSimDelay` 发题前 base 归一**（s163 分支判定前）：非 `https?:` 前缀 URL 经 `__zw_parse_url(_us, _zwDocBaseURL())` 归一为绝对形再进前缀匹配；解析失败/无 base 回落原始形（旧行为兜底）。trickle/delay.py 判定均为 query/子串形不受归一影响。既修本切片 driving 面，也为一切「动态臂发题相对 URL」面补齐延迟仿真通道。
- **part03 `_ssDoVisit` 计数门控 parser-inserted**：`_ssMediaBlocks` 计数/归零加 `noOverlay === true` 限定（= s152 静态交错扫描臂，即 parser-inserted 面）。动态插入/attr 变更臂（noOverlay falsy）不计数——s349 泵不再等 script-created 取回，media 不匹配臂既有语义不变（`_ssMediaBlocks` false 本就不计数）。
- **常驻钉**：`js_dom_bridge_tests/part109_s409_script_created_link_stylesheet.rs` 双钉——①`script_created_link_not_blocking_faces_s409`：双形 link 插入后 IIFE 内即时读 `__zwScriptBlockingSheetsPending`（mid-window 计数门断言面，旧动态臂=true）+ execute 末尾检查点后 fetch log 齐写且相对形发题 URL 已归一为绝对 interactions 路径（wire 面钉归一生效）；②`fetch_sim_delay_relative_normalization_faces_s409`：`_zwFetchSimDelay` 纯函数面六案（相对 delay 形归一命中→1000 / 绝对形本态→1000 / 归一后异目录不误延迟→0 / the-link-element 同名端点不误延迟→0 / delay.py ms 形不变→50 / 无延迟资源→0）。

## 过程陷阱

1. **钉①挂起 = assert_eq! 宏格式化参数二次 lock 自死锁**（首版断言 `assert_eq!(fetch_log.lock().unwrap().len(), 0, "…{:?}", fetch_log.lock().unwrap())`）：断言失败时 panic 分支才求值格式化参数，判别式第一个 guard 临时横跨整个宏展开语句存活 → 第二次 lock 与自己死锁（std Mutex 不可重入）。yama `ptrace_scope=1` 下 gdb 对后台 setsid 进程不可 attach，改用**相位探针**（AtomicUsize 相位 + 看门狗线程超时打印 exit(9)）定位——探针证 shim 全链无挂、双 link 0.12s 全过，挂点恰在断言宏。已补录 learning（match 判别式 guard 死锁同根因宏变体）。
2. **桥测微任务窗口心智模型错误**：首版断言假设「桥测沙箱 setTimeout 落 _defer → 同 turn 不排空 → 同步段零发题」——实际 V8 每次脚本求值末尾自动 perform microtask checkpoint，`_defer` 发题链**同一 execute 内**泵完（fetch log=2 非 0）。「同步段零发题」在桥测不可观察；runner timer wheel 的 1000ms 真窗由钉②纯函数延迟值 + driving solo 实测承载。断言面改为可观察事实：mid-window 计数门态 + 检查点后发题齐写 + URL 归一。
3. **前台 600s 上限撞并行编译负载**：并行流（ZeroWeb-2）r5506 探针编译突发 load 161，前台 test-guard 编译段 10 分钟超时（exit 143）——转 setsid nohup + done 旗标后台轮询（既有惯例），负载回落后正常。
4. **0 测试假绿**：桥测过滤器写文件名 `js_dom_bridge_tests::part109`（不在测试路径）静默零匹配 exit 0——过滤值应为测试名子串 `s409`（上轮 pinrun.sh 同款）。跑后必须核对 `running N tests` 非零。

## 验证（/tmp/s409/ 全量日志）

- 双钉全绿：`cargo test -p zero-engine --lib s409 -- --test-threads 2` 2 passed 0 failed（0.15s）。
- driving solo（runner 重编含 WIP shim，token `_b409` strings 命中 3 钉证新鲜）：interactions 目录全 9 键 **全 Pass**（双 driving F→P，双胞胎 style 保持）。
- 单元：engine lib **3896P/10F**（s408 记录 3886P/10F；+10P = 本切片双钉 2 + 并行流合入 8〔p39 epoch 章改族 7 = d3f5eb3a5 + engine NaN censor 钉 1 = 6fe8fad31〕；失败清单与 s408 **逐名相同零位移**）；webview lib **849P/2F**（csp_gate/image_decoder 在册对，与 s408 同形）。
- 全树 sweep ×2（同二进制，s292 协议）：双跑全等 **8484 keys 8245P/213F/26T**。join-2 对 s408 干净基线（pairs-2）＝**恰 2 STATE-DIFF 全 driving Fail→Pass、new=0**；join-cross（本轮 run1×run2）＝**stable 8484 state-diff=0 new=0 双跑全等**；join-1 对 s408 run1 另见 select-home-end F→T（在册 s292 watchdog flake 族，本轮双跑一致 T 形单列）+ GONE 7 键 = s408 遗留探针已删（run1 单列，s408 账本在案）。
- fmt：本切片文件 `rustfmt --check` 干净（全仓 `cargo fmt --all -- --check` 亦净——callbacks.rs 既有脏已由并行流 7c24ec52c 清偿）。clippy：`-p zero-engine --all-targets -- -D warnings` 干净。
- smoke：`make product-smoke` + `make product-smoke-legacy` 双 PASS（struct-check 0 issues）。
- `make test` 全量：仅 engine lib 一腿 FAILED（在册 10F 逐名同形），其余 153 腿全绿（含 T58 fmt 门 + QuickJS clippy 门）。

## 口径

- 本流域在册红：**214F+24T → 212F+24T**（−2F = driving 双键 F→P 双跑一致）。
  （sweep 双跑 8484 keys 8245P/213F/26T；select-home-end 本轮双跑 T 形，在册 flake 族单列口径不变）
- engine lib 单测在册红 10F 承 s408 口径零位移（含 r307 HEAD 单列）。

## 残差与边界

- runner timer wheel 真时序窗（1000ms）桥测不可观察——纯函数面（钉②）+ driving solo 承载；若后续加第二 delay 端点族需按本切片同款「归一先行」核对前缀分支。
- `_zwFetchSimDelay` 归一读 `__zw_parse_url`/`_zwDocBaseURL` 全局，host 桥缺席时回落原始形（裸 sandbox/reftest 车道零行为差）。
- 相对形发题 URL 归一发生在 fetch wrapper，`_zwSsVisitCss` 传入的 `_ssHref` 原始形不变——memo 键（`_zwSsUrlTextMemo`）仍按 `_ssAbs`，无键面漂移。
- s349 等待泵对 script-created stylesheet 的「不等」语义是 spec 正确面（不入 script-blocking 集）；若未来语料出现「script-created sheet 须阻塞」反例（无此规范依据），届时重审门限定。

## 下一步

document-metadata 目录已清零（9 键全 Pass）。散单续勘按 s408 池体检视：the-input-element 散单 3F / tentative/ constructed-stylesheets 机制（CSSStyleSheet constructor + adoption + CSP，9T 大机制，勘根后按簇决策）/ 挂账池承 s404/s405（forward map 空返勘根/渲染流 5 面/form-submission-target 6T/sizes-auto 半边/ilf+modulepreload 观察池）；select/combobox（簇 D）与 A2+D 簇立项征询仍待用户决策，不动。
