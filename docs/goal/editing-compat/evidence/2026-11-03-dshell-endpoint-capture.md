# 双壳终点边界捕获轮：链外边界 keep-shell/caret/typing 链三面 delta

日期：2026-11-03（editing 流，控制面下一步 ①，gate27 收尾 + 双壳终点边界捕获面）

## 0. 范围

multitest 双壳家族（:2521-:2831，56 案）中 start 边界在 inline 链外（原链无
inline 元素）的 delete/forwarddelete 面：multitest 挂账 39 断言中 delete 10 +
fd 10 + 探针基线 43F 中 delete/fd ~20。ipara/ilb 家族的 13+6 断言另有内容丢失
/caret 世界失步 bug（C30 typed 段整丢形），独立轮归因不在本轮；inserttext 直
替 13 案不走 deleteSelection 路径，不在本轮。

探针：`zz-probe-dshell.html`（56 案 multitest 忠实移植，基线 43F）、
`zz-probe-dshell2.html`（9 案 delete-only 中间态：树+caret）。

## 1. 归因：等价点下钻方向修正 + 三面连锁

### 1.1 等价点双向下钻模型（勘前静态追踪的方向倒置）

`_zwDelGetLastEquivalentPoint`（start 侧）两规则皆向内容下钻：rule 1
（offset==len && 上邻为 inline）上行一格；rule 2（前兄弟非空 inline）下钻至
其尾。`_zwDelGetFirstEquivalentPoint`（end 侧）对称：rule 2 下钻至前兄弟尾。
链外边界由此入壳：`(abc,3) → (div,1) → (b,0) → (i,0) → (def,0)`；
`{`-形元素位 `(div,1)` 同链；`<img>` 截断（rule 2 len≠0 门）。

### 1.2 三面连锁（zz-probe-dshell2 九案逐 caret 实证，基线表见 §3）

1. **keep-shell 误算**：归一化 startNode 已在壳内，`_zwDelKeepShellOf` 按壳内
   起算 → 空壳 b 幸存（M9 `abc{<b><i>def</i></b>g]hi` → `abc<b></b>hi`）。
2. **caret 入壳**：收尾 collapseToStart 经已摘壳 remap 落 `(b,0)/(i,0)`
   （M4/M5/M9）；Chrome 落原边界（"moved to end of the preceding text
   node"）。same-node 整节点删空形（M1/M2/C02/C05）删空 def 后由 caret 归一
   面清空文本+空 inline 至幸存壳为止 → `abc<b></b>ghi` caret b@0。
3. **typing 态无载体**：`__zw_ce_capture_typing_chain` 只收 start 链，链外
   边界收空 → typed 段裸奔/错位（C02 got `abc<b>def</b>ghi`，期望
   `<b><i>def</i></b>`）。

same-node 臂早退（不进 strip 循环）+ 删空形依赖 caret 归一面的隐性清理，是
M2 矛盾（早退臂"应"不摘壳但实测 i 被摘）的解释——隐性清理只到幸存壳为止。

### 1.3 Chrome 语义（multitest recording 注释 + 期望形）

- 空包裹全摘：last visible thing 被整体删除时不留幸存壳；
- caret 落原边界（链外语义）；
- typing 态携带选区内**首个可见内容**的 format 链（仅整含；`de]f` 部分含 →
  不保留；img 首可见 → 不保留——C28 期望 `abcdefghi` typed 裸奔）；
- 链内边界（T5 幸存壳面）三面皆维持原路径。

## 2. 实现（part01f.js 五笔，+125/−5）

- **Edit A**（deleteSelection 顶部）：原边界快照 `origStartContainer/
  origStartOffset` + `origChainHasInline` 判据（`_zwDelKeepShellOf` 同款上行
  链扫，nodeType==1 && inline 元素才置位——文本是 inline node，必须过滤）。
- **Edit B**（keep-shell 臂）：`origChainHasInline ? _zwDelKeepShellOf(startNode)
  : null`——链外无幸存壳（M9 形壳全摘）；链内臂逐字节不变。
- **Edit C**（same-node 臂）：链外 + 整节点删空 → 空文本节点显式移除 + 空
  inline 包裹链摘除（delKeepShell 门 + guard 256）；caret 落原边界（min 钳
  位）。部分含切片（M3 形）只走 caret delta，不摘壳。
- **Edit D**（blockMerging 早退臂）：链外 caret delta 同形（M4/M5/M9 出口）。
- **Edit E**（捕获扩展）：`_zwDelFirstVisibleInRange`（节点级行进：可见文本
  /img 候选须整含——头界 `!== 'before'` 且尾界 `!== 'after'`；inline 包裹只
  作下钻通道；guard 双层）+ 捕获尾扩展：主链收空且非 collapsed 时取首个整含
  可见内容，文本 → 复用上行链捕获（collapsed 标记防再入；`<a>` 排除/off 表
  滤/reverse 序全复用），img/部分含 → 空。
- **Edit F**（13017c0a3，判读轮修复）：Edit E 整含两判方向互换——
  `_zwDelGetPosition` 返回**首参相对次参**方位（BpCompare −1=A<B），head ≥
  start 应拒 `'after'`、tail ≤ end 应拒 `'before'`；首版两处恰好反写，判定
  实际要求「候选横跨整个选区」，本族语料形态恒无候选 → capture 恒 `[]`。
  分阶段探针（zz-probe-dshell3：capture 直调 / delete 后 tags 存活 /
  inserttext 消费三段二分）定位断点在 capture 本身，非快照回写或消费面。

## 3. 探针基线与修复预期（zz-probe-dshell2 九案）

| 案 | 基线 dom / caret | 修复预期 dom / caret |
| --- | --- | --- |
| M1 `{…}` 整含 | （首行输出缺失，C02 形同） | `abc ghi` / t:abc@3 |
| M2 `[…def]` 整含 | `abc<b></b>ghi` / b@0 | `abc ghi` / t:abc@3 |
| M3 `{…de]f` 部分 | `abc<b><i>f</i></b>ghi` / i@0 | 树不变 / t:abc@3 |
| M4 `[img de]f` | `abc<b><i>f</i></b>ghi` / i@0 | 树不变 / t:abc@3 |
| M5 `{…img def]` | `abc<b></b>ghi` / b@0 | `abc ghi` / t:abc@3 |
| M6 `[s] de]f` | `abc<b><i>f</i></b>ghi` / div@1 | 树不变 / t:abc@3 |
| M7 链内 `[def</i></b>]` | `abc<b></b>ghi` / b@0 | 不变（链内臂） |
| M8 `ab[c…de]f` | `ab<b><i>f</i></b>ghi` / ab@2 | 不变（边界即原位） |
| M9 `{…g]hi` | `abc<b></b>hi` / b@0 | `abc hi` / div@1 |

zz-probe-dshell 56 案基线 43F；修复预期 delete/fd 面翻转（C02/C05/C06/
C11-C14/C24-C29/C42-C45 = 16 案），C03/C04/C07-C10/C38/C39/C50-C53 12 案
PASS 恒等；ipara/ilb 面（C15-C21/C30-C37/C40/C41/C46-C49/C54-C56）失败模式
可能位移（caret 输入世界变了），仍 FAIL，独立轮归因。

**修复后终判读（13017c0a3 release bin；fix 前首判 dshell 56 案仅 15 翻转、
typing 面裸奔——归因即 Edit F）**：dshell delete/fd 面 **35/35 全 PASS**——
整含 typing 链 8 案 C01/C02/C05/C06/C11-C14 typed 段重裹逐字
`abc<b><i>def</i></b>ghi`，img/部分含/s 空壳面恒等；残余 **21F 全 ipara/ilb
独立轮面**（ipara 13 + ilb 8，模式位移在 §6 记录）。dshell2 九案 DOM 全合
修复预期（M2/M5 `abcghi`、M3/M4/M6 树不变、M7/M8 链内不变、M9 `abchi`）；
caret M2/M4/M6 `t:abc@3`、M7 `b@0`、M8 `ab@2`、M9 `div@1` 精确，M1/M3/M5
落 `div@1`（§3 表 t:abc@3 的边界等价点表示差异，见 §6）。

## 4. gate27 补账（勘误定谳：外部 stop + 断言收尾批量打印，非零输出死亡）

**勘误**：前轮「零输出死亡」判读作废——runner 断言只在收尾批量打印，长腿中
途零行为正常缓冲形态（E1 36.8 min 静默后全量收尾为直接反证）；E3/E24 停跑系
03:12:22 外部显式 stop（journal `Stopping` 双 unit 同秒记录，无 oomd、无
kernel trap），非崩溃。判读判据统一为 transcript 行数 + 墙钟，rc 不作数
（02:37 勘误沿用）。

- **del-rerun/E1**（电池保全 bin，02:17:02→02:53:52 = 36.8 min，三跑争用下）：
  全量干净收尾 7,951 行 = **7,339P/296F/1T，与 gate21 基线逐位一致**——
  delete.html pre-fix 基线在电池 bin 补齐。`[th-time]` 为逐片计时遥测（首片
  setup 226.7s 系争用放大），非看门狗击杀记录，前轮「8 变体零断言」系中途
  误读。
- **diag-delete2**（make -one 现盘 post-pull bin，02:20:16→02:57:26）：
  7,339P/296F/1T，与 E1/gate21 逐位一致——pre-fix 基线双 bin 零漂移
  （COORD 03:00 追加④）。
- **E3**（pre-ipara bin cc097dbc3，02:40→03:12 外部 stop）：32m CPU 满载至
  停，零 transcript 行 = 正常缓冲形态被截断，无数据点；判别腿结案不复驱。
- **E2 fd 腿**（02:58:58→03:12 外部 stop，停时仍满载在跑）：无数据点；由
  **E2fd**（zw-g27-e2fd，03:16:24 起，同电池 bin 直跑 forwarddelete）复驱，
  对照 gate21 6,831P/258F/1T，结果见 §5。
- **kernel 崩溃叙事终局**：电池 v1 双腿（SIGSEGV/GPF）仍为唯一 kernel 实证
  崩溃对；E1/diag 双 bin 两个独立干净全量样本 → 判环境性/低频一次性，重跑法
  归因穷尽，无 bisect 靶，控制面 ⓪ 项按此口径消解。
- **E4**（bin-g27-battery multitest 全腿，03:12:39 起，pre-delta bin）：−85
  缺口同 bin 归因数据点，结果见 §5。
- **E4 终态**（zw-g27-e4 复驱，09:13:36→09:47:02 = 33.4 min，pre-delta 电池
  bin，E2fd 争用）：7,053 行 = **9,546P/428F/1T 干净全量收尾**（1T =
  `?9001-last :: 1 test unreported (tests:975)`）。切片级 diff（乙实例，
  COORD 十六轮）：对电池腿 9,569P/430F/1T **除 ?9001-last 外全部切片逐位
  恒等**，该片 battery=892P/107F → E4=869P/105F（−23P/−2F）＝同 bin 墙钟
  截断深度漂移；对 gate21 9,647P/437F/1T 的 −85 缺口同源（gate21 该片报满
  1,084，battery/E4 截断深度递减）——**multitest −85 归因收口：零真实行为
  差，纯末片页超时截断漂移，无 bisect 靶**。
- **E2fd 复驱终态**（zw-g27-e2fd，09:13:36→09:46:18 = 32.7 min，pre-delta
  电池 bin）：7,315 行 = **6,809P/250F/2T 干净全量收尾**——fd 腿 leg8 有效账
  成立（判据=行数+墙钟，rc=1 系 test-guard 信号映射不作数）。对 gate21
  6,831P/258F/1T 差 −22P/−8F/+1T：多出的一 T 系 **?6001-7000 新增截断片**
  （`1 test unreported (tests:599)`，≈30 verdicts 与缺额算术精确自洽；
  ?7001-last 1T gate21 同有、非缺额载体——勘误：本段初稿挂 ?7001-last 系
  归因误置）——页超时截断 flake（g19 溢出桶、join-different 1T 漂移同族），
  且系电池 bin（gate27 树）跨树对照 gate21 bin，不作回归挂账；250F 面为
  gate21 已知簇（collapsed 跨块合并 compare + foreColor qV）。

## 5. 门禁（gate28 三腿：run 电池 8 leg + other 通道 + engine lib）

对账基线（pre-delta）：delete.html 7,339P/296F/1T（E1/diag 双 bin 逐位）、
fd 6,831P/258F/1T（gate21；E2fd 同 bin 复驱确认，§4）、multitest 9,647P/437F/1T
（gate21）vs 电池腿 9,569P/430F/1T（−85 已收口 = E4 §4，零行为差）、
other 13,134P/1,600F、enginelib 3,709P/0F。post-delta 预期：multitest
430F→~410（`<b><i>` 家族 45 compare 行中 delete 10 + fd 10 翻转、ilb 9 +
ipara 16 允许模式位移；判读以行集 diff 对 `g28-mt-baseline-all.txt`/`g28-bi
-family-baseline.txt`，计数仅对账——末片截断漂移已知签名）、delete.html
F ≤296（链外边界面部分翻转可能）、fd F ≤258、P→F 必须为零、other/enginelib
逐位恒等。

**电池已放（zw-g28-battery，10:09:27 起，bin = 13017c0a3 release + Edit F）**：
multitest → insertparagraph → inserttext → typing-text → insertlinebreak →
forecolor → delete.html → forwarddelete 八腿（TIME_LIMIT=10800）→ other 通道
→ enginelib 顺序，done 标记 gate28-done.txt，预计 ~3h。

**multitest 腿实测（10:10:09 bin 完成 → 10:41:04 落地）：9,596P/433F/0T
rc=2**——对电池基线 9,569P/430F/1T：P +27/F +3/T −1（本轮 ?9001-last 未截断
全片跑完，基线轮该片 1T 截断）。fail 行集 diff（恒等行提取器对 E4 日志对拍
验证：E4 428 条 ⊆ 基线 430 条，仅差 2 行恰 ?9001-last 截断受害者）：

- **F→P 23**（only-in-baseline）：`<b><i>` 双壳 delete 10 + forwarddelete 10
  全数翻转（本轮主目标面 ✓）+ insertparagraph img 族 3 连带（deleteSelection
  为 ipara 前导，部分含 img 形连带回收）；
- **P→F 26，违门（P→F 必须零）**：
  - **22 案真回归（?8001-9000 完整片，基线轮该片全跑）**：`[foo<X>bar]</X>baz`
    族（b/i/s/sub/sup/u/a/font×9/嵌套 a+font/u/sub+font 22 变体）+ delete +
    inserttext，签名 22/22 一致 `expected "abaz" got "baz"`——**typed 段整丢**。
    根因：caret 落原边界两臂（selExit/same-node）前提「原边界节点在删除链外
    必存活」对该形态不成立——origStart=(foo,0) 本身在删程内被全删（detached），
    collapse 塌到已摘节点使选区世界失效，inserttext 落空。目标面 `abc{...}ghi`
    形 origStart=(abc 尾)/div 边界在删程外恒存活，故探针 35/35 与 F→P 面不踩；
  - 4 案（?9001-last，基线轮截断未跑=新暴露非回归归因）：`<b>{abc</b>…}`
    跨块形 got `<b></b><b>gh</b>` vs exp `<b>gh</b>`——typed 段在但双壳（caret
    落进幸存空壳内致 typing 重包裹），修复轮与上一条同面收口。

**修复方向（电池落地后修复轮执行，电池窗口内不动 engine JS）**：caret 两臂
加 origStartContainer 存活门（删后连接性/宿主可达检查），不存活回落 legacy
`direction=='forward'` 分支；same-node strip 臂同面检查（`[bar]baz` 单独形
同风险）。capture 臂（笔5）对该族首可见=foo 链空返 [] 语义正确，不在修复面。
multitest 腿判读：主目标面达成、P→F 违门——**gate28 批不得据此判绿，修复笔
落地后 multitest/delete/fd/insertparagraph 四腿需复跑**（ipara 腿 P→F 20 同
caret 臂面，见下）；电池余腿（it/typing/ilb/fc/delete/fd/other/enginelib）
继续取全量 scoping 与恒等数据不中断。
**探针 v3 补案与修复前基线（10:55-10:59）**：zz-probe-dualshell.html 扩
P14 `[foo<b>bar]</b>baz`（22 案核心形）/P15 u 壳/P16 i 壳/P17 `[abcdef]`
同 text 全选形（same-node 臂 origStart==startNode 面）；修复前基线（bin
13017c0a3+Edit F，g28-probe-v3-prefix.log）：P14/P15/P16 final
`<div>baz</div>` typed 整丢（22 案签名复现 ✓）、P17 `<div>a<br></div>`
（typed 存活但占位 br 滞留形）。修复后复验以 P14-P16 转 `<div>abaz</div>`、
P17 转净形 + 双壳目标面 35/35 不退为判据。

**bin 漂移注记（10:53 pull --rebase 带入并行流 engine 提交）**：multitest +
insertparagraph 两腿跑在 pre-s254 bin（13017c0a3 树 10:10 build）；10:53 起
工作树含 `5002fa0c1`（s254 import map 接线，script-sandbox 域，**不触
part01f.js**）→ 其后腿（inserttext 起）make 重建拾取 s254。编辑 run 语料面
无模块系统消费面，噪声预期 ≈0；**enginelib 腿计数将含 s254 自身的 +4 script-
sandbox 翻绿，恒等复核按面判读不按原始计数**（基线 3,709P/0F + s254 面）。

**insertparagraph 腿实测（10:40:43→10:58:17 落地，pre-s254 bin）：6,968P/
227F/0T rc=2**——对 gate27 基线 6,984P/211F/0T：P −16/F +16（F→P 4、P→F 20，
行集 diff 定谳）。**P→F 20 全部同 caret 臂前提破面（gate28 delta 新增）**：
- **ol/li 族 15 案**（`<ol><li>[abc]</li>…` insertparagraph ± inserttext，
  dps div/p 双系）：签名恒一 `exp <div><br></div> got <ol><li><br></li></ol>`
  形（li 出列拆分没走通，占位 br 滞留 li）——`[abc]` 全选的 origStart=(abc,0)
  在删程内被摘，deleteSelection 前导 caret 臂 detached，ipara 主拆分支在坏
  caret 世界走错臂；
- **stylewithcss 族 4 案**（`[foo<span style=color>bar]</span>` 与
  `<span style>[bar]</span>` 形，swcss true/false × dps div/p）：签名恒一
  `exp <p>foo</p><p>baz</p> got <p>foo</p><p><span style></span>baz</p>`
  （空壳残留）——同 caret detached 面；
- **CE-false 族 2 案**（`<p><span contenteditable=false>abc</span>[X]</p>` /
  镜像形）：修复笔后复跑定谳是否同根回收。
**F→P 4**：`foo{<span style=color>bar</span>}baz` 形（swcss × dps 4 组合）——
`{...}` 边界 origStart 在删程外存活，caret 臂正常翻转 = delta 正面在 ipara
语料连带回收，修复笔存活门不影响该面。
**复跑腿扩为四腿**：multitest/delete/fd/insertparagraph（ipara 腿 P→F 20
中 ≥19 案与 multitest 22 案同根，修复笔预期同面回收；CE-false 2 案待验）。

**inserttext 起四腿实测（11:01:07→11:03:09 落地，含 s254 bin）——fail 行集
恒等提取 diff 对 gate27 同腿段逐位为零回归**：
- **inserttext**：2,782P/227F/0T rc=2，对基线 2,782P/227F 总数恒等，P→F=0/
  F→P=0（?2001-last 正常收尾非截断，2.5 min 为单页语料缓存热）。无 delete
  前置步骤，不触 caret 臂前提破面，与根因模型一致；
- **typing-text-in-direct-child-of-editing-host**：6P/10F/0T rc=2，10 fail
  恒等行集逐位同基线（6P/10F 小面原样）；
- **insertlinebreak**：50 fail 恒等行集逐位同基线（P→F=0/F→P=0）；
- **forecolor**：2,279P/48F/0T，48 fail 恒等行集逐位同基线（P→F=0/F→P=0；
  11:02 中途快照 43 为腿未完成时读数，11:06 终态复核 48）。
四腿恒等实证 s254 bin 漂移在编辑 run 语料面噪声 ≈0（漂移注记成立）；回归
面仍收敛在 multitest 22 + ipara 20 = 42 案。**delete.html/forwarddelete 两
腿待落（重点核 `[foo<X>bar]</X>baz` 族 P→F 扩散——delete/fd 是 caret 臂
主消费语料，扩散面可能大于 multitest）**，落地后按同口径判读。

**delete.html 腿实测（11:03:09→11:40:33 落地，37.4 min 干净全量，含 s254
bin）：7,344P/291F/1T rc=2**——对 E1 基线 7,339P/296F/1T：P +5/F −5/T 恒等
（`?7001-last` 同页截断受害者逐位同）。行集 diff：**P→F 0（`[foo<X>bar]</X>`
族零扩散，delete 语料主门通过）**、F→P 5 全为 `foo{<X>bar</X>}baz` 部分含形
正面翻转（裸 span 1 + swcss false/true × span-color + b 共 5，`{` 起点
origStart 在删程外存活 = caret 臂正面消费面，与 ipara 腿 F→P 4 同款 delta
正面；pre-delta 该五案 caret 落位走 legacy 分支翻 compare）。fd 腿同口径
判读待落（11:40:33 起）。

**delete 腿实测（11:03:09→11:40:33 落地，含 s254 bin）：7,344P/291F/1T
rc=2——P→F = 0，`[foo<X>bar]</X>` 族零扩散**。对基线 E1 7,339P/296F/1T
（g28-del-e1-fails.txt 297 行，Fail+Timeout 归一化行集 diff）：
ΔP=+5/ΔF=−5 恰互偿，Timeout 行集同基线（1T 恒等）。F→P 5 全为
`{...}` 部分含形——`foo{<span>bar</span>}baz` ×2（swcss false/true）+
`foo{<b>bar</b>}baz` ×2（swcss false/true）+ `<p>foo{<span
style=color:#aBcDeF>bar</span>}baz` ×1 compare innerHTML——`{...}` 边界
origStart 在删程外存活、caret 臂正常翻转，与 ipara 腿 F→P 4 同族正面
连带（deleteSelection 为 ipara 前导 + delete.html 直驱双消费面）。
**判读：delete 腿过门（P→F 零）**——delta caret 臂在 delete.html 语料
面净正面；修复笔存活门落地后本腿预期逐位恒 v1（五翻转面不经存活门，
不受回落语义影响），复跑仍按行集 diff 复核。

## 6. 残余与挂账

- ipara/ilb 家族内容丢失/caret 世界失步（C30 typed 段整丢、C55 ilb 双写形）
  ——本轮 delta 改变其 delete 出口 caret 后失败模式位移，独立轮归因；
  修复后终判读残余 21F 全落此面（探针维 ipara 13 + ilb 8），第十五轮已认领
  （前置 part01f.js 提交已落地）；
- dshell2 M1/M3/M5 caret 落 `div@1`（=t:abc@3 边界等价点表示差异；inserttext
  同点消费 C01-C14 全绿实证 multitest DOM compare 不可见）——语料面无断言
  selection 锚点形态，仅记档；未来出现 selection 断言语料再收口；
- 电池 v1 双腿 SIGSEGV/GPF 一次性崩溃对按「环境性/低频一次性」口径消解
  （E1/diag 双 bin 干净全量，重跑法归因穷尽）——若再现以新样本重开；
- multitest ?9001-last solo 切片过滤语法（`multitest.html?9001-last` 直跑
  rc=1 即败）未通，−85 缺口以 E4 同 bin 全腿重测对账替代；
- qV 残余 ~109F、list FIXME、delete li/dt/dd FIXME、shim
  DocumentFragment.appendChild 移动语义/Range.extractContents 共享根（控制
  面下一步 ②-④ 原样顺延）。
