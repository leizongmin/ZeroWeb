# 后台页面与重复导航资源生命周期

**版本**：v2.1 / 2026-10-23
**状态**：Active（M1 基线 + M2 三步全 DONE；DC-1 页面+worker 面钳制 DONE + 后台 timer 策略面 DONE——隐藏页 arm 定时器 1s 下限（三族单口钳制）、恢复自动回原延迟；M3 DC-2 呈现链双断点修复 + rAF 饥饿修复 + 可见性消息切片 + offscreen 声明级门控 DONE——动画页 2→512+ 帧/30s、`__zw_raf` 2→1575 次/30s、`SetVisibility` 四层链 + rAF 泵 hidden 门控、offscreen 空转消除（compositor 0.498→0.003 cores）且 onscreen 对照保持、静态零回归；M4 GUI 生产腿首轮 DONE——DC-1 真实宿主端到端（前台 0.052 → hidden 0.004 → 恢复 0.046 cores）、hidden tab 呈现链空转缺口以 `page_hidden` 声明维度收口（同段差分 compositor -38%/browser -50%，恢复回带））
**父目标**：[zero-web](zero-web.md) / **控制面**：[master](background-resource-lifecycle/master.md)

## 0. 执行摘要与 Mission

验证后台 timer、视口外动画、重复导航、停滞资源和页面关闭是否释放旧文档并减少无效工作。按同环境基线建立资源门禁，不先编造 CPU/RSS 达标数字。

依据：2026-10-20 用户要求将 [Ladybird 建议](../research/research-ladybird-progress-2026-10-20.md)转为后续 goal。

## 基线与 Support Envelope

静态基线：`apps/browser/src/process_backend.rs` 存在 `renderer_poll_order_prioritizes_active_tab_and_throttles_background_tabs`；renderer 运行入口为 `apps/renderer/src/runtime.rs`，页面共享入口在 `crates/page-runtime/src/lib.rs`。轮询节流不证明 timer/动画/合成器已停无效工作。

生产资源基线：M1 已建立（2026-10-22，12 腿，headless CDP + GUI X :20 双口径），见 [M1 基线](background-resource-lifecycle/evidence/2026-10-22-m1-baseline/ANALYSIS.md)。要点：嵌套零延迟 timer ~20 万 iter/s 无钳制；前后台 tab CPU 无差异（后台无策略）；CSS 动画不驱动持续呈现（视口内外一致，呈现链未激活）；重复导航 renderer RSS +224MB 且 stall fetch 连接随导航滞留；tab 开关 renderer 进程复用、内存不回落。M2-S1（2026-10-22）修复挂起 fetch 无取消路径——调度器 `cancel_by_url` + renderer 页面 fetch 登记 + browser `cancel_tab` 扩展，生产复测连接随导航关闭（61 次 create/cancel 零失手），见 [M2-S1](background-resource-lifecycle/evidence/2026-10-22-m2-s1-fetch-cancel/ANALYSIS.md)。M2-S2（2026-10-22）双腿 120 次导航判定 RSS 为爬坡+有界平台（非无界增长，M1 线性读数为窗口假象），见 [M2-S2](background-resource-lifecycle/evidence/2026-10-22-m2-s2-rss-attribution/ANALYSIS.md)。M2-S3（2026-10-22）补 DC-3 文档残留计数（`TimerBridge::pending_stats` + 复位诊断日志行）并生产验证：fetch 15/15 复位放弃、命令零残留、timer entry 有界滞留（24.8d expiry 上界）且回调已释放；BG-01 高频 interval timer 120s 腿同为爬坡+有界平台——M1 三个增长源候选全部收口，见 [M2-S3](background-resource-lifecycle/evidence/2026-10-22-m2-s3-timer-form/ANALYSIS.md)。DC-1 轮（2026-10-22）修复页面 timer 嵌套 4ms 钳制——shim 任务链 nesting（创建闭包携带 level、fire 绝对赋值 level+1），单测钉住第 7 代边界，生产复测 ~237 iter/s（修前 ~20 万，~840× 降）、renderer CPU 1.46→0.05 cores，见 [DC-1 钳制](background-resource-lifecycle/evidence/2026-10-22-dc1-nested-clamp/ANALYSIS.md)；随后 worker 面补齐——WORKER_BOOTSTRAP_JS 同款任务链 nesting + interval 重臂逐代钳（V8/QuickJS 共用），4 单测含真线程端到端墙钟下限。M3 DC-2 轮（2026-10-23）定性并修复「CSS 动画不驱动持续呈现」的两个独立断点：(a) browser keep-alive 自举死锁——`animations_active` 唯一产生源是 Get 响应采样而常态 Get 又以该标志为前置，改为每新提交 frame 代探测一次；(b) 预算渲染路径从不导出动画声明（M2-S1b 记档缺口，「稳定后 render() 全量帧接手」在生产无调用方），Style/Paint 步镜像 render_html 同源导出。修复后动画页 30s 帧数 2→512+（采集封顶）、settle 腿 187 dumps 后 true→false 翻转冻结、静态页 3 帧零回归；代价面首次可测（browser present-blit 2.7–3.3 cores、present 缓存 RSS ~540MB、compositor 重光栅 0.4–0.5 cores、offscreen 声明级空转 0.5 cores），见 [DC-2 呈现链](background-resource-lifecycle/evidence/2026-10-22-m3-dc2-present-chain/ANALYSIS.md)。rAF 帧驱动饥饿（tick 挂帧发布、compositor 采样不产生 renderer 发布）随后同日修复——shim 在 rAF 注册/取消/tick 末经 `__zw_raf_interest` 回报挂起数（js_worker 无条件注册回调，OFF 模式零开销），renderer 主循环节拍泵按挂起数提交裸 `__zw_raf_tick`（镜像媒体泵门控先例；每拍全量 SetDomSnapshot 重形态实测 6.7 CPU-s/30s 被否决），回调 mutation 走既有 drain 通道；纯 rAF 循环 `__zw_raf` 2→1575 次/30s、泵代价 0.035 cores、静态零回归，见 [rAF 泵](background-resource-lifecycle/evidence/2026-10-23-m3-dc2-raf-pump/ANALYSIS.md)。可见性消息切片同日交付——`SetVisibility` IPC（protocol 枚举末尾 append）+ shim 四 getter 改读宿主全局 `__ZW_HIDDEN`（原硬编码 visible）+ `__zw_dispatch_visibilitychange` 派发 + browser poll 前后台 tab 差分单播 + renderer `page_hidden` 宿主态与跨导航重注入，rAF 泵加 hidden 门控（后台 tab 停泵、恢复可见自动续）；四层单测钉住（roundtrip id36 / shim 派发契约 / 差分出站序 / renderer 状态翻转），headless 腿零回归（anim 与上轮同带、static 同带）且 probe 新增 `visibility_state` 生产采样；GUI tab 切换端到端无 CDP 自动化驱动归 M4 GUI 腿，见 [可见性消息](background-resource-lifecycle/evidence/2026-10-23-m3-dc2-visibility/ANALYSIS.md)。DC-1 后台 timer 策略面（2026-10-23）随后交付——页面隐藏（`__ZW_HIDDEN`，SetVisibility 链注入）时经页面 timer API arm 的定时器 delay 一律抬到 1s 下限：钳制落 shim `_zwClampDelay` 单口（setTimeout/setInterval re-arm/requestIdleCallback 三族，与 nesting 4ms 钳制叠加取 max），IDB 事务推进等 shim 内部任务派发是 event-loop task 不受钳制（迁 `_zwInternalTask` 内部通道，part02 共 13 处）；re-arm 时点判定、已入队 entry 不重排，恢复可见后新 arm 自动回原延迟无需显式恢复；shim 单测 + 真链集成测 + 浏览器多进程 owner 测三层钉住（多进程测抓出首轮桥级钳制误伤 IDB 的设计缺陷并驱动返工）；worker 面、intensive throttling、GUI 生产 CPU 对照待后续，见 [后台 timer 策略面](background-resource-lifecycle/evidence/2026-10-23-dc1-background-timer-policy/ANALYSIS.md)。DC-2 offscreen 声明级门控（2026-10-23）同日收口 M3 主链——renderer `send_compositor_animation_state` 发送单点按本帧存活作用域过滤声明（`visible_declarations`，advance 输入侧；视口剔除后无采样面的动画不进 compositor 状态），全剔即空集 → `animations_active` 不置位 → 采样重光栅与 keep-alive 呈现链不对不可见动画空转；恢复经 advance 集合差分自动闭合（作用域重现声明重现），engine 缓存/增量臂零改动（engine 侧导出期门控因断恢复链否决）；生产双腿差分钉住——offscreen compositor 0.498→0.003 cores、browser present 3.301→0.006（空转消除），onscreen 对照 compositor 0.361/browser 1.876（同带保持，合法工作不误停），帧泵与采样面解耦（两腿帧 dump 同 512 封顶、offscreen 帧为 fills=1 空帧）；offscreen 期间 gCS 覆盖缺失、终结事件丢失、动画时间轴重启为已知边界，见 [offscreen 门控](background-resource-lifecycle/evidence/2026-10-23-m3-dc2-offscreen-declaration-gate/ANALYSIS.md)。M4 GUI 生产腿首轮（2026-10-23）把 DC-1/DC-2 差分推到真实宿主 tab 切换路径——双形态编排（ui-driver JSONL 功能腿 + xdotool 判别腿，前者每 tick 强制成帧失灵敏只能做功能验证）：DC-1 timer 钳制端到端成立（renderer 前台 0.052 → hidden 0.004 → 恢复 0.046 cores，13× 塌缩回带）；判别腿抓到 hidden tab 呈现链缺口——offscreen 门控只按视口剔除，tab 隐藏页动画声明照发、`animations_active` 保持置位，browser keep-alive Get 循环对不可见 surface 持续空转（compositor 0.593 cores/browser 4.918）；`visible_declarations` 增 `page_hidden` 维度收口（hidden 页声明清空 → animations_active 回落 → keep-alive 自动停，与 offscreen 维度同构、集合差分自动恢复），修复验证同段差分 compositor→0.370/browser→2.448（-38%/-50%，前台 static 页同构对照），恢复段 0.565 回前台同带，残留为窗口级基线成本，见 [M4 GUI tab 切换](background-resource-lifecycle/evidence/2026-10-23-m4-gui-tab-switch/ANALYSIS.md)。SW 面为无延迟 FIFO 简化模型（delay 参数被忽略）不适用本钳制语义；promise 续体欠钳为已知缺口（FIXME）。观测资产：`scripts/background-resource-{observe.sh,probe.mjs,sampler.py,fixture-server.py}` + `tests/fixtures/background-resource/`。

范围：各进程 CPU、RSS 趋势、旧文档/回调/连接/队列与呈现次数；前后台切换、重复导航和关闭后恢复。

边界：复用 [page-state-compat](archive/page-state-compat.md)、[navigation-compat](navigation-compat.md)、现有 regression-guard。可见性 API 规范复用历史成果，本 goal负责长期生产资源；不绕过 iframe 或深结构门控。

碰撞：按 [run-rules §9–11](../rally/run-rules.md)协调 browser/renderer/page-runtime/engine/compositor；只写本目标控制面；不停止用户实例。

## Done Criteria / 验收场景（P0）

| ID | 条件与动作 | 期望与验证 |
|---|---|---|
| DC-1 / BG-01 | 连续零延迟嵌套 timer，切后台再恢复 | 嵌套最小延迟符合规范；后台策略明确且恢复正确。拟新增 timer 时序测试与生产 CPU 采样 |
| DC-2 / BG-02 | 动画移出视口、页面静止、隐藏标签 | 可跳过工作不持续重复呈现；音视频等合法工作不误停。生产帧计数和进程 CPU 对照 |
| DC-3 / BG-03 | 重复导航，新页有不结束子资源 | 已替换文档、回调与资源可释放；RSS 不随替换次数无界增长。拟新增本地延迟 HTTP fixture 与文档存活计数 |
| DC-4 / BG-04 | 关闭/重开页面并切回前台 | 本次 renderer/连接/队列退出或复用符合生命周期；前台输入与动画恢复。真实宿主操作与资源清理记录 |
| DC-5 | 固定场景持续运行并复测修复 | 时长、预热、样本、阈值和环境先记录；缺指标标 NOT_RUN/INCONCLUSIVE，不用单次峰值证明无泄漏 |

## 里程碑与实施交接

| 阶段 | 文件/职责与依赖 | 产物与验证 |
|---|---|---|
| M1 | 现有 regression-guard/进程采样 + 隔离浏览器 fixture | 四类场景基线；固定 run ID/profile/PID/视口/时间；先对比前后台 |
| M2 | page-runtime/renderer timer 与导航生命周期 | 首个增长来源调用链、最小回归；确认后修共享 owner |
| M3 | engine/compositor 帧调度 | 空闲/视口外工作门控与恢复行为；真实 GPU 计数 |
| M4 | 既有性能资源门禁 | 建立同环境阈值，长期配对复测、DC 审计与清理 |

实现来源：进程采样优先复用 regression-guard；文档/队列计数拟加在真实 owner 的诊断入口，默认不输出到页面。本地 fixture 放最近集成测试资源；守卫限制资源和墙钟。建议分批：基线资产、确认泄漏修复、调度修复。回滚本批调度或 owner 修改，保留观测与旧失败证据。

## 执行与 Document Control

`./scripts/rally-goal-background-resource-lifecycle.sh` 直接执行启动 Rally，加 `--dry-run` 仅预览；统一 CLI 与证据协议见 [启动说明](ladybird-followups/README.md)。

启动前不采样用户浏览器。入口稳定，master/evidence 更新，archive 只追加；启动后 `CONTINUE` / `BLOCK`，全部 DC 与适用门禁有证才 `DONE`。
