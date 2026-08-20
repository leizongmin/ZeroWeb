---
name: "zeroweb-guidelines"
description: "ZeroWeb 专属工程方法论。在本仓编写、审查或重构涉及样式/布局/字体/渲染/IPC/存储/测试的 Rust 代码，或排查性能问题、做兼容性修复时使用——内容是从 docs/learnings/ 144 篇踩坑记录中提炼的跨条目不变式。"
---

# ZeroWeb 工程方法论

本 skill 是 `docs/learnings/` 踩坑记录的**方法论蒸馏层**：只收录被多篇独立记录印证的跨条目不变式，不重复单条记录的细节。需要某条规则的原始证据/细节时，按子系统到 [INDEX.md](../../../docs/learnings/INDEX.md)（脚本生成，勿手改）检索原文。

## 使用方式与持久性

- 触发后整个编码任务期间持续生效，不因多轮对话或任务切换遗忘；仅当用户明确说「跳过准则」时暂停，恢复编码后自动重新生效。
- 动代码前只扫与改动子系统相关的主题（改字体/整形代码 → 主题二；动 IPC → 主题五）；CR 时对照检查触碰到的条目。
- **权衡**：这些不变式倾向谨慎而非速度。小改动不必全量过 24 条，自行判断。
- 本 skill 是领域层，叠加在 AGENTS.md 编码准则 / `lei-code-guidelines`（行为层）之上，不复述其行为规则。

## 与「简单至上」的关系

ZeroWeb 中一切来自网页的输入（HTML/CSS/JS 传值）都是**不可信输入**。#1/#2（解析拒绝）、#21（外部尺寸封顶）属于 AGENTS.md「不可简化」清单中的信任边界校验，#16 属于防数据丢失——不得以「简单至上」「不为不可能场景写错误处理」为由放宽这三类。

## 让步边界

以下场景可临时放宽（合入 main 前必须恢复，放宽结束自动恢复全部）：

- 原型 / spike：#7 像素等价、#11 性能三件套、#23 定向性能门禁可先单跑看方向，合入前补全验证
- 紧急热修复且用户确认：#11 可先单跑，事后补固定 CPU 对照

任何场景下不可放宽：#1、#2、#16、#21（信任边界与数据丢失）。

## 维护

每日 rally cronjob 按 [docs/rally/learning-maintenance.md](../../../docs/rally/learning-maintenance.md) 自动维护。

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

CPU/GPU、headless/native、JS-shim/native-DOM 每多一对双路径，就多一个静默分叉面。测试若只验证两条线的交集（CPU 软件渲染正确 + GPU 无头单图元正确），用户实际走的路径（GPU 窗口 + 合成器多进程）仍可能是错的。必须建立跨路径等价断言，像素级对比是唯一可靠锚点。

### 7. 「复用上一帧像素」类优化必须先过像素等价测试

blit、脏矩形、缓存帧优化会改变混合操作序列；半透明混合不满足结合律，「看起来一样」不算数。必须与全量渲染做逐像素等价对比再合入。

### 8. 数据结构变更必须同步全链路转换点

新增字段不会自动穿过显式转换边界（`TextFragment` → `InlineLayoutFragment` → `PaintFragment` → stored path）。加字段时必须审计所有转换路径，只测单一入口会让其他路径静默丢字段。

## 四、性能

### 9. 稳定策略开关禁止进热路径，在边界快照传递

进程/pass 内稳定的开关（环境变量、配置）不允许出现在逐节点、逐 run、逐 glyph 循环里。在公开入口读一次，把值传进递归 helper；或用进程级快照机制。本项目已因此消掉多个 >3% 的 `getenv` 热点。

### 10. 逐 token/节点循环禁止从输入头重扫

循环内任何 `char_indices().nth(pos)`、全树 collect、每 IFC 全文档扫描都会把 O(n) 放大成 O(n²)；昂贵子树扫描应先判断有无消费方，无消费方直接跳过；大对象穿越递归优先借用（`Box`/`Cow`）或 `Arc` 共享，不做深拷贝。

### 11. 性能验证三件套：固定 CPU 对照 + 正反序双跑 + 渲染不变断言

共享主机有频率漂移，整页时间证明不了收益。性能改动必须用固定 CPU 微基准对照、A/B 正反序双跑排除漂移、PNG SHA-256（或逐像素）验证渲染零变化。三者缺一就容易把噪声当收益。

## 五、多进程与 IPC

### 12. 多进程测试必须用新鲜构建的 peer binaries

独立 bin（zero-renderer、zero-compositor）不在测试包依赖树里，`cargo test -p X` 不会重编它们。spawn 陈旧二进制会产出**稳定的**假绿/假红，症状伪装成代码回归。测试前显式 `cargo build` peer 或用 `CARGO_BIN_EXE` 注入；spawn 查找链要完整（env → CARGO_BIN_EXE → exe 上溯 → PATH）且找不到时显式报错，不静默回退。

### 13. 高频生产-低频消费的全量状态消息用 latest-wins 合并

生产者高频发完整快照（`ViewPainted`、输入帧）而消费者只能处理有限帧时，逐帧处理会堆积过期工作。单槽 latest-wins 邮箱保持最终语义不变并消除 N-1 次重复转换。

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

### 22. 跨线程闭包只能缓存纯值，扩展接口不得改变既有解析结果

`Send + Sync` 回调想跨调用缓存状态，只能存纯值类型（`String`/数字/纯值 map）——`Document`、`StyleSystem` 等含 `Cell`/`Rc`/回调的复合类型天生不是 `Send`。向既有算法传入扩展数据结构时，必须保持原解析路径输出不变，新逻辑在「扩展未改变主结果」守卫下启用。

### 23. 正确性/合规修复必须过定向性能门禁

必要的 spec 合规/安全修复常在热路径上顺手引入 2–16x 回归，全量周期性门禁要一周后才暴露（canvas stroke mask 16.7x、worker terminate 轮询 8.8x、CSP 源列表 2.1x——三例同模式、不同根因）。触碰 parser/热路径的修复合入前跑定向测量（`ZERO_WEB_BENCH_CRATES=<crate> make bench-gate`）；预计算/缓存优化还要复核同对象其他基准（parse/check 互为约束），检查路径省的钱不能在解析路径输光。

### 24. 页面 JS 同步等待的 IPC 消息处理必须绕开主循环 dispatch

renderer/worker 进程内任何「页面 JS 正在等结果」的新 IPC 消息类型（命令、求值、托管类），处理路径必须挂在 reader 线程路由或独立托管线程上，不得只挂主循环 `dispatch_message`——主循环会被同步 automation/脚本执行长期占住，挂在其上的消息永远轮不到，形成跨进程三方互等死锁（页面 JS 等响应 ↔ 主循环等 JS 执行 ↔ 命令排队），且死锁侧无任何日志，症状是端到端测试 20s 超时。SW 求值下放 renderer 即因此死锁；响应/命令同走 reader 线程旁路是不依赖主循环的正确基线。

