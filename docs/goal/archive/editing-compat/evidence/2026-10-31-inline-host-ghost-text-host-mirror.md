# inline-host 跨子测文本渗漏根修 + R383a 桶级跳并域收缩（join 回归根修）

**日期**: 2026-10-31
**前置**: [attr 门落地](2026-10-31-attr-gate-land.md) §6 挂账 2（inline-host 800F 残余簇——① 跨子测文本渗漏）；
本轮同时收口 attr 门轮遗留的 join 三案回归（该轮门禁未含 join 三案）

## 1. 缺陷 A：followed-by-text 跨子测文本渗漏（56045919e）

### 归因（zz-probe-abc 五步探针，修前态 t0/t2/t3/t4 F）

探针 `tests/wpt-runner/wpt-data/editing/other/zz-probe-abc.html`（gitignored 工作件）
复现语料 `followed by text` 族的 finally-remove 渗漏：t1（语料同流程）Pass，但后续
子测/纯读/强制 apply 后 `abc` 恒在（t2/t3/t4 F）；t0（同 turn append+remove）也 F。

机制链：`container`（div，sel 父）下 `createTextNode` 句柄化文本 append → pending 批
apply 落宿主基座；finally `text.remove()` 走 part04 R129 ChildNode.remove **sel 父形态**
（文本句柄挂主文档元素），旧实现仅 `_zwMarkRemovedHandle` + `__zw_remove_handle` 单
wire。RemoveHandle 臂（`js_dom_bridge.rs` apply 段）按 `handles.get(&handle)` 解析
NodeId——文本已落基座后跨批，句柄表换代后解析失败 → `if let` 温和 no-op → 宿主树
永久滞留幽灵文本。后续子测 `container.innerHTML` 融合序列化读基座，全部带尾巴。

### 修复（part04.js R129 sel 父形态）

`__zw_remove_handle` 之后补发 `__zw_remove_child_at(父 sel, 基底索引)` 宿主镜像——批
独立寻址，跨批可解析：

- **索引**：`_zwChildBaseCache` 基底缓存扫（nodeType+data 匹配；基座条目无句柄身份，
  同 data 重复文本取首见，与解析文本路径同弱序）；**仅基座命中才发**——未 apply 的
  pending 文本基座查无，误发会按失配索引删无辜兄弟。
- **代际门**（`_r129Bkt.stamp !== _zwApplyGeneration()`，R383 同款判据）：同代（同 turn
  append+remove，宿主往返前）以 RemoveHandle 同批解析为主，补发冗余且有害——t0 实证
  `with-xyz` 的 innerHTML 读触发融合物化，基底缓存含 XYZ 重放虚影，按虚影索引补发使
  after-remove 复现 XYZ。engine 纯测试环境代际恒 0（stamp 同 0）恒同代 → 全块跳过，
  Range/MO 单测面零扰动。
- **视图补偿不落**（有意）：sel 父形态桶/反链保持原状——remove 时点的桶级记账改变
  surround leaf-newParent 路径中间态视图（R294 两版 A/B 实证 r259 `bodyLast` 翻红，
  见源内注记）；宿主基座半区已消除语料幽灵源，桶残留仅身份记账。

## 2. 缺陷 B：attr 门轮 join 三案回归（cb3cff932）

### 归因（bisect + 单案探针 + ZW_DBG_LOG 打点）

本轮按惯例复跑 join 三案验零回归，得 4,959P/2,559F（基线 6,330P/1,188F，−1,371P）。
WIP stash A/B 逐位同值 → 非本轮 WIP；`22b4f1f2a`（attr 门落前）复跑 = 基线逐位复原 →
**回归引入点 = daf3bc4c5（attr 门轮）**——该轮门禁（white-space-style/cannot-have-div/
engine lib/dom/selection 电池）未含 join 三案。新败 1,371 行**全部**集中在
`join-different-white-space-style-left-line-and-right-paragraph`（三方法各 +457，
页级 35-59/59 全 F 形态），comm 对照 P→F 0、F→P 0。

zz-probe-lrlp 单案探针（`abc<div class="nowrap">[]def<br>ghi</div>` backspace）单子测
Pass、同页三轮循环 r2 起 F（got=pre-join 基树残影）→ 页内跨子测状态累积型。打点
（`ZW_DBG_LOG=1` + 临时 R380closed/R383skip 埋点）实证：

```
R383skip sel=…body>div:nth-child(1) stamp=3 gen=4..10 a=1 added0=SPAN:def
```

R383a 桶级早退在 runner 配对环境（游标闩恒真）对**所有** stamp 滞后桶跳并——而
apply 后桶保留 identity 记账（pa2b 不清桶）stamp 必滞后。r1 的 join span 落基座后
条目仍在桶（stamp 3），r2 setup 换代（gen 3→4+）后该桶被整桶跳并——**live 视图
消费面（childNodes/firstChild/编辑 join walk）一并致盲**，r2 的 join walk 读到基树
残影 no-op，断言回落 stale 基树快照。t4 幽灵（attr 门轮立项动机）只在序列化层暴露，
live 视图从不需要跳并。

首查的 PSD truthy-"0" 臂（`__zw_contains === '1'` 严判）经 A/B（临时回退 `!!`）
证非：lrlp 全案 2,559F 逐位同值——门开不开不改变结局，盲源在合并层。

### 修复（R383a 域收缩）

跨代际桶跳并收缩到**序列化 walk**：`_zwSerSkipCrossGen` 模块标志由
`_zwFusedInnerHtmlSel` 与 part04 outerHTML sel 分支在各自 `_childNodeList` 调用期
try/finally 置位，`_zwOverlayPendingChildNodes` 仅在标志置位时执行 R383a 早退。
序列化双计防线（t4 语义）原样保持；live 视图恢复全并语义（死条目由 link 断链 +
R382 按条目代际兜底剔除，22b4 语义）。

### 撞车合并（rebase 时发现）

push 前与并行流撞车：其 f9688ed8c 以 R4620 `__zwCeWalkMerging` 域门旁路修同一回归
（union 树三跳 bisect，与本流归因同源同判），442ccb7bf 后 join 三案 7,518P/0F 全扫。
合并取域收缩单门、移除 `!__zwCeWalkMerging` 旁路臂——CE delete walk 是 live 视图
消费不置序列化标志，R4620 诉求被自然覆盖；序列化层两版语义一致（跳并防双计）。
组合态全门复验见 §3.2。

## 3. 语料账（同驱动同参，R5342 自适应墙）

### 3.1 本流分支账（cb3cff932 + 56045919e，rebase 前）

| 语料 | attr 门轮账 | 本轮终账（A+B 合并树） | 对照 |
|---|---|---|---|
| insertparagraph-in-inline-editing-host | 80P/800F | **170P/710F** | +90P，0 新败 |
| white-space join 三案 | （回归态 4,959P/2,559F） | **6,330P/1,188F** | 基线 fail 集逐位 identical |
| insertparagraph-with-white-space-style | 616P/488F | 616P/488F | 零位移 |
| insertparagraph-in-editing-host-cannot-have-div | 420P/0F | 420P/0F | 零位移 |

inline-host 失败集 comm 逐位对照：710F ⊂ 800F（对 attr 门轮 0 新败）。其中缺陷 A
单独账（WIP 树）：80P→160P，清零 80 行全部 `defaultParagraphSeparator=p`
（20 行 followed-by-text + 60 行内容承载变体）；缺陷 B 域收缩再 +10P（attr 门
序列化对部分子测恢复开门）。探针账：zz-probe-abc t1–t4 全 Pass（t0 与 HEAD 逐字节
同值 F，见 §4）；zz-probe-lrlp 三轮循环全 Pass。

### 3.2 组合态账（rebase 撞车合并后，域收缩 × 并行流 lrlp 两轮）

push 前 `git pull --rebase` 与并行流撞车（其 f9688ed8c R4620 `__zwCeWalkMerging`
旁路臂 + 442ccb7bf 壳保留等值降级已把 join 三案推到 7,518P/0F 全扫，且其 union 树
bisect 与本流缺陷 B 归因**同源同判**）。合并取域收缩单门替代 R4620 旁路臂后全门
复验：

| 门 | 组合态结果 | 对照 |
|---|---|---|
| join 三案 | **7,518P/0F** | fix13 全扫账保持，零位移 |
| inline-host | **170P/710F** | = 分支账，零新败 |
| white-space-style / cannot-have-div | 616P/488F / 420P/0F | 零位移 |
| engine lib | 3612P/11F | 已知红集逐条 identical |
| webview lib | 822P/1F | 已知红唯一（r3259） |

组合态证域收缩覆盖 R4620 诉求（CE walk 走 live 视图不跳并），且并行流 join 修复
在域收缩下全数保持。

## 4. 已知留置

- **zz-probe-abc t0 同代视图幽灵**：同 turn append+remove 后、下次 apply 前，融合
  序列化仍见该文本（宿主基座在下次 apply 收敛，t4 实证）。视图半区补偿经 R294
  两版 A/B 证伪（r259 `bodyLast`），不落；语料断言均在 apply 后窗口，不受影响。
- **inline-host 710F 残余**：主簇 `assert_in_array` 行断插入形态（insertLineBreak/
  insertParagraph 在 inline host 的 LF+`<br>` 插入缺失，display 五型 42/24 行对称
  分布）——归 inline-host 模型行断语义，非渗漏面。
- **R383a 前提缺口**（域收缩后残余风险面）：「跨代际桶条目必已落基座」在 subset
  apply（`apply_mutations_subset` 私有批）场景仍可破——序列化滤除会把未落座条目
  一并隐藏。现网语料未见暴露，挂账观察。

## 5. 门禁

| 门 | 结果 |
|---|---|
| engine lib（test-guard 包裹，分支账） | 3611P/**11F**——失败集与已知红逐条 identical（8 fetch/request 族 + iframe-sw + r113 + s24） |
| join 三案（分支账） | 6,330P/1,188F，fail 集 `diff` 对 22b4f1f2a 基线 **identical** |
| white-space-style / cannot-have-div（分支账） | 616P/488F / 420P/0F 零位移 |
| 组合态复验（rebase 后，§3.2） | join 7,518P/0F、inline-host 170P/710F、wss/chd 零位移、engine lib 3612P/11F、webview lib 822P/1F |
| zz-probe-abc / zz-probe-lrlp | §3 探针账 |
| pre-commit guard | 三提交均 PASS（子代理隔离扫描，0 发现） |
