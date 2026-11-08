# M3 切片二：PyRoute 保序注册表落地 + 链头 12 臂迁入

日期：2026-11-08。蓝图 = py 普查档 §三；基线锚 = 同日归因档
（[m3-baseline-reattribution](2026-11-08-m3-baseline-reattribution.md)，
日志 `/tmp/m3-baseline-164121.log`）；前置 = 切片一
（[route-module-extract](2026-11-08-m3-slice1-route-module-extract.md)）。

## 一、交付面

- `PyRoute` 保序注册表（testharness_py_routes.rs）：`PyRouteMatch` 谓词
  （本批落地 `Exact` 变体——路径全等；`DirSuffix`/`Custom` 随 xhr 族与
  inspect-headers 批次引入）+ `PyRouteHandler`（`fn(&PyEndpointCtx, clean,
  path_part, req) -> Option<Result<FetchResponse, String>>`——`Some` =
  终答，`None` = 落穿下一臂，承载既有臂内条件不返回即续链语义）+
  `py_routes()` 表（表序 = 既有 if 链臂序，先命中先返回）。
- 首批 12 条全等匹配臂迁入（dispatch 链头：站点根 is_empty →
  image-and-stash.py 段）：cache.py / set-cookie.py / echo-content-escaped.py /
  stylesheet.py×2 / serve-custom-response.py / css.py /
  referrer-checker-img.py / 404-response-with-actual-image-data.py /
  image-alternate-no-store.py / image-and-stash.py + 站点根。
- dispatch 改注册表循环在前 + 残链在后（迁移边界前缀推进——残链臂序
  不变，后续批次逐段前移）；臂体独占的 6 个 dispatch 局部绑定删除
  （本次修改孤儿），残链仍用 4 绑定保留。

## 二、等价性证据

- **机械等价**：12 条 handler 体与 HEAD 原臂体逐行比对（dedent 4 空格 +
  `return X(...)` → `Some(X(...))` 包装两变换）零内容差（变换数 15 =
  14×Ok + 1×fixture；referrer 臂 2 处、image-and-stash 臂 3 处）；
  Exact 谓词由原 `if clean == "..."` 条件程序化提取（`is_empty()` ↔
  `Exact("")` 同语义）。
- **全量 A/B**：`/tmp/m3s2-ab-175420.log`（17:54–18:02，systemd 单元
  m3s2-ab-175420，同 test-guard 门 `--per-proc-mem 4 --total-mem 8
  --time-limit 900`，run 内 cargo Finished 33.31s）对照基线
  `/tmp/m3-baseline-164121.log`：头条全等 **13,582P/1,337F/8T**；逐
  subtest multiset（14,915 键含重复计数）对称差**空集**；七域分布逐项
  全等（fetch 2,533 / xhr 1,248 / websockets 741 / streams 1,019 /
  url 7,412 / mimesniff 1,898 / eventsource 76）。对照脚本口径同切片一
  档方法注记（`LC_ALL=C grep -a` 文本模式 multiset）。

## 三、切片序列（后续批次）

- 切片三：xhr 族臂（DirSuffix 谓词引入，dispatch 586-2285 段）；
  切片四：eventsource + fetch/api 族；切片五：inspect-headers/方法豁免
  （Custom 谓词引入）+ 尾段杂项臂全量收口（注册表承接 84 臂，残链清空）；
  四 fetcher 闭包臂随后同框架分表迁移。
- S12 cors-preflight-redirect 补丁收编（现 fetch/api/resources/redirect.py
  臂 OPTIONS 分支）随注册表批次落；update 脚本缺口清偿（语料 pin 形态
  核对 → 销账或补条目）收口时办。
