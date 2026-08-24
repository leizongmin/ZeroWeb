# R223 Evidence — doc 级子 parentNode 健壮写入

**日期**: 2026-08-25
**切片**: M4——R223(a) row 29 foreignDoc 族 22F 的 null.nodeType 根因修复
**改动面**: `part03.js`（`_r223SetParent` helper + doc 级 appendChild/insertBefore 四站点）+ `part21.rs`（单测）

## 一、根因链（三层探针）

1. **step 包装栈捕获**：null.nodeType 抛自 sim 的
   `ensurePreInsertionValidity`（`parent_.nodeType`，parent_ = null）。
2. **myInsertNode 包装 dump**：foreignDoc.childNodes =
   `[doctype@p set, HTML@pNULL, comment@p set]`——html 子 parentNode 恒 null，
   kids[1] 取到它 → parent_ = null。
3. **属性描述符探针**：html 子的 parentNode 是 **getter-only accessor**
   （赋值抛 "which has only a getter"）；handle 形态 comment 的 parentNode 经
   get trap 读 `_zwNodeParent` 注册表——doc 级 `c.parentNode = this` 对两形态
   分别抛/静默 no-op（sloppy），从未生效。

（R222 的「foreignDoc.childNodes 空」读数经复核为**探针伪影**——probe 读
`contentWindow.foreignDoc` 是 undefined，守卫跳过后打印空列表误导；本轮修正
probe 方法后真实结构如上。）

## 二、修法

`_r223SetParent(node, parent)`（doc 级 appendChild 1 站 + insertBefore 3 站）：

- **handle 形态**（`__zwHandle` 为 string）：写 `_zwNodeParent[h]` 的 R180
  `plainParent` 槽（doc 是 plain 对象无 sel/handle）——get trap 读回；
- **其余**：恒经 `defineProperty` own 数据属性遮蔽继承的 getter-only
  accessor（裸赋值 sloppy 静默 no-op），raw 赋值兜底。

## 三、验证链（vs R222）

| 项 | R222 | R223 | Δ |
|---|---|---|---|
| Range-insertNode | 1669P | **1693P** | +24 |
| dom/traversal | 1595P | 1602P | +7 |
| Range-mutations | 1338P | 1336P | -2 |
| dom/nodes / collections | 12663 / 49 | 同 | 0 |
| dom/events | 579 | 579（复跑确认，首轮 577 为 flake） | 0 |
| surround / extract / clone / delete | 893/103/156/68 | 同 | 0 |

净 **≈ +29P**。

- **engine 单测**：**2376 全绿**（新增 `r223_doc_append_child_parent_node_sticks`
  ——注册表路径 comment 育儿 + insertBefore membership；element-proxy 的
  育儿链断言面由 WPT 族承载，trap 分支序不在本切片断言面）。
- **fmt / clippy**：零警告；**make test** 1F = XOpenDisplayFailed 环境项。

## 四、R224 靶点

- insertNode 剩 148F：HRE ~70（foreignDoc/xmlDoc/document 作 node 的跨容器族）
  + text-differ / assert_unreached 散布（~78）。
- surround 剩 ~350F（893P 基线重聚类）。

## 五、commit

9087f15f3
