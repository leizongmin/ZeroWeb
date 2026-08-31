# R388 — MutationObserver-document 剩余 2F 归因合并（诊断轮）+ M4 全量 sweep 基线刷新

**日期**: 2026-08-31
**HEAD**: `8a28c0f8e`（R387 收官态，零源码改动轮——R354/R378 先例）
**性质**: ① R387 遗留定性勘误（探针实证）② M4 全量 sweep 基线刷新

---

## 1. R387 定性勘误：previousSibling 失败不是 L2 身份域

R387 把 "parser script insertion mutation" 的剩余失败定性为「L2 身份域（selector 规范化）」。
本轮探针证伪：

- **身份面无问题**：`childNodes` 条目对带 id 的解析元素**本就携带 `#id` selector**
  （`node_entry_json → unique_selector_for_node` 优先 `stable_selector_for_node` 的 id 分支），
  探针 `SCRIPT:#s001 , P:#n00 , SCRIPT:#s002 , byIdSel:#s002 , ident:true`——包装与
  `getElementById` 产物同一 proxy（`_proxyCache` 命中）。
- **真实差距是 parse-position 可见性**：复刻 WPT 用例流程（createElement script +
  textContent + appendChild → 动态执行 → body.appendChild(newElement)），实测
  `inserted_element.previousSibling` = `#n01`（**全预解析树的 body 末元素**），而 WPT 期望
  `#s002`——流式解析下 script 执行时 `s002` 之后的元素**尚不存在**。record 的 prev 是
  「树真相」，WPT 期望「解析位置真相」。R385 报错信息里的 positional wrapper
  （`script:nth-child(10)`）是 late-tree 元素（s012 邻域）的位置路径形态，非 selector
  规范化缺失。

**结论**：MutationObserver-document 剩余 2F（script-insertion 的 previousSibling +
removal 的 nextSibling）**共享同一根因**——JS 观察到全预解析树而非增量解析状态
（R373 定性的 parse-time 架构域）。修复需「解析位置面」（按解析位置屏蔽后续元素的
兄弟/树查询计算），波及所有树查询路径，深结构，维持挂账不硬解。

## 2. M4 全量 sweep 基线刷新（R387 后）

命令 `make testharness-dom TIME_LIMIT=2000`（test-guard 包裹，空窗 load1≈0.3 起跑）：

| 指标 | R385 基线 | R388 刷新 | 变化 |
|------|-----------|-----------|------|
| Pass | 55,808 | **55,808** | 恒等（R387 的 +1 在 dom/nodes 内部：document 文件 3F→2F，同 P 数） |
| Fail | 12（7 文件） | **11**（6 文件） | **-1**（document parser-insertions 转绿） |
| Timeout | 15 | 16 | 文档化轮转族（Node-parentNode / insertBefore-iframe-crash 等，单跑复验 Pass 先例） |

Fail 集合（11，全部定性挂账）：document 2F（parse-position 域，见 §1）+ remove-and-adopt-thcrash
（window.open 环境基建）+ click-on-absolute-pseudo（Chromium 专有）+ Range-mutations
dataChange/replaceData 2F（R353 游离树堆积域）+ historical 3F（stale 期望）+
window-extends-event-target 2F（EventTarget 继承域转档）。

## 3. 对控制面的修正

- master.md「剩余 2F 定性」由「L2 身份域 + parse-position 架构域」**修正为两项同属
  parse-position 架构域**（单一根因，合并挂账）。
- R388 任务线（L2 身份域窄面切片）撤销——假设被探针证伪，无代码改动。

## 4. 验证

- engine v8 2510P（诊断插桩加/删后全量复验）；工作树零残留（诊断代码未提交）。
- 全量 sweep 门如 §2（Exit 1 为既有 Fail 集合的预期门禁信号，非新增）。
