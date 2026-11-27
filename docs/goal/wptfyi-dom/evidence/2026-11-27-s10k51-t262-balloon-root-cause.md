# s10k51 · test262-builtins 6 GiB 气球根因定谳与修复（插队督办⓪）

日期：2026-11-27。任务源：master.md ⓪（html 流 DC-3 全量快照 14 连 CAPKILL）。

## 结论（定谳）

**气球＝spec 一致的无限循环，不是引擎 bug。** 触发链：

1. test262 语料案例覆写 `RegExp.prototype.exec`（首杀案 `Symbol.split/last-index-exceeds-str-size.js`，override 返恒 `{length:0,index:0}`）。
2. shim 脚本运行胶水读 `document.currentScript` → `Proxy.getElementsByTagName` → `_zwFilterByTagNameNS` → `_zwAsciiLower`（part05，`/[A-Z]/g` 全局 replace）→ V8 `@@replace` 全局循环逐轮咨询被覆写 exec。
3. exec 恒返非 null 真值对象（不推进 lastIndex）→ spec 循环永不终止 → GB/s 分配债 → 6 GiB。
4. worker 是 runner 内线程 → 气球在 runner 进程 → test-guard CAPKILL 整个 runner → 全批死 → html DC-3 快照 14 轮冻结。击杀位 13.5k 案 ≈ RegExp Symbol.split 族字节序位置，14 轮同位自洽。

**上游同炸实证**：standalone V8 15.0.245.2（本仓 vendored 版本，workspace 二进制同源）最小复现 `'AbC'.replace(/[A-Z]/g, fn)`＋同 override → `Fatal JavaScript out of memory: Ineffective mark-compacts` 1.39 GB。Chromium 不炸＝Blink 内部全 C++、无同 realm JS shim——本仓 shim 层是特有暴露面。

## 定位过程（探针漏斗，语料内嵌 `/*---` marker 探针，runner 文本输出读地真值）

- solo 复跑单案 → 气球（4.21 GB rc=124）＝案例内禀；standalone V8 跑案例本体 LEN=2 正确＝非 V8 核心/flags。
- h（override 返 null）净 vs e/f/i（返对象）炸：分界线＝返 null/对象。
- j/k/l/m：receiver、match 形状无关；n：计数封顶捕到 **500+ 次调用**＝循环实证。
- p/q：第一次调用 stack 空＋单参 S='contains'（另处胶水单参调用＝spec `RegExpExec(R,S)` 形）。
- r：干净记录调用方 stack＝`RegExp.exec ← [Symbol.split]`，单参，lastIndex 0→1。
- **u（决定性）**：同 override（返对象不写 lastIndex）split **5 次调用即终止**＝V8 split 本体无罪。
- v：真案例语义（override 内写 lastIndex=100）solo 气球＝split 路径无关，循环在别处。
- **x（真凶栈）**：call 5 与 35 同栈 `RegExp.exec ← [Symbol.replace] ← String.replace ← _zwAsciiLower ← _zwFilterByTagNameNS ← Proxy.getElementsByTagName ← get currentScript`。

## V8 源码核对（tag 15.0.245.2）

- Torque 分派（regexp-split.tq）：`Cast<FastJSRegExp>` 快路径（CSA `RegExpPrototypeSplitBody`，`RegExpExecInternal` 内部 exec 不看覆写）否则 `runtime::RegExpSplit`（C++，spec 逐字）。
- runtime-regexp.cc 慢路径循环每迭代必推进 `string_index`（null→Advance；end==prev→Advance；否则 string_index=prev=end）＝至多 length+2 迭代，与 500+ 观测不容 → V8 双路径均排除为循环宿主 → 循环在 shim 的全局 replace。
- `@@replace` 全局循环对「恒返空匹配不返 null」的 exec 无终止保障＝spec 形态；test262 未测此面（不可测 hazard）。

## 修复（本刀）

`_zwAsciiLower`（part05，getElementsByTagName/标签过滤热路径）与 `_zwTtAsciiLower`（part12 TT sink 映射）改 charCode 循环（仅 A-Z +32 折叠，非 ASCII 不动，语义与原 regex replace 恒等），零 RegExp 面——语料覆写 exec 后页面一切 RegExp 方法（test/split/replace/match）全部毒化，shim 内部机器必须绕开。修复面二进制 md5 `c1388cba460f`（含 k50，HEAD ef07db54b 基 + 本刀）。

## 验证

- solo 真案例：修复前 4.21 GB CAPKILL；修复后 **1s / 154 MB / rc=1**（runner 存活，案例因剩余毒化面记 Fail）。
- heap_limits 收容实验（future 备案）：显式 heap_limits(0, 256MB) 下气球仍 `FatalProcessOutOfMemory`（不可捕获）；`add_near_heap_limit_callback` + `IsolateHandle::terminate_execution` 实证可打断（243MB 触发、脚本收 termination 异常、进程存活），但 terminated isolate `Dispose()` fatal（巨堆 CHECK）需 forget/专门处置——留作下一层收容设计输入。
- **FULL built-ins 面（对端击杀面全集）验证臂完跑（07:11:00Z，95min，bin `c1388cba460f` 断言收口）＝DC-3 解冻验收成立**：23,458 案 **23,065P/393F**，rc=1 干净完跑，**max RSS hwm 648MB**（对端击杀形 13.5k 案 4.1GB→6GiB CAPKILL）＝气球消除 ~10×，fatal/OOM 标记 0。首杀案 last-index-exceeds-str-size＝9ms 进程存活，按预测记毒化 TypeError Fail（全 face 毒化 TypeError 恰 5 案＝首杀案＋RegExpStringIteratorPrototype custom-exec 4 案＝系统性暴露实践中罕见）；RegExp Symbol.split 族 42P/2F 全族复活。393F 构成＝RegExp 96/Iterator 74/Array 57/Promise 40/Object 17/…＝案级有界（同 realm shim 局限既有构成，首份全脸基线，无对端完跑前值可比）。
- **dom 腿热路径复验（07:23Z 完跑，bin `03e2ace209db`＝HEAD 4974ee5df 面，含 rebase 并入的 css 流码刀）＝本刀零回归**：57,420P/362F/0 Timeout，完跑形与 dc1 孤立基线（`evidence/dc1-dom.log`）逐字节同款（尾页 Pass→guard rc=1 完跑退形）；F 键集 62 vs dc1 65（61 共享＋4 被刀修复），热路径靶向全绿（Document/Element-getElementsByTagName 18+18P/0F、GTNS 14+14P/0F、case.html 30P、collections 0F；GT 页 3F dc1 已在＝既有面）。唯一新 F 键 `MutationObserver-attributes`（「records expected 1 got 2」）**二分实证排除本轮刀**＝撤 part05/part12 刀字节重建（bin `936f026e1c12`）后单页依旧 1F——回归属 dc1→HEAD 窗内其他码刀（DC-2 k44-k50 或 css 流 f634df5f2 域），下轮独立 bisect 挂账；复验后已重建回 HEAD 面（`03e2ace209db` 恒等＝确定性重建再证）。

## 剩余暴露（本轮不修，已定性为案级失败非进程死）

exec 被覆写时 shim 全域 regex 使用全部毒化：classList proxy get/has trap 的 `/^\d+$/.test(prop)` 恒真 → string prop 全被当索引 → 方法消失 → runner 完成度探针报 `classList.contains is not a function`（bounded TypeError，案例记 Fail）；上游 testharness.js get_stack 的 script_url 全局 replace（:4816）在 stack 捕获路径同型未爆。系统性方案候选（下轮）：t262 页面剥离 shim（语料纯 JS 不需要 DOM shim，上游形态同）/ case 源独立 realm / shim 索引键 helper 全量化。判读基线：本刀后案例 Fail 构成从「超时/击杀」变为「毒化 TypeError」——计 F 面变化属预期非回归。

## 工件

- /tmp/s10k51/：arm2-BIG.log（兄弟轮 FULL 决定性 13,573 案 4,294,212KB 击杀记录）、probe-{a..z3}.*（探针输出）、ver-case.sh/.log（solo 复验）、ver-full.sh/.log（FULL 验证臂）、v8probe（standalone V8 复现 crate，heap_limits/near-callback 两实验）。
- 语料探针文件 zz-probe-*.js ×30 已全部删除（gitignored，无树残留）。
- 账本：.acceptance/dps/s10k51-t262-balloon-hunt.md（分族臂数 + 本回执）。
