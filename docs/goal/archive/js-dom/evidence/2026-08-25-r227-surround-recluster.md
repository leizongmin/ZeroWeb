# R227 Evidence — surroundContents 947F 重聚类（负结果轮：两枚 kill-switch/单侧校验实测记录）

**日期**: 2026-08-25
**切片**: M4——R227(a) surround ~350F 重聚类（893P/947F 基线）
**改动面**: 无 land 代码（两个 A/B 实验均负，已回退）——本轮产出为聚类数据与两项决策依据

## 一、947F 聚类（按错误签名）

| 簇 | 计数 | 形态 |
|---|---|---|
| assert_unreached「DOMs were not equal」 | 282 | 树 mismatch 深形态（跨轮残留族） |
| **cDP is not a function**（node2 91 + nodeB 17） | 108 | sim 的 isAncestorContainer/getPosition 深入 iframe 合成树（docEl/body）缺方法面 |
| assert_throws_dom HRE must be thrown | 93 | host 不抛（CharData 路径 `_r215NoValidate` 抑制校验） |
| First differing Text/Comment/PI node | ~220 | extract 边界精度（含 comment 分裂族 ~84：expected "Stuwxyz" got "Stuvwxyz"） |
| assert_throws_dom INVALID_STATE_ERR | 30 | host 不抛（部分包含检查漏 CharData 容器内形态） |
| startOffset expected 2 got 4 等 | ~30 | selectNode(newParent) 同步族 |

## 二、两个 A/B 负结果（决策依据）

1. **R219 原型兜底 kill-switch 复测**（Node.prototype.cDP/contains/hasChildNodes）：
   R222/R225/R226 之后的树形态下仍 **893→865P（-28）**——与 R221 时记录一致。跨轮
   残留（restoreIframe 只清 doc 首末子）未根除前不能启用。**保持关**，108F cDP 簇
   绑定在「iframe contentDocument 每轮 fresh-doc」深项上。
2. **单侧祖先 HRE 预检**（surroundContents 入口对 newParent 上行链查
   inclusive ancestor → 抛）：**893→891P（-2）**。sim 的 ensurePreInsertion 在
   **extract 塌缩后**的 parent_ 上判定（与 host 变更前 startContainer 链不同），
   单侧实现必产生 host-早抛/sim-不抛的翻转。**回退**。正解 = host CharData 路径
   复刻 sim 的 extract→insert 全序（R228+）。

## 三、R228 靶点（按 ROI）

- **comment/PI 区间 surround**（~84F）：[detachedComment,3,4] 族——真 DOM 对部分
  选中 CharData（含 comment）split 后插 newParent；host 的 R212 路径只覆盖
  Text/CDATA（`_r212isCd2` 含 7/8 但 extract 的 R211 分支只走 3/4）。
- **surround 的 sim 全序复刻**（HRE 93F + INVALID_STATE 30F + startOffset 30F）：
  CharData 路径按 mySurroundContents 序（extract 塌缩 → clear newParent →
  insertNode(newParent)（含完整 pre-insertion validity，parent_ = 塌缩后容器）→
  appendChild(frag) → selectNode 同步）。
- **fresh-doc 残余深项**（解锁 R219 开关 + 282 assert_unreached 大簇）。

## 四、commit

无代码 land（实验已回退，工作树零 diff）。
