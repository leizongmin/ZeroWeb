# R228 Evidence — detached 同节点 CharData 区间 surround（comment/PI/text 的 35–38,x 族）

**日期**: 2026-08-25
**切片**: M4——R228(a) comment/PI 区间 surround（R227 聚类的 comment 分裂族 84F）
**改动面**: `part06.js`（extractContents R211 分支放宽 + 三处 null 守卫 + `_r212` 门放宽 + insertNode HRE 不再吞）+ `part23.rs`（回归单测）

## 一、根因（R227 聚类 + 本轮逐步实证）

WPT Range-surroundContents 35–38,x（range 落在 **detached** Comment/PI/Text 的
data 区间内，如 `[detachedComment,3,detachedComment,4]`）双断言失败：
「must be thrown」（host 不抛 HRE）+ 「expected "Stuwxyz" got "Stuvwxyz"」
（extract 空转，data 不变）。根因链四环：

1. **extractContents R211 分支的 parentNode 守卫**：`sc.parentNode && …`——
   detached 节点 parentNode null 直接落空，中段切片/deleteData 不执行。
2. **surroundContents `_r212` 门同款守卫**：CharData 路径要求同父非空——
   detached 同节点形态根本进不了 extract。
3. **detached 分支内三处 null 崩**：`_r211p.childNodes` / `_r211p.ownerDocument`
   / kids 索引（indexOf 空数组 -1）。
4. **insertNode 的 HRE 被 `try/catch` 吞**：`_r212` 路径 `try { insertNode } catch {}`
   ——sim 序（common.js mySurroundContents）extract 变更树后 insertNode 对叶子
   容器抛 HRE 并上抛；host 吞错使「must be thrown」族不抛。

## 二、修法（与 sim 全序对齐）

1. R211 分支 guard 放宽：`_r228sameNode = (sc === ec && isCd(sc))`——同节点
   CharData 无需父容器（spec extract 的 clone-切片 + deleteData 不依赖父）；
   异节点仍需同父定位。
2. detached 同节点子分支：中段切片 → deleteData → collapse 到 `(容器, startOffset)`
   （无父定位步骤）；`_r211kids`/`_r211frag` 的 null 守卫。
3. `_r212` 门同款放宽（`sc === ec` 短路 parentNode 检查）。
4. `insertNode(newParent)` 的 HRE **不再吞**（去掉 try/catch）——上抛对齐 sim。

https://dom.spec.whatwg.org/#dom-range-extractcontents
https://dom.spec.whatwg.org/#dom-range-surroundcontents

## 三、验证链（vs R227）

| 项 | R227 | R228 | Δ |
|---|---|---|---|
| Range-surroundContents | 893P/947F | **943P/897F** | **+50，0 新失败** |
| Range-extractContents | 99P/88F | 111P/76F（全量 115P） | **+12**（连带：extract 的 R228 分支直接受益） |
| Range-insertNode | 1840P/0F | 1840P/0F | 0（100% 保持） |
| Range-deleteContents | 65P | 65P（全量 68P） | 0 |

全量套件：ranges 37678→**37740（+62）**；nodes 12661→12660（-1 flake 带）、
events 579→577（±2 flake 带，R223 轮同款波动）、collections 49 / traversal 1602
全稳。净 **≈ +60P**。

失败集 diff：**fixed 50 / new-fails 0**（纯增）。

- **engine 单测**：**2380 全绿**（新增 `r228_detached_chardata_interval_surround`
  ——detached comment 区间 surround：data 切片（"w" 移除）+ HRE 上抛双断言）。
- **fmt / clippy**：零警告。

## 四、R229 靶点

- **surround 剩 897F 重聚类**：assert_unreached 282 / cDP 108（fresh-doc 深项）/
  HRE ~43 残余 + INVALID_STATE 30（sim 全序复刻其余步骤：clear newParent /
  appendChild(frag) / selectNode 同步）。
- 深项：fresh-doc 残余（解锁 R219 开关）/ customElements 多 registry /
  :scope query-root / lone-surrogate wire / MO-document parser 记录。

## 五、commit

c6cbec413
