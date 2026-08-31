# R242 Evidence — 元素 ec 祖先形态 extract/surround 全序（24,x 全解 +38P）

**日期**: 2026-08-25
**切片**: M4——R242(a) 24,x assert_unreached 32F（`[testDiv,2,paras[4],1]`）
**改动面**: `part06.js`（extract 新分支 + surround 元素/leaf 两路）+ `part23.rs`（r242 单测）
**commit**: `d8ce01d1c`

## 一、形态与 sim 语义

24,x = `[testDiv, 2, paras[4], 1]`——sc 元素祖先、ec **元素**直接子、
双侧 clean 边界（so 在 sc 子边界、eo 在 ec 子边界）。sim
（myExtractContents ancestor 分支，last partially contained child
为元素）：

1. contained = sc 的 `[so, ecIdx)` 子**本体移入 frag**（复用 R241
   wrapper 域 move 兜底）；
2. ec **shallow clone** 入 frag + 子区间 `[ec,0,ec,eo]` 递归提取
   （ec 的 `[0,eo)` 子移入 clone）；
3. range 塌缩 `(sc, so)`。

## 二、修复三件（part06）

1. **extractContents R242 分支**：上述三步（guard：sc element +
   ec element 直接子 + eo ≤ ec.childNodes.length + so ≤ ecIdx）。
2. **surround 元素 newParent 同形态全序**：extract → 清子 → 按位
   insert → appendChild(frag) → selectNode。
3. **surround leaf-newParent 同形态先 extract 后 HRE**。

**单测边界注记**：engine 沙箱树形态完好时 spec 正确行为是 partial
检查先抛 InvalidStateError（[d,1,p3,1] 的 p3 是 partial 非 Text）；
WPT 24,x 的成功期望来自 harness 克隆树遍历盲区（R240 已证）。
单测覆盖 extract standalone（surround 断言由 WPT 承载）。

## 三、验证链（vs R241 基线）

| 项 | R241 | R242 | Δ |
|---|---|---|---|
| Range-surroundContents | 1734P/106F | **1768P/72F** | **+34，0 新失败**（24,x 32F 全解 + HRE 18/19,x 残余 +2） |
| Range-extractContents | 128P | **132P** | +4 |
| Range-insertNode | 1841P/0F | 1841P/0F | 0（100% 保持） |
| Range-delete/clone | 67/155 | 67/155 | 0 |
| **ranges 全量**（除 probe） | 40080 行 | 40080 行 | set-diff **38 Fail→Pass / 0 反向** |

- **native 同值**：ZW_NATIVE_DOM=1 surround 1734→1768P（+34 一致）。
- **engine 单测**：**2388 全绿**（新增 r242_element_ec_ancestor_extract）。
- fmt/clippy 干净。

## 四、R243 靶点（72F 重聚类）

| 簇 | 计数 | 行 | 备注 |
|---|---|---|---|
| cDP | 40 | 17,x 17 + 30,x 23 | 绑 host foreignDoc surround 全序（R235 负结果——**重评**：R236–R242 全序落地后 host 行为面大改，负结果前提可能失效） |
| HRE | 14 | 25/26,x 各 6 + 18/19,x 各 1 | 25/26,x `[document,0,…]` Document 容器 |
| startOffset | 11 | 16,x | harness-iframe index 算术 |
| differing | 4 | 13/14,x | 残余 |
| assert_unreached 2 + other 1 | 3 | 18/19/28,x | 残余 |

- **首选**：cDP 40 重评（R235 负结果复测——当时 -28 的 sim 翻转面
  在 R236–R242 十一个新分支后可能已消失）。
- 次选：25/26,x Document 容器 HRE 12F。
