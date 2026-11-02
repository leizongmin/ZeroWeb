# R5397 归因轮：clearance × 塌穿机器两案取证（未修复，挂账）（2026-11-03）

本轮无代码变更（R5396 收口后按账本续扫，两案归因均落深结构方向，按协议记档跳过）。

## 案一：negative-clearance-after-bottom-margin.html（✗2.55%）— chromium 语义全分解

取证：REFTEST_DUMP 双页 + chromium headless oracle 探针 ×3（基础几何 / elementFromPoint /
变体 mt=300 定谳负 clearance 行为）+ ZW 临时布局树探针（用后即删）。

chromium 实测几何（probe2，body margin:0，相对 float 顶）：

- outer h=150、P h=150；float rel [0,100)（border-top 50 white + content 50 green）
- **inner rel [50,100) h=50**；empty border box rel [50,50)；cleared rel 100；bb rel [100,150)
- 可见合成（z-index:-1 层叠上下文内 float 绘制序高于 in-flow 块背景）= 方块 [50,150) ✓
- ZW 实绘：inner [99,198)、bb [198,248)、float green [50,100)+white 盖 [0,50)
  → 左带 [50,248)、右带 [99,248)（与 PNG 量化吻合）

chromium 语义链（四条，缺一不可）：

1. **§8.3.1 兄弟塌穿打断**：「bottom margin 与下一 in-flow 兄弟 top margin 塌穿，
   **除非该兄弟有 clearance**」——empty mb:49 与 cleared mt:98 不合并。
2. **自塌缩首子 margin 逃逸父上方**：empty 自塌缩（mt0∪mb49=49）经 inner（无
   border/padding、auto 高、首子链）逃逸到 inner border box 上方 → inner 顶 =
   P content 顶 + 49 = rel 50；empty border box = inner content 顶 = rel 50。
3. **负 clearance 不拉上（定谳实验）**：cleared 假设位 = 50 + 98 = 148（已过
   float 底 100）→ **留在假设位**。变体探针（cleared mt=300）：cleared = 301 =
   P content + 300（=collapse(49,300) 假设位）≠ 100 → 证实「假设位已过 float 底
   则不引入 clearance」（CSS 2.1 §9.5.2「hypothetical 不 past floats 才引入」）。
4. cleared 落点统一公式 = **max(容器 content 顶 + collapse(前兄弟 mb, 自身 mt), float 底)**
   （案一 max(1+98, 100)=100；变体 max(1+300, 100)=301）——与 ZW 现有 C1/C2 分支
   `clear_bottom.max(uncollapsed_pos)` 同构；ZW 该臂本身正确。
5. 父高合成：cleared mb(0) ∪ inner mb(0) 底侧塌穿出 → inner h = cleared border
   底 − inner 顶 = 50；bb [100,150)；outer/P h=150。

**ZW 缺口定位**：主缺口 = 规则 1（塌穿未在 cleared 兄弟处打断）连锁——ZW 把整条
塌穿链值（collapse(49,98)=98）作为逃逸量下推 inner（应为 49），且 cleared 落点因此
偏离。修复须动**共享兄弟塌穿语义**（taffy 预布局 + float adjust + containment +
高度合成跨层），牵动 margin-collapse 全族绿案基线（033/034/035/104/125/142/157 等
谱系都在现行为上调优）——深结构方向，挂账待专门轮次。

## 案二：margin-collapse-104.xht（✗4.17%）— 初始签名

取证：REFTEST_DUMP（chromium oracle 未跑，签名已足够定性深浅）。

- 案面：div.control（h=5em, mb=−5em, red）+ div.test（float:left BFC, green）内
  唯一空子（margin 5em 2.5em, red）。
- 期望：float BFC 内空子自塌缩 margin **-contained**（§8.3.1「establishes BFC 的
  元素 margin 不与 in-flow 子塌穿」）→ float 高 = 塌缩 margin 100、顶 = 流位
  （control 顶），绿方块盖住红 control。
- ZW 实绘：红 [50,150) + 绿 [150,250)——float 被子自塌缩 margin 推下 100 =
  margin 逃逸到 BFC float 上方（BFC 违例），float 高 100 本身正确。
- 与案一同根域（自塌缩 margin × 容器 containment），且涉 float 子高/位合成
  （taffy 预布局侧），同判深结构挂账。

## 下一步建议

floats-clear 族择轻重排：clear-applies-to-014（7.36%）、floats-031（3.89%）、
floats-145（2.06%）先侦察；本案族（负 clearance / BFC 子 margin 逃逸）待专门
轮次按上文语义链逐规则落臂（规则 1 为根，规则 2/4/5 顺势）。
