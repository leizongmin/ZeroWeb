# M2 s293 — 运行核图片布局表同步（naturalWidth 12F→0F、srcset 克隆几何面解锁；全树 A/B 翻转恰 12 枚全 F→P 第 25 个连续完美；plain-src 运行时 img 触发缺口 ×2 挂账）

日期：2026-11-05。切片号 s293（承 s292 下一步候选①——运行时表同步，同域直
后继）。勘根+施工主体为上一 session 产出（429 中断于测试重构段），本轮接
续完成：测试勘误收敛（静态哨兵期望改正 + img2 臂删除）、验证链全跑、落账。

## 勘根

**核心缺口**：load 期图源清单来自 `extract_img_srcs`（静态 HTML 源解析 +
R5275 模板 walk），运行时创建/换源的 img（script 注入、srcset 换源）不在
其中——位图 settle（`__zwImgRunUpdate5277` fetch→createImageBitmap 成功回
调）后 `cached_image_sizes/ratios/no_ratio` 三表无该资源键，双面失真：

1. **布局零盒**：layout 无固有信号 → 无信号 auto 回落（replica 探针勘根
   s292：srcset 克隆 bb 730/bare1、vb1 bb 高 10 等拉伸面）；
2. **s292 IDL 几何臂关门**：`__zwImgHasIntrinsicSignal` 三表任一含键 + 快
   照冻结判据 miss → dN==null 回落 naturalDim 近似——corpus
   naturalWidth-naturalHeight-width-height srcset/1x+2x 克隆 12F 维持
   （IDL width 期望 720 实测 300）。

**修复路径**（host 排空式，非 shim 直写）：运行核 settle 臂经宿主回调
`__zwImgSyncRuntimeTable(_imSrc)` 入待补队列；webview 异步回调泵轮钩子
（execute 入口 / drain_async_callbacks bulk / drain_next_async_callback_if_
pending 逐 resolve ×3）逐 resolve 后 `apply_runtime_image_syncs()` 排空——
decode_data_uri 原样键 / HTTP base join→image_cache 命中优先→image_source_
fetcher→resource_get(MEDIUM,"image")（与 fetch_image_subresources 同口径；
已知边界 image_natural_sizes 不补，paint border-image 9-slice 面不在本片断
言域）→解码→R717 分类→补三表 + `runtime_img_sync_applied` 容器（s292 信
号门快照冻结，补表增量并列查询）+ image_cache→`repaint_after_runtime_image_
sync()`（sync_pipeline_page_state + repaint_cached_viewport + absorb_render_
result + R150 refresh_layout_rect_snapshot）——同泵轮内落表+重绘，IDL 几何
臂读到真实渲染维。队列 256 上限丢旧（runner 泵轮逐 resolve 排空永不触顶；
无泵消费环境封顶防滞留增长）。

**测试勘根二件（上轮遗留，本轮定谳）**：

- **静态哨兵期望错误**：上轮 test 2 静态哨兵期望 300/100 与 corpus 矛盾——
  corpus 静态 viewBox 面期望 data-width=720/height=240（s292 约束方程既有
  行为）。429 中断前该期望未经 solo 验证。本轮改正并经探针证实 300 = 布局
  CB 不可解析（img 直挂 body，auto CB → default object size fallback 300×
  100，s292 缺口1 臂设计内）+ 几何臂忠实上报渲染维 310−10=300，非信号门缺
  口。测试页 img 移入显式 740px CB 后 720/240 归位。
- **img2 plain-src 运行时臂删除**：守卫形为上轮自创（corpus 无此形态，全
  克隆走 srcset）。探针三轮（zz-s293-probe 全 replica，已删）实证 img2 不
  settle 是两个**先在引擎缺口叠加**，均非 s293 引入（见挂账①②）。

## 施工（2 文件 + 1 新集成测试）

- `crates/engine/src/js_dom_shim/part03.js`（+10 行）：运行核 settle 臂
  （`_zwAvailImgs[_imSrc]` 记账后、`_zwImgDispatch('load')` 前）插
  `__zwImgSyncRuntimeTable(_imSrc)` try/catch 调用——srcset/plain-src/
  picture 选中源全走此臂，天然覆盖。
- `crates/webview/src/webview.rs`（+225 行）：`RuntimeImageSignalTables`
  struct（sizes/ratios/no_ratio 三 HashMap）+ `runtime_img_sync_pending` /
  `runtime_img_sync_applied`（Arc<Mutex>）字段 + `apply_runtime_image_syncs`
  + `repaint_after_runtime_image_sync` + 泵轮钩子 ×3 + `__zwImgSyncRuntimeTable`
  回调注册（256 丢旧）+ `__zwImgHasIntrinsicSignal` 门并列查询 applied 容器。
- `tests/integration/tests/r5424_runtime_img_table_sync.rs`（新，两测）：
  ①★load-bearing corpus 12F 驱动形（script 注入 srcset img，viewBox-only
  SVG + " 1x"、740px CB、border/margin 5px display:block——IDL 720/240 翻
  转 + naturalWidth/Height 300/100 resourceState 臂不受补表影响）；②静态哨
  兵（load 期清单 img 同 CB 形——s293 零变更哨兵，corpus data-width=720 口
  径）。fetch handler 只服务 data: SVG 百分号解码直构 FetchResponse（runner
  wpt_data_fetch_handler 同形）；pump_until_settled 泵到 img.complete + 补一
  轮稳态 + `__zwFireStaticImgSettle()`（runner 生命周期 task2 同款）。

## 验证

- **r5424 两测 2/2 Pass**（test-guard 包裹，0.33s；勘误收敛后）。
- **前置**：concat node --check 18 片干净；`cargo fmt` clean（webview 2 处
  + integration vec_init_then_push 修正）；`make guarded-clippy` workspace
  -D warnings 干净（8.42s 末轮）。
- **solo naturalWidth**（B 侧全量域）：A 246P/12F（s292 基线）→ **B 258P/
  0F/0T**——s292 残 12F 恰 = srcset/1x+2x 变体全数翻绿，面边界精确闭合。
- **solo 哨兵门**（B 侧，期望零位移）：density-vs-zoom **6P/3F**（3F 同
  s292 基线 = svg-img-density-vs-zoom.html ×3 zoom 渲染流车道，键级同集）；
  relevant-mutations **126P/0F/0T**（s291 基线原样）。
- **全树 A/B**（双方 TIME_LIMIT=14400 RAF_FRAME_DRIVEN=1 test-guard 内建）：
  **8419 键双侧同集（A-only/B-only 全 0），翻转恰 12 枚全 F→P** 全为
  naturalWidth srcset 面（6 面 × 2 密度），P2F=0 / REGRESSED=0，行级
  7970P/395F/57T（A）→ **7982P/383F/57T**（B）（ΔP=+12/ΔF=−12/ΔT=0）——
  **第 25 个连续完美 A/B**。
- **make test**：**80 套件 9130P/5F**（基线 79 套件 9127P/6F + 我方 r5424
  测试二进制 +1 套件/+2P + r3033 上游自修复翻绿 +1P/−1F，s292 跨流归因闭账
  对账）——5F 全为已知 CSSOM cssText 域红（p56/p99s/r2809/r2810/r3276，
  s288-s292 逐键同红），**本片零新增红**。
- **product-smoke / reftest / bench-gate**：product-smoke **PASS**（struct-
  check 0 issues）+ product-smoke-legacy struct-check failures **0**
  （trend-only）；`make reftest` **Layout 521/521 + Text 204/204 全 100%**
  EXIT=0；定向 bench-gate（`ZERO_WEB_BENCH_CRATES=zero-webview,zero-engine`）
  **GATE PASS** 19 指标全在预算内——首轮 FAIL 三指标（page/medium layout/
  paint/first_paint ~2.2x 均匀超调、peak_rss PASS）为负载窗测量噪声
  （15min loadavg 89 收敛中 + 并行流 rustc cohort），静窗复跑即 PASS，
  首轮日志不保留定谳依据（复跑日志为准）。

## 挂账更新

- **新挂账① plain-src 运行时 img 触发缺口**（本片探针定谳，corpus 无此形
  态）：`createElement('img')` + `.src=`/setAttribute('src') 运行时 img 不
  settle，三段复合——set 期 run detached 延步（s291 ⓷ isConnected 检查）
  插入后无 flush 驱动；插入钩子 s290 ⓵ spec 正确排除 plain-src（「does not
  use srcset or picture」env 门只管 srcset/picture 路径——spec 行为非
  bug）；set/insert 触发臂被 `_realTag` handle-only 回落 DIV 挡（s168 在册
  容忍语义——IDL setter part05:992 与 attr-mutated IMG 分支 part03:3826
  均宿主表 miss）。三段独立，根修候选 = _realTag 宿主侧补登记或 defer flush
  挂 observers_tick。本片不修（守卫面自创非语料钉面，按精准修改准则不入片）。
- **新挂账② 运行时创建元素 sweep-settle 身份缺口**（s285 物化缺口族扩展
  面）：`__zwFireStaticImgSettle` sweep qsa 可见该元素（qsa=2）但逐元素臂
  settle 同树 img1 跳过 img2——qsa wrapper 身份链（sel/handle → attr store）
  断裂致选源臂 getAttribute 空 → `if (!_zwImSrc) continue` 静默跳过。根修
  随 s285 宿主 documentElement 物化缺口根修一并评估。
- s285 挂账（宿主 documentElement 物化缺口根修）维持；s221 picture 父 IMG
  臂 env 化维持独立评估。

## 下一步候选

1. **picture 父 IMG 臂 env 化独立评估**（s221 microtask 时序面，维持）。
2. 宿主 documentElement 物化缺口根修（s285 挂账 + 本片挂账②并案评估）。
3. plain-src 运行时 img 触发缺口根修（挂账①——需先立 corpus 钉面或确认
   runner 生命周期 sweep 已覆盖实际断言域）。

## 证据

- 上一 session（429 前勘根/施工轮）：s293-probe-baseline.log（A 侧基线探
  针）、s293-probe-wip.log（WIP 探针）、s293-solo-wip.log（naturalWidth
  solo B 侧 258P/0F/0T）、s293-fulltree-b.log（B 侧全树 7982P/383F/57T）、
  s293-fulltree-a.log（A 侧全树 7970P/395F/57T，本轮补跑）；B 侧 runner 二
  进制存档 target/s293-runner-b（cp 回 target/release 后复用 solo 哨兵门）。
- 本轮（勘误收敛+验证链）：s293-density-vs-zoom-solo.log（6P/3F 基线同键）、
  s293-relevant-mutations-solo.log（126P/0F/0T）、s293-r5424-test.log（2/2
  Pass 终态轮；中间轮含静态哨兵 300 误期望与探针迭代，探针文件已删不留档）、
  s293-make-test.log（80 套件 9130P/5F）、s293-reftest.log（521+204 全绿）、
  s293-product-smoke.log / s293-product-smoke-legacy.log、
  s293-bench-gate.log（定向 GATE PASS 静窗轮）。
- 日志 md5：probe-baseline 27319030、probe-wip 39a47ded、solo-wip aa335785、
  fulltree-b a08ac57c、fulltree-a eb440b74、density-solo 386f1acd、relmut-
  solo cbe39a83、r5424-test d7be5049（终态轮）、make-test 5b0fd069、
  reftest 789ebe2a、smoke a1b1d56e、smoke-legacy 57a8902b、bench-gate
  be24e5df。
