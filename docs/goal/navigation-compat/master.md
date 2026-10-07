# 导航与浏览上下文兼容 — 运行时控制面板（master.md）

**入口文档**: [../navigation-compat.md](../navigation-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-10-08（M2-S1 location 86.1% + M2-S2 traverse 93.3%；全量 25.9%→29.2%）

---

## 当前状态

**专项定位**：会话历史/导航事件（轻面，独立推进）+ iframe 浏览上下文（深面，
**用户门控切片**，先例 R1043/Phase A IFC）。轻面可不等门控独立收口出数字。

**与兄弟 goal 的边界**：
- rendering-compat — viewport/滚动/渲染面不碰（style-system/layout-engine/render-foundation
  不在 envelope）；iframe 渲染面缺口记账回流
- event-loop-spec（已归档）— IO/RO 与事件循环遗产为消费基础
- zero-web P1a — location 读侧已落；写侧导航语义归本 goal
- zero-protocol / 多进程 — fission 排除；M3 触进程模型即 BLOCK 上报用户

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 四 corpus 导入 + 基线 | ✅ 落地（fetch 三域 + 嵌套 resources 2026-10-07 恢复后入库；基线 20.9%，S1+S2 后全量 29.2%） |
| P2 | history pushState/replaceState/state/length/back/forward/go 语义 | 🔶 the-history-interface 81.6% 持平（余 8F 集中在 navigation 重载/跨源 SecurityError 面） |
| P3 | popstate/hashchange 事件序 + location 写侧导航语义（带重入 guard） | 🔶 S1+S2 落地：location 41.9%→**86.1%**、traversal 62.2%→**93.3%**；余 exotic 内部方法面（preventExtensions/setPrototypeOf）+ Navigation API 面 |
| P4 | iframe 浏览上下文最小面（contentWindow/frames/parent/top + 属性语义） | ⏳ M3 **用户门控** |
| P5 | bfcache / fission 挂账定稿 | ⏳ M4 |

## 已完成切片

- **M1 runner 通道 + 首轮基线（2026-10-07）**：`testharness-navigation` 子命令 +
  `navigation_case_skipped` 筛减规则 + Makefile target；基线 309 案 387 子测试
  81 Pass = 20.9%。证据：[evidence/2026-10-07-m1-baseline.md](evidence/2026-10-07-m1-baseline.md)。
- **M2-S1 Location 接口语义（2026-10-08）**：Location [LegacyUnforgeable] 面 +
  `window.Location` 接口对象 + port + 写侧 SYNTAX_ERR + Document.location dedup +
  runner 绝对路径 helper 通道（stringifiers.js）。location 41.9%→**86.1%**；余 5F =
  exotic 面 ×3（FIXME 已标）+ runner 无端口 URL ×1 + Navigation API ×1。
  证据：[evidence/2026-10-08-m2-s1-location-interface.md](evidence/2026-10-08-m2-s1-location-interface.md)。
- **M2-S2 traverse 事件序（2026-10-08）**：fragment navigation 同步 popstate +
  isTrusted（`__zwTrusted` 口）+ PopStateEvent hasUAVisualTransition/非 new TypeError +
  scrollRestoration per-entry + runner 顶层 `let` accessor 转发导出（R201 同款，
  before-load-hash 族根因）。traversal 62.2%→**93.3%**、event-order 12/13。
  证据：[evidence/2026-10-08-m2-s2-traverse-events.md](evidence/2026-10-08-m2-s2-traverse-events.md)。
- **质量门禁（两切片合并跑）**：`make test` 20,180 P / 0 F 全绿；clippy -D warnings
  零 warning；fmt 零 diff；全量语料零基线回归（M1 81 Pass 案全保持）。

## 下一步计划

1. **M2 ③ 跨文档导航语义**：navigating-across-documents 4.8%（2/42）——load/unload
   事件序与跨文档 URL 归属；依赖跨文档导航链（单文档 runner 形态限制，先评估
   可达子集再动）。
2. **M2 ④ Navigation API 评估切片**：0.4%（1/228），`window.navigation` 全域缺席——
   按 master 前议独立评估（量级 ~196 子测试），排在 ③ 后或并行。
3. **M2 ⑤ exotic 内部方法面评估**（Location [[PreventExtensions]]/[[SetPrototypeOf]]）：
   影响 [LegacyUnforgeable] 接口建模（Proxy vs 引擎层 exotic 支持），跨切片收益
   （location 3F + 后续 Window/Document 同族面）。
4. **M3**：frame tree 最小面——**启动前须用户点名批准**（2026-10-04 已征询待批复，
   维持挂起，见下）。

**待用户决策清单**：
- [ ] M3 iframe 深结构切片启动授权（未获批期间 DC-3 保持 pending，不阻塞 M2 收口）
  **2026-10-04 已征询待批复**（goal 待决策巡检 msg `om_x100b63125d66c4a8b2070829b8dd058`；
  建议 = 暂不批准——M1/M2 尚未启动，待 M2 收口时随推进一并裁决）。批复前维持立项态挂起。
  **2026-10-06 48h 跟进提醒已发（一次性，到期）**（msg `om_x100b63647f7910a0b49d70def6b88ff`）：
  仍零回复——不回复即默认维持挂起，此后不再重复催办。
