# ③'-c 作者样式表值半边 + ⑤ fontname plain-first —— fix15a 门禁与判读

日期：2026-11-05（Round 5 实施轮）
刀面：`crates/engine/src/js_dom_shim/part01c.js`（+116/−2，未提交时记录）
对照基线：fix14e（7b6077b92 后，/tmp/qv-fix4/fix14e-*.log）
门禁批：systemd `zw-fix15a-gate`（14:15:47 起，24G service 模式，无代理变量与 fix14e 同口径），脚本 `/tmp/qv-fix4/run-fix15a.sh`

## 1. 机制事实与刀面定义

host gCS 桥（`_zwJusCs`）对 `<link>` 作者样式表盲（合成源契约 P9-9/R5107 面）：

- forecolor `#purple`（reset.css `#purple{color:purple}`）qV-before 读黑（= 初值填充），qI-before 混合选区读假——语料 forecolor.js :440-459 累计 6 值行 + 2 compare 行红。
- fontname tag 模式 pre/listing 族 qV-after 读 serif 而非 sans-serif（⑤ 存活面 3 行）。

修复三笔（全在值面通道，apply 面不动）：

1. **作者样式表扁平快照三件套**（`_zwPlainAuthorSnapshot` / `_zwPlainAuthorMatches` / `_zwPlainAuthorDecl`）：`document.styleSheets` → per-sheet `cssRules` 扁平收集（16 sheet / 256 规则封顶），仅收 style 规则（selectorText+style 双在）；简单选择器匹配（tag/.class/#id/:link/:visited，正则解析拒绝组合器/属性/通配/其他伪类——越界宁漏配不误配）；末次匹配规则胜 + getPropertyValue 优先 cssText regex 兜底。快照页生命周期缓存一次（styleSheets getter 每访问走桥面全量重查，shim trap 域成本不可见 R350 教训；语料壳页静态 sheet）。
2. **`__zwPlainComputed(elt, prop, qvMode)` 第三参门控作者臂**：插在 inline decl 之后、font 提示臂之前（作者 normal 声明 > 表现提示 > UA 规则）。**仅 qvMode（值面通道）开**：apply 面 ownFacePre/parFacePre 等价分组消费方对近端作者声明可见性敏感——#purple 臂无门控首版实测 apply 面 z/ba 序 scramble、pre/listing compare got `bazfoo` 跨块并段（A/B stash 对照证伪"apply 面回归"假设后改门控收窄，见 §3）。作者臂在 apply 面的等价分组语义归 ③'-a/b/d 同窗挂账，本刀不动。
3. **`_zwValEffectiveValue` fontname/forecolor plain-first**：`__zwPlainComputed(node, prop, true)` 优先（含 inline、作者规则、font 提示、UA 臂——完整近端级联），空回落 `_zwJusCs` host 读数（P9-9 通道口径保留）。fontsize 档位面维持 host 优先不动（getLegacyFontSize 消费 host 16px 缺省口径）。

⑤ face 提示：plain 走查既有 `font[face]` attr → font-family 臂本就覆盖 tag 模式映射；此前 ⑤ 存活面不绿系值面查询通道走 host gCS（face 提示盲）而非 plain 走查——plain-first 后 face 提示 + UA mono 臂 + 作者 body serif 全部可达，无需新映射臂。

## 2. 探针（zz-probe-authface.html，14 Pass / 0 Fail 常驻入库）

- **S1 数据源钉**：`document.styleSheets` 可达 reset.css `#purple` 规则（修复前后恒绿，守桥面）。
- **T1/T2 目标面**：`#purple` qV-before 黑→紫（rgb(128, 0, 128)）、混合选区 qI-before 假→真。
- **T3 apply 机制钉**：`#purple` ba[r ba]z 外层 blue 壳下内层包裹子串（`<span id="purple">ba<font color="#0000ff">r</font></span>` + `<font color="#0000ff">ba</font>`）。
- **T4 外层壳下 qV 紫**；**T5 pre before serif + qI 混合**；**T6/T7 pre/listing apply 后 qV=sans-serif + qI 均一 false**。
- **G1-G8 守卫**：a[href] UA 臂 / font color attr / plain serif / fontsize "3" / hilitecolor 透明爬升 / sw=true inline 读数 / b 元素不串色 / ⑫ Arm1 bg-merge 载体并值形。
- **zz-probe-carrier** 32 Pass / 0 Fail（③ 载体形守卫回归面，J1-J5 join 判据守卫在内）恒绿。

## 3. A/B 定谳：探针上下文既有伪象（非回归）

首版实现（作者臂无 qvMode 门控）探针 T3/T6/T7 全量 compare 红（got `z<font>ba</font>` 序颠倒 / `bazfoo` 跨块并段），假设 apply 面 join/merge 被作者臂打乱。两步定谳：

1. qvMode 门控收窄后探针 got 逐字节相同 → apply 面假设证伪（门控前后 apply 面不可见作者臂）。
2. git stash 变更前二进制对跑同探针：T3/T6/T7 compare **变更前同红** → 定谳为探针页与 corpus 页对跨块/尾段 wrap 序的**既有环境差异**（corpus 同 fixture 行 compare 基线绿：fontname.log swF 行 compare Pass）。T3/T6/T7 改 loose 模式 + 机制子串钉测，行级权威归 corpus 门禁。

## 4. 门禁判读（fix15a 对 fix14e，行级 fail 集元组）

| 语料 | 基线 | fix15a | 判定 |
|---|---|---|---|
| fontsize | 2,417P/5F | 2,417P/5F | 恒等 ✓ |
| fontname | 2,222P/3F | 2,222P/3F | 恒等（未治愈，归因见 §6） |
| forecolor | 2,310P/17F | 2,318P/**9F** | **F2P 8 / P2F 0** ✓ |
| backcolor | 1,026P/0F | 1,026P/0F | 恒等 ✓ |
| hilitecolor | 1,218P/0F | 1,218P/0F | 恒等 ✓ |
| enginelib | 3,731P/5F | 3,731P/5F | 五红名逐名恒一 ✓（p56_adopted/p99s_custom_property/r3276/r2810/r2809） |

**forecolor F2P 8 行全 #purple 系**（③'-c 直接收益）：ba[r ba]z qV/qI before ×4（swT+swF）+ b[a]r 外层壳 qV before ×2 + **b[a]r compare ×2（超预期治愈）**。compare 治愈机制：apply 面等价早退路径（`_zwVaLooselyEquivalent`/`_zwVaEquivalentValues`）直接消费 `_zwValEffectiveValue`——plain-first 让 effective value 读到作者紫，判等正确早退跳包。属值面修复经 apply 判等消费点的**间接收益**，不违背 ③'-a/b/d 挂账（ownFacePre/parFacePre 分组语义另面）。剩余 9F 全既有存活面（table qV after / blue>brown 嵌套壳 compare ×3 / brown 跨元素 qI/qV after / qS-before ×1 / 双 span compare ×2），集合闭合零新增。

multitest 10 片判读见 §7。

## 5. 预期收益对账

- fontname 3F→0F：**未命中**（3F 恒）——⑤ 修复对 corpus 页未生效而探针同 fixture 全绿，归因窗 §6。
- forecolor 17F→11F：**实际 17F→9F 超预期**（compare 2 行连带治愈）。

## 6. fontname 3F 恒归因窗——首轮定谳：值面修复在 corpus 同壳页生效，红行系同片前序行状态污染

红行：pre `[foo<pre>bar</pre>baz]` qI/qV after + listing `fo[o<listing>b]ar</listing>` qV after（行同基线，P2F 0）。

矛盾面：pre 行 fixture 与探针 T6 **完全相同**（fixture + cmds + impl.js 同构建二进制），探针 qVa=sans-serif / qIa=false 全绿，corpus 仍红；且 corpus compare 双模式 Pass（DOM 形态正确、值读数错）。runner setup（tests.js :5558 `innerHTML = "<div contenteditable></div><p>test"`）与探针 runCase 逐字相同；tests.js 无全局样式注入（grep insertRule/createElement("style")/body.style 零命中）。

**corpus-clone bisect 页首轮（zz-probe-fncorpus.html ?1-1000，fontname.html 壳逐字 + `data/zz-fnmini.js` 仅含目标 2 行、零前序行）**：pre qI/qV after + listing qI/qV after **全 Pass**（rows=34 fails=2，唯二 Fail = `queryCommandState("stylewithcss") before` ×2 = ⑩ 轮已定谳的 bootstrap qS-before quirk 另面非本刀面）。双跑互证：并行轮 E1 腿 75s + 本流补跑 64s，rows/fails/红行集逐行同构（fix15b 碰头让位见 RUNSTATE）。

**定谳**：三笔修复（含 qvMode 作者臂 + plain-first）在 fontname.html 同壳页环境生效——corpus 全量 3F 恒红的机制 = **同片前序语料行留下的页级状态污染**（qV_b="serif" 绿不作反证：作者臂命中与 host 回落读数同值不可区分）。⑤ 存活面 3 行（语料权威面）维持挂账；修复本身正确性由探针 T5-T7 + fncorpus 双跑钉死。

挂账（下轮）：行级 bisect 前序污染源——`data/zz-fnmini.js` 的 `// BISECT-SEG` 标记处注入 fontname.js 前 N 条语料行（python 切数组体按 `^["{]` 条目边界），log2(~1300)≈11 轮 ×~70s；辅证腿 fontname `?2001-last` 单片（并行轮 E2 rows=0 失败未跑成，红行在 ?1001-2000 与 ?2001-last 两片独立进程均复现 → 污染在同片内前序行，非片间）。

## 7. multitest 10 片判读（judge-fix15a-mt.py 行级元组，对 fix12b-multitest.log 基线）

**F2P 0 / P2F 0 / newF 1 / lost 1**——两异常均尾带伪影（⑨/⑪ 口径）：

- **lost 1** = 基线合并日志片 10 的文件级 `Timeout editing/run/multitest.html?9001-last` 伪影行（fix12b 批腿片 10 在 tests=1077 截断、pending=1；判读权威的 fix12b solo 复核 1,092 行日志未留存文件）。片 1-9 行集逐位恒等（fail 数 30/24/34/24/50/49/59/47/14 逐值同）。
- **newF 1** = `[["forwarddelete",""],["inserttext","d"]] "<p>abc[]<br> </p> <p><br></p>" compare innerHTML`——落基线截断点后基线从未执行的 fd+insertText compare 新执行带（⑧ 轮 31 newF 同族同带先例非行为回归；comm 实证该行属片 10 基线未执行的 16 行差集）。fix15a 片 10 = 1,092 行/85 fail，与 ⑫ 记录的 fix12b solo 复核（1,092/85）计数恒一，且对 fix12b 批腿 84 fail 集 **F2P 0**（84 红行全数仍红，零翻转）。

enginelib 3,731P/5F 五红名（p56_adopted/p99s_custom_property/r3276/r2810/r2809）对 fix14e 逐名恒一；探针 authface 14P/0F + carrier 32P/0F。**verdict OK：零 P2F，forecolor F2P 8。**

## 8. 挂账不变

- ③'-a（等价壳分裂漏包 ×2）/ ③'-b（tag 模式 carry font 重发 ×1）/ ③'-c-apply 半边（作者臂 apply 等价分组）/ ③'-d（ghost br ×2）。
- ④ fontsize br qI-after ×2。
- ⑥ J-A/J-D justifycenter 祖先壳改写刀（~42 行）。
- ⑦ F1/F2。multitest ?9001-last 截断带口径（⑨ 轮骨架 flake 族判定不动）。
- 探针上下文跨块/尾段 wrap 序伪象（§3，A/B 已证与源码无关，不修）。

## 9. 补充定谳（第二轮核验）：fontname 3 红机制升级——variant 切割伪象（§6「前序行状态污染」的修正，bisect 挂账改判）

§6 定谳「同片前序语料行留下的页级状态污染」经第二轮核验（克隆壳页 zz-fn-clone.html
原序复现 + 原序现场 queryCommandValue 包装器 dump + corpus 行集切齐审计）**升级为
variant 切割伪象**——机制在 subset-tests.js 的 subtest 窗口过滤，不在前序行状态：

1. **机制**：tests.js 每 case 的 subtest 计数连续（exec 断言 ×N → checks → compare →
   before 查询断言 ×6 → after 查询断言 ×6）。`subsetTest`（subset-tests.js）对窗口外
   subtest **直接 skip**，而 **execCommand 调用在 exec 断言的 fn 体内**——窗口外 exec
   断言被 skip ⇒ **execCommand 从不执行**；同 case 的 after 查询取值（tests.js
   :5618-5628）不在 subsetTest 内、**无条件同步跑** ⇒ 读到未 apply 的裸 DOM ⇒
   `qI after=true / qV after="serif"` 恒红。
2. **切齐铁证**（fix15a-fontname.log）：红案 `[foo<pre>bar</pre>baz]` swF 共 16 个
   subtest，被 `?1-1000`/`?1001-2000` 窗口线从中间切开——**?1-1000 段 11 行全 Pass**
   （exec ×2 + checks + **compare innerHTML** + stylewithcss 查询 6 + fontname qI
   before；exec 真跑、apply 正确、compare 包裹形绿），**?1001-2000 段仅 5 行**
   （fontname qS/qV before + qI/qS/qV after，**无 exec 行**——execCommand 在该页从未
   执行）。16=11+5 精确切齐；污染假说若成立，?1001-2000 段应有 exec 行（真跑后被污染
   读错），实测无 ⇒ 切割机制唯一自洽。listing `fo[o<listing>b]ar</listing>` qV after
   同理（红在 ?2001-last 段，exec 断言落窗前段被 skip）。
3. **原序现场实证**：克隆页（壳页同构 + sw=true 起手 hack + 全量原序 + variant
   ?1001-2000）复现 pre 双红；页内 `document.queryCommandValue` 包装器对含 `<pre>`
   案的每次 fontname 查询 dump（选区端点 + 当前 contenteditable DOM）——**所有
   qV=serif 调用时刻 DOM 均裸形**（`foo<pre>bar</pre>baz`），无一次包裹形 ⇒ 值读数
   全部发生在 apply 之前。
4. **bisect 挂账改判（重要，防假阳性）**：§6 挂账的行级 bisect（zz-fnmini.js 注入
   fontname.js 前 N 条语料行）**注定得到假阳性**——注入前序行会使目标案 subtest 计数
   后移，跨过 variant 窗口线后 exec 断言被 skip，「复现红」测到的是切割伪象而非前序行
   状态。三红系切割伪象恒红（历代基线同红一致，?1-1000 段全 Pass 同代在库），行级
   权威改以零切割环境为准（zz-probe-authface T5-T7 / zz-probe-fncorpus 零前序裁片，
   双环境全绿已钉）。语料页内同案跨段断言切割是 corpus runner 的既存运行学特性，
   不属 engine 修复面，不改 runner。

5. **判别窗终审实测（判读方独立证据，与上四证同判互证）**：fontname.html 裁单窗
   `?1001-1006`（红案 after 断言 #1003-1005 窗内、exec 断言 #990-991 窗外）——
   切割伪象假说预测红（exec 被 skip → after 读裸 DOM）、污染假说预测绿（无前序
   exec 应更干净）；**实测 qI/qV after 双红** = 切割机制实锤。精确断言编号
   （grep 断言行重编号）：case 64 = #990-1005，片 1 跑 #990-1000（exec ×2、checks、
   compare、sw 查询 6、fontname qI before 全 Pass）、片 2 跑 #1001-1005。判别页
   已删（断言编号绑定语料排列无常驻价值，方法在此记档）。
6. **同族权威面补证**：case 62（`[foo<listing>bar</listing>baz]` swF 全选，落
   ?1-1000 窗内完整执行序列）在 fix14e 基线与 fix15a 均 qI/qV after **Pass**——
   tag 模式 face 读数在真实执行路径上基线已正确，⑤ plain-first 修复保留为正确
   语义收口（face 提示 + UA mono 臂），语料端无红可治。bisect 工作文件
   zz-fnmini.js 不再消费，可清理。

