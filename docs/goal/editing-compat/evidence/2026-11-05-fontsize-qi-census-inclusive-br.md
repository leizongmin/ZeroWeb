# ④ fontsize qI-after ×2 收口：qI 普查独立含 br（Chrome delta）

日期：2026-11-05 深夜窗口（门禁跨零点到 11-06）　基线：367afa4ff（⑭ 后）
改动：crates/engine/src/js_dom_shim/part01c.js +21/−2（单文件两刀）

## 1. 目标与归因（控制面已闭合，本轮执行）

读数面 8 行中剩 fontsize br qI-after ×2（语料 fontsize.js :38/:43 双变体）：
fixture `<p>[foo<p><br><p>bar]`，execCommand fontsize=4 后 Chrome 录音
`{"fontsize":[false,false,"3",true,false,"4"]}` 要求 qI after=true。引擎侧
⑦ 落地的 `_zwJusIsVisible` br 臂（extraneous br 判不可见）使 qI 普查
（`_zwValGetAllEffectivelyContained` + `_zwValIsFormattable` 过滤）剔掉选区内
未包裹 br → 普查集均一（两个 span 同值）→ qI=false。Chrome 实际普查含该 br
（br 缺省 16px→legacy "3"，与包裹 span "4" 混合）→ qI=true。vendored
implementation.js isFormattableNode 的 isVisible 过滤与 Chrome 录音相悖，
语料为权威。qV 首节点口径不受影响（③' 轮已对齐）。

## 2. 实现两刀（part01c.js）

1. 新谓词 `_zwValIsFormattableQI`：br 短路 `_zwJusIsEditable(node)`
   （绕 ⑦ extraneous-br 可见性过滤，保留 editable 门）；非 br 走
   `_zwValIsFormattable` 原语义不动。br 短路免掉 FormattableVisible 下行
   （trap 域成本同既有 `_zwValIsFormattable` 门）。
   https://w3c.github.io/editing/execCommand.html#indeterminate
2. `__zw_ce_value_query` 普查改用 QI 谓词（单遍走查，indeterm 混值检查含
   extraneous br）；qV ref = 普查序首个通过 `_zwValIsFormattable` 的节点，
   无则回落 rng.startContainer（原语义）。

**qV 零漂移实证**：走查实现 `part01c.js:2117-2132` 核实 condition 为纯过滤
（:2126 `&& condition(node)` 仅在 push 判定，爬升/遍历/终止与 condition 无关）
→ qV 过滤集 ⊆ QI 普查集且同文档序，首过滤节点与旧全量过滤走查 nodes[0]
恒等。目标 fixture 的 qV 行两变体保持 Pass。

## 3. 门禁终账

探针（qI-only 二进制；bin-qi-marker=3、bin-mtime > src-mtime 双批验证）：

- zz-probe-qv5 十钉 10/0F（q8 主批 + wt2 worktree 双批一致）。
- zz-probe-ab 四案 4/0F（③'-fs 修复面不受扰，ledger 落账）。

五语料（q8 主批，verifier 23:20 独立复核；fontsize 另做 2,422 行全行 diff）：

| 语料 | 行数 | F | 判读 |
|---|---|---|---|
| fontsize | 2,422 | **3** | 5F→3F：全行 diff 恰 2 目标行（qI after 双变体）Fail→Pass，其余 2,420 行零位移；同 fixture qV 行保持 Pass |
| forecolor | 2,327 | 5 | 红集对 q7 逐字节恒一 |
| backcolor | 1,026 | 0 | 恒一 |
| hilitecolor | 1,218 | 0 | 恒一 |
| fontname | 2,225 | 3 | 红集对 q7 逐字节恒一 |

multitest（jccut 基线 10,077 行，行级元组放行门 P2F/newF=0）：

- q8 主批片 1-3000 + 5001-9000（verifier 23:26 mtime 审计 = qI-only 二进制，
  23:08 f1 part01e 写盘重建前）：逐片 fail 30/24/34/24/49/59/47/14 对基线
  逐值恒一；九片判读（含缺片侧）F2P=0/P2F=0/newF=0。
- wt2 worktree qI-only 补片 **4001-5000 = 1,000 行/50F，红行逐名对基线恒一**
  （该带含 480 条 backcolor qI/qV 断言——走 `__zw_ce_value_query` 改动路径的
  最敏感带，verifier 23:21 点名要求的纯净腿）。
- wt2 worktree qI-only 尾片 **9001-last = 1,029 行/79F，0 个基线外新红**
  （行数 1,029/1,080/1,092 三代各异 = ⑨/⑪ 尾带完成深度漂移口径；fd 行
  `abc[]<br>` 按 tailband-fail-census = round-13 起稳定红非 flake，
  tuple-invisible vs jccut）。
- q9 全扫（qI+F1 混合二进制，f1-resume 轮的批）：本人独立复核判读
  **P2F=0/F2P=0/newF=0/lost=5**（lost 全为 delete/forwarddelete 面截断带行，
  与 value-query 面无关；verifier 23:50 同判）。
- q8 尾片 + enginelib 原腿因 23:08 混二进制污染（verifier 23:26 旗）作废，
  由上两行 wt2 qI-only 腿替代。

enginelib（qI-only worktree 腿 wt3b，wt/target 软链 /home 真盘 + systemd
MemoryMax=24G + test-guard --compile-first）：engine lib 二进制
**3,740 passed / 5 failed**（14.96s）逐值对齐基线，五红名
p56/p99s/r3276/r2810/r2809 逐名恒一，零新红；q8 混二进制腿同值作旁证
（verifier 复核入账）。

## 4. 门禁批基建事故存证（三条，均不涉行为）

1. **q8 首批被杀**：后台任务 10 分钟时限误配，杀于 mt 片 4001-5000 运行中
   （0 行残桩）；EXIT trap 复原语料（Q8-RESTORED，metas=10 对账）。续批
   （q8b）完成片 5001-9000 + 尾片 + enginelib。
2. **混二进制污染（verifier 23:26 定谳）**：f1-resume 23:08 前 part01e 写盘
   触发 q8b 中段 rebuild → 尾片 + enginelib 跑在 qI+F1 二进制上。 Remediation
   = worktree 隔离（367afa4ff + 仅 ④ diff，不动他轮在飞文件）。
3. **worktree 语料/磁盘三连**：① 语料复制撞 q9 切片期（1-variant 带毒态，
   metas=1 被我误读为 10——echo 硬编码串骗了自己，slice 腿 1 行伪影，
   wt2-ABORT 守卫拦下）；② dev 构建两度写满 20G tmpfs（wt-qi/target 17G，
   f1-resume 两轮 DISK-RESCUE 有据；改 CARGO_TARGET_DIR→/home + wt/target
   软链后解决）；③ 语料缺 `/common/subset-tests.js` 整页崩 1 行 Fail
   （worktree 语料最小集 = editing+resources+images+common，逐路径核对存在性）。
   q8 日志文件家族于盘满事故后被清（无认领 NOTE）；其判读结论由 verifier
   23:20/23:26 独立复核 + ledger 逐腿行 + 本档引用三方存续，证据效力不受损。

## 5. 判读与挂账

**④ 读数面 8 行全账收口**（forecolor purple ×6 已随 ③'-c ⑬ 轮治愈 +
fontsize qI-after ×2 本轮 F2P）。放行门全绿：P2F=0、newF=0（documented
fd 尾带行除外，口径不变）、enginelib 五红名恒一。落地提交 =
part01c.js 单文件两刀 + 本档 + 控制面同步。

挂账不变：③' 存活读数面（forecolor 余 5 行之 1/3/4/5 族 + fontsize 残 3 =
table qV + straddle qI/qV 同族）、⑥ J-B table 16F / J-C 跨壳 br ~44F、
part01c.js 2,294 行超限拆分、F1/F2 大项（F1 已由并行轮推进中）。
