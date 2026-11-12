# M2 切片 411 — maxlength 仅适用六文本态（type=number 写入面不限长）——maxlength-number 1T 解挂

日期：2026-11-12（承 s410 下一步：maxlength-number 1T 勘根）
commit：见 git log s411 提交

## referent 与勘根

driving 键：`the-input-element/maxlength-number.html :: maxlength doesn't apply to input type=number`（file-level **Timeout**，s409 起在册 1T，the-input-element 目录最后一红）。

案面：`<input type=number maxlength=1>`，`async_test` 内 `send_keys(elem, "1234")` → `.then` 断言 `value === "1234"`（maxlength 不适用于 number 态）——**无 .catch**。

三层勘根（runner 相位探针 + 归纳对照，探针毕即删）：

1. **对照归纳锁定 maxlength 因子**：`number-invalid-scientific-key-events.html`（同目录 Pass）对 `<input type=number>`（无 maxlength）`send_keys("1e+1")` 全绿——number 键入通道（s209 编辑器双槽 + 文法否决 + R3901 净化绕行）健康；唯一差异 = maxlength 属性。
2. **相位探针定位挂点**（testharness.rs 临时 eprintln，勘根毕即删）：char '1' 全序完成（dispatch Ok）；char '2' 起 `dispatch_action → Some("action was not applicable: Some(MaxLengthReached)")` → `return Some(error)` 后 runner 再无输出——链路卡死非报错失败。
3. **挂死机制链**：webview InsertText 路径 `TextActionState.max_length` 从 maxlength 属性**无条件**读取（不问 type）→ plan_text_insert 截断语义把 '2' 截成空 → `Err(MaxLengthReached)` → `dispatch_action` 不豁免（仅 DisabledTarget）→ `resolve_testdriver_command` 以错误 reject send_keys promise → 案面 `.then` 无 `.catch` → **unhandled rejection**；runner 无 unhandled-rejection→Fail 换算（s410 date-readonly 的 Fail 是 `await` **handled** rejection 转 test failure，本案不同）→ async_test 永不 done → 文件预算耗尽 → **Timeout**。corpus 探针（Promise.race 强制 1.5s dump）同样全灭 = 页面 promise 链被 reject 后 harness 无更多可完成测试，runner 只能按预算记文件级 Timeout。

spec 锚：maxlength 属性仅适用于 text/search/url/tel/email/password 六态 + textarea；其余态（number/date/checkbox/…）属性不适用即无截断语义。type 缺省（missing value default）与非法值（invalid value default）均落 Text 态 → 限制仍生效。https://html.spec.whatwg.org/multipage/input.html#attr-input-maxlength

## 施工

- **webview plan 侧 type 门**（`user_actions.rs` TextActionState 构建）：新增 `maxlength_applies_to_input_type` 判定——非 INPUT tag 直接适用（textarea 面）；INPUT 按已识别**非文本关键字黑名单**（button/checkbox/color/date/datetime-local/file/hidden/image/month/number/radio/range/reset/submit/time/week 16 态）判不适用，空串（missing default）与非法值（invalid default = Text）判适用——与 shim `_zwNormalizeInputType` R3901 白名单语义逐项对齐（初稿白名单六态+空串形态会把 `type=foo` 误判不适用，终稿改黑名单补齐 invalid default 面）→ 不适用态 `max_length: None`。
- **shim 原子脚本 live 守卫同口径门**（`part01.js __zw_text_input` maxlength 读取块）：`_zwNormalizeInputType(el.getAttribute('type'))` ∈ 六态才读 maxlength（TEXTAREA getAttribute('type')=null → normalize 'text' 适用；非法值 normalize 落 'text' 适用——与既有 normalize 语义零漂移）。plan 侧仅 noop 判定、真写入在原子脚本——两层缺一不可（双写同口径）。
- **常驻钉两件**：桥测钉 part110（`js_dom_bridge_tests/part110_s411_input_maxlength_number.rs`，三腿：number 不限长 '12' 全保留〔驱动案镜像，旧版第二键起截断〕/ text 限长既有绿面不回退〔'1'〕/ 非法 type=foo invalid default 限长生效〔'1'〕）+ webview plan 钉（`tests/user_actions.rs maxlength_does_not_apply_outside_text_like_input_states_s411`，number 连键 '1''2''3' 全写入 value='123'〔旧版第二键起 MaxLengthReached noop〕+ 非法 type=foo 限长对照 MaxLengthReached）。
- runner testharness.rs 零改动（s410 Enter 臂不动；MaxLengthReached 对**适用态**（text 打满后续键）的 reject 语义既有面保持——本切片不扩豁免面，见残差）。

## 过程陷阱

1. **Timeout ≠ 挂死的默认归因**：初判按 s410 同族推测「plan 截断 → assert 值错 → Fail」，实际文件级 Timeout。corpus 探针（race 强制 settle）全灭才暴露「runner 无 unhandled-rejection 换算、链 reject 后 harness 静挂至预算」的机制——reject 型案面的 Fail/Timeout 分野在**案面是否 await**（handled→Fail / unhandled→挂到预算）。相位探针（eprintln）仍是 wedge 定位最快通道（s373 先例）。
2. **runner 直跑 filter 坑重踩**：`--filter maxlength-number` 直跑静默空集——单文件 solo 必须走 `make testharness-html-semantics FILTER=<名>`（make 目标传裸值位置参数，s168 既有条目）。
3. **shim 语法门必跑**：shim JS 非编译面，改动后 `node --check` 须按 js_dom_bridge.rs include_str! 拼接序（part01/part01b/part02..06/part01c-h/part02a/part07/part08a/part08/part08c/part08b/part08d/part09/part09b/part10-typed-om/part11-color/part11-computed-color，25 文件）拼装检查——单文件 check 假报（s373 既有）。本轮全装检查 PASS。
4. **重启动后先核对上轮断点再施工**：429 打断的上轮已留全套勘根/施工 WIP 与 /tmp/s411 验证工件（sweep/join/maketest 日志），但**常驻钉未写、证据未绑定终稿代码**——接手轮须重跑电池与 sweep 换终稿证据，不得沿用 WIP 树数字入账（本轮电池与 sweep ×2 全部终稿重跑）。
5. **并行流合入踩掉共享门禁**（碰头信号，已机械清偿 + 单列记账）：HEAD 两处 fmt 脏（css-parser values/mod.rs mod 序〔random() 刀 99c573d30〕+ engine computed_style.rs 测试区〔calc serialize 钉〕）+ 一处 clippy -D warnings 红（computed_style.rs collapsible_if）令 `make test` fmt 腿与全图 clippy 全停。三处均机械等价变换（mod 排序 / rustfmt / if 收拢 let-chain），对侧 clone 工作面（layout-engine sizing WIP）与涉改文件零重叠，独立 commit 清偿并飞书通报。
6. **并行流钉被并行流改造打破**：HEAD 上 t49 cssom inline style 钉（b1fdb1db7，11-07 落）红——同流 1 分钟后的 99c573d30「写面 width 槽 invalid 拒」改造了 widthHexVerbatim 断言面（'#F00' 逐字保留 → ''）。stash 后 clean HEAD 复跑同红，归因铁证非本切片位移；engine 电池名册 t49 单列（css-parser/CSSOM 域，渲染流台账）。
7. **CPU 争用 flake 单列**：dir-solo 首跑与 clippy 并发，`focus-dynamic-type-change-on-blur`（22×5 RAF tick 子测试，RAF_FRAME_DRIVEN 时序敏感）文件级 Timeout；sweep 双跑同文件 120 子测试全 Pass + 隔离 solo 复跑 exit=0 全绿——争用 flake 不在册（s292 族判据：同二进制状态翻转 = 时序 flake）。

## 验证（/tmp/s411/ 全量日志，终稿代码）

- driving solo（dir-solo 全目录跑内独立复核非 FILTER 假绿）：maxlength-number **Pass**（旧二进制证据 = 上轮 sweep-1/2 Timeout 行）；number-invalid-scientific-key-events 保持 Pass（number 键入通道不回归）；date-readonly-keyboard 保持 Pass（s410 臂不回归）；checkable 父 label 2F 保持 F（渲染域移交，零位移）。
- 单元：engine lib **3904P/10F**（失败清单 = s410 在册 9 名逐名相同零位移 + t49 cssom 单列〔并行流钉，见陷阱 6〕；+1P = 本切片桥测钉 part110；+7P = 并行流合入 calc serialize 钉全绿）；webview lib **850P/2F**（+1P = 本切片 plan 门钉；csp_gate/image_decoder 在册 2F 与 s410 同形零位移）；`cargo fmt --all -- --check` 全仓净（跨流两处清偿后）；`cargo clippy -p zero-engine -p zero-webview --all-targets -- -D warnings` 净（跨流 collapsible_if 清偿后）。
- 终局 runner 重编：freshness 门过（bin mtime > shim mtime，release 构建）；sweep 双跑（RAF_FRAME_DRIVEN，test-guard 包裹）**run1 = run2 = 8249P/212F/25T/11 NotRun**，join-cross **stable=8484 state-diff=0 new=0**（同二进制确定性对）。三 join（s292 协议，基线 /tmp/s410/pairs-*.txt）：
  - join-1（对 s410 run1）：**恰 1 new = maxlength-number 子测试 Pass + 恰 1 gone = 同文件 file 键 Timeout**（键形迁移 file-T→subtest-P），stable=8483 **state-diff=0**——纯 driving 位移，零其他变化。
  - join-2（对 s410 run2）：同 new/gone 对 + 恰 2 STATE-DIFF = select-home-end 两键 Fail→Timeout（s292 watchdog flake 族漂移单列；两键为 Home/End/PageUp/PageDown 键序案，maxlength 路径不涉）。
  - join-cross（run1×run2）：全等零位移。
- `make test` 全量：**154 腿 153 绿 + 仅 engine lib 一腿 FAILED**（3904P/10F，失败清单与电池名册逐名相同 = 在册 9 + t49 单列；make 退出码非零由该在册红腿承接，同 s410 收口形态）。本轮编译段与并行流 workspace 测试编译并跑（load 峰值 492），测试面零争用位移。

## 口径

- 本流域在册红：**211F+24T → 211F+23T**（−1T = maxlength-number T→P 双跑一致键形迁移；select 族 2 键本轮双跑齐 T 形漂移单列——s409 轮 F→T、s410 轮 T→F 回摆、本轮 T，正相 F 形台账承 s410，族回摆时 ±2 随之单列；checkable 父 label 2F 保持 F 渲染域移交单列不变）。
- engine lib 在册红承 s410 口径 9F 零位移 + t49 cssom 单列（渲染流台账）。

## 残差与边界

- **适用态打满后续键的 reject 面**：`<input type=text maxlength=1>` 打满后再键 → MaxLengthReached noop → reject → 无 catch 案面同款挂死风险。真浏览器语义 = 静默丢弃字符（promise resolve）。当前语料无此案（maxlength.html 全目录 Pass，其打满面不走 send_keys）；若未来语料出现，修法优先 `dispatch_action` 豁免面扩 MaxLengthReached（静默 drop 语义），挂账池记。
- **maxLength IDL 反射面**：`_REFLECTED_UINT.maxLength { a:'maxlength', d:-1 }`（part01.js）为无条件属性反射——spec 对不适用态（number 等）getter 应返 -1（属性不适用=缺省）；当前 number + maxlength=1 读 `elem.maxLength` 返 1。纯 IDL 读面，本切片语料无 driving；与 minLength 同族，挂账池记。
- webview 门与 shim 门为**双写同口径**（plan noop 判定 + 原子脚本 live 守卫各自门控）——未来若 maxlength 适用态清单变化（无此规范演化迹象），两处需同步。
- unhandled-rejection→Fail 换算缺失是 runner 通用面（任何 .then-无-catch 案 + reject 型链都会挂到预算），影响面 = 文件级 Timeout 不可区分根因；上 testharness.js 上游语义（harness 有 unhandled rejection 追踪）属 runner 换算层缺口，挂账池记（勘根成本高，无当前 driving）。
- 跨流门禁清偿（computed_style.rs clippy/fmt + css-parser mod 序）与本切片改动分 commit，语义零变化；渲染流后续 rebase 无冲突面（其 WIP 在 layout-engine sizing）。

## 下一步

the-input-element 目录清零（3F+1T 散单全解挂）。续勘候选：tentative/ constructed-stylesheets 机制勘根 9T（CSSStyleSheet constructor + adoption + CSP 大机制，勘根后按簇决策）/ 挂账池承 s404/s405（forward map 空返勘根/渲染流 5 面/form-submission-target 6T/sizes-auto 半边/ilf+modulepreload 观察池 + 本切片新增：适用态打满 reject 面、maxLength/minLength IDL 不适用态反射、unhandled-rejection 换算）；checkable 父 label 2F 渲染域移交承 s410；select/combobox（簇 D）与 A2+D 簇立项征询仍待用户决策。
