# R246 Evidence — 17,x / 13–14,x sim 树等价深诊断（探针轮，无代码 land）

**日期**: 2026-08-25
**切片**: M4——R246(a) 17,x sim 树等价（诊断推进，修复切片记 R247）
**改动面**: 无代码改动（R246-probe 三轮探针，已清理）
**基线**: surround 1806P/34F 不变

## 一、探针设计（positionTests 阶段 post-harness，无预调用污染）

R246-probe 在 harness domTests 完成后（positionTests step 开头）对
i∈{17,13,14}, j=0 dump：双侧 `assertNodesEqual` 同款 `nextNode` walk
的逐 hop 对照（16 hop）+ 首差节点格式化（含 `UNT:<Internal>` 无
nodeType 形态标记）+ 双侧 `documentElement`/`#test` 子名 dump。

## 二、关键发现

### 17,0（`[foreignDoc.documentElement,0,…,1]` + paras[0]）

- walk 对照（hop1–7 完全一致：#document→html→HTML→P(id=a)→HEAD→TITLE→
  text），**hop8 分歧：A=foreignComment（foreignDoc 级）vs E=BODY**——
  host 侧 html 子树在 TITLE 后直接跳到 foreignDoc 级 comment，**BODY
  不在 walk 路径**。
- 但 `deA`（dump 时 `documentElement` getter 的子名）= `[HEAD, BODY]`——
  **dump 时刻 docEl 对象 = [HEAD, BODY]，P 缺失**：host surround 在
  iframe-window factory docEl 容器上**整体 no-op**（树未变更）。
- 矛盾解释：walk 起点是 `actualRoots[0]`（sc=docEl 的**surround 前引用**），
  dump 的 documentElement 是 R234 动态 getter（首个元素子）——两个
  docEl **对象不同**。host no-op 后 walk 走的旧引用树含 P（17,0 的 P
  来自 walk 侧另一引用形态），dump 走 getter 树无 P。
- **结论**：17,x host 侧根因 = iframe-window factory docEl 容器上
  `_coveredChildren`/insertBefore 链路 no-op（engine 顶层
  `implementation.createHTMLDocument` 形态 R245 单测已通，iframe-window
  形态未通——两 factory 路径行为不一致）。

### 13,0 / 14,0（`[document.documentElement,0/1,…,2]` + paras[0]）

- walk hop1–9 双侧一致（P 已插到 docEl[0] 位置——双侧树都显示了
  `[P, HEAD, BODY]` 形态的 html 子序列），**hop10 分歧：DIV#test 内
  A=P(id=a) vs E=P(id=b)**——host 的 testDiv 仍含 paras[0]（id=a，
  即 newParent 原件），sim 侧 testDiv 已从 id=b 开始（paras[0] 已移走）。
- `deA` = `[P]`（documentElement getter 此刻返回的**首个元素子是 P**，
  P 的子名 dump 只有一个条目）——**R234 动态 documentElement getter 的
  「首个元素子」语义被插入的 P 干扰**：P 插在 html 前，getter 返回 P
  而非 html（真实浏览器此形态 documentElement 亦返回首个元素子，但
  sim/walk 用的是固定 html 引用）。
- **结论**：13/14,x 根因 = host 的 newParent 移动用了**克隆路径**
  （R237 收尾的 clone 循环把 covered 子克隆进 newParent + 原件 remove，
  但 newParent 自身**原件留在 testDiv**，docEl 里的是 clone/或引用
  错位）+ documentElement 动态 getter 与 walk 固定引用的对象域分裂
  加剧 dump 歧义。

## 三、R247 修复切片（按依赖序）

1. **host surround 在 iframe-window factory docEl 的 no-op**：定位
   `_coveredChildren`/`_r237` insertBefore 链在 iframe-window factory
   docEl（与顶层 implementation factory 形态差异点）的断点——探针对
   两条 factory 路径做同款 dump 差分。
2. **newParent 移动语义**：13/14,x 的 clone-vs-move——surround 的
   newParent 须**移动本体**（spec selectNode(newParent) 后 range 指
   本体父），host 当前在 docEl 容器上留原件。
3. documentElement getter 对象域分裂（诊断工具层）：probe dump 统一
   经 sc 引用而非 getter，避免误读。

## 四、验证

- 基线复核：surround 1806P/34F（探针清理后与 R245 一致，零漂移）。
- 无代码 land → 无回归面；ranges/nodes/insertNode 沿用 R245 基线。
