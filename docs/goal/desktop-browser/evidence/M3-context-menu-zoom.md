# M3-S2 — 右键上下文菜单 + 缩放联动真实窗口演示流记账

**日期**: 2026-09-27
**goal**: [docs/goal/desktop-browser.md](../../desktop-browser.md)（M3 内容工具第二切片）
**环境**: 同 M1/M2/M3-S1（Linux x86_64，无人值守 `xvfb-run -a` 真实 X11 窗口；test-guard 包裹）。

## 1. 演示流入口（可脚本重放）

```sh
bash scripts/desktop-browser-m3-smoke.sh        # 腿 1 download-flow（M3-S1）+ 腿 2 menu-zoom-flow
```

- 驱动状态机：`apps/browser/src/menu_zoom_smoke.rs`（`--menu-zoom-smoke-base/-dir` CLI）
- 状态只读面新增：`smoke_context_menu_visible/origin/item_ids`（菜单几何与条目源）、
  `smoke_clipboard_text`（arboard 读回）、`smoke_page_zoom`（shell 缩放状态）
- fixture：`examples/m3-downloads/menu.html`（多行文本页，供拖拽选择与 reflow 观察）

## 2. 流程与断言（全部真实输入路径）

| 步骤 | 输入路径 | 断言 |
|------|----------|------|
| context-menu | 页面区**右键** | 菜单可见 + Page 菜单条目断言（back/forward/reload/-/save_as/print/-/view_source/inspect）；截图 01（菜单真实渲染） |
| menu-reload | **菜单 reload 行真实点击**（行几何与渲染/命中测试同源：普通行 32s + 分隔行 10s 累积） | 菜单关闭 + 导航 epoch 前进 + URL 不变 |
| inspect-tab | 再次右键 → **inspect 行点击** | 检查元素新标签打开（`zero://inspect?url=…`）；截图 02；Ctrl+W 关闭回文本页 |
| selection-copy | **拖拽选中**文本（press→8 步 move→release）→ 右键 → Selection 菜单 → **copy 行点击** | Selection 菜单含 copy；剪贴板读回非空（实测 25 字节） |
| zoom-in | **Ctrl+＋**（真实快捷键） | shell 缩放 1.0→1.1 + 页面区 reflow 像素签名变化（gui_smoke 同款 visual_change 契约）；截图 03 |
| zoom-reset | **Ctrl+0** | 缩放回 1.0 |

## 3. 结果（2026-09-27 实测）

- **menu-zoom-flow 腿全绿**（与 download-flow 同轮双 PASS），复跑通过
- 截图证据：`.acceptance/desktop-browser-m3/menu-zoom-flow/01…03.png`

## 4. 本切片修复的产品缺陷（演示流迭代捕获）

**剪贴板写入即失主（X11）**：`clipboard.rs` 旧实现每次读写新建 `arboard::Clipboard`
并即弃——X11 selection 所有权随实例存活，写完即丢，复制的内容其他应用与本进程
**都读不回**。演示流首次全流程跑通时被「剪贴板读回」断言捕获。修复：进程内
OnceLock 保活单实例（读写共用；初始化失败缓存为不可用）。`copy_page_selection`
无选中文本时补 warn 日志（诊断可见性）。

**方法论注记（菜单行点击几何）**：菜单行点击坐标必须按「普通行 32s + 分隔行 10s
累积」计算（与渲染/命中测试同源），均匀行高公式在含分隔符的菜单上会点到菜单外
（inspect 行实测偏出 28s）——点击落空不会报错只会关菜单，断言必须锚定动作副作用
（epoch/新标签/剪贴板）而非「无异常」。

## 5. DC-3 判定映射

| DC-3 条目 | 状态 | 证据 |
|-----------|------|------|
| 下载管理器演示流全绿 | ✅ | evidence/M3-download-manager.md（M3-S1） |
| 页面查找演示流全绿（engine 文本搜索 API 就位） | ⏳ M3 余项 | Ctrl+F 查找栏 UI 已存在（find_state/found 计数），engine 文本搜索 API 最小面评估待独立切片（P4） |
| 缩放演示流全绿（页面 viewport 联动） | ✅ | 本文 §2（Ctrl+± 状态 + reflow 像素断言 + 复位；④ 桥 viewport 消费的页面侧信号 = reflow 实测） |
| 右键上下文菜单演示流全绿（复制/粘贴/检查元素入口） | ✅ | 本文 §2（复制 ✅ / 检查元素入口 ✅；粘贴依赖 Editable 场景，地址栏粘贴属地址栏演示流域，记账不重复） |
