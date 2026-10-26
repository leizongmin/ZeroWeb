---
name: zeroweb-guidelines
description: 从 docs/learnings 提炼 ZeroWeb 工程不变式。在本仓编写、审查或重构样式、布局、字体、渲染、IPC、存储及测试代码，排查性能与兼容性问题，或跨 git worktree 共享构建产物时使用。
---

# ZeroWeb 工程方法论

本 skill 是 `docs/learnings/` 踩坑记录的**方法论蒸馏层**：只收录被多篇独立记录印证的跨条目不变式，不重复单条记录的细节。需要某条规则的原始证据/细节时，按子系统到 [INDEX.md](../../../docs/learnings/INDEX.md)（脚本生成，勿手改）检索原文。

## 使用方式与持久性

- 触发后整个编码任务期间持续生效，不因多轮对话或任务切换遗忘；仅当用户明确说「跳过准则」时暂停，恢复编码后自动重新生效。
- 动代码前按改动选择主题，CR 时对照触碰到的条目；先核对适用条件，不把局部经验扩展为无条件禁令。
- **权衡**：这些不变式倾向谨慎而非速度。小改动不必全量过 35 条，自行判断。
- 本 skill 是领域层，叠加在 AGENTS.md 编码准则 / `lei-code-guidelines`（行为层）之上，不复述其行为规则。

| 改动入口 | 必读条目 |
|---|---|
| CSS parser / shorthand | #1–2、#10、#21；触碰热路径加 #23 |
| 字体 / shaping / 缓存 | #3–5、#8；性能改动加 #11 |
| 绘制 / 合成 / 帧复用 | #6–8、#12–14、#17 |
| 布局 / 后处理 pass | #18、#29、#34 |
| IPC / 同步脚本 / 宿主桥 | #12–15、#24–25、#30、#32 |
| 存储 / 并发缓存 | #16、#19、#22 |
| 性能 / 测试工具 | #9–11、#17–20、#23、#27–28、#31、#33、#35；shim 循环加 #26 |

## 与「简单至上」的关系

ZeroWeb 中一切来自网页的输入（HTML/CSS/JS 传值）都是**不可信输入**。#1/#2（解析拒绝）、#21（外部尺寸封顶）属于 AGENTS.md「不可简化」清单中的信任边界校验，#16/#31 属于防数据丢失——不得以「简单至上」「不为不可能场景写错误处理」为由放宽这三类。

## 让步边界

以下场景可临时放宽（合入 main 前必须恢复，放宽结束自动恢复全部）：

- 原型 / spike：#7 像素等价、#11 性能三件套、#23 定向性能门禁、#26 trap 域成本模型、#27 产物目录隔离、#28 执行体证据核验、#29 pass 归因与臂覆盖审计、#33 双侧影响预核对、#34 Path B 信号镜像臂、#35 归因采样协议可先单跑看方向，合入前补全验证
- 紧急热修复且用户确认：#11 可先单跑，事后补同条件配对对照

任何场景下不可放宽：#1、#2、#16、#21、#31（信任边界与数据丢失）。

## 维护

每日 rally cronjob 按 [docs/rally/learning-maintenance.md](../../../docs/rally/learning-maintenance.md) 自动维护。
修改适用边界后对照 [评估样例](evals/evals.json) 检查决策；离线核对不等于真实 Agent 行为评测通过。

## 一、解析与语法边界

### 1. 共享值 parser 总比消费方 grammar 宽

解析 CSS 值时，通用 parser（长度、颜色）接受的范围总是比具体属性的 grammar 宽。跨属性/shorthand 边界后，必须按**消费方 grammar** 再收紧：未知 token 必须整体拒绝而非静默忽略，重复组件按 grammar 判定（`text-decoration-line` 允许重复、颜色不允许），delimiter 数量与空段必须校验，跨 longhand 的 shorthand（`place-*`）要逐侧验证。

### 2. keyword 匹配必须验证 token 边界

裸 `starts_with(keyword)` 会把更长的 ident 拆成「关键字 + 剩余」，非法语法因此被接受。匹配 keyword 后必须确认后续是 token 边界；分段语法局部成功后不得忽略尾部输入。

## 二、字体与文本

### 3. 跨子系统的度量/advance/metadata 必须单一 owner

layout、paint、raster 各自维护同一度量的副本必然分叉。任何改变 used font size/度量的功能，必须同时核对 shaping size、layout fragment width、paint consumed advance、raster size 四处；度量路径与 shaping 路径必须同源（以 hmtx 精确值为准，hinting 取整会在逐字符累计后变成可见误差）。

### 4. fallback face 必须独立解析自己的 descriptor

FontLoader 只解析 primary face 的 feature descriptor 会让 secondary face 用错 descriptor；不同 face 的 descriptor/size-adjust/unicode-range 会产生不同 glyph 序列，必须进入对应 face 的决策与 cache key。

### 5. 缓存键必须等于构造函数的真实输入

键过宽 → 缓存不生效；键缺输入 → 静默错误结果。缓存键必须覆盖全部影响输出的元数据（face descriptor、fallback chain、axes），并按真实消费层次分组（如「字体链+字号 → 字符」两级结构）。

## 三、渲染与双路径

### 6. 双实现路径必须有共享等价断言

CPU/GPU、headless/native、JS-shim/native-DOM 每多一对双路径，就多一个静默分叉面。测试若只验证两条线的交集（CPU 软件渲染正确 + GPU 无头单图元正确），用户实际走的路径（GPU 窗口 + 合成器多进程）仍可能是错的。渲染路径必须有像素等价断言；DOM/API 路径比较可观察状态与事件。视觉和行为分别验证，截图不能替代语义断言。

### 7. 「复用上一帧像素」类优化必须先过像素等价测试

blit、脏矩形、缓存帧优化会改变混合操作序列；半透明混合不满足结合律，「看起来一样」不算数。必须与全量渲染做逐像素等价对比再合入。

### 8. 数据结构变更必须同步全链路转换点

新增字段不会自动穿过显式转换边界（`TextFragment` → `InlineLayoutFragment` → `PaintFragment` → stored path）。加字段时必须审计所有转换路径，只测单一入口会让其他路径静默丢字段。

## 四、性能

### 9. 稳定策略开关禁止进热路径，在边界快照传递

进程/pass 内稳定的开关（环境变量、配置）不允许出现在逐节点、逐 run、逐 glyph 循环里。在公开入口读一次，把值传进递归 helper；或用进程级快照机制。本项目已因此消掉多个 >3% 的 `getenv` 热点。

### 10. 逐 token/节点循环禁止从输入头重扫

循环内任何 `char_indices().nth(pos)`、全树 collect、每 IFC 全文档扫描都会把 O(n) 放大成 O(n²)；昂贵子树扫描应先判断有无消费方，无消费方直接跳过；大对象穿越递归优先借用（`Box`/`Cow`）或 `Arc` 共享，不做深拷贝。

### 11. 性能验证三件套：同条件配对 + 正反序双跑 + 正确性断言

共享主机有频率漂移，整页单次时间证明不了收益。相关路径做同条件 A/B 正反序对照；微基准按需绑核并记录亲和性，正式 bench-gate 必须保持既有基线的 CPU/可用并行度/配置口径。绑核导致 `available_parallelism()` 或配置变化时记 INCONCLUSIVE，不改报告或基线强行比较。纯渲染优化用 PNG SHA-256 或逐像素验证不变；语义修复核对预期变化及邻近回归，非渲染路径用对应行为断言。

依据：[布局开关配对测量](../../../docs/learnings/performance/2026-08/2026-08-18-snapshot-layout-env-at-pass-boundaries.md)、[绑核与基线不可比](../../../docs/learnings/performance/2026-09/2026-09-13-affinity-available-parallelism-budget-gate.md)。

## 五、多进程与 IPC

### 12. 多进程测试必须用新鲜构建的 peer binaries

独立 bin（zero-renderer、zero-compositor）不在测试包依赖树里，`cargo test -p X` 不会重编它们。spawn 陈旧二进制会产出**稳定的**假绿/假红，症状伪装成代码回归。测试前显式 `cargo build` peer 或用 `CARGO_BIN_EXE` 注入；spawn 查找链要完整（env → CARGO_BIN_EXE → exe 上溯 → PATH）且找不到时显式报错，不静默回退。

### 13. 高频生产-低频消费的全量状态消息用 latest-wins 合并

生产者高频发可替代的完整快照（如 `ViewPainted`）而消费者只能处理有限帧时，逐帧处理会堆积过期工作。确认中间状态无独立副作用后用单槽 latest-wins 消除重复转换；按键、点击、增量变更和事务消息不能按该规则丢弃。

### 14. watchdog 必须同时具备资源终止与降级路径

只更新状态、不终止卡死进程、不切换帧来源，会把一次慢帧放大为永久空白。跨进程链路的超时机制必须同时提供「杀掉无响应进程」和「回退 legacy/可用路径」两个出口；空闲后再产帧的合法场景不能误判超时。

### 15. 消息暂存队列不能同时作为优先输入和回退输出

等待特定响应时把无关消息塞回同一队列，会让循环不再阻塞、单核自旋，目标响应即使到达也读不到。无关消息应进独立局部队列，退出等待后再恢复。

## 六、存储

### 16. 持久化状态必须单一写入者，事务必须统一 latest view

原子 rename 只保证单次替换无半文件，不解决多进程内存副本的 lost update——共享目录 ≠ 共享所有权。跨进程持久化必须确立单一 writer，其他方走 IPC。事务系统内 get/add/put/delete/cursor 必须共享同一套 mutation 可见性判断（latest view），各操作重复实现不完整检查必然分叉。

## 七、测试与验证

### 17. 测试断言必须验证真实行为，警惕 stale 数据假绿

断言「快照序号变化」会被空白首帧满足；断言图元非空会放过格式错误；读对比文件前必须确认它是新鲜产物（orphan stale PNG / 缺资源的 webfont 都会造成假绿或假红）。断言要落在真实 DOM 状态、消费方可解析的内容上。

### 18. 布局诊断必须用实际输出验证，不得纯代码推测

code-trace 只是假设，A/B 零效果时继续猜「大概是字体墙」会指向完全错误的函数。reftest 修复前先 `REFTEST_DEBUG=1 make reftest-oracle` 读 ZW 实际输出的 origin/size/color，再用 probe 确认执行路径。

### 19. 进程级全局资源必须跨模块串行化

Rust test harness 在同进程并行跑用例。持有进程级全局资源（FD 计数、全局单例、持久化设置文件、空闲端口）的测试，必须复用同一把跨模块锁或显式串行；多进程 fixture 动态资源要验证完整闭包再导入。

### 20. 跨平台测试必须显式处理资源差异并先门控能力

字体路径、路径分隔符、端口机制因平台而异，只适配单平台 = 其他平台假失败；跑 GPU/性能基准前先探测能力并与 baseline 的 platform_class/cpu_model 匹配，不匹配标 INCONCLUSIVE，不更新共享 baseline。fixture 路径用 `concat!(env!("CARGO_MANIFEST_DIR"), ...)` 编译期拼接，不依赖进程 cwd。

## 八、数值与资源边界

### 21. 外部可控尺寸先封顶，缓冲区算术用饱和运算

JS 传入的 f32/u32 推导出的尺寸/偏移/半径，使用前必须有界；f32→i32 是饱和而非回绕，饱和到 `i32::MAX` 后的加减必然 overflow panic。像素缓冲区大小恒用 usize 域 + `saturating_mul`/`checked_mul`。所有开 socket 的旁路复用统一资源预算，防止 FD 耗尽。

### 22. 跨线程缓存满足实际 trait 与所有权约束，扩展接口保持既有语义

`Send + Sync` 回调的捕获状态须满足实际包装类型的 trait 与生命周期要求；如 `Arc<Mutex<T>>` 要求 `T: Send`，加锁不会把非 Send 对象变为 Send。优先缓存计算后的值；当前 `Document`/`StyleSystem` 的非线程安全成员不应跨线程捕获，类型是否可传递由编译器和具体字段决定，不能只凭出现 `Cell` 就断言非 Send。已满足约束的线程安全复合类型不受“纯值”限制。向既有算法传入扩展数据时核对原解析语义与新增分支，避免不相关结果改变。

依据：[回调缓存 Document 的编译失败](../../../docs/learnings/patterns/2026-08/2026-08-05-callback-closure-send-sync-no-document.md)；以当前类型定义与编译器约束为准。

### 23. 正确性/合规修复必须过定向性能门禁

必要的 spec 合规/安全修复常在热路径上顺手引入 2–16x 回归，全量周期性门禁要一周后才暴露（canvas stroke mask 16.7x、worker terminate 轮询 8.8x、CSP 源列表 2.1x——三例同模式、不同根因）。触碰 parser/热路径的修复合入前跑定向测量（`ZERO_WEB_BENCH_CRATES=<crate> make bench-gate`）；预计算/缓存优化还要复核同对象其他基准（parse/check 互为约束），检查路径省的钱不能在解析路径输光。

### 24. 页面 JS 同步等待的 IPC 消息处理必须绕开主循环 dispatch

renderer/worker 进程内任何「页面 JS 正在等结果」的新 IPC 消息类型（命令、求值、托管类），处理路径必须挂在 reader 线程路由或独立托管线程上，不得只挂主循环 `dispatch_message`——主循环会被同步 automation/脚本执行长期占住，挂在其上的消息永远轮不到，形成跨进程三方互等死锁（页面 JS 等响应 ↔ 主循环等 JS 执行 ↔ 命令排队），且死锁侧无任何日志，症状是端到端测试 20s 超时。SW 求值下放 renderer 即因此死锁；响应/命令同走 reader 线程旁路是不依赖主循环的正确基线。

### 25. 嵌入宿主桥面必须与页面 runtime 契约逐项对齐

WebView 等嵌入宿主暴露给 JS shim 的能力桥（`__zw_*` callback/hook、contentWindow API 暴露）缺失、上下文错位（client id/referrer/相对 URL 基准不按当前 global 解析）或未按每个脚本执行入口武装（独立 `execute_script` 不触发 begin-script hook）时，页面侧对象停在 stale 状态或 API 缺失——症状伪装成时序/超时问题，等待和加超时只会掩盖缺失的发现通道。新桥面必须对照页面 runtime 契约逐项核对存在性、client 上下文与武装时机（SW registration 发现桥、Cache API iframe 上下文、投影轮询重装三例同模式）；排查时断言要读底层 manager 状态而非页面侧缓存投影。

### 26. shim 热循环内的属性读必须按 proxy trap 域计成本，循环不变量提出循环外

js_dom_shim 中凡对象可能是 `_makeProxy`/`_wrapHandle` 产物，「属性读」即 get trap + 可能的 host 回调往返（`__zw_*`，µs 级），不是纯内存访问——代码看起来是纯 JS，成本完全不可见，扫描循环随条目累积二次方放大成文件级 Timeout（R350：Range-mutations 五用例 90s 只跑 28%，单操作微基准却全部「便宜」）。写 shim 扫描循环前必须先问：每条目读什么、读几次、读的对象属于哪个域；恒定键（尤其另一对象的键）必须读一次提出循环外（trap 域这是 50x 级差异，非常规微优化）；前置守卫本身含昂贵读（如 parentNode 上行）时改为「便宜键比对命中后才做守卫验证」。

**验证方法陷阱**：单段微基准可能全部「不慢」（缓存/短路掩盖组合效应），需用等价轻量实现整体替换做 A/B 得到总量差（native 11.46ms vs stub 0.10ms），再逐段把原逻辑加回定位。

### 27. 跨 worktree 禁止共享构建产物目录

cargo 的产物身份（package ID / `-C metadata`）不区分 path 依赖在不同 worktree 的源码差异：共享同一 `CARGO_TARGET_DIR` 时，第二棵树编译同名 crate 会把第一棵树的 rmeta/rlib 原地覆盖，而 fingerprint 仍判「fresh」——后续编译静默链接到错误产物。症状（A/B 测出伪值、下游 crate `unresolved` 符号）从不指向污染源。跨树构建必须用独立 `CARGO_TARGET_DIR`；复用共享 target 时，返回主树先对在另一棵树编译过的 crate `cargo clean -p`，不信任增量结果。

依据：[worktree 共享 target 指纹污染致 A/B 伪值](../../../docs/learnings/platform/2026-08/2026-08-18-worktree-shared-target-dir-fingerprint-collision.md)、[共享 CARGO_TARGET_DIR 静默覆盖主树产物](../../../docs/learnings/bugs/2026-10/2026-10-02-shared-cargo-target-worktree-corruption.md)。

### 28. 执行体代码版本必须以最终链接产物内容为证据

Fresh 输出、构建耗时、`deps/` 下 rlib 的字符串检索、对另一个 bin 的 `-p` 构建，都不是「该二进制包含某段代码」的证据：deps/ 下同一 crate 可并存多个 fingerprint 变体，独立 bin target 不在其他包的构建闭包内（修 shim 后 `-p zero-browser` 不会重编实际执行它的 zero-renderer）。探针、A/B、对账等行为验证前，先对将要执行的产物本体核验：`nm <bin> | grep <新符号>`（base 应 0 命中）或 `grep -a -c <特征串> <bin>`；主进程二进制 grep 不到 shim 字符串属链接器 GC 正常现象，核验对象必须是实际执行该代码的进程二进制。

依据：[cargo Fresh 与多 rlib 变体](../../../docs/learnings/bugs/2026-10/2026-10-07-cargo-fresh-multirlib-not-base-evidence.md)、[shim 修复未进 renderer 二进制](../../../docs/learnings/bugs/2026-10/2026-10-07-renderer-shim-not-in-browser-binary.md)。

### 29. 布局输出是 pass 叠加态：归因按 pass 粒度 kill-switch，加臂先审计门控覆盖与回填覆盖

布局最终几何是 taffy + 十余个后处理 pass 的叠加态，单看最终渲染值无法归因到机制——推断「某 CSS 语义不存在/已被承担」前，先用 kill-switch 环境变量逐个关闭 pass 定位真实承担者（绿案可能是另一条回填/位移 pass 间接撑住的）。给 pass 加臂或按书写模式门控时，必须同时审计：未门控书写模式（vertical）下同一 CSS 语义由谁接管——本项目 vertical 与水平是两套代码，水平臂修的 bug 在 vertical 侧几乎总有对应缺口；以及后续 pass（凡「把 auto 盒长回内容」的回填）是否会覆盖本臂的写入。

依据：[abspos AR 传递与 R1743 回填覆盖](../../../docs/learnings/bugs/2026-10/2026-10-07-abspos-ar-transferred-size-floors.md)、[vertical 写入模式 AR 传递缺口](../../../docs/learnings/bugs/2026-10/2026-10-07-taffy-ar-transfer-no-content-max-vertical.md)。

### 30. shim 异步多事件接口的事件必须 task 承载，禁止用 microtask 链模拟事件间隔

FileReader、XHR 进度族、IDB request 族、MediaElement 族等多事件异步接口，shim 内事件派发一律走 task 粒度（runner 的确定性 setTimeout FIFO stub 是基座），不得用 `Promise.resolve().then` 链逐事件间隔。平台语义里相邻事件是独立 task，promise 续体（含 WPT promise_test 完成回调）在 task 之间全部 drain，EventWatcher 状态机必然推进到位；microtask 链则把「最后一个事件」与 harness 完成回调排进同一条微任务队列竞速，成败取决于测试写法（`wait_for` 位置、中间 await 跳数），失败形态随用例漂移、无法用确定序解释，极难归因。无 host 环境回落 microtask 仅粒度退化、序不变。

依据：[FileReader 事件 task 承载](../../../docs/learnings/platform/2026-10/2026-10-11-wpt-eventwatcher-task-dispatch.md)。

### 31. 不可再生语料资产：工具失败/清理路径禁止删除既有文件，权威源只认 pinned 快照

测试语料（wpt-data 等）的可再生性由 git 跟踪、pinned 快照、manifest 三处台账共同决定，三者皆外的本地案只有一份副本，删除即永久丢失。维护工具（导入/下载/重建目录）的失败与清理路径不得 `rm -f` 或覆盖写「本次产物」——同一目标路径可能正是语料既有案的唯一副本；目标已存在时先备份到资产目录外或拒绝执行，导入后核对分母与逐案结果未漂移。corpus 的权威源是 pinned 快照，live 上游 master 只是补充源：上游持续改名/归并 ref，master 下载的失败率随导入时点推迟必然上升——失败路径被触发是常态而非例外，不能按「罕见分支」降低其安全等级。

依据：[import-wpt ref 404 连带删除既有案](../../../docs/learnings/bugs/2026-10/2026-10-13-import-wpt-master-pinned-mismatch.md)、[空 REF 触发失败路径删文件](../../../docs/learnings/bugs/2026-09/2026-09-12-import-wpt-empty-ref-deletes-test-file.md)、[连锁 fetch-wpt-data 抹掉未入 tag fixture](../../../docs/learnings/bugs/2026-10/2026-10-06-wpt-data-refetch-wipes-untracked-fixtures.md)。

### 32. shim 同一树节点可并存多族稳定 JS 包装：identity 操作先认来源族，写回保持原对象

js_dom_shim 里「同一 DOM 节点 = 同一个 JS 对象」不成立，且多族并存是设计使然：树行走/getElementById/querySelector 返回按 `(sel, handle)` 缓存的 sel-proxy，innerHTML 解析经 named-prop 写入的 `globalThis[id]` 就是活树真实节点对象，文本节点包装逐访问新建——跨族 `===` 恒为 false。涉及节点 identity 的保存/断言路径先确认对象来源族；跨族匹配用「双侧重投影到同一族再比」的 match-only 模式，凡写回端点/容器的值保持调用方路径原对象——重投影对象作写入目标反而丢 identity。`globalThis[id] = node` 型注册的 guard 不能只查 `undefined`：同一文档重复解析是常态，注册值必须自带世代标记并可覆盖刷新，否则上一代死 wrapper 滞留，方法落空、删除断言静默失真。

依据：[js_dom_shim 多族包装 identity 恒败与吸附目标重投影陷阱](../../../docs/learnings/bugs/2026-10/2026-10-14-js-dom-wrapper-family-identity.md)。

### 33. self-source reftest 的判定单位是双侧像素一致，PASS 不证规范符合

corpus 的 test/ref 两页均由本引擎渲染，判定是两页像素差而非单页绝对正确性：双侧同错的自洽 PASS 会长期掩盖共享渲染缺陷（新功能与旧功能在 ref 页混搭时，旧功能缺口以新案形式冒头；probe 时两侧行为差先归因「渲染路径不自洽」，不直接归因 ZW vs chromium）。修共享缺陷前先画两侧预期渲染对照，确认方案下「双侧一致」不变式仍成立——只让 test 侧回退的单侧门控必然失配（ref 侧无该属性、门永不触发，两侧朝相反方向变化），可行方向只有双侧同修到一致或整体回退。

依据：[border-radius 弧带缺失被 corpus 自洽掩盖](../../../docs/learnings/bugs/2026-10/2026-10-18-border-radius-band-sharp-strips.md)、[配对型 reftest 单侧门控无法恢复双侧一致](../../../docs/learnings/bugs/2026-10/2026-10-19-reftest-pair-single-side-gate-futile.md)。

### 34. collect 期 styles 依赖判定必须触发即记录，paint 空 styles 重跑走信号集镜像

layout 与 paint 复用同一 collect 逻辑却是两条通道：存储容器（Path A）持真实 styles，非存储容器 paint 走 Path B 以空 styles 重跑 IFC（R72 裁决，不可改为传真实 styles）。collect/finalize 新增任何 `styles.get(&id)` 依赖分叉，Path B 查表恒空、判定必然塌向固定侧——layout 趟正确、paint 趟劈叉，症状是 `inline_layout` 单测全绿而真实页面绘制错位。正确形态：判定为真**即记录**进 `NodeIdSet`（触发即记录天然覆盖所有门控组合，优于信号侧重扫 DOM），容器收尾存进 `LayoutBox`（store-gate return 之前——Path B 容器恰是非存储容器），分叉改双通道 `styles.get(&id).map(直判).unwrap_or_else(|| 信号集.contains(&id))`，paint 经既有 builder 通道注入同一信号集逐位镜像；嵌套递归 clone 子 ctx 时子层触发 merge-back 回容器 ctx。给该面写断言前先查 store-gate：纯 Ahem 存储路径单测绿不证明 Path B 正确，需另建空 styles + 注入信号的镜像臂断言。

依据：[IFC collect styles 依赖判定的存储信号镜像](../../../docs/learnings/patterns/2026-10/2026-10-23-ifc-collect-styles-dependent-branch-needs-stored-signal.md)——同一失败形态此前已四度独立修复（R4310 vertical_walk_nodes、R4312 block_child_walk_nodes、R5177 color_divergent、R5235 nested_atomic）而规则未成文，本条为第五例成文收敛。

### 35. 归因实验先采样再查档后 bisect，单跑数字不构成归因证据

corpus 单案 mismatch % 与 bench wall-ms 都有运行间/窗间非确定性（HashMap 迭代序/绘制序抖动放大成亚像素 AA 差；CPU 频率窗使微基准跨窗漂移 2–3×），单采样恰好连续命中同档会呈现「确定性」假象，而 bisect 每臂只跑一次，必然把波动钉成无辜代码片——同一轮可先后得出两个相互矛盾的归因结论。归因协议固定三步：先同二进制对目标案连跑 ≥3–4 次，值有波动即判 flake、退出代码实验；再查 goal 文档证据链，漂移案名大概率已归档为已知 flake；仅对采样稳定的案做撤片/stash 实验。「单跑数字」与机制分析矛盾时优先怀疑采样不足，不修补机制解释——门控谓词静态可读，机制矛盾是比数字更硬的证据。

依据：[单案 reftest % 值运行间波动——单采样钉出无辜代码片](../../../docs/learnings/bugs/2026-10/2026-10-25-reftest-single-sample-attribution-flake.md)、[CPU 频率窗效应——bench-gate 跨窗漂移误判归因](../../../docs/learnings/performance/2026-09/2026-09-14-cpu-freq-window-bench-drift.md)。
