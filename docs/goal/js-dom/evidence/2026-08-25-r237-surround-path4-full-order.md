# R237 Evidence — surround 路径 4 全序对齐（12–14,x 全解，+69P 纯增）

**日期**: 2026-08-25
**切片**: M4——R237(a) 12–14,x 残余 differing（docEl 容器 extract/surround 首差形态）
**改动面**: `part06.js`（surroundContents 路径 4 三件）+ `part23.rs`（r237 单测）
**commit**: `1729088ea`

## 一、探针实证（R237-probe.html，已清理）

对 12,0 `[documentElement,0,documentElement,1]` + newParent=paras[0]
双引擎 dump：

| 引擎 | docEl 子序 | newParent 内容 |
|---|---|---|
| host | `[BODY, P]`（P 在末尾） | P{2}（原文本残留 + head 克隆） |
| sim | `[P{1}, BODY]`（P 在位） | P{1}（head 原件） |

三分歧：① 路径 4 漏 sim 步骤 2「清 newParent 既有子」；②
`appendChild(newParent)` 恒插末尾而非 (startContainer, startOffset)
位；③ 边界从不更新（漏步骤 6 selectNode）。

## 二、修复三件（part06 路径 4）

1. **清 newParent 既有子**（步骤 2）——**先清再 clone**：首版清在
   clone 循环后把 covered 子克隆一并误删（单测 cleared:0 抓回）。
2. **按位插入**（步骤 4）：`insertBefore(newParent, ref)`，ref =
   `childNodes[startOffset]`；null 时 appendChild 兜底。
3. **selectNode 边界**（步骤 6）：setStart/setEnd 到
   (父, idx)-(父, idx+1)。

## 三、验证链（vs R236 基线）

| 项 | R236 | R237 | Δ |
|---|---|---|---|
| Range-surroundContents | 1543P/297F | **1612P/228F** | **+69，0 新失败**（12–14,x 36F + 清-序连带 +33） |
| Range-insertNode | 1841P/0F | 1841P/0F | 0（100% 保持） |
| Range-extract/delete/clone | 121/67/155 | 121/67/155 | 0 |
| **ranges 全量**（除 probe） | 40080 行 | 40080 行 | set-diff **69 Fail→Pass / 0 反向** |

- **native 同值**：ZW_NATIVE_DOM=1 surround 1543→1612P（+69 一致）。
- **engine 单测**：**2384 全绿**（新增 r237_surround_path4_full_order：
  order/cleared/sel 三段断言）。
- fmt/clippy 干净；探针文件已清理。

## 四、R238 靶点（228F 重聚类）

| 簇 | 估计 | 备注 |
|---|---|---|
| assert_unreached | ~55 | 克隆子树内部残留深项 |
| cDP | 40 | 17,x/30,x——绑 host foreignDoc surround 全序（R235 负结果） |
| HRE / INVALID_STATE | ~36/30 | sim 全序残余（extract 序/validity 位） |
| differing 残余 | ~50 | 19,x detached + 28,x + 24,x + 38,x |
| partial-msg 12 / startOffset 11 / other 2 | 25 | 24,x message + 16,x index |

- **首选**：HRE 36 重评（R236/R237 全序落地后 host 行为面已变——旧
  归因可能失效，重跑聚类按新消息取样）。
- 次选：19,x detached differing（`[detachedPara1,0,detachedPara1,1]`
  族 ~28F）。
