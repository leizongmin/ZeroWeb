# M3 切片三：注册表 +11 臂（beacon/service-worker/xhr 头段）

日期：2026-11-08。蓝图 = py 普查档 §三；基线锚 = 同日归因档
（[m3-baseline-reattribution](2026-11-08-m3-baseline-reattribution.md)，
日志 `/tmp/m3-baseline-164121.log`）；前置 = 切片二
（[registry-head12](2026-11-08-m3-slice2-registry-head12.md)）。

## 一、交付面

- `PyRouteMatch` 扩两变体：`Exacts(&[&str])`（多端点 OR 共享 handler，
  beacon 三联）+ `DirSuffix { dir, suffix }` / `DirSuffixes { dir,
  suffixes }`（目录前缀 + 单/多文件名后缀——原多行 `starts_with &&`
  (ends_with || …) 臂 1:1 对位）。
- 首批后 11 条臂迁入（迁移边界前缀推进，残链头 = fetch/api/resources/
  redirect.py）：beacon inspect-header/content-type-and-body/url-echo
  三联 · fetch-with-body-worker · xhr upload/form/echo-content-type ·
  fetch/api/resources echo-content(+h2) · xhr access-control-origin-
  header / over-1-meg.txt / basic-allow-star / basic-allow(+
  no-credentials 二联) / redirect-cors。
- 注册表 23 行序 = 既有臂序全等（切片二 12 + 本批 11）；残链仍用
  dispatch 绑定保留，编译零警告零错误。

## 二、等价性证据

- **机械等价**：11 条 handler 体与 HEAD 原臂体逐行比对（dedent 4 空格 +
  `return X(...)` → `Some(X(...))` 包装两变换）零内容差；谓词逐臂核对
  （beacon 三联 OR ↔ Exacts、echo-content 对/basic-allow 对 OR ↔
  DirSuffixes、单后缀臂 ↔ DirSuffix）。
- **过程记录**（多重 spawn 爆发仲裁，见 master.md 碰撞管理）：本批由
  前序实例生成源码后中断，初版 7 条单后缀 DirSuffix 条目带列表字面量
  缺陷（`suffix: "['…']"`，永不匹配→落穿）——只存在于工作树中间态，
  从未进提交（git show 54c5f4cea 纯 12 臂态可证）；18:12 在树实例
  sed 修复为裸路径字面量 + 机械等价验证，修复随 6be021b87 落库；
  18:14 观察注「终态无缺陷」系只看修复后文件未见修复事件，已对账。
- **全量 A/B（双跑）**：落库行为门 = `/tmp/m3s3-ab-181241.log`
  （18:12–18:21，修复实例起跑，同 test-guard 门）；本实例独立复跑 =
  `/tmp/m3s3-ab-181630.log`（18:16–18:25，systemd 单元 m3s3-ab-181630，
  run 内 cargo Finished 33.84s）。两窗有重叠段但双双对照基线
  `/tmp/m3-baseline-164121.log` 落证一致：头条全等 **13,582P/1,337F/8T**；
  逐 subtest multiset（14,915 键含重复计数）对称差**空集**；七域分布
  逐项全等（fetch 2,533 / xhr 1,248 / websockets 741 / streams 1,019 /
  url 7,412 / mimesniff 1,898 / eventsource 76）。对照口径同切片一档
  方法注记（`LC_ALL=C grep -a` 文本模式 multiset）。

## 三、切片序列（后续批次）

- 切片四：xhr 中段臂（headers/chunked/parse-headers/preflight 族，
  DirSuffix 为主）+ preflight-cache/returns-* 多行 OR 臂（DirSuffixes
  已承接）；切片五：eventsource 族 + fetch/api 尾段 + inspect-headers
  方法豁免/杂项臂收口（84 臂全量入表，残链清空）；四 fetcher 闭包臂
  随后同框架分表迁移。
- S12 cors-preflight-redirect 补丁收编（redirect.py 臂 OPTIONS 分支）
  随注册表批次落——语料 pin 形态核对已销账（36cf18ba4，条目维持）。
