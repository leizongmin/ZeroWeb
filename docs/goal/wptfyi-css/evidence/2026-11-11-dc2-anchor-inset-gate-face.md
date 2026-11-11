# DC-2 组F 布局刀3：anchor() inset 求值 + 双介入门（anchorlay3 10,549 → 11,863 净 +1,314）

日期：2026-11-11。前序：[position-area 区域定位布局刀2](2026-11-11-dc2-position-area-layout-face.md)。
css-anchor-position-1 §anchor-pos anchor() inset 求值布局期落地——刀2 残余
表第一大桶（writing-modes-001 单页 1,296F）全清。

## 批内容（提交 405b12a3c + be4c89a60 fmt 补）

1. **layout-engine `engine/anchor_pos.rs` 扩展（apply_inset_walk 新 pass）**
   ——engine.rs 11.7c 挂点（taffy 主布局 + 刀2 区域定位 pass 后）：
   - `AnchorGeo`/`AnchorMap` 锚注册（pass1 同刀2 架构）：`anchor-name` →
     视口 border-box 几何 + 双 flag（`inline_flow` 纯 inline 流盒 /
     `in_scroller` overflow scroll/auto 严格祖先内）。树序后者覆盖；
     position-anchor 默认锚 = 显式名同口径经 `gated_lookup` 消费。
   - anchor() inset 求值（css-anchor-position-1 §anchor-pos）：物理边
     left/right/top/bottom/center、纯百分比、bare 缺省边 = f 槽属性同名边
     （`top: anchor(--a)` ≡ `anchor(--a top)`）；`<length>|<percentage>`
     fallback 第二参。CSS2 §10.3.7 约束式：双侧 inset 拉伸定尺寸、单侧
     定位（margin 委托 `resolve_abspos_real_length` 既有口径）。
   - **双介入门（`gated_lookup` 统一消费，语料实证归因）**：
     ① 纯 inline 流锚门——display:inline 跨行分片几何不可靠（fragment
       边界未建模）；inline-block 原子盒 border box 可靠，不受门。
     ② 滚动上下文失配门——锚在 overflow scroll/auto 严格祖先内而目标在
       其外时坐标系失配（引擎不建模 scroll offset，盒坐标 = 布局静态位，
       FIXME 撤门条件 = 滚移感知锚几何）；同滚上下文两侧同基允许。
     pass2 区域定位只读坐标不受门，pass3 inset 求值经门。
2. **style-system 承载**：ComputedStyle `anchor_inset_top/right/bottom/left`
   四字段（原文 `Option<Box<str>>`）+ `apply.rs` 四臂（`anchor(` 子串捕获取
   原文，`anchor-size(` 无该子串不误捕）+ Default。
3. **g1f e2e 单测 12 案全绿**（刀2 9 案 + 新 3 案：inline 流锚 fallback
   消费钉位、滚上下文失配 fallback 消费、同滚上下文正常介入形；既有
   fixture 补显式 display——`ComputedStyle::default()` display=Inline
   与真实页 UA 表 div=Block 不同形，块级锚/abspos 落树均需显式 Block）。

## 验证（g3b/g3c 链）

- **css-anchor-position 语料 253 页**：**10,549P/2,267F → 11,863P/953F
  净 +1,314，逐页零回归**——9 页增益：**writing-modes-001 单页
  0P/1,296F → 1,296P/0F 全清**（anchor() 全值域 × 8 写作模式组合，刀2
  第一挂账桶兑现）+ anchor-position-001/002/003（+1/+3/+4）、borders-001
  （+1）、dynamic-002（+2）、anchor-query-fallback（+3）、pseudo-element-anchor
  （+2）、anchor-in-scroller-with-left-side-scrollbar（+2）。
- **三轮零漂移**：results-g4（五修树）与 results-g5（fd1-gate 二进制复用轮
  + 2152 终轮）对 g4 账面 11,863P/953F 逐页零漂移；2152 终轮语料目录混入
  他流 zz-g6probe-* 探针 2 页（7P/9F，非本语料，不计账）。
- **门控账（对未门控中间轮 results-g3 11,862P）**：净 +1——inline-002
  （0P/4F→2P/2F）与 anchor-scroll-js-expose（0P/4F→1P/3F）按门设计恢复；
  代价 inline-001/003 各 −1P（单分片 inline 锚几何本可用，门一刀切），
  挂账首片段语义（§anchor-pos first-fragment）+ 行内盒几何 y 口径，下刀。
- **make test**：3875P/7F（g3c 轮 HEAD 0061c7fff；7F = js_dom_bridge
  img/svg settle + fetch/sw/websocket 既有欠账组，对 fd1 轮失败集逐名
  一致，与本刀零交集）。
- **门禁**：fmt EXIT=0（be4c89a60）；guarded clippy workspace EXIT=0。

## 残余分桶（刀3 后 top 失败页，下一步直接取用）

| 桶 | 页面 | F | 缺面 |
|---|---|---|---|
| position-area wm/dir + align/justify | align-justify-wm-dir 56 / wm-dir 44 / align-justify 14 | ~114 | 刀2 挂账原样：非 htb wm 映射 + align/justify-self 蕴含 |
| anchor() 复杂形/动态 | anchor-name-003 35(4P) / anchor-position-004 32 / anchor-inside-outside 16 / anchor-name-mutation 15 | ~98 | 锚动态增删/自对齐蕴含/containment 语境 |
| anchor-size() | anchor-size-001 26 / anchor-size-writing-modes 24 | ~50 | anchor-size() inset/fallback 求值 |
| position-area-basic 布局残余 | position-area-basic | 47 | 区域格非满填对齐形 |
| fallback 布局 | try-tactic-basic 30 / anchor-query-fallback 13 | ~43+ | 溢出检查 + last-successful 状态机 + @position-try 应用引擎 |
| 滚移感知 | scrollable-containing-block-size 13 / anchor-scroll-js-expose 3 | ~16 | scroll offset 建模（引擎盒坐标=静态位 FIXME） |
| inline 首片段 | inline-001 4 / inline-003 4 / inline-002 2 | 10 | 首片段语义 + 行内盒 y 口径（门撤条件） |
| apply.rs 数学函数 | hypot-pow-sqrt-serialize 等 9 腿 | 9 | IMPLEMENTED_VALUE_FUNCTIONS 缺 pow/sqrt/exp/log/hypot/abs/sign/round/mod/rem（css-parser 侧 eef2600da 已备，下批候选） |
