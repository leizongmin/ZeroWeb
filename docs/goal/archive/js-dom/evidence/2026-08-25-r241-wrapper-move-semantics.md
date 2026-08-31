# R241 Evidence — wrapper 域 move 语义兜底（28,x 簇 +33P，双份树根因）

**日期**: 2026-08-25
**切片**: M4——R241(a) 28,x generic cross-container surround 全序
**改动面**: `part06.js`（祖先 extract contained-move 的强制摘除）+ `part23.rs`（r241 单测）
**commit**: `dcd812416`

## 一、R241-probe 全树 dump（已清理）

对 28,0 `[testDiv,0,comment,5]` + newParent=paras[0] 双引擎全树递归
dump（含属性/文本/CDATA），实证 host 树**双份**：

- host：`DIV=[P#a[全部提取内容拷贝…], P#a[同款拷贝], P#b 原件…]`
- sim：`DIV=[P#a[提取内容], #comment"oup?" remainder]`

根因：WPT iframe 的 covered 子生活在 **querySelector wrapper 域**
（R240 实证），对 fragment 的 `appendChild` 是 **clone 语义**——
原件残留 sc、拷贝进 frag，newParent 得拷贝后原树还有原件。
R240 的单测通过是因为 engine 沙箱的子是主文档 `_zwMEl` 产物
（appendChild 是 move）——两域 appendChild 语义不同。

## 二、修复

contained-move 循环 append 后**强制摘除**：原件仍在
`sc.childNodes`（indexOf 命中）则 `removeChild`——move 语义兜底，
对 appendChild 本就 move 的域是无害 no-op。

## 三、验证链（vs R240 基线）

| 项 | R240 | R241 | Δ |
|---|---|---|---|
| Range-surroundContents | 1701P/139F | **1734P/106F** | **+33，0 新失败**（28,x 簇主解） |
| Range-extractContents | 126P | **128P** | +2 |
| Range-insertNode | 1841P/0F | 1841P/0F | 0（100% 保持） |
| Range-delete/clone | 67/155 | 67/155 | 0 |
| **ranges 全量**（除 probe） | 40080 行 | 40080 行 | set-diff **36 Fail→Pass / 0 反向**（vs R239 基线，含 R240 的 +1） |

- **native 同值**：ZW_NATIVE_DOM=1 surround 1701→1734P（+33 一致）。
- **engine 单测**：**2387 全绿**（新增 r241：move 兜底无双份断言）。
- fmt/clippy 干净；探针已清理。

## 四、R242 靶点（106F 重聚类）

| 簇 | 计数 | 行 | 备注 |
|---|---|---|---|
| cDP | 40 | 17,x 17 + 30,x 23 | 绑 host foreignDoc surround 全序（R235 负结果） |
| assert_unreached | 34 | 24,x 32 + 18/19,x 各 1 | 24,x 跨子区间 `[testDiv,2,paras[4],1]`（与 28,x 同为 wrapper 域家族——R241 兜底未覆盖 sc≠ec 元素形态） |
| HRE | 14 | 25/26,x 各 6 + 18/19,x 各 1 | Document 容器 `[document,0,…]` |
| startOffset | 11 | 16,x | harness-iframe index 算术 |
| differing | 6 | 13/14/x 4 + 24,x 2 | 残余 |

- **首选**：24,x assert_unreached 32F——`[testDiv,2,paras[4],1]`
  与 28,x 同族但 so=2 且 ec 是**元素**（paras[4]）内 text——祖先
  分支的 ec 是元素形态扩展（ec.parentNode.parentNode === sc 的
  二级后代 + contained-move 复用）。
- 次选：25/26,x Document 容器 HRE 12F。
