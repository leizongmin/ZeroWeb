# Console 兼容 — console IDL 与格式化语义面

**版本**: v1.1
**日期**: 2026-10-14
**状态**: Active——M1 收口（2026-10-14），M2 语义收敛推进中
**执行模式**: WPT 驱动（上游 console corpus 为验收标尺）+ 语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——开发者面）

> **▶ 当前状态（2026-10-14，M1 收口）**
> - **M1 完成**：console corpus 通道（`testharness-console` 子命令 + Makefile
>     target）+ 可执行子集甄别 + 捕获通道结论落 evidence/。基线 **8/28 =
>     28.6%**（可执行面 8/23 = 34.8%；BiDi `log.entryAdded` 5 案 Unsupported
>     如实记账）。证据：[evidence/2026-10-14-m1-baseline.md](evidence/2026-10-14-m1-baseline.md)。
> - **M2 失败簇**：① WebIDL namespace IDL 面（空原型中间层 + `Symbol.toStringTag`，
>     5 子测试）；② count/time 家族 label ToString 转换 + 转换异常再抛（10 子测试）。
> - **卡点**：无（BiDi 捕获通道为 M2 后续切片，落地路径已定——shim 页内事件面
>     + testdriver `bidi` 白名单，不动 `__zw_console_log` 宿主回调契约）。
> - **下一步**：M2 切片 1（namespace IDL 面 + label 转换语义）→ 基线对比落
>     evidence/ → BiDi 捕获通道切片。

> **说明**
> 本文档是 ZeroWeb「Console 兼容」专项目标执行契约。console 是网页调试与
> 日志的基础 API；console.log/warn/error 面已存在，console 错误序列化刚由
> devtools 流 PR #101 收口，但 console/ corpus 的 IDL 与格式化语义从未度量。
>
> **▶ 拆分动机（2026-10-08，GB-20261007 批复后快赢三连之三，用户「按照建议来」）**：
> ① 面已存在、语料极小、趁 PR #101 上下文尚热；② 无渲染面/深结构交集。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：
> - **既有面**：console.log/warn/error/info shim；错误序列化（PR #101 `109ce5fa0`
>     instanceof 守卫 + 空栈空串回退）
> - **未度量面**：console/count/group/table/time/trace/assert/dir 格式化与
>     状态语义
> - **WPT corpus**：`console/`——依赖 console 检查器回调的用例需 runner 侧
>     console 消息捕获通道（M1 甄别可执行子集）

---

## Mission

以 **WPT console corpus 可执行子集为验收标准**，收敛 console API 的 IDL 面
（方法存在性/参数容错）与格式化语义（%c/%s/%d 占位、对象序列化口径），使
调试输出与日志采集行为正确。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **工作面协调**：console 消息捕获通道触 engine/page-runtime 面时，先与
   devtools 流按 run-rules §9 协调（PR #101 是其工作面），不单方面改契约
3. **格式化口径以 corpus 为准**：不引入 chrome 私有扩展面

覆盖范围：
1. **IDL 面** — 各方法存在性/最小参数容错（不抛异常语义）
2. **格式化** — 占位符替换、对象/数组/Error 序列化（与 PR #101 口径一致）
3. **状态语义** — count/group/time 家族的计数与栈语义（可执行子集口径）

### 排除（明确不在范围内）

- **devtools 面板呈现**（过滤/搜索/UI）— devtools goal 域
- **console 持久化/远程采集** — 未立项域，挂账
- **非标准扩展**（chrome 的 $0/inspect 等）— 不做

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | console 各方法语义/格式化 | part 系 JS 语义 |
| wpt-runner | console 消息捕获通道（如 M1 甄别需要） | 先与 devtools 流协调 |
| WPT 资产 | console corpus 导入 | fetch 脚本 + 账本回填 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 devtools（Active）**：console 序列化口径共用 PR #101 定义，改动先协调
- **与 webstorage/cookies（本批同立）**：零交集

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 导入与基线

- [x] console corpus 导入 + 可执行子集甄别（console 捕获通道需求结论落 evidence/）——2026-10-14，[evidence/2026-10-14-m1-baseline.md](evidence/2026-10-14-m1-baseline.md)
- [x] 分类通过率基线落 evidence/，并回填 docs/compat/trends/wpt-suites.csv planned 行——8/28 = 28.6%（可执行面 8/23 = 34.8%）

### DC-2: IDL 与格式化语义收敛

- [ ] 可执行子集口径收敛，分级通过率不再下行两个连续轮次

### DC-3: 测试与质量不可退让

- [ ] 每个语义修复附带回归测试（imported-tests.txt 账本纪律）
- [ ] make test 全绿、make reftest 零回归、product-smoke welcome 逐字节恒值

---

## 活跃里程碑

### M1 — WPT 导入与基线（含捕获通道甄别）✅ 2026-10-14

### M2 — IDL/格式化/状态语义收敛（DC-2 清单逐项）

### M3 — 收口（最终账本 + 归档）
