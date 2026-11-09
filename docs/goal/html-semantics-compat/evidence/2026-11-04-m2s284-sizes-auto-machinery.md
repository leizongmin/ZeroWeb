# M2 s284 — img sizes=auto 机具三件套（sizes-auto 族 6F+1T→68P/4F/1T、dynamic-attributes 3F→3P，imgfamily GAINED=69/FIXED=1/REGRESSED=0）

日期：2026-11-04。切片号 s284（承 s283 下一步候选①——sizes=auto 机具
三件套：F5 简化挂起项解除 + 渲染宽度源尺寸 + lazy script-created 通道）。

## 勘根

1. **F5 简化挂账三根因**（探针页法 + shim trace 定谳）：①parse sizes
   attr 的 auto 关键字臂缺席（s275 前置简化恒忽略 auto 项 → 落 100vw
   → 51w 源被选 → ref1 断言面全假）；②初版语境判据 `imgSel != null`
   把 script 创建节点（identity = handle `__n{n}`、sel 恒 null）整体
   错判无语境——同页 createElement img 与静态 img 行为分叉的机制位；
   ③script 创建 lazy 图 gBCR handle 恒 miss（宿主 rect 快照按 selector
   键）→ 插入后零 rect → defer 臂挂冻结请求后无唤醒点（observers_tick
   未接渲染机会）。
2. **初版全局 gBCR handle 解析暴露既有缺口面 35 行**（family A/B
   REGRESSED 勘根定性，两层）：a) naturalWidth 族 ~30 行 = clone img
   真实 rect 泄入 s283 IDL rendered 臂——基线经 attr 臂巧合通过
   （density 折算值恰等期望；全局点亮后 rendered 值 20/50 泄入期望
   10/40 断言面）；SVG 纵横比-only 填充行（data-width=720 基线即 18F）
   非本片位移。b) fetch 守卫 5 行 = 插入 hook 通用点亮对每个插入 img
   盲重验——引擎 fetch 核缺 spec update-the-image-data 冗余更新早退
  （image-base-url 不重取断言 / decode attach 时序 ×2 /
   current-request-microtask 任务计数 / relevant-mutations picture
   ancestor）。
3. **处置 = 域边界归位**：渲染/fetch 缺口（槽 2a/2b 真面）另立候选
  （见下一步候选），本片把暴露面收窄到 img 机具自身调用点——宿主
   回调 `__zw_getBoundingClientRect` 保持共享面零变化。

## 施工（Rust 1 文件 + shim 2 文件 + 语料 1 资产）

- `crates/engine/src/js_dom_bridge/callbacks.rs`：新宿主回调
  `__zw_resolve_handle_selector`（handle→selector 正置表
  `sel_handle_map_snapshot` 查询，未命中返空串）——刻意不并入 gBCR
  共享回调（回调注释记 blast radius 勘根）。
- `crates/engine/src/js_dom_shim/part03.js`：
  - parse auto 臂（spec letter-exact，images.html#parse-a-sizes-
    attribute）：img 非 null 且 allows auto-sizes 时——being rendered →
    auto = concrete object size width（gBCR 宽）并记 last auto-sizes
    width；未 rendered 且 last 非 null → 用 last；否则 auto 项忽略续。
  - `__zwImgAllowsAutoSizes5277`（loading=lazy + sizes 原始值
    auto/auto,\* ASCII CI 无 trim）/`__zwImgBeingRendered5277`
   （isConnected + gBCR 非全零）/`__zwImgResolveIdentity5277`
   （handle→selector 解析，仅 img 机具调用点使用）三 helper。
  - 三 rect 消费点（BeingRendered/NearViewport/parse auto 臂）rect
    查询前自解析身份，解析成功走 selector 面静态身份，未命中回落
    零 rect 旧行为。
  - source 分支 omitted sizes ≡ 'auto' 语境 threading（img 允许时）。
  - 插入 hook（`__zwImgTreeMutated5277` 非 picture 父 IMG 臂，spec
    relevant mutation 插入步）**门控 allowsAuto**——通用点亮 = 盲重验
    （槽 2b 机制），非 auto img 插入回落基线行为（无重验）。
  - defer 臂旗标 `_zwLazyDeferredAny5277`（仅 rect 不可判且非
    display:none 才重臂，常态 tick 零开销）+ sweep 扩展（唤醒循环前置
    删 allowsAuto img 冻结请求 → viewport-revalidate 走冻结值直验，
    s275 语义）。
- `crates/engine/src/js_dom_shim/part01.js`：observers_tick 尾旗标门
  → `_defer(__zwImgSweepDeferredLazy5277)`——observers tick = host
  渲染机会（runner 探针循环与 renderer page_scripts tick 同点），`_defer`
  microtask 殿后本 checkpoint 的 RO/IO 回调，与 spec 渲染机会 RO 批序
  一致（sizes-auto-resize-observer 页：页面 RO 先改 sizes，引擎 sweep
  再按新值重选）。
- 语料 ③f：`images/red-16x16.png`（130B，gh api contents 按 lib.sh
  WPT_REV pin 取件，jsdelivr 字节 cmp 核对）+
  `25-html-semantics-compat.sh` fetch_raw + `update-wpt-data.sh`
  升级注记——sizes-auto-dynamic-attributes 页断言面 404 → onerror
  载入断言永假 3F，补后转子测真面（3P，含 eager flip 面）。

## 验证

- **naturalWidth solo（暴露面收窄后）**：240P/18F——失败集与基线
  **逐键一致**（diff 空，日志 s284-naturalwidth-confined.log vs
  s283-imgfamily-b.log）；基线 18F = SVG 纵横比-only 填充行（既有
  引擎缺口，非本片位移）。
- **sizes 目录 solo**：sizes-auto 68P/4F + 1 文件 Timeout（4F = 行 28
  writing-mode vertical-rl 逻辑属性 / 行 34 abs-pos replaced 宽，均
  既有布局缺口；Timeout = 行 11 hidden attr vs author display 级联
  位）；dynamic-attributes 3P；resize-observer-removes-auto 1P；
  implicit-sizes-ignores-width 1P（s283 修复面保持）。探针页
  probe-s284-sweep/clone-hint 已删（临时勘根件）。
- **fetch 守卫 5 页复绿**（hook 门控后）：relevant-mutations 113P/0F、
  image-base-url 1P、decode-quick-attach 9P、microtask-002 1P；
  empty-src-no-current-request 回基线形态 1P + 1 文件 Timeout
  （ungated hook 期 "With srcset" 子测完成系盲重验意外副产品，同守卫
  缺口不确定面，归候选①）。
- **family A/B**（FILTER=the-img-element，A = s283-imgfamily-b.log 同
  runner 同语料 + ③f 补件；日志 s284-imgfamily-c.log）：904P/45F/9T →
  969P/48F/8T，键级 **GAINED=69 / FIXED=1 / LOST=2 / REGRESSED=0 /
  T-changed=0**。FIXED=1 = resize-observer-removes-auto（本片真战果）；
  GAINED=69 = sizes-auto 机具面 + ③f 语料位移；LOST=2 归因：
  probe-s283-layout.html（基线日志先于探针删除的陈旧面）+
  sizes-auto-dynamic-attributes 文件级 Timeout 键 → 子测真面（③f
  注记预告位移）。
- **clippy + fmt**：zero-engine --all-targets 零告警；fmt --check 干净。
- **make test**：9085P/5F——5F 恰为 s283 勘根⑤五红测原键（p56/p99s/
  r2809/r2810/r3276，CSSOM cssText 序列化域兄弟流在途，rule 9/10
  记账），QuickJS 矩阵绿。
- **全树 A/B**（A = s283-fulltree-b.log，同 runner；日志
  s284-fulltree-c.log）：7876P/407F/62T → 7943P/410F/60T，键级
  **GAINED=70 / FIXED=1 / LOST=2 / REGRESSED=0 / T-changed=0**。
  LOST=2 归因：video-loading-lazy-in-viewport（基线文件级 Timeout →
  本轮子测 Pass——s281 账本 video-loading 族残留 1T 的时序翻转，
  方向良性非本片面）+ sizes-auto-dynamic-attributes 文件键（③f
  语料位移同 family）；GAINED=70 = sizes-auto 机具行 + 语料位移 +
  video 翻转 1 行。

## 挂账更新

- **s283 候选①（sizes=auto 机具三件套）销案收口**。
- **新挂账（勘根定性，域外候选）**：
  ①fetch 核冗余更新守卫（spec update-the-image-data 同 URL 已可用
  早退——image-base-url/decode/microtask/relevant-mutations 5 行 +
  empty-src "With srcset" 真面，fetch 语义域）；
  ②SVG 纵横比-only 块级填充（naturalWidth 18F 基线真面——CSS2
  §10.3.4 + css-images 默认尺寸算法纵横比臂，rendering 域跨切）；
  ③clone img density 校正渲染缺口（潜在面：rect 流入 IDL rendered
  臂即现形 10→20，rendering 域）；
  ④display:none 后 rect 快照清零（IDL rendered/not-rendered 边界，
  not-rendered 断言面潜在）。
- **保持**：cloneNode 返回代理 IDL 断（clone.width undefined，既有
  挂账，corpus 经 document.images 重查规避）；sizes-auto 残 4F+1T
  （行 28/34 布局面、row 11 级联面——各归其域）。

## 下一步候选

1. **fetch 核冗余更新守卫**（新挂账①，5+1 行真面，engine fetch 语义）。
2. **SVG 纵横比-only 填充**（新挂账②，naturalWidth 18F，rendering
   域跨切）。
3. adoption/resize-observer 真面勘根（s283 候选③保持）。

## 证据

- `evidence/logs/s284-naturalwidth-confined.log`（naturalWidth solo，
  收窄后）
- `evidence/logs/s284-sizesdir-gated.log`（sizes 目录 solo，hook 门控后）
- `evidence/logs/s284-imgfamily-c.log`（family B；A =
  s283-imgfamily-b.log）
- `evidence/logs/s284-fulltree-c.log`（全树 B；A = s283-fulltree-b.log）
- `evidence/logs/s284-make-test.log`（全量 make test）
- 日志 md5：naturalwidth f9d180fe…、sizesdir 3991e84b…、imgfamily
  90f703e8…、fulltree fbfde8b1…、make-test a01dfbaf…
