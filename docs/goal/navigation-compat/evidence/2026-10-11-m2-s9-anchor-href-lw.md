# M2-S9 锚激活 href 动态写可见性（latest-wins 读）+ 复核轮组成态验证

**日期**: 2026-10-11
**通道**: `make testharness-navigation`（test-guard 包裹；探针页 FILTER 单案定因，用后即删）
**基线**: S8 [2026-10-10-m2-s8-vertical-blockstart-scrollx.txt](2026-10-10-m2-s8-vertical-blockstart-scrollx.txt)
（407 Pass 行 / 476 案 = 85.5%）

## 复核轮组成态验证（先行）

S8 落地后 main 已合入兄弟流提交（layout R5044-R5047、engine-dom t8r-4、perf 基线等）。
本轮先在**未含本轮变更**的 main 组成态复跑全量：407 Pass 行，per-subtest 精确 diff 与
S8 evidence **逐行恒等**（35 Fail + 27 Timeout + 14 NotRun）——平行流合入对本语料零漂移。

## 残余 76 案处置逐案核对（复核轮第二项）

对照 M4 收口定性（[2026-10-09-m4-closeout-assessment.md](2026-10-09-m4-closeout-assessment.md)）
与各切片 evidence，S8 态 76 非 Pass 全数有账：

- **B 类 runner 形态/挂账（53）**：replace-before-load 38（M3 iframe 载体）；bfcache 族 6
  （entries/dispose/activation ×2/history-back/order-in-bfcache-restore——P5 定稿挂账）；
  navigate-multiple-location/-pushState 2T（S4Q task 排队模型评估挂账）；
  replaceState-inside-back-handler-infinite.optional 1T（S4P 速率限制封顶后摆动，两态非 Pass）；
  intercept-popstate-no-handler 1F（S4T 同步结算模型互斥）；008 1T（P2 挂账）；
  location_port 1F（runner 无端口 URL 形态，S1 记账）；navigate-anchor-cross-origin 1T
  （**本轮收口，见下**）；reload-service-worker-fetch-event 1F（SW fetch 管线 + iframe +
  跨文档三重门控——本轮补记入账）。
- **A 域回流（23）**：DOM 焦点域 16（focus-reset basic/multiple-intercept 2T 需
  test_driver.send_keys TAB 真键注入；autofocus load 期 14 NotRun = 7 断言 × 2 variants
  （S4K 分母扩张后计数翻倍——M4 记 7 为 variants 前），setup 期
  `document.activeElement` 断言依赖 autofocus load 处理）；渲染域 6（scroll-behavior
  reload 族 4F scroll anchoring/rect 快照 + scroll-to-fragid vertical-rl/inline-nearest 2F）；
  the-iframe-element sandbox allow_top_navigation 2F（M3 iframe 门控）。

复核结论：**C 类可切片余量清零属实**；本轮唯一发现 = navigate-anchor-cross-origin 的
S4I 挂账定因有误（见下），其余挂账逐案核验维持。

## 本轮修复：navigate-anchor-cross-origin（Timeout→Pass）

### 定因（探针页逐值，probe-anchor-xorigin 用后即删）

S4I 挂账记「`a.href = v` IDL 赋值不落 attr」——**定因有误**。探针实测：

1. `a.href = v` **确实落 attr**（R3069 reflected 分支 → `__zw_set_attr` → pending
   `DomMutation::SetAttr` mutation）；`getAttribute`（latest-wins 通道）立即可见。
2. 但 `a.href` getter 返 `''`：part03 URL 分解分支（R2838/R5009）读 **纯快照**
   `__zw_get_attr`（defaultValue/aria 稳定读语义），同批 mutation apply 前不入快照 →
   round-trip 断。
3. click 激活链 `_zwAnchorActivate`（M2-S4H）的 attr 回落同样读纯快照 → href 读空 →
   navigate 缺发 → 全案 Timeout。download presence 路径 M2-S4Y 已用
   `__zw_has_attr_lw`，href 回落漏配。

即：**attr 持久化从未缺失，缺的是读侧 latest-wins**——回流 element IDL 域的挂账定性
（M4 A 域回流第 3 项「attr 持久化面回流 dom goal」）就此销项，缺口在本 goal envelope 内。

### 修（两点，part03.js）

| 变更 | 说明 | 先例 |
|---|---|---|
| URL 分解分支 A/AREA 原始 href 读 `__zw_get_attr` → `__zw_get_attr_lw`（typeof 守卫回落） | getter round-trip 恢复（真浏览器 `a.href = v; a.href` 即返解析绝对 URL）；BASE/LINK 子分支维持纯快照不动 | R3202 FORM 反射同款（`f.method='POST'; f.method` stale 同型） |
| `_zwAnchorActivate` attr 回落同改 latest-wins | 「脚本设 href 后同步 click」形态激活链 href 可见，navigate 正常派发 | M2-S4Y download presence `__zw_has_attr_lw` 同款 |

### 探针复跑（修后）

navigate 事件正常派发，逐值与上游断言吻合：`canIntercept=false`（跨源）/`type='push'`/
`sameDocument=false`/`destination.key=""`/`index=-1`/`destination.url === a.href`
（双修后 getter 解析一致）/`userInitiated=false`（click() 不签发瞬态激活）。

## 验证

- 单案 FILTER：`navigate-anchor-cross-origin.html` **Pass**（全部 9 项断言过）。
- 全量：408 Pass 行 / 476 案 = **85.7%**（原始 txt 见
  [2026-10-11-m2-s9-anchor-href-lw.txt](2026-10-11-m2-s9-anchor-href-lw.txt)）。
- **零回归（双臂）**：① navigation per-subtest 精确 diff vs S8 evidence——恰一行
  Timeout→Pass（探针行已剔除，probe 页未入证据 txt）；② html-syntax 全套件（61,303
  结果行）**pre-change 基线构建 vs post-change 逐行恒等**（0 diff）——getter 分支共享面
  双臂测量；61,218→61,217 对 M4 close 的 −1 为窗口内既有漂移（pre-change 基线同样 61,217），
  与本轮无关。套件状态行 61,217 P / 73 F / 13 T 与 M4 close 一致。
- 残余 75：35 Fail + 26 Timeout + 14 NotRun。

## 质量门禁

- `make test`：**20,404 P / 0 F**（part03.js 变更轮，EXIT=0）。归因记录：第 1/2 次全量
  各出现 1 例不同忙窗 flake（`page_scripts::finish_page_load_fires_body_onload_r2946` /
  `process_backend::indexed_db_owner_tests::cross_renderer_transactions_wait_for_conflicting_scope`
  ——均 multiprocess 时序敏感面、与本轮 anchor href 变更无涉；solo 单测 0.06s/0.41s 恒过、
  zero-renderer 全套件 solo 237P 恒过；S4I 同型先例；且 ZeroWeb-2 平行 clone 同机跑套件
  加剧负载窗）。第 3 次全量全绿实证。
- 纯 .js shim 变更无 .rs（fmt/clippy 不适用）。
