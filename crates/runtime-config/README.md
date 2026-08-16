# ZeroWeb Runtime Config (`zero-runtime-config`)

> ZeroWeb 运行时环境变量配置的唯一入口

## 概述

`zero-runtime-config` 集中定义并解析 ZeroWeb 浏览器运行时的环境变量开关（渲染后端、多进程、沙箱、compositor 等）。业务 crate 不应直接读取环境变量：新增产品级开关时，先在本 crate 定义名称、默认值与解析函数，再同步更新 `docs/runtime-environment.md`。

## 主要功能

- **权威清单** — `ENVIRONMENT_VARIABLES` 常量表列出全部受支持的环境变量（名称、默认值、用途说明），覆盖渲染后端（`ZEROWEB_RENDERER`）、子进程路径（`ZERO_RENDERER_PATH` / `ZW_IMAGE_DECODER_BIN`）、网络（`ZERO_HTTP2` / `ZERO_MAX_CONNECTIONS_*`）、隐私与缓存（`ZERO_PRIVATE` / `ZERO_CACHE_DIR`）、compositor 性能与沙箱参数（`ZW_COMPOSITOR_*`）
- **统一解析函数** — 面向不同开关语义的解析原语：`enabled_when_true`（仅 `1` / `true` 启用）、`enabled_by_default`（默认启用，`0` / `false` 禁用）、`enabled_unless_zero`（兼容 kill-switch 语义）、`optional_path` / `optional_string`（空值视为未配置）、`positive_usize`（正整数配置，非法值回退默认）
- **渲染模式解析** — `renderer_mode()` 读取 `ZEROWEB_RENDERER`，返回 `auto` / `gpu` / `cpu` 之一的原始值，非法 UTF-8 显式报错
- **单点事实来源** — 业务 crate 通过本 crate 读取配置，避免环境变量名称与语义在多个 crate 间漂移

## 使用示例

```rust
use zero_runtime_config::{enabled_by_default, positive_usize};

// 默认启用的开关：仅 "0"/"false" 禁用
let compositor = enabled_by_default("ZW_COMPOSITOR_ASYNC_SCROLL");

// 正整数配置：非法值回退默认
let max_conns = positive_usize("ZERO_MAX_CONNECTIONS_PER_ORIGIN", 6);

// 遍历权威清单（如生成环境变量文档）
for var in zero_runtime_config::ENVIRONMENT_VARIABLES {
    println!("{} (默认 {})：{}", var.name, var.default, var.description);
}
```
