# R236 Evidence — 祖先元素区间 extract/surround 全序（23,x 32F 全解 + extract 连带 +3）

**日期**: 2026-08-25
**切片**: M4——R236(a) node-与-后代区间（23,x `[paras[0],0,paras[0].firstChild,7]`）
**改动面**: `part06.js`（extractContents 祖先分支 + surroundContents leaf/元素两路）+ `part23.rs`（r236 单测）
**commit**: `78d0664fd`

## 一、形态与根因

range 形态：sc 是 ec 的**元素祖先容器**且 ec 是其**直接 CharData 子**
（`[p,0, t,7]`）。host 全链路空转：

- `_coveredChildren` 对 sc≠ec 恒 null；
- extractContents 落 R234 的 else-branch（仅 collapse 无树变更）；
- surroundContents leaf 路径无匹配分支直接抛 HRE、元素路径 defer return。

sim（common.js myExtractContents ancestor 分支）：first partially
contained child = null，last = ec → **clone ec 的 [0,eo) 头切片入 frag +
ec.deleteData(0,eo) 削头（remainder 留树）**，range 塌缩 (sc, so)。

## 二、修复三件（part06）

1. **extractContents 祖先分支**：sc element + ec 直接 CD 子 → 头切片
   clone + deleteData + 塌缩 (sc, so)（对齐 sim 全序）。
2. **surroundContents leaf-newParent**：同形态先 extract（削头）再
   insertNode 后抛 HRE（挂入 `_r215NoValidate` 块新分支）。
3. **surroundContents 元素 newParent**：extract → **清 newParent 既有子**
   （sim 步骤 2——首版漏此步使 wrap 残留原文本 "Efghijkl"，expected
   头切片）→ insertNode（newParent === sc 自身时 R215 inclusive-ancestor
   查在**树变更后**抛 HRE，对齐 23,0 形态）→ appendChild(frag) →
   selectNode 边界。

## 三、验证链（vs R235 基线）

| 项 | R235 | R236 | Δ |
|---|---|---|---|
| Range-surroundContents | 1509P/331F | **1543P/297F** | **+34，0 新失败**（23,x 32F 全解 + 12,14,x 残余 +2） |
| Range-extractContents | 118P | **121P** | +3（恰为 range 23 三 subtest，set-diff 0 新增） |
| Range-insertNode | 1841P/0F | 1841P/0F | 0（100% 保持） |
| Range-delete/clone | 67/155 | 67/155 | 0 |
| **ranges 全量**（除 probe） | 40080 行 | 40080 行 | set-diff **37 Fail→Pass / 0 反向** |

- **native 同值**：ZW_NATIVE_DOM=1 surround 1509→1543P（+34 一致）。
- **engine 单测**：**2383 全绿**（新增 r236_ancestor_element_range_surround_extract：
  ex 削头 + leaf 序 + 元素 wrap 全序三段断言）。
- fmt/clippy 干净。

## 四、R237 靶点（297F 重聚类）

| 簇 | 估计 | 备注 |
|---|---|---|
| differing | ~100 | 12–14,x docEl 容器（expected `<p id=a><head>…` 形态）+ 19,x detached + 28,x |
| assert_unreached | ~63 | 克隆子树内部残留深项 |
| cDP | 40 | 17,x/30,x——绑 host surround 全序（R235 负结果） |
| HRE / INVALID_STATE | 37/30 | 重评独立性 |
| partial-selected 12 / startOffset 11 / other | ~27 | 24,x message + 16,x index |

- **首选**：12–14,x 残余 differing ~36F（R234 动态 getter 后 host 对
  docEl 容器 extract 的首差形态——expected 元素含 head vs got object，
  疑 host 的 clone+remove 记账与 sim move 语义在 docEl 容器上的残余）。
- 次选：19,x detached differing ~28F。
