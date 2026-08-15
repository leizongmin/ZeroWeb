# ZeroBrowser 图标生成器 (`zero-icon-gen`)

> 从源 SVG 生成各平台所需的图标资产

## 概述

`zero-icon-gen` 是 ZeroBrowser 的图标资产生成工具。从一份源 SVG 出发，批量产出 Linux、Windows、macOS 三端以及运行时窗口所需的全部图标格式与尺寸，避免手工导出和多工具拼接。

## 运行

```bash
cargo run -p zero-icon-gen
```

自定义源 SVG 或输出目录：

```bash
cargo run -p zero-icon-gen -- --svg path/to/icon.svg --out path/to/outdir
```

查看帮助：

```bash
cargo run -p zero-icon-gen -- --help
```

## 默认值

| 项     | 默认路径                            |
| ------- | ----------------------------------- |
| 源 SVG  | `apps/browser/assets/app-icon.svg`  |
| 输出目录 | `apps/browser/assets/icons-gen/`    |

## 产物

| 产物                          | 用途                                                  |
| ----------------------------- | ----------------------------------------------------- |
| `icon-16.png` … `icon-512.png` | Linux `.desktop` 与运行时窗口图标                     |
| `zero-browser.ico`            | Windows 应用图标（含 16/32/48/64/256 多尺寸）         |
| `iconset/icon_*.png`          | macOS `.iconset`，配合 `iconutil` 生成 `.icns`        |
| `window-icon-256.rgba`        | 运行时 `winit::window::Icon` 使用的 256px RGBA 原始数据 |

### macOS `.icns`

macOS 的 `.icns` 必须在 macOS 上额外执行（依赖系统自带 `iconutil`）：

```bash
iconutil -c icns <out>/iconset -o <out>/zero-browser.icns
```

封装脚本见 `scripts/package-macos.sh`。

## 渲染质量

- **超采样降采样**：≤64px 的小尺寸采用 4× 超采样后 box-filter 降采样，显著减少锯齿与半透明边缘损失。
- **大尺寸直接光栅化**：≥128px 时直接 `resvg` 输出，避免无谓的中间像素开销。

## 依赖

```
zero-icon-gen
├── resvg  — SVG 解析与光栅化
├── png    — PNG 编码
└── ico    — Windows ICO 容器封装
```
