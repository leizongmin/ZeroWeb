# M3-S1 — 下载管理器真实窗口演示流（触发 / 进度记账 / 在文件夹中显示）记账

**日期**: 2026-09-27
**goal**: [docs/goal/desktop-browser.md](../../desktop-browser.md)（M3 内容工具首切片）
**环境**: 同 M1/M2（Linux x86_64，无人值守 `xvfb-run -a` 真实 X11 窗口；test-guard 包裹）。

## 1. 切片性质：功能接线 + 验收流同切片 land

**基线事实**：browser-shell 的 DownloadManager 此前是**纯状态模型**（start_download 仅
单测调用；net/renderer 无 attachment 分流；无落盘路径；无「打开所在文件夹」动作）。
本切片补齐最小真实链路（全部落在 `apps/browser`，**零 net/engine/renderer crate 改动**
——§9 碰撞边界内自决）：

1. **附件拦截**（`process_backend.rs`）：`drain_pending_fetches` 检查响应头
   `Content-Disposition: attachment`（大小写不敏感），解析 `filename=`（回退 URL
   末段 → `download.bin`；清洗路径分隔符与 `..`——信任边界：响应头来自外部服务器）。
   命中 → 入 `pending_downloads` 队列，renderer 改收良性占位页（多进程边界内
   browser 侧独决）。
2. **TabManager 转交**：`take_pending_downloads()`（沿用 pending_loaded 既有模式）。
3. **应用层落盘**（`app.rs save_attachment_download`）：解析目标目录（settings
   `download_directory` 优先，空值回退 `$HOME/Downloads`，自动创建）→ 文件名去重
   （`name (n).ext`）→ `start_download` → `update_progress(0,N)` → 落盘 →
   `mark_completed`（写失败 `mark_failed`）→ 面板自动弹出（Chrome 下载气泡语义）。
4. **面板扩展**（`app_render.rs`）：无活动下载时展示最近完成项（"Download complete"
   + 文件名 + `Show in folder` 按钮）；按钮命中矩形与渲染同源
   （`download_panel_action_rect_for`，物理像素）。
5. **「在文件夹中显示」**（`show_download_in_folder`）：面板按钮真实点击 →
   `xdg-open`/`open`/`explorer`（随平台）spawn + `SMOKE_EVENT …
   download_show_in_folder dir=…` 日志锚。

## 2. 演示流入口（可脚本重放）

```sh
bash scripts/desktop-browser-m3-smoke.sh        # 无 DISPLAY 自动 xvfb-run；SKIP_BUILD=1 跳构建
```

- 驱动状态机：`apps/browser/src/download_smoke.rs`（`--download-smoke-base/-dir` CLI，
  与既有 smoke 模式互斥）
- 状态只读面新增：`smoke_downloads`（url/filename/state/downloaded/total 快照）、
  `smoke_download_dir`、`smoke_last_opened_download_dir`、`smoke_download_panel_visible`
- fixture：`examples/m3-downloads/{page.html,file.zip}`——page.html 为整页大链接
  （页面区中心点击必中），file.zip 为确定性 3072 字节内容
- **落盘隔离**：流程 start 时把 `download_directory` 设为演示流目录（`.acceptance/…
  /files`），COMPLETE 前恢复空值——不写用户真实 Downloads，用户设置不留痕

## 3. 流程与断言（真实输入路径）

| 步骤 | 输入路径 | 断言 |
|------|----------|------|
| page-load | `navigate_to` page.html | 快照 URL 落地 |
| trigger | 页面区中心**点击链接**（真实点击） | attachment 响应拦截 → 下载入账 |
| download-done | 等待落盘 | 管理器状态 `Completed` + `downloaded==total>0` + **磁盘文件字节 == 账面** + 面板可见；截图 01（面板 "Download complete / file.zip / Show in folder" 真实渲染） |
| downloads-page | `open_downloads_page` | `zero://downloads` 呈现；截图 02 |
| show-in-folder | 面板按钮**真实点击** | 动作执行：`last_opened_download_dir == download_dir`；截图 03 |

脚本级复核：`download_completed` / `download_show_in_folder` 两条 SMOKE_EVENT 日志锚、
零 fallback 红线、**落盘文件与 fixture 源文件 `cmp` 逐字节一致**。

## 4. 结果（2026-09-27 实测）

- **download-flow 腿全绿**，复跑通过（单轮 ~1.5s）
- 截图证据：`.acceptance/desktop-browser-m3/download-flow/01…03.png`
- 进度记账口径（如实记账）：本地代理模型下响应体整包一次到达，状态链
  Pending → Downloading(0/N) → Completed(N/N) 在管理器内完成，**中段传输条的
  持续可视化需分块传输**（renderer 流式路径），本切片为模型级三态 + 终态断言；
  分块进度采样挂账（见 §5）

## 5. 余账

- **分块下载进度采样**：需响应体流式分块到达（renderer 流式 IPC 已有
  `X-Zero-Stream-Chunk` 机制），attachment 分流挂到流式路径后可采样真实进度条中段
  ——M3 余项或 M5 收口时评估
- 下载条目级「打开文件」动作（打开下载的文件本身，非所在目录）——M3 余项候选
- 下载页内条目级操作（暂停/取消/重试）已有模型态，UI 面随 M4 数据面切片评估
