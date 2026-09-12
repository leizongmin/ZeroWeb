# 编码兼容 — 运行时控制面板（master.md）

**入口文档**: [../encoding-compat.md](../encoding-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-09-12（立项）

## 当前状态

轻量快赢切片：encoding 标签全表 + TextDecoder legacy 编码 + TextEncoder UTF-8。
与 html-syntax-compat 划界：文档级编码嗅探归其挂账 / JS API 面归本 goal。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | encoding/ corpus 导入 + 基线 | ⏳ M1 纯资产 |
| P2 | labels 标签匹配全表（数据化） | ⏳ M2 |
| P3 | TextDecoder legacy 编码解码（windows-125x/GBK/Shift_JIS/EUC-KR 等） | ⏳ M2 |
| P4 | TextEncoder + BOM/fatal/ignore 模式 + 编码往返 | ⏳ M3 |
| P5 | 文档级编码嗅探挂账定稿（html-syntax-compat 划界） | ⏳ M4 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：encoding/ fetch + 导入 + 基线（goals/30 编号脚本可跑 fetch 步）+ suites CSV 回填

**待用户决策清单**：（空——启动顺序由用户点名）
