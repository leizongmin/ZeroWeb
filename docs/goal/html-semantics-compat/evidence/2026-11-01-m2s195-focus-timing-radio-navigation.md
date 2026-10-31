# M2-S195 focus 时序域 radio 焦点导航——8F 翻绿 + 1 文件 Timeout 解体，runner Actions 键盘链 4 面 + shim 1 臂

## 入场定刀位

s194 收口后在册次席候选：focus 时序域 12F（checkable-active-onblur 8F +
radio-focus-navigation 4 files 8 subtest F）。勘根拆分：radio 族为
runner Actions 键盘链 + shim radio 臂单根面，本刀收口；
checkable-active 8F 为 `:active` 伪类通道整体缺位（matcher 无臂、无
Document active 态、无指针/键盘钩子），独立根因留 s196 候选。

## 勘根

缺口四根（radio-focus-navigation-{checked,disabled,group-first-focus,
group-focusable-focus} 4 files 8F + radio-focused-and-scrolled-into-view
整文件 Timeout）：

1. **doc 级 qSA 选择器列表快照滞后面**（s195probe 同 tick 实证：本树
   walk 5 命中 / 列表 JSON 4 命中分歧）：doc 级 `querySelectorAll`
   复合清单（逗号 + 空白）走 `detHtml()` `_tree.innerHTML` 序列化 JSON
   往返（part03 R164/R165 只覆盖裸 tag/compound 单体走 live mutTree），
   `appendChild` 动态元素突变 apply 前不可见。`dispatch_tab_focus` 候选
   枚举用复合清单 → 动态创建的 radio 不在 Tab 候选面。
2. **Actions key 步 origin 动态焦点全臂 miss**：Actions `send()` 的 key
   步 origin = send 时点 `document.activeElement`（R3254-K2 先例），
   createElement 产物无 id/唯一 tag/attr，selectorFor 启发式（`__zwSelector`
   qSA 验证也走滞后快照、R145 forward map turn 末 apply 才 merge、
   nth-of-type 验证同滞后）全 miss → `__zw_td_selector` 恒空 → 整链
   "testdriver target has no stable selector" Unhandled rejection（
   radio-focus-navigation-checked Tab 落动态 radio 后续步全红）。
3. **键名/修饰键状态缺位**：keydown/keyup 事件 `key`/`code` 透传 PUA
   原字符（`'\u{E004}'`），页 `e.key === 'Tab'` 过滤恒 miss（UI Events
   应为 `"Tab"`）；无 Shift 粘滞位（focus-utils.js navigateFocusBackward
   = keyDown(kShift)+keyDown(kTab)+keyUp(kTab)+keyUp(kShift) 序，方向
   信息丢失）；Tab 默认动作落 InsertText PUA 字面——text control 被插
   脏值且无焦点迁移（Tab 非字符键，无 keypress；HTML §6.6.5 sequential
   focus navigation 默认动作）。
4. **shim radio 箭头键臂语义缺位**：part06 R3254-K5 箭头键 = 原地选中
   （焦点不动、input+change 派旧目标）——当时语料只断事件序可观察面；
   group-focusable-focus 新钉面断言 focus + checked 同步落新 radio
   （HTML §4.10.5.6 箭头键 = focusing steps → activation bandwidth，
   Chromium convention）。

## 施工（runner 1 文件 + shim 1 臂）

- **`tests/wpt-runner/src/testharness.rs`**：
  - `dispatch_tab_focus(webview, forward)`：候选枚举按 9 个单体选择器
    拆查再 `__zwHandle` 去重合并（同 tick 动态元素在面，保复合清单语义）
    + `compareDocumentPosition` 文档序排序；①radio 组代表剔除——同
    form owner（R4005 `el.form`）+ 同 name 只保留代表（checked 优先，
    否则首个可聚焦成员；HTML spec 未规定组代表，Chromium platform
    convention，WPT 钉面）；②Shift+Tab 反向导航（`fwd` 参数）；③焦点
    不在序中 → 文档序最近邻选择（旧 `next=seq[0]` 只对 body 起点等价，
    checked 动态切换代表后 Tab from 被剔成员会错落首元素）。
  - 降级键臂（`apply_testdriver_command` selector None 分流）：keydown/
    keyup 改派 `__zw_dispatch_key_at_active` live activeElement 通道
    （R4300 同款 generator，键名映射/Shift 粘滞同源）；默认动作只保留
    selector-free 的 Tab 焦点导航；需 selector 的臂（arrows 消费、Enter
    buttonish、InsertText）无目标跳过（FIXME）。非 key 操作维持原拒绝。
  - selector 臂键面：Tab/ArrowUp/ArrowDown 事件键名 UI Events 化；
    `td_shift_sticky` 页级粘滞位（修饰键 keydown 置位/keyup 清零，跨
    命令持续；Tab 步取方向 + 后续键事件 `shiftKey` 继承注入
    `DomEventDetail`）；Tab 默认动作 = `dispatch_tab_focus`（不再
    InsertText PUA）；ArrowUp/Down 经 `script_select_key_action` 消费
    判定（radio 组移动新语义消费返回 true；非 radio/SELECT 落既有
    InsertText 路径不动——SELECT type-ahead 与滚动键面本刀不扩）。
- **`crates/engine/src/js_dom_shim/part06.js`**：R3254-K5 radio 箭头键
  臂改焦点移动语义——先 focusing steps（`tgt2.focus()`：blur/focus/
  focusin 序 + activeElement 更新），activation 随后（checkedness 置新
  清旧），input+change 派到**新 checked** radio。FIXME：组枚举 qSA 不
  区分 form owner——跨 form 同名 radio 并组（spec = same form owner +
  same name；语料无跨 form 钉面）。

## 验证

- 定向 radio 族：checked 4/4 + disabled 1/1 + group-first-focus 1/1 +
  group-focusable-focus 2/2 全 Pass；radio-focused-and-scrolled-into-view
  整文件 Timeout 解体为单测 Fail（focus() 的 scroll-into-view 面未实现
  ——挂账新面，文件可完成即断言可观察）。
- forms 全树（stash 隔离基线 vs WIP 对照，同机同 runner）：4435P/265F/12T
  → **4443P/258F/11T**（ΔP=+8/ΔF=−7/ΔT=−1 精确闭合）。逐测试键 diff
  （(key,state) 对 sort -u 去重 + state-set join）恰 8 键 F→P（radio
  族 4 files）+ scrolled 文件 Timeout→Fail，其余全树 bit-exact。基线
  复现 s194 数字精确——上游 4 个 layout 提交（R4100 等）无 forms 位移，
  归因干净。
- interactive-elements 439P/44F **bit-exact**（在册同键；44F 逐文件
  分布 = dialog-element layout/top-layer/closedby 既有族，无键盘/焦点
  面位移）。
- engine lib 3642P/1F 在册同键（`test_ua_display_li_list_item_canvas_
  inline_s24_flip`，s166 起基线在集，style-system li UA 表跨流碰头候选）。
- clippy（`if` 折叠修正后全 workspace -D warnings 过）+ fmt 过；shim
  全序 node --check OK。clippy 修正是语义等价折叠，修正后定向 radio 族
  复跑全 Pass（REVERIFY_EXIT 见 `/tmp/s195/radio-target-reverify.log`）。

## 残留/挂账

- **checkable-active-onblur-with-click 6F + checkable-active-onblur 2F**：
  `:active` 伪类通道整体缺位（`matches_pseudo_class` `_ => false`、无
  Document active 态、无 pointerdown/up + 键盘 Space 钩子）→ s196 首席
  候选（镜像 :focus/:modal 先例）。
- **radio-focused-and-scrolled-into-view 1F**：焦点迁移的 scroll-into-
  view 面未实现（scrollTop 断言 got 0）。
- 跨 form 同名 radio 并组 FIXME（shim 臂）；Actions 链中途夭弃粘滞位
  残留 FIXME（runner）；降级臂 arrows/Enter skip FIXME（runner）。
- `dispatch_tab_focus` 候选去重 O(n²)（n = 可聚焦元素数，语料面无性能
  钉；真浏览器页面大 doc 面出现时再议）。

## 下一步候选

s196：`:active` 通道 8F（checkable-active-onblur 族，Document active
态 + 指针/键盘钩子 + matcher 臂 + 选择器面）→ disabled-003 crash/hang
2F 独立勘根；customizable select 177F 挂起待拍板（GB-20261031 征询在途）。
