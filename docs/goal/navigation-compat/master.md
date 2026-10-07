# 导航与浏览上下文兼容 — 运行时控制面板（master.md）

**入口文档**: [../navigation-compat.md](../navigation-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-10-07（M1 runner 通道落地 + 首轮基线 20.9%；三域 fetch 网络挂账）

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
| P1 | 四 corpus（html/browsers + history + navigation-api + iframe）导入 + 基线 | 🔶 主体落地（770 案在库 / 基线 387 子测试 20.9%）；scroll-to-fragid + unloading-documents + the-iframe-element 与嵌套 resources 因 GitHub 网络间歇中断未落地（fetch 脚本已补列 DIRS，幂等续拉） |
| P2 | history pushState/replaceState/state/length/back/forward/go 语义 | 🔶 部分在位（the-history-interface 81.6%）；Location 接口面 / hashchange 赋值序为缺口 |
| P3 | popstate/hashchange 事件序 + location 写侧导航语义（带重入 guard） | ⏳ M2（基线已定位：traverse 事件序断言 / location 同步导航 `expected "baz" but got "foo"`） |
| P4 | iframe 浏览上下文最小面（contentWindow/frames/parent/top + 属性语义） | ⏳ M3 **用户门控** |
| P5 | bfcache / fission 挂账定稿 | ⏳ M4 |

## 已完成切片

- **M1 runner 通道 + 首轮基线（2026-10-07）**：`testharness-navigation` 子命令 +
  `navigation_case_skipped` 筛减规则（iframe 依赖 245 / window.open 70 / legacy 多页
  95 记账）+ Makefile `fetch-wpt-navigation` / `testharness-navigation` target；
  fetch 脚本改按 pin 实际布局取语料（pin 下无顶层 history/——会话历史 corpus 落
  `html/browsers/history/**`）并补列嵌套 resources 叶。基线：309 案 387 子测试
  81 Pass = **20.9%**（navigation-api 0/196 全域未实现；history-interface 81.6% /
  location 41.9% / history-traversal 62.2% / navigating-across-documents 0%）。
  证据：[evidence/2026-10-07-m1-baseline.md](evidence/2026-10-07-m1-baseline.md)；
  CSV 已回填 active 行（2026-10-07）。

## 下一步计划

1. **P1 收尾**：网络恢复后 `FORCE=1 bash tests/wpt-runner/scripts/goals/60-navigation-compat.sh`
   续拉三域 + 嵌套 resources → 重跑三域基线刷新 evidence + CSV（the-iframe-element
   属 M3 门控面，基线零执行记账不阻塞）。
2. **M2 切片排序建议**（按基线 gap 聚类，见 evidence §gap 归类）：① Location 接口
   语义（`window.Location` 构造器/原型 + hash 赋值 + 写侧重入 guard）② traverse
   事件序（load/popstate 序 + isTrusted + PopStateEvent 属性面）③ 跨文档导航语义。
   Navigation API（0/196，量级 ~196 子测试）建议作为独立评估切片排在 ①② 之后。
3. **M3**：frame tree 最小面——**启动前须用户点名批准**。

**待用户决策清单**：
- [ ] M3 iframe 深结构切片启动授权（未获批期间 DC-3 保持 pending，不阻塞 M2 收口）
  **2026-10-04 已征询待批复**（goal 待决策巡检 msg `om_x100b63125d66c4a8b2070829b8dd058`；
  建议 = 暂不批准——M1/M2 尚未启动，待 M2 收口时随推进一并裁决）。批复前维持立项态挂起。
  **2026-10-06 48h 跟进提醒已发（一次性，到期）**（msg `om_x100b63647f7910a0b49d70def6b88ff`）：
  仍零回复——不回复即默认维持挂起，此后不再重复催办。
