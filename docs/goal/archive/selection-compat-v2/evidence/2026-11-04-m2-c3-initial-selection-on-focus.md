# M2 / C3 initial-selection-on-focus 收敛 + 伴随回归清算（2026-11-04）

## 结论

C3 目标案 `selection/contenteditable/initial-selection-on-focus.tentative.html?div/?span`
149F → **0F**（?div 99F→100P、?span 50F→51P，run A/B 双轮稳定），判据满足：
目标簇 151 subtests（≤300）且零新引擎概念——全部为既有 shim 钩子的时序/门控
修正与 runner 键盘路由补全，未新增任何 DOM/选区引擎概念。

伴随收敛（同批工作，语料同面）：collapse.html 3F→3P、
focus-propagation ×2 F→P、no-beforeinput Typing Delete F→P、
caret/move-around-generated-content ×2 F→P（A/B 双轮稳定）。

全量对 M1 canonical 基线零回归对照：run A 出现 25 回归 → 逐案取证定位
5 簇机制、全部修复 → run B 22 → run C 2（paste 死循环案 90s 预算挤压连锁
假红）→ run D 1（anchor 案——paste 管线激活后暴露的两个深层根因，见簇6/7）
→ run D′（终态二进制，+Fix A v2/Fix B）**0 回归**，且基线 Pass subtest
缺失人工扫 = 0（compare 脚本盲区复核，见下）。

## 运行方式

```
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 7200 -- \
  ./target/release/zero-wpt-runner testharness-selection --json
```

全量 675 案 / 51,757 subtests，单轮 ~11-13 分钟。对比脚本逐 subtest diff
全量 run JSON vs M1 canonical 基线（`2026-11-04-m1-baseline.json.gz`，
35,638P/16,115F/4N）。轮次落盘 /tmp（m2-c3-run.json=A、m2-c3-run-b.json=B、
m2-c3-run-c.json=C、m2-c3-run-d.json=D、m2-c3-run-dprime.json=D′），
对比口径与 M1 attribution 文一致。**已知盲区**：基线有而新跑缺失的 subtest
不计入 REGRESSIONS——D′ 轮补人工全量扫（见判据核对）。

## 轮次数字

| 轮 | 二进制状态 | FIXES | REGRESSIONS | 说明 |
|---|---|---:|---:|---|
| A | 时序+空选区已修（part04/part01） | 154 | 25 | 暴露 runner 键盘路由缺陷与 paste 死循环 |
| B | + runner accel/nav 路由 | 154 | 22 | 簇1/簇2 修绿实证 |
| C | + paste 快照插入 | 158 | 2 | 死循环解除；paste-text-after 仍 case 级 Timeout（簇6 在案）；onselectstart Select All ×2 退化属死循环案 90s×N 预算挤压连锁假红（D 轮消失）；editing face P 13,334→12,000 同因（case 级 Timeout 吞后续 subtest） |
| D | 同 C 终态前夜（paste 管线激活） | 158 | 1 | 簇6 显影：anchor 案 Fail（簇7）；paste-text-after 案级 Timeout（基线 2P 缺失——盲区外诚实账 3 subtests/2 案）；move-around +1 新 Fail subtest（前置修复推进测试链揭示次位失败，非基线退化） |
| D′ | + Fix A v2/Fix B（终态） | 158 | **0** | 零回归定谳：REGRESSIONS=0，基线 Pass subtest 缺失人工扫=0 |

totals（D′ 终态）：P 35,638→35,796（+158，恰=FIXES）、F 16,111→15,954
（−157）、N 8→8；selection face P 22,304→22,459、F 13,751→13,597；
editing face P 13,334→13,337、F 2,360→2,357。

## 目标案与伴随案详情（B 轮 vs 基线；paste 两案为 D′ 轮）

- `initial-selection-on-focus.tentative.html?div`：{Fail 99, Pass 1} → {Pass 100}
- `initial-selection-on-focus.tentative.html?span`：{Fail 50, Pass 1} → {Pass 51}
- `collapse.html`：{Fail 3} → {Pass 3}（collapse() 聚焦 editing host 挂初选区，
  Chromium SetFocusedElementIfNeeded 语义）
- `selection-outside-focused-editing-host.tentative.html`：{Pass 2, Fail 1} 保持
  （剩余 1F 属 C2 modify 簇，本 M2 明确不动，见 master.md 挂账表）
- `caret/move-around-generated-content.html` ×2：Fail→Pass（bonus，导航路由
  补全后 caret 落点断言过）
- `html-text-copy-paste-of-anchor-with-href-in-content-editable.html`：
  基线 {Pass 1} → A/B Fail → D′ {Pass 1}（簇7 修复；`<a href>` 复制→粘贴后
  href 保留断言过）
- `paste-text-after-collapsible-white-space-whose-container-remove-non-first
  -children.html`：基线 {Pass 2} → C/D 案级 Timeout → D′ {Pass 2}（簇6 修复；
  页面自带 `while (child = host.childNodes[1]) frag.appendChild(child)` yank
  环不再自旋，两 promise_test 走完）

## 回归根因五簇（run A 25 案逐案取证，全部修复）

### 簇1 focus 钩子时序（×2：focus-propagation at-the-editing-host/at-the-window）

`__zw_initial_selection_on_focus` 原挂在 focus/focusin dispatch **之后**——
focus 事件 listener 内断言「选区已挂」时钩子尚未执行。uievents 规范
「focus MUST be taken before dispatch」（https://wpt.fyi/uievents/#focus-event-sync
及 DOM spec focus 事件派发步），Chromium SetSelectionFromNone 先于 focus
派发。修复（part04.js）：钩子调用移到 dispatch 之前。

### 簇2 空选区键入兜底 caret（×1：no-beforeinput Typing Delete）

`__zw_ce_caret_range` 原对空选区兜底 `selectNodeContents(host)+collapse`——
空选区键入照常变异内容，违背「无选区 → 编辑命令 no-op 零事件」quirk。
修复（part01.js）：空选区返 null，5 处调用方早退；`__zw_ce_paste_html` 同门。

### 簇3 runner accel 键 fallthrough（×1：onselectstart Select All）

test-driver Actions 链 ctrl/meta+A 原无路由，PUA 单字符 fallthrough 到
InsertText 字面插入 `'a'`（断言 got "78a9"）。修复：testharness.rs Actions
分支加 `accel_sticky` 持久化位（ctrl  / meta  keydown 置位
keyup 清零），a/c/v/x 命中时路由 `script_exec_command(selectAll/copy/paste/cut)`
（script_gen.rs 新增单点 execCommand 包装）；EditorTestUtils
sendCopyShortcutKey/sendPasteShortcutKey/sendSelectAllShortcutKey 走 Actions
链后可达真命令。

### 簇4 runner nav PUA 键 fallthrough（×20：onselectstart Shift+Arrow/Home/End/PageUp/Down）

导航键 PUA 编码（-、）原同样 fallthrough InsertText，字面
PUA 字符污染编辑区致断言崩。修复（testharness.rs）：`nav_key_name` 识别表
扩全并在 Actions 分支抑制（导航位移语义由 shim 侧选区逻辑承接，runner 不再
注入文本）。

### 簇5 paste 默认动作 detached fragment 死循环（×1：html-text-copy-paste-of-anchor）

`__zw_ce_paste_html`（part01.js，paste 默认动作核）原
`while (frag.firstChild) { refParent.insertBefore(n, refNode) }` 活环消费——
CEF 产物为 M 域节点（无 sel/handle 印），insertBefore 不发生源位摘除，
frag.firstChild 永不推进 → 死循环（探针 zzz-probe-cp T4 execCommand('paste')
90s Script error 实证）。修复（C 轮）：先
`Array.prototype.slice.call(frag.childNodes)` 快照（4096 护栏），逐个显式
`frag.removeChild(n)`——环终止实证。**但插入半边仍未修**：M 域子
insertBefore 全分支 miss 静默丢弃，anchor 案 D 轮仍 Fail → 簇7。

### 簇6 appendChild 无 handle 子挂 handle 容器缺旧父剔除（×1：paste-text-after-collapsible-white-space，D 轮案级 Timeout）

语料页面自带 paste listener 执行
`while (child = host.childNodes[1]) frag.appendChild(child)`——appendChild
trap 对「无 handle 子」（innerHTML= 同 turn 的 _zwMEl/_zwMText pending 解析子
与 sel 章子两类）挂 handle 容器（fragment/shadow，目标 sel 空）走
fall-through：只 `_handleChildren` registry push + parentNode getter 重指，
**旧 sel 父的融合 childNodes 不收缩** → 页面环恒见同子自旋 90s case 超时
（探针 zzz-probe-m2fix3 T8 实证：parsed 子 reparent 后 host kids 恒 2、frag
逐轮膨胀 g=21；JS createElement 产物 handle 子走既有 R51 臂正常 g=1——
真变量是子身份，dispatch 语境排除，T4/T7 对照）。

修复（part04.js，Fix A v2）：fall-through 前置旧父剔除块，R5340 removeChild
pending 解析子臂同款三件——wire `__zw_remove_child_at(oldSel, idx)`（apply 序
在 SetInnerHTML 之后，pending 子已落地）+ 桶 retract（`_zwPendBucket`
added splice + removed push，overlay 同 turn 即隐=自旋环终止）+ 迭代器/range
通知 + `child.parentNode = null` 断链 + childList removed record（`_mo_notify`
汇流点全簿记）+ liveNL sync；D1 印章子（`__zwSelPathWrite`）wire 让位同 R5340
双删互斥先例。门 = `!child.__zwHandle && handle && !sel && (child.__zwSelector
|| nodeType 1/3)`，旧父读经 child.parentNode getter 取 sel/handle，先于 R84
parentNode getter 重指执行。**v1 教训在案**：v1 门仅 `child.__zwSelector`，
T8 复跑 A:g=21 不动实证 innerHTML= 子实为无印 M 域视图（`_zwFragmentAdded`
_zwMEl 代理树），v2 扩门后 T8 全绿（A:g=1:kids=1:frag=1）。

### 簇7 paste 插入核 M 域子错层（×1：html-text-copy-paste-of-anchor，A/B/D Fail）

簇5 快照修复后插入仍静默丢失：快照循环对 CEF 产物 M 域子逐个
`refParent.insertBefore(n, refNode)`——insertBefore trap 对 M 子无 wire 分支
（R334 门=子 sel、R5058 门=子 sel+text、R51 门=子 handle），空宿主 caret
（sc=DIV@0 → refNode=null）与 M 元素任何形态组合均 miss → 引擎零落地
（探针 zzz-probe-m2fix2 T6 直调 `__zw_ce_paste_html(p,"HELLO")` 返回空
innerHTML 实证；T2 证明 CEF 解析 `<a href>` 本身正常、T1 证明 copy store
含 href——断点定位在插入层）。

修复（part01.js，Fix B）：插入核重写为锚位计算 + 引擎整包落地（本文件
plain 壳引擎收敛既定先例同款）——① 文本 caret：中段拆分走「左半留驻
（nodeValue setter 走 R48 `__zw_set_child_text` wire）+ 右半转义殿后于
payload」等价 splitText 双文本节点语义，锚=文本后/前首元素兄弟
`_zwPosSelOf` + beforebegin/afterend，无兄弟落容器 beforeend/afterbegin；
② 元素容器 caret：child[at] 元素 → `_zwPosSelOf` + beforebegin，文本子 →
前元素兄弟 afterend，越界 → 容器 beforeend；③ wire
`__zw_insert_adjacent_html(anchorSel, anchorPos, payload)` 一次整包（引擎侧
解析为权威，shim 侧不建 stand-in——语料断言在 apply 后 turn 走引擎态）；
④ caret 落位 best-effort 容器尾（FIXME 在案：Chrome 语义为粘贴体尾；
shim-only 容器 `_zwPosSelOf` 寻址失败静默不落）。保留
beforeinput(paste)→变更→input 事件序与非塌陷 deleteContents。
T6 直调序列（text+anchor 同 turn 双 paste）与 T3 execCommand 端到端全绿，
paste-with-newline-three-times 三连续粘贴（drain 间 sel 章子路径）无回归。

## 探针生命周期

诊断期临时探针 `zzz-probe-m2fix{,2,3}.html`（T1 copy store/T2 CEF 解析/
T3 paste 端到端/T4 wrapper 子 reparent/T5 护栏 case2 复现/T6 插入链分解/
T7 dispatch yank 环 trace/T8 parsed vs wrapper 子 reparent 控制实验）——
Fix A v2/Fix B 全绿（probe1 EXIT=0 ×5、probe2 EXIT=0 ×2、probe3 T8
A:g=1:kids=1:frag=1）后于 D′ 全量前删除（`rm zzz-probe-m2fix{,2,3}.html`），
语料目录 `zzz` 残留 = 0，D′ 全量 675 案不含探针案。

本 goal 依赖的两目标案与族扫 13 案均为上游 WPT 语料（`editing/other/` +
`editing/include/editor-test-utils.js`，gitignored wpt-data 按 WPT_REV pin
raw fetch 幂等补拉惯例，通道与映射见 M1 语料 evidence）——非本 goal 新作
语料，无入库面。

## 判据核对

- C3 判据「≤300 子测试且零新引擎概念」：目标簇 151 subtests ✓；改动面为
  既有钩子时序（part04 移位）、门控收窄（part01/part06）、runner 键盘路由
  补全（testharness.rs）、既有簿记机制复用（R5340 三件/IAH 先例/R48 写
  wire）——无一新增引擎概念 ✓
- 零回归：run D′ 全量 vs M1 canonical 基线 REGRESSIONS=0（FIXES 158 全
  保留）✓；**盲区人工扫**：基线 35,749 subtests 逐名对 D′ 缺失检测 = 1，
  且该名（selection-nested-video）基线态为 Timeout 非 Pass（D′ 呈现为同案
  case 级 Timeout 标记，N→N 非退化——compare 脚本按 subtest 名差记
  NEW 的记账伪影）→ 基线 Pass subtest 缺失 = **0** ✓
- 新增 subtest 诚实账：move-around-contenteditable-false +1 Fail（前置修复
  推进测试链揭示的次位失败，基线未运行该 subtest，非基线退化）；本 goal
  域外新失败无
- paste/copy 族扫（13 案逐文件单跑 + exec-command-with-text-editor ×3）：
  全部与 M1 基线状态分布逐案一致，无回归信号（solo 结果不作回归判据，
  M1 先例；全量 D′ 复核为准）
- copy-paste 管线（簇3/5/6/7 修复）与焦点初选区（C3 主修）互为前提：
  paste 依赖「聚焦宿主挂 caret」+「Actions 链 ctrl+V 可达」，均为本批落地
- 门禁账（收口轮终态复跑第一手）：make test workspace 腿
  **3724P/5F**——5 红逐名 = `p56_adopted_snapshot_restore_hook_keeps_page_value`
  / `p99s_rule_style_preserves_custom_property_case` /
  `stylesheet_add_rule_remove_rule_r3276` / `stylesheets_rule_style_r2810`
  / `stylesheets_write_r2809`，与 webcrypto-v3 收口账（2026-11-03 mend
  closure §五，tree `b3d86b77c` 3719P/5F）及本 goal M1 门禁账（3722P/5F
  同族）逐名一致 = **s249 CSSOM 序列化尾分号在册红，HEAD 既有**；
  browser 腿 742P/1F `default_actions_work_without_javascript` = browser
  forms/typing 兄弟流在案红（webcrypto-v3 同账在案，rendering-compat
  R5412 同日 stash 实证 clean HEAD 恒红）——均 run-rules §9 不代修，
  **本轮 shim 改动零新增红**（P 3722→3724 增量 = part06 新增测试腿）。
  fmt 门禁（T58 前置）PASS；`make guarded-clippy` 全工作区 -D warnings
  **EXIT=0**（收口轮终态复跑）。

## 基线与对照口径

- 基线：`2026-11-04-m1-baseline.json.gz`（M1 canonical，P/F/N 逐 subtest）。
- 对照脚本：/tmp/m2-compare.py（cases/totals/faces/FIXES/REGRESSIONS/
  new-subtests/目标案详情七段），solo 结果不作为回归判据（M1 先例：
  上下文相关案 solo 与全量表现不一致）。

## run-e canonical 复跑确认（收口轮）

D′ 终态二进制幂等复跑 **run-e**（09:28–09:40，guard 包裹 4/8/7200，墙钟
~11 分钟，675 案 / 51,757+1 subtests）：**FIXES=158 / REGRESSIONS=0 /
nonPass→nonPass status-change=0**，faces/totals/目标案详情与 D′ 逐段全等
（totals P 35,796 恰 = +158）——C3 收敛与零回归双确认，run-e 为下账
canonical。固化：`run-e.json.gz`（462 KB）+ 全量七段对照
[`2026-11-04-m2-c3-compare-e.txt`](2026-11-04-m2-c3-compare-e.txt)（对比
脚本输出原件）落本文档目录；raw JSON 留 `.acceptance/m2-c3/`（诊断目录
不入库惯例）。

run-e 对照输出补充两项诚实账（与判据核对节一致）：

- NEW subtest `selection/caret/move-around-contenteditable-false.html ::
  Move caret from empty editable paragraph ...`（Fail）——前置修复推进
  测试链揭示的次位失败，基线未运行该 subtest，非基线退化。
- `selection/selection-nested-video.html` Timeout 记账伪影：基线 Timeout
  subtest 名与 run case 级 Timeout 标记差异致 compare 脚本记 NEW/MISSING
  各 1，N→N 非退化。
- **T1 复核（fontSize ×3）**：run-e 逐条核验 `editing/event.html ::
  Command fontSize value ""/"quasit"/"15px" input event` **仍全 Fail**——
  master.md T1 挂账维持原状（33 号域），editing face +3P 另有来源 =
  focus-propagation ×2 + no-beforeinput ×1（伴随清单），与本批目标域
  一致，无未归因翻转。
