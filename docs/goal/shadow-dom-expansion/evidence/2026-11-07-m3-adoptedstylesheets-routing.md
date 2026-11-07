# M3 清单 2——adoptedStyleSheets 残面归口 paperwork（2026-11-07）

## 口径（WC testharness 通道，`wc-after-fmtfix.json` 同基线实测）

`shadowrootadoptedstylesheets` 族在跑用例 **9 文件 5P/23F/0T**（module
载入链家族——async-fetch/modulepreload 系——已按 M3 簇 D 剔池在案，不在
本口径）：

| 文件（declarative/tentative/shadowrootadoptedstylesheets/） | P | F |
|---|---|---|
| basic | 0 | 2 |
| clonenode | 2 | 9 |
| idl-absent-and-empty | 1 | 1 |
| idl-feature-detection | 0 | 1 |
| idl-reflection | 0 | 2 |
| invalid-specifier | 0 | 2 |
| multiple | 0 | 2 |
| same-document | 1 | 0 |
| serialization | 1 | 3 |
| **合计** | **5** | **23** |

## 归口结论

**机制面**（按 F 消息簇）：

1. **CSSOM 构造样式表缺失**——`shadowrootadoptedstylesheets` 具名
   specifier → `ShadowRoot.adoptedStyleSheets` 数组恒 0（basic/clonenode
   的 sanity 断言 `expected 1/2 but got 0` 全簇）。需要 constructable
   StyleSheet 构造与 specifier→sheet 解析链。
2. **adopted 样式应用管线（渲染耦合主因）**——`adoptedStyleSheets` 内
   sheet 对 shadow 树内容的样式应用
   （`expected "rgb(0, 0, 255)" but got ""`）：computed style 管线需消费
   adopted sheet 列表，属渲染管线挂接面。
3. **IDL 反射面**——`shadowRootAdoptedStyleSheets` IDL ↔ content
   attribute 双向反射缺失（idl-reflection 2F）。
4. **下游连锁**——clonenode 9F 中多数为 sanity（①②）失败后的级联断言
   （克隆再处理/保序/序列化回写），根面同①②。

**渲染耦合原因**：第 2 簇必须 stylesheet 进 cascade/computed style 才可
断言，runner 无该管线；第 1 簇的 CSSOM 构造样式表与渲染共享 sheet 基础
设施。两簇均非 JS 语义单点可修。

**去向**：rendering-compat（挂账条目见其控制面 2026-11-07 留痕 bullet）。
42 号（本 goal）不再实现该族。

**45 号边界核对**：`shadow-dom-rendering.md` 划界明文
「constructable stylesheets / CSSStyleSheet.replaceSync 不在本 goal——
M0 甄别后具名挂账」，本归口与其一致，无双改风险。

## 关联

- 42 号契约清单 2（M3 收口清单第 2 项）的完成留痕。
- 测量数据：`.acceptance/shadow-m3-residual/wc-after-fmtfix.json`
  （365 文件全语料；M4 冻结复测后如口径漂移以冻结值为准回写本表）。
