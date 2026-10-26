# M2 切片 52（s52）——R5257 focus 滚动进视口 + 视口 scroll document 级派发；R5258 Element.moveBefore

日期：2026-10-24。基线 s51（431P/51F/1T，483 子测试）→ 本轮 **433P/50F/0T（483 子测试），REG 0 / GAIN 3**。

## 刀一：focus 滚动进视口（focus-after-close 文件级 Timeout 收口）

`focus-after-close.html` 文件级 Timeout——7 子测试 6 Pass，唯一悬挂是末腿「Focus should not scroll…outside viewport」：`outViewPortButton.focus()`（`style.top=innerHeight+10; position:absolute`）后等 document 的 scroll 事件（要求 scrollTop>0）永挂。探针实证双根因：

1. **proxy focus() 无滚动步**——spec focusing steps 尾 UA 滚动面（CSSOM View scroll a target into view）缺失；
2. **视口 scroll 事件只派 window**（R3051 `_zwFireScroll` window 臂）——`document.addEventListener('scroll')` 永收不到（spec 视口滚动事件 target 是 Document）。

修法（part04 focus 尾 + part01 `_zwFireScroll` + R4600 包旗）：

- **focus 尾滚动步**：gBCR 出界才滚（nearest：上出界顶对齐、下出界有高底贴齐），复用 `globalThis.scrollTo`（R3047 `_winScroll` + R3051 事件序自洽，documentElement.scrollTop 窗口镜像读回）。
- **几何 fallback**：动态创建元素（createElement→appendChild）无布局几何（runner gBCR 零盒、宿主 gBCR 空串探针实证；offsetTop 同源 `_layoutRect` 亦零）——读 proxy **inline style**（JS 侧 cssText，handle-only 可达；宿主 computed style 仅收 selector）近似：position 非 static 且 top 可解析 → top 即文档 y（abspos ICB 近似）。顶对齐兜底（R3060 center/end≈top 同款 documented 近似）。FIXME: 样式表定位不在近似面，随渲染流样式几何底座补。
- **no-scroll 变体**：`__zwFocusNoScroll` 旗——R4600 dialog 内部 focusing steps（`_zwDialogRunFocusingStepsR4600` 执行器 + inert fixup body.focus()）包旗不滚（Chromium close 恢复不滚，同测试后半 scrollTop==0 断言面）。
- **document 级派发**：`_zwFireScroll` window 臂补 `_dispatchWithBubble('html',…,'doc')`——非冒泡事件 bubble 腿 `event.bubbles` 门控（part03 实证），window 监听不双计。

## 刀二：Element.moveBefore（movebefore-setup 1F 收口）

`dialog-open-movebefore-setup.html` 1F：`new_parent.moveBefore(dialog, null)` TypeError（API 未实现），后续断言全灭。spec moveBefore = 免 removal/insertion steps 的树内移动——close watcher 组、top-layer、dialog open 态**原地保持**（本 shim 的 insertBefore 本就无 dialog 删除钩子，移动面天然与 spec 可观察语义一致）。实现（part04 trap）= WebIDL 校验（param 1 非 Node TypeError、param 2 非节点非 null TypeError）+ 委派既有 insertBefore 机械（R296/R300 pre-insert 校验族 + 宿主 wire + registry 记账 + MO childList 全继承）。FIXME: spec 同文档门（跨 Document → HierarchyRequestError）未加，语料无钉。

ESC 面既有（R5236/R4300）：两 dialog 均显式 `closedby=closerequest`、show() 各压组栈 → moveBefore 不触组栈 → ESC 关组栈顶 second、first 保持——测试断言链全通。

## 契约（part83 `focus_scroll_into_view_movebefore_r5257`）

(a) 视口外 focus → nearest 滚动位（几何 stub gBCR 440）+ document/window 各恰一次 scroll + scrollTop 窗口镜像；(b) showModal/close 内部 focusing steps 全程零滚动 + close 恢复原焦点；(c) moveBefore 归属新父 + 双 dialog open 原地保持 + `__zw_esc_dialog_cancel()` 关组栈顶 second；(d) refNode 非本父子 → NotFoundError；(e) 全程仅 (a) 一对 scroll 事件。首版 (d) 腿测试自身错臂（ref 取了已是子的 d1）当场修正为 `moveBefore(ob, ob)`。

## 验证

- focus-after-close **Timeout → 7 子测试全 Pass**；movebefore-setup **1F → 1P**
- engine lib：**3448P/0F**（+契约 part83）；fmt --check 干净；clippy 0 告警（test-guard 包裹）
- 全域 interactive-elements：**433P/50F/0T（483 子测试）对 s51 逐位 REG 0 / GAIN 3**（python 逐位 diff，仅 4 行变化全可归因）
- release runner 含 R5257 锚点 ×4 / R5258 锚点 ×1（grep 实证）；探针页已删除

## 余账

- **declarative shadow 联合切片**（原 A 族 16F + H 前置 4F）：待用户点名协调 rendering-compat
- C 族渲染流 20F（abspos-dialog-layout 6F / dialog-overlay 2F / default-color 2F 等布局/绘制面）、B 族指针管线深水、F 族 name-attribute 3F 维持挂账
- focus 滚动的样式表定位近似面（FIXME）随渲染流样式几何底座；moveBefore 跨文档同文档门（FIXME）随 declarative shadow/parser 批次
