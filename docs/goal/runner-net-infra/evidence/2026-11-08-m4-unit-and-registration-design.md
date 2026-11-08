# M4 准备面：注册面 + 单测设计（DC-4 py 池门拆解）

日期：2026-11-08。前置 = 切片二
（[registry-head12](2026-11-08-m3-slice2-registry-head12.md)）落地的
`PyRoute`/`PyRouteMatch`/`PyRouteHandler` 面；门原文 = 入口文档
「DC-4 数值门落定」py 池节 + [py 普查档 §二](2026-11-08-m0-py-endpoint-census.md)
P-A/P-B。本档为 M4 实施设计，落地在 M3 迁移收口后执行（单测写入
py_routes 模块属共享面，按碰撞管理节仲裁）。

## 一、注册面（已落地面 + 新端点声明口径）

- **现状**：`py_routes()` 保序表即注册面——新端点 = `PyRoute { matches,
  handle }` 表项声明 + handler fn，不动 dispatch 循环（普查档 §三
  「新端点 = 注册项声明，不再动 dispatch 闭包」已达成）；谓词变体随
  批次补齐（Exact ✅ → DirSuffix/Prefix/Custom 随 xhr/尾段臂迁入）。
- **P-A 动态 stash 端点承接**（47 号三目录池：fetch/http-cache
  http-cache.py、fetch/stale-while-revalidate、fetch/origin
  redirect-and-stash.py）：端点 = 注册项 + ctx stash 表（PyEndpointCtx
  既有 12 项 Arc<Mutex> 表形态可扩展——per-case 键自清语义沿用 v3 M3
  cookie 表先例）；47 号只写端点表项，框架不动（普查档划界）。
- **P-B 判据回显端点承接**（39 号 referrer-checker.py 家族）：无跨请求
  stash，注册项 = Exact/Suffix 谓词 + 回显 handler；既有先例三变体
  （referrer-checker.py、echo-referrer.py）已在迁移序列内。

## 二、单测设计（DC-4 门：注册面 + 单测各 ≥1 例/族）

- **落位**：`#[cfg(test)] mod tests` 于 testharness_py_routes.rs 内随族
  就近；模块拆分（2000 行线——现 ~3.9k 行，迁移收口后按族拆子模块）
  时测试随族走。构建 = cargo test 既有门（make test 入口，test-guard
  包裹）。
- **P-A 族测点（≥1 例）**：cache.py 路由——请求带 token 参数 → 断言
  响应 ETag/状态 + stash 表写入副作用；再发条件请求 → 断言 304 路径
  （fixture 状态机语义抽测，不整案跑）。
- **P-B 族测点（≥1 例）**：referrer-checker-img.py 路由——
  expected_referrer 缺席 → 200 PNG；带参 → 404 文本（批次内已迁臂直接
  可测）；inspect-headers POST 豁免臂迁入后补方法豁免分支测例。
- **注册表语义测点**（表完备后）：Exact 不命中 → 落穿（需一个 None
  返回 handler，取自 xhr 族臂内条件不返回形态）；表序先命中先返回
  （两条重叠谓词臂序断言）。
- **测试基建**：handler 为 fn 指针 + ctx 可手工构造（fresh stash）——
  无需起 runner 进程；FetchRequest 构造复用现有测试 helper（如有）或
  最小字面构造。

## 三、M4 其余待办（非本档范围，登记备查）

- 四 fixture 零回归面落证（①websockets echo 族 ②S3b `/echo-query_v13`
  ③S12 cors-preflight-redirect ④inspect-headers POST 豁免）——迁移
  切片 A/B multiset 空集已逐批覆盖，M4 收口时按面引用切片档归集。
- wss 池批次门复验（M2 池 101 键口径不变则免重跑，树/语料漂移时重验）。
- 下游控制面互指（47 号 http-cache / 39 号 gap-batch1 归档入口声明
  「框架可消费」）——M4 收口提交统一办。
- 头条不回落（13,582P ≥ 13,275P ✓，归因档已锚）。
