# d43：B4-S1 元素锚/跨容器 range select-delete 主臂落树

日期：2026-11-16。前置：d39 B3 诊断轮定谳 beforeinput 计数 908F 主簇 = 元素锚
range 选区 silent return（`__zw_ce_delete_impl` sc 非 text 直接 return，树零变，
≈820F）；master.md 下一步①即本刀。上一 session 在本刀编码中途被 429 使用上限
中断，本 session 接续完成收尾（复核遗留 diff、删除未接线的调试函数、终树重验）。

## 落码（part01.js，+359/−5，唯一产品变更）

1. **`_zwCeSelectDelete` 主臂**（新 ~300 行）：非塌缩跨容器 / 元素锚 range 的
   Backspace/Delete 语义。
   - 摘除按 Range.deleteContents 爬升形（dom.spec §dom-range-deletecontents）：
     文本端点部分覆盖截断 → 同容器元素 splice（块子摘内容保壳）→ 元素端点容器
     全覆盖子整摘 → 分支爬升摘兄弟 → lca 层中段摘除。分支爬升的兄弟遍历先快照
     成数组再逐个摘（活链 `nextSibling` 在 removeChild 寻址 miss 时不前进 =
     悬挂死点，deleting-in-list-items 90s 执行超时实证）。
   - 合并/占位按 Blink 行为（deleting-in-list-items / joining-dl /
     non-collapsed 语料定谳）：跨块且左侧有残余 → 右块首行并入左块（复用 join
     壳机制）+ 摘空右侧祖先块剪枝；左侧无残余 → 摘空左块移除；同块摘空 →
     `<br>` 占位落最深摘空块壳。
   - getTargetRanges = 边界规范化端点（`_zwCeCanonGtrStart/End`）：起点向后走入
     child[so] 首文本、终点向前走入 child[eo−1] 末文本，无文本可走保持元素锚。
   - LCA 预检：端点在宿主域外/断链 → 拒收回落章程 silent return（不派事件）。
   - caret 收尾 `_sdFinish`：重建塌缩 range 写回 selection + 派 input。
2. **beforeinput 派发段拆分**（`_zwCeJoinDispatch` / `_zwCeJoinMutate`）：
   `_zwCeJoinApply` = Dispatch + Mutate（既有调用方行为不变）；select-delete
   主臂自派 beforeinput 后直呼 Mutate——摘除与 join 同在单次 beforeinput 内
   承载，不双派事件（语料 bi=1 断言面）。
3. **Backspace/Delete 入口路由**（两处对称）：非塌缩且（sc 非文本 ∨ ec≠sc）
   交 ⑤ 主臂；④ 空界形（sc 文本尾 + ec 偏移 0）保持上游 join 臂口径；但
   sc 容器嵌于 ec 容器内的镜像形（`<div><p>abc[</p>]def</div>`）排除出空界形
   交主臂（该方向上游 join 臂无口径，基线 bi=2 假绿对照、探针实证 bi=1 +
   `<p>abcdef</p>` 归一）。核拒收回落章程 silent return（元素锚 caret 不回退）。

FIXME 挂账（语料残余面）：左块尾随 `<br>` 消费（join 后 `list-item1<br>-item2`
vs 期望无 br，与 caret join 臂同域）与嵌套列表/多行右块摘除未实现，挂 B4 后续刀
（diff 注释 FIXME(nested-list-deep) 等同域标记）。

本 session 收尾变更：删除上一 session 遗留的未接线调试函数 `_zwDbgNode`
（定义无调用点），终树重建 runner 重跑全部对拍（见下）。

## 对拍（全语料，zz-probe 排除口径；基线 = d42 终态二进制）

| 面 | d42 基线 | d43 终树 | 判定 |
|---|---|---|---|
| input-events 全量（30 文件） | 461P/2867F/3T | **715P/2613F/3T** | **净 +254，F2P=254/P2F=0** ✓ |
| html-semantics（950 文件哨兵） | P 5378 | P 5378 | P 恒等 ✓（F/T 漂移仅 select-home-end 已知 flaky 带互翻） |
| selection 全语料（738 文件哨兵） | 48,969P/4,340F/9T | **48,979P**/4,352F/9T | F2P=+10，真 P2F=0 ✓ |

input-events 逐文件 F2P（全部落在 B4-S1 目标簇，零面回退）：

| 文件 | P |
|---|---|
| joining-dl-elements?Delete | 36→132（+96） |
| joining-dl-elements?Backspace | 36→72（+36） |
| joining-dl-element-and-another-list?Delete | 20→44（+24） |
| deleting-in-list-items?Delete{ol,ul} | 13→36（+23×2） |
| non-collapsed-selection?{Backspace,Delete} | 8→20（+12×2） |
| deleting-in-list-items?Backspace{ol,ul} | 12→23（+11×2） |
| backspace / forwarddelete 主文件 | 38→41 / 48→51（+3/+3） |

selection 增量（d42→d43b 逐文件）：

- F2P +2×5：`select-all-and-delete-in-html-element-having-contenteditable`
  （selectAllChildren + Backspace/Delete 走主臂）+ typing-around-link 四变体
  各 +2（「deleting text before middle of a link」形）。
- +22 笔名义新增 Fail：`delete-without-unwrapping-first-line-of-child-block`
  四变体（+4/+6/+4/+8）——基线页级中断未上报的子测在 d43 跑到并红于 gtr 断言，
  两侧同红域、Pass 0 变化（d37 selection 对拍已记档同形 artifact，非回归）。
- `stringifier_editable_element` Timeout 位置子测级↔页级互翻（案级 9T 恒等）。

## 终树重验链（429 中断收尾）

上一 session 的 runner（/tmp/zw-runner-d43，10:52 构建）晚于 part01.js 末次
编辑（10:50:54），三份全量账（d43-final/d43-sem/d43-sel）即该树产物。本 session
删除死函数后重建 `/tmp/zw-runner-d43b` 重跑：

- input-events 全量重跑与 d43-final 逐子测恒等，唯一差异 =
  joining-dl-elements?Backspace 本轮页级 Timeout（预算边缘 flaky）——该变体
  单独复跑 72P/368F 与记录恒等；其余 29 文件逐子测零差异。
- html-semantics、selection 两哨兵用 d43b 终树二进制重跑（数字见上表）。

## 守卫与门禁

- 语料：`./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit
  {1800,1200,2700} -- <runner> testharness-{input-events,html-semantics,selection} --json`。
- engine lib（终树）：`cargo test -p zero-engine --lib`（dev-guard 24G scope +
  test-guard 包裹）**4059 passed / 0 failed**（较 d37 记录 4033 增 26 = 并行流
  新落钉测，零失败）。
- fmt：`cargo fmt --all -- --check` 有一处 diff，位于 `crates/dom/src/tests/
  tests_9_query_coverage.rs`（main 既有，来自并行流 commit 3eddbadbb，非本轮
  变更、不在本轮写面，未触碰）；本轮 diff 仅 part01.js（JS，fmt/clippy 不消费），
  `git diff --check` 干净。clippy 未重跑：本轮零 Rust 变更，engine lib 测试
  编译已覆盖构建面。

## 遗留（持位）

- B4-S1 语料残余：左块尾随 `<br>` 消费、嵌套列表/多行右块摘除（主臂 FIXME），
  forwarddelete 镜像已同核覆盖但深锚形仍保守拒收（silent return）。
- delete-without-unwrapping 四变体的 gtr 期望（跨 child-block 选区端点规范化）
  未对齐——+22 名义红面为后续刀的直接标的。
- 修复合并/占位行为面无新增常驻 reftest：本刀行为断言由上游语料常驻承载
  （deleting-in-list-items / joining-dl / non-collapsed 三族 32 变体全量入
  input-events 对拍哨兵），与 d37 以来各刀同口径。
