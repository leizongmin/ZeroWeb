# 刀 G2-a：set 门 unit 臂 famName 直取（css-typed-om +25）

日期：2026-11-19　文件：`crates/engine/src/js_dom_shim/part10-typed-om.js`（单 hunk +8/-2）
电池：`.acceptance/wptfyi-css/knife-g2a-battery.sh`，日志 `knife-g2a-battery-1709.log`，results `results-knife-g2a/`

## 机制定谳

`_zwOmValidateSet` unit 臂旧法用 `_zwOmUnitType(t.unit)` 返回 map 再迭代族表取 famName。
ARITH 刀（b449da8f0）引入零指数键省略形状后，`<number>` 值的 map 为 `{}`（无 length/exponent
键），迭代得 `famName=null` → `want=[]` → set 恒 throw TypeError。HEAD 考古（pre-faceC 基）
同位置同炸：旧债非 ARITH 回归，但 ARITH 形状使既有法证定位路径显形。

修复：famName 改从 `_zwOmUnitFamily` 族表 `[0]` 直取（经 `_zwOmUnitInfo(t.unit)`），
`<number>` → 'number'、`<percentage>` → 'percent'。负 flex 门、map[want] 查表、throw 路径不动；
`_zwOmUnitType` 本身不动（arith/gCS 消费方保形）。根因与 G2-a/b/c 分解由刀战仲裁账本
`NOTES-knife-decomp-launcher-arbitration.txt` G2 节定谳。

## 首发法证

- 17:08:54 首发 rc=2：battery 脚本 patch 基（`git diff HEAD` 预存）对在飞 faceC 树态
  `--check` 上下文错位。修复：`git checkout HEAD -- part10` 剥 face 后重发（patch 文件
  保全 face）。与 arith 电池首发（14:40:38）同类：face 不得在树内发射。

## 探针

17:05-17:06 `knife-g2a-probe.sh`（typed-om 单语料 vs arith2/b1 基）：+25 gained / 15 页，
与电池分毫不差。探针 lost=7411 为目录级假账（单语料目录 vs 5 语料基目录缺件），
typed-om 页零损失，REAL 面仅看 gained。

## 电池账（5 语料腿 ×2，17:09:15-17:34:27，rc=0）

TOTAL: gained=25 lost=0 regressed=0 pages_gain=15 pages_loss=0（RECONCILE 17:34:27）

15 页全落 `css/css-typed-om/the-stylepropertymap/properties/`：animation-iteration-count +2、
fill-opacity +1、flex-grow +3、flex-shrink +2、flood-opacity +1、font-size-adjust +2、
font-weight +1、opacity +3、order +1、shape-image-threshold +3、stop-opacity +1、
stroke-miterlimit +2、stroke-opacity +1、tab-size +1、z-index +1。

## Provenance

- b0 md5 `685b895c6a425254f3918fcc20855922` ＝ arith2 电池 b1 同 md5（HEAD+arith+faceC
  内容确定性双证，基态清洁）。
- b1 md5 `1077acec20dc8d630912922f254ee2ba`。
- RIDE 3 文件（part115_s423 test / es_module s419 / residue-survey doc）b0 窗与收账窗
  零扰动。b1 构建窗 17:09:48-17:10:2x；兄弟 fontcycle/g2b 流文件首写均 ≥17:29
  （parse_layout 17:29:48、computed_style 17:35:47），不入 b1 构建，+25 归因干净。

## 残面（后续刀）

- G2-b：per-property computed-opaque 面（border-image-* 族，兄弟 fork g2b 电池在飞认领）。
- G2-c：percent 形两面（fill-opacity ÷100 vs opacity 保形）。
- column-count/line-height 残留 + calc 形（declared/ComputedStyle 非纯 unit 值路径）。
