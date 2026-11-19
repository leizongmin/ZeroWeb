# editing-compat — R5341 缺陷 #2 修复（runner 完成接线 load 门控）+ 三例 Timeout delete 族同根确认（2026-10-30）

承接 [M2 次轮自旋簇归因](2026-10-30-m2-r2-spin-cluster-attribution.md)：簇 C（缺陷 #2）本轮修复
（R5341，tests/wpt-runner/src/testharness.rs）；预算窗挂账的「other 面 list/inline delete
自旋案是否同根因」以 delete 族旗舰案（三例 Timeout 之 ①）新二进制复跑收口。

注意：本轮 runner 二进制含两笔引擎/runner 变更——R5340（part04.js removeChild 分支，
rebase 折叠后随 f94c5a914 入 main）+ R5341（本笔）；对照基线（预算窗 2,404P/11,159F/3T，
a27566987 落档）为双落前二进制，账面差异需按三分类归位（见 §3）。

## 1. 三例 Timeout 之 ①：delete 族同根确认（R5340 解锁实证）

`delete-without-unwrapping-first-line-of-child-block`（预算窗 600s 放宽探针定性的
非预算族——300s 预算 ~3min 吃满 4G scope OOM、timeout-only 零断言行）在 R5340 后
二进制单案复跑（`make testharness-editing-other-one FILTER=delete-without-unwrapping
TIME_LIMIT=900`）：

- **200 Fail / 0 Timeout**，全部为 forwardDeleteCommand/deleteCommand 变体的真实
  语义断言（assert_in_array / assert_equals，例：`at <b>abc\n[]\n</b><div id="child">…`
  的 delete-only-preceding-empty-line 断言），exit=1 正常带 Fail。
- 结论：**同根因**——delete 标记循环卡死（removeChild pending 解析子静默穿透，
  R5340 已修）不解除则该案永无断言行；解锁后语义面 Fail 属 execCommand/delete
  语义簇（后续轮次目标，非本环）。
- 预算窗三例 Timeout 记账更新：① 同根已解（本节）；③ white-space join 有限慢
  （预算窗已定性，随预算解锁；本轮实为 other 面平墙过紧的外部截断假象，
  R5342 修复，见 §3.1）；② inserttext-last-mutation 同为 removeChild 自旋根因
  ——R5340 后全量复跑出 64 行真实断言 0 Timeout（§3.1），OOM 簇（DOM 无界
  增长）为自旋的下游表现，自旋解除即收敛；残余语义 Fail 归 execCommand/
  delete 语义簇序列。三例 Timeout 全部收口。

## 2. R5341：缺陷 #2 修复（runner 完成接线过早置 all_loaded）

### 机制回顾（归因全文见上轮 evidence 簇 C）

静态路径生命周期脚本尾同步直调 `__zw_mark_harness_loaded()`，早于同脚本内三任务
setTimeout 链的 load 派发 → `test_environment.all_loaded=true` 抢在全部 load handler
之前 → load handler 内同步 `test()`（testharness 声明即同步执行）首测 done 时
all_done() 四条件全真 → 提前 complete() → 同 handler 及后续 listener 的声明被
`Tests.prototype.push`（phase===COMPLETE 守卫）静默丢弃。真实浏览器原生时序：
vendored testharness.js 自身 load listener（93-100 行）在 load 事件派发完成的
setTimeout(0) 里才置 all_loaded。

### 修复（testharness.rs 三处 + 回归钉）

1. **静态路径尾改 load 门控**：生命周期脚本尾钩子由同步直调改为
   `addEventListener('load', function () { setTimeout(function () {
   __zw_mark_harness_loaded(); }, 0); })`——逐字镜像原生 listener 时序（load 派发
   完成含全部 load handler 后才置 all_loaded）。尾段提为
   `HARNESS_LOADED_STATIC_TAIL` 常量（巨字面量以 `to_string() +` 拼接，正文逐位
   不动）。
2. **write 注入路径维持原状**：fetch 服务源 eval 尾直调
   (`HARNESS_LOADED_TAIL_CALL`) 不动——document.open 后 load 事件不重发（s98 已
   核），挂 load 门控会使 all_loaded 永不置位（270896e25 的 scripting-1 102 伪
   Timeout 打回）。两路径不对称性在两常量 doc 注释写明。
3. **resident 回归钉两枚**（bin 单测，随 make test 常驻）：
   - `load_handler_sync_tests_survive_completion_wiring_r5341`：行为钉——vendored
     真 harness + load handler 内同步双 `test()` 页，断言双结果全 Pass（修复前本
     页只报 M1-a）。修复机制在真 Tests/all_done/complete 交互里，MINI_HARNESS
     无 all_loaded 面不可替代。
   - `harness_loaded_tails_pin_defect2_load_gate`：形态钉——静态尾必须 load 门控
     且不得直调、write 注入尾必须直调且不得挂 listener（双向防漂移）。
- fmt/clippy（`-p zero-wpt-runner --all-targets -D warnings`）过。

### 验收（探针双 Pass）

归档探针
[evidence/2026-10-30-load-handler-sync-test-probe.html](2026-10-30-load-handler-sync-test-probe.html)
（load+focus+2 trivial tests）经真实 editing 通道复跑（先临时置
wpt-data/editing/other/zz-defect2-probe.html 滤 zz-defect2-probe 单案，验毕即删，
语料盘点回 96 案）：

```
Pass editing/other/zz-defect2-probe.html :: M1-a sync first
Pass editing/other/zz-defect2-probe.html :: M1-b sync second
```

修复前同形态只报 M1-a（上轮 evidence 簇 C 归因记录）——双 Pass 即验收判据达成。

## 3. 新二进制对照面（本节账面随跑随记）

### 3.1 editing other 面全量复跑（R5340+R5341 双落二进制）

首跑（`make testharness-editing-other TIME_LIMIT=7200`，transcript
/tmp/editing-other-corpus.log，快照
/tmp/editing-other-corpus.r5341-fullrun.log）：86 有效案，账面
**2,406P / 4,790F / 3T**，对照预算窗基线 2,404P/11,159F/3T。逐案折叠变体
query 后 diff（基线 2026-10-30-run-budget-unlock-rerun-other.json）：
**差异恰为 10 案**，分三类——

1. **R5340 解锁（+1,144F，三例 Timeout 中的三案全部翻真实断言行）**：
   delete-without-unwrapping-first-line-of-child-block T1→F200（与 §1 单案
   探针 200F/0T 逐位一致）、insertparagraph-in-inline-editing-host.tentative
   T1→F880、inserttext-last-mutation T1→F64（② OOM 簇在 R5340 后出 64 行
   真实断言——OOM 根因同为 removeChild 自旋的 DOM 无界增长，自旋解除后
   循环收敛；§1 记其「仍开放」按本账修正为同根已解，残余语义 Fail 归
   execCommand/delete 语义簇）。
2. **R5341 缺陷 #2 修复的分母回归（+5F/+2P，load handler 声明形态四案）**：
   selectall-without-focus P1→P3、selectall-in-editinghost F1→F4、
   inline-editing-host-has-placeholder-with-before/after-pseudo-element
   F1→F2 各 +1——handler 内同步 test() 子集不再被提前 complete 丢弃，
   与探针双 Pass 同机制。
3. **墙钟记账假象（-7,518F +3T，join-different-white-space-style 三案）**：
   见下——非回归，corpus 脚本 other 面平墙过紧（R5342 修复）。

**三案墙钟归因（同树 A/B，归因纪律 #35）**：全量复跑中三案
`external wall-clock kill (120s, rc=137)` 且零行落盘。同树双二进制单案
（FILTER=join-different-white-space-style-paragraphs，60 变体）实测：

| 二进制 | 结果 | 耗时 |
|---|---|---|
| R5340-only（stash R5341 后重建） | 3,456 行全出（0P/3,456F/0T，逐行对齐基线） | 2m47.5s（TIME_LIMIT=1800） |
| R5340+R5341（工作树） | 3,456 行全出（同上，逐行对齐基线） | ~2-2.5min（TIME_LIMIT=600） |

即：**该案单案耗时 ~150-170s 在两二进制上同值**，本就超 120s 平墙——首跑
三案 Timeout 是 corpus 脚本 other 面平墙（120s，自适应公式仅接线 run 通道）
的外部截断，叠加 runner 按案缓冲 → 截断时零行落盘 → Timeout 记账假象。
**R5341 就三案无责**（双二进制同耗时同结果）；预算窗基线（pre-R5340 时代）
能 <120s 完成属机器负载差异，不再追（双现役二进制已同值，低于归因阈值）。

**R5342**（corpus 脚本，本笔随落）：other 面接线同形自适应墙
`60 + nv×15 + 90`（other 面每变体页 CASE_TIMEOUT=10s 上限 → 15s/变体系数；
nv=0 下限 150s 对齐 run 面已记的 150s 下限形）；run 通道分支逐字不动
（预算窗挂账的 420 下限重写属该流计划，不越界）。显式 CASE_WALL_S 仍优先。

R5342 后全量复跑（transcript /tmp/editing-other-corpus-r5342.log）终账：
86 有效案，**2,406P / 12,308F / 0T / 0U**——Timeout 清零，join 三案全量
入账（2607/1455/3456 F 逐位对齐基线）。总账逐位核对：F 12,308 = 基线
11,159 + R5340 解锁 1,144 + R5341 分母回归 5；P 2,406 = 基线 2,404 +
R5341 分母回归 2——分母完整、无墙钟假象、无语义反转。二进制时点：
f94c5a914 + R5341 + R5342（rebase 到并行流 886ef12bf 落后账面，时点注记
按本节）。

### 3.2 scripting-1 对照（270896e25 面）

`make testharness-html-semantics FILTER=scripting-1`：R5341 二进制
216P/176F/39T；R5340-only 二进制（stash 后重建）同窗复跑
216P/176F/39T——**verdict 流逐行 diff 为空（双 md5
02d537215242a999533a9ad4f871de37）**，R5341 对 scripting-1 零 verdict 级
影响，270896e25 的 102 伪 Timeout 修复不回退。

- **102.html Fail 为既有**：两二进制同报
  `Fail .../execution-timing/102.html :: scheduler: defer script after
  initial onload event`——先于 R5341 存在，属 94eb9f0d2（并行流共享面
  part03/webview/testharness）时代或语料 pin 漂移（s98 终账 315 案集 vs
  本轮 242 案集不可直接比）——按归因纪律记档不越界修，归并行流核对。
- R5341 唯一可见形迹：css-module/charset.html Timeout 行内 state dump
  `loaded:false`（R5340-only 为 `loaded:true`）——同 verdict 纯诊断差异。
  机制：该案 module script import `.py` 服务的 css module 永不 resolve →
  module script 挂起 → load 事件不派发（三任务链 100ms 重试持续）→
  load 门控的 mark 不触发。该案测试本就不报账（Timeout 两态同形），
  语义不变；此形迹提示 load 永不派发的页面在 R5341 后 all_loaded 恒
  false——与 §3.1 三案墙钟归因合并看，other 面无此形态案（10 案 diff
  已穷尽），暂不另立挂账，若后续通道出现同形 Timeout 再评估 readyState
  兜底门。

### 3.3 make test

`make test`（R5341 全部修改在场）：**3601 passed / 9 failed**——9 红与
M2 首轮记录的 fetch/Request/Response/ServiceWorker 族完全同集（HEAD 既有，
留并行流域归因）；本轮新增两枚回归钉
（`load_handler_sync_tests_survive_completion_wiring_r5341` 行为钉 +
`harness_loaded_tails_pin_defect2_load_gate` 形态钉）全绿。fmt/clippy
（`-p zero-wpt-runner --all-targets -D warnings`）过。

## 4. 影响面与挂账

- 影响面：全部静态路径 testharness 通道的完成时机后移至 load+setTimeout(0)——
  与真实浏览器一致；端点语义不变（显式 done()/complete() 页不受 all_loaded 门
  约束）。load 被 100ms 重试链延迟的页（慢图/慢 stylesheet，s157 面）完成随
  load 后移——与上游 wptrunner 同形。
- html-syntax 99.90% 收口态基线：完成时机后移理论上不影响端点账（该面无已知
  load-handler 同步 test() 形态），但 94eb9f0d2（并行流 testharness.rs 触碰）+
  R5341 两笔新二进制的全量抽查义务按预算窗时点注记累计挂账，本轮先以 scripting-1
  对照覆盖。
- 复验入口：`cargo test -p zero-wpt-runner --bin zero-wpt-runner
  load_handler_sync`（行为钉）；探针文件经 editing 通道直跑比对双 Pass；
  `make testharness-html-semantics FILTER=scripting-1` 对照 s98 终账。
