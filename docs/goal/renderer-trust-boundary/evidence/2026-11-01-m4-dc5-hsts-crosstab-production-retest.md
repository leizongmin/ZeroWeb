# M4 DC-5 补测：切片 7 跨 tab HSTS 全局注册生产面复测（真实网络端到端）

**日期**：2026-11-01
**基线 HEAD**：`f92d3a45c`（切片 7 HSTS per-profile 全局存储）
**性质**：生产面复测（无代码变更）——切片 7 常驻回归为单测级；本轮对「动态注册跨普通 tab 生效」补生产接收方实证，DC-5 已交付面复测覆盖随之收口。

## 方法

沿用 [HSTS 生产面复测](2026-11-01-m4-dc5-hsts-production-retest.md) 配方：`--ui-driver`（Ctrl+L 真实输入）+ `--profile` 隔离会话 + `DISPLAY=:52`（自起 DRI3 headless 显示设施）+ 真实站点 `registry.npmmirror.com`（STS `max-age=5184000`；本域及父域不在 preload 列表，归因可隔离——排除依据沿用上轮 `context.rs` 40 条逐一核对）。差异仅一处：腿 2 前插 **Ctrl+T 新开 tab**，`http://` 导航发生在 tab 2（该 tab 从未自行收到 STS 头）。

二进制新鲜度：`make browser-build` 重建 release（旧 binary 为 16:24 d4768c825 时点，不含切片 7）；行为进一步以行为判别式自证（见下）。

## 正文链（run 1 即过，16 步 exit 0，`/tmp/zw-wayland-rt/hsts-x2.log`）

```
11:06:03.642  HTTP fetch done url=https://registry.npmmirror.com/ status=200      ← tab 1 https 导航加载
11:06:03.642  HSTS registered for registry.npmmirror.com: max-age=5184000         ← tab 1 document 响应头 → 动态注册（切片 7 全局面）
11:06:07.368  [Ctrl+T] → Spawned renderer 2 for tab 2                             ← tab 2 全新普通 tab，独立 SecurityContext
11:06:08.175  fetch upgraded tab 2: http://registry.npmmirror.com/ -> https://…   ← 判别式：tab 2 出站前升级
11:06:08.251  HTTP fetch done url=https://registry.npmmirror.com/ status=200      ← 升级后真实加载（升级先于自身响应 76ms）
11:06:10.121  assert_url=https://registry.npmmirror.com/ status=passed            ← tab 2 提交 URL 为 https
```

**判别式逻辑**：tab 2 自身从未收到 STS 头，切片 5 per-tab 语义下其 `check_resource_url` 无注册可查、`fetch upgraded tab 2` 不可能出现（切片 5 单测的「per-tab 隔离：未注册 tab 同 host 不升级」断言正是此形态）；升级唯一可能来源 = tab 1 注册落入的共享全局存储（切片 7）。升级时间戳（08.175）先于 tab 2 自身响应（08.251），排除了「tab 2 响应头自注册后再升级」的解释。preload 归因排除沿用上轮（域不在列表；且 `HSTS registered` 日志仅出自响应头动态注册路径）。

截图 `01-tab1-https.png` / `02-tab2-http.png` 在册（tab 栏双 tab、tab 2 激活渲染真实 npmmirror 内容；临时产物不入仓）。

## 观察记档（如实，非本 goal 面不追）

1. **地址栏显示与提交 URL 分叉**：升级后 tab 2 提交 URL 为 https（assert_url passed），地址栏文本仍显示用户键入的 `http://registry.npmmirror.com/`（截图 02 可见）。Chrome 地址栏显示升级后 https。属浏览器 UI 显示面（键入文本未随提交刷新），非本 goal 安全/IPC 面——记一行供 UI 流参考。
2. tab 2 加载后响应头二次 `HSTS registered`（同 host 重注册打日志）——上轮已记档语义无碍不修。

## 验证汇总

- 无代码变更 → 无 clippy/fmt/test 门禁触发；release 二进制经 `make browser-build` 刷新至切片 7 源码；真实进程 `--ui-driver` exit 0，正文链事件序列即证据。
- 进程清理：timeout 包裹、正常退出无残留；`--profile`/脚本/日志在 `/tmp/zw-wayland-rt/`（临时产物不入仓）；:52 显示设施按先例保留；:20 用户桌面未触碰。

## DC-5 账面更新

已交付修复面生产接收方复测覆盖：headless CDP 8/8 + GUI 清除链 3/3（[DC-5 复测](2026-11-01-m4-dc5-production-ipc-retest.md)）+ fd 通道 dma-buf 端到端（[M4 补验](2026-11-01-m4-dmabuf-fd-channel-e2e-unblock-and-seccomp-getsockopt-fix.md)）+ 多进程 HSTS 单 tab 动态注册链（[HSTS 复测](2026-11-01-m4-dc5-hsts-production-retest.md)）+ **跨 tab 全局注册升级（本轮，切片 7 面）**。DC-5 余项仅剩决策面复测（依赖 ①②③④ 拍板）。
