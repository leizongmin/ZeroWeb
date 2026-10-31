# inline-host shift×normal 100F 收口：陈旧裸 tag selector 重建重绑改锚——RemoveHandle 删错节点致语料自插 br 幽灵永生

**日期**: 2026-10-31
**基线**: pre `{}<br>` 簇收口后 inline-host 780P/100F（5e3da508b）
**结果**: **880P/0F 全扫**（F→P 100 / P→F 0）

## 1. 归因修正

R5356 轮把剩余簇概括为「补断 `<br>` 落宿主 span 之后（container 子级）而非内部，属
insertLineBreak 落位语义」。本轮探针+trace 证伪该定性：**补断落位一直正确**（span 内部
`<br><br>`），`</span>` 之后的可见 `<br>` 是**语料测试自己 append 又 finally remove 的
brElement 幽灵**——remove 发出的 RemoveHandle 被引擎解析到 span 的无辜 br 上删掉，真
br 永生，经 shim html 刷新主同步路径在每个后续子测重吸收。机制与补断落位无关，单根因
解释全部 100F。

## 2. 根因链（zw-probe-ihost-shift 探针 + ZW_APPLY_TRACE/[rebind] 插桩实证，插桩已剥离）

语料形态（editor-test-utils.js）：`container.appendChild(document.createElement("br"))` →
`setupEditingHost("{}")`（span 清空重设）→ `sendEnterKey(kShift)` → assert →
`finally brElement.remove()`。

1. **注册时唯一、随后失唯一**：append 批 apply 时 span 刚被 `innerHTML="{}"` 清空，文档
   恰有一个 br（语料自插的）→ `unique_selector_for_node` 的裸 tag 形态当时候选且
   `query_selector_all("br").len()==1` 验证通过 → `selector_handle_map` 入库
   `br → __n2`。Enter 后 span 补断填入 2 个 br，选择器失唯一但表项**不随文档更新**。
2. **重建重绑按猜测锚定**：runner 刷新环 `refresh_if_html_changed`（webview.rs:1181）
   检测 html 差分 → `render()` 全量重建（slotmap 换代，`persistent_handle_nodes` 清空）
   → 按 `selector_handle_map` 逆视逐 handle `rebind_handle_node(handle, "br")` →
   `query_selector` **首见命中**。`[rebind]` 插桩实录两跳：
   `__n2 sel=br -> 20v1 parent=DIV`（幸运正确）→ `__n2 sel=br -> 20v1 parent=SPAN`
   （改锚到 span 首个 br）。
3. **删错节点**：finally 批 `RemoveHandle{__n2}` 经 pn 命中 20v1（span 的无辜 br）→
   `[apply-rh]` 插桩实录 `remove ok=true parent_tag=SPAN kids Some("BR,BR")->"BR"`。
   引擎文档中语料自插的真 br 完好。
4. **幽灵永生**：宿主 html 持 `</span><br>` → 每次基底重建重解析把幽灵收编进
   `_zwChildBaseCache`（探针实证拷贝指纹 `base=["span#-@S","br#-@S"]`——无 handle 带
   selector 标记）→ 后续所有 GOT 多一个容器级 br。尾簇 9 注记「无 id 回落原路径由
   shim html 刷新主同步路径收敛」的前提不成立——刷新序列化源即含拷贝的视图，自
   perpetuate。

为何只 shift×normal 全军覆没：ghost 是否入账取决于序列化比对点（assert 在 finally
remove 之前执行，本测试自身不受影响，毒化后续子测），分簇形态为语料序内累积效应；
pre×* 簇已由 5e3da508b（同 turn 双删互斥）先行收口，本轮后 shift×normal 全翻零新败，
单机制覆盖全部残余。

## 3. 修复（撞车仲裁：part04 同面与翻译守卫两笔撤改，保留 rebind 歧义守卫）

本流提交（原 f9ab18400）含三笔：pipeline rebind 歧义守卫 + bridge 翻译歧义守卫 +
part04「元素 remove 无 id 臂择路 + 位置寻址补发」。rebase 时发现并行流 345140946
（R5356b 轮前置笔）已独立落地同一幽灵 br 根因的 part04 面：「无 id pending 元素跨批
remove 宿主镜像补发」（inline-host 720P→820P）——journal 序 childAt 先于
RemoveHandle，pn miss 时按父基底缓存扫描位置补发删真目标。按「消费域收敛、不回退
对方修复面」先例仲裁落地，其中翻译守卫在组合态复验中二次撤改。终稿 83b90e689
（rebase 并入对方 R5392/R5395/R5398/s176 后落地，翻译门与 part04 同面块在落地前
撤净，未进入远端；中间态 8c19d8a66 仅本地存在过）：

- **part04 同面机制撤改零残留**：本流的择路/位置补发块撤下，采纳对方版本（rebase
  冲突解决取 HEAD 侧：`_r5356ElPs`/`_r5356ElIdx` 预扫 + `__zw_remove_child_at` 位置
  补发 + `__zw_remove_handle`）。rebase 后该文件与对方树 diff 零净变化。
- **bridge 翻译歧义守卫撤改（语义保守性定案，超时证据经复证不作依据）**：本流曾给
  既有 `RemoveHandle→Remove` 翻译臂（`persistent_sel` 存在即翻译，对方树既有语义）加
  「selector 须唯一命中，歧义跳过」门。该门改变载荷语义：join 语料（send_keys 跨批
  join）存在 selector 失唯一后的 RemoveHandle wire，既有语义按 `query_selector`
  首见命中移除；歧义跳过使这类 wire 静默 no-op，引擎树滞留 JS 侧已认为移除的节点
  （行为分歧风险，与超时无关）。撤改当时观察到的 join-different 重型变体 Timeout 差
  （含门 2T / 撤门 1T）后经基线复证定谳为 R5342 预算墙随机截断（§6，基线零本流改动
  同样截断、变体逐轮漂移）——非该门因果，不作为撤改依据；撤改终案以语义论据独立
  成立。已恢复 plain 翻译臂并在原位留注记，契约文档变体表措辞还原。本流「猜测移除」
  防御的正确落点在 rebind 守卫（下条），不在本臂。
- **保留 pipeline `rebind_handle_node` 歧义守卫（污染源头守卫，本流独立面）**：
  selector 在**新 doc** `query_selector_all != 1` 时跳过重绑（pipeline/mod.rs），留空
  走 unknown-handle 既有 lenient 语义，堵截 [rebind] 实录的 `query_selector("br")`
  首见猜测锚定把 pn 写歪、经 handles 喂给 apply 臂删错节点的入口。组合态全量语料
  复验该守卫与对方面零冲突（§5.2），引擎钉测锚定不变量。

## 4. 验证

- **探针页**（zw-probe-ihost-shift.html，未跟踪）t3（语料 ③ 同形）：修复前
  `ctr.innerHTML=<span contenteditable=""><br><br></span><div>efg</div><br>`（幽灵尾）；
  修复后 `<span contenteditable=""><br><br></span><div>efg</div>`，engCtr=[span,div]
  零幽灵。remove 时点 ghost 态实录：`gen=11, bkt.stamp=6, base=["span#-@S","br#-@S"]`。
- **全量语料**：inline-host **880P/0F**（修复前 780P/100F），EXIT=0，零 case 级 Timeout。
- **位移基线**：见下节终账。

## 5. 门禁终账

### 5.1 本流分支账（rebase 前，原 f9ab18400，插桩剥离后干净二进制逐文件分跑）

| 语料 | 修复前账 | 修复后账 | 判定 |
|---|---|---|---|
| inline-host | 780P/100F | **880P/0F** | F→P 100 / P→F 0，终树复验逐位一致 |
| join paragraphs | 3456P/0F | 3456P/0F | 恒等 |
| join lrlp | 2607P/0F | 2607P/0F | 恒等 |
| join lprl | 1455P/0F | 1455P/0F | 恒等 |
| cannot-have-div | 420P/0F | 420P/0F | 恒等 |
| white-space-style | 616P/488F | 616P/488F | 恒等 |
| insertlinebreak-ws | 0P/552F | 0P/552F | 恒等（首录账保持） |
| engine lib | 3614P/10F | **3615P/10F** | +1P=新增钉测，10 已知红恒等（终树复验） |
| webview lib | 825P/0F | 825P/0F | 恒等 |

### 5.2 合并树组合态复验（rebase 后，本流两笔守卫撤留定案 + 对方 R5356b/R5371/s175 面）

单干净二进制逐文件分跑（systemd MemoryMax=24G 守卫，空闲机器单流）：

| 语料 | 对方树基线 | 合并树实测 | 判定 |
|---|---|---|---|
| inline-host | 880P/0F | **880P/0F exit=0** | 恒等 |
| join paragraphs | 3456P/0F | 3450P/0F/1T exit=1 | 已完成行全 Pass；1T=预算边距截断，见 §6 |
| join lrlp | 2607P/0F | 2607P/0F exit=0 | 恒等 |
| join lprl | 1455P/0F | 1455P/0F exit=0 | 恒等 |
| cannot-have-div | 420P/0F | 420P/0F exit=0 | 恒等 |
| white-space-style | 848P/256F | 848P/256F exit=1 | 恒等（exit=1 为 256F 既有失败账） |
| insertlinebreak-ws | 528P/24F | 528P/24F exit=1 | 恒等（exit=1 为 24F 既有失败账） |
| engine lib | 3,617P/10F（b5fbe98b9 记账） | **3,618P/10F** | +1P=本流钉测 ok；10 失败逐条=fetch 契约簇 9 + s24 渲染流债，已知红集恒等 |
| webview lib | 830P/0F | **830P/0F** | 恒等 |

fmt `--all --check` 净、clippy `-p zero-engine --all-targets` 净（合并树）。插桩残留
grep 清零（ZW_APPLY_TRACE / __zwDbgGhostState / [rebind] / [apply-rh] / [pn-write]）。

新增引擎层钉测（crates/engine/src/tests/pipeline.rs，
`rebind_handle_node_skips_ambiguous_selector_ihost_ghost_br`）：唯一 selector 重绑正控 +
失唯一后重绑必须跳过（R100 换代清零前置断言自锚）。

复验过程记账（撤改守卫的两轮 A/B）：合并树首跑（含翻译歧义守卫）join-different
3448P/1T，空闲复跑 3447P/2T，且两次均在 join-different 结束秒出现 lrlp/lprl 启动即
败（空输出 exit=1）的级联瞬态；撤翻译守卫复跑 3450P/1T、lrlp/lprl 换全名
（`join-different-white-space-style-left-line-and-right-paragraph` /
`-left-paragraph-and-right-line`——首轮所用短名 `join-left-*` 语料文件不存在，
启动即败与守卫无关）后 2607P/1455P 全扫零位移。

### 5.3 终树 83b90e689 组合态电池（撤净翻译门 + 并入对方 R5392/R5395/R5398/s176 后）

单干净二进制逐文件分跑（systemd MemoryMax=24G 守卫，空闲机器单流），§5.2 之后又
rebase 进对方 11 笔（含 R5392/R5395/R5398/s176），amend 撤净翻译门后终树全量复跑：

| 语料 | 终树账 | exit | 判定 |
|---|---|---|---|
| inline-host | 880P/0F | 0 | 全扫恒等，零 case 级 Timeout |
| join-different | 3447P/0F/2T | 1 | 0F；2T=墙沿截断第五轮新变体集（§6） |
| join lrlp | 2607P/0F | 0 | 恒等 |
| join lprl | 1455P/0F | 0 | 恒等 |
| cannot-have-div | 420P/0F | 0 | 恒等 |
| white-space-style | **1104P/0F** | 0 | 对方新面收口后翻绿（§5.2 时 848P/256F），全分母 848+256=1104 |
| insertlinebreak-ws | **552P/0F** | 0 | 对方新面收口后翻绿（§5.2 时 528P/24F），全分母 528+24=552 |
| engine lib | **3,626P/10F** | 1 | 较 §5.2 账 3,618 的 +8P=对方新面新增测（本流钉测已在 3,618 内）；10 失败逐名=已知红集恒等（fetch/Request 族 8 + iframe SW + s24） |
| webview lib | 830P/0F | 0 | 恒等 |

wss/ilb 翻绿与 engine +8P 均来自对方树新并面，非本流改动位移；本流 rebind 守卫在
对方三轮全扫账（§5.2 表）上逐位零位移。终树 join-different 3447P/0F/2T 计入 §6
漂移表第五行。终树 fmt `--all --check` 净、clippy `-p zero-engine --all-targets`
净。

## 6. join-different 残余 T 归因：预算边距截断（基线复证定谳）

合并树复验 join-different 出现残余 Timeout（撤翻译门轮观察 3450P/0F/1T，逐轮 1T/2T
漂移）。为排除本流守卫引入时序回归，在**对方树基线 aa4e9d190**（本流提交的父提交，
零本流改动；独立 clone + 独立构建二进制，zb-attrib3.service systemd 守卫，墙钟
~21 分钟）直跑同语料复证：

| 树 | 本流面 | 账 | 超时变体 |
|---|---|---|---|
| aa4e9d190（基线） | 无 | 3455P/0F/**1T** exit=1 | select-boundary pre-wrap×pre |
| 合并树首跑 | 两守卫均在 | 3448P/0F/1T | forwarddelete pre-line×pre |
| 合并树空闲复跑 | 两守卫均在 | 3447P/0F/**2T** | forwarddelete pre×pre-line + pre-line×pre |
| 合并树撤翻译门 | 仅 rebind 守卫 | 3450P/0F/1T | select-boundary nowrap×pre-line |
| 终树 83b90e689（§5.3 电池） | 仅 rebind 守卫 | 3447P/0F/**2T** | backspace pre-line×pre-wrap + forwarddelete pre-line×pre |

定谳三点：

1. **基线同样 1T**——无本流任何改动的树同机同语料同样截断一行，1T 非本流守卫引入。
2. **超时变体逐轮漂移**（五轮五组互不相同，见上表超时变体列）——无固定受害者，
   是 R5342 自适应墙（60+nv×15+90）预算边界的随机截断：语料 3,456 变体恰在墙沿，墙到
   时在跑的变体被标 Timeout，各轮 P 差（3,447–3,455）即被截变体的剩余子测数。
3. **所有轮次 0F**——只有截断没有失败；历史全扫账 3456P/0F 为更轻树上跑满的分母，
   现行重树（R5392/R5395/R5398 面叠加后）恰越墙沿。

处理：按预算边距截断记档，不作为回归挂账；需要 3,456 全分母复验时单语料直跑或调高
CASE_WALL_S。其余面 inline-host 880 全扫无残余簇，探针页保持未跟踪。

**挂账（预算/性能簇，下轮认领）**：对方最近一次 join 7,518P/0F 门禁记账（R5371）
早于 b5fbe98b9（s175 unique_selector 可达门），墙沿边距疑似该笔 JS 侧重
Registrations 成本推高。预算边距 Timeout 预存于对方树（上表基线行），非本流引入，
非本轮正确性范围。
