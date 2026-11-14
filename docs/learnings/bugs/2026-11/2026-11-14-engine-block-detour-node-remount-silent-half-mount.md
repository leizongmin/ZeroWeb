---
date: 2026-11-14
modules: engine
---

# 经 engine 块两跳摘挂的节点回插宿主静默半挂——重挂一律新建节点

## 问题描述

insertParagraph B 族刀（拆点块界空白归一）钉测复现：宿主级 wrap 臂把新块尾端空白文本从 engine 块 `sep1` 摘除后用原节点 `host.insertBefore(node, sep1.nextSibling)` 回插宿主，`insertBefore` 正常返回、无异常，但节点既不出现在 `host.childNodes` 也不进序列化——尾空格凭空消失。同批把行首空白从 wrap 集合排除后**额外**新建文本节点重挂宿主，则产生双份空格（原节点其实一直挂得好好的）。

## 根因分析

两类现象同源于宿主桥 mixed plain/engine 挂载语义（DC-4 探针 Q6 系列已定谳过第一类）：

1. **静默半挂**：原节点解析自 plain 树 → plain `removeChild` 出宿主 → `appendChild` 进 engine 块（此步对 plain 子是序列化拷贝，引擎侧句柄与原节点脱钩）→ 再次摘挂后回插宿主。此时节点内部状态已被两跳搬运污染，`insertBefore` 设好父指针但 view/childNodes 不动——半挂节点对 childNodes、序列化、遍历全部不可见，且不抛错。
2. **重挂双份**：从未摘挂的节点（只是从"待搬运集合"里排除）本就挂在正确位置，任何"重建+回插"都是多余副本。摘挂与否决定语义，不能对两类节点统一套重挂。

## 解决方案

- 回插一律 `document.createTextNode(node.nodeValue)` 新建后插入——fresh 节点走正常物化路径（本仓先例：wrap 臂 `bNorm1w[0]/[1]` 文本、m3a 臂 `head1` 文本均实证物化）。值是纯字符串时无身份需求，新建等价。
- 只从集合排除、从未摘挂的节点**不重挂**——宿主原位即正确位（挂载点天然在其后）。
- 多节点回插注意序：`insertBefore(x, 固定后继)` 正序迭代保序；`insertBefore(x, sep.nextSibling)` 这类"跟随锚"反向迭代保序（头插正序会倒序）。
- 排障手段：直接 dump `host.childNodes` 逐子 nodeType/nodeValue 与 innerHTML 对比——半挂节点在 childNodes 缺席而序列化面"少一块"是签名。

DC-4 原始结论（part01.js `js_dom_bridge` 注释）只覆盖"engine 块挂载用 insertBefore 在 fully-plain 宿主静默半挂"与"raw appendChild 对在挂 plain 子是序列化拷贝"两形；本案例补第三形：**plain 子经 engine 块中转后回插宿主同样半挂**。通用纪律一句话：跨 engine/plain 边界搬运过的节点不再直接回挂，重建等价物。

## 验证

B 族刀钉测 19/19（CE-false 跨行键尾空格复现→修复后 exp0 逐字节等）、C4 钉 9/9、全 ip 钉面 46/46（含 offset-0 双钉；终态 commit 25f6a3943）；门禁批 ip/ab/mt/enginelib＋ilb 核验腿五腿全绿见 `.acceptance/dps/batch-ipm3-b-gate-v6.log`。
