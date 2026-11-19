# ③ teal `[a]bc` 谜团收口——M-wire handle 父 + 裸 text ref 落位缺口 append 退化修复

日期：2026-11-05　轮次：⑫ 后续（③ 残尾收口）　状态：fix14e 定稿

## 结论

`fontSize` 语料末行（`?2001-last`，swF，`<span style="font-size:32px; background-color:rgb(0, 128, 128)">[a]bc</span>` → fontSize 5）compare 红，为 ③ 残尾收口最后 1 行（上轮挂账「独立探针/语料序窗全绿而真管线红」谜团）。根因不在语料管线交互，而在 M-wire 落位分发的参考点寻址缺口：

**handle 父 + 裸 text ref（无 handle、无 selector 印）+ 元素 newNode 的 `insertBefore`，part04.js 落位分发全支落空**（:9616 append 支收 null ref；:9634 印支收 selector ref；:9653 R97/R101 支收 handle ref；:9678/:9700 R265 支收 textEl；:9756 R5058 收 TEXT newNode；:9770 D3 收 pos-sel 元素 ref）→ `newParent` 壳 M 侧静默缺失 → 后续 `_zwJusMovePreservingRanges` 把文本挪进未接线壳 → M 侧序列化同时丢失壳与文本。

实证矩阵（上轮 td3 SERIAL + 本轮复证）：

| 形 | ref 可寻址性 | 结果 |
| --- | --- | --- |
| `a[b]c` | splitText 产物 ref 带 handle → R97 支 | 过 |
| `[a]bc` | 解析原件裸 ref → 全支 miss | 失（壳+文本双丢） |
| 独立探针 FS-A（页首执行） | ref 走 sel-印 支 | 过 |
| 语料序窗 K=0/K=1/K=4（修后） | 同语料管线 | 过 |

## 修复（part01d.js `_zwVaForceValue` extwrap 臂）

裸 ref 且 handle 父时，`insertBefore(newParent, node)` 退化为 `insertBefore(newParent, null)`（refNode==null append 分支，零参考点要求恒接线）；文本运输仍走原 movePreserving（td3 SERIAL 证其 removeChild 恒接线；append 进已接线壳走 R265 textEl 恒接线）。append 与插 node 前同 JS 树，纯接线修复，位次与语义零变化。

## 门限演化（三次证伪，全记录）

1. **宽门**（任意裸 ref 一律 append）：teal 全绿，但 fontsize **47F**（2,416P/6F 基线 +41 新红）——`foo[bar…]baz`/`foo{…}`/关键词 span 41 例中列形，append 尾插位次漂移证伪。失败集复跑同（fix14/fix14b）。
2. **父 `__zwSelector` 门**：47F 同集且 teal 复红——克隆父与解析父代理面均不暴露 selector，判别无效。
3. **node 末位门**：41 复原（2,417P/5F）但 teal 全语料复红——K=1 窗行形状（单片段末位）≠ 全语料行上下文形状（多片段中列）。期间发现窗口判行误认：窗口签名命中 idx 164 的首匹配行，而真红行在 `?2001-last` 片——修正判行后以全语料 compare 行为准。
4. **handle 父门（定稿 fix14e）**：壳为 shim `createElement` 产物持 handle，解析父无 handle 走 sel 支 pos-sel 恒接线——门限精确切分两族。

## 门禁矩阵（fix14e 定稿树）

| 腿 | 结果 | 基线对照 |
| --- | --- | --- |
| fontsize 全语料 | **2,417P/5F**，teal compare **Pass**（HEAD 同行 Fail 对照 fix14c） | 2,416P/6F：**F2P 1**（teal）、P2F 0、newF 0 |
| fontsize 残 5 行 | 2×`<p>[foo<p><br><p>bar]`、1×table、2×`ba[r</font>b]az`（读数面族） | 与基线逐行恒 |
| zz-probe-carrier | 32P/0F | 恒（16 例含 J1-J5/G1-G5 守卫） |
| zz-bisect K=1 窗 | 0F | 修前红（12:29 旧账） |
| fontname | **2,222P/3F**（pre/listing 读数面族） | 3F 恒 |
| forecolor | **2,310P/17F**（嵌套色壳/purple/table 族） | 17F 恒（③' 四机制归因挂账） |
| hilitecolor | **1,218P/0F** | 0F 恒 |
| backcolor | **1,026P/0F** | 0F 恒 |
| enginelib（make guarded-test） | **3,731P/5F**，五红名（p56 adopted snapshot + 4×s249 CSSOM）与基线逐名恒一 | 3,727P/5F（P 增量来自并行轮新增用例，红名集不变） |

## 过程误差记录（防复发）

- **嵌套 flock 幻影日志**：外层 `flock` 持锁调用自 flock 的脚本 → 内层 `flock -n` 静默失败跳跑 → 复跑判读读到旧账（bisect-K1.log mtime 12:29 早于全部本轮编辑，两轮「复跑」实为幻影）。判读前必须核对目标日志 mtime 晚于本次变更时刻。
- **grep 计 Pass/Fail 同串**：teal 计数串 `32px.*a\]bc.*compare innerHTML` 同现于 Pass 行，`fails:1` 实为 Pass 行误计。失败计数一律 `grep -c '^Fail '` 前缀锚定。

## 挂账

- `part01d.js:571`（VA wrap sibCrit 臂）与 `part01c.js:988`（generic wrap）同形 `insertBefore(newParent, nodeList[0])` 位点：解析父 sel 支恒接线，现行语料无红证，不扩改；回归由语料门禁守护。
- 上轮控制面「documentElement.cloneNode detached 树 querySelector」疑点证否——JS 侧树全程正确、M 侧序列化单侧丢失，与克隆树查询无关。
