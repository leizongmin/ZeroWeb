---
date: 2026-09-13
modules: webview,renderer,compositor,protocol,render-foundation,layout-engine,engine
---

# 修正字体 URL 后丢字：资源与度量必须穿过整条生产链路

## 问题描述

morning.work 外部 CSS 的字体相对地址被按文章 URL 解析，产生 404。
仅修正 URL 后字体请求成功、WebView 单测通过，但生产 GPU 窗口的代码文字消失。
最小用例用非 Ahem 名称的 CSS alias 显示 40px `XXXX`：Chrome 为 160×40 绿块，
原版宽约 88px，URL-only 候选丢字；补资源后又暴露基线与 intrinsic 度量错误。

## 根因分析

1. 外链 CSS 合并丢失声明所在 URL。旧字符串扫描还会误改注释/字符串，并漏掉
   quoted URL 的结束括号。CSS tokenizer 已提供可复用的 token/offset 边界。
2. 下载字体只注册在 renderer；compositor 仅加载平台字体。跨 IPC 的数字 font ID
   不是跨进程共享资源，也不是不同 surface 间的全局身份。
3. latest-wins 合并允许丢掉任意中间帧。“曾发送字体注册”不代表最终消费者收到了。
4. 更新字体 resolver 不足以刷新整条度量链：inline-only 重测没有携带字体上下文，
   shrink-to-fit 又使用估算宽度，paint 的空 styles 回退没有恢复真实 ascent。
5. 首帧到达早于字体加载完成。采集来源是生产 GPU，也仍可能采到错误时机的帧。

## 解决方案

同步/异步 CSS 合并共用 tokenizer URL 归一化；快照传递当前引用的下载字体资源，
按 surface/document 导入并映射 ID，命名空间变化统一清理 CPU/GPU 栅格缓存。
未变化资源不重复解析，下载字体随导航释放；无效资源不能替换可用 registry。

字体加载后同步 resolver 和 metric map；纯文本下载字体的 intrinsic 与 IFC 共用
测量路径，重测传入字体上下文，并把下载字体 ascent 保存在 layout 输出、恢复到
paint 回退路径。初始采集等待同一导航的有效完成通知，不以首帧作为字体就绪证据。

必须同时验证请求、字体身份、布局尺寸、paint 基线和最终像素。用自定义 alias
避开 `Ahem` 字体名的合成捷径；增加不同字号/字数、Ahem/Lato 相同 ID 的多 surface、
导航释放、同步/线程化光栅与 browser UI 字体隔离断言。

严格区域对照很重要：一次中间版本全图差异仅 0.18%，字体区域却差 12.5%，仍应
拒绝。最终无固定宽高的生产用例为 160×40，区域差异 0/6400，几何差 0px。
范围、失败实验和未通过的既有工程门槛见
[首例验收报告](../../../acceptance/site-optimizer-font-pipeline.md)。
