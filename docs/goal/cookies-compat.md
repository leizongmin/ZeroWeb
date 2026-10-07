# Cookie 兼容 — document.cookie 语义面

**版本**: v1.0
**日期**: 2026-10-08
**状态**: Active（已立项，快赢切片——无用户门控项）
**执行模式**: WPT 驱动（上游 cookies corpus 为验收标尺）+ 语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——存储与状态面）

> **说明**
> 本文档是 ZeroWeb「Cookie 兼容」专项目标执行契约。document.cookie 是登录态/
> 偏好的基础面；net crate 既有 cookie jar（cookie_parse 为既有 bench 项、
> net-api 收口时 cookie-jar credentials 已打底），但 JS 面读写语义从未度量。
>
> **▶ 拆分动机（2026-10-08，GB-20261007 批复后快赢三连之二，用户「按照建议来」）**：
> ① cookie jar 本体已存在，JS 面是纯接线；② corpus 小，快赢节奏。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：
> - **jar 本体**：zero-net cookie jar（解析/传输既有测试）
> - **JS 面**：document.cookie getter/setter 语义（拼接序/过期/path/domain/
>     secure/httpOnly 可见性）未度量
> - **WPT corpus**：`cookies/`——**依赖 HTTP 响应头（Set-Cookie via .headers/.py）
>     的用例在静态语料 runner 上不可执行**，M1 盘点按可执行子集甄别并落 skip
>     规则（JS 面用例为主口径）

---

## Mission

以 **WPT cookies corpus 可执行子集为验收标准**，收敛 document.cookie 的
读写/属性/可见性语义，使登录态与偏好类站点的 cookie 行为正确。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **静态语料口径诚实**：HTTP 头驱动用例的不可执行性如实计入分母说明
   （wpt-suites.csv note 列），不静默丢弃
3. **jar 本体不动**：net crate 传输语义既有测试守卫，本 goal 只做 JS 面接线

覆盖范围：
1. **读写语义** — getter 拼接序（name=value; …）、setter 解析容差、覆盖语义
2. **属性面** — expires/max-age/path/domain/secure/httpOnly 对 JS 可见性与
   作用域的影响
3. **origin/权限** — 同源判定、document.cookie 异常（Secure 上下文等）

### 排除（明确不在范围内）

- **CookieStore API**（`cookie-store/` corpus）— 未实现的异步 API，挂账
- **网络传输语义**（请求附带/响应入 jar）— zero-net jar 本体既有域
- **SameSite/分区 cookie（CHIPS）** — 传输策略域，挂账待网络侧需求

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | document.cookie 桥面语义 | part 系 JS 语义 |
| zero-net | jar 查询/写入接口的 JS 面适配（本体不动） | 接线 |
| WPT 资产 | cookies corpus 导入 + 可执行子集甄别 | fetch 脚本 + 账本回填 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 webstorage-compat（本批同立）**：零交集可并行
- **与 net-api-compat（已归档）**：其 cookie-jar credentials 腿为消费基础
- **与 security-hardening（Active）**：Secure 上下文判定共用既有 secure-contexts
  语义，不重定义

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 导入与基线

- [ ] cookies corpus 导入 + 可执行子集甄别（不可执行面及理由落 evidence/）
- [ ] 分类通过率基线落 evidence/，并回填 docs/compat/trends/wpt-suites.csv planned 行

### DC-2: document.cookie 语义收敛

- [ ] 读写/属性/可见性语义对齐可执行子集口径，分级通过率不再下行两个连续轮次

### DC-3: 测试与质量不可退让

- [ ] 每个语义修复附带回归测试（单测或导入常驻断言集，imported-tests.txt 账本纪律）
- [ ] make test 全绿、make reftest 零回归、product-smoke welcome 逐字节恒值

---

## 活跃里程碑

### M1 — WPT 导入与基线（含可执行性甄别）

### M2 — document.cookie 语义收敛（DC-2 清单逐项）

### M3 — 收口（最终账本 + 归档）
