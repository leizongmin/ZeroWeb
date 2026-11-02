# M2 s230 — keyboard 簇 3F 勘根分型 → radio ArrowRight 双根修复（runner 派发缺口 + FCC 身份反查）

日期：2026-11-02。切片号 s230（承 s229）。

## 入场定刀位

s229 下一步候选：forms misc 散面续勘（table/label 簇 3F 或 keyboard 3F）。
table/label 簇 3F（form.html 2F + form_owner_and_table 1F）勘根即发现全部落在
用户门禁 GB-20261031「form owner table 4F 解析器 form element pointer 基建面」
范围内（控制文档 222-231 行，挂起待拍板）——车道碰撞声明，不入侵。转
keyboard 簇 3F，数据源复用 s228-forms-baseline.log。

## keyboard 簇分型（3F 三根，互不同根）

- **date-readonly-keyboard**：`Unhandled rejection "Error: action was not
  applicable: Some(NotApplicable)"`——runner send_keys 对 date input 类型门
  直接拒绝而非派键。runner/testdriver 面，独立根，未动。
- **radio-focused-and-scrolled-into-view**：`scrollTop=0 expected >500`——
  focus 步骤缺 scroll into view。focus 管线/布局面，独立根，未动。
- **radio-keyboard-navigation-order**（本轮定刀）：ArrowRight 组内导航 +
  form.elements 身份，正中 part06 `__zw_select_key_action` 在册 FIXME 邻域。

## 勘根（探针 6 轮 + Rust 插桩 3 轮）

测试结构：6 组 radio（inside/before/after/mix-form/mix/doc），脚本动态
`createElement` + `form=` 属性 + `firstInGroup.after()` 插入 `XNew`，然后
`document.forms[formId].elements[0].focus()` 后 for-of 遍历
`document.forms[formId].elements`，每步断言 `radio === document.activeElement`
再 `pressRight()`（testdriver Actions `keyDown("")`）。

- **探针1/2（集合成员复刻）**：form= 动态控件完全不进 elements 集合
  （host 列表 stale 缺席 + walk 臂 form= 属性跳过——两路都不收）；focus 落点
  正确。与真实测试失败形态（expected inside1 got insideNew）对不上。
- **探针3/4（逐迭代追踪）**：失败形态随插桩扰动翻转（s229 同款）——
  真实测试 it0 失败（active 已是 insideNew）、插桩版 it1 失败
  （radio=insideNew active=inside1——pressRight 空转）。
- **Rust 插桩①（`__zw_form_controls` 调用序）**：host 列表前 3 次调用 stale
  （3 项）、后 2 次刷新为正确文档序（insideNew 第 2 位）——host 顺序从不是
  insideNew 在前，集合顺序故事排除。
- **Rust 插桩②（`__zw_focus_changed`）**：全程唯一焦点注册 `#inside1`——
  导航焦点从未经 retained 状态同步。
- **探针5（keydown/focusin 追踪）**：`kd->inside1` 键事件到达焦点元素，
  但无任何后续 focus 事件——导航链断。
- **Rust 插桩③（shim 臂内打点，借道只读回调）**：`__zw_select_key_action`
  **零调用**——按键默认动作分发根本不进 shim 臂。
- **定谳①（runner 派发缺口）**：testharness.rs Actions 键盘链
  `matches!(key, "\u{E013}" | "\u{E015}")` 只映射 ArrowUp/Down——
  `\u{E014}`(Right)/`\u{E012}`(Left) 缺席，ArrowRight 落空。shim 臂四向
  齐备（part06 `dir2 = ArrowDown||Right ? 1 : ArrowUp||Left ? -1`）。
- **修复①后新形态**：导航生效，失败点后移至 `expected insideNew but got
  insideNew`——同节点不同 wrapper 对象。
- **探针6（wrapper 身份签名）**：elements 产出 `sel=#insideNew|h=null`
  （selector wrapper）、activeElement 产出 `sel=null|h=__n0`（handle
  wrapper）——双族断裂。
- **定谳②（FCC 身份反查缺席）**：qSA 产物走 `_zwQueryWrapIdentity`
  （R331：命中 handle 节点返回原 handle proxy——canonical 身份），而
  `_formControls` host 条目裸 `_wrapSelector` 造新 selector proxy。真实
  浏览器 `form.elements[k] === newInput`（JS 持有引用）必须成立——错在
  FCC 侧。

## 施工（±25 行，两文件）

- `tests/wpt-runner/src/testharness.rs`：Actions 键盘链补
  `\u{E012}`(ArrowLeft)/`\u{E014}`(ArrowRight) 映射（match 四向），注释记录
  s230 勘根证据（shim 臂四向齐备而本链只派 Up/Down）。
- `crates/engine/src/js_dom_shim/part05.js`：`_formControls` host 条目
  `.map(_wrapSelector)` → `.map(_zwQueryWrapIdentity)`（R331 身份反查，
  与 document QSA 面同款；静态控件无 handle 注册回落原路径零变化；满足
  R100 教训 #7「query 返回点」限定——集合重建即查询返回点）。

## 验证

- solo：radio-keyboard-navigation-order Fail → **Pass**（双修复缺一不可——
  只修 runner 键位到 expected/got 同节点不同对象的身份断言）。
- 域 A/B：全树 (key,state) join（对比 s229-ab.log 基线，8098 键）——
  lost/gained 各恰 1 行 = `radio-keyboard-navigation-order :: Radio button
  keyboard navigation should proceed in tree-order.` Fail→Pass，**零其他
  状态翻转**。全树 7502P/518F/81T（日志 s230-ab.log）。
- clippy `-p zero-wpt-runner --all-targets -D warnings` 干净；fmt 干净；
  part06.js 插桩验后撤净（git diff 零残留）。
- 探针 6 个验后全删；Rust 插桩 3 处验后全撤。

## 承袭挂账（更新）

- keyboard 簇余下 2F 各自独立根未动：date-readonly-keyboard（runner
  send_keys 类型门 NotApplicable 拒绝——runner/testdriver 面）、
  radio-focused-and-scrolled-into-view（focus 缺 scroll into view——
  focus/布局面）。
- part06 `__zw_select_key_action` 在册 FIXME（组枚举 qSA 不区分 form
  owner）保持：本测试各组 name 全仓唯一，qSA 文档序枚举恰好等价 form-owner
  组——钉面仍未出现，FIXME 注记不动。
- 其余承袭不动（parent-label 布局零盒 2F；following-label Actions flake
  2F；table/label 簇 3F 用户门禁；散面余项等）。

## 方法论教训

- 「键事件到达但默认动作不生效」先查 runner 派发面的键位覆盖面与 shim
  消费面的键位覆盖面是否成对——本例 shim 四向、runner 两向，箭头族按
  E012-E015 连号核对即可一眼定位。
- 同节点双 wrapper（sel 族 vs handle 族）身份断裂的判别探针：dump
  `__zwSelector`/`__zwHandle` 双签名——修复方向选「错的那一侧」（qSA 已
  canonical 而 FCC 裸 wrap，则修 FCC 对齐 R331），不扩大到全局 `_wrapSelector`。
