---
date: 2026-11-06
modules: engine
---

# 探针判读必须覆盖目标语料行族全部断言面，而非只核树

## 问题描述

editing 语料每行是断言族（execCommand 返回值 + compare innerHTML + qIndeterm/qState/qValue before/after 共 ~9 断言），探针结果 dump 同时携带树与选区（`html=` + `sel=` 两字段）。本轮（nesh 两刀，styled `:688` 形）探针判读只逐字核对 `html=` 五案全绿即判「探针全绿」，未读 `sel=`、未对照语料行的 State/Value 断言面——首轮门禁批 iol 腿 VERDICT REVIEW：`:688` 同测试 `queryCommandState/Value("insertorderedlist") after` 双行 P2F=2，被迫定谳加刀（选区保全）+ 四腿重门，一轮门禁成本。

## 根因

两层叠加：

1. **判读面覆盖不足**：探针 dump 的 `sel=` 字段当时已是异常值（`DIV@1..1` 塌宿主层，对照同 dump 内守卫案的 in-span 正常态即可发现），判读 pass 条件只写了 html 逐字相等。
2. **机制盲区**：ST 臂 `insertBefore` 子树搬移在引擎选区重锚语义下把搬移子树内的 range 端点甩到壳外（宿主层）——树完全正确而选区丢失，qState/qV 壳标读面按端点爬祖先失配。pre-fix 该双行恰绿全凭保壳树下端点恰在列表内，「树修对反而暴露选区丢失」是典型假稳定面。

## 解决方案

- 代码：挪壳前按 newShell 相对索引路径快照两端点（`_zwListPathFrom`），拆块后显式回写（`_zwListResolvePath` + `setStart/setEnd`）；端点不全在壳内则不回写（不半新半旧混搭）。
- 判读规程：探针宣布全绿前逐项核对——① dump 的每个字段（含 `sel=`）与期望形；② 目标语料行族的完整断言面清单（compare / qInd/qState/qV before+after），逐断言确认探针可观察或给出不可观察理由。相关：[dbg 探针读点错位](2026-11-06-dbg-probe-read-point-mismatch-harness-assert.md)（断言面错位伪影）——两案合并的判读纪律：**探针绿 = 树 + 选区 + 全断言面三方对齐，缺一即灰**。
