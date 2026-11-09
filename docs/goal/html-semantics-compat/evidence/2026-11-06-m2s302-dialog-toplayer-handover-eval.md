# M2 s302 — 评估移交：the-dialog-element reftest 26 红页簇归属 rendering-compat 成立（纯评估零源码改动 docs-only；shim 语义面已完备、成员权序注记随移交）

日期：2026-11-06。切片号 s302（承 s301 下一步候选①）。评估结论：26 红页
全部落 rendering-compat 域（UA 定位/::backdrop/top-layer 绘制/`:open`
伪类四根），html-semantics 域内 JS 语义面已由既往切片完备、零剩余可修
钉形；移交含 shim 侧前置注记（模态栈顺序），不投机预建。纯评估零源码
改动。

## 勘根

**簇形态**（s300 全树 b2 日志）：26 页 ✗1.04%–100%，html/semantics
reftest 通道 117 红中最大本域簇。

**shim 语义面已完备**（既往切片存量）：showModal/close/requestClose/
cancel 事件/closedby/autofocus/fully-active 门/part03 :2409 接口表/
:17337 invoker command 面全在；testharness 通道 dialog 行为面已大量
收编。红页全部为**渲染消费面**，非 JS 行为面。

**四根分解**（逐页归因）：

1. **dialog UA 定位/centering（既有挂账渲染流 R-s31）**：
   style-system/src/lib.rs:1901-1916——`dialog[open]`→display:block
   （R3290）已落；UA 定位（`position:absolute/fixed; inset; margin:auto`
   居中）两轮实验回归后**显式不注入**，`:modal` 状态通道与 matcher 判定
   已落（`Document::is_modal_dialog` 镜像 :focus 注入通道；pipeline
   inject_modal_state 逐帧重注），UA 规则待布局侧 abspos 收缩/fit-content
   语义补齐后按规范回填（原文挂账渲染流）。
2. **::backdrop 伪元素（零支持）**：grep 全域仅 backdrop-filter 属性面；
   无 ::backdrop 伪元素机制。fullscreen spec UA 默认（全视口淡黑覆盖 +
   dialog 之上/页内容之下堆叠）未实现——modal-dialog-backdrop 99.73%/
   backdrop-opacity 100% 及 backdrop-* 五页全根于此（ref 页以显式定位
   div 模拟 backdrop+UA 样式实证期望形态）。
3. **top-layer 绘制逃逸（paint/compositing 架构面）**：top-layer-parent-*
   八页（clip/filter/mask/opacity/overflow-clip/hidden/scroll/transform）
   恒 ~100%——spec 要求 top layer 元素**无视祖先**裁剪/滤镜/遮罩/透明度/
   溢出/变换，ZW 按普通流内盒渲染全数命中祖先效应；stacking 三页
   （28.86%/45.34%/9.38%）、nesting 15.71%、containing-block 37.67%
   同根（top layer 堆叠序/包含块豁免）。popover-overlay 18.75% 为
   popover×top-layer 交互面（shim popover 属性面已有存量，top-layer
   成员消费归渲染）。
4. **`:open` 伪类（Selectors L4 #open-state，零支持）**：matcher 仅
   `:modal` 落地；dialog-open-pseudo-invalidation 页 `dialog:open`
   不匹配 → 作者红底生效 → 2.93% 红。

**shim 侧前置注记（随移交，不预建）**：模态成员权威=
shim 选择器 Vec（栈序）→ pipeline inject_modal_state（stale 选择器
query 落空自动剔除=移除语义已处理）→ render Document
`modal_dialogs` **无序集**（contains-only）。top-layer stacking 消费需
顺序结构存活进 render Document——本通道无观测钉形（:modal 成员判定与
顺序无关、testharness 无 top-layer 序断言面），按准则 #2/#5 不投机改
造，顺序结构化随渲染流消费面一起落。

**小差幅页（1.04%–9.38%）非独立根**：position-static/removed-element/
remove-readd/backdrop-in-flow 等均为上述四根在具体几何下的部分命中
（如 removed-element 页 ZW 流内盒恰与 ref 部分重合）——单根分解已覆盖，
无 shim-bridge 动态失效面（render 路径每帧 fresh-parse，经典失效语义
不适用）。

## 移交清单（rendering-compat 域候选输入）

1. dialog UA 定位回填（R-s31 既有挂账承继，blocked 于 abspos
   shrink-to-fit/fit-content 布局语义）。
2. ::backdrop 伪元素机制 + fullscreen spec UA 默认样式。
3. top-layer 绘制逃逸架构（祖先效应豁免 + 全局堆叠；8×~100% 族）。
4. `:open`/`:closed` 伪类（Selectors L4）。
5. 前置注记：模态栈顺序结构化（shim Vec 已保序，render Document 集合
   需改序结构——随 3 一起落）。
6. popover×top-layer 成员消费（popover-overlay 页）。

## 下一步候选（本域）

1. **forms/the-select-element reftest 41 红页簇归属勘根**（117 红中
   最大簇；select 渲染几何 vs shim 语义划界，涉 rendering form-visual
   域碰头面——勘根后按例施工或移交）。
2. the-bdi-element 13 红页簇（bidi 隔离渲染面，疑渲染域移交候选）。
3. the-img-element reftest 8 红页簇（本域元素，勘根立钉面）。

## 证据

- 数据面：s300-reftest-full-b2.log（26 页态+差幅）、语料页/ref 逐页
  核对（modal-dialog-backdrop 双页全文、dialog-open-pseudo-invalidation
  全文、popover-overlay 头 30 行）、代码锚：style-system lib.rs
  :1901-1916（R3290/R-s31）、dom/src/document/modal.rs（成员权威）、
  dom/src/query.rs:183/:509/:1448（:modal 判定）、style-system/matcher/
  mod.rs:334-335（:modal 臂）、engine/pipeline/mod.rs:565-577
  （inject_modal_state）、shim part03.js:10156+（requestClose 等）。
- grep 存证：`::backdrop` 伪元素零命中（仅 backdrop-filter）、`:open`
  零命中、`top_layer` 渲染面零命中（仅 :modal 判定通道）。
- 承前账本：s301（移交评估协议先例）、s300-reftest-full-b2.log。
