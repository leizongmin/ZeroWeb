# lrlp 壳丢失两笔落地 + R383×CE walk 撞车仲裁——join 三案 7,254P/264F（96.5%）

**日期**: 2026-10-31
**前置**: [join paragraphs 簇清零](2026-10-31-join-skipws-direction-paragraphs-sweep.md)
（fix10 = 6,330P/1,188F，残余 lrlp 引擎收敛簇）；并行流
[daf3bc4c5 attr 门](2026-10-31-attr-gate-land.md)（R383 跨代际桶跳并）rebase 拉入。

本轮三段：①上轮 429 中断遗留的 R4620 join 落位两笔（InsertBefore ref 同族
漂移兜底 + 壳 caret 落位钳制）验证落地——lrlp 三方法 396F/方法 → 88F/方法；
②union 树（拉入 attr 门后）join 语料跌 2,295P 的撞车归因与仲裁——R383 桶级
跳并旁路门（`__zwCeWalkMerging`）落地，逐位恢复 fix11 账面；③残余 264F
形态定性（normal 白空间壳降级簇，下轮首位候选）。

## 1. R4620 join 落位两笔（上轮遗留 WIP，本轮验证）

上轮 rate-limit 中断于验证前，工作区遗留两笔（4fc399b5c 基座）：

- **`js_dom_bridge.rs` InsertBefore ref 兜底**：R4620 `ins_targets` 批前预解析
  机制扩展到 `InsertBefore` 的 `ref_selector`（批前预解析 + apply 失配时查表，
  查表命中须仍挂树；ref 已被批内摘除维持 skip——语义对齐 JS ref 脱离抛错）；
  并在 `InsertAdjacentHtml` apply 成功后**刷新同 selector 预解析表**（批内
  SetInnerHtml 重定基座换 NodeId，后续同 ref 兜底须用 re-base 后仍挂树的
  NodeId）。根因：join per-run 壳序——plain 壳 iah(beforebegin@ref) 先插入
  元素兄弟使同 ref `:nth-child` 索引后移，handle 壳 InsertBefore 的 ref
  selector（shim 落位时点计算）apply 时失配 lenient skip → 壳丢失（lrlp fd
  几何 `<b>def</b>` 先落、`__n0` span 恒失联实证）。
- **`part01.js` 壳 caret 落位钳制**：plain 壳本地落位只入 shim 侧数组，
  registry/sel 融合视图缺席时 `indexOf(shell) = -1` 负 offset → `setStart`
  IndexSizeError → 整批 wire 永不 drain（lrlp bs 几何实证）。缺席时落容器尾。

**验证（4fc399b5c + 两笔，/tmp/join-fix11-full.log）**：join 三案
**7,254P/264F（96.5%）**，对 fix10 逐测试 ID **F→P 924 / P→F 0**，结果行
7,518/7,518 恒等；lrlp 396F/方法 → 88F/方法，paragraphs/lprl 保持 0F。

## 2. 撞车归因：R383 桶级跳并 × CE walk 对象 identity 消费

rebase 拉入 daf3bc4c5（attr 门）+ 88ca4f5db（s168 shadow）+ 606be5680
（渲染）后 union 树全量复跑：join 三案 **4,959P/2,559F——lrlp P→F 2,295**，
其余两案零位移。逐测试 ID 对照失败形态升级：fix10 时代是「壳丢失」（提取
发生、def 消失），union 上是**整动作 no-op**（右块原样未动、`<br>` 残留）。

### bisect（单案 lrlp，test-guard + ZW_SCRIPT_TIMEOUT_MS=300000 同参）

| 树 | lrlp 账 | 判读 |
|---|---|---|
| 4fc399b5c（fix10 基线，无 WIP） | 1,419P/1,188F | 396F/方法 |
| 22b4f1f2a（= daf3bc4c5^；含 503b layout 竖框） | 1,419P/1,188F | 503b/22b 零影响 |
| **daf3bc4c5（attr 门）** | **48P/2,559F** | **翻转点** |
| 606be5680 union 全量（无 WIP） | 48P/2,559F | 后续提交零边际影响 |

罪首 = daf3bc4c5 的 **R383「跨代际桶不并入视图」**（`part05.js`
`_zwOverlayPendingChildNodes` 桶级 `return out`）。

### 机制

attr 门 t4 幽灵面与 join walk 面对同一跳并的**消费方式不同**：

- 序列化读（t4/inline-host）：读宿主 innerHTML 字符串——apply 后基底快照已含
  条目，跳并消幽灵正确，且是 R383 的修复目标；
- **CE delete 族 walk**（`__zw_ce_delete`/`__zw_ce_forward_delete`）：对视图做
  **对象 identity 消费**——caret 锚链持有 setup innerHTML 物化的 JS 侧 pending
  壳对象，join 形状 Node 复算逐节点比对；send_keys 前置的引擎动作（caret 置位）
  已发布 apply 游标（daf3bc4c5 webview apply 点即时发布）+ 换代 → walk 时
  setup 桶 stamp 已跨代 → R383 跳并把 pending 壳对象从视图摘除 → 锚重解析
  失配 → 整动作 no-op。R382 条目级 ghost 确认（注释明写「CE join 探针实证
  div1 双 span」，本为护 join 面而生）被 R383 桶级粗跳**前置短路**，从未运行。

与 R383a 的 t8r4_k3 同构：跳并的「条目必已落基座」前提在某消费域不成立，
区别是 t8r4_k3 在 engine 纯测试域（游标闩可判别），join 在生产 webview 域
（游标闩恒置位，闩不可判别）——须按**消费域**判别。

### 修法：CE walk 域门旁路（`__zwCeWalkMerging`）

- `part05.js` R383 条件加 `!globalThis.__zwCeWalkMerging` 旁路臂（注释锚定
  join 语料回归证据）；
- `part01.js` 两入口 impl 换名 + wrapper：置位 → try/finally 复位，调用点
  零改动。旁路仅覆盖 walk 窗口（动作结束即清位），序列化读面不受影响；
  R382 条目级确认位于该行之下，照常运行不失效。
- 形态同 2026-10-31 撞车仲裁终验先例（`__zwCeWorldSyncing` 回灌 stamping
  旁路门）：跨流域共享路径语义冲突按**域门**收敛，不回退对方修复面。

## 3. 验证账

| 门 | 结果 |
|---|---|
| join 三案（union + WIP + 域门，/tmp/join-fix13-union.log） | **7,254P/264F（96.5%）**；对 fix11 逐测试 ID **零翻转**（Pass→Pass 7,254 / Fail→Fail 264 恒等，7,518 行恒等） |
| insertparagraph 语料零位移（attr 门收益保全） | cannot-have-div **420P/0F** ✓、inline-host **80P/800F** ✓、white-space-style **616P/488F** ✓（与 attr 门 evidence §2 逐位一致） |
| fmt / clippy | `cargo fmt --all --check` clean；clippy（zero-engine / zero-wpt-runner）0 warning |
| engine lib（make test，/tmp/make-test-fix13.log） | 3,611P/11F，11 红逐个对账 = 已知红集合恒等（9 fetch 域 + r113 + s24），零新增 |
| webview lib | 终态 **822P/1F**，唯一红 `test_document_scrolling_element_r3259`（已知）；过程发现 1 笔本流欠账，见 §3.1 |

### 3.1 webview 钉子欠账认领与清偿（`execute_dom_script_path_generation_bump_call_site_present_d2`）

make test 中止于 workspace 段后单独复跑 webview lib 得 821P/**2F**，新红即上述
钉子：断言 user_actions.rs 内独立脚本形态 `globalThis.__zw_apply_generation_bump();"`
出现 **== 1** 次，当前实际 3 次。归因：3 处均为本流 **0920f3fd8**（join 轮
03:00）合法配对面——executor 分支 / sandbox 直执分支 / 第 4 应用路径
（注释明写「与其余 3 条应用路径对齐，apply 后统一换代」）；钉子写于
D2 轮单站点时代，`== 1` 把存在性意图写死成唯一性假设。掩盖链：0920f3fd8
之后 make test 每轮中止于 workspace 段，webview lib 无人单独复跑；attr 门轮
引用的 822P/1F 是 01:37 insertParagraph 轮旧账（当时 count 仍为 1）。

清偿：钉子断言改 `>= 3`——三条 user_actions.rs 内应用路径各须一处配对
换代通知，任何一处删行/注释化即红；新增合法站点不误红。guarded 单测
绿 + webview lib 全量复跑 822P/1F（已知红唯一）双验。

观察注记：`undo-insertparagraph-after-moving-split-nodes` 4F——不在两流域
任一基线表，子测为 insertParagraph+undo 面（无 delete 键），CE 域门不可达，
union 态既有观察、不阻塞、挂认领。

## 4. 残余 264F 定性：normal 白空间壳降级簇（下轮首位）

264F = 88×3 方法，全部 lrlp，几何统一为**左行内行 + 右块**（
`<span nowrap>abc</span><div normal>[]def…`）。结构/落位/移除全对，唯一
delta = 壳携带 `style="white-space:normal"`：

- 右块首行为裸 run → actual `<span style="white-space:normal">def</span>`，
  expected **裸文本** `def`；
- 右块首行在 `<b>` 内 → actual `<b style="white-space:normal">def</b>`，
  expected **裸元素** `<b>def</b>`（style 属性多余）。

即 join 壳创建（part01 `_sh.setAttribute('style', 'white-space:' + _wsVal)`）
在有效白空间为 normal 时应**降级为不包裹/不注入**（Chrome 语义：normal 无需
保留壳）。修复面在壳创建分支 + caret 落位锚（壳缺席时 caret 落点语义），
与本轮两笔共享区域，下轮首位候选。

## 5. 撞车流程注记

- 上轮 429 中断的 WIP 恢复：stash → pull --rebase → pop，自动合并干净；
  遗留 /tmp/join-fix11-full.log 逐测试 ID 账直接消费，未重复跑基线。
- 本轮 bisect 三跳（union-clean → daf3 → daf3^）各一次 build+单案跑，
  二进制逐次重建（shim include_str 编译进引擎，JS 改动需重链）。
- 跨域电池（dom/selection）本轮未复跑：域门仅 CE delete 入口置位，R383 对
  序列化读语义零改动；attr 门轮已交 dom 55,948P/527F/14T、selection
  27,589P/23,591F/5T 基线，insertparagraph 三案零位移为交叉证据。
