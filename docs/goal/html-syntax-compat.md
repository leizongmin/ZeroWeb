# HTML 文档面兼容 — html/syntax / html/dom

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active（已立项，未启动——启动顺序由用户点名）
**执行模式**: WPT 驱动（上游 html/syntax + html/dom corpus 为验收标尺）+ 语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——HTML 文档面）

> **说明**
> 本文档是 ZeroWeb「HTML 文档面兼容」专项目标执行契约。解析底座为 html5ever
> （crates/dom），预期基线不低；价值在用 WPT 把解析树一致性、DOM 接口边缘语义
> 度量化，并收编已归档 html-compat（fixture 制）之后的 WPT 化延续。
>
> **▶ 拆分动机（2026-09-12 用户决策，第二批立项）**：① html-compat（已归档）
> 是仓内 fixture 验收，未对齐上游 WPT 语料——本 goal 为其 WPT 化续篇；②
> innerHTML 183 处/DOMParser 18 处面已存在，度量基线即可定位缺口。
>
> **▶ 基线事实（2026-09-12 实测，js_dom_shim grep）**：
> - innerHTML 183 处 / insertAdjacentHTML 14 处 / DOMParser 18 处 / XMLSerializer
>     3 处——解析面存在（html5ever 底座）
> - createContextualFragment 0 处——Range 解析上下文面缺
> - **WPT corpora**：`html/syntax/`（解析树 + serializer）、`html/dom/`
>     （DOM 接口语义面）；html/semantics/forms 已归 form-validation 不重复

---

## Mission

以 **WPT html/syntax + html/dom 真实用例为验收标准**，度量并收敛解析树
一致性（innerHTML/DOMParser/序列化往返）与 DOM 接口边缘语义，把 html-compat
的 fixture 制验收升级为上游 WPT 持续账本。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **html5ever 底座不动**：解析器核心为外部 crate，缺口在桥接/序列化侧修
3. **表单语义不重复**：constraints 面已归 form-validation（已归档），跳过其域

覆盖范围：解析树一致性（innerHTML/outerHTML/DOMParser）/ insertAdjacentHTML /
序列化（XMLSerializer + HTML serializer 边缘）/ html/dom 接口语义
（Document/Element 全接口面）/ createContextualFragment 补面。

### 排除（明确不在范围内）

- **表单约束校验** —— form-validation goal 已归档收口
- **文档级编码嗅探** —— encoding-compat 挂账域（双向记账）
- **script 执行语义** —— zero-web P1a + js-dom 域（已归档）
- **html/webappapis（timers/event loop 面）** —— event-loop-spec 已归档 +
  timing-animation-compat 覆盖，不重复立项

---

## Support Envelope

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| dom | 序列化器/解析桥边缘语义 | html5ever 核心不动 |
| engine shim | innerHTML/insertAdjacentHTML/DOMParser/createContextualFragment | part 系 JS 语义 |
| WPT 资产 | html/syntax + html/dom 子集导入 | fetch 脚本 + 账本 |

**依赖约束（run-rules §9）**：与 form-validation（已归档）表单域跳过；与
encoding-compat 嗅探划界双向记账；与 js-dom（已归档）DOM 接口已收口处不重复立账，
其 parse-position 架构域遗留问题（R373）为本 goal 输入。

---

## Done Criteria

- [ ] **DC-1**：html/syntax + html/dom corpus 可执行子集导入 + 分类基线 +
      suites CSV 回填
- [ ] **DC-2**：解析树一致性（innerHTML/outerHTML/DOMParser）逐簇修齐
- [ ] **DC-3**：序列化边缘 + html/dom 接口语义 + createContextualFragment 补面
- [ ] **DC-4**：`make test` 全绿 + clippy `-D warnings` + fmt + reftest 零回归

## 活跃里程碑

**M1** 导入基线 → **M2** 解析树一致性 → **M3** 序列化/接口面 → **M4** 收口。

## Final Output Protocol

`DONE`（DC-1~4 全满足）/ `CONTINUE: <下一步>`（默认）/ `BLOCK: <原因>`。

## Document Control / Archive Policy

入口文档实质变化才改；控制平面 `docs/goal/html-syntax-compat/master.md`；
archive/ 只追加；evidence/ 持续追加。
