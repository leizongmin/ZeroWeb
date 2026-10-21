# M2-S4V — location.reload() 接线 Navigation API reload 面（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹；FILTER 单案迭代）
**全量运行日志**: [2026-10-10-m2-s4v-location-reload-face.txt](2026-10-10-m2-s4v-location-reload-face.txt)（终跑）；per-subtest 精确 diff 对 [2026-10-10-m2-s4u-window-close-jsurl.txt](2026-10-10-m2-s4u-window-close-jsurl.txt)（S4U 终态）
**前序**: [2026-10-10-m2-s4u-window-close-jsurl.md](2026-10-10-m2-s4u-window-close-jsurl.md)

## 切片内容

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **location.reload() 接线 `navigation.reload()`**（原 no-op 存根不派事件）：navigate 'reload' 事件 + intercept 生命周期；headless 无真文档重载维持 no-op reapply 近似 | `part01.js`（Location operations 表） | spec location.reload ≡ reload 导航——navigate 'reload' 须派发 |
| **reload 目的态承继**：navigation.reload() 的 navigate fire 增 `destState`（当前 entry navState） | `part02.js` | spec reload destination 复用当前 entry——getState() 可见 updateCurrentEntry 写入态；WPT navigate-destination-getState-reload |

## 数字

| corpus 域 | S4U 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api（记账链延续） | 238/255 = 93.3% | **239/255 = 93.7%** | +1 |
| 全量 | 381 P / 476 = 80.0% | **382 P / 476 = 80.3%** | +1 |

**1 翻 Timeout → Pass，零回归**（per-subtest 精确 diff，F/T 集合恰删 1 行零新增）：
navigate-destination-getState-reload。corpus 内 location.reload 其余使用者均为 iframe 门控/
未导入案（零 blast radius，逐案核对）。

## 质量门禁

- `make test`：全绿 **20,361 P / 0 F**（实跑）。
- clippy/fmt：本轮 diff 纯 JS（part01.js/part02.js）+ 文档，无 `.rs` 变更（S4R 片 clippy
  -D warnings 干净、fmt 零 diff 维持）。
- `git diff --check`：零问题。
