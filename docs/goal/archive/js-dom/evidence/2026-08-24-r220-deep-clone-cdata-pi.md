# R220 Evidence — deep-clone CDATA/PI 保型 + 跨轮残留结构根因定位

**日期**: 2026-08-24
**切片**: M4——R220(a) insertNode ~740F 重聚类 → 结构根因定位；（b）docEl head/body 子链接成对实验
**改动面**: `part03.js`（`_zwDeepCloneEl` CDATA/PI 分支）+ `part05.js`（评估记录注释）+ `part21.rs`（回归单测）

## 一、insertNode 747F 聚类（1094P 基线）

| 簇 | 量 | 形态 |
|---|---|---|
| assert_unreached | 516 | rows 16/18/20–28,x 整行 ~40F |
| assert_throws_dom HRE | 86 | 跨容器 foreignDoc/xmlDoc 形态 |
| null-nodeType | 66 | rows 25 [document,0,document,1] 等 |
| assert_true/equals | 79 | 形态混合 |

**关键实证**：抽样 16,0 / 25,0 / 27,0 **单独跑全部 Pass**、累积跑才 Fail——
残余主体是**跨轮残留**（同一页面两 iframe 的共享态泄漏），非 per-subtest 语义。

## 二、结构根因（R220-drift 探针）

- `referenceDoc.appendChild(actualIframe.contentDocument.documentElement
  .cloneNode(true))` 的克隆产物 = **HTML(0) 空壳**——iframe doc 的 docEl
  （part05 解析/合成形态）`childNodes` 恒空（head/body 是基础 detached doc 的
  字面量、不在 docEl 下）→ referenceDoc 拿到空 body，每轮 restoreIframe 克隆
  回给两 iframe 的也是空壳 + setupRangeTests 前置内容丢失 → 后续轮次树形态
  与 host 分歧逐步累积。

## 三、成对实验（已试已回退）

**linkage**：把 doc.head/doc.body 链入 docEl.childNodes（part05 R216 段后）。
结果：rows 12–16 **-158P**（1094→936P）。

R221-probe 树签名定位分歧：A=HTML(3, HEAD, BODY, +插入节点) vs E=HTML(2)——
**平行树**：restoreIframe 每轮用 referenceDoc 的克隆**替换** doc 内容后，
document.head/body 仍指 factory 时的原字面量（setupRangeTests 的 paras 挂在
字面量 body 树），而 rangeFromEndpoints 消费的是克隆 docEl 树——两树分裂使
sim/host 无法对齐。

**结论**：head/body 子链接只在 factory 时成立，restoreIframe 换 docEl 即破——
正确修法是 **fresh-doc 级**（每轮整体重建 iframe doc，含 head/body/docEl 一体），
即 R208 家族深项的实证依据。已记 R221 靶点。

## 四、本轮 land 件

`_zwDeepCloneEl` 的 CDATA/PI 子分支：旧落 `return null` → 含 CDATA/PI 子树的
深克隆丢子。CDATA 经 ownerDocument.createCDATASection 重建（R218 尾节点保型
同款）、PI 经 _zwMPiFromBogus 视图重建（target/data 保持）。当前 docEl 空壳
路径不经此分支（WPT 零位移），但任何含 CDATA/PI 子的元素深克隆（cloneContents
等）消费本分支——spec 正确性 + fresh-doc 落地后的前置件。

## 五、验证链

- insertNode 1094P / surround 854P / extract 100P / clone 153P 全不变（零位移）
- engine 单测 **2366 全绿**（新增 `r220_deep_clone_preserves_cdata_and_pi_children`
  ——元素/CDATA/PI 三形态子保型断言）
- fmt / clippy 零警告

## 六、commit

231359e22
