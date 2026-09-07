# 键盘默认动作 — 运行时控制面板（master.md）

**入口文档**: [../keyboard-default-actions.md](../keyboard-default-actions.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（K3 残余切片 A+B+C——insertAdjacentHTML 同步子视图 + 隐式提交 default button 规则 + submitter 全链 + apply 消退补偿清除；implicit-submission 3F→**3/3 全 Pass**，keyboard 套件 15P→18P）

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
| K3 | implicit submission（Enter 提交规则）缺失 | ✅ M2 切片 2（Esc，bcd59d7ed）+ **残余切片 A+B（2026-09-07，本流落地）**：uE007→Submit 路由 + **隐式提交规则细化闭合**——① shim `insertAdjacentHTML` 同步子视图（R293/R304 先例：基底置空 + `_zwSelPendingParent` 挂槽 + parentNode 重指）；② sel→pending 身份归一（签名匹配 `_zwPendingParsedForSel` + apply-pending 父子序定位兄弟 sel proxy）；③ FORM named access（`form.text`/`form.submitButton`，spec dom-form-nameditem）；④ engine `default_submit_button_selector`（tree order 首个 submit button）+ `enclosing_form_selector` 升级 `unique_selector_for_node`（多 form 文档 tag 回落歧义修复）；⑤ PlannedEvent.submitter 全链（SubmitEvent.submitter 透传，R2984 通道接通）；⑥ disabled default button 隐式提交无动作（spec implicit-submission）。⑦ 切片 C——`__zw_apply_generation_bump` 定点清除 sel-less 解析补偿节点（apply 后 host 快照已含，桶内残留使融合视图基底+overlay 双计，副本无 sel 致 listener 键与派发键错位；handle 节点 identity 记账不受影响）。**implicit-submission.optional.html 3F→3/3 全 Pass**（套件 15P→18P）；engine 序列化保真由探针证伪（`test_serialization_form_roundtrip_r3254_k3_probe` 实证重放往返完全保真）。K3 全缺口闭合 ✅|
| K4 | 激活键语义（空格/Enter → click 合成 + 两键差异）缺失 | ✅ M2 切片 1（f6eaed4d5）+ **切片 2（581258c66）**：空格→button-ish 递归 Activate + **时序闭合**——`script_buttonish_probe` 目标分类（BUTTON / input type=button|submit|reset），runner send_keys 空格对 buttonish 目标 keydown 只派事件、**keyup 后**才激活（UI Events/Chromium：Enter=keydown、Space=keyup）；单测 send_keys_space_activates_button_on_keyup_r3254_k4 三组 + test_buttonish_probe 七 target 分类（engine part06）。Enter→Submit 臂（uE007 路由）既有 |
| K5 | select 键盘导航（展开/移动/type-ahead）缺失 | 🔶 M3 切片 1/2（051e594df+627d3ff20）+ **切片 3（581258c66）**：方向键/Home/End 选项移动 + radio 方向键组内移动 + **type-ahead 多字符缓冲**（`__zw_select_type_ahead`——500ms idle 缓冲、前缀匹配跳 disabled、不可匹配缓冲即弃、select.value 赋值 + input/change 事件序；单测 send_keys_select_type_ahead_multi_char_r3254_k5 确定性序列四断言）。**残余仅展开键语义**（headless 无展开态 UI——goal 明示 JS 面验收，defer 有据；上游 customizable keyboard-behavior 亦为 .optional）|

## 下一步计划

1. ~~**M1 切片 1**：WPT 键盘交互用例导入 + 基线~~ ✅ 2026-09-07（1P/4F + ENTER 映射修复，evidence/2026-09-07-m1-keyboard-baseline.md）
2. ~~**M1 切片 2**：失败聚类修复——testdriver Actions 键盘链~~ ✅ 2026-09-07（commit aad135b52；keydown/keyup 派发层 + keydown-input-events 驱动）
3. ~~**M1 切片 3**：默认动作分发层骨架——修饰键位透传~~ ✅ 2026-09-07（DomEventDetail 四 modifier 位字段 + __zw_dispatch_event init dict 透传 + runner send_keys 修饰键映射（uE008/uE009/uE00A/uE03D → 对应位 true 的 keydown/keyup 对）；modifier-keys.html 4F 全灭；单测 test_modifier_key_dispatch_r3254_k2_slice3 五组断言；keyboard 套件 6P/12F→**10P/4F**）。**残余 4F 根因定位**：implicit-submission 3F = js-dom 融合视图 insertAdjacentHTML 双计（跨域协调，K3 行）；paged.html 1F = runner 无真滚动管线（keyboard-page-scrolling 共享面，P1 记录）。**勘误**：本切片曾记「keypress-not-fired 随位透传转 Pass」——误记，该用例实际仍 Timeout（见切片 4）
4. ~~**M1 残余切片 4**：send_keys 普通字符键事件序~~ ✅ 2026-09-07（commit ece2282ec；上一轮 CONTINUE 悬案归因：**既非负载 flake 亦非 js_dom_shim 改动回归**——runner send_keys 普通字符只走裸 InsertText 无键事件，用例 3 个 promise_test 等 keyup 永不 resolve → 全 pending Timeout；「转 Pass」系切片 3 的 evidence 误记）。修复：keydown（cancelable）→未取消→ InsertText → keypress（Ctrl/Meta 抑制）→ keyup 全序 + send_keys 串内修饰键持久化（跨 send 清零；非字符键行为不变）；单测 send_keys_dispatches_key_event_sequence_and_suppresses_modifier_keypress 三组断言；keyboard 套件 10P/4F→**13P/4F**）。**残余 4F 均跨域在案**：implicit-submission 3F（js-dom 融合视图）+ paged 1F（真滚动管线）
5. ~~**K4 切片 2**：空格 keyup 激活时序~~ ✅ 2026-09-07（commit 581258c66；K4 残余 defer 解除——`script_buttonish_probe` 分类 + runner send_keys 空格 buttonish 路径延迟到 keyup 激活；单测三组 + engine probe 七 target 分类）
6. ~~**K5 切片 3**：select type-ahead~~ ✅ 2026-09-07（commit 581258c66；K5 残余中 type-ahead 解除——shim `__zw_select_type_ahead` 500ms 缓冲 + 前缀匹配 + 不可匹配即弃；本地 runner 单测标明本地——无上游强制用例，键盘映射 UA-dependent 与 customizable keyboard-behavior .optional 同因）。配套：fetch-keyboard-subset.sh 补 snap 三案 + helpers（可复现拉取）
7. **K3 残余切片 A+B**：insertAdjacentHTML 同步子视图 + 隐式提交 default button 规则 → ✅ 2026-09-07（本流 0 defer 面落地——上轮记录的「跨域 js-dom 共享面」实为本流域可闭环：js_dom_shim 近 14 天无其他流活跃编辑）。
   **切片 A（shim 同步视图）**：`insertAdjacentHTML` sel 路径三件补偿（part04）——①插入父站定址（beforebegin/afterend=父，afterbegin/beforeend=自身）；②`_zwChildBaseCache` 基底置空（R304 同款）；③解析顶层子挂 `_zwSelPendingParent` 槽 + parentNode 重指（R136 同款）。兄弟 getter `sel→pending` 身份归一（part03 `_zwPendingParsedForSel` 签名匹配 + apply-pending 父子序定位，返回与查询入口同族的 sel proxy）+ FORM named access（`form.text`/`form.submitButton`）。
   **切片 B（engine/webview/page-runtime）**：`default_submit_button_selector`（tree order DFS 直读节点属性——勘误：不经字符串 selector 重解析，无 id 控件 tag 回落恒命中首个）；`enclosing_form_selector` 升级 `unique_selector_for_node`（多 form 文档 tag 回落歧义）；`PlannedEvent.submitter` 字段 + `plan_submit`/`dispatch_planned_event` 全链透传（SubmitEvent.submitter，R2984 通道）；webview Submit 分支 default button 三规则（enabled→submitter=按钮；disabled→提交无动作返成功零 effects；无按钮→submitter=None）+ `script_control_disabled_probe`。
   **验证**：implicit-submission.optional.html **3F→2P/1F**（subtest 1 submitter 断言绿 + subtest 3 disabled 阻断绿）；keyboard 套件 **15P→16P**；单测 +4 组（`test_default_submit_button_selector_r3254_k3`、`implicit_submission_default_button_rules_r3254_k3`、`implicit_submission_inserted_form_r3254_k3`、`test_insert_adjacent_html_fusion_view_r3254_k3`）；engine 2646 / page-runtime 130 / webview 694 全绿；clippy 零警告。
8. **K3 残余切片 C**：apply 消退补偿清除 → ✅ 2026-09-07（初版根因假设「engine 序列化保真分歧」被探针证伪——`test_serialization_form_roundtrip_r3254_k3_probe` 实证 apply_mutations_to_html 重放往返对连续 form 插入完全保真；真实根因 = 同步视图补偿的 sel-less 解析节点在 host apply 后仍留 pending 桶 → 融合视图基底+overlay 双计（body children 出现空选择器副本 IFRAME[]/FORM[]/INPUT[]），populateForm 命中的 nextSibling 是副本、listener 键与派发键错位）。修复 = `__zw_apply_generation_bump` 定点清除 sel-less 槽节点（全局 `_zwPendingAdded` + id 索引 + 所属父桶 + 槽位；handle 节点 identity 记账不受影响）。**implicit-submission.optional.html 3/3 全 Pass**，keyboard 套件 15P→**18P**；单测 +1 组（serialize→re-parse 保真回归守卫）；engine 2647 全绿。

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + 分发层骨架 | ✅ 切片 1/2/3 + 残余切片 4 全部完成（2026-09-07）——基线 + keydown/keyup 派发层 + 修饰键位透传 + send_keys 键事件序（6P/12F→13P/4F，残余 4F 全为跨域根因） |
| M2 — 表单键与激活 | ✅ 切片 1 ✅（K4 空格激活）+ 切片 2 ✅（K3 Esc）+ 切片 3 ✅（K4 切片 2 空格 keyup 时序）+ **残余切片 A+B+C ✅（隐式提交规则闭合——implicit-submission 3/3 全 Pass，2026-09-07）** |
| M3 — select 导航 + radio/checkbox + 事件序 | 🔶 切片 1/2 ✅ + **切片 3 ✅（K5 type-ahead，581258c66）**；事件序断言（M1 基线全过）；残余仅展开键语义（headless 无展开态，defer 有据）|

## 验证基线

- 测试基线：立项时点全绿（13,192+）；clippy 零警告
- WPT 键盘默认动作面：M1 切片 2 后 **6P/12F（9 用例全可执行）** @ WPT_REV 315976933870（evidence 同日追加段）→ M1 切片 3 后 10P/4F → M1 残余切片 4 后 13P/4F → M2 切片 2（page-scrolling 流，3fa1a5f17）后 15P/10F/2T → **K3 残余切片 A+B+C 后 18P/7F/2T**——implicit-submission 3/3 全 Pass；残余 7F/2T 均为 snap 布局/scrollIntoView rect 跨域既有项（keyboard-page-scrolling/rendering 流协调点）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过

## Done Criteria 清点（2026-09-07 收尾状态）

| DC | 条目 | 状态 |
|---|---|---|
| DC-1 | 上游键盘用例导入 | ✅ 9 用例（uievents/keyboard 5 + implicit-submission + snap 三案），全可执行 |
| DC-1 | 通过率报告 + evidence | ✅ 1P/4F → 6P/12F → 10P/4F → 13P/4F → **15P/10F/2T**（M2 切片 2 后，Timeout 仅余跨域 2 案）；多轮 evidence 追加（含切片 4 勘误段）|
| DC-2 | Enter 提交 + implicit submission + validation 联动 | ✅ uE007→Submit 路由 + **隐式提交规则闭合**（K3 切片 A+B+C：default button 三规则 + submitter 全链 + disabled 阻断 + apply 消退补偿——**WPT implicit-submission 3/3 全 Pass**）；formnovalidate 联动既有 form-validation 管线 |
| DC-2 | 空格/Enter 激活（两键差异）+ preventDefault | ✅ 空格→button-ish 递归 Activate + **两键时序差异闭合**（Enter=keydown、Space=keyup，K4 切片 2，三组单测）；Enter→Submit 臂；keydown preventDefault 抑制（keydown-input-events 2P/0F + send_keys 通道同款）|
| DC-2 | Esc dialog cancel/close + select 展开态收起 | ✅ dialog cancel/close 三组断言；select 展开态——headless 无展开态 UI（goal 明示 JS 面验收），defer 记录 |
| DC-3 | select 导航（方向键/Home/End） | ✅ 跳 disabled/clamp/value 编程选中 + input+change 事件序；**type-ahead 多字符缓冲**（K5 切片 3——本地单测标明本地，无上游强制用例有据）|
| DC-3 | radio 方向键组内移动 | ✅ checked 迁移 + 跳 disabled + clamp；空格切换 checkbox=K4 Activate 管线（checkbox 臂既有）|
| DC-4 | 事件序（keydown→keypress→keyup/click + cancelable） | ✅ keydown-input-events 2P/0F（keydown→beforeinput→input 序 + preventDefault 抑制）；keypress 派发（composed 面 3P/0F）；修饰键位透传（M1 切片 3，modifier-keys 4F 全灭 + 五组单测）；**send_keys 普通字符 keydown→InsertText→keypress（Ctrl/Meta 抑制）→keyup 全序 + 串内修饰持久化**（残余切片 4，keypress-not-fired 3P + 三组单测）；跨 send 修饰状态清零语义有单测 |
| DC-5 | cargo test 全绿 / clippy / 资产化 | ✅ runner 207 + engine 2647 + page-runtime 130 + webview 694 全绿（2026-09-07 K3 切片 A+B+C 轮）；clippy 零警告 / 每切片带单测（本轮 +5 组）|

**收尾结论**：分发层骨架（K2 切片 2/3/4——send_keys 通道事件序补全）、激活键含两键
时序差异（K4 切片 1/2——Space=keyup、Enter=keydown 全语义）、Esc（K3 Esc 面）、
select/radio 导航 + type-ahead（K5 切片 1/2/3）、**隐式提交规则全语义**（K3 残余切片
A+B+C——default button 三规则 + submitter 全链 + insertAdjacentHTML 同步子视图 + apply
消退补偿清除）全部落地并有断言资产；**implicit-submission.optional.html 3/3 全 Pass**
（上轮「跨域 js-dom 共享面」归因实为本流域可闭环面）。**唯一 defer = select 展开键语义**
（headless 无展开态 UI——goal 明示 JS 面验收）；残余套件 Fail/Timeout 均为 snap 布局/
scrollIntoView rect 跨域既有项（keyboard.html snap 断言 6F + paged 1F = snap 布局域；
paged/scroll-padding 2T = renderer S3 几何，协调点记录）。不阻塞流域收口判定。
