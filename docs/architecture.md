# ZeroBrowser Architecture Overview

本文面向新贡献者，回答三个问题：

1. 这个仓库现在到底实现到了哪里
2. 各个 crate 分别负责什么
3. 应该从哪里开始阅读和修改

更正式的需求与约束见 [docs/specs/zero-browser-spec-rfc.md](/home/lei/work/ZeroBrowser/docs/specs/zero-browser-spec-rfc.md)，当前执行状态见 [docs/goal/zero-browser/master.md](/home/lei/work/ZeroBrowser/docs/goal/zero-browser/master.md)。

> **注意**
> 本文描述的是实验性代码库的当前结构和目标方向，不代表项目已经达到可日常使用、可商用或其他生产可用状态。任何这类用途都需要自行评估风险。

## Design Goals

- 构建一个可嵌入的 Rust `webview` 库
- 构建一个完整浏览器应用
- 在开源协作下保持核心代码与演进节奏可控
- 让主线依赖尽量保持宽松许可证边界
- 用自动化测试、基准和文档约束 AI-first 开发流程

## Layered Workspace

### Applications

| Path | Responsibility |
|------|----------------|
| `apps/browser` | 浏览器应用入口；当前仍是占位程序 |
| `apps/webview-demo` | 最小演示程序，用于串起宿主窗口和渲染基础设施 |

### Product And API Layer

| Crate | Responsibility |
|-------|----------------|
| `crates/webview-api` | 对外暴露稳定嵌入 API，屏蔽底层渲染细节 |
| `crates/browser-shell` | 浏览器产品层 UI，包括标签页、地址栏、历史等 |

### Engine Layer

| Crate | Responsibility |
|-------|----------------|
| `crates/dom` | DOM 树、HTML 集成、节点和文档模型 |
| `crates/css-parser` | CSS tokenizer、parser、选择器和值解析 |
| `crates/style-system` | 级联、继承、计算值、DOM 样式匹配 |
| `crates/layout-engine` | 布局整合层，把样式转换成布局树和几何输出 |
| `crates/engine-core` | 渲染主管线，负责串起解析、样式、布局、paint 和 composite |
| `crates/canvas` | Canvas 2D 绘制能力 |

### Infrastructure Layer

| Crate | Responsibility |
|-------|----------------|
| `crates/render-foundation` | GPU/CPU 渲染、字体栈、图片缓存、图元基础设施 |
| `crates/host-runtime` | 平台窗口、事件循环、surface 生命周期、输入事件 |
| `crates/net` | HTTP/HTTPS、URL、导航历史、Cookie |
| `crates/security` | 同源策略、CORS、CSP 和相关安全规则 |
| `crates/storage` | localStorage、sessionStorage、IndexedDB |
| `crates/protocol` | IPC 消息、序列化、进程间协议边界 |
| `crates/wasm-sandbox` | 受控 WASM 执行环境 |
| `crates/script-sandbox` | 页面脚本 / 扩展脚本运行时；目前仍是占位方向 |

### Test Infrastructure

| Path | Responsibility |
|------|----------------|
| `tests/integration` | 跨 crate 集成测试 |
| `tests/wpt-runner` | Web Platform Tests 相关基础设施 |
| `tests/benchmarks` | benchmark 结果产物 |

## Request-To-Pixels Flow

ZeroBrowser 当前的主线数据流可以按下面理解：

1. `net` 负责 URL、导航和资源获取。
2. `dom` 基于 `html5ever` 把 HTML 解析为 DOM 树。
3. `css-parser` 解析样式规则。
4. `style-system` 把选择器和规则匹配到 DOM 节点，生成计算样式。
5. `layout-engine` 把计算样式转换为布局树和几何信息。
6. `engine-core` 把布局结果转换为绘制命令和合成层。
7. `render-foundation` 把图元输出到 GPU/CPU 渲染后端。
8. `host-runtime` 管理窗口和 surface，把帧显示到平台宿主。
9. `webview-api` 把这条链路包装成嵌入式 API，供浏览器 shell 或第三方应用调用。

这条链路已经在一部分测试和 demo 中跑通，但还没有覆盖“真实网页 + 完整 JavaScript + 完整浏览器 UI”。

## Current Maturity

可以粗略把仓库分成三档：

- **已有实质实现**: DOM、CSS parser、style system、layout engine、engine core、render foundation、host runtime、net、security、storage、protocol、canvas、wasm-sandbox、webview-api
- **框架已在，但产品未成形**: `apps/browser`、`browser-shell`
- **仍是占位方向**: `script-sandbox`

因此，今天的 ZeroBrowser 更接近“一个正在成形的浏览器内核工作区”，而不是“已经完成的浏览器产品”。

## Constraints That Matter

在做设计和实现时，下面这些约束比“写出更多代码”更重要：

- **许可证约束**: 核心路径优先选择 MIT、Apache-2.0、BSD 等宽松许可证依赖
- **最小修改原则**: 避免无关重构和推测性抽象
- **公共 API 文档**: 对外 API 必须有文档注释
- **日志与可观测性**: 正式日志使用 `tracing`
- **验证优先**: 行为变化需要测试，性能敏感路径需要基准或说明

## Recommended Reading Order

如果你是第一次进入这个仓库，建议按这个顺序建立上下文：

1. [README.md](/home/lei/work/ZeroBrowser/README.md)
2. 本文档
3. [docs/specs/zero-browser-spec-rfc.md](/home/lei/work/ZeroBrowser/docs/specs/zero-browser-spec-rfc.md)
4. [docs/goal/zero-browser/master.md](/home/lei/work/ZeroBrowser/docs/goal/zero-browser/master.md)
5. 目标 crate 的 `README.md`
6. 对应 crate 的 `src/lib.rs` 和测试文件

## Good Starting Points

比较适合新贡献者切入的方向：

- 扩展现有单元测试和集成测试
- 补充 WPT runner 的覆盖面
- 完成 `browser-shell` 的产品层骨架
- 推进 `webview-api` 与真实导航链路的衔接
- 修补样式、布局和渲染的兼容性缺口
