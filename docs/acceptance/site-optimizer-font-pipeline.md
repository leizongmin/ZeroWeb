# 网站优化首例：morning.work 下载字体生产链路

关联：[网站驱动优化 skill PR #26](https://github.com/leizongmin/ZeroWeb/pull/26)。
本例只修复此次网站暴露的字体资源、度量传递和采集时机，不宣称全站 Chrome 一致。
工作分支为 `fix/downloaded-font-production-pipeline`，基线为 `9dbc84db3bc8779e0746923d2dbfb0e189a50a86`。

## 用户任务与最初失败

通过 ZeroBrowser 阅读 morning.work 的 Fedora 三指拖拽文章和 Go 数组/切片文章，
检查代码能否正常显示。公开只读访问，不登录、不提交表单、不修改网站。

原版把 `/JetBrainsMono/JetBrainsMono.css` 中的相对字体地址按文章 URL 解析，
产生八个字体 404。最初的 URL-only 候选通过 WebView 单测，却让生产合成器丢字，
因此被拒绝。这是本例的失败实验，不从最终记录中删除。

## 根因与修复边界

| 边界 | 问题 | 本次修复 |
|---|---|---|
| 外部 CSS → 资源请求 | 合并 CSS 后丢失声明所在 URL | 复用 CSS tokenizer，在同步/异步合并前统一解析相对 URL；保留字符串、注释、BOM、局部 fragment |
| renderer → compositor | 只有数值 font ID，没有字体资源 | 绘制快照携带本帧引用的下载字体字节及 face index；每个 surface/document 独立导入、重映射 |
| 字体生命周期 → 栅格缓存 | 不同页面的相同数值 ID 可指向不同字体 | 导航释放下载字体；切换字体命名空间时清 CPU glyph cache、GPU atlas，并刷新光栅线程 |
| layout → intrinsic/paint | 收缩仍使用估算宽度；一次重测漏传字体上下文；paint 空 styles 时丢失基线 | 下载字体度量刷新，纯文本叶盒的 intrinsic 共用测量路径，保存并恢复文档字体 ascent |
| 页面进度 → 截图 | 首帧早于 webfont 加载 | 采集器等待同一导航的有效完成通知，再走原有生产帧确认链 |

字体资源采用完整快照而非“注册一次后只发送 ID”：renderer publisher、browser 暂存、
compositor 队列都可能丢弃中间帧，未覆盖所有确认边界的增量资源协议会再次漏字。
接收端对未变化资源不重复解析。每帧最多 128 个字体，单字体与字体总字节预算均为
32 MiB；拒绝重复 ID、平台 ID 冲突、空数据和无效 face。资源变化原子提交，失败不
替换可用 registry。代价是活跃下载字体字节随帧重复传输，尚未实现 ACK 型资源缓存。

IPC 的 bincode 结构发生变化，部署时必须一起更新 browser、renderer 和 compositor，
不能混用旧辅助进程；`serde(default)` 不代表旧 bincode 消息兼容。公开 Rust 结构
`PaintSnapshotParams`、`FontFamilyMetrics`、`LayoutBox` 增加字段，直接构造它们的
调用方须同步更新（适用时采用 `Default`）。

普通系统字体保留既有基线策略；此次 intrinsic 补齐仅覆盖下载字体的纯文本叶盒，
不把复杂嵌套的整个固有尺寸算法重写纳入本例。规范依据见代码中的 CSS Values、
CSS Fonts、CSS Sizing 与 CSS2 行布局链接。没有域名、selector 或固定文案特判。

## 确定性回归与生产验收

常驻资产位于 [downloaded-fonts fixture](../../tests/wpt-runner/fixtures/downloaded-fonts/README.md)。
字体仍使用仓库既有 Ahem/Lato 资产，不复制网站或另带第三方字体。

- URL 单测：外部、嵌套 import、页面内联样式分别保留正确基址；覆盖引号、转义、
  注释、字符串、空 URL、绝对 URL 和局部 fragment。
- renderer/protocol：只发送引用到的下载字体；重复 glyph 去重；任一最新快照独立
  携带资源；序列化保留字节、face index 和 ID。
- 消费方：两个 surface 可使用相同源 ID 的不同字体；无效更新不覆盖有效资源；
  空快照/导航释放字体；browser 下载字体不能替换 chrome UI 字体。
- 光栅化：下载字体经 IPC 转换后的同步/线程化 compositor 输出，与本地加载同字体
  的像素一致，且明确断言非空，避免两张空白图假通过。
- 度量：用非 `Ahem` 名称的 CSS alias，检查 40px × 4 字、20px × 3 字的宽高与
  基线，防止字体名捷径掩盖真实加载链路。

生产实验使用 GUI Chrome 152 与 ZeroBrowser 多进程 GPU 窗口，1022×676、DPR 1、
en-US、light。同一最小输入没有固定宽高，字体响应故意延迟 500ms。

| 检查点 | 结果 |
|---|---|
| 文本状态 | 两端均为 `XXXX` |
| 元素几何 | 两端均为 160×40，最大误差 0px |
| 字体目标区域 | 0/6400 差异像素 |
| 未遮罩全帧 | 4/690872 差异像素；原阈值未修改 |
| 截图来源 | Chrome GUI CDP；ZeroBrowser `production-window-gpu` / `compositor_bitmap` |

仅资源传递修复的中间版本曾出现 5px 基线偏移，目标区域差异 12.5%，虽然全图仅
0.18% 也被判失败。最终通过的是无固定宽高的 intrinsic 用例，不以修改过的页面
替代原失败输入。固定尺寸 transport 用例作为相邻变体保留。

证据保存在本地忽略目录 `.acceptance/font-pipeline/`：`intrinsic/comparison-final.json`、
Chrome/ZeroWeb manifest、原始全帧及区域图、字体请求与延迟日志。中间失败也保留。
截图不纳入源码仓库，亦不包含浏览器 profile、cookie 或认证头。

最终生产采集二进制 SHA-256（Linux release，多进程配套构建）：

| 二进制 | SHA-256 |
|---|---|
| zero-browser | `b00aa0d545d0108dba4c9fd899138bfb0419fc2ab76a9ebf27824ab0d17ca3da` |
| zero-renderer | `f5e7b6d78c0afc02f738fd56a716657d36afc363f265e6759e1c9e1268c58f02` |
| zero-compositor | `6534bfcc5d6fe36ed31dccdb65636054133bde2c7c3bc000eba456114e6e315a` |

## 真实网站与工程门禁

两篇文章的八个 JetBrainsMono 字体请求均返回 200，原有错误目录的字体 404 消失。
标题、正文选定代码的状态断言相同；严格整页对照仍 **FAIL**：

| 页面 | 全帧差异 | 代码区域差异 | 几何最大差异 |
|---|---:|---:|---:|
| Fedora 三指拖拽文章 | 6.92% | 5.34%（未过 5% 门槛） | 355.25px（行内代码换行位置） |
| Go 数组/切片文章 | 9.57% | 2.25%（通过区域门槛） | 7.5px |

全帧门槛为 3%，几何门槛为 2px，均未修改。剩余问题包括复杂 inline 的字距/换行、
标题与内容横向位置、徽章样式；不能把字体请求成功或代码区域通过等同于整页兼容。
对应记录为 `morning/comparison-final.json` 和 `go/comparison.json`。

原生 GUI 回放通过地址栏打开 Go 文章并滚动，再打开独立字体测试标签；鼠标切换
两个标签、恢复页面、关闭字体标签后，原文章及浏览器 UI 文字仍可见。证据为
`gui-go-scroll.png`、`gui-go-restored-final.png`、`gui-ahem-restored-final.png` 和
`gui-after-close-final.png`。一次连续 Ctrl+T/Ctrl+L 自动输入发生焦点时序错误，
错误导航截图保留；随后通过地址栏点击、全选、输入完成，不把失败尝试计为通过。

工程检查：完整 `make test`（含默认 V8 工作区及 Makefile 的 QuickJS/应用追加检查）、
`cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all -- --check`
通过；reftest 为 687/687，通过表单输入重排/帧数硬预算。测试、构建与基准均经
项目内存/墙钟包裹器执行。

完整 `make bench-gate` 已跑完 16 个 crate、3 个页面及 retained form input，但最终
**FAIL：113 个指标中 15 个超预算**（7 个 DOM、3 个 engine、1 个 layout、3 个
WebView，以及 medium 首帧墙钟）。页面总耗时与 RSS 硬预算通过，不抵消相对指标
失败。报告为 `tests/benchmarks/results/benchmark_20260913_174216.json`，完整原始
微基准样本另存 `bench-full-samples/`。平台型号/核数与基线一致；测量期间同机存在
其他工作区基准负载，不能据此宣称提速，也不能自动豁免超预算。

当前候选的总体判定为 **inconclusive**，尚未升级为验收通过的 best。定向原版/候选
对照仅用于归因，不替代完整门禁，也不修改共享性能基线。

后续诊断固定 CPU 2，按每个 crate 的 Cargo artifact 绑定对应 feature 的二进制，
预编译后再做三轮交替原版/候选测量；每个微基准 warm-up 1s、measurement 3s、
sample-size 20，与原配置一致。14 个微基准的“三轮 p50 的中位数”候选/原版比值
为 0.949–1.089：简单 paint 1.012、复杂 paint 1.008、block layout 1.034，最大值
为 inject_css 的 1.089。没有复现完整报告中的约 5 倍 paint 差距，但这不是尾延迟
无回归证明，也不是完整门禁通过。样本见 `paired-perf-matched.jsonl`。
第一次诊断因发现同名基准的多个 feature 二进制而中止，其不完整数据单独保留，
不并入上述三轮统计。

medium 页面另作三轮同核交替测量，沿用 `--iterations 15`（工具丢弃预热后每轮
14 个有效样本）：原版 p95 为 208.19 / 218.46 / 226.04ms，候选为
411.19 / 423.96 / 228.99ms。候选尾延迟仍不稳定，不能将性能失败整体判作既有
基线问题或排除新增回归。完整样本见 `paired-page.jsonl`。下一步需在可控负载下
重跑完整门禁并定位 medium 各阶段尾延迟；本 PR 保持 Draft，不申请自动改基线。

欢迎页失败提前截断的其余产品 smoke 已逐项补跑：morning 桌面、375px、320px，以及
welcome 375px、320px，五项结构门均通过；没有删减断言。

已确认的基线门槛问题：`make product-smoke` 欢迎页在原版和候选均为 20.30%，
超过 20% 阈值；两张 PNG 逐字节相同，SHA-256 为
`6c2ad685aeb0e5cefabf876188041d104be817781e09a3e62cd285f1b1456156`。
结构检查通过。这不是本次引入的回归，但不应写成门禁通过。用户已单独批准仅将
此项作为已知基线失败注明后创建普通 PR，前提为其他检查通过；不覆盖性能、字体
回归、完整测试或 clippy。两张 PNG 已另存为 `welcome-original.png` 和
`welcome-candidate.png`，不会被后续 smoke 的默认输出覆盖。

## 未覆盖与续作

采集等待初始导航资源完成，不等同于所有延迟 JS、动态 FontFace、动画及站外脚本
都已稳定；本例不声称脚本零错误、Web Vitals 达标或全站字体完全一致。
Firefox、macOS、Windows、Android 未进行真实窗口验收。
同一 Agent 回放不是独立人工体验，也不是无人值守可靠性的统计认证。

后续关注复杂 inline intrinsic、站点徽章/行内背景、Ctrl+F 生产匹配，以及带资源确认
与回收协议的字体传输成本优化；不通过扩大截图遮罩、改 oracle 或绕过安全策略解决。
