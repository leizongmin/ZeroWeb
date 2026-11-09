# M2 s320 — the-dialog-element top-layer 族勘根：26 红五面分派，8×100% 组=①+②复合面，零本流施工面

日期：2026-11-07。切片号 s320（承 s319 下一步候选①：dialog top-layer 族勘根分派）。

## Solo 基线

`make reftest-upstream FILTER=the-dialog-element`：**45 案 19P/26F（42.2%）**。
目录 183 文件含 testharness 面与 -ref 资产，reftest 面 45 案。DC-14 分类：
真通过-可信 11 + 近纯色可疑 1（top-layer-remove-popover-attribute，空转绿）
+ 近似通过 7 + 不一致 26。
日志 [logs/s320-dialog-initial.log](logs/s320-dialog-initial.log)。

## 勘根结论（五面分派，26 红全覆盖）

### Face ① ::backdrop 伪元素盒零支持（14 案）

css-parser 已识别 `::backdrop`（parser/mod.rs is_known_pseudo_element 白名单含
"backdrop"，作者规则可解析存活），但 layout-engine **零 ::backdrop 盒生成**、
style-system 无 UA 默认 backdrop 样式（fullscreen spec UA sheet
`::backdrop{position:fixed;inset:0}` + dialog::backdrop `rgba(0,0,0,0.1)`）、
engine paint 零 backdrop 绘制序——三段全缺。

- top-layer-parent-{clip,filter,mask,opacity,overflow-clip,overflow-hidden,
  overflow-scroll,transform} **8×~100%**：复合面 ①+②。像素探针
  （product-smoke top-layer-parent-transform → /tmp/s320-tlp-transform.png）：
  **全视口红**——body 红背景传播 canvas（backdrop 缺失）+ dialog 被 parent
  `transform:scale(0)` 连带归零（祖先效果未逃逸）。layout-dump 探针：dialog
  in-flow block x=8 w=784 h=18 于 parent 内、**零 top-layer 提升、零 backdrop
  盒**。案型=rel=match green-dialog-and-backdrop（fixed 全屏绿 backdrop div +
  绿 pseudodialog），ZW 双侧差=整视口。单修任一面不可过：须 backdrop 盒生成 +
  dialog/backdrop 双双 hoist（逃逸祖先效果链）两步齐备。
- modal-dialog-backdrop ✗99.73%：①+②+③ 复合（UA backdrop 默认灰罩缺失 +
  dialog UA 定位未注入）。像素探针（/tmp/s320-mdb.png）：绿盒左上 in-flow
  （author top:100px 对非定位元素失效），零 rgba(0,0,0,0.1) 全屏灰罩。
- modal-dialog-backdrop-opacity ✗100.00%：①（backdrop opacity:0.5 全屏绿罩缺失）。
- backdrop-in-flow ✗2.08%：**单面案**（diff 恰=2×50×100=10000px 绿盒像素数）——
  ① + backdrop `position:static/relative` 应归一为 `absolute`（fullscreen spec
  stacking layer 归一语义）。
- backdrop-inherits ✗2.08%：① + CSS 自定义属性继承链（dialog 上 `--backdrop-bg`
  经 `var()` 进入 backdrop 样式）。
- backdrop-dynamic-style-change ✗2.08%：① + 动态类切换重估（`.green::backdrop`）。
- backdrop-descendant-selector ✗4.17%：① + 组合器匹配语义（::backdrop 非后代
  ——`#dialog-parent > ::backdrop` 绿臂 vs `#dialog ::backdrop` 红臂禁匹配）。

### Face ② top-layer hoist/绘制逃逸（9 案）

fullscreen spec #new-stacking-layer：top layer 元素 containing block=初始包含块、
高于一切 z-index、按 showModal 进栈序定序、逃逸祖先 opacity/overflow/transform/
filter/mask/clip。ZW dialog 恒 in-flow（探针实锤），提升机制零实现：

- top-layer-stacking.tentative ✗45.34%：top layer 压 z-index:3 红 div + 层内
  z-index/transform 子树正常 + 多 dialog 栈序。
- top-layer-containing-block ✗37.67%：百分比基准=初始包含块（offscreen 窄父内
  dialog width:90% 仍 90% 视口）+ 逃逸 opacity:0.3/overflow:hidden 祖先。
- top-layer-nesting ✗15.71%：DOM 嵌套 dialog 按栈序（yellow→blue→green）非
  z-index/树序。
- dialog-popover-overlay ✗18.75%：`<dialog open popover>` 与 .blocker 的
  top-layer 层叠序。
- top-layer-stacking-dynamic ✗9.38% / top-layer-stacking-correct-order-remove-readd
  ✗4.17% / removed-element-is-removed-from-top-layer ✗4.17%：动态 close/show/
  remove 的进出栈重排与序保持。
- top-layer-position-static ✗1.04%：`position:static` 在 top layer 归一为
  `absolute`（与 backdrop-in-flow 同型归一语义；UA 定位③同族面）。
- backdrop-stacking-order ✗28.86%：②序 + ①（backdrop z-index 无效语义 +
  多层 backdrop/dialog 交替序）。

### Face ③ dialog UA 定位（R-s31 既有挂账，渲染流）

style-system lib.rs:1909 注记：UA `dialog{position:absolute;inset-inline:0;
margin:auto}` + `dialog:modal{position:fixed;inset-block:0}` 实验后不注入——
待布局侧 abspos shrink-to-fit/fit-content 补齐（light-dismiss 盒外点击 9 回归
先科）。`:modal` 状态通道与 matcher 已落地（Document::is_modal_dialog）。
本轮无新增单驱动案——modal-dialog-backdrop 复合驱动；centering 族不在 reftest
面（testharness 页）。

### Face ④ :open 伪类零支持（1 案）

dialog-open-pseudo-invalidation ✗2.93%（`d.show()` 非模态 + `dialog:open{green}`）：
- css-parser：is_known_simple_pseudo_class 白名单**已含 "open"**（解析识别，
  规则不因无效被丢弃）。
- style-system matcher：无 Open 匹配臂 → 恒不匹配（:modal/:fullscreen
  「识别→运行时状态永不匹配」同型谱系）。
- dom query.rs PseudoClass 枚举无 Open 变体（querySelector 面零识别）。
**跨流协作面**：CSS 渲染侧（style-system）主导，dom 侧（本流 crate）单侧施工
无案可绿（dialog 可见但恒红）——维持整体移交，本流侧记协付跟进点
（PseudoClass::Open 变体 + Document open-state 注入，镜像 Modal 谱系）。

### Face ⑤ dialog 特例显示面（2 案）

- modal-dialog-display-contents ✗3.07%：dialog/::backdrop `display:contents`
  应按 `block` 计算（fullscreen spec stacking layer 特例 + getComputedStyle
  回读文本臂）。
- modal-dialog-generated-content ✗6.20%：dialog::before/::after abspos 生成
  内容 + ::backdrop content 面（①邻域）。

## 空转绿挂账（渲染流落 hoist 时同簇回归风险）

当前 ✓ 绿案中 4 案为 **top-layer 缺席下的空转绿**，hoist 实施时须同簇回归：
- element-removed-from-top-layer-has-original-position ✓0.20%：从未 hoist=恒
  原位，「移除还原位」空洞成立。
- top-layer-backdrop-remove-add-ordering ✓0.00%：零 backdrop 盒=零序可排，
  test/ref 双侧近空页。
- top-layer-remove-popover-attribute ✓0.00%：DC-14 近纯色审计 flag（双侧近空）。
- dialogs-with-no-backdrop / modal-dialog-in-{iframe,object,replaced-renderer,
  table-column}/sibling ✓：ref 多为原位绿盒，ZW in-flow 恰好同形
  （modal-dialog-in-iframe 0.17% 近似带）。

## 施工

无——五面根因全落渲染流域 crate（layout-engine 盒生成/提升、style-system
伪元素 UA 样式与伪类 matcher、engine paint 绘制序对端协作）或跨流协作面，
按工作面纪律不施工（s314/s316/s319 移交先例）。

## 验证

- solo 基线日志 1 份（45 案 19P/26F 逐案 diff%）。
- 像素探针 2 份（transform 全红视口 / modal-dialog-backdrop 绿盒左上无灰罩）。
- layout-dump 探针 1 份（dialog in-flow x=8 w=784 h=18、零 backdrop 盒零提升）。
- 案型源读 15 页（face 分派逐案核对 ref 契约）。
- 本切片 diff 零代码触碰——无测试门禁新增面，全量基线承 s318 终态
  （make test 80 腿 ok + 3783P/9F；reftest 15994P/1026F）。

## 跨流移交（对端 R5431 挂账㉔ 细化，不改动对端控制面）

对端 s302（00e8c764c）已承接四根（UA 定位/::backdrop 零支持/top-layer 绘制
逃逸/:open 伪类零支持）。本轮细化交付：
1. **①+② 复合面证据**：8×100% 组须 backdrop 盒生成 + hoist 双步齐备方可过，
   单面修复无案可绿（transform 像素探针 + layout-dump 双证据）。
2. **空转绿清单**：4 案 hoist 落地时的同簇回归哨兵（上文）。
3. **归一语义族**：backdrop-in-flow 与 top-layer-position-static 的
   static/relative→absolute 归一（fullscreen spec stacking layer）。
4. **Face ⑤**：display:contents→block 特例 + 生成内容面（对端账未单列，
   本轮补记）。
5. **:open 协付点**：本流 dom/query.rs PseudoClass::Open + open-state 注入
   待对端 style-system matcher 臂立项时同步跟进（单独施工无案可绿）。

## 下一步候选

1. **ol start/type/rev 列表标记值族**（grouping-content 6 真红，标记值归属
   待勘根——本流 engine 列表标记 paint 面或渲染流标记盒面）。
2. svg-img-with-external-stylesheet / srcdoc-print 两散红勘根（embedded 4 中
   剩余 2）。
3. customizable-select ~30（forms tentative 高级面）评估立项价值。
4. （挂账承前）embed fallback-image（R57）、print pagination 7 键、
   range-intrinsic-size、wbr 断行机会、br-bidi 段界跨流移交跟踪；
   zzz-probe 语料卫生归渲染流域；iframe pdf 结构性已知红不立项。
