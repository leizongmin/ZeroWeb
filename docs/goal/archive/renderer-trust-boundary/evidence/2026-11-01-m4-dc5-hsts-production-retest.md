# M4 DC-5 补测：多进程 HSTS 生产接收方复测（真实网络端到端）

**日期**：2026-11-01
**基线 HEAD**：`d4768c825`（fd 通道 dma-buf 补验 + seccomp getsockopt 修复）
**性质**：生产面复测（无代码变更）——M2 切片 5（多进程 HSTS 注册）此前仅单测级有据，DC-5 在册余项「HSTS 生产面」本轮收口。

## 方法

`--ui-driver`（Ctrl+L 地址栏真实输入路径）+ 真实公网 HTTPS 站点 + `--profile` 隔离会话（排除 session restore 重放上一会话 URL 的干扰）。本地自签 CA 不可行：net 栈为 reqwest `rustls-tls`（webpki-roots，无系统根加载口），故选真实 STS 站点。

**站点选择关键约束**：内置 HSTS preload 列表（`crates/security/src/context.rs` `load_preload_list`，40 条）会在每个新 per-tab SecurityContext 预置 github.com 等常用域——升级日志必须能归因到**动态注册**（切片 5 修复面）而非 preload。故正文链用 `registry.npmmirror.com`（STS `max-age=5184000`；本域及父域均不在 preload 列表，父域检查 `parts.len()>2` 不误触）。

## 正文链（run 3，`hsts-prod3.log`，--profile 隔离，14 步 exit 0）

```
09:10:54.185  Navigating to: GET https://registry.npmmirror.com/        ← 腿1 导航（https）
09:10:54.360  HSTS registered for registry.npmmirror.com: max-age=5184000 ← 切片5 生产面：真实 document 响应头 → 动态注册
09:10:55.985  Navigating to: GET http://registry.npmmirror.com/         ← 腿2 导航（http）
09:10:55.993  fetch upgraded tab 1: http://registry.npmmirror.com/ -> https://registry.npmmirror.com/
              ↑ 升级先于腿2 自身响应（56.06）到达——唯一可能来源是 54.36 的动态注册（preload 排除已证）
09:10:55.993  fetch IPC enqueue tab 1 req_id=2 type=document https://registry.npmmirror.com/ ← 出站已是 https
09:10:56.060  HTTP fetch done url=https://registry.npmmirror.com/ status=200 bytes=1288  ← 升级后真实加载成功
```

页面渲染截图 `01-https-npmmirror.png` / `02-after-http-nav.png` 在册（临时产物不入仓）。preload 归因排除依据：`context.rs:402-449` 全 40 条逐一核对（npmmirror.com 不在）；`hsts.rs` 父域检查仅对 `parts.len()>2` 生效且父域 npmmirror.com 无注册。

## 同轮附带验证与观察（如实记档）

1. **preload 升级面生产验证**（run 1/2，github.com）：`fetch upgraded tab 2: http://github.com/ -> https://github.com/` + 530KB document https 200——preload 路径在多进程 fetch 管线同样生效（非切片 5 修复面，属既有能力基线）。
2. **run 1 教训**：driver 起始步在窗口/地址栏就绪前发出（首 0.4ms 内），Ctrl+L/type/Enter 静默落空，wait_idle 立即安静提前推进；脚本前置 `wait_ms 1500` 修复（run 2/3 腿1 正常）。
3. **run 2 观察 A（session restore 重放）**：非隔离 profile 下上轮会话 URL（http://github.com/）启动时自动重放导航并经 preload 升级——既有设计，正文链用 `--profile` 隔离规避。
4. **run 2 观察 B（docs.rs 超时）**：`https://docs.rs/` 在浏览器 net 栈 30s Timeout（shell curl 同刻 <8s OK）；疑传输栈差异（IPv6/HTTP2 路径），与本 goal 面无关，不追——选站换 npmmirror。
5. **日志措辞 nit（不修）**：`HSTS registered` 在重复注册时也打（`register_from_header` 解析成功即 true）；语义无碍，run 3 的 56.06 二次日志即腿2 升级后响应的正常重注册。
6. per-tab HstsStore 粒度不变（与切片 5 记档一致：真浏览器全局存储为后续对齐项，非本轮范围）。

## 验证汇总

- 无代码变更 → 无 clippy/fmt/test 门禁触发；三轮真实进程 `--ui-driver` 均 exit 0，正文链事件序列即证据。
- 进程清理：无残留；`--profile` 隔离目录与脚本/日志均在 `/tmp/zw-wayland-rt/`（临时产物不入仓）。

## DC-5 账面更新

已交付修复面生产接收方复测覆盖：headless CDP 8/8 + GUI 清除链 3/3（[DC-5 复测](2026-11-01-m4-dc5-production-ipc-retest.md)）+ fd 通道 dma-buf 端到端（[M4 补验](2026-11-01-m4-dmabuf-fd-channel-e2e-unblock-and-seccomp-getsockopt-fix.md)）+ **多进程 HSTS 生产链（本轮）**。DC-5 余项仅剩决策面复测（依赖 ①②③④ 拍板）。
