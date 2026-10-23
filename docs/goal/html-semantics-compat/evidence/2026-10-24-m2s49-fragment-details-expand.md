# M2 切片 49（s49）——B/F 族分诊挂账 + R5253 fragment 导航 details 展开收口

日期：2026-10-24。基线 s48（425P/55F/2T）→ 本轮 **426P/54F/2T，REG 0 / GAIN 1**。

## 本轮路径

oracle 无碰撞（全域 F/T 清单与 s48 逐位对齐）→ 按上轮序进入 B 族（dialog light dismiss 5F）→ R5252 两腿模型试错回退 → F 族分诊 → R5253 收口 auto-expand 1F。

## B 族 5F 分诊（全部挂账，零净变更）

### bounds-clicking 2F → 渲染流面（并 C 族）

`dialog-closedby-bounds-clicking.html` (1,1)/(99,99) 两案红。探针实证：`elementFromPoint` 对 dialog 元素自身盒命中返回 null（modal+modeless 同断），后代普通元素正常——宿主 hit-test 对 dialog 自身盒的命中缺口是渲染流底座问题，与 C 族 top-layer 15F 同源。挂账渲染流协调面，不在语义切片单修。

### pointer-capture 2F → 指针管线深水（R5252 试错回退）

`dialog-light-dismiss-pointer-capture.html` 2F（touch/mouse + setPointerCapture）。spec 是两腿模型：pointerdown 与 pointerup 两者 target 都在链外才 close request；capture/拖拽把 up retarget 回链内 → 不关。现实现是 pointerdown 单次判定（part01 `__zw_dialog_light_dismiss_check`，唯一调用点 part06 down 序）。

R5252 落地两腿模型后三洞：
1. backdrop-receives-events 案红——s30 宿主 hit-test 把栈顶模态外落点 retarget 成 dialog 自身，contains=true 短路，down 腿漏记 pending；
2. pointer-capture 第二案仍红——第一轮 capture 键未隐式释放，up 腿用残留 capture 误救回；
3. corner-cases 页面永挂（10 子测试 1F/8P/1T，modal 组入口卡死）。

**决策：git checkout 回退 part01/part06**。修洞需系统性指针管线契约（down/up 腿状态机 + capture 生命周期隐式释放）支撑，非单切片可安全收口。挂账定性：指针管线深水——两腿模型 + capture 隐式释放，需先有指针序列契约测试面。

### corner-cases 1F → 既有挂账

`Pressing escape, when focusin calls showModal, modeless`——与全域基线一致（modeless ESC + focusin showModal 竞态），30 子测试 29P/1F。维持既有挂账。

## F 族分诊

- `closed-details-layout-apis.tentative.html` 1F：closed details 内 content-visibility 布局结果——渲染流面（closed details 内容不进布局树），挂账渲染流协调。
- `name-attribute.html` 3F：mutation event/toggle event order、connected-in-xhr-response、connected-in-template——依赖 mutation events、XHR 文档解析、template 附着面，均为未实现底座，非 details 语义本身。
- `toggleEvent.html` 1T：文件级 Timeout，9 子测试全 Pass 后 completion 未触发（疑似测试尾段收敛循环）；本轮未深查，挂账待分诊。

## R5253：fragment 导航 details 展开步（GAIN 1）

WPT `auto-expand-details-element-fragment.html`：`location.hash = '#target'` 而 target 落于 closed details 内——spec「scroll to the fragment」要求先展开 target 链上未 open 的 details 祖先（target 自身是 details 则开它），展开后才有几何可量。
[HTML §scroll-to-the-fragment](https://html.spec.whatwg.org/multipage/browsing-the-web.html#scroll-to-the-fragment-spec)

**刀一（part02 `_scrollToAnchorForHash`）**：几何量测前插展开步——target 链逐个对未 open 的 details 祖先 `setAttribute('open', '')`（走既有属性通道，toggle 事件 / name 排他组链自动接上）。

**刀二（rAF 补偿臂）**：展开是属性 mutation，批应用后才有新布局——同步链内 gBCR 拿旧几何，scrollIntoView 落不了位。仅在真正展开时挂 `requestAnimationFrame` 重入 `_scrollToAnchorForHash`：重入时展开步 no-op（全已 open，不再挂臂），既有 scrollIntoView/窗口位同步按新几何落位。不涉 closed details 的既有滚动面零变化。

### 契约（part80 `fragment_nav_expands_details_r5253`）

钉 (a) hash 导航展开 closed details（翻转：展开步被撤 → closed，toggle 链不触发）；(b) target 链外 closed details 不被误开（展开步限 target 祖先链）；(c) 无匹配 hash 不炸。

**契约环境限制注记**：滚动落位（pageYOffset 离开 0）不在契约断言面——bridge 契约 sandbox 无帧驱动（rAF 同步 stub 时 mutation 未 apply）也无布局供给面（开 `__ZW_RAF_FRAME_DRIVEN` + 手动 `__zw_raf_tick` 仍落不了位，details 展开后几何不可量），该面由 WPT 单案承载（runner RAF_FRAME_DRIVEN 已 Pass）。契约 (b) 曾误写成「裸 target 滚动落位」——同因不成立（契约环境任何锚点滚动都不落位），改为几何无关的「链外 details 不误开」断言。

### 验证

- WPT 单案 auto-expand：Pass（GAIN 实证）
- engine lib：3445P/0F（含新契约）；fmt --check 干净；clippy -p zero-engine 0 告警
- 全域 interactive-elements：426P/54F/2T vs s48 425P/55F/2T——REG 0 / GAIN 1（auto-expand 翻 Pass，其余逐位对齐）

## 下一步候选

按挂账序：A 族 closedby×slot 16F（Chrome-parity 同红钉住）或 H 族前置 4F（declarative shadow parser+渲染流联合切片）；toggleEvent 1T 分诊可作小切片插入。C 族渲染流 20F、指针管线两腿模型需跨子系统协调，单独排期。
