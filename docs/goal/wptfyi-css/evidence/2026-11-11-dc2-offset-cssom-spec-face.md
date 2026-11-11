# DC-2 组F 布局刀1：offsetParent/offsetLeft/offsetTop CSSOM View spec 化（anchorf2 10,455 → anchorofsom 10,485 净 +30）

日期：2026-11-11。前序：[anchor 族声明面文法批](2026-11-11-dc2-anchor-family-decl-face.md)。
CSSOM View offset 族从近似语义升级为 spec 算法——旧实现 offsetLeft/Top 恒
viewport 相对、offsetParent 恒 body proxy，相对语义断言面（anchor/
position-area 语料的 offset 断言页）全挂，本刀升级解锁。

## 批内容（提交 cf5e8d0a2，rebase 后 5af93e7e0 前身）

1. **shim part05 `_zwOffsetParentSel`/`_zwOffsetCbEstablisher`/`_zwOffsetOffset`
   （新增）**：offsetParent spec 算法（cssom-view-1 §dom-htmlelement-offsetparent）
   ——最近 positioned 祖先（position != static）/ CB 建立者祖先
   （transform/perspective/filter != none、contain paint|layout|content|strict）/
   最近 td/th/table 祖先，单遍 walk 取最近命中（64 深度守卫）；fixed 自身
   CB=viewport → null（链上 CB 建立者命中除外——fixed 不被 positioned 祖先
   包含，WPT offsetParent-fixed oracle）；html/body → null（R4003c 死循环防护
   保持，walk 守卫双保险）；detached/hidden（无 rect）→ null 语义不变。
2. **shim part05 `_zwOffsetOffset`**：offsetLeft/Top 从恒 viewport 相对改为
   相对 offsetParent padding edge（减 op border 宽）+ round 最近整数
   （subpixel-sizes-and-offsets 钉 5.5→6）；fixed（op null）→ viewport；
   op 无 rect 或宿主回调缺失回落 viewport 近似（兼容臂）。
3. **shim part04**：offsetTop/Left/Parent 三臂接线（offsetParent 臂删旧
   body proxy 注释段，spec 算法注释 + 规范链接）。
4. **bridge 测试**：part01 `test_layout_geometry_e2e` mock 语义同步（mock 下
   #d 与 body 同 rect、body 无 border → 相对偏移 0,0）；part60 新增
   `test_offsetparent_spec_positioned_walk_and_relative_offsets`——walk 关键字
   分支钉面（positioned 命中 `#rel`/td 命中 `#cell`/fixed null/body null）
   + 相对换算（child 30−10、50−20）+ fixed round（5.4→5、6.6→7）。
   transform/perspective/filter/contain 建立者判定面由 WPT
   offsetParent-fixed/offsetParent_element_test 语料覆盖（lib 沙箱 mock rect
   无布局树，同一 walk 判定逻辑 lib 侧钉关键字分支）。

## 验证（anchorofsom 轮，14:22）

- **css-anchor-position 语料 253 页**：**10,455P/2,361F → 10,485P/2,331F
  净 +30，逐页零回归**。十六页 improved（全部 0→N 翻正或 F 收敛）：
  anchor-center-{htb,vrl}×{htb,vrl} 四写作模式组合页 0→3×4（+12）、
  last-successful-{animation,basic,change-fallbacks,change-try-rule}
  0→2×4 + intermediate-ignored 0→1（+9）、anchor-function-chain 0→1、
  anchor-position-inline-002 0→2、anchor-position-multicol-001 0→1、
  anchor-size-001 0→2、mixed-dependency-chain 0→1、
  position-area-ambiguous-self 0→1、position-area-chain 0→1。
- **css-color 漂移门**：10,905P 零漂移（64 页 case 数不变）。
- **顺序依赖排查**：last-successful-basic.html solo 复跑两轮
  （results-probe-flaky1/2）均 2P/1F，与全量内一致——提升稳定非顺序侥幸。
- **make test**：3866P/7F（7F 为 main 既有欠账组 be6703d58 定谳——img/svg
  settle + fetch/service-worker/websocket 族，与本刀零交集）。
- **门禁**：fmt EXIT=0；guarded clippy workspace EXIT=0
  （logs/clippy-g2final-*.log）。

## 残余与边界

- last-successful 族四页残余各 1F（'Base position without fallback now
  successful'）——需 position-try fallback 布局状态机（刀2 已挂账 fallback
  不介入门）。
- offsetWidth/Height 仍 border-box 近似（scroll 面未动，本刀范围外）。
