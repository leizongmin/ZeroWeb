# R248 Evidence — harness 内联探针定位 17/13,x「幽灵 P」+ 显式摘除守卫 land（WPT 净 0）

**日期**: 2026-08-25
**切片**: M4——R248(a) win.run() 内联探针（12 轮迭代）+ part06 显式摘除守卫
**改动面**: `part06.js`（surround R237 插入前显式 `newParent.remove()`）+ wpt-data 探针（已全部还原）
**commit**: 见 master.md 本轮记录

## 一、探针链（Range-test-iframe run() 内联 + 探针页双窗 dump）

1. **对象域标签**（run() 创建点）：A/E 双侧 sc/node 标签完全对称
   （`#1:HTML/cnOWN/pOWN`）——对象域分裂假设（R246 ③）排除。
2. **双树 dump（深度 3）**：POST sc 子树双侧完全一致；**ROOT 树发现
   「幽灵 P」**——13,0 的 A 侧 DIV = **6 P** vs E 侧 **5 P**（sim 正确移出
   paras[0]，host 的 DIV 残留第六个 P 对象）。
3. **p0 父链探针**：`p0pA[null] p0pE[null]`——双侧 paras[0].parentNode
   均为 null（对象真值一致），但 A 的 DIV childNodes **仍含 p0**（单向
   断链：父链置空而子列表未 splice）。
4. **run() 内联 op 包装**：testDiv.removeChild / p0.parentNode setter 陷阱
   ——surround 期间 **零调用/零写入**，但 p0.parentNode 终值 null。
   **结论：父链置空走的是 `defineProperty(value:)` 覆盖路径**（R243/R245
   同款），绕过 accessor 陷阱与 removeChild——某处 defineProperty 把
   p0.parentNode 置 null/重写而未同步 testDiv.childNodes。

## 二、修复（land 部分）

`part06.js` surroundContents 的 R237 插入前新增 **显式 ChildNode.remove**
（`newParent.remove()`，Node.prototype.remove R238 泛型——wrapper/handle/
plain 三域通吃的 mutation-emitting 路径）+ try/catch 守卫：

- 动机：factory docEl insertBefore 的「从旧父摘除」按 identity 在
  wrapper 域可能 miss（旧父 childNodes 存 wrapper 而非本体）→ 摘除静默
  失败使 newParent 原件留旧父。
- 效果：本形态下 DIV 仍 6 P（探针 round 12 实测——幽灵 P 的真正写入者
  是 defineProperty 覆盖路径，非 insertBefore 的摘除 miss），守卫作为
  spec `concept-node-pre-insert` 的 adopt 步显式化 land（防御性正确）。

## 三、验证链（vs R245 基线）

| 项 | R245 | R248 | Δ |
|---|---|---|---|
| Range-surroundContents | 1806P/34F | 1806P/34F | 净 0（逐 subtest diff=0） |
| ranges 全量 | 40080 行 | 40080 行 | set-diff **0/0** |
| Range-insertNode | 1841P/0F | 1841P | 100% 保持 |
| dom/nodes 失败集 | 57 | 57 | 逐条一致 |
| make test | — | 17 suite ok + 1F XOpenDisplayFailed 环境项 | 已知项 |

- fmt/clippy（`-D warnings`）干净；wpt-data 探针全部还原（R248 标记 0）。

## 四、R249 靶点（幽灵 P 的 defineProperty 写入者）

- grep surround 链（part03/05/06）所有 `defineProperty(...,'parentNode',
  {value:` 写点，对「值变更时旧父 childNodes 未 splice」加同步摘除——
  幽灵 P 的确切机制（谁在 surround 期对 p0 做 defineProperty 覆盖）。
- 17,x 的 "[object Object]" 首差节点同源排查（isEqualNode 对含幽灵
  对象的树 walk）。
