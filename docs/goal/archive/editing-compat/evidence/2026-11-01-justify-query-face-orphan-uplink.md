# justify 查询面 uplink 断裂兜底——setter 孤儿 wrapper 读数链修复（2026-11-01）

## 0. 结论速览

| 门禁 | 本轮 | 上一账 |
|---|---|---|
| justifyright 真页（3 variant 全分母） | **3,667P/1,147F/0T**（+220P） | 3,447P/1,367F/0T |
| justifyfull 真页（3 variant 全分母） | **3,622P/1,140F/0T**（+220P） | 3,402P/1,360F/0T |
| justifyleft 真页（3 variant 全分母） | 2,290P/213F/0T **逐位恒等** | 同左 |
| justifycenter 真页（8 variant） | 交集对账 **F→P=222/P→F=0**/onlyNew=0（全分母当日环境截断，五跑 + 双重 A/B 定谳与本轮 diff 无关，见 §3.4） | 4,728P/1,359F/0T |
| zz-jrprobe 探针 | 6 形状全绿（v=right s=true） | （新建探针，commit 前删） |

逐测试 ID 对账（整行减判定为 ID，全分母）：jr common=4,814 F→P=220 **P→F=0** onlyOld=0 onlyNew=0；jf common=4,762 F→P=220 **P→F=0** onlyOld=0 onlyNew=0；jl common=2,503 0/0 逐位恒等。分母逐位恒等 ⟹ 无截断无漂移。

## 1. 根因：`host.innerHTML=` setter 产物子 wrapper 是孤儿

上轮 `_zwJusWrapViaHtml` flat 模型 wrap 走 `host.innerHTML =` setter 整段重写（sanctioned 写路径——裸 DOM 搬运不回流读数，上轮探针 w3/w5 实证）。代价：setter 产物子 wrapper ** parentNode=null**——`div.childNodes` 可枚举（nKids=2）而 `child.parentNode=null`、兄弟链 null（t8r `_wcRebuildAsHandle` detached 物化树的形态）。后果两条，queryCommandValue/State 三面同断：

1. `_zwJusGetAlignmentValue` 的 parentNode 爬链（祖先显式提示：style text-align / align 映射集 / center 标签）对孤儿直接落空 → Value 恒缺省 `'left'`；
2. `_zwJusIsContained` 的 BpCompare 祖先链同断 → 查询 contained 叶集合异常 → State 恒 false。

zz-jrprobe 探针 `span-caret-between` 形状实证：execCommand("justifyright") 后 innerHTML 正确（`<div style=text-align:right>…`），而 `queryCommandValue="justifyright"` 返 `left`、State 返 false。justifycenter 旧日志同构失败（127× "center" got "left"）、right/full 同构——**共享查询面残缺，非本轮 wrap 引入**（上轮簇落账时点已存在，jr/jf 各 ~417 Value + ~410 State 每页）。

## 2. 修复三笔（part01c.js +102/−3）

1. **宿主 stash**（`__zw_ce_justify_apply` 入口）：apply 时点 startContainer 上行链完好（wrap 前节点），爬链找 editing host 存 `globalThis.__zwJusLastHost`。换代写覆盖，陈旧 stash 下钻不中 → 链 null → 保留直读缺省，无副作用。
2. **`_zwJusQueryRoot`**：读数时自 range.startContainer 爬链找宿主；爬链断（孤儿端点）→ stash 兜底。健康路径 O(depth) 即返回，无 DFS 成本。
3. **合成链读数**（`_zwJusChainDown` + `_zwJusAvFromChain` + `_zwJusQueryAlignment`）：只对「直读返缺省 'left' 且上行链不可达宿主」的节点触发 DFS 下钻——自宿主 `childNodes` 逐层下探合成祖先链（孤儿 childNodes 可枚举），`_zwJusAvFromChain` 自内向外（target 端先）跳过非元素/inline 显示节点取最近块级元素 computed text-align，缺字面值（''/start/end）时沿链外扩做显式提示补读（style attr regex → center 标签 → align 属性映射集，镜像 `_zwJusGetAlignmentValue` 判序；`_zwJusAlignmentHint` 的 parentNode 爬行对孤儿不可用，链外扩等价替代）。链上无提示 → null（保留直读缺省——真左对齐不受影响）。

门在 `_zwJusQueryAlignment` 入口：`directAv !== 'left'` 或可达宿主即直返——修复面恰覆盖「apply 后应读非 left 值而读数链断」的形状，left 期望路径零接触（jl 逐位恒等的机理，见 §3.2）。

## 3. 验证明细

### 3.1 探针（zz-jrprobe，6 形状，commit 前已删）

ctrl-basic、span-caret-between、sibling-align-attr、sibling-align-style、table-brace、center-multi-block——修复后 6/6 全绿（apply 后 v=right s=true i=false）。探针三轮插桩迭代定位根因：nKids 可数而 parentNode=null → 兜底下钻 → 链上读出 right。

### 3.2 三页全分母 + 逐 ID 对账

- jr：3,447P/1,367F → **3,667P/1,147F**（+220P）、jf：3,402P/1,360F → **3,622P/1,140F**（+220P）、jl 逐位不动。
- +220 结构：110 个（命令表×setup）对 × 每对恰 2 条（`queryCommandValue after` + `queryCommandState after`）——Indeterm/innerHTML/return-value 全不动，**修复面恰为查询面**。
- 翻绿 setup 形状族（jr/jf 分布全同，37 unique setup）：align 容器多块 20、其他 10（pre/h1/dir=rtl start/quasit 等）、table 选区/cell 4、span-caret 1。机理两型：wrap 产物孤儿（链上读新 wrapper 显式 style）；apply 后祖先提示本应可达但 contained/爬链面同断（BpCompare 祖先链失效面）。
- **jl 逐位恒等机理**：justifyleft 期望值即 `'left'`，断裂读数恰返缺省 `'left'`——负负得正，Value/State 意外正确；残余 213F 全在 innerHTML/merge/setup-exception 族，查询面修复不触及。唯一纯查询候选 setup 为 setup 抛异常族（`TypeError: reading 'color'`），非查询面缺口。

### 3.3 justifycenter 交集对账

共享查询面同款收益：对上一账 4,728P/1,359F 全分母日志交集 common=5,695，F→P=202、**P→F=0**、onlyNew=0（无新增行无新增失败）。

### 3.4 门禁电池（systemd 单串行批，test-guard 包裹）

- 七语料面（逐文件分跑）：join 三案 3,456P/0F/0T + 2,607P/0F/0T + 1,455P/0F/0T = **7,518P/0F/0T**、wss **1,104P/0F/0T**、ilb **552P/0F/0T**、chd **420P/0F/0T**、void 双语料 **72P/0F/0T + 48P/0F/0T** 全零位移。
- ihost 宽 filter `inline`：**883P/74F/0T**——共同分母 888 行对上一账逐位恒等（880P + 8F 既有：shadow-root delete 4F + placeholder 伪元素 4F）；新可见 69 行（inserthtml-do-not-preserve 54 + removing-inline-style 12 + paste-in-list 1 + br-tag 1 + delete-at-end-boundary 1）= **多页合跑静默截断方差**（昨 888 行/今 957 行分母，10-31 已记教训）——其中 66F 经 **pristine HEAD A/B 定谳既有**（stash 本轮 diff 重建基线二进制重跑五页，逐位同败：54F/9F/1F/1F/1F 全复现），3P 为 removing-inline 单页通过断言。本轮 diff 机械不可能触及（justify 查询面函数不被这些页面调用）+ A/B 实证双重定谳。
- engine lib **3,642P/1F**（s24 已知红唯一，`slice24 known-deviation F2` panic 逐名 identical；+1P 他流入账正漂移）；webview lib **834P/0F**（+2P 他流漂移）。

### 3.4a justifycenter 全分母不可得定谳链（五跑 + 双重 A/B，挂账下轮归因）

jc 页当日五跑全被 `?5001-6000` variant 截断，且**与本轮 diff 无关**：

| 跑 | 二进制 | case timeout | ?5001-6000 行数 | 结论 |
|---|---|---|---|---|
| 昨 jc-real（基线） | 修复前 | 150s | 884（全） | 昨日环境跑完 |
| 今 jc | 本轮修复 | 150s | 884−391 | 1T |
| 今 jc2 | 本轮修复 | 300s | 884−55 | 1T |
| 今 jc3 | 本轮修复 | 600s | 884−20 | 1T |
| 今 jc4 | 修复+**DFS 关断** | 150s | 884−530 | 1T——DFS 非因 |
| 今 jc5 | 本轮修复（1min load 1.2 静窗） | 300s | 754 | 1T——负载单因非 |
| 今 pristine A/B | **stash 全部本轮 diff** | 150s | 171 | 1T——**与本轮 diff 无关** |

定谳逻辑：DFS 关断（jc4，工作量严格小于基线）仍截断且更重 → DFS 非因；静窗（load 1.2）仍截断（jc5）→ 负载非单因；**裸 HEAD（无本轮任何改动）同样截断且最重（171 行）** → 变量在本轮 diff 之外。交集对账不受影响：jc2 交集 common=6,032，F→P=222、**P→F=0**、onlyNew=0（first run 交集 202 同判）。

**挂账（跨域，非本轮 scope）**：`?5001-6000` variant 墙钟异动——昨日（b171ddbf9 提交时点树）150s 跑完 884 行，今日同树 pristine 仅 171 行。两个候选因待下轮归因：① 当日外部负载浪潮（他项目 ZeroSeed agent 全速构建/测试，15min load 峰值 16.69，与截断时点强相关但静窗反例不全消）；② rebase 并入的他流面（dc49d1e7c CE 环 details 快路 touches 物化路径 / 7098bd67e layout float）——jc-real 是否 rebase 前二进制待考古。归因路径：静窗 + 逐提交 bisect（revert dc49d1e7c 单跑 variant）。本轮不改对方代码（撞车仲裁「消费域收敛、不回退对方修复面」先例）。
- jr 复跑（剥桩后最终工件）：**3,667P/1,147F/0T，`cmp` 逐位 identical**——剥桩零行为位移，头条数字绑定交付工件。
- 七语料面（逐文件分跑）：join 三案 3,456P/0F/0T + 2,607P/0F/0T + 1,455P/0F/0T = **7,518P/0F/0T**、wss **1,104P/0F/0T**、ilb **552P/0F/0T**、chd **420P/0F/0T**、void 双语料 **72P/0F/0T + 48P/0F/0T** 全零位移。
- ihost 宽 filter `inline`：**883P/74F/0T**——共同分母 888 行对上一账逐位恒等（880P + 8F 既有：shadow-root delete 4F + placeholder 伪元素 4F）；新可见 69 行（inserthtml-do-not-preserve 54 + removing-inline-style 12 + paste-in-list 1 + br-tag 1 + delete-at-end-boundary 1）= **多页合跑静默截断方差**（昨 888 行/今 957 行分母，10-31 已记教训）——其中 66F 经 **pristine HEAD A/B 定谳既有**（stash 本轮 diff 重建基线二进制重跑五页，逐位同败：54F/9F/1F/1F/1F 全复现），3P 为 removing-inline 单页通过断言。本轮 diff 机械不可能触及（justify 查询面函数不被这些页面调用）+ A/B 实证双重定谳。
- engine lib **3,642P/1F**（s24 已知红唯一，`slice24 known-deviation F2` panic 逐名 identical；+1P 他流入账正漂移）；webview lib **834P/0F**（+2P 他流漂移）。

### 3.5 工件口径注记

- jr/jf/jl 三页逐 ID 对账跑于剥桩前二进制（含 `__zwJusDbg` 未引用导出）；剥桩 = 纯删除未引用函数定义，`node --check` PASS；**剥桩后最终工件复跑 jr：3,667P/1,147F/0T `cmp` 逐位 identical**——交付工件与头条数字绑定。
- DFS 关断 bisect 探针（TEMP-BISECT 一行）已恢复剥离，最终 diff +102/−3 与修复面一致。
- jc 全分母未能于当日复现（§3.4a），头条以交集对账记账；环境恢复后静窗复跑可补全分母数。

## 4. 残余与挂账

- **jc `?5001-6000` variant 墙钟异动**（§3.4a，跨域挂账）：pristine A/B 复现，与本轮 diff 无关；下轮静窗 + 逐提交 bisect 归因（dc49d1e7c lineage / 环境两候选）。
- **merge FIXME 族维持**：jr ~38 + jf ~36 + jl ~15 setup 为块级合并缺口（Chrome 把块移入相邻对齐容器，wrap 路径未实现，FIXME 已标 part01c wrap 节）——三页剩余 F 的主族。
- setup-exception 族：`TypeError: reading 'color'`（jl 1 处）等 setup 抛异常，非 justify 域。
- quasit align 4F：引擎 computed style 未知元素 `text-align:inherit` 求值债（渲染流域，不跨界，上轮已记）。
- **下一步候选**：style-with-value 簇（5,890F）立项归因（控制面簇序次轮后半）。

## 5. 门禁命令口径

- 全页对账：`ZW_CORPUS_CASE_TIMEOUT_SECS=150 ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1500 -- ./target/release/zero-wpt-runner testharness-editing-run justifyright`（jc/jr/jf/jl 同型）。
- 七语料：case timeout 60、guard 600-900，逐文件分跑（多文件合跑静默截断教训）。
- 双 lib：`cargo test -p zero-engine --lib` / `cargo test -p zero-webview --lib`（test-guard 包裹，DISPLAY=:20）。
- 对账脚本口径：整行减判定（`Pass|Fail ` 前缀剥离）为 ID，交集计数 F→P/P→F/onlyOld/onlyNew；setup 归并键 = 命令表 + setup 串。
