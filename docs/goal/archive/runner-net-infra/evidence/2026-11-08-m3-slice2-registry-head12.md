# M3 切片二+三：PyRoute 保序注册表落地 + 链头 23 臂迁入（12+11 合并交付）

日期：2026-11-08。蓝图 = py 普查档 §三；基线锚 = 同日归因档
（[m3-baseline-reattribution](2026-11-08-m3-baseline-reattribution.md)，
日志 `/tmp/m3-baseline-164121.log`）；前置 = 切片一
（[route-module-extract](2026-11-08-m3-slice1-route-module-extract.md)）。
切片二（12 臂）落 54c5f4cea；切片三（+11 臂）与谓词修复随后续提交
落库（54c5f4cea 为纯 12 臂态——git show 零 DirSuffix/Exacts 可证）；
两态各自的 A/B 证据见 §二/§三。

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
- **全量 A/B（12 臂态）**：`/tmp/m3s2-ab-175420.log`（17:54–18:02，
  systemd 单元 m3s2-ab-175420，同 test-guard 门 `--per-proc-mem 4
  --total-mem 8 --time-limit 900`，run 内 cargo Finished 33.31s）对照
  基线 `/tmp/m3-baseline-164121.log`：头条全等 **13,582P/1,337F/8T**；
  逐 subtest multiset（14,915 键含重复计数）对称差**空集**；七域分布
  逐项全等（fetch 2,533 / xhr 1,248 / websockets 741 / streams 1,019 /
  url 7,412 / mimesniff 1,898 / eventsource 76）。对照脚本口径同切片一
  档方法注记（`LC_ALL=C grep -a` 文本模式 multiset）。

## 三、切片三（随后续提交落地：beacon→xhr redirect-cors 11 臂）

本切片在工作树续迁（未含于 54c5f4cea——该提交 18:05:53 早于本切片
转换落树，HEAD 的 .rs 零 DirSuffix/Exacts 可证；12 臂态 A/B 不覆盖
这 11 臂，等价性由本节补证，23 臂态 A/B 为落库行为门）：

- **追加 11 臂**（残链头前缀推进）：beacon/resources 三端点
  （inspect-header.py / content-type-and-body.py / url-echo.py，
  `Exacts` 谓词引入）/ fetch-with-body-worker.py（Exact）/
  xhr upload.py / form.py / echo-content-type.py /
  access-control-origin-header.py / over-1-meg.txt /
  access-control-basic-allow-star.py（`DirSuffix` 引入）/
  fetch echo-content.py+.h2.py 与 xhr access-control-basic-allow(.py+
  -no-credentials.py)（`DirSuffixes` 引入）/ xhr redirect-cors.py。
  残链头推进至 fetch/api/resources/redirect.py。
- **谓词缺陷事故（已修）**：7 条单后缀 DirSuffix 条目初版把列表字面量
  写进字符串（`suffix: "['/upload.py']"` 形态）——永不匹配且原臂已从
  残链删除，相关端点全部落穿静态回退（缺陷仅存于工作树未提交中间态
  ——git show 证 54c5f4cea 纯 12 臂态，从未入任何提交；修复→A/B→
  提交顺序符合门禁）。
  机械验证发现后 7 条修为裸路径字面量；`Exacts`/`DirSuffixes` 条目
  逐项核对与原条件一致。
- **机械等价**：11 条 handler 体与 4601b8945 原臂体逐行比对（dedent +
  Some 包装同口径）零内容差。
- **全量 A/B（23 臂态，谓词修复后）**：`/tmp/m3s3-ab-181241.log`
  （18:12–18:21，systemd 单元 m3s3-ab-181241，同 test-guard 门）对照
  基线 `/tmp/m3-baseline-164121.log`：头条全等 **13,582P/1,337F/8T**；
  逐 subtest multiset（14,915 键含重复计数）对称差**空集**；七域分布
  逐项全等（fetch 2,533 / xhr 1,248 / websockets 741 / streams 1,019 /
  url 7,412 / mimesniff 1,898 / eventsource 76——与 12 臂态同值）。
  此跑为切片三（含谓词修复）落库提交的行为门。另有冗余印证跑
  `/tmp/m3s3-ab-181630.log`（18:16–18:25，同 test-guard 门）同
  multiset 空集。

## 四、切片序列（后续批次）

- 切片四：fetch/api 族（redirect.py / redirect-empty-location /
  method / authentication / clean-stash / preflight / trickle 等，
  DirSuffix 余量）+ eventsource 族；切片五：inspect-headers/方法豁免
  （Custom 谓词引入）+ 尾段杂项臂全量收口（注册表承接 84 臂，残链
  清空）；四 fetcher 闭包臂随后同框架分表迁移。
- S12 cors-preflight-redirect 补丁收编（现 fetch/api/resources/redirect.py
  臂 OPTIONS 分支）随注册表批次落；update 脚本缺口清偿（语料 pin 形态
  核对 → 销账或补条目）收口时办。
