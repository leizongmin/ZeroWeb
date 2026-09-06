# 键盘默认动作 — 运行时控制面板（master.md）

**入口文档**: [../keyboard-default-actions.md](../keyboard-default-actions.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（M2 切片 1 完成——空格激活 button（K4），webview Activate 递归复用）

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
| K2 | 默认动作分发层缺失 | 🔶 部分接通：runner send_keys uE007→Submit + webview CE-first Enter 分发（2026-09-07）；keydown/keyup 事件派发断言面待 Actions 键盘链（addKeyboard not a function）|
| K3 | implicit submission（Enter 提交规则）缺失 | ⬜ M2 |
| K4 | 激活键语义（空格/Enter → click 合成 + 两键差异）缺失 | 🔶 M2 切片 1（2026-09-07，f6eaed4d5）：空格→button-ish 目标递归 Activate（click 合成全管线复用）；Enter→Submit 臂上轮已接（uE007 路由 + 表单提交）。残余：keydown/keyup 两键时序差异（Space=keyup 触发、Enter=keydown 触发——runner 单发通道下语义合并，Actions 键盘链可细分，defer 记录）|
| K5 | select 键盘导航（展开/移动/type-ahead）缺失 | ⬜ M3 |

## 下一步计划

1. ~~**M1 切片 1**：WPT 键盘交互用例导入 + 基线~~ ✅ 2026-09-07（1P/4F + ENTER 映射修复，evidence/2026-09-07-m1-keyboard-baseline.md）
2. **M1 切片 2**：失败聚类修复——testdriver Actions 键盘链（addKeyboard/addKey…/send → keydown/keyup 派发，keydown-input-events 驱动）
3. **M1 切片 3**：默认动作分发层骨架（keydown 派发进 runner send_keys 路径——现仅 action 语义无事件序）

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + 分发层骨架 | ✅ 切片 1/2 完成（2026-09-07）——基线 6P/12F 全案可执行 + keydown/keyup 派发层；切片 3 分发表扩展按残余聚类推进 |
| M2 — 表单键与激活 | 🔶 切片 1 ✅（K4 空格激活，2026-09-07）；K3 implicit submission 规则细化待 js-dom 视图断链修复后复评 |
| M3 — select 导航 + radio/checkbox + 事件序 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿（13,192+）；clippy 零警告
- WPT 键盘默认动作面：M1 切片 2 后 **6P/12F（9 用例全可执行）** @ WPT_REV 315976933870（evidence 同日追加段）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
