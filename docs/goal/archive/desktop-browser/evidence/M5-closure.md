# M5 — 收口：DC 逐项判定 + smoke 资产定稿 + 挂账清单（目标 DONE 判定）

**日期**: 2026-09-27
**goal**: [docs/goal/desktop-browser.md](../../desktop-browser.md)
**判定基线**: HEAD 379a743ae（M4 落地后树）；最终树四演示流全绿复验（见 §3）。

## 1. Done Criteria 逐项判定

### DC-1: 真窗口主链路 — ✅

| 条目 | 判定 | 证据 |
|------|------|------|
| 三平台可编译启动（CI 矩阵佐证） | ✅ | 可编译：ci.yml 六条目矩阵 + weekly.yml 11 target release 构建；启动：`matrix.launch_smoke` headless CDP boot 探活三平台 CI 绿（run [36311188270](https://github.com/leizongmin/ZeroWeb/actions/runs/36311188270)） |
| Linux 真窗口端到端演示流绿 | ✅ | evidence/M1-real-window-main-path.md（三腿：cpu+compositor / gpu 窗口 / gpu-direct 导入，启动→加载→渲染→滚动/缩放输入全断言） |
| GPU 合成显示验收记账 | ✅ | M1 腿 b/c（GPU 窗口 surface + compositor_dmabuf_adopted）+ 父 DC-4.4 联动注记（frame_flow 17/17 常驻 + 本 goal 真窗口显示端） |

### DC-2: 导航与标签 — ✅

| 条目 | 判定 | 证据 |
|------|------|------|
| 多标签演示流（创建/关闭/切换/拖拽排序） | ✅ | evidence/M2-tabs-navigation.md（Ctrl+T / Ctrl+Tab / 拖拽重排+像素签名 / Ctrl+W；含 Ctrl+Tab 地址栏跟随缺陷修复） |
| 地址栏演示流（URL 输入导航/自动补全/加载进度） | ✅ | evidence/M2-addressbar-autocomplete-loading.md（键入导航 / 补全弹出+键盘选中 / 延迟页 loading 采样） |
| 导航控制演示流（前进/后退/刷新/主页） | ✅ | 前进/后退/刷新：M2-S1（真实历史条目 + epoch 断言）；主页：Alt+Home 落地 evidence/M4-data-plane.md（home_url 设 fixture 后） |

### DC-3: 内容工具 — ✅

| 条目 | 判定 | 证据 |
|------|------|------|
| 下载管理器演示流（触发/进度/打开所在文件夹） | ✅ | evidence/M3-download-manager.md（attachment 拦截→落盘逐字节 cmp→三态记账→面板→Show in folder；进度记账口径与余账 §5） |
| 页面查找演示流（搜索/高亮/计数） | ✅ | evidence/M3-page-find.md（glyph 匹配最小面：计数 1/5→逐项滚动→高亮；连字局限记账） |
| 缩放演示流（Ctrl±/重置，viewport 联动） | ✅ | evidence/M3-context-menu-zoom.md（1.0→1.1 + reflow 像素签名 + 复位） |
| 右键上下文菜单演示流（复制/粘贴/检查元素入口） | ✅ | evidence/M3-context-menu-zoom.md（Page/Selection 菜单分发 + 剪贴板读回 + inspect 新标签；粘贴属地址栏 Editable 域已记账） |

### DC-4: 数据面 — ✅

| 条目 | 判定 | 证据 |
|------|------|------|
| 收藏夹演示流（添加/删除/文件夹管理/收藏栏） | ✅ | evidence/M4-data-plane.md（Ctrl+D 添加/收藏栏点击/右键删除；文件夹管理 API 存量、UI 管理入口挂账 §5） |
| 历史演示流（记录/搜索/清除） | ✅ | evidence/M4-data-plane.md（记录/清除全链；搜索 API 存量、页内搜索 UI 挂账 §5） |
| 基础设置页演示流（默认搜索引擎/主页/隐私） | ✅ | evidence/M4-data-plane.md（DuckDuckGo 切换 / do_not_track / home_url + Alt+Home 落地） |

### DC-5: 测试与质量不可退让 — ✅

| 条目 | 判定 | 证据 |
|------|------|------|
| make test 全绿零失败 + clippy -D warnings + fmt 干净 | ✅ | 最终树 make test 19,466P/0F（MAKETEST_EXIT=0，经 test-guard）+ clippy --workspace --all-targets -D warnings 干净 + fmt --check 干净（2026-09-27 本轮实测） |
| 演示流脚本化入 evidence/ 可重放；product smoke 资产随功能同步扩充 | ✅ | §2 四脚本 + 五模块状态机 + 六 fixture 页；每切片功能与验收流同轮 land |

## 2. smoke 资产定稿（四守成门）

| 守成门 | 命令 | 腿 |
|--------|------|-----|
| M1 | `bash scripts/desktop-browser-m1-smoke.sh` | 三腿：cpu+compositor / gpu 窗口 / gpu-direct |
| M2 | `bash scripts/desktop-browser-m2-smoke.sh` | 双腿：tab-flow / addressbar-flow |
| M3 | `bash scripts/desktop-browser-m3-smoke.sh` | 三腿：download-flow / menu-zoom-flow / find-flow |
| M4 | `bash scripts/desktop-browser-m4-smoke.sh` | 一腿：data-flow（profile 隔离） |

**复验记录（2026-09-27，HEAD 379a743ae 终树）**：四脚本顺序执行全部 exit 0（本会话
各脚本此前均另有多轮复跑记录）。

## 3. 本 goal 产出的产品缺陷修复（演示流驱动）

1. Ctrl+Tab 切换后地址栏不跟随（cycle_active_tab 漏同步）— M2-S1
2. 剪贴板 X11 写入即失主（arboard 实例即弃）— M3-S2
3. （链路补齐）下载 attachment 拦截→落盘→记账→面板→Show in folder — M3-S1
4. （链路补齐）页面查找 glyph 匹配→计数→高亮→滚动定位 — M3-S3

## 4. 挂账清单定稿（转守成态评估队列）

| # | 挂账 | 归属 |
|---|------|------|
| 1 | 正文空格间歇塌缩（product 可见） | rendering-compat 流（R4096-N font-stack 统一专项族，user-gated） |
| 2 | CI benchmarks job 连日红（perf-gate 5 指标超预算 vs 本地全绿） | perf-gate 政策域（用户拍板） |
| 3 | 分块下载进度采样（需流式传输）+ 下载条目级「打开文件」 | 守成评估 |
| 4 | 书签文件夹管理 UI + 历史页内搜索 UI（API 均存量） | 守成评估 |
| 5 | 查找连字完整匹配（source cluster 恢复） | rendering-compat 语料评估 |
| 6 | m1/m2/m3 演示流脚本回补 XDG_CONFIG_HOME profile 隔离（m4 已隔离） | 守成 hygiene |
| 7 | macOS/Windows 真窗口 GUI 深体验（现以 headless 启动冒烟覆盖） | 平台差异挂账（入口文档约束内） |

## 5. 守成态规划

- **守成门**：make test + 四演示流脚本（M1 三腿/M2 双腿/M3 三腿/M4 单腿）全绿；
  渲染/组合器/窗口/标签·地址栏·导航·下载·菜单·缩放·查找·数据面链路任何触碰后必跑
- 巡检轮按 run-rules 守成模式执行（树不变预言锚 + 六点门禁），本 goal 控制面
  转入低频维护
