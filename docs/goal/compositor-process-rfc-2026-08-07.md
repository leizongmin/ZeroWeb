# 合成器独立进程 + GPU 隔离 RFC（#4 调研建议，D 组多进程演进）

版本：v1.0（设计稿，待实施规划）｜ 日期：2026-08-07 ｜ 状态：设计

> 依据：Ladybird 2026-05 合成器独立进程 + 2026-06 WebContent 不再直接访问
> GPU（canvas/WebGL 命令在沙箱化合成器进程回放，共享内存传输）（调研报告
> §3.3/§3.6）。动机：GPU 驱动漏洞是浏览器攻击面，隔离到最小权限进程。
> 前置：D1（ImageDecoder 独立进程）已完成——本 RFC 是多进程演进的后半。

## 一、现状审计（2026-08-07）

| 项 | 当前实现 |
|---|---|
| 合成 | 内嵌于 engine pipeline（无独立 compositor 模块） |
| GPU 访问 | `crates/render-foundation/src/gpu/`（wgpu：atlas/mesh/pipeline/renderer）——**渲染进程内直接访问 GPU**，无进程隔离 |
| 帧输出 | 渲染进程帧缓冲 → IPC → 浏览器（apps/renderer 的 paint_export） |
| 已隔离 | 网络（fetch 走 browser）、图像解码（D1 image-decoder 进程） |

## 二、目标架构（三片，对照 Ladybird 演进顺序）

```
切片 C1：Compositor 模块显式化（纯重构，零行为变更）
  现状：合成逻辑散在 pipeline
  目标：engine 内 Compositor 模块（帧缓冲管理 + 图元提交接口）
  验证：reftest/oracle 全量无 diff

切片 C2：合成器独立进程（合成层出进程）
  apps/compositor（zero-compositor）：帧缓冲合成 + backing store 管理
  renderer → Compositor 经 protocol IPC 提交图元/命令
  验证：多进程渲染与进程内逐像素一致（A/B）

切片 C3：GPU 访问移入合成器（GPU 隔离）
  wgpu 上下文从 renderer 移入 compositor 进程；renderer 不再直接访问 GPU
  canvas/WebGL 命令经共享内存（SharedMemoryChannel）回放到 compositor
  验证：GPU 路径（browser --renderer=gpu）与基线一致 + 合成器沙箱生效
```

## 三、收益与成本

| 收益 | 成本/风险 |
|---|---|
| GPU 驱动漏洞隔离（最大攻击面之一） | 多进程 IPC 复杂度（图元/命令传输） |
| 合成器崩溃不拖垮渲染（Ladybird 同路径） | wgpu 上下文迁移（surface/资源所有权） |
| 为 GPU 隔离 + 沙箱化铺路（Linux seccomp） | reftest headless 路径保留进程内合成 |

## 四、实施要点

- **协议扩展**：`zero-protocol` 新增 Compositor 消息族（帧提交/命令流/backing store 管理），
  参照 D1 的 ImageDecode 消息模式（request_id 匹配）
- **共享内存**：canvas/WebGL 大体积命令走 `SharedMemoryChannel`（transport.rs 已有实现）
- **沙箱**：compositor 进程 Linux seccomp 最小权限（LibSandbox 模式——Ladybird 2026-07 每进程独立沙箱规则）
- **回退**：`ZW_COMPOSITOR_PROCESS=0` 切回进程内合成（D1 同款 fail-open 模式）
- **headless/测试**：wpt-runner/reftest 保留进程内合成（测试确定性优先，同渲染线程 RFC）

## 五、验收标准（C2 合入）

1. `make test` / reftest / oracle 与基线无差异
2. 多进程渲染逐像素 = 进程内渲染（A/B）
3. product-smoke diff ≤ 阈值
4. 回退开关可用；compositor 崩溃 → renderer 不崩（进程隔离验证）

## 六、与其他 RFC 的关系

- 前置/并行：`render-threading-rfc-2026-08-07.md`（C2 可在 S2 后实施——渲染线程
  产出的 DisplayList 就是合成器进程的输入边界）
- 依赖 D1 的 protocol 消息模式先例
- GPU 隔离（C3）与渲染线程 RFC 的「GPU 光栅化线程」合并规划（同一 wgpu 上下文迁移）
