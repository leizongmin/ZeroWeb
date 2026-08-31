# R231 Evidence — 同节点区间 extract 不塌缩（sim 早返回形态实证）

**日期**: 2026-08-25
**切片**: M4——R231(a) endOffset expected 8/9 got 2 簇（~93F，2,x/27,x 等）
**改动面**: `part06.js`（R211 同节点分支移除塌缩）+ `part23.rs`（回归测试扩展 `r231:2,8` 断言）

## 一、方法论突破：沙箱内直跑 sim 源

前几轮对 sim（common.js myExtractContents）塌缩语义的推断反复出错（R229 按
spec self-branch 实现 (容器, startOffset) 塌缩，R231 复核发现 sim 实际**不
塌缩**）。本轮把 `myExtractContents` 源 + 5 个 helper（isText/
isAncestorContainer/indexOf/nodeLength/ownerDocument）注入沙箱直接执行，
实证 `[t,2,8]` 的终态：

```json
{"frag":{},"so":2,"eo":8,"data":"Op"}
```

**data 削为 "Op"（deleteData 执行）但 range 保持 (t,2)-(t,8)**——sim 的
CharacterData first/last 子路径在中段 clone/deleteData 后**早返回**，尾部
`range.setStart/setEnd` 塌缩不执行。WPT 的 expected 即此形态。

## 二、修法

R211 同节点分支（`sc === ec`）移除 setStart/setEnd（R229 的 self-collapse
与 R228 的 detached collapse 均已证伪）；异节点同父保持 (父, si+1)
（else 分支语义不变）。

## 三、验证链（vs R230）

| 项 | R230 | R231 | Δ |
|---|---|---|---|
| Range-surroundContents | 1269P/571F | **1385P/455F** | **+116，0 新失败**（diff 116 fixed / 0 new） |
| Range-insertNode | 1840P/0F | 1840P/0F | 0（100% 保持） |
| Range-extractContents / deleteContents | 115P / 65P | 115P / 65P | 0 |

- **engine 单测**：**2380 全绿**（回归测试扩展 `r231:2,8`——同节点区间
  surround 抛 HRE 后边界保持 (2,8) + data 已削）。
- **fmt / clippy**：零警告。

## 四、R232 靶点（当前 455F）

| 簇 | 计数 | 备注 |
|---|---|---|
| assert_unreached | ~110 | fresh-doc 跨轮残留族 |
| cDP 缺方法面 | 108 | R219 开关（fresh-doc 深项绑定） |
| HRE / INVALID_STATE must-thrown | ~67 | sim 全序残余 |
| 其他 Text/differing 残余 | ~170 | 待重聚类 |

## 五、commit

9159d9a93
