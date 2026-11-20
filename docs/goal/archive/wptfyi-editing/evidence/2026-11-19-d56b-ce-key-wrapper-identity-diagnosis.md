# 单键隔离归因重构与 CE 键管线 wrapper 身份失稳诊断（2026-11-19）

> 性质：①(b)（跨元素边界 caret join）落刀前的净页/隔离归因轮。本证据
> **勘误本流上一轮两份净页探针结论**（v1 caret 伪影、v2 8-in-1 跨测污染），
> 并给出 bs/fd 失败键的单键隔离分类账与机制诊断链。基线二进制
> /tmp/d56-wip0-runner（＝d56 收口树，HEAD b4d20851b 内嵌 part01.js 同代）。

## 1. 探针方法论勘误链

| 轮次 | 方法 | 结论 | 处置 |
|---|---|---|---|
| v1（上一轮） | 8 形合一页，caret 用 `gEditor.children[1]…` | T4 内容丢失净页复现、T7 p/p 直连净页不 join | **全伪影**：corpus 形状外层 div 嵌在 gEditor 内，children[1] 不存在，caret 落空（实读 corpus 源码 880-935 行 querySelector 约定后定谳） |
| v2（本轮前半） | 8 形合一页，caret 改 corpus querySelector 约定 | T1-T4/T6/T7 净页 Pass；T5/T8 挂 | **部分失真**：T5 挂是前测 `<b>` ghost 污染读数（T8 形状 corpus 不存在，系合成变体）；Pass 侧亦不可靠（见 §2） |
| 单键隔离（本轮主方法） | 从 corpus **verbatim 抽取** promise_test 块，一键一页，/tmp/d56-wip0-runner ×1-2 轮 | §3 分类账 | **本轮定谳口径**。抽块器两 bug 已修：bs/fd 页名同前缀互覆（`fbase[:6]` 均为 `input-`）；corpus header 以注释行结尾且 `<script>` 未闭合，直拼吞块致 `await` 顶层 SyntaxError。修后页校验：块以 `}, 'KEY');` 收尾截取＋`promise_test(` 计数=1 |

## 2. 单键隔离分类账（20 键，×1-2 轮确定性）

页生成器：`/tmp` 内联 python（verbatim 块抽取）；运行：`./target/test-guard
--per-proc-mem 4 --total-mem 8 --time-limit 300 -- /tmp/d56-wip0-runner
testharness-input-events "zz-iso-*" --json`。PF/FF＝父断言（getTargetRanges
事件序）/innerHTML 子断言。

### ③ 页内毒化（隔离 PP、corpus 全程挂）

- bs-00 `Backspace at "<p><i>abc</i></p><p><b>[]def</b></p>"` PP ×1
- fd-00 `Delete at "<p><b>abc[]</b></p><p><b>def</b></p>"` PP ×1

### 事件计数族（innerHTML 过、bi=0，PF）——8 键

bs-06（T1 形）/bs-07（T1 i 形）/bs-10（T5 形）/bs-11/bs-12/bs-13、fd-01/fd-02。
join 全部发生且形状正确，唯 `gBeforeinput.length==0`。

### 无 join 族（FF，DOM 未变，bi=0）——4 键

bs-05（T2 形 `<div>abc<p><b>[]def</b></p></div>`）×2、bs-08（T6 list）、
bs-09（li×2）、fd-03。got＝初始内容原样。

### 其余真净页 bug（与 wrapper 身份无关的独立族）

- bs-01/bs-02 pre↔p 空白 join（got `<pre>abcdef</pre>` 形）×2
- bs-03 hr（got `<div>abc<hr>def</div>`＋bi=0）
- bs-04 br+ws（无操作，gtr 区间错形 `(<p>,1)-(<p>,2)`）
- bs-14 img innerHTML 序列化（FP，父过）
- bs-15 RTL（`שלום[]hello` FF）

## 3. 机制诊断链（zz-diag-d56{a..m}，恒失败探针吐账）

1. **管线代理 trace（d56h）**：Actions 键全程被调——probe isHost=1 →
   `delete("html:nth-child(1) > body:nth-child(2) > div:nth-child(1)")`
   enter → `caretRange=#text:@0`（**空文本节点@0**，非 def@0）→ ret 后 DOM
   未变。键路由与 shim 入口无恙，**caret 在键时刻解析错锚**。
2. **直调绕过（d56g）**：harness 全序列后绕过 Actions 直接调
   `__zw_ce_delete("div[contenteditable]")` → join 完美
   （`<div>abc<b>def</b></div>`）。shim walk 在 fresh 页正常。
3. **probe 无毒（d56i）**：手动复刻动作层 probe 脚本（同 selector）→
   before/after caret_range 均 =def、wrapper 恒等、监听器在。probe 脚本
   本身无毒。
4. **身份churn 实录（d56j2）**：querySelector 身份代理——initializeTest 解析
   `div[contenteditable]=>W1`（监听器挂 W1）；序列中 `div=>W1` 后出现
   `div=>W4`，此后**所有** `div[contenteditable]` 查询返 W4（含 Actions
   probe/delete）。监听器在 W1、派发在 W4 → `gBeforeinput` 恒空（事件计数族
   根因）。
5. **身份切换触发读（d56m 对照）**：**省去** corpus 块的
   `let abc = gEditor.querySelector("div").firstChild` 前置读时
   `w2 === gEditor` 恒真（无切换）；该读在场＝W1→W4 切换＋键死（d56h）。
   e1/e2 变体实证：删该行或挪到 collapse 之后 → join 复活。
6. **纯 shim 双对象实证（d56K/d56L，无 corpus harness）**：
   `gE.querySelector("div") === gE.firstChild === true`（query/树行走同物），
   但 `qB.parentNode !== q1`——同一嵌套 div 的 **parse 期 wrapper 经 parent
   指针仍可达**，与 canonical 对象并存。元素级 querySelector 无 spec 上下文
   自含问题（`querySelector('div')===gE: 0`）。
7. **事件面残余（d56m）**：join 活时，挂在 canonical wrapper 的**新建**
   beforeinput 监听器仍 biCount=0；而 d56c 混合页第二测（harness 自挂监听器
   ＋document 捕获在场）gBI=1。事件可达性与「监听器挂载时点×家族」耦合，
   未收口（下一步诊断项）。

## 4. 与既有 learning 的关系

- `docs/learnings/bugs/2026-10/2026-10-14-js-dom-wrapper-family-identity.md`
  已建档「同一树节点多族稳定 JS 包装、跨族 identity 恒败」为结构属性，既定
  方针＝跨族匹配 match-only、写回保调用方原对象（不统一对象）。本轮新增量：
  **corpus harness 流是该方针的残余缺口面**——监听器挂载（initializeTest 于
  setupEditor 写前解析 gEditor）与键时派发（walk 内 querySelector 解析 el）
  天然跨族；`querySelector("div")` 类前置读触发 canonical 切换把缺口放大成
  键级死亡（bi=0 / 无 join）。
- D3 世界同步（caret_range 10306-10461）即该方针的 caret 应用；本轮观测的
  错锚（#text:@0）为其 sel-path 重解析在 anonymous 子树＋陈旧 parent 指针
  下的落点偏差。

## 5. bs/fd 失败账分桶预估（77F＋64F 项口径）

| 桶 | 机制 | 采样翻面潜力 |
|---|---|---|
| bi=0 事件计数族（嵌套 div 外形 bs 22＋fd 15 键 ×2 项为主） | wrapper 身份churn：监听面/派发面跨族 | 修事件面对齐 → 父断言翻面（join 侧 8 键直接全过；无 join 侧需并修 caret 错锚） |
| ③ 页内毒化 | 历史态 rebake 毒（d49 域既有结论） | 不动 |
| pre↔p ws、hr、br+ws、img 序列化、RTL | 各自独立族（部分带 bi=0 共面） | 需逐族刀 |

## 6. 运行产物

- 隔离页：`tests/wpt-runner/wpt-data/input-events/zz-iso-{bs,fd}-*.html`
  （20 页，gitignored）；结果 `/tmp/d56-iso/zz-iso-*.json`、
  `/tmp/d56-iso/jobs.txt`
- 诊断页：`zz-diag-d56{a,b,c,f,g,h,i,j2,k,l,m}.html`（gitignored）；结果
  `/tmp/d56-diag{a,b,c,f,g,h,i,j2,k,l,m}.json`
- v2 探针：`zz-probe-d56j.html` ＋ 单形页 `zz-probe-d56j{t1..t8}.html`；
  结果 `/tmp/d56-probe-v2-{1,2}.json`、`/tmp/d56-probe-t{5,8}-{1,2}.json`
- 二进制：/tmp/d56-wip0-runner（d56 收口树 release，01:32 构建，全程基线）

## 7. 下一刀方向

1. **bi=0 事件计数族**（最大桶）：按 2026-10-14 match-only 方针补 CE 键
   管线事件面对齐——派发面（`_zwCeJoinDispatch`/`_zwCeUnitApply` 的 el 解析）
   与 harness 监听面的家族调和；兼修 caret 错锚（anonymous 链 sel-path
   落点）后无 join 侧同批翻面。
2. 事件可达性残余（d56m biCount=0 vs d56c-2 gBI=1 的时点×家族耦合）先行
   小探针收口，再动 part01.js。
3. pre↔p ws / hr / img 序列化 / RTL 各族保持既有挂账，逐族单独刀。
