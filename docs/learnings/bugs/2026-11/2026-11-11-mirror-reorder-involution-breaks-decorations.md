---
date: 2026-11-11
modules: layout-engine
---

# 镜像类重排的复合安全性：装饰感知 gap 重分配破坏对合，恒等面须语义级短路

## 问题描述

R5501 给 `direction:rtl` 页面激活段落级 UBA 行级序列重排后，corpus A/B 红灯 12 案（bidi-box-model-014/015/023/024/029/030/038/039、text-align-justify-001/justifyall-001/002/006）：RTL div 内纯 ASCII 文本「First <span>Second</span>」（span 带 margin-left: 2em）被渲染成视觉序反转＋行几何错位 120px，而 UAX #9 L2 语义下 RTL 段基纯强 L 内容全部解析为偶 level，视觉序应恒等于逻辑序（chromium 中 RTL div 与 LTR div 逐像素一致——这正是 bidi-box-model 测试族的断言面）。

## 根因分析

行级重排 `reorder_lines_l2_for_para_bidi` 的降层循环（l 自 line_max 降到 1，每层对「全部片段 ≥l」的极大连续 run 调 `mirror_reverse_fragment_xs`）对**全偶行**（如全部 level 2）会执行 l=2、l=1 两次镜像，依赖「两次应用＝恒等」的对合性还原原几何。但 `mirror_reverse_fragment_xs` 是**装饰感知的空间重排**而非纯序列交换：相邻净 gap 按当前几何现算且负值钳 0、装饰（margin/padding）随属主片段在新游标位重排——第一次镜像把 span 的 40px margin 和 20px 词间隙重新分配后，第二次镜像面对的是**已经变化的几何**（gap 现算值 −280 钳 0），输出落点偏移 120px。ZW_R5501_PROBE 插桩实证：pre `First@0/Second@160` → l=2 后 `First@180/Second@40` → l=1 后 `First@340/Second@220`，非恒等。

字符级路径（`exact_l2_permutation`）对全偶序列天然返回恒等（None 短路），所以该 bug 只在行级片段序列面显形；且只在新激活的 RTL-基-纯-ASCII 页显形——此前含 RTL 脚本的页行内必有奇 level 片段，降层循环的每层反转都是**单次语义反转**（非依赖二次抵消），对合性从未被要求过。

## 解决方案

恒等面必须在**语义层短路**，不依赖数值对合：行级重排入口先判定 `line_has_odd`（任一片段 min/max level 为奇——原子盒基 level 1 的回落值计入），全偶行直接跳过整个降层循环（UAX #9 L2 数学恒等：全偶 level 的逐层反转两两抵消）。修复后 12 案 corpus 全绿，且该短路对既有奇 level 页零影响（判定为真时循环照旧）。

## 避免方式

- 给「反转/镜像/换位」类 pass 加第二个应用层（如逐层降级的 L2）前，先证明单次操作是**纯对合**（无状态依赖、无数值钳制、无随几何变化的输入）；装饰/间隙感知的空间重排几乎必然不满足。
- 「两次应用应抵消」不是恒等的实现方式，是恒等的**数学性质**；实现恒等面用谓词短路（本例 level 奇偶），把重排操作留给真正需要反转的层。
- 空间重排函数若要参与逐层复合，把「净 gap 钳 0」这类非线性整形从可逆路径剥离（或在第二次应用前保存原始几何）。
- 排查手法：对复合路径的每层应用做插桩打印 pre/post 状态（临时 env 探针，用后撤），单看最终值无法区分「一次错排」与「两次非对合」。
