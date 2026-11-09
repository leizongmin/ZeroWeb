# M1 语料对账（DC-1，2026-11-05）

## 对账方法

上游 pin `315976933870b34d6ea30e3f6643403edae678ba` git trees API
`recursive=1` 全量 blob 清单 vs 本地 `tests/wpt-runner/wpt-data/` 逐扩展名计数。

## dom/ranges：66/66 = 100%

| 扩展名 | 上游 | 本地 |
|---|---:|---:|
| .html | 65 | 65 |
| .js（Range-mutations.js 共享 helper） | 1 | 1 |

本地布局：顶层 44 .html + `tentative/` 19（OpaqueRange/createValueRange 提案面）
+ `crashtests/` 2（非断言崩溃页）。

## html/dom：539/539 = 100%（测试资产口径）

| 扩展名 | 上游 | 本地 | 说明 |
|---|---:|---:|---|
| .html | 487 | 487 | |
| .js | 31 | 31 | reflection.js 等共享 helper |
| .xhtml | 13 | 13 | |
| .headers | 6 | 6 | wptserve sidecar |
| .mjs | 1 | 1 | render-blocking support |
| .css | 1 | 1 | render-blocking support |
| .yml | 7 | 0 | CI 配置，非语料——脚本按扩展名分流不拉 |
| .py | 3 | 0 | 上游生成脚本（含 idlharness 生成器），非语料 |
| .md/.htaccess/.txt | 3 | 0 | 文档/服务器配置，非语料 |

DC-1 预期的「idlharness.any.js 拉 1 个失败」实际未发生：上游 html/dom 无
idlharness 资产（build 期生成，树里本就不存在），deep 拉取零失败、零跳过。

## 与立项日报的差异

立项日（2026-10-27，commit `e546ebaef`）实测 dom/ranges 65、html/dom 489
（raw 限流部分跳过）；本轮对账证明幂等续拉已在后续轮次内自然补全，
两边均达上游 pin 全量。M1 状态行「语料预置未启动」过时——预置实际随
立项完成，缺的是通道扩面（见 2026-11-05-m1-channel-expansion.md）。
