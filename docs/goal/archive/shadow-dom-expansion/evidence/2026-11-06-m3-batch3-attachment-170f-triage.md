# M3 Batch 3 — attachment.html 170F 甄别收口

**日期**: 2026-11-06
**前置**: Batch 2 后 declarative-shadow-dom-attachment.html 0P/654F → 484P/170F
（goal doc 附加面外溢行）；本甄别用探针法证把 170F 收敛为两个机制面 + 一个
gethtml 通过性虚构的发现。

**测量口径**: 二进制 = 05:56 build6（含 Batch 2 38b972751 + t8c WIP 当时态 +
看门狗随乘），**早于 d8f473ff8**（簇 C dom 层先行 + 物化宿主名单 VALID_SHADOW_HOST_NAMES
落在其后）；下列行为均以该二进制为准，现树 WIP 已含宿主名单、复测待其落地。

## 失败簇分布（当前态复跑 654 行 = 484P/170F，与 Batch 2 账持平）

- **safelisted 臂 68F** = 17 元素 × {open,closed} × {df false,true}，
  断言 `assert_true(!wrapper.querySelector('template'))`（模板该消未消）。
- **disallowed 臂 102F** = 25 元素 × {open,closed,invalid} × {df false,true}，
  断言 `assert_true(!!template)`（模板该留不留）。
- nochildren 元素（iframe/noscript/script/select/style/textarea/title）断言
  跳过，不在失败集 ✓。

## 机制一：safelisted 臂——setHTMLUnsafe 后声明式模板**从不物化**（引擎+视图双端）

探针（zz-attach2-probe）逐形状取证：

- oracle 同形（`<div><span>light</span><template shadowrootmode=open>` 尾置未闭，
  L1/L5/L7）与闭模板内嵌形（L2）、首子闭模板（L4 前身 S4/S8/S9）——setHTMLUnsafe
  后 `querySelector('template')` 恒命中、`el.shadowRoot` 恒无；**getHTML/innerHTML
  读取不触发物化**（惰性假设否决）。
- oracle 通过的真相（L5 verbatim 复刻）：`firstElementChild.attachShadow(initDict)`
  在模板未物化的宿主上**新建空命令式 SR**（真实浏览器同参 attachShadow 应返回
  声明式根并带模板内容），oracle 随后 append slot + getHTML 序列化恰与期望串
  吻合（序列化器藏掉残留模板——L5 after-attach 时 el 子节点含 TEMPLATE 但
  getHTML 输出无 template）→ **gethtml.html 声明式象限的通过是「模板滞留 +
  attachShadow 重建 + 序列化遮蔽」虚构链**，Batch 2 的「双端物化」实际未发生
  在引擎树。attachment.html 直接断言模板缺席 + shadow 内容，戳穿该链。

## 机制二：disallowed 臂——context 父模板被错误物化消费（102F 主体）

- void/非法父标签形（area/br/param + 解析丢弃的 caption/head/html/tbody/td）：
  模板父级 = 片段 context 元素（frag_doc 合成 body，∈ 宿主名单）→ 旧二进制
  物化消费模板（wrapper.innerHTML 中模板消失、light span 留光干）。上游 spec：
  **context 元素上的声明式根不附着**（测试期望模板保留 + getAttribute 断言）。
- canvas 形（宿主为解析元素但非 valid 名单）：旧二进制**无宿主名单**，模板与
  其后内容一并被吞（wrapper.innerHTML 仅剩空 canvas）——同根：物化无约束。
- 修复方向：物化 pass 增加 context 元素排除（片段解析调用点透传 context 身份，
  全文档解析不受影响）+ valid shadow host 名单（现树 WIP 已见 VALID_SHADOW_HOST_NAMES
  于 shadow.rs:163，context 排除待核）。

## 修复归属与协同（run-rules §9）

两机制修复面均在 `crates/dom`（document/shadow.rs 物化 pass + parser/declarative_shadow.rs
预变换 + shim 视图臂对齐）= 簇 C 流（m3c2）活跃编辑面——本甄别按 §9 只定位不落地：
①待 m3c2 落地后重建复测 attachment.html，量化其名单/context 处理对 170F 的覆盖；
②残余（setHTMLUnsafe 即时物化 + attachShadow 返回声明式根语义）在其面上排期。
附件面与簇 C referenceTarget 同文件高碰撞，先手归档本账供其取用。

## 探针留存

wpt-data 下 zz-attach-probe.html / zz-attach2-probe.html（形状×时序取证，
gitignored 研删产物，不入库）。原始 JSON：/tmp/m3b2-attach*.json（会话内一次性，
未归档）。
