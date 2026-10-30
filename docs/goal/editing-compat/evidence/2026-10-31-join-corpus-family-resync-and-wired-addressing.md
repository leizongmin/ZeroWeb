# editing-compat — join 语料第二动作 no-op 根因（双 wrapper 家族视图滞后）+ plain 寻址补全（2026-10-31）

承接 [white-space join 簇实现落地 + 语料管线阻塞发现](2026-10-30-ws-join-impl-and-pipeline-blocker.md)
（D1/D2/D3 编号沿用）与 [自旋四案簇 A 修复后复测收口](2026-10-30-cluster-a-r263-ring-gate-fix.md)。
本轮交付：join-different-white-space-style 三案语料 **0P/7,518F → 3,108P/4,410F**
（41.3%），并落地 R380 fused-view 双计修复。核心发现：CE 动作与页面脚本
分属**同 context 内两族 wrapper 对象图**，query 家族子树视图只在经由本家族的
wired 写时前进——join 语料「同页第二动作起成片 no-op」的完整根因链。

## 1. R380：用户动作私有批 apply 后换代通知缺失（双计根因）

`execute_dom_script` executor/sandbox 两分支的 apply 后均无换代通知，动作
resolve 后的立即读命中「host 快照已含子 + overlay 副本」双计（WPT editing
join 语料 sendBackspaceKey → assert innerHTML 的 span 逐测试递增实证）。
修复：apply 后补 `__zw_apply_generation_bump`（user_actions.rs 两分支对偶，
与 apply_mutations_subset / apply_pending_shared_mutations 尾部同契约，
R379/pa2b 先例）。

## 2. 双 wrapper 家族模型（H 探针实证链）

同一 JS context（canary 相同实证）下存在两族 wrapper 对象图：

- **页面家族**：语料页 `const editingHost = document.querySelector(...)`
  持有，`setupEditingHost` 的 innerHTML setter 经它更新子树；
- **query 家族**：CE 动作 `document.querySelector(sel)` 产物，子树视图只在
  经由本家族的 wired 写时前进。

页面 setup 走页面家族后，query 家族滞留**上一动作形态**（H2 实证：kid1 =
上轮动作的 shell span、孙代 plain 无写印）。fused 序列化（innerHTML
getter）读引擎态恒 fresh，掩盖家族陈旧。动作 walk 在陈旧树上早退
（g6:src-not-last 守卫 / recorded=0）→ join 语料同页第二动作起 no-op。
此前多轮假设逐个证伪：class-vs-style 差异（实为动作顺序）、页面树残留
span（Z2 divNext=null）、跨 context 双沙箱（canary 相同）、query 世界永久
陈旧（多键流在 query 世界内自洽）。

## 3. 修复链（part01.js 为主）

1. **`_zwCeCaretAnchorStale` + remap 门拓宽**：plain（无印补偿域）之外，
   stamped 但锚落点非原对象（`_zwFragSelPathOf` 锚在当前文档解析出不同
   节点）同样触发 `_zwCeReresolveCaret`——joined 语料二轮扩面（zw-join-probe-e
   Y2 实证）。live stamped 保留原 range（同轮重解析无收益）。
2. **`__zw_ce_caret_range` 世界同步**：raw 选区锚链上的 contenteditable
   宿主（页面家族）非 el 本体时，以其 innerHTML 回灌 el 子树——两家族
   fused 序列化同源，内容一致时为幂等重建；同世界 caret（连续键入流）
   contains 命中零开销跳过。
3. **`_zwPosSelOf`（part03）+ insertBefore plain-ref 臂（part04）**：世界
   同步后 el 子树为 innerHTML 解析产物（plain，无 `__zwSelector` 印），
   wired insertBefore 的 `refNode.__zwSelector` 分支门全不命中 → 落位静默
   本地化（壳 span 建成但引擎丢失，probe-h4 recorded=6 无 Insert 实证）。
   `_zwPosSelOf` 自 ref 上行至首个 sel 祖先按**元素序**（`:nth-child` 口径，
   与引擎 structural fastpath 计数一致）渲染位置链，引擎
   `find_by_selector` 按当前树态解析——与 RemoveChildAtBySelPath /
   SetChildTextByPath 的 plain 寻址对偶，补 insertBefore 半边。
4. **join plain 壳引擎收敛（part01 `_zwCeJoinApply` 落位段）**：整壳迁出
   产物（无 handle/sel 印）经 universal 落位只做本地化 → 引擎侧只剩提取
   remove、壳内容 lost（fix6 363 案回归实证，见 §4）。补序列化落地：
   ref 可寻址 → InsertAdjacentHtml(beforebegin)；否则 dst 位置串 +
   beforeend。引擎侧「提取 remove + copy」= move 语义；handle 壳走既有
   wire，跳过防双插。

## 4. 回归对照（世界同步的连带面）

fix6（修复 1-3 后）全套 1,701P 但 left-paragraph-and-right-line 的整壳迁出
族 **363 案回归**（fix4 通过 → fix6 actual 丢壳：`<div>abc</div><span>[]def</span>`
→ def 消失）。根因即 §3-4 的 plain 壳缺口——该族走单元素壳分支（shell =
迁出的 span 本体，plain），append 无法寻址。修复 4 落地后全套对拍清零
（§5：对 fix6 +1,407P 零回归），该语料单文件 255F（对照 fix6 全套期间
同文件 1,153F 量级）。

## 5. 量化账（join-different 三案全套，test-guard + ZW_SCRIPT_TIMEOUT_MS=300000）

| 轮 | 内容 | P | F |
|---|---|---|---|
| 基线 | 2026-10-30 文档账 | 0 | 7,518 |
| fix1 | R380 bump + caret remap 门 | 1,044 | — |
| fix4 | join 壳形态/行界迁出 + ④臂 | 1,278 | 6,240 |
| fix6 | + 世界同步 + 位置串 insertBefore | 1,701 | 5,817 |
| **fix7** | + plain 壳序列化收敛 | **3,108** | **4,410** |

fix7 终账（对拍 fix6 逐测试 ID）：**+1,407P / 零回归**；对拍 fix4 基准集
亦零回归。分方法：backspace 502（fix4）→ **1,228**、forwarddelete 406 →
**940**、select-boundary 370 → **940**——forwarddelete/select-boundary 面
同样消费 `_zwCeJoinApply`（跨容器空界选区臂），plain 寻址补全两臂同收。

## 6. 剩余长尾（本轮不改）

- forwarddelete / select-boundary 面 ~800F/面：多为 `<b>` 等行内壳变体与
  真跨块选区 defer（2026-10-30 文档既有账）。
- `<b>[]def<br>ghi</b>` 族 backspace：walk 的行界扫描以 `<b>` 为行容器但
  提取后 join 形态不在 accept 集（1 案抽查）。

## 7. 验证与回归门

- shim 全量拼接 `node --check` 语法通过（bridge 顺序 part01→part08d）。
- 探针 H（临时页，已删）：H1/H2 双动作批 recorded=7 含 InsertBefore，
  实际 DOM 双动作一致且形态正确。
- `make test`（DISPLAY=:20，test-guard；本轮编译期峰值插曲：`cargo clean -p
  zero-engine` 清 incremental ICE 后全量重建在 16 路并行下峰值 >25GB 连杀
  四轮、缓存反复中毒，最终 `CARGO_BUILD_JOBS=2` + 标准 8GB 守卫通过）：
  **8,889P/9F**——9 失败与已知红集逐一对应（fetch/Request/Response/
  iframe-service-worker 域），零新增。
- `cargo fmt --all --check` ✓、`cargo clippy -p zero-webview -p zero-engine
  --all-targets` ✓ 零告警。

## 8. 撞车合并（rebase 遇并行流 D1/D2/D3 落地，2026-10-31）

push 前 `git pull --rebase` 撞上远端并行流 be4874b99（marker 语料管线
D1/D2/D3 独立落地，与本文 §3 修复链同源异构）。语义判定后合并：

- **D1 数据写通道统一远端**：enum/apply/rewrite 六处取远端命名
  （`SetChildTextByPath{anchor_selector}`/`RemoveChildAtPath`），删除本文
  两个变体与 wire 注册；part03 两处调用点（`_zwD1PersistData`/
  removeChild 穿透）改用远端 wire 名，守卫补 `__zwSelPathWrite` 印排除
  （远端 stamping 域走其 accessor，kill-switch `ZW_SEL_PATH_MUTATION=0`
  下远端不 stamp、本路径照常兜底——双域互斥不双写）。本文
  `_zwFragSelPathOf` 保留（D3 依赖，远端无此名）。
- **D3 两级重锚并存**：`__zw_ce_caret_range` 内远端登记表身份重锚
  （`_zwCeReanchorRange`，D1 stamping 域）优先，未命中回落本文 sel-path
  重解析（`_zwCeReresolveCaret`，plain/陈旧补偿 proxy 域）——两级各覆盖
  对方盲区；本文世界同步块仍居两级之前（先同步 el 再重锚）。
- `_zwCeJoinApply` 冲突 hunk 取本文完整版（单元素壳 + innerHTML 物化 +
  源位落位 + plain 壳引擎收敛，远端侧为旧 handle 子修复）。

合并后复验（shim `node --check` ✓）与撞车仲裁：

- **复跑测得 1,395P/6,123F（对 fix7 -1,713P）**——该账混入并行中间轮同树
  在途 WIP（双方共用 target-editing 构建树），不能作为本合并的独立验证账；
  但暴露的两类交互缺陷为真：
  1. **登记表重锚 × 世界同步换代**：`_zwCeReanchorRange` 的登记表 tops 是
     setup 代 regIdx，世界同步重建 el 子树后位次指向换代前——一级重锚把
     陈旧锚映射到错位节点，交由二级 sel-path 重解析才位置正确；
  2. **D1 stamping × 回灌子树**：世界同步 `el.innerHTML=` 回灌触发远端
     stamping 钩子，stamp 域访问器的移除覆盖对回灌子树有缺口（行界 br
     残留引擎树，fused 序列化放大）。
- **仲裁由并行中间轮落地（同树在途，2026-10-31）**：`__zw_ce_caret_range`
  世界同步发生的本轮调用跳过一级重锚（`_zwSynced` 门）+ 回灌期间
  `__zwCeWorldSyncing` 抑制 D1 stamping（part04 钩子门）——回灌子树归
  plain 补偿域，走本文二级重解析与 D1 路径。合并终验以组合态为准
  （并行轮 owns 该验证轮）。
- **make test 口径同注**：本方合并态 make test（同树含并发 WIP）测得
  engine lib 3,611P/11F——已知红集 9 之外 +2F（`r113` style-sheet、
  `s24_flip` ua-display），结构推理归属并发 WIP（`computed_style.rs` 正被
  其编辑，display 计算域与本文触碰面零交叠；编译通过即排除测试引用已删
  变体），归因由并行轮组合态验证收口。
