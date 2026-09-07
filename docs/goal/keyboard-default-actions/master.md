# 键盘默认动作 — 运行时控制面板（master.md）

**入口文档**: [../keyboard-default-actions.md](../keyboard-default-actions.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（M1 残余切片 4——send_keys 普通字符键事件序 + 修饰键串内持久化，keyboard 套件 10P/4F→**13P/4F**；勘误 keypress-not-fired 误记 Pass；全量 make test 18,980 全绿复核）

---

## 当前状态

**专项定位**：键盘/编辑方向三拆之一（form-validation 拆分时用户已点名的「其次键盘默认
动作」）。HTML 控件键盘默认动作（Enter 提交/空格激活/Esc/select 导航）补齐，WPT 真实
用例驱动。与 form-validation（提交校验管线）天然衔接。

**与兄弟 goal 的边界**：
- editing-contenteditable — 编辑宿主内的键（键入/删除/换行）归其管；本目标管非编辑宿主
  默认动作。分发顺序：编辑宿主优先消费
- keyboard-page-scrolling — 滚动键归其管
- form-validation — Enter 提交走其 interactive validation 管线（不重建，衔接）
- js-dom — 碰 js_dom_shim 事件段前先 `git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ FocusManager：Tab 导航 + tabindex 排序 + 13 单测
- ✅ html_actions submit 路径 + form-validation requestSubmit 阻断
- ✅ dialog 状态机（R3290：show/showModal/close + open 反射）
- ⚠️ keydown 默认动作分发层（按控件类型 + 按键）无系统性实现
- ⚠️ implicit submission / 激活键（空格 vs Enter）/ select 键盘导航缺失
- ⚠️ WPT 键盘默认动作上游用例覆盖为零

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| K1 | WPT 用例覆盖为零 | ✅ 首批 6 用例 2026-09-07（uievents/keyboard 5 + implicit-submission；基线 1P/4F，evidence/2026-09-07-m1-keyboard-baseline.md）|
| K2 | 默认动作分发层缺失 | ✅ M1 切片 2/3 + 残余切片 4（2026-09-07）：keydown/keyup 事件派发层（Actions 键盘链）+ 修饰键位透传（DomEventDetail shift/ctrl/alt/meta 四位 → script_gen detail JSON → __zw_dispatch_event KeyboardEvent init dict → event.*Key 可读——modifier-keys 4F 全灭）+ send_keys 普通字符 keydown→InsertText→keypress→keyup 事件序 + Ctrl/Meta keypress 抑制 + 串内修饰键持久化（keypress-not-fired 3 Timeout 全灭，commit ece2282ec）。分发表现有臂：Tab/printable/Backspace/Enter/Esc/SELECT+radio 方向键/修饰键事件对 |
| K3 | implicit submission（Enter 提交规则）缺失 | 🔶 部分：uE007→Submit 路由 + 表单管线已接（M1）；Esc dialog cancel/close 已接（M2 切片 2，bcd59d7ed）；隐式提交规则细化待 js-dom 视图断链修复后复评。**残余聚类根因（2026-09-07 切片 3 探针定位）**：implicit-submission.optional.html 3F 全为 `populateForm` 前置（insertAdjacentHTML('afterbegin') 后 iframe.nextSibling=null——js-dom 融合视图 pending/host 双计 + body childNodes stale 空），非隐式提交实现缺陷（跨域 js-dom 共享面，协调点记录）|
| K4 | 激活键语义（空格/Enter → click 合成 + 两键差异）缺失 | 🔶 M2 切片 1（2026-09-07，f6eaed4d5）：空格→button-ish 目标递归 Activate（click 合成全管线复用）；Enter→Submit 臂上轮已接（uE007 路由 + 表单提交）。残余：keydown/keyup 两键时序差异（Space=keyup 触发、Enter=keydown 触发——runner 单发通道下语义合并，Actions 键盘链可细分，defer 记录）|
| K5 | select 键盘导航（展开/移动/type-ahead）缺失 | 🔶 M3 切片 1/2（2026-09-07，051e594df+627d3ff20）：select 方向键/Home/End 选项移动 + radio 方向键组内移动均接通（跳 disabled/clamp/input+change 事件序）；残余：type-ahead 多字符缓冲、展开键语义（headless 无展开态——goal 明示 JS 面验收，defer 记录）|

## 下一步计划

1. ~~**M1 切片 1**：WPT 键盘交互用例导入 + 基线~~ ✅ 2026-09-07（1P/4F + ENTER 映射修复，evidence/2026-09-07-m1-keyboard-baseline.md）
2. ~~**M1 切片 2**：失败聚类修复——testdriver Actions 键盘链~~ ✅ 2026-09-07（commit aad135b52；keydown/keyup 派发层 + keydown-input-events 驱动）
3. ~~**M1 切片 3**：默认动作分发层骨架——修饰键位透传~~ ✅ 2026-09-07（DomEventDetail 四 modifier 位字段 + __zw_dispatch_event init dict 透传 + runner send_keys 修饰键映射（uE008/uE009/uE00A/uE03D → 对应位 true 的 keydown/keyup 对）；modifier-keys.html 4F 全灭；单测 test_modifier_key_dispatch_r3254_k2_slice3 五组断言；keyboard 套件 6P/12F→**10P/4F**）。**残余 4F 根因定位**：implicit-submission 3F = js-dom 融合视图 insertAdjacentHTML 双计（跨域协调，K3 行）；paged.html 1F = runner 无真滚动管线（keyboard-page-scrolling 共享面，P1 记录）。**勘误**：本切片曾记「keypress-not-fired 随位透传转 Pass」——误记，该用例实际仍 Timeout（见切片 4）
4. ~~**M1 残余切片 4**：send_keys 普通字符键事件序~~ ✅ 2026-09-07（commit ece2282ec；上一轮 CONTINUE 悬案归因：**既非负载 flake 亦非 js_dom_shim 改动回归**——runner send_keys 普通字符只走裸 InsertText 无键事件，用例 3 个 promise_test 等 keyup 永不 resolve → 全 pending Timeout；「转 Pass」系切片 3 的 evidence 误记）。修复：keydown（cancelable）→未取消→ InsertText → keypress（Ctrl/Meta 抑制）→ keyup 全序 + send_keys 串内修饰键持久化（跨 send 清零；非字符键行为不变）；单测 send_keys_dispatches_key_event_sequence_and_suppresses_modifier_keypress 三组断言；keyboard 套件 10P/4F→**13P/4F**）。**残余 4F 均跨域在案**：implicit-submission 3F（js-dom 融合视图）+ paged 1F（真滚动管线）

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + 分发层骨架 | ✅ 切片 1/2/3 + 残余切片 4 全部完成（2026-09-07）——基线 + keydown/keyup 派发层 + 修饰键位透传 + send_keys 键事件序（6P/12F→13P/4F，残余 4F 全为跨域根因） |
| M2 — 表单键与激活 | 🔶 切片 1 ✅（K4 空格激活）+ 切片 2 ✅（K3 Esc dialog cancel/close）；隐式提交规则细化跨域待复评 |
| M3 — select 导航 + radio/checkbox + 事件序 | 🔶 切片 1/2 ✅（K5 select + radio 方向键，2026-09-07）；事件序断言（M1 基线 keydown-input-events 已全过）；type-ahead defer |

## 验证基线

- 测试基线：立项时点全绿（13,192+）；clippy 零警告
- WPT 键盘默认动作面：M1 切片 2 后 **6P/12F（9 用例全可执行）** @ WPT_REV 315976933870（evidence 同日追加段）→ M1 切片 3 后 10P/4F → M1 残余切片 4 后 13P/4F → **M2 切片 2（page-scrolling 流，3fa1a5f17）后 15P/10F/2T**——keyboard.html 8/8 可完成（2P/6F，snap 断言差异=snap 布局跨域缺口）；残余 implicit-submission 3F（js-dom 融合视图）+ paged/scroll-padding 2T（scrollIntoView 无 rect，renderer S3 跨域），K3/P1 行记录
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过

## Done Criteria 清点（2026-09-07 收尾状态）

| DC | 条目 | 状态 |
|---|---|---|
| DC-1 | 上游键盘用例导入 | ✅ 9 用例（uievents/keyboard 5 + implicit-submission + snap 三案），全可执行 |
| DC-1 | 通过率报告 + evidence | ✅ 1P/4F → 6P/12F → 10P/4F → 13P/4F → **15P/10F/2T**（M2 切片 2 后，Timeout 仅余跨域 2 案）；多轮 evidence 追加（含切片 4 勘误段）|
| DC-2 | Enter 提交 + implicit submission + validation 联动 | 🔶 uE007→Submit 路由 + 表单管线已接；规则细化（单/多控件、formnovalidate）待 js-dom 视图断链修复后复评（跨域）|
| DC-2 | 空格/Enter 激活（两键差异）+ preventDefault | ✅ 空格→button-ish 递归 Activate（click 合成全管线）；Enter→Submit 臂；keydown preventDefault 抑制（keydown-input-events 2P/0F + send_keys 通道同款）|
| DC-2 | Esc dialog cancel/close + select 展开态收起 | ✅ dialog cancel/close 三组断言；select 展开态——headless 无展开态 UI（goal 明示 JS 面验收），defer 记录 |
| DC-3 | select 导航（方向键/Home/End） | ✅ 跳 disabled/clamp/value 编程选中 + input+change 事件序 |
| DC-3 | radio 方向键组内移动 | ✅ checked 迁移 + 跳 disabled + clamp；空格切换 checkbox=K4 Activate 管线（checkbox 臂既有）|
| DC-4 | 事件序（keydown→keypress→keyup/click + cancelable） | ✅ keydown-input-events 2P/0F（keydown→beforeinput→input 序 + preventDefault 抑制）；keypress 派发（composed 面 3P/0F）；修饰键位透传（M1 切片 3，modifier-keys 4F 全灭 + 五组单测）；**send_keys 普通字符 keydown→InsertText→keypress（Ctrl/Meta 抑制）→keyup 全序 + 串内修饰持久化**（残余切片 4，keypress-not-fired 3P + 三组单测）；跨 send 修饰状态清零语义有单测 |
| DC-5 | cargo test 全绿 / clippy / 资产化 | ✅ 全量 make test 18,980 全绿（2026-09-07 切片 4 轮，v8+quickjs 双 phase）/ 零警告 / 每切片带单测（M1 残余切片 4 +1）|

**收尾结论**：分发层骨架（K2 切片 2/3/4——send_keys 通道事件序补全）、激活键（K4）、
Esc（K3 Esc 面）、select/radio 导航（K5）全部落地并有断言资产；隐式提交规则细化与
type-ahead/展开键为跨域/headless 限制项——残余 4F 均有精确根因（implicit-submission
3F = js-dom 融合视图双计，paged 1F = 真滚动管线，协调点记录）。不阻塞流域收口判定。
