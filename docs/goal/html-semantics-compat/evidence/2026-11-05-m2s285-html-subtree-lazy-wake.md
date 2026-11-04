# M2 s285 — html 子树 lazy 唤醒链修复（empty-src 2P 文件完成、imgfamily FIXED=1/REGRESSED=0、fulltree FIXED=5/REGRESSED=0）

日期：2026-11-05。切片号 s285（承 s284 下一步候选①——fetch 核冗余更新
守卫）。勘根转向：候选①的真阻断面不在守卫，在 html 子树 script 创建
lazy 图的插入后唤醒链死锁；守卫面 5 行现状绿，候选①保持挂账。

## 勘根

1. **宿主物化缺口**（探针梯 probe1-7 定谳）：documentElement 父插入
   shim 树有、宿主 apply 静默丢弃（宿主侧 `__zw_query_all('html > img')`
   空、`body > img` 有；createElement 路径同）→ forward handle map 无
   条目 → `__zw_resolve_handle_selector` 恒空 → NearViewport rect 永不
   可判 → s271 门恒 false → 冻结请求挂而永无消费点 → 无限 re-defer →
   文件 Timeout。
2. **等待窗周期抹除**（RunUpdate 全臂临时 trace 定谳）：runner
   loadStep5275 等待循环每 100ms 重跑 `fireSettle5275` →
   `__zwFireStaticImgSettle` 全表重置（含 s275 frozen 冻结请求表）——
   重试期是**同文档**动态窗口，动态期 defer 臂刚登记的请求活不过
   100ms（trace：DEFER 后 SETTLE-WIPE 连环）→ 插入旗标臂查无 frozen →
   永久 dormant。s284 勘根挂账「empty-src With srcset 真面归候选①」
   的真机制位在此。
3. corpus test1 "Without srcset" error 来源查明：`src=""` setter 以空
   串走 RunUpdate → else 臂（fetch 不可用/空 src）派 error（trace
   `NOFETCH src=` 实证）——基线即过面，本片不动。

## 施工（shim 2 文件 + runner 1 文件）

- `crates/engine/src/js_dom_shim/part03.js`：
  - **NearViewport miss 判近臂**：身份未命中（script 创建节点 sel null
    + forward map 无条目）分两支——body 子树 = 应用时窗（快照随后必
    命中）保持判不近（below-viewport-dynamic 零回归向量）；html 子树 =
    宿主不物化 rect 永不可判，且 shim 树已连接、display≠none → 判近
    （headless 无真视口无法证明在视口外，spec 插入即相关变异应启动
    装载——s70「旗标置即开」同哲学：视口模型未就绪按可见处理）。
  - **插入旗标臂**（`__zwImgTreeMutated5277` 非 picture 父 IMG 臂，
    s284 allowsAuto 门控的补充）：非 auto lazy 图插入且无 state/不在飞
    → 重臂唤醒旗标，observers_tick sweep → viewport-revalidate（有
    frozen 直验 s275 语义；无 frozen 全量重验过门）。不依赖 frozen
    存活（等待窗抹除的二阶兜底）；eager/已启动图不臂，规避 s284 勘根
    ungated hook 盲重验 5 行面。
- `crates/engine/src/js_dom_shim/part04.js`：`__zwFireStaticImgSettle`
  加 skipResets 参数——等待窗重试路径只跑静态扫描/pending drain，7 表
  重置（选源记录/代际/在飞/frozen/after-load 队列）跳过；跨案隔离由
  每次文档两个全量调用点保证（runner 初始 settle + window load 派发
  钩子 part06）。
- `tests/wpt-runner/src/testharness.rs`：`fireSettle5275` 透传参，
  loadStep5275 重试路径调 drain-only（初始 settle 调用不变）。

## 验证

- **empty-src solo**：2P 文件完成（基线 1P + 1 文件 Timeout；日志
  s285-empty-src-solo.log，md5 4afa1fd6）。
- **fetch 守卫 5 哨兵 solo 复绿**：relevant-mutations 121P/0F、
  image-base-url 1P、current-request-microtask 零 Fail 行。
- **family A/B**（FILTER=the-img-element，A = s284-imgfamily-c.log，
  B = s285-imgfamily-d.log）：973P/54F/7T，键级 **FIXED=1 /
  GAINED=0 / LOST=0 / REGRESSED=0**（FIXED = empty-src T→P 即本片
  目标面；T-changed=1 同面）。
- **全树 A/B**（A = s284-fulltree-c.log，B = s285-fulltree-d.log）：
  7951P/406F/59T，键级 **FIXED=5 / GAINED=0 / LOST=0 / REGRESSED=0 /
  T-changed=1**。FIXED = empty-src T→P + video-loading 4F→P。
- **video 4F→P 归因**：机制零耦合（grep 实证 part05 video 机具零引用
  gated 表；NearViewport/插入臂 img-tag 门控）——归因等待窗时序位移
  （家族 s281 起时序不稳定账，s284 曾记 in-viewport T→P 翻转同型）；
  4 面全树 + 两轮 solo 三语境稳定绿、同族 load-when-visible 三语境
  稳定红（翻转率非噪声）；方向良性（日志 s285-video-family-run1/2）。
- **clippy + fmt**：zero-wpt-runner --all-targets -D warnings 零告警；
  fmt --check 干净。
- **make test**：9088P/5F——5F 恰为 s283 勘根⑤五红测原键（p56/p99s/
  r2809/r2810/r3276，CSSOM cssText 序列化域兄弟流在途，rule 9/10
  记账）。首轮 6F 含 r5296 paint 偶发（单测复跑 PASS 实证偶发；paint
  域与 shim/runner 改动无耦合）。
- 探针 7 页已删（probe-s285-frozen/sweep/resolve/ab/corp2/ctx/ctx2）；
  临时 trace 6 处已撤。

## 挂账更新

- **原候选①（fetch 核冗余更新守卫）保持挂账**——本片勘根其真阻断
  面为唤醒链死锁（已修），守卫面 5 行现状绿，守卫缺口本身（同 URL
  已可用不重取）仍在 fetch 语义域。
- **新挂账：宿主树 documentElement 父插入物化缺口**（宿主 apply 域
  ——shim 树有宿主无，html 子树 script 节点无 forward map 条目/rect
  快照；本片 shim 侧判近臂为该缺口的机具侧补偿，宿主侧根修另立候选）。
- **保持**：SVG 纵横比-only 填充（naturalWidth 18F）、clone density、
  display:none rect 清零、sizes-auto 残 4F+1T（各归其域）。

## 下一步候选

1. **fetch 核冗余更新守卫**（候选①保持，engine fetch 语义域）。
2. **SVG 纵横比-only 填充**（naturalWidth 18F，rendering 域跨切）。
3. **宿主 documentElement 物化缺口根修**（新挂账，宿主 apply 域）。

## 证据

- `evidence/logs/s285-empty-src-solo.log`（empty-src 页 solo 2P）
- `evidence/logs/s285-imgfamily-d.log`（family B；A = s284-imgfamily-c）
- `evidence/logs/s285-fulltree-d.log`（全树 B；A = s284-fulltree-c）
- `evidence/logs/s285-video-family-run1.log` / `run2.log`（video 族
  归因两轮 solo）
- `evidence/logs/s285-make-test.log`（全量 make test，9088P/5F）
- 日志 md5：empty-src 4afa1fd6、imgfamily 24a2fe04、fulltree 3f1a704a、
  video-run1 a3dae59c、video-run2 45601a92、make-test 88f7b4a1
