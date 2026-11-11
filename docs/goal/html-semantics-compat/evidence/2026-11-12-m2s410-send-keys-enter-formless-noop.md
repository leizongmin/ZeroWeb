# M2 切片 410 — send_keys Enter formless 无隐式提交目标默认动作 no-op（date-readonly-keyboard 1F 解挂；checkable 父 label 2F 渲染域移交）

日期：2026-11-12（承 s409 下一步：the-input-element 散单 3F）
commit：见 git log s410 提交

## referent 与勘根

driving 键（s409 收口时 the-input-element 在册 3F+1T 的散单面）：

- `the-input-element/date-readonly-keyboard.html :: Readonly date input should not change value via keyboard interaction`（本切片施工解挂）
- `the-input-element/checkable-active-onblur-with-click.html :: Checkboxes/Radios … pressing the parent label with a pointing device`（2F 同根，勘根后移交渲染域，单列不动）

### date-readonly-keyboard：runner Enter 默认动作臂错置

案面 `<input type=date readonly value=1970-01-01>`，四 promise_test：focus → send_keys(" ") → send_keys("") → send_keys("") → assert value 不变。旧版逐键归因：

- Space：runner InsertText 分类 → `script_text_control_snapshot` 收 readonly date 快照 → `plan_text_insert` → `Err(ReadOnlyTarget)` → dispatch_action 错误 → send_keys 内部既有 **scroll_fallback 兜底**（readonly 目标按滚动语义消化）→ 存活（既有绿面）。
- ArrowDown（uE015）：scroll 键分支直接派 keydown/keyup，不进 dispatch_action → 存活。
- **Enter（uE007）：`_ =>` 默认动作臂无 formless 判定** → `HtmlUserAction::Submit` → webview Submit 臂查 enclosing form 为无 → `Ok(noop(ActionNoopReason::NotApplicable))` → `dispatch_action` 只豁免 `DisabledTarget`，NotApplicable 照返错误串 → send_keys promise 链 reject → 案面 unhandled rejection → **Fail**。

真浏览器语义：implicit submission 以 form owner 为前提，formless 目标 Enter = 无默认动作 = 静默 no-op（keydown/keyup 照常完成、不抛错）。https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#implicit-submission

### checkable-active-onblur-with-click 2F：父 label 零矩形（渲染域移交）

案面 `<label id=lc>abc <input type=checkbox id=cb2 checked></label>`，press (2,2) origin。双层探针（勘根毕即删）：

1. **桥测探针证 shim `:active` 路由无恙**：沙箱 `__zwSetActiveFromSel('#lc', true)` → label 无 for → `_zwLabeledControl` childNodes DFS → `cur.control` = cb2 → `input:active` 命中（`#cb2` sel 直配亦命中）；for-label（`<label for=cb1>`）同通。shim 通道（S196 单槽 + `_zwSetActiveFromSel` 祖先 LABEL walk）排除嫌疑。
2. **corpus 探针证父 label 布局面零矩形**：`getBoundingClientRect()` —— 父 label（含文本 + 行内 input）= **0,0,0,0**；同案子 input cb2 = 12,79,13,13、following label = 32,12.2,26.4,18.6 皆有实矩形。Actions `pointerMove(2,2,{origin:label})` 计算绝对坐标 (2,2) → pointerdown 落点命中 BODY → `:active`=BODY → `input:active` null → 断言红（探针 DUMP：`A:rect(label)=0,0,0,0 … A:input:active=NULL,any:active=BODY#`；对照 B 键 following label 实矩形命中 LABEL#）。

根因在 layout-engine inline-container boxing（含混排内容的 label 未产盒），属渲染流 face → 按 s405 先例移交渲染域记账，本流域不动引擎、不设 workaround。s389 分诊「checkable-active-onblur 2」的流内可施工性到案面勘根为止：shim 面无施工点。

## 施工

- **runner `enter_formless_noop` 臂**（`tests/wpt-runner/src/testharness.rs` `_ =>` 分支，`enter_formless_buttonish` 判定之后）：`!enter_formless_buttonish && (uE007|uE006) && 非 CE 宿主（contenteditable probe ≠1）&& 无 enclosing form（form probe ≠1）` → 无默认动作——不进 dispatch_action、不 reject 链，只完成键事件序（keydown 已派、keyup 分支尾部恒派）。三重例外保既有绿面不动：
  - `enter_formless_buttonish` 优先（formless button 的 PointerEvent click 通道既有，判定含在前提取反内）；
  - CE 宿主排除（insertParagraph 默认动作活在 webview Submit 臂，跳过即丢行为）；
  - form 内目标排除（隐式提交族既有绿面不动）。
  探针函数 `script_contenteditable_probe`/`script_enclosing_form_probe` 均为 script_gen.rs 既有（纯组合，零引擎改动）。uE006（numpad Enter）同享语义。

## 过程陷阱

1. **并行流编译面撞车**：桥测探针首跑撞 zero-layout-engine E0004（并行流 calc-mix 刀 `CalcExpr::Mix(_)` 漏臂，其修复 c1a069a1c 在远端）——`git stash push` → `git pull --rebase`（至 5c02b84d4）→ `git stash pop`，探针照跑。双 clone 并行下勘根前先同步对侧域编译健康。
2. **runner 日志 grep 二进制字节**：dir-solo 日志含 NUL，`grep` 静默零输出——须 `grep -a`（wpt-runner 调用坑既有条目重踩）。
3. **probe B 探针工件不入根因链**：corpus 探针 B（following label press）DUMP 显示 input:active NULL 但真案该键 Pass——探针自身派发时序与真案 Actions 管道（hover 迁移/S196 预标记相位）有差，属探针专用工件；主根因（父 label 零矩形）由对照面（A 零矩形 vs B 实矩形）独立成立，B 面分歧不追。
4. **电池 lib 腿 101 ≠ 新红**：engine lib 9F/webview lib 2F 初看刺眼，实为在册名册（roster diff 对 s409 逐名核对，见验证节）；lib 腿失配须先做名册 diff 再定性，不能只看 exit code。

## 验证（/tmp/s410/ 全量日志）

- driving solo：date-readonly-keyboard **Pass**（dir-solo 全目录跑内独立复核非 FILTER 假绿；旧二进制证据 = /tmp/s409 sweep-2 Fail 行）；checkable 父 label 2F 保持 F、maxlength-number 1T 保持 T（零位移）。
- 单元：engine lib **3897P/9F**（s409 名册 10F diff = **恰 `r307_tree_order_identity_domains` F→P 单腿位移**——本切片改动仅在 runner crate，engine lib 编译面不感知；favorable 位移归并行流合入域）；webview lib **849P/2F**（csp_gate/image_decoder 在册对，与 s409 同形零位移）；`cargo fmt --all -- --check` 干净；`cargo clippy -p zero-wpt-runner --all-targets -- -D warnings` 干净。
- 终局 runner 重编：freshness 门过（bin mtime > src mtime，release 构建免重编 0.21s）；sweep 双跑（RAF_FRAME_DRIVEN，test-guard 包裹）**8484 keys run1=8246P/212F/26T、run2=8246P/214F/24T**，双跑 new=0/gone=0 键集全等（对 s409 8245P/213F/26T：+1P −1F 双跑一致 = date-readonly F→P）。三 join（s292 协议）：
  - join-1（对 s409 run1）：**恰 1 STATE-DIFF = date-readonly-keyboard Fail→Pass**，stable=8483 new=0 gone=0——与设计面逐键一致。
  - join-2（对 s409 run2）：3 STATE-DIFF = 上键 + select-home-end 两键 **Timeout→Fail**（flake 族漂移，见下）。
  - join-cross（run1×run2）：恰 2 STATE-DIFF = select-home-end 两键（run1 T / run2 F）——s292 watchdog flake 族既录 F→T/T→F 双向漂移（s409 轮 F→T、本轮 T→F 回摆）；两键为 Home/End/PageUp/PageDown 键序案，Enter 路径不涉，非本切片位移。
- `make test` 全量：**153 腿全绿 + 仅 engine lib 一腿 FAILED**（3897P/9F，失败清单与电池名册逐名相同；r307 本轮 `... ok`——F→P 位移跨跑稳定非 flake）；`make test` 后 release runner 产物被 target-disk-guard 分级清理的坑未触发（本切片 final sweep 先于 make test 完成）。

## 口径

- 本流域在册红：**212F+24T → 211F+24T**（−1F = date-readonly-keyboard F→P 双跑一致；checkable 父 label 2F 保持 F 归因单列 = 渲染域移交；select-home-end 承 s409 单列口径——本轮 run2 F 形即族正相、run1 T 形漂移单列，不计位移）。
- engine lib 在册红承 s409 口径（9F + r307 单列；本轮 r307 F→P favorable 位移，名册缩至 9F 齐平「9 在册」旧口径）。

## 残差与边界

- `enter_formless_noop` 判定在 Enter 键序上增加两轮 execute_script 往返（CE probe + form probe）——send_keys 本就逐键往返，runner 端可接受；formless 非 Enter 键不受影响。
- `unwrap_or(false)` 兜底：探针执行失败（桥缺席等）按「有默认动作」走旧 Submit 臂——失败开放（fail-open）方向保既有绿面，符合 runner 车道语义。
- checkable 父 label 零矩形勘根结论移交渲染域（inline-container boxing），本流域不设 shim/workaround；渲染流修复后 2F 应自然解挂（届时在册口径再 −2F）。
- formless readonly date 的 Enter 无默认动作是 spec 正确面；若未来语料出现「formless Enter 须抛 not-applicable 错」反例（无此规范依据），届时重审。

## 下一步

maxlength-number.html 1T 勘根（file-level Timeout，先验语料资产存在性——s227 先例 images/ 缺文件与引擎 bug 不可区分）/ tentative/ constructed-stylesheets 机制勘根 9T（CSSStyleSheet constructor + adoption + CSP 大机制，勘根后按簇决策）/ 挂账池承 s404/s405；checkable 父 label 2F 已移交渲染域记账（s405 先例）；select/combobox（簇 D）与 A2+D 簇立项征询仍待用户决策，不动。
