# 切片四前置：残链 61 臂普查 + 谓词变体映射 + 批次切分

日期：2026-11-08。基态 = b01a2b098（注册表 23 臂：
Exact/Exacts/DirSuffix/DirSuffixes 四变体）；残链 = dispatch 注册表循环后
62 个 4 空格缩进顶层 `if`（含原 84 臂中未迁 61 + 口径注记 1，§四）。
本档为切片四~六执行前置，逐臂普查以本档 + 执行切片证据档为准。

## 一、谓词变体映射（结论：现有四变体已足，Custom 随切片五引入）

- 裸 `starts_with("xhr/")` 两臂实为**多后缀析取**（preflight-cache 三后缀 /
  returns-origin 两后缀）→ 既有 `DirSuffixes` 直承载，无需新变体。
- `status.py` 双域臂（`ends_with("/status.py") && (starts_with("xhr/") ||
  == "fetch/api/resources/status.py")`）→ 拆两条**连续表项同 handler**
  （DirSuffix {xhr/, /status.py} + Exact，先命中先返回语义严格等价）。
- inspect-headers 方法豁免族三臂（405 非 GET/OPTIONS 非 inspect、
  OPTIONS preflight、inspect-headers 回显）→ 方法条件谓词，`Custom(fn)`
  变体随该批引入（普查档 §三蓝图「方法豁免」口径落地）。

## 二、批次切分（每批全量 A/B multiset 空集门）

> 执行注记（同日切片四档）：切片四实际以 40 臂合并批执行（本表四+五+
> 六a 前段，fetch redirect.py → cors-cookie），谓词面引入 Suffix/Custom
> （§一 两处修正见切片四档 §一/§三）；残链 19 臂按本表六a 尾 → 六b →
> 六c 续推。

| 批次 | 臂段（残链序） | 臂数 | 新谓词面 |
|---|---|---:|---|
| 切片四 | xhr 续段（redirect-cors → access-control-preflight-request-header-lowercase 前）+ fetch/api 全等四臂 | ~19 | 无（四变体承载） |
| 切片五 | xhr 收口（no-custom-header → status.py）+ fetch authentication + dump-authorization + corsenabled/redirect/well-formed/pass/content | ~14 | status.py 拆双表项 |
| 切片六a | eventsource 族（message → cors.py）+ get-set-cookie + .asis | ~11 | 无 |
| 切片六b | clean-stash + preflight.py + inspect-headers 方法豁免族 | ~5 | `Custom(fn)` |
| 切片六c | 尾段全等杂项（fetch trickle → load-error-events）+ 静态回退尾保留 | ~13 | 无 |

- 残链清空后：dispatch 只剩注册表循环 + 静态回退尾（注册表承接
  py 面 100%）；随后按 [fetcher 清单档](2026-11-08-fetcher-closure-inventory.md)
  分表迁 script/service-worker 闭包臂。
- 批内机械等价验证沿用切片二/三双件套：handler 体 vs HEAD 原臂体
  dedent+Some-wrap 逐行比对 + 谓词程序化提取比对（切片三曾以该法抓到
  DirSuffixes 列表字面量缺陷——每批必做）。

## 三、残链族序（执行对照用）

xhr 续段（redirect-cors 起）→ fetch/api 全等（redirect-empty-location/
method/authentication）→ xhr 回显/延迟/截断族（echo-headers/delay/trickle/
top.txt/image.gif）→ xhr CORS preflight 判据族（preflight-cache 族 →
safelisted 头族 → no-custom-header → lowercase/sorted/headers-origin/
invalid-status → echo-content-cors）→ fetch authentication/dump-authorization
→ xhr corsenabled/redirect/well-formed/pass/content/status → eventsource
族 ×9 + get-set-cookie/.asis → clean-stash/preflight/inspect-headers 族 →
fetch trickle/bad-chunk-encoding → 尾段全等（common/redirect、dom encoding、
cache-storage 四臂、cors-approved/denied、blank.html、websockets check、
xmldecl trickle、load-error-events）。

## 四、口径注记

- 残链顶层 if 计数 62 vs 84−23=61：差 1 = residual 提取窗口含注册表
  循环尾/残链边界行一条非臂 if（执行切片迁移时按臂注释逐条核对，
  不以计数硬门——臂清单以内容为准）。
- 批次臂數为普查概数（±1），执行切片以实际提取臂清单落档。
