# UI/指针事件兼容 — 运行时控制面板（master.md）

**入口文档**: [../uievents-compat.md](../uievents-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-08（goal Done 维持；待决策清单 4 项首次征询登记）

## 当前状态

**M3 + 尾簇 1/2/4/5/6/7/8/9/10/11/12/13/14/15/16/17/18/19 落地（2026-10-05）**：基线 230P → M2 片 1 343P →
M3 375P → 尾簇 1 451P → 尾簇 2 457P → 尾簇 4 492P → 尾簇 5 502P → 尾簇 6a 1107P →
尾簇 6b+6c 1115P → 尾簇 7 1167P → 尾簇 8 1202P → 尾簇 9 1218P → 尾簇 10 1570P →
尾簇 11 1583P → 尾簇 12 1588P → 尾簇 13 1590P → 尾簇 14 1594P → 尾簇 15 1609P →
尾簇 16 1610P → 尾簇 17 1616P → 尾簇 18 1620P → 尾簇 19 1736P → 尾簇 20/21 1738P →
尾簇 22 1739P → 尾簇 23 1745P → 尾簇 24 1750P → 尾簇 25 1755P → 尾簇 27 1757P →
尾簇 28 1759P → 尾簇 30 1763P → 尾簇 31 1778P → 尾簇 32 1782P → 尾簇 33 1785P →
尾簇 34 1787P → 尾簇 35 1790P → 尾簇 37 1790P/174F → 尾簇 38 1790P/174F →
尾簇 39 1791P/173F → 尾簇 41 **1794P/170F（+1575 累计）**
（corpus：**1794P/170F/96TO**，`TIME_LIMIT=3600`——尾簇 39：utils.js 资产补拉
解锁三 page-threw 案 + 解锁面三连修（constructor 身份/composed/cancelable），
`uievents/mouse/attributes.html` file-Fail→**named Pass**，逐条 diff 仅 3 文件
翻转零涟漪——[evidence/2026-10-07-m3-tail39.md](evidence/2026-10-07-m3-tail39.md)；
尾簇 37：layer-coords-transform **1F→1P**（layerX/layerY 反射，shim + native
双路径；总册漂移已归因——
[evidence/2026-10-07-m3-tail37.md](evidence/2026-10-07-m3-tail37.md)）；
尾簇 38：click/dblclick 携指针坐标（corpus 恒等零涟漪——
[evidence/2026-10-07-m3-tail38.md](evidence/2026-10-07-m3-tail38.md)）；
**wheel 族 3 案全绿**（尾簇 35——Actions scroll 步接通 + WheelEvent 派发）；textInput 族 24P 全绿
（上轮 22P/1F/1TO：api CE execCommand 案 Pass + enter-textarea CE 案 Pass）；
其余零回归。尾簇 33 的 textContent 同步可见性挂账解明并修复：根因 = `'value' in`
has 白名单无条件 true（div 等 CE 元素误入 value getter 分支）+ CE Enter 元素
caret no-op + innerHTML mutation 异步可见性。尾簇 36（Δ0 归因轮）：
mousemove-between 的序断言缺口腔定位——**body 级 margin:auto 水平居中失效**
（探针实证 a 盒 [8,208] vs 真实浏览器居中 [300,500]，absolute 坐标步全错位——
渲染流域 layout-engine 面，跨流挂账）；mouse 族余挂 layerX/layerY 反射
（transform 感知计算——shim 面可做，2F 收益）。**reftest 复核**：尾簇 31-35 的
shim/Rust 大改后 make reftest **704/704（failed 0）零回归**（DC-4 reftest 项
尾簇 28 后首次复核）。尾簇 19：improvements=120 / regressions=1 已记录；尾簇 20：+2；
尾簇 21：Δ0 基建轮；尾簇 22：portal 首段 +1；尾簇 23：portal 第二段 +6；尾簇 24：
portal 第三段 +5；尾簇 25：触式捕获路由 +5；尾簇 26：Δ0 id 值面；尾簇 27：
frame-hold 重接 +2；尾簇 28：frame-hold 残留清零 +2；尾簇 29：判例轮 Δ0（?touch
subtest 4 上游即 Fail）；尾簇 30：Meta 转义三认 + 跨界序 buttons 活掩码 +4 +
textInput 资产补齐（7 案 page-threw 转具名）；尾簇 31：**TextEvent 语义域 +
selection key 存活面 +14**（见下节与
[evidence/2026-10-06-m3-tail31.md](evidence/2026-10-06-m3-tail31.md)）、
[evidence/2026-10-06-m3-tail30.md](evidence/2026-10-06-m3-tail30.md)、
[evidence/2026-10-06-m3-tail29.md](evidence/2026-10-06-m3-tail29.md)、
[evidence/2026-10-06-m3-tail28.md](evidence/2026-10-06-m3-tail28.md)。
证据：[evidence/2026-10-05-m3-tail13.json](evidence/2026-10-05-m3-tail13.json)（尾簇 13 后）、
[evidence/2026-10-05-m3-tail13.md](evidence/2026-10-05-m3-tail13.md)（根因链 + 排除路径）、
[evidence/2026-10-05-m3-tail12.json](evidence/2026-10-05-m3-tail12.json)（尾簇 12 后）、
[evidence/2026-10-05-m3-tail12.md](evidence/2026-10-05-m3-tail12.md)（根因链 + 排除路径）、
[evidence/2026-10-05-m3-tail11.json](evidence/2026-10-05-m3-tail11.json)（尾簇 11 后）、
[evidence/2026-10-05-m3-tail11.md](evidence/2026-10-05-m3-tail11.md)（根因链 + 排除路径）、
[evidence/2026-10-05-m3-tail10.json](evidence/2026-10-05-m3-tail10.json)（尾簇 10 后）、
[evidence/2026-10-05-m3-tail10.md](evidence/2026-10-05-m3-tail10.md)（根因链 + 排除路径）、
[evidence/2026-10-04-m3-tail9.json](evidence/2026-10-04-m3-tail9.json)（尾簇 9 后）、
[evidence/2026-10-04-m3-tail9.md](evidence/2026-10-04-m3-tail9.md)（根因链 + 排除路径）、
[evidence/2026-10-04-m3-tail8.json](evidence/2026-10-04-m3-tail8.json)（尾簇 8 后）、
[evidence/2026-10-04-m3-tail8.md](evidence/2026-10-04-m3-tail8.md)（根因链 + 排除路径）、
[evidence/2026-10-04-m3-tail7.json](evidence/2026-10-04-m3-tail7.json)（尾簇 7 后）、
[evidence/2026-10-04-m3-tail7.md](evidence/2026-10-04-m3-tail7.md)（根因链 + bisect 记录 +
排除路径）、
[evidence/2026-10-03-m2-slice1.json](evidence/2026-10-03-m2-slice1.json)（M2 片 1）、
[evidence/2026-10-03-m2m3-after.json](evidence/2026-10-03-m2m3-after.json)（M3 后）、
[evidence/2026-10-03-m3-tail.json](evidence/2026-10-03-m3-tail.json)（尾簇 1 后）、
[evidence/2026-10-04-m3-tail4.json](evidence/2026-10-04-m3-tail4.json)（尾簇 4 后）、
[evidence/2026-10-04-m3-tail5.json](evidence/2026-10-04-m3-tail5.json)（尾簇 5 后）、
[evidence/2026-10-04-m3-tail6a.json](evidence/2026-10-04-m3-tail6a.json)（尾簇 6a 后）、
[evidence/2026-10-04-m3-tail6bc.json](evidence/2026-10-04-m3-tail6bc.json)（尾簇 6b+6c 后）。
门禁：`make test` **EXIT=0**（尾簇 41 后一次通过）+ clippy（quickjs 面，
-D warnings）EXIT=0 + fmt 无 diff + shim 拼接 node --check 全绿 + corpus 全量
**1794P/170F/96TO**（尾簇 41——逐条 diff 仅 3 文件翻转零回归）+ reftest
**704/704（failed 0）**（尾簇 37 复核；38/39/41 零渲染面改动复用）+
**product-smoke EXIT=0**（diff 15.40% ≤ 20% + struct-check PASS——本 goal
首跑，DC-4 M4 列项补齐）。历史（尾簇 34 后）：1787P/176F/96TO。

**M4 尾簇 41（2026-10-07，本轮）——CANDIDATE 池收口，DC-1~4 全满足，goal Done**。
三修：① tilt 跨集部分给值互不派生（`_zwPointerTiltInit` 前置分支——给定保持/
缺省落默认，单集给值仍整集派生）；② pointer 层跨界事件携源 pointerType
（`_zwLayerCross`——touch/pen 流边界事件此前缺省标 mouse）；③ 处理站内
release 延迟生效三态（'armed'/'skip'/'fire' 相位 + 派发序号一次性对齐 +
up 序列入口站传参）+ 站外入口站 lost handler 重捕获抑制（五流验证收敛，
修复 capture_{touch,mouse}_and_release_at_got_capture + lostpointercapture_
is_first + pointerrawupdate_changes_pointer_capture 六子测）。重归类 5F：
mouse_capture_change_hover（capture 驱动 `:hover` 计算样式——style-system 面）、
multiple_pointerover（多指针状态机模型面）、events_after_lostpointercapture_
remove（runner 同代命中缓存 + Chromium 自注 bug 域）——进挂账定稿清单。
**DC-1~4 全满足**（DC-4 product-smoke 首跑通过），挂账定稿完成，goal Done。
详见 [evidence/2026-10-07-m4-tail41.md](evidence/2026-10-07-m4-tail41.md)。

**M3 尾簇 39（2026-10-07，本轮）——utils.js 资产补拉 + 解锁面三连修（1790P→1791P）**。
goal 脚本补 `fetch_raw "uievents/mouse/resources/utils.js"`（目录列举只收
.html，attributes / cancel-mousedown-in-subframe / mousemove_prevent_default_
action 三案 import 该资源 page-threw 各折 1 条文件级 Fail——tail-30 人工补拉
先例的脚本化）。解锁面断言链逐位修三件（part05/06）：① MouseEvent constructor
身份统一（R109 WrappedME own non-enumerable 覆盖，tail-11 PEM3Wrapped 同款）；
② composed 随 bubbles 缺省（mouse 分支补 `composed: _mBub`——PointerEvent 分支
同款已有）；③ cancelable 显式透传 + enter/leave 缺省 false（`_mCan` 镜像
`_pCan`）。attributes.html file-Fail→**named Pass**；另两案转具名余挂：
mousemove_prevent_default 2F（selectionchange/dragstart 默认动作——selection/
DnD 域）、cancel-mousedown-in-subframe TO（iframe 子框架消息路由域）。
composed/cancelable 全 corpus 零涟漪（逐条 diff 仅 3 文件）。详见
[evidence/2026-10-07-m3-tail39.md](evidence/2026-10-07-m3-tail39.md)。

**M3 尾簇 38（2026-10-07，本轮）——UA click/dblclick 携指针坐标（corpus 恒等零涟漪）**。
`__zw_pointer_up_sequence` 的 chorded click/主 click/dblclick 三分支 init dict
补 `clientX/clientY`（UI Events §5.2.2——click 与 mouseup 同指针位置；此前
auxclick/mouseup/contextmenu 均已携带、click 族漏网，派生坐标恒 0）。键盘激活 /
`element.click()` 无坐标 click 与 R108 宿主激活泛型 click 按既注契约不动。
单测 up-sequence 直驱双连击断言坐标面（part21）。corpus 逐条 diff 恒等——
语义正确性收口（语料无 click 坐标断言面，尾簇 37 预判成立）。详见
[evidence/2026-10-07-m3-tail38.md](evidence/2026-10-07-m3-tail38.md)。

**M3 尾簇 37（2026-10-07，6b8626f6a）——layerX/layerY 反射（shim + native 双路径）+ 基线重跑漂移归因（174F）**。
两改面 + 单测：① shim（part05）MouseEvent props 注册表补 layerX/layerY
（WheelEvent/PointerEvent 父链继承）+ `_zwMouseCoordInit` 派生（显式 init
floor 采信 / 缺省 = pageX/Y——headless 布局无变换几何，分层祖先偏移不可诚实
计算，恒走「无层」分支）；② native v8 模板（dom_bindings/event.rs）派生对补
layerX/layerY（pageX/offsetX 同款模式，生产唯一路径）；③ 单测四面断言
（native 派生 / 显式 floor / 派发面恒等 pageX / PointerEvent 继承）。
语义依据：WPT layer-coords-transform（w3c/uievents#398 + bugzilla 1975653）。
**inside 案根因升级**：指针命中 body（inner 未变换盒不含点位）——过案需变换
感知 gBCR + scrollIntoView + 滚动后命中全链，属渲染流域深结构（与 gBCR 同步
布局同域），跨流挂账。**相邻缺口登记**：UA click 派发 init dict 缺
clientX/clientY（auxclick/mouseup 均有）——语料断言面极薄未修，DC-2 坐标面
候选簇。**漂移归因**：HEAD 基线全量重跑与本轮逐条 diff——总册双 2059 恒等，
唯一行为 diff = layer 案翻转；尾簇 35 账册 2060 总册未复现，-1 属核算漂移
（tail-4/24 先例）。详见
[evidence/2026-10-07-m3-tail37.md](evidence/2026-10-07-m3-tail37.md)。

**M3 尾簇 33（2026-10-07，本轮）——execCommand text-control 事件序 + maxlength 哨兵 + textarea Enter 换行（1782P→1785P）**。
三件：① **execCommand('insertText') text-control 分支补事件序**（beforeinput（可取消，
取消返 false）→ value 变更 → input——R312 trusted 印记；**不派 textInput**（api 的
reject listener 断言面——execCommand 编辑命令与真键盘 send_keys 分流，Chromium 同此）
——此前裸 value 赋值不派事件，api 尾部 3 promise 永挂）；② **maxlength 哨兵修复**
（`HTMLMaxLength` 缺省 **-1** 被当有效截断值——`combined.length > -1` 恒真使无
maxlength 的输入被截成空串——探针实证 `assign:combined=""`；补 `+ml >= 0` 门）；
③ **textarea 的 Enter = 换行**（runner send_keys 目标分流：E006/E007 on TEXTAREA →
InsertText{'\n'}——text_plan 的 textInput followup 面；真实浏览器 textarea 不参与
implicit submission）+ **CE Enter 补 textInput(data='\n')**（`__zw_ce_enter`——
basic.sub.js 在 input handler 内断言 textInputEvents===1）+ `__zw_ce_insert` 参数化
（withTextInput=false——execCommand CE 分支复用 caret 管线不派 textInput）。
**加时序加固**：send_keys 的自聚焦从独立 focus 脚本**拼进首键 keydown 派发脚本**
（零额外 execute_script 往返——独立脚本在 workspace 并发负载下使命令轮询窗偶发
错过，内嵌 fixture 间歇 Timeout 三次复现实证，单跑 5/5 Pass 同款 flake 形态）。
门禁：editing 域 183P 无新增 Fail（`Changing selection from handler` 为尾簇 31 stash
基线确认的既有 Fail）+ keyboard 域 3 TO 恒值。余挂：**api CE execCommand 案**（TO 转
Fail——`__zw_ce_insert` appendChild 走 mutation 通道异步 apply，input handler 同步段
读 textContent 为空——textContent 同步可见性挂账）+ enter-textarea 的 CE 案（同族）。
详见 [evidence/2026-10-07-m3-tail33.md](evidence/2026-10-07-m3-tail33.md)。

**M3 尾簇 34（2026-10-07，本轮）——'value' in tag-gate + CE Enter 元素 caret/本地树变更（textInput 族全绿）**。
三件：① **`'value' in el` tag-gate**（part05 has 白名单——`prop === 'value'` 无条件
true 使 CE div 等非 form-control 元素误入 value getter 分支；消费方
`'value' in el ? el.value : el.textContent` 读恒 ''——gate 到 IDL value 接口成员族
INPUT/TEXTAREA/SELECT/OPTION/BUTTON/OUTPUT/METER/PROGRESS/PARAM/LI/DATA）；②
**CE Enter 元素 caret 分支**（`__zw_ce_enter` 空宿主/元素边界 caret 旧版 no-op——
input 缺失使 basic.sub.js 驱动 resolve 永挂；补完整事件序 beforeinput →
textInput('\n') → `<br>` 直插 → input）；③ **CE Enter 变更走本地 shim 树**
（caret 拆分 + `<br>` insertBefore + tail 节点——取代 innerHTML setter 的异步
mutation 可见面）。余挂收窄：textInput 域清零；keyboard-click-event TO（交互层）
与 CE insertParagraph（编辑 goal 邻接）维持。详见
[evidence/2026-10-07-m3-tail34.md](evidence/2026-10-07-m3-tail34.md)。

**M3 尾簇 35（2026-10-07，本轮）——wheel 源 scroll 步接通（wheel 族 3 案全绿）**。
四段通道：① stub `Actions.prototype.scroll` 编码（此前 no-op「记帐不重放」）+
`addWheel/setWheel` 源注册；② stub send() scroll 步入 plan（`wheel_scroll` op——
viewport `'@'` 绝对/元素中心偏移两形）；③ runner 命令处理（`resolve_pointer_target`
命中同指针命令）→ `script_wheel_scroll`；④ shim `__zw_wheel_scroll`——WheelEvent
派发（delta 透传、cancelable、R312 trusted 印记；滚动默认动作 headless 不消费）。
**「wheel-scrolling 需真滚动」预判推翻**——三案均只断言事件面（EventRecorder
target 序 + delta 透传），headless 无真滚动管线不阻塞本 goal 语料；真滚动默认动作
维持挂账（语料未断言滚动面）。详见
[evidence/2026-10-07-m3-tail35.md](evidence/2026-10-07-m3-tail35.md)。

**M3 尾簇 32（2026-10-06，39ca800f1）——send_keys 自聚焦 + CE ForwardDelete + Enter 键序（1778P→1782P）**。
三件：① **send_keys 入口自聚焦目标**（runner——WebDriver per-element send 语义；
slice22 `__zw_host_focus` 通道——多 promise_test 交错 focus 的组合时序下激活管线
焦点归属错位 → click 不发：keyboard-click-event 4 子测全挂的根因，zzprobe 三段
二分实证）；② **uE006 接 formless buttonish 激活**（Enter click 通路与 uE007 一致
——tail11 修复只判了 uE007）；③ **CE ForwardDelete 接通**（`__zw_ce_forward_delete`
——deleteContentForward 语义 mirror ce_delete，代理对安全；取代尾簇 31 noop）+
**动作 noop 不中断键事件序**（keyup 恒派——enter-input input 案 +1；旧版 Submit
noop 提前 return 跳过 keyup）。keyboard-click-event 行为面已通（探针 4/4——
EventWatcher/mini-report 交互层 TO 余挂，testharness 深水面）。textInput 族
16P→**18P**（delete + delete-selection 的 CE div ×2）、enter-input +1。余挂：
execCommand insertText 对 text control ×3（api.html）、enter 两文件的 textarea/CE
尾案（Submit/CE Enter 域）、keyboard-click-event TO（testharness 交互层）。详见
[evidence/2026-10-06-m3-tail32.md](evidence/2026-10-06-m3-tail32.md)。

**M3 尾簇 31（2026-10-06，7c550a079）——TextEvent 语义域 + selection key 存活面（textInput 族 2P→16P）**。
五件（engine shim ×4 + page-runtime/webview/runner Rust 面）：
① **TextEvent 接口特化**（part05/06）：无 constructor 语义（`new TextEvent()` TypeError——
throw wrapper 共享 prototype）+ 内部 ctor 供 createEvent/dispatch + `data` 字段 +
`initTextEvent`（0 参 TypeError、data 缺省 `'undefined'`、view 缺省 null）；
② **textInput 事件序**（UI Events legacy 附录——beforeinput → textInput(TextEvent) →
input）：text control 管线（`text_plan` followup 首位插 textInput）+ CE 插入管线
（`__zw_ce_insert`，完整保留可取消语义）；shim dispatch 层 textInput → TextEvent 实例；
③ **编辑键 key 名**（runner send_keys）：Backspace/Tab/Enter（uE003/uE004/uE006|uE007）/
Delete（uE017）透传 **WebDriver 键名**而非原始 PUA 转义 + uE006（Return）接 Enter 语义
（Submit）+ uE017 接 **ForwardDelete**（page-runtime 新 variant + `plan_text_forward_delete`
——collapsed 删 caret 后一 UTF-16 单元；CE 宿主 noop 挂账）；④ **testdriver selectorFor
shim 权威形优先**（`element.__zwSelector` 回查自指才信任——stub tag/attr/nth 启发式对
无 id 元素产 tag 形与 shim listener key 形不一致 → keydown/keyup 对无 id text control
静默丢——zzprobe 复刻实证）；⑤ **selection 状态 key sel 主导**（`_textSelKey`——
`_elKey` handle 主导形在 render_html rebuild 后 handle 换代孤儿化：页面 setter 写的
selection 新 proxy 读缺省 (0,0)）+ **textarea value getter 直读分支补 child text**
（非 stable key 直读 value attr 恒 ''——初始 child-text textarea 的 .value 丢初值，
DeleteBackward 快照 clamp (0,0) → NothingToDelete）。
门禁：keyboard 语料回归全绿（keydown/modifier/keypress）+ selection 域 2965P 无新增
Fail（stash 基线比对）+ textInput 族 16P/5TO（余挂 = execCommand insertText 对 text
control ×3、CE ForwardDelete、CE Enter 面——editing 域挂账）。详见
[evidence/2026-10-06-m3-tail31.md](evidence/2026-10-06-m3-tail31.md)。

**M3 尾簇 30（2026-10-06，本轮）——Meta 转义双认 + 跨界序 buttons 活掩码（1759P→1763P）**。
两件（engine shim part06）：① `_zwModifierForKey` meta 分支 U+E053→**U+E03D ‖
U+E053 ‖ 'Meta' 三认**（mouseevent_key_pressed +2；U+E053 系上游
boundary_events_modifier_no_pointer_movement ?Meta 变体写死 key——单改 U+E03D 曾
致该文件 4F，双认后 48P 恢复）；② `_zwLayerCross`/`_zwReentryCheck` 跨界序
buttons 0→当下按下掩码（synthetic tentative mouseover buttons ×2；UI Events
MouseEventInit——button un-initialized / buttons 活掩码）。textInput 族资产补齐
（fetch 脚本首轮漏 support/ 子目录——人工补拉 common.js + basic.sub.js，7 案
page-threw 转具名 2P/11F/2TO，**TextEvent 语义域新缺口**）。排除路径：compat-
mouse-events-when-removing-nodes 4F + boundary_events_attributes_during_drag 2F
= 动态 createElement 物化同 turn gBCR 零盒（ZW_TD_DEBUG 实证命中扫描全零）——
归 1a 待拍板项；tentative B 域（out/leave@被移除元素）维持尾簇 20 回退挂账。
详见 [evidence/2026-10-06-m3-tail30.md](evidence/2026-10-06-m3-tail30.md)。

**M3 尾簇 29（2026-10-06，986f8294e）——pointercapture_in_frame ?touch subtest 4 判例定谳：上游即 Fail（Δ0 记账轮，portal 段收口）**。
零源码改动。唯一余挂 ?touch subtest 4 判定**上游即 Fail**：wpt.fyi Chrome master run
（chrome-157 linux，run 6293748152795136，2026-10-06）?touch **4/6**（ZeroWeb 同变体
5/6 已超上游）；Chromium pem.cc `SendTouchPointerEvent`（#660-673）touch down 隐式
捕获 → gotpointercapture@down target（move 前）+ lostpointercapture（up 后）必录，
receivedEventList 恒 5 条 ≠ expected 3 条——直接操纵型指针全域不可过；上游 master
expected 未改（subtest 5 有 touch 特例、subtest 4 无——作者桌面视角遗漏）。ZeroWeb
实测序与 Chromium 推演逐事件一致。按尾簇 4 interleaved 先例记入挂账不再追；
上轮「touch move 专项」预判修正（move 已随捕获路由派发，非抑制缺口）。详见
[evidence/2026-10-06-m3-tail29.md](evidence/2026-10-06-m3-tail29.md)。

**M3 尾簇 28（2026-10-06，本轮）——frame-hold 残留清零（portal 第五段收口）**。一件：
非 hold portal up 分支补 `st.portalFrameHold = null`——隐式释放置 hold 后，up 落点在
frame 内、释放后 up 随命中目标出框的形态（subtest 1/2 类）此前不清 hold——残留跨
subtest 泄漏使后续 portal 事件 .id 误报帧根（probe 实证 subtest 2 单页全绿、语料序
贯态下全误报）。+2：pointercapture_in_frame 15→17P（subtest 5 ?mouse/?pen 转绿），
文件 18 子测全部具名、文件级 Timeout 消失；唯一余挂 ?touch subtest 4（非捕获触式
move 抑制的推广面，随 touch move 专项）——
[evidence/2026-10-06-m3-tail28.md](evidence/2026-10-06-m3-tail28.md)。

**M3 尾簇 27（2026-10-06，70f0e595e）——frame-hold 重接（portal 第五段，载体法）**。一件：
move/up 站 frame-hold 路由以 **body 视图原型包装载体**重接（`Object.create(bodyView)` +
own `.id`=doc.documentElement.id + 继承 dispatchEvent/冒泡链——绕开 tail-26 的 docEl
无 dispatchEvent 断面）；up 载体派发后清 hold/downSel/portalFrame。+2：subtest 6
?mouse/?pen Pass、subtest 5 ?mouse/?pen 转具名 Fail（hold-up own .id 在 corpus 内页读
空——`_r159HtmlAttrs` 槽提取序差异待比对）。对象图结论（tail-26 断面解明）：hold 块三
链均回 body 视图 = docEl 无 dispatchEvent——载体法为解。见
[evidence/2026-10-06-m3-tail27.md](evidence/2026-10-06-m3-tail27.md)。

**M3 尾簇 26（2026-10-06，00e59b400）——portal target id 值面落定 + frame-hold 实验回退（Δ0）**。
两件落地：① `doc.__zwBodyId` 值槽（part01 入口戳旁——body 视图 getAttribute 链断点的
portal 直读源，probe 实证 portal 事件 target.id 恢复）；② 合成 docEl `.id` 反射器
（R207 html 元素）。frame-hold 路由（mouse/pen release 后 frame 持有至 up）端到端通但
hold target 身份落 body（对象图未明）——回退保留挂点，portal 第五段入口=
entry doc 对象图探明。见 [evidence/2026-10-06-m3-tail26.md](evidence/2026-10-06-m3-tail26.md)。

**M3 尾簇 25（2026-10-06，24945baa8）——触式捕获路由（portal 第四段首片）**。一件：move 步
触式早退分支内加捕获路由——`_zwProcessPendingCapture` 结算 + 捕获生效时触式 move 随
捕获目标派发（早退抑制仅指非捕获形态的无 hover/边界序；捕获期 move 照常）。+5：
pointercapture_in_frame 11→13P（?touch subtest 3+6）+ 涟漪
setpointercapture_to_same_element ×2（触式 pending 此前永不结算）+ predicted_events
?touch（触式 pointermove 恢复）。余挂：?mouse/?pen subtest 5（frame 级捕获概念）+
subtest 6/?touch 4 断言面 + inner body `.id` 值面——
[evidence/2026-10-06-m3-tail25.md](evidence/2026-10-06-m3-tail25.md)。

**M3 尾簇 24（2026-10-06，649ec7c54）——portal 第三段（up 站外层捕获守卫）**。一件：up 站
portal 判定加 `st.capture['1'] || st.pending['1']` 前置——tail-23 只给 move/down 两站
加守卫，up 站漏网：down@outer 捕获后 move/up 进 iframe 区域时 pointerup 被劫持入
inner frame（probe 实证 pointerup@outerFrame + lostpointercapture 双缺失）。
pointercapture_in_frame 0 子测→11P（?mouse 1-4/?pen 1-4 全绿、?touch 1/2/4/5 绿、
subtest 3 触式转具名 Fail——TO 口径转移覆盖面扩张）。余挂：subtest 5/6
release-on-next-pointermove 链 + frame 级捕获概念——
[evidence/2026-10-06-m3-tail24.md](evidence/2026-10-06-m3-tail24.md)。

**M3 尾簇 23（2026-10-06，f82ea3bee）——subframe portal 第二段（per-frame capture）**。四件：
① body id 落视图（`_zwFinishIframeEntry` 直提 markup body id → setAttribute）；②
per-frame capture 状态机（`__zwPortalCapture*` + `st.portalFrame/portalCap/
portalCapPending`——活跃 frame 校验 NotFoundError 双向面 + 单指针单捕获 + **pending
模型**（gotpointercapture 随下个事件派发前结算——sync 派发 log 序反）+ pointerup
隐式释放 lostpointercapture）；③ inner target capture 方法补齐；④ portal 路由守卫
（捕获期事件随捕获目标/外层捕获优先/离框清活跃态）。+6（pointercapture_in_frame
subtest 1+2 ×3 变体 TO→Pass）；余 subtest 3-6（outer-frame 变体 + EventWatcher 面）
挂 portal 第三段。调试陷阱（跨函数 st / 裸标识符）入排除路径——
[evidence/2026-10-06-m3-tail23.md](evidence/2026-10-06-m3-tail23.md)。

**M3 尾簇 22（2026-10-06，074429d8b）——subframe portal 首段**。四件：① runner 命中下探
（最小包含盒命中 IFRAME → `@zwframe:<iframeSel>` frame-qualified 目标）；② shim portal
派发（`_zwPortalSplitSel`/`_zwPortalDispatch`——move/down/up 三站早退，解析
`_iframeDocCache` → inner body → 模板树原生冒泡派发 PointerEvent，probe 已证 body→doc
可达）；③ 模板元素 `.id` 反射器（工厂对象）；④ R255 分支 `_r159BodyAttrs` 补提取。
+1（mouse_pointercapture_inactivate_pointer TO→Pass）。余挂精确断面：inner body 视图
`.id` 直读（proxy trap 吞）、per-frame capture 状态机、跨 frame capture 无效判定——
[evidence/2026-10-06-m3-tail22.md](evidence/2026-10-06-m3-tail22.md)。

**M3 尾簇 21（2026-10-06，ad8a35023）——零净 P 基建 + 根因精化轮**。三件基建：retarget 路径
host elementFromPoint 回声弃用（skipHostSel——同 turn 移除后 host 视图恒返被移除元素
自身，move 级重定向全数失效的断面）+ post-rawupdate 重定向后清瞬态/重入态（防 settle
双补派）+ 参数面扩展。三族根因测绘（probe 实证）：①同 turn 动态元素 apply 后 gBCR 恒
零（handle 元素 rect plumbing——五个族 ~20F 同根因，修在渲染/rect 快照层，跨流域碰头项）；
②live 查询面对同步移除不感知（_zwRemovedSels 与 live-first query 脱节——retarget 命中
回声死循环）；③subframe 面测绘完成（冒泡派发 ✓/脚本 ✓/frames[0] ✓；缺 .id 反射器/
setPointerCapture/gBCR/per-frame 捕获态）。Δ0 honest 记账，排除路径见
[evidence/2026-10-06-m3-tail21.md](evidence/2026-10-06-m3-tail21.md)。

**M3 尾簇 20（2026-10-06，c64beab8f）——remove-hover 连通祖先重定向 + runner 安全 id 选择器**。两件
（细节/回退/oracle 冲突记录见 [evidence/2026-10-06-m3-tail20.md](evidence/2026-10-06-m3-tail20.md)）：
① move 步跨界序中移除/替换 hover 目标 → move 对重定向**最近连通祖先**（mouseenter 链
语义，不用命中测试——替换场景新子未入 enter 链）——mousemove_after_mouseover_target_removed
移除两案 +2P，替换两案余挂同 turn stale gBCR；② runner selectorFor 非安全 id 落 attr
筛选路径（`id="outerFrame body"` 曾产非法 `#outerFrame body` → target not found 全簇
断面）——pointercapture_in_frame 18F→3×文件级 Timeout（subframe 事件路由 blocker
暴露：指针命中不下探 iframe）。尝试并回退 1 件：mousedown/mouseup 拆除边界（out/leave@
被移除元素）——与 after_target_removed 稳定 oracle 冲突（tentative 文件自注浏览器分歧），
按稳定语料优先回退，冲突入档。

**M3 尾簇 19（2026-10-05，93c189648）——pointerrawupdate 语义 + chorded button 键序 + coalesced 克隆 + 反射面**。七件
（engine shim；细节/排除路径见 [evidence/2026-10-05-m3-tail19.md](evidence/2026-10-05-m3-tail19.md)）：
① pointerrawupdate 派发（secure + listener 存在性门槛，无 listener 整站跳过；站点序
跨界序 → rawupdate → pointermove；Process-Pending/capture 重定向复用 dispatch 层既有
机制）；② chorded button 键序（down/up 早退分支——多键按下/中间释放派 pointermove，
中间释放照常结 click/auxclick 携剩余掩码）；③ `_zwButtonMask`（WebDriver index↔PE
位序互换——middle=4/right=2）；④ got/lostpointercapture buttons 显式携带；
⑤ coalesced 队列克隆（非冒泡同属性克隆 + target 首派回填/redispatch 保持 + `_zw19Clone`
印章隔离构造注入序列）；⑥ deferred mouse 层跨界（rawupdate 后结算 compat mouse 边界序）；
⑦ 反射面（onpointerrawupdate secure-only exposure、persistentDeviceId、pseudoTarget）。
已知回归 1：pointerrawupdate.html 非 https 变体 1P→0P——runner 单 origin https 使
secure 分流不可测，scheme 分流属 runner 级改造挂账。corpus +119（总册 1948→2040）。

**M3 尾簇 18（2026-10-05，91c61702b）——range 拖拽取值默认动作 + 空命中坐标保持**。三件：
① range input 拖拽取值（shim down/move/up 三站 hook）：INPUT[type=range] 按下/拖拽
按指针位置设 value（input 事件）+ release 派 change；轴向——水平 pct=(x-rx)/rw、
垂直（inline style writing-mode 含 vertical）pct=1-(y-ry)/rh（`vertical-lr;rtl` 顶
=max，WPT「up drag → 100」面）——尾簇 17 回退的 w/h 比启发式由 inline-style 检测 +
反转修正替代；② runner `resolve_pointer_target` **空命中坐标保持**：命中扫描为空
（指针移出任何元素盒到文档空白区）不再回落 origin+RAW offset（坐标错页 (0,186)
断链实证），改派 body 落点 + 计算坐标（93,202）——真浏览器事件落 html/body；
③ touch move 早退路径补 range 取值（touch 无 move 对但取值是默认动作）。+4。
细节与回退史见 [evidence/2026-10-05-m3-tail18.md](evidence/2026-10-05-m3-tail18.md)。

**M3 尾簇 17（2026-10-05，aa992c1c1）——window.visualViewport 最小实现**。CSSOM-View
VisualViewport（headless 近似：scale=1、offset/page 恒 0，width/height 经 getter 实时
读 innerWidth/innerHeight——`__zw_user_resize` 更新后自动跟随）。range_input 族的
ReferenceError 整子测拒绝解除（+6）。range input 拖拽取值默认动作未落地（轴向约定
首版 w/h 比启发式使 vertical-on-vertical 2 例回归，已回退——尾簇 18 候选，需按
writing-mode/appearance 约定专项）。细节见
[evidence/2026-10-05-m3-tail17.md](evidence/2026-10-05-m3-tail17.md)。

**M3 尾簇 16（2026-10-05，58b1854c7）——跨界/移动事件按钮按层分流**。UI Events「un-
initialized」语义：compat mouse 面（mousemove/mouseover/out/enter/leave，move 双路
+ `_zwLayerCross` 四站 + `_zwReentryCheck`）`button` 恒 **0**；pointer 面（pointer-
over/enter 等）保持 **-1**（PE spec move/边界无按键）。mouseevent_move_button 转绿
+ mouseenter-mouseleave-on-drag 3/3（button 断言面）。细节见
[evidence/2026-10-05-m3-tail16.md](evidence/2026-10-05-m3-tail16.md)。

**M3 尾簇 15（2026-10-05，dcf5a4d89）——down 序列同位幂等 + compat mouse 抑制链归一**。
三件（shim）：① down 序列 mouse 分支**同位同目标幂等**（`overSel` 相同且指针坐标
未变 → 不再派 move 对——Actions move 已派过一对，down 隐含迁移重复派发即双 move）；
② pointerdown 取消 → `st.compatSuppressed` 手势级标记（至下次 down 清除），
mousemove（move 双路）/mouseup 随之抑制——PE spec §11 canceling pointerdown 全链
抑制；③ **pointerup 自身取消不再抑制 mouseup**（Chrome 实测——旧 upPrevented 门为
suppress_compat 族 pointerdown 面的误推广，撤销）。mouse-pointer-preventdefault
8/8 + suppress_compat 连带 +2。细节见
[evidence/2026-10-05-m3-tail15.md](evidence/2026-10-05-m3-tail15.md)。

**M3 尾簇 14（2026-10-05，c2fe2a8af）——事件构造器 `length` 归一 + init* 0 参 TypeError**。
spec 各事件构造器仅 `type` 必填（eventInitDict 可选）——wrapper（UIEventCtor109/
WrappedME/WrappedKB/PEM3Wrapped）与 shim 基构造器签名 `(type, options)` → `(type)`
（options 经 `arguments[1]` 运行时读取），`eventType.length === 1`（WPT
Event-subclasses-init 断言面）；initUIEvent/initMouseEvent/initKeyboardEvent 补
0 参 TypeError 守卫。+4。

**M3 尾簇 13（2026-10-05，c2eb6c48c）——查询视图基座去重 + hit test 幽灵过滤**。两件：
① `with_query_view_doc` 的 InsertAdjacentHtml 重放加**基座已反映去重**——dom_html
换代写入点（R55 重注册换新 Arc = 最新 cached_html / user_actions 批末更新）使基座
可能已含已 apply 的落地拷贝，全量重放再插一次即双计；fragment 首 id 已在基座 →
跳过重放（无 id 片段保守重放）。**候选方案史**：视图基座钉定（注册/导航 Arc 对
匹配）两版均因语义冲突废弃——注册钉定破坏 R358 就地换代可见性（3 单测红）、读取
侧盲钉钉到导航过渡态上一页 html（多文件序列 selectorFor('body') null 断链）；
② runner 命中测试 area 循环加 `isConnected` 过滤（R47 视图保留已移除元素可查询
——幽灵经 live-first `__zw_contains` 判否）。image-map 族 18/18。细节与排除路径见
[evidence/2026-10-05-m3-tail13.md](evidence/2026-10-05-m3-tail13.md)。

**M3 尾簇 12（2026-10-05，e96325586）——pending 捕获换防时序 + compat mouseup 捕获落点**。
两件（shim up 序列）：① `capturedSel` 改在 `_zwProcessPendingCapture` 换防**后**取
（PE spec §9.2「before dispatching the next pointer event」——pointerdown 里
setPointerCapture 的 pending 旧版在 pointerup 派发内才换防，capturedSel 恒旧值）；
② compat mouseup 落点分流：mouse 随 `upEff`（捕获有效落点——隐式释放后内部重定向
不可达）、touch 随 `upSel`（同 touchstart/touchend 同目标语义；首版统一 upEff 曾
回归 touch 面，-1 轮修正）。click_during_parent_capture mouse 面 +4。细节见
[evidence/2026-10-05-m3-tail12.md](evidence/2026-10-05-m3-tail12.md)。

**M3 尾簇 11（2026-10-05，5eaa2c81c）——click/auxclick/contextmenu 的 PointerEvent 实例化**。
四件：① UA 指针 click 族 → PE 实例（up/down 序列 detail 携 pointerType，R108 经原型链
接通一次；webdriver 折叠 click 泛型零变化——双翻转坑不复现）；② PE wrapper 的
constructor 身份统一（`event.constructor === window.PointerEvent` 断言面）；③
click() API + Actions 链 ENTER（formless buttonish——旧版 InsertText{'\u{E007}'} 对
button 类 input 抛 InvalidStateError 整链拒绝）→ 非指针生成 click
{pointerId:-1, pointerType:''}；④ send_keys 通道同语义。细节与排除路径（含 bisect
假因果陷阱）见 [evidence/2026-10-05-m3-tail11.md](evidence/2026-10-05-m3-tail11.md)。

**M3 尾簇 10（2026-10-05，b983b8ccb）——untrusted 事件构造 native 路径坐标语义**。
native MouseEvent 模板（唯一生产构造路径——R384）坐标族 floor 化（`init_floor_int`，
浏览器语义非 WebIDL truncate）+ pageX/Y/offsetX/Y 派生（init 显式则 floor 采信、
缺省 = floor(client)；`is_number` 门防缺失键 NaN→0 压派生——首轮 4 范围全灭根因）。
对齐尾簇 6 的 shim 面 `_zwMouseCoordInit`。corpus **+352P 零回归**（细节与排除路径见
[evidence/2026-10-05-m3-tail10.md](evidence/2026-10-05-m3-tail10.md)）。

**M3 尾簇 9（2026-10-05，ce7ca7f37）——image-map 命中 + 跨目标 click 组合 + 跨批 handle
插入序列化落地**。五件 + 宿主活性修复（根因链与排除路径见
[evidence/2026-10-04-m3-tail9.md](evidence/2026-10-04-m3-tail9.md)）：

1. **image-map 几何命中**（runner 命中测试 JS）：`img[usemap]` → map → area 的
   rect/circle/poly 几何命中（文档序首个 inside 即 top-most）。+12。
2. **跨目标 click 组合以实派 target 为准**：`effDown = st.downSel || downSel ||
   upSel`——stub 编码的 downSel 是 origin 元素近似（image-map 面 origin=img 实派
   =area）。+2。
3. **变异代际扩面**：悬停元素属性变异 / IMG image-map 属性 / AREA 插入（三站）推进
   `mutTick`。
4. **settle 延迟结算**：命中目标未落 applied 快照时 defer 不消费代际。
5. **指针命令入口 settle**（`mut_hover_settle_if_dirty`）：同批命令连发场景的渲染
   机会近似。

附带宿主活性修复：createElement 产物跨 apply 代际（含 refresh_if_html_changed 重建
窗口）后 `InsertBefore{child_handle}` 硬错曾**卡死共享 mutation 队列**（webview 吞咽
Err 游标不推进，批尾全丢）——创建代际印章 + 跨批改走 child outerHTML 序列化落地
（InsertAdjacentHtml 物化，shim 唯一真相）；apply 硬错批丢弃推进游标（P19 钉死的
apply 侧 lenient 禁令不动，活性修在 webview 批丢弃层）；id 型 handle 节点移除改派
selector Remove（镜像绑定失效时 RemoveHandle no-op 泄漏）。

**M3 尾簇 8（2026-10-04，6045a7462）——mutation 驱动悬停重定向 + 修饰键全局态**。三件
（细节与排除路径见 [evidence/2026-10-04-m3-tail8.md](evidence/2026-10-04-m3-tail8.md)）：

1. **修饰键全局态**（`_zwModifiers`）：keydown/keyup 维护（事件位先算、后落态），
   合成 pointer/mouse 事件缺省携带当下态（detail 显式值优先）。+32 主修复面。
2. **瞬态悬停重结算**（`hoverTransient {sel, reattached, px, py}` + 双路 settle）：
   rAF 派发点同步结算（渲染机会边界近似——OFF 模式 rAF 同步执行、测试尾段同一脚本
   任务，探测环来不及）+ 探测环几何结算（poll → fresh 命中测试 → 端点/瞬态两段
   跨界）。px/py 快照守卫防 cleanup 重插（指针已移开）泄漏。
3. **R3254-K2 修饰键单测校准**：持久态语义下每臂补 keyup 复原（意图不变）。

**M3 尾簇 7（2026-10-04，1cbbeaaa8）——R334 重插同 turn 查询可见性 + variant 大小写
保真**。三件：

1. **by-selector 重插索引**（part05 `_zwPendReselBySel`）：R334 sel 子重插的结构 wire
   异步 drain，窗口内 gEBI 恒 null（mouseover-at-removing 30 迭代链 i1 起同步抛错 →
   整链 microtask 塌缩进单轮、永不自愈）。R51c by-id 索引登记不了它——重插子宿主侧已
   无节点，`nd.id` 读链落空返 ''。以 R334 已知 `__zwSelector` 为键直登，gEBI 查
   `'#'+id`（in-doc 门 = 槽位 parentSel 树中判定）；表随 apply 代际 bump 清空。
   mouseover-at-removing **1P/29F→30P×2 全绿**。
2. **探测轮首 flush**（`WebView::flush_pending_shared_mutations` + `take_probe` 入口）：
   上一轮 JS 排队的结构 wire 本轮 JS 前落活 DOM（跨轮查询/命中少断一代；队列空零成本）。
3. **variant 值大小写保真**（`case_variants`）：旧实现小写源码后提取 variant——
   `?Shift`/`?preventDefault=…` 被写成 `?shift`/`?preventdefault=…`，页面
   `URLSearchParams.get`（大小写敏感）miss。修复后 modifier 族 `keyDown(undefined)`
   16F 解除为真语义缺口 2P/4F×4；click_during_parent_capture / synthetic-button-state
   的 pd=/buttonType= 变体首次跑真实配置。

尾簇 7 排除路径（bisect 实证，勿重试）：`with_query_view_doc` 把 `InsertAdjacentSelElement`
纳入 live_ok 排除 + 视图烘焙——live_ok 扫 append-only 队列全史，任一 sel-insert 入队
（即使已 drain）即永久切视图路径，after_target_removed 4P→2P 真回归。

**M3 尾簇 4（2026-10-04，650cdfa2e）——mutation 族收口**。三件套：

1. **runner 执行时点重解析 + shim html 结构性刷新**：指针命令出队时以 fresh
   gBCR 做元素 origin 中心+offset 命中（`resolve_pointer_target`，计划只编
   origin+offset、'@' 前缀 = viewport 绝对）；shim outerHTML 变化 → `render_html`
   全量重建 + handle 重绑 + rect 快照刷新（`WebView::refresh_if_html_changed`，
   R100 persistent_handle_nodes 经 `rebind_handle_node` 回填）。刷新经**宿主
   mutation 代际计数**（`js_dom_bridge::mutation_version`）门控——无 mutation 批
   零成本跳过（探测环 ~1ms/拍，无门控的逐拍百 KB 序列化曾跑穿 test-guard
   time-limit）。
2. **detach→re-insert 跨批语义（shim 根因）**：页内 `removeChild` 后跨 turn
   `appendChild` 曾致宿主 `InsertAdjacentSelElement` child 失配 lenient skip
   （R361 批内 stash 跨批即弃，元素永久丢失——探针实证）。新增
   `DetachedNodeStash`（selector→序列化片段，FIFO 封顶 32，导航边界清）：
   `Remove`/`RemoveChildAt` 记账、insert 失配时重解析片段插回（片段跨宿主全量
   重建有效，NodeId 跨重建失效故用片段）。配 `_zwIsConnected` 同步移除标记
   优先（脚本内 remove 后 host apply 前连接性即为否）。
3. **pointer/mouse 双层跨界拆分**：`_zwPtrState.mouseOverSel` 独立 compat mouse
   hover 位——touch 抬起悬停拆除仅 pointer 层（mouse 边界只随真实位置跨界，
   WPT ?touch mouse 子测试面）；pointerdown/up 派发中目标被页内 listener 移除 →
   compat mouse 事件**重定向**新落点（命中测试/宿主父链回退；hoverable 双层
   cross、touch 仅 mouse 层——`_zwRetargetSel` 加 skipCross 参数）。

结果：after_target_removed 主文件 3→**12/12 全绿**；37 subtest 净改善
（after_target_removed +12、capturing_boundary_event_handler_at_ua_shadowdom +9、
after_target_appended +9、pointerup_after_pointerdown_target_removed +3 等）。
表面回归 6 全部归因排除：appended_interleaved ×3 = vacuous pass 丧失（该族
expected 含 click@ 记账而页内 logEvent 只收 "mouse" 前缀——**pin 版上游即
Fail**，见 [evidence/2026-10-04-upstream-tentative.md](evidence/2026-10-04-upstream-tentative.md)）；
setpointercapture_to_same_element_twice ?touch ×2 = 已存于 committed HEAD
（m3-tail.json 基线早于尾簇 1 touch 语义）；predicted_events ?touch ×1 =
case 超时调度伪影（双树隔离跑逐字节同型）。

**M3 尾簇 5（2026-10-04，本轮）——insert-under-cursor 重入 + touch 隐式捕获
up 前清除**。两件：

1. **hover 重入旗标**：hover 元素被 remove（`_zwMarkRemoved` 钩子）或同父
   move 重挂（appendChild 已连接 sel 子，R334 分支记旗标——move 不抽
   Remove mutation 不走 mark）→ 下一指针事件派发前对回连的原 hover 元素补派
   over/enter 双层重入面（`_zwReentryCheck`，move/down/up 三站；跨界他元素即
   失效）——WPT after_target_appended moved variant「(child-moved) →
   pointerover@child → pointerup@child」断言面。
2. **touch 隐式捕获 up 前清除**：`_zw_implicit_` 前缀 pending 于 up 序列入口
   删除（此前 Process-Pending 于 pointerup 派发时换防并把 up 重定向 down
   目标；Chromium 行为 up@新命中目标——WPT after_target_appended ?touch
   「pointerdown@parent,(child-attached) → pointerup@child」断言面）。显式
   setPointerCapture 不受影响（capture 族回归扫零变化）。

结果：after_target_appended 9→**19P**（?mouse 全绿；?touch 残余 5 案 = Chromium
touch 接触失效语义尾簇：D2 隐含迁移 enter 序上游 expected 为 child→parent
内层先序〔逆于同文件 ?mouse 的 parent→child〕+ 重入后 up 的 teardown 抑制
+ enter@parent 抑制）。

**M3 尾簇 6a（2026-10-04，本轮）——untrusted 事件构造语义（分数坐标 + tilt/
angle 互换）**。两件，均纯构造层（派发路径共用 ctor 但面不变）：

1. **坐标 floor + page/offset 派生**（`_zwMouseCoordInit`）：MouseEvent/WheelEvent/
   DragEvent 全型 + PointerEvent 的 click/auxclick/contextmenu 三型坐标构造值
   floor（UI Events long 语义；PE fractional 例外集——其余 pointer 型保 double）；
   pageX/pageY/offsetX/offsetY dict 未给时自 clientX/clientY 派生（旧版落 0——
   fractional untrusted 168F 根因）。
2. **tiltX/tiltY ↔ azimuthAngle/altitudeAngle 归一**（`_zwPointerTiltInit` +
   `_zwTiltToAzAlt`）：spec 换算 + ±90° 角点简并（tan 爆炸 → altitude 0/双 90
   azimuth 0）+ 常规角 1-ulp snap（tan(π/4)≈0.9999… 的 atan 链漂移）。

结果：**corpus 502P→1107P（+605，零回归）**——corpus 总 subtests 1195→1955
（fractional untrusted 此前脚本中断未及注册的 ~760 subtest 全量入册）：
fractional untrusted 104→680P（+576）、tilt 1→24P（全绿）、constructor 双文件 +6。
证据：[evidence/2026-10-04-m3-tail6a.json](evidence/2026-10-04-m3-tail6a.json)。
门禁：shim 拼接 node --check 全绿（纯 JS 变更，无 Rust 面）。

**M3 尾簇 6b+6c（2026-10-04，本轮）——mousedown→focus 默认动作链 + touch 接触
失效语义收口**。三件：

1. **焦点迁移序修齐**（part04 proxy focus/blur 陷阱）：失焦相位先于获焦相位且
   blur 先于 focusout（WPT focus-events expected「blur@a → focusout@a →
   focus@b → focusin@b」；旧序 blur 迟到获焦相位后——focus.html 族全灭根因）+
   relatedTarget 双向（blur/focusout@旧 携新焦点、focus/focusin@新 携旧焦点）+
   失焦相位 activeElement 落空（spec focusing steps）。slice22 的
   `__zw_host_focus/__zw_host_blur` 首次接线（`_zwFocusSel`，down 序列 mousedown
   未取消时可聚焦目标迁移——此前 `script_host_focus` 无 caller 死代码）。
2. **up 内变异 post-up 结算**（尾簇 6c）：mutTick 代际（remove/appendChild 钩子
   推进）+ wire 落点记录（upMutSel）——同元素重插 → 重入面（F4/F5）；异元素插入
   → 跨界序**延迟结算**（pendingCross 于下一指针命令入口 flush——up 内插入元素
   本 turn 宿主树不可派发，zwprobe 实证空目标事件）；移除型变异不触发结算两分支
   （拆除照旧——after_target_removed pointerup-remover 断言面）。
3. **child-first enter 序**：跨界后拆除武装一次（`touchChildFirstEnter`，pointer
   层 enter 链目标先序消费即清——F1-D2 断言序；无跨界裸拆除保持外先内——F3/F4
   D1 断言序）。

结果：**corpus 1107P→1115P（+8，零回归）**：after_target_appended **24/24 全绿**
（?touch 3→8P——teardown 抑制 + 延迟跨界 + child-first 序）、focus-events +3
（focus / focus-contained / focus-automated same-DocumentOwner；残余 = iframe 跨
文档焦点（different-DocumentOwner）与 keydown→focus activation 两案）。
证据：[evidence/2026-10-04-m3-tail6bc.json](evidence/2026-10-04-m3-tail6bc.json)
（含 2 案本地 zwprobe 探针已剔除入账）。门禁：node --check + 双族聚焦跑全绿。

**DC-4 门禁（2026-10-06，尾簇 30 后）**：workspace **49 test result 全 0 failed**
+ clippy（quickjs 面，-D warnings）EXIT=0 + shim 拼接 node --check 全绿（纯 JS shim
变更，无 Rust 面；corpus 门 = make testharness-uievents 全量 1763P/183F/96TO 零回归）。
历史（尾簇 28 后）：workspace 19,870P/0F + reftest 704/704。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | uievents + pointerevents corpus 导入 + 基线 | ✅ 2026-10-03 |
| P2 | 鼠标事件序/坐标/click 组合语义修齐 | ✅ 核心修齐（2026-10-07——事件序/坐标/click 组合/layer 反射/click 坐标/TextEvent 域全落地，见已完成切片尾簇 1-39；**余挂 = 尾簇 40 挂账定稿清单**，无 in-stream 必修面） |
| P3 | Pointer 生命周期 + capture 三方法 + enter/leave 边界序 | ✅ 核心修齐（2026-10-07——capture 三方法 + portal 段 + 边界序 + touch 接触失效 + rawupdate 语义落地；**余挂 = 尾簇 40 挂账定稿清单**） |
| P4 | touch-events / pointerlock / IME 组合挂账定稿 | ✅ 2026-10-07（尾簇 40 编目 + 尾簇 41 CANDIDATE 池收口；重入条件见定稿清单各类注记） |

## M4 挂账定稿清单（尾簇 40，2026-10-07——corpus 1791P/173F/96TO 逐案编目）

编目基线 = 尾簇 39 全量跑 `--json`（HEAD 65b97dbc3 逐案归类，覆盖 173F/96TO 精确
到条）。各类根因与重入条件：

| 类 | F | TO | 归因 | 重入条件 |
|---|---|----|----|---------|
| touch-action + 滚动管线（渲染流域跨流） | 51 | 16 | parsing/computed/inheritance 值面（css-parser/style-system）+ touch-action 交互判定与 swipe/axis-lock（真滚动管线 headless 缺失） | 渲染流实现 touch-action 计算值 + 滚动消费后碰头（run-rules §9 工作面不重叠） |
| 同 turn gBCR 零盒族（1a——用户拍板） | 30 | 0 | reappend/removing_last_over/compat-mouse-removing/mousemove_after_mouseover/during_drag/drag_on_added_range/innerHTML/rawupdate_remove_target——变异同 turn 布局不刷新（尾簇 30 ZW_TD_DEBUG 实证） | gBCR 同步布局深结构改造（RenderPipeline Arc 化——run-rules §11 用户点名） |
| tentative（上游自注分歧/tentative/manual） | 18 | 1 | synthetic-button-state 按钮/移除序（Chromium 分歧自注）、persistentDeviceId（tentative 值面）、pointermove_after_pointerover_removed | 上游定稿或产品需要时重开 |
| interleaved（上游 expected 记账 bug） | 18 | 0 | appended/removed_interleaved ×3 变体——页内 logEvent 只收 mouse 前缀而 expected 含 click@ 记账（tail-4 pin 版判例：上游即 Fail 不再追） | 上游修 expected 后重跑 |
| shadow DOM / slot retarget（web-components 前置） | 15 | 3 | mouse_target_after_*_removed ×9、to_slotted_target ×3、from_slot、pointercapture-in-shadow-dom 等 | declarative shadow DOM（shadowrootmode）+ slot retarget 立项（用户点名前置） |
| iframe/subframe/portal 邻接 | 14 | 1 | click_during_parent_capture/click_during_capture（iframe 内捕获链）、capture_mouse_and_release 双案（pointerout after lost capture）、pointercapture_in_frame ?touch、after_adoption、cancel-mousedown-in-subframe | subframe 事件路由深化立项（portal 段后续，in-stream 可选） |
| testharness 交互层 TO（EventWatcher 深水面） | 0 | 59 | pointerevent_attributes ×10 / haspointercapture ×6 / releasepointercapture ×8 / setpointercapture ×6 / pointercancel 族 ×5 / boundary_in_capturing ×3 / sequence_at_implicit_release ×4 等——多 promise_test 交错 + EventWatcher 依赖 runner 命令窗语义，单案行为面已对齐（探针实证） | runner testharness 深水面专项（跨 corpus 基建，非本 goal 语义面） |
| DOMAIN（域外挂账） | 3 | 8 | mousemove_prevent_default（selection/DnD 默认动作——DnD 已排除、selection 属 editing goal）、mouse-on-object（object 兼容）、drag-interaction、pen 变体 TO | 各属域 goal 立项时带案 |
| coalesced/predicted 真采样（headless 无采样） | 6 | 5 | under_load/attributes/movement 加总、constructor getCoalescedEvents、predicted ?touch | 真采样管线立项（master M4 既有挂账候选） |
| RENDER（渲染流域邻接） | 5 | 1 | mousemove-between（margin:auto 居中——尾簇 36 归因）、layer-coords inside（变换几何——尾簇 37）、layout_change_mouseover、hover-generates-content、interpolation、click-on-html | 随渲染流 hover/居中/变换几何落地 |
| KEYBOARD_FOCUS（键盘域邻接） | 2 | 2 | focus-automated-blink（iframe 跨文档焦点）、focus-management（keydown→focus activation）、keyboard-click/accesskey TO | 键盘域重入时带案（已归档 goal） |
| SCHEME（runner secure 分流——专项拍板） | 2 | 0 | pointerrawupdate 非 https 变体（runner 单 origin https 使 secure 分流不可测——尾簇 19 已知） | runner `.https.html` scheme 分流专项拍板（master M4 既有） |
| RAWUPDATE 余点 | 1 | 0 | flush_pointercapture（rawupdate 前捕获结算时序） | in-stream 候选池 |
| CANDIDATE（尾簇 41 收口） | 5 | 0 | 尾簇 41 修 3F（capture_*_and_release 双案 + tilt 部分集 init）；余 5F 重归类：mouse_capture_change_hover ×3（capture 驱动 `:hover` 计算样式——style-system 面，渲染流域）、multiple_pointerover（多指针状态机模型面）、events_after_lostpointercapture_remove（runner 同代命中缓存 + Chromium 自注 bug 域） | `:hover` 随渲染流 hover 态落地；多指针随结构性扩展立项；Chromium 侧修后重跑 |



## 已完成切片

- **M1（2026-10-03）**：corpus 导入 + runner 通道 + 分类基线 1024 subtests 230P +
  suites CSV 回填（[evidence/2026-10-03-m1-baseline.md](evidence/2026-10-03-m1-baseline.md)）。
- **M2 片 1（2026-10-03，5d5669118）**：Actions stub 重写为上游多源 API 面 +
  PointerEvent 分支 + 边界事件序 + click 组合序（dblclick/公共祖先/auxclick/
  contextmenu）+ viewport 命中测试。+113。
- **M3（2026-10-03，19d5bb1c4）**：Pointer Capture 语义化 + 捕获重定向 + active
  pointer 状态机 + Actions 逐步重放 + Event.prototype 面。+32。
- **M3 尾簇（2026-10-03，7561ba05a）**：element-origin 偏移语义（中心+offset 命中）
  + coalesced/predicted stub。+76。
- **M3 尾簇 2（2026-10-04，f3564b5c2）**：目标移除重定向（命中测试重定向 +
  dangling 边界面 + touch 悬停拆除 + 祖先链快照回退）。+6（after_target_removed
  pointerdown-remover 3 variant 转绿；uievents/mouse 17→19P、order-of-events
  6→7P）。
- **M3 尾簇 4（2026-10-04，650cdfa2e）**：mutation 族收口（执行时点重解析 + 跨批
  detach 片段 stash + 双层跨界 + compat 重定向）。457P→492P。
- **M3 尾簇 5（2026-10-04，d6433f0e8）**：insert-under-cursor 重入 + touch 隐式
  捕获 up 前清除。492P→502P。
- **M3 尾簇 6a（2026-10-04，68edcce1b→baaabf835）**：untrusted 构造语义（坐标
  floor + page/offset 派生 + tilt/angle 归一）。502P→1107P（总册 1195→1955）。
- **M3 尾簇 6b+6c（2026-10-04，001375034 校准）**：mousedown→focus 默认动作链（迁移序 +
  relatedTarget + slice22 钩子接线）+ touch 接触失效收口（up 内变异 post-up
  结算 + 延迟跨界 + child-first 序）。1107P→1115P。
- **M3 尾簇 7（2026-10-04，1cbbeaaa8）**：R334 重插同 turn 查询可见性（by-selector 索引 +
  探测轮首 flush）+ variant 值大小写保真。1115P→1167P（总册 1955→1948——pd=/
  buttonType= 变体按真实配置计册）。
- **M3 尾簇 8（2026-10-04，6045a7462）**：修饰键全局态（`_zwModifiers` + 合成事件缺省
  携带）+ mutation 驱动瞬态悬停重结算（rAF 派发点同步 + 探测环几何，px/py 守卫）。
  1167P→1202P。
- **M3 尾簇 9（2026-10-05，ce7ca7f37）**：image-map 几何命中（runner 命中测试 JS 扩
  area shape/coords 面）+ 跨目标 click 组合实派 target 优先 + mutation 代际扩面
  （悬停元素属性/IMG image-map 属性/AREA 插入）+ settle 延迟结算 + 指针命令入口
  settle + 跨批 handle 插入序列化落地（共享队列卡死修复）。1218P（前值 1202P）。
- **M3 尾簇 10（2026-10-05，b983b8ccb）**：native MouseEvent 模板坐标 floor + page/offset
  派生（`init_floor_int` + is_number 门）。1218P→1570P（+352）。
- **M3 尾簇 11（2026-10-05，5eaa2c81c）**：click/auxclick/contextmenu PointerEvent 实例化 +
  非指针生成 pointerId=-1（click() API / Enter 激活）+ Actions 链 ENTER 默认动作补齐。
  1570P→1583P（+13）。
- **M3 尾簇 12（2026-10-05，e96325586）**：pending 捕获换防时序（capturedSel 后取）+
  compat mouseup 捕获落点分流（mouse=upEff / touch=upSel）。1583P→1588P（+5）。
- **M3 尾簇 13（2026-10-05，c2eb6c48c）**：视图重放基座去重（fragment id 已在基座跳过）+
  命中测试 isConnected 幽灵过滤。1588P→1590P（+2，image-map 族 18/18 收口）。
- **M3 尾簇 14（2026-10-05，c2fe2a8af）**：事件构造器 `length` 归一（wrapper + shim 基构造
  `(type)` 化）+ init* 0 参 TypeError 守卫。1590P→1594P（+4）。
- **M3 尾簇 15（2026-10-05，dcf5a4d89）**：down 序列同位幂等 + compat mouse 抑制链归一
  （compatSuppressed 手势标记 / 撤销 upPrevented 过泛化）。1594P→1609P（+15）。
- **M3 尾簇 16（2026-10-05，58b1854c7）**：跨界/移动事件按钮按层分流（compat mouse=0 /
  pointer=-1）。1609P→1610P（+1）。
- **M3 尾簇 17（2026-10-05，aa992c1c1）**：window.visualViewport 最小实现。
  1610P→1616P（+6）。
- **M3 尾簇 18（2026-10-05，91c61702b）**：range 拖拽取值默认动作（轴向修正）+ 空命中
  坐标保持。1616P→1620P（+4）。
- **M3 尾簇 19（2026-10-05，93c189648）**：pointerrawupdate 语义（secure + listener 门槛 +
  dispatch 层 Process-Pending 复用）+ chorded button 键序/位掩码映射 + got/lost buttons
  显式携带 + coalesced 队列克隆 + deferred mouse 层跨界 + 反射面
  （onpointerrawupdate/persistentDeviceId/pseudoTarget）。1620P→1736P（+116，总册
  1948→2040）。
- **M3 尾簇 20（2026-10-06，c64beab8f）**：remove-hover 连通祖先重定向（move 步）+
  runner selectorFor 安全 id 门（pointercapture_in_frame 首断面解除，subframe 路由
  blocker 转记 Timeout）。1736P→1738P（+2）。
- **M3 尾簇 21（2026-10-06，ad8a35023）**：retarget host-echo 弃用 + 瞬态清零 + 参数面
  扩展；三族 blocker 根因测绘（动态元素 rect / live 查询移除感知 / subframe 面）。
  1738P→1738P（Δ0，基建轮）。
- **M3 尾簇 22（2026-10-06，074429d8b）**：subframe portal 首段（runner 命中下探
  `@zwframe:` + shim portal 冒泡派发）+ 模板元素 `.id` 反射器 + R255 body attrs 补
  提取。1738P→1739P（+1）。
- **M3 尾簇 23（2026-10-06，f82ea3bee）**：portal 第二段——per-frame capture 状态机
  （pending 模型 + 隐式释放）+ body id 落视图 + 视图方法补齐 + 路由守卫。
  1739P→1745P（+6）。
- **M3 尾簇 24（2026-10-06，649ec7c54）**：portal 第三段——up 站外层捕获守卫。
  1745P→1750P（+5，pointercapture_in_frame 0→11P）。
- **M3 尾簇 25（2026-10-06，24945baa8）**：触式捕获路由（早退分支内捕获结算 + 捕获期
  move 随捕获目标）。1750P→1755P（+5）。
- **M3 尾簇 26（2026-10-06，00e59b400）**：portal target id 值面（__zwBodyId 槽 + docEl
  `.id` 反射器）+ frame-hold 实验（回退）。1755P→1755P（Δ0）。
- **M3 尾簇 27（2026-10-06，70f0e595e）**：frame-hold 重接（载体法：body 视图原型包装 +
  own .id=html id）。1755P→1757P（+2，subtest 6 ?mouse/?pen Pass；subtest 5 转具名
  Fail）。
- **M3 尾簇 28（2026-10-06，本轮）**：frame-hold 残留清零（非 hold portal up 分支）。
  1757P→1759P（+2，pointercapture_in_frame 15→17P，文件 TO 消失）。
- **M3 尾簇 29（2026-10-06，986f8294e）**：pointercapture_in_frame ?touch subtest 4 判例
  定谳（上游即 Fail——wpt.fyi Chrome ?touch 4/6 + pem.cc 隐式捕获推演，ZeroWeb 5/6
  超上游）。Δ0，portal 段收口。
- **M3 尾簇 30（2026-10-06，本轮）**：Meta 转义三认 + 跨界序 buttons 活掩码 +
  textInput 资产补齐。1759P→1763P（+4，零回归）。
- **M3 尾簇 31（2026-10-06，7c550a079）**：TextEvent 语义域 + selection key 存活面
  + 编辑键 key 名 + selectorFor 权威形。1763P→1778P（+14，零回归）。
- **M3 尾簇 32（2026-10-06，39ca800f1）**：send_keys 自聚焦 + uE006 formless
  buttonish 激活 + CE ForwardDelete + 动作 noop 键序。1778P→1782P（+4，零回归）。
- **M3 尾簇 33（2026-10-07，本轮）**：execCommand text-control 事件序 + maxlength
  哨兵 + textarea Enter 换行 + CE Enter textInput。1782P→1785P（+3；api CE 案
  TO 转具名 Fail 挂账）。
- **M3 尾簇 34（2026-10-07，本轮）**：'value' in tag-gate + CE Enter 元素 caret/
  本地树变更。1785P→1787P（+2，textInput 族 24P 全绿；单测校准 CE Enter mutation 形态）。
- **M3 尾簇 35（2026-10-07，本轮）**：wheel 源 scroll 步接通（stub/runner/shim
  四段通道）。wheel 族 3 案全绿（1F/2TO→3P）。
- **M3 尾簇 37（2026-10-07，6b8626f6a）**：layerX/layerY 反射（shim props 注册表 +
  `_zwMouseCoordInit` 派生 / native 模板派生对 + 四面单测）。layer-coords-transform
  1F→1P；HEAD 基线重跑漂移归因（总册 2059 双跑恒等）。
- **M3 尾簇 38（2026-10-07，53eee8dfe）**：UA click/dblclick 携指针坐标（up-sequence
  三分支 init dict 补 clientX/clientY + up-sequence 直驱单测）。corpus 逐条恒等
  （1790P/174F/95TO，零涟漪）。
- **M3 尾簇 39（2026-10-07，本轮）**：utils.js 资产补拉（goal 脚本 fetch_raw）
  + 解锁面三连修（constructor 身份 / composed / cancelable）。
  attributes.html file-Fail→named Pass；1790P→1791P。
- **M4 尾簇 40（2026-10-07，本轮）**：corpus 173F/96TO 挂账定稿编目（14 类
  逐案归因 + 重入条件，见「M4 挂账定稿清单」节；docs-only 零源码改动）。
  P2/P3 转核心修齐关账，余挂全部具名转移。
- **M4 尾簇 41（2026-10-07，d688d89e7）**：CANDIDATE 池收口（tilt 跨集部分给值
  互不派生 + 跨界事件携源 pointerType + 站内 release 延迟生效三态）。3F 修复
  1791P→1794P，余 5F 根因重归类进定稿清单；M4 门禁全绿（make test EXIT=0 +
  product-smoke 首跑通过）。[evidence/2026-10-07-m4-tail41.md](evidence/2026-10-07-m4-tail41.md)。

## 下一步计划

**goal 已 Done（2026-10-07 尾簇 41 收口）**：无 in-stream 必修面，仅余长线挂账
与待用户决策项，均在重入条件满足时由对应 goal/专项带案，本控制面转为归档态。

1. **长线挂账**（重入条件见定稿清单）：touch-action/滚动管线（渲染流碰头）、
   gBCR 同步布局（用户拍板 1a）、shadow DOM/slot（用户点名立项）、runner scheme
   分流（专项拍板）、testharness 交互层 TO（跨 corpus 基建专项）、`:hover`
   capture 驱动计算样式（渲染流）、多指针状态机（结构性扩展立项）。

**待用户决策清单**：gBCR 同步布局 RenderPipeline Arc 化（1a——30F 重入条件）；
declarative shadow DOM 立项（15F/3TO 重入条件）；runner `.https.html` scheme
分流专项（2F）；testharness 交互层 TO 专项（59TO——跨 corpus 基建，非本 goal 单独
可关）。pointerlock/IME/touch-events 维持排除（入口文档排除项，无需决策）。
**2026-10-08 已征询待批复**（goal 待决策巡检 msg `om_x100b634e253428a0c3401b05db3a524`；
建议 = ①Arc 化暂不立项（渲染流战役在途）②shadow DOM 暂缓并入 web-components 盘点
③scheme 分流不立项（仅 2F）④testharness TO 专项待 navigation M2 收口后评估——
四项批复前均维持挂账，不影响在途流）。
**2026-10-10 48h 跟进提醒已发（一次性，到期）**（合并消息含 multicol-2 新征询，msg
`om_x100b6390da10c06cc2463a02747595b`）：仍零回复——不回复即默认四项全部维持挂账，
此后不再催。
