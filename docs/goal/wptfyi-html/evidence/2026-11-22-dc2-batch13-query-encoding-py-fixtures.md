# DC-2 · 批十三：query-encoding 族 .py fixture 备料腿（批十挂账）＋批十二勘误

2026-11-22。批十刀位余量「resource.py/blank.py/stash.py/css-tmpl.py fixture
缺面（runner 本域可先备料）」落地：resolve-url.js 15 面 × 9 基页文档编码矩阵
的被测端点此前未仿真（静态回退 404），本批按上游 wptserve handler 逐分支
1:1 补齐（pin 315976933 实读核对，含 worker 引导脚本四模板逐字节、q 正则
首现语义、500 异常臂、stash take miss 无 Content-Type 空体 200）。

附带**批十二勘误**：`route_speculative_stash` take 臂漏设 body（R3021 wire
约定：valid-UTF-8 body_bytes 走 body 文本，serialize_response 取
`body.clone()`）→ JS fetch `text()` 恒空 → compare 分支「空==空」空转，
7 键假 Pass。批十三重跑 126 执行键 **43 Pass / 83 Timeout**（A/B 两轮逐键
零漂移），speculative 通道净记账 **50→43**（-7 归批十二虚记，非批十三
损失）；7 键全 document-write 形——fix 后真实头序列比较生效，两 put 头
序列不等 → 断言抛出 → done 不达 → 30s 计时 TO（0 Fail）。批十二证据
「首 take 非空……全部满足」表述随之勘误；嵌套 write 管道头序列差为
engine 碰头输入新增一形。

## 变更（四文件 + wpt-data 资产）

1. `tests/wpt-runner/src/testharness_py_routes/misc.rs`——四端点
   （resource.py 多型资源回显/blank.py 编码空页/stash.py q 原文回读/
   css-tmpl.py 编码样式表模板）+ speculative take 臂 body 补设（勘误）。
2. `tests/wpt-runner/src/testharness_py_routes/mod.rs`——
   `PyEndpointCtx.query_encoding_stash` 字段 + 四 Exact 路由注册。
3. `tests/wpt-runner/src/testharness.rs`——query_encoding_stash 案级构建
   （per-case、案内键自清、跨案零泄漏，同 speculative_stash 先例）。
4. `tests/wpt-runner/scripts/goals/62-wptfyi-html-infra.sh`——
   green-2x2.png + media 面 5 资产显式补档（media//images/ 在 html/ 子树
   外；上游 pin 无 .ogv 档，ogv MIME 改名臂为上游死分支）。
5. wpt-data（gitignored，脚本拉取）——media/ 补 5 文件
   （green-at-15.webm 666689B、A4.mp4 53409B、movie_5.mp4 31603B、
   green-at-15.mp4 299193B、movie_300.mp4 2757913B，均与上游逐字节同
   大小核验）；green-2x2.png 已在盘（118B）。

## infra 套件全量 A/B（132 键，实测）

同 base 双 release 二进制（b13-before=HEAD@7a877f4ba、b13-after=批十三
工作树），test-guard 包裹（per-proc 4G / total 8G / 1800s）。前 session
08:12 限流中断未及全量实测（headline「-16 P 全落 messagechannel 抖动域」
为未测推导值，作废）；本 session 09:22-09:28 复跑补测：

| | Pass | Fail | Timeout | PF | 子测试 |
|---|---|---|---|---|---|
| before（HEAD） | 665 | 352 | 151 | 3 | 1171 |
| after（批十三） | 674 | 376 | 138 | 3 | 1191 |

- **净 +9 P / +24 F / -13 T**；18 变更键全落 query-encoding 面，其余
  114 键（含 messagechannel 抖动键）逐键零漂移（本对二进制零抖动）。
- before 665/352/151/PF3 与批十 after 记录逐项吻合（基线锚定）。
- 定向 85 键三轮 A/B（before 141P/214F/150T → after 150P/238F/137T）
  18 键 Δ 与全量逐键一致，互证。

面级归因（端点上线前静态 404 → poll/fetch 挂起 TO；上线后 poll 收敛、
真实断言开始执行，F 增为断言面曝光而非回归）：

| 面 | Δ | 形态 |
|---|---|---|
| xhr | +3 P | utf 页 1F→1P |
| eventsource | +3 P ＋2 TO→F | utf 页 1P1T→2P；windows 页 1F1T→2F |
| loading | +3 P，F +17 | utf 页 1P（断言面 3F→6F 扩面）；windows 页 3F→7F |
| css | 8 TO→F | poll 收敛后内容断言落 F——CSSOM 门面（余 97 TO 待 stylesheet 载入链碰头刀） |

收益兑现仍受引擎门制约（CSSOM/worker 通道/iframe/doc.write），批十挂账
的 shim 编码越界刀（非特殊 scheme 文档编码越界 ~54 F 等）继续待碰头。

## 门禁

- `cargo fmt -p zero-wpt-runner -- --check` clean（rebase 后复验）。
- scoped `cargo clippy -p zero-wpt-runner --all-targets`：runner 本域零
  警告（依赖面 zero-engine lib 15 条在渲染流域文件，同批九以来既状）。
- `cargo test -p zero-wpt-runner --bins`（test-guard --compile-first）：
  329 passed / 0 failed（rebase 后复验）。

[逐文件 JSON](2026-11-22-dc2-batch13-query-encoding-py-fixtures.json) ·
[批十二证据](2026-11-22-dc2-batch12-syntax-speculative-parsing.md) ·
[批十证据](2026-11-22-dc2-batch10-query-encoding-getsub.md)
