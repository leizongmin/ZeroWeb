---
date: 2026-10-28
modules: engine
---

# 解析产物属性必须工厂直灌，不走 setAttribute（R4100 形态）

## 问题

`_zwMBuildBodyTree` 新深臂（host 单次解析深 JSON 建本地树）用 `setAttribute`
逐个灌属性，`make test` 的 `test_details_name_exclusivity_r4100` 拦截：innerHTML
产物 details[name] open 互斥实测 `0,1,1`，期望 `1,0,1`（树序第一 open 胜出）。

## 根因

解析产物应**镜像 host 解析器已裁决的标记态**（如 R4100 的 open 互斥在解析器
侧按树序裁决），而 `setAttribute` 是**语义写入面**——attribute 写尾（R4100
others-prong：真开变迁关组内其他 open、`_zwMReflectIdl` id/cls 反射等）会在
构造期逐 attr 触发，后者覆盖前者，破坏 host 裁决态。

## 解决方案

- 建树路径一律走 `_zwMEl(snap)` 工厂：attrs 以 `{name:value}` 映射直灌本地
  attrs 数组 + `id`/`cls` 独立字段喂 IDL 反射——与浅臂 `_zwMBuildNode` 消费
  `__zw_parse_html_query` 快照同形。
- 映射用 `Object.create(null)`：`__proto__` 是合法属性名，普通对象赋值走原型
  setter 丢属性（host JSON.parse 产物是自有属性，可正确承载）。
- 判别法：写这段属性时若任何 attribute 写尾/IDL 反射会「做语义」，就是
  写入面；解析产物构造期要的是「落状态」不是「做语义」。
- 既有 `_zwMBuildDeepEntry`（WC-M2 template contents 路径）同为 setAttribute
  灌法，template contents 的 details open 互斥理论同暴露——2026-10-28 挂账
  encoding-legacy-mb M3 甄别。
