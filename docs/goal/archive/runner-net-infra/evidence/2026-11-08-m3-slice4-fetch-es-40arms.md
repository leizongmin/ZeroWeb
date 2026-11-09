# M3 切片四：残链清障 + 40 臂合并批迁入（fetch/api 头段 + xhr 中段 + eventsource 主块）

日期：2026-11-08。蓝图 = [切片四前置普查档](2026-11-08-slice4-residual-census.md)
+ py 普查档 §三；基线锚 = [归因档](2026-11-08-m3-baseline-reattribution.md)
（日志 `/tmp/m3-baseline-164121.log`）；前置 = [切片二+三档]
(2026-11-08-m3-slice2-registry-head12.md)。执行批界与普查档 §二 的差异见
§四；本批落库后残链 19 臂，链头 = xhr/resources/get-set-cookie.py。

## 一、交付面

- **死臂清障**：残链头 redirect-cors 残留臂（切片三 splice 边界遗留，
  与注册表行 23 同谓词；handler 全路径 Some ⇒ 不可达死代码）删除。
  根因定位：转换脚本 splice 区间解包取 `bounds[-1]` 元组首元（末臂
  起点）而非边界臂起点——每批最后一臂必残留（切片三 redirect-cors、
  本批 cors-cookie 同源），本批两处均清。
- **追加 40 臂**（迁移边界前缀推进）：fetch/api/resources/redirect.py
  （DirSuffix）/ redirect-empty-location / method / authentication（Exact）；
  xhr authentication 族（**Custom 谓词引入**——全等 + authN 目录三委托
  端点复合条件）/ image.gif / echo-headers（**Suffix 引入**——裸后缀）/
  delay（Suffix）/ trickle / top.txt（Exact）/ basic-put-allow /
  reset-token / preflight-cache 三联 / returns-\* 对（DirSuffixes）/
  headers / chunked / parse-headers / safelisted 响应·请求头对 /
  no-custom-header / lowercase / sorted / headers-origin / invalid-status /
  echo-content-cors（DirSuffix）/ fetch authentication（Exact）/
  dump-authorization（Suffix）/ corsenabled / redirect / well-formed.xml /
  pass.txt / content（DirSuffix）/ status.py（**Custom**——xhr/ 前缀 +
  fetch/api 精确双域）/ eventsource 九臂（message → cors-cookie，
  DirSuffix）。
- 注册表 63 行序 = 既有臂序全等（23 + 40，合流后 64 行见 §五）；残链
  19 臂保留。编译核验勘误：初版「零警告」系执行实例误记——A/B 编译
  实际带 37 条 unused-ctx warning（`/tmp/m3s4-ab-185020.log` 法证可
  证），合流后 `cargo fix` 修复为零警告，见 §三/§五。
- **谓词变体终态**：Exact/Exacts/DirSuffix/DirSuffixes/Suffix/Custom 六种。
  普查档 §一「四变体已足」修正两处：裸后缀臂 ×3 需 `Suffix`（DirSuffix
  需目录前缀，不可承载）；status.py 双域以 Custom 单表项落（普查档双
  表项拆分方案被替代——语义等价，表项更省）。

## 二、等价性证据

- **机械等价（双件套）**：40 条 handler 体与 HEAD 原臂体逐行比对
  （dedent 4 空格 + `return X(...)` → `Some(X(...))` 包装两变换；批内
  零 `return Err`）零内容差；谓词逐臂程序化提取比对（含 Custom 两臂
  纯函数体与原条件空白归一全等、Suffix/DirSuffixes 行与原条件逐 token
  对位）。转换脚本初版 28 条单后缀 DirSuffix 行曾复发列表字面量缺陷
  （`suffix: "['…']"`，与切片三同根——脚本模板未修），机械验证前
  `grep "suffix: \"\['"` 计数拦截，28 条修正为裸路径字面量后过验证。
- **全量 A/B**：`/tmp/m3s4-ab-185020.log`（systemd 单元 m3s4-ab-185020，
  test-guard 门 `--per-proc-mem 4 --total-mem 8 --time-limit 900`，
  run 内 cargo Finished，墙钟 shell 层 1500s）对照基线
  `/tmp/m3-baseline-164121.log`：**multiset 空集**——双侧各 14,927 条
  结果（分母一致，均跑至末案 historical.html 收尾；A/B 尾部 `EXIT=1`
  与基线 `make_exit=2` 同形态，全量跑含预期失败退出码非零属常态）；
  总账 13,582P/1,337F/8T 全等；七域锚点逐项全等：wss 面 Pass
  424=424、S3b constructor/016 2=2、S12 cors-preflight-redirect
  10=10、inspect-headers 面 0=0（该族在残链六b 未迁，行为不变）。
  对照分析于 20:17–20:20 由后续 session 以本日志补齐——执行实例在
  A/B 跑完（18:58）后中断（spawn 爆发余波），对照当时未做。

## 三、过程记录

- 本批由在编实例全链执行（转换脚本 /tmp/m3s4_convert.py → 机械验证
  /tmp/m3s4_verify.py → cargo check → 全量 A/B）；转换脚本两缺陷
  （splice 解包、list-repr 模板继承）均在验证/对账环节拦截并修于
  落库前，未进任何提交。
- 普查档 62-vs-61 口径注记（「非臂 if」）修正：差 1 = 死重复臂
  redirect-cors（真实臂形态 if），git show 注册表行 23 同谓词可证；
  普查档残链头「redirect-cors」同因（b01a2b098 树态残链头即死臂）。
- 执行实例在 A/B 跑完（18:58）后中断（spawn 爆发余波），对照分析
  当时未做；20:17–20:20 后续 session 以孤儿日志补齐 multiset + 七域
  复核（§二），随后完成合流与落库（§五）。
- 编译警告勘误：初版 cargo check「零警告」记录不实——A/B 编译
  （终态）即带 37 条 unused-ctx warning（`/tmp/m3s4-ab-185020.log`
  grep 计数与合流后重跑一致）；根因 = 迁移生成签名对无 stash
  preamble 的 handler 统一写 `ctx:` 而非 `_ctx:`。合流后 `cargo fix`
  全量修复（37 处改名，语义空变）。

## 五、合流记录（FF 取齐 + 冲突解决 + 修复后复核）

- main 524b3b548→a47d00cf9 FF 六提交（autostash），冲突一处 =
  注册表表尾（上游 s346 追加行 vs 本批 40 行同锚追加）。解法：原链
  序连续性优先——本批 40 行紧随 23 既有臂（行 24–63），s346
  module-fetch-tester.py 行合流至表尾第 64 行（新端点非原链臂，上游
  本意即追加表尾）；handler 函数体两侧共存无冲突。
- 语义不相交判定（三条）：s346 行 Exact 谓词与 40 臂谓词面零交集
  （html/semantics 路径无本批谓词可命中，行序无行为差）；d8d563e14
  trickle POST 修复为 dispatch 上游包裹器且拦截路径
  fetch/api/resources/trickle.py 与本批迁移臂 xhr/resources/trickle.py
  为不同端点（零接触）；c6dcbce0a http-cache M2 wire Err→TypeError
  在 fetcher 闭包层，本批零 return Err、None=落穿不变。
- 合流后复核链：verify 脚本重跑（rustfmt 折行 + 数组尾逗号后按本档
  既定「空白归一」口径比对）40 臂等价/谓词全对/死臂已清/残链 19
  不变式成立；`cargo fix` 37 处 + `rustfmt -p` 后 cargo check 零
  警告、fmt 净；runner 单测 298P/0F（含 s346 module-fetch-tester
  状态机钉；其前置 = 对 origin 既有红 testharness.rs:14416 E0063 的
  最小补齐 `cache: None`——c6dcbce0a 漏改 test 初始化器，缺省语义
  无歧义，一行机械修复随本批落库）。
- 账面口径注记：本批 A/B 基线为合流前 16:41 基线（multiset 空集）
  ——合流带入的账面扰动（trickle POST 200 空体、http-cache wire
  等）归上游各自提交与其 A/B 账；下一批 A/B 前须对合流后 HEAD
  重录基线锚。

## 四、批界与普查档差异（执行对照）

- 普查档 §二 五批次（四 ~19 / 五 ~14 / 六a ~11 / 六b ~5 / 六c ~13）；
  本批合并执行 四+五+六a 前段 = 40 臂（fetch redirect.py → cors-cookie），
  机械双件套 + 全量 A/B 门不变，批数减少省两轮全量 A/B 墙钟。
- 残链 19 臂 = 普查档六a 尾（get-set-cookie/.asis/cors.py）+ 六b
  （clean-stash/preflight/inspect-headers 族）+ 六c 尾段全等 13 臂；
  后续切片按普查档 §二 续推。
- S12 cors-preflight-redirect 收编面（fetch/api/resources/redirect.py 臂
  OPTIONS 分支）已随本批迁入注册表（route_fetch_redirect）。
