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

## 3. 修复（三笔协同）

### ① pipeline `rebind_handle_node` 歧义守卫（根修）

selector 在**新 doc** `query_selector_all != 1` 时跳过重绑（pipeline/mod.rs）。留空走
unknown-handle 既有 lenient 语义，好于按猜测锚定。这是污染源头守卫：pn 不再被写歪。

### ② bridge `RemoveHandle`→`Remove` 翻译歧义守卫（同类防御）

持久表翻译臂（js_dom_bridge.rs pre-pass）加同款唯一性门：歧义 selector 的翻译会按
`query_selector` 首见命中**猜测移除**，同型错删。守卫后留 RemoveHandle 原样走
handles-miss lenient 路径。契约文档变体表同步注明。

### ③ part04 元素 remove 无 id 臂择路 + 位置寻址补发（真删路径）

无 id handle 元素移除先扫父基底缓存（`_zwChildBaseCache`）中的落地拷贝（nodeType==1
∧ 无 `__zwHandle` ∧ 标签同名；`_rmPrev` 可定位时从其后扫起降同名兄弟误伤面；代际门
同 R129 文本面 ①a——仅跨代际桶生效，同代直走 RemoveHandle 保 R294 surround 中间态
零扰动）：

- **拷贝在场**（= 宿主基座未同步该移除的 corruption 态）：**抑制 RemoveHandle**，改发
  位置寻址 `__zw_remove_child_at(父sel, 拷贝索引)`（跨代际免疫）+ 基底 splice 视图
  补偿。两 wire 互斥是防双删关键：若 RemoveHandle 先正确摘除真目标，同批
  RemoveChildAt 索引因前序移除平移会命中无辜兄弟（[span,br,div] 删 br 后 idx1=div）。
  拷贝在场恰证明宿主未同步，抑制无泄漏面。
- **拷贝缺席**：回落原 RemoveHandle 路径零变化。

引擎侧 `RemoveChildAt` 语义（父 miss / 越界 lenient warn+skip）保证补发在真目标已被
前序操作摘除时静默跳过。

## 4. 验证

- **探针页**（zw-probe-ihost-shift.html，未跟踪）t3（语料 ③ 同形）：修复前
  `ctr.innerHTML=<span contenteditable=""><br><br></span><div>efg</div><br>`（幽灵尾）；
  修复后 `<span contenteditable=""><br><br></span><div>efg</div>`，engCtr=[span,div]
  零幽灵。remove 时点 ghost 态实录：`gen=11, bkt.stamp=6, base=["span#-@S","br#-@S"]`。
- **全量语料**：inline-host **880P/0F**（修复前 780P/100F），EXIT=0，零 case 级 Timeout。
- **位移基线**：见下节终账。

## 5. 门禁终账

插桩剥离后干净二进制逐文件分跑；fmt 重排（guard 换行，语义等价）后终树复验 inline-host
与 engine lib，逐位一致：

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

新增引擎层钉测（crates/engine/src/tests/pipeline.rs，
`rebind_handle_node_skips_ambiguous_selector_ihost_ghost_br`）：唯一 selector 重绑正控 +
失唯一后重绑必须跳过（R100 换代清零前置断言自锚）。fmt `--all --check` 净、
clippy `-p zero-engine --all-targets` 净。插桩残留 grep 清零（ZW_APPLY_TRACE /
__zwDbgGhostState / [rebind] / [apply-rh] / [pn-write] 全剥离；命中的 pipeline
url-chain eprintln 与 part04 既有 __zwDbgS 为先前轮次存量，不在本轮 hunk 内）。

## 6. 剩余

inline-host 880 全扫，无残余簇。探针页保持未跟踪。
