# DC-2 color 批收口：part03 raw 影子条件 + mix 插值空间别名折叠

日期：2026-11-11 · 分支 main · 范围 css-color computed none 保形 + color-mix xyz none 组

## 背景与两层修复

前轮（[r2320 values serialize + color cc ctx](2026-11-11-dc2-values-serialize-r2320-color-cc-ctx.md)）
记账留下两个观察项：hsl/hwb computed none 组 8 案回归、color-mix xyz none 组
-13。本轮双双定谳并修复——两者是同一机制的两层：

1. **layer 1（part03.js raw 影子保存条件）**：declared 写侧归一剥 %（WPT
   color-valid-hsl 锚 `'hsl(120 80% none)' → 'hsl(120 80 none)'`，声明面正确
   行为），computed 读同一 store 丢分量语法。旧影子条件
   `!/none/i.test(core)` 把「none 保形但语法变形」群排除在影子外。cc8 基线
   这 8 案通过是幻影：旧 serial 无视输入语法一律补 %，恰好把剥 % 的裸 decl
   折回 %-形。修复 = 条件放宽为 `pri.value !== core && /none/i.test(pri.value)`
   （core 含 none 字面也存原值）。
2. **layer 2（part11-computed-color.js mix 插值空间别名折叠）**：影子修复让
   computed 读到 raw 原值后，`color-mix(in xyz, …)` 组反掉 13 案——
   `color()` side 解析折叠别名（`xyz`→`xyz-d65`），mix 解析的插值空间不折叠，
   `_zwCCMixSide` 按 space 全等匹配 own 形失败 → rgb 回退丢 none 旗标 →
   none→0 全权重归一（实产 `0.25`，期望 `0.5` carry / `none` 双 none 传播）。
   cc8 喂归一 core（已 xyz-d65）恰好绕开该缺陷。修复 = mix 解析层同一处
   折叠别名（`_zwCCSpaces[space].alias || space`，与 side 解析先例同规），
   输出侧 `ast.space === 'xyz' ? 'xyz-d65' : …` 死三元式随改清理。

二层根因细节与避免方式沉淀：
[learnings/bugs/2026-11-11-css-color-raw-shadow-accidental-pass.md](../../../docs/learnings/bugs/2026-11/2026-11-11-css-color-raw-shadow-accidental-pass.md)。

## 探针法证链

探针页（三页，用后已删，diag 不入库）：

- zz-probe-hsl-none：gCS 通道 + `__zwComputedColor` 直调对照——set 时
  `STORED:hsl(120 80 none)`、直调收 none-mangled raw，锁定写面影子缺失。
- zz-probe-cc-route：路由确认（`__zwCCComputedInstalled`、needsPlain、
  plainComp 各臂产物）。
- zz-probe-mix-none（layer 2 定谳）：
  `stored=color-mix(in xyz-d65, color(xyz-d65 none none none), color(xyz-d65 0.5 0.6 0.7))`、
  `CCraw=…0.25…`（raw xyz 形，错）、`CCstored=…0.5 0.6 0.7`（归一 core，对）、
  `CCd65=…0.5 0.6 0.7`（手写 xyz-d65，对）——三路对照把缺陷钉死在别名折叠
  缺失而非 none 语义。修复后复跑：`CCraw=CCstored=CCd65=color(xyz-d65 0.5 0.6 0.7)`、
  端到端 gcs 同值。

## 归属勘误

前轮 evidence 将 part03.js raw-shadow hunk（04:27:26）法证归「editing 链他流」。
本轮完整因果链复核：该 hunk 与本批 hsl/hwb 8 案恢复不可分割（不提交即回归），
且本 clone 内无 editing 链活动会话（editing 电池全链 `cd` ZeroWeb-3，见碰撞
记录）；随本 color 批一并提交，前轮「他流认领」记载就此修正。learnings 文档
同批入库。

## 测量事故记账（v7 伪红不作账）

v7 记录轮（zw-corpus-v7 service，04:46:29 起）运行至中途，同 clone 他流 session
执行 clean→clippy→test 门禁舞步，`target/release/` 整目录被清（含 zero-wpt-runner
二进制，`cargo clean --release` 时点 04:47:38，清理日志余 414 个空目录块）：
values corpus 跑至 ~90s 后二进制消失，后续案秒失败（cases=180 pass=1140
fail=2522，健康口径应 ~250+ 案 pass≈2200），color 迭代谓词 `! -x runner` 失败
exit 2。按 rally 竞态规则伪红不入账，results-v7/ 与 zwv7 日志已清除；作废前
部分结果经 transcript 快照留档（logs/zwv7-values-transcript-snapshot.log），
对 v6 逐页对账 common 180 页 0 回归——伪红未掩盖真实回归，与 v8 终账互证。
release 二进制 04:56:13 重建（全量 2m16s，guard scope 包裹）后以 v8 全量重跑。

## v8 终账（同一二进制，含全部在途批）

zw-corpus-v8 service（04:57 起，results-v8/，日志 zwv8-*）：

- **values：256 案 P 2,213 / F 3,380（timeout 7），对 results-v5 净 +673**——
  与 v6 记录轮总数逐字一致（第三轮确定性验证）；color 侧 shim 改动对 values
  语料零漂移。
- **color：63 案 P 10,782 / F 228，对 results-cc8 净 +26，逐页 0 回归**
  （63 页，探针页已删无污染）。
- 恢复面核对：color-computed-color-mix-function **943 = cc8 943**
  （shadow-only 中间态 930 的 13 案全数恢复）；color-computed-hsl 3752P/1F、
  color-computed-hwb 55P/1F 与 cc8 持平（8 案恢复保持，各页余 1F 为 cc8 既有
  calc/sign 失败）；升页 +21 color-computed-relative-color、+5
  alpha-color-computed = currentcolor ctx 批增益，归因干净。
- 三态对照链闭合：cc8（幻影通过）→ shadow-only（mix 13 红）→
  shadow+alias（cc8 等值 + currentcolor +26、零降）。

## 已知缺口（挂账不追）

- C：atan2 混合单位型（angle/number 混槽）specified 折叠不覆盖。
- D：trig 函数时间单位（`sin(1s)` 类）折叠不覆盖。

## 验证

- 单页探针复跑全绿（zz-probe-mix-none 三路一致，见上）。
- clippy --workspace --all-targets -D warnings 绿（隔离 target-clippy，
  `.acceptance/wptfyi-css/logs/zwv6-gates.log` CLIPPY-EXIT=0）。
- make test 全量（systemd cgroup MemoryMax=24G）：147 套件 **3835P/7F**——
  7F 为 main 既有欠账（img/svg settle + ws/net 族），HEAD be6703d58 detached
  worktree 冷编译复跑同 7 测试全红定谳，与本批无关（定谳过程见 master.md
  碰撞记录）；日志 `.acceptance/wptfyi-css/logs/zwv6-maketest3.log`。
- corpus：`CSS_CORPUS_JSON_DIR=.acceptance/wptfyi-css/results-v8`，systemd
  user service `zw-corpus-v8`，test-guard `--per-proc-mem 4 --total-mem 8
  --time-limit 3600`。HEAD 提交态（14e9ae2e2）复录交叉验证（results-v7，
  日志 zwv7-*，systemd scope zeroweb-dev-guard-ZeroWeb-2-v7run）：values
  2213P/3373F+7T、color 10782P/228F——与 v8/v6 逐字一致，确定性四轮闭环。
  注：04:46 伪红轮已清空的 results-v7/zwv7 命名由本复录轮复用，产物即本次
  HEAD 复录。
