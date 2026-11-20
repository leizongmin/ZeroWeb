# DC-2 B1 切片 3：typing-around-link 边界臂与通道门（d23–d29）+ unlink 臂 stamped 家族（d31）

日期：2026-11-15（d23 会话起，d29 收口；d31 unlink 臂 P2F 修复续入）
范围：engine shim CE insertText 变异核、execCommand 通道、WPT runner caret-move 门、unlink 实应用臂
语料面：`typing-around-link-element-at-{collapsed,non-collapsed}-selection.tentative`、
`typing-around-link-element-after-joining-paragraphs`、`link-boundaries-insertion`、
`unlink.html`、`input-events-exec-command.html`

## 结论

- typing-around-link 三文件边界切片**未收敛**（净零相对 HEAD 起点后经读断修复转正，
  最终 selection 通道交集 F2P=18 / P2F=0，其中 tal 面 +4）。
- 关键修复（保留）：`_zwCeReresolveCaret` 读断逃逸——caret 快照锚读短（`data.length <
  startOffset`，跨 execute 家族排水致 shim 侧读 `""`）时按"读已断"放行重映射，消费
  stale anchor 直落换代后同位节点。杀掉"同页第二动作起变异不可见"一类
  （zz-probe-b1s3-solo S2 实证：修前 `<p>XY<a>abc</a></p>` 键入滞留 overlay；修后过）。
- 关键收口（d29）：边界臂加**输入通道门**（`__zw_ce_inserttext_mutate` 第三参
  `viaExec`）——execCommand('insertText') 通道臂让位（part06 exec 分流传 true，键盘
  径不传）。依据：link-boundaries-insertion（execCommand 形）两端点均期望入链接
  （`Linka`/`aLink`），tal 键入形两端点均期望外落，同位点期望相反；Chrome
  TypingCommand/InsertTextCommand 分野同构。d28 全量账本 P2F=2
  （link-boundaries-insertion 两子测）由该门修复，复跑 2/2 Pass。

## 账本（selection 通道，(case, subtest) 交集法）

基线 `/tmp/head-selfull.json`（HEAD b8a57c694 前二进制，673 case）：
`47712P / 2299F / 4T`。

d28（边界臂 + 读断逃逸 + keep 门 v2 + caret-move，含通道门前夜）：
738 case（+65 = b8a57c694 whitespaces/plaintext-only 语料入链），交集：

- F2P=18：join-pre-and-other-block ×6、editing-div-outside-body ×5、
  typing-around-link ×4（collapsed/non-collapsed 各 ×2，`target=ContentEditable`
  及 `&parent=b`）、zz-probe-b1s3 ×2、end-key-with-wbr ×1。
  非 tal 增益来自 `__zw_ce_caret_move`（End/Home/Arrow 键入 runner 门 +
  `script_ce_caret_move`），非边界臂。
- P2F=2：link-boundaries-insertion 两子测 → d29 通道门修复（2/2 Pass 复验）。
- Timeout：d28 9 例 vs head 4 例，新增 5 例全部为 b8a57c694 新入链语料
  （tal 内壳形 ×4 + whitespaces/chrome-compat/forwarddelete.tentative，后者
  head 本无此 case），非引擎回归。
- d28 新 case 65 个：942P/2423F（b8a57c694 语料入链，非本轮引擎变更产物）。

## d29 通道门复验（定向三面）

- `link-boundaries-insertion.html`：2/2 Pass（P2F 修复）。
- `typing-around-link-element-at-collapsed-selection.tentative`：d28↔d29 逐子测
  diff = **0 变化**（F2P 增益全保留）。
- `zz-probe-b1s3.html`（驻留探针）：4P/9F 与 d28 一致。

## 引擎里实验（均回退，留档）

- 元素 sel-path 登记实验：`__zwSelPathRef` 从未有任何写点（grep 0 hit）——前轮
  wire 臂的登记门是死代码；本轮补 `_zwSelStampSubtree` 元素盖章仍失败：选区
  linkEl 是 **mirror 家族 wrapper**，非盖章的 parse 对象，两族不相交。回退
  part03，保持 CLEAN vs HEAD。
- IAH 直发 wire 径（`__zw_insert_adjacent_html_path` 补发 + splice 跳过）：边界
  splice 产物落 overlay 位、caret 锚视图位缺失，mid=`<p>X<a>abc</a>X</p>` 覆盖位
  错 + 回落双计。回退。
- `_zwBndNoSlot` 槽抑制旗：trap 机械无消费端，惯性。移除，以 FIXME 留档。

## 遗留（FIXME 留档于 `_zwCeTypingAroundLink`）

边界 splice 的**双计/孤儿**类未修：本地 splice（removeChild+insertBefore）入
R51 pending 桶（视图 [X-, A, X-Y]），直发 wire 又落引擎 mirror（引擎 [A, X]），
两补偿面并存。tal 内壳形（`child=b`/`parent=b&child=i`）仍 Timeout/Fail。
收敛前置条件：单一补偿面 + caret 引擎同位快照（d24 切片）。

## 基础设施变更（保留）

- `DomMutation::InsertAdjacentHtmlByPath` 变体 + `detached_stash`/`detached_by_tag`
  FIFO 摘除暂存（js_dom_bridge.rs）：part06 createLink 非 collapsed 包裹臂在用
  （`__zw_insert_adjacent_html_path` 回调，callbacks.rs）。
- `script_ce_caret_move`（script_gen.rs）+ runner `ce_caret_consumed` 门
  （testharness.rs）：End/Home/Arrow caret-move 接入执行链。
- `__zw_ce_caret_move`（part06.js）：caret-move 键的 CE 选区移动核。

## 验证

- engine lib（CARGO_TARGET_DIR=/tmp/d19tgt，test-guard）：3984P / 0F。
- cargo fmt --check：过。clippy -p zero-engine --all-features（guard 内）：无告警。
- shim 装配语法检查（/tmp/check_shim.sh 22 parts 串联）：SHIM-OK。

## d31（2026-11-15 续）：unlink 臂 P2F 修复与 stamped 家族变异原语定谳

### 缘起

d29 收口复验发现本批 P2F=1：`input-events/input-events-exec-command.html ::
execCommand("createLink") should remove the link`（期望 `abc`，实得
`a<a href="https://example.com/">b</a>c`）。HEAD 上该断言**空转假绿**——
createLink 无 wrap 臂时 DOM 零变更、断言恒真；本批 createLink 非 collapsed
包裹臂落地后真形暴露。

### 根因链（三轮探针定谳，zz-probe T5/T6/T7 一次性诊断 + P 系列驻留）

1. **d29 遗留调试引用**：臂内 `ulDbg.push(...)` 三处残留而 `ulDbg` 声明已在
   清理时删除——首个 hit 抛 ReferenceError 被外层 `catch (_eUl)` 静默吞臂，
   unlink 整臂 no-op（T1–T4 全 "a<a>b</a>c"）。删残留即 T1/T4（createLink 建
   `<a>` 形）转 Pass。
2. **stamped/wire-only 家族变异原语语义**（P1–P4 探针，`txt` 即该家族）：
   - `removeChild` 生效（标准子树语义，摘 a 连带子）；
   - **既有子移动（move-out）静默 no-op**——`insertBefore(existing)` 后
     `parentNode` 不换（T5：pnTxt=false/inA=true/mid 零变异）；
   - **fresh 插入只收工厂产物**——`createTextNode`/`createElement` 生效
     （T6 insTxt=aB…c），parse 节点 `cloneNode(true)` 产物**整类拒收**
     （T7：clInTxt=false、视图零变异）；
   - 本地操作不发树 wire（createLink 臂同插后引擎无双份实证）。
   克隆与移动两配方在该家族全灭——唯一可靠本地径是**工厂重建直插**
   （文本 `createTextNode`、元素 `createElement`+属性拷贝递归）。
3. **印记判别缺口**：`__zwSelector` 印记在 shim 域可读（diag pSel=#txt、
   fp=#txt:1），parse 建 `<a>` 的 parentNode 即 host2 本体（pIdent=true）——
   stamped 分支判定本身可靠；缺口在克隆配方本身（第 2 条）。tier A plain
   路径另加**首轮落地校验**（probe-move 未让位即改工厂重建），防该家族
   经 plain 包装实例绕过 stamped 判定。

### 修复（part06.js unlink 臂）

- 子树遍历改 `childNodes` 数组（firstChild/nextSibling live-list 在
  createLink insertAfterPath 落地后滞留插入前形态，walk=0 整臂 no-op）。
- stamped 分支：tier A 工厂重建直插 + `removeChild(ulA)`；tier B 工厂壳
  （属性拷贝去 href）含工厂 children 整体插原位。
- plain tier A：保留 move-out（保对象身份，unlink.html 391P 依赖）+ 首轮
  落地校验回退工厂重建。
- wire landing（IAH beforebegin + 路径摘 a）保留；stamped 域摘除路径按
  插入顶层子数移位（同批同点不移位命中新插节点 net-zero）。

### 定谳记录（驻留探针 `editing/run/zz-probe-d31-unlink.html`，8P/0F）

P1 removeChild 子树语义、P2 工厂插入 unwrap、P3 移动 no-op 家族锁定、
P4 cloneNode 拒收/工厂收、T1 createLink→unlink 立即、T2 parse 建 a 选区
内落锚、T3 parse 建 a 宿主级选区、T4 P2F 原测全序（createLink→读→unlink）。

### 面账（d31 终态）

| 面 | 期望 | 实得 |
|---|---|---|
| `editing/run/unlink.html` | 391P/4F 不漂 | 391P/4F ✓ |
| input-events 全量 | 222P+1（P2F 转 Pass） | 223P/936F/3T ✓（恰 +1P/−1F） |
| `input-events-exec-command.html` remove-the-link | Pass | Pass ✓ |
| `zz-probe-d31-unlink.html`（新驻留） | 8P/0F | 8P/0F ✓ |
| `zz-probe-b1s3.html` | 4P/9F 不漂 | 4P/9F ✓ |
| `editing/run/bold.html` | 3039P/9F 不漂 | 3039P/9F ✓ |

unlink.html d29 臂落地累计 370P/25F → 391P/4F（净 +21，余 4F = FIXME(d24
unlink 余样式重裹)）。input-events 通道 head→d31 交集：F2P=12、P2F=1→0。

### 遗留 FIXME

- FIXME(d31)：工厂重建器对注释等罕见 nodeType 仍克隆兜底（该家族可能拒收
  →本地视图丢该节点，语料无此形）；工厂重建不保对象身份——no-op 家族的
  unwrap 后选区引用换新对象，caret/选区保持依赖 d24 引擎同位快照。
- FIXME(d24)：`style=font-weight`/`class=bold` 的余样式重裹（spec
  setSelectionValue 机制）未接——unlink.html 余 4F。
