# M2-S2 — 地址栏真实窗口演示流（键入导航 / 自动补全 / 加载指示）记账

**日期**: 2026-09-27
**goal**: [docs/goal/desktop-browser.md](../../desktop-browser.md)（M2 导航与标签，DC-2 收口切片）
**环境**: 同 M1/M2-S1（Linux x86_64，无人值守 `xvfb-run -a` 真实 X11 窗口；test-guard 包裹）。

## 1. 演示流入口（可脚本重放）

```sh
bash scripts/desktop-browser-m2-smoke.sh        # 腿 1 tab-flow（M2-S1）+ 腿 2 addressbar-flow
SKIP_BUILD=1 bash scripts/desktop-browser-m2-smoke.sh
```

- 驱动状态机：`apps/browser/src/addressbar_smoke.rs`（`--addressbar-smoke-base=
  <origin> --addressbar-smoke-dir=<dir>` CLI，与既有 smoke 模式互斥）
- 状态只读面新增：`smoke_autocomplete_urls` / `smoke_autocomplete_highlight`
  （app_smoke_state.rs；建议列表与高亮位是补全弹出的状态源）
- fixture：`examples/m2-tabs/{one,two,slow}.html`——`slow.html` 由本地服务延迟
  1.5s 响应，使加载窗口可被采样

## 2. 流程与断言（全部真实输入路径）

| 步骤 | 输入路径 | 断言 |
|------|----------|------|
| seed-load | `navigate_to` one.html | 加载完成（历史记录入账，供补全匹配） |
| typed-url | **Ctrl+L** 聚焦全选 + **逐字符键入**完整 two.html URL | 地址栏文本 == 键入串；截图 01 |
| typed-nav | **Enter**（无选中建议 → 导航键入文本） | 活动标签快照 URL == two.html |
| suggest-popup | Ctrl+L + 键入 `two` | 建议列表含搜索建议（"two — Bing 搜索"）+ 历史项 two.html；**ArrowDown ×2** 高亮位 == 历史项索引；截图 02（下拉真实渲染） |
| suggest-nav | **Enter**（按高亮建议导航） | 快照 URL == two.html（建议导航落地）；截图 03 |
| loading-indicator | `navigate_to` slow.html；**Poll tick 主动渲染采样**（见 §4） | `any_tab_loading()==true` 窗口内抓帧：标签条 spinner + 工具栏停止钮 + 地址栏 spinner（截图 04，人工核验真实 loading 态） |
| slow-loaded | 等待延迟响应 | 快照 URL == slow.html；截图 05 |

终锚：`ADDR_SMOKE_COMPLETE … steps=typed-nav,suggest-nav,loading-indicator`；
脚本另断言 `Compositor disconnected` / `SMOKE_FAILURE` / `panicked` 红线零出现。

## 3. 结果（2026-09-27 实测）

- **addressbar-flow 腿全绿**，复跑通过（与 tab-flow 腿同轮两 PASS）
- 截图证据：`.acceptance/desktop-browser-m2/addressbar-flow/01…05.png`
  ——`02-suggest-popup.png` 实证补全下拉在真实窗口渲染（搜索建议 + 历史项两行、
  键盘高亮第二行）；`04-loading.png` 实证加载指示全链（tab spinner / 停止钮 /
  地址栏 spinner / 过渡 Loading… 页）

## 4. 方法论注记：Poll tick 采样（帧回调链断流绕行）

首跑发现：`navigate_to(slow)` 后事件循环进入整段空闲——无 OS 事件时 16ms
`AppEvent::Poll` tick 是唯一保证的推进点（`about_to_wait` 每 tick 派发），但
RedrawRequested 链不保证在 loading 窗口内存活，帧回调 gate 分支采不到样。修法：
`AddressbarSmoke::sample_mid_load` 挂接主循环 Poll tick——`WaitingSlowLoad` 且
`any_tab_loading()` 时主动 `render_full_scene_gpu_capture` 抓帧（GPU 路径同线程
安全，与主循环既有捕获调用同款）。此前提（load 期空闲断流）同为 M2-S1 帧自驱动
(`needs_redraw` 保活) 的根因补注。

## 5. DC-2 判定映射（收口）

| DC-2 条目 | 状态 | 证据 |
|-----------|------|------|
| 多标签演示流（创建/关闭/切换/拖拽排序）全绿 | ✅ | evidence/M2-tabs-navigation.md（M2-S1） |
| 地址栏演示流（URL 输入导航/自动补全/加载进度）全绿 | ✅ | 本文 §2/§3（键入导航 + 补全弹出选中导航 + 加载指示） |
| 导航控制演示流（前进/后退/刷新/主页）全绿 | ✅（主页项并入 M4） | 后退/前进/刷新：M2-S1；主页按钮依赖 home_url 配置（默认外网 example.com）→ M4 设置演示流先配本地 fixture 再按 Alt+Home，随设置流一并记账 |
