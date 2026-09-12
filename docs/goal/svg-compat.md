# SVG 文档面兼容 — svg/

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active（已立项，未启动——启动顺序由用户点名）
**执行模式**: WPT 驱动（上游 svg/ corpus 为验收标尺）+ 语义修齐；
**SVG-as-渲染面归 rendering-compat**，本 goal 只做 SVG 文档模型/交互 JS 面
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——SVG 文档面）

> **说明**
> 本文档是 ZeroWeb「SVG 文档面兼容」专项目标执行契约。当前 SVG 能力 =
> **usvg 静态栅格化**（image/背景图路径，rendering-compat 域）；SVG 作为
> **文档**（`<svg>` 内联 DOM 树/SVG 元素接口/交互几何）几乎全缺。
>
> **▶ 拆分动机（2026-09-12 用户决策，第二批立项）**：① 内联 SVG 是现代页面
> 图标/插图的通用载体，SVG DOM 面缺位使交互式 SVG 不可用；② 与 rendering-compat
> 的 svg-as-image 修复面（R4000/R4128 谱系）划界清晰，可独立推进。
>
> **▶ 基线事实（2026-09-12 实测，js_dom_shim grep）**：
> - **SVG 接口面几乎全零**：SVGSVGElement 2 处 / createSVGPoint 0 /
>     SVGGeometryElement 0 / getScreenCTM 0——文档模型未建
> - DOMParser 18 处 / XMLSerializer 3 处——XML 解析底座可复用
> - rendering 侧：usvg 0.47 栅格化已接（R4104-R4110 transform 谱系，
>     rendering-compat 账本）
> - **WPT corpora**：`svg/`（SVG DOM/几何/交互；大量用例含渲染断言——
>     runner skip 规则启动时定）

---

## Mission

以 **WPT svg/ 真实用例为验收标准**（先 DOM/几何面，渲染断言面如实标注），
落地 SVG 文档模型最小面（SVGElement/SVGSVGElement/几何元素接口 + viewBox/
CTM 计算），使内联交互式 SVG（图标系统/数据图）可用。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动；corpus 内渲染断言面占比启动时盘点
2. **文档模型 ≠ 渲染**：SVG 元素的呈现仍走现有渲染管线；本 goal 是 DOM/几何接口
3. **XML 底座复用**：DOMParser/XMLSerializer 既有面为解析基础

覆盖范围：SVGDocument/SVGElement 类层级 + SVGSVGElement（viewBox/getScreenCTM
最小面）/ SVGGeometryElement（getTotalLength/getPointAtLength）/ currentTranslate
/ SVG 元素事件目标化。SMIL/动画评估记账。

### 排除（明确不在范围内）

- **SVG 渲染正确性** —— rendering-compat 域（usvg 栅格化/背景图路径）
- **SMIL 动画** —— 评估后挂账
- **SVG-in-CSS（mask/clip-path 引用）** —— rendering-compat M9 已有面，不重复
- **webgpu/webgl 上下文** —— webgl-compat 域

---

## Support Envelope

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | SVG DOM 接口面 | part 系 JS 语义 |
| dom | SVG 元素类层级标注 | 不改 parser 核心 |
| WPT 资产 | svg/ 子集导入（DOM 面优先，渲染面 skip 规则启动时定） | fetch 脚本 + 账本 |

**依赖约束（run-rules §9）**：与 rendering-compat —— 渲染面/样式面不碰
（style-system/render-foundation 不在 envelope）；与 html-syntax-compat ——
XML 序列化底座共享（DOMParser），改动互核；与 uievents-compat —— SVG 事件
目标化消费其事件语义，无协议耦合。

---

## Done Criteria

- [ ] **DC-1**：svg/ corpus 可执行子集导入（DOM 面清单优先）+ 分类基线 +
      suites CSV 回填；渲染断言面占比盘点入档
- [ ] **DC-2**：SVG DOM 类层级 + SVGSVGElement viewBox/CTM 最小面修齐
- [ ] **DC-3**：SVGGeometryElement 几何接口 + 事件目标化修齐；SMIL 评估记账
- [ ] **DC-4**：`make test` 全绿 + clippy `-D warnings` + fmt + reftest 零回归

## 活跃里程碑

**M1** 导入基线（含渲染面占比盘点）→ **M2** SVG DOM 类层级 → **M3** 几何接口/
事件 → **M4** 收口（SMIL/渲染断言面挂账定稿）。

## Final Output Protocol

`DONE`（DC-1~4 全满足 + 挂账定稿）/ `CONTINUE: <下一步>`（默认）/ `BLOCK: <原因>`。

## Document Control / Archive Policy

入口文档实质变化才改；控制平面 `docs/goal/svg-compat/master.md`；
archive/ 只追加；evidence/ 持续追加。
