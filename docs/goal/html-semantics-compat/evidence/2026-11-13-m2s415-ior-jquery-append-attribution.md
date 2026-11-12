# M2 切片 415（round-1）— ior jQuery append() 新红归因：同源二进制翻转，源归因排除

日期：2026-11-13 凌晨（承 s414 下一步首位：ior 新红三跑归因）
commit：本账本随 docs commit（零代码变更轮）

## referent

`the-select-element/inserted-or-removed.html :: The last selected OPTION should win; Inserted by jQuery append()`——s414 勘根定谳的真新红（+1F）。失败签名 `assert_equals: expected "Second" but got "First"`：测试纯同步（s414 已核源码——模拟 jQuery 1.x/2.x DOM 移动：detached select innerHTML → fragment 逐个 appendChild → target.appendChild(fragment)，断言 last-wins selectedness），无 jQuery 库依赖、无异步面。该子测 s236（2026-11-02）前 1F 修复后 **11 天稳定 Pass**（s236–s413 数十轮 sweep 全 Pass），2026-11-12 23:12 s414 批起确定性 Fail。

## round-1 归因腿（/tmp/s415/，全部 test-guard 包裹 + RAF_FRAME_DRIVEN）

| 批次 | 树 | 二进制 | 条件 | 结果 |
|---|---|---|---|---|
| s413 批（11-12 20:04） | 0079443c6 | 当批构建 | 全域 sweep ×2 | **Pass ×2** |
| s414 批（11-12 23:12） | 0079443c6+注释 | 当批构建（freshness 门过） | sweep ×2 | Fail ×2 |
| flake3（11-12 23:20） | 同上 | 同上 | 负载窗（maketest 编译并发） | Fail ×3 |
| flake4（11-13 02:00） | bd41f9bd2 | 重建（freshness 门过） | 静机 load 10 | Fail ×3 |
| **attr（11-13 02:10，本轮）** | **纯 0079443c6**（detached checkout，零注释零上游） | 重建 BUILD_EXIT=0 | 静机 | **Fail ×3** |

## 定谳

**源归因排除**：纯 s413 源重建二进制 Fail ×3 vs 同源 s413 批二进制 Pass ×2——同源不同构建/不同时点翻转。s414 注释（已证 inert：纯注释 + attr 树无注释同态）与上游五 commit（attr 树不含）双双排除。翻转维度残留在**源码之外**：构建产物布局（rustc 并行 codegen 非逐位可复现 → codegen 单元划分/内联决策漂移 → 堆布局/分配序漂移）或运行时环境（同二进制跨时点翻转假设待检验——20:04 批二进制已被 23:12 重建覆盖，无法直接复跑）。

**勘根方向收窄（腿排程）**：
1. ~~同源连构建二进制 sha256 对比~~ **腿 1 已落锤（11-13 03:00，/tmp/s415/leg1-*）**：main 树（9204a0b31 基）同源 touch-rebuild 双构建 sha256 **A=af305840… / B=44a383b1… 不同**——rustc 输出非逐位可复现实证；双构建各 ior ×3 全 Fail（连前计 6 构建连 Fail：23:12 批 / attr / A / B——随机 regime 假设概率坍缩）。产物非确定 + 全近期构建同态 → 「20:04 Pass」 outlier 解释收窄为构建布局域（某 layout regime 触发/豁免 bug，20:04 构建落豁免侧）或机器时点状态。
2. **腿 2（首位）失败面探针插桩**：`_zwOptStateKeyOf` plain stamp 值 dump + `_mo_notify` per-point/container 臂相位记录（s236 状态键归一机制，s373 时序 dump 先例）——定位当前 Fail 的失灵臂（fragment 迁移后 selectedness 重算哪层断链）；探针直答「为何 Fail」，「为何 20:04 Pass」降为次级（修复优先于考古）。
3. 该子测断言对象为 fragment 双 option 迁移后的 selectedness 重算——与 s236 subtest 5 同根机制域（`_mo_notify` 钩挂位次/affKey 逐点臂），探针复用其插桩法。

## 口径影响

s414 账本在册红 +1F（213F+16T）不变；本切片零代码变更，纯勘根轮。ior 键在勘根收口前按在册红记账（11 天稳定 Pass 前科 + 跨源翻转实证 → 按 flake 治理不抹红、不入 favorable 单列）。

## 下一步

归因腿 2（探针插桩 stamp dump 双构建对比）为首位；腿 1 sha256 二分次之（单构建 ~15m 机械腿）。勘根收口后按根因域定施工/挂账（若坐实构建布局敏感 → 引擎状态键机制抗布局加固候选；若坐实运行时时序 → s236 钉面时序域勘根延续）。