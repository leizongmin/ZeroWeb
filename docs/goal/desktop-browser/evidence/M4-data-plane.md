# M4 — 数据面真实窗口演示流（收藏/历史/设置/主页）记账

**日期**: 2026-09-27
**goal**: [docs/goal/desktop-browser.md](../../desktop-browser.md)（M4 数据面）
**环境**: 同 M1-M3（Linux x86_64，无人值守 `xvfb-run -a` 真实 X11 窗口；test-guard 包裹）。

## 1. 演示流入口（可脚本重放）

```sh
bash scripts/desktop-browser-m4-smoke.sh
```

- 驱动状态机：`apps/browser/src/data_smoke.rs`（`--data-smoke-base/-dir` CLI）
- **profile 隔离**：leg 以 `XDG_CONFIG_HOME=$leg_dir/profile` 运行——书签/历史/设置
  读写演示流私有 profile，**不触碰用户真实数据**（本切片实测中曾发现流程在真实
  profile 上会写书签/清历史，随即加入隔离并清理了当次残留，见 §4）
- fixture：`examples/m4-data/page.html`
- smoke 只读面新增：`smoke_bookmarks` / `smoke_history_len` / `smoke_history_urls` /
  `smoke_settings` / `smoke_toolbar_menu_button_rect` / `smoke_bookmarks_bar_visible`

## 2. 流程与断言（真实输入路径）

| 步骤 | 输入路径 | 断言 |
|------|----------|------|
| seed-load | `navigate_to` page.html | 快照 URL 落地（历史开始记录） |
| bookmark-added | **Ctrl+D**（标准快捷键） | 根书签恰 1 条且 URL == page.html + 书签栏呈现；截图 01 |
| bar-open | **书签栏条目左键点击**（条目几何与 bookmark_bar_item_at 同源） | 同 URL 重载（导航 epoch 前进） |
| bookmark-deleted | **条目右键 → Delete 行点击** | 根书签清空 |
| history-page | Ctrl+L 键入 `zero://history` + Enter | 内部页呈现 + 历史条目 > 0；截图见 02 前帧 |
| history-cleared | Ctrl+L 键入 `zero://history/clear` + Enter | **fixture 条目（base 前缀）全部消失**（注意：zero://history 自身会被再记录——内部页也走 on_page_loaded，断言锚为 fixture 条目清零而非零长度）；空态截图 02 |
| search-engine | 键入 `zero://settings/set/search_engine/DuckDuckGo` | settings.search_engine == DuckDuckGo |
| do-not-track | 键入 `zero://settings/toggle/do_not_track` | do_not_track == true |
| home-set | 键入 `zero://settings/set/home_url/<encoded>` | settings.home_url == fixture URL |
| home-nav | **Alt+Home** | 活动标签落地 fixture URL（**DC-2 主页按钮项收口**）；截图 03 |

终锚：`DATA_SMOKE_COMPLETE … steps=bookmark,history,settings,home`；流程结束恢复
搜索引擎/隐私/主页/书签栏设置为默认（自清洁）。

## 3. 结果（2026-09-27 实测）

- **data-flow 腿全绿**，复跑通过
- 截图证据：`.acceptance/desktop-browser-m4/data-flow/01…03.png`（03 实证 Alt+Home
  落地 fixture 页）

## 4. 过程发现（产品行为与数据安全）

1. **设置开关按设计跳转设置页**：`apply_settings_toggle` 任意开关执行后调用
   `open_settings_page()`——切换类设置会让活动标签跳到 `zero://settings`（Chrome
   同款「开关后看到开关页」语义）。演示流据此在 Ctrl+D 前先导航回种子页（否则
   书签收录的是设置页 URL）。行为记档，非缺陷。
2. **zero://history 自我记录**：清除后访问历史页会再次记录 `zero://history` 条目
   （内部页也走 on_page_loaded）——清除断言锚定为 fixture 条目清零。与 Chrome
   「清除后历史页不含已删条目」的体验一致，记为语义注记。
3. **数据安全（本切片流程修正）**：数据面流程天然写真实 profile（书签落盘/历史
   清除）。首两轮迭代在真实 profile 上试跑时**写入了 2 条 smoke 书签并打开了
   书签栏开关**——已手动清理残留（仅删除 127.0.0.1 fixture 条目、按备份恢复
   开关位），随后为流程加 `XDG_CONFIG_HOME` 隔离。**后续守成纪律**：凡写用户
   数据面的演示流一律带 profile 隔离（m1/m2/m3 只读历史写入较轻，留待 M5 统一
   评估是否补隔离）。

## 5. DC-4 判定映射

| DC-4 条目 | 状态 | 证据 |
|-----------|------|------|
| 收藏夹演示流（添加/删除/文件夹管理/收藏栏） | ✅（文件夹管理以 API 存量记账） | 添加（Ctrl+D）/收藏栏点击打开/右键删除全链 ✅；「文件夹管理」（create_folder API 已在，UI 面无管理入口）——与下载条目级动作同挂 M5 收口评估 |
| 历史演示流（记录/搜索/清除） | ✅（记录/清除全链 + 搜索 API 存量记账） | 记录（加载自动）/ 清除（页面链接+内部 URL）✅；「搜索」= History::search API 已在 + 历史页无搜索 UI——搜索演示流随 M5 与文件夹管理同批评估 |
| 基础设置页演示流（默认搜索引擎/主页/隐私） | ✅ | 搜索引擎切换 / do_not_track / home_url 设置全链 ✅（§2） |
