# M2-S12 autofocus load 期处理 + 结算 autofocus 委托合格性门（2026-10-20）

S11 挂账「autofocus load 期 14 NotRun」收口——两个独立根因同轮（runner 初始
autofocus 缺处理 + shim 结算委托首匹配不筛合格性），`navigation-api/focus-reset/
autofocus.html` 7 断言 × 2 变体全数翻绿，全量 NotRun 清零。

## 根因一：runner 初始 autofocus 处理（testharness.rs）

S11 定性：`promise_setup` 首断言「Non-navigation API autofocus was processed」
失败——`<button autofocus>` 载入期从未获焦，14 子测试全部 NotRun（setup 抛
assert 则同 case 后续 promise_test 不执行）。引擎（产品面与 runner 面）均无
「flush autofocus candidates」处理：crates/dom FocusManager 只有 Tab 扫描
（S11 复用），无载入期初始聚焦管线。

**修**：testharness runner 路径在 `run_page_scripts_strict()` 之前注入一次性
初始聚焦脚本（HTML §6.5.5 the autofocus attribute——flush autofocus candidates
的「ready for post-load tasks」相位：文档序首个合格 candidate 获焦）。runner
模型里页面脚本同步执行 + rAF 同步 stub，setup 期 `rAF×2` await 在脚本任务内
即解——该相位必须落在**全部页面脚本之前**才可观察（真浏览器 post-parse 相位
的页可观察等价位）。合格性门：connected（querySelectorAll 语义自带）+
非 disabled + 非 hidden input；首个合格即 break（flush 语义：余者作废）。

仅 runner 路径执行，零产品面；产品宿主解析路径的 autofocus 初始处理未接线，
`FIXME(产品面)` 已标（随真实 focus 域另行立项）。

## 根因二：结算 autofocus 委托合格性门（part02.js `_navMaybeResetFocus`）

S4E 的结算焦点重置委托用 `querySelector('[autofocus]')` 首匹配即聚焦，不筛
合格性。flush 语义是**文档序首个合格 candidate**：delayed 变体两案要求
「disabled 首候选跳过、次位递补」（Two elements... first gets disabled）与
「唯一候选 disabled → 落 body」（disabled before finished）。**修**：改
`querySelectorAll('[autofocus]')` 文档序迭代——disabled 跳过续查、hidden
input 跳过、`focus` 可调用才取；全灭维持既有 body 落点。属性移除面免改：
`autofocusTarget.autofocus = false`IDL setter 落 attr 移除，选择器不再匹配
（attribute removed before finished 案）；committed→finished 之间插入的
[autofocus] 元素在 finish 时点被拾取（introduced between 案）；finished 后
插入无 flush 面维持 body（introduced after 案）。

## 波及面封口

- runner 注入仅 testharness 路径（reftest 渲染管线不经过）；语料树 grep
  `autofocus`：testharness 路径仅 navigation-api 一族 3 案，css 域 12 处全是
  reftest/crashtest/注释引用（不进 testharness 执行体）。
- shim 结算门只在「导航期间无焦点变更且 focusReset 非 manual」分支触达，
  每 navigation finish 至多一次全量 `[autofocus]` 扫描（原 querySelector
  同阶成本，非逐帧/逐节点热路径）。

## 验证

- 隔离：`make testharness-navigation FILTER=autofocus` 14/14 全 Pass
  （delayed-checkpoint + same-checkpoint 各 7）。
- 全量：`make testharness-navigation`（TIME_LIMIT=2700）**437P/34F/24T/0NR**，
  全量 423→**437**/476（**91.8%**）；对 [S11 账本](2026-10-20-m2-s11-focus-reset-tab.txt)
  per-subtest 语义 diff **恰 14 行**（autofocus 双变体 NotRun→Pass），
  Fail/Timeout 逐行恒等零回归（34F + 24T 集合与 S11 完全一致）。make exit 1
  = 既有慢性面签名（sandbox 双案 `open()`→null，2026-10-20 组成态巡检同记）。
- 域计数：navigation-api 252→**266**/280（**95.0%**）；域内残差 11F + 3T
  逐行与 S11 恒等。NotRun 全量清零。
- 全量批为忙窗跑（另两 clone 并行 rally 流收尾期），per-subtest diff 仍恰
  14 行无 Timeout 抖动——busy-window flake 未现。
- reftest / product-smoke 未跑：变更面（runner 注入 + 结算委托门）零布局/
  绘制触达，reftest 页面无导航语义路径，S11 同理由维持（708 比较 0 不一致
  账本仍新）。

- `cargo fmt --all -- --check` 干净；`cargo clippy --workspace --all-targets
  -D warnings` 零 warning；`make test`（DISPLAY=:20）完整跑完 **76 suites /
  22,200P / 2F / 199I**——唯二 F = S11 同记跨流陈旧 GPU 断言
  （`gpu_full_scene_filter_grayscale/saturate`，R5194 Rec601→709 断言未随更新，
  rendering 域待修，本流不修）。首跑 make test 曾 1F 忙窗 compositor SIGSEGV
  （service_worker committed_navigation 测试，solo 复跑恒过，与本流 diff 无涉
  ——browser/compositor 不链接 runner 与 shim JS）后全量重跑完成。

原始输出：[2026-10-20-m2-s12-autofocus-load.txt](2026-10-20-m2-s12-autofocus-load.txt)。
