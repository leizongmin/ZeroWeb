# R240 Evidence — 祖先 extract 的 contained 中段子移动（+1P，28,x 深诊断推进）

**日期**: 2026-08-25
**切片**: M4——R240(a) 28,x differing 28F（实为 `[testDiv,0,comment,5]` 形态）
**改动面**: `part06.js`（R236 祖先分支 contained-children 移动 + 快照）+ `part23.rs`（r240 单测）
**commit**: `36c54d135`

## 一、28,x 形态深诊断

28,x = `[testDiv,0, comment,5]`（sc=testDiv、ec=comment 直接 CD 子）——
R236 祖先分支的直接形态。三轮探针（R240-probe 沙箱 + harness
nextNode trace）实证：

1. **sim 的 partial 检查在此形态不触发**：nextNode trace（expected
   iframe 内）只有 3 站 `DIV→P→text→null`——text 的
   nextNodeDescendants 爬升中 P.nextSibling 为 null（克隆树的
   paras 未挂进 testDiv 克隆的 childNodes——setupRangeTests 的
   `document.querySelector("#test")` 返回 wrapper 域对象，append
   落 wrapper 列表），遍历提前终止，comment 未被扫到。
2. **host 同样不触发**（两侧对称成功通过 partial 检查）——分歧在
   surround 主体：sim 的 generic extract 移动中段子 + 削头，host
   的 R236 分支只削头。

## 二、修复（extract 侧语义补全）

R236 祖先分支补 **contained 中段子移动**（spec containedChildren：
sc 的 [so, ecIdx) 子本体移入 frag）。过程坑：按下标迭代时
appendChild 使 childNodes 收缩滑位——stale ecIdx 把 ec 本体也拖进
frag（探针 `ex-frag=[P,"oup?","bet s"]` 错序含 remainder）。
**快照后移动**修正。

## 三、验证链（vs R239 基线）

| 项 | R239 | R240 | Δ |
|---|---|---|---|
| Range-extractContents | 125P | **126P** | +1 |
| Range-surroundContents | 1701P/139F | 1701P/139F | 0（28,0 从 walk 级 First-differing 推进到深层 isEqualNode 分歧——28,x 需 generic cross-container surround 全序，R241 靶点） |
| Range-insertNode | 1841P/0F | 1841P/0F | 0（100% 保持） |
| Range-delete/clone | 67/155 | 67/155 | 0 |
| native extract | — | 126P | 同值 |

- **engine 单测**：**2386 全绿**（新增 r240 单测：快照防滑位 + 削头
  remainder 留树双断言）。fmt/clippy 干净；探针已清理。

## 四、R241 靶点

- **首选**：28,x generic cross-container surround 全序（sim 对
  `[testDiv,0,comment,5]` 的 surround 成功路径：extract 移动中段子 +
  削头 → insert newParent → appendChild(frag) → select——host 的
  R236 surround 分支已具备骨架，需接通新 contained-move extract 并对
  齐 walk 深层形态）。
- 次选：25/26,x Document 容器 HRE 12F。
- 深项：cDP 40（17/30,x）/ 24,x 跨子区间 32F / startOffset 16,x 11F /
  克隆树 wrapper-domain 挂链（本轮实证的 querySelector wrapper 域
  断链——assert_unreached 族共同根因候选）。
