# M2-S11 — CORS preflight 面：缓存键 URL 维度 + 覆盖判定 spec 化 + 值条件语言头 + method 归一（2026-10-29）

**切片**: goal v2.13 小片清单——「M2-S11 CORS preflight 面立项」：preflight 缓存
污染链、ACAM/ACAH 覆盖判定偏离、语言头误预检、自定义方法大小写面。
**七文件 +40 / 0 回归**，全通道 10949 → **10989**：

| 文件 | 前 → 后 |
|---|---|
| cors-redirect-preflight | 12/30 → **30/30 收口** |
| cors-preflight-star | 24/34 → **34/34 收口** |
| cors-preflight | 19/22 → **22/22 收口** |
| cors-no-preflight | 13/15 → **15/15 收口** |
| cors-preflight-cache | 1/2 → **2/2 收口** |
| cors-preflight-response-validation | 0/2 → **2/2 收口** |
| cors-preflight-redirect | 1/10 → 5/10（余 5F 记账 1） |

## 失败面分解（45F → 5F）

| 组 | 腿数 | 根因 |
|---|---|---|
| cors-redirect-preflight 成功腿缓存污染 + cors→same-origin 缺预检 | 18 | preflight 缓存 origin 级键——同 origin 异 URL 误命中跳过预检（前腿成功写入 5s TTL 条目，后续腿 x-did-preflight 0）；重定向臂同源收尾跳无预检门（response tainting 缺位） |
| cors-preflight-star 覆盖判定 + method 大小写 | 10 | ACAM 大小写不敏感比对（PATCH/patch、DELETE/delete 须 token 精确匹配）；`*` 通配对含凭据请求放行（仅无凭据可通配）；fetch 入口 method 盲 `toUpperCase()`（非 normalize 表方法 patcH/chicken/patch 须保原串，ACRM/ACAM 逐字比对前提） |
| cors-preflight-redirect 缓存污染 + after-preflight 拒绝链 | 9（余 5） | 5F 同上缓存污染链（redirect.py OPTIONS 无 Max-Age → 5s 默认条目跨 token 污染）；「on preflight failed」5 腿修复后转绿；「after preflight failed」5 腿独立缺口（记账 1） |
| cors-preflight 语言头误入 ACRH | 3 | accept-language/content-language 恒入预检名单（bzh/eu 合法语值应 safelist）→ ACRH 混入 + `x-control-request-headers` 精确断言失配；ACRH 逗号后空格面（`, ` vs `,`） |
| cors-no-preflight 语言头误预检 | 2 | 同上——fr/eu 合法值触发预检（期望免预检直发） |
| cors-preflight-cache 跨 URL 误命中 | 1 | origin 级键使 wildcard Authorization 测试复用前测试条目（URL 键后逐 token 独立） |
| cors-preflight-response-validation 畸形值 | 2 | ACAM/ACAH 值无 token 语法校验（"Bad value" 含空格须按 extract failure 转 network error，此前放行） |

## 实现面（`part01.js` 单文件，8 项）

1. **preflight 缓存键补 URL 维度**（入口 hit/store + hop hit/store 四处）——键
   `(origin, URL, credentials)` 对齐 fetch spec §cors-preflight-cache 条目定义；
   origin 级键是 redirect-preflight 污染链与 cache wildcard 面的共同根因。
2. **settlePre ACAM/ACAH 覆盖判定 spec 化**（§cors-preflight-fetch）——ACAM/ACAH
   值先经 `#method`/`#field-name` 逐项 token 语法校验（非 tchar → failure →
   network error）；ACAM 非 safelisted method 逐项**大小写敏感**匹配、`*` 仅无凭据
   通配、缺 ACAM 拒；ACAH 统一面：逐项大小写不敏感匹配、authorization 恒须显式
   列出（`*` 永不覆盖）、其余名 `*` 仅无凭据通配（含凭据时仅字面同名头可匹配——
   star `["*", "1"]` 面角标）。
3. **缓存 method 存取去 lowercase**——method token 精确匹配（ACAM `PATCH` 后
   `patch` 须重预检）。
4. **accept-language/content-language 值条件 safelist**（`_zwPreNamesOf`）——值
   仅含 `0-9 A-Z a-z SP * , - . ; =` 字节集且长度 ≤128 免预检，破格值（`\x01`、`@`）
   仍入列；对齐 fetch spec §cors-safelisted-request-header 现行版（旧「2024 移出
   安全名单恒入列」注记过时——现行 spec 已回归值条件面）。
5. **hop 重预检门补 `_zwRespTaintingCors`**——跨源 cors 链 response tainting 恒
   "cors" 不回退（M2-S3b 已按入口跨源初始化、跨跳处置位置位），同源收尾跳仍走
   cors 分支预检（主 fetch 递归语义；cors-redirect-preflight「cors to same
   origin」五码全绿的前提）。
6. **两臂缓存仅覆盖检查成功后写入**——hop 臂 store 移到 `!rp.ok` reject 之后，
   对齐主路径与 spec store 步位置（失败预检不再落缓存）。
7. **fetch 入口 method 归一走 `_zwNormalizeMethod`**（§concept-method-normalize）
   ——只大写 GET/HEAD/POST/PUT/DELETE/OPTIONS/TRACE/CONNECT 八固定 token，
   patch/patcH/chicken/SUPER 等保原串；旧盲 `toUpperCase()` 使 ACRM 回显与 ACAM
   逐字比对全翻红（star patch 族 + cors-preflight patcH/chicken 面）。
8. **ACRH 排序后 `,` 无空格连接**（入口 + hop 两处）——preflight stash
   `x-control-request-headers` 精确断言 `content-type,x-test-header-a,...` 形态。

## 验证

- 构建 0 warning（`/tmp/m2s11-build-r2.log`）；probe r1 过程轮（method 归一前）
  star patch B/D 腿、cors-preflight patcH/chicken 翻红定界 → 修复 7 清偿（r2 全绿）。
- 全通道 A/B（`netapi-m2s11-r2.json` vs **直接前轮** `netapi-m2s10-r2.json`，
  逐 subtest）：**+40 增益 / 0 回归 / MISSING 0**（10949 → 10989）；per-file Δ 仅
  七目标文件，其余全通道逐文件零变化。
- 回归观察名单零扰动：cors-preflight-status 27/27、not-cors-safelisted 12/12、
  xhr access-control-preflight-cache 族全绿保持。

## 记账（未清偿）

1. cors-preflight-redirect「after preflight failed」×5——hop 臂对裸 preflight.py
   （无 `allow_headers`）的预检期望拒绝（ACAH 缺失 → network error），r2 实测
   resolve；链路含 entry 预检 200 后 30x 重定向 + 同 origin 跳，待专项定界。
2. cors-preflight-referrer 6F（基线既有，非本轮回归）：preflight Referer
   origin-only 形态带尾斜杠（期望 `http://wpt.test:8000`，实发
   `http://wpt.test:8000/`）——`_zwRefOut` origin 序列化面，独立小片。
