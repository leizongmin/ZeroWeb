# M2 s424 收口账——红池分诊轮：video 立项评估（挂 GB-20261114-VIDEO）+ 余件单红
# 逐件归因 + button-click-submits 施工翻绿（span 祖先激活链补齐，稳定红面 −1T）

- 日期: 2026-11-14；基线: HEAD e1ce76c9d（s423 收口），驱动: 红池分诊 CONTINUE
  指令「video 帧像素表面 FIXME 跨域大件立项评估与其余单红逐件归因」。本轮
  三段产出：①video 立项评估（独立账本 evidence/2026-11-14-video-frame-
  surface-proposal.md，挂用户门 GB-20261114-VIDEO，零源码改动）；②余件单红
  逐件归因（9 文件 solo 实测定性）；③button-click-submits 1T 勘根施工翻绿
  （纯 JS shim 单文件 part04.js，本刀唯一源码改动）。

## 一、video 立项评估（摘要，详录见提案账本）

- 勘根定谳基建大半在场（zero-media 真解码 + VideoPlayerRegistry 真播放 +
  runner 播放泵与 `__zw_get_image_wire` 同键注入），缺口三层：shim canvas
  三源消费臂 + wire 快照陈旧 / decode 驱动 resize·playing 事件链缺席 +
  settle 零维也派 resize / 语料资产 400x300-red-resize-200x150-green.{webm,
  mp4} 白名单外未 fetch。
- 分期提案 P1（canvas wire 消费臂）→ P2（stub 臂对齐 + settle 缺席 error
  语义 + 资产 fetch）→ P3（decode 驱动事件链，跨 zero-media/webview/engine，
  建议另开 RFC）。在册池收益 = resize-during-playback 2F 须 P1+P2+P3 全链；
  P1/P2 单独位移 0。
- **红形实测修正**（solo s424 补探针）：canPlayType 元素臂已真值放行
  （part03.js:32115 能力表，M4g-d + M3 切片 1），红形 = 断言①
  `videoWidth expected 400 but got 0`（资产缺席解码探针无真值，settle 链
  零维仍派 resize 不派 error——part06.js:13593 语义缺口入 P2）；「canPlayType
  恒 '' → assert_implements_optional 立即 Fail」旧判已废弃，仅存于
  part03.js:23791 interface stub 臂（P2 收窄为 stub 臂对齐）。
- **决策请求挂用户门 GB-20261114-VIDEO**（P1+P2 立项与否 / P3 另开 RFC 与
  否），不阻塞本轮其他施工。

## 二、余件单红逐件归因（9 文件 solo 实测，runner 重建 2m14s exit 0）

| 文件 | 红形（solo 实测） | 定性归属 |
|---|---|---|
| tabular-data/processing-model-1/col-span-limits.html 2F | offsetWidth 51.99985 vs 53（分数像素列宽/边框计算；IDL/parse clamp 子测 3/4/5 全过） | 渲染流（表格列 border/width 计算） |
| forms/the-textarea-element/textarea-placeholder-lineheight.html 1F | placeholder 行高 19.5 vs 19（分数 line-height 盒） | 渲染流（行盒计算） |
| forms/the-select-element/select-many-options.tentative.html 1F | 断言 3 vs 6（`appearance: base-select` 门） | 簇 D GB-20261031（customizable-select 渲染簇） |
| forms/the-select-element/select-option-focusable.tentative.html 1F | base-select 解析缺席 | 簇 D GB-20261031 |
| forms/the-select-element/select-listbox-touch.optional.html 1F | testdriver Actions addPointer 'touch'——runner 仅 label-passes touch，无 touchstart/touchend（testharness.rs:2022 注记） | runner M3 注入面（挂账先例同族） |
| forms/the-select-element/select-multiple-resize-does-not-autoscroll.html 1F | scrollHeight==clientHeight==106 断言（resize 后 autoscroll 面） | 渲染流（scroll metrics） |
| embedded-content/the-video-element/resize-during-playback.html 2F | width after first resize expected 400 got 0（资产缺席） | video 立项面 GB-20261114-VIDEO（断言① P2 半面 + 断言② P3 全链） |
| forms/the-button-element/button-click-submits.html 1T | pending:2——span 嵌 button 激活链缺失，测试无 timeout fallback → 文件级 T | **本刀施工修复（见三）** |
| document-metadata/the-style-element/style-error-01.html 1T | async_test 等待 style 元素 error 事件（`@import "support/css-red.txt"` text/plain MIME 门） | s164 FIXME 在册（error 事件臂，记账不扩刀） |

归因汇总：46 单红文件 + 15T 红池逐件定性后，本车道可施工面仅余
button-click-submits（本轮施工）；其余归渲染流（colspan/taplh/selresize/
dialog R-s31 簇/img density s416 移交/embed intrinsic sizing ×2/
video-loading-lazy/select-required-attribute/closed-details-layout-apis/
rowspan-0）、簇 D GB-20261031（selmany/selfocus 及 customizable-select/
combobox 族）、runner M3（seltouch）、navigation M3 门控（form-submission-
target 6T + links target_blank）、s137 用户门（form-nameditem）、s164
FIXME（styleerr）、在册 flake 家族（checkable-active-onblur-with-click）、
video 立项面（vidresize）。

## 三、button-click-submits 施工翻绿（本刀）

### 勘根

- 红形：12 测 10 Pass、2 async_test 永不 report（无 timeout fallback）→
  文件级 Timeout。两案 = "clicking the child of a button with .click()
  should trigger a submit"（span.click()）与 "…by dispatching a bubbling
  event…"（span 派发冒泡 click），form 监听 submit 断言 `ev.target === form`。
- 根因：两合成 click 默认动作位（part04.js dispatchEvent 位 `if
  (notPrevented && event.type === 'click')` 与 click() API 位）均只按 click
  目标自身 tag 跑激活分支（A/AREA → `_zwAnchorActivate`；INPUT/BUTTON →
  `_zwSubmitBtnDefaultAction`）——无最近激活祖先上溯。spec 激活目标 = 最近
  （含自身）具激活行为的祖先（interaction.html#the-target-element），非激活
  目标（span）嵌在 button 内时激活落祖先，shim 缺位。
- disabled 面既有口径可复用：s211 `_zwButtonEffType` / `_zwFeDisabled`
  （fieldset 传染 + lw 层 IDL setter 写面同层——语料 `button.disabled=true`
  案实测守得住，见验证 disabled 子测）。

### 施工（part04.js 单文件三处）

1. 新共享帮手 `_zwClickAncestorActivate(sel, handle, ev)`（紧邻
   `_zwSubmitBtnDefaultAction`，Annex B 声明提升，双形态走链同
   `_zwSubmitBtnDefaultAction` form 反查/`_zwInvokerCommandActivate` 精神）：
   `ev.bubbles !== true` 直接 return（激活行为沿传播路径逐节点触发——语料
   non-bubbling 案钉面）；跳过自身上溯（调用方已按目标自身 tag 分支）；
   INPUT/BUTTON 命中即算 eff type（BUTTON '' | submit、INPUT submit/image
   为 submit；reset 单独；`_zwFeDisabled` 传染判 disabled 表单控件无激活
   行为）→ submit/image 先 `_zwImgRecordClickCoords` 再
   `_zwSubmitBtnDefaultAction`，reset 走 asReset 臂；type=button 或 disabled
   均**吸收 walk**（交互内容遮蔽更远祖先，R155 interactive 判定同精神）；
   A/AREA → `_zwAnchorActivate` 后止；32 hop 上限。
2. dispatchEvent 位接线（`return notPrevented;` 前）：目标自身 tag ∉
   {A,AREA,INPUT,BUTTON} → walk（event 透传坐标）。
3. click() API 位接线（A/AREA 自身分支后）：目标自身 tag ∉
   {A,AREA,INPUT,BUTTON,LABEL} → walk（ev 透传）。LABEL 自身即激活目标
   （R155 转发面已处理，含自身语义），排除防双激活。

不改动：指针/宿主 click 链（UA 管线自带激活语义）、R313 disabled 门、
popovertarget/invoker 自走链（含自身，与新 walk 无重叠冲突——commandfor
button eff type 折叠 'button' 后被 walk 吸收）、R155 LABEL 转发。

### 验证

- 25 part 规范序拼装 node --check **SYNTAX-OK**（js_dom_bridge.rs include
  序；extract/拼装工件 /tmp/s424/shim-assembled.js）。
- solo button-click-submits: **12/12 全 Pass，文件级 T 解挂**（两 span 案
  翻 P；non-bubbling 案不提交、disabled .disabled=true 案不提交、fieldset
  传染案不提交、legend 豁免案提交——钉面全保持）。
- solo forms/the-button-element/ 目录: **57P/0F/0T**（25 文件全绿，含
  button-click-submits-with-commandfor 面——commandfor button 被 walk
  eff-type 折叠吸收，零冲突）。
- 全域 sweep ×2 join 对 s423 双基线（\x1f pairs、stable 集语义、
  (key,state) sort -u、awk hash-join）:
  - my1 vs b1: **stable=8384 state-diff=1 new=71 gone=2**——new = 2 driving
    子测 Pass + 69 select-event 子测（s423 b1 一次性 timing flake 文件级 T
    未上报子测，本复跑复原，s419/s423 在册 flake 家族）；gone = 2 文件级 T
    键（driving 解挂 + select-event flake 键）；diff = 簇 D
    select-home-end-pagedown-pageup.optional 文件键 F→T（在册闪变家族，
    s422/s423 收口同款）。
  - my2 vs b2: **stable=8454 state-diff=0 new=2 gone=1**——完全干净：new =
    driving 两子测 Pass，gone = driving 文件级 T 键，零 state 翻转。
  - my2 vs my1: **stable=8455 state-diff=1 new=0 gone=0**——唯簇 D
    home-end-detailed.optional 文件键 F→T（同族闪变对偶面）。
- 新增红 **0**；唯二 state DIFF 均为簇 D 在册闪变家族。纯 JS 轮 fmt/clippy
  不适用；引擎/Rust 零 diff。

## 口径

- 在册红 s423 收口 203F+14T（轮1）/ 201F+15T（轮2）→ 本轮终态 raw
  **202F+13T**（轮1）/ **201F+14T**（轮2）——归一簇 D 闪变（±1F↔T）与
  select-event flake（±1T 键 + 69 子测键）后稳定红面 **−1T**（
  button-click-submits 文件级 Timeout 解挂），new 红 0。
- 语料坑入册：①激活行为沿传播路径逐节点——walk 必须 `bubbles===true` 门
  （non-bubbling 案祖先不在路径）；②`_zwFeDisabled` 读 lw 层与 IDL setter
  写面同层，`button.disabled=true` 动态置禁案守得住；③pairs 提取剥状态
  token 用 substr（`$1=""` 触发 awk OFS 重建折叠子测名内多空格，~322 键
  对 s423 基线假失配——本轮烧一轮 join 重跑；file-level Timeout 键同坑：
  `t=$0; sub()` 改 `$0` 不改 `t`，须三参 `sub(re,"",t)`）。
- 用户门维持：GB-20261031（簇 D select 立项征询）、GB-20261114-VIDEO
  （video 立项 P1/P2/P3 分期决策）、s137（form-nameditem override builtins
  fix#19c 冲突）、sizes-auto F5（渲染流移交）。
- 下一步: 红池分诊续收尾轮——余池全部归域（渲染流/簇 D/M3/用户门/FIXME）
  无本车道可施工单红；下一施工候选按优先级：渲染流到货后 dialog R-s31 簇
  14 行与 img density/embed sizing 面复验，或 GB-20261114-VIDEO 决策后
  P1+P2 video 消费臂切片，或簇 D 决策后 A2 簇分诊续。
