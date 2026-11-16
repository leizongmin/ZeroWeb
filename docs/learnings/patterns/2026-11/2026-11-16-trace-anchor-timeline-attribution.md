---
date: 2026-11-16
modules: apps/renderer,apps/compositor,bench
---

# 跨进程仪拍锚提取：时间线归属法与 stderr 前缀伪影

## 问题描述

T76 管线段仪拍（`ZW_FRAME_PIPELINE_TRACE`：renderer `relay_write` + compositor
`queue_recv`/`raster_done`，`publish` 复用 `FRAME_PUBLISH_TRACE`）的日志提取器
按 `[renderer-N]` stderr 前缀给锚行归类，出现两类静默错误：

1. 某样本 claim renderer 的 `relay_write` 行**丢失 stderr 前缀**，被归类到
   browser 侧后整条锚链判「缺失」，该帧（全样本首帧最慢真实点 453ms）被从
   判读中静默剔除，报告归因为「锚缺失」——与原始日志矛盾。
2. 判读器按裸 `frame_id` 直键归组：renderer 实例更替后 fid 撞号，前一实例的
   compositor 锚与后一实例的 publish 锚混入同一帧，产生 `transport=-1503ms`
   的物理不可能分段。

## 根因分析

- renderer stderr 是 tracing 订阅者按 **chunk** 回显落盘：同 chunk 内次行起
  `[renderer-N] ` 前缀丢失。任何按前缀直键的行分类都不可靠——前缀是伪影，
  不是身份。
- `frame_id` 是 renderer 实例内自增的，跨实例必然撞号；「compositor 侧锚无
  前缀所以归 compositor」也不成立，因为锚的归属对象是「哪个 renderer 会话的
  帧」，不是「哪个进程发的日志行」。

## 解决方案

**时间线归属法**：全部 publish 行（含无前缀的）先按墙钟排序登记为归属事件；
每个非 publish 锚归属「同 fid、时刻最近且先行」的 publish 事件——

```python
recent = []           # (t, renderer, fid) 只收 renderer 侧 publish
for t, kind, r, fid in sorted(events):
    if kind == "pub":
        recent.append((t, r, fid))
    else:             # relay_write/queue_recv/raster_done
        cand = [x for x in recent if x[2] == fid and x[0] <= t]
        if cand:
            anchors.setdefault((cand[-1][1], fid), {})[kind] = t
```

- 归属键从 `(前缀, fid)` 换成 `(归属实例, fid)`，撞号自然消解（判读器同法，
  每条 publish 事件挂独立的锚集）。
- 残余风险要披露：实例交替发布时（A publish→B publish→锚到达）锚可能归错
  实例；用 `unanchored` 计数与负段帧检查兜底。
- 提取器 docstring 必须披露前缀伪影，防止后续按「日志里有前缀」直觉改回直键。

## 如何避免

凡跨进程/跨实例的仪拍锚提取（T77 光栅段细分、T78 内存仪拍同构），默认用
时间线归属；前缀只作展示，不作归属键。判读时检查三个哨：负段帧（物理不可能
分段=归属错）、unanchored 计数、同帧锚链完整性；「锚缺失」归因必须回原始
日志逐行核对后再写进报告。
