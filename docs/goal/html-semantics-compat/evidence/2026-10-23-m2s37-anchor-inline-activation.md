# M2 切片 37 — 挂账：anchor-with-inline 导航缺口（2026-10-23）

## 范围

s20 起挂账：anchor-with-inline-element.html 文件级 Timeout——synthetic
click 落 a[href] 内 inline/svg 后代，anchor hash 导航缺口。

## 探针钉定（单案隔离；首批多案探针 onhashchange handler 竞争污染作废）

- **裸 a 后代 click 也不导航**（hash 空）——与 summary 无关。
- summary 包裹形态：导航与翻转**都不发生**。
- 根因 1（R5239 主修）：proxy 面 dispatch 的 R112 post 段 nearest
  activation 只执行 act:label/act:details/act:button，**act:a/act:area 落
  input-change 兜底**——nearest 命中 a（target 为其后代）时 anchor 激活
  从未执行。
- 根因 2（svg 案，主修后露出）：summary 豁免
  `_zwHasInteractiveAncestorBefore4009` 只认 `_zwIsInteractiveContentEl`
  （ns 门对非 HTML ns 恒 false）——**SVG ns a[href] 不被认作可激活祖
  先**，summary 误翻转（导航已通但 `assert_false(details.open)` 红）。

## spec 钉定

- activation target 就近语义：nearest activatable ancestor 的激活行为
  消化本次点击（the-a-element activation behavior：fragment 导航）。
- summary 激活条件：event target 非「interactive content 后代」——svg a
  有 activation behavior（navigate），真浏览器 nearest 消化后 summary
  不翻转（语料 svg circle/text 两案断言面）。

## 落地（R5239，part03 两处）

1. R112 post 段补 `act:a`/`act:area` 分支：执行 `_zwAnchorActivate`；
   **receiver 自身点击（target === act 站）让位**（part04 click() post
   段/R106 dispatchEvent post 段已按 receiver tag 执行，再跑会双导航）。
2. `_zwHasInteractiveAncestorBefore4009` 祖先 walk 补 A/AREA href
   presence 判定（SVG a 同面）——不扩 `_zwIsInteractiveContentEl` 共享
   函数（HTML interactive content 定义不列 SVG a，扩了改 label 等共享
   消费面）。

## 集群验证

- **anchor-with-inline-element.html：Timeout 全红 → 5P/0F**（i/span/
  svg circle/svg text/总案全绿）。
- A/B 对照（git stash 双跑）：summary 域 **REG 0 / GAIN 5**（6 files
  20 subtest **20P/0F/0T 全绿**，此前 2F/2T 全是 anchor-with-inline 形
  态红）；details 域 42 subtest **逐位一致**；label 域 99 subtest **逐
  位一致**。
- interactive-elements 全域 98 files 482 subtest **415P/65F/2T**——
  Timeout 2 文件均既有挂账（toggleEvent.html s20 起、focus-after-close
  .html s32 起）；dialog 子集对 s35 账本 **REG 0 / GAIN 1**（即 s36
  requestClose 案）。
- forms 域 401 files 4627 subtest 对 s34/s36 基线**计数逐位一致**
  （数据 [2026-10-23-m2s37-anchor-inline-activation.json](2026-10-23-m2s37-anchor-inline-activation.json)）。

## 已知边界

- 点击 details 内非激活后代（无 summary）不翻转——nearest 命中
  act:details 但 R112 分支有 `_zwUaDispatch` 门（click() 通道由 part04
  内联段承接、且内联段也只认 receiver 自身）。真浏览器会翻转。与
  anchor 案同根的邻近缺口，未在本次挂账内，记待办。
- 多案 async_test 共享 `window.onhashchange` 的探针形态有 handler 竞
  争（首批探针结论作废的原因）——语料 await 串行形态无此问题。

## 门禁

zero-engine lib **3429P/0F**；clippy `-D warnings`、fmt clean；零新增
依赖。runner release 二进制重建。R5239 撞号检查通过。
