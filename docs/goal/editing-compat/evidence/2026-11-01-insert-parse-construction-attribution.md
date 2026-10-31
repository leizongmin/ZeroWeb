# 插入侧解析残余归因：host 桥与 JSON.parse 免费，99% 在 `_zwMEl` 逐实例安装面——无隐藏二次项，结构性收口归并深水轮

**日期**: 2026-11-01
**前置**: [CE 快路轮 evidence §4 插入侧残余地图](2026-10-31-insert-side-attribution-and-ce-entry-fast-path.md)首项——bulk fill 本地解析（seg55）~21µs/span，「解析器优化，独立轮（全 innerHTML 面共享，爆炸半径大）」。
**结果**: tape v4/v4b 两级细分定谳——seg55 的 ~42ms（2000 span，21µs/span）**99% 在深树 wrapper 构建段**：host 桥 `__zw_parse_html_subtree` 0-1ms + `JSON.parse` 0-1ms ≈ 免费；构建段 36-49ms 均匀分布于 `_zwMEl`（~2,350 行逐实例安装构造器，18069-20416 行）全程，无单一致 vignette（最大单区 mutation/attr 方法族 ~8-13ms，占 ~25%）。`_zwBridgeSet` 为 O(1) Map（4096 清空上限），无 ring-registry 型隐藏二次项。**本轮不改代码**：安全切片（预建描述符消闭包，仅 boolean-reflected + title/lang/dir 两区）估算收益 2-9ms/窗 ≈ 墙钟 1-5%，在噪声带——按「未兑现机制不写收益注记」纪律不落；结构性收益（20µs→~7µs/元素）需全表面 this 化改造 = 45+ 方法体等价审查 = 深水轮，与 t8r 物化面（seg54，同为逐 span wrapper 构建）合并立项。

## 1. 方法（tape v4 两级细分，测后全剥）

沿前轮 tape 方法（`__zwTkReset/__zwTk/__zwTkDump` 全局 + `Date.now()` 顺序段累计），
两级插桩：v4 在 `_zwMBuildBodyTree` 内部四点（p0 入口 / p1 host 桥前 / p2 host 桥后 /
p3 JSON.parse 后 / p4 构建循环后）+ `_zwMBuildDeepTreeNode` 条目计数；v4b 追加 14 点
（m0-m13）纵切 `_zwMEl` 全函数。探针复用 `zz-probe-s5-insert.html` w0（zz-s5 =
fresh div `innerHTML=` 2000 span，单次解析调用），3 样本 ×2 轮。语法门 = parts 1-7
顺序拼接 `node --check`。测完 `git checkout` 全剥（`grep -c __zwTk` 归零复核）。
v4b 插桩自身开销 ~28ms/窗（2000×14 次 Date.now），区域占比按相对份额读。

## 2. 归因（tape v4，3 样本×3 轮）

| 段（zz-s5 窗累计） | 3 轮样本 | 判读 |
|---|---|---|
| wall | 162/135/111 · 158/130/114 · 154/128/108 | 与 CE 快路轮 fill 136ms 同带 |
| p0→p1（concat + body 容器） | 1-3ms | ≈0 |
| p1→p2（host `__zw_parse_html_subtree`） | **0-1ms** | 免费 |
| p2→p3（`JSON.parse` 深树 JSON） | **0-1ms** | 免费 |
| p3→p4（深树构建 `_zwMEl`/`_zwMText` 递归） | **36-49ms** | **~99%**（el=2000/tx=0） |
| 兜底浅臂 | 未走（p2b/p2c 零命中） | — |

上轮地图「本地解析 ~21µs/span」的承担者据此钉死为 **wrapper 逐实例安装**，非解析/
传输——host Rust 解析器与 JSON 通道均无优化空间（合计 <2ms）。

## 3. tape v4b：`_zwMEl` 全函数 14 点纵切（3 样本，v4b 二进制 wall 161-204 含插桩开销）

| 区（检查点间） | 样本 ms | 内容 |
|---|---|---|
| m1→m2（proto 查表 + setPrototypeOf） | 2 | — |
| m3→m4（title/lang/dir forEach×3 define） | 1-6 | 预建描述符可切片 |
| m4→m5（boolean reflected 11 define） | 1-6 | 预建描述符可切片 |
| m7→m8（事件三方法闭包） | 3 | — |
| **m9→m10（mutation/attr 方法族：focus/blur/prepend…remove/setAttribute/removeAttribute/NS 族）** | **10-16** | 最大单区 ~25% |
| m10→m11（outerHTML setter + attributes NamedNodeMap 视图 + removeChild） | 2-8 | — |
| m11→m12（appendChild/insertBefore/replaceChild/textContent/innerHTML 序列化族 define） | 4-9 | — |
| m12→m13（id/className IDL accessor 化 + `_zwBridgeSet` + tag 门控尾段） | 6-24 | 高方差 |
| 其余（literal/`__zwIsMEl`/ownerDocument/get·hasAttribute/hasAttributes 族） | 0-4 | — |

三点判读：

1. **成本分布式**：`_zwMEl` 是 ~2,350 行逐实例安装构造器（约 25 次
  `defineProperty` + 约 100+ 闭包 + setPrototypeOf + 桥登记/元素），每 span 走完全
   程；无单一致 vignette——与 ring-registry/S8（单一 villain 可切）不同型。
2. **无隐藏二次项**：`_zwBridgeSet` = Map O(1)（4096 上限 clear），`_zwMText` 空扫
  （tx=0 本窗），构建段对 span 数线性。
3. **安全切片收益在噪声带**：唯二无语义变化的切片（m3→m4 + m4→m5 预建描述符/共享
   this 形函数，own 描述符语义逐位保留，等价边仅描述符 getter `.call` 劫持——无已知
   消费面）估算 2-9ms/窗 = 墙钟 1-5%，与 trap 读复用同款噪声带——不落。

## 4. 决策与重排

- **结构性收益的真实刀位** = wrapper 安装面共享化：基座成员 this 化 + 预建描述符一
  次安装（估算 20µs→~7µs/元素，窗墙钟 −10% 上下）或共享原型全 hoist（更彻底，但
  own/inherited 描述符语义变——`getOwnPropertyNames`/`hasOwnProperty`/
  `getOwnPropertyDescriptor` 可见面，js-dom 语料可断言）→ 两者都要求 45+ 方法体
  （含 R167 链派发中转站、R3018/R4100 属性写尾、R262/R208 removeChild 义务链）逐体
  等价审查——**深水轮**，与既有 HCLiveInvalidate（R55/R304 身份稳定）同风险级。
- **地图重排**：seg55（解析侧构建）与 seg54（t8r 物化 `_wcRebuildAsHandle`，同为逐
  span wrapper 构建、同样吃逐实例安装面）**合并立项**为「wrapper 安装面」单一深水
  轮——一次审计覆盖两侧 ~42µs/span，审计清单：own-property 可见面 gPN/hOP/descriptor
  三查 + R55/R304 身份稳定 + R309 视图语义 + R167 链派发中转站 `_zwMEvLs` 双引用。
- seg55「解析器优化」原 scoping 关闭（解析器与通道无成本可收）。

## 5. 复现入口

- 探针页：`tests/wpt-runner/wpt-data/editing/run/zz-probe-s5-insert.html`（w0 =
  zz-s5 主窗）；`make testharness-editing-run-one FILTER=zz-probe-s5-insert TIME_LIMIT=600`。
- tape 复建：§1 两级检查点清单（`_zwMBuildBodyTree` p0-p4 + `_zwMEl` m0-m13 锚点行）。
