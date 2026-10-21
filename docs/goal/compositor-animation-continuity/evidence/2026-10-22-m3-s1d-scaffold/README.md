# M3：S1d 前置脚手架 + DC-4 呈现前置阻塞发现（主线程 canvas 不出图）

日期：2026-10-22 ｜ 里程碑：M3（S1d 脚手架批 + 发现批）｜ 判据：脚手架自检 + DC-4 前置缺口

## 判定

两项。**①脚手架就绪**：comp04a fixture + probe/observe/analyze 工具链落地，flag 关对照
腿（DC-4 (b) 的 OFF 半边）通过——`SHIM_BASELINE_PASS`，worker 侧 beat 跨忙窗停顿 931ms
复现 comp04 基线签名（~949ms 同量级），错误路径回流、正控通道、帧活性全绿。
**②发现 DC-4 呈现前置阻塞（BASELINE 级）**：**主线程 canvas 2D 内容在应用显示路径
从不出图**（headless 截图 / headless frame dump / 真实窗口 frame dump 三路同证）——
页面只看到 canvas 元素的 CSS 背景色，位图内容（含静态 draw-once、display:block、
动画三种形态）全部缺失。根因已插桩定位（见下）。DC-4 的「支持的 2D 子集持续呈现」
在打通 canvas 桥之前不可判定，该修复成为 S1b 的直接前置（同需 registry 跨线程共享
机制，与 RFC §2.2/§5 备选案共用）。

## ① 脚手架（本批新增）

| 件 | 说明 |
|---|---|
| tests/fixtures/compositor-animation/comp04a-worker-threaded-canvas.html | DC-4 场景 fixture（自冻结版 comp04 派生，几何/时间线常量一致；mode 分流：`typeof __zw_worker_create` 探测 S1a 宿主桥——threaded 走 transferControlToOffscreen+真线程绘制+terminate@6500ms，shim 维持 comp04 同款基线语义） |
| scripts/compositor-animation-comp04a-probe.sh/.mjs | CDP 探针（comp04 驱动同款连接面）：模式面 / worker 侧 beat 时钟（wt，Date.now()）跨忙窗间隔（线程活性判据）/ 转移面 / terminate 静默 / 错误路径；判定状态 SHIM_BASELINE_PASS · THREADED_PROBE_PASS 等 |
| scripts/compositor-animation-dc4-observe.sh | frame_dump 512 帧观察（dc3-observe 同构；on=ZW_THREADED_WORKERS=1 / off 对照） |
| scripts/compositor-animation-dc4-analyze.py | canvas 区域 (24,24)-(264,144) 逐帧哈希 → 变化事件 + 冻结段结构化识别（不依赖时间线常量）：off 预期忙窗 ≥800ms 冻结段；on 预期连续变化 + terminate 终末冻结 |

### flag 关对照实测（validate-050202 + observe-off-validate）

| 面 | 指标 | 值 | 解读 |
|---|---|---|---|
| 模式 | mode / worker_host | `shim` / `false` | S1a 未落地，走同 realm 对照（预期） |
| 线程活性 | wt 跨忙窗最大间隔 | **931ms**（忙窗前 ~53ms） | 同 realm worker 与主线程同载——comp04 基线 949ms 复现 |
| 投递 | at 跨忙窗最大间隔 | 932ms | 主线程送达停摆（固有语义对照列） |
| 转移 | transfer | `probe-only`（typeof=function） | flag 关只探不调（R3313 语义不动） |
| 错误路径 | worker_err | `Uncaught Error: dc4-expected` | shim 错误事件回流页面 onerror ✓（DC-4 (d) shim 半边） |
| 正控 | mutation → 新帧 / ctrl 条带 | `OK` / 换色 | 采样链路活跃 |
| 帧序列 | 全帧 distinct / canvas 区域 distinct | 111 / **1** | 全帧持续变化（beat 条带），canvas 区域恒定——见发现② |
| 探针判定 | state | **SHIM_BASELINE_PASS** | OFF 对照腿成立 |

驱动器修正记录：页面 `performance.now()` 的 time origin 不保证与 loadHtml 对齐
（实测可继承前一文档/renderer 起点），探针时间线改为「load 完成锚点 + 相对偏移」，
`waitPageOffset` 对 evaluate 失败有界退出（首版绝对目标 + 失败置零导致 120s 挂起，
已修）。

## ② 发现：主线程 canvas 位图在应用路径从不出图（DC-4 前置阻塞）

### 现象（三路显示路径 × 多形态）

- headless CDP 截图（`ZW_HEADLESS_GPU_SCREENSHOT=1`）：comp04 冻结基线的 s0–s3 截图
  即已空白（复核证据目录截图可见），本批 comp04a 四采样同；
- headless + 真实窗口 frame_dump（compositor 侧产物）：canvas 区域逐帧哈希全程恒定
  （`distinct_region_hashes=1`，119 帧跨忙窗无一片变化），同帧全帧哈希 111 distinct
  （beat 条带文本持续更新——文本/背景绘制链路正常）；
- 形态无关：静态 draw-once、双 canvas（文档流 + position:absolute）、display:block、
  16ms 动画环，全部空白；页面侧执行确认（attr 已写、fillRect 无异常、draws 计数推进）。

### 插桩定位（临时插桩已回退，证据 canvas-presentation-diag.txt）

三路由白 → 渲染器侧插桩（webview absorb / publish 载荷 / painter 各早退点 +
paint_node_inner 入口），真实窗口 + renderer stderr echo 采得每帧一致签名：

```
paint_node_inner canvas box_id=Some(NodeId(17v1))          # 走查到达 canvas 盒
paint_canvas entered ctx_attr=Some("1")                    # attr 有，ctx id=1
paint_canvas skip no-ctx-snapshot ctx=1 reg_contexts=0     # 71/72 帧：painter 持有的 registry 空
```

`canvas_images` 始终空（webview absorb 零命中）→ 发布无像素 → compositor ImageCache
无 key → 位图缺失，仅 CSS 背景出图（compositor 日志 `fills=3` 佐证）。

### 根因

**多进程 tab 路径的页面 JS 沙盒与绘制侧各持一个 CanvasRegistry 实例**：
`apps/renderer/src/js_worker.rs:1173`（`js_worker_main` = renderer-js 页面 JS 线程）
自建 `Arc::new(CanvasRegistry::new())` 并将页面域 `__zw_canvas_op` 注册到该实例；
绘制侧 painter/pipeline 持有的是 webview 构造时的另一实例（webview.rs:704-706，
`new_with_indexed_db_owner`）。页面绘制写入孤儿 registry，painter 侧恒空。进程内路径
（run_page_scripts/reftest/WPT harness）经 webview.rs:3102/3252/4184 注册、共享同
Arc——这解释了 canvas 2d reftest 全绿而应用路径全空。

### 修复方向（下一步批，S1b 前置）

将绘制侧 registry Arc 下推到 renderer runtime 启动处并传入 `js_worker_main`（跨线程
共享 `Arc<Mutex<CanvasRegistry>>` 已核实 Send，Mutex 单操作粒度；与 M3-S1 RFC §5
补记的共享 registry 案同一机制）。触面：apps/renderer（runtime/js_worker 签名 +
spawn 参数）+ 零行为回归面（进程内路径不动）。验证：comp04a flag 关跑 canvas 区域
出现持续变化（`distinct_region_hashes>1` + 忙窗 ≥800ms 冻结段复现）+ canvas reftest
语料 + workers-compat 零回归。

## 工件

| 文件 | 内容 |
|---|---|
| comp04a-probe.json / comp04a-s{0,3}.png / probe-headless.txt | flag 关探针判定与截图（s3 可见空白 canvas + 全绿条带） |
| observe-off-analysis.txt / run-config.txt | frame_dump 区域哈希分析（region 恒定 = 位图缺失的 compositor 侧证据） |
| canvas-presentation-diag.txt | 插桩诊断全量行（361 行：72 帧走查 / 71 次 no-ctx-snapshot reg_contexts=0） |

## 局限

- 脚手架 threaded 腿（mode=threaded 的 transfer/terminate/静默判定）未跑——前置①S1a
  宿主桥未落地②本发现修复，工具链就绪待用。
- 插桩运行时的具体数值（reg_contexts 计数、NodeId）依赖当次运行，诊断行仅作归因
  证据；修复批将以 comp04a flag 关全链复跑定版。
- observe-off 的 512 帧上界在 16s 窗内实际产出 111 帧（compositor 实际出帧率），
  冻结段分析口径不受影响（区域恒定即证据）。
