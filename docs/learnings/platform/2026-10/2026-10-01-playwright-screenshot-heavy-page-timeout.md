---
date: 2026-10-01
modules: render-foundation, testing
---

# playwright page.screenshot 在 ZeroWeb 重量级结果页需 15s+，自设 10s 超时必失败

## 问题描述

用 playwright `page.screenshot({ timeout: 10000 })` 对 html5test.co 出分后的完整结果页
（500+ 行结果表）截图，稳定超时（Timeout 10000ms exceeded）；同一二进制对本地轻量页面
截图秒级完成；对卡死态轻内容页面也秒级完成。间歇性表现为「截图偶发失败」，且容易误判
为「修复破坏了渲染」。

## 根因分析

ZeroWeb 的 CDP 截图链路（Page.captureScreenshot → 合成 → 序列化）在重量级 DOM 页面上
实测约 15s（1280×900，完整结果页；`SHOT OK 15059 ms`）。playwright 默认超时 30s 能容纳，
自设 10s 不能。耗时与页面 DOM/绘制量正相关，与系统负载相关（批量采样进行中会更慢）。

早期 t3 探针用默认超时所以一直成功；新探针「收紧」超时反而制造了假阳性。

## 解决方案

对 ZeroWeb 重页面的截图一律用 ≥45s 超时：

```js
await page.screenshot({ path: out, timeout: 45000 });
```

## 如何避免

探针脚本截图参数不要低于 45s；「截图超时」先区分慢与挂（用 30s+ 重跑一次），
不要直接升级为产品缺陷。
