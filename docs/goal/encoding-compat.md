# 编码兼容 — TextEncoder / TextDecoder / encoding 标签

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active（已立项，未启动——启动顺序由用户点名）
**执行模式**: WPT 驱动（上游 encoding/ corpus 为验收标尺）+ 语义修齐；
轻量快赢切片
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——编码面）

> **说明**
> 本文档是 ZeroWeb「编码兼容」专项目标执行契约。多语言页面（GBK/Shift_JIS/EUC-KR
> legacy 编码声明）依赖 encoding 标准的标签匹配与解码面。
>
> **▶ 拆分动机（2026-09-12 用户决策，第二批立项）**：① CJK 互联网仍有大量
> legacy 编码页面，解码缺位 = 乱码；② 面小、独立、零渲染碰撞，适合快赢出数字。
>
> **▶ 基线事实（2026-09-12 实测，js_dom_shim grep）**：
> - TextEncoder 14 处 / TextDecoder 25 处（zero-web P1a 落 UTF-8 面）
> - **legacy 编码基本缺位**：latin1 1 处 / gbk 1 处——encoding 标签匹配
>     （window-1252/GBK/Shift_JIS/EUC-KR/ISO-2022-JP 等全表）未建
> - **WPT corpora**：`encoding/`（testharness；labels 表 + 解码/编码往返 +
>     streams 接口面）

---

## Mission

以 **WPT encoding/ 真实用例为验收标准**，落地 encoding 标准的编码标签表、
TextDecoder 全编码解码面（legacy 编码含双字节）、TextEncoder（UTF-8）语义，
使 legacy 编码页面与多语言内容正确呈现。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **编码表数据化**：labels/encoding 映射照 encoding 标准 tables 落数据表，不散写
3. **与 font-wall 域无关**：解码正确性 ≠ 字形渲染（后者归 rendering-compat 字体栈）

覆盖范围：TextDecoder（UTF-8 + legacy 全表）/ TextEncoder（UTF-8）/ 编码标签
匹配（labels）/ BOM 语义 / 编码往返（encode↔decode）。

### 排除（明确不在范围内）

- **文档级编码嗅探（<meta charset / BOM 嗅探 → 文档解码）** —— html 解析域，
  与 html-syntax-compat 划界：本 goal 只做 JS API 面；嗅探面挂账
- **字体渲染乱码** —— rendering-compat font-wall 域
- **GB18030-2022 新增映射点** —— 上游标准滞后评估，启动时记账

---

## Support Envelope

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | TextEncoder/TextDecoder + labels 表 | part 系 JS 语义 |
| 资产 | encoding 标准映射表（数据化） | 单一事实源 |
| WPT 资产 | encoding/ 子集导入 | fetch 脚本 + 账本 |

**依赖约束（run-rules §9）**：与 html-syntax-compat —— 文档级嗅探归其/JS API 归本
goal 的划界线，双向记账；与其他 goal 无共享面。

---

## Done Criteria

- [ ] **DC-1**：encoding/ corpus 可执行子集导入 + 分类基线落 evidence/ + suites CSV
      planned 行转数据行
- [ ] **DC-2**：labels 标签匹配全表 + TextDecoder legacy 编码（windows-125x/GBK/
      Shift_JIS/EUC-KR/ISO-2022-JP 等）解码语义修齐，通过率可追踪提升
- [ ] **DC-3**：TextEncoder UTF-8 语义 + BOM/ fatal/ ignore 模式 + 编码往返修齐
- [ ] **DC-4**：`make test` 全绿 + clippy `-D warnings` + fmt + reftest 零回归

## 活跃里程碑

**M1** 导入基线（纯资产；corpus fetch 用编号脚本 `tests/wpt-runner/scripts/goals/30-encoding-compat.sh`（索引 README；幂等续拉，GitHub API 限流时部分目录跳过属预期））→ **M2** 标签表 + legacy 解码 → **M3** TextEncoder/模式
语义 → **M4** 收口（文档级嗅探挂账定稿）。

## Final Output Protocol

`DONE`（DC-1~4 全满足 + 挂账定稿）/ `CONTINUE: <下一步>`（默认）/ `BLOCK: <原因>`。

## Document Control / Archive Policy

入口文档实质变化才改；控制平面 `docs/goal/encoding-compat/master.md`；
archive/ 只追加；evidence/ 持续追加。
