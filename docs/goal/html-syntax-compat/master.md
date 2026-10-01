# HTML 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../html-syntax-compat.md](../html-syntax-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-09-12（立项）

## 当前状态

html-compat（已归档，fixture 制）的 WPT 化续篇：html/syntax + html/dom。
html5ever 底座不动，缺口在桥接/序列化侧；js-dom R373 parse-position 架构域
遗留问题为本 goal 输入。表单域（form-validation 已归档）跳过。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | html/syntax + html/dom corpus 导入 + 基线 | ⏳ M1 纯资产 |
| P2 | 解析树一致性（innerHTML/outerHTML/DOMParser）逐簇修齐 | ⏳ M2 |
| P3 | 序列化边缘（XMLSerializer/HTML serializer） | ⏳ M3 |
| P4 | html/dom 接口语义 + createContextualFragment 补面 | ⏳ M3 |
| P5 | 文档级编码嗅探（`<meta charset`/BOM 嗅探 → 文档解码，document.characterSet）——encoding-compat goal M4 转入（双向记账，2026-10-02）：其 JS API 面（TextDecoder/labels/legacy 解码 99.96%）已收，document 解码面归本 goal | ⏳ 与 P1 corpus 同批（encoding/ 域 bom-handling/eof-*/utf-32*/sniffing 案面已拉至 wpt-data，可随本 goal M1 一并基线） |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：corpus fetch + 导入 + 基线（goals/40 编号脚本可跑 fetch 步）+ suites CSV 回填

**待用户决策清单**：（空——启动顺序由用户点名）
