# R254 Evidence — 幽灵烘焙点定位（clone 循环中间态）+ 修复 land（13/14,x 全解 +4P）

**日期**: 2026-08-25
**切片**: M4——R254(a) 烘焙点重定位（gen 代际探针）→ 根因锁定 → 修复 land
**基线**: surround 1806P/34F；修复后 1810P/30F（净 +4，set-diff 0 回归）

## 一、gen 代际探针（R253 遗留方法）

在 R254-probe（13,0 场景）加 `doc._zwQWrapBump` getter 读数（= `_zwQWrapGen` 代际）
+ `_zwNodeIdx` 状态 dump + 幽灵/上移对象深展开：

- `GEN-pre g0 → GEN-post g0`：surround 全过程 **gen 从未 bump**——
  `_zwQWrapCache` 未失效，但同样**没有中途烘焙**（idx pre/post 均 null）。
  R253 的「缓存失效窗口」候选被排除：幽灵不在查询缓存域。
- 幽灵深 dump（GHOST{}）决定性证据：
  - `outer=<p id="a"><head><title></title></head></p>`（isUP=false isNP=false，
    既非上移本尊也非 newParent 本身，是**第三个对象**）
  - 上移本尊（UPLIFT{}，isNP=true）kids=[HEAD,BODY] 正确，其 BODY 子树内
    div#test 首子即幽灵。
- **根因**：surround 主路径的 clone 循环（R2930 正序 clone → 逆序 remove）在
  克隆 covered children（docEl 的 [HEAD, BODY]）时，**newParent（paras[0]）仍挂在
  BODY>div#test 内**。`kids[i].cloneNode(true)` 逐子克隆：克隆 HEAD 后 append 进
  newParent；克隆 BODY 时，BODY 深克隆把「此刻的 div#test（仍含 newParent——
  其内容是半完成中间态：只含 HEAD-clone）」一并复制。newParent 的克隆中间态
  从此烘进 BODY-clone 内的 div#test——即 R250 起追踪的「幽灵 P#a{HEAD-only}」。
  sim（common.js mySurroundContents）无此问题：步骤 3 extract 先移出原件。

## 二、修复（land）

`part06.js` surroundContents 主路径：clone 循环**前**先 `newParent.remove()`
（R248 的 insert 前摘除提前；幂等——已 detached 时 no-op）。spec 序等效性：
surround 步骤 3 extract 移出原件时 newParent 若在覆盖子树内也已被移出。

- probe 复核：G4 A=div[P#b,…] 与 E 侧完全一致（幽灵消失）。
- 主文档 proxy 域的 remove 同步语义（`__zw_remove` host 回调异步批处理）
  是**另一个**已记未解决问题（本切片首轮单测实证 npPn 未即时置 null），
  WPT 13/14,0 走 iframe 工厂域不受影响。

## 三、A/B 验证

| 面 | 基线 | 修复后 | set-diff |
|----|------|--------|----------|
| Range-surroundContents（polyfill） | 1806P/34F | 1810P/30F | +4 / 0 回归 |
| Range-surroundContents（native, ZW_NATIVE_DOM=1） | — | 1810P/30F（13,0/14,0 同翻绿） | 双路径对等 |
| ranges 全量（Range-*） | 38676P/1401F | 38680P/1397F | +4 / 0 / 0 |

翻绿用例：13,0 + 14,0 的 resulting DOM + resulting range position（R246 起
追踪的 docEl 容器覆盖形态四连）。

## 四、附带修复（遗留红灯，CLAUDE.md「不允许留给下一轮」）

`test_surround_invalid_state_and_step_order_r210`（part21.rs）在 clean main
即失败——bisect 定位 **R239**（partial-check 改 nextNode 序 + 先于 nodeType
检查）破坏了其场景②（跨容器 range + Document newParent：现按 R239 序先抛
InvalidStateError，r210 首版期望 InvalidNodeTypeError 已过时）。修正：场景②
改用无部分包含的 range（`[testDiv,0,testDiv,1]`）验证 nodeType 校验。

## 五、测试与门禁

- 新增单测 `r254_surround_clone_detaches_newparent_before_deepclone`（part23.rs，
  iframe 工厂域，五断言：covered kids / 摘除即时性 / 上移首子 / 无幽灵 / wrap 克隆空壳）
- engine 2394 全绿（2393 + R254 新增）；`cargo fmt --check` 干净；
  `cargo clippy -p zero-engine --all-targets -D warnings` 干净；
  workspace clippy 干净
- 全部经 `make testharness-dom`（test-guard 包裹）

## 六、R255 靶点

- 16,x startOffset 11F（`[document.body,4,document.body,5]` harness-iframe
  index 算术）
- 17,x 12F（foreignDoc docEl 容器 + 元素 newParent 族）
- 18/19,x 4F（`[paras[0],0,paras[0],1]` self-surround / detachedPara1 同形态）
- 28,x 1F / 30,x 2F
