# M2-S178 radio 组排他 sweep 净件臂 R2998 捕获缺失——form reset 恢复落空根修，renderer 基线两案销账

## 入场定刀位

s177 新立 renderer lib 全量基线 354P/5F 的挂账勘定（page_scripts 2 案——
`form_interaction_fixture_complete_sequence` / `javascript_disabled_skips_listeners_not_default_actions`，
stash 对照在案既有红，测试体自 2026-08 8cb3e919a 后未动，失败时点不明）。

## 勘根（探针三态实证）

两案失败签名同根：

- fixture 案：reset 后 `state["plan"]` 应 `"basic"`（默认选中 radio）实为 `""`——
  fixture reset 监听 `queueMicrotask(syncTestState)` 在恢复后读 live checkedness，
  `""` = 双 radio unchecked 的读数（恢复落空直接显形）；
- js-disabled 案：reset 后 `#basic` 的 `checked` 属性应恢复实为无。

探针（临时案，验后删）三态：

```text
P1 快照后        basic dc/checked = true/true
P2 组排他后      basic dc/checked = false/false   ← defaultChecked 翻假（应保 true）
P3 reset 后      basic checked = false            ← 恢复落空
```

**根因单点**：s125/s131 radio 组排他 sweep `_zwRadioUncheckGroupOthers`
（part05.js）净件臂（`_checkedDirtyState` 无脏件）**直接移除 checked 内容
属性、绕过 `.checked=` setter 的 R2998 捕获**。spec：IDL checkedness 置
false **不改 default checkedness**（内容属性不动，HTML §4.10.5.4）；shim
为 render 序列化把属性一并移除属 R2998 注记的同款污染（「.checked= 写属性
供 render 故污染，缓存保护 default*」）——setter 臂（part05:501）先捕获后
写，sweep 臂漏捕获 → `_boolDefaultDirty` 未设 → `defaultChecked` getter
回落 live attr（已移除）恒 false → `form.reset()` / UA reset 的
`c.checked = c.defaultChecked` 恢复落空。

时序链：`set_checked_for_test(#pro)` → setter 值臂 sweep #basic → 净件
属性移除（无捕获）→ reset 读 `defaultChecked`=false → 不恢复。捕获点在
移除前，读数 = 移除前属性存在性 = 真默认（R2998 语义）。

## 施工（1 文件 +8，零 rust 零协议）

part05.js `_zwRadioUncheckGroupOthers` 净件臂：移除前补
`_captureBoolDefault(_gk131, 'checked', sel, handle)`（注释记 spec 依据与
两案实证）。后续 `setAttribute/removeAttribute/defaultChecked=` 重同步链
不变（part04:3738/4111 清脏回落 live attr）。脏件臂不动（`_checkedDirtyState`
=false，dirty 旗标保持——其捕获在自身首次 setter 写时已落）。s132 type
变更、clone、form owner 变更等全部 sweep 调用点走同一共享出口，单点修复
全量覆盖。

## 验证

- **renderer lib 358P/3F**（基线 354P/5F：+2 = 两目标案转绿；余 3F =
  在册 fetch 契约族，s177 stash 对照在案）。两案即本 bug 的常驻回归钉
  （红/绿证据：s177 基线日志红、本轮绿）。
- 探针三态修复后：P2 `true/false`（捕获保住）、P3 `true`（恢复成功）；
  探针验后删净（git status 仅 part05.js +8）。
- **engine lib 3625P/10F**（基线 3619+并行流新增 6 测；10F 与 s176 在册
  `/tmp/s176/engine_fails.txt` 逐键 diff SAME——fetch 契约族 9 +
  ua_display_li）。
- **forms 全树 4697 common 键 (key,state) join 零位移**（对 s176
  forms_final 4699 键；402F 同数）——LOST 2 =
  `focus-dynamic-type-change-on-blur` 两子测本轮未报（文件 79/80 行，
  已报全 Pass）＝环境截断族（s162 LOST 先例），非语义位移。
- **interactive-elements 483 键对 s176 终态逐键 bit-exact 零位移**。
- shim 装配序拼接（part01,01b,02-07,08a,08,08c,08b,08d）node --check
  SYNTAX_OK（113929 行）；rust 侧零 diff（fmt/clippy 无对象）。
- webview lib 免跑（crate 零 diff；user_actions 消费面未触及）。

## 归因与边界

- 失败时点未逐一 bisect（勘根以机制定谳而非引入提交）：sweep 净件移除
  属性 + defaultChecked 读链的组合在 8cb3e919a（2026-08 action
  transaction 系）后就位即红——本流 renderer lib 全量基线 s177 才首立，
  红从未被本流门禁看见过；修复以机制正确性 + 两案转绿 + 全域零位移承载，
  不追认中间轮次。
- OPTION selected 的 selectedness sweep（`_zwAskForReset`）不在本刀：其
  默认层语义 s135 已单独落（`_optSelAttrDefault` 捕获），无在册红案，
  不扩大。

## 下一步候选（本流）

renderer 基线余 3F（fetch 契约族——engine 同族 9F 的兄弟面，归因勘定或
根修）；forms 全树 402F 家族分析；interactive 44F 归因复读。

## 中间产物

/tmp/s178/：probe 输出（探针已删）、engine_lib.log、renderer_lib.log、
forms_s178.log、interactive_s178.log、forms_new/base.tsv、inter_new/base.tsv、
shim_concat.js。
