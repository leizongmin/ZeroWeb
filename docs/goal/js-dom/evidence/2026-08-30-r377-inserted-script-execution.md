# R377 — 插入期脚本执行（R328 遗留 shim 侧落点）+ 立项材料沉淀

**日期**: 2026-08-30
**切片**: R328 遗留「克隆 script 插入期执行」的 shim 侧收口（replaceWith/
insertAdjacent fragment 展开路径）+ 已知 Fail 深项立项材料沉淀
**改动面**: `js_dom_shim/part05.js`（`_insertAdjacentVariadic` script 执行钩子）

## 1. 落地件

**插入期脚本执行**（part05 `_insertAdjacentVariadic`）：script 节点（handle）
入树后收集其 registry 文本子，按 classic 脚本语义执行——全局作用域 `(0,eval)`；
异常按 report-the-exception 上报主 window；run-once 标记（`_zwRanScripts`）防
重插重跑（clone 产物是全新脚本节点，首插必跑）。覆盖两条插入路径：fragment
展开（R321 循环——template clone 的 script 子）与直接 script 项。

## 2. 立项材料沉淀（本轮评估结论）

**remove-next-sibling-during-replace-with 剩余缺口双件**（本轮探针逐层剥）：

1. **sel 域 fused innerHTML**——`container.innerHTML` 在 pending 桶非空时须
   从融合 childNodes 序列化（host 字符串是 apply 滞后旧树）。本轮实验实现后
   探针发现**双障碍**：① `_childNodeList` 融合视图不剔除 sel 移除标记子（
   `_zwRemovedSels` 在读时已空——replaceWith 流程中标记被中间环节清除，待查
   清除点）；② 子序列化依赖 outerHTML 对 pending 新子递归正确。两障碍均涉
   pending-apply 生命周期与移除标记的交互——非轻量切片，实验代码已回退（零
   残留），与 parse-time MO 一并转档 **pending-apply 生命周期专项**。
2. **已知 Fail 全域定性完成**（R373/R376 备档 + 本轮）：MutationObserver-
   document 3F（parse-time 架构）、remove-next-sibling（本专项）、
   remove-and-adopt-thcrash（window.open）、click-on-absolute-pseudo（不追）、
   ranges dataChange/replaceData 2F（游离树堆积域）、historical 3F（stale
   不追）、window-extends 2F（EventTarget 继承域）。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 语义面 | `container.querySelector('script')` 在 replaceWith 同 turn 可寻址（R371 rekey + 本轮执行钩子联合收口）；script 内容全局执行（`document.querySelector('b').remove()` 生效——探针实证 b 进 pending-removed） |
| 哨兵 | ChildNode- 123P / ParentNode- 2132P / Node-appendChild 11P / insertBefore 40P / replaceChild 58P 恒等（执行钩子零回归）；engine v8 2500 / quickjs 1475；integration 784P |
| 全量 dom sweep（polyfill，TIME_LIMIT=2400） | **55807P（+3）/真实 Fail 文件集恒等零新增**（含探针自抛文件已清理） |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

**过程教训**：实验性 fused innerHTML 在探针揭示「移除标记在读时已空」后按
「实验代码零残留」原则整体回退——负结果与清除点待查（标记清除链路）一并记
入 pending-apply 生命周期专项材料。

## 4. 后续

- **pending-apply 生命周期专项**（新立项材料）：fused innerHTML + 移除标记
  清除链路 + parse-time MO 三项同根（host apply 异步滞后与 JS 同步视图的生
  命周期边界）——材料已齐，待专项评审。
- 已知 Fail 集合计 6 原有 + historical 3F[stale 不追] + window-extends 2F[转
  档]，全域定性维持。
- 主线剩余：M5/M7 default-on（待用户点名，改 Mission 级单向门）；M3 已达成；
  M4 基线持续维护；M2 已收口；M8/DC-8 已收敛。
