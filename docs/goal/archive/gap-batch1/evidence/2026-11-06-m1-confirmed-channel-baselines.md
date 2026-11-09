# M1 通道基线 —— CONFIRMED 六面干跑（2026-11-06）

通道五件套落地（DC-2）：`fetch-gap-batch1-subset.sh`（十二 corpus 一批落盘）+
六个 runner 子命令臂（`testharness-quirks/domparsing/domxpath/
subresource-integrity/cors/import-maps`）+ `gap_batch1_case_skipped` 内容规则
（ref/manual/非 testharness 形态/iframe/worker）+ Makefile target（test-guard
包裹，--per-proc-mem 4 --total-mem 8 --time-limit 1800）+ weekly 分片接线
（catch-all 缺省约定自动发现，本地 ci_discover_shards 复核：123 套件 / 六新
面入 5 个 catch-all 分片、fetch 目标全对、≥40 断言过）。

## 干跑基线（pin 31597693，2026-11-06，JSON 同目录）

| 面 | 文件 | 子测试 | P | F | T | 通过率 |
|---|---:|---:|---:|---:|---:|---:|
| quirks | 6 | 75 | 35 | 40 | 0 | 46.7% |
| domparsing | 51 | 1,217 | 151 | 1,066 | 0 | 12.4% |
| domxpath | 32 | 124 | 21 | 102 | 1 | 16.9% |
| subresource-integrity | 8 | 78 | 32 | 42 | 4 | 41.0% |
| cors | 23 | 346 | 92 | 252 | 2 | 26.6% |
| import-maps | 46 | 80 | 34 | 23 | 22 (+1 NotRun) | 42.5% |
| **合计** | **166** | **1,920** | **365** | **1,525** | **29** | **19.0%** |

**DC-4 数值门（CONFIRMED，按决策规则 = 干跑基线实数）**：十二面合计干跑 P
365 为零回落下限；分面门 = 各面基线 P 数（35/151/21/32/92/34）。
mixed-content 既有账（blob 1T + imageset 1F）另行零回归锚（M2 同轮验证）。

## 失败簇归因（bucket 级，引擎面归属按 goal 划界）

- **domparsing 1,066F**：主簇 = shim 接口面缺口——
  `object[method_name] is not a function` 224F + `target.append is not a
  function` 112F + Unhandled rejection TypeError 群 ~400F（XML 文档 DOM 接口
  面归属 34 号 dom-expansion 划界）；parseFromString HTML 面基本通过。
- **cors 252F**：`cors-makeheader.py` 动态端点缺位（CORS 判据回体端点——
  runner 无该 wptserve handler 仿真臂）为主簇；`status-async` 群 status=0。
  端点仿真 = runner 基建挂账（mixed-content gen/ 同类先例），不进本 goal DC。
- **quirks 40F**：unitless-length 面为 css-parser quirks 单位解析缺位
  （`expected "567px" but got "1234"` 25F 簇）——CSS quirks 解析语义归
  rendering/css 域，非本 goal；quirks 模式判定本身的通道面（classname/
  supports/history）已过账。
- **domxpath 102F**：`XPathEvaluator` 全局构造器缺位（part06 只接
  document.evaluate，R2981 实用子集）17F+4F 簇；`count()` 等 XPath 函数
  未实现 11F 簇；XMLDocument.evaluate 缺 4F——接口面归属引擎 shim 域。
- **SRI 42F**：`Bad load succeeded` 16F 簇 = script/style 元素加载路径
  integrity 强制缺位（`_zwFetchVerifyIntegrity` 只覆盖 fetch/模块面——
  part01 锚点诚实范围）；样式加载色判据 15F 簇同源。
- **import-maps 23F+22T**：模块解析/多 import map 竞争面散案 F；22T =
  module 加载挂起（import_map.rs 与引擎模块管线交互面），逐案 M3 复核。

## 共享路径修复（本轮随通道落地）

`inline_local_scripts` classic 脚本投递面补 `?pipe=sub` substitution 管道
臂（wptserve `pipe=sub` = 模板替换管道语义，`.sub` 命名约定之外的直引模板
文件——cors support.js `{{ports[...]}}` 形态）。此前 token 残留 →
SyntaxError → CORS 全局 ReferenceError 连坐；修复后 cors 子测试分母
172→346、P 36→92（文件级中止解除，测试全注册）。回归锚 =
`inline_local_scripts_pipe_sub_query_substitutes_tokens` 单元测试（token
替换 + 无 pipe 裸引用不替换双臂）；`cargo test -p zero-wpt-runner --bin`：
285 passed（2 环境性失败 s249/s250 为语料缺文件，clean main 复现同红——
非本轮变更引入）。零回归面：`?pipe=sub` 无 token 文件替换为 no-op；既有
通道 `testharness-mixed-content` 逐位复跑（11 子测试账不变）。
