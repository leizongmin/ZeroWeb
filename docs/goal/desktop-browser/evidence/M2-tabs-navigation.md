# M2 — 标签族/导航控制真实窗口演示流（记账）

**日期**: 2026-09-27
**goal**: [docs/goal/desktop-browser.md](../../desktop-browser.md)（M2 导航与标签）
**环境**: 同 M1（Linux x86_64，无人值守 `xvfb-run -a` 真实 X11 窗口，wgpu Vulkan
adapter；每腿 test-guard 包裹）。

## 1. 演示流入口（可脚本重放）

```sh
bash scripts/desktop-browser-m2-smoke.sh        # 无 DISPLAY 自动 xvfb-run；SKIP_BUILD=1 跳构建
```

- 驱动状态机：`apps/browser/src/tab_smoke.rs`（`--tab-smoke-url-one/--tab-smoke-url-two/
  --tab-smoke-dir` CLI，与 gui_smoke/parity_smoke 互斥）
- 状态只读面：`apps/browser/src/app_smoke_state.rs`（smoke_* 访问器，release 可用，
  与 `#[cfg(test)]` 的 *_for_test 族分离）+ `TabManager::smoke_navigation_epoch`
- fixture：`examples/m2-tabs/{one,two}.html`（本地 python3 http.server 随机 loopback 端口）

## 2. 流程与断言（全部经真实输入路径）

| 步骤 | 输入路径 | 断言 |
|------|----------|------|
| first-load | `navigate_to`（与地址栏提交同路径） | 活动标签快照 URL == one.html |
| new-tab | **Ctrl+T** 键盘快捷键 | tab 数 1→2、活动标签切换；截图 01 |
| tab-two | 新标签内 `navigate_to` two.html | 快照 URL + **地址栏文本** == two.html；截图 02 |
| switch-back | 页面点击失焦 + **Ctrl+Tab** | 活动标签回到第一标签 + 地址栏跟随（见 §4 缺陷修复）；截图 03 |
| in-tab-nav | 第一标签内 `navigate_to` two.html | 快照 URL == two.html（构造真实历史条目） |
| go-back | **Alt+Left** | 跨真实历史条目回到 one.html；截图 04 |
| go-forward | **Alt+Right** | 恢复 two.html；截图 05 |
| reload | **F5** | 导航 epoch 前进（实测 5→6）+ URL 保持；截图 06 |
| tab-drag | 标签条 **press + 8 步 move + release**（阈值 4px 激活） | tab 顺序 `[1,2]→[2,1]` + 标签条像素签名可视变化（28/64 采样变化）；截图 07 |
| close-tab | **Ctrl+W** | 剩余 tab 恰为第二标签且活动；截图 08 |

终锚：`TAB_SMOKE_COMPLETE … steps=new-tab,switch,in-tab-nav,back,forward,reload,drag,close`。

## 3. 结果（2026-09-27 实测）

- **全流程绿**，复跑通过（连续两轮 exit 0，单轮约 10s 含启动）
- 截图证据：`.acceptance/desktop-browser-m2/tab-flow/01…08.png`（整窗，标签条 + 地址栏
  + 页面区同框；`07-reordered.png` 实证重排后标签条与活动态）
- 结构性发现记档：启动欢迎页（zero://newtab）**不经 shell.navigate、不产生历史条目**，
  后退语义从首个文档导航起算——演示流据实改为标签内二次导航构造历史（§2 in-tab-nav），
  语义与 Chrome NTP 可回退不同，记为产品差异观察（不阻塞，M4 设置/历史演示流复核）

## 4. 本切片修复的产品缺陷

**Ctrl+Tab 后地址栏不跟随活动标签**：`cycle_active_tab`（Ctrl+Tab / Ctrl+PgUp/PgDn）
缺 `update_address_bar_from_active_tab()` 与 `on_active_tab_changed()` 回调——切换后
地址栏仍显示前一标签 URL；而鼠标点标签、Ctrl+1~9、Ctrl+9 路径均有同步。属 browser
UI 接线层（本流 Support Envelope），共享路径单点修复：

- `apps/browser/src/app_input.rs` `cycle_active_tab`：补两行同步（与 Ctrl+1 路径同款）
- 回归锚：`apps/browser/src/tests.rs` `ctrl_tab_updates_address_bar_to_active_tab`
  （单测红→绿验证）+ M2 演示流 switch-back 步骤断言（真实窗口级）

**演示流方法论注记**：该缺陷由「状态断言 + 真实输入路径」的演示流在第一次全流程
运行中即捕获——纯像素断言（M1 形态）或纯单测都不会将其暴露为用户可见回归
（地址栏渲染本身正常，错的是状态源），印证 DC-2 采用状态断言演示流的必要性。

## 5. DC-2 判定映射

| DC-2 条目 | 状态 | 证据 |
|-----------|------|------|
| 多标签演示流（创建/关闭/切换/拖拽排序）全绿 | ✅ | 本文 §2/§3（创建 Ctrl+T / 切换 Ctrl+Tab / 拖拽排序 / 关闭 Ctrl+W） |
| 地址栏演示流（URL 输入导航/自动补全/加载进度）+ 导航控制演示流全绿 | 🔄 部分 | 导航控制（后退/前进/刷新）✅ 本文 §2；主页按钮依赖 home_url 配置（默认外网 example.com，离线演示流不覆盖）→ 并入 M4 设置演示流（先设 home_url 为本地 fixture 再按 Alt+Home）；**M2-S2 余项**：地址栏键入导航 + 自动补全弹出 + 加载进度指示演示流 |
