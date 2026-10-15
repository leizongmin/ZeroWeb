---
date: 2026-10-15
modules: tests/wpt-runner
---

# WPT runner 单案超时网对同步循环/状态污染无效——守卫墙钟是最后防线

## 问题

html-semantics M1 基线采集（2026-10-15）九域 1151 案遇到两类挂死，均表现为
runner 进程 0 字节输出直到守卫墙钟 kill（rc=124）或长时间无响应：

1. `tabular-data/processing-model-1/span-limits.html`：单案即可挂死。页面用
   `innerHTML +=` 追加 65533 行 + `rowspan=65534` 槽位填充，引擎表算法进入
   有效无限的同步计算。`ZW_CORPUS_CASE_TIMEOUT_SECS=2` 压测下仍挂——单案
   超时网（10s/60s，`corpus_case_timeout`）对此类不经过 await 点的同步循环
   完全失效。
2. `links/downloading-resources/`：18 案逐案跑每案 rc=1 秒级返回；同进程
   全集必挂（~7 分钟无输出）。按 `header-origin*`/`header-referrer*` 前缀
   拆两半各自稳定完成——前案 download/navigation 遗留状态污染后案，且被
   污染案同样越过单案超时网。

## 根因

- runner 的用例超时依赖 JS 执行路径上的检查点；纯同步的原生计算（布局/表
  算法/大循环）不产生检查点机会，超时无法触发。
- 用例间无状态隔离（同进程顺序执行），navigation/download 等跨文档遗留
  状态可无限期阻塞后案的 harness 完成。
- 两种情况最后都只剩 test-guard 墙钟兜底——而 `--json` 模式全部结果在进程
  退出前一次性输出，挂死 = 该域全部数字丢失，不只是挂死案本身。

## 解决与避免

- 通道侧挂账排除：`html_semantics_case_skipped` 按 FIXME 精确排除两类案
  （span-limits 单案 / downloading-resources 整目录），零执行记账，修复后
  移除。基线数字用拆分半跑侧证补账（downloading-resources 18 案全 Timeout，
  无 pass 信号损失）。
- 大域基线采集按域拆跑（`FILTER=<domain>/`），单域挂死不烧全部墙钟，且
  逐域 JSON 便于补跑。
- 排查套路：先逐案跑分清「单案挂」vs「组合挂」；组合挂按文件名前缀/目录
  二分到最小污染集，再决定排除粒度。
- 结构性修法（待做）：同步循环需要原生侧 interrupt（V8/QuickJS deadline
  中断）或用例级子进程隔离；在超时网层面修补无效。
