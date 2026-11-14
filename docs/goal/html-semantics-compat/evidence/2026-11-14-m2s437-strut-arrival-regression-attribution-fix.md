# M2 s437 — strut 门到货位移检查揭 99 键 P→F 稳定回归 → s423 视图臂 img 面勘根修复轮

日期：2026-11-14。前情：s436 收口（window.open 评估轮，零源码改动）后按
惯例跑例常位移检查（s437 双跑 join vs s433 终稿基线），揭出 **99 键稳定
P→F**（98×naturalWidth-naturalHeight-width-height + 1×dialog
focus-after-close）。本切片勘根归因 + 修复 + 双跑 join 验证：**98
naturalWidth 键全数复正、零新位移**；dialog 面分诊为 phase-flip 非代码。
修复 = `pending_view_selector_rect` 对 IMG 选择器早退回落快照臂（= 父
提交 img 面行为），14 行。

## 发现（位移检查）

s436 收口时树 = main 88edca67f（干净），双跑
`make testharness-html-semantics`（无 FILTER，默认 5400s TIME_LIMIT，
实际 ~2.7 min/腿）join 对 s433 双跑终稿基线（/tmp/s433 键集）：

| 轮 | stable-P | stable-F | stable-T | 双轮沾染 |
|---|---|---|---|---|
| s433 基线（前轮） | 8309 | 202 | 3 | — |
| s437 位移检查（本轮树） | 8210 | 301 | 2 | 2 |

集合差：**P.gone = F.new = 98×naturalWidth…width-height.html + 1×dialog
focus-after-close.html**；T.gone = 1（149.html，提取工件）。98 键失败
签名同型：`assert_equals img.width/img.height`——AR 面 expected 720 got
720 / expected 240 got **100**；no-aspect 面 expected 300 got **720**。
视图错值形态：宽满填（视口宽 720）、高钉在 SVG 固有高（100）。

## bisect 归因（criterion 二值化）

判据脚本 bad iff 目标文件有 Fail 行（git-bad = 红面在案；crit.sh 对绿
solo rc=0 亦判 good——初版只放行 rc∈{1,2} 曾把绿腿误记 125-skip，已手
补边界腿校正）；`make testharness-html-semantics
FILTER=naturalWidth-naturalHeight-width-height` solo（make 门禁入口）。
范围 [s433 后续刀…HEAD]，腿序（工件 /tmp/s437/）：

| 腿 | 判据 | 结果 |
|---|---|---|
| **783b22267（s423 宽门 −1007 伪通过揭开 → 门控收窄纯 inline style 族保账本）** | 98F rc=2 | **first bad** |
| 330ba2b40（783b22267^，s423 施工前） | 0F rc=0 | last good |

**同窗口近邻刀排除**：997899265（空绝位 auto 高 strut 门，layout-engine
sizing.rs）到货早于回归揭出， solo + leg 双验与 98 键零因果（engine-half
反掏刀 `git diff 997899265^ 997899265 -- layout-engine | git apply -R`
→ 0F 维持 ⇒ strut 门对 naturalWidth 面无影响）。**臂级 kill 刀**：
callbacks.rs s423 selector 臂门置 `if false &&`（S437KNIFE）→ solo
**0F** ⇒ 错值源 = s423 臂的现算视图路径，非快照回落臂。

**假设证伪记录**：初始假设「门控收窄把 pending 非 style 族关到门外→
回落陈旧快照」——S437PROBE 在门关闭分支插桩零命中行（98F 重现下
probe-run.log 无 probe 输出）⇒ 该分支在语料写形下不可达，真机制在
门**开**路径（语料写形确为纯 inline style 族，门恒开）。

## 根因（机制链）

门开路径 `pending_view_selector_rect`（computed_style_cache.rs，783b22267
新增）对基座既有元素以「解析 base HTML + 重放 pending + cascade +
intrinsic overrides + `LayoutEngine::compute`」**现算**几何。img 布局
尺寸携带**非 DOM 资源态**：解码固有尺寸由渲染泵注入
`Pipeline.image_sizes`（[`crate::pipeline::Pipeline::build_img_intrinsic_all`]，
pipeline/mod.rs），经 `compute_with_img_intrinsic` 落布局。视图 re-parse
只有 DOM+CSS：

- 视图 intrinsic override 面唯一入口 `apply_intrinsic_size_overrides`
  （computed_style.rs）仅 canvas/video 属性分支、**无 img 分支**——
  代码自证注释「解码后真值不走此路径（computed-style 路径无
  img_intrinsic 解码尺寸源）」。
- 无 width/height 属性的 img（SVG data: URL 语料形态）在视图布局退化
  成占位满填 720×视口宽、高钉 SVG 固有高 100——而 **rect 快照臂在 img
  已加载后即渲染后真值**（300/240，生产渲染 pass 已跑）。

即：783b22267 的现算臂对 img 面是「用更少状态重算出一个更错的值」，
置换了本来正确的快照读。

### 修复（14 行，pending_view_selector_rect 内）

selector 解析（`find_by_selector`）后早退：

```rust
if matches!(
    doc.get(nid).map(|n| &n.kind),
    Some(zero_dom::NodeKind::Element(e)) if e.local_name() == "img"
) {
    return None;
}
```

返 None → 调用方（callbacks.rs s423 臂）miss 回落快照臂 = **父提交
img 面行为逐字节恢复**；非 img 选择器零行为变化（早退仅 img 命中）。
`FIXME(replaced-elements)` 注记 object/embed 同源缺口未排除（s423 臂
锚点查询面不含 img 之外替换元素，语料未见回归）。

**弃选方案（Shape A）记账**：把 `Pipeline.image_sizes` 接进视图臂
（视图侧补 img intrinsic override 分支）为根修——但桥接调用点只有
`rs_html`/`rs_mut` 锁、无 Pipeline 句柄，跨 crate 铺管道需改调用链
签名面，超本刀范围且收益为零（快照已是渲染后真值，视图重算不劣于
快照读）；留作 runner/管道侧后续能力项，不入本轮。

### dialog 面分诊（非代码，phase-flip）

`focus-after-close.html :: Focus should be moved to the previously
focused element (Complex dialog usage)`：14:29 双跑（s436 收口轮）
**绿**、今日所有跑**红**（含 25fdd6e45 父提交 solo×3）⇒ 同树跨时窗
状态翻转 = 时序 flake，与 783b22267 零因果（bad 腿 bisect 时窗内该面
独立于目标 98 键存在）。分诊注记待下轮位移检查复验翻回；不入修复面。
detailed.optional（T 沾染）= 在册振荡；149.html（for='  WINdow 沾染）
= 提取工件（在册）。

## 验证

| 检查 | 结果 |
|---|---|
| solo naturalWidth-naturalHeight-width-height | **258P/0F/0T exit 0**（修复前 98F rc=2） |
| 双跑 join vs s433 基线 | fix stable-P **8308**/F 203/T 2 vs base 8309/202/3；naturalWidth 键 fix.sP ×258、fix.sF/fix.sT ×0（98 键全数回位）；集合差仅三个已分类非代码面（dialog phase-flip、detailed.optional、149.html） |
| `cargo fmt --all -- --check` | 绿 |
| `make guarded-clippy`（workspace，-D warnings） | exit 0 绿 |
| `make test`（workspace 全量） | **160 腿 10761P/1F/66I**；唯一 F =
  `image_batch_e2e::renderer_publish_batch_image_fetch_reaches_concurrent_
  in_flight`——zero-browser bin 被 target-disk-guard 清理（panic 于
  artifact 存在性检查 image_batch_e2e.rs:233，`make test` 自身 build 不含
  zero-browser；s116 在册形态，环境副作用非行为失败，与 engine 改动无
  因果）；clippy 腿绿（零 warning，两 profile Finished） |

工件：/tmp/s437/{sweep-1,sweep-2,fix-sweep-1,fix-sweep-2}.log、
fix-sweep-*.exit（rc=2；内容完整性以 8515 键双腿逐一对账，rc 偏移
在册）、crit.sh、leg.sh、{leg-bad783,leg-good330,ab-engine-half,
ab-gate-kill,probe-run,fix-solo-naturalwidth}.log、fix-join.sh。

## 跨流记账（run-rules #9/#10）

- 783b22267 = **渲染流 g36 提交**（s423 pending selector rect 管线，
  触 shared engine face）。他流 corpus（reftest 16594）A/B 零真回归
  ——他们控制面看不到本流 testharness 语料，两 corpus 键集不交叠；
  本流侧 99 键伪红由本刀清偿，渲染流侧无需返修。
- 本修复改动面 = 783b22267 新增函数内一个 img 早退分支：锚点/位置面
  查询非 img 元素，构造性不受影响；快照臂行为 = 父提交，双 corpus
  （reftest + testharness）均回到 783b22267 前等值形态。
- 方法论沉淀：**视图现算路径等值性前提**——资源态（解码尺寸/解码帧）
  不入 DOM/CSS 视图，替换元素几何在视图臂现算须显式排除或接解码源；
  与 s423 门控收窄（CSSOM 等值性）同一判据家族的资源态补全。

## 结论

s437 收口：strut 门到货轮位移检查揭出的 99 键回归 = 1 代码面（98
naturalWidth，783b22267 视图臂 img 缺资源态，本刀修复）+ 1 phase-flip
（dialog，非代码，待复验）。修复后全域键面回到 s433 基线等值带（差集
仅已分类非代码面）。挂账：replaced-elements 视图臂解码源（FIXME 在码）；
dialog phase-flip 下轮复验。
