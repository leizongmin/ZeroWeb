# M2-S3 切片：暂停/取消/命中测试一致性（DC-3）RFC

状态：Active ｜ 日期：2026-10-21 ｜ 上游：[目标契约](../compositor-animation-continuity.md) DC-3 / [控制面](master.md)
前置：S1（采样链路，commits 72183d732/77d4db07f）与 S2（查询一致性 + 结束事件，[RFC](m2-slice-query-consistency.md)）已落地；flag 默认仍关。

## 0. 执行摘要

DC-3：「动画过程中点击、暂停、取消、切后台再恢复 → 命中当前几何，暂停/取消立即正确，恢复不跳旧世代；真实输入和事件断言」。现状核查（2026-10-21 源码级）：命中测试对 CSS transform 零感知（`hit_test.rs` 全文无 transform 处理，点击走 `webview.hit_test_element` → layout 基线几何）——compositor 采样让视觉动起来后，点击命中与视觉几何分叉是该路径**新引入**的不一致；`animation-play-state: paused` 已解析并 gCS 序列化（`computed_style.rs:2585`）但动画运行时无暂停语义（`animation.rs` 无 paused 状态，动画照跑）；`animationcancel` 仅有 display:none 触发（R347，`cancel_display_none`），动画名移除等取消因与导出域派发去重未验。本切片按 S3a 取消 → S3b 暂停 → S3c 命中 → S3d COMP-03 证据推进；协议零新增消息（暂停经 start 时间重映射、取消复用既有事件管线、命中走进程级 provider 槽）。

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

**单一权威 = engine 动画运行时**。引擎已拥有取消真相（R347 `cancel_display_none`；`drain_just_finished` 同款 drain 模式先例）：

- engine `AnimationRuntime` 增 `cancelled` 记账 + `drain_cancelled()`（镜像 `drain_just_finished`，`FinishedAnimation` 扩展或平行结构携带 name/delay 时刻）；取消成因从 display:none 扩到**活跃动画的 animation-name/timing 从 computed style 消失**（tick 对账：runtime 活跃集 vs 本帧样式，消失即取消；spec-correct：CSS Animations §animationcancel）。导出域约束不变：仅自然终结与取消两类事件，暂停不派发事件。
- renderer 复用 `dispatch_animation_events` 管线派发 animationcancel（elapsedTime=0，镜像 engine R347 语义）；compositor 导出域的声明消失（代际替换）由既有 per-(surface,epoch) 替换 + 回落基线处理（S1c 已就绪，零改动）——事件与采样两条线**不互相派发**，避免 S2 RFC §2.2 预担心的双派发。
- exactly-once：`drain` 语义一次取走即不重发；导航换代 `Self::new` 整体重置（S2b 同款），跨代无 cancel 回放（场景 d）。

### 2.2 暂停面（S3b）

**语义落 engine 运行时，compositor 经 start 重映射零协议变更**：

- engine 运行时给活跃动画增 `paused_at: Option<f64>`：tick 时对账 computed style 的 `animation-play-state`，flip 到 paused 记当前 `current_time` 并冻结进度输出（后续 tick 返回 paused_at 时刻插值）；flip 回 running 以 `start += now - paused_at` 平移重基（进度连续，不跳变）。预存在缺口（flag 关动画照跑）由此一并修复。
- compositor 导出映射：导出声明 `start_time_s`（engine `ExportedAnimationDeclaration`）对暂停动画直接编码平移后的 start（`start = now - delay - phase × duration`，phase = paused_at 冻结相位），协议 `start_monotonic_ns` 消费侧零改动——compositor `sample_declaration` 对该声明采样恒得冻结相位；恢复运行时代际替换为新 start（既有 advance 判变发布路径，零新消息）。fill/backwards 交互按 `sample_declaration` 既有分支，暂停落在 delay 期内时覆盖为 None（回落底层，与 compositor 基线镜像）。
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
| S3a | engine `animation.rs`（cancelled 记账 + drain_cancelled + 对账取消）+ `pipeline/mod.rs`（事件组装 Cancel 臂）；renderer `runtime.rs`（animationcancel 派发接线，复用 `dispatch_animation_events`） | 无 |
| S3b | engine `animation.rs`（paused_at 记账/冻结/重基）+ `animation/compositor_export.rs`（暂停 start 平移导出）；pipeline `animation-play-state` 对账 | 无 |
| S3c | engine `js_dom_bridge.rs` 或新 `animation/hit_provider.rs`（槽）+ `hit_test.rs`（逆映射点测）；webview 接线；renderer `query_overrides.rs`（element_key 视图 + 注册）+ `runtime.rs`；browser `app_input*.rs`（`ZW_INJECT_CLICKS` 注入器）+ observe 脚本扩展 | 无 |
| S3d | fixture `comp03-hit-test-pause-cancel.html`（三动画元素 + 命中/暂停/取消回执外显 + `__zwBusy` 复用）+ `scripts/compositor-animation-dc3-observe.sh` + 分析脚本扩展（相位冻结段识别、命中回执格）+ 证据目录 | 无 |

**回滚切点**：S3a/S3b/S3c 各自成批、flag 门控（S3b engine 暂停语义为 flag 无关的 spec-correct 修复，独立成批可回滚）；S3d 证据批。翻默认仍属后续独立决策（perf-gate 前置不变）。

## 3. 假设与待定

| 项 | 性质 | 说明 |
|---|---|---|
| 暂停经 start 重映射与 `sample_declaration` 既有分支兼容（delay 期暂停、fill 交互） | Assumption | 实现时以单测钉住：delay 期暂停 → 覆盖 None；forwards fill 已结束后置 paused → 不复活（`finished_at_t0` 语义不受扰动） |
| 暂停态导出声明与 S2b 终结判定的交互 | Assumption | 暂停不产生 finished；暂停跨越原定结束时刻后恢复，结束时刻随重基平移（spec：暂停不计入活跃时长），`drain_finished_events` 以重基后声明判定——实现时核对 `sample_declaration(…).finished` 与重基一致性 |
| engine tick 对账取消的触发时机（样式变更帧 vs 下一 tick） | Assumption | 取消即时性上限 = 一次样式对账 + 内容帧提交 + 声明消息往返；场景 (c) 以「帧序列收敛到基线 + 事件恰一次」验收，不逐帧断言延迟 |
| OS 级遮挡/最小化的「切后台」证据 | Open（环境受限） | WSLg 无窗口自动化工具；如后续环境具备 xdotool/wmctrl 再补遮挡证据，当前以忙窗暂停-恢复承载恢复语义 |
| `getBoundingClientRect`/`RectBridge` 几何查询面的动画一致性 | Open | 本切片只覆盖点击命中；几何查询面扩展按需另立小切片（S2 RFC §2.1 gCS 覆盖不含几何 API，同理） |
| WPT 钉住面 | 待办（S3a/S3b 实现批） | CSS Animations 既有 WPT（animationcancel/play-state 相关）回归必须同批核对，行为修复不得翻绿为红 |

## 4. 与后续切片的契约预留

- 协议本切片零新增消息/字段；暂停经 `start_monotonic_ns` 重映射承载。若 M4 GPU 采样路径需要显式 paused 状态位（避免每代重发），预留 `CompositorAnimationDeclaration.timing` 的可选扩展点——仅在出现真实消费者时追加（S2 RFC §4 `finished` 预留位同款纪律）。
- 命中 provider 槽签名（element_key → 折叠仿射）是 engine↔renderer 内部边界；若未来静态 transform 命中一致性立项（缺口表），复用同槽扩展键域即可，不新增第二槽。

