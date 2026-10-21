# M2-S3 切片：暂停/取消/命中测试一致性（DC-3）RFC

状态：Active ｜ 日期：2026-10-21 ｜ 上游：[目标契约](../compositor-animation-continuity.md) DC-3 / [控制面](master.md)
前置：S1（采样链路，commits 72183d732/77d4db07f）与 S2（查询一致性 + 结束事件，[RFC](m2-slice-query-consistency.md)）已落地；flag 默认仍关。

## 0. 执行摘要

DC-3：「动画过程中点击、暂停、取消、切后台再恢复 → 命中当前几何，暂停/取消立即正确，恢复不跳旧世代；真实输入和事件断言」。现状核查（2026-10-21 源码级）：命中测试对 CSS transform 零感知（`hit_test.rs` 全文无 transform 处理，点击走 `webview.hit_test_element` → layout 基线几何）——compositor 采样让视觉动起来后，点击命中与视觉几何分叉是该路径**新引入**的不一致；`animation-play-state: paused` 已解析并 gCS 序列化（`computed_style.rs:2585`）但动画运行时无暂停语义（`animation.rs` 无 paused 状态）；`animationcancel` 仅有 display:none 触发（R347，`cancel_display_none`），且真实 renderer 动画时钟冻结（`webview.render()` → `render_html` 不调 `tick_animation_styles`），engine 运行时对导出域动画的完成与取消均不可见——S2b 因此用 renderer 侧惰性判定，取消同构。本切片按 S3a 取消 → S3b 暂停 → S3c 命中 → S3d COMP-03 证据推进；协议仅增一个 serde-default 字段（暂停冻结相位），取消复用既有事件管线，命中走进程级 provider 槽。设计修正记录：初稿的「engine 单一权威取消」「start 重映射零协议暂停」经代码推演否决（见 §2.1/§2.2 修正说明），以当前为准。

## 1. 背景与验收场景

### 现状事实（源码核查，2026-10-21）

| 事实 | 位置 | 影响 |
|---|---|---|
| 点击命中 = layout 基线几何，无 transform 参与 | `crates/engine/src/hit_test.rs`（rect 全部来自 LayoutBox snapshot）、`crates/webview/src/webview.rs:2265`、`apps/renderer/src/runtime.rs:3441`（click → `webview.hit_test_element`） | compositor 采样后视觉几何 ≠ 命中几何（transform 通道；opacity 不影响命中） |
| `animation-play-state` 解析 + gCS 序列化，运行时无消费 | `crates/css-parser/src/values/parse_basic.rs:1273`、`crates/engine/src/js_dom_bridge/computed_style.rs:2585`；`crates/engine/src/animation.rs` 无 paused | 暂停语义缺失为**预存在缺口**（flag 关同样动画照跑）；compositor 域无暂停映射 |
| `animationcancel` 仅 display:none 路径（R347） | `crates/engine/src/animation.rs:1088`（`cancel_display_none`）、`pipeline/mod.rs:42` | 动画名移除/样式替换的取消不派发；S2b 只做自然终结，导出域取消未验 |
| 采样按单调钟推进、代际替换即时生效 | S1c（声明 per-(surface,epoch) 单调替换、结束后回落基线）+ S2c（Get 前台 key 漂移修复） | 取消/暂停的 compositor 侧存储与回落机制已就绪，缺的是声明源头的正确性 |
| engine 不依赖 zero-protocol | `crates/engine/Cargo.toml` 无 zero-protocol | 命中修正钩子在 engine 侧必须是 protocol-free 类型，renderer 桥接（S2a gCS provider 同款分层） |
| 窗口模式无 OS 级输入注入工具 | 环境无 xdotool/wmctrl；browser 无合成输入 env | 「真实输入」断言需 browser 侧 env 门控注入器（走既有 MouseEvent 管线） |

### 验收场景（S3d / COMP-03，fixture `comp03-hit-test-pause-cancel.html` + dc3 observe 扩展）

flag 开真实窗口（autostart 自驱动 + env 门控点击注入）：

- **(a) 命中当前几何**：旋转/平移中的元素在「仅采样变换下才与指针重合」的位置注入 click → 事件目标为该元素（fixture 外显回执 + renderer 日志双证）；基线几何位置（视觉已离开）同时注入 click → 不命中该元素。
- **(b) 暂停立即正确**：脚本置 `animation-play-state: paused` → 像素相位冻结在暂停时刻（frame_dump 相位序列停走），查询（gCS 覆盖）与像素同相；恢复 running → 从冻结相位继续（相位不跳变不重启）。
- **(c) 取消立即正确且恰一次**：脚本移除动画名 / display:none → animationcancel 恰一次（renderer 逐事件日志），compositor 回落基线像素（帧序列收敛到基线），不重发不漏发；S2b animationend 语义不受扰动。
- **(d) 恢复不跳旧世代**：忙窗（`__zwBusy`）跨越暂停/取消时点后，恢复的像素/查询/事件均为新状态，无旧代声明回放；导航换代不派发旧代 cancel（S2b epoch 重置语义延伸核对）。

范围裁剪（明示）：静态（非动画）CSS transform 的命中不一致为预存在缺口，本切片只修**导出域动画通道**（compositor 路径新引入的分叉），静态情形记缺口表；`getBoundingClientRect` 等几何查询面的动画一致性不在本切片（记待定）；OS 级窗口最小化/遮挡自动化在本环境不可行（无 xdotool/wmctrl），「切后台」子句以忙窗暂停-恢复为载体验证恢复语义，OS 遮挡记局限（§3）。

## 2. 已定设计（Decision）

### 2.1 取消面（S3a）

**单一权威 = renderer 导出代际差分（S2b 惰性判定同构）**。实现推演修正（2026-10-21，源码核查）：真实 renderer 每帧走 `webview.render()` → `pipeline.render_html`（`webview.rs:1926/1932`），该路径**不调** `tick_animation_styles`——引擎动画时钟在真实 renderer 冻结（M1 BASELINE_FROZEN 的根因），engine 运行时既看不到动画完成（故 S2b 用 renderer 侧判定）也看不到取消（R347 `cancel_display_none` 仅活在 pump/`render_html_animated` 路径）。「engine drain_cancelled 权威」方案在目标场景失效，修正为：

- renderer `AnimationExportState` 增代际差分：publish 时对比 `engine_decls` 上代与本代 animation_id 集合，**同 epoch 内**「上代有、本代无、且不在 `finished_reported`」→ 记取消（name/selector 取上代 `engine_decls` 条目），`drain_finished_events` 同点出队（改扩为 `drain_animation_events`：自然终结 + 取消两类），`dispatch_animation_events` 既有管线派发，elapsedTime = duration × iterations（R347 既有口径）。`finished_reported` 对冲：已终结动画的声明消失（fill 消费后被导出规则剔除）不得二次派发 cancel。
- 导出域外沿（明示简化）：声明消失即取消的判定覆盖 name 移除、display:none、导出规则拒绝（如关键帧中途改写为 matrix）三类；前两类 spec-correct，第三类 CSS 动画仍在（主线程冻结不可见）——记 §3 已知简化，不阻塞。
- engine R347 保持现状（pump/headless 路径），真实 renderer 无双派发（时钟冻结）；runner 泵路径不导出 compositor（S1b 边界），两路径无交集。
- exactly-once：差分每代际只评估一次（已评估的消失 id 记入 `cancelled_reported: HashSet<u64>`，镜像 `finished_reported`）；导航换代 `Self::new` 整体重置（S2b 同款），跨代无 cancel 回放（场景 d）。
- compositor 侧声明消失由既有 per-(surface,epoch) 替换 + 回落基线处理（S1c 已就绪，零改动）。

### 2.2 暂停面（S3b）

**冻结相位经协议显式承载（start 重映射方案否决）**。实现推演修正：compositor 采样按墙钟（`sample_declaration(now)`），start 重映射冻结一次后相位仍随墙钟推进，唯一免 churn 的冻结语义是把冻结相位写进声明——

- protocol `CompositorAnimationDeclaration` 增 `frozen_phase_s: Option<f64>`（`#[serde(default)]`，FR-11 末尾字段纪律，Android 通配安全）；`sample_declaration` 增分支：`frozen_phase_s = Some(p)` 时活跃相位恒为 p（fill 规则沿用既有分支：delay 期暂停 → backwards fill 或 None，结束后暂停 → finished + fill 语义）。三个消费者（compositor `animation_overrides::apply`、renderer 查询覆盖、S2b 终结判定）同函数自动一致；暂停动画 `finished` 恒 false，不触发 S2b animationend。
- engine `ExportedAnimationDeclaration` 增 `paused: bool`（computed style `animation-play-state` 对账产物，分类时已知，引擎时钟冻结不影响该布尔）；renderer 发布转换处维护状态机：paused false→true 翻转时以 `sample_declaration` 求当刻相位 p，发 `frozen_phase_s = Some(p)`（声明字节稳定 → sent 判等零 churn）；true→false 恢复时发 `frozen_phase_s = None` 且 `start_monotonic_ns` 重基为 `now − delay − p×duration`（进度从 p 继续，暂停时长不计入，spec CSS Animations §play-state）；未暂停声明该字段恒 None（序列化不变）。
- engine 运行时暂停语义（pump/headless 路径 + WPT 钉住面）：样式段（4b 对账）对 `animation-play-state: paused` 的活跃动画记 `paused_at`，tick 输出冻结于暂停时刻插值；恢复 `start_time += now − paused_at`。预存在缺口（flag 关/pump 路径 play-state 无效）由此一并修复。
- 查询一致免费获得：S2a `QueryOverrideTable` 消费同一条声明，gCS 覆盖与像素天然同相（场景 b 前半）。

### 2.3 命中面（S3c）

**engine 进程级 provider 槽 + renderer 同源采样（S2a gCS provider 同款分层）**：

- engine 新槽：`set_animation_transform_provider(Option<Arc<dyn Fn(u64 element_key) -> Option<[f64; 6]>>> + Send + Sync>)`——element_key 与导出声明 `element_key` 同源（renderer doc 内可比，规避 S2a 跨文档选择器形态问题）；返回折叠 2D 仿射（列主序 6 元组），engine 不引 protocol 类型。
- renderer 注册：发布点（`publish_declarations` 同点）增 element_key → transform 声明通道视图；provider = `sample_declaration` + protocol `fold_affine`（与 compositor 像素折叠同一函数，FR-6 单一实现纪律）。仅 transform 通道注册（opacity 不影响命中几何）。
- hit_test 应用点：`hit_test.rs` 候选节点矩形包含测试前，若节点 element_key 命中 provider → 以逆矩阵映射查询点再测基线 rect（点变换 O(1)，不改 rect 缓存结构；矩阵奇异即跳过 provider 回落基线，单测钉住）；命中选择器/文本/图片三路同源受益。flag 关未注册 provider → 零行为变化（FR-9 同款）。静态 transform 不经此路径（无 provider 键），维持现状并记缺口。
- 「真实输入」注入器：browser env 门控 `ZW_INJECT_CLICKS="x,y@t[;…]"`（导航完成后相对秒），到点走既有 winit 输入入口组装 `MouseEventParams::Click` 下发 renderer（`app_input` 既有路径，非 renderer 直派）——复用全链路命中管线；env 缺省零行为。观察脚本 dc3-observe 增传该 env。

### 2.4 触面清单

| 步 | 触点 | 新增 IPC |
|---|---|---|
| S3a | renderer `runtime.rs`（`AnimationExportState` 代际差分 + `cancelled_reported` + `drain_finished_events` 扩为双类事件）；单测（恰一次/换代不回放/finished 不二次 cancel） | 无 |
| S3b | protocol `animation.rs`（`frozen_phase_s` 字段 + `sample_declaration` 分支）；engine `animation/compositor_export.rs`（`paused` 布尔）+ `animation.rs`/pipeline 4b（运行时 paused_at 记账/冻结/重基，pump 路径）；renderer `runtime.rs`（paused 状态机 + 发布转换） | 无新消息；`CompositorAnimationDeclaration` 增 serde-default 字段（FR-11 追加纪律） |
| S3c | engine `js_dom_bridge.rs` 或新 `animation/hit_provider.rs`（槽）+ `hit_test.rs`（逆映射点测）；webview 接线；renderer `query_overrides.rs`（element_key 视图 + 注册）+ `runtime.rs`；browser `app_input*.rs`（`ZW_INJECT_CLICKS` 注入器）+ observe 脚本扩展 | 无 |
| S3d | fixture `comp03-hit-test-pause-cancel.html`（三动画元素 + 命中/暂停/取消回执外显 + `__zwBusy` 复用）+ `scripts/compositor-animation-dc3-observe.sh` + 分析脚本扩展（相位冻结段识别、命中回执格）+ 证据目录 | 无 |

**回滚切点**：S3a/S3b/S3c 各自成批、flag 门控（S3b protocol 字段 serde-default 向后兼容，engine 运行时暂停语义为 flag 无关的 spec-correct 修复独立成批可回滚）；S3d 证据批。翻默认仍属后续独立决策（perf-gate 前置不变）。

## 3. 假设与待定

| 项 | 性质 | 说明 |
|---|---|---|
| 导出域外沿简化：声明消失即取消（含导出规则拒绝类） | Assumption（已知简化） | 关键帧中途改写为 matrix 等导出拒绝会让仍存续的 CSS 动画（主线程冻结不可见）收到 animationcancel，spec 上该动画未取消——真实页面触发面极窄，如实记录；后续如需精确可在 engine_decl 条目携带取消成因再分型 |
| `frozen_phase_s` 与 `sample_declaration` 既有分支兼容（delay 期暂停、fill 交互） | Assumption | 实现时以单测钉住：delay 期暂停 → backwards fill 或 None；已结束后置 paused → 不复活（finished 语义不受扰动）；alternate 方向的相位换算走既有进度函数 |
| 暂停跨越原定结束时刻后恢复的终结判定 | Assumption | 恢复重基 start 后 `sample_declaration(…).finished` 以新 start 判定——结束时刻随暂停时长平移（spec：暂停不计入活跃时长）；S2b `drain_finished_events` 无感知（同函数） |
| 取消即时性上限 | Assumption | = 一次样式对账 + 内容帧提交 + 声明消息往返；场景 (c) 以「帧序列收敛到基线 + 事件恰一次」验收，不逐帧断言延迟 |
| OS 级遮挡/最小化的「切后台」证据 | Open（环境受限） | WSLg 无窗口自动化工具；如后续环境具备 xdotool/wmctrl 再补遮挡证据，当前以忙窗暂停-恢复承载恢复语义 |
| `getBoundingClientRect`/`RectBridge` 几何查询面的动画一致性 | Open | 本切片只覆盖点击命中；几何查询面扩展按需另立小切片（S2 RFC §2.1 gCS 覆盖不含几何 API，同理） |
| WPT 钉住面 | 待办（S3a/S3b 实现批） | CSS Animations 既有 WPT（animationcancel/play-state 相关）回归必须同批核对，行为修复不得翻绿为红 |

## 4. 与后续切片的契约预留

- 协议本切片零新增消息；`CompositorAnimationDeclaration` 增 serde-default 字段 `frozen_phase_s`（暂停冻结相位，§2.2）。若 M4 GPU 采样路径需要更细的暂停控制（如引擎侧暂停时刻戳），预留 `timing` 的可选扩展点——仅在出现真实消费者时追加（S2 RFC §4 `finished` 预留位同款纪律）。
- 命中 provider 槽签名（element_key → 折叠仿射）是 engine↔renderer 内部边界；若未来静态 transform 命中一致性立项（缺口表），复用同槽扩展键域即可，不新增第二槽。

