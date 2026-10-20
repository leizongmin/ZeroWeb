# M2-S11 focus-reset Tab 焦点导航 + intercept() 显式覆写语义（2026-10-20）

S4E 挂账「Tab 键焦点导航 2T」收口——两独立根因同轮（runner 宿主接线 + shim
intercept options 覆写门），focus-reset 域 basic/multiple-intercept 两案 Timeout
全数翻绿；autofocus load 期 14 NotRun 定性入账（S12 候选）。

## 根因一：runner TAB 焦点迁移接线（testharness.rs）

send 命令 TAB（WebDriver uE004）此前映射 host `MoveFocus { forward: true }` 默认动作 →
action plan 只产出 `PageEffect::Focus(next)` 效果（page-runtime html_actions.rs
`plan_focus`），真浏览器宿主由 renderer `apply_webview_action_effects` 消费——
**runner 宿主丢弃 effects**，页面可见焦点不变：focus-reset helpers 的
`waitForFocus`（target focus listener 门铃）永挂，basic/multiple-intercept 两案
case Timeout（S4E~S10 连续挂账「DOM 焦点域」实为 runner 宿主形态缺口）。

**修**：TAB 默认动作改走 `dispatch_tab_focus`——页面原生 focusing steps：

- 候选集 = 标准可聚焦面（`a[href],area[href],button,input,select,textarea,
  summary,[tabindex]`），排除 disabled / tabindex=-1 / input hidden；
- tab 序 = 正 tabindex 升序在前 + 零/缺省按文档序（HTML §6.5.3 sequential
  focus navigation）；
- 起点 = 页面真实 `document.activeElement`（querySelector 同源 `_makeProxy`
  缓存 identity 相等，`__zwHandle` 兜底双判）；
- 迁移 = `next.focus()`（shim el.focus() 连接臂自带 blur/focusout → focus/
  focusin + `_activeElKey` 同步——与既有 `_zwUnfocusIfFocused` handle 形键
  同键体系；host 侧 `_zwFocusSel` 写 sel 形键与 remove 钩子恒 miss，页面原生
  focus 是唯一同键入口）；
- 无下一可聚焦 → 焦点不变（真浏览器 Tab 落浏览器 UI，headless 近似驻留）。

同键 keydown 不再拼首键 `__zw_host_focus` 自聚焦前缀——焦点导航键的迁移起点是
页面当前焦点，send 目标自聚焦会把起点抢到 send 目标（起点错位）。keydown/keyup
事件序维持（非字符键取消只抑制字符插入类序的既门）。

## 根因二：intercept() scroll/focusReset 显式覆写门（part05.js）

旧版每次 `intercept()` 无条件覆写 `_zwScrollMode`/`_zwFocusReset` 缺省值；spec
intercept steps「If options["scroll"] exists, then set event's scroll behavior」
只在 member **存在**时覆写（navigateevent-intercept）。multiple-intercept 的
「manual + (not provided)」「(not provided) + manual」双序案面要求先前 'manual'
保留、后续无参 intercept() 不回落缺省。**修**：两 member 加 has 门（undefined/
null 同门跳过——与上方枚举校验既门一致）；消费面缺省同型不变（part02
`_zwFocusReset === 'manual'` 判定 / `_zwScrollMode || 'after-transition'`）。

## 波及面封口

`send_keys` TAB 全语料仅两处：focus-reset resources/helpers.mjs（本轮销项）+
iframe sandbox helper（M3 门控未跑）；css 域 scroll-marker/focus-visible 等 TAB
案不在本地 runner registry。首键自聚焦前缀抑制仅 TAB 键生效，编辑键/Enter 语义
不变（uievents 既有绿面不触碰）。

## 验证

- 隔离：focus-reset 域全绿——basic 8/8、multiple-intercept 6/6、change-focus-*
  ×4、focus-reset-timing ×2、reentry-from-focus-reset ×2 全 Pass。
- 全量：`make testharness-navigation` **423P/34F/24T/14NR**（423/476 =
  **88.9%**），对 [S10 账本](2026-10-20-m2-s10-location-port.txt) per-subtest
  语义 diff：+13 Pass（basic 7 子测试 + multiple-intercept 6 子测试）、−2 case
  Timeout 行（恰两案）、Fail/NotRun **逐行恒等零回归**。make exit 1 = 既有慢性
  面签名（sandbox 双案 `open()`→null，2026-10-20 组成态巡检同记）。
- 域计数：navigation-api **239→252**（账本分母 255 口径）；域内残差 11F + 3T +
  14NR 逐行与 S10 恒等。autofocus.html 14 NotRun 定性：`promise_setup` 首断言
  「load 期 `<button autofocus>` 已聚焦」失败（引擎 load 期 autofocus 处理全缺
  ——DOM 焦点域新切片候选，S12）。
- `cargo fmt --all -- --check` 干净；`cargo clippy --workspace --all-targets
  -D warnings` 零 warning；`make test`（DISPLAY=:20，剪贴板前置需 X）完整跑完
  **76 suites / 22,186P / 2F / 199I**——唯二 F 即下节跨流陈旧 GPU 断言（gate
  if 复合命令吞其退出码，make 按成功收尾；除此外全绿，含 runner 单测与
  js_dom_bridge 既有钉面）。reftest 未跑：变更面（navigation API shim 语义 +
  runner TAB 接线）零布局/绘制触达，S9 复核 708 比较 0 不一致账本维持。

## 跨流发现（记账，不越界）

`make test` 中 render-foundation `gpu_full_scene_filter_grayscale/saturate`
两案灰度断言 r=54 出期望窗 60..=100：R5194（`8e7ba040f`，2026-10-19）GPU
saturate/grayscale 矩阵改 709 派生权重（0.213/0.715/0.072，54 = 0.2126×255
恰为 709 luma），两条全场景测试断言未随更新（仍按 Rec601 60..=100）。归因
construction 级（render-foundation 测试二进制不链接 engine/runner，与本流
diff 无涉）；该套件在 adapter 探测失败环境整段跳过、故此前未暴露。rendering
域记账项，不由本流修复。

原始输出：[2026-10-20-m2-s11-focus-reset-tab.txt](2026-10-20-m2-s11-focus-reset-tab.txt)。
